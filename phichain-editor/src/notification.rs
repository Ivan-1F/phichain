use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{Align2, WidgetText};
use egui_toast::{Toast, ToastKind, ToastOptions, ToastStyle, Toasts};

#[derive(Resource, Deref, DerefMut)]
pub struct ToastsStorage(Toasts);

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToastsStorage>()
            .add_systems(Update, show_egui_notifies_system);
    }
}

pub trait ToastsExt {
    fn info(&mut self, message: impl Into<WidgetText>);
    fn error(&mut self, message: impl Into<WidgetText>);
    fn success(&mut self, message: impl Into<WidgetText>);
}

fn create_toast(kind: ToastKind, text: WidgetText) -> Toast {
    Toast {
        text,
        kind,
        options: ToastOptions::default()
            .duration_in_seconds(8.0)
            .show_progress(true),
        style: ToastStyle::default(),
    }
}

impl ToastsExt for Toasts {
    fn info(&mut self, text: impl Into<WidgetText>) {
        self.add(create_toast(ToastKind::Info, text.into()));
    }

    fn error(&mut self, text: impl Into<WidgetText>) {
        self.add(create_toast(ToastKind::Error, text.into()));
    }

    fn success(&mut self, text: impl Into<WidgetText>) {
        self.add(create_toast(ToastKind::Success, text.into()));
    }
}

impl Default for ToastsStorage {
    fn default() -> Self {
        Self(Toasts::new().anchor(Align2::RIGHT_TOP, (-10.0, 10.0)))
    }
}

fn show_egui_notifies_system(
    mut context: Query<&mut EguiContext>,
    mut toasts: ResMut<ToastsStorage>,
) {
    if let Ok(mut ctx) = context.get_single_mut() {
        toasts.show(ctx.get_mut())
    }
}
