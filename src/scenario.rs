use crate::data::Scenario;
use crate::runner;
use crate::validator::Validator;

pub async fn run(scenario: Scenario) {
    let validator = Validator::new(scenario.validations());
    let pacer = scenario.pacer();
    let duration = scenario.duration();

    runner::Runner::new()
        .run(scenario.clone(), validator, pacer, duration)
        .await;
}
