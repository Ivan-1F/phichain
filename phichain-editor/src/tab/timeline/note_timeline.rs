use crate::assets::ImageAssets;
use crate::chart::note::{Note, NoteKind};
use crate::constants::CANVAS_WIDTH;
use crate::editing::pending::Pending;
use crate::highlight::Highlighted;
use crate::selection::{SelectEvent, Selected, SelectedLine};
use crate::tab::timeline::{Timeline, TimelineSettings, TimelineViewport};
use crate::timing::BpmList;
use bevy::asset::{Assets, Handle};
use bevy::hierarchy::Parent;
use bevy::prelude::*;
use bevy_egui::EguiUserTextures;
use egui::{Color32, Sense, Stroke, Ui};

pub struct NoteTimelinePlugin;

impl Plugin for NoteTimelinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoteTimelineDragSelection>();
    }
}

/// Represents the drag-selection on the note timeline
#[derive(Resource, Debug, Default)]
pub struct NoteTimelineDragSelection(pub Option<(egui::Vec2, egui::Vec2)>);

pub fn note_timeline_drag_select_system(
    In(ui): In<&mut Ui>,
    viewport: Res<TimelineViewport>,
    bpm_list: Res<BpmList>,
    note_query: Query<(&Note, &Parent, Entity, Option<&Selected>, Option<&Pending>)>,
    mut select_events: EventWriter<SelectEvent>,
    timeline: Timeline,

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

pub fn note_timeline_system(
    In(ui): In<&mut Ui>,
    selected_line_query: Res<SelectedLine>,
    timeline_viewport: Res<TimelineViewport>,
    bpm_list: Res<BpmList>,
    note_query: Query<(
        &Note,
        &Parent,
        Entity,
        Option<&Highlighted>,
        Option<&Selected>,
        Option<&Pending>,
    )>,
    mut select_events: EventWriter<SelectEvent>,
    timeline: Timeline,
    timeline_settings: Res<TimelineSettings>,
    assets: Res<ImageAssets>,
    images: Res<Assets<Image>>,
    textures: Res<EguiUserTextures>,
) {
    let selected_line = selected_line_query.0;
    let viewport = timeline_viewport;

    let note_timeline_viewport = viewport.note_timeline_viewport();

    for (note, parent, entity, highlighted, selected, pending) in note_query.iter() {
        if parent.get() != selected_line {
            continue;
        }

        let x = note_timeline_viewport.min.x
            + (note.x / CANVAS_WIDTH + 0.5) * note_timeline_viewport.width();
        let y = timeline.time_to_y(bpm_list.time_at(note.beat));

        let get_asset = |handle: &Handle<Image>| {
            (
                images.get(handle).unwrap().size(),
                textures.image_id(handle).unwrap(),
            )
        };

        let handle = match (note.kind, highlighted.is_some()) {
            (NoteKind::Tap, true) => &assets.tap_highlight,
            (NoteKind::Drag, true) => &assets.drag_highlight,
            (NoteKind::Hold { .. }, true) => &assets.hold_highlight,
            (NoteKind::Flick, true) => &assets.flick_highlight,
            (NoteKind::Tap, false) => &assets.tap,
            (NoteKind::Drag, false) => &assets.drag,
            (NoteKind::Hold { .. }, false) => &assets.hold,
            (NoteKind::Flick, false) => &assets.flick,
        };

        let (size, image) = get_asset(handle);

        let size = match note.kind {
            NoteKind::Hold { hold_beat } => egui::Vec2::new(
                note_timeline_viewport.width() / 8000.0 * size.x as f32,
                y - timeline.time_to_y(bpm_list.time_at(note.beat + hold_beat)),
            ),
            _ => egui::Vec2::new(
                note_timeline_viewport.width() / 8000.0 * size.x as f32,
                note_timeline_viewport.width() / 8000.0 * size.y as f32,
            ),
        };

        let center = match note.kind {
            NoteKind::Hold { hold_beat: _ } => egui::Pos2::new(x, y - size.y / 2.0),
            _ => egui::Pos2::new(x, y),
        };

        let mut tint = if selected.is_some() {
            Color32::LIGHT_GREEN
        } else {
            Color32::WHITE
        };

        if pending.is_some() {
            tint = Color32::from_rgba_unmultiplied(tint.r(), tint.g(), tint.b(), 20);
        }

        let response = ui.put(
            egui::Rect::from_center_size(center, size),
            egui::Image::new((image, size))
                .maintain_aspect_ratio(false)
                .fit_to_exact_size(size)
                .tint(tint)
                .sense(Sense::click()),
        );

        if response.clicked() {
            select_events.send(SelectEvent(vec![entity]));
        }
    }

    for percent in timeline_settings.lane_percents() {
        ui.painter().rect_filled(
            egui::Rect::from_center_size(
                egui::Pos2::new(
                    note_timeline_viewport.min.x + note_timeline_viewport.width() * percent,
                    viewport.0.center().y,
                ),
                egui::Vec2::new(2.0, viewport.0.height()),
            ),
            0.0,
            if percent == 0.5 {
                Color32::from_rgba_unmultiplied(0, 255, 0, 40)
            } else {
                Color32::from_rgba_unmultiplied(255, 255, 255, 40)
            },
        );
    }
}
