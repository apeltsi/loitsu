use std::sync::atomic::Ordering;

static ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn get_unique_id() -> u32 {
    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}
