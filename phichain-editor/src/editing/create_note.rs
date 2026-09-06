use bevy::prelude::*;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::note::{Note, NoteKind};

use crate::editing::history::Edits;
use crate::editing::pending::Pending;
use crate::hotkey::{Hotkey, HotkeyContext, HotkeyExt};
use crate::identifier::{Identifier, IntoIdentifier};
use crate::schedule::EditorSet;
use crate::timeline::{TimelineContext, TimelineItem};
use crate::utils::convert::BevyEguiConvert;
use crate::{constants::CANVAS_WIDTH, selection::SelectedLine};

#[allow(clippy::enum_variant_names)]
enum CreateNoteHotkeys {
    PlaceTap,
    PlaceDrag,
    PlaceFlick,
    PlaceHold,
}

impl IntoIdentifier for CreateNoteHotkeys {
    fn into_identifier(self) -> Identifier {
        match self {
            CreateNoteHotkeys::PlaceTap => "phichain.place_tap".into(),
            CreateNoteHotkeys::PlaceDrag => "phichain.place_drag".into(),
            CreateNoteHotkeys::PlaceFlick => "phichain.place_flick".into(),
            CreateNoteHotkeys::PlaceHold => "phichain.place_hold".into(),
        }
    }
}

pub struct CreateNotePlugin;

impl Plugin for CreateNotePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (create_note_system, remove_pending_note_on_esc_system).in_set(EditorSet::Edit),
        )
        .add_hotkey(
            CreateNoteHotkeys::PlaceTap,
            Hotkey::new(KeyCode::KeyQ, vec![]),
        )
        .add_hotkey(
            CreateNoteHotkeys::PlaceDrag,
            Hotkey::new(KeyCode::KeyW, vec![]),
        )
        .add_hotkey(
            CreateNoteHotkeys::PlaceFlick,
            Hotkey::new(KeyCode::KeyE, vec![]),
        )
        .add_hotkey(
            CreateNoteHotkeys::PlaceHold,
            Hotkey::new(KeyCode::KeyR, vec![]),
        );
    }
}

fn create_note_system(
    mut commands: Commands,
    ctx: TimelineContext,
    hotkey: HotkeyContext,

    selected_line: Res<SelectedLine>,

    window_query: Query<&Window>,
    bpm_list: Res<BpmList>,

    mut edits: Edits,

    pending_note_query: Query<(&Note, Entity), With<Pending>>,
) -> Result {
    let Ok(window) = window_query.single() else {
        return Ok(());
    };
    let Some(cursor_position) = window.cursor_position() else {
        return Ok(());
    };

    let rect = ctx.viewport.0.into_egui();

    for item in &ctx.settings.container.allocate(rect) {
        if let TimelineItem::Note(timeline) = &item.timeline {
            let viewport = item.viewport;
            let line_entity = timeline.line_entity_from_fallback(selected_line.0);

            if !viewport.contains(cursor_position.into_egui().to_pos2()) {
                continue;
            }

            let calc_note_attrs = || {
                let time = ctx.y_to_time(cursor_position.y);
                let beat = bpm_list.beat_at(time).value();
                let beat = ctx.settings.attach(beat);

                let x = (cursor_position.x - viewport.min.x) / viewport.width();

                let lane_percents = ctx.settings.lane_percents();

                let x = lane_percents
                    .iter()
                    .map(|p| (p, (p - x).abs()))
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap()
                    .0;

                let x = x - 0.5;

                (x, beat)
            };

            let mut create_note = |kind: NoteKind| {
                let (x, beat) = calc_note_attrs();

                let note = Note::new(kind, true, beat, x * CANVAS_WIDTH, 1.0);

                edits.once(t!("history.create_notes", count = 1), move |commands| {
                    commands.spawn((note, ChildOf(line_entity)));
                });
            };

            if hotkey.just_pressed(CreateNoteHotkeys::PlaceTap) {
                create_note(NoteKind::Tap);
            }

            if hotkey.just_pressed(CreateNoteHotkeys::PlaceDrag) {
                create_note(NoteKind::Drag);
            }

            if let Ok((pending_note, entity)) = pending_note_query.single() {
                let mut pending_note = *pending_note;
                if let NoteKind::Hold { .. } = pending_note.kind {
                    let (x, beat) = calc_note_attrs();
                    pending_note.kind = NoteKind::Hold {
                        hold_beat: (beat - pending_note.beat).max(ctx.settings.minimum_beat()),
                    };
                    pending_note.x = x * CANVAS_WIDTH;
                    commands.entity(entity).insert(pending_note);
                }
            }

            if hotkey.just_pressed(CreateNoteHotkeys::PlaceFlick) {
                create_note(NoteKind::Flick);
            }

            if hotkey.just_pressed(CreateNoteHotkeys::PlaceHold) {
                if let Ok((pending_note, entity)) = pending_note_query.single() {
                    commands.entity(entity).despawn();
                    // Resolve the final cursor position here too: the preview's
                    // queued replacement has not necessarily executed this frame.
                    let (x, beat) = calc_note_attrs();
                    let mut note = *pending_note;
                    note.kind = NoteKind::Hold {
                        hold_beat: (beat - note.beat).max(ctx.settings.minimum_beat()),
                    };
                    note.x = x * CANVAS_WIDTH;
                    edits.once(t!("history.create_notes", count = 1), move |commands| {
                        commands.spawn((note, ChildOf(line_entity)));
                    });
                } else {
                    let (x, beat) = calc_note_attrs();
                    commands.entity(line_entity).with_children(|parent| {
                        parent.spawn((
                            Note::new(
                                NoteKind::Hold {
                                    hold_beat: Beat::ONE,
                                },
                                true,
                                beat,
                                x * CANVAS_WIDTH,
                                1.0,
                            ),
                            Pending,
                        ));
                    });
                }
            }
        }
    }

    Ok(())
}

fn remove_pending_note_on_esc_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, (With<Pending>, With<Note>)>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }
}
