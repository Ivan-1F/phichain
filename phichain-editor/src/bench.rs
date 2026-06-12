use bevy::app::AppExit;
use bevy::prelude::*;
use std::time::Instant;

use crate::project::Project;

/// Wall-clock time at which handling of the latest [`crate::project::LoadProject`] began
#[derive(Resource)]
pub struct ProjectLoadStart(pub Instant);

/// Logs the click-to-editable time of project loading.
///
/// When the `PHICHAIN_BENCH_LOAD` environment variable is set to `1`,
/// the time is also printed to stdout as `BENCH_LOAD_MS=<ms>` and the app exits.
/// Combined with the `--project` flag, this allows automated load time benchmarking
pub struct BenchPlugin;

impl Plugin for BenchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Last,
            report_load_time_system.run_if(resource_added::<Project>),
        );
    }
}

fn report_load_time_system(
    start: Option<Res<ProjectLoadStart>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(start) = start else {
        return;
    };

    let millis = start.0.elapsed().as_millis();
    info!("project loaded in {millis}ms (click-to-editable)");

    if std::env::var("PHICHAIN_BENCH_LOAD").is_ok_and(|value| value == "1") {
        println!("BENCH_LOAD_MS={millis}");
        exit.write(AppExit::Success);
    }
}
