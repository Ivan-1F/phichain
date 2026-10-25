use crate::editing::history::Edits;
use crate::selection::Selected;
use bevy::prelude::*;
use egui::{Align, Layout, Ui};
use phichain_chart::event::{LineEvent, LineEventKind};

pub fn multiple_events_inspector(
    In(mut ui): In<Ui>,
    query: Query<(&LineEvent, Entity), With<Selected>>,
    mut edits: Edits,
) -> Result {
    ui.label(t!(
        "tab.inspector.multiple_events.title",
        amount = query.iter().len()
    ));
    ui.separator();

    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
        if ui
            .button(t!("tab.inspector.multiple_events.negate"))
            .clicked()
        {
            let events: Vec<_> = query
                .iter()
                .filter(|(event, _)| event.kind != LineEventKind::Opacity)
                .map(|(event, entity)| {
                    (
                        entity,
                        LineEvent {
                            value: event.value.negated(),
                            ..*event
                        },
                    )
                })
                .collect();
            edits.once(
                t!("history.edit_events", count = events.len()),
                move |commands| {
                    for (entity, event) in events {
                        commands.entity(entity).insert(event);
                    }
                },
            );
        }
    });
    Ok(())
}
