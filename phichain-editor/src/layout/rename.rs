use crate::layout::LayoutPresetManager;
use bevy::prelude::{Event, ResMut, Trigger};
use bevy_persistent::Persistent;
use std::ops::IndexMut;

#[derive(Debug, Clone, Event)]
pub struct RenameLayout {
    pub index: usize,
    pub name: String,
}

pub fn rename_layout_observer(
    trigger: Trigger<RenameLayout>,
    mut manager: ResMut<Persistent<LayoutPresetManager>>,
) -> bevy::prelude::Result<()> {
    manager.presets.index_mut(trigger.index).name = trigger.name.clone();

    manager.persist()?;

    Ok(())
}
