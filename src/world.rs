use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

pub const BOARD_WIDTH: i32 = 9;
pub const BOARD_HEIGHT: i32 = 9;
pub const TILE_WIDTH: f32 = 86.0;
pub const TILE_HEIGHT: f32 = 43.0;

pub fn render(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let diamond = meshes.add(isometric_tile_mesh());
    let dark = materials.add(ColorMaterial::from(Color::srgb(0.17, 0.28, 0.32)));
    let light = materials.add(ColorMaterial::from(Color::srgb(0.20, 0.33, 0.36)));

    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            let grid = IVec2::new(x, y);
            let material = if (x + y) % 2 == 0 {
                dark.clone()
            } else {
                light.clone()
            };

            commands.spawn((
                Mesh2d(diamond.clone()),
                MeshMaterial2d(material),
                Transform::from_translation(grid_to_world(grid).extend(0.0)),
            ));
        }
    }
}

pub fn board_origin() -> Vec2 {
    Vec2::new(
        0.0,
        -((BOARD_WIDTH + BOARD_HEIGHT) as f32 * TILE_HEIGHT * 0.25),
    )
}

pub fn grid_to_world(grid: IVec2) -> Vec2 {
    top_down_to_iso(grid.as_vec2()) + board_origin()
}

pub fn world_to_grid(world: Vec2) -> IVec2 {
    let point = iso_to_top_down(world - board_origin());
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

pub fn in_board(grid: IVec2) -> bool {
    grid.x >= 0 && grid.x < BOARD_WIDTH && grid.y >= 0 && grid.y < BOARD_HEIGHT
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
        let grid = IVec2::new(3, 7);
        let projected = grid_to_world(grid) + Vec2::new(3.0, -2.0);
        assert_eq!(world_to_grid(projected), grid);
    }
}
