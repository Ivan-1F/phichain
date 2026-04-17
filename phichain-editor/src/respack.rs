use std::path::PathBuf;

use anyhow::{Context, Result};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use phichain_assets::{apply_respack, builtin_respack_dir, load_respack, load_respack_from_dir};

use crate::misc::WorkingDirectory;
use crate::notification::{ToastsExt, ToastsStorage};
use crate::settings::EditorSettings;

pub struct RespackPlugin;

impl Plugin for RespackPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_reload_respack)
            .add_systems(Startup, trigger_reload_on_startup);
    }
}

/// Re-read the active resource pack from editor settings and apply it to the world.
///
/// `EditorSettings.game.respack` is the single source of truth:
/// - `None` → built-in pack
/// - `Some(name)` → external pack under `<working_dir>/respacks/<name>`
///
/// Callers that want to switch packs update the setting first, then trigger this event.
#[derive(Event, Debug)]
pub struct ReloadRespack;

fn trigger_reload_on_startup(
    settings: Res<Persistent<EditorSettings>>,
    mut commands: Commands,
) {
    // The built-in pack is already active (loaded by `AssetsPlugin::build`);
    // only trigger a reload when the user has selected a custom pack.
    if settings.game.respack.is_some() {
        commands.trigger(ReloadRespack);
    }
}

/// Apply the active pack. Defers to a queued closure because applying needs
/// exclusive `&mut World` access, which observers can't hold directly.
fn handle_reload_respack(_: On<ReloadRespack>, mut commands: Commands) {
    commands.queue(|world: &mut World| apply(world));
}

fn apply(world: &mut World) {
    match load_and_apply(world) {
        Ok(label) => {
            toast(world, |t| t.success(format!("Loaded resource pack: {label}")));
        }
        Err(err) => {
            error!("Resource pack load failed: {err:#}");
            toast(world, |t| t.error(format!("Failed to load resource pack: {err:#}")));
        }
    }
}

fn load_and_apply(world: &mut World) -> Result<String> {
    let selection = world
        .resource::<Persistent<EditorSettings>>()
        .game
        .respack
        .clone();

    match selection {
        Some(name) => {
            let path = resolve_respack_path(&name, world)?;
            let pack = load_respack(&path)
                .with_context(|| format!("loading {}", path.display()))?;
            apply_respack(pack, world)?;
            Ok(name)
        }
        None => {
            let pack = load_respack_from_dir(&builtin_respack_dir())
                .context("loading built-in resource pack")?;
            apply_respack(pack, world)?;
            Ok("built-in".to_owned())
        }
    }
}

fn resolve_respack_path(name: &str, world: &mut World) -> Result<PathBuf> {
    let dir = world
        .resource::<WorkingDirectory>()
        .respacks()
        .context("accessing respacks directory")?;
    Ok(dir.join(name))
}

fn toast(world: &mut World, f: impl FnOnce(&mut ToastsStorage)) {
    if let Some(mut toasts) = world.get_resource_mut::<ToastsStorage>() {
        f(&mut toasts);
    }
}
