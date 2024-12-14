use crate::hotkey::modifier::Modifier;
use crate::hotkey::next::{Hotkey, HotkeyState};
use crate::identifier::{Identifier, IntoIdentifier};
use bevy::app::{App, Plugin};
use bevy::input::ButtonInput;
use bevy::prelude::{
    Commands, Component, Entity, IntoSystemConfigs, KeyCode, Query, Res, ResMut, Update,
};
use phichain_game::GameSet;

#[derive(Debug, Clone, Component)]
pub struct RecordingHotkey {
    pub id: Identifier,

    pub modifiers: Vec<Modifier>,
    pub key: Option<KeyCode>,
}

impl RecordingHotkey {
    pub fn new(id: impl IntoIdentifier) -> Self {
        Self {
            id: id.into_identifier(),
            modifiers: vec![],
            key: None,
        }
    }
}

impl RecordingHotkey {
    fn accept_modifiers(&self) -> bool {
        self.key.is_none()
    }

    pub fn push(&mut self, key: KeyCode) {
        if let Some(modifier) = Modifier::from_key_code(&key) {
            if self.accept_modifiers() && !self.modifiers.contains(&modifier) {
                self.modifiers.push(modifier);
            }
        } else {
            self.key.replace(key);
        }
    }

    pub fn hotkey(&self) -> Option<Hotkey> {
        self.key.map(|key| Hotkey::new(key, self.modifiers.clone()))
    }
}

pub struct RecordHotkeyPlugin;

impl Plugin for RecordHotkeyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, record_hotkey_system.in_set(GameSet));
    }
}

fn record_hotkey_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut RecordingHotkey, Entity)>,
    mut state: ResMut<HotkeyState>,
) {
    if let Ok((mut recording, entity)) = query.get_single_mut() {
        for key in keyboard.get_just_pressed() {
            recording.push(*key);
            if let Some(hotkey) = recording.hotkey() {
                state.set(recording.id.clone(), hotkey);
                // TODO: save to file
                commands.entity(entity).despawn();
            }
        }
    }
}
