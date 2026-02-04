use clap::Parser;
use env_logger::Env;
use log::info;
use std::time::Instant;
use sudoku::{DynamicSudoku, error::Result, generator::DynamicGeneratorSettings};

fn parse_generator_settings(s: &str) -> Result<DynamicGeneratorSettings> {
    let settings: DynamicGeneratorSettings = serde_json::from_str(s)?;
    Ok(settings)
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, value_parser = parse_generator_settings)]
    generator_settings: DynamicGeneratorSettings,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(&args);

    env_logger::Builder::from_env(Env::default().default_filter_or("info,varisat=warn"))
        .format_indent(Some(0))
        .init();

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
