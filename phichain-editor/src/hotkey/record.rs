use crate::hotkey::modifier::Modifier;
use crate::hotkey::{Hotkey, HotkeyContext};
use crate::identifier::{Identifier, IntoIdentifier};
use bevy::app::{App, Plugin};
use bevy::input::ButtonInput;
use bevy::prelude::{Commands, Component, IntoSystemConfigs, KeyCode, Res, Update};
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

    mut ctx: HotkeyContext,

    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let mut should_save = false;

    if let Ok((mut recording, entity)) = ctx.query.get_single_mut() {
        for key in keyboard.get_just_pressed() {
            recording.push(*key);
            if let Some(hotkey) = recording.hotkey() {
                ctx.state.set(recording.id.clone(), hotkey);

                should_save = true;
                commands.entity(entity).despawn();
            }
        }
    }

    if should_save {
        let _ = ctx.save();
    }
}
