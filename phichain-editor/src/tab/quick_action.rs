use crate::audio::AudioDuration;
use crate::notification::{ToastsExt, ToastsStorage};
use crate::settings::{AspectRatio, EditorSettings};
use crate::timing::{ChartTime, SeekToEvent};
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use egui::{vec2, Ui};
use phichain_chart::bpm_list::BpmList;

pub fn quick_action(ui: &mut Ui, world: &mut World) {
    let mut state: SystemState<(
        ResMut<Persistent<EditorSettings>>,
        ResMut<ToastsStorage>,
        Res<ChartTime>,
        Res<BpmList>,
        Res<AudioDuration>,
        EventWriter<SeekToEvent>,
    )> = SystemState::new(world);

    let (mut editor_settings, mut toasts, time, bpm_list, duration, mut events) =
        state.get_mut(world);

    ui.horizontal(|ui| {
        ui.label(t!("tab.settings.category.audio.playback_rate.label"));
        ui.add(
            egui::DragValue::new(&mut editor_settings.audio.playback_rate)
                .suffix("x")
                .range(0.01..=1.0)
                .speed(0.01),
        );

        let space = ui.available_width() - 300.0;
        if space > 0.0 {
            ui.add_space(space)
        }

        // -------- Aspect Ratio Control --------

        egui::ComboBox::from_label("")
            .width(55.0)
            .selected_text(format!("{}", editor_settings.game.aspect_ratio))
            .show_ui(ui, |ui| {
                let mut changed = false;
                if ui
                    .selectable_label(
                        matches!(editor_settings.game.aspect_ratio, AspectRatio::Free),
                        t!("game.aspect_ratio.free"),
                    )
                    .clicked()
                {
                    changed = true;
                    editor_settings.game.aspect_ratio = AspectRatio::Free;
                }

                macro_rules! aspect_ratio_button {
                    ($ui:expr, $width:expr, $height:expr, $label:expr) => {
                        if $ui
                            .selectable_label(
                                matches!(
                                    editor_settings.game.aspect_ratio,
                                    AspectRatio::Fixed {
                                        width: $width,
                                        height: $height
                                    }
                                ),
                                $label,
                            )
                            .clicked()
                        {
                            changed = true;
                            editor_settings.game.aspect_ratio = AspectRatio::Fixed {
                                width: $width,
                                height: $height,
                            };
                        }
                    };
                }

                aspect_ratio_button!(ui, 4.0, 3.0, "4:3");
                aspect_ratio_button!(ui, 16.0, 9.0, "16:9");
                aspect_ratio_button!(ui, 21.0, 9.0, "21:9");
                aspect_ratio_button!(ui, 1.0, 1.0, "1:1");

                if changed {
                    match editor_settings.persist() {
                        Ok(_) => {}
                        Err(error) => {
                            toasts.error(format!("Failed to persist editor settings: {}", error))
                        }
                    }
                }
            });

        // -------- Progress Control --------

        let seconds = time.0;
        let mut second_binding = seconds;
        let beats = bpm_list.beat_at(seconds).value();
        let mut beat_binding = beats;

        ui.horizontal(|ui| {
            ui.add(
                egui::Slider::new(&mut second_binding, 0.0..=duration.0.as_secs_f32())
                    .show_value(false),
            );
            ui.add_sized(
                vec2(55.0, 18.0),
                egui::DragValue::new(&mut second_binding)
                    .speed(0.05)
                    .custom_formatter(|x, _| format!("{:.2}", x))
                    .range(0.0..=duration.0.as_secs_f32()),
            );
            let max_beat = bpm_list.beat_at(duration.0.as_secs_f32());
            ui.add_sized(
                vec2(55.0, 18.0),
                egui::DragValue::new(&mut beat_binding)
                    .speed(0.05)
                    .custom_formatter(|x, _| format!("{:.2}", x))
                    .range(0.0..=max_beat.value()),
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
