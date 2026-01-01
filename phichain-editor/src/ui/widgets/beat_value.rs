use egui::{Response, Ui, Vec2, Widget};
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use std::cmp::Ordering;
use std::ops::RangeInclusive;

pub struct BeatValue<'a> {
    beat: &'a mut Beat,
    clamp_range: RangeInclusive<Beat>,
    reversed: bool,
    density: Option<u32>,
}

impl<'a> BeatValue<'a> {
    pub fn new(beat: &'a mut Beat) -> Self {
        Self {
            beat,
            clamp_range: Beat::MIN..=Beat::MAX,
            reversed: false,
            density: None,
        }
    }

    pub fn density(mut self, density: u32) -> Self {
        self.density = Some(density);
        self
    }

    pub fn range(mut self, range: RangeInclusive<Beat>) -> Self {
        self.clamp_range = range;
        self
    }

    pub fn reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
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

            let display_beat = *self.beat;

            let speed = self.density.map(|d| 1.0 / d as f32).unwrap_or(0.01);

            ui.spacing_mut().item_spacing.x = 4.0;

            let (response_value, response_whole, response_numer, response_denom) = if self.reversed
            {
                let response_value = ui.add(
                    egui::DragValue::new(&mut value)
                        .range(0.0..=f32::MAX)
                        .custom_formatter(move |_, _| format!("{:?}", display_beat))
                        .speed(speed),
                );
                ui.spacing_mut().interact_size = Vec2::new(20.0, 18.0);
                let response_denom = ui.add(
                    egui::DragValue::new(&mut denom)
                        .range(1..=u32::MAX)
                        .speed(1),
                );
                let response_numer = ui.add(
                    egui::DragValue::new(&mut numer)
                        .range(0..=u32::MAX)
                        .speed(1),
                );
                ui.spacing_mut().interact_size = Vec2::new(40.0, 18.0);
                let response_whole = ui.add(
                    egui::DragValue::new(&mut whole)
                        .range(0..=u32::MAX)
                        .speed(1),
                );

                (
                    response_value,
                    response_whole,
                    response_numer,
                    response_denom,
                )
            } else {
                let response_whole = ui.add(
                    egui::DragValue::new(&mut whole)
                        .range(0..=u32::MAX)
                        .speed(1),
                );
                ui.spacing_mut().interact_size = Vec2::new(20.0, 18.0);
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
                ui.spacing_mut().interact_size = Vec2::new(40.0, 18.0);

                let response_value = ui.add(
                    egui::DragValue::new(&mut value)
                        .range(0.0..=f32::MAX)
                        .custom_formatter(move |_, _| format!("{:?}", display_beat))
                        .speed(speed),
                );

                (
                    response_value,
                    response_whole,
                    response_numer,
                    response_denom,
                )
            };

            let response = response_value
                .union(response_whole)
                .union(response_numer)
                .union(response_denom);

            if whole != self.beat.beat() || numer != self.beat.numer() || denom != self.beat.denom()
            {
                *self.beat = clamp_to_range(beat!(whole, numer, denom), &self.clamp_range);
            } else if value != self.beat.value() {
                let new_beat = if let Some(density) = self.density {
                    beat::utils::attach(value, density)
                } else {
                    value.into()
                };
                *self.beat = clamp_to_range(new_beat, &self.clamp_range);
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

#[allow(dead_code)]
pub trait BeatExt {
    fn beat(&mut self, beat: &mut Beat) -> Response;
}

impl BeatExt for Ui {
    fn beat(&mut self, beat: &mut Beat) -> Response {
        BeatValue::new(beat).ui(self)
    }
}
