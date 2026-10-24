use crate::editing::history::Edits;
use crate::timeline::settings::TimelineSettings;
use crate::timing::ChartTime;
use crate::ui::widgets::beat_value::BeatValue;
use bevy::prelude::*;
use egui::{ScrollArea, Ui};
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::{BpmList, BpmPoint};
use phichain_chart::id::BpmPointId;

pub fn bpm_list_tab(
    In(mut ui): In<Ui>,
    points: Query<(Entity, &BpmPointId, &BpmPoint)>,
    timeline_settings: Res<TimelineSettings>,
    chart_time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    mut edits: Edits,
) {
    let mut points: Vec<_> = points.iter().collect();
    points.sort_by_key(|(_, id, point)| (point.beat, **id));
    ScrollArea::vertical().show(&mut ui, |ui| {
        for (index, (entity, _, point)) in points.iter().enumerate() {
            ui.push_id(*entity, |ui| {
                ui.horizontal_top(|ui| {
                    egui::Grid::new("bpm_point").num_columns(2).show(ui, |ui| {
                        let mut editor =
                            edits.component(*entity, *point, t!("history.edit_bpm_point"));
                        ui.label(t!("tab.bpm_list.point.beat"));
                        let prev = index
                            .checked_sub(1)
                            .and_then(|prev| points.get(prev))
                            .map(|(_, _, point)| point.beat);
                        let next = points.get(index + 1).map(|(_, _, point)| point.beat);
                        ui.add_enabled_ui(point.beat != Beat::ZERO, |ui| {
                            editor.edit(ui, "beat", |ui, point| {
                                let before = point.beat;
                                ui.add(
                                    BeatValue::new(&mut point.beat)
                                        .range(
                                            prev.unwrap_or(Beat::MIN)..=next.unwrap_or(Beat::MAX),
                                        )
                                        .density(timeline_settings.density),
                                );
                                if Some(point.beat) == prev || Some(point.beat) == next {
                                    point.beat = before;
                                }
                            });
                        })
                        .response
                        .on_disabled_hover_text(t!("tab.bpm_list.zero_beat_not_editable"));
                        ui.end_row();
                        ui.label(t!("tab.bpm_list.point.bpm"));
                        editor.edit(ui, "bpm", |ui, point| {
                            ui.add(egui::DragValue::new(&mut point.bpm).range(0.01..=f32::MAX));
                        });
                        ui.end_row();
                    });
                    if ui
                        .add_enabled(point.beat != Beat::ZERO, egui::Button::new(" × "))
                        .on_disabled_hover_text(t!("tab.bpm_list.zero_beat_not_editable"))
                        .clicked()
                    {
                        let entity = *entity;
                        edits.once(t!("history.delete_bpm_point"), move |commands| {
                            commands.entity(entity).despawn();
                        });
                    }
                });
            });
            ui.separator();
        }
        if ui.button(t!("tab.bpm_list.new")).clicked() {
            let (beat, bpm) = match points.last() {
                Some((_, _, last)) => {
                    let playhead = timeline_settings.attach(bpm_list.beat_at(chart_time.0).value());
                    let beat = if playhead < Beat::ZERO
                        || points.iter().any(|(_, _, point)| point.beat == playhead)
                    {
                        last.beat + Beat::ONE
                    } else {
                        playhead
                    };
                    (beat, bpm_list.bpm_at(bpm_list.time_at(beat)))
                }
                None => (Beat::ONE, 120.0),
            };
            edits.once(t!("history.create_bpm_point"), move |commands| {
                commands.spawn(BpmPoint::new(beat, bpm));
            });
        }
    });
}
