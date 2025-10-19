use phichain_chart::beat::Beat;
use phichain_format::official::OfficialInputOptions;

/// CLI wrapper for OfficialInputOptions
#[derive(Debug, Clone, clap::Args)]
pub struct CliOfficialInputOptions {
    /// Disable easing fitting (enabled by default)
    #[arg(long, default_value_t = false)]
    no_easing_fitting: bool,

    /// The epsilon used during easing fitting
    #[arg(long, default_value_t = 0.1)]
    easing_fitting_epsilon: f32,

    /// For constant events, how long to shrink them to (format: "numer/denom" or "whole+numer/denom" or "whole")
    #[arg(long, value_parser = clap::value_parser!(Beat), default_value = "1/4")]
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
