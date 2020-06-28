use log::{error, info};
use crate::pace::Pacer;
use crate::scenario::Scenario;
use std::time::Duration;

#[derive(Debug)]
pub struct Runner {
    message_buf: usize,
}

impl Runner {
    pub fn new() -> Self {
        Self {
            message_buf: 100,
        }
    }

    pub async fn run<P>(&self, scenario: Scenario, pacer: P, run_duration: Duration)
    where P: Pacer + Send + 'static {
        let (mut tx, mut rx) = tokio::sync::mpsc::channel(self.message_buf);

        let client = reqwest::ClientBuilder::new()
            .pool_idle_timeout(scenario.idle_timeout)
            .build().unwrap();
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
}