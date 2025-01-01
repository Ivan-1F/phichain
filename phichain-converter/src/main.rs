use clap::{Parser, ValueEnum};
use phichain_chart::format::official::OfficialChart;
use phichain_chart::format::rpe::RpeChart;
use phichain_chart::primitive::{Format, PrimitiveChart};
use phichain_chart::serialization::PhichainChart;
use std::io::Write;
use std::path::PathBuf;
use strum::Display;

#[derive(ValueEnum, Debug, Display, Clone)]
#[clap(rename_all = "kebab_case")]
#[strum(serialize_all = "snake_case")]
enum Formats {
    Official,
    Phichain,
    Rpe,
    Primitive,
}

#[derive(Debug, Parser)]
#[command(name = "phichain-converter")]
#[command(about = "Converts Phigros charts between different formats")]
struct Args {
    /// The input chart format
    #[arg(short, long, required = true)]
    input: Formats,
    /// The output chart format
    #[arg(short, long, required = true)]
    output: Formats,

    /// The path of the input chart
    #[arg(required = true)]
    path: PathBuf,
}

fn convert(args: Args) -> anyhow::Result<()> {
    let file = std::fs::File::open(&args.path)?;

    println!("Converting chart into primitive chart...");

    let primitive = match args.input {
        Formats::Official => {
            let chart: OfficialChart = serde_json::from_reader(file)?;
            chart.into_primitive()?
        }
        Formats::Phichain => {
            let chart: PhichainChart = serde_json::from_reader(file)?;
            phichain_compiler::compile(chart)?
        }
        Formats::Rpe => {
            let chart: RpeChart = serde_json::from_reader(file)?;
            chart.into_primitive()?
        }
        Formats::Primitive => {
            let chart: PrimitiveChart = serde_json::from_reader(file)?;
            chart.into_primitive()?
        }
    };

    println!("Converting chart into `{}` chart...", args.output);

    let output = match args.output {
        Formats::Official => {
            let chart = OfficialChart::from_primitive(primitive)?;
            serde_json::to_string(&chart)?
        }
        Formats::Phichain => {
            let chart = PhichainChart::from_primitive(primitive)?;
            serde_json::to_string(&chart)?
        }
        Formats::Rpe => {
            let chart = RpeChart::from_primitive(primitive)?;
            serde_json::to_string(&chart)?
        }
        Formats::Primitive => {
            let chart = PrimitiveChart::from_primitive(primitive)?;
            serde_json::to_string(&chart)?
        }
    };

    let output_path = args.path.with_extension(format!("{}.json", args.output));

    let mut output_file = std::fs::File::create(output_path)?;
    output_file.write_all(output.as_bytes())?;

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(err) = convert(args) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
