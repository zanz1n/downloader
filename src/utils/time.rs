use std::time::{Duration, Instant};

#[inline]
pub fn fmt_since(instant: Instant) -> String {
    fmt_duration(since(instant))
}

#[inline]
pub fn since(instant: Instant) -> Duration {
    Instant::now() - instant
}

#[inline]
pub fn fmt_duration(latency: Duration) -> String {
    if latency > Duration::from_secs(1) {
        format!("{:.1}s", latency.as_secs_f64())
    } else if latency > Duration::from_millis(1) {
        format!("{}ms", latency.as_millis())
    } else {
        format!("{}μs", latency.as_micros())
    }
}
