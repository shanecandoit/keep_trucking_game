mod camera;
mod debug;
mod truck;
mod ui;
mod world;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Fixed simulation clock. Wall-clock frames may render faster or slower than
/// gameplay expects, so simulation time advances on this clock instead of
/// `Time::delta_secs`. Speed is controlled by the debug panel (press `Space`).
#[derive(Resource, Default)]
pub struct SimClock {
    elapsed: f32,
    speed: SimSpeed,
    last_delta: f32,
}

impl SimClock {
    pub fn elapsed_secs(&self) -> f32 {
        self.elapsed
    }

    pub fn delta_secs(&self) -> f32 {
        self.last_delta
    }

    pub fn speed_label(&self) -> &'static str {
        match self.speed {
            SimSpeed::Paused => "paused",
            SimSpeed::Normal => "1x",
            SimSpeed::Fast => "3x",
        }
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.speed, SimSpeed::Paused)
    }

    pub fn tick(&mut self, delta: f32) {
        self.last_delta = delta * self.speed.multiplier();
        self.elapsed += self.last_delta;
    }

    pub fn cycle_speed(&mut self) {
        self.speed = match self.speed {
            SimSpeed::Paused => SimSpeed::Normal,
            SimSpeed::Normal => SimSpeed::Fast,
            SimSpeed::Fast => SimSpeed::Paused,
        };
    }
}

#[derive(Default, Clone, Copy)]
enum SimSpeed {
    #[default]
    Paused,
    Normal,
    Fast,
}

impl SimSpeed {
    fn multiplier(self) -> f32 {
        match self {
            Self::Paused => 0.0,
            Self::Normal => 1.0,
            Self::Fast => 3.0,
        }
    }
}

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
        .insert_resource(world::TownMap::load_default())
        .insert_resource(camera::PanState::default())
        .insert_resource(ui::Focus::default())
        .insert_resource(SimClock::default())
        .insert_resource(ClearColor(Color::srgb(0.12, 0.10, 0.07)))
        .add_systems(Startup, render)
        .add_systems(Update, update)
        .add_systems(Update, camera::zoom)
        .add_systems(Update, debug::update_pause)
        .run();
}

/// High-level gameplay update. Domain modules own the details of each update.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn update(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut camera_transforms: Query<&mut Transform, (With<Camera>, Without<truck::Truck>)>,
    truck_positions: Query<(Entity, &GlobalTransform), With<truck::Truck>>,
    mut trucks: Query<(Entity, &mut Transform, &mut truck::Truck)>,
    debug_stats: Query<&mut Text, With<debug::DebugStats>>,
    debug_cursor: Query<&mut Text, (With<debug::DebugCursor>, Without<debug::DebugStats>)>,
    mut focus: ResMut<ui::Focus>,
    mut pan_state: ResMut<camera::PanState>,
    mut sim_clock: ResMut<SimClock>,
    map: Res<world::TownMap>,
    mut focus_visuals: Query<
        (&mut Transform, &mut Visibility, &ui::FocusVisual),
        (Without<truck::Truck>, Without<Camera>),
    >,
) {
    if keys.just_pressed(KeyCode::Space) {
        sim_clock.cycle_speed();
    }
    sim_clock.tick(time.delta_secs());

    camera::update(&buttons, windows, &mut camera_transforms, &mut pan_state);
    ui::update(
        &buttons,
        windows,
        cameras,
        truck_positions,
        &mut focus,
        &mut focus_visuals,
        &map,
    );
    truck::update_clicks(buttons, windows, cameras, &focus, &mut trucks, &map);
    debug::update(
        &time,
        &sim_clock,
        windows,
        cameras,
        &trucks,
        debug_stats,
        debug_cursor,
        &focus,
        &map,
    );
    truck::update(sim_clock.delta_secs(), &mut trucks);
}

/// High-level scene construction. Bevy handles frame rendering after this.
fn render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    road_network: Res<world::RoadNetwork>,
    map: Res<world::TownMap>,
) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: camera::INITIAL_SCALE,
            ..OrthographicProjection::default_2d()
        }),
    ));
    draw_bg(
        &mut commands,
        &mut meshes,
        &mut materials,
        &map,
        road_network.tier,
    );
    draw_bg_ui(&mut commands, &mut meshes, &mut materials);
    draw_fg(&mut commands, &mut meshes, &mut materials, &map);
    draw_fg_ui(&mut commands);
}

fn draw_bg(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    map: &world::TownMap,
    road_tier: world::RoadTier,
) {
    world::draw_bg(commands, meshes, materials, map, road_tier);
}

fn draw_bg_ui(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    // Focus box and mouse pointer live between the terrain and actors.
    ui::draw_bg_ui(commands, meshes, materials);
    debug::render(commands);
}

fn draw_fg(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    map: &world::TownMap,
) {
    truck::draw_trucks(commands, meshes, materials, map);
    world::draw_buildings(commands, meshes, materials, map);
}

fn draw_fg_ui(commands: &mut Commands) {
    // Reserved for actor/status UI that should render above trucks/buildings.
    ui::draw_fg_ui(commands);
}
