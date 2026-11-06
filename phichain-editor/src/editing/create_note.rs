use bevy::prelude::*;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::note::{Note, NoteKind};

use crate::constants::CANVAS_WIDTH;
use crate::editing::history::Edits;
use crate::hotkey::{Hotkey, HotkeyContext, HotkeyExt};
use crate::identifier::{Identifier, IntoIdentifier};
use crate::schedule::EditorSet;
use crate::timeline::container::TimelineLayout;
use crate::timeline::note::NoteTimeline;
use crate::timeline::pending::{spawn_pending, ActivePending};
use crate::timeline::pending_status::{PendingHint, PendingStatus};
use crate::timeline::TimelineContext;
use crate::utils::convert::BevyEguiConvert;

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
        app.add_systems(Update, create_note_system.in_set(EditorSet::Edit))
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

    layout: TimelineLayout,
    note_timelines: Query<(), With<NoteTimeline>>,

    window_query: Query<&Window>,
    bpm_list: Res<BpmList>,

    mut edits: Edits,

    active_pending: ActivePending,
    notes: Query<&Note>,
) -> Result {
    let Ok(window) = window_query.single() else {
        return Ok(());
    };
    let Some(cursor_position) = window.cursor_position() else {
        return Ok(());
    };

    let Some(item) = layout
        .at(cursor_position.into_egui().to_pos2())
        .filter(|item| note_timelines.contains(item.timeline))
    else {
        return Ok(());
    };
    let Some(line_entity) = item.target else {
        return Ok(());
    };
    let viewport = item.viewport;

    let pending_note = active_pending
        .on(item.timeline)
        .and_then(|entity| Some((notes.get(entity).ok()?, entity)));

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

    if let Some((pending_note, entity)) = pending_note {
        let mut pending_note = *pending_note;
        if let NoteKind::Hold { .. } = pending_note.kind {
            let (x, beat) = calc_note_attrs();
            pending_note.kind = NoteKind::Hold {
                hold_beat: (beat - pending_note.beat).max(ctx.settings.minimum_beat()),
            };
            pending_note.x = x * CANVAS_WIDTH;
            // The preview may be despawned by a cascade applied before this command
            commands.entity(entity).try_insert(pending_note);
        }
    }

    if hotkey.just_pressed(CreateNoteHotkeys::PlaceFlick) {
        create_note(NoteKind::Flick);
    }

    if hotkey.just_pressed(CreateNoteHotkeys::PlaceHold) {
        if let Some((pending_note, entity)) = pending_note {
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
            let status = PendingStatus {
                title: t!("tab.timeline.pending.hold").into(),
                hints: vec![PendingHint::key(
                    hotkey.label(CreateNoteHotkeys::PlaceHold),
                    t!("tab.timeline.pending.confirm"),
                )],
            };
            spawn_pending(
                &mut commands,
                item.timeline,
                status,
                (
                    Note::new(
                        NoteKind::Hold {
                            hold_beat: Beat::ONE,
                        },
                        true,
                        beat,
                        x * CANVAS_WIDTH,
                        1.0,
                    ),
                    ChildOf(line_entity),
                ),
            );
        }
    }

    Ok(())
}
