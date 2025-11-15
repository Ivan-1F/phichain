use crate::official::fitting::fit_events;
use crate::official::from_phichain::{
    phichain_to_official, OfficialOutputError, OfficialOutputOptions,
};
use crate::official::schema::{OfficialChart, OfficialNote, OfficialNoteKind};
use crate::ChartFormat;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::event::{LineEvent, LineEventKind};
use phichain_chart::note::{Note, NoteKind};
use phichain_chart::offset::Offset;
use phichain_chart::serialization::{PhichainChart, SerializedLine};
use phichain_chart::{beat, event};
use phichain_compiler::helpers::{are_contiguous, map_if, remove_if};
use phichain_compiler::sequence::EventSequence;
use thiserror::Error;

mod fitting;
pub mod from_phichain;
pub mod schema;

const DEFAULT_EASING_FITTING_EPSILON: f32 = 1e-1;

fn merge_constant_events(events: Vec<LineEvent>) -> Vec<LineEvent> {
    events.into_iter().fold(Vec::new(), |mut merged, event| {
        if let Some(last) = merged.last_mut() {
            if last.value.is_numeric_constant()
                && event.value.is_numeric_constant()
                && are_contiguous(last, &event)
            {
                // extend the previous event instead of adding a new one
                last.end_beat = event.end_beat;
                return merged;
            }
        }
        merged.push(event);
        merged
    })
}

#[derive(Debug, Error)]
pub enum OfficialInputError {
    #[error("expected at leat one line")]
    NoLine,
    #[error("unsupported formatVersion, expected 1 or 3, got {0}")]
    UnsupportedFormatVersion(u32),
}

#[derive(Debug, Clone)]
pub struct OfficialInputOptions {
    /// Enable easing fitting
    pub easing_fitting: bool,
    /// The epsilon used during easing fitting
    pub easing_fitting_epsilon: f32,
    /// For constant events, how long to shrink them to
    pub constant_event_shrink_to: Beat,
}

impl Default for OfficialInputOptions {
    fn default() -> Self {
        Self {
            easing_fitting: true,
            easing_fitting_epsilon: DEFAULT_EASING_FITTING_EPSILON,
            constant_event_shrink_to: beat!(1, 4),
        }
    }
}

impl ChartFormat for OfficialChart {
    type InputOptions = OfficialInputOptions;
    type InputError = OfficialInputError;

    type OutputOptions = OfficialOutputOptions;
    type OutputError = OfficialOutputError;

    fn to_phichain(self, opts: &Self::InputOptions) -> Result<PhichainChart, Self::InputError> {
        official_to_phichain(self, opts)
    }

    fn from_phichain(
        phichain: PhichainChart,
        opts: &Self::OutputOptions,
    ) -> Result<Self, Self::OutputError> {
        phichain_to_official(phichain, opts)
    }
}

pub fn official_to_phichain(
    official: OfficialChart,
    options: &OfficialInputOptions,
) -> Result<PhichainChart, OfficialInputError> {
    if official.lines.is_empty() {
        return Err(OfficialInputError::NoLine);
    }

    if !matches!(official.format_version, 1 | 3) {
        return Err(OfficialInputError::UnsupportedFormatVersion(
            official.format_version,
        ));
    }

    let mut phichain = PhichainChart {
        offset: Offset(official.offset * 1000.0),
        bpm_list: BpmList::single(official.lines[0].bpm),
        ..PhichainChart::empty()
    };

    for line in official.lines {
        let t: fn(f32) -> Beat = |x| Beat::from(x * 1.875 / 60.0);
        let x: fn(f32) -> f32 = |x| (x - 0.5) * CANVAS_WIDTH;
        let y: fn(f32) -> f32 = |x| (x - 0.5) * CANVAS_HEIGHT;

        let create_note = |above: bool, note: &OfficialNote| {
            let kind: NoteKind = match note.kind {
                OfficialNoteKind::Tap => NoteKind::Tap,
                OfficialNoteKind::Drag => NoteKind::Drag,
                OfficialNoteKind::Hold => NoteKind::Hold {
                    hold_beat: t(note.hold_time),
                },
                OfficialNoteKind::Flick => NoteKind::Flick,
            };

            Note::new(
                kind,
                above,
                t(note.time),
                note.x / 18.0 * CANVAS_WIDTH,
                note.speed,
            )
        };

        let move_events = line
            .move_events
            .iter()
            .flat_map(|event| match official.format_version {
                // reference: https://github.com/MisaLiu/phi-chart-render/blob/master/src/chart/convert/official.js#L203
                1 => vec![
                    event!(
                        LineEventKind::X,
                        t(event.start_time) => t(event.end_time),
                        x((event.start_x / 1e3).round() / 880.0) => x((event.end_x / 1e3).round() / 880.0),
                    ),
                    event!(
                        LineEventKind::Y,
                        t(event.start_time) => t(event.end_time),
                        y(event.start_x % 1e3 / 530.0) => y(event.end_x % 1e3 / 530.0),
                    ),
                ],
                3 => vec![
                    event!(
                        LineEventKind::X,
                        t(event.start_time) => t(event.end_time),
                        x(event.start_x) => x(event.end_x),
                    ),
                    event!(
                        LineEventKind::Y,
                        t(event.start_time) => t(event.end_time),
                        y(event.start_y) => y(event.end_y),
                    ),
                ],
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();

        let move_events = [
            merge_constant_events(move_events.x()),
            merge_constant_events(move_events.y()),
        ]
        .concat();

        let rotate_event_iter = line.rotate_events.iter().map(|event| {
            event!(
                LineEventKind::Rotation,
                t(event.start_time) => t(event.end_time),
                event.start => event.end,
            )
        });

        let opacity_event_iter = line.opacity_events.iter().map(|event| {
            event!(
                LineEventKind::Opacity,
                t(event.start_time) => t(event.end_time),
                event.start * 255.0 => event.end * 255.0,
            )
        });

        let speed_event_iter = line.speed_events.iter().map(|event| {
            event!(
                LineEventKind::Speed,
                t(event.start_time) => t(event.end_time),
                event.value / 2.0 * 9.0,
            )
        });

        let events: Vec<_> = move_events
            .into_iter()
            .chain(rotate_event_iter)
            .chain(opacity_event_iter)
            .chain(speed_event_iter)
            .collect();

        // Shrink constant events to 1/4
        let events = map_if(
            &events,
            |event| {
                event.value.is_numeric_constant()
                    && event.duration() > options.constant_event_shrink_to
            },
            |mut event| {
                event.end_beat = event.start_beat + options.constant_event_shrink_to;
                event
            },
        );

        let events = if options.easing_fitting {
            // Fit events for each kind (speed events are kept as-is, others are fitted)
            events
                .group_by_kind()
                .into_iter()
                .flat_map(|(kind, events)| {
                    if kind.is_speed() {
                        events
                    } else {
                        fit_events(events, options.easing_fitting_epsilon)
                    }
                })
                .collect()
        } else {
            events
        };

        // Remove redundant constant suffix events
        let events: Vec<_> = events
            .group_by_kind()
            .into_iter()
            .flat_map(|(_, events)| {
                let sorted_events = events.sorted();

                let mut prev_end_value = None;

                remove_if(&sorted_events, |event| {
                    let is_redundant = prev_end_value.is_some_and(|prev_end| {
                        event.value.is_numeric_constant() && event.value.start() == prev_end
                    });

                    prev_end_value = Some(event.value.end());
                    is_redundant
                })
            })
            .collect();

        // Convert numeric constant transition events to constant events
        let events = map_if(
            &events,
            |event| event.value.is_numeric_constant() && !event.value.is_constant(), // is numeric constant but not really a constant event
            |event| LineEvent {
                value: event.value.into_constant(),
                ..event
            },
        );

        let mut line = SerializedLine {
            notes: line
                .notes_above
                .iter()
                .map(|x| create_note(true, x))
                .chain(line.notes_below.iter().map(|x| create_note(false, x)))
                .collect(),
            events,

            ..Default::default()
        };

        let speed_events = line.events.speed().sorted();

        for note in &mut line.notes {
            if let NoteKind::Hold { .. } = note.kind {
                let mut speed = 0.0;
                for event in &speed_events {
                    let result = event.evaluate_inclusive(note.beat.value());
                    if let Some(value) = result.value() {
                        speed = value;
                    }
                }

                note.speed /= speed / 9.0 * 2.0;
            }
        }

        phichain.lines.push(line);
    }

    Ok(phichain)
}
