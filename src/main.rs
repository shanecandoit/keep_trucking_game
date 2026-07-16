use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::window::PrimaryWindow;

const BOARD_WIDTH: i32 = 9;
const BOARD_HEIGHT: i32 = 9;
const TILE_WIDTH: f32 = 86.0;
const TILE_HEIGHT: f32 = 43.0;
const PLAYER_SIZE: f32 = 30.0;
const PLAYER_SPEED: f32 = 260.0;

#[derive(Component)]
struct Truck {
    destination: Option<Vec3>,
}

#[derive(Component)]
struct DebugText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Keep Trucking - Isometric Skeleton".into(),
                resolution: (1200.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, render)
        .add_systems(Update, update)
        .run();
}

/// The single gameplay entry point. Keep the high-level game loop here while
/// allowing each gameplay area to have its own small update function.
fn update(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut trucks: Query<(&mut Transform, &mut Truck)>,
    mut debug_text: Query<&mut Text, With<DebugText>>,
) {
    update_clicks(buttons, windows, cameras, &mut trucks);
    update_debug_cursor(windows, cameras, &mut debug_text);
    update_truck(time, &mut trucks);
}

/// Bevy performs the actual frame rendering. This setup function constructs
/// the entities and render components that Bevy will draw each frame.
fn render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let origin = board_origin();
    render_board(&mut commands, origin, &mut meshes, &mut materials);
    render_truck(&mut commands, origin, &mut meshes, &mut materials);
    render_ui(&mut commands);
}

fn render_board(
    commands: &mut Commands,
    origin: Vec2,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let diamond = meshes.add(isometric_tile_mesh());
    let dark = materials.add(ColorMaterial::from(Color::srgb(0.17, 0.28, 0.32)));
    let light = materials.add(ColorMaterial::from(Color::srgb(0.20, 0.33, 0.36)));

    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            let grid = IVec2::new(x, y);
            let position = grid_to_world(grid, origin);
            let material = if (x + y) % 2 == 0 {
                dark.clone()
            } else {
                light.clone()
            };

            commands.spawn((
                Mesh2d(diamond.clone()),
                MeshMaterial2d(material),
                Transform::from_translation(position.extend(0.0)),
            ));
        }
    }
}

fn isometric_tile_mesh() -> Mesh {
    let half_width = TILE_WIDTH * 0.5;
    let half_height = TILE_HEIGHT * 0.5;
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, half_height, 0.0],
            [half_width, 0.0, 0.0],
            [0.0, -half_height, 0.0],
            [-half_width, 0.0, 0.0],
        ],
    );
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}

fn render_truck(
    commands: &mut Commands,
    origin: Vec2,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let start = IVec2::new(BOARD_WIDTH / 2, BOARD_HEIGHT / 2);
    let truck = commands
        .spawn((
            Transform::from_translation(grid_to_world(start, origin).extend(2.0)),
            Truck { destination: None },
        ))
        .id();

    let top = meshes.add(face_mesh(&[
        [0.0, 18.0, 0.0],
        [PLAYER_SIZE, 5.0, 0.0],
        [0.0, -8.0, 0.0],
        [-PLAYER_SIZE, 5.0, 0.0],
    ]));
    let left = meshes.add(face_mesh(&[
        [-PLAYER_SIZE, 5.0, 0.0],
        [0.0, -8.0, 0.0],
        [0.0, -25.0, 0.0],
        [-PLAYER_SIZE, -12.0, 0.0],
    ]));
    let right = meshes.add(face_mesh(&[
        [0.0, -8.0, 0.0],
        [PLAYER_SIZE, 5.0, 0.0],
        [PLAYER_SIZE, -12.0, 0.0],
        [0.0, -25.0, 0.0],
    ]));

    let top_material = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.12, 0.12)));
    let left_material = materials.add(ColorMaterial::from(Color::srgb(0.78, 0.04, 0.04)));
    let right_material = materials.add(ColorMaterial::from(Color::srgb(0.55, 0.02, 0.02)));

    let top_entity = commands
        .spawn((
            Mesh2d(top),
            MeshMaterial2d(top_material),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.3)),
        ))
        .id();
    let left_entity = commands
        .spawn((
            Mesh2d(left),
            MeshMaterial2d(left_material),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.2)),
        ))
        .id();
    let right_entity = commands
        .spawn((
            Mesh2d(right),
            MeshMaterial2d(right_material),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
        ))
        .id();

    commands
        .entity(truck)
        .add_children(&[top_entity, left_entity, right_entity]);
}

fn face_mesh(vertices: &[[f32; 3]; 4]) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.to_vec());
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}

fn render_ui(commands: &mut Commands) {
    commands.spawn((
        Text::new("Click a tile to send the red block there\nMouse debug:"),
        DebugText,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(24.0),
            left: Val::Px(24.0),
            ..default()
        },
    ));
}

fn update_debug_cursor(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    debug_text: &mut Query<&mut Text, With<DebugText>>,
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

    let origin = board_origin();
    let top_down = iso_to_top_down(iso_world - origin);
    let grid = IVec2::new(top_down.x.round() as i32, top_down.y.round() as i32);
    for mut text in debug_text.iter_mut() {
        *text = Text::new(format!(
            "Click a tile to send the red block there\nMouse debug:\nscreen: ({:.0}, {:.0})\niso world: ({:.1}, {:.1})\ntop-down: ({:.2}, {:.2})\ntile: ({}, {})",
            screen.x, screen.y, iso_world.x, iso_world.y, top_down.x, top_down.y, grid.x, grid.y
        ));
    }
}

fn update_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    trucks: &mut Query<(&mut Transform, &mut Truck)>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok(world_cursor) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };

    let origin = board_origin();
    let grid = world_to_grid(world_cursor, origin);
    if in_board(grid) {
        let destination = grid_to_world(grid, origin).extend(2.0);
        for (_, mut truck) in trucks.iter_mut() {
            truck.destination = Some(destination);
        }
    }
}

fn update_truck(time: Res<Time>, trucks: &mut Query<(&mut Transform, &mut Truck)>) {
    for (mut transform, mut truck) in trucks.iter_mut() {
        let Some(destination) = truck.destination else {
            continue;
        };
        let current = transform.translation;
        let distance = current.distance(destination);
        let step = PLAYER_SPEED * time.delta_secs();

        if distance <= step {
            transform.translation = destination;
            truck.destination = None;
        } else {
            transform.translation = current.move_towards(destination, step);
        }
    }
}

fn board_origin() -> Vec2 {
    Vec2::new(
        0.0,
        -((BOARD_WIDTH + BOARD_HEIGHT) as f32 * TILE_HEIGHT * 0.25),
    )
}

fn grid_to_world(grid: IVec2, origin: Vec2) -> Vec2 {
    top_down_to_iso(grid.as_vec2()) + origin
}

fn world_to_grid(world: Vec2, origin: Vec2) -> IVec2 {
    let point = iso_to_top_down(world - origin);
    IVec2::new(point.x.round() as i32, point.y.round() as i32)
}

fn top_down_to_iso(top_down: Vec2) -> Vec2 {
    Vec2::new(
        (top_down.x - top_down.y) * TILE_WIDTH * 0.5,
        (top_down.x + top_down.y) * TILE_HEIGHT * 0.5,
    )
}

fn iso_to_top_down(iso: Vec2) -> Vec2 {
    let x = iso.x / (TILE_WIDTH * 0.5);
    let y = iso.y / (TILE_HEIGHT * 0.5);
    Vec2::new((x + y) * 0.5, (y - x) * 0.5)
}

fn in_board(grid: IVec2) -> bool {
    grid.x >= 0 && grid.x < BOARD_WIDTH && grid.y >= 0 && grid.y < BOARD_HEIGHT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_round_trips_top_down_coordinates() {
        let top_down = Vec2::new(2.25, 6.75);
        let iso = top_down_to_iso(top_down);
        let recovered = iso_to_top_down(iso);

        assert!((top_down - recovered).length() < 0.0001);
    }

    #[test]
    fn grid_projection_rounds_to_the_nearest_tile() {
        let origin = board_origin();
        let grid = IVec2::new(3, 7);
        let projected = grid_to_world(grid, origin) + Vec2::new(3.0, -2.0);

        assert_eq!(world_to_grid(projected, origin), grid);
    }
}
