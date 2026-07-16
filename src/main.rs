mod debug;
mod truck;
mod world;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

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

/// High-level gameplay update. Domain modules own the details of each update.
fn update(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut trucks: Query<(&mut Transform, &mut truck::Truck)>,
    mut debug_text: Query<&mut Text, With<debug::DebugText>>,
) {
    truck::update_clicks(buttons, windows, cameras, &mut trucks);
    debug::update_cursor(windows, cameras, &mut debug_text);
    truck::update(time, &mut trucks);
}

/// High-level scene construction. Bevy handles frame rendering after this.
fn render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    world::render(&mut commands, &mut meshes, &mut materials);
    truck::render(&mut commands, &mut meshes, &mut materials);
    debug::render(&mut commands);
}
