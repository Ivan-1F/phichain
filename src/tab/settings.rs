use bevy::prelude::*;
use egui::{Layout, Ui};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
enum SettingCategory {
    #[default]
    General,
    Graphics,
}

pub fn settings_tab(In(ui): In<&mut Ui>) {
    let available_height = ui.available_height();

    let id = egui::Id::new("settings-category");
    let set_category = |category: SettingCategory| {
        ui.data_mut(|data| data.insert_temp(id, category));
        category
    };
    let data = ui.data(|data| data.get_temp::<SettingCategory>(id));
    let category = match data {
        None => set_category(SettingCategory::default()),
        Some(category) => category,
    };

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_height(available_height);
            ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
                ui.set_max_width(80.0);

                if ui
                    .selectable_label(
                        category == SettingCategory::General,
                        t!("tab.settings.category.general"),
                    )
                    .clicked()
                {
                    ui.data_mut(|data| data.insert_temp(id, SettingCategory::General));
                }

                if ui
                    .selectable_label(
                        category == SettingCategory::Graphics,
                        t!("tab.settings.category.graphics"),
                    )
                    .clicked()
                {
                    ui.data_mut(|data| data.insert_temp(id, SettingCategory::Graphics));
                }
            });
        });
        ui.separator();
        ui.vertical(|ui| match category {
            SettingCategory::General => {
                ui.heading(t!("tab.settings.category.general"));
            }
            SettingCategory::Graphics => {
                ui.heading(t!("tab.settings.category.graphics"));
            }
        });
    });
}
