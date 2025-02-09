use crate::settings::EditorSettings;
use bevy::app::App;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, SystemInfo};
use bevy::ecs::entity::Entities;
use bevy::prelude::{
    Event, EventReader, EventWriter, Plugin, Res, Resource, Startup, Time, Update,
};
use bevy::render::renderer::RenderAdapterInfo;
use bevy_persistent::Persistent;
use serde_json::{json, Value};
use uuid::Uuid;
use crate::constants;

#[derive(Debug, Clone, Resource)]
pub struct TelemetryManager {
    uuid: Uuid,
}

impl TelemetryManager {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

pub struct TelemetryPlugin;

impl Plugin for TelemetryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TelemetryManager::new())
            .add_event::<PushTelemetryEvent>()
            .add_systems(Update, handle_push_telemetry_event_system)
            .add_systems(Startup, startup_system);
    }
}

#[derive(Debug, Clone, Event)]
pub struct PushTelemetryEvent {
    event_type: &'static str,
    metadata: Value,
}

impl PushTelemetryEvent {
    pub fn new(event_type: &'static str, metadata: Value) -> Self {
        Self {
            event_type,
            metadata,
        }
    }
}

fn handle_push_telemetry_event_system(
    mut events: EventReader<PushTelemetryEvent>,
    diagnostics: Res<DiagnosticsStore>,
    system_info: Res<SystemInfo>,
    adapter_info: Res<RenderAdapterInfo>,
    editor_settings: Res<Persistent<EditorSettings>>,
    entities: &Entities,
    time: Res<Time>,
    telemetry_manager: Res<TelemetryManager>,
) {
    for event in events.read() {
        let mut fps = 0.0;
        if let Some(value) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|d| d.value())
        {
            fps = value;
        }

        let payload = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "reporter": "phichain-editor",
            "uuid": telemetry_manager.uuid,
            "type": event.event_type,
            "system": {
                "os": system_info.os.trim(),
                "kernel": system_info.kernel.trim(),
                "cpu": system_info.cpu.trim(),
                "core_count": system_info.core_count.trim(),
                "memory": system_info.memory.trim(),
            },
            "adapter": &***adapter_info,
            "environment": {
                "container": "none",  // TODO
                "ci": false,  // TODO
            },
            "phichain": {
                "beta": constants::IS_BETA,
                "version": env!("CARGO_PKG_VERSION"),
                "debug": cfg!(debug_assertions),
            },
            "performance": {
                "fps": fps,
                "entities": entities.len(),
                "cpu": 0.5,  // TODO
                "memory": 1024,  // TODO
            },
            "config": **editor_settings,
            "uptime": time.elapsed().as_secs_f32(),

            "metadata": event.metadata,
        });

        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    }
}

fn startup_system(mut events: EventWriter<PushTelemetryEvent>) {
    events.send(PushTelemetryEvent::new("start", json!({})));
}
