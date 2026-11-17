// crates/varex-plugin-host/src/example_blocklist.rs
use async_trait::async_trait;
use varex_core::plugin::{ConnectCtx, Plugin};
use varex_core::{Result, VarexError};

pub struct BlocklistPlugin { pub blocked: Vec<String> }

#[async_trait]
impl Plugin for BlocklistPlugin {
    fn name(&self) -> &'static str { "example-blocklist" }
    fn priority(&self) -> i32 { 100 }
    async fn on_connect(&self, c: &ConnectCtx<'_>) -> Result<()> {
        if self.blocked.iter().any(|h| c.host == h) { Err(VarexError::HostDenied(c.host.into())) } else { Ok(()) }
    }
}
