pub use phichain_chart::constants::*;

pub const INDICATOR_POSITION: f32 = 0.85;
pub const BASE_ZOOM: f32 = 400.0;

/// Opacity of pending (being-placed) notes and events, shared by the timelines and the game preview
pub const PENDING_OPACITY: f32 = 0.5;

/// Fill opacity of the drag-selection marquee on timelines
pub const DRAG_SELECTION_OPACITY: f32 = 0.2;

/// Opacity of the horizontal beat grid lines on timelines
pub const BEAT_LINE_OPACITY: f32 = 40.0 / 255.0;

/// Opacity of the vertical lane separator lines on timelines
pub const LANE_LINE_OPACITY: f32 = 40.0 / 255.0;

pub const IS_BETA: bool = true;
