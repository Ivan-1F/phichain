use std::borrow::Cow;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use phichain_assets::{
    apply_respack, builtin_respack_dir, load_respack, load_respack_meta, load_respack_preview,
    LoadedRespackPreview, RespackMeta,
};
use serde::{Deserialize, Serialize};

use crate::misc::WorkingDirectory;
use crate::notification::{ToastsExt, ToastsStorage};
use crate::settings::EditorSettings;

pub struct RespackPlugin;

impl Plugin for RespackPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_reload_respack)
            .add_observer(handle_select_respack)
            .add_systems(Startup, reload_saved_pack_on_startup);
    }
}

/// Re-read the active resource pack from editor settings and apply it to the world.
///
/// Leaves `EditorSettings.game.respack` untouched; use [`SelectRespack`] to switch packs.
#[derive(Event, Debug, Default)]
pub struct ReloadRespack;

/// Request switching to a different resource pack.
///
/// The setting is only persisted after the new pack loads successfully; on failure
/// the setting reverts to its previous value so the UI never drifts from the
/// actually-loaded pack.
#[derive(Event, Debug)]
pub struct SelectRespack(pub RespackSource);

/// Identifies a resource pack.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RespackSource {
    #[default]
    Builtin,
    Custom(PathBuf),
}

impl RespackSource {
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Filesystem path to the pack.
    pub fn path(&self) -> Cow<'_, Path> {
        match self {
            Self::Builtin => Cow::Owned(builtin_respack_dir()),
            Self::Custom(path) => Cow::Borrowed(path),
        }
    }
}

fn reload_saved_pack_on_startup(world: &mut World) {
    let is_custom = world
        .resource::<Persistent<EditorSettings>>()
        .game
        .respack
        .is_custom();
    if !is_custom {
        return;
    }
    if let Err(err) = load_and_apply(world) {
        error!("Resource pack load failed: {err:#}");
        let mut settings = world.resource_mut::<Persistent<EditorSettings>>();
        settings.game.respack = RespackSource::Builtin;
        let _ = settings.persist();
    }
}

fn handle_reload_respack(_event: On<ReloadRespack>, mut commands: Commands) {
    commands.queue(move |world: &mut World| match load_and_apply(world) {
        Ok(name) => {
            toast(world, |t| {
                t.success(t!("respack.load.succeed", name = name))
            });
        }
        Err(err) => {
            error!("Resource pack load failed: {err:#}");
            toast(world, |t| {
                t.error(t!("respack.load.failed", error = format!("{err:#}")))
            });
        }
    });
}

fn handle_select_respack(event: On<SelectRespack>, mut commands: Commands) {
    let target = event.0.clone();
    commands.queue(move |world: &mut World| {
        let previous = world
            .resource::<Persistent<EditorSettings>>()
            .game
            .respack
            .clone();
        if previous == target {
            return;
        }

        // temporally apply the new selection, then try to load it.
        world
            .resource_mut::<Persistent<EditorSettings>>()
            .game
            .respack = target;

        match load_and_apply(world) {
            Ok(name) => {
                let _ = world.resource_mut::<Persistent<EditorSettings>>().persist();
                toast(world, |t| {
                    t.success(t!("respack.load.succeed", name = name))
                });
            }
            Err(err) => {
                error!("Resource pack load failed: {err:#}");
                {
                    let mut settings = world.resource_mut::<Persistent<EditorSettings>>();
                    settings.game.respack = previous;
                    let _ = settings.persist();
                }
                toast(world, |t| {
                    t.error(t!("respack.load.failed", error = format!("{err:#}")))
                });
            }
        }
    });
}

/// Load and apply the selected pack, returning its localized display name.
fn load_and_apply(world: &mut World) -> Result<String> {
    let source = world
        .resource::<Persistent<EditorSettings>>()
        .game
        .respack
        .clone();
    let path = source.path();
    let pack = load_respack(&path).with_context(|| format!("loading {}", path.display()))?;
    let name = pack.meta.name.get(&rust_i18n::locale()).to_owned();
    apply_respack(pack, world)?;
    Ok(name)
}

fn toast(world: &mut World, f: impl FnOnce(&mut ToastsStorage)) {
    if let Some(mut toasts) = world.get_resource_mut::<ToastsStorage>() {
        f(&mut toasts);
    }
}

pub struct RespackEntry {
    pub source: RespackSource,
    pub meta: RespackMeta,
    pub preview: LoadedRespackPreview,
}

impl RespackEntry {
    /// Load a single pack's meta and preview images from disk.
    pub fn load(source: RespackSource) -> Result<Self> {
        let path = source.path();
        let meta = load_respack_meta(&path)
            .with_context(|| format!("loading meta for {}", path.display()))?;
        let preview = load_respack_preview(&path)
            .with_context(|| format!("loading preview for {}", path.display()))?;
        Ok(Self {
            source,
            meta,
            preview,
        })
    }
}

/// Scan the respacks directory for packs that decode successfully.
/// Failed packs are skipped with a warning. Results are sorted by path.
pub fn scan_respacks(working_dir: &WorkingDirectory) -> Vec<RespackEntry> {
    let Ok(dir) = working_dir.respacks() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut packs: Vec<RespackEntry> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|pack_path| {
            pack_path.is_dir() || pack_path.extension().is_some_and(|ext| ext == "zip")
        })
        .filter_map(|pack_path| {
            RespackEntry::load(RespackSource::Custom(pack_path.clone()))
                .inspect_err(|err| warn!("skipping respack {}: {err:#}", pack_path.display()))
                .ok()
        })
        .collect();
    packs.sort_by(|a, b| a.source.path().cmp(&b.source.path()));
    packs
}
