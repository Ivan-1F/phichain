use std::sync::Arc;

use crate::misc::WorkingDirectory;
use crate::respack::{scan_respacks, ReloadRespack, RespackEntry};
use crate::settings::EditorSettings;
use crate::tab::settings::SettingCategory;
use bevy::prelude::World;
use egui::{
    Color32, Context, Image, Rect, RichText, Sense, TextureHandle, TextureOptions, Ui, UiBuilder,
    Vec2,
};
use phichain_assets::builtin_respack_dir;

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
                let builtin = RespackEntry::load(builtin_respack_dir())
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

        let mut changed = false;
        for cached in packs.iter() {
            let (entry, _) = cached;
            let key = entry.setting_key();
            let selected = settings.game.respack.as_deref() == key;
            if pack_row(ui, cached, selected) && !selected {
                settings.game.respack = key.map(str::to_owned);
                changed = true;
            }
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(t!("tab.settings.category.respack.reload.label"));
            if ui
                .button(t!("tab.settings.category.respack.reload.button"))
                .clicked()
            {
                ui.ctx()
                    .data_mut(|d| d.remove::<Arc<Vec<Cached>>>(cache_id));
                world.trigger(ReloadRespack);
            }
        });
        ui.label(
            RichText::new(t!("tab.settings.category.respack.reload.description"))
                .weak()
                .size(11.0),
        );

        if changed {
            world.trigger(ReloadRespack);
        }
        changed
    }
}

fn upload_previews(ctx: &Context, entry: &RespackEntry) -> [TextureHandle; 4] {
    let key = entry.path.display().to_string();
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

fn pack_row(ui: &mut Ui, cached: &Cached, selected: bool) -> bool {
    let (entry, previews) = cached;

    let filename = entry.filename();

    let title = if entry.meta.name.is_empty() {
        filename.to_owned()
    } else {
        entry.meta.name.clone()
    };

    let description = if entry.meta.description.is_empty() {
        RichText::new(t!("tab.settings.category.respack.no_description")).italics()
    } else {
        RichText::new(entry.meta.description.clone())
    }
    .weak()
    .size(11.0);

    ui.scope_builder(UiBuilder::new().sense(Sense::click()), |ui| {
        let fill = if selected {
            ui.visuals().selection.bg_fill.linear_multiply(0.35)
        } else {
            Color32::TRANSPARENT
        };
        egui::Frame::new()
            .fill(fill)
            .corner_radius(4)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&title).strong());
                        ui.label(description);
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Iterate in reverse so tap/drag/flick/hold appear left→right.
                        for tex in previews.iter().rev() {
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::splat(PREVIEW_SIZE), Sense::hover());
                            let size = tex.size_vec2();
                            let scale = (PREVIEW_SIZE / size.x).min(PREVIEW_SIZE / size.y).min(1.0);
                            let draw = Rect::from_center_size(rect.center(), size * scale);
                            Image::new(tex).paint_at(ui, draw);
                        }
                    });
                });
            });
    })
    .response
    .clicked()
}
