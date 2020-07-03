use crate::data::{Pace, Scenario};
use crate::runner;
use crate::pace::{Rate, Linear};


pub async fn run(scenario: Scenario) {
    let pace = scenario.clone().pace;
    let duration = scenario.duration();

    match pace {
        Pace::Rate(inner) => {
            let pacer: Rate = inner.into();
            runner::Runner::new().run(scenario, pacer, duration).await;
        },
        Pace::Linear(inner) => {
            let pacer: Linear = inner.into();
            runner::Runner::new().run(scenario, pacer, duration).await;
        },
    };
}
