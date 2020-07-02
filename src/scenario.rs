use crate::data::Scenario;
use crate::{runner, pace};
use crate::pace::Pacer;


pub async fn run(scenario: Scenario) {
    let pace = scenario.pace.clone();
    let duration = scenario.duration();
    runner::Runner::new().run(scenario, pace, duration).await;
}
