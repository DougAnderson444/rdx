use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Core sleep implementation
pub struct WasmSleep {
    end_time: AtomicU64,
    completed: AtomicBool,
}

impl WasmSleep {
    pub fn new(duration: Duration) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            end_time: AtomicU64::new(now + duration.as_millis() as u64),
            completed: AtomicBool::new(false),
        }
    }

    // Implements the WIT ready: func() -> bool interface
    pub fn ready(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let is_ready = now >= self.end_time.load(Ordering::Relaxed);
        if is_ready {
            self.completed.store(true, Ordering::Relaxed);
        }
        is_ready
    }
}

// Poll implementation matching WIT interface
pub fn poll(pollables: &[&WasmSleep]) -> Vec<u32> {
    let mut ready_indices = Vec::new();

    for (idx, pollable) in pollables.iter().enumerate() {
        if pollable.ready() {
            ready_indices.push(idx as u32);
        }
    }

    ready_indices
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sleep_ready() {
        let sleep = WasmSleep::new(Duration::from_millis(1));
        assert!(!sleep.ready()); // Should not be ready immediately

        std::thread::sleep(Duration::from_millis(2));
        assert!(sleep.ready()); // Should be ready after waiting
    }

    #[test]
    fn test_poll_multiple() {
        let sleep1 = WasmSleep::new(Duration::from_millis(1));
        let sleep2 = WasmSleep::new(Duration::from_millis(100));

        // Nothing should be ready immediately
        assert!(poll(&[&sleep1, &sleep2]).is_empty());

        // Wait for first sleep to complete
        std::thread::sleep(Duration::from_millis(2));
        assert_eq!(poll(&[&sleep1, &sleep2]), vec![0]);

        // Wait for second sleep to complete
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(poll(&[&sleep1, &sleep2]), vec![0, 1]);
    }
}
