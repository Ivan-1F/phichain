use crate::layout::ui_state::UiState;
use crate::layout::Layout;
use crate::notification::{ToastsExt, ToastsStorage};
use bevy::prelude::{Event, ResMut, Trigger};

#[derive(Debug, Clone, Event)]
pub struct ApplyLayout(pub Layout);

pub fn apply_layout_observer(
    event: On<ApplyLayout>,
    mut ui_state: ResMut<UiState>,
    mut toasts: ResMut<ToastsStorage>,
) -> bevy::prelude::Result<()> {
    ui_state.state = event.0.clone();

    toasts.success(t!("menu_bar.layout.messages.applied"));

    Ok(())
}
