use super::*;
use crate::timeline::container::{laid_out_timelines, ordered_timelines, spawn_timeline_container};
use crate::timeline::drag_selection::TimelineDragSelection;
use crate::timeline::event::EventTimeline;
use crate::timeline::kind::TimelineKindRegistrationExt;
use crate::timeline::note::NoteTimeline;
use crate::timeline::settings::TimelineSettings;
use crate::timeline::target::FollowSelectedLine;
use crate::timing::ChartTime;
use egui::{Context, Event, Id, Modifiers, PointerButton, Pos2, RawInput, Rect};
use phichain_chart::{bpm_list::BpmList, offset::Offset};
use phichain_game::audio::AudioDuration;
use std::time::Duration;

fn world_with_badges() -> World {
    let mut app = App::new();
    app.add_timeline_kind(
        NoteTimeline,
        |_: In<(Ui, Entity)>| {},
        |_: In<(Entity, Rect)>| vec![],
        |_: In<Entity>| "Notes".to_owned(),
    )
    .add_timeline_kind(
        EventTimeline,
        |_: In<(Ui, Entity)>| {},
        |_: In<(Entity, Rect)>| vec![],
        |_: In<Entity>| "Events".to_owned(),
    );
    let mut world = std::mem::take(app.world_mut());
    world.insert_resource(TimelineViewport(bevy::math::Rect::new(
        8.0, 8.0, 592.0, 392.0,
    )));
    world.insert_resource(TimelineSettings {
        show_waveform: false,
        ..Default::default()
    });
    world.init_resource::<BpmList>();
    world.insert_resource(Offset(0.0));
    world.insert_resource(ChartTime(0.0));
    world.insert_resource(AudioDuration(Duration::from_secs(60)));
    world.init_resource::<TimelineDragSelection>();
    world.init_resource::<Messages<crate::selection::Select>>();
    world.spawn(Window::default());
    spawn_timeline_container(&mut world);
    world
}

fn render(world: &mut World, ctx: &Context, events: Vec<Event>) -> egui::FullOutput {
    ctx.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(600.0, 400.0))),
            events,
            ..Default::default()
        },
        |ctx| {
            world
                .query::<&mut Window>()
                .single_mut(world)
                .unwrap()
                .set_cursor_position(ctx.pointer_hover_pos().map(|pos| Vec2::new(pos.x, pos.y)));
            egui::CentralPanel::default().show(ctx, |ui| {
                // Render the real panel, including its overlapping drag-selection surface.
                timeline_tab(In(ui.new_child(egui::UiBuilder::new())), world);
            });
        },
    )
}

fn frame(world: &mut World, ctx: &Context, events: Vec<Event>) -> Vec<(String, Pos2)> {
    let output = render(world, ctx, events);
    fn collect(shape: egui::Shape, texts: &mut Vec<(String, Pos2)>) {
        match shape {
            egui::Shape::Text(text) => texts.push((
                text.galley.text().into(),
                text.visual_bounding_rect().center(),
            )),
            egui::Shape::Vec(shapes) => shapes.into_iter().for_each(|shape| collect(shape, texts)),
            _ => {}
        }
    }
    let mut texts = vec![];
    for shape in output.shapes {
        collect(shape.shape, &mut texts);
    }
    texts
}

fn position(texts: &[(String, Pos2)], label: &str) -> Pos2 {
    texts
        .iter()
        .find(|(text, _)| text.contains(label))
        .unwrap_or_else(|| panic!("Missing {label:?} in {texts:?}"))
        .1
}

fn click(world: &mut World, ctx: &Context, pos: Pos2) -> Vec<(String, Pos2)> {
    frame(world, ctx, vec![Event::PointerMoved(pos)]);
    for pressed in [true, false] {
        frame(
            world,
            ctx,
            vec![Event::PointerButton {
                pos,
                pressed,
                button: PointerButton::Primary,
                modifiers: Modifiers::NONE,
            }],
        );
    }
    frame(world, ctx, vec![])
}

#[test]
fn badge_menu_inserts_then_removal_reaches_the_empty_state_and_can_create_again() {
    let mut world = world_with_badges();
    let before = ordered_timelines(&mut world);
    let ctx = Context::default();
    let texts = frame(&mut world, &ctx, vec![]);
    assert!(!texts
        .iter()
        .any(|(text, _)| text == egui_phosphor::regular::PLUS));

    let badge = laid_out_timelines(&mut world)[0].viewport;
    let pos = egui::pos2(badge.center().x, badge.top() + badge.height() * 0.1);
    let texts = frame(&mut world, &ctx, vec![Event::PointerMoved(pos)]);
    let add = position(&texts, egui_phosphor::regular::PLUS);
    frame(&mut world, &ctx, vec![Event::PointerMoved(add)]);
    let texts = frame(
        &mut world,
        &ctx,
        vec![Event::PointerButton {
            pos: add,
            pressed: true,
            button: PointerButton::Primary,
            modifiers: Modifiers::NONE,
        }],
    );
    // The full panel's background has a drag response even during this button click.
    assert!(ctx.dragged_id().is_some());
    position(&texts, egui_phosphor::regular::PLUS);
    for _ in 0..3 {
        let texts = frame(&mut world, &ctx, vec![]);
        position(&texts, egui_phosphor::regular::PLUS);
        position(&texts, egui_phosphor::regular::X);
    }
    frame(
        &mut world,
        &ctx,
        vec![Event::PointerButton {
            pos: add,
            pressed: false,
            button: PointerButton::Primary,
            modifiers: Modifiers::NONE,
        }],
    );
    let texts = frame(&mut world, &ctx, vec![]);
    let texts = click(
        &mut world,
        &ctx,
        position(&texts, &t!("tab.timeline.note_timeline")),
    );
    click(
        &mut world,
        &ctx,
        position(&texts, &t!("tab.timeline.binding")),
    );

    let after = ordered_timelines(&mut world);
    assert_eq!(after.len(), 3);
    assert_eq!((after[0], after[2]), (before[0], before[1]));
    assert!(world.get::<NoteTimeline>(after[1]).is_some());
    assert!(world.get::<FollowSelectedLine>(after[1]).is_some());
    assert!(!egui::Popup::is_id_open(
        &ctx,
        Id::new(("timeline_badge", before[0])).with("add_menu")
    ));

    for remaining in (0..3).rev() {
        let badge = laid_out_timelines(&mut world)[0].viewport;
        let pos = egui::pos2(badge.center().x, badge.top() + badge.height() * 0.1);
        let texts = frame(&mut world, &ctx, vec![Event::PointerMoved(pos)]);
        click(
            &mut world,
            &ctx,
            position(&texts, egui_phosphor::regular::X),
        );
        assert_eq!(ordered_timelines(&mut world).len(), remaining);
    }
    let texts = frame(&mut world, &ctx, vec![]);
    position(&texts, &t!("tab.timeline.empty_title"));
    let texts = click(
        &mut world,
        &ctx,
        position(&texts, &t!("tab.timeline.create")),
    );
    let texts = click(
        &mut world,
        &ctx,
        position(&texts, &t!("tab.timeline.event_timeline")),
    );
    click(
        &mut world,
        &ctx,
        position(&texts, &t!("tab.timeline.binding")),
    );
    let after = ordered_timelines(&mut world);
    assert_eq!(after.len(), 1);
    assert!(world.get::<EventTimeline>(after[0]).is_some());
}

#[test]
fn dragging_hides_actions_immediately_and_still_swaps_timelines() {
    let mut world = world_with_badges();
    let ctx = Context::default();
    let texts = frame(&mut world, &ctx, vec![]);
    let start = position(&texts, "Notes");
    let end = position(&texts, "Events");
    let before = ordered_timelines(&mut world);
    let texts = frame(&mut world, &ctx, vec![Event::PointerMoved(start)]);
    position(&texts, egui_phosphor::regular::PLUS);

    let texts = frame(
        &mut world,
        &ctx,
        vec![Event::PointerButton {
            pos: start,
            pressed: true,
            button: PointerButton::Primary,
            modifiers: Modifiers::NONE,
        }],
    );
    assert!(ctx.is_being_dragged(Id::new(("timeline_badge", before[0]))));
    let no_actions = |texts: &[(String, Pos2)]| {
        assert!(!texts.iter().any(|(text, _)| {
            text == egui_phosphor::regular::PLUS || text == egui_phosphor::regular::X
        }));
    };
    no_actions(&texts);
    no_actions(&frame(&mut world, &ctx, vec![Event::PointerMoved(end)]));
    no_actions(&frame(
        &mut world,
        &ctx,
        vec![Event::PointerButton {
            pos: end,
            pressed: false,
            button: PointerButton::Primary,
            modifiers: Modifiers::NONE,
        }],
    ));
    assert_eq!(ordered_timelines(&mut world), vec![before[1], before[0]]);
}

#[test]
fn expanded_badge_has_a_continuous_background_and_two_dividers() {
    let mut world = world_with_badges();
    let ctx = Context::default();
    let texts = frame(&mut world, &ctx, vec![]);
    let pos = position(&texts, "Notes");
    let output = render(&mut world, &ctx, vec![Event::PointerMoved(pos)]);
    let lines: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::LineSegment { points, stroke } if points[0].x == points[1].x => {
                assert_eq!(stroke.width, 1.0);
                Some(*points)
            }
            _ => None,
        })
        .collect();
    assert_eq!(lines.len(), 2);
    let height = lines[0][1].y - lines[0][0].y;
    let expected = Rect::from_min_max(
        lines[0][0] - egui::vec2(height, 0.0),
        lines[1][1] + egui::vec2(height, 0.0),
    );
    assert!(output.shapes.iter().any(|shape| matches!(
        &shape.shape,
        egui::Shape::Rect(rect) if rect.rect == expected && rect.fill == ctx.style().visuals.widgets.inactive.weak_bg_fill
    )));
}
