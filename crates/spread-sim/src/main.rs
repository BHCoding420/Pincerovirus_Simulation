use std::{error::Error, path::PathBuf, sync::Arc, time::Instant};

use clap::Parser;
use spread_sim_core::{
    model::{
        self,
        output::{self, Output},
    },
    validator::DummyValidator,
    InsufficientPaddingError,
};

/// Command line arguments.
#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long = "scenario")]
    scenario: PathBuf,
    #[arg(long = "out")]
    out: PathBuf,
    #[arg(long = "padding", default_value_t = 10)]
    padding: usize,
    #[arg(long = "slug", default_value_t = true)]
    slug: bool,
    #[arg(long = "rocket", default_value_t = false)]
    rocket: bool,
    #[arg(long = "starship", default_value_t = false)]
    starship: bool,
}

/// Entrypoint of the `spread-sim` binary.
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let scenario = model::scenario::load(&args.scenario)?;

    println!("Scenario: {}", scenario.name);
    println!("Ticks: {}", scenario.ticks);

    let simulate: Box<dyn FnOnce() -> Result<Output, InsufficientPaddingError>> = if args.rocket {
        Box::new(move || {
            let validator = Arc::new(DummyValidator);
            spread_sim_rocket::launch(scenario, args.padding, validator, args.starship)
        })
    } else {
        Box::new(move || Ok(spread_sim_slug::creep(scenario)))
    };

    println!(
        "Running simulation... {}",
        if args.rocket { "🚀" } else { "🐌" }
    );

    let start = Instant::now();
    let output = simulate()?;
    let duration = start.elapsed();

    println!("Time: {}ms", duration.as_millis());

    output::save(&output, &args.out)?;

    Ok(())
}
