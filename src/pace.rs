use std::default::Default;
use std::ops::Sub;
use std::time::Duration;

pub trait Pacer {
    /// fn pace(&self, elapsed: Duration, hits: u64) -> (wait: Duration, stop: bool)
    fn pace(&self, elapsed: Duration, hits: u64) -> (Duration, bool);
    fn rate(&self, elapsed: Duration) -> f64;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rate {
    pub freq: u64,
    pub per: Duration,
}

impl Default for Rate {
    fn default() -> Self {
        Self {
            freq: 1,
            per: Duration::from_secs(1),
        }
    }
}

impl Pacer for Rate {
    fn pace(&self, elapsed: Duration, hits: u64) -> (Duration, bool) {
        let immediately = Duration::from_secs(0);

        if self.freq == 0 || self.per == immediately {
            return (immediately, false);
        } else if self.per < immediately {
            return (immediately, true);
        }

        let n = elapsed.as_nanos() / self.per.as_nanos();
        let expected = self.freq as u64 * n as u64;

        if hits < expected {
            return (immediately, false);
        }

        let interval = self.per.as_nanos() / self.freq as u128;

        let delta = Duration::from_nanos((hits + 1) * interval as u64);

        (delta.sub(elapsed), false)
    }

    fn rate(&self, _elapsed: Duration) -> f64 {
        self.freq as f64 / self.per.as_nanos() as f64 * 1e9
    }
}

#[cfg(test)]
mod test {
    fn float_eq(x: f64, y: f64) -> bool {
        println!("{} {}", x, y);

        let marg = 1e-6 * x.abs().min(y.abs());
        (x - y).abs() <= marg.max(1e-6)
    }

    use crate::pace::Pacer;
    use std::time::Duration;

    #[test]
    fn pacer_pace_test() {
        let sec = Duration::from_secs(1);
        let immediately = Duration::from_secs(0);

        let table = vec![
            ((1, sec), (1 * sec, 0), (immediately, false)),
            ((1, sec), (2 * sec, 0), (immediately, false)),
            ((1, sec), (1 * sec, 1), (sec, false)),
            ((1, sec), (1 * sec, 2), (2 * sec, false)),
            ((1, sec), (1 * sec, 10), (10 * sec, false)),
            ((1, sec), (11 * sec, 10), (immediately, false)),
            (
                (2, sec),
                (49 * sec / 10, 9),
                (Duration::from_millis(100), false),
            ),
            ((0, sec), (sec, 0), (immediately, false)),
            ((1, Duration::from_secs(0)), (sec, 0), (immediately, false)),
            ((0, Duration::from_secs(0)), (sec, 0), (immediately, false)),
        ];

        for ((freq, per), (elapsed, hits), (wait, stop)) in table {
            let r = super::Rate { freq, per };
            let (gwait, gstop) = r.pace(elapsed, hits);
            assert_eq!(gstop, stop);
            assert_eq!(gwait, wait);
        }
    }

    #[test]
    fn pacer_rate_test() {
        let sec = Duration::from_secs(1);
        let min = sec * 60;

        let table = vec![
            ((60, 1 * min), 1.0),
            ((120, 1 * min), 2.0),
            ((30, 1 * min), 0.5),
            ((500, 1 * sec), 500.0),
        ];

        for ((freq, per), expected) in table {
            let r = super::Rate { freq, per };
            let have = r.rate(Duration::from_secs(0));
            assert!(float_eq(have, expected));
        }
    }
}
