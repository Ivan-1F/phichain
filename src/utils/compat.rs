use bevy::prelude::KeyCode;

pub trait ControlKeyExt {
    fn control() -> Self;
}

impl ControlKeyExt for KeyCode {
    fn control() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self::SuperLeft
        }
        #[cfg(not(target_os = "macos"))]
        {
            Self::ControlLeft
        }
    }
}
