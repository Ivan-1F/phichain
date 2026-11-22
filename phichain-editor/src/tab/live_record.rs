use crate::constants::CANVAS_WIDTH;
use crate::editing::history::Edits;
use crate::hotkey::display_key_code;
use crate::schedule::EditorSet;
use crate::selection::SelectedLine;
use crate::timeline::settings::TimelineSettings;
use crate::timing::{ChartTime, Pause, Paused, Resume};
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use egui::{Color32, RichText, Ui};
use phichain_chart::beat::{self, Beat};
use phichain_chart::bpm_list::BpmList;
use phichain_chart::line::Line;
use phichain_chart::note::{Note, NoteKind};

const MAX_KEYS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordNoteType {
    Tap,
    Drag,
    Flick,
    Hold,
    AutoTapHold,
}

impl RecordNoteType {
    fn label(self) -> &'static str {
        match self {
            RecordNoteType::Tap => "Tap (藍鍵)",
            RecordNoteType::Drag => "Drag (黃鍵)",
            RecordNoteType::Flick => "Flick (粉鍵)",
            RecordNoteType::Hold => "Hold (長條)",
            RecordNoteType::AutoTapHold => "自動 (短按 Tap / 長按 Hold)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneNoteOverride {
    UseGlobal,
    Tap,
    Drag,
    Flick,
    Hold,
}

impl LaneNoteOverride {
    fn label(self) -> &'static str {
        match self {
            LaneNoteOverride::UseGlobal => "跟隨全域",
            LaneNoteOverride::Tap => "Tap",
            LaneNoteOverride::Drag => "Drag",
            LaneNoteOverride::Flick => "Flick",
            LaneNoteOverride::Hold => "Hold",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActiveHold {
    pub start_beat: Beat,
    pub x: f32,
    pub above: bool,
    pub speed: f32,
    pub line_entity: Entity,
    pub auto_mode: bool,
}

#[derive(Debug, Clone)]
pub struct LaneConfig {
    pub key: KeyCode,
    pub note_override: LaneNoteOverride,
    pub pressed: bool,
    pub active_hold: Option<ActiveHold>,
}

#[derive(Resource)]
pub struct LiveRecordState {
    pub recording: bool,
    pub key_count: usize,
    pub global_note_type: RecordNoteType,
    pub lanes: Vec<LaneConfig>,
    pub rebinding_lane: Option<usize>,

    pub snap_to_beat: bool,
    pub snap_x_to_timeline_lanes: bool,
    pub spread_percent: f32,
    pub above: bool,
    pub note_speed: f32,
    pub offset_ms: f32,
    pub auto_play_on_record: bool,

    pub recorded_count: usize,
}

impl Default for LiveRecordState {
    fn default() -> Self {
        let mut state = Self {
            recording: false,
            key_count: 4,
            global_note_type: RecordNoteType::Tap,
            lanes: Vec::new(),
            rebinding_lane: None,

            snap_to_beat: true,
            snap_x_to_timeline_lanes: false,
            spread_percent: 80.0,
            above: true,
            note_speed: 1.0,
            offset_ms: 0.0,
            auto_play_on_record: true,

            recorded_count: 0,
        };
        state.apply_preset(4);
        state
    }
}

impl LiveRecordState {
    pub fn default_keys_for(k: usize) -> Vec<KeyCode> {
        match k {
            4 => vec![KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD, KeyCode::KeyF],
            6 => vec![
                KeyCode::KeyS,
                KeyCode::KeyD,
                KeyCode::KeyF,
                KeyCode::KeyJ,
                KeyCode::KeyK,
                KeyCode::KeyL,
            ],
            7 => vec![
                KeyCode::KeyA,
                KeyCode::KeyS,
                KeyCode::KeyD,
                KeyCode::KeyF,
                KeyCode::KeyJ,
                KeyCode::KeyK,
                KeyCode::KeyL,
            ],
            8 => vec![
                KeyCode::KeyA,
                KeyCode::KeyS,
                KeyCode::KeyD,
                KeyCode::KeyF,
                KeyCode::KeyJ,
                KeyCode::KeyK,
                KeyCode::KeyL,
                KeyCode::Semicolon,
            ],
            _ => {
                let pool = [
                    KeyCode::KeyA,
                    KeyCode::KeyS,
                    KeyCode::KeyD,
                    KeyCode::KeyF,
                    KeyCode::KeyJ,
                    KeyCode::KeyK,
                    KeyCode::KeyL,
                    KeyCode::Semicolon,
                    KeyCode::KeyQ,
                    KeyCode::KeyW,
                    KeyCode::KeyE,
                    KeyCode::KeyR,
                ];
                pool.into_iter().take(k.clamp(1, MAX_KEYS)).collect()
            }
        }
    }

    pub fn apply_preset(&mut self, k: usize) {
        let k = k.clamp(1, MAX_KEYS);
        self.key_count = k;
        self.rebinding_lane = None;
        self.lanes = Self::default_keys_for(k)
            .into_iter()
            .map(|key| LaneConfig {
                key,
                note_override: LaneNoteOverride::UseGlobal,
                pressed: false,
                active_hold: None,
            })
            .collect();
    }

    pub fn resize_lanes(&mut self, new_k: usize) {
        let new_k = new_k.clamp(1, MAX_KEYS);
        if new_k == self.lanes.len() {
            self.key_count = new_k;
            return;
        }
        let defaults = Self::default_keys_for(new_k);
        while self.lanes.len() < new_k {
            let idx = self.lanes.len();
            let candidate = defaults
                .get(idx)
                .copied()
                .unwrap_or(KeyCode::KeyA);
            self.lanes.push(LaneConfig {
                key: candidate,
                note_override: LaneNoteOverride::UseGlobal,
                pressed: false,
                active_hold: None,
            });
        }
        self.lanes.truncate(new_k);
        self.key_count = new_k;
        if let Some(idx) = self.rebinding_lane {
            if idx >= new_k {
                self.rebinding_lane = None;
            }
        }
    }

    pub fn lane_x(&self, lane_index: usize, timeline_settings: &TimelineSettings) -> f32 {
        let count = self.lanes.len().max(1);
        let raw_x = if count == 1 {
            0.0
        } else {
            let spread = (self.spread_percent.clamp(10.0, 100.0) / 100.0) * CANVAS_WIDTH;
            let t = lane_index as f32 / (count - 1) as f32; // 0.0 .. 1.0
            (t - 0.5) * spread
        };

        if self.snap_x_to_timeline_lanes {
            timeline_settings.attach_x(raw_x)
        } else {
            raw_x
        }
    }

    pub fn effective_note_type(&self, lane_index: usize) -> RecordNoteType {
        match self
            .lanes
            .get(lane_index)
            .map(|l| l.note_override)
            .unwrap_or(LaneNoteOverride::UseGlobal)
        {
            LaneNoteOverride::UseGlobal => self.global_note_type,
            LaneNoteOverride::Tap => RecordNoteType::Tap,
            LaneNoteOverride::Drag => RecordNoteType::Drag,
            LaneNoteOverride::Flick => RecordNoteType::Flick,
            LaneNoteOverride::Hold => RecordNoteType::Hold,
        }
    }

    pub fn is_key_captured(&self, key: KeyCode) -> bool {
        if self.rebinding_lane.is_some() {
            return true;
        }
        if self.recording {
            return self.lanes.iter().any(|l| l.key == key);
        }
        false
    }
}

pub struct LiveRecordPlugin;

impl Plugin for LiveRecordPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LiveRecordState>()
            .add_systems(Update, live_record_input_system.in_set(EditorSet::Edit));
    }
}

fn current_quantized_beat(
    chart_time: f32,
    offset_ms: f32,
    snap_to_beat: bool,
    bpm_list: &BpmList,
    timeline_settings: &TimelineSettings,
) -> Beat {
    let adjusted_time = (chart_time + offset_ms / 1000.0).max(0.0);
    let raw_beat = bpm_list.beat_at(adjusted_time).value().max(0.0);
    if snap_to_beat {
        timeline_settings.attach(raw_beat)
    } else {
        beat::utils::attach(raw_beat, 64)
    }
}

fn live_record_input_system(
    mut state: ResMut<LiveRecordState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    chart_time: Res<ChartTime>,
    paused: Res<Paused>,
    bpm_list: Res<BpmList>,
    timeline_settings: Res<TimelineSettings>,
    selected_line: Res<SelectedLine>,
    mut edits: Edits,
) {
    // 1. 處理按鍵重新綁定
    if let Some(lane_idx) = state.rebinding_lane {
        for &key in keyboard.get_just_pressed() {
            if key == KeyCode::Escape {
                state.rebinding_lane = None;
                return;
            }
            if let Some(lane) = state.lanes.get_mut(lane_idx) {
                lane.key = key;
            }
            state.rebinding_lane = None;
            return;
        }
        return;
    }

    // 更新視覺按下狀態（未錄製時也可試按確認鍵位）
    for lane in &mut state.lanes {
        lane.pressed = keyboard.pressed(lane.key);
    }

    let now_beat = current_quantized_beat(
        chart_time.0,
        state.offset_ms,
        state.snap_to_beat,
        &bpm_list,
        &timeline_settings,
    );

    // 若已停止錄製或暫停，自動結算所有按住中的 Hold
    if !state.recording || paused.0 {
        let mut notes_to_spawn: Vec<(Note, Entity)> = Vec::new();
        for lane in &mut state.lanes {
            if let Some(active) = lane.active_hold.take() {
                let min_beat = timeline_settings.minimum_beat();
                let diff = now_beat - active.start_beat;
                let kind = if active.auto_mode && diff < min_beat {
                    NoteKind::Tap
                } else {
                    NoteKind::Hold {
                        hold_beat: diff.max(min_beat),
                    }
                };
                let note = Note::new(kind, active.above, active.start_beat, active.x, active.speed);
                notes_to_spawn.push((note, active.line_entity));
            }
        }
        if !notes_to_spawn.is_empty() {
            let count = notes_to_spawn.len();
            state.recorded_count += count;
            edits.once(t!("history.create_notes", count = count), move |commands| {
                for (note, line_entity) in notes_to_spawn {
                    commands.spawn((note, ChildOf(line_entity)));
                }
            });
        }
        return;
    }

    // 2. 正在錄製且播放中：處理每個軌道的按下與放開
    let line_entity = selected_line.0;
    let above = state.above;
    let speed = state.note_speed;
    let mut notes_to_spawn: Vec<(Note, Entity)> = Vec::new();

    for idx in 0..state.lanes.len() {
        let key = state.lanes[idx].key;
        let note_type = state.effective_note_type(idx);
        let x = state.lane_x(idx, &timeline_settings);

        if keyboard.just_pressed(key) {
            match note_type {
                RecordNoteType::Tap => {
                    notes_to_spawn.push((Note::new(NoteKind::Tap, above, now_beat, x, speed), line_entity));
                }
                RecordNoteType::Drag => {
                    notes_to_spawn.push((Note::new(NoteKind::Drag, above, now_beat, x, speed), line_entity));
                }
                RecordNoteType::Flick => {
                    notes_to_spawn.push((Note::new(NoteKind::Flick, above, now_beat, x, speed), line_entity));
                }
                RecordNoteType::Hold => {
                    state.lanes[idx].active_hold = Some(ActiveHold {
                        start_beat: now_beat,
                        x,
                        above,
                        speed,
                        line_entity,
                        auto_mode: false,
                    });
                }
                RecordNoteType::AutoTapHold => {
                    state.lanes[idx].active_hold = Some(ActiveHold {
                        start_beat: now_beat,
                        x,
                        above,
                        speed,
                        line_entity,
                        auto_mode: true,
                    });
                }
            }
        }

        if keyboard.just_released(key) {
            if let Some(active) = state.lanes[idx].active_hold.take() {
                let min_beat = timeline_settings.minimum_beat();
                let diff = now_beat - active.start_beat;
                let kind = if active.auto_mode && diff < min_beat {
                    NoteKind::Tap
                } else {
                    NoteKind::Hold {
                        hold_beat: diff.max(min_beat),
                    }
                };
                notes_to_spawn.push((
                    Note::new(kind, active.above, active.start_beat, active.x, active.speed),
                    active.line_entity,
                ));
            }
        }
    }

    if !notes_to_spawn.is_empty() {
        let count = notes_to_spawn.len();
        state.recorded_count += count;
        edits.once(t!("history.create_notes", count = count), move |commands| {
            for (note, target_line) in notes_to_spawn {
                commands.spawn((note, ChildOf(target_line)));
            }
        });
    }
}

pub fn live_record_tab(In(mut ui): In<Ui>, world: &mut World) {
    let mut sys_state: SystemState<(
        ResMut<LiveRecordState>,
        Res<SelectedLine>,
        Query<&Line>,
        Res<Paused>,
        Commands,
    )> = SystemState::new(world);

    let (mut state, selected_line, line_query, paused, mut commands) = sys_state.get_mut(world);

    let line_name = line_query
        .get(selected_line.0)
        .map(|l| l.name.clone())
        .unwrap_or_else(|_| "未知判定線".to_string());

    egui::ScrollArea::vertical().show(&mut ui, |ui| {
        // 頂部錄製控制列
        ui.horizontal(|ui| {
            let record_label = if state.recording {
                RichText::new("■ 停止錄製").color(Color32::WHITE).strong()
            } else {
                RichText::new("● 開始錄製").color(Color32::WHITE).strong()
            };
            let record_fill = if state.recording {
                Color32::from_rgb(210, 50, 65)
            } else {
                Color32::from_rgb(40, 145, 95)
            };

            if ui
                .add(egui::Button::new(record_label).fill(record_fill).min_size(egui::vec2(108.0, 30.0)))
                .clicked()
            {
                state.recording = !state.recording;
                state.rebinding_lane = None;
                if state.recording {
                    state.recorded_count = 0;
                    if state.auto_play_on_record && paused.0 {
                        commands.trigger(Resume);
                    }
                } else if state.auto_play_on_record && !paused.0 {
                    commands.trigger(Pause);
                }
            }

            if state.recording {
                if paused.0 {
                    ui.colored_label(
                        Color32::from_rgb(255, 195, 70),
                        "待命中（按空白鍵開始播放即可打鍵錄製）",
                    );
                } else {
                    ui.colored_label(
                        Color32::from_rgb(255, 95, 110),
                        format!("錄製中 · 本次已錄 {} 個音符", state.recorded_count),
                    );
                }
            } else {
                ui.label(format!("目標判定線：{}", line_name));
            }
        });

        ui.separator();

        // 基本設定區
        egui::Grid::new("live_record_settings_grid")
            .num_columns(2)
            .spacing([16.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                // 幾 K 設定
                ui.label("鍵數 (K 數)");
                ui.horizontal(|ui| {
                    let mut k = state.key_count;
                    if ui
                        .add(egui::DragValue::new(&mut k).range(1..=MAX_KEYS).suffix("K"))
                        .changed()
                    {
                        state.resize_lanes(k);
                    }
                    for preset in [4, 6, 7, 8] {
                        if ui
                            .selectable_label(state.key_count == preset, format!("{preset}K"))
                            .clicked()
                        {
                            state.apply_preset(preset);
                        }
                    }
                });
                ui.end_row();

                // 全域音符類型
                ui.label("預設音符類型");
                egui::ComboBox::from_id_salt("live_record_global_note_type")
                    .selected_text(state.global_note_type.label())
                    .show_ui(ui, |ui| {
                        for kind in [
                            RecordNoteType::Tap,
                            RecordNoteType::Drag,
                            RecordNoteType::Flick,
                            RecordNoteType::Hold,
                            RecordNoteType::AutoTapHold,
                        ] {
                            ui.selectable_value(&mut state.global_note_type, kind, kind.label());
                        }
                    });
                ui.end_row();

                // 音符朝向
                ui.label("音符朝向");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut state.above, true, "上方 (Above)");
                    ui.selectable_value(&mut state.above, false, "下方 (Below)");
                });
                ui.end_row();

                // 軌道分布寬度
                ui.label("軌道分布寬度");
                ui.add(
                    egui::Slider::new(&mut state.spread_percent, 20.0..=100.0)
                        .suffix("%")
                        .step_by(5.0),
                );
                ui.end_row();

                // 音符速度
                ui.label("音符速度");
                ui.add(
                    egui::DragValue::new(&mut state.note_speed)
                        .range(0.1..=10.0)
                        .speed(0.05),
                );
                ui.end_row();

                // 延遲補償
                ui.label("錄製偏移補償");
                ui.add(
                    egui::DragValue::new(&mut state.offset_ms)
                        .range(-200.0..=200.0)
                        .speed(1.0)
                        .suffix(" ms"),
                );
                ui.end_row();

                // 對齊節拍
                ui.label("對齊節拍線");
                ui.checkbox(&mut state.snap_to_beat, "依照時間線節拍密度自動量化");
                ui.end_row();

                // 對齊垂直線
                ui.label("對齊垂直參考線");
                ui.checkbox(&mut state.snap_x_to_timeline_lanes, "將 X 座標吸附至時間線垂直格線");
                ui.end_row();

                // 自動播放
                ui.label("聯動播放控制");
                ui.checkbox(&mut state.auto_play_on_record, "開始錄製時自動播放、停止時自動暫停");
                ui.end_row();
            });

        ui.separator();

        // 即時打鍵視覺指示條（4K / 6K / 8K 鍵盤回饋）
        ui.label(RichText::new("即時按鍵狀態（可直接試按確認鍵位）").strong());
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            let lane_count = state.lanes.len();
            let avail_w = ui.available_width();
            let btn_w = ((avail_w - (lane_count.saturating_sub(1) as f32) * 6.0) / lane_count.max(1) as f32)
                .clamp(44.0, 96.0);

            for (i, lane) in state.lanes.iter().enumerate() {
                let effective = state.effective_note_type(i);
                let accent = match effective {
                    RecordNoteType::Tap => Color32::from_rgb(0, 195, 255),
                    RecordNoteType::Drag => Color32::from_rgb(255, 205, 55),
                    RecordNoteType::Flick => Color32::from_rgb(255, 85, 130),
                    RecordNoteType::Hold | RecordNoteType::AutoTapHold => {
                        Color32::from_rgb(60, 235, 200)
                    }
                };
                let fill = if lane.pressed {
                    accent
                } else {
                    Color32::from_rgb(30, 36, 48)
                };
                let text_color = if lane.pressed {
                    Color32::BLACK
                } else {
                    Color32::WHITE
                };

                let label = format!("K{}\n{}", i + 1, display_key_code(lane.key));
                ui.add(
                    egui::Button::new(RichText::new(label).color(text_color).strong())
                        .fill(fill)
                        .min_size(egui::vec2(btn_w, 46.0)),
                );
            }
        });

        ui.separator();

        // 每軌獨立鍵位與音符類型設定表
        ui.label(RichText::new("各軌道鍵位與音符類型").strong());
        ui.add_space(4.0);
        egui::Grid::new("live_record_lanes_grid")
            .num_columns(3)
            .spacing([12.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("軌道");
                ui.label("綁定按鍵（點擊後按下鍵盤）");
                ui.label("此軌音符類型");
                ui.end_row();

                let lane_count = state.lanes.len();
                for i in 0..lane_count {
                    ui.label(format!("Lane {}", i + 1));

                    let is_rebinding = state.rebinding_lane == Some(i);
                    let key_btn_text = if is_rebinding {
                        RichText::new("請按下按鍵... (Esc 取消)")
                            .color(Color32::from_rgb(255, 210, 80))
                            .strong()
                    } else {
                        RichText::new(display_key_code(state.lanes[i].key)).monospace()
                    };

                    if ui.button(key_btn_text).clicked() {
                        state.rebinding_lane = if is_rebinding { None } else { Some(i) };
                    }

                    egui::ComboBox::from_id_salt(format!("live_record_lane_override_{i}"))
                        .selected_text(state.lanes[i].note_override.label())
                        .show_ui(ui, |ui| {
                            for opt in [
                                LaneNoteOverride::UseGlobal,
                                LaneNoteOverride::Tap,
                                LaneNoteOverride::Drag,
                                LaneNoteOverride::Flick,
                                LaneNoteOverride::Hold,
                            ] {
                                ui.selectable_value(
                                    &mut state.lanes[i].note_override,
                                    opt,
                                    opt.label(),
                                );
                            }
                        });

                    ui.end_row();
                }
            });
    });

    sys_state.apply(world);
}
