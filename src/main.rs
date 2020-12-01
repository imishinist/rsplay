use clap::{Clap, ValueHint};
use futures::future;
use log::error;
use rsplay::{data, scenario, Scenario};
use std::path::PathBuf;
use std::process;

#[derive(Clap, Debug)]
#[clap(author, about, version)]
struct Opts {
    #[clap(long, value_hint=ValueHint::FilePath, about = "scenario file path")]
    scenario: PathBuf,
}

async fn do_main(scenarios: Vec<Scenario>) {
    let tasks = scenarios.into_iter().map(scenario::run).collect::<Vec<_>>();

    future::join_all(tasks).await;
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let opt: Opts = Opts::parse();
    let scenarios = data::load(opt.scenario).unwrap_or_else(|err| {
        error!("load scenario error: {:?}", err);
        process::exit(1);
    });

    do_main(scenarios).await;
}
