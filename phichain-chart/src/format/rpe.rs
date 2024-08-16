//! Re:PhiEdit json format

use crate::bpm_list::BpmList;
use crate::easing::Easing;
use crate::format::Format;
use crate::primitive;
use crate::primitive::{PrimitiveChart, PrimitiveCompatibleFormat};
use crate::serialization::PhichainChart;
use num::{Num, Rational32};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tracing::warn;

#[derive(Default, Debug, Clone, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum NoteKind {
    #[default]
    Tap = 1,
    Drag = 4,
    Hold = 2,
    Flick = 3,
}

// generated by https://transform.tools/json-to-rust-serde
// TODO: event layer, parent support
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Beat(i32, i32, i32);

impl From<Beat> for crate::beat::Beat {
    fn from(val: Beat) -> Self {
        crate::beat::Beat::new(val.0, Rational32::new(val.1, val.2))
    }
}

impl From<crate::beat::Beat> for Beat {
    fn from(value: crate::beat::Beat) -> Self {
        Beat(value.beat(), value.numer(), value.denom())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpeChart {
    #[serde(rename = "BPMList")]
    bpm_list: Vec<BpmPoint>,
    #[serde(rename = "META")]
    meta: Meta,
    judge_line_list: Vec<JudgeLine>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BpmPoint {
    bpm: f32,
    start_time: Beat,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Meta {
    #[serde(rename = "RPEVersion")]
    rpeversion: i32,
    background: String,
    charter: String,
    composer: String,
    id: String,
    level: String,
    name: String,
    offset: i32,
    song: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JudgeLine {
    event_layers: Vec<EventLayer>,
    #[serde(default)]
    notes: Vec<Note>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventLayer {
    #[serde(default)]
    alpha_events: Vec<CommonEvent<i32>>,
    #[serde(rename = "moveXEvents")]
    move_xevents: Vec<CommonEvent<f32>>,
    #[serde(rename = "moveYEvents")]
    move_yevents: Vec<CommonEvent<f32>>,
    #[serde(default)]
    rotate_events: Vec<CommonEvent<f32>>,
    #[serde(default)]
    speed_events: Vec<SpeedEvent>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommonEvent<T: Num> {
    bezier: i32,
    #[serde(rename = "bezierPoints")]
    bezier_points: [f32; 4],
    easing_type: i32,
    end: T,
    end_time: Beat,
    start: T,
    start_time: Beat,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpeedEvent {
    end: f32,
    end_time: Beat,
    start: f32,
    start_time: Beat,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Note {
    above: i32,
    end_time: Beat,
    position_x: f32,
    speed: f32,
    start_time: Beat,
    size: f32,         // ignored
    visible_time: f32, // ignored
    #[serde(rename = "type")]
    kind: NoteKind,
}

static RPE_EASING: [Easing; 30] = [
    Easing::Linear,
    Easing::Linear,
    Easing::EaseOutSine,
    Easing::EaseInSine,
    Easing::EaseOutQuad,
    Easing::EaseInQuad,
    Easing::EaseInOutSine,
    Easing::EaseInOutQuad,
    Easing::EaseOutCubic,
    Easing::EaseInCubic,
    Easing::EaseOutQuart,
    Easing::EaseInQuart,
    Easing::EaseInOutCubic,
    Easing::EaseInOutQuart,
    Easing::EaseOutQuint,
    Easing::EaseInQuint,
    Easing::EaseOutExpo,
    Easing::EaseInExpo,
    Easing::EaseOutCirc,
    Easing::EaseInCirc,
    Easing::EaseOutBack,
    Easing::EaseInBack,
    Easing::EaseInOutCirc,
    Easing::EaseInOutBack,
    Easing::EaseOutElastic,
    Easing::EaseInElastic,
    Easing::EaseOutBounce,
    Easing::EaseInBounce,
    Easing::EaseInOutBounce,
    Easing::EaseInOutElastic,
];

impl PrimitiveCompatibleFormat for RpeChart {
    fn into_primitive(self) -> anyhow::Result<PrimitiveChart> {
        let mut primitive = PrimitiveChart {
            offset: self.meta.offset as f32,
            bpm_list: BpmList::new(
                self.bpm_list
                    .iter()
                    .map(|x| crate::bpm_list::BpmPoint::new(x.start_time.clone().into(), x.bpm))
                    .collect(),
            ),
            lines: vec![],
            ..Default::default()
        };

        let e = |id: i32| {
            RPE_EASING.get(id as usize).copied().unwrap_or_else(|| {
                warn!("Unknown easing type: {}", id);
                Easing::Linear
            })
        };

        for line in self.judge_line_list {
            let x_event_iter = line
                .event_layers
                .iter()
                .flat_map(|layer| layer.move_xevents.clone())
                .map(|event| crate::event::LineEvent {
                    kind: crate::event::LineEventKind::X,
                    start_beat: event.start_time.into(),
                    end_beat: event.end_time.into(),
                    value: crate::event::LineEventValue::transition(
                        event.start,
                        event.end,
                        e(event.easing_type),
                    ),
                });
            let y_event_iter = line
                .event_layers
                .iter()
                .flat_map(|layer| layer.move_yevents.clone())
                .map(|event| crate::event::LineEvent {
                    kind: crate::event::LineEventKind::Y,
                    start_beat: event.start_time.into(),
                    end_beat: event.end_time.into(),
                    value: crate::event::LineEventValue::transition(
                        event.start,
                        event.end,
                        e(event.easing_type),
                    ),
                });
            let rotate_event_iter = line
                .event_layers
                .iter()
                .flat_map(|layer| layer.rotate_events.clone())
                .map(|event| crate::event::LineEvent {
                    kind: crate::event::LineEventKind::Rotation,
                    start_beat: event.start_time.into(),
                    end_beat: event.end_time.into(),
                    // negate value for rotation
                    value: crate::event::LineEventValue::transition(
                        -event.start,
                        -event.end,
                        e(event.easing_type),
                    ),
                });
            let alpha_event_iter = line
                .event_layers
                .iter()
                .flat_map(|layer| layer.alpha_events.clone())
                .map(|event| crate::event::LineEvent {
                    kind: crate::event::LineEventKind::Opacity,
                    start_beat: event.start_time.into(),
                    end_beat: event.end_time.into(),
                    value: crate::event::LineEventValue::transition(
                        event.start as f32,
                        event.end as f32,
                        e(event.easing_type),
                    ),
                });
            let speed_event_iter = line
                .event_layers
                .iter()
                .flat_map(|layer| layer.speed_events.clone())
                .map(|event| crate::event::LineEvent {
                    kind: crate::event::LineEventKind::Speed,
                    start_beat: event.start_time.into(),
                    end_beat: event.end_time.into(),
                    value: crate::event::LineEventValue::transition(
                        event.start,
                        event.end,
                        Easing::Linear, // speed events' easing are fixed to be Linear
                    ),
                });

            primitive.lines.push(primitive::line::Line {
                notes: line
                    .notes
                    .iter()
                    .map(|note| {
                        let start_beat = crate::beat::Beat::from(note.start_time.clone());
                        let end_beat = crate::beat::Beat::from(note.end_time.clone());
                        let kind: crate::note::NoteKind = match note.kind {
                            NoteKind::Tap => crate::note::NoteKind::Tap,
                            NoteKind::Drag => crate::note::NoteKind::Drag,
                            NoteKind::Hold => crate::note::NoteKind::Hold {
                                hold_beat: end_beat - start_beat,
                            },
                            NoteKind::Flick => crate::note::NoteKind::Flick,
                        };

                        crate::note::Note::new(
                            kind,
                            note.above == 1,
                            start_beat,
                            note.position_x,
                            note.speed,
                        )
                    })
                    .collect(),
                events: x_event_iter
                    .chain(y_event_iter)
                    .chain(rotate_event_iter)
                    .chain(alpha_event_iter)
                    .chain(speed_event_iter)
                    .collect(),
            });
        }

        Ok(primitive)
    }

    fn from_primitive(primitive: PrimitiveChart) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut rpe = RpeChart {
            bpm_list: primitive
                .bpm_list
                .0
                .iter()
                .map(|x| BpmPoint {
                    bpm: x.bpm,
                    start_time: x.beat.into(),
                })
                .collect(),
            meta: Meta {
                offset: primitive.offset as i32,
                ..Default::default()
            },
            judge_line_list: vec![],
        };

        let e = |easing: Easing| {
            RPE_EASING
                .iter()
                .position(|x| *x == easing)
                .unwrap_or_else(|| {
                    warn!("Unknown easing type: {}", easing);
                    1
                })
        };

        for primitive::line::Line { notes, events } in primitive.lines {
            let mut line = JudgeLine::default();
            for note in notes {
                let kind = match note.kind {
                    crate::note::NoteKind::Tap => NoteKind::Tap,
                    crate::note::NoteKind::Drag => NoteKind::Drag,
                    crate::note::NoteKind::Hold { .. } => NoteKind::Hold,
                    crate::note::NoteKind::Flick => NoteKind::Flick,
                };
                let end_beat = match note.kind {
                    crate::note::NoteKind::Hold { hold_beat } => note.beat + hold_beat,
                    _ => note.beat,
                };
                line.notes.push(Note {
                    above: if note.above { 1 } else { 2 },
                    end_time: end_beat.into(),
                    position_x: note.x,
                    speed: note.speed,
                    start_time: note.beat.into(),
                    size: 1.0,
                    visible_time: 999999.0,
                    kind,
                });
            }
            let mut event_layer = EventLayer::default();
            for event in events {
                let (start, end, easing) = match event.value {
                    crate::event::LineEventValue::Transition { start, end, easing } => {
                        (start, end, easing)
                    }
                    crate::event::LineEventValue::Constant(value) => (value, value, Easing::Linear),
                };

                let mut rpe_event = CommonEvent {
                    bezier: 0,
                    bezier_points: [0.0, 0.0, 0.0, 0.0],
                    easing_type: 0,
                    end,
                    end_time: event.end_beat.into(),
                    start,
                    start_time: event.start_beat.into(),
                };

                if let Easing::Custom(a, b, c, d) = easing {
                    rpe_event.bezier_points = [a, b, c, d];
                    rpe_event.bezier = 1;
                    rpe_event.easing_type = 1;
                } else {
                    rpe_event.easing_type = e(easing) as i32;
                }

                match event.kind {
                    crate::event::LineEventKind::X => {
                        event_layer.move_xevents.push(rpe_event);
                    }
                    crate::event::LineEventKind::Y => {
                        event_layer.move_yevents.push(rpe_event);
                    }
                    crate::event::LineEventKind::Rotation => {
                        // negate value for rotation
                        event_layer.rotate_events.push(CommonEvent {
                            bezier: rpe_event.bezier,
                            bezier_points: rpe_event.bezier_points,
                            easing_type: rpe_event.easing_type,
                            end: -rpe_event.end,
                            end_time: rpe_event.end_time,
                            start: -rpe_event.start,
                            start_time: rpe_event.start_time,
                        });
                    }
                    crate::event::LineEventKind::Opacity => {
                        event_layer.alpha_events.push(CommonEvent {
                            bezier: rpe_event.bezier,
                            bezier_points: rpe_event.bezier_points,
                            easing_type: rpe_event.easing_type,
                            end: rpe_event.end as i32,
                            end_time: rpe_event.end_time,
                            start: rpe_event.start as i32,
                            start_time: rpe_event.start_time,
                        });
                    }
                    crate::event::LineEventKind::Speed => {
                        event_layer.speed_events.push(SpeedEvent {
                            start,
                            end,
                            end_time: event.end_beat.into(),
                            start_time: event.start_beat.into(),
                        })
                    }
                }
            }
            line.event_layers.push(event_layer);

            rpe.judge_line_list.push(line);
        }

        Ok(rpe)
    }
}

impl Format for RpeChart {
    fn into_phichain(self) -> anyhow::Result<PhichainChart> {
        // let mut bpm_list = crate::bpm_list::BpmList::new(
        //     self.bpm_list
        //         .iter()
        //         .map(|x| crate::bpm_list::BpmPoint::new(x.start_time.clone().into(), x.bpm))
        //         .collect(),
        // );
        // bpm_list.compute();
        // let mut phichain = PhichainChart::new(self.meta.offset as f32, bpm_list, vec![]);
        //
        // let easing = |id: i32| {
        //     Easing::iter().nth((id - 1) as usize).unwrap_or_else(|| {
        //         warn!("Unknown easing type: {}", id);
        //         Easing::Linear
        //     })
        // };
        //
        // for line in self.judge_line_list {
        //     let x_event_iter = line
        //         .event_layers
        //         .iter()
        //         .flat_map(|layer| layer.move_xevents.clone())
        //         .map(|event| LineEvent {
        //             kind: LineEventKind::X,
        //             start: event.start,
        //             end: event.end,
        //             start_beat: event.start_time.into(),
        //             end_beat: event.end_time.into(),
        //             easing: easing(event.easing_type),
        //         });
        //     let y_event_iter = line
        //         .event_layers
        //         .iter()
        //         .flat_map(|layer| layer.move_yevents.clone())
        //         .map(|event| LineEvent {
        //             kind: LineEventKind::Y,
        //             start: event.start,
        //             end: event.end,
        //             start_beat: event.start_time.into(),
        //             end_beat: event.end_time.into(),
        //             easing: easing(event.easing_type),
        //         });
        //     let rotate_event_iter = line
        //         .event_layers
        //         .iter()
        //         .flat_map(|layer| layer.rotate_events.clone())
        //         .map(|event| LineEvent {
        //             kind: LineEventKind::Rotation,
        //             start: event.start,
        //             end: event.end,
        //             start_beat: event.start_time.into(),
        //             end_beat: event.end_time.into(),
        //             easing: easing(event.easing_type),
        //         });
        //     let alpha_event_iter = line
        //         .event_layers
        //         .iter()
        //         .flat_map(|layer| layer.alpha_events.clone())
        //         .map(|event| LineEvent {
        //             kind: LineEventKind::Opacity,
        //             start: event.start as f32,
        //             end: event.end as f32,
        //             start_beat: event.start_time.into(),
        //             end_beat: event.end_time.into(),
        //             easing: easing(event.easing_type),
        //         });
        //     let speed_event_iter = line
        //         .event_layers
        //         .iter()
        //         .flat_map(|layer| layer.speed_events.clone())
        //         .map(|event| LineEvent {
        //             kind: LineEventKind::Speed,
        //             start: event.start,
        //             end: event.end,
        //             start_beat: event.start_time.into(),
        //             end_beat: event.end_time.into(),
        //             easing: Easing::Linear, // speed events' easing are fixed to be Linear
        //         });
        //
        //     phichain.lines.push(LineWrapper::new(
        //         Default::default(),
        //         line.notes
        //             .iter()
        //             .map(|note| {
        //                 let start_beat = crate::beat::Beat::from(note.start_time.clone());
        //                 let end_beat = crate::beat::Beat::from(note.end_time.clone());
        //                 let kind: crate::note::NoteKind = match note.kind {
        //                     NoteKind::Tap => crate::note::NoteKind::Tap,
        //                     NoteKind::Drag => crate::note::NoteKind::Drag,
        //                     NoteKind::Hold => crate::note::NoteKind::Hold {
        //                         hold_beat: end_beat - start_beat,
        //                     },
        //                     NoteKind::Flick => crate::note::NoteKind::Flick,
        //                 };
        //
        //                 crate::note::Note::new(
        //                     kind,
        //                     note.above == 1,
        //                     start_beat,
        //                     note.position_x,
        //                     note.speed,
        //                 )
        //             })
        //             .collect(),
        //         x_event_iter
        //             .chain(y_event_iter)
        //             .chain(rotate_event_iter)
        //             .chain(alpha_event_iter)
        //             .chain(speed_event_iter)
        //             .collect(),
        //     ));
        // }
        //
        // Ok(phichain)

        unimplemented!("");
    }

    fn from_phichain(_phichain: PhichainChart) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        unimplemented!("");
    }
}
