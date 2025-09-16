use crate::timeline::TimelineContext;
use crate::utils::convert::BevyEguiConvert;
use bevy::ecs::system::SystemState;
use bevy::prelude::{Res, Resource, World};
use bevy_kira_audio::AudioSource;
use colorous::{Gradient, INFERNO};
use egui::epaint::{Vertex, WHITE_UV};
use egui::{Color32, Mesh, Painter, Pos2, Rect};
use phichain_chart::offset::Offset;
use realfft::num_complex::Complex32;
use realfft::RealFftPlanner;

#[derive(Debug)]
pub struct AudioMono {
    pub sample_rate: u32, // Hz
    pub data: Vec<f32>,   // mono [-1.0, 1.0]
}

pub fn make_spectrogram(source: &AudioSource) -> SpectrogramU8 {
    let mono = load_audio(source);
    make_spectrogram_u8(&mono, 2048, 512, 80.0)
}

fn load_audio(source: &AudioSource) -> AudioMono {
    AudioMono {
        sample_rate: source.sound.sample_rate,
        data: source
            .sound
            .frames
            .iter()
            .map(|frame| frame.as_mono().left)
            .collect(),
    }
}

#[derive(Debug, Resource)]
pub struct Spectrogram(pub SpectrogramU8);

#[derive(Debug, Clone)]
pub struct SpectrogramU8 {
    pub n_frames: usize,
    pub n_bins: usize,
    pub data: Vec<u8>,
    pub sample_rate: u32,
    pub hop_length: u32,
}

pub fn draw(painter: &Painter, world: &mut World) {
    let mut state = SystemState::<(TimelineContext, Res<Spectrogram>, Res<Offset>)>::new(world);
    let (ctx, spectrogram, offset): (TimelineContext, Res<Spectrogram>, Res<Offset>) =
        state.get_mut(world);

    let opacity = ctx.settings.spectrogram_opacity.clamp(0.0, 1.0);
    if !ctx.settings.show_spectrogram || opacity <= 0.01 {
        return;
    }

    let spec = &spectrogram.0;
    // Avoid rendering when there are not enough frames for interpolation
    if spec.n_frames < 2 {
        return;
    }
    let rect = ctx.viewport.0.into_egui();
    let y_to_time = |x: f32| ctx.y_to_time(x) + offset.0 / 1000.0;

    let cols = (rect.width() / 2.5).clamp(256.0, 512.0) as usize;
    let rows = (rect.height() / 2.5).clamp(220.0, 600.0) as usize;

    render_spectrogram_egui(
        painter,
        rect,
        spec,
        &INFERNO,
        &y_to_time,
        RenderOpts {
            cols,
            rows,
            freq: FreqScale::Log {
                fmin_hz: 80.0,
                fmax_hz: spec.sample_rate as f32 * 0.5,
            },
            time_aa: 0.0,
            opacity,
        },
    );
}

fn make_spectrogram_u8(audio: &AudioMono, n_fft: usize, hop: usize, top_db: f32) -> SpectrogramU8 {
    assert!(n_fft >= 16 && hop >= 1 && hop <= n_fft);

    let win: Vec<f32> = (0..n_fft)
        .map(|n| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * n as f32 / (n_fft as f32 - 1.0)).cos())
        .collect();

    let nb = n_fft / 2 + 1;
    if audio.data.len() < n_fft {
        return SpectrogramU8 {
            n_frames: 0,
            n_bins: nb,
            data: vec![],
            sample_rate: audio.sample_rate,
            hop_length: hop as u32,
        };
    }
    let n_frames = 1 + (audio.data.len() - n_fft) / hop;

    let mut planner = RealFftPlanner::<f32>::new();
    let r2c = planner.plan_fft_forward(n_fft);
    let mut inbuf = r2c.make_input_vec(); // len = n_fft
    let mut outbuf = r2c.make_output_vec(); // len = n_fft/2+1

    let mut db_mat = vec![0f32; n_frames * nb];
    let mut max_db = -1e30f32;

    let win_pow_sum: f32 = win.iter().map(|w| w * w).sum::<f32>().max(1e-12);
    let pow_norm = 1.0 / win_pow_sum;

    for f in 0..n_frames {
        let start = f * hop;
        for i in 0..n_fft {
            inbuf[i] = audio.data[start + i] * win[i];
        }
        r2c.process(&mut inbuf, &mut outbuf).unwrap();

        let row = &mut db_mat[f * nb..(f + 1) * nb];
        for b in 0..nb {
            let c: Complex32 = outbuf[b];
            let p = (c.re * c.re + c.im * c.im) * pow_norm; // 功率 ~ 能量
            let db = 10.0 * p.max(1e-12).log10(); // dB
            row[b] = db;
            if db > max_db {
                max_db = db;
            }
        }
    }

    let min_db = max_db - top_db;
    let mut out_u8 = vec![0u8; n_frames * nb];
    for (i, &db) in db_mat.iter().enumerate() {
        let v = ((db - min_db) / top_db).clamp(0.0, 1.0);
        out_u8[i] = (v * 255.0).round() as u8;
    }

    SpectrogramU8 {
        n_frames,
        n_bins: nb,
        data: out_u8,
        sample_rate: audio.sample_rate,
        hop_length: hop as u32,
    }
}

pub enum FreqScale {
    #[allow(dead_code)]
    Linear,
    Log {
        fmin_hz: f32,
        fmax_hz: f32,
    },
}

pub struct RenderOpts {
    pub cols: usize,
    pub rows: usize,
    pub freq: FreqScale,
    pub time_aa: f32,
    pub opacity: f32,
}

fn lut256(cmap: &Gradient) -> [Color32; 256] {
    let mut out = [Color32::BLACK; 256];
    for (i, px) in out.iter_mut().enumerate() {
        let c = cmap.eval_continuous(i as f64 / 255.0);
        *px = Color32::from_rgb(c.r, c.g, c.b);
    }
    out
}

pub fn render_spectrogram_egui(
    painter: &Painter,
    rect: Rect,
    spec: &SpectrogramU8,
    cmap: &Gradient,
    y_to_time: &dyn Fn(f32) -> f32,
    opts: RenderOpts,
) {
    let cols = opts.cols.clamp(2, 4096);
    let rows = opts.rows.clamp(1, 4096);
    let w = rect.width().max(1.0);
    let h = rect.height().max(1.0);
    let dx = w / cols as f32;
    let dy = h / rows as f32;

    let lut = lut256(cmap);

    let nb = (spec.n_bins - 1) as f32;
    let x_to_bin: Vec<f32> = match opts.freq {
        FreqScale::Linear => (0..cols)
            .map(|i| i as f32 / (cols - 1) as f32 * nb)
            .collect(),
        FreqScale::Log { fmin_hz, fmax_hz } => {
            let nyq = spec.sample_rate as f32 * 0.5;
            let r = (fmax_hz / fmin_hz).ln();
            (0..cols)
                .map(|i| {
                    let u = i as f32 / (cols - 1) as f32;
                    let f = fmin_hz * (r * u).exp();
                    (f / nyq * nb).clamp(0.0, nb)
                })
                .collect()
        }
    };

    let samp_bin = |f: usize, b: f32| -> u8 {
        let b0 = b.floor() as usize;
        let b1 = b0.saturating_add(1).min(spec.n_bins - 1);
        let t = (b - b0 as f32).clamp(0.0, 1.0);
        let base = f * spec.n_bins;
        let v0 = spec.data[base + b0] as f32;
        let v1 = spec.data[base + b1] as f32;
        (v0 + (v1 - v0) * t).round() as u8
    };

    let mut mesh = Mesh::default();
    mesh.vertices.reserve(rows * cols * 2);
    mesh.indices.reserve(rows * (cols - 1) * 6);

    for ry in 0..rows {
        let y0 = rect.top() + ry as f32 * dy;
        let y1 = y0 + dy;
        let yc = (y0 + y1) * 0.5;

        let t = y_to_time(yc);
        let ff = t * spec.sample_rate as f32 / spec.hop_length as f32;

        let (lo, hi) = if opts.time_aa > 0.0 {
            let half = 0.5 * opts.time_aa;
            (
                (ff - half).floor().max(0.0) as usize,
                ((ff + half).ceil() as usize).min(spec.n_frames),
            )
        } else {
            let f0 = ff.floor().clamp(0.0, (spec.n_frames - 1) as f32) as usize;
            (f0, (f0 + 1).min(spec.n_frames))
        };
        let span = (hi - lo).max(1) as f32;

        let base = mesh.vertices.len() as u32;
        for (cx, bf) in x_to_bin.iter().enumerate() {
            let x = rect.left() + cx as f32 * dx;
            let bf = *bf;

            let g = if opts.time_aa <= 0.0 {
                let f0 = lo;
                let f1 = (f0 + 1).min(spec.n_frames - 1);
                let a = (ff - f0 as f32).clamp(0.0, 1.0);
                let v0 = samp_bin(f0, bf) as f32;
                let v1 = samp_bin(f1, bf) as f32;
                (v0 + (v1 - v0) * a).round() as u8
            } else {
                let mut acc = 0f32;
                for f in lo..hi {
                    acc += samp_bin(f, bf) as f32;
                }
                (acc / span).round() as u8
            };

            let base = lut[g as usize];
            let c = Color32::from_rgba_unmultiplied(
                base.r(),
                base.g(),
                base.b(),
                (opts.opacity * 255.0) as u8,
            );
            mesh.vertices.push(Vertex {
                pos: Pos2::new(x, y0),
                uv: WHITE_UV,
                color: c,
            });
            mesh.vertices.push(Vertex {
                pos: Pos2::new(x, y1),
                uv: WHITE_UV,
                color: c,
            });
        }
        for i in 0..(cols - 1) {
            let i0 = base + (i as u32) * 2;
            mesh.indices
                .extend_from_slice(&[i0, i0 + 1, i0 + 2, i0 + 2, i0 + 1, i0 + 3]);
        }
    }

    painter.add(mesh);
}
