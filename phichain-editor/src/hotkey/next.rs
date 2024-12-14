use crate::hotkey::modifier::{Modifier, AVAILABLE_MODIFIERS};
use crate::hotkey::record::{RecordHotkeyPlugin, RecordingHotkey};
use crate::identifier::{Identifier, IntoIdentifier};
use crate::misc::WorkingDirectory;
use bevy::app::App;
use bevy::ecs::system::SystemParam;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::utils::HashMap;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::fmt::Display;
use std::fs::File;
use std::path::Path;
use std::{fs, iter};

pub enum EditorHotkeys {
    SaveProject,
    CloseProject,

    PauseResume,
    Forward,
    Backward,

    Undo,
    Redo,
}

impl IntoIdentifier for EditorHotkeys {
    fn into_identifier(self) -> Identifier {
        match self {
            EditorHotkeys::SaveProject => "phichain.save_project".into(),
            EditorHotkeys::CloseProject => "phichain.close_project".into(),
            EditorHotkeys::PauseResume => "phichain.pause_resume".into(),
            EditorHotkeys::Forward => "phichain.forward".into(),
            EditorHotkeys::Backward => "phichain.backward".into(),
            EditorHotkeys::Undo => "phichain.undo".into(),
            EditorHotkeys::Redo => "phichain.redo".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hotkey {
    pub key: KeyCode,
    pub modifiers: Vec<Modifier>,
}

impl Hotkey {
    pub fn new(key: KeyCode, modifiers: Vec<Modifier>) -> Self {
        Self { key, modifiers }
    }

    fn modifiers_pressed(&self, input: &ButtonInput<KeyCode>) -> bool {
        AVAILABLE_MODIFIERS.iter().all(|modifier| {
            let modifier_pressed = modifier
                .get_key_codes()
                .iter()
                .any(|key_code| input.pressed(*key_code));
            self.modifiers.contains(modifier) == modifier_pressed
        })
    }

    pub fn just_pressed(&self, input: &ButtonInput<KeyCode>) -> bool {
        self.modifiers_pressed(input) && input.just_pressed(self.key)
    }
}

impl Display for Hotkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key_name = format!("{:?}", self.key);
        let content = AVAILABLE_MODIFIERS
            .iter()
            .filter(|modifier| self.modifiers.contains(modifier))
            .map(|modifier| modifier.to_string())
            .chain(iter::once(key_name))
            .collect::<Vec<String>>()
            .join(" + ");

        write!(f, "{}", content)
    }
}

#[derive(Debug, Clone, Default, Resource)]
pub struct HotkeyState(IndexMap<Identifier, Hotkey>);

impl HotkeyState {
    pub fn get(&self, hotkey: impl IntoIdentifier) -> Option<Hotkey> {
        self.0.get(&hotkey.into_identifier()).cloned()
    }

    pub fn set(&mut self, id: impl IntoIdentifier, hotkey: Hotkey) {
        self.0.insert(id.into_identifier(), hotkey);
    }

    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        fs::write(
            path,
            serde_yaml::to_string(
                &self
                    .0
                    .iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect::<HashMap<_, _>>(),
            )
            .expect("Failed to serialize hotkey config"),
        )
    }
}

/// Holds the default value for all the possible hotkeys
#[derive(Debug, Clone, Default, Resource)]
pub struct HotkeyRegistry(pub IndexMap<Identifier, Hotkey>); // id -> default

pub trait HotkeyExt {
    fn add_hotkey(&mut self, id: impl IntoIdentifier, default: Hotkey) -> &mut Self;
}

impl HotkeyExt for App {
    fn add_hotkey(&mut self, id: impl IntoIdentifier, default: Hotkey) -> &mut Self {
        self.world
            .resource_mut::<HotkeyRegistry>()
            .0
            .insert(id.into_identifier(), default);

        self
    }
}

pub struct HotkeyPlugin;

impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HotkeyRegistry>()
            .init_resource::<HotkeyState>()
            .add_hotkey(
                EditorHotkeys::SaveProject,
                Hotkey::new(KeyCode::KeyS, vec![Modifier::Control]),
            )
            .add_hotkey(
                EditorHotkeys::CloseProject,
                Hotkey::new(KeyCode::KeyW, vec![Modifier::Control]),
            )
            .add_hotkey(
                EditorHotkeys::PauseResume,
                Hotkey::new(KeyCode::Space, vec![]),
            )
            .add_hotkey(
                EditorHotkeys::Forward,
                Hotkey::new(KeyCode::BracketLeft, vec![]),
            )
            .add_hotkey(
                EditorHotkeys::Backward,
                Hotkey::new(KeyCode::BracketRight, vec![]),
            )
            .add_hotkey(
                EditorHotkeys::Undo,
                Hotkey::new(KeyCode::KeyZ, vec![Modifier::Control]),
            )
            .add_hotkey(
                EditorHotkeys::Redo,
                Hotkey::new(KeyCode::KeyZ, vec![Modifier::Control, Modifier::Shift]),
            )
            .add_systems(Startup, load_hotkey_settings_system)
            .add_plugins(RecordHotkeyPlugin);
    }
}

fn parse_hotkey_config(
    value: Value,
    registry: Res<HotkeyRegistry>,
) -> IndexMap<Identifier, Hotkey> {
    if let Some(mapping) = value.as_mapping() {
        let mut result: IndexMap<Identifier, Hotkey> = IndexMap::new();

        for (id, default) in &registry.0 {
            if let Some(hotkey) = mapping
                .get(id.to_string())
                .and_then(|x| serde_yaml::from_value::<Hotkey>(x.clone()).ok())
            {
                result.insert(id.clone(), hotkey);
            } else {
                warn!(
                    "Missing hotkey config for {}, using default value ({})",
                    id, default
                );
                result.insert(id.clone(), default.clone());
            }
        }

        result
    } else {
        registry.0.clone()
    }
}

fn load_hotkey_settings_system(
    working_dir: Res<WorkingDirectory>,
    registry: Res<HotkeyRegistry>,
    mut state: ResMut<HotkeyState>,
) {
    let config_path = working_dir
        .config()
        .expect("Failed to locate config directory")
        .join("hotkey.yml");

    if let Ok(file) = File::open(&config_path) {
        if let Ok(data) = serde_yaml::from_reader::<File, Value>(file) {
            state.0 = parse_hotkey_config(data, registry);
        } else {
            state.0 = registry.0.clone();
        }
    } else {
        // no hotkey config exist, use default values from registry
        state.0 = registry.0.clone();
    }

    // write the fixed config back
    let _ = state.save_to(config_path);
}

#[derive(SystemParam)]
pub struct HotkeyContext<'w, 's> {
    state: Res<'w, HotkeyState>,
    input: Res<'w, ButtonInput<KeyCode>>,

    query: Query<'w, 's, &'static RecordingHotkey>,
}

impl HotkeyContext<'_, '_> {
    pub fn just_pressed(&self, hotkey: impl IntoIdentifier) -> bool {
        if self.query.is_empty() {
            self.state
                .get(hotkey)
                .map(|x| x.just_pressed(&self.input))
                .unwrap_or(false)
        } else {
            // disable all hotkey while recording hotkey
            false
        }
    }
}
