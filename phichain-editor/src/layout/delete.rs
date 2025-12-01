use crate::layout::LayoutPresetManager;
use crate::notification::{ToastsExt, ToastsStorage};
use bevy::prelude::{Event, ResMut, Trigger};
use bevy_persistent::Persistent;

#[derive(Debug, Clone, Event)]
pub struct DeleteLayout(pub usize);

pub fn delete_layout_observer(
    trigger: Trigger<DeleteLayout>,
    mut manager: ResMut<Persistent<LayoutPresetManager>>,
    mut toasts: ResMut<ToastsStorage>,
) -> bevy::prelude::Result<()> {
    manager.presets.remove(trigger.0);

    toasts.success(t!("menu_bar.layout.messages.delete"));

    Ok(())
}
