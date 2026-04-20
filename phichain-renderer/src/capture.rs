//! Offscreen capture pipeline.
//!
//! The camera renders into a GPU texture we own (not a window). Each frame
//! we copy that texture into a staging buffer that the CPU can map and read,
//! then ship the raw RGBA bytes back to the main world over a channel.

use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssets,
    render_graph::{self, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
    render_resource::{
        Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, MapMode,
        PollType, TexelCopyBufferInfo, TexelCopyBufferLayout, TextureFormat, TextureUsages,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue},
    texture::GpuImage,
    Extract, Render, RenderApp, RenderSystems,
};
use crossbeam_channel::{Receiver, Sender};
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Raw RGBA bytes (possibly row-padded — see [`unpad_rows`]) for each rendered frame.
#[derive(Resource, Deref)]
pub struct FrameReceiver(Receiver<Vec<u8>>);

#[derive(Resource, Deref)]
struct FrameSender(Sender<Vec<u8>>);

pub struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = crossbeam_channel::unbounded();
        app.insert_resource(FrameReceiver(rx));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.insert_resource(FrameSender(tx));

        // Run our copy node right after the camera finishes rendering.
        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(CaptureLabel, CaptureNode);
        graph.add_node_edge(bevy::render::graph::CameraDriverLabel, CaptureLabel);

        render_app
            .add_systems(ExtractSchedule, extract_copiers)
            .add_systems(Render, send_frame.after(RenderSystems::Render));
    }
}

/// Create an offscreen render target and the readback buffer for it.
///
/// Spawns an `ImageCopier` entity the render graph will pick up via extract.
/// The returned `RenderTarget` should be attached to a `Camera2d`.
pub fn setup_offscreen_target(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    render_device: &RenderDevice,
    width: u32,
    height: u32,
) -> RenderTarget {
    let size = Extent3d {
        width,
        height,
        ..default()
    };

    // GPU-only texture the camera renders into. COPY_SRC lets us read it back.
    let mut image = Image::new_target_texture(width, height, TextureFormat::Rgba8UnormSrgb, None);
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let handle = images.add(image);

    commands.spawn(ImageCopier::new(handle.clone(), size, render_device));
    RenderTarget::Image(handle.into())
}

/// Strip the per-row padding from a mapped staging buffer to get tight
/// `width * height * 4` RGBA bytes. wgpu requires each row copied into a
/// buffer to be aligned to 256 bytes, so the buffer may be wider than the image.
pub fn unpad_rows(bytes: &[u8], width: u32, height: u32) -> Vec<u8> {
    let row = width as usize * 4;
    let padded = RenderDevice::align_copy_bytes_per_row(row);
    if row == padded {
        return bytes.to_vec();
    }
    let mut out = Vec::with_capacity(row * height as usize);
    for chunk in bytes.chunks_exact(padded).take(height as usize) {
        out.extend_from_slice(&chunk[..row]);
    }
    out
}

/// Marks a GPU image that should be copied out every frame.
#[derive(Component, Clone)]
struct ImageCopier {
    buffer: Buffer,
    src: Handle<Image>,
    enabled: Arc<AtomicBool>,
}

impl ImageCopier {
    fn new(src: Handle<Image>, size: Extent3d, render_device: &RenderDevice) -> Self {
        let padded_row = RenderDevice::align_copy_bytes_per_row(size.width as usize) * 4;
        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("phichain_readback_buffer"),
            size: padded_row as u64 * size.height as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            buffer,
            src,
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }
}

#[derive(Resource, Default, Deref)]
struct ExtractedCopiers(Vec<ImageCopier>);

fn extract_copiers(mut commands: Commands, q: Extract<Query<&ImageCopier>>) {
    commands.insert_resource(ExtractedCopiers(q.iter().cloned().collect()));
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, RenderLabel)]
struct CaptureLabel;

struct CaptureNode;

impl render_graph::Node for CaptureNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        ctx: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let Some(copiers) = world.get_resource::<ExtractedCopiers>() else {
            return Ok(());
        };
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();
        let queue = world.resource::<RenderQueue>();

        for copier in copiers.iter() {
            if !copier.enabled.load(Ordering::Relaxed) {
                continue;
            }
            let Some(src) = gpu_images.get(&copier.src) else {
                continue;
            };

            let padded_row =
                RenderDevice::align_copy_bytes_per_row(src.size.width as usize * 4) as u32;

            let mut encoder = ctx
                .render_device()
                .create_command_encoder(&CommandEncoderDescriptor::default());
            encoder.copy_texture_to_buffer(
                src.texture.as_image_copy(),
                TexelCopyBufferInfo {
                    buffer: &copier.buffer,
                    layout: TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(NonZero::new(padded_row).unwrap().into()),
                        rows_per_image: None,
                    },
                },
                src.size,
            );
            queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
    }
}

/// After the render graph runs, map the staging buffer and ship bytes back.
/// Blocks until the GPU finishes the copy (`poll` is mandatory on native).
fn send_frame(
    copiers: Res<ExtractedCopiers>,
    render_device: Res<RenderDevice>,
    sender: Res<FrameSender>,
) {
    for copier in copiers.iter() {
        if !copier.enabled.load(Ordering::Relaxed) {
            continue;
        }
        let slice = copier.buffer.slice(..);

        // map_async completes on the GPU's schedule; use a tiny channel to wait.
        let (done_tx, done_rx) = crossbeam_channel::bounded(1);
        slice.map_async(MapMode::Read, move |r| {
            r.expect("buffer map failed");
            done_tx.send(()).unwrap();
        });
        render_device
            .poll(PollType::wait_indefinitely())
            .expect("device poll");
        done_rx.recv().expect("map_async completion");

        // Main world may have dropped its receiver on app exit — ignore errors.
        let _ = sender.send(slice.get_mapped_range().to_vec());
        copier.buffer.unmap();
    }
}
