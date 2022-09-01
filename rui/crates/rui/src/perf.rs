//! A Frames Per Second counter.

use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime};

/// Measures Frames Per Second (FPS).
#[derive(Debug)]
pub struct FPSCounter {
    measure: f64,
    prev_ts: Instant,
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
            measure: 1. / 60.,
            prev_ts: Instant::now(),
        }
    }

    /// Updates the FPSCounter and returns number of frames.
    pub fn tick(&mut self) -> usize {
        let now = Instant::now();
        let duration = now - self.prev_ts;
        self.prev_ts = now;
        const SMOOTHING: f64 = 0.9;
        self.measure = (self.measure * SMOOTHING) + (duration.as_secs_f64() * (1.0 - SMOOTHING));
        self.fps()
    }

    pub fn fps(&self) -> usize {
        (1. / self.measure) as usize
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
