use bevy_kira_audio::prelude::Decibels;

/// Convert an amplitude value (e.g. 0.0~1.2) to decibels
/// Returns silence for amplitude <= 0.0.
pub fn amplitude_to_db(amplitude: f32) -> Decibels {
    if amplitude <= 0.0 {
        Decibels::SILENCE
    } else {
        Decibels(20.0 * amplitude.log10())
    }
}
