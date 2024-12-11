use crate::editing::command::note::CreateNote;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use bevy::prelude::*;
use num::iter;
use phichain_chart::beat;
use phichain_chart::easing::Easing;
use phichain_chart::note::{Note, NoteKind};
use phichain_game::GameSet;

/// A pending filling notes task
#[derive(Debug, Clone, Component)]
pub struct FillingNotes {
    pub from: Option<Entity>,
    pub to: Option<Entity>,

    pub density: u32,
    pub easing: Easing,
    pub kind: NoteKind,
}

impl FillingNotes {
    pub fn from(entity: Entity) -> Self {
        Self {
            from: Some(entity),
            to: None,

            density: 16,
            easing: Easing::EaseInOutSine,
            kind: NoteKind::Drag,
        }
    }

    pub fn to(&mut self, entity: Entity) {
        self.to = Some(entity);
    }
}

/// Generate a note sequence from a note to another note with a [`FillingNotes`] option
pub fn generate_notes(from: Note, to: Note, options: &FillingNotes) -> Vec<Note> {
    let delta = beat!(1, options.density);
    let beats = iter::range_step(from.beat, to.beat, delta).collect::<Vec<_>>();

    beats
        .iter()
        .enumerate()
        .map(|(i, beat)| {
            let y = i as f32 / beats.len() as f32;
            let x = options.easing.inverse(y).unwrap();

            Note::new(
                options.kind,
                true,
                *beat,
                (from.x - to.x).abs() * x + from.x.min(to.x),
                1.0,
            )
        })
        .skip(1)
        .collect()
}

pub struct FillingNotesPlugin;

impl Plugin for FillingNotesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CancelFillEvent>()
            .add_event::<ConfirmFillEvent>()
            .add_systems(
                Update,
                (
                    handle_cancel_fill_event_system,
                    handle_confirm_fill_event_system,
                )
                    .in_set(GameSet),
            );
    }
}

#[derive(Debug, Default, Event)]
pub struct CancelFillEvent;

#[derive(Debug, Default, Event)]
pub struct ConfirmFillEvent;

fn handle_cancel_fill_event_system(
    mut commands: Commands,
    mut events: EventReader<CancelFillEvent>,
    query: Query<Entity, With<FillingNotes>>,
) {
    for _ in events.read() {
        if let Ok(entity) = query.get_single() {
            commands.entity(entity).despawn();
        } else {
            warn!("received `CancelFillEvent` when no `FillingNotes` present in the world");
        }
    }
}

fn handle_confirm_fill_event_system(
    mut commands: Commands,
    mut events: EventReader<ConfirmFillEvent>,
    query: Query<(&FillingNotes, Entity)>,
    note_query: Query<(&Note, &Parent)>,
    mut do_command: EventWriter<DoCommandEvent>,
) {
    for _ in events.read() {
        if let Ok((filling, entity)) = query.get_single() {
            let Some(from) = filling.from else {
                warn!("received `ConfirmFillEvent` when no an active `FillingNotes` is not complete: missing `from`");
                return;
            };
            let Some(to) = filling.to else {
                warn!("received `ConfirmFillEvent` when no an active `FillingNotes` is not complete: missing `to`");
                return;
            };
            let Ok((from, from_parent)) = note_query.get(from) else {
                warn!("received `ConfirmFillEvent` when no an active `FillingNotes` is not valid: `from` is not valid");
                return;
            };
            let Ok((to, to_parent)) = note_query.get(to) else {
                warn!("received `ConfirmFillEvent` when no an active `FillingNotes` is not valid: `to` is not valid");
                return;
            };

            if from_parent.get() != to_parent.get() {
                warn!("received `ConfirmFillEvent` when no an active `FillingNotes` is not valid: parent lines of `from` and `to` are not identical");
                return;
            }

            let notes = generate_notes(*from, *to, filling);

            let create_note_commands: Vec<_> = notes
                .iter()
                .copied()
                .map(|note| EditorCommand::CreateNote(CreateNote::new(from_parent.get(), note)))
                .collect();

            do_command.send(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(create_note_commands),
            )));

            commands.entity(entity).despawn();
        } else {
            warn!("received `ConfirmFillEvent` when no `FillingNotes` present in the world");
        }
    }
}
