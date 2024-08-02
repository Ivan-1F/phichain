use crate::audio::AudioDuration;
use crate::settings::EditorSettings;
use crate::timing::{ChartTime, SeekToEvent};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use egui::Ui;
use phichain_chart::bpm_list::BpmList;

pub fn quick_action_tab(
    In(ui): In<&mut Ui>,
    mut editor_settings: ResMut<Persistent<EditorSettings>>,

    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    duration: Res<AudioDuration>,
    mut events: EventWriter<SeekToEvent>,
) {
    ui.horizontal(|ui| {
        ui.label(t!("tab.settings.category.audio.playback_rate"));
        ui.add(
            egui::DragValue::new(&mut editor_settings.audio.playback_rate)
                .suffix("x")
                .clamp_range(0.01..=1.0)
                .speed(0.01),
        );

        let space = ui.available_width() - 195.0;
        if space > 0.0 {
            ui.add_space(space)
        }

        // -------- Progress Control --------

        let seconds = time.0;
        let mut second_binding = seconds;
        let beats = bpm_list.beat_at(seconds).value();
        let mut beat_binding = beats;

        ui.horizontal(|ui| {
            ui.add(
                egui::Slider::new(&mut second_binding, 0.0..=duration.0.as_secs_f32())
                    .custom_formatter(|x, _| format!("{:.2}", x))
                    .drag_value_speed(0.05),
            );
            let max_beat = bpm_list.beat_at(duration.0.as_secs_f32());
            ui.add(
                egui::DragValue::new(&mut beat_binding)
                    .speed(0.05)
                    .custom_formatter(|x, _| format!("{:.2}", x))
                    .clamp_range(0.0..=max_beat.value()),
            );
        });

        if second_binding != seconds {
            events.send(SeekToEvent(second_binding));
        }

        if beat_binding != beats {
            events.send(SeekToEvent(bpm_list.time_at(beat_binding.into())));
        }
    });
}
