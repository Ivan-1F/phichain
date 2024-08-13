use std::env;
use std::path::PathBuf;

// https://github.com/bevyengine/bevy/blob/91fa4bb64905121a18b40df0062dbd85714aa3ce/crates/bevy_asset/src/io/file/mod.rs#L18
pub fn get_base_path() -> PathBuf {
    if let Ok(asset_root) = env::var("BEVY_ASSET_ROOT") {
        PathBuf::from(asset_root)
    } else if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        env::current_exe()
            .map(|path| path.parent().map(ToOwned::to_owned).unwrap())
            .unwrap()
    }
}
