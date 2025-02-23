use crate::state::LineState;
use crate::utils;
use crate::utils::EventSequence;
use itertools::Itertools;
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::event::LineEvent;
use std::ops::Range;

pub type BeatRange = Range<Beat>;

/// Merges overlapping or adjacent ranges from a slice of ranges
///
/// This function takes a slice of `Range<T>` values and merges any ranges that overlap
/// or are directly adjacent (i.e. where one range's end is equal to the next range's start)
pub fn merge_ranges<T>(ranges: &[Range<T>]) -> Vec<Range<T>>
where
    T: Ord + Copy,
{
    let mut ranges = ranges.to_owned();
    ranges.sort_by_key(|r| r.start);

    ranges
        .into_iter()
        .coalesce(|mut a, b| {
            if b.start <= a.end {
                a.end = a.end.max(b.end);
                Ok(a)
            } else {
                Err((a, b))
            }
        })
        .collect()
}

pub fn find_ranges<F>(events: &Vec<LineEvent>, predicate: F) -> Vec<BeatRange>
where
    F: Fn(LineState) -> bool,
{
    let mut ranges: Vec<BeatRange> = vec![];
    let splits = utils::event_split_points(events);

    let minimum = beat!(1, 32);

    let mut current_range_start = None;

    for (from, to) in splits.iter().copied().tuple_windows() {
        let mut current = from;
        while current <= to {
            let state = events.evaluate_state(current);
            if predicate(state) {
                if current_range_start.is_none() {
                    current_range_start = Some(from);
                }
            } else if let Some(current_range_start) = current_range_start.take() {
                ranges.push(current_range_start..to);
            }

            current += minimum;
        }
    }

    if let Some(current_range_start) = current_range_start {
        ranges.push(current_range_start..Beat::MAX);
    }

    ranges
}

#[cfg(test)]
mod tests {
    mod merge_ranges {
        use crate::range::{merge_ranges, BeatRange};
        use phichain_chart::beat;

        #[test]
        fn test_empty_ranges() {
            let merged: Vec<BeatRange> = merge_ranges(&[]);
            assert!(merged.is_empty());
        }

        #[test]
        fn test_single_range() {
            let merged = merge_ranges(&[beat!(10)..beat!(20)]);
            assert_eq!(merged, vec![beat!(10)..beat!(20)]);
        }

        #[test]
        fn test_non_overlapping_ranges() {
            let merged = merge_ranges(&[
                beat!(0)..beat!(5),
                beat!(10)..beat!(15),
                beat!(20)..beat!(25),
            ]);
            assert_eq!(
                merged,
                vec![
                    beat!(0)..beat!(5),
                    beat!(10)..beat!(15),
                    beat!(20)..beat!(25)
                ]
            );
        }

        #[test]
        fn test_overlapping_ranges() {
            let merged = merge_ranges(&[beat!(0)..beat!(10), beat!(5)..beat!(15)]);
            assert_eq!(merged, vec![beat!(0)..beat!(15)]);
        }

        #[test]
        fn test_adjacent_ranges() {
            let merged = merge_ranges(&[beat!(0)..beat!(5), beat!(5)..beat!(10)]);
            assert_eq!(merged, vec![beat!(0)..beat!(10)]);
        }

        #[test]
        fn test_mixed_ranges() {
            let merged = merge_ranges(&[
                beat!(0)..beat!(10),
                beat!(20)..beat!(30),
                beat!(5)..beat!(25),
                beat!(40)..beat!(50),
            ]);
            assert_eq!(merged, vec![beat!(0)..beat!(30), beat!(40)..beat!(50)]);
        }
    }
}
