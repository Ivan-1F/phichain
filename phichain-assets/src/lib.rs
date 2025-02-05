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
        app.add_systems(Startup, load_assets_system);
    }
}

#[cfg(feature = "egui")]
fn load_assets_system(mut egui_context: bevy_egui::EguiContexts, image_assets: Res<ImageAssets>) {
    egui_context.add_image(image_assets.tap.clone());
    egui_context.add_image(image_assets.drag.clone());
    egui_context.add_image(image_assets.hold.clone());
    egui_context.add_image(image_assets.flick.clone());
    egui_context.add_image(image_assets.tap_highlight.clone());
    egui_context.add_image(image_assets.drag_highlight.clone());
    egui_context.add_image(image_assets.hold_highlight.clone());
    egui_context.add_image(image_assets.flick_highlight.clone());
}
