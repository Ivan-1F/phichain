use crate::settings::EditorSettings;
use crate::translation::Languages;
use bevy::prelude::World;
use bevy_persistent::Persistent;
use egui::Ui;
use rust_i18n::set_locale;

pub fn language_combobox(ui: &mut Ui, world: &mut World) -> bool {
    let languages = world.resource::<Languages>().0.clone();
    let mut settings = world.resource_mut::<Persistent<EditorSettings>>();

    let mut combobox_changed = false;
    egui::ComboBox::from_label("")
        .selected_text(
            languages
                .get(&settings.general.language)
                .unwrap_or(&settings.general.language),
        )
        .show_ui(ui, |ui| {
            for (id, name) in languages {
                if ui
                    .selectable_label(settings.general.language == *id, name)
                    .clicked()
                {
                    set_locale(&id);
                    settings.general.language = id;
                    combobox_changed = true;
                }
            }
        });

    combobox_changed
}
