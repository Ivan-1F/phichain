#[derive(Default)]
pub(super) struct ObjectCounts {
    pub notes: usize,
    pub events: usize,
    pub tracks: usize,
}

impl ObjectCounts {
    pub fn text(&self) -> String {
        [
            (self.notes, "history.object.note", "history.object.notes"),
            (self.events, "history.object.event", "history.object.events"),
            (self.tracks, "history.object.track", "history.object.tracks"),
        ]
        .into_iter()
        .filter(|(count, _, _)| *count > 0)
        .map(|(count, singular, plural)| {
            t!(if count == 1 { singular } else { plural }, count = count).into_owned()
        })
        .collect::<Vec<_>>()
        .join(&t!("history.object.separator"))
    }
}
