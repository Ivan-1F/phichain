use bevy::prelude::*;

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

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ImageAssets>()
            .add_systems(Startup, load_assets);
    }
}

fn load_assets(mut image_assets: ResMut<ImageAssets>, asset_server: Res<AssetServer>) {
    *image_assets = ImageAssets {
        tap: asset_server.load("image/tap.png"),
        drag: asset_server.load("image/drag.png"),
        hold: asset_server.load("image/hold.png"),
        flick: asset_server.load("image/flick.png"),
        tap_highlight: asset_server.load("image/tap.highlight.png"),
        drag_highlight: asset_server.load("image/drag.highlight.png"),
        hold_highlight: asset_server.load("image/hold.highlight.png"),
        flick_highlight: asset_server.load("image/flick.highlight.png"),
        line: asset_server.load("image/judgeline.png"),
    };
}
