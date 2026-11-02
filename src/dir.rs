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

impl Dir {
    pub fn frecency(&self, now: u64) -> f64 {
        let weight = match now.saturating_sub(self.last_access) {
            age if age < HOUR => 4.0,
            age if age < DAY => 2.0,
            age if age < WEEK => 0.5,
            _ => 0.25,
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
        assert_eq!(dir(10.0, now - 60).frecency(now), 40.0);
        assert_eq!(dir(10.0, now - 2 * HOUR).frecency(now), 20.0);
        assert_eq!(dir(10.0, now - 2 * DAY).frecency(now), 5.0);
        assert_eq!(dir(10.0, now - 2 * WEEK).frecency(now), 2.5);
    }

    #[test]
    fn frecency_handles_future_timestamp() {
        assert_eq!(dir(1.0, 200).frecency(100), 4.0);
    }
}
