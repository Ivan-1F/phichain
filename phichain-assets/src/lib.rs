use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "image/tap.png")]
    pub tap: Handle<Image>,
    #[asset(path = "image/drag.png")]
    pub drag: Handle<Image>,
    /// Combined hold texture (top to bottom: tail | body | head).
    /// Split into separate parts by [`HoldAtlas`] after loading.
    #[asset(path = "image/hold.png")]
    pub hold: Handle<Image>,
    #[asset(path = "image/flick.png")]
    pub flick: Handle<Image>,
    #[asset(path = "image/tap.highlight.png")]
    pub tap_highlight: Handle<Image>,
    #[asset(path = "image/drag.highlight.png")]
    pub drag_highlight: Handle<Image>,
    /// Combined hold highlight texture (top to bottom: tail | body | head).
    /// Split into separate parts by [`HoldAtlas`] after loading.
    #[asset(path = "image/hold.highlight.png")]
    pub hold_highlight: Handle<Image>,
    #[asset(path = "image/flick.highlight.png")]
    pub flick_highlight: Handle<Image>,
    #[asset(path = "image/line.png")]
    pub line: Handle<Image>,
    #[asset(path = "image/hit.png")]
    pub hit: Handle<Image>,
}

/// Resource pack configuration parsed from `info.yml`.
///
/// Compatible with Phi Recorder and Phira resource pack formats.
/// Unknown fields are silently ignored via `#[serde(deny_unknown_fields)]` being absent.
#[derive(Debug, Clone, Resource, Deserialize)]
#[serde(default)]
pub struct ResPackInfo {
    pub name: String,
    pub author: String,
    pub description: String,
    #[serde(rename = "holdAtlas")]
    pub hold_atlas: [u32; 2],
    #[serde(rename = "holdAtlasMH")]
    pub hold_atlas_mh: [u32; 2],
    #[serde(rename = "hideParticles")]
    pub hide_particles: bool,
    #[serde(rename = "holdRepeat")]
    pub hold_repeat: bool,
}

impl Default for ResPackInfo {
    fn default() -> Self {
        Self {
            name: "Phichain Default".to_owned(),
            author: "Phichain".to_owned(),
            description: String::new(),
            hold_atlas: [50, 50],
            hold_atlas_mh: [0, 110],
            hide_particles: false,
            hold_repeat: false,
        }
    }
}

/// Hold texture parts split from the combined hold image.
#[derive(Resource)]
pub struct HoldParts {
    pub body: Handle<Image>,
    pub head: Handle<Image>,
    pub tail: Handle<Image>,
    pub body_highlight: Handle<Image>,
    pub head_highlight: Handle<Image>,
    pub tail_highlight: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/click.ogg")]
    pub click: Handle<AudioSource>,
    #[asset(path = "audio/drag.ogg")]
    pub drag: Handle<AudioSource>,
    #[asset(path = "audio/flick.ogg")]
    pub flick: Handle<AudioSource>,
    #[asset(path = "audio/metronome.wav")]
    pub metronome: Handle<AudioSource>,
}

/// Setup bevy asset root environment variable
///
/// In debug environment, it will be the parent of `CARGO_MANIFEST_DIR`, aka phichain project root
///
/// In production environment, it will be `CARGO_MANIFEST_DIR`
///
/// This value can be overwritten using the `PHICHAIN_ASSET_ROOT` environment variable
pub fn setup_assets() {
    let asset_root = match env::var("PHICHAIN_ASSET_ROOT") {
        Ok(phichain_asset_root) => PathBuf::from(phichain_asset_root),
        Err(_) => {
            #[cfg(debug_assertions)]
            {
                let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                manifest.parent().expect("Failed to get root path").into()
            }

            #[cfg(not(debug_assertions))]
            {
                env::current_exe()
                    .expect("Failed to get path to the current exe")
                    .parent()
                    .map(ToOwned::to_owned)
                    .expect("Failed to get parent path of the current exe")
            }
        }
    };

    env::set_var("BEVY_ASSET_ROOT", asset_root);
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        let asset_root = env::var("BEVY_ASSET_ROOT")
            .map(PathBuf::from)
            .expect("BEVY_ASSET_ROOT should be set by setup_assets()");
        let info_path = asset_root.join("assets/image/info.yml");
        let info: ResPackInfo = std::fs::read_to_string(&info_path)
            .ok()
            .and_then(|data| serde_yaml::from_str(&data).ok())
            .unwrap_or_default();

        app.insert_resource(info)
            .init_collection::<ImageAssets>()
            .init_collection::<AudioAssets>()
            .add_systems(
                Update,
                split_hold_textures_system.run_if(not(resource_exists::<HoldParts>)),
            );

        #[cfg(feature = "egui")]
        app.add_systems(bevy_egui::EguiPrimaryContextPass, setup_egui_images_system);
    }
}

/// Premultiplied-alpha copies of image assets for correct egui rendering.
///
/// Since bevy_egui 0.39.1, the fragment shader no longer premultiplies alpha for
/// user textures, but the blend mode is still `PREMULTIPLIED_ALPHA_BLENDING`.
/// User textures (loaded by Bevy as straight alpha) must be premultiplied before
/// registration. Bevy's sprite renderer uses straight-alpha blending, so we keep
/// separate premultiplied copies here for egui use only.
///
/// See: https://github.com/vladbat00/bevy_egui/pull/465
/// See: https://github.com/vladbat00/bevy_egui/releases/tag/v0.39.1
#[cfg(feature = "egui")]
#[derive(Resource, Default)]
pub struct EguiImageAssets {
    pub tap: Handle<Image>,
    pub drag: Handle<Image>,
    pub hold: Handle<Image>,
    pub flick: Handle<Image>,
    pub tap_highlight: Handle<Image>,
    pub drag_highlight: Handle<Image>,
    pub hold_highlight: Handle<Image>,
    pub flick_highlight: Handle<Image>,
}

#[cfg(feature = "egui")]
fn setup_egui_images_system(
    mut commands: Commands,
    mut egui_context: bevy_egui::EguiContexts,
    image_assets: Res<ImageAssets>,
    hold_parts: Option<Res<HoldParts>>,
    mut images: ResMut<Assets<Image>>,
    egui_images: Option<Res<EguiImageAssets>>,
) {
    if egui_images.is_some() {
        return;
    }

    let Some(hold_parts) = hold_parts else {
        return;
    };

    let mut make_premultiplied = |handle: &Handle<Image>| -> Handle<Image> {
        let src = images.get(handle).expect("image should be loaded");
        let mut copy = src.clone();
        premultiply_alpha(&mut copy);
        let copy_handle = images.add(copy);
        egui_context.add_image(bevy_egui::EguiTextureHandle::Strong(copy_handle.clone()));
        copy_handle
    };

    let egui_assets = EguiImageAssets {
        tap: make_premultiplied(&image_assets.tap),
        drag: make_premultiplied(&image_assets.drag),
        hold: make_premultiplied(&hold_parts.body),
        flick: make_premultiplied(&image_assets.flick),
        tap_highlight: make_premultiplied(&image_assets.tap_highlight),
        drag_highlight: make_premultiplied(&image_assets.drag_highlight),
        hold_highlight: make_premultiplied(&hold_parts.body_highlight),
        flick_highlight: make_premultiplied(&image_assets.flick_highlight),
    };

    commands.insert_resource(egui_assets);
}

/// Premultiply alpha in-place in linear space for correct GPU blending.
/// See [`EguiImageAssets`] for context.
#[cfg(feature = "egui")]
fn premultiply_alpha(image: &mut Image) {
    let data = image.data.as_mut().expect("image should have data");
    for chunk in data.chunks_exact_mut(4) {
        let r = chunk[0] as f32 / 255.0;
        let g = chunk[1] as f32 / 255.0;
        let b = chunk[2] as f32 / 255.0;
        let a = chunk[3] as f32 / 255.0;
        // Premultiply in linear space for correct blending
        let r_lin = r.powf(2.2) * a;
        let g_lin = g.powf(2.2) * a;
        let b_lin = b.powf(2.2) * a;
        chunk[0] = (r_lin.powf(1.0 / 2.2) * 255.0) as u8;
        chunk[1] = (g_lin.powf(1.0 / 2.2) * 255.0) as u8;
        chunk[2] = (b_lin.powf(1.0 / 2.2) * 255.0) as u8;
    }
}

/// Split a combined hold texture into tail, body, and head parts.
///
/// The layout is top-to-bottom: tail (atlas[0] pixels) | body | head (atlas[1] pixels).
fn split_hold_image(image: &Image, atlas: [u32; 2]) -> (Image, Image, Image) {
    let w = image.width();
    let h = image.height();
    let [tail_h, head_h] = atlas;
    let body_h = h - tail_h - head_h;
    let bpp = 4u32; // RGBA8
    let row = w * bpp;

    let data = image.data.as_ref().expect("image should have data");

    let crop = |y_start: u32, height: u32| -> Image {
        if height == 0 {
            // 1x1 transparent pixel placeholder
            return Image::new(
                Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![0, 0, 0, 0],
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
        }
        let start = (y_start * row) as usize;
        let end = start + (height * row) as usize;
        Image::new(
            Extent3d {
                width: w,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data[start..end].to_vec(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
    };

    let tail = crop(0, tail_h);
    let body = crop(tail_h, body_h);
    let head = crop(tail_h + body_h, head_h);

    (tail, body, head)
}

fn split_hold_textures_system(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    mut images: ResMut<Assets<Image>>,
    info: Res<ResPackInfo>,
) {
    let Some(hold_image) = images.get(&image_assets.hold) else {
        return;
    };
    let (tail, body, head) = split_hold_image(hold_image, info.hold_atlas);

    let Some(hold_hl_image) = images.get(&image_assets.hold_highlight) else {
        return;
    };
    let (tail_hl, body_hl, head_hl) = split_hold_image(hold_hl_image, info.hold_atlas_mh);

    commands.insert_resource(HoldParts {
        body: images.add(body),
        head: images.add(head),
        tail: images.add(tail),
        body_highlight: images.add(body_hl),
        head_highlight: images.add(head_hl),
        tail_highlight: images.add(tail_hl),
    });
}
