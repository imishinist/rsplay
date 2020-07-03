use crate::data::Scenario;
use crate::runner;

pub async fn run(scenario: Scenario) {
    let pacer = scenario.pacer();
    let duration = scenario.duration();

    runner::Runner::new().run(scenario.clone(), pacer, duration).await;
}
