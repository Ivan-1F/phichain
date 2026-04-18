pub mod loader;
pub mod meta;

use std::env;
use std::path::PathBuf;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_kira_audio::prelude::StaticSoundData;
use bevy_kira_audio::AudioSource;
use image::DynamicImage;

pub use crate::loader::{
    load_respack, load_respack_from_dir, load_respack_from_zip, LoadedAudio, LoadedImages,
    LoadedRespack,
};
pub use crate::meta::RespackMeta;

/// Game-side image handles sourced from the active resource pack.
#[derive(Resource)]
pub struct ImageAssets {
    pub tap: Handle<Image>,
    pub tap_highlight: Handle<Image>,
    pub drag: Handle<Image>,
    pub drag_highlight: Handle<Image>,
    pub flick: Handle<Image>,
    pub flick_highlight: Handle<Image>,
    pub hit: Handle<Image>,
    pub line: Handle<Image>,
}

/// Hit sound handles sourced from the active resource pack.
#[derive(Resource)]
pub struct HitSoundAssets {
    pub tap: Handle<AudioSource>,
    pub drag: Handle<AudioSource>,
    pub flick: Handle<AudioSource>,
}

/// Editor-only audio assets that are not part of the resource pack.
#[derive(Resource)]
pub struct EditorAudioAssets {
    pub metronome: Handle<AudioSource>,
}

/// Hold texture parts split from the combined hold image, using [`RespackMeta`]'s atlas config.
#[derive(Resource)]
pub struct HoldParts {
    pub body: Handle<Image>,
    pub head: Handle<Image>,
    pub tail: Handle<Image>,
    pub body_highlight: Handle<Image>,
    pub head_highlight: Handle<Image>,
    pub tail_highlight: Handle<Image>,
}

/// Hit effect sprite atlas derived from the resource pack's `hit.png` and `hit_grid` config.
#[derive(Resource)]
pub struct HitEffectAtlas {
    pub layout: Handle<TextureAtlasLayout>,
    pub frame_count: u32,
    /// Size of a single animation frame in texture pixels.
    pub frame_size: UVec2,
}

#[derive(Resource)]
pub struct RespackDimensions {
    /// Width of a non-hold note texture (tap/drag/flick share the same width).
    pub note_width: f32,
    /// Height of the hold body slice (the middle stretched part between tail and head).
    pub hold_body_height: f32,
}

/// Setup bevy asset root environment variable
///
/// In debug environment, it will be the parent of `CARGO_MANIFEST_DIR`, aka phichain project root
///
/// In production environment, it will be the parent of the current executable
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

/// Return the path to the built-in resource pack directory.
pub fn builtin_respack_dir() -> PathBuf {
    let root =
        env::var("BEVY_ASSET_ROOT").expect("BEVY_ASSET_ROOT should be set by setup_assets()");
    PathBuf::from(root).join("assets/respack")
}

/// Return the path to the editor-only audio directory (metronome, etc.).
fn editor_audio_dir() -> PathBuf {
    let root =
        env::var("BEVY_ASSET_ROOT").expect("BEVY_ASSET_ROOT should be set by setup_assets()");
    PathBuf::from(root).join("assets/audio")
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        load_builtin(app.world_mut())
            .unwrap_or_else(|err| abort_broken_install("built-in resource pack", err));

        #[cfg(feature = "egui")]
        app.add_systems(bevy_egui::EguiPrimaryContextPass, setup_egui_images_system);
    }
}

fn load_builtin(world: &mut World) -> anyhow::Result<()> {
    let pack = load_respack_from_dir(&builtin_respack_dir())?;
    apply_respack(pack, world)?;
    load_editor_audio(world)?;
    Ok(())
}

/// Print a user-facing broken-install error and terminate the process.
///
/// Built-in assets ship with the binary, so a missing or corrupted file here
/// indicates a broken installation rather than a runtime bug. We surface this
/// distinction clearly instead of dumping a panic stack trace.
fn abort_broken_install(what: &str, err: anyhow::Error) -> ! {
    eprintln!();
    eprintln!("error: failed to load {what}");
    eprintln!();
    eprintln!("  {err:#}");
    eprintln!();
    eprintln!("This usually means the phichain installation is incomplete or corrupted.");
    eprintln!("Please reinstall or restore the missing files.");
    eprintln!();
    std::process::exit(1);
}

/// Apply a loaded resource pack to the Bevy world, replacing any previously active pack's
/// resources. Consumer systems automatically pick up new handles on the next frame.
pub fn apply_respack(loaded: LoadedRespack, world: &mut World) -> anyhow::Result<()> {
    let LoadedRespack {
        meta,
        images,
        audio,
    } = loaded;

    // Images
    let (image_assets, hold_parts, hit_atlas, dimensions) = world.resource_scope(
        |world, mut bevy_images: Mut<Assets<Image>>| {
            world.resource_scope(
                |_, mut atlas_layouts: Mut<Assets<TextureAtlasLayout>>| {
                    build_image_resources(images, &meta, &mut bevy_images, &mut atlas_layouts)
                },
            )
        },
    );

    // Audio
    let hit_sound = world.resource_scope(|_, mut sources: Mut<Assets<AudioSource>>| {
        build_hit_sound_assets(audio, &mut sources)
    })?;

    world.insert_resource(meta);
    world.insert_resource(image_assets);
    world.insert_resource(hold_parts);
    world.insert_resource(hit_atlas);
    world.insert_resource(hit_sound);
    world.insert_resource(dimensions);

    // Invalidate egui premultiplied copies so they get rebuilt from the new handles.
    #[cfg(feature = "egui")]
    world.remove_resource::<EguiImageAssets>();

    Ok(())
}

fn build_image_resources(
    images: LoadedImages,
    meta: &RespackMeta,
    bevy_images: &mut Assets<Image>,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> (ImageAssets, HoldParts, HitEffectAtlas, RespackDimensions) {
    let note_width = images.tap.width() as f32;

    let tap = bevy_images.add(dynamic_to_bevy(images.tap));
    let tap_highlight = bevy_images.add(dynamic_to_bevy(images.tap_highlight));
    let drag = bevy_images.add(dynamic_to_bevy(images.drag));
    let drag_highlight = bevy_images.add(dynamic_to_bevy(images.drag_highlight));
    let flick = bevy_images.add(dynamic_to_bevy(images.flick));
    let flick_highlight = bevy_images.add(dynamic_to_bevy(images.flick_highlight));
    let line = bevy_images.add(dynamic_to_bevy(images.line));

    // Split hold into head/body/tail parts based on hold_atlas / hold_highlight_atlas.
    let hold_bevy = dynamic_to_bevy(images.hold);
    let (tail, body, head) = split_hold_image(&hold_bevy, meta.hold_atlas);
    let hold_body_height = body.height() as f32;
    let hold_hl_bevy = dynamic_to_bevy(images.hold_highlight);
    let (tail_hl, body_hl, head_hl) = split_hold_image(&hold_hl_bevy, meta.hold_highlight_atlas);

    let hold_parts = HoldParts {
        body: bevy_images.add(body),
        head: bevy_images.add(head),
        tail: bevy_images.add(tail),
        body_highlight: bevy_images.add(body_hl),
        head_highlight: bevy_images.add(head_hl),
        tail_highlight: bevy_images.add(tail_hl),
    };

    // Build hit effect atlas based on hit_grid.
    let hit_image = dynamic_to_bevy(images.hit);
    let [cols, rows] = meta.hit_grid;
    let frame_size = UVec2::new(hit_image.width() / cols, hit_image.height() / rows);
    let hit = bevy_images.add(hit_image);
    let hit_atlas = HitEffectAtlas {
        layout: atlas_layouts.add(TextureAtlasLayout::from_grid(
            frame_size, cols, rows, None, None,
        )),
        frame_count: cols * rows,
        frame_size,
    };

    (
        ImageAssets {
            tap,
            tap_highlight,
            drag,
            drag_highlight,
            flick,
            flick_highlight,
            hit,
            line,
        },
        hold_parts,
        hit_atlas,
        RespackDimensions {
            note_width,
            hold_body_height,
        },
    )
}

fn build_hit_sound_assets(
    audio: LoadedAudio,
    sources: &mut Assets<AudioSource>,
) -> anyhow::Result<HitSoundAssets> {
    Ok(HitSoundAssets {
        tap: sources.add(decode_audio(audio.tap)?),
        drag: sources.add(decode_audio(audio.drag)?),
        flick: sources.add(decode_audio(audio.flick)?),
    })
}

fn decode_audio(data: Vec<u8>) -> anyhow::Result<AudioSource> {
    use anyhow::Context;
    let sound = StaticSoundData::from_cursor(std::io::Cursor::new(data))
        .context("failed to decode audio")?;
    Ok(AudioSource { sound })
}

fn load_editor_audio(world: &mut World) -> anyhow::Result<()> {
    use anyhow::Context;
    let path = editor_audio_dir().join("metronome.wav");
    let bytes =
        std::fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let source = decode_audio(bytes)?;
    let handle = world.resource_mut::<Assets<AudioSource>>().add(source);
    world.insert_resource(EditorAudioAssets { metronome: handle });
    Ok(())
}

fn dynamic_to_bevy(img: DynamicImage) -> Image {
    let rgba = img.into_rgba8();
    let (w, h) = rgba.dimensions();
    Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
}

/// Split a combined hold texture into tail, body, and head parts.
///
/// The layout is top-to-bottom: tail (atlas[0] pixels) | body | head (atlas[1] pixels).
/// A zero-height part becomes a 1×1 transparent placeholder (wgpu requires non-zero dims).
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
    hold_parts: Res<HoldParts>,
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
        let r_lin = r.powf(2.2) * a;
        let g_lin = g.powf(2.2) * a;
        let b_lin = b.powf(2.2) * a;
        chunk[0] = (r_lin.powf(1.0 / 2.2) * 255.0) as u8;
        chunk[1] = (g_lin.powf(1.0 / 2.2) * 255.0) as u8;
        chunk[2] = (b_lin.powf(1.0 / 2.2) * 255.0) as u8;
    }
}

