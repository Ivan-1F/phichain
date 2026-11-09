//! Finding what a story points at, independently of the window size, layout and language
//!
//! Widgets are found in the accessibility tree by role and name, as a screen reader names them.
//! Panels and what timelines paint are computed from editor state with the editor's own layout.

use crate::identifier::{Identifier, IntoIdentifier};
use crate::tab::EditorTab;
use bevy_egui::egui;
use bevy_egui::egui::accesskit::{Action, Role};
use egui_dock::DockState;
use rust_i18n::t;

/// What `Locator`s resolve against: the screen as the editor last drew it
pub struct Screen<'a> {
    pub tree: &'a Tree,
    pub dock: &'a DockState<Identifier>,
}

#[derive(Clone)]
pub enum Locator {
    Query(Query),
    /// A tab's panel as laid out by the dock, with its tab bar
    Panel(Identifier),
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

/// Nodes named by the translation of `key`, through their own text or the label they are labelled by
pub fn by_name(key: impl Into<String>) -> Query {
    Query::step(Step {
        name: Some(key.into()),
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
        }
    }
}

type Node<'tree> = kittest::AccessKitNode<'tree>;

/// Whether `node` is named `name`, by its own text or the label it is labelled by
///
/// Labels themselves do not count, so a name leads to what it describes.
fn named(node: &Node<'_>, name: &str) -> bool {
    node.role() != Role::Label && node.label().as_deref() == Some(name)
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
    /// Widgets count as named when they or an enclosing region inside their tab has a name.
    /// Areas are interactive without being widgets, such as what timelines paint and the dock's own handles.
    pub fn audit(&self) -> Audit {
        let mut audit = Audit::default();
        for node in descendants(self.state.root()) {
            let data = node.data();
            let interactive =
                data.supports_action(Action::Click) || data.supports_action(Action::Focus);
            if !interactive || node.role() == Role::Label || has_name(&node) {
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

fn has_name(node: &Node<'_>) -> bool {
    std::iter::successors(Some(*node), |node| node.parent())
        .take_while(|node| !matches!(node.role(), Role::Pane | Role::Window))
        .any(|node| node.label().is_some_and(|name| !name.trim().is_empty()))
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
