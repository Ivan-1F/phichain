use crate::layout::ui_state::UiState;
use crate::layout::{LayoutPreset, LayoutPresetManager};
use crate::notification::{ToastsExt, ToastsStorage};
use bevy::prelude::{Event, On, Res, ResMut};
use bevy_persistent::Persistent;

#[derive(Debug, Clone, Event)]
pub struct NewLayout(pub String);

pub fn create_layout_observer(
    event: On<NewLayout>,
    mut manager: ResMut<Persistent<LayoutPresetManager>>,
    ui_state: Res<UiState>,
    mut toasts: ResMut<ToastsStorage>,
) -> bevy::prelude::Result<()> {
    manager.presets.push(LayoutPreset {
        name: event.0.clone(),
        layout: ui_state.state.clone(),
    });

    manager.persist()?;

    toasts.success(t!("menu_bar.layout.messages.created"));

    Ok(())
}
