mod audio;
mod game;
mod general;
mod hotkey;

use crate::notification::{ToastsExt, ToastsStorage};
use crate::settings::EditorSettings;
use crate::tab::settings::audio::Audio;
use crate::tab::settings::game::Game;
use crate::tab::settings::general::General;
use crate::tab::settings::hotkey::Hotkey;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use egui::{Layout, RichText, Ui, WidgetText};
use enum_dispatch::enum_dispatch;
use strum::{EnumIter, IntoEnumIterator};

pub trait SettingUi {
    fn item(
        self,
        name: impl Into<WidgetText>,
        description: Option<impl Into<RichText>>,
        widget: impl FnOnce(&mut Ui) -> bool,
    ) -> bool;
}

impl SettingUi for &mut Ui {
    fn item(
        self,
        name: impl Into<WidgetText>,
        description: Option<impl Into<RichText>>,
        widget: impl FnOnce(&mut Ui) -> bool,
    ) -> bool {
        let (_, right) = crate::ui::sides::Sides::new().show(
            self,
            |ui| {
                ui.vertical(|ui| {
                    ui.label(name);
                    if let Some(description) = description {
                        ui.label(description.into().size(11.0));
                    }
                });
            },
            widget,
        );

        right
    }
}

#[enum_dispatch(SettingCategories)]
pub trait SettingCategory {
    fn name(&self) -> &str;
    fn ui(&self, ui: &mut Ui, settings: &mut EditorSettings, world: &mut World) -> bool;
}

#[enum_dispatch]
#[derive(Copy, Clone, PartialEq, Eq, Debug, EnumIter)]
enum SettingCategories {
    General,
    Audio,
    Game,
    Hotkey,
}

pub fn settings_tab(In(mut ui): In<Ui>, world: &mut World) {
    settings_ui(&mut ui, world);
}

pub fn settings_ui(ui: &mut Ui, world: &mut World) {
    world.resource_scope(
        |world, mut editor_settings: Mut<Persistent<EditorSettings>>| {
            world.resource_scope(|world, mut toasts: Mut<ToastsStorage>| {
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
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if category.ui(ui, &mut editor_settings, world) {
                                match editor_settings.persist() {
                                    Ok(_) => {}
                                    Err(error) => toasts.error(format!(
                                        "Failed to persist editor settings: {}",
                                        error
                                    )),
                                }
                            }
                        });
                    });
                });
            });
        },
    );
}
