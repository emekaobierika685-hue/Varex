// crates/varex-wisp/src/tcp.rs
use std::sync::Arc;
use std::time::{Duration, Instant};
use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::debug;
use varex_core::{metrics_defs, VarexError};
use crate::ping::OutFrame;
use crate::protocol::STREAM_TCP;
use crate::server::{AppState, StreamHandle};
use crate::stream::Credits;

pub async fn spawn_tcp_stream(
    stream_id: u32, stream_type: u8, host: String, port: u16,
    state: &Arc<AppState>, out_tx: mpsc::Sender<OutFrame>,
) -> varex_core::Result<StreamHandle> {
    debug_assert_eq!(stream_type, STREAM_TCP);
    state.ssrf.check_port(port)?;
    state.ssrf.check_hostname(&host)?;
    let addrs = state.dns.resolve(&host, port).await?;
    state.ssrf.check_resolved(&addrs)?;
    let target = addrs.first().copied().ok_or_else(|| VarexError::Dns("no addrs".into()))?;

    let connect_timeout = Duration::from_secs(state.config.wisp.connect_timeout_secs);
    let started = Instant::now();
    let socket = tokio::time::timeout(connect_timeout, TcpStream::connect((target, port)))
        .await.map_err(|_| VarexError::ConnectTimeout)?.map_err(VarexError::Io)?;
    metrics_defs::connect_latency(started.elapsed());

    let credits = Credits::new(state.config.wisp.credit_window);
    let (to_upstream_tx, mut to_upstream_rx) = mpsc::channel::<Bytes>(state.config.wisp.stream_queue_capacity);
    let (close_tx, close_rx) = mpsc::channel::<(u32, u8)>(1);
    let cfg = state.config.clone();
    let out_tx_up = out_tx.clone();
    let credits_up = credits.clone();
    tokio::spawn(async move {
        if let Err(e) = run_tcp(stream_id, socket, &mut to_upstream_rx, out_tx_up, credits_up, &cfg).await {
            debug!(stream_id, error = %e, "tcp error");
        }
        let _ = close_tx.send((stream_id, 0x01)).await;
    });
    Ok(StreamHandle { to_upstream: to_upstream_tx, credits, close_rx })
}

async fn run_tcp(
    stream_id: u32, socket: TcpStream, to_upstream_rx: &mut mpsc::Receiver<Bytes>,
    out_tx: mpsc::Sender<OutFrame>, credits: Arc<Credits>, cfg: &varex_core::Config,
) -> varex_core::Result<()> {
    let (mut read_half, mut write_half) = socket.into_split();
    let read_chunk = cfg.wisp.tcp_read_chunk;
    let max_bytes = cfg.wisp.max_bytes_per_stream;
    let idle = Duration::from_secs(cfg.wisp.idle_timeout_secs);

    let writer = tokio::spawn(async move {
        let mut total: u64 = 0;
        while let Some(chunk) = to_upstream_rx.recv().await {
            credits.acquire().await;
            if max_bytes > 0 && total + chunk.len() as u64 > max_bytes { return Err(VarexError::ByteBudget); }
            if write_half.write_all(&chunk).await.is_err() {
                return Err(VarexError::Io(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "write failed")));
            }
            total += chunk.len() as u64;
            metrics_defs::bytes_in(chunk.len() as u64);
        }
        let _ = write_half.shutdown().await;
        Ok::<_, VarexError>(())
    });

    let reader = async move {
        let mut buf = vec![0u8; read_chunk];
        let mut frame = bytes::BytesMut::with_capacity(read_chunk + 5);
        let mut total: u64 = 0;
        loop {
            let n = match tokio::time::timeout(idle, read_half.read(&mut buf)).await {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => n,
                Ok(Err(e)) => return Err(VarexError::Io(e)),
                Err(_) => return Err(VarexError::IdleTimeout),
            };
            if max_bytes > 0 && total + n as u64 > max_bytes { return Err(VarexError::ByteBudget); }
            frame.clear();
            crate::protocol::Encoder::data(&mut frame, stream_id, &buf[..n]);
            if out_tx.send(OutFrame::Data(frame.split())).await.is_err() { break; }
            metrics_defs::bytes_out(n as u64);
            total += n as u64;
        }
        Ok::<(), VarexError>(())
    };

    let (r, w) = tokio::join!(reader, writer);
    r.and(w.map_err(|e| e))?;
    Ok(())
}
