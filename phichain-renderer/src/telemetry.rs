use crate::args::{Args, Codec, MsaaLevel};
use crate::encoder::pick_encoder;
use bevy::app::AppExit;
use bevy::prelude::Resource;
use phichain_chart::metrics::ChartMetrics;
use phichain_i18n::locale;
use phichain_telemetry::adapter::Adapter;
use phichain_telemetry::hardware::Hardware;
use phichain_telemetry::payload::{PhichainMeta, TelemetryPayload};
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

/// Everything the payload needs.
///
/// This struct is filled incrementally during the execution, and got finalized when rendering finishes
/// It will then be used as the telemetry payload
#[derive(Default, Clone)]
struct Inner {
    metadata: Metadata,
    hardware: Option<Hardware>,
    adapter: Option<Adapter>,
}

#[derive(Resource, Clone)]
pub struct Shared(Arc<Mutex<Inner>>);

impl Shared {
    pub fn new(args: &Args) -> Self {
        Self(Arc::new(Mutex::new(Inner {
            metadata: initial(args),
            ..Default::default()
        })))
    }

    pub fn update<F: FnOnce(&mut Metadata)>(&self, f: F) {
        if let Ok(mut guard) = self.0.lock() {
            f(&mut guard.metadata);
        }
    }

    pub fn set_hardware(&self, value: Hardware) {
        if let Ok(mut guard) = self.0.lock() {
            guard.hardware = Some(value);
        }
    }

    pub fn set_adapter(&self, value: Adapter) {
        if let Ok(mut guard) = self.0.lock() {
            guard.adapter = Some(value);
        }
    }

    fn snapshot(&self) -> Inner {
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

    let Inner {
        mut metadata,
        hardware,
        adapter,
    } = shared.snapshot();

    metadata.success = exit.is_success();
    if !metadata.success && metadata.error_kind.is_none() {
        metadata.error_kind = Some("UnknownExit");
    }

    let mut builder = TelemetryPayload::builder()
        .reporter("phichain-renderer")
        .event_type(EVENT_TYPE)
        .maybe_device_id(phichain_telemetry::device::get_device_id())
        .phichain(PhichainMeta::new(
            env!("CARGO_PKG_VERSION"),
            cfg!(debug_assertions),
        ))
        .metadata(serde_json::to_value(&metadata).unwrap());

    if let Some(hw) = hardware {
        builder = builder.extra("hardware", hw);
    }
    if let Some(a) = adapter {
        builder = builder.extra("adapter", a);
    }

    let _ = phichain_telemetry::send(&builder.build());
}
