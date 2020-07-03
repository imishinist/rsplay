use crate::pace::Pacer;
use crate::data::Scenario;
use log::{error, info};
use std::time::Duration;
use std::pin::Pin;

#[derive(Debug)]
pub struct Runner {
    message_buf: usize,
}

impl Runner {
    pub fn new() -> Self {
        Self { message_buf: 100 }
    }

    pub async fn run(&self, scenario: Scenario, pacer: Pin<Box<dyn Pacer + Send>>, run_duration: Duration)
    {
        let (mut tx, rx) = spmc::channel();

        let client = reqwest::ClientBuilder::new()
            .pool_idle_timeout(scenario.idle_timeout)
            .build()
            .unwrap();
        let url = scenario.url();
        info!("start scenario run");
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            let mut count = 0;
            loop {
                let elapsed = start.elapsed();
                if elapsed > run_duration {
                    info!("scenario run exit");
                    break;
                }

                let (wait, stop) = pacer.pace(elapsed, count);
                if stop {
                    info!("stop by pacer: elapsed={:?}, hits={}", elapsed, count);
                    break;
                }

                if wait.as_nanos() != 0 {
                    tokio::time::delay_for(wait).await;
                }

                if let Err(err) = tx.send(()) {
                    error!("receiver dropped: {:?}", err);
                    break;
                }
                count += 1;
            }
        });

        let workers = (0..2)
            .map(|_| {
                let client = client.clone();
                let rx = rx.clone();
                let url = url.clone();
                tokio::spawn(async move {
                    info!("worker spawn");
                    while let Ok(_) = rx.recv() {
                        let response = match client.get(url.as_str()).send().await {
                            Ok(res) => res,
                            Err(e) => {
                                error!("{:?}", e);
                                return;
                            }
                        };
                        info!("status = {}", response.status());
                    }
                })
            })
            .collect::<Vec<_>>();
        futures::future::join_all(workers).await;
    }
}
