// crates/varex-wisp/src/server.rs
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, ConnectInfo, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info};

use varex_core::{dnscache::DnsCache, metrics_defs, ratelimit::RateLimiter, ssrf::SsrfPolicy, Config, Result, VarexError};

use crate::ping::OutFrame;
use crate::protocol::{Encoder, Packet, HEADER_LEN, REASON_NETWORK, REASON_REFUSED, REASON_TIMEOUT, STREAM_TCP, STREAM_UDP};
use crate::stream::Credits;

pub struct AppState {
    pub config: Config,
    pub ssrf: SsrfPolicy,
    pub dns: DnsCache,
    pub rate: Mutex<RateLimiter>,
    pub active_connections: AtomicUsize,
    pub per_ip_streams: Mutex<HashMap<IpAddr, usize>>,
}

impl AppState {
    pub fn new(config: Config) -> Result<Self> {
        let ssrf = SsrfPolicy::new(&config.security)?;
        let dns = DnsCache::new(config.security.dns_cache_ttl_secs, config.security.dns_cache_capacity);
        let rate = RateLimiter::new(config.security.per_ip_rate_per_sec, config.security.per_ip_burst);
        Ok(Self { config, ssrf, dns, rate: Mutex::new(rate),
            active_connections: AtomicUsize::new(0), per_ip_streams: Mutex::new(HashMap::new()) })
    }
}

pub fn router(state: Arc<AppState>) -> Router {
    let wisp_path = state.config.server.wisp_path.clone();
    Router::new()
        .route(&wisp_path, get(ws_upgrade))
        .route(&state.config.observability.health_path, get(health))
        .route(&state.config.observability.ready_path, get(ready))
        .with_state(state)
}

async fn health() -> impl IntoResponse { (StatusCode::OK, "ok") }

async fn ready(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    if st.active_connections.load(Ordering::Acquire) >= st.config.wisp.max_connections {
        (StatusCode::SERVICE_UNAVAILABLE, "at capacity")
    } else { (StatusCode::OK, "ready") }
}

async fn ws_upgrade(ws: WebSocketUpgrade, State(st): State<Arc<AppState>>, ConnectInfo(peer): ConnectInfo<SocketAddr>) -> Response {
    let ip = peer.ip();
    {
        let mut limiter = st.rate.lock().await;
        limiter.gc();
        if !limiter.allow(ip) { metrics_defs::rate_limited(); return (StatusCode::TOO_MANY_REQUESTS, "rate limited").into_response(); }
    }
    let current = st.active_connections.fetch_add(1, Ordering::AcqRel);
    if current >= st.config.wisp.max_connections {
        st.active_connections.fetch_sub(1, Ordering::AcqRel);
        return (StatusCode::SERVICE_UNAVAILABLE, "connection limit").into_response();
    }
    metrics_defs::conn_open();
    ws.on_upgrade(move |socket| async move {
        let _guard = ConnGuard { state: st.clone() };
        if let Err(e) = run_connection(socket, st, ip).await { debug!(%ip, error = %e, "conn ended"); }
    })
}

struct ConnGuard { state: Arc<AppState> }
impl Drop for ConnGuard {
    fn drop(&mut self) {
        self.state.active_connections.fetch_sub(1, Ordering::AcqRel);
        metrics_defs::conn_closed();
    }
}

async fn run_connection(socket: WebSocket, state: Arc<AppState>, peer_ip: IpAddr) -> Result<()> {
    info!(%peer_ip, "wisp connection opened");
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (out_tx, mut out_rx) = mpsc::channel::<OutFrame>(1024);

    let writer = tokio::spawn(async move {
        while let Some(first) = out_rx.recv().await {
            let mut batch: Vec<OutFrame> = Vec::with_capacity(16);
            batch.push(first);
            while let Ok(more) = out_rx.try_recv() {
                batch.push(more);
                if batch.len() == 32 { break; }
            }
            for frame in batch {
                if ws_sender.feed(frame.into_message()).await.is_err() { return; }
            }
            if ws_sender.flush().await.is_err() { return; }
        }
    });

    // §0x03 CONTINUE lines 17–18: initial CONTINUE on stream 0.
    {
        let mut initial = BytesMut::with_capacity(HEADER_LEN + 4);
        Encoder::continue_(&mut initial, 0, state.config.wisp.credit_window);
        if out_tx.send(OutFrame::Data(initial)).await.is_err() {
            writer.abort();
            return Err(VarexError::Protocol("writer died".into()));
        }
    }

    let mut streams: HashMap<u32, StreamHandle> = HashMap::new();
    let mut ping_interval = tokio::time::interval(Duration::from_secs(state.config.wisp.ws_ping_interval_secs.max(1)));
    ping_interval.tick().await;

    loop {
        tokio::select! {
            biased;
            _ = ping_interval.tick() => {
                if out_tx.send(OutFrame::Ping(Bytes::new())).await.is_err() { break; }
            }
            Some((id, reason)) = recv_any_close(&mut streams) => {
                streams.remove(&id);
                let mut buf = BytesMut::with_capacity(HEADER_LEN + 1);
                Encoder::close(&mut buf, id, reason);
                if out_tx.send(OutFrame::Data(buf)).await.is_err() { break; }
                let mut guard = state.per_ip_streams.lock().await;
                if let Some(n) = guard.get_mut(&peer_ip) { *n = n.saturating_sub(1); }
                metrics_defs::stream_closed();
            }
            incoming = ws_receiver.next() => {
                let Some(Ok(msg)) = incoming else { break };
                match msg {
                    Message::Binary(raw) => {
                        let pkt = match Packet::parse(Bytes::from(raw)) {
                            Ok(p) => p, Err(e) => { debug!(%peer_ip, error=%e, "bad packet"); continue; }
                        };
                        match pkt {
                            Packet::Connect { stream_id, stream_type, port, host } => {
                                if streams.contains_key(&stream_id) || streams.len() >= state.config.wisp.max_streams_per_connection {
                                    let mut buf = BytesMut::with_capacity(HEADER_LEN + 1);
                                    Encoder::close(&mut buf, stream_id, REASON_REFUSED);
                                    let _ = out_tx.send(OutFrame::Data(buf)).await; continue;
                                }
                                {
                                    let mut g = state.per_ip_streams.lock().await;
                                    let n = g.entry(peer_ip).or_insert(0);
                                    if *n >= state.config.security.max_streams_per_ip {
                                        let mut buf = BytesMut::with_capacity(HEADER_LEN + 1);
                                        Encoder::close(&mut buf, stream_id, REASON_REFUSED);
                                        let _ = out_tx.send(OutFrame::Data(buf)).await; continue;
                                    }
                                    *n += 1;
                                }
                                let handle = match stream_type {
                                    STREAM_TCP => crate::tcp::spawn_tcp_stream(stream_id, stream_type, host, port, &state, out_tx.clone()).await,
                                    STREAM_UDP => crate::udp::spawn_udp_stream(stream_id, stream_type, host, port, &state, out_tx.clone()).await,
                                    other => Err(VarexError::BadConnect(format!("stream_type 0x{other:02x}"))),
                                };
                                match handle {
                                    Ok(h) => { streams.insert(stream_id, h); metrics_defs::stream_open(); }
                                    Err(e) => {
                                        let mut buf = BytesMut::with_capacity(HEADER_LEN + 1);
                                        Encoder::close(&mut buf, stream_id, e.close_reason());
                                        let _ = out_tx.send(OutFrame::Data(buf)).await;
                                        metrics_defs::ssrf_blocked();
                                    }
                                }
                            }
                            Packet::Data { stream_id, payload } => {
                                if let Some(h) = streams.get(&stream_id) {
                                    h.credits.consume();
                                    let _ = h.to_upstream.send(payload).await;
                                }
                            }
                            Packet::Continue { stream_id, buffer_remaining } => {
                                if let Some(h) = streams.get(&stream_id) { h.credits.replenish(buffer_remaining); }
                            }
                            Packet::Close { stream_id, reason } => {
                                debug!(stream_id, reason, "client closed");
                                streams.remove(&stream_id);
                                let mut g = state.per_ip_streams.lock().await;
                                if let Some(n) = g.get_mut(&peer_ip) { *n = n.saturating_sub(1); }
                                metrics_defs::stream_closed();
                            }
                            Packet::Info { .. } => {}
                        }
                    }
                    Message::Ping(_) | Message::Pong(_) => {}
                    Message::Close(_) => break,
                    Message::Text(_) => {}
                }
            }
        }
    }

    for (id, _) in streams.drain() {
        let mut buf = BytesMut::with_capacity(HEADER_LEN + 1);
        Encoder::close(&mut buf, id, 0x01);
        let _ = out_tx.send(OutFrame::Data(buf)).await;
    }
    drop(out_tx);
    let _ = writer.await;
    info!(%peer_ip, "wisp connection closed");
    Ok(())
}

pub struct StreamHandle {
    pub to_upstream: mpsc::Sender<Bytes>,
    pub credits: Arc<Credits>,
    pub close_rx: mpsc::Receiver<(u32, u8)>,
}

async fn recv_any_close(streams: &mut HashMap<u32, StreamHandle>) -> Option<(u32, u8)> {
    if streams.is_empty() { std::future::pending::<()>().await; return None; }
    let mut futs: Vec<_> = streams.iter_mut()
        .map(|(id, h)| { let id = *id; async move { h.close_rx.recv().await.map(|(_, r)| (id, r)) } })
        .collect();
    futures::future::select_all(futs).await.ok().map(|(v, _, _)| v)
}

pub fn close_msg(stream_id: u32, err: &VarexError) -> BytesMut {
    let reason = match err {
        VarexError::ConnectTimeout | VarexError::IdleTimeout => REASON_TIMEOUT,
        VarexError::Io(_) | VarexError::Dns(_) => REASON_NETWORK,
        _ => REASON_REFUSED,
    };
    let mut buf = BytesMut::with_capacity(HEADER_LEN + 1);
    Encoder::close(&mut buf, stream_id, reason);
    metrics_defs::close(reason);
    buf
}
