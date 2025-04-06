use crate::settings::{EditorSettings, ShowLineAnchorOption};
use crate::tab::settings::{SettingCategory, SettingUi};
use crate::translation::Languages;
use crate::ui::latch;
use crate::ui::widgets::language_combobox::language_combobox;
use bevy::prelude::World;
use egui::{Color32, RichText, Ui};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct General;

impl SettingCategory for General {
    fn name(&self) -> &str {
        "tab.settings.category.general.title"
    }

    fn ui(&self, ui: &mut Ui, settings: &mut EditorSettings, world: &mut World) -> bool {
        latch::latch(ui, "general-settings", settings.general.clone(), |ui| {
            let mut finished = false;

            let languages = world.resource::<Languages>().0.clone();

            finished |= ui.item(
                RichText::new(format!(
                    "{} {}",
                    egui_phosphor::regular::GLOBE,
                    t!("tab.settings.category.general.language.label")
                ))
                .color(Color32::LIGHT_BLUE),
                Some(t!("tab.settings.category.general.language.description")),
                |ui| language_combobox(ui, languages, &mut settings.general.language),
            );

            ui.separator();

            finished |= ui.item(
                t!("tab.settings.category.general.timeline_scroll_sensitivity.label"),
                Some(t!(
                    "tab.settings.category.general.timeline_scroll_sensitivity.description"
                )),
                |ui| {
                    let response = ui.add(
                        egui::DragValue::new(&mut settings.general.timeline_scroll_sensitivity)
                            .speed(0.1)
                            .range(0.01..=f32::MAX),
                    );

                    response.drag_stopped() || response.lost_focus()
                },
            );

            ui.separator();

            finished |= ui.item(
                t!("tab.settings.category.general.highlight_selected_line.label"),
                Some(t!(
                    "tab.settings.category.general.highlight_selected_line.description"
                )),
                |ui| {
                    let response = ui.checkbox(&mut settings.general.highlight_selected_line, "");
                    response.changed()
                },
            );

            ui.separator();

            finished |= ui.item(
                t!("tab.settings.category.general.show_line_anchor.label"),
                Some(t!(
                    "tab.settings.category.general.show_line_anchor.description"
                )),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut settings.general.show_line_anchor,
                            ShowLineAnchorOption::Never,
                            t!("tab.settings.category.general.show_line_anchor.never"),
                        )
                        .clicked()
                            || ui
                                .selectable_value(
                                    &mut settings.general.show_line_anchor,
                                    ShowLineAnchorOption::Always,
                                    t!("tab.settings.category.general.show_line_anchor.always"),
                                )
                                .clicked()
                            || ui
                                .selectable_value(
                                    &mut settings.general.show_line_anchor,
                                    ShowLineAnchorOption::Visible,
                                    t!("tab.settings.category.general.show_line_anchor.visible"),
                                )
                                .clicked()
                    })
                    .inner
                },
            );

            ui.separator();

            finished |= ui.item(
                t!("tab.settings.category.general.send_telemetry.label"),
                Some(t!(
                    "tab.settings.category.general.send_telemetry.description"
                )),
                |ui| {
                    let response = ui.checkbox(&mut settings.general.send_telemetry, "");
                    response.changed()
                },
            );

            finished
        })
        .is_some()
    }
}
