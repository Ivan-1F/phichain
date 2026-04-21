use std::sync::Arc;

use crate::misc::WorkingDirectory;
use crate::respack::{scan_respacks, ReloadRespack, RespackEntry, RespackSource, SelectRespack};
use crate::settings::EditorSettings;
use crate::tab::settings::{SettingCategory, SettingUi};
use crate::ui::sides::SidesExt;
use crate::ui::widgets::button_frame::button_frame;
use bevy::prelude::World;
use egui::{Context, Image, Rect, RichText, Sense, TextureHandle, TextureOptions, Ui, Vec2};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Respack;

const PREVIEW_SIZE: f32 = 32.0;

/// An entry paired with its four preview textures already uploaded to egui.
type Cached = (RespackEntry, [TextureHandle; 4]);

impl SettingCategory for Respack {
    fn name(&self) -> &str {
        "tab.settings.category.respack.title"
    }

    fn description(&self) -> &str {
        "tab.settings.category.respack.description"
    }

    fn ui(&self, ui: &mut Ui, settings: &mut EditorSettings, world: &mut World) -> bool {
        let cache_id = egui::Id::new("respack-cache");

        let packs: Arc<Vec<Cached>> = ui
            .ctx()
            .data_mut(|d| d.get_temp::<Arc<Vec<Cached>>>(cache_id))
            .unwrap_or_else(|| {
                let builtin = RespackEntry::load(RespackSource::Builtin)
                    .expect("built-in resource pack must load");
                let v: Vec<Cached> = std::iter::once(builtin)
                    .chain(scan_respacks(world.resource::<WorkingDirectory>()))
                    .map(|e| {
                        let p = upload_previews(ui.ctx(), &e);
                        (e, p)
                    })
                    .collect();
                let v = Arc::new(v);
                ui.ctx().data_mut(|d| d.insert_temp(cache_id, v.clone()));
                v
            });

        for cached in packs.iter() {
            let (entry, _) = cached;
            let selected = settings.game.respack == entry.source;
            if respack_row_ui(ui, cached, selected) && !selected {
                world.trigger(SelectRespack(entry.source.clone()));
            }
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        let reload_clicked = ui.item(
            t!("tab.settings.category.respack.reload.label"),
            Some(t!("tab.settings.category.respack.reload.description")),
            |ui| {
                ui.button(t!("tab.settings.category.respack.reload.button"))
                    .clicked()
            },
        );
        if reload_clicked {
            ui.ctx()
                .data_mut(|d| d.remove::<Arc<Vec<Cached>>>(cache_id));
            world.trigger(ReloadRespack);
        }

        ui.separator();

        let respack_dir = world.resource::<WorkingDirectory>().respacks();
        let path_text = match &respack_dir {
            Ok(p) => p.display().to_string(),
            Err(e) => e.to_string(),
        };
        let open_clicked = ui.item(
            t!("tab.settings.category.respack.open_folder.label"),
            Some(RichText::new(path_text)),
            |ui| {
                ui.button(t!("tab.settings.category.respack.open_folder.button"))
                    .clicked()
            },
        );
        if open_clicked {
            if let Ok(path) = respack_dir {
                let _ = open::that(path);
            }
        }

        // selection changes are persisted by the `SelectRespack` handler, not here.
        false
    }
}

fn upload_previews(ctx: &Context, entry: &RespackEntry) -> [TextureHandle; 4] {
    let key = entry.source.path().display().to_string();
    let p = &entry.preview;
    let upload = |suffix: &str, img: &image::DynamicImage| {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let color =
            egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
        ctx.load_texture(
            format!("respack/{key}/{suffix}"),
            color,
            TextureOptions::LINEAR,
        )
    };
    [
        upload("tap", &p.tap),
        upload("drag", &p.drag),
        upload("flick", &p.flick),
        upload("hold", &p.hold),
    ]
}

fn respack_row_ui(ui: &mut Ui, cached: &Cached, selected: bool) -> bool {
    let (entry, previews) = cached;
    let locale = rust_i18n::locale();

    let title = if !entry.meta.name.is_empty() {
        entry.meta.name.get(&locale).to_owned()
    } else if let RespackSource::Custom(path) = &entry.source {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_owned()
    } else {
        String::new()
    };

    button_frame(ui, selected, |ui, text_color| {
        ui.set_width(ui.available_width());

        let description_color = text_color.linear_multiply(0.7);
        let description = if entry.meta.description.is_empty() {
            RichText::new(t!("tab.settings.category.respack.no_description"))
                .italics()
                .color(description_color)
        } else {
            RichText::new(entry.meta.description.get(&locale).to_owned()).color(description_color)
        }
        .size(11.0);

        ui.sides(
            |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(&title).color(text_color).strong());
                    ui.label(description);
                });
            },
            |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // iterate in reverse so tap/drag/flick/hold appear left→right.
                    for tex in previews.iter().rev() {
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::splat(PREVIEW_SIZE), Sense::hover());
                        let size = tex.size_vec2();
                        let scale = (PREVIEW_SIZE / size.x).min(PREVIEW_SIZE / size.y).min(1.0);
                        let draw = Rect::from_center_size(rect.center(), size * scale);
                        Image::new(tex).paint_at(ui, draw);
                    }
                });
            },
        );
    })
    .clicked()
}
