use crate::options::{
    CliCommonOutputOptions, CliOfficialInputOptions, CliOfficialOutputOptions, CliRpeInputOptions,
};
use crate::utils::i18n_str;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use strum::Display;

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
    #[arg(required = true)]
    input: PathBuf,
    #[arg(required = false)]
    output: PathBuf,

    #[arg(long)]
    from: Option<Format>,
    #[arg(long)]
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

#[derive(Debug, Clone)]
struct ParsedArgs {
    input: Option<Format>,
    input_path: PathBuf,

    output: Format,
    output_path: PathBuf,

    official_input_options: CliOfficialInputOptions,
    official_output_options: CliOfficialOutputOptions,
    rpe_input_options: CliRpeInputOptions,
    common_output_options: CliCommonOutputOptions,
}

impl From<Args> for ParsedArgs {
    fn from(value: Args) -> Self {
        Self {
            input: value.from,
            input_path: value.input,
            output: value.to,
            output_path: value.output,
            official_input_options: value.official_input_options.into(),
            official_output_options: value.official_output_options.into(),
            rpe_input_options: value.rpe_input_options.into(),
            common_output_options: value.common_output_options.into(),
        }
    }
}

pub fn cli() {
    let args = Args::parse();
    let parsed_args = ParsedArgs::from(args);
    dbg!(&parsed_args);
}
