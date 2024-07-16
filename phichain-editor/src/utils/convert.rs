/// Helper trait to convert math related types between bevy and egui
pub trait BevyEguiConvert {
    type Egui;
    type Bevy;

    fn into_egui(self) -> Self::Egui;
    #[allow(dead_code)]
    fn into_bevy(self) -> Self::Bevy;
}

impl BevyEguiConvert for egui::Vec2 {
    type Egui = egui::Vec2;
    type Bevy = bevy::math::Vec2;

    fn into_egui(self) -> Self::Egui {
        self
    }

    fn into_bevy(self) -> Self::Bevy {
        Self::Bevy::new(self.x, self.y)
    }
}

impl BevyEguiConvert for bevy::math::Vec2 {
    type Egui = egui::Vec2;
    type Bevy = bevy::math::Vec2;

    fn into_egui(self) -> Self::Egui {
        egui::Vec2::new(self.x, self.y)
    }

    fn into_bevy(self) -> Self::Bevy {
        self
    }
}

impl BevyEguiConvert for egui::Rect {
    type Egui = egui::Rect;
    type Bevy = bevy::math::Rect;

    fn into_egui(self) -> Self::Egui {
        self
    }

    fn into_bevy(self) -> Self::Bevy {
        Self::Bevy::new(self.min.x, self.min.y, self.max.x, self.max.y)
    }
}

impl BevyEguiConvert for bevy::math::Rect {
    type Egui = egui::Rect;
    type Bevy = bevy::math::Rect;

    fn into_egui(self) -> Self::Egui {
        Self::Egui::from_min_max(
            self.min.into_egui().to_pos2(),
            self.max.into_egui().to_pos2(),
        )
    }

    fn into_bevy(self) -> Self::Bevy {
        self
    }
}
