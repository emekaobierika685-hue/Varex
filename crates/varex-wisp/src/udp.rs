// crates/varex-wisp/src/udp.rs
use std::sync::Arc;
use bytes::Bytes;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use varex_core::{metrics_defs, VarexError};
use crate::ping::OutFrame;
use crate::protocol::STREAM_UDP;
use crate::server::{AppState, StreamHandle};
use crate::stream::Credits;

pub async fn spawn_udp_stream(
    stream_id: u32, stream_type: u8, host: String, port: u16,
    state: &Arc<AppState>, out_tx: mpsc::Sender<OutFrame>,
) -> varex_core::Result<StreamHandle> {
    debug_assert_eq!(stream_type, STREAM_UDP);
    state.ssrf.check_port(port)?;
    state.ssrf.check_hostname(&host)?;
    let addrs = state.dns.resolve(&host, port).await?;
    state.ssrf.check_resolved(&addrs)?;
    let target = addrs.first().copied().ok_or_else(|| VarexError::Dns("no addrs".into()))?;

    let bind_addr = if target.is_ipv4() { "0.0.0.0:0" } else { "[::]:0" };
    let socket = UdpSocket::bind(bind_addr).await.map_err(VarexError::Io)?;
    socket.connect((target, port)).await.map_err(VarexError::Io)?;

    let credits = Credits::new(u32::MAX);
    let (to_upstream_tx, mut to_upstream_rx) = mpsc::channel::<Bytes>(state.config.wisp.stream_queue_capacity);
    let (close_tx, close_rx) = mpsc::channel::<(u32, u8)>(1);
    let cfg = state.config.clone();
    let socket = Arc::new(socket);

    let reader_sock = socket.clone();
    let reader_out = out_tx.clone();
    let reader_id = stream_id;
    let reader = tokio::spawn(async move {
        let mut buf = vec![0u8; 65535];
        loop {
            match reader_sock.recv(&mut buf).await {
                Ok(n) => {
                    let mut frame = bytes::BytesMut::with_capacity(n + 5);
                    crate::protocol::Encoder::data(&mut frame, reader_id, &buf[..n]);
                    if reader_out.send(OutFrame::Data(frame)).await.is_err() { break; }
                    metrics_defs::bytes_out(n as u64);
                }
                Err(_) => break,
            }
        }
    });

    tokio::spawn(async move {
        let mut total: u64 = 0;
        while let Some(chunk) = to_upstream_rx.recv().await {
            if cfg.wisp.max_bytes_per_stream > 0
                && total + chunk.len() as u64 > cfg.wisp.max_bytes_per_stream { break; }
            if socket.send(&chunk).await.is_err() { break; }
            total += chunk.len() as u64;
            metrics_defs::bytes_in(chunk.len() as u64);
        }
        reader.abort();
        let _ = close_tx.send((stream_id, 0x01)).await;
    });

    Ok(StreamHandle { to_upstream: to_upstream_tx, credits, close_rx })
}
