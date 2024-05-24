mod general;

use crate::notification::{ToastsExt, ToastsStorage};
use crate::settings::EditorSettings;
use crate::tab::settings::general::General;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use egui::{Layout, Ui};
use enum_dispatch::enum_dispatch;
use strum::{EnumIter, IntoEnumIterator};

#[enum_dispatch(SettingCategories)]
pub trait SettingCategory {
    fn name(&self) -> &str;
    fn ui(&self, ui: &mut Ui, settings: &mut EditorSettings) -> bool;
}

#[enum_dispatch]
#[derive(Copy, Clone, PartialEq, Eq, Debug, EnumIter)]
enum SettingCategories {
    General,
    Graphics,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
struct Graphics;
impl SettingCategory for Graphics {
    fn name(&self) -> &str {
        "tab.settings.category.graphics.title"
    }

    fn ui(&self, ui: &mut Ui, _settings: &mut EditorSettings) -> bool {
        ui.label("This is the graphics settings tab");
        false
    }
}

pub fn settings_tab(
    In(ui): In<&mut Ui>,
    mut editor_settings: ResMut<Persistent<EditorSettings>>,
    mut toasts: ResMut<ToastsStorage>,
) {
    let available_height = ui.available_height();

    let id = egui::Id::new("settings-category");
    let set_category = |category: SettingCategories| {
        ui.data_mut(|data| data.insert_temp(id, category));
        category
    };
    let data = ui.data(|data| data.get_temp::<SettingCategories>(id));
    let category = match data {
        None => set_category(SettingCategories::from(General)),
        Some(category) => category,
    };

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_height(available_height);
            ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
                ui.set_max_width(80.0);

                for c in SettingCategories::iter() {
                    if ui.selectable_label(category == c, t!(c.name())).clicked() {
                        ui.data_mut(|data| data.insert_temp(id, c));
                    }
                }
            });
        });
        ui.separator();

        ui.vertical(|ui| {
            ui.heading(t!(category.name()));
            if category.ui(ui, &mut editor_settings) {
                match editor_settings.persist() {
                    Ok(_) => {}
                    Err(error) => {
                        toasts.error(format!("Failed to persist editor settings: {}", error))
                    }
                }
            }
        });
    });
}
