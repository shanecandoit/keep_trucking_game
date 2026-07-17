use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::window::PrimaryWindow;

use crate::ui::Focus;
use crate::world;

const TRUCK_WIDTH: f32 = 30.0;
const TRUCK_SCALE: f32 = 0.62;
// Roughly two road tiles per second at normal simulation speed. Keeping this
// tied to the visible road scale makes cross-town routes feel consequential.
const TRUCK_SPEED: f32 = 65.0;
const ROUTE_DOT_Z: f32 = 1.25;
pub const FUEL_GALLONS_PER_TILE: f32 = 0.08;
pub const WEAR_PER_TILE: f32 = 0.002;
pub const STARTING_FUEL_GALLONS: f32 = 24.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TruckId(pub u64);

#[derive(Resource)]
pub struct RouteDebugAssets {
    waypoint_mesh: Handle<Mesh>,
    destination_mesh: Handle<Mesh>,
    waypoint_material: Handle<ColorMaterial>,
    destination_material: Handle<ColorMaterial>,
}

#[derive(Component)]
pub struct RouteWaypointDebug {
    owner: Entity,
    route_revision: u64,
    waypoint: Vec3,
}

#[derive(SystemParam)]
pub struct RouteDebug<'w, 's> {
    commands: Commands<'w, 's>,
    assets: Res<'w, RouteDebugAssets>,
}

#[derive(Component)]
pub struct Truck {
    pub id: TruckId,
    pub route: Vec<Vec3>,
    pub fuel_gallons: f32,
    pub wear: f32,
    pub odometer_tiles: f32,
    pub active_contract: Option<u64>,
    route_revision: u64,
}

pub fn draw_trucks(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    map: &world::TownMap,
) {
    commands.insert_resource(RouteDebugAssets {
        waypoint_mesh: meshes.add(Circle::new(3.25)),
        destination_mesh: meshes.add(Circle::new(5.0)),
        waypoint_material: materials.add(ColorMaterial::from(Color::srgba(0.15, 0.95, 0.95, 0.78))),
        destination_material: materials
            .add(ColorMaterial::from(Color::srgba(0.98, 0.75, 0.16, 0.95))),
    });

    let start = map.center();
    let truck = commands
        .spawn((
            Transform::from_translation(world::grid_to_world(map, start).extend(3.0))
                .with_scale(Vec3::splat(TRUCK_SCALE)),
            Truck {
                id: TruckId(1),
                route: Vec::new(),
                fuel_gallons: STARTING_FUEL_GALLONS,
                wear: 0.0,
                odometer_tiles: 0.0,
                active_contract: None,
                route_revision: 0,
            },
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

pub fn update(sim_delta: f32, trucks: &mut Query<(Entity, &mut Transform, &mut Truck)>) {
    for (_entity, mut transform, mut truck) in trucks.iter_mut() {
        if truck.fuel_gallons <= 0.0 {
            continue;
        }
        let Some(destination) = truck.route.first().copied() else {
            continue;
        };
        let previous = transform.translation;
        let distance = transform.translation.distance(destination);
        let step = TRUCK_SPEED * sim_delta;
        if distance <= step {
            transform.translation = destination;
            truck.route.remove(0);
        } else {
            transform.translation = transform.translation.move_towards(destination, step);
        }
        let distance_tiles = previous.distance(transform.translation) / world::ROAD_STEP_WORLD;
        truck.odometer_tiles += distance_tiles;
        truck.fuel_gallons = (truck.fuel_gallons - distance_tiles * FUEL_GALLONS_PER_TILE).max(0.0);
        truck.wear += distance_tiles * WEAR_PER_TILE;
    }
}

pub fn update_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    focus: &Focus,
    trucks: &mut Query<(Entity, &mut Transform, &mut Truck)>,
    map: &world::TownMap,
    debug: &mut RouteDebug,
) {
    if !buttons.just_pressed(MouseButton::Left) || focus.click_consumed {
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
    let clicked = world::world_to_grid(map, world_cursor);
    let target = world::building_at(map, clicked)
        .map(|building| building.entrance)
        .or_else(|| world::is_road(map, clicked).then_some(clicked));
    let Some(target) = target else { return };

    for (entity, transform, mut truck) in trucks.iter_mut() {
        if focus.selected != Some(entity) {
            continue;
        }
        if truck.active_contract.is_some() {
            warn!(truck = ?truck.id, "manual route ignored while truck has an active tow contract");
            continue;
        }
        let start = world::world_to_grid(map, transform.translation.truncate());
        if let Some(waypoints) = debug.assign_route(map, entity, &transform, &mut truck, target) {
            info!(?start, ?target, waypoints, "truck route assigned");
        } else {
            warn!(?start, ?target, "no road route found for truck");
        }
    }
}

impl RouteDebug<'_, '_> {
    pub fn assign_route(
        &mut self,
        map: &world::TownMap,
        entity: Entity,
        transform: &Transform,
        truck: &mut Truck,
        target: IVec2,
    ) -> Option<usize> {
        let start = world::world_to_grid(map, transform.translation.truncate());
        let path = world::road_path(map, start, target)?;
        truck.route = path
            .into_iter()
            .skip(1)
            .map(|grid| world::grid_to_world(map, grid).extend(3.0))
            .collect();
        truck.route_revision = truck.route_revision.wrapping_add(1);
        let assets = &self.assets;
        spawn_route_debug(
            &mut self.commands,
            entity,
            truck.route_revision,
            &truck.route,
            assets,
        );
        Some(truck.route.len())
    }
}

fn spawn_route_debug(
    commands: &mut Commands,
    owner: Entity,
    route_revision: u64,
    route: &[Vec3],
    assets: &RouteDebugAssets,
) {
    let last_index = route.len().saturating_sub(1);
    for (index, waypoint) in route.iter().copied().enumerate() {
        let is_destination = index == last_index;
        commands.spawn((
            Mesh2d(if is_destination {
                assets.destination_mesh.clone()
            } else {
                assets.waypoint_mesh.clone()
            }),
            MeshMaterial2d(if is_destination {
                assets.destination_material.clone()
            } else {
                assets.waypoint_material.clone()
            }),
            Transform::from_translation(waypoint.truncate().extend(ROUTE_DOT_Z)),
            RouteWaypointDebug {
                owner,
                route_revision,
                waypoint,
            },
        ));
    }
}

pub fn sync_route_debug(
    mut commands: Commands,
    waypoints: Query<(Entity, &RouteWaypointDebug)>,
    trucks: Query<(Entity, &Truck)>,
) {
    for (entity, marker) in waypoints.iter() {
        let active = trucks
            .iter()
            .find(|(truck, _)| *truck == marker.owner)
            .is_some_and(|(_, truck)| {
                marker.route_revision == truck.route_revision
                    && route_contains_waypoint(&truck.route, marker.waypoint)
            });
        if !active {
            commands.entity(entity).despawn();
        }
    }
}

fn route_contains_waypoint(route: &[Vec3], waypoint: Vec3) -> bool {
    route
        .iter()
        .any(|candidate| candidate.distance_squared(waypoint) < 0.001)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_debug_only_keeps_unconsumed_waypoints() {
        let first = Vec3::new(1.0, 2.0, 3.0);
        let second = Vec3::new(4.0, 5.0, 3.0);
        let mut route = vec![first, second];

        assert!(route_contains_waypoint(&route, first));
        assert!(route_contains_waypoint(&route, second));

        route.remove(0);

        assert!(!route_contains_waypoint(&route, first));
        assert!(route_contains_waypoint(&route, second));
    }
}
