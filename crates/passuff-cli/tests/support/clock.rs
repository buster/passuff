use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};
use passuff_core::Clock;

pub struct TestClock {
    now: Mutex<DateTime<Utc>>,
}

impl TestClock {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    pub fn advance_seconds(&self, seconds: i64) {
        if let Ok(mut now) = self.now.lock() {
            *now += Duration::seconds(seconds);
        }
    }
}

impl Clock for TestClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock().unwrap_or_else(|error| error.into_inner())
    }
}
