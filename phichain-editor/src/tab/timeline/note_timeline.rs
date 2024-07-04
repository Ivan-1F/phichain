use crate::constants::CANVAS_WIDTH;
use crate::editing::pending::Pending;
use crate::selection::{SelectEvent, Selected, SelectedLine};
use crate::tab::timeline::TimelineViewport;
use crate::timeline::TimelineContext;
use bevy::hierarchy::Parent;
use bevy::prelude::*;
use egui::{Color32, Sense, Stroke, Ui};
use phichain_chart::bpm_list::BpmList;
use phichain_chart::note::Note;

pub struct NoteTimelinePlugin;

impl Plugin for NoteTimelinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoteTimelineDragSelection>();
    }
}

/// Represents the drag-selection on the note timeline
#[derive(Resource, Debug, Default)]
pub struct NoteTimelineDragSelection(pub Option<(egui::Vec2, egui::Vec2)>);

#[allow(dead_code)]
pub fn note_timeline_drag_select_system(
    In(ui): In<&mut Ui>,
    viewport: Res<TimelineViewport>,
    bpm_list: Res<BpmList>,
    note_query: Query<(&Note, &Parent, Entity, Option<&Selected>, Option<&Pending>)>,
    mut select_events: EventWriter<SelectEvent>,
    timeline: TimelineContext,

    mut selection: ResMut<NoteTimelineDragSelection>,
    window_query: Query<&Window>,

    selected_line: Res<SelectedLine>,
) {
    let note_timeline_viewport = viewport.note_timeline_viewport();
    let window = window_query.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let calc_note_attrs = || {
        let time = timeline.y_to_time(cursor_position.y);
        let x = (cursor_position.x - note_timeline_viewport.min.x) / note_timeline_viewport.width()
            - 0.5;
        (x, time)
    };

    let response = ui.allocate_rect(
        egui::Rect::from_min_max(
            egui::Pos2::new(note_timeline_viewport.min.x, note_timeline_viewport.min.y),
            egui::Pos2::new(note_timeline_viewport.max.x, note_timeline_viewport.max.y),
        ),
        Sense::drag(),
    );

    if response.drag_started() {
        let (x, time) = calc_note_attrs();
        selection.0 = Some((egui::Vec2::new(x, time), egui::Vec2::new(x, time)));
    }

    if response.dragged() {
        let (x, time) = calc_note_attrs();
        selection.0 = Some((selection.0.unwrap().0, egui::Vec2::new(x, time)));
    }

    if let Some((start, now)) = selection.0 {
        let start_x =
            note_timeline_viewport.min.x + (start.x + 0.5) * note_timeline_viewport.width();
        let start_y = timeline.time_to_y(start.y);
        let now_x = note_timeline_viewport.min.x + (now.x + 0.5) * note_timeline_viewport.width();
        let now_y = timeline.time_to_y(now.y);
        ui.painter().rect(
            egui::Rect::from_two_pos(
                egui::Pos2::new(start_x, start_y),
                egui::Pos2::new(now_x, now_y),
            ),
            0.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 20),
            Stroke::NONE,
        );
    }

    if response.drag_stopped() {
        if let Some((from, to)) = selection.0 {
            let rect = egui::Rect::from_two_pos(
                (from * egui::Vec2::new(CANVAS_WIDTH, 1.0)).to_pos2(),
                (to * egui::Vec2::new(CANVAS_WIDTH, 1.0)).to_pos2(),
            );
            // ignore too small selections. e.g. click on a note
            if rect.area() >= 0.001 {
                let x_range = rect.x_range();
                let time_range = rect.y_range();

                let notes = note_query
                    .iter()
                    .filter(|x| x.1.get() == selected_line.0)
                    .filter(|x| {
                        let note = x.0;
                        x_range.contains(note.x) && time_range.contains(bpm_list.time_at(note.beat))
                    })
                    .map(|x| x.2)
                    .collect::<Vec<_>>();

                select_events.send(SelectEvent(notes));
            }
        }
        selection.0 = None;
    }
}
