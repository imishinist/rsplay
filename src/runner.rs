use crate::data::Scenario;
use crate::pace::Pacer;
use crate::validator::Validator;
use futures::future;
use log::{error, info, warn};
use reqwest::ClientBuilder;
use std::pin::Pin;
use std::time::{Duration, Instant};
use tokio::time;

pub struct Runner {
    scenario: Scenario,
    run_duration: Duration,

    validator: Validator,
}

impl Runner {
    pub async fn run(&self, pacer: Pin<Box<dyn Pacer + Send>>) {
        let (mut tx, rx) = spmc::channel();

        let client = ClientBuilder::new()
            .pool_idle_timeout(self.scenario.idle_timeout)
            .build()
            .unwrap();
        let url = self.scenario.url();
        let validator = self.validator.clone();
        let run_duration = self.run_duration;

        info!("start scenario run");
        tokio::spawn(async move {
            let start = Instant::now();
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
                    time::delay_for(wait).await;
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
                let validator = validator.clone();
                let client = client.clone();
                let rx = rx.clone();
                let url = url.clone();
                tokio::spawn(async move {
                    info!("worker spawn");
                    while rx.recv().is_ok() {
                        let response = match client.get(url.as_str()).send().await {
                            Ok(res) => res,
                            Err(e) => {
                                error!("{:?}", e);
                                return;
                            }
                        };
                        match validator.validate(response) {
                            Ok(_) => info!("OK"),
                            Err(err) => warn!("{}", err),
                        }
                    }
                })
            })
            .collect::<Vec<_>>();
        future::join_all(workers).await;
    }
}

pub struct RunnerBuilder<ScenarioType, DurationType, ValidatorType> {
    scenario: ScenarioType,
    run_duration: DurationType,

    validator: ValidatorType,
}

impl RunnerBuilder<(), (), ()> {
    pub fn new() -> Self {
        RunnerBuilder {
            scenario: (),
            run_duration: (),
            validator: (),
        }
    }
}

impl RunnerBuilder<Scenario, Duration, Validator> {
    pub fn build(self) -> Runner {
        Runner {
            scenario: self.scenario,
            run_duration: self.run_duration,
            validator: self.validator,
        }
    }
}

impl<ScenarioType, DurationType, ValidatorType>
    RunnerBuilder<ScenarioType, DurationType, ValidatorType>
{
    pub fn scenario(
        self,
        scenario: Scenario,
    ) -> RunnerBuilder<Scenario, DurationType, ValidatorType> {
        RunnerBuilder {
            scenario,
            run_duration: self.run_duration,
            validator: self.validator,
        }
    }

    pub fn run_duration(
        self,
        run_duration: Duration,
    ) -> RunnerBuilder<ScenarioType, Duration, ValidatorType> {
        RunnerBuilder {
            scenario: self.scenario,
            run_duration,
            validator: self.validator,
        }
    }

    pub fn validator(
        self,
        validator: Validator,
    ) -> RunnerBuilder<ScenarioType, DurationType, Validator> {
        RunnerBuilder {
            scenario: self.scenario,
            run_duration: self.run_duration,
            validator,
        }
    }
}
