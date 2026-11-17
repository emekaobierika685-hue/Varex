// crates/varex-wisp/src/ping.rs
use axum::extract::ws::Message;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum OutFrame {
    Data(bytes::BytesMut),
    Ping(bytes::Bytes),
    Close,
}

impl OutFrame {
    pub fn into_message(self) -> Message {
        match self {
            OutFrame::Data(b) => Message::Binary(b.freeze()),
            OutFrame::Ping(p) => Message::Ping(p),
            OutFrame::Close => Message::Close(None),
        }
    }
}

pub type OutTx = mpsc::Sender<OutFrame>;
pub type OutRx = mpsc::Receiver<OutFrame>;
