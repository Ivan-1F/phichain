use crate::layout::LayoutPresetManager;
use crate::notification::{ToastsExt, ToastsStorage};
use bevy::prelude::{Event, On, ResMut};
use bevy_persistent::Persistent;

#[derive(Debug, Clone, Event)]
pub struct DeleteLayout(pub usize);

pub fn delete_layout_observer(
    event: On<DeleteLayout>,
    mut manager: ResMut<Persistent<LayoutPresetManager>>,
    mut toasts: ResMut<ToastsStorage>,
) -> bevy::prelude::Result<()> {
    manager.presets.remove(event.0);

    manager.persist()?;

    toasts.success(t!("menu_bar.layout.messages.delete"));

    Ok(())
}
