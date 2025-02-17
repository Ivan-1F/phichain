use crate::constants;
use crate::settings::EditorSettings;
use bevy::app::App;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, SystemInfo};
use bevy::ecs::entity::Entities;
use bevy::log::{debug, error, info};
use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::time::common_conditions::on_timer;
use bevy_mod_reqwest::{BevyReqwest, ReqwestErrorEvent, ReqwestResponseEvent};
use bevy_persistent::Persistent;
use serde_json::{json, Value};
use std::path::Path;
use std::time::Duration;
use std::{env, process};
use sysinfo::Pid;
use uuid::Uuid;

const TELEMETRY_URL: &str = "https://telemetry.phichain.rs/report";
const TELEMETRY_REPORT_TIMEOUT: Duration = Duration::from_secs(15);
const TELEMETRY_REPORT_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Resource)]
pub struct TelemetryManager {
    uuid: Uuid,
    queue: Vec<Value>,
}

impl TelemetryManager {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            queue: vec![],
        }
    }
}

pub struct TelemetryPlugin;

impl Plugin for TelemetryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TelemetryManager::new())
            .add_event::<PushTelemetryEvent>()
            .add_systems(Update, handle_push_telemetry_event_system)
            .add_systems(
                Update,
                flush_telemetry_queue_system.run_if(on_timer(TELEMETRY_REPORT_INTERVAL)),
            )
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

fn is_ci() -> bool {
    env::var("CI").is_ok()
}

fn container_environment() -> Option<&'static str> {
    // check for Kubernetes
    if env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        return Some("kubernetes");
    }

    // check for Docker by detecting the presence of known files
    if Path::new("/.dockerenv").exists() || Path::new("/run/.dockerenv").exists() {
        return Some("docker");
    }

    // alternatively, check for the "container" environment variable set to "docker"
    if let Ok(container) = env::var("container") {
        if container.to_lowercase() == "docker" {
            return Some("docker");
        }
    }

    // check for Podman by detecting the presence of a Podman-specific file
    if Path::new("/run/.containerenv").exists() {
        return Some("podman");
    }
    // alternatively, check if the "container" environment variable indicates Podman
    if let Ok(container) = env::var("container") {
        if container.to_lowercase() == "podman" {
            return Some("podman");
        }
    }

    None
}

fn telemetry_enabled() -> bool {
    !matches!(
        env::var("PHICHAIN_TELEMETRY_DISABLED")
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
        "true" | "yes" | "1"
    ) && !matches!(
        env::var("DO_NOT_TRACK")
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
        "true" | "yes" | "1"
    )
}

fn telemetry_debug() -> bool {
    matches!(
        env::var("PHICHAIN_TELEMETRY_DEBUG")
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
        "true" | "yes" | "1"
    )
}

fn handle_push_telemetry_event_system(
    mut events: EventReader<PushTelemetryEvent>,
    diagnostics: Res<DiagnosticsStore>,
    system_info: Res<SystemInfo>,
    adapter_info: Res<RenderAdapterInfo>,
    editor_settings: Res<Persistent<EditorSettings>>,
    entities: &Entities,
    time: Res<Time>,
    mut telemetry_manager: ResMut<TelemetryManager>,
) {
    for event in events.read() {
        let mut fps = 0.0;
        if let Some(value) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|d| d.value())
        {
            fps = value;
        }

        let mut system = sysinfo::System::new_all();
        system.refresh_all();

        let pid = process::id();
        let process = system.process(Pid::from_u32(pid)).unwrap();

        let info = os_info::get();

        let payload = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "reporter": "phichain-editor",
            "uuid": telemetry_manager.uuid,
            "type": event.event_type,
            "system": {
                "arch": env::consts::ARCH,
                "os": env::consts::OS,
                "name": &info.os_type().to_string(),
                "version": &info.version().to_string(),
                "family": env::consts::FAMILY,
                "bitness": info.bitness().to_string(),
                "kernel": system_info.kernel.trim(),
            },
            "hardware": {
                "cpu": system.cpus().first().unwrap().brand(),
                "core_count": system.cpus().len(),
                "memory": system.total_memory(),
                "memory_formatted": format!("{:.1} GiB", system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0),
            },
            "adapter": &***adapter_info,
            "environment": {
                "container": container_environment().unwrap_or("none"),
                "ci": is_ci(),
                "test": cfg!(test),
            },
            "phichain": {
                "beta": constants::IS_BETA,
                "version": env!("CARGO_PKG_VERSION"),
                "debug": cfg!(debug_assertions),
            },
            "performance": {
                "fps": fps,
                "entities": entities.len(),
                "cpu": process.cpu_usage(),
                "memory": process.memory(),
            },
            "config": **editor_settings,
            "uptime": time.elapsed().as_secs_f32(),

            "metadata": event.metadata,
        });

        if telemetry_debug() {
            info!(
                "[telemetry] {}",
                serde_json::to_string_pretty(&payload).unwrap()
            );
        }

        telemetry_manager.queue.push(payload);
    }
}

fn startup_system(mut events: EventWriter<PushTelemetryEvent>) {
    events.send(PushTelemetryEvent::new(
        "phichain.editor.started",
        json!({}),
    ));
}

fn flush_telemetry_queue_system(
    mut reqwest: BevyReqwest,
    telemetry_manager: Res<TelemetryManager>,
) {
    if !telemetry_enabled() {
        return;
    }

    let data = telemetry_manager.queue.clone();
    if data.is_empty() {
        return;
    }

    debug!("Flushing telemetry queue with {} entries", data.len());

    let request = reqwest
        .post(TELEMETRY_URL)
        .timeout(TELEMETRY_REPORT_TIMEOUT)
        .json(&data)
        .build()
        .unwrap();

    reqwest
        .send(request)
        .on_response(
            |trigger: Trigger<ReqwestResponseEvent>,
             mut telemetry_manager: ResMut<TelemetryManager>| {
                let response = trigger.event();
                if response.status().is_success() {
                    info!(
                        "Successfully sent telemetry event, response: {}",
                        response.as_str().unwrap_or("<unknown>")
                    );
                    telemetry_manager.queue.clear();
                } else {
                    error!(
                        "Failed to send telemetry data, bad response, status code: {}",
                        response.status()
                    );
                }
            },
        )
        .on_error(|trigger: Trigger<ReqwestErrorEvent>| {
            let e = &trigger.event().0;
            error!("Failed to send telemetry data, request failed: {:?}", e);
        });
}
