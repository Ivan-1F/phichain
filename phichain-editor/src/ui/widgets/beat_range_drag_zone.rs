use crate::timeline::TimelineContext;
use egui::{Id, Rangef, Rect, Sense, Ui};
use phichain_chart::beat::Beat;

/// A drag zone for editing beat ranges (start/end) with precise accumulation.
///
/// This widget handles the common drag interaction pattern used in timeline editing,
/// where users drag the top or bottom edge of a rect to adjust beat positions.
// TODO: consider introducing a `BeatRange` trait or component
pub struct BeatRangeDragZone<'a, T: Clone + PartialEq + Send + Sync + 'static> {
    rect: Rect,
    id: &'static str,
    ctx: &'a TimelineContext<'a>,
    data: &'a mut T,
    get_start: fn(&T) -> f32,
    get_end: fn(&T) -> f32,
    set_start: fn(&mut T, Beat),
    set_end: fn(&mut T, Beat),
}

pub struct Drag<T> {
    pub from: T,
    pub to: T,
}

impl<'a, T: Clone + PartialEq + Send + Sync + 'static> BeatRangeDragZone<'a, T> {
    pub fn new(
        rect: Rect,
        id: &'static str,
        ctx: &'a TimelineContext<'a>,
        data: &'a mut T,
        get_start: fn(&T) -> f32,
        get_end: fn(&T) -> f32,
        set_start: fn(&mut T, Beat),
        set_end: fn(&mut T, Beat),
    ) -> Self {
        Self {
            rect,
            id,
            ctx,
            data,
            get_start,
            get_end,
            set_start,
            set_end,
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
                (self.get_start)(self.data)
            } else {
                (self.get_end)(self.data)
            };
            let initial_y = self.ctx.beat_f32_to_y(initial_beat);
            ui.data_mut(|data| data.insert_temp(precise_id, initial_y));
        }

        if response.dragged() {
            let drag_delta = response.drag_delta();
            let precise_y: f32 = ui.data(|data| data.get_temp(precise_id).unwrap());
            let new_y = precise_y + drag_delta.y;
            ui.data_mut(|data| data.insert_temp(precise_id, new_y));

            let new_beat = self.ctx.y_to_beat_f32(new_y);

            let min_step = 1.0 / self.ctx.settings.density as f32;
            let current_start = (self.get_start)(self.data);
            let current_end = (self.get_end)(self.data);

            let clamped = if start {
                new_beat.min(current_end - min_step)
            } else {
                new_beat.max(current_start + min_step)
            };

            let attached = self.ctx.settings.attach(clamped);
            if start {
                (self.set_start)(self.data, attached);
            } else {
                (self.set_end)(self.data, attached);
            }
        }

        if response.drag_stopped() {
            let from: T = ui.data(|data| data.get_temp(snapshot_id).unwrap());
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
