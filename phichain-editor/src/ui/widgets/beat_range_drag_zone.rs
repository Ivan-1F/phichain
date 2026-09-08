use crate::timeline::TimelineContext;
use egui::{Id, Rangef, Rect, Response, Sense, Ui};
use phichain_chart::beat::Beat;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;

/// A trait for types that have a beat range on timeline.
pub trait TimelineBeatRange {
    fn start_beat(&self) -> Beat;
    fn end_beat(&self) -> Beat;
    fn set_start_beat(&mut self, beat: Beat);
    fn set_end_beat(&mut self, beat: Beat);
}

impl TimelineBeatRange for Note {
    fn start_beat(&self) -> Beat {
        self.beat
    }

    fn end_beat(&self) -> Beat {
        Note::end_beat(self)
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
    fn start_beat(&self) -> Beat {
        self.start_beat
    }

    fn end_beat(&self) -> Beat {
        self.end_beat
    }

    fn set_start_beat(&mut self, beat: Beat) {
        self.start_beat = beat;
    }

    fn set_end_beat(&mut self, beat: Beat) {
        self.end_beat = beat;
    }
}

/// Start and end handles for a beat range. The widget keeps only pointer geometry;
/// callers decide how its responses update the document.
pub struct BeatRangeDragZone<'a, T: TimelineBeatRange> {
    rect: Rect,
    id: Id,
    ctx: &'a TimelineContext<'a>,
    data: &'a mut T,
}

impl<'a, T: TimelineBeatRange> BeatRangeDragZone<'a, T> {
    pub fn new(rect: Rect, id: Id, ctx: &'a TimelineContext<'a>, data: &'a mut T) -> Self {
        Self {
            rect,
            id,
            ctx,
            data,
        }
    }

    pub fn show(mut self, ui: &mut Ui) -> [Response; 2] {
        [self.handle(ui, true), self.handle(ui, false)]
    }

    fn handle(&mut self, ui: &mut Ui, start: bool) -> Response {
        let height = (self.rect.height() / 2.0).min(5.0);
        let zone = Rect::from_x_y_ranges(
            self.rect.x_range(),
            if start {
                Rangef::from(self.rect.max.y - height..=self.rect.max.y)
            } else {
                Rangef::from(self.rect.min.y..=self.rect.min.y + height)
            },
        );
        let id = self.id.with(start);
        let mut response = ui
            .interact(zone, id, Sense::drag())
            .on_hover_and_drag_cursor(egui::CursorIcon::ResizeVertical);
        let precise_id = id.with("precise_y");
        if response.drag_started() {
            let beat = if start {
                self.data.start_beat()
            } else {
                self.data.end_beat()
            };
            ui.data_mut(|data| data.insert_temp(precise_id, self.ctx.beat_to_y(beat)));
        }
        if response.dragged() || response.drag_stopped() {
            if let Some(y) = ui.data(|data| data.get_temp::<f32>(precise_id)) {
                let delta = if response.drag_stopped() {
                    ui.input(|input| input.pointer.delta().y)
                } else {
                    response.drag_delta().y
                };
                let y = y + delta;
                ui.data_mut(|data| data.insert_temp(precise_id, y));
                let beat = self.ctx.settings.attach(self.ctx.y_to_beat_f32(y));
                let step = self.ctx.settings.minimum_beat();
                if start {
                    let before = self.data.start_beat();
                    let beat = beat.min(self.data.end_beat() - step).max(Beat::ZERO);
                    self.data.set_start_beat(beat);
                    if self.data.start_beat() != before {
                        response.mark_changed();
                    }
                } else {
                    let before = self.data.end_beat();
                    let beat = beat.max(self.data.start_beat() + step);
                    self.data.set_end_beat(beat);
                    if self.data.end_beat() != before {
                        response.mark_changed();
                    }
                }
            }
        }
        if response.drag_stopped() {
            ui.data_mut(|data| data.remove::<f32>(precise_id));
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::history::{open_document, EditorHistory, Edits, HistoryPlugin};
    use crate::id_index::IdIndexPlugin;
    use crate::tab::timeline::TimelineViewport;
    use crate::timeline::settings::TimelineSettings;
    use crate::timing::ChartTime;
    use crate::ui::edit::gesture_response;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::*;
    use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect};
    use phichain_chart::bpm_list::BpmList;
    use phichain_chart::note::NoteKind;
    use phichain_game::audio::AudioDuration;
    use std::time::Duration;

    fn frame(app: &mut App, ctx: &Context, entity: Entity, events: Vec<Event>) -> Rect {
        let mut rect = Rect::NOTHING;
        let _ = ctx.run(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 600.0))),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let mut note = *app.world().get::<Note>(entity).unwrap();
                    let mut state: SystemState<(TimelineContext, Edits)> =
                        SystemState::new(app.world_mut());
                    let (timeline, mut edits) = state.get_mut(app.world_mut());
                    rect = Rect::from_min_max(
                        egui::pos2(100.0, timeline.beat_to_y(note.end_beat())),
                        egui::pos2(200.0, timeline.beat_to_y(note.beat)),
                    );
                    let background =
                        ui.interact(ui.max_rect(), Id::new("background"), Sense::drag());
                    ui.interact(rect, Id::new("note"), Sense::click());
                    for response in
                        BeatRangeDragZone::new(rect, Id::new("hold"), &timeline, &mut note).show(ui)
                    {
                        gesture_response(&mut edits, &response, "resize hold", move |commands| {
                            commands.entity(entity).insert(note);
                        });
                    }
                    assert!(
                        !background.dragged(),
                        "the range handle must capture the drag before the background"
                    );
                    state.apply(app.world_mut());
                });
            },
        );
        app.update();
        rect
    }

    fn button(pos: Pos2, pressed: bool) -> Event {
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }
    }

    #[test]
    fn hold_handles_preview_clamp_and_undo_including_movement_on_release() {
        let mut app = App::new();
        app.add_plugins((IdIndexPlugin, HistoryPlugin))
            .insert_resource(TimelineSettings {
                zoom: 0.5,
                ..Default::default()
            })
            .insert_resource(BpmList::default())
            .insert_resource(ChartTime(0.0))
            .insert_resource(TimelineViewport(bevy::math::Rect::from_corners(
                Vec2::ZERO,
                Vec2::new(400.0, 600.0),
            )))
            .insert_resource(AudioDuration(Duration::from_secs(60)));
        let original = Note::new(
            NoteKind::Hold {
                hold_beat: Beat::from(2.0),
            },
            true,
            Beat::ONE,
            0.0,
            1.0,
        );
        let entity = app.world_mut().spawn(original).id();
        open_document(app.world_mut());
        let ctx = Context::default();
        let rect = frame(&mut app, &ctx, entity, vec![]);
        let pos = rect.center_top() + egui::vec2(0.0, 2.0);
        frame(
            &mut app,
            &ctx,
            entity,
            vec![Event::PointerMoved(pos), button(pos, true)],
        );
        frame(
            &mut app,
            &ctx,
            entity,
            vec![Event::PointerMoved(pos - egui::vec2(0.0, 40.0))],
        );
        let intermediate = app.world().get::<Note>(entity).unwrap().end_beat();
        assert!(intermediate > original.end_beat());
        let release = pos - egui::vec2(0.0, 90.0);
        frame(
            &mut app,
            &ctx,
            entity,
            vec![Event::PointerMoved(release), button(release, false)],
        );
        let after = *app.world().get::<Note>(entity).unwrap();
        assert_eq!(after.beat, original.beat);
        assert!(after.end_beat() > intermediate);
        assert!(!app.world().resource::<EditorHistory>().has_gesture());
        app.world_mut()
            .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
            .unwrap();
        assert_eq!(app.world().get::<Note>(entity), Some(&original));
        app.world_mut()
            .resource_scope(|world, mut history: Mut<EditorHistory>| history.redo(world))
            .unwrap();
        assert_eq!(app.world().get::<Note>(entity), Some(&after));

        let rect = frame(&mut app, &ctx, entity, vec![]);
        let pos = rect.center_bottom() - egui::vec2(0.0, 2.0);
        frame(
            &mut app,
            &ctx,
            entity,
            vec![Event::PointerMoved(pos), button(pos, true)],
        );
        frame(
            &mut app,
            &ctx,
            entity,
            vec![Event::PointerMoved(pos + egui::vec2(0.0, 900.0))],
        );
        let at_zero = app.world().get::<Note>(entity).unwrap();
        assert_eq!(at_zero.beat, Beat::ZERO);
        assert_eq!(at_zero.end_beat(), after.end_beat());
        frame(
            &mut app,
            &ctx,
            entity,
            vec![button(pos + egui::vec2(0.0, 900.0), false)],
        );
        app.world_mut()
            .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
            .unwrap();
        assert_eq!(app.world().get::<Note>(entity), Some(&after));
    }
}
