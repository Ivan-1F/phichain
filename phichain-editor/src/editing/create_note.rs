use bevy::prelude::*;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::note::{Note, NoteKind};

use crate::editing::command::note::CreateNote;
use crate::editing::command::EditorCommand;
use crate::editing::pending::Pending;
use crate::editing::DoCommandEvent;
use crate::project::project_loaded;
use crate::timeline::TimelineContext;
use crate::{constants::CANVAS_WIDTH, selection::SelectedLine, tab::timeline::TimelineViewport};
use phichain_chart::note::NoteBundle;

pub struct CreateNoteSystem;

impl Plugin for CreateNoteSystem {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (create_note_system, remove_pending_note_on_esc_system).run_if(project_loaded()),
        );
    }
}

fn create_note_system(
    mut commands: Commands,
    timeline: TimelineContext,
    keyboard: Res<ButtonInput<KeyCode>>,

    selected_line: Res<SelectedLine>,

    window_query: Query<&Window>,
    bpm_list: Res<BpmList>,

    timeline_viewport: Res<TimelineViewport>,

    mut event: EventWriter<DoCommandEvent>,

    mut pending_note_query: Query<(&mut Note, Entity), With<Pending>>,
) {
    let window = window_query.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let note_timeline_viewport = timeline_viewport.note_timeline_viewport();

    if !note_timeline_viewport.contains(cursor_position) {
        return;
    }

    let calc_note_attrs = || {
        let time = timeline.y_to_time(cursor_position.y);
        let beat = bpm_list.beat_at(time).value();
        let beat = timeline.timeline_settings.attach(beat);

        let x = (cursor_position.x - note_timeline_viewport.min.x) / note_timeline_viewport.width();

        let lane_percents = timeline.timeline_settings.lane_percents();

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

        event.send(DoCommandEvent(EditorCommand::CreateNote(CreateNote::new(
            selected_line.0,
            note,
        ))));
    };

    if keyboard.just_pressed(KeyCode::KeyQ) {
        create_note(NoteKind::Tap);
    }

    if keyboard.just_pressed(KeyCode::KeyW) {
        create_note(NoteKind::Drag);
    }

    if let Ok((mut pending_note, _)) = pending_note_query.get_single_mut() {
        if let NoteKind::Hold { .. } = pending_note.kind {
            let (x, beat) = calc_note_attrs();
            pending_note.kind = NoteKind::Hold {
                hold_beat: (beat - pending_note.beat)
                    .max(timeline.timeline_settings.minimum_beat()),
            };
            pending_note.x = x * CANVAS_WIDTH;
        }
    }

    if keyboard.just_pressed(KeyCode::KeyE) {
        create_note(NoteKind::Flick);
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Ok((pending_note, entity)) = pending_note_query.get_single() {
            commands.entity(entity).despawn();
            event.send(DoCommandEvent(EditorCommand::CreateNote(CreateNote::new(
                selected_line.0,
                *pending_note,
            ))));
        } else {
            let (x, beat) = calc_note_attrs();
            commands.entity(selected_line.0).with_children(|parent| {
                parent.spawn((
                    NoteBundle::new(Note::new(
                        NoteKind::Hold {
                            hold_beat: Beat::ONE,
                        },
                        true,
                        beat,
                        x,
                        1.0,
                    )),
                    Pending,
                ));
            });
        }
    }
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
