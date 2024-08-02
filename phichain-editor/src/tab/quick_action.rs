use crate::audio::AudioDuration;
use crate::settings::EditorSettings;
use crate::tab::game::AspectRatio;
use crate::timing::{ChartTime, SeekToEvent};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use egui::{vec2, Ui};
use phichain_chart::bpm_list::BpmList;

pub fn quick_action_tab(
    In(ui): In<&mut Ui>,
    mut editor_settings: ResMut<Persistent<EditorSettings>>,

    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    duration: Res<AudioDuration>,
    mut events: EventWriter<SeekToEvent>,

    mut aspect_ratio: ResMut<AspectRatio>,
) {
    ui.horizontal(|ui| {
        ui.label(t!("tab.settings.category.audio.playback_rate"));
        ui.add(
            egui::DragValue::new(&mut editor_settings.audio.playback_rate)
                .suffix("x")
                .clamp_range(0.01..=1.0)
                .speed(0.01),
        );

        let space = ui.available_width() - 300.0;
        if space > 0.0 {
            ui.add_space(space)
        }

        // -------- Aspect Ratio Control --------

        egui::ComboBox::from_label("")
            .width(55.0)
            .selected_text(format!("{}", *aspect_ratio))
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(
                        matches!(*aspect_ratio, AspectRatio::Free),
                        t!("game.aspect_ratio.free"),
                    )
                    .clicked()
                {
                    *aspect_ratio = AspectRatio::Free;
                }

                macro_rules! aspect_ratio_button {
                    ($ui:expr, $aspect_ratio:expr, $width:expr, $height:expr, $label:expr) => {
                        if $ui
                            .selectable_label(
                                matches!(
                                    *$aspect_ratio,
                                    AspectRatio::Fixed {
                                        width: $width,
                                        height: $height
                                    }
                                ),
                                $label,
                            )
                            .clicked()
                        {
                            *$aspect_ratio = AspectRatio::Fixed {
                                width: $width,
                                height: $height,
                            };
                        }
                    };
                }

                aspect_ratio_button!(ui, aspect_ratio, 4.0, 3.0, "4:3");
                aspect_ratio_button!(ui, aspect_ratio, 16.0, 9.0, "16:9");
                aspect_ratio_button!(ui, aspect_ratio, 21.0, 9.0, "21:9");
                aspect_ratio_button!(ui, aspect_ratio, 1.0, 1.0, "1:1");
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
                    .clamp_range(0.0..=duration.0.as_secs_f32()),
            );
            let max_beat = bpm_list.beat_at(duration.0.as_secs_f32());
            ui.add_sized(
                vec2(55.0, 18.0),
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
