use crate::pace::{self, Pacer};
use crate::url_format;
use reqwest::Url;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::path::Path;
use std::fs::File;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,

    #[serde(with = "url_format")]
    pub url: Url,

    #[serde(default)]
    pub exit: ExitKind,

    #[serde(default)]
    pub pace: Pace,

    #[serde(with = "humantime_serde", default)]
    pub idle_timeout: Option<Duration>,

    #[serde(default)]
    validation: Option<Vec<Validation>>,
}

impl Scenario {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn duration(&self) -> Duration {
        match self.exit {
            ExitKind::Period(t) => t,
            ExitKind::Count(cnt) => {
                let sec = cnt as f64 / self.pace.rate(Duration::from_secs(0));
                Duration::from_secs(sec as u64)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Pace {
    #[serde(rename = "rate")]
    Rate {
        freq: u64,
        #[serde(with = "humantime_serde")]
        per: Duration,
    },
    Linear {
        a: f64,
        b: f64,
    }
}

impl Default for Pace {
    fn default() -> Self {
        use Pace::*;
        // 1 req/s
        Rate {
            freq: 1,
            per: Duration::from_secs(1),
        }
    }
}

impl Pacer for Pace {
    fn pace(&self, elapsed: Duration, hits: u64) -> (Duration, bool) {
        use Pace::*;
        match self {
            Rate { freq, per } => pace::Rate {freq: freq.clone(), per: per.clone()  }.pace(elapsed, hits),
            Linear { a, b } => pace::Linear { a: a.clone(), b: b.clone() }.pace(elapsed, hits),
        }
    }

    fn rate(&self, elapsed: Duration) -> f64 {
        use Pace::*;
        match self {
            Rate { freq, per } => pace::Rate {freq: freq.clone(), per: per.clone()  }.rate(elapsed),
            Linear { a, b } => pace::Linear { a: a.clone(), b: b.clone() }.rate(elapsed),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExitKind {
    #[serde(rename = "period", with = "humantime_serde")]
    Period(Duration),
    #[serde(rename = "count")]
    Count(usize),
}

impl Default for ExitKind {
    fn default() -> Self {
        use ExitKind::*;
        Period(Duration::from_secs(60))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Validation {
    name: String,
    status_code: u32,
}


pub fn load<P: AsRef<Path>>(filename: P) -> anyhow::Result<Vec<Scenario>> {
    let file = File::open(filename)?;
    Ok(serde_yaml::from_reader(file)?)
}
#[cfg(test)]
mod tests {
    use reqwest::Url;
    use std::time::Duration;
    use super::{*, Pace::*, ExitKind::*};

    fn to_scenarios(scenario_str: &str) -> Vec<Scenario> {
        serde_yaml::from_str(scenario_str).unwrap()
    }

    #[test]
    fn scenario_unmarshal_test() {
        assert_eq!(
            to_scenarios(
                r#"
        - name: normal-scenario
          url: http://localhost:8080/test?q=1
          exit:
            count: 100
          pace:
            freq: 1
            per: 1s
          idle_timeout: 10s
          validation:
            - name: status = 200
              status_code: 200
        - name: period-scenario
          url: http://localhost:8080/test?q=1
          exit:
            period: 10s
        - name: minimum-scenario
          url: http://localhost:8080/test?q=1
        "#
            ),
            vec![
                Scenario {
                    name: "normal-scenario".to_owned(),
                    url: Url::parse("http://localhost:8080/test?q=1").unwrap(),
                    exit: Count(100),
                    pace: Rate {
                        freq: 1,
                        per: Duration::from_secs(1)
                    },
                    idle_timeout: Some(Duration::from_secs(10)),
                    validation: Some(vec![Validation {
                        name: "status = 200".to_owned(),
                        status_code: 200
                    }]),
                },
                Scenario {
                    name: "period-scenario".to_owned(),
                    url: Url::parse("http://localhost:8080/test?q=1").unwrap(),
                    exit: Period(Duration::from_secs(10)),
                    pace: Default::default(),
                    idle_timeout: None,
                    validation: None
                },
                Scenario {
                    name: "minimum-scenario".to_owned(),
                    url: Url::parse("http://localhost:8080/test?q=1").unwrap(),
                    exit: Default::default(),
                    pace: Default::default(),
                    idle_timeout: None,
                    validation: None
                },
            ]
        );
    }
}
