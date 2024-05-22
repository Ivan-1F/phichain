use crate::assets::ImageAssets;
use crate::chart::note::{Note, NoteKind};
use crate::constants::CANVAS_WIDTH;
use crate::editing::pending::Pending;
use crate::selection::{SelectEvent, Selected, SelectedLine};
use crate::tab::timeline::{Timeline, TimelineSettings, TimelineViewport};
use crate::timing::BpmList;
use bevy::asset::{Assets, Handle};
use bevy::hierarchy::Parent;
use bevy::prelude::{Entity, EventWriter, Image, In, Query, Res};
use bevy_egui::EguiUserTextures;
use egui::{Color32, Ui};

pub fn note_timeline_system(
    In(ui): In<&mut Ui>,
    selected_line_query: Res<SelectedLine>,
    timeline_viewport: Res<TimelineViewport>,
    bpm_list: Res<BpmList>,
    note_query: Query<(&Note, &Parent, Entity, Option<&Selected>, Option<&Pending>)>,
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

    for (note, parent, entity, selected, pending) in note_query.iter() {
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

        let handle = match note.kind {
            NoteKind::Tap => &assets.tap,
            NoteKind::Drag => &assets.drag,
            NoteKind::Hold { .. } => &assets.hold,
            NoteKind::Flick => &assets.flick,
        };

        let (size, image) = get_asset(handle);

        let size = match note.kind {
            NoteKind::Hold { hold_beat } => egui::Vec2::new(
                note_timeline_viewport.width() / 8000.0 * size.x as f32,
                timeline.duration_to_height(bpm_list.time_at(hold_beat)),
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
                .sense(egui::Sense::click()),
        );

        if response.clicked() {
            select_events.send(SelectEvent(entity));
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
            Color32::from_rgba_unmultiplied(255, 255, 255, 40),
        );
    }
}
