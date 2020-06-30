use crate::data::Scenario;
use crate::runner;


pub async fn run(scenario: Scenario) {
    let pacer = scenario.get_pace();
    let duration = scenario.duration();
    runner::Runner::new().run(scenario, pacer, duration).await;
}
