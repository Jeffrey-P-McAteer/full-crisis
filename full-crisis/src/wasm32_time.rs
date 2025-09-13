/// WASM time implementation using performance.now()
use std::time::Duration;
use web_sys::window;

#[derive(Debug, Clone, Copy)]
pub struct PlatformInstant {
    timestamp_ms: f64,
}

impl PlatformInstant {
    pub fn now() -> Self {
        let timestamp_ms = if let Some(window) = window() {
            if let Some(performance) = window.performance() {
                performance.now()
            } else {
                // Fallback to a monotonic counter
                static mut COUNTER: f64 = 0.0;
                unsafe {
                    COUNTER += 1.0;
                    COUNTER
                }
            }
        } else {
            // Ultimate fallback - use a monotonic counter
            static mut COUNTER: f64 = 0.0;
            unsafe {
                COUNTER += 1.0;
                COUNTER
            }
        };
        
        Self { timestamp_ms }
    }
    
    pub fn duration_since(&self, earlier: Self) -> Duration {
        let diff_ms = self.timestamp_ms - earlier.timestamp_ms;
        Duration::from_millis(diff_ms.max(0.0) as u64)
    }
    
    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        now.duration_since(*self)
    }
}

impl PartialEq for PlatformInstant {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp_ms == other.timestamp_ms
    }
}

impl PartialOrd for PlatformInstant {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.timestamp_ms.partial_cmp(&other.timestamp_ms)
    }
}

/// Re-export storage functions for consistency with native_time
pub fn get_attr(name: &str) -> Option<String> {
    super::wasm32_storage::get_attr(name)
}

pub fn set_attr(name: &str, value: &str) {
    super::wasm32_storage::set_attr(name, value)
}