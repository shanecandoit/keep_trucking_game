use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::window::PrimaryWindow;

use crate::world;

const TRUCK_WIDTH: f32 = 30.0;
const TRUCK_SPEED: f32 = 260.0;

#[derive(Component)]
pub struct Truck {
    pub route: Vec<Vec3>,
}

pub fn render(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let start = IVec2::new(world::BOARD_WIDTH / 2, world::BOARD_HEIGHT / 2);
    let truck = commands
        .spawn((
            Transform::from_translation(world::grid_to_world(start).extend(2.0)),
            Truck { route: Vec::new() },
        ))
        .id();

    let top = meshes.add(face_mesh(&[
        [0.0, 18.0, 0.0],
        [TRUCK_WIDTH, 5.0, 0.0],
        [0.0, -8.0, 0.0],
        [-TRUCK_WIDTH, 5.0, 0.0],
    ]));
    let left = meshes.add(face_mesh(&[
        [-TRUCK_WIDTH, 5.0, 0.0],
        [0.0, -8.0, 0.0],
        [0.0, -25.0, 0.0],
        [-TRUCK_WIDTH, -12.0, 0.0],
    ]));
    let right = meshes.add(face_mesh(&[
        [0.0, -8.0, 0.0],
        [TRUCK_WIDTH, 5.0, 0.0],
        [TRUCK_WIDTH, -12.0, 0.0],
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

pub fn update(time: Res<Time>, trucks: &mut Query<(&mut Transform, &mut Truck)>) {
    for (mut transform, mut truck) in trucks.iter_mut() {
        let Some(destination) = truck.route.first().copied() else {
            continue;
        };
        let distance = transform.translation.distance(destination);
        let step = TRUCK_SPEED * time.delta_secs();
        if distance <= step {
            transform.translation = destination;
            truck.route.remove(0);
        } else {
            transform.translation = transform.translation.move_towards(destination, step);
        }
    }
}

pub fn update_clicks(
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
    let clicked = world::world_to_grid(world_cursor);
    let target = world::building_at(clicked)
        .map(|building| building.entrance)
        .or_else(|| world::is_road(clicked).then_some(clicked));
    let Some(target) = target else { return };

    for (transform, mut truck) in trucks.iter_mut() {
        let start = world::world_to_grid(transform.translation.truncate());
        if let Some(path) = world::road_path(start, target) {
            truck.route = path
                .into_iter()
                .skip(1)
                .map(|grid| world::grid_to_world(grid).extend(2.0))
                .collect();
        }
    }
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
