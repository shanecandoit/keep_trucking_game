use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::world;

#[derive(Component)]
pub struct DebugText;

pub fn render(commands: &mut Commands) {
    commands.spawn((
        Text::new("Click a tile to send the truck there\nMouse debug:"),
        DebugText,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(24.0),
            left: Val::Px(24.0),
            ..default()
        },
    ));
}

pub fn update_cursor(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    debug_text: &mut Query<&mut Text, With<DebugText>>,
    map: &world::TownMap,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Some(screen) = window.cursor_position() else {
        return;
    };
    let Ok(iso_world) = camera.viewport_to_world_2d(camera_transform, screen) else {
        return;
    };
    let top_down = world::iso_to_top_down(iso_world - world::board_origin(map));
    let grid = IVec2::new(top_down.x.round() as i32, top_down.y.round() as i32);

    for mut text in debug_text.iter_mut() {
        *text = Text::new(format!(
            "Click a tile to send the truck there\nMouse debug:\nscreen: ({:.0}, {:.0})\niso world: ({:.1}, {:.1})\ntop-down: ({:.2}, {:.2})\ntile: ({}, {})",
            screen.x, screen.y, iso_world.x, iso_world.y, top_down.x, top_down.y, grid.x, grid.y
        ));
    }
}
