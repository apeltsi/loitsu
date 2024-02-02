use std::sync::atomic::Ordering;

static ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Get a valid unique id for an entity, component, drawable or other managed object
pub fn get_unique_id() -> u32 {
    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Reserve a certain number of ids for use, returning the first id in the reserved range
pub fn reserve_id_space(amount: u32) -> u32 {
    ID_COUNTER.fetch_add(amount, Ordering::Relaxed)
}
