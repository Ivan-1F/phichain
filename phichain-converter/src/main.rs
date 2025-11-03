mod options;

use crate::options::{CliOfficialInputOptions, CliOfficialOutputOptions};
use clap::{Parser, ValueEnum};
use phichain_chart::serialization::PhichainChart;
use phichain_format::official::from_phichain::phichain_to_official;
use phichain_format::official::official_to_phichain;
use phichain_format::official::schema::OfficialChart;
use phichain_format::primitive::PrimitiveChart;
use phichain_format::rpe::schema::RpeChart;
use phichain_format::Format;
use rust_i18n::t;
use std::io::Write;
use std::path::PathBuf;
use strum::Display;

rust_i18n::i18n!("locales", fallback = "en-US");

#[derive(ValueEnum, Debug, Display, Clone)]
#[clap(rename_all = "kebab_case")]
#[strum(serialize_all = "snake_case")]
enum Formats {
    Official,
    Phichain,
    Rpe,
    Primitive,
}

macro_rules! define_format_args {
    (
        $(
            $field:ident => $variant:ident
        ),* $(,)?
    ) => {
        #[derive(Debug, Parser)]
        struct FormatArgs {
            $(
                #[arg(long, num_args = 0..=1, help = t!(concat!("cli.format_args.", stringify!($field))).to_string())]
                $field: Option<Option<PathBuf>>,
            )*
        }

        impl FormatArgs {
            fn collect_formats(&self, order: &[String]) -> Vec<(Formats, Option<PathBuf>)> {
                // Create a map of format name -> (Format, PathBuf)
                let mut format_map = std::collections::HashMap::new();
                $(
                    if let Some(path) = &self.$field {
                        format_map.insert(
                            stringify!($field).to_string(),
                            (Formats::$variant, path.clone())
                        );
                    }
                )*

                // Build result in the order specified by command line
                let mut format_args = vec![];
                for arg in order {
                    if let Some(entry) = format_map.get(arg) {
                        format_args.push(entry.clone());
                    }
                }

                format_args
            }

            /// Get the list of all supported format flag names
            fn format_flags() -> &'static [&'static str] {
                &[$(stringify!($field)),*]
            }
        }
    };
}

define_format_args! {
    phichain => Phichain,
    official => Official,
    rpe => Rpe,
    primitive => Primitive,
}

/// Extract the order of format flags from command line arguments
fn extract_format_order() -> Vec<String> {
    let format_flags = FormatArgs::format_flags();

    std::env::args()
        .filter_map(|arg| {
            if let Some(flag) = arg.strip_prefix("--") {
                if format_flags.contains(&flag) {
                    return Some(flag.to_string());
                }
            }
            None
        })
        .collect()
}

#[derive(Debug, Parser)]
#[command(name = "phichain-converter")]
#[command(about = t!("cli.about").to_string())]
#[command(after_help = t!("cli.examples").to_string())]
struct Args {
    #[command(flatten)]
    #[command(
        next_help_heading = cli_format_args_heading()
    )]
    formats: FormatArgs,

    #[command(flatten)]
    #[command(
        next_help_heading = cli_official_input_heading()
    )]
    official_input_options: CliOfficialInputOptions,

    #[command(flatten)]
    #[command(
        next_help_heading = cli_official_output_heading()
    )]
    official_output_options: CliOfficialOutputOptions,
}

fn cli_official_input_heading() -> &'static str {
    match t!("cli.official_input.heading") {
        Cow::Borrowed(s) => s,
        Cow::Owned(_) => {
            unreachable!()
        }
    }
}

fn cli_official_output_heading() -> &'static str {
    match t!("cli.official_output.heading") {
        Cow::Borrowed(s) => s,
        Cow::Owned(_) => {
            unreachable!()
        }
    }
}

fn cli_format_args_heading() -> &'static str {
    match t!("cli.format_args.heading") {
        Cow::Borrowed(s) => s,
        Cow::Owned(_) => {
            unreachable!()
        }
    }
}

impl Args {
    fn parse_args(self, format_order: &[String]) -> anyhow::Result<ParsedArgs> {
        let format_args = self.formats.collect_formats(format_order);

        if format_args.len() != 2 {
            anyhow::bail!(
                "Expected exactly 2 format flags, got {}. Usage: <input-format> <input-path> <output-format> [output-path]",
                format_args.len()
            );
        }

        let (input_format, input_path) = &format_args[0];
        let (output_format, output_path) = &format_args[1];

        let input_path = input_path
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Input format must have a path specified"))?;

        let output_path = output_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("output.json"));

        Ok(ParsedArgs {
            input: input_format.clone(),
            path: input_path,
            output: output_format.clone(),
            output_path,
            official_input_options: self.official_input_options,
            official_output_options: self.official_output_options,
        })
    }
}

struct ParsedArgs {
    input: Formats,
    path: PathBuf,
    output: Formats,
    output_path: PathBuf,

    official_input_options: CliOfficialInputOptions,
    official_output_options: CliOfficialOutputOptions,
}

fn convert(args: ParsedArgs) -> anyhow::Result<()> {
    if !args.path.exists() {
        anyhow::bail!("No such file: {}", args.path.display());
    }

    if args.path.is_dir() {
        anyhow::bail!("Expected a file, got a directory: {}", args.path.display());
    }

    let file = std::fs::File::open(&args.path)?;

    let output = if matches!(args.input, Formats::Official)
        && matches!(args.output, Formats::Phichain)
    {
        println!("Converting official chart into phichain chart...");

        let chart: OfficialChart = serde_json::from_reader(file)?;
        let phichain = official_to_phichain(chart, args.official_input_options.into())?;

        println!(
            "Converted to phichain chart: {} lines, {} notes, {} events",
            phichain.lines.len(),
            phichain.lines.iter().map(|l| l.notes.len()).sum::<usize>(),
            phichain.lines.iter().map(|l| l.events.len()).sum::<usize>(),
        );

        serde_json::to_string(&phichain)?
    } else if matches!(args.input, Formats::Phichain) && matches!(args.output, Formats::Official) {
        println!("Converting phichain chart into official chart...");

        let chart: PhichainChart = serde_json::from_reader(file)?;
        let phichain = phichain_to_official(chart, args.official_output_options.into())?;

        println!(
            "Converted to official chart: {} lines, {} notes, {} events",
            phichain.lines.len(),
            phichain
                .lines
                .iter()
                .map(|l| l.notes_above.len() + l.notes_below.len())
                .sum::<usize>(),
            phichain
                .lines
                .iter()
                .map(|l| l.move_events.len()
                    + l.opacity_events.len()
                    + l.rotate_events.len()
                    + l.speed_events.len())
                .sum::<usize>(),
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

    let mut output_file = std::fs::File::create(&args.output_path)?;
    output_file.write_all(output.as_bytes())?;

    Ok(())
}

/// Normalize locale from system to rust-i18n format
fn normalize_locale(locale: &str) -> String {
    // Remove encoding suffix and replace underscore
    let base = locale.split('.').next().unwrap_or(locale).replace('_', "-");

    // Map to available translation files
    match base.as_str() {
        "C" | "POSIX" => "en-US".to_string(),
        // macOS verbose formats
        "zh-Hans-CN" | "zh-Hans" | "zh-Hans-SG" => "zh-CN".to_string(),
        "zh-Hant-CN" | "zh-Hant-TW" | "zh-Hant" | "zh-Hant-HK" | "zh-Hant-MO" => {
            "zh-TW".to_string()
        }
        // already normalized
        _ => base,
    }
}

/// Get system locale with fallback
fn locale() -> String {
    std::env::var("PHICHAIN_LANG")
        .ok()
        .or(sys_locale::get_locale().map(|loc| normalize_locale(&loc)))
        .unwrap_or_else(|| "en-US".to_string())
}

fn main() {
    rust_i18n::set_locale(&locale());

    let args = Args::parse();
    let parsed_args = match args.parse_args(&extract_format_order()) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };
    if let Err(err) = convert(parsed_args) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
