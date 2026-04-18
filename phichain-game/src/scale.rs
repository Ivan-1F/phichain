//! Note size calculation.
//!
//! Notes must render at a consistent visual size regardless of viewport
//! dimensions or the active resource pack's texture resolution. Helpers here
//! normalize a texture's pixel size into "rendered units" (world units for the
//! game viewport, egui units for the editor timeline).
//!
//! The reference is the non-highlighted tap texture of the active pack
//! ([`phichain_assets::RespackDimensions::note_width`]). Every other texture
//! is scaled in proportion, so highlighted textures (with extra glow) render
//! wider without shrinking the core note.

/// Fraction of the viewport occupied by a reference note at `note_scale = 1.0`.
const NOTE_WIDTH_RATIO: f32 = 989.0 / 8000.0;

/// Rendered units per texture pixel.
///
/// Multiply by a texture's pixel size to get its rendered size in the same
/// units as `viewport_width`. `pack_note_width` is the active pack's reference
/// non-highlighted tap pixel width.
pub fn texel_unit(viewport_width: f32, pack_note_width: f32, user_scale: f32) -> f32 {
    NOTE_WIDTH_RATIO * viewport_width * user_scale / pack_note_width
}

/// World scale of the line entity. Children (note sprites) inherit this.
pub fn line_world_scale(viewport_width: f32) -> f32 {
    viewport_width * 3.0 / 1920.0
}

/// Local `Transform` scale for a note sprite. Combined with the line parent's
/// world scale, the rendered size matches `texel_unit` times the texture size.
pub fn note_sprite_local_scale(viewport_width: f32, pack_note_width: f32, user_scale: f32) -> f32 {
    texel_unit(viewport_width, pack_note_width, user_scale) / line_world_scale(viewport_width)
}
