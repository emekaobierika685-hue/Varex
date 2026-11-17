// crates/varex-wisp/src/lib.rs
#![forbid(unsafe_code)]
pub mod ping;
pub mod protocol;
pub mod server;
pub mod stream;
pub mod tcp;
pub mod udp;

pub use server::{router, AppState};
