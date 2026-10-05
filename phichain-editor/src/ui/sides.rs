//! Right-aligned controls with labels using the remaining space.

use egui::Ui;

pub trait SidesExt {
    fn sides<RetL, RetR>(
        self,
        left: impl FnOnce(&mut Ui) -> RetL,
        right: impl FnOnce(&mut Ui) -> RetR,
    ) -> (RetL, RetR);
}

impl SidesExt for &mut Ui {
    fn sides<RetL, RetR>(
        self,
        left: impl FnOnce(&mut Ui) -> RetL,
        right: impl FnOnce(&mut Ui) -> RetR,
    ) -> (RetL, RetR) {
        egui::Sides::new().shrink_left().show(self, left, right)
    }
}
