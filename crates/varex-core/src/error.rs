// crates/varex-core/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VarexError {
    #[error("protocol violation: {0}")]
    Protocol(String),
    #[error("invalid connect payload: {0}")]
    BadConnect(String),
    #[error("host denied by policy: {0}")]
    HostDenied(String),
    #[error("port denied by policy: {port}")]
    PortDenied { port: u16 },
    #[error("resolved address blocked: {addr}")]
    AddressBlocked { addr: String },
    #[error("dns resolution failed: {0}")]
    Dns(String),
    #[error("rate limited")]
    RateLimited,
    #[error("connection limit reached")]
    ConnectionLimit,
    #[error("stream limit reached")]
    StreamLimit,
    #[error("connect timeout")]
    ConnectTimeout,
    #[error("idle timeout")]
    IdleTimeout,
    #[error("byte budget exhausted")]
    ByteBudget,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("config invalid: {0}")]
    Config(String),
    #[error("plugin failure: {0}")]
    Plugin(String),
    #[error("rewriter failure: {0}")]
    Rewriter(String),
    #[error("transport failure: {0}")]
    Transport(String),
}

impl VarexError {
    pub fn close_reason(&self) -> u8 {
        match self {
            Self::ConnectTimeout | Self::IdleTimeout => 0x02,
            Self::HostDenied(_)
            | Self::PortDenied { .. }
            | Self::AddressBlocked { .. }
            | Self::RateLimited
            | Self::ConnectionLimit
            | Self::StreamLimit
            | Self::ByteBudget => 0x03,
            Self::Io(_) | Self::Dns(_) | Self::Transport(_) => 0x04,
            _ => 0x01,
        }
    }
}

pub type Result<T> = std::result::Result<T, VarexError>;
