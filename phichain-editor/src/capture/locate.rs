//! Finding what a story points at, independently of the window size, layout and language
//!
//! Widgets are found in the accessibility tree by role and name, as a screen reader names them.
//! Panels and what timelines paint are computed from editor state with the editor's own layout.

use crate::identifier::{Identifier, IntoIdentifier};
use crate::tab::EditorTab;
use crate::timeline::container::{LaidOutTimeline, TimelineLayout};
use crate::timeline::event::{track_rect, EventTimeline};
use crate::timeline::note::NoteTimeline;
use crate::timeline::TimelineContext;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Query as EcsQuery, With};
use bevy_egui::egui;
use bevy_egui::egui::accesskit::{Action, Role};
use egui_dock::DockState;
use phichain_chart::beat::Beat;
use phichain_chart::event::LineEventKind;
use rust_i18n::t;

/// What `Locator`s resolve against: the screen as the editor last drew it
pub struct Screen<'a, 'w, 's> {
    pub tree: &'a Tree,
    pub dock: &'a DockState<Identifier>,
    pub timelines: &'a Timelines<'w, 's>,
}

/// The timelines as laid out on the timeline tab
#[derive(SystemParam)]
pub struct Timelines<'w, 's> {
    context: TimelineContext<'w>,
    layout: TimelineLayout<'w, 's>,
    notes: EcsQuery<'w, 's, (), With<NoteTimeline>>,
    events: EcsQuery<'w, 's, (), With<EventTimeline>>,
}

impl Timelines<'_, '_> {
    /// The only laid out timeline of a kind
    fn only(
        &self,
        is_kind: impl Fn(&LaidOutTimeline) -> bool,
        kind: &str,
    ) -> Result<egui::Rect, String> {
        let found = self.layout.timelines().filter(is_kind).collect::<Vec<_>>();
        match found.as_slice() {
            [timeline] => Ok(timeline.viewport),
            [] => Err(format!("no {kind} timeline is shown")),
            _ => Err(format!("{} {kind} timelines are shown", found.len())),
        }
    }

    fn note_timeline(&self) -> Result<egui::Rect, String> {
        self.only(|item| self.notes.contains(item.timeline), "note")
    }

    fn event_timeline(&self) -> Result<egui::Rect, String> {
        self.only(|item| self.events.contains(item.timeline), "event")
    }

    /// Where `beat` is drawn, if inside `viewport`
    fn beat_y(&self, viewport: egui::Rect, beat: f32) -> Result<f32, String> {
        let y = self.context.beat_to_y(Beat::from(beat));
        viewport
            .y_range()
            .contains(y)
            .then_some(y)
            .ok_or_else(|| format!("beat {beat} is outside the timeline, seek or zoom first"))
    }
}

#[derive(Clone)]
pub enum Locator {
    Query(Query),
    /// A tab's panel as laid out by the dock, with its tab bar
    Panel(Identifier),
    NoteTimeline,
    EventTimeline,
    /// A beat on one of the lanes of the note timeline, counted from the left
    NoteTimelineAt {
        beat: f32,
        lane: usize,
    },
    /// A beat in the middle of an event track
    EventTimelineAt {
        kind: LineEventKind,
        beat: f32,
    },
}

/// The area of the note timeline
pub fn note_timeline() -> Locator {
    Locator::NoteTimeline
}

/// The area of the event timeline
pub fn event_timeline() -> Locator {
    Locator::EventTimeline
}

/// A beat on a lane of the note timeline; beats between grid lines show how placing snaps
pub fn note_timeline_at(beat: f32, lane: usize) -> Locator {
    Locator::NoteTimelineAt { beat, lane }
}

/// A beat in the middle of an event track
pub fn event_timeline_at(kind: LineEventKind, beat: f32) -> Locator {
    Locator::EventTimelineAt { kind, beat }
}

impl From<Query> for Locator {
    fn from(query: Query) -> Self {
        Self::Query(query)
    }
}

/// Nodes of the accessibility tree, each step searching below the node the previous one found
///
/// Every step must match exactly one node.
/// A changed UI then fails the story instead of silently pointing it elsewhere.
#[derive(Clone)]
pub struct Query {
    steps: Vec<Step>,
}

#[derive(Clone, Default)]
struct Step {
    role: Option<Role>,
    /// Translation key of the name
    name: Option<String>,
}

pub fn by_role(role: Role) -> Query {
    Query::step(Step {
        role: Some(role),
        ..Default::default()
    })
}

/// The panel of a tab, with its tab bar
pub fn panel(tab: EditorTab) -> Panel {
    Panel(tab.into_identifier())
}

#[derive(Clone)]
pub struct Panel(Identifier);

impl Panel {
    /// Search the tab's content, the only part of the panel the accessibility tree has nodes for
    pub fn find(self, inner: Query) -> Query {
        by_role(Role::Pane)
            .name(format!("tab.{}.title", self.0))
            .find(inner)
    }
}

impl From<Panel> for Locator {
    fn from(panel: Panel) -> Self {
        Self::Panel(panel.0)
    }
}

impl Query {
    fn step(step: Step) -> Self {
        Self { steps: vec![step] }
    }

    fn last(&mut self) -> &mut Step {
        self.steps.last_mut().expect("a query has a step")
    }

    pub fn name(mut self, key: impl Into<String>) -> Self {
        self.last().name = Some(key.into());
        self
    }

    /// Search below what this query finds
    pub fn find(mut self, inner: Query) -> Self {
        self.steps.extend(inner.steps);
        self
    }

    fn resolve<'tree>(&self, tree: &'tree Tree) -> Result<Node<'tree>, String> {
        let mut scope = tree.state.root();
        for step in &self.steps {
            let name = step.name.as_ref().map(|key| t!(key.as_str()).into_owned());
            let found = descendants(scope)
                .skip(1)
                .filter(|node| step.role.is_none_or(|role| node.role() == role))
                .filter(|node| name.as_deref().is_none_or(|name| named(node, name)))
                .collect::<Vec<_>>();
            scope = match found.as_slice() {
                [node] => *node,
                [] => return Err(format!("nothing matches {}", step.describe())),
                nodes => {
                    return Err(format!(
                        "{} nodes match {}, narrow the query",
                        nodes.len(),
                        step.describe()
                    ))
                }
            };
        }
        Ok(scope)
    }
}

impl Step {
    fn describe(&self) -> String {
        let mut parts = vec![];
        if let Some(role) = self.role {
            parts.push(format!("role {role:?}"));
        }
        if let Some(name) = &self.name {
            parts.push(format!("name `{name}`"));
        }
        parts.join(", ")
    }
}

impl Locator {
    /// The region in logical window coordinates
    pub fn resolve(&self, screen: &Screen) -> Result<egui::Rect, String> {
        match self {
            Self::Query(query) => {
                let node = query.resolve(screen.tree)?;
                screen
                    .tree
                    .rect(node)
                    .ok_or_else(|| "the node has no bounds".to_owned())
            }
            Self::Panel(id) => screen
                .dock
                .iter_leaves()
                .map(|(_, leaf)| leaf)
                .find(|leaf| leaf.tabs.get(leaf.active.0) == Some(id))
                .map(|leaf| leaf.rect)
                .ok_or_else(|| format!("tab `{id}` is not open in front")),
            Self::NoteTimeline => screen.timelines.note_timeline(),
            Self::EventTimeline => screen.timelines.event_timeline(),
            Self::NoteTimelineAt { beat, lane } => {
                let viewport = screen.timelines.note_timeline()?;
                let lanes = screen.timelines.context.settings.lane_percents();
                let percent = lanes
                    .get(*lane)
                    .ok_or_else(|| format!("lane {lane} is beyond the {} lanes", lanes.len()))?;
                let y = screen.timelines.beat_y(viewport, *beat)?;
                let x = viewport.min.x + viewport.width() * percent;
                Ok(egui::Rect::from_center_size(
                    egui::pos2(x, y),
                    egui::Vec2::ZERO,
                ))
            }
            Self::EventTimelineAt { kind, beat } => {
                let viewport = screen.timelines.event_timeline()?;
                let y = screen.timelines.beat_y(viewport, *beat)?;
                let x = track_rect(viewport, *kind).center().x;
                Ok(egui::Rect::from_center_size(
                    egui::pos2(x, y),
                    egui::Vec2::ZERO,
                ))
            }
        }
    }
}

type Node<'tree> = kittest::AccessKitNode<'tree>;

/// Whether `node` is named `name`
fn named(node: &Node<'_>, name: &str) -> bool {
    accessible_name(node).as_deref() == Some(name)
}

/// The name of a node, by its own text, the label it is labelled by, or else the region it is in
///
/// A control inside a named edit region is named after it, like a field in a labelled group.
/// Regions are not looked for beyond the tab or window, and labels themselves have no name.
fn accessible_name(node: &Node<'_>) -> Option<String> {
    if node.role() == Role::Label {
        return None;
    }
    let own = |node: &Node<'_>| node.label().filter(|name| !name.trim().is_empty());
    own(node).or_else(|| {
        std::iter::successors(node.parent(), |node| node.parent())
            .take_while(|node| !matches!(node.role(), Role::Pane | Role::Window))
            .find_map(|node| own(&node))
    })
}

/// `node` and everything below it, in tree order
fn descendants(node: Node<'_>) -> impl Iterator<Item = Node<'_>> {
    with_depth(node).map(|(node, _)| node)
}

/// Nodes in tree order with their depth
fn with_depth(root: Node<'_>) -> impl Iterator<Item = (Node<'_>, usize)> {
    let mut stack = vec![(root, 0)];
    std::iter::from_fn(move || {
        let (node, depth) = stack.pop()?;
        let children = node.children().collect::<Vec<_>>();
        stack.extend(children.into_iter().rev().map(|child| (child, depth + 1)));
        Some((node, depth))
    })
}

/// The editor's accessibility tree of the last frame
pub struct Tree {
    state: kittest::State,
    scale_factor: f32,
}

impl Tree {
    pub fn new(update: egui::accesskit::TreeUpdate, scale_factor: f32) -> Self {
        Self {
            state: kittest::State::new(update),
            scale_factor,
        }
    }

    /// AccessKit bounds are in physical pixels
    fn rect(&self, node: Node<'_>) -> Option<egui::Rect> {
        let bounds = node.bounding_box()?;
        let rect = egui::Rect::from_min_max(
            egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
            egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
        );
        Some(rect / self.scale_factor)
    }

    /// Every node with its role, name, value and bounds, for finding out why a locator failed
    pub fn dump(&self) -> String {
        with_depth(self.state.root())
            .map(|(node, depth)| format!("{}{}\n", "  ".repeat(depth), describe(&node)))
            .collect()
    }

    /// Interactive nodes that assistive technology cannot name, with the tab they are in
    ///
    /// Areas are interactive without being widgets, such as what timelines paint and the dock's own handles.
    pub fn audit(&self) -> Audit {
        let mut audit = Audit::default();
        for node in descendants(self.state.root()) {
            let data = node.data();
            let interactive =
                data.supports_action(Action::Click) || data.supports_action(Action::Focus);
            if !interactive || node.role() == Role::Label || accessible_name(&node).is_some() {
                continue;
            }
            let tab = std::iter::successors(node.parent(), |node| node.parent())
                .find(|node| node.role() == Role::Pane)
                .and_then(|pane| pane.label())
                .unwrap_or_else(|| "outside tabs".to_owned());
            let line = format!("[{tab}] {}", describe(&node));
            match node.role() {
                Role::Unknown | Role::Window => audit.areas.push(line),
                _ => audit.widgets.push(line),
            }
        }
        audit
    }
}

#[derive(Default)]
pub struct Audit {
    pub widgets: Vec<String>,
    pub areas: Vec<String>,
}

impl Audit {
    pub fn report(&self) -> String {
        format!(
            "Widgets without a name ({}):\n{}\nInteractive areas that are not widgets ({}):\n{}",
            self.widgets.len(),
            self.widgets
                .iter()
                .map(|line| format!("  {line}\n"))
                .collect::<String>(),
            self.areas.len(),
            self.areas
                .iter()
                .map(|line| format!("  {line}\n"))
                .collect::<String>(),
        )
    }
}

fn describe(node: &Node<'_>) -> String {
    let bounds = node
        .bounding_box()
        .map(|rect| {
            format!(
                " [{:.0}, {:.0}, {:.0}, {:.0}]",
                rect.x0, rect.y0, rect.x1, rect.y1
            )
        })
        .unwrap_or_default();
    format!(
        "{:?} name={:?} value={:?}{bounds}",
        node.role(),
        node.label(),
        node.value()
    )
}
