// crates/varex-http-relay/src/client.rs
use std::time::Duration;
use varex_core::{Result, VarexError};

pub struct UpstreamClient { pub(crate) inner: InnerClient, user_agent: String }

#[cfg(feature = "tls-impersonate")] type InnerClient = wreq::Client;
#[cfg(all(feature = "plain", not(feature = "tls-impersonate")))] type InnerClient = reqwest::Client;

impl UpstreamClient {
    #[cfg(feature = "tls-impersonate")]
    pub fn build(user_agent: String, impersonate: bool) -> Result<Self> {
        use wreq_util::Emulation;
        let mut b = wreq::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(32)
            .pool_idle_timeout(Duration::from_secs(60))
            .redirect(wreq::redirect::Policy::none())
            .user_agent(user_agent.clone());
        if impersonate { b = b.emulation(Emulation::Chrome136); }
        let inner = b.build().map_err(|e| VarexError::Config(format!("wreq: {e}")))?;
        Ok(Self { inner, user_agent })
    }

    #[cfg(all(feature = "plain", not(feature = "tls-impersonate")))]
    pub fn build(user_agent: String, _impersonate: bool) -> Result<Self> {
        let inner = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(32)
            .pool_idle_timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(user_agent.clone())
            .build()
            .map_err(|e| VarexError::Config(format!("reqwest: {e}")))?;
        Ok(Self { inner, user_agent })
    }

    pub fn user_agent(&self) -> &str { &self.user_agent }
}
