use crate::layout::Layout;
use crate::UiState;
use bevy::prelude::{Event, ResMut, Trigger};

#[derive(Debug, Clone, Event)]
pub struct ApplyLayout(pub Layout);

pub fn apply_layout_observer(
    trigger: Trigger<ApplyLayout>,
    mut ui_state: ResMut<UiState>,
) -> bevy::prelude::Result<()> {
    ui_state.state = trigger.0.clone();

    Ok(())
}
