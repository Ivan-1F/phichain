use crate::timeline::TimelineItem;
use egui::Rect;

/// A [`TimelineItem`] managed by a [`TimelineContainer`] with the fraction it takes of the whole timeline viewport
#[derive(Debug, Clone)]
pub struct ManagedTimeline {
    pub timeline: TimelineItem,
    pub fraction: f32,
}

/// A [`TimelineItem`] managed by a [`TimelineContainer`] with an allocated viewport
#[derive(Debug, Clone)]
pub struct AllocatedTimeline<'a> {
    pub timeline: &'a TimelineItem,
    pub viewport: Rect,
}

/// A container holds all the timelines on the timeline tab
///
/// Responsible for allocating viewports for each timeline
#[derive(Debug, Clone, Default)]
pub struct TimelineContainer {
    /// All [`ManagedTimeline`] this [`TimelineContainer`] holds
    ///
    /// This vector is guaranteed to be sorted by [`fraction`](ManagedTimeline.fraction)
    pub timelines: Vec<ManagedTimeline>,
}

impl TimelineContainer {
    /// Push a new [`TimelineItem`] to the right of the container
    pub fn push_right(&mut self, timeline: TimelineItem) {
        for timeline in &mut self.timelines {
            timeline.fraction /= 1.2;
        }
        self.timelines.push(ManagedTimeline {
            timeline,
            fraction: 1.0,
        });
    }

    /// Remove a timeline with given index from the container
    ///
    /// The rest timelines will fill the remaining space
    pub fn remove(&mut self, index: usize) {
        self.timelines.remove(index);
        if let Some(last) = self.timelines.last_mut() {
            last.fraction = 1.0;
        }
    }

    /// Offset the fraction of a timeline at the given index by some delta
    ///
    /// # Panics
    ///
    /// This function will panic if the index is out of bound.
    pub fn offset_timeline(&mut self, index: usize, by: f32) {
        let last = (index > 0) // usize will overflow when -1 if index is 0
            .then(|| self.timelines.get(index - 1).map(|x| x.fraction))
            .flatten();
        let next = self.timelines.get(index + 1).map(|x| x.fraction);
        let timeline = self.timelines.get_mut(index).unwrap();

        let new_fraction = timeline.fraction + by;
        timeline.fraction = new_fraction
            .max(last.unwrap_or(0.0) + 0.05)
            .min(next.map(|x| x - 0.05).unwrap_or(1.0));
    }

    /// Get the viewport of a timeline at the given index
    ///
    /// # Panics
    ///
    /// This function will panic if the index is out of bound
    pub fn get_timeline_viewport(&self, index: usize, whole: Rect) -> Rect {
        let mut viewport = whole;

        for (i, timeline) in self.timelines.iter().enumerate() {
            viewport.set_right(whole.left() + timeline.fraction * whole.width());

            if i == index {
                return viewport;
            }

            viewport.set_left(viewport.right());
        }

        panic!("Index out of bound")
    }

    /// Allocate all the timelines in the given viewport
    pub fn allocate(&self, viewport: Rect) -> Vec<AllocatedTimeline> {
        self.timelines
            .iter()
            .enumerate()
            .map(|(index, timeline)| AllocatedTimeline {
                timeline: &timeline.timeline,
                viewport: self.get_timeline_viewport(index, viewport),
            })
            .collect()
    }
}
