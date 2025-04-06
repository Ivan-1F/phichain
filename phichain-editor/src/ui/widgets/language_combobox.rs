use bevy::utils::HashMap;
use egui::Ui;
use rust_i18n::set_locale;

pub fn language_combobox(
    ui: &mut Ui,
    languages: HashMap<String, String>,
    value: &mut String,
) -> bool {
    let mut combobox_changed = false;
    egui::ComboBox::from_label("")
        .selected_text(languages.get(value).unwrap_or(value))
        .show_ui(ui, |ui| {
            for (id, name) in languages {
                if ui.selectable_label(value == &id, name).clicked() {
                    set_locale(&id);
                    *value = id;
                    combobox_changed = true;
                }
            }
        });

    combobox_changed
}
