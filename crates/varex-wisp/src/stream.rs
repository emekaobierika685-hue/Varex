// crates/varex-wisp/src/stream.rs
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

pub struct Credits {
    remaining: AtomicU32,
    window: u32,
    notify: Notify,
}

impl Credits {
    pub fn new(window: u32) -> Arc<Self> {
        Arc::new(Self { remaining: AtomicU32::new(window), window, notify: Notify::new() })
    }
    pub fn consume(&self) {
        let prev = self.remaining.fetch_sub(1, Ordering::AcqRel);
        if prev == 0 { self.remaining.store(0, Ordering::Release); }
    }
    pub fn replenish(&self, n: u32) {
        let cur = self.remaining.load(Ordering::Acquire);
        self.remaining.store(cur.saturating_add(n).min(self.window), Ordering::Release);
        self.notify.notify_waiters();
    }
    pub async fn acquire(&self) {
        loop {
            let cur = self.remaining.load(Ordering::Acquire);
            if cur > 0 {
                if self.remaining.compare_exchange(cur, cur - 1, Ordering::AcqRel, Ordering::Acquire).is_ok() { return; }
                continue;
            }
            self.notify.notified().await;
        }
    }
}
