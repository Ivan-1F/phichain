use crate::timeline::TimelineContext;
use egui::{Id, Rangef, Rect, Sense, Ui};
use phichain_chart::beat::Beat;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;

/// A trait for types that have a beat range on timeline.
pub trait TimelineBeatRange {
    fn start_beat_value(&self) -> f32;
    fn end_beat_value(&self) -> f32;
    fn set_start_beat(&mut self, beat: Beat);
    fn set_end_beat(&mut self, beat: Beat);
}

impl TimelineBeatRange for Note {
    fn start_beat_value(&self) -> f32 {
        self.beat.value()
    }

    fn end_beat_value(&self) -> f32 {
        self.end_beat().value()
    }

    /// Set start beat while keeping end_beat unchanged (adjusts hold_beat for Hold notes)
    fn set_start_beat(&mut self, beat: Beat) {
        let old_end = self.end_beat();
        self.beat = beat;
        if let Some(hold_beat) = self.hold_beat_mut() {
            *hold_beat = old_end - beat;
        }
    }

    fn set_end_beat(&mut self, beat: Beat) {
        Note::set_end_beat(self, beat);
    }
}

impl TimelineBeatRange for LineEvent {
    fn start_beat_value(&self) -> f32 {
        self.start_beat.value()
    }

    fn end_beat_value(&self) -> f32 {
        self.end_beat.value()
    }

    fn set_start_beat(&mut self, beat: Beat) {
        self.start_beat = beat;
    }

    fn set_end_beat(&mut self, beat: Beat) {
        self.end_beat = beat;
    }
}

/// A drag zone for editing beat ranges (start/end) with precise accumulation.
///
/// This widget handles the common drag interaction pattern used in timeline editing,
/// where users drag the top or bottom edge of a rect to adjust beat positions.
pub struct BeatRangeDragZone<'a, T: TimelineBeatRange + Clone + PartialEq + Send + Sync + 'static> {
    rect: Rect,
    id: &'static str,
    ctx: &'a TimelineContext<'a>,
    data: &'a mut T,
}

pub struct Drag<T> {
    pub from: T,
    pub to: T,
}

impl<'a, T: TimelineBeatRange + Clone + PartialEq + Send + Sync + 'static>
    BeatRangeDragZone<'a, T>
{
    pub fn new(
        rect: Rect,
        id: &'static str,
        ctx: &'a TimelineContext<'a>,
        data: &'a mut T,
    ) -> Self {
        Self {
            rect,
            id,
            ctx,
            data,
        }
    }

    /// Show the drag zones for both start and end.
    /// Returns `Some((from, to))` if drag stopped and data changed.
    pub fn show(mut self, ui: &mut Ui) -> Option<Drag<T>> {
        let result_start = self.process_drag_zone(ui, true);
        let result_end = self.process_drag_zone(ui, false);

        result_start.or(result_end)
    }

    fn process_drag_zone(&mut self, ui: &mut Ui, start: bool) -> Option<Drag<T>> {
        let drag_zone = Rect::from_x_y_ranges(
            self.rect.x_range(),
            if start {
                Rangef::from(self.rect.max.y - 5.0..=self.rect.max.y)
            } else {
                Rangef::from(self.rect.min.y..=self.rect.min.y + 5.0)
            },
        );

        let response = ui
            .allocate_rect(drag_zone, Sense::drag())
            .on_hover_and_drag_cursor(egui::CursorIcon::ResizeVertical);

        let precise_id = Id::new(self.id).with("precise").with(start);
        let snapshot_id = Id::new(self.id).with("snapshot");

        if response.drag_started() {
            ui.data_mut(|data| data.insert_temp(snapshot_id, self.data.clone()));
            let initial_beat = if start {
                self.data.start_beat_value()
            } else {
                self.data.end_beat_value()
            };
            let initial_y = self.ctx.beat_f32_to_y(initial_beat);
            ui.data_mut(|data| data.insert_temp(precise_id, initial_y));
        }

        if response.dragged() {
            let precise_y = ui.data(|data| data.get_temp::<f32>(precise_id))?;

            let drag_delta = response.drag_delta();
            let new_y = precise_y + drag_delta.y;
            ui.data_mut(|data| data.insert_temp(precise_id, new_y));

            let new_beat = self.ctx.y_to_beat_f32(new_y);

            let min_step = 1.0 / self.ctx.settings.density as f32;
            let current_start = self.data.start_beat_value();
            let current_end = self.data.end_beat_value();

            let clamped = if start {
                new_beat.max(0.0).min(current_end - min_step)
            } else {
                new_beat.max(current_start + min_step)
            };

            let attached = self.ctx.settings.attach(clamped);
            if start {
                self.data.set_start_beat(attached);
            } else {
                self.data.set_end_beat(attached);
            }
        }

        if response.drag_stopped() {
            let from = ui.data(|data| data.get_temp::<T>(snapshot_id))?;
            ui.data_mut(|data| data.remove::<T>(snapshot_id));
            ui.data_mut(|data| data.remove::<f32>(precise_id));

            if from != *self.data {
                return Some(Drag {
                    from,
                    to: self.data.clone(),
                });
            }
        }

        None
    }
}
