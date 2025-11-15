use o2o::o2o;
use phichain_chart::beat::Beat;
use phichain_format::official::from_phichain::OfficialOutputOptions;
use phichain_format::official::OfficialInputOptions;
use phichain_format::rpe::schema::RpeInputOptions;
use rust_i18n::t;
use phichain_format::CommonOutputOptions;

/// CLI wrapper for OfficialInputOptions
#[derive(Debug, Clone, clap::Args, o2o)]
#[map(OfficialInputOptions)]
pub struct CliOfficialInputOptions {
    #[arg(long, default_value_t = false, help = t!("cli.official_input.no_easing_fitting").to_string())]
    #[map(easing_fitting, !~)]
    no_easing_fitting: bool,

    #[arg(long, default_value_t = 0.1, help = t!("cli.official_input.easing_fitting_epsilon").to_string())]
    easing_fitting_epsilon: f32,

    #[arg(long, value_parser = clap::value_parser!(Beat), default_value = "1/4", help = t!("cli.official_input.constant_event_shrink_to").to_string())]
    constant_event_shrink_to: Beat,
}

/// CLI wrapper for OfficialInputOptions
#[derive(Debug, Clone, clap::Args, o2o)]
#[map(OfficialOutputOptions)]
pub struct CliOfficialOutputOptions {
    #[arg(long, value_parser = clap::value_parser!(Beat), default_value = "1/32", help = t!("cli.official_output.minimum_beat").to_string())]
    minimum_beat: Beat,
}

// CLI wrapper for RpeInputOptions
#[derive(Debug, Clone, clap::Args, o2o)]
#[map(RpeInputOptions)]
pub struct CliRpeInputOptions {
    #[arg(long, help = t!("cli.rpe_input.remove_fake_notes").to_string())]
    remove_fake_notes: bool,
    #[arg(long, help = t!("cli.rpe_input.remove_ui_controls").to_string())]
    remove_ui_controls: bool,
}

// CLI wrapper for CommonOutputOptions
#[derive(Debug, Clone, clap::Args, o2o)]
#[map(CommonOutputOptions)]
pub struct CliCommonOutputOptions {
    #[arg(long, help = t!("cli.common_output.round").to_string(), default_value_t = 2)]
    round: u32,
}
