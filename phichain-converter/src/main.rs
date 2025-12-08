mod options;
mod utils;

use crate::options::{
    CliCommonOutputOptions, CliOfficialInputOptions, CliOfficialOutputOptions, CliRpeInputOptions,
};
use crate::utils::i18n_str;
use clap::{Parser, ValueEnum};
use phichain_chart::serialization::PhichainChart;
use phichain_format::official::OfficialChart;
use phichain_format::rpe::RpeChart;
use phichain_format::{ChartFormat, CommonOutputOptions};
use serde::Serialize;
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

#[derive(Serialize)]
#[serde(untagged)]
enum Chart {
    Official(OfficialChart),
    Phichain(PhichainChart),
    Rpe(RpeChart),
}

impl Chart {
    fn apply_common_output_options(self, common_options: &CommonOutputOptions) -> Self {
        match self {
            Chart::Official(chart) => {
                Chart::Official(chart.apply_common_output_options(common_options))
            }
            Chart::Phichain(chart) => {
                Chart::Phichain(chart.apply_common_output_options(common_options))
            }
            Chart::Rpe(chart) => Chart::Rpe(chart.apply_common_output_options(common_options)),
        }
    }
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
                #[arg(long, num_args = 0..=1, help = i18n_str(concat!("cli.format_args.", stringify!($field))))]
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
#[command(about = i18n_str("cli.about"))]
#[command(after_help = i18n_str("cli.examples"))]
struct Args {
    #[command(flatten)]
    #[command(
        next_help_heading = i18n_str("cli.format_args.heading")
    )]
    formats: FormatArgs,

    #[command(flatten)]
    #[command(
        next_help_heading = i18n_str("cli.official_input.heading")
    )]
    official_input_options: CliOfficialInputOptions,

    #[command(flatten)]
    #[command(
        next_help_heading = i18n_str("cli.official_output.heading")
    )]
    official_output_options: CliOfficialOutputOptions,

    #[command(flatten)]
    #[command(
        next_help_heading = i18n_str("cli.rpe_input.heading")
    )]
    rpe_input_options: CliRpeInputOptions,

    #[command(flatten)]
    #[command(
        next_help_heading = i18n_str("cli.common_output.heading")
    )]
    common_output_options: CliCommonOutputOptions,
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
            rpe_input_options: self.rpe_input_options,
            common_output_options: self.common_output_options,
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
    rpe_input_options: CliRpeInputOptions,

    common_output_options: CliCommonOutputOptions,
}

fn convert(args: ParsedArgs) -> anyhow::Result<()> {
    if !args.path.exists() {
        anyhow::bail!("No such file: {}", args.path.display());
    }

    if args.path.is_dir() {
        anyhow::bail!("Expected a file, got a directory: {}", args.path.display());
    }

    let file = std::fs::File::open(&args.path)?;

    let input = match args.input {
        Formats::Official => Chart::Official(serde_json::from_reader(file)?),
        Formats::Phichain => Chart::Phichain(serde_json::from_reader(file)?),
        Formats::Rpe => Chart::Rpe(serde_json::from_reader(file)?),
        Formats::Primitive => {
            unimplemented!()
        }
    };

    let phichain = match input {
        Chart::Official(official) => official.to_phichain(&args.official_input_options.into())?,
        Chart::Phichain(phichain) => phichain.to_phichain(&())?,
        Chart::Rpe(rpe) => rpe.to_phichain(&args.rpe_input_options.into())?,
    };

    let output = match args.output {
        Formats::Official => Chart::Official(OfficialChart::from_phichain(
            phichain,
            &args.official_output_options.into(),
        )?),
        Formats::Phichain => Chart::Phichain(PhichainChart::from_phichain(phichain, &())?),
        Formats::Rpe => Chart::Rpe(RpeChart::from_phichain(phichain, &())?),
        Formats::Primitive => {
            unimplemented!()
        }
    };

    let output = output.apply_common_output_options(&args.common_output_options.into());

    let output_file = std::fs::File::create(&args.output_path)?;
    serde_json::to_writer(output_file, &output)?;

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
    tracing_subscriber::fmt().init();

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
