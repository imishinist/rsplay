use crate::{pace, runner};
use crate::pace::Pacer;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub url: String,

    pub exit: ExitKind,

    pub pace: Pace,

    pub disable_keepalive: bool,
    #[serde(with = "humantime_serde")]
    pub keepalive: Duration,
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,

    validation: Vec<ValidationDef>,
}

impl Scenario {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn get_pace(&self) -> impl pace::Pacer {
        match self.pace {
            Pace::Rate {freq, per} => {
                pace::Rate{freq, per}
            }
        }
    }

    pub fn duration(&self) -> Duration {
        match self.exit {
            ExitKind::Period(t) => t,
            ExitKind::Count(cnt) => {
                let sec = cnt as f64 / self.get_pace().rate(Duration::from_secs(0));
                Duration::from_secs(sec as u64)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Pace {
    #[serde(rename = "rate")]
    Rate{
        freq: u64,
        #[serde(with = "humantime_serde")]
        per: Duration,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExitKind {
    #[serde(rename = "period", with = "humantime_serde")]
    Period(Duration),
    #[serde(rename = "count")]
    Count(usize),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValidationDef {
    name: String,
    status_code: u32,
}

pub fn load<P: AsRef<Path>>(filename: P) -> anyhow::Result<Vec<Scenario>> {
    let file = File::open(filename)?;
    Ok(serde_yaml::from_reader(file)?)
}

pub async fn run(scenario: Scenario) {
    let pacer = scenario.get_pace();
    let duration = scenario.duration();
    runner::Runner::new()
        .run(scenario, pacer, duration).await;
}
