// crates/varex-core/src/ratelimit.rs
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Instant;

struct Bucket { tokens: f64, last: Instant }

pub struct RateLimiter {
    buckets: HashMap<IpAddr, Bucket>,
    rate: f64,
    burst: f64,
    last_gc: Instant,
}

impl RateLimiter {
    pub fn new(rate_per_sec: f64, burst: u32) -> Self {
        Self { buckets: HashMap::new(), rate: rate_per_sec, burst: burst as f64, last_gc: Instant::now() }
    }
    pub fn allow(&mut self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let b = self.buckets.entry(ip).or_insert(Bucket { tokens: self.burst, last: now });
        let elapsed = now.duration_since(b.last).as_secs_f64();
        b.tokens = (b.tokens + elapsed * self.rate).min(self.burst);
        b.last = now;
        if b.tokens >= 1.0 { b.tokens -= 1.0; true } else { false }
    }
    pub fn gc(&mut self) {
        if self.last_gc.elapsed().as_secs() < 60 { return; }
        self.last_gc = Instant::now();
        let burst = self.burst; let now = Instant::now();
        self.buckets.retain(|_, b| (b.tokens + now.duration_since(b.last).as_secs_f64() * self.rate) < burst);
    }
}
