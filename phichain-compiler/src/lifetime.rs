use crate::range::find_ranges;
use crate::range::{merge_ranges, BeatRange};
use itertools::Itertools;
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::serialization::SerializedLine;
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct LineLifetime(Vec<BeatRange>);

impl Debug for LineLifetime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl LineLifetime {
    pub fn is_unit(&self) -> bool {
        self.0.is_empty()
    }

    pub fn entries(&self) -> &Vec<BeatRange> {
        &self.0
    }

    #[allow(dead_code)]
    pub fn is_valid(&self, beat: Beat) -> bool {
        self.0.iter().any(|range| range.contains(&beat))
    }

    pub fn overlaps(&self, other: &LineLifetime) -> bool {
        for range1 in &self.0 {
            for range2 in &other.0 {
                if range1.end > range2.start && range2.end > range1.start {
                    return true;
                }
            }
        }

        false
    }
}

pub fn find_lifetime(line: &SerializedLine) -> LineLifetime {
    let mut ranges: Vec<BeatRange> = vec![];

    // TODO: optimize lifetime for notes
    // currently: if the line have note, create a range from 0 to the last note's end beat
    if let Some(last_note) = line.notes.iter().sorted_by_key(|note| note.beat).last() {
        ranges.push(beat!(0)..last_note.end_beat());
    }

    let visible_ranges = find_ranges(&line.events, |state| state.is_visible());
    ranges.extend(visible_ranges);

    LineLifetime(merge_ranges(&ranges))
}

pub fn merge_lifetimes(lifetimes: Vec<LineLifetime>) -> LineLifetime {
    LineLifetime(merge_ranges(
        &lifetimes
            .iter()
            .flat_map(|l| l.0.clone())
            .collect::<Vec<_>>(),
    ))
}
