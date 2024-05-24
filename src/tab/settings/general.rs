use crate::settings::EditorSettings;
use crate::tab::settings::SettingCategory;
use crate::ui::latch;
use egui::Ui;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct General;

impl SettingCategory for General {
    fn name(&self) -> &str {
        "tab.settings.category.general.title"
    }

    fn ui(&self, ui: &mut Ui, settings: &mut EditorSettings) -> bool {
        egui::Grid::new("general-settings-grid")
            .num_columns(2)
            .spacing([20.0, 2.0])
            .striped(true)
            .show(ui, |ui| {
                latch::latch(ui, "general-settings", settings.general, |ui| {
                    let mut finished = false;
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
