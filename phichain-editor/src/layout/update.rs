use crate::layout::ui_state::UiState;
use crate::layout::LayoutPresetManager;
use crate::notification::{ToastsExt, ToastsStorage};
use bevy::prelude::{Event, Res, ResMut, Trigger};
use bevy_persistent::Persistent;
use std::ops::IndexMut;

#[derive(Debug, Clone, Event)]
pub struct UpdateLayout(pub usize);

pub fn update_layout_observer(
    trigger: Trigger<UpdateLayout>,
    mut manager: ResMut<Persistent<LayoutPresetManager>>,
    ui_state: Res<UiState>,
    mut toasts: ResMut<ToastsStorage>,
) -> bevy::prelude::Result<()> {
    manager.presets.index_mut(trigger.0).layout = ui_state.state.clone();

    manager.persist()?;

    toasts.success(t!("menu_bar.layout.messages.update"));

    Ok(())
}
