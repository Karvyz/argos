use std::{
    sync::atomic::{AtomicI64, Ordering},
    time::Instant,
};

const MUTE_TAIL_NS: i64 = 100_000_000;

pub struct SpeakerGate {
    origin: Instant,
    active_until_ns: AtomicI64,
}

impl SpeakerGate {
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
            active_until_ns: AtomicI64::new(-1),
        }
    }

    fn now_ns(&self) -> i64 {
        Instant::now().duration_since(self.origin).as_nanos() as i64
    }

    pub fn report_playing(&self) {
        self.active_until_ns
            .store(self.now_ns() + MUTE_TAIL_NS, Ordering::Relaxed);
    }

    pub fn is_muted(&self) -> bool {
        self.now_ns() < self.active_until_ns.load(Ordering::Relaxed)
    }
}

