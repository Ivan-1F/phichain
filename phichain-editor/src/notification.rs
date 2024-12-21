use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{Align2, WidgetText};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

#[derive(Resource, Deref, DerefMut)]
pub struct ToastsStorage(Toasts);

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<ToastsStorage>()
            .add_systems(Update, show_egui_notifies);
    }
}

pub trait ToastsExt {
    fn info(&mut self, message: impl Into<WidgetText>);
    fn error(&mut self, message: impl Into<WidgetText>);
    fn success(&mut self, message: impl Into<WidgetText>);
}

impl ToastsExt for Toasts {
    fn info(&mut self, text: impl Into<WidgetText>) {
        self.add(Toast {
            text: text.into(),
            kind: ToastKind::Info,
            options: ToastOptions::default()
                .duration_in_seconds(8.0)
                .show_progress(true),
        });
    }

    fn error(&mut self, text: impl Into<WidgetText>) {
        self.add(Toast {
            text: text.into(),
            kind: ToastKind::Error,
            options: ToastOptions::default()
                .duration_in_seconds(8.0)
                .show_progress(true),
        });
    }

    fn success(&mut self, text: impl Into<WidgetText>) {
        self.add(Toast {
            text: text.into(),
            kind: ToastKind::Success,
            options: ToastOptions::default()
                .duration_in_seconds(8.0)
                .show_progress(true),
        });
    }
}

impl Default for ToastsStorage {
    fn default() -> Self {
        Self(Toasts::new().anchor(Align2::RIGHT_TOP, (-10.0, 10.0)))
    }
}

fn show_egui_notifies(mut context: Query<&mut EguiContext>, mut toasts: ResMut<ToastsStorage>) {
    if let Ok(mut ctx) = context.get_single_mut() {
        toasts.show(ctx.get_mut())
    }
}
