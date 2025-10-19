use clap::{Parser, ValueEnum};
use phichain_chart::serialization::PhichainChart;
use phichain_format::official::official_to_phichain;
use phichain_format::official::schema::OfficialChart;
use phichain_format::primitive::PrimitiveChart;
use phichain_format::rpe::RpeChart;
use phichain_format::Format;
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
    if !args.path.exists() {
        anyhow::bail!("No such file: {}", args.path.display());
    }

    if args.path.is_dir() {
        anyhow::bail!("Expected a file, got a directory: {}", args.path.display());
    }

    let file = std::fs::File::open(&args.path)?;

    let output =
        if matches!(args.input, Formats::Official) && matches!(args.output, Formats::Phichain) {
            println!("Converting official chart into phichain chart...");

            let chart: OfficialChart = serde_json::from_reader(file)?;
            let phichain = official_to_phichain(chart)?;

            println!(
                "Converted to phichain chart: {} lines, {} notes, {} events",
                phichain.lines.len(),
                phichain.lines.iter().map(|l| l.notes.len()).sum::<usize>(),
                phichain.lines.iter().map(|l| l.events.len()).sum::<usize>(),
            );

            serde_json::to_string(&phichain)?
        } else {
            println!("Converting chart into primitive chart...");

            let primitive = match args.input {
                Formats::Official => {
                    let chart: OfficialChart = serde_json::from_reader(file)?;
                    chart.into_primitive()?
                }
                Formats::Phichain => {
                    let chart: PhichainChart = serde_json::from_reader(file)?;
                    phichain_format::compile_phichain_chart(chart)?
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

            println!(
                "Converted to primitive chart: {} lines, {} notes, {} events",
                primitive.lines.len(),
                primitive.lines.iter().map(|l| l.notes.len()).sum::<usize>(),
                primitive
                    .lines
                    .iter()
                    .map(|l| l.events.len())
                    .sum::<usize>(),
            );

            println!("Converting chart into `{}` chart...", args.output);

            match args.output {
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
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
