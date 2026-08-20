use std::time::{Duration, Instant};

/// Transport-independent idle/handshake timeout helper.
#[derive(Debug, Clone)]
pub struct ConnectionTimeout {
    started: Instant,
    timeout: Duration,
}

impl ConnectionTimeout {
    pub fn new(timeout: Duration) -> Self {
        Self { started: Instant::now(), timeout }
    }

    pub fn expired(&self) -> bool {
        self.started.elapsed() >= self.timeout
    }

    pub fn remaining(&self) -> Duration {
        self.timeout.saturating_sub(self.started.elapsed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn expires_after_timeout() {
        let timeout = ConnectionTimeout::new(Duration::from_millis(1));
        sleep(Duration::from_millis(2));
        assert!(timeout.expired());
    }
}
