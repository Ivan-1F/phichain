use crate::identifier::Identifier;
use bevy_egui::egui;
use bevy_egui::egui::accesskit::Role;
use kittest::Queryable;
use rust_i18n::t;

/// Describes where something is on screen, independently of the window size, layout and language
pub enum Locator {
    /// The content region of a tab
    Tab(Identifier),
}

impl Locator {
    /// The region in logical window coordinates, or `None` when it is not on screen
    pub fn resolve(&self, tree: &Tree) -> Option<egui::Rect> {
        match self {
            Locator::Tab(id) => {
                let key = format!("tab.{id}.title");
                let title = t!(key.as_str());
                tree.rect(tree.root().query_by_role_and_label(Role::Pane, &title)?)
            }
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Locator::Tab(id) => format!("tab {id}"),
        }
    }
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

    fn root(&self) -> Node<'_> {
        Node(self.state.root())
    }

    /// AccessKit bounds are in physical pixels
    fn rect(&self, node: Node<'_>) -> Option<egui::Rect> {
        let bounds = node.0.bounding_box()?;
        let rect = egui::Rect::from_min_max(
            egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
            egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
        );
        Some(rect / self.scale_factor)
    }
}

/// Lets kittest queries run on the editor's tree
#[derive(Clone, Copy)]
struct Node<'tree>(kittest::AccessKitNode<'tree>);

impl std::fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        kittest::debug_fmt_node(self, f)
    }
}

impl<'tree> kittest::NodeT<'tree> for Node<'tree> {
    fn accesskit_node(&self) -> kittest::AccessKitNode<'tree> {
        self.0
    }

    fn new_related(&self, node: kittest::AccessKitNode<'tree>) -> Self {
        Self(node)
    }
}
