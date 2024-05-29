use crate::settings::EditorSettings;
use crate::tab::settings::SettingCategory;
use crate::translation::Languages;
use crate::ui::latch;
use bevy::prelude::World;
use egui::Ui;
use rust_i18n::set_locale;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct General;

impl SettingCategory for General {
    fn name(&self) -> &str {
        "tab.settings.category.general.title"
    }

    fn ui(&self, ui: &mut Ui, settings: &mut EditorSettings, world: &mut World) -> bool {
        egui::Grid::new("general-settings-grid")
            .num_columns(2)
            .spacing([20.0, 2.0])
            .striped(true)
            .show(ui, |ui| {
                latch::latch(ui, "general-settings", settings.general.clone(), |ui| {
                    let mut finished = false;

                    let languages = world.resource::<Languages>();
                    ui.label("Language");
                    let mut combobox_changed = false;
                    egui::ComboBox::from_label("")
                        .selected_text(
                            languages
                                .0
                                .get(&settings.general.language)
                                .unwrap_or(&settings.general.language),
                        )
                        .show_ui(ui, |ui| {
                            for (id, name) in &languages.0 {
                                if ui
                                    .selectable_label(settings.general.language == *id, name)
                                    .clicked()
                                {
                                    settings.general.language.clone_from(id);
                                    set_locale(id);
                                    combobox_changed = true;
                                }
                            }
                        });
                    finished |= combobox_changed;
                    ui.end_row();

                    ui.label(t!(
                        "tab.settings.category.general.timeline_scroll_sensitivity"
                    ));
                    let response = ui.add(
                        egui::DragValue::new(&mut settings.general.timeline_scroll_sensitivity)
                            .speed(0.1)
                            .clamp_range(0.01..=f32::MAX),
                    );
                    finished |= response.drag_stopped() || response.lost_focus();
                    ui.end_row();

                    finished
                })
                .is_some()
            })
            .inner
    }
}
