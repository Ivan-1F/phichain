use phichain_chart::beat::Beat;
use phichain_format::official::from_phichain::OfficialOutputOptions;
use phichain_format::official::OfficialInputOptions;
use rust_i18n::t;

/// CLI wrapper for OfficialInputOptions
#[derive(Debug, Clone, clap::Args)]
pub struct CliOfficialInputOptions {
    #[arg(long, default_value_t = false, help = t!("cli.official_input.no_easing_fitting").to_string())]
    no_easing_fitting: bool,

    #[arg(long, default_value_t = 0.1, help = t!("cli.official_input.easing_fitting_epsilon").to_string())]
    easing_fitting_epsilon: f32,

    #[arg(long, value_parser = clap::value_parser!(Beat), default_value = "1/4", help = t!("cli.official_input.constant_event_shrink_to").to_string())]
    constant_event_shrink_to: Beat,
}

impl From<CliOfficialInputOptions> for OfficialInputOptions {
    fn from(cli: CliOfficialInputOptions) -> Self {
        OfficialInputOptions {
            easing_fitting: !cli.no_easing_fitting,
            easing_fitting_epsilon: cli.easing_fitting_epsilon,
            constant_event_shrink_to: cli.constant_event_shrink_to,
        }
    }
}

/// CLI wrapper for OfficialInputOptions
#[derive(Debug, Clone, clap::Args)]
pub struct CliOfficialOutputOptions {
    #[arg(long, value_parser = clap::value_parser!(Beat), default_value = "1/32", help = t!("cli.official_output.minimum_beat").to_string())]
    minimum_beat: Beat,
}

impl From<CliOfficialOutputOptions> for OfficialOutputOptions {
    fn from(cli: CliOfficialOutputOptions) -> Self {
        OfficialOutputOptions {
            minimum_beat: cli.minimum_beat,
        }
    }
}
