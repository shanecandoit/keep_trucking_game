use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::window::PrimaryWindow;

use crate::{truck::Truck, world};

const CYAN: Color = Color::srgba(0.15, 0.95, 0.95, 0.65);
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

pub fn render(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let material = materials.add(ColorMaterial::from(CYAN));
    let border_mesh = meshes.add(tile_outline_mesh());

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
        Sprite::from_color(CYAN, Vec2::splat(7.0)),
        Transform::from_translation(Vec3::new(0.0, 0.0, FOCUS_Z + 0.1)),
        Visibility::Hidden,
        FocusVisual {
            kind: FocusVisualKind::HoveredTile,
        },
    ));
}

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
