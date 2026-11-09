//! Naming controls for assistive technology, like `aria-labelledby` and `aria-label` on the web
//!
//! Widgets that show their own text are named by egui; these name the rest where they are drawn.
//! Stories of the documentation capture find controls by the same names.

use egui::Response;

pub trait AccessibleName {
    /// Name this control or region after a label drawn next to it
    fn labelled(self, label: &Response) -> Self;

    /// Name a control that shows no text of its own, also shown when hovering it
    fn named(self, name: impl Into<String>) -> Self;
}

impl AccessibleName for Response {
    fn labelled(self, label: &Response) -> Self {
        // egui names widgets drawn with empty text by that empty text, which would hide the label
        self.ctx.accesskit_node_builder(self.id, |node| {
            if node.label().is_some_and(str::is_empty) {
                node.clear_label();
            }
        });
        self.labelled_by(label.id)
    }

    fn named(self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.ctx
            .accesskit_node_builder(self.id, |node| node.set_label(name.as_str()));
        self.on_hover_text(name)
    }
}
