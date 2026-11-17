// crates/varex-core/src/config.rs
use serde::{Deserialize, Serialize};
use crate::error::{Result, VarexError};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)] pub server: ServerConfig,
    #[serde(default)] pub wisp: WispConfig,
    #[serde(default)] pub security: SecurityConfig,
    #[serde(default)] pub transport: TransportConfig,
    #[serde(default)] pub observability: ObservabilityConfig,
    #[serde(default)] pub http_relay: HttpRelayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub bind: String,
    pub public_url: String,
    pub wisp_path: String,
    pub relay_path: String,
}
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0:4000".into(),
            public_url: "http://localhost:4000".into(),
            wisp_path: "/wisp/".into(),
            relay_path: "/relay".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WispConfig {
    pub max_connections: usize,
    pub max_streams_per_connection: usize,
    pub credit_window: u32,
    pub stream_queue_capacity: usize,
    pub tcp_read_chunk: usize,
    pub idle_timeout_secs: u64,
    pub connect_timeout_secs: u64,
    pub ws_ping_interval_secs: u64,
    pub max_bytes_per_stream: u64,
    pub auth_password: String,
}
impl Default for WispConfig {
    fn default() -> Self {
        Self {
            max_connections: 1024,
            max_streams_per_connection: 256,
            credit_window: 127,
            stream_queue_capacity: 64,
            tcp_read_chunk: 32 * 1024,
            idle_timeout_secs: 120,
            connect_timeout_secs: 10,
            ws_ping_interval_secs: 25,
            max_bytes_per_stream: 0,
            auth_password: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityConfig {
    pub dns_cache_ttl_secs: u64,
    pub dns_cache_capacity: u64,
    #[serde(default)] pub allowed_ports: Vec<u16>,
    #[serde(default)] pub allowed_hosts: Vec<String>,
    #[serde(default)] pub denied_hosts: Vec<String>,
    pub per_ip_rate_per_sec: f64,
    pub per_ip_burst: u32,
    pub max_streams_per_ip: usize,
}
impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            dns_cache_ttl_secs: 30,
            dns_cache_capacity: 1024,
            allowed_ports: vec![80, 443, 8080, 8443, 3000, 5000, 8000],
            allowed_hosts: vec![],
            denied_hosts: vec![
                "*.internal".into(), "*.local".into(), "localhost".into(),
                "*.metadata.google.internal".into(),
            ],
            per_ip_rate_per_sec: 20.0,
            per_ip_burst: 60,
            max_streams_per_ip: 512,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransportConfig {
    pub user_agent: String,
    pub tls_impersonation: bool,
}
impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                         AppleWebKit/537.36 (KHTML, like Gecko) \
                         Chrome/131.0.0.0 Safari/537.36".into(),
            tls_impersonation: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityConfig {
    pub metrics_path: String,
    pub health_path: String,
    pub ready_path: String,
    pub json_logs: bool,
}
impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics_path: "/metrics".into(),
            health_path: "/healthz".into(),
            ready_path: "/readyz".into(),
            json_logs: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpRelayConfig {
    pub enabled: bool,
    pub impersonate: bool,
    pub max_body_bytes: u64,
}
impl Default for HttpRelayConfig {
    fn default() -> Self {
        Self { enabled: true, impersonate: true, max_body_bytes: 1 << 30 }
    }
}

impl Config {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| VarexError::Config(format!("read {}: {e}", path.display())))?;
        let cfg: Self = toml::from_str(&raw)
            .map_err(|e| VarexError::Config(format!("parse {}: {e}", path.display())))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<()> {
        if self.server.bind.parse::<std::net::SocketAddr>().is_err() {
            return Err(VarexError::Config(format!("server.bind: {}", self.server.bind)));
        }
        if !self.server.wisp_path.starts_with('/') || !self.server.relay_path.starts_with('/') {
            return Err(VarexError::Config("paths must start with '/'".into()));
        }
        if self.wisp.credit_window == 0 {
            return Err(VarexError::Config("wisp.credit_window must be > 0".into()));
        }
        if self.wisp.stream_queue_capacity == 0 || self.wisp.tcp_read_chunk == 0 {
            return Err(VarexError::Config("wisp buffers must be > 0".into()));
        }
        if self.security.per_ip_rate_per_sec <= 0.0 {
            return Err(VarexError::Config("security.per_ip_rate_per_sec must be > 0".into()));
        }
        for p in &self.security.denied_hosts {
            globset::Glob::new(p).map_err(|e| VarexError::Config(format!("deny glob {p}: {e}")))?;
        }
        Ok(())
    }
}
