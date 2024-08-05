use clap::{Parser, ValueEnum};
use phichain_chart::format::official::OfficialChart;
use phichain_chart::format::rpe::RpeChart;
use phichain_chart::format::Format;
use phichain_chart::serialization::PhichainChart;
use std::io::Write;
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
enum Formats {
    Official,
    Phichain,
    Rpe,
}

#[derive(Debug, Parser)]
#[command(name = "phichain-converter")]
#[command(about = "Converts Phigros charts between different formats")]
struct Args {
    #[arg(short, long, required = true)]
    input: Formats,
    #[arg(short, long, required = true)]
    output: Formats,

    #[arg(required = true)]
    path: PathBuf,
}

fn convert(args: Args) -> anyhow::Result<()> {
    let file = std::fs::File::open(&args.path)?;
    let chart = match args.input {
        Formats::Official => {
            let chart: OfficialChart = serde_json::from_reader(file)?;
            chart.into_phichain()?
        }
        Formats::Phichain => {
            let chart: PhichainChart = serde_json::from_reader(file)?;
            chart.into_phichain()?
        }
        Formats::Rpe => {
            let chart: RpeChart = serde_json::from_reader(file)?;
            chart.into_phichain()?
        }
    };

    let output = match args.output {
        Formats::Official => serde_json::to_string(&OfficialChart::from_phichain(chart)?)?,
        Formats::Phichain => serde_json::to_string(&PhichainChart::from_phichain(chart)?)?,
        Formats::Rpe => serde_json::to_string(&RpeChart::from_phichain(chart)?)?,
    };

    let output_path = args.path.with_extension(match args.output {
        Formats::Official => "official.json",
        Formats::Phichain => "phichain.json",
        Formats::Rpe => "rpe.json",
    });

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
