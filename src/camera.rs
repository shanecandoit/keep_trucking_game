use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub const INITIAL_SCALE: f32 = 1.65;
const MIN_SCALE: f32 = 0.65;
const MAX_SCALE: f32 = 2.4;

#[derive(Resource, Default)]
pub struct PanState {
    dragging: bool,
    last_cursor: Option<Vec2>,
}

pub fn update(
    buttons: &ButtonInput<MouseButton>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: &mut Query<&mut Transform, (With<Camera>, Without<crate::truck::Truck>)>,
    state: &mut PanState,
) {
    let Ok(window) = windows.single() else {
        state.dragging = false;
        state.last_cursor = None;
        return;
    };

    if buttons.just_pressed(MouseButton::Right) {
        state.dragging = true;
        state.last_cursor = window.cursor_position();
    }

    if buttons.just_released(MouseButton::Right) {
        state.dragging = false;
        state.last_cursor = None;
    }

    if !state.dragging || !buttons.pressed(MouseButton::Right) {
        return;
    }

    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Some(previous) = state.last_cursor.replace(cursor) else {
        return;
    };
    let delta = cursor - previous;

    // A camera moving right makes the world appear to move left, so invert
    // the drag delta to make the map follow the cursor like a grabbed sheet.
    let world_delta = Vec3::new(-delta.x, delta.y, 0.0);
    for mut transform in cameras.iter_mut() {
        transform.translation += world_delta;
    }
}

pub fn zoom(mut wheel: EventReader<MouseWheel>, mut cameras: Query<&mut Projection, With<Camera>>) {
    let amount = wheel.read().fold(0.0, |total, event| {
        total
            + match event.unit {
                MouseScrollUnit::Line => event.y,
                MouseScrollUnit::Pixel => event.y / 20.0,
            }
    });
    if amount == 0.0 {
        return;
    }

    for mut projection in cameras.iter_mut() {
        if let Projection::Orthographic(orthographic) = projection.as_mut() {
            orthographic.scale =
                (orthographic.scale * 0.88_f32.powf(amount)).clamp(MIN_SCALE, MAX_SCALE);
        }
    }
}
