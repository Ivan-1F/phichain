use crate::misc::WorkingDirectory;
use bevy::app::App;
use bevy::log::info;
use bevy::log::tracing_subscriber::Layer;
use bevy::log::{tracing_subscriber, BoxedLayer};
use bevy::prelude::Resource;
use chrono::Local;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::{fs, io};

/// Hold the [`tracing_appender`] guard
#[derive(Resource)]
#[allow(dead_code)]
struct LogGuard(tracing_appender::non_blocking::WorkerGuard);

pub fn custom_layer(app: &mut App) -> Option<BoxedLayer> {
    let path = app.world().resource::<WorkingDirectory>().log().ok()?;

    let appender = tracing_appender::rolling::never(path, "latest.log");

    let (non_blocking, guard) = tracing_appender::non_blocking(appender);

    app.insert_resource(LogGuard(guard));

    Some(Box::new(vec![tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .boxed()]))
}

pub fn roll_latest() -> io::Result<()> {
    let dir = WorkingDirectory::default().log()?;
    let latest = dir.join("latest.log");
    if !latest.exists() {
        return Ok(());
    }
    info!("Rolling latest log...");

    let date = Local::now().format("%Y-%m-%d").to_string();

    let mut n = 1u32;
    loop {
        let candidate = dir.join(format!("{}-{}.log.gz", date, n));
        if !candidate.exists() {
            let mut src = File::open(&latest)?;
            let mut encoder = GzEncoder::new(File::create(candidate)?, Compression::default());
            io::copy(&mut src, &mut encoder)?;
            encoder.finish()?;
            break;
        }
        n += 1;
    }

    fs::remove_file(latest)?;
    Ok(())
}
