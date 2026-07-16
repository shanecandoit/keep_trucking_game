use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::window::PrimaryWindow;

use crate::{truck::Truck, world};

const CYAN: Color = Color::srgba(0.15, 0.95, 0.95, 0.65);
const LIGHT_BEAM: Color = Color::srgba(0.15, 0.95, 0.95, 0.18);
const LIGHT_BASE: Color = Color::srgba(0.15, 0.95, 0.95, 0.38);
const LIGHT_HEIGHT: f32 = 110.0;
const LIGHT_RADIUS: f32 = 9.0;
const FOCUS_Z: f32 = 1.0;

#[derive(Resource, Default)]
pub struct Focus {
    pub selected: Option<Entity>,
    pub hovered: Option<IVec2>,
    pub click_consumed: bool,
}

#[derive(Component)]
pub struct FocusVisual {
    pub kind: FocusVisualKind,
}

#[derive(Clone, Copy)]
pub enum FocusVisualKind {
    SelectedTile,
    HoveredTile,
}

pub fn draw_bg_ui(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let material = materials.add(ColorMaterial::from(CYAN));
    let border_mesh = meshes.add(tile_outline_mesh());
    let beam_mesh = meshes.add(light_beam_mesh());
    let base_mesh = meshes.add(light_base_mesh());
    let beam_material = materials.add(ColorMaterial::from(LIGHT_BEAM));
    let base_material = materials.add(ColorMaterial::from(LIGHT_BASE));

    commands.spawn((
        Mesh2d(border_mesh),
        MeshMaterial2d(material.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, FOCUS_Z)),
        Visibility::Hidden,
        FocusVisual {
            kind: FocusVisualKind::SelectedTile,
        },
    ));
    commands.spawn((
        Mesh2d(beam_mesh),
        MeshMaterial2d(beam_material),
        Transform::from_translation(Vec3::new(0.0, 0.0, FOCUS_Z + 0.1)),
        Visibility::Hidden,
        FocusVisual {
            kind: FocusVisualKind::HoveredTile,
        },
    ));
    commands.spawn((
        Mesh2d(base_mesh),
        MeshMaterial2d(base_material),
        Transform::from_translation(Vec3::new(0.0, 0.0, FOCUS_Z + 0.2)),
        Visibility::Hidden,
        FocusVisual {
            kind: FocusVisualKind::HoveredTile,
        },
    ));
}

pub fn draw_fg_ui(_commands: &mut Commands) {}

pub fn update(
    buttons: &ButtonInput<MouseButton>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    trucks: Query<(Entity, &GlobalTransform), With<Truck>>,
    focus: &mut Focus,
    visuals: &mut Query<(&mut Transform, &mut Visibility, &FocusVisual), Without<Truck>>,
) {
    focus.click_consumed = false;

    let Ok(window) = windows.single() else {
        update_visuals(focus, &trucks, visuals);
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        focus.hovered = None;
        update_visuals(focus, &trucks, visuals);
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        update_visuals(focus, &trucks, visuals);
        return;
    };
    let Ok(world_cursor) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        update_visuals(focus, &trucks, visuals);
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
        info!(?entity, "selected truck");
    }

    update_visuals(focus, &trucks, visuals);
}

fn update_visuals(
    focus: &Focus,
    trucks: &Query<(Entity, &GlobalTransform), With<Truck>>,
    visuals: &mut Query<(&mut Transform, &mut Visibility, &FocusVisual), Without<Truck>>,
) {
    let selected_grid = focus.selected.and_then(|selected| {
        trucks
            .iter()
            .find(|(entity, _)| *entity == selected)
            .map(|(_, transform)| world::world_to_grid(transform.translation().truncate()))
    });

    for (mut transform, mut visibility, visual) in visuals.iter_mut() {
        let target = match visual.kind {
            FocusVisualKind::SelectedTile => selected_grid,
            FocusVisualKind::HoveredTile => focus.hovered,
        };
        let visible = target.is_some();
        let grid = target.unwrap_or(IVec2::ZERO);
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if visible {
            transform.translation = world::grid_to_world(grid).extend(transform.translation.z);
        }
    }
}

fn tile_outline_mesh() -> Mesh {
    let half_width = world::TILE_WIDTH * 0.5;
    let half_height = world::TILE_HEIGHT * 0.5;
    let mut mesh = Mesh::new(
        PrimitiveTopology::LineStrip,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, half_height, 0.0],
            [half_width, 0.0, 0.0],
            [0.0, -half_height, 0.0],
            [-half_width, 0.0, 0.0],
            [0.0, half_height, 0.0],
        ],
    );
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 3, 4]));
    mesh
}

fn light_beam_mesh() -> Mesh {
    // A constant-width vertical rectangle is the 2D projection of the light
    // cylinder. It extends above the building silhouette while remaining in
    // the background-UI z layer, so buildings can occlude its lower section.
    quad_mesh([
        [-LIGHT_RADIUS, 0.0, 0.0],
        [LIGHT_RADIUS, 0.0, 0.0],
        [LIGHT_RADIUS, LIGHT_HEIGHT, 0.0],
        [-LIGHT_RADIUS, LIGHT_HEIGHT, 0.0],
    ])
}

fn light_base_mesh() -> Mesh {
    let half_width = world::TILE_WIDTH * 0.22;
    let half_height = world::TILE_HEIGHT * 0.22;
    quad_mesh([
        [0.0, half_height, 0.0],
        [half_width, 0.0, 0.0],
        [0.0, -half_height, 0.0],
        [-half_width, 0.0, 0.0],
    ])
}

fn quad_mesh(vertices: [[f32; 3]; 4]) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.to_vec());
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}
