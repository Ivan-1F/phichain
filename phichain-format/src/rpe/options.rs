#[derive(Debug, Clone)]
pub struct RpeInputOptions {
    /// If true, notes with `isFake = true` will be removed. Otherwise, it will retain as a real note
    pub remove_fake_notes: bool,
    /// If true, lines with non-empty `attachUI` will be removed. Otherwise, it will retain as a normal line
    pub remove_ui_controls: bool,
}
