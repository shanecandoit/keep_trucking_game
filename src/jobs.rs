use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::SimClock;
use crate::company::{Company, format_money};
use crate::session::GameSession;
use crate::truck::{FUEL_GALLONS_PER_TILE, RouteDebug, Truck, TruckId, WEAR_PER_TILE};
use crate::ui::{Focus, ScreenPanel};
use crate::world;

const OFFER_LIFETIME_SECS: f32 = 180.0;
const HOOKUP_DURATION_SECS: f32 = 5.0;
const TRAVEL_TILES_PER_SEC: f32 = 2.0;
const FUEL_PRICE_CENTS_PER_GALLON: f32 = 350.0;
const WEAR_RESERVE_CENTS_PER_TILE: f32 = 22.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VehicleClass {
    PassengerCar,
}

impl VehicleClass {
    fn label(self) -> &'static str {
        match self {
            Self::PassengerCar => "passenger car",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Urgency {
    Standard,
}

impl Urgency {
    fn label(self) -> &'static str {
        match self {
            Self::Standard => "standard",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureReason {
    OutOfFuel,
    TruckUnavailable,
    NoRoute,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TowState {
    Offered,
    Accepted,
    EnRoutePickup,
    HookingUp,
    EnRouteDropoff,
    Completed,
    Declined,
    Expired,
    Failed(FailureReason),
}

impl TowState {
    fn label(self) -> &'static str {
        match self {
            Self::Offered => "OFFERED",
            Self::Accepted => "ACCEPTED",
            Self::EnRoutePickup => "EN ROUTE TO CUSTOMER",
            Self::HookingUp => "HOOKING UP",
            Self::EnRouteDropoff => "TOWING",
            Self::Completed => "COMPLETED",
            Self::Declined => "DECLINED",
            Self::Expired => "EXPIRED",
            Self::Failed(FailureReason::OutOfFuel) => "FAILED: OUT OF FUEL",
            Self::Failed(FailureReason::TruckUnavailable) => "FAILED: TRUCK UNAVAILABLE",
            Self::Failed(FailureReason::NoRoute) => "FAILED: NO ROUTE",
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct JobEstimate {
    pub approach_tiles: usize,
    pub tow_tiles: usize,
    pub duration_secs: f32,
    pub fuel_gallons: f32,
    pub fuel_cost_cents: i64,
    pub wear_reserve_cents: i64,
    pub expected_margin_cents: i64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct JobActuals {
    pub approach_tiles: f32,
    pub tow_tiles: f32,
    pub duration_secs: f32,
    pub fuel_gallons: f32,
    pub wear: f32,
    pub fuel_cost_cents: i64,
    pub wear_reserve_cents: i64,
}

impl JobActuals {
    pub fn operating_cost_cents(self) -> i64 {
        self.fuel_cost_cents + self.wear_reserve_cents
    }
}

#[derive(Clone, Copy, Debug)]
pub struct JobAssignment {
    pub truck_id: TruckId,
    pub dispatched_at: f32,
    odometer_start: f32,
    fuel_start: f32,
    wear_start: f32,
    tow_odometer_start: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct TowContract {
    pub id: u64,
    pub pickup: IVec2,
    pub dropoff: IVec2,
    pub vehicle: VehicleClass,
    pub urgency: Urgency,
    pub payout_cents: i64,
    pub offered_at: f32,
    pub expires_at: f32,
    pub state: TowState,
    pub estimate: JobEstimate,
    pub actuals: JobActuals,
    pub assignment: Option<JobAssignment>,
    hookup_completes_at: Option<f32>,
}

impl TowContract {
    fn accept(&mut self) -> bool {
        if self.state != TowState::Offered {
            return false;
        }
        self.state = TowState::Accepted;
        true
    }

    fn decline(&mut self) -> bool {
        if self.state != TowState::Offered {
            return false;
        }
        self.state = TowState::Declined;
        true
    }

    fn expire(&mut self, now: f32) -> bool {
        if self.state != TowState::Offered || now < self.expires_at {
            return false;
        }
        self.state = TowState::Expired;
        true
    }

    fn dispatch(&mut self, assignment: JobAssignment, estimate: JobEstimate) -> bool {
        if self.state != TowState::Accepted {
            return false;
        }
        self.assignment = Some(assignment);
        self.estimate = estimate;
        self.state = TowState::EnRoutePickup;
        true
    }

    fn arrive_at_pickup(&mut self, now: f32, odometer_tiles: f32) -> bool {
        if self.state != TowState::EnRoutePickup {
            return false;
        }
        let Some(assignment) = self.assignment else {
            return false;
        };
        self.actuals.approach_tiles = odometer_tiles - assignment.odometer_start;
        self.hookup_completes_at = Some(now + HOOKUP_DURATION_SECS);
        self.state = TowState::HookingUp;
        true
    }

    fn begin_tow(&mut self, odometer_tiles: f32) -> bool {
        if self.state != TowState::HookingUp {
            return false;
        }
        let Some(assignment) = self.assignment.as_mut() else {
            return false;
        };
        assignment.tow_odometer_start = Some(odometer_tiles);
        self.state = TowState::EnRouteDropoff;
        true
    }

    fn complete(
        &mut self,
        now: f32,
        odometer_tiles: f32,
        fuel_gallons: f32,
        wear: f32,
    ) -> Option<i64> {
        if self.state != TowState::EnRouteDropoff {
            return None;
        }
        let assignment = self.assignment?;
        self.actuals.tow_tiles =
            odometer_tiles - assignment.tow_odometer_start.unwrap_or(odometer_tiles);
        self.actuals.duration_secs = now - assignment.dispatched_at;
        self.actuals.fuel_gallons = assignment.fuel_start - fuel_gallons;
        self.actuals.wear = wear - assignment.wear_start;
        self.actuals.fuel_cost_cents =
            (self.actuals.fuel_gallons * FUEL_PRICE_CENTS_PER_GALLON).round() as i64;
        self.actuals.wear_reserve_cents =
            (self.actuals.wear / WEAR_PER_TILE * WEAR_RESERVE_CENTS_PER_TILE).round() as i64;
        self.state = TowState::Completed;
        Some(self.payout_cents - self.actuals.operating_cost_cents())
    }
}

#[derive(Resource)]
pub struct Contracts {
    pub current: TowContract,
}

#[derive(Component)]
pub struct JobTablet;

#[derive(Component)]
pub struct JobCardTitle;

#[derive(Component)]
pub struct JobCardBody;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobAction {
    Accept,
    Reject,
    Dispatch,
}

#[derive(SystemParam)]
pub(crate) struct JobInputUi<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    focus: Res<'w, Focus>,
    presentation: ResMut<'w, JobPresentation>,
    actions: Query<'w, 's, (&'static Interaction, &'static JobAction), With<Button>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CardView {
    Offer,
    Active,
    Hidden,
}

#[derive(Clone, Copy, Debug)]
struct CardTransition {
    target: CardView,
    elapsed: f32,
}

#[derive(Resource, Debug)]
pub struct JobPresentation {
    view: CardView,
    transition: Option<CardTransition>,
}

impl Default for JobPresentation {
    fn default() -> Self {
        Self {
            view: CardView::Offer,
            transition: None,
        }
    }
}

impl JobPresentation {
    fn swipe_to(&mut self, target: CardView) {
        self.transition = Some(CardTransition {
            target,
            elapsed: 0.0,
        });
    }
}

fn advance_card_transition(presentation: &mut JobPresentation, delta_secs: f32) -> f32 {
    let Some(mut transition) = presentation.transition else {
        return 0.0;
    };
    transition.elapsed += delta_secs;
    if transition.target == CardView::Hidden {
        let progress = (transition.elapsed / 0.24).min(1.0);
        if progress >= 1.0 {
            presentation.view = CardView::Hidden;
            presentation.transition = None;
        } else {
            presentation.transition = Some(transition);
        }
        return progress * 480.0;
    }

    let half = 0.22;
    if transition.elapsed < half {
        presentation.transition = Some(transition);
        return transition.elapsed / half * 480.0;
    }
    presentation.view = transition.target;
    let progress = ((transition.elapsed - half) / half).min(1.0);
    if progress >= 1.0 {
        presentation.transition = None;
    } else {
        presentation.transition = Some(transition);
    }
    (1.0 - progress) * 480.0
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
enum JobPoiKind {
    Pickup,
    Dropoff,
}

struct PoiColors {
    halo: Color,
    ring: Color,
}

#[derive(Component)]
pub struct JobPoi {
    kind: JobPoiKind,
}

pub fn setup(
    mut commands: Commands,
    map: Res<world::TownMap>,
    mut session: ResMut<GameSession>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let buildings = world::building_specs(&map);
    assert!(
        buildings.len() >= 2,
        "tow jobs require at least two buildings"
    );

    let pickup_index = session.next_index(buildings.len());
    let mut dropoff_index = session.next_index(buildings.len() - 1);
    if dropoff_index >= pickup_index {
        dropoff_index += 1;
    }
    let pickup = buildings[pickup_index].entrance;
    let dropoff = buildings[dropoff_index].entrance;
    let tow_tiles =
        route_tiles(&map, pickup, dropoff).expect("generated tow locations must connect");
    let payout_cents = quote_payout(tow_tiles, Urgency::Standard);
    let estimate = estimate_job(&map, map.center(), pickup, dropoff, payout_cents)
        .expect("starting truck must reach generated tow locations");
    let contract = TowContract {
        id: session.next_contract_id(),
        pickup,
        dropoff,
        vehicle: VehicleClass::PassengerCar,
        urgency: Urgency::Standard,
        payout_cents,
        offered_at: 0.0,
        expires_at: OFFER_LIFETIME_SECS,
        state: TowState::Offered,
        estimate,
        actuals: JobActuals::default(),
        assignment: None,
        hookup_completes_at: None,
    };
    info!(
        contract = contract.id,
        ?pickup,
        ?dropoff,
        payout_cents,
        "tow call generated"
    );
    commands.insert_resource(Contracts { current: contract });
    commands.insert_resource(JobPresentation::default());
    spawn_job_tablet(&mut commands);
    spawn_job_poi(
        &mut commands,
        &mut meshes,
        &mut materials,
        &map,
        pickup,
        JobPoiKind::Pickup,
        PoiColors {
            halo: Color::srgba(0.15, 0.95, 0.95, 0.18),
            ring: Color::srgba(0.15, 0.95, 0.95, 0.88),
        },
    );
    spawn_job_poi(
        &mut commands,
        &mut meshes,
        &mut materials,
        &map,
        dropoff,
        JobPoiKind::Dropoff,
        PoiColors {
            halo: Color::srgba(1.0, 0.68, 0.14, 0.18),
            ring: Color::srgba(1.0, 0.68, 0.14, 0.92),
        },
    );
}

fn spawn_job_tablet(commands: &mut Commands) {
    commands
        .spawn((
            JobTablet,
            ScreenPanel,
            Interaction::default(),
            BackgroundColor(Color::srgb(0.12, 0.13, 0.13)),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(24.0),
                bottom: Val::Px(24.0),
                width: Val::Px(430.0),
                height: Val::Px(520.0),
                padding: UiRect::all(Val::Px(13.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .with_children(|tablet| {
            tablet.spawn((
                Text::new("KEEP TRUCKING  /  FIELD TABLET"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.68, 0.72, 0.70)),
                Node {
                    height: Val::Px(18.0),
                    ..default()
                },
            ));
            tablet
                .spawn((
                    BackgroundColor(Color::srgb(0.88, 0.84, 0.70)),
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        padding: UiRect::all(Val::Px(18.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(9.0),
                        ..default()
                    },
                ))
                .with_children(|paper| {
                    paper.spawn((
                        Text::new("TOW REQUEST"),
                        JobCardTitle,
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.16, 0.17, 0.15)),
                    ));
                    paper.spawn((
                        Text::new(""),
                        JobCardBody,
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.20, 0.21, 0.18)),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                    ));
                    paper
                        .spawn((Node {
                            height: Val::Px(42.0),
                            column_gap: Val::Px(10.0),
                            justify_content: JustifyContent::FlexEnd,
                            ..default()
                        },))
                        .with_children(|actions| {
                            spawn_job_button(
                                actions,
                                JobAction::Reject,
                                "REJECT",
                                Color::srgb(0.50, 0.18, 0.14),
                            );
                            spawn_job_button(
                                actions,
                                JobAction::Accept,
                                "ACCEPT",
                                Color::srgb(0.18, 0.43, 0.27),
                            );
                            spawn_job_button(
                                actions,
                                JobAction::Dispatch,
                                "DISPATCH",
                                Color::srgb(0.18, 0.38, 0.48),
                            );
                        });
                });
        });
}

fn spawn_job_button(
    actions: &mut ChildSpawnerCommands,
    action: JobAction,
    label: &str,
    color: Color,
) {
    actions
        .spawn((
            Button,
            action,
            BackgroundColor(color),
            Node {
                width: Val::Px(106.0),
                height: Val::Px(38.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.98, 0.96, 0.88)),
        ));
}

fn spawn_job_poi(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    map: &world::TownMap,
    grid: IVec2,
    kind: JobPoiKind,
    colors: PoiColors,
) {
    let halo = meshes.add(Circle::new(12.0));
    let ring = meshes.add(Annulus::new(6.0, 9.0));
    let halo_material = materials.add(ColorMaterial::from(colors.halo));
    let ring_material = materials.add(ColorMaterial::from(colors.ring));
    commands
        .spawn((
            JobPoi { kind },
            Transform::from_translation(world::grid_to_world(map, grid).extend(1.45)),
            Visibility::Hidden,
        ))
        .with_children(|marker| {
            marker.spawn((
                Mesh2d(halo),
                MeshMaterial2d(halo_material),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ));
            marker.spawn((
                Mesh2d(ring),
                MeshMaterial2d(ring_material),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.05)),
            ));
        });
}

pub fn handle_input(
    clock: Res<SimClock>,
    map: Res<world::TownMap>,
    mut contracts: ResMut<Contracts>,
    mut ui: JobInputUi,
    mut trucks: Query<(Entity, &Transform, &mut Truck)>,
    mut route_debug: RouteDebug,
) {
    let clicked = ui.actions.iter().find_map(|(interaction, action)| {
        (*interaction == Interaction::Pressed).then_some(*action)
    });
    let contract = &mut contracts.current;
    let accept_requested =
        ui.keys.just_pressed(KeyCode::KeyA) || clicked == Some(JobAction::Accept);
    let reject_requested =
        ui.keys.just_pressed(KeyCode::KeyD) || clicked == Some(JobAction::Reject);
    let dispatch_requested =
        ui.keys.just_pressed(KeyCode::KeyJ) || clicked == Some(JobAction::Dispatch);

    if accept_requested && contract.accept() {
        ui.presentation.swipe_to(CardView::Active);
        info!(contract = contract.id, "tow call accepted");
    }
    if reject_requested && contract.decline() {
        ui.presentation.swipe_to(CardView::Hidden);
        info!(contract = contract.id, "tow call declined");
    }
    if !dispatch_requested || contract.state != TowState::Accepted {
        return;
    }

    let Some(selected) = ui.focus.selected else {
        warn!("select a truck before dispatching the accepted tow job");
        return;
    };
    let Some((entity, transform, mut truck)) =
        trucks.iter_mut().find(|(entity, _, _)| *entity == selected)
    else {
        return;
    };
    if truck.active_contract.is_some() {
        warn!(truck = ?truck.id, "selected truck is already assigned");
        return;
    }

    let start = world::world_to_grid(&map, transform.translation.truncate());
    let Some(estimate) = estimate_job(
        &map,
        start,
        contract.pickup,
        contract.dropoff,
        contract.payout_cents,
    ) else {
        contract.state = TowState::Failed(FailureReason::NoRoute);
        return;
    };
    if truck.fuel_gallons + f32::EPSILON < estimate.fuel_gallons {
        warn!(
            available = truck.fuel_gallons,
            required = estimate.fuel_gallons,
            "truck lacks fuel for tow job"
        );
        return;
    }
    if route_debug
        .assign_route(&map, entity, transform, &mut truck, contract.pickup)
        .is_none()
    {
        contract.state = TowState::Failed(FailureReason::NoRoute);
        return;
    }

    let assignment = JobAssignment {
        truck_id: truck.id,
        dispatched_at: clock.elapsed_secs(),
        odometer_start: truck.odometer_tiles,
        fuel_start: truck.fuel_gallons,
        wear_start: truck.wear,
        tow_odometer_start: None,
    };
    assert!(contract.dispatch(assignment, estimate));
    truck.active_contract = Some(contract.id);
    info!(contract = contract.id, truck = ?truck.id, "tow job dispatched");
}

pub fn update_contracts(
    clock: Res<SimClock>,
    map: Res<world::TownMap>,
    mut contracts: ResMut<Contracts>,
    mut company: ResMut<Company>,
    mut trucks: Query<(Entity, &Transform, &mut Truck)>,
    mut route_debug: RouteDebug,
) {
    let now = clock.elapsed_secs();
    let contract = &mut contracts.current;
    if contract.expire(now) {
        info!(contract = contract.id, "tow call expired");
        return;
    }

    let Some(assignment) = contract.assignment else {
        return;
    };
    let Some((entity, transform, mut truck)) = trucks
        .iter_mut()
        .find(|(_, _, truck)| truck.id == assignment.truck_id)
    else {
        contract.state = TowState::Failed(FailureReason::TruckUnavailable);
        return;
    };
    if truck.fuel_gallons <= 0.0
        && matches!(
            contract.state,
            TowState::EnRoutePickup | TowState::EnRouteDropoff
        )
    {
        truck.active_contract = None;
        contract.state = TowState::Failed(FailureReason::OutOfFuel);
        return;
    }

    let truck_grid = world::world_to_grid(&map, transform.translation.truncate());
    match contract.state {
        TowState::EnRoutePickup if truck.route.is_empty() && truck_grid == contract.pickup => {
            assert!(contract.arrive_at_pickup(now, truck.odometer_tiles));
            info!(contract = contract.id, "truck arrived; hookup started");
        }
        TowState::HookingUp if now >= contract.hookup_completes_at.unwrap_or(f32::INFINITY) => {
            if route_debug
                .assign_route(&map, entity, transform, &mut truck, contract.dropoff)
                .is_none()
            {
                truck.active_contract = None;
                contract.state = TowState::Failed(FailureReason::NoRoute);
                return;
            }
            assert!(contract.begin_tow(truck.odometer_tiles));
            info!(contract = contract.id, "vehicle hooked; tow leg started");
        }
        TowState::EnRouteDropoff if truck.route.is_empty() && truck_grid == contract.dropoff => {
            let contribution = contract
                .complete(now, truck.odometer_tiles, truck.fuel_gallons, truck.wear)
                .expect("active tow leg must complete");
            company.cash_cents += contribution;
            company.reputation += 1;
            truck.active_contract = None;
            info!(
                contract = contract.id,
                payout_cents = contract.payout_cents,
                operating_cost_cents = contract.actuals.operating_cost_cents(),
                contribution,
                "tow job completed"
            );
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn update_tablet(
    time: Res<Time>,
    clock: Res<SimClock>,
    company: Res<Company>,
    contracts: Res<Contracts>,
    focus: Res<Focus>,
    trucks: Query<(Entity, &Truck)>,
    mut presentation: ResMut<JobPresentation>,
    mut tablet: Query<(&mut Node, &mut Visibility), (With<JobTablet>, Without<Button>)>,
    mut title: Query<&mut Text, (With<JobCardTitle>, Without<JobCardBody>)>,
    mut body: Query<&mut Text, (With<JobCardBody>, Without<JobCardTitle>)>,
    mut actions: Query<
        (
            &JobAction,
            &Interaction,
            &mut BackgroundColor,
            &mut Visibility,
        ),
        (With<Button>, Without<JobTablet>),
    >,
) {
    let contract = &contracts.current;
    let estimate = contract.estimate;
    let total_tiles = estimate.approach_tiles + estimate.tow_tiles;
    let remaining = (contract.expires_at - clock.elapsed_secs()).max(0.0);
    let offer_age = (clock.elapsed_secs() - contract.offered_at).max(0.0);
    let selected_truck = focus
        .selected
        .and_then(|selected| trucks.iter().find(|(entity, _)| *entity == selected));
    let (dispatch_ready, dispatch_note) = match selected_truck {
        None => (false, "Select a truck to dispatch".to_string()),
        Some((_, truck)) if truck.active_contract.is_some() => {
            (false, "Selected truck is already assigned".to_string())
        }
        Some((_, truck)) if truck.fuel_gallons + f32::EPSILON < estimate.fuel_gallons => (
            false,
            format!(
                "Needs {:.1} gal; selected truck has {:.1} gal",
                estimate.fuel_gallons, truck.fuel_gallons
            ),
        ),
        Some((_, truck)) => (true, format!("Unit {} ready to dispatch", truck.id.0)),
    };

    let card_offset = advance_card_transition(&mut presentation, time.delta_secs());

    for (mut node, mut visibility) in tablet.iter_mut() {
        node.right = Val::Px(24.0 - card_offset);
        *visibility = if presentation.view == CardView::Hidden && presentation.transition.is_none()
        {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }

    let (heading, contents) = if presentation.view == CardView::Offer {
        (
            "CUSTOMER TOW REQUEST".to_string(),
            format!(
                "FORM #{id:04}                         {status}\n\nVEHICLE     {vehicle}\nPICKUP      Road tile ({px}, {py})\nDELIVER TO  Road tile ({dx}, {dy})\nURGENCY     {urgency}\n\nDISTANCE    {total_tiles} tiles\nTIME        {duration:.1} min\nFUEL        {fuel:.2} gal\nEXPIRES     {remaining:.0} min  /  received {offer_age:.0} min ago\n\nPAYOUT      {payout}\nOPERATING   {costs}\nMARGIN      {margin}",
                id = contract.id,
                status = contract.state.label(),
                vehicle = contract.vehicle.label(),
                px = contract.pickup.x,
                py = contract.pickup.y,
                dx = contract.dropoff.x,
                dy = contract.dropoff.y,
                urgency = contract.urgency.label(),
                duration = estimate.duration_secs,
                fuel = estimate.fuel_gallons,
                payout = format_money(contract.payout_cents),
                costs = format_money(estimate.fuel_cost_cents + estimate.wear_reserve_cents),
                margin = format_money(estimate.expected_margin_cents),
            ),
        )
    } else {
        let assignment = contract.assignment;
        let assigned_truck = assignment
            .map(|assignment| format!("Unit {}", assignment.truck_id.0))
            .unwrap_or_else(|| "Unassigned".to_string());
        let live = assignment
            .and_then(|assignment| {
                trucks
                    .iter()
                    .find(|(_, truck)| truck.id == assignment.truck_id)
                    .map(|(_, truck)| {
                        let distance = truck.odometer_tiles - assignment.odometer_start;
                        let fuel = assignment.fuel_start - truck.fuel_gallons;
                        (distance, fuel)
                    })
            })
            .unwrap_or((0.0, 0.0));
        let next_action = match contract.state {
            TowState::Accepted => dispatch_note,
            TowState::EnRoutePickup => "Driving to customer pickup".to_string(),
            TowState::HookingUp => "Securing customer vehicle".to_string(),
            TowState::EnRouteDropoff => "Towing to destination".to_string(),
            TowState::Completed => "Delivery complete".to_string(),
            TowState::Failed(reason) => format!("Resolve failure: {reason:?}"),
            _ => contract.state.label().to_string(),
        };
        let elapsed = assignment
            .map(|assignment| clock.elapsed_secs() - assignment.dispatched_at)
            .unwrap_or(0.0);
        let eta = (estimate.duration_secs - elapsed).max(0.0);
        let receipt = if contract.state == TowState::Completed {
            format!(
                "\nRECEIPT  {} revenue - {} operating = {} contribution",
                format_money(contract.payout_cents),
                format_money(contract.actuals.operating_cost_cents()),
                format_money(contract.payout_cents - contract.actuals.operating_cost_cents()),
            )
        } else {
            String::new()
        };
        (
            format!("ACTIVE JOB  #{:04}", contract.id),
            format!(
                "STATUS      {status}\nTRUCK       {assigned_truck}\nNEXT        {next_action}\n\nPICKUP      ({px}, {py})\nDROP-OFF    ({dx}, {dy})\nETA         {eta:.1} min\n\nLIVE DIST   {distance:.1} tiles\nLIVE FUEL   {fuel:.2} gal\nEST. COST   {costs}\nPAYOUT      {payout}{receipt}",
                status = contract.state.label(),
                px = contract.pickup.x,
                py = contract.pickup.y,
                dx = contract.dropoff.x,
                dy = contract.dropoff.y,
                distance = live.0,
                fuel = live.1,
                costs = format_money(estimate.fuel_cost_cents + estimate.wear_reserve_cents),
                payout = format_money(contract.payout_cents),
            ),
        )
    };

    for mut text in title.iter_mut() {
        *text = Text::new(heading.clone());
    }
    for mut text in body.iter_mut() {
        *text = Text::new(format!(
            "{contents}\n\nCOMPANY CASH  {}    REP  {}",
            format_money(company.cash_cents),
            company.reputation
        ));
    }

    let transitioning = presentation.transition.is_some();
    for (action, interaction, mut color, mut visibility) in actions.iter_mut() {
        let shown = !transitioning
            && match action {
                JobAction::Accept | JobAction::Reject => {
                    presentation.view == CardView::Offer && contract.state == TowState::Offered
                }
                JobAction::Dispatch => {
                    presentation.view == CardView::Active && contract.state == TowState::Accepted
                }
            };
        *visibility = if shown {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let base = match action {
            JobAction::Accept => Color::srgb(0.18, 0.43, 0.27),
            JobAction::Reject => Color::srgb(0.50, 0.18, 0.14),
            JobAction::Dispatch if !dispatch_ready => Color::srgb(0.35, 0.35, 0.32),
            JobAction::Dispatch => Color::srgb(0.18, 0.38, 0.48),
        };
        color.0 = if *interaction == Interaction::Hovered && shown {
            base.lighter(0.12)
        } else {
            base
        };
    }
}

pub fn update_pois(
    time: Res<Time>,
    contracts: Res<Contracts>,
    mut markers: Query<(&JobPoi, &mut Transform, &mut Visibility)>,
) {
    for (marker, mut transform, mut visibility) in markers.iter_mut() {
        let visible = poi_is_visible(contracts.current.state, marker.kind);
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if visible {
            let phase = match marker.kind {
                JobPoiKind::Pickup => 0.0,
                JobPoiKind::Dropoff => std::f32::consts::PI,
            };
            let pulse = 1.0 + (time.elapsed_secs() * 2.1 + phase).sin() * 0.12;
            transform.scale = Vec3::splat(pulse);
        }
    }
}

fn poi_is_visible(state: TowState, kind: JobPoiKind) -> bool {
    match state {
        TowState::Accepted | TowState::EnRoutePickup | TowState::HookingUp => true,
        TowState::EnRouteDropoff => kind == JobPoiKind::Dropoff,
        _ => false,
    }
}

fn route_tiles(map: &world::TownMap, start: IVec2, end: IVec2) -> Option<usize> {
    world::road_path(map, start, end).map(|path| path.len().saturating_sub(1))
}

fn estimate_job(
    map: &world::TownMap,
    truck: IVec2,
    pickup: IVec2,
    dropoff: IVec2,
    payout_cents: i64,
) -> Option<JobEstimate> {
    let approach_tiles = route_tiles(map, truck, pickup)?;
    let tow_tiles = route_tiles(map, pickup, dropoff)?;
    Some(estimate_for_distances(
        approach_tiles,
        tow_tiles,
        payout_cents,
    ))
}

fn estimate_for_distances(
    approach_tiles: usize,
    tow_tiles: usize,
    payout_cents: i64,
) -> JobEstimate {
    let total_tiles = approach_tiles + tow_tiles;
    let fuel_gallons = total_tiles as f32 * FUEL_GALLONS_PER_TILE;
    let fuel_cost_cents = (fuel_gallons * FUEL_PRICE_CENTS_PER_GALLON).round() as i64;
    let wear_reserve_cents = (total_tiles as f32 * WEAR_RESERVE_CENTS_PER_TILE).round() as i64;
    JobEstimate {
        approach_tiles,
        tow_tiles,
        duration_secs: total_tiles as f32 / TRAVEL_TILES_PER_SEC + HOOKUP_DURATION_SECS,
        fuel_gallons,
        fuel_cost_cents,
        wear_reserve_cents,
        expected_margin_cents: payout_cents - fuel_cost_cents - wear_reserve_cents,
    }
}

fn quote_payout(tow_tiles: usize, urgency: Urgency) -> i64 {
    let urgency_bonus = match urgency {
        Urgency::Standard => 0,
    };
    8_500 + tow_tiles as i64 * 250 + urgency_bonus
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offered_contract() -> TowContract {
        TowContract {
            id: 1,
            pickup: IVec2::ZERO,
            dropoff: IVec2::X,
            vehicle: VehicleClass::PassengerCar,
            urgency: Urgency::Standard,
            payout_cents: 20_000,
            offered_at: 0.0,
            expires_at: 60.0,
            state: TowState::Offered,
            estimate: JobEstimate::default(),
            actuals: JobActuals::default(),
            assignment: None,
            hookup_completes_at: None,
        }
    }

    #[test]
    fn accepting_and_declining_are_only_valid_for_offered_calls() {
        let mut accepted = offered_contract();
        assert!(accepted.accept());
        assert_eq!(accepted.state, TowState::Accepted);
        assert!(!accepted.decline());

        let mut declined = offered_contract();
        assert!(declined.decline());
        assert_eq!(declined.state, TowState::Declined);
        assert!(!declined.accept());
    }

    #[test]
    fn estimate_separates_approach_and_loaded_distance() {
        let estimate = estimate_for_distances(10, 20, 20_000);

        assert_eq!(estimate.approach_tiles, 10);
        assert_eq!(estimate.tow_tiles, 20);
        assert_eq!(estimate.duration_secs, 20.0);
        assert!((estimate.fuel_gallons - 2.4).abs() < 0.0001);
        assert_eq!(estimate.expected_margin_cents, 18_500);
    }

    #[test]
    fn generated_contract_routes_between_reachable_locations() {
        let map = world::TownMap::load_default();
        let estimate = estimate_job(
            &map,
            map.center(),
            IVec2::new(10, 2),
            IVec2::new(40, 20),
            25_000,
        )
        .expect("known rural route should be reachable");

        assert!(estimate.approach_tiles > 0);
        assert!(estimate.tow_tiles > 40);
    }

    #[test]
    fn tow_contract_lifecycle_records_actual_costs_and_margin() {
        let mut contract = offered_contract();
        assert!(contract.accept());
        let estimate = estimate_for_distances(10, 20, contract.payout_cents);
        let assignment = JobAssignment {
            truck_id: TruckId(1),
            dispatched_at: 10.0,
            odometer_start: 100.0,
            fuel_start: 20.0,
            wear_start: 0.10,
            tow_odometer_start: None,
        };

        assert!(contract.dispatch(assignment, estimate));
        assert!(contract.arrive_at_pickup(15.0, 110.0));
        assert_eq!(contract.state, TowState::HookingUp);
        assert!(contract.begin_tow(110.0));
        let contribution = contract
            .complete(30.0, 130.0, 17.6, 0.16)
            .expect("tow leg should complete");

        assert_eq!(contract.state, TowState::Completed);
        assert_eq!(contract.actuals.approach_tiles, 10.0);
        assert_eq!(contract.actuals.tow_tiles, 20.0);
        assert_eq!(contract.actuals.duration_secs, 20.0);
        assert!((contract.actuals.fuel_gallons - 2.4).abs() < 0.0001);
        assert_eq!(contract.actuals.operating_cost_cents(), 1_500);
        assert_eq!(contribution, 18_500);
    }

    #[test]
    fn accepted_card_swipes_out_and_returns_as_active() {
        let mut presentation = JobPresentation::default();
        presentation.swipe_to(CardView::Active);

        let outgoing = advance_card_transition(&mut presentation, 0.11);
        assert_eq!(presentation.view, CardView::Offer);
        assert!(outgoing > 0.0);

        let incoming = advance_card_transition(&mut presentation, 0.22);
        assert_eq!(presentation.view, CardView::Active);
        assert!(incoming > 0.0);

        assert_eq!(advance_card_transition(&mut presentation, 0.11), 0.0);
        assert!(presentation.transition.is_none());
    }

    #[test]
    fn job_pois_follow_contract_progress() {
        assert!(!poi_is_visible(TowState::Offered, JobPoiKind::Pickup));
        assert!(poi_is_visible(TowState::Accepted, JobPoiKind::Pickup));
        assert!(poi_is_visible(TowState::Accepted, JobPoiKind::Dropoff));
        assert!(!poi_is_visible(
            TowState::EnRouteDropoff,
            JobPoiKind::Pickup
        ));
        assert!(poi_is_visible(
            TowState::EnRouteDropoff,
            JobPoiKind::Dropoff
        ));
        assert!(!poi_is_visible(TowState::Completed, JobPoiKind::Dropoff));
    }
}
