use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::truck::Truck;
use crate::ui::{Focus, ScreenPanel};
use crate::world;

use super::SimClock;

const DEBUG_PANEL_BACKGROUND: Color = Color::srgba(0.055, 0.05, 0.04, 0.88);
const DEBUG_PANEL_TEXT: Color = Color::srgb(0.92, 0.90, 0.84);

#[derive(Component)]
pub struct DebugStats;

#[derive(Component)]
pub struct DebugCursor;

#[derive(Component)]
pub struct PauseOverlay;

pub fn render(commands: &mut Commands) {
    commands.spawn((
        Text::new(""),
        DebugStats,
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(DEBUG_PANEL_TEXT),
        BackgroundColor(DEBUG_PANEL_BACKGROUND),
        Interaction::default(),
        ScreenPanel,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(24.0),
            left: Val::Px(24.0),
            width: Val::Px(310.0),
            min_height: Val::Px(152.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
    ));
    commands.spawn((
        Text::new(""),
        DebugCursor,
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(DEBUG_PANEL_TEXT),
        BackgroundColor(DEBUG_PANEL_BACKGROUND),
        Interaction::default(),
        ScreenPanel,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(24.0),
            right: Val::Px(24.0),
            width: Val::Px(290.0),
            min_height: Val::Px(126.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
    ));
    commands.spawn((
        Text::new("⏸ paused"),
        PauseOverlay,
        TextColor(Color::srgb(0.95, 0.85, 0.2)),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(4.0),
            left: Val::Px(4.0),
            ..default()
        },
    ));
}

pub fn update_pause(
    sim_clock: Res<SimClock>,
    mut overlay: Query<&mut Visibility, With<PauseOverlay>>,
) {
    let visible = sim_clock.is_paused();
    for mut visibility in overlay.iter_mut() {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    time: &Time,
    sim_clock: &SimClock,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    trucks: &Query<(Entity, &mut Transform, &mut Truck)>,
    mut debug_stats: Query<&mut Text, With<DebugStats>>,
    mut debug_cursor: Query<&mut Text, (With<DebugCursor>, Without<DebugStats>)>,
    focus: &Focus,
    map: &world::TownMap,
) {
    let truck_count = trucks.iter().count();
    let fps = 1.0 / time.delta_secs();

    let selected_line = match focus.selected {
        Some(entity) => {
            let route = trucks
                .iter()
                .find(|(e, _, _)| *e == entity)
                .map(|(_, _, truck)| truck.route.as_slice())
                .unwrap_or_default();
            let route_len = route.len();
            let task = if route_len > 0 { "driving" } else { "idle" };
            let next = route
                .first()
                .map(|point| world::world_to_grid(map, point.truncate()));
            let destination = route
                .last()
                .map(|point| world::world_to_grid(map, point.truncate()));
            let route_coordinates = match (next, destination) {
                (Some(next), Some(destination)) => format!(
                    "next waypoint: ({}, {})\ndestination: ({}, {})",
                    next.x, next.y, destination.x, destination.y
                ),
                _ => "next waypoint: none\ndestination: none".to_string(),
            };
            format!(
                "selected entity: {entity:?}\nroute waypoints: {route_len}\n{route_coordinates}\ncurrent task: {task}"
            )
        }
        None => "selected entity: none\nroute waypoints: 0\nnext waypoint: none\ndestination: none\ncurrent task: none".to_string(),
    };

    for mut text in debug_stats.iter_mut() {
        *text = Text::new(format!(
            "FPS: {fps:>4.0}\nentities (trucks): {truck_count:>3}\nsim time: {:>8.1}s ({})\n{}",
            sim_clock.elapsed_secs(),
            sim_clock.speed_label(),
            selected_line
        ));
    }

    let Ok(window) = windows.single() else {
        return;
    };
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
    let hovered = world::in_board(map, grid);

    for mut text in debug_cursor.iter_mut() {
        *text = Text::new(format!(
            "Mouse debug\nscreen:    ({:>5.0}, {:>5.0})\niso world: ({:>7.1}, {:>7.1})\ntop-down:  ({:>6.2}, {:>6.2})\ntile:      ({:>3}, {:>3}){}",
            screen.x,
            screen.y,
            iso_world.x,
            iso_world.y,
            top_down.x,
            top_down.y,
            grid.x,
            grid.y,
            if hovered { "" } else { " (off-board)" }
        ));
    }
}
