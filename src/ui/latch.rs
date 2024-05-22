/// Interactable UI component where changes in-progress should be ignored,
/// only producing output when the interaction is fully finished
///
/// Takes a closure which inspects the mutable State, modifying the original value, returning if the value is confirmed
///
/// This function will return the old value in [`Some`] after the value is confirmed
///
/// # Example
///
/// ```
/// if let Some(from) = latch::latch(ui, "note", note.clone(), |ui| {
///     let mut finished = false;
///     let mut changed = false;
///     
///     let response = ui.add(egui::DragValue::new(&mut note.x).speed(1));
///     changed |= response.changed();
///     finished |= response.drag_stopped();
///     
///     finished && changed
/// })
/// {
///     let to = note;
///     // `note` changed from `from` to `to`    
/// }
/// ```
pub fn latch<State, F>(
    ui: &mut egui::Ui,
    id_src: impl std::hash::Hash,
    state: State,
    f: F,
) -> Option<State>
where
    F: FnOnce(&mut egui::Ui) -> bool,
    // bounds implied by insert_temp
    State: 'static + Clone + std::any::Any + Send + Sync,
{
    let persisted_id = egui::Id::new(id_src);

    let fn_response = f(ui);

    if ui
        .data(|data| data.get_temp::<State>(persisted_id))
        .is_none()
    {
        ui.data_mut(|data| data.insert_temp(persisted_id, state));
    }

    let from = ui.data(|data| data.get_temp::<State>(persisted_id).unwrap());

    if fn_response {
        ui.data_mut(|data| data.remove::<State>(persisted_id));
        Some(from)
    } else {
        None
    }
}
