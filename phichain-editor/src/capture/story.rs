use super::locate::Locator;
use super::runner::Command;
use crate::identifier::IntoIdentifier;
use crate::tab::EditorTab;
use phichain_chart::project::ProjectMeta;

/// The commands of one documentation page, run in a single editor session from a new project
pub struct Story {
    pub project: ProjectMeta,
    pub commands: Vec<Command>,
}

/// A region named in a manifest, for the documentation to mark
pub struct Anchor {
    pub name: String,
    pub locator: Locator,
}

impl Anchor {
    /// The content region of a tab, named `tab.<id>`
    pub fn tab(tab: EditorTab) -> Self {
        let id = tab.into_identifier();
        Self {
            name: format!("tab.{id}"),
            locator: Locator::Tab(id),
        }
    }
}

impl Story {
    pub fn new(project: ProjectMeta) -> Self {
        Self {
            project,
            commands: vec![],
        }
    }

    /// Capture the window as the image `id`, with the regions of `anchors`
    pub fn image(&mut self, id: &'static str, anchors: impl IntoIterator<Item = Anchor>) {
        self.commands.push(Command::Image {
            id,
            anchors: anchors.into_iter().collect(),
        });
    }
}
