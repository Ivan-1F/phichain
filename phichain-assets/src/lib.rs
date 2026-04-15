use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;
use std::env;
use std::path::PathBuf;

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "image/tap.png")]
    pub tap: Handle<Image>,
    #[asset(path = "image/drag.png")]
    pub drag: Handle<Image>,
    #[asset(path = "image/hold.png")]
    pub hold: Handle<Image>,
    #[asset(path = "image/flick.png")]
    pub flick: Handle<Image>,
    #[asset(path = "image/tap.highlight.png")]
    pub tap_highlight: Handle<Image>,
    #[asset(path = "image/drag.highlight.png")]
    pub drag_highlight: Handle<Image>,
    #[asset(path = "image/hold.highlight.png")]
    pub hold_highlight: Handle<Image>,
    #[asset(path = "image/hold_head.png")]
    pub hold_head: Handle<Image>,
    #[asset(path = "image/hold_head.highlight.png")]
    pub hold_head_highlight: Handle<Image>,
    #[asset(path = "image/hold_tail.png")]
    pub hold_tail: Handle<Image>,
    #[asset(path = "image/flick.highlight.png")]
    pub flick_highlight: Handle<Image>,
    #[asset(path = "image/line.png")]
    pub line: Handle<Image>,
    #[asset(path = "image/hit.png")]
    pub hit: Handle<Image>,
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
        app.init_collection::<ImageAssets>()
            .init_collection::<AudioAssets>();

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
    mut images: ResMut<Assets<Image>>,
    egui_images: Option<Res<EguiImageAssets>>,
) {
    if egui_images.is_some() {
        return;
    }

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
        hold: make_premultiplied(&image_assets.hold),
        flick: make_premultiplied(&image_assets.flick),
        tap_highlight: make_premultiplied(&image_assets.tap_highlight),
        drag_highlight: make_premultiplied(&image_assets.drag_highlight),
        hold_highlight: make_premultiplied(&image_assets.hold_highlight),
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
