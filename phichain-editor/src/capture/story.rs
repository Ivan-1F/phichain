use super::locate::{panel, Locator};
use super::runner::{Command, Shot};
use super::FPS;
use crate::identifier::IntoIdentifier;
use crate::tab::EditorTab;
use phichain_chart::project::ProjectMeta;

/// Seconds the cursor takes to reach a target unless told otherwise
const MOVE_SECONDS: f64 = 0.4;

/// The commands of one documentation page, run in a single editor session from a new project
pub struct Story {
    pub project: ProjectMeta,
    pub commands: Vec<Command>,
}

/// A region named in a manifest, for the documentation to mark
#[derive(Clone)]
pub struct Anchor {
    pub name: String,
    pub locator: Locator,
}

impl Anchor {
    pub fn new(name: impl Into<String>, locator: impl Into<Locator>) -> Self {
        Self {
            name: name.into(),
            locator: locator.into(),
        }
    }

    /// The panel of a tab, named `tab.<id>`
    pub fn panel(tab: EditorTab) -> Self {
        Self::new(format!("tab.{}", tab.into_identifier()), panel(tab))
    }
}

impl Story {
    pub fn new(project: ProjectMeta) -> Self {
        Self {
            project,
            commands: vec![],
        }
    }

    pub fn move_to(&mut self, target: impl Into<Locator>) {
        self.move_to_over(target, MOVE_SECONDS);
    }

    pub fn move_to_over(&mut self, target: impl Into<Locator>, seconds: f64) {
        self.commands
            .push(Command::MoveTo(target.into(), frames(seconds).max(1)));
    }

    /// Move to the target and click it with the left button
    pub fn click(&mut self, target: impl Into<Locator>) {
        self.move_to(target);
        self.commands.push(Command::Click);
    }

    /// Capture the window as the image `id`, with the pointer out of the window
    pub fn image(&mut self, id: &'static str) -> &mut Shot {
        self.commands.push(Command::Leave);
        self.commands.push(Command::Image(Shot::new(id)));
        let Some(Command::Image(shot)) = self.commands.last_mut() else {
            unreachable!();
        };
        shot
    }

    /// List the interactive nodes on screen that have no name, in `<id>.audit.txt`
    pub fn audit(&mut self, id: &'static str) {
        self.commands.push(Command::Audit(id));
    }
}

fn frames(seconds: f64) -> u32 {
    (seconds * FPS).round() as u32
}
