use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Mutex, OnceLock};
static NODE: AtomicI64 = AtomicI64::new(1);
pub(crate) fn configure_node(node: u16) {
    NODE.store(i64::from(node), Ordering::Relaxed);
}

// Snowflake: milliseconds from 2024-01-01, 10-bit node, 12-bit sequence.
// State is monotonic across clock rollback; node IDs must differ across writers.
pub fn new_id() -> i64 {
    static STATE: OnceLock<Mutex<(i64, i64)>> = OnceLock::new();
    let node = NODE.load(Ordering::Relaxed);
    let mut state = STATE
        .get_or_init(|| Mutex::new((0, 0)))
        .lock()
        .expect("ID generator mutex");
    let time = (chrono::Utc::now().timestamp_millis() - 1_704_067_200_000).max(state.0);
    let mut sequence = if time == state.0 { state.1 + 1 } else { 0 };
    let time = if sequence >= 4096 {
        sequence = 0;
        time + 1
    } else {
        time
    };
    *state = (time, sequence);
    (time << 22) | (node << 12) | sequence
}
pub fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests;
