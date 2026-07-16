use bevy::prelude::*;
use bevy::window::PrimaryWindow;

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
