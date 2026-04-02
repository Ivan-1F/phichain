mod error;
mod i18n;
mod options;
mod telemetry;

use crate::error::{unwrap_infallible, ConvertError};
use crate::i18n::{i18n_str, locale};
use crate::options::{
    CliCommonOutputOptions, CliOfficialInputOptions, CliOfficialOutputOptions, CliRpeInputOptions,
};
use clap::{Parser, Subcommand, ValueEnum};
use owo_colors::OwoColorize;
use phichain_chart::serialization::PhichainChart;
use phichain_format::official::OfficialChart;
use phichain_format::rpe::RpeChart;
use phichain_format::{ChartFormat, CommonOutputOptions};
use rust_i18n::t;
use serde::Serialize;
use std::io::Read;
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

#[derive(ValueEnum, Debug, Display, Copy, Clone, Serialize)]
#[clap(rename_all = "kebab_case")]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum Format {
    Official,
    Phichain,
    Rpe,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "phichain-converter telemetry")]
#[command(hide = true)]
struct TelemetryCli {
    #[command(subcommand)]
    command: TelemetryCommand,
}

#[derive(Subcommand, Debug, Clone)]
enum TelemetryCommand {
    Flush { path: PathBuf },
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

    #[arg(long, help = t!("cli.no_telemetry").to_string())]
    no_telemetry: bool,
}

fn infer_format(content: &str) -> Result<Format, ConvertError> {
    let value: serde_json::Value = serde_json::from_str(content)?;

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

fn read_input(path: &std::path::Path) -> Result<String, ConvertError> {
    if path.as_os_str() == "-" {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        return Ok(input);
    }

    if !path.exists() {
        return Err(ConvertError::NoSuchFile(path.to_path_buf()));
    }

    if path.is_dir() {
        return Err(ConvertError::ExpectedFile(path.to_path_buf()));
    }

    Ok(std::fs::read_to_string(path)?)
}

#[derive(Serialize)]
struct ChartMetrics {
    lines: usize,
    notes: usize,
    events: usize,
}

fn collect_chart_metrics(lines: &[phichain_chart::serialization::SerializedLine]) -> ChartMetrics {
    let mut metrics = ChartMetrics {
        lines: 0,
        notes: 0,
        events: 0,
    };
    for line in lines {
        metrics.lines += 1;
        metrics.notes += line.notes.len();
        metrics.events += line.events.len();
        let child = collect_chart_metrics(&line.children);
        metrics.lines += child.lines;
        metrics.notes += child.notes;
        metrics.events += child.events;
    }
    metrics
}

#[derive(Serialize)]
struct ConvertTelemetry {
    locale: String,
    from: Option<Format>,
    to: Format,
    format_inferred: bool,
    success: bool,
    error_kind: Option<&'static str>,
    duration_ms: u64,
    input: Option<ChartMetrics>,
    output: Option<ChartMetrics>,
    options: serde_json::Value,
}

impl Chart {
    fn metrics(&self) -> ChartMetrics {
        match self {
            Chart::Phichain(c) => collect_chart_metrics(&c.lines),
            Chart::Official(c) => ChartMetrics {
                lines: c.lines.len(),
                notes: c
                    .lines
                    .iter()
                    .map(|l| l.notes_above.len() + l.notes_below.len())
                    .sum(),
                events: c
                    .lines
                    .iter()
                    .map(|l| {
                        l.move_events.len()
                            + l.rotate_events.len()
                            + l.opacity_events.len()
                            + l.speed_events.len()
                    })
                    .sum(),
            },
            Chart::Rpe(c) => ChartMetrics {
                lines: c.judge_line_list.len(),
                notes: c.judge_line_list.iter().map(|l| l.notes.len()).sum(),
                events: c
                    .judge_line_list
                    .iter()
                    .map(|l| {
                        l.event_layers
                            .iter()
                            .map(|layer| {
                                layer.move_x_events.len()
                                    + layer.move_y_events.len()
                                    + layer.rotate_events.len()
                                    + layer.alpha_events.len()
                                    + layer.speed_events.len()
                            })
                            .sum::<usize>()
                    })
                    .sum(),
            },
        }
    }
}

fn convert(args: Args, meta: &mut ConvertTelemetry) -> Result<(), ConvertError> {
    let input = read_input(&args.input)?;

    let (from, inferred) = match args.from {
        Some(f) => (f, false),
        None => (infer_format(&input)?, true),
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

    meta.from = Some(from);
    meta.format_inferred = inferred;

    let input_chart = match from {
        Format::Official => Chart::Official(serde_json::from_str(&input)?),
        Format::Phichain => Chart::Phichain(serde_json::from_str(&input)?),
        Format::Rpe => Chart::Rpe(serde_json::from_str(&input)?),
    };

    meta.input = Some(input_chart.metrics());

    let phichain = match input_chart {
        Chart::Official(official) => official.to_phichain(&args.official_input_options.into())?,
        Chart::Phichain(phichain) => unwrap_infallible(phichain.to_phichain(&())),
        Chart::Rpe(rpe) => rpe.to_phichain(&args.rpe_input_options.into())?,
    };

    let output_chart = match args.to {
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

    meta.output = Some(output_chart.metrics());

    let output = output_chart.apply_common_output_options(&args.common_output_options.into());

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
    // Route `phichain-converter telemetry <subcommand>` before normal arg parsing
    if std::env::args().nth(1).as_deref() == Some("telemetry") {
        let cli = TelemetryCli::parse_from(std::env::args().skip(1));
        match cli.command {
            TelemetryCommand::Flush { path } => {
                let _ = telemetry::flush(path);
            }
        }
        return;
    }

    tracing_subscriber::fmt().init();
    rust_i18n::set_locale(&locale());

    let args = Args::parse();
    let no_telemetry = args.no_telemetry;

    let mut meta = ConvertTelemetry {
        locale: locale(),
        from: args.from,
        to: args.to,
        format_inferred: args.from.is_none(),
        success: false,
        error_kind: None,
        duration_ms: 0,
        input: None,
        output: None,
        options: serde_json::json!({
            "official_input": &args.official_input_options,
            "official_output": &args.official_output_options,
            "rpe_input": &args.rpe_input_options,
            "common_output": &args.common_output_options,
        }),
    };

    let start = std::time::Instant::now();
    let result = convert(args, &mut meta);
    meta.duration_ms = start.elapsed().as_millis() as u64;
    meta.success = result.is_ok();
    if let Err(ref e) = result {
        meta.error_kind = Some(e.variant_name());
    }

    if !no_telemetry && !phichain_telemetry::env::telemetry_disabled() {
        let _ = telemetry::track(
            "phichain.converter.convert",
            serde_json::to_value(&meta).unwrap(),
        );
    }

    if let Err(err) = result {
        eprintln!("{}", err.red());
        std::process::exit(1);
    }
}
