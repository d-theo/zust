use std::path::PathBuf;

const HOUR: u64 = 3600;
const DAY: u64 = 24 * HOUR;
const WEEK: u64 = 7 * DAY;

#[derive(Debug, Clone, PartialEq)]
pub struct Dir {
    pub path: PathBuf,
    pub count: f64,
    pub last_access: u64,
}

/// Multiplier applied to `count` depending on the age of the last access.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Weights {
    pub hour: f64,
    pub day: f64,
    pub week: f64,
    pub older: f64,
}

impl Default for Weights {
    fn default() -> Weights {
        Weights { hour: 4.0, day: 2.0, week: 0.5, older: 0.25 }
    }
}

impl Dir {
    pub fn frecency(&self, now: u64, weights: &Weights) -> f64 {
        let weight = match now.saturating_sub(self.last_access) {
            age if age < HOUR => weights.hour,
            age if age < DAY => weights.day,
            age if age < WEEK => weights.week,
            _ => weights.older,
        };
        self.count * weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(count: f64, last_access: u64) -> Dir {
        Dir { path: PathBuf::from("/tmp"), count, last_access }
    }

    #[test]
    fn frecency_weights_by_age() {
        let now = 10 * WEEK;
        let w = Weights::default();
        assert_eq!(dir(10.0, now - 60).frecency(now, &w), 40.0);
        assert_eq!(dir(10.0, now - 2 * HOUR).frecency(now, &w), 20.0);
        assert_eq!(dir(10.0, now - 2 * DAY).frecency(now, &w), 5.0);
        assert_eq!(dir(10.0, now - 2 * WEEK).frecency(now, &w), 2.5);
    }

    #[test]
    fn frecency_uses_custom_weights() {
        let now = 10 * WEEK;
        let w = Weights { hour: 1.0, day: 3.0, week: 5.0, older: 7.0 };
        assert_eq!(dir(2.0, now - 60).frecency(now, &w), 2.0);
        assert_eq!(dir(2.0, now - 2 * HOUR).frecency(now, &w), 6.0);
        assert_eq!(dir(2.0, now - 2 * DAY).frecency(now, &w), 10.0);
        assert_eq!(dir(2.0, now - 2 * WEEK).frecency(now, &w), 14.0);
    }

    #[test]
    fn frecency_handles_future_timestamp() {
        assert_eq!(dir(1.0, 200).frecency(100, &Weights::default()), 4.0);
    }
}
