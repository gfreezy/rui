//! A Frames Per Second counter.

use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime};

/// Measures Frames Per Second (FPS).
#[derive(Debug)]
pub struct FPSCounter {
    /// The last registered frames.
    last_second_frames: Vec<u128>,
}

impl Default for FPSCounter {
    fn default() -> Self {
        FPSCounter::new()
    }
}

impl FPSCounter {
    /// Creates a new FPSCounter.
    pub fn new() -> FPSCounter {
        FPSCounter {
            last_second_frames: Vec::with_capacity(128),
        }
    }

    /// Updates the FPSCounter and returns number of frames.
    pub fn tick(&mut self) -> usize {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        let a_second_ago = (ts / 1_000_000 - 1) * 1_000_000;

        self.last_second_frames.retain(|t| *t > a_second_ago);

        self.last_second_frames.push(ts);
        self.last_second_frames.len()
    }
}

#[track_caller]
pub fn measure_time<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let ret = f();
    return ret;

    let duration = Instant::now() - start;
    let location = std::panic::Location::caller();

    tracing::debug!(
        "{label}: {}ms, {}:{}",
        duration.as_millis(),
        location.file(),
        location.line()
    );
    ret
}
