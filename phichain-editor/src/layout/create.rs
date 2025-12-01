use crate::layout::{LayoutPreset, LayoutPresetManager};
use crate::notification::{ToastsExt, ToastsStorage};
use crate::UiState;
use bevy::prelude::{Event, Res, ResMut, Trigger};
use bevy_persistent::Persistent;

#[derive(Debug, Clone, Event)]
pub struct NewLayout(pub String);

pub fn create_layout_observer(
    trigger: Trigger<NewLayout>,
    mut manager: ResMut<Persistent<LayoutPresetManager>>,
    ui_state: Res<UiState>,
    mut toasts: ResMut<ToastsStorage>,
) -> bevy::prelude::Result<()> {
    manager.presets.push(LayoutPreset {
        name: trigger.0.clone(),
        layout: ui_state.state.clone(),
    });

    manager.persist()?;

    toasts.success(t!("menu_bar.layout.messages.created"));

    Ok(())
}
