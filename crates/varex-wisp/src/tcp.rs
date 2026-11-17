// crates/varex-wisp/src/tcp.rs
//!
//! Phase 1a: flow control is now actually enforced and advertised.
//!
//! Wisp v1.2 §0x03 CONTINUE: after the initial CONTINUE on stream 0, the
//! server is expected to send CONTINUE packets as its internal write
//! queue drains, telling the client how much room is available. Without
//! that, a well-behaved client stalls after `window` DATA packets and
//! never resumes.
//!
//! Design:
//!   - `to_upstream` is an mpsc with capacity = `credit_window`.
//!   - The WebSocket reader uses `try_send`. If the queue is full the
//!     client has ignored its credit; that is a protocol violation, and
//!     we close the stream with REASON_REFUSED.
//!   - The writer task, after every successful write to upstream,
//!     computes `available = window.saturating_sub(queue_len)` and emits
//!     CONTINUE(stream_id, available). This preserves the invariant
//!     `client_credit + queue_len = window` at all times.
//!   - Stream termination is signalled via a single `close_tx` mpsc that
//!     the main connection loop drains. No per-iteration future rebuild.
use std::time::{Duration, Instant};

use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::debug;

use varex_core::{metrics_defs, VarexError};

use crate::ping::OutFrame;
use crate::protocol::{Encoder, HEADER_LEN, STREAM_TCP};
use crate::server::{AppState, StreamHandle};
use std::sync::Arc;

pub async fn spawn_tcp_stream(
    stream_id: u32,
    stream_type: u8,
    host: String,
    port: u16,
    state: &Arc<AppState>,
    out_tx: mpsc::Sender<OutFrame>,
    close_tx: mpsc::Sender<(u32, u8)>,
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
        .await
        .map_err(|_| VarexError::ConnectTimeout)?
        .map_err(VarexError::Io)?;
    metrics_defs::connect_latency(started.elapsed());

    // Queue capacity == credit window. The invariant
    //   client_credit + queue_len == window
    // is what makes `try_send` a valid credit-violation check.
    let window = state.config.wisp.credit_window;
    let (to_upstream_tx, to_upstream_rx) = mpsc::channel::<Bytes>(window as usize);

    let cfg = state.config.clone();
    let out_tx_up = out_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = run_tcp(stream_id, socket, to_upstream_rx, out_tx_up, window, &cfg).await {
            debug!(stream_id, error = %e, "tcp stream ended");
        }
        // Single close event; the main loop consumes this.
        let _ = close_tx.send((stream_id, 0x01)).await;
    });

    Ok(StreamHandle { to_upstream: to_upstream_tx })
}

async fn run_tcp(
    stream_id: u32,
    socket: TcpStream,
    mut to_upstream_rx: mpsc::Receiver<Bytes>,
    out_tx: mpsc::Sender<OutFrame>,
    window: u32,
    cfg: &varex_core::Config,
) -> varex_core::Result<()> {
    let (mut read_half, mut write_half) = socket.into_split();
    let read_chunk = cfg.wisp.tcp_read_chunk;
    let max_bytes = cfg.wisp.max_bytes_per_stream;
    let idle = Duration::from_secs(cfg.wisp.idle_timeout_secs);
    let continue_buf_capacity = HEADER_LEN + 4;

    // Writer task: drain client->upstream queue, emit CONTINUE on drain.
    let writer = async move {
        let mut total: u64 = 0;
        let mut cont_buf = BytesMut::with_capacity(continue_buf_capacity);

        while let Some(chunk) = to_upstream_rx.recv().await {
            if max_bytes > 0 && total + chunk.len() as u64 > max_bytes {
                return Err(VarexError::ByteBudget);
            }
            if write_half.write_all(&chunk).await.is_err() {
                return Err(VarexError::Io(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "upstream write failed",
                )));
            }
            total += chunk.len() as u64;
            metrics_defs::bytes_in(chunk.len() as u64);

            // The queue just shrank by one. Tell the client how many
            // free slots remain. This is the fix for the stall bug.
            let remaining = to_upstream_rx.len() as u32;
            let available = window.saturating_sub(remaining);
            cont_buf.clear();
            Encoder::continue_(&mut cont_buf, stream_id, available);
            if out_tx.send(OutFrame::Data(cont_buf.split())).await.is_err() {
                return Err(VarexError::Io(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "websocket writer died",
                )));
            }
        }
        let _ = write_half.shutdown().await;
        Ok::<_, VarexError>(())
    };

    // Reader task: upstream -> WebSocket DATA.
    let reader = async move {
        let mut buf = vec![0u8; read_chunk];
        let mut frame = BytesMut::with_capacity(read_chunk + HEADER_LEN);
        let mut total: u64 = 0;
        loop {
            let n = match tokio::time::timeout(idle, read_half.read(&mut buf)).await {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => n,
                Ok(Err(e)) => return Err(VarexError::Io(e)),
                Err(_) => return Err(VarexError::IdleTimeout),
            };
            if max_bytes > 0 && total + n as u64 > max_bytes {
                return Err(VarexError::ByteBudget);
            }
            frame.clear();
            Encoder::data(&mut frame, stream_id, &buf[..n]);
            if out_tx.send(OutFrame::Data(frame.split())).await.is_err() {
                break;
            }
            metrics_defs::bytes_out(n as u64);
            total += n as u64;
        }
        Ok::<(), VarexError>(())
    };

    let (r, w) = tokio::join!(reader, writer);
    r.and(w.map_err(|e| e))?;
    Ok(())
}
