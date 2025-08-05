use crate::editing::command::event::EditEvent;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use bevy::prelude::*;
use egui::{Align, Layout, Ui};
use phichain_chart::event::{LineEvent, LineEventKind};

pub fn multiple_events_inspector(
    In(mut ui): In<Ui>,
    query: Query<(&LineEvent, Entity)>,
    mut event_writer: EventWriter<DoCommandEvent>,
) -> Result {
    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
        if ui
            .button(t!("tab.inspector.multiple_events.negate"))
            .clicked()
        {
            let commands = query
                .iter()
                .filter(|(event, _)| event.kind != LineEventKind::Opacity)
                .map(|(event, entity)| {
                    EditorCommand::EditEvent(EditEvent::new(
                        entity,
                        *event,
                        LineEvent {
                            value: event.value.negated(),
                            ..*event
                        },
                    ))
                })
                .collect::<Vec<_>>();

            event_writer.write(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(commands),
            )));
        }
    });

    Ok(())
}
