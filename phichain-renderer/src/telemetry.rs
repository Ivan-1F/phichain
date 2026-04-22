use crate::args::{Args, Codec, MsaaLevel};
use crate::encoder::pick_encoder;
use bevy::app::AppExit;
use bevy::prelude::Resource;
use phichain_chart::metrics::ChartMetrics;
use phichain_i18n::locale;
use phichain_telemetry::Reporter;
use serde::Serialize;
use std::sync::{Arc, Mutex};

const EVENT_TYPE: &str = "phichain.renderer.render";

#[derive(Debug, Clone, Default, Serialize)]
pub struct VideoMeta {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub msaa: Option<MsaaLevel>,
    pub codec: Option<Codec>,
    pub hwaccel: bool,
    pub encoder_name: &'static str,
    pub quality_mode: &'static str,
    pub crf: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GameMeta {
    pub note_scale: f32,
    pub fc_ap_indicator: bool,
    pub multi_highlight: bool,
    pub hide_hit_effect: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Metadata {
    pub locale: String,
    pub from_sec: Option<f32>,
    pub to_sec: Option<f32>,
    pub music_duration_sec: Option<f32>,
    pub respack_used: bool,
    pub video: VideoMeta,
    pub game: GameMeta,
    pub chart: Option<ChartMetrics>,
    pub success: bool,
    pub error_kind: Option<&'static str>,
    pub duration_ms: u64,
    pub frames_written: u32,
    pub avg_fps: f32,
    pub realtime_factor: f32,
}

/// Shared handle so Bevy systems and the main thread can both mutate `Metadata` while the app is running.
#[derive(Resource, Clone)]
pub struct Shared(Arc<Mutex<Metadata>>);

impl Shared {
    pub fn new(args: &Args) -> Self {
        Self(Arc::new(Mutex::new(initial(args))))
    }

    pub fn update<F: FnOnce(&mut Metadata)>(&self, f: F) {
        if let Ok(mut guard) = self.0.lock() {
            f(&mut guard);
        }
    }

    fn snapshot(&self) -> Metadata {
        self.0.lock().expect("telemetry lock poisoned").clone()
    }
}

fn initial(args: &Args) -> Metadata {
    let (quality_mode, crf) = if args.video.bitrate.is_some() {
        ("bitrate", None)
    } else {
        ("crf", Some(args.video.crf))
    };

    Metadata {
        locale: locale(),
        from_sec: args.from,
        to_sec: args.to,
        respack_used: args.respack.is_some(),
        video: VideoMeta {
            width: args.video.width,
            height: args.video.height,
            fps: args.video.fps,
            msaa: Some(args.video.msaa),
            codec: Some(args.video.codec),
            hwaccel: args.video.hwaccel,
            encoder_name: pick_encoder(args.video.codec, args.video.hwaccel),
            quality_mode,
            crf,
        },
        game: GameMeta {
            note_scale: args.game.note_scale,
            fc_ap_indicator: args.game.fc_ap_indicator,
            multi_highlight: !args.game.no_multi_highlight,
            hide_hit_effect: args.game.hide_hit_effect,
        },
        ..Default::default()
    }
}

pub fn report(shared: &Shared, exit: AppExit) {
    if phichain_telemetry::env::telemetry_disabled() {
        return;
    }

    let mut meta = shared.snapshot();
    meta.success = exit.is_success();
    if !meta.success && meta.error_kind.is_none() {
        meta.error_kind = Some("UnknownExit");
    }

    let reporter = Reporter::new(
        "phichain-renderer",
        env!("CARGO_PKG_VERSION"),
        cfg!(debug_assertions),
    );
    let _ = reporter.track(EVENT_TYPE, serde_json::to_value(&meta).unwrap());
}
