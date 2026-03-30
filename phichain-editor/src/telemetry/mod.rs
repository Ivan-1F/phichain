use crate::constants;
use crate::settings::EditorSettings;
use bevy::app::App;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::entity::Entities;
use bevy::log::{debug, error, info};
use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::time::common_conditions::on_timer;
use bevy_mod_reqwest::{BevyReqwest, ReqwestErrorEvent, ReqwestResponseEvent};
use bevy_persistent::Persistent;
use phichain_chart::event::LineEvent;
use phichain_chart::line::Line;
use phichain_chart::note::Note;
use phichain_chart::project::Project;
use phichain_telemetry::payload::{PhichainMeta, TelemetryPayload};
use serde_json::{json, Value};
use std::process;
use std::time::Duration;
use sysinfo::Pid;
use uuid::Uuid;

const TELEMETRY_REPORT_TIMEOUT: Duration = Duration::from_secs(15);
const TELEMETRY_REPORT_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Resource)]
pub struct TelemetryManager {
    uuid: Uuid,
    device_id: String,
    queue: Vec<Value>,
}

impl TelemetryManager {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            device_id: phichain_telemetry::device::get_device_id(),
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
            .add_systems(Startup, send_startup_event_system)
            .add_systems(Startup, log_telemetry_hint_system);
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
    adapter_info: Res<RenderAdapterInfo>,
    entities: &Entities,
    time: Res<Time>,
    mut telemetry_manager: ResMut<TelemetryManager>,

    project: Option<Res<Project>>,
    note_query: Query<&Note>,
    line_query: Query<&Line>,
    event_query: Query<&LineEvent>,
) {
    for event in events.read() {
        let project_info = if let Some(ref project) = project {
            json!({
                "id": project.id,
                "notes": note_query.iter().len(),
                "lines": line_query.iter().len(),
                "events": event_query.iter().len(),
            })
        } else {
            json!(null)
        };

        let diagnostic = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS);
        let fps_samples = diagnostic
            .map(|x| x.values().take(5).collect::<Vec<_>>())
            .unwrap_or_default();
        let average_fps = diagnostic.and_then(|x| x.average()).unwrap_or_default();

        let mut system = sysinfo::System::new_all();
        system.refresh_all();

        let pid = process::id();
        let process = system.process(Pid::from_u32(pid)).unwrap();

        let mut phichain_meta =
            PhichainMeta::new(env!("CARGO_PKG_VERSION"), cfg!(debug_assertions));
        phichain_meta.beta = constants::IS_BETA;

        let payload = TelemetryPayload::builder()
            .reporter("phichain-editor")
            .event_type(event.event_type)
            .device_id(telemetry_manager.device_id.clone())
            .phichain(phichain_meta)
            .metadata(event.metadata.clone())
            .extra("session_id", telemetry_manager.uuid)
            .extra("hardware", json!({
                "cpu": system.cpus().first().unwrap().brand(),
                "core_count": system.cpus().len(),
                "memory": system.total_memory(),
                "memory_formatted": format!("{:.1} GiB", system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0),
            }))
            .extra("adapter", &***adapter_info)
            .extra("project", project_info)
            .extra("performance", json!({
                "fps_samples": fps_samples,
                "fps": average_fps,
                "entities": entities.len(),
                "memory": process.memory(),
            }))
            .extra("uptime", time.elapsed().as_secs_f32())
            .build();

        if phichain_telemetry::env::telemetry_debug() {
            eprintln!(
                "[telemetry] {}",
                serde_json::to_string_pretty(&payload).unwrap()
            );
        } else {
            telemetry_manager
                .queue
                .push(serde_json::to_value(&payload).unwrap());
        }
    }
}

fn send_startup_event_system(mut events: EventWriter<PushTelemetryEvent>) {
    events.write(PushTelemetryEvent::new(
        "phichain.editor.started",
        json!({}),
    ));
}

fn log_telemetry_hint_system() {
    if phichain_telemetry::env::telemetry_disabled() {
        info!("Telemetry disabled by environment variable");
    } else {
        info!("Phichain now collects completely anonymous telemetry regarding usage.");
        info!("This information is used to shape the Phichain roadmap, prioritize features and improve performance.");
        info!("You can learn more, including how to opt-out if you'd not like to participate in this anonymous program, by visiting https://phicha.in/telemetry");
    }
}

fn flush_telemetry_queue_system(
    mut reqwest: BevyReqwest,
    settings: Res<Persistent<EditorSettings>>,
    mut telemetry_manager: ResMut<TelemetryManager>,
) {
    if phichain_telemetry::env::telemetry_disabled() || !settings.general.send_telemetry {
        debug!("Telemetry disabled, skipping...");
        return;
    }

    let data = std::mem::take(&mut telemetry_manager.queue);
    if data.is_empty() {
        return;
    }

    debug!("Flushing telemetry queue with {} entries", data.len());

    let request = reqwest
        .post(phichain_telemetry::TELEMETRY_URL)
        .timeout(TELEMETRY_REPORT_TIMEOUT)
        .json(&data)
        .build()
        .unwrap();

    reqwest
        .send(request)
        .on_response(move |trigger: Trigger<ReqwestResponseEvent>| {
            let response = trigger.event();
            if response.status().is_success() {
                info!(
                    "Successfully sent telemetry event, response: {}",
                    response.as_str().unwrap_or("<unknown>")
                );
            } else {
                error!(
                    "Failed to send telemetry data, bad response, status code: {}",
                    response.status()
                );
            }
        })
        .on_error(|trigger: Trigger<ReqwestErrorEvent>| {
            let e = &trigger.event().0;
            error!("Failed to send telemetry data, request failed: {:?}", e);
        });
}
