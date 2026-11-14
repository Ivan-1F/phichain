use super::locate::{panel, Locator};
use super::runner::{Command, Shot};
use super::FPS;
use crate::identifier::IntoIdentifier;
use crate::tab::EditorTab;
use bevy::input::keyboard::KeyCode;

/// Seconds the cursor rests on a target before clicking or pressing a key there
const REST_SECONDS: f64 = 0.4;

/// The commands of one documentation page, run in a single editor session from the home screen
#[derive(Default)]
pub struct Story {
    pub commands: Vec<Command>,
}

/// A region of an image named in its manifest, for the documentation to mark
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
    pub fn wait(&mut self, seconds: f64) {
        self.commands.push(Command::Wait(frames(seconds)));
    }

    /// Move to the target and rest there a moment, as a hand does before acting
    pub fn move_to(&mut self, target: impl Into<Locator>) {
        self.commands.push(Command::MoveTo(target.into()));
        self.wait(REST_SECONDS);
    }

    /// Move to the target and click it with the left button
    pub fn click(&mut self, target: impl Into<Locator>) {
        self.move_to(target);
        self.commands.push(Command::Click);
    }

    pub fn press(&mut self, key: KeyCode) {
        self.commands.push(Command::Press(key));
    }

    /// Type into whatever has keyboard focus
    pub fn type_text(&mut self, text: &'static str) {
        self.commands.push(Command::Type(text));
    }

    /// Pick a file the working directory starts with, such as [`MUSIC`](super::MUSIC), in the file dialog that is open
    pub fn choose(&mut self, file: &'static str) {
        self.commands.push(Command::Choose(file));
    }

    /// Move the chart to a beat
    pub fn seek(&mut self, beat: f32) {
        self.commands.push(Command::Seek(beat));
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

    /// Record the window as the video `id` while `record` runs
    ///
    /// The pointer starts out of the window, so it enters where the video first points instead of crossing from the story before.
    pub fn video(&mut self, id: &'static str, record: impl FnOnce(&mut Self)) {
        self.commands.push(Command::Leave);
        self.commands.push(Command::StartVideo(id));
        record(self);
        self.commands.push(Command::StopVideo);
    }

    /// Mark where step `step` of the instructions begins, for the documentation to follow along
    pub fn step(&mut self, step: u32, run: impl FnOnce(&mut Self)) {
        self.commands.push(Command::Step(step));
        run(self);
    }

    /// Outline the target in the video for a while, without affecting the editor
    pub fn highlight(&mut self, target: impl Into<Locator>, seconds: f64) {
        self.commands
            .push(Command::Highlight(target.into(), frames(seconds)));
    }

    /// List the interactive nodes on screen that have no name, in `<id>.audit.txt`
    pub fn audit(&mut self, id: &'static str) {
        self.commands.push(Command::Audit(id));
    }
}

fn frames(seconds: f64) -> u32 {
    (seconds * FPS).round() as u32
}
