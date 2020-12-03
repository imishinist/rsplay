use crate::data::Scenario;
use crate::runner::RunnerBuilder;
use crate::validator::Validator;

pub async fn run(scenario: Scenario) {
    let validator = Validator::new(scenario.validations());
    let pacer = scenario.pinned_pacer();
    let duration = scenario.duration();

    let runner = RunnerBuilder::new()
        .scenario(scenario)
        .validator(validator)
        .run_duration(duration)
        .build();

    runner.run(pacer).await;
}
