use egui::{Response, Ui, Widget};
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use std::cmp::Ordering;
use std::ops::RangeInclusive;

pub struct BeatValue<'a> {
    beat: &'a mut Beat,
    clamp_range: RangeInclusive<Beat>,
}

impl<'a> BeatValue<'a> {
    pub fn new(beat: &'a mut Beat) -> Self {
        Self {
            beat,
            clamp_range: Beat::MIN..=Beat::MAX,
        }
    }

    pub fn range(mut self, range: RangeInclusive<Beat>) -> Self {
        self.clamp_range = range;
        self
    }
}

impl Widget for BeatValue<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let mut value = self.beat.value();

            let mut whole = self.beat.beat();
            let mut numer = self.beat.numer();
            let mut denom = self.beat.denom();

            let response_whole = ui.add(
                egui::DragValue::new(&mut whole)
                    .range(0..=u32::MAX)
                    .speed(1),
            );
            let response_numer = ui.add(
                egui::DragValue::new(&mut numer)
                    .range(0..=u32::MAX)
                    .speed(1),
            );
            let response_denom = ui.add(
                egui::DragValue::new(&mut denom)
                    .range(1..=u32::MAX)
                    .speed(1),
            );

            let response_value = ui.add(
                egui::DragValue::new(&mut value)
                    .range(0.0..=f32::MAX)
                    .custom_formatter(|x, _| format!("{:?}", Beat::from(x as f32)))
                    .speed(0.01),
            );

            fn has_focus_changed(response: &Response) -> bool {
                response.has_focus() || response.lost_focus() || response.gained_focus()
            }

            // check which widget's id as the id for the whole widget,
            // so that the focus events work as expected
            // we assume that only one widget will be focused at the same time
            let response = if has_focus_changed(&response_whole) {
                response_whole
                    .union(response_numer)
                    .union(response_denom)
                    .union(response_value)
            } else if has_focus_changed(&response_numer) {
                response_numer
                    .union(response_whole)
                    .union(response_denom)
                    .union(response_value)
            } else if has_focus_changed(&response_denom) {
                response_denom
                    .union(response_numer)
                    .union(response_whole)
                    .union(response_value)
            } else {
                // has_focus_changed(response_value) or nothing changed
                response_value
                    .union(response_denom)
                    .union(response_numer)
                    .union(response_whole)
            };

            if whole != self.beat.beat() || numer != self.beat.numer() || denom != self.beat.denom()
            {
                *self.beat = clamp_to_range(beat!(whole, numer, denom), &self.clamp_range);
            } else if value != self.beat.value() {
                *self.beat = clamp_to_range(value.into(), &self.clamp_range);
            }

            response
        })
        .inner
    }
}

fn clamp_to_range(x: Beat, range: &RangeInclusive<Beat>) -> Beat {
    let (mut min, mut max) = (*range.start(), *range.end());

    if min.cmp(&max) == Ordering::Greater {
        (min, max) = (max, min);
    }

    match x.cmp(&min) {
        Ordering::Less | Ordering::Equal => min,
        Ordering::Greater => match x.cmp(&max) {
            Ordering::Greater | Ordering::Equal => max,
            Ordering::Less => x,
        },
    }
}

pub trait BeatExt {
    fn beat(&mut self, beat: &mut Beat) -> Response;
}

impl BeatExt for Ui {
    fn beat(&mut self, beat: &mut Beat) -> Response {
        BeatValue::new(beat).ui(self)
    }
}
