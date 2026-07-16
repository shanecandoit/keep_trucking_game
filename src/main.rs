mod debug;
mod truck;
mod ui;
mod world;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Keep Trucking - Isometric Trucking Tycoon".into(),
                resolution: (1200.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(world::RoadNetwork::default())
        .insert_resource(ui::Focus::default())
        .insert_resource(ClearColor(Color::srgb(0.12, 0.10, 0.07)))
        .add_systems(Startup, render)
        .add_systems(Update, update)
        .run();
}

/// High-level gameplay update. Domain modules own the details of each update.
#[allow(clippy::too_many_arguments)]
fn update(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    truck_positions: Query<(Entity, &GlobalTransform), With<truck::Truck>>,
    mut trucks: Query<(&mut Transform, &mut truck::Truck)>,
    mut debug_text: Query<&mut Text, With<debug::DebugText>>,
    mut focus: ResMut<ui::Focus>,
    mut focus_visuals: Query<
        (&mut Transform, &mut Visibility, &ui::FocusVisual),
        Without<truck::Truck>,
    >,
) {
    ui::update(
        &buttons,
        windows,
        cameras,
        truck_positions,
        &mut focus,
        &mut focus_visuals,
    );
    truck::update_clicks(buttons, windows, cameras, &focus, &mut trucks);
    debug::update_cursor(windows, cameras, &mut debug_text);
    truck::update(time, &mut trucks);
}

/// High-level scene construction. Bevy handles frame rendering after this.
fn render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    road_network: Res<world::RoadNetwork>,
) {
    commands.spawn(Camera2d);
    world::render(
        &mut commands,
        &mut meshes,
        &mut materials,
        road_network.tier,
    );
    truck::render(&mut commands, &mut meshes, &mut materials);
    debug::render(&mut commands);
    ui::render(&mut commands, &mut meshes, &mut materials);
}
