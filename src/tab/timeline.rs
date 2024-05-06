use bevy::{prelude::*, render::camera::Viewport};

pub struct TimelineTabPlugin;

impl Plugin for TimelineTabPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimelineViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)))
            .add_systems(Update, update_note_timeline_camera_viewport);
    }
}

#[derive(Resource, Debug)]
pub struct TimelineViewport(pub Rect);

impl TimelineViewport {
    pub fn note_timeline_viewport(&self) -> Rect {
        Rect::from_corners(
            self.0.min,
            Vec2 {
                x: self.0.min.x + self.0.width() / 3.0 * 2.0,
                y: self.0.max.y,
            },
        )
    }

    pub fn event_timeline_viewport(&self) -> Rect {
        Rect::from_corners(
            Vec2 {
                x: self.0.min.x + self.0.width() / 3.0 * 2.0,
                y: self.0.min.y,
            },
            self.0.max,
        )
    }
}

#[derive(Component)]
pub struct NoteTimelineCamera;

#[derive(Component)]
pub struct EventTimelineCamera;

fn update_note_timeline_camera_viewport(
    mut note_timeline_camera_query: Query<
        &mut Camera,
        (With<NoteTimelineCamera>, Without<EventTimelineCamera>),
    >,
    mut event_timeline_camera_query: Query<
        &mut Camera,
        (With<EventTimelineCamera>, Without<NoteTimelineCamera>),
    >,
    window_query: Query<&Window>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    timeline_viewport: Res<TimelineViewport>,
) {
    let mut note_timeline_camera = note_timeline_camera_query.single_mut();
    let mut event_timeline_camera = event_timeline_camera_query.single_mut();
    let window = window_query.single();

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let set_viewport = |camera: &mut Camera, viewport_pos: Vec2, viewport_size: Vec2| {
        let viewport_pos = viewport_pos * scale_factor;
        let viewport_size = viewport_size * scale_factor;

        if viewport_pos.x < 0.0
            || viewport_pos.y < 0.0
            || viewport_size.x <= 0.0
            || viewport_size.y <= 0.0
            || viewport_pos.x + viewport_size.x > window.width() * scale_factor
            || viewport_pos.y + viewport_size.y > window.height() * scale_factor
        {
            println!("{}x{}", window.width(), window.height());
            return;
        }

        camera.viewport = Some(Viewport {
            physical_position: viewport_pos.as_uvec2(),
            physical_size: viewport_size.as_uvec2(),
            depth: 0.0..1.0,
        });
    };

    set_viewport(
        &mut note_timeline_camera,
        timeline_viewport.note_timeline_viewport().min,
        timeline_viewport.note_timeline_viewport().size(),
    );

    set_viewport(
        &mut event_timeline_camera,
        timeline_viewport.event_timeline_viewport().min,
        timeline_viewport.event_timeline_viewport().size(),
    );
}
