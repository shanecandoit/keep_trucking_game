use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use std::collections::{HashMap, HashSet, VecDeque};

pub const TILE_WIDTH: f32 = 54.0;
pub const TILE_HEIGHT: f32 = 27.0;

const SMALL_TOWN_ASCII: &str = include_str!("../assets/maps/small_town.txt");

#[derive(Resource)]
pub struct TownMap {
    width: i32,
    height: i32,
    roads: HashSet<IVec2>,
}

impl TownMap {
    pub fn load_default() -> Self {
        Self::parse(SMALL_TOWN_ASCII).expect("assets/maps/small_town.txt must be a valid map")
    }

    fn parse(source: &str) -> Result<Self, String> {
        let rows: Vec<&str> = source.lines().filter(|line| !line.is_empty()).collect();
        let Some(first) = rows.first() else {
            return Err("map is empty".into());
        };
        let width = first.chars().count();
        if width == 0 {
            return Err("map has an empty first row".into());
        }

        let mut roads = HashSet::new();
        for (y, row) in rows.iter().enumerate() {
            if row.chars().count() != width {
                return Err(format!("map row {} has a different width", y + 1));
            }
            for (x, tile) in row.chars().enumerate() {
                match tile {
                    '.' => {}
                    '#' => {
                        roads.insert(IVec2::new(x as i32, y as i32));
                    }
                    other => {
                        return Err(format!(
                            "unsupported map tile '{other}' at ({x}, {y}); use '.' or '#'"
                        ));
                    }
                }
            }
        }

        Ok(Self {
            width: width as i32,
            height: rows.len() as i32,
            roads,
        })
    }

    pub fn center(&self) -> IVec2 {
        IVec2::new(self.width / 2, self.height / 2)
    }
}

#[allow(dead_code)] // Concrete is reserved for the progression/unlock system.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub enum RoadTier {
    #[default]
    Gravel,
    Concrete,
}

#[derive(Resource, Default)]
pub struct RoadNetwork {
    pub tier: RoadTier,
}

#[derive(Component, Clone, Copy)]
pub struct Building;

#[derive(Clone, Copy)]
pub struct BuildingSpec {
    pub grid: IVec2,
    pub entrance: IVec2,
}

const BUILDINGS: [BuildingSpec; 4] = [
    BuildingSpec {
        grid: IVec2::new(3, 3),
        entrance: IVec2::new(4, 3),
    },
    BuildingSpec {
        grid: IVec2::new(9, 1),
        entrance: IVec2::new(10, 1),
    },
    BuildingSpec {
        grid: IVec2::new(14, 5),
        entrance: IVec2::new(14, 4),
    },
    BuildingSpec {
        grid: IVec2::new(17, 9),
        entrance: IVec2::new(17, 10),
    },
];

pub fn draw_bg(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    map: &TownMap,
    road_tier: RoadTier,
) {
    let diamond = meshes.add(isometric_tile_mesh());
    let sand = materials.add(ColorMaterial::from(Color::srgb(0.69, 0.60, 0.43)));
    let scrub = materials.add(ColorMaterial::from(Color::srgb(0.61, 0.57, 0.40)));
    let gravel = materials.add(ColorMaterial::from(road_tier.base_color()));
    let pebble = road_tier.pebble_color();

    for y in 0..map.height {
        for x in 0..map.width {
            let grid = IVec2::new(x, y);
            let material = if is_road(map, grid) {
                gravel.clone()
            } else if (x + y) % 2 == 0 {
                sand.clone()
            } else {
                scrub.clone()
            };

            let tile = commands
                .spawn((
                    Mesh2d(diamond.clone()),
                    MeshMaterial2d(material),
                    Transform::from_translation(grid_to_world(map, grid).extend(0.0)),
                ))
                .id();

            if is_road(map, grid) && road_tier == RoadTier::Gravel {
                render_gravel_pebbles(commands, tile, grid, pebble);
            }
        }
    }
}

impl RoadTier {
    fn base_color(self) -> Color {
        match self {
            Self::Gravel => Color::srgb(0.47, 0.40, 0.29),
            Self::Concrete => Color::srgb(0.48, 0.49, 0.46),
        }
    }

    fn pebble_color(self) -> Color {
        match self {
            Self::Gravel => Color::srgb(0.30, 0.25, 0.18),
            Self::Concrete => Color::srgb(0.38, 0.39, 0.37),
        }
    }
}

impl RoadNetwork {
    #[allow(dead_code)] // Called when the tier-2 infrastructure unlock is added.
    pub fn unlock_concrete(&mut self) {
        self.tier = RoadTier::Concrete;
    }
}

fn render_gravel_pebbles(commands: &mut Commands, tile: Entity, grid: IVec2, color: Color) {
    let offsets = match (grid.x + grid.y) % 3 {
        0 => [
            Vec2::new(-17.0, 3.0),
            Vec2::new(8.0, -4.0),
            Vec2::new(21.0, 5.0),
        ],
        1 => [
            Vec2::new(-9.0, -5.0),
            Vec2::new(13.0, 4.0),
            Vec2::new(1.0, 7.0),
        ],
        _ => [
            Vec2::new(-22.0, -2.0),
            Vec2::new(-2.0, 4.0),
            Vec2::new(16.0, -5.0),
        ],
    };

    for (index, offset) in offsets.into_iter().enumerate() {
        let pebble = commands
            .spawn((
                Sprite::from_color(color, Vec2::splat(if index == 1 { 4.0 } else { 3.0 })),
                Transform::from_translation(offset.extend(0.1)),
            ))
            .id();
        commands.entity(tile).add_child(pebble);
    }
}

pub fn is_road(map: &TownMap, grid: IVec2) -> bool {
    map.roads.contains(&grid)
}

pub fn building_specs() -> &'static [BuildingSpec] {
    &BUILDINGS
}

pub fn building_at(grid: IVec2) -> Option<BuildingSpec> {
    BUILDINGS
        .iter()
        .copied()
        .find(|building| building.grid == grid)
}

pub fn road_path(map: &TownMap, start: IVec2, target: IVec2) -> Option<Vec<IVec2>> {
    if !is_road(map, start) || !is_road(map, target) {
        return None;
    }

    let mut frontier = VecDeque::from([start]);
    let mut came_from = HashMap::from([(start, None)]);
    let directions = [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y];

    while let Some(current) = frontier.pop_front() {
        if current == target {
            let mut path = Vec::new();
            let mut cursor = Some(current);
            while let Some(point) = cursor {
                path.push(point);
                cursor = came_from[&point];
            }
            path.reverse();
            return Some(path);
        }

        for direction in directions {
            let next = current + direction;
            if is_road(map, next) && !came_from.contains_key(&next) {
                frontier.push_back(next);
                came_from.insert(next, Some(current));
            }
        }
    }

    None
}

pub fn draw_buildings(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    map: &TownMap,
) {
    let top_mesh = meshes.add(face_mesh(&[
        [0.0, 34.0, 0.0],
        [42.0, 15.0, 0.0],
        [0.0, -4.0, 0.0],
        [-42.0, 15.0, 0.0],
    ]));
    let left_mesh = meshes.add(face_mesh(&[
        [-42.0, 15.0, 0.0],
        [0.0, -4.0, 0.0],
        [0.0, -35.0, 0.0],
        [-42.0, -16.0, 0.0],
    ]));
    let right_mesh = meshes.add(face_mesh(&[
        [0.0, -4.0, 0.0],
        [42.0, 15.0, 0.0],
        [42.0, -16.0, 0.0],
        [0.0, -35.0, 0.0],
    ]));
    let shop_door_mesh = meshes.add(face_mesh(&[
        [-7.0, 10.0, 0.0],
        [7.0, 10.0, 0.0],
        [7.0, -10.0, 0.0],
        [-7.0, -10.0, 0.0],
    ]));
    let person_door_mesh = meshes.add(face_mesh(&[
        [-4.0, 7.0, 0.0],
        [4.0, 7.0, 0.0],
        [4.0, -7.0, 0.0],
        [-4.0, -7.0, 0.0],
    ]));
    let top_material = materials.add(ColorMaterial::from(Color::srgb(0.48, 0.50, 0.51)));
    let left_material = materials.add(ColorMaterial::from(Color::srgb(0.34, 0.36, 0.37)));
    let right_material = materials.add(ColorMaterial::from(Color::srgb(0.27, 0.29, 0.30)));
    let shop_material = materials.add(ColorMaterial::from(Color::srgb(0.93, 0.66, 0.18)));
    let person_material = materials.add(ColorMaterial::from(Color::srgb(0.08, 0.10, 0.11)));

    for spec in building_specs() {
        let building = commands
            .spawn((
                Transform::from_translation(grid_to_world(map, spec.grid).extend(2.0)),
                Building,
            ))
            .id();
        let top = commands
            .spawn((
                Mesh2d(top_mesh.clone()),
                MeshMaterial2d(top_material.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.3)),
            ))
            .id();
        let left = commands
            .spawn((
                Mesh2d(left_mesh.clone()),
                MeshMaterial2d(left_material.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.2)),
            ))
            .id();
        let right = commands
            .spawn((
                Mesh2d(right_mesh.clone()),
                MeshMaterial2d(right_material.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
            ))
            .id();
        let shop_door = commands
            .spawn((
                Mesh2d(shop_door_mesh.clone()),
                MeshMaterial2d(shop_material.clone()),
                Transform::from_translation(Vec3::new(-16.0, -18.0, 0.4)),
            ))
            .id();
        let person_door = commands
            .spawn((
                Mesh2d(person_door_mesh.clone()),
                MeshMaterial2d(person_material.clone()),
                Transform::from_translation(Vec3::new(10.0, -21.0, 0.5)),
            ))
            .id();
        commands
            .entity(building)
            .add_children(&[top, left, right, shop_door, person_door]);
    }
}

pub fn board_origin(map: &TownMap) -> Vec2 {
    Vec2::new(0.0, -((map.width + map.height) as f32 * TILE_HEIGHT * 0.25))
}

pub fn grid_to_world(map: &TownMap, grid: IVec2) -> Vec2 {
    top_down_to_iso(grid.as_vec2()) + board_origin(map)
}

pub fn world_to_grid(map: &TownMap, world: Vec2) -> IVec2 {
    let point = iso_to_top_down(world - board_origin(map));
    IVec2::new(point.x.round() as i32, point.y.round() as i32)
}

pub fn top_down_to_iso(top_down: Vec2) -> Vec2 {
    Vec2::new(
        (top_down.x - top_down.y) * TILE_WIDTH * 0.5,
        (top_down.x + top_down.y) * TILE_HEIGHT * 0.5,
    )
}

pub fn iso_to_top_down(iso: Vec2) -> Vec2 {
    let x = iso.x / (TILE_WIDTH * 0.5);
    let y = iso.y / (TILE_HEIGHT * 0.5);
    Vec2::new((x + y) * 0.5, (y - x) * 0.5)
}

pub fn in_board(map: &TownMap, grid: IVec2) -> bool {
    grid.x >= 0 && grid.x < map.width && grid.y >= 0 && grid.y < map.height
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
    fn projection_round_trips_top_down_coordinates() {
        let top_down = Vec2::new(2.25, 6.75);
        assert!((top_down - iso_to_top_down(top_down_to_iso(top_down))).length() < 0.0001);
    }

    #[test]
    fn grid_projection_rounds_to_the_nearest_tile() {
        let map = TownMap::load_default();
        let grid = IVec2::new(3, 7);
        let projected = grid_to_world(&map, grid) + Vec2::new(3.0, -2.0);
        assert_eq!(world_to_grid(&map, projected), grid);
    }

    #[test]
    fn ascii_map_loads_the_small_town_road_network() {
        let map = TownMap::load_default();

        assert_eq!((map.width, map.height), (31, 15));
        assert!(is_road(&map, IVec2::new(0, 7)));
        assert!(is_road(&map, IVec2::new(30, 7)));
        assert!(is_road(&map, IVec2::new(16, 4)));
        assert!(!is_road(&map, IVec2::new(0, 0)));
    }

    #[test]
    fn rural_branch_routes_into_the_center_grid() {
        let map = TownMap::load_default();
        let path = road_path(&map, IVec2::new(10, 0), IVec2::new(20, 10))
            .expect("branch and center grid should be connected");

        assert_eq!(path.first(), Some(&IVec2::new(10, 0)));
        assert_eq!(path.last(), Some(&IVec2::new(20, 10)));
    }

    #[test]
    fn concrete_is_locked_until_the_road_network_unlocks_it() {
        let mut roads = RoadNetwork::default();
        assert_eq!(roads.tier, RoadTier::Gravel);

        roads.unlock_concrete();

        assert_eq!(roads.tier, RoadTier::Concrete);
    }
}
