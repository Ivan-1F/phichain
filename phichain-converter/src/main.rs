mod options;
mod utils;

use crate::options::{
    CliCommonOutputOptions, CliOfficialInputOptions, CliOfficialOutputOptions, CliRpeInputOptions,
};
use crate::utils::i18n_str;
use anyhow::bail;
use clap::{Parser, ValueEnum};
use phichain_chart::serialization::PhichainChart;
use phichain_format::official::OfficialChart;
use phichain_format::rpe::RpeChart;
use phichain_format::{ChartFormat, CommonOutputOptions};
use rust_i18n::t;
use serde::Serialize;
use std::path::PathBuf;
use strum::Display;

rust_i18n::i18n!("locales", fallback = "en-US");

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

#[derive(ValueEnum, Debug, Display, Clone)]
#[clap(rename_all = "kebab_case")]
#[strum(serialize_all = "snake_case")]
enum Format {
    Official,
    Phichain,
    Rpe,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "phichain-converter")]
#[command(about = i18n_str("cli.about"))]
#[command(after_help = i18n_str("cli.examples"))]
pub struct Args {
    #[arg(required = true, help = t!("cli.input").to_string())]
    input: PathBuf,
    #[arg(required = false, default_value = "output.json", help = t!("cli.output").to_string())]
    output: PathBuf,

    #[arg(long, help = t!("cli.from").to_string())]
    from: Option<Format>,
    #[arg(long, help = t!("cli.to").to_string())]
    to: Format,

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

fn convert(args: Args) -> anyhow::Result<()> {
    if !args.input.exists() {
        bail!("No such file: {}", args.input.display());
    }

    if args.input.is_dir() {
        bail!("Expected a file, got a directory: {}", args.input.display());
    }

    let file = std::fs::File::open(&args.input)?;

    let Some(from) = args.from else {
        bail!("Format inference is not yet supported")
    };

    let chart = match from {
        Format::Official => Chart::Official(serde_json::from_reader(file)?),
        Format::Phichain => Chart::Phichain(serde_json::from_reader(file)?),
        Format::Rpe => Chart::Rpe(serde_json::from_reader(file)?),
    };

    let phichain = match chart {
        Chart::Official(official) => official.to_phichain(&args.official_input_options.into())?,
        Chart::Phichain(phichain) => phichain.to_phichain(&())?,
        Chart::Rpe(rpe) => rpe.to_phichain(&args.rpe_input_options.into())?,
    };

    let output = match args.to {
        Format::Official => Chart::Official(OfficialChart::from_phichain(
            phichain,
            &args.official_output_options.into(),
        )?),
        Format::Phichain => Chart::Phichain(PhichainChart::from_phichain(phichain, &())?),
        Format::Rpe => Chart::Rpe(RpeChart::from_phichain(phichain, &())?),
    };

    let output = output.apply_common_output_options(&args.common_output_options.into());

    let output_file = std::fs::File::create(&args.output)?;
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
    if let Err(err) = convert(args) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
