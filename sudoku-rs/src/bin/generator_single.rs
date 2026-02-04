use clap::Parser;
use env_logger::Env;
use log::*;
use std::time::Instant;
use sudoku::{DynamicSudoku, error::Result, generator::DynamicGeneratorSettings};

fn parse_generator_settings(s: &str) -> Result<DynamicGeneratorSettings> {
    let settings = serde_json::from_str(s)?;
    Ok(settings)
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, value_parser = parse_generator_settings)]
    generator_settings: DynamicGeneratorSettings,
}

fn main() -> Result<()> {
    let args = Args::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or("info,varisat=warn"))
        .format_indent(Some(0))
        .init();

    debug!("{:?}", args);

    let before = Instant::now();

    let grid = DynamicSudoku::generate(args.generator_settings, |progress| {
        info!("Generation progress: {progress:?}");
        Ok(())
    })?;

    let after = Instant::now();
    let total_time = after - before;

    println!("{grid}");
    dbg!(total_time);

    Ok(())
}
