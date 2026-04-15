use crate::layout::LayoutPresetManager;
use crate::notification::{ToastsExt, ToastsStorage};
use bevy::prelude::{Event, On, ResMut};
use bevy_persistent::Persistent;
use std::ops::IndexMut;

#[derive(Debug, Clone, Event)]
pub struct RenameLayout {
    pub index: usize,
    pub name: String,
}

pub fn rename_layout_observer(
    event: On<RenameLayout>,
    mut manager: ResMut<Persistent<LayoutPresetManager>>,
    mut toasts: ResMut<ToastsStorage>,
) -> bevy::prelude::Result<()> {
    manager.presets.index_mut(event.index).name = event.name.clone();

    manager.persist()?;

    toasts.success(t!("menu_bar.layout.messages.rename"));

    Ok(())
}
