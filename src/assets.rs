use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_egui::EguiContexts;
use bevy_kira_audio::AudioSource;

#[derive(Resource, Debug, Default)]
pub struct ImageAssets {
    pub tap: Handle<Image>,
    pub drag: Handle<Image>,
    pub hold: Handle<Image>,
    pub flick: Handle<Image>,
    pub tap_highlight: Handle<Image>,
    pub drag_highlight: Handle<Image>,
    pub hold_highlight: Handle<Image>,
    pub flick_highlight: Handle<Image>,
    pub line: Handle<Image>,
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

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ImageAssets>()
            .add_systems(Startup, load_assets)
            .init_collection::<AudioAssets>();
    }
}

fn load_assets(
    mut egui_context: EguiContexts,
    mut image_assets: ResMut<ImageAssets>,
    asset_server: Res<AssetServer>,
) {
    *image_assets = ImageAssets {
        tap: asset_server.load("image/tap.png"),
        drag: asset_server.load("image/drag.png"),
        hold: asset_server.load("image/hold.png"),
        flick: asset_server.load("image/flick.png"),
        tap_highlight: asset_server.load("image/tap.highlight.png"),
        drag_highlight: asset_server.load("image/drag.highlight.png"),
        hold_highlight: asset_server.load("image/hold.highlight.png"),
        flick_highlight: asset_server.load("image/flick.highlight.png"),
        line: asset_server.load("image/line.png"),
    };

    // TODO: optimize this
    egui_context.add_image(image_assets.tap.clone());
    egui_context.add_image(image_assets.drag.clone());
    egui_context.add_image(image_assets.hold.clone());
    egui_context.add_image(image_assets.flick.clone());
}
