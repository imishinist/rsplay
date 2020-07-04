use clap::{value_t_or_exit, App, Arg, ArgMatches};
use log::error;
use rsplay::{data, scenario};

struct CommandOption {
    scenario_file: String,
}

fn get_option(matches: ArgMatches) -> CommandOption {
    CommandOption {
        scenario_file: value_t_or_exit!(matches, "scenario", String),
    }
}

async fn do_main(scenarios: Vec<data::Scenario>) {
    let tasks = scenarios
        .into_iter()
        .map(|scenario| scenario::run(scenario))
        .collect::<Vec<_>>();

    futures::future::join_all(tasks).await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let app = App::new("rsplay")
        .version("0.1.0")
        .author("Taisuke Miyazaki<imishinist@gmail.com>")
        .about("http request with scenario")
        .arg(
            Arg::with_name("scenario")
                .help("scenario file path")
                .required(true)
                .long("scenario")
                .takes_value(true),
        );

    let opt = get_option(app.get_matches());
    let scenarios = data::load(opt.scenario_file).map_err(|err| {
        error!("load scenario error: {:?}", err);
        err
    })?;

    do_main(scenarios).await;
    Ok(())
}
