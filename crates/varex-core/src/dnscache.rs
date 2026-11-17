// crates/varex-core/src/dnscache.rs
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use crate::error::{Result, VarexError};

struct Entry { addrs: Vec<IpAddr>, expires: Instant }

pub struct DnsCache {
    inner: Mutex<HashMap<String, Entry>>,
    ttl: Duration,
    capacity: usize,
}

impl DnsCache {
    pub fn new(ttl_secs: u64, capacity: u64) -> Self {
        Self { inner: Mutex::new(HashMap::new()), ttl: Duration::from_secs(ttl_secs), capacity: capacity as usize }
    }
    pub async fn resolve(&self, host: &str, port: u16) -> Result<Vec<IpAddr>> {
        let key = format!("{host}:{port}");
        {
            let mut map = self.inner.lock().await;
            if let Some(e) = map.get(&key) {
                if e.expires > Instant::now() { return Ok(e.addrs.clone()); }
                map.remove(&key);
            }
        }
        let addrs: Vec<IpAddr> = tokio::net::lookup_host((host, port)).await
            .map_err(|e| VarexError::Dns(format!("{host}: {e}")))?
            .map(|s| s.ip()).collect();
        if addrs.is_empty() { return Err(VarexError::Dns(format!("{host}: no addresses"))); }
        let mut map = self.inner.lock().await;
        if map.len() >= self.capacity { map.clear(); }
        map.insert(key, Entry { addrs: addrs.clone(), expires: Instant::now() + self.ttl });
        Ok(addrs)
    }
}
