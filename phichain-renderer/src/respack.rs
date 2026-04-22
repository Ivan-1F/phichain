//! Optional custom resource pack via `--respack`.

use bevy::prelude::*;
use phichain_assets::{apply_respack, load_respack};
use rust_i18n::t;

use crate::args::Args;

pub struct RespackPlugin;

impl Plugin for RespackPlugin {
    fn build(&self, app: &mut App) {
        let Some(path) = app.world().resource::<Args>().respack.clone() else {
            return;
        };
        let pack = load_respack(&path).unwrap_or_else(|err| {
            eprintln!(
                "error: {}",
                t!(
                    "cli.error.load_respack_failed",
                    path = path.display(),
                    error = format!("{err:#}")
                )
            );
            std::process::exit(1);
        });
        if let Err(err) = apply_respack(pack, app.world_mut()) {
            eprintln!(
                "error: {}",
                t!(
                    "cli.error.apply_respack_failed",
                    path = path.display(),
                    error = format!("{err:#}")
                )
            );
            std::process::exit(1);
        }
        info!(
            "{}",
            t!("cli.status.loaded_respack", path = path.display())
        );
    }
}
