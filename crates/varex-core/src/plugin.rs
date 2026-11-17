// crates/varex-core/src/plugin.rs
use std::net::IpAddr;
use async_trait::async_trait;
use crate::error::Result;

pub struct ConnectCtx<'a> { pub host: &'a str, pub port: u16, pub stream_type: u8, pub resolved: &'a [IpAddr] }
pub struct RequestCtx<'a> { pub method: &'a str, pub url: &'a str, pub headers: &'a mut Vec<(String, String)> }
pub struct ResponseCtx<'a> { pub status: u16, pub headers: &'a mut Vec<(String, String)> }

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn priority(&self) -> i32 { 0 }
    async fn on_connect(&self, _ctx: &ConnectCtx<'_>) -> Result<()> { Ok(()) }
    async fn on_request(&self, _ctx: &mut RequestCtx<'_>) -> Result<()> { Ok(()) }
    async fn on_response(&self, _ctx: &mut ResponseCtx<'_>) -> Result<()> { Ok(()) }
    async fn on_close(&self, _stream_id: u32, _reason: u8) {}
}
