use crate::pace;
use crate::pace::Pacer;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    name: String,
    url: String,

    exit: ExitKind,

    pace: Pace,

    disable_keepalive: bool,
    #[serde(with = "humantime_serde")]
    keepalive: Duration,
    #[serde(with = "humantime_serde")]
    idle_timeout: Duration,

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
    let run_duration = scenario.duration();

    let message_size = 20;
    let (mut tx, mut rx) = tokio::sync::mpsc::channel(message_size);

    let client = reqwest::Client::new();
    let url = scenario.url.as_str();

    info!("start scenario run");
    tokio::spawn(async move {
        let start = std::time::Instant::now();
        let mut count = 0;
        loop {
            let elapsed = start.elapsed();
            if elapsed > run_duration {
                info!("scenario run exit");
                return;
            }

            let (wait, stop) = pacer.pace(elapsed, count);
            if stop {
                info!(
                    "stop by pacer: elapsed={:?}, hits={}",
                    elapsed, count
                );
                return;
            }

            tokio::time::delay_for(wait).await;

            if let Err(err) = tx.send(()).await {
                error!("receiver dropped: {:?}", err);
                return;
            }
            count += 1;
        }
    });

    while let Some(_) = rx.recv().await {
        let response = match client.get(url).send().await {
            Ok(res) => res,
            Err(e) => {
                error!("{:?}", e);
                return;
            }
        };
        info!("status = {}", response.status());
    }
}
