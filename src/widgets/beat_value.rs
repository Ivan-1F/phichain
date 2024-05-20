use crate::chart::beat::Beat;
use egui::{Response, Ui, Widget};
use num::Rational32;
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

    pub fn clamp_range(mut self, range: RangeInclusive<Beat>) -> Self {
        self.clamp_range = range;
        self
    }
}

impl<'a> Widget for BeatValue<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let mut value = self.beat.value();

            let mut whole = self.beat.beat();
            let mut numer = self.beat.numer();
            let mut denom = self.beat.denom();

            ui.add(
                egui::DragValue::new(&mut whole)
                    .clamp_range(0..=u32::MAX)
                    .speed(1),
            );
            ui.add(
                egui::DragValue::new(&mut numer)
                    .clamp_range(0..=u32::MAX)
                    .speed(1),
            );
            ui.add(
                egui::DragValue::new(&mut denom)
                    .clamp_range(1..=u32::MAX)
                    .speed(1),
            );

            ui.add(
                egui::DragValue::new(&mut value)
                    .clamp_range(0.0..=f32::MAX)
                    .custom_formatter(|x, _| format!("{:?}", Beat::from(x as f32)))
                    .speed(0.01),
            );

            if whole != self.beat.beat() || numer != self.beat.numer() || denom != self.beat.denom()
            {
                *self.beat = clamp_to_range(
                    Beat::new(whole, Rational32::new(numer, denom)),
                    &self.clamp_range,
                );
            } else if value != self.beat.value() {
                *self.beat = clamp_to_range(value.into(), &self.clamp_range);
            }
        })
        .response
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
