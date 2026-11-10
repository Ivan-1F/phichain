use egui::{Response, Ui};
use indexmap::IndexMap;
use rust_i18n::set_locale;

pub fn language_combobox(
    ui: &mut Ui,
    languages: IndexMap<String, String>,
    value: &mut String,
) -> Response {
    let mut changed = false;
    let mut response = egui::ComboBox::from_label("")
        .selected_text(languages.get(value).unwrap_or(value))
        .show_ui(ui, |ui| {
            for (id, name) in languages {
                if ui.selectable_label(value == &id, name).clicked() {
                    set_locale(&id);
                    *value = id;
                    changed = true;
                }
            }
        })
        .response;

    if changed {
        response.mark_changed();
    }
    response
}
