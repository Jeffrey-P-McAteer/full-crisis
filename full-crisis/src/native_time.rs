/// Native time implementation using std::time
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct PlatformInstant {
    instant: Instant,
}

impl PlatformInstant {
    pub fn now() -> Self {
        Self {
            instant: Instant::now(),
        }
    }
    
    pub fn duration_since(&self, earlier: Self) -> Duration {
        self.instant.duration_since(earlier.instant)
    }
    
    pub fn elapsed(&self) -> Duration {
        self.instant.elapsed()
    }
}

impl PartialEq for PlatformInstant {
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl PartialOrd for PlatformInstant {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.instant.partial_cmp(&other.instant)
    }
}

pub fn get_attr(name: &str) -> Option<String> {
    super::native_storage::get_attr(name)
}

pub fn set_attr(name: &str, value: &str) {
    super::native_storage::set_attr(name, value)
}