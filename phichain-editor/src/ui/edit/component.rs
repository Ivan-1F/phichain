use super::gesture_response;
use crate::editing::history::Edits;
use crate::ui::sides::SidesExt;
use bevy::prelude::*;
use egui::{Response, Ui, UiBuilder, Widget, WidgetText};
use std::hash::Hash;

/// A frame-local component value shared by its editable fields.
/// Writes go through Edits; undo snapshots remain owned by the ECS recorder.
pub struct ComponentEditor<'a, 'w, 's, T> {
    edits: &'a mut Edits<'w, 's>,
    entity: Entity,
    value: T,
    description: String,
}

impl<'w, 's> Edits<'w, 's> {
    /// Edit an existing, registered document component on an identified entity.
    ///
    /// Clones the current value for this frame. Creating or dropping the context
    /// neither writes to ECS nor starts or commits a history entry. Fields queue
    /// their writes individually and share the latest value within this context.
    pub fn component<'a, T>(
        &'a mut self,
        entity: Entity,
        value: &T,
        description: impl Into<String>,
    ) -> ComponentEditor<'a, 'w, 's, T>
    where
        T: Component + Clone + PartialEq,
    {
        ComponentEditor {
            edits: self,
            entity,
            value: value.clone(),
            description: description.into(),
        }
    }
}

impl<T: Component + Clone + PartialEq> ComponentEditor<'_, '_, '_, T> {
    /// The latest value, including fields already edited in this frame.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Draw a labeled property using the inspector's left/right layout.
    ///
    /// `key` identifies the UI field, not a reflected component path. `draw`
    /// edits the frame-local component through one adapted widget, which may
    /// have several internal controls. Its interaction determines whether the
    /// write is a discrete operation or part of a continuous gesture.
    ///
    /// Each invocation describes one edit region. Unrelated operations must use
    /// separate fields; the callback is not an arbitrary transaction detector.
    pub fn field<R>(
        &mut self,
        ui: &mut Ui,
        key: impl Hash,
        label: impl Into<WidgetText>,
        draw: impl FnOnce(&mut FieldUi<'_>, &mut T) -> R,
    ) -> R {
        ui.sides(|ui| ui.label(label), |ui| self.edit(ui, key, draw))
            .1
    }

    /// Draw an edit region without a label, for controls such as timeline handles.
    ///
    /// Uses the same writeback and interaction policy as `field`. Identity is
    /// scoped to the parent UI, entity and key, independently of rendering order.
    /// The callback must report one operation through `add` or `custom`. Changes in
    /// that callback belong to this region; only a changed value is written back.
    /// Disabled regions discard even changes caused by a widget's value clamping.
    pub fn edit<R>(
        &mut self,
        ui: &mut Ui,
        key: impl Hash,
        draw: impl FnOnce(&mut FieldUi<'_>, &mut T) -> R,
    ) -> R {
        let id = ui.id().with((self.entity, key));
        ui.scope_builder(UiBuilder::new().id(id), |ui| {
            let before = self.value.clone();
            let enabled = ui.is_enabled();
            let mut field_ui = FieldUi { ui, response: None };
            let result = draw(&mut field_ui, &mut self.value);
            let response = field_ui.response.expect("an edit region needs one widget");
            if !enabled {
                self.value = before;
                return result;
            }
            let changed = self.value != before;
            let entity = self.entity;
            match response {
                EditResponse::Once(response) => {
                    if changed && response.changed() {
                        let next = self.value.clone();
                        self.edits.once(self.description.clone(), move |commands| {
                            commands.entity(entity).insert(next);
                        });
                    }
                }
                EditResponse::Gesture(responses) => {
                    for response in responses {
                        let next = (changed && response.changed()).then(|| self.value.clone());
                        gesture_response(
                            self.edits,
                            &response,
                            self.description.clone(),
                            move |commands| {
                                if let Some(next) = next {
                                    commands.entity(entity).insert(next);
                                }
                            },
                        );
                    }
                }
            }
            result
        })
        .inner
    }
}

/// An edit region's widget host. It deliberately does not dereference to egui::Ui:
/// document widgets report their interaction policy through EditWidget or `custom`.
pub struct FieldUi<'a> {
    ui: &'a mut Ui,
    response: Option<EditResponse>,
}

impl FieldUi<'_> {
    /// Add the region's widget and retain its interaction for component writeback.
    /// Compound widgets adapt their internal controls once, outside the inspector.
    pub fn add(&mut self, widget: impl EditWidget) -> Response {
        assert!(
            self.response.is_none(),
            "use a separate region for each widget"
        );
        let response = widget.edit_ui(self.ui);
        self.record(response)
    }

    /// Use native egui layout and explicitly report one edit operation.
    /// Independent operations need separate regions so each retains its own value.
    pub fn custom(&mut self, draw: impl FnOnce(&mut Ui) -> EditResponse) -> Response {
        assert!(
            self.response.is_none(),
            "use a separate region for each edit"
        );
        let response = draw(self.ui);
        self.record(response)
    }

    fn record(&mut self, response: EditResponse) -> Response {
        let combined = match &response {
            EditResponse::Once(response) => response.clone(),
            EditResponse::Gesture(responses) => responses
                .iter()
                .cloned()
                .reduce(|response, other| response.union(other))
                .expect("an edit widget must report a response"),
        };
        self.response = Some(response);
        combined
    }

    /// A checkbox is a discrete edit even when it retains keyboard focus.
    pub fn checkbox(&mut self, value: &mut bool, text: impl Into<WidgetText>) -> Response {
        self.add(egui::Checkbox::new(value, text))
    }
}

/// Draw a widget using egui and report how its edits should be grouped.
/// Implementations must report value changes as well as idle interaction frames.
pub trait EditWidget {
    fn edit_ui(self, ui: &mut Ui) -> EditResponse;
}

/// Native control responses retained until the component edit callback completes.
/// Internal controls keep their own focus and drag identities. Data changes made
/// in one callback form one region update, not independently inferred operations.
pub enum EditResponse {
    Once(Response),
    Gesture(Vec<Response>),
}

impl EditWidget for egui::DragValue<'_> {
    fn edit_ui(self, ui: &mut Ui) -> EditResponse {
        EditResponse::Gesture(vec![self.ui(ui)])
    }
}

impl EditWidget for egui::Checkbox<'_> {
    fn edit_ui(self, ui: &mut Ui) -> EditResponse {
        EditResponse::Once(self.ui(ui))
    }
}

impl EditWidget for egui::TextEdit<'_> {
    fn edit_ui(self, ui: &mut Ui) -> EditResponse {
        EditResponse::Gesture(vec![self.ui(ui)])
    }
}
