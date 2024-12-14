use bevy::prelude::KeyCode;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    Control,
    Shift,
    Alt,
}

impl Modifier {
    #[allow(dead_code)]
    pub fn is(&self, key_code: &KeyCode) -> bool {
        self.get_key_codes().contains(key_code)
    }

    pub fn get_key_codes(&self) -> Vec<KeyCode> {
        match self {
            #[cfg(target_os = "macos")]
            Modifier::Control => vec![KeyCode::SuperLeft, KeyCode::SuperRight],
            #[cfg(not(target_os = "macos"))]
            Modifier::Control => vec![KeyCode::ControlLeft, KeyCode::ControlRight],

            Modifier::Shift => vec![KeyCode::ShiftLeft, KeyCode::ShiftRight],

            Modifier::Alt => vec![KeyCode::AltLeft, KeyCode::AltRight],
        }
    }

    #[allow(dead_code)]
    pub fn from_key_code(key_code: &KeyCode) -> Option<Self> {
        match key_code {
            #[cfg(target_os = "macos")]
            KeyCode::SuperLeft | KeyCode::SuperRight => Some(Self::Control),
            #[cfg(not(target_os = "macos"))]
            KeyCode::ControlLeft | KeyCode::ControlRight => Some(Self::Control),
            KeyCode::ShiftLeft | KeyCode::ShiftRight => Some(Self::Shift),
            KeyCode::AltLeft | KeyCode::AltRight => Some(Self::Alt),
            _ => None,
        }
    }
}

impl Display for Modifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(target_os = "macos")]
            Modifier::Control => write!(f, "Command"),
            #[cfg(not(target_os = "macos"))]
            Modifier::Control => write!(f, "Ctrl"),

            Modifier::Shift => write!(f, "Shift"),

            #[cfg(target_os = "macos")]
            Modifier::Alt => write!(f, "Option"),
            #[cfg(not(target_os = "macos"))]
            Modifier::Alt => write!(f, "Alt"),
        }
    }
}

pub const AVAILABLE_MODIFIERS: &[Modifier] = &[Modifier::Control, Modifier::Shift, Modifier::Alt];
