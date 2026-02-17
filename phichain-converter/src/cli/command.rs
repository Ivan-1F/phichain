use clap::{command, value_parser, Arg, ArgAction, Command, ValueEnum};
use phichain_chart::beat::Beat;
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
enum Format {
    Official,
    Phichain,
    Rpe,
}

pub fn command() -> Command {
    command!()
        .next_help_heading("Global options")
        .arg(
            Arg::new("overwrite")
                .short('y')
                .action(ArgAction::SetTrue)
                .global(true)
                .help("Overwrite output file without asking"),
        )
        .next_help_heading("Input / Output")
        .args([
            Arg::new("input")
                .short('i')
                .action(ArgAction::Append)
                .required(true)
                .value_parser(value_parser!(PathBuf))
                .help("Input file"),
            Arg::new("output")
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf))
                .help("Output file"),
        ])
        .next_help_heading("Per-file options - Common")
        .arg(
            Arg::new("format")
                .short('f')
                .action(ArgAction::Append)
                .value_parser(value_parser!(Format))
                .help("Set format"),
        )
        .next_help_heading("Output options - Common")
        .arg(
            Arg::new("round")
                .long("round")
                .action(ArgAction::Append)
                .value_parser(value_parser!(u32))
                .default_value("2")
                .help("Number of decimal places to round output values (event values and note x positions)"),
        )
        // Official input options
        .next_help_heading("Input options - Official")
        .args([
            Arg::new("no_easing_fitting")
                .long("no-easing-fitting")
                .action(ArgAction::Append)
                .num_args(0..=1)
                .default_missing_value("true")
                .help("Disable easing fitting"),
            Arg::new("easing_fitting_epsilon")
                .long("easing-fitting-epsilon")
                .action(ArgAction::Append)
                .value_parser(value_parser!(f32))
                .default_value("0.1")
                .help("The epsilon used during easing fitting"),
            Arg::new("constant_event_shrink_to")
                .long("constant-event-shrink-to")
                .action(ArgAction::Append)
                .value_parser(value_parser!(Beat))
                .default_value("1/4")
                .help("For constant events, how long to shrink them to"),
        ])
        // Official output options
        .next_help_heading("Output options - Official")
        .arg(
            Arg::new("minimum_beat")
                .long("minimum-beat")
                .action(ArgAction::Append)
                .value_parser(value_parser!(Beat))
                .default_value("1/32")
                .help("The minimum beat used for event cutting"),
        )
        // RPE input options
        .next_help_heading("Input options - RPE")
        .args([
            Arg::new("remove_fake_notes")
                .long("remove-fake-notes")
                .action(ArgAction::Append)
                .num_args(0..=1)
                .default_missing_value("true")
                .help("Remove notes with isFake = true"),
            Arg::new("remove_ui_controls")
                .long("remove-ui-controls")
                .action(ArgAction::Append)
                .num_args(0..=1)
                .default_missing_value("true")
                .help("Remove lines with non-empty attachUI"),
        ])
}
