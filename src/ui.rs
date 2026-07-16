use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{truck::Truck, world};

const CYAN: Color = Color::srgb(0.15, 0.95, 0.95);

#[derive(Resource, Default)]
pub struct Focus {
    pub selected: Option<Entity>,
    pub hovered: Option<IVec2>,
    pub click_consumed: bool,
}

pub fn render(_commands: &mut Commands) {}

pub fn update(
    buttons: &ButtonInput<MouseButton>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    trucks: Query<(Entity, &GlobalTransform), With<Truck>>,
    focus: &mut Focus,
    mut gizmos: Gizmos,
) {
    focus.click_consumed = false;

    let Ok(window) = windows.single() else {
        draw_focus(focus, &trucks, &mut gizmos);
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        focus.hovered = None;
        draw_focus(focus, &trucks, &mut gizmos);
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        draw_focus(focus, &trucks, &mut gizmos);
        return;
    };
    let Ok(world_cursor) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        draw_focus(focus, &trucks, &mut gizmos);
        return;
    };

    let hovered = world::world_to_grid(world_cursor);
    focus.hovered = world::in_board(hovered).then_some(hovered);

    if buttons.just_pressed(MouseButton::Left)
        && let Some((entity, _)) = trucks
            .iter()
            .find(|(_, transform)| transform.translation().truncate().distance(world_cursor) < 36.0)
    {
        focus.selected = Some(entity);
        focus.click_consumed = true;
    }

    draw_focus(focus, &trucks, &mut gizmos);
}

fn draw_focus(
    focus: &Focus,
    trucks: &Query<(Entity, &GlobalTransform), With<Truck>>,
    gizmos: &mut Gizmos,
) {
    // Gizmos are intentionally drawn from the UI/focus layer so the visual
    // selection state remains independent of the truck's model rendering.
    if let Some(hovered) = focus.hovered {
        draw_dot(gizmos, world::grid_to_world(hovered));
    }

    if let Some(selected) = focus.selected
        && let Some((_, transform)) = trucks.iter().find(|(entity, _)| *entity == selected)
    {
        draw_tile_border(
            gizmos,
            world::world_to_grid(transform.translation().truncate()),
        );
    }
}

fn draw_dot(gizmos: &mut Gizmos, center: Vec2) {
    let radius = 4.0;
    gizmos.line_2d(center + Vec2::X * radius, center - Vec2::X * radius, CYAN);
    gizmos.line_2d(center + Vec2::Y * radius, center - Vec2::Y * radius, CYAN);
}

fn draw_tile_border(gizmos: &mut Gizmos, grid: IVec2) {
    let center = world::grid_to_world(grid);
    let half_width = world::TILE_WIDTH * 0.5;
    let half_height = world::TILE_HEIGHT * 0.5;
    let corners = [
        center + Vec2::new(0.0, half_height),
        center + Vec2::new(half_width, 0.0),
        center + Vec2::new(0.0, -half_height),
        center + Vec2::new(-half_width, 0.0),
    ];

    for index in 0..corners.len() {
        gizmos.line_2d(corners[index], corners[(index + 1) % corners.len()], CYAN);
    }
}
