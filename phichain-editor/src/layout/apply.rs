use crate::layout::Layout;
use crate::notification::{ToastsExt, ToastsStorage};
use crate::UiState;
use bevy::prelude::{Event, ResMut, Trigger};

#[derive(Debug, Clone, Event)]
pub struct ApplyLayout(pub Layout);

pub fn apply_layout_observer(
    trigger: Trigger<ApplyLayout>,
    mut ui_state: ResMut<UiState>,
    mut toasts: ResMut<ToastsStorage>,
) -> bevy::prelude::Result<()> {
    ui_state.state = trigger.0.clone();

    toasts.success(t!("menu_bar.layout.messages.applied"));

    Ok(())
}
