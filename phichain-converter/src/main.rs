mod i18n;
mod options;

use crate::i18n::{i18n_str, locale};
use crate::options::{
    CliCommonOutputOptions, CliOfficialInputOptions, CliOfficialOutputOptions, CliRpeInputOptions,
};
use clap::{Parser, ValueEnum};
use owo_colors::OwoColorize;
use phichain_chart::serialization::PhichainChart;
use phichain_format::official::OfficialChart;
use phichain_format::rpe::RpeChart;
use phichain_format::{ChartFormat, CommonOutputOptions};
use rust_i18n::t;
use serde::Serialize;
use std::path::PathBuf;
use strum::Display;
use thiserror::Error;

/// Extract value from `Result<T, Infallible>`.
fn unwrap_infallible<T>(result: Result<T, std::convert::Infallible>) -> T {
    match result {
        Ok(v) => v,
        Err(e) => match e {},
    }
}

#[derive(Debug, Error)]
enum ConvertError {
    NoSuchFile(PathBuf),
    ExpectedFile(PathBuf),
    UnableToInferFormat,
    Io(#[from] std::io::Error),
    Json(#[from] serde_json::Error),
    OfficialInput(#[from] phichain_format::official::OfficialInputError),
    OfficialOutput(#[from] phichain_format::official::OfficialOutputError),
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertError::NoSuchFile(path) => {
                write!(f, "{}", t!("cli.error.no_such_file", path = path.display()))
            }
            ConvertError::ExpectedFile(path) => {
                write!(
                    f,
                    "{}",
                    t!("cli.error.expected_file", path = path.display())
                )
            }
            ConvertError::UnableToInferFormat => {
                write!(f, "{}", t!("cli.error.unable_to_infer_format"))
            }
            ConvertError::Io(e) => write!(f, "{e}"),
            ConvertError::Json(e) => write!(f, "{e}"),
            ConvertError::OfficialInput(e) => write!(f, "{e}"),
            ConvertError::OfficialOutput(e) => write!(f, "{e}"),
        }
    }
}

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

fn infer_format(path: &std::path::Path) -> Result<Format, ConvertError> {
    let content = std::fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&content)?;

    if value.get("BPMList").is_some() && value.get("META").is_some() {
        return Ok(Format::Rpe);
    }

    if value.get("formatVersion").is_some() && value.get("judgeLineList").is_some() {
        return Ok(Format::Official);
    }

    if value.get("format").is_some()
        && value.get("bpm_list").is_some()
        && value.get("lines").is_some()
    {
        return Ok(Format::Phichain);
    }

    Err(ConvertError::UnableToInferFormat)
}

fn convert(args: Args) -> Result<(), ConvertError> {
    if !args.input.exists() {
        return Err(ConvertError::NoSuchFile(args.input.clone()));
    }

    if args.input.is_dir() {
        return Err(ConvertError::ExpectedFile(args.input.clone()));
    }

    let file = std::fs::File::open(&args.input)?;

    let (from, inferred) = match args.from {
        Some(f) => (f, false),
        None => (infer_format(&args.input)?, true),
    };

    if inferred {
        eprintln!(
            "{}",
            t!(
                "cli.status.inferred_format",
                format = from.to_string().cyan()
            )
        );
    }

    let chart = match from {
        Format::Official => Chart::Official(serde_json::from_reader(file)?),
        Format::Phichain => Chart::Phichain(serde_json::from_reader(file)?),
        Format::Rpe => Chart::Rpe(serde_json::from_reader(file)?),
    };

    let phichain = match chart {
        Chart::Official(official) => official.to_phichain(&args.official_input_options.into())?,
        Chart::Phichain(phichain) => unwrap_infallible(phichain.to_phichain(&())),
        Chart::Rpe(rpe) => unwrap_infallible(rpe.to_phichain(&args.rpe_input_options.into())),
    };

    let output = match args.to {
        Format::Official => Chart::Official(OfficialChart::from_phichain(
            phichain,
            &args.official_output_options.into(),
        )?),
        Format::Phichain => Chart::Phichain(unwrap_infallible(PhichainChart::from_phichain(
            phichain,
            &(),
        ))),
        Format::Rpe => Chart::Rpe(unwrap_infallible(RpeChart::from_phichain(phichain, &()))),
    };

    let output = output.apply_common_output_options(&args.common_output_options.into());

    let output_name = if args.output.as_os_str() == "-" {
        serde_json::to_writer(std::io::stdout(), &output)?;
        println!(); // newline after JSON
        t!("cli.status.stdout").to_string()
    } else {
        let output_file = std::fs::File::create(&args.output)?;
        serde_json::to_writer(output_file, &output)?;
        args.output.display().to_string()
    };

    eprintln!(
        "{}",
        t!(
            "cli.status.converted",
            input = args.input.display().to_string().cyan(),
            from = from.to_string().cyan(),
            output = output_name.green(),
            to = args.to.to_string().green()
        )
    );

    Ok(())
}

fn main() {
    tracing_subscriber::fmt().init();

    rust_i18n::set_locale(&locale());

    let args = Args::parse();
    if let Err(err) = convert(args) {
        eprintln!("{}", err.red());
        std::process::exit(1);
    }
}
