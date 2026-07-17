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
pub struct JobDebug;

pub fn setup(mut commands: Commands, map: Res<world::TownMap>, mut session: ResMut<GameSession>) {
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
    commands.spawn((
        Text::new(""),
        JobDebug,
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::srgb(0.96, 0.91, 0.78)),
        BackgroundColor(Color::srgba(0.055, 0.05, 0.04, 0.90)),
        Interaction::default(),
        ScreenPanel,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            bottom: Val::Px(110.0),
            width: Val::Px(620.0),
            min_height: Val::Px(146.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
    ));
}

pub fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    clock: Res<SimClock>,
    focus: Res<Focus>,
    map: Res<world::TownMap>,
    mut contracts: ResMut<Contracts>,
    mut trucks: Query<(Entity, &Transform, &mut Truck)>,
    mut route_debug: RouteDebug,
) {
    let contract = &mut contracts.current;
    if keys.just_pressed(KeyCode::KeyA) && contract.accept() {
        info!(contract = contract.id, "tow call accepted");
    }
    if keys.just_pressed(KeyCode::KeyD) && contract.decline() {
        info!(contract = contract.id, "tow call declined");
    }
    if !keys.just_pressed(KeyCode::KeyJ) || contract.state != TowState::Accepted {
        return;
    }

    let Some(selected) = focus.selected else {
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

pub fn update_debug(
    clock: Res<SimClock>,
    session: Res<GameSession>,
    company: Res<Company>,
    contracts: Res<Contracts>,
    mut text: Query<&mut Text, With<JobDebug>>,
) {
    let contract = &contracts.current;
    let estimate = contract.estimate;
    let total_tiles = estimate.approach_tiles + estimate.tow_tiles;
    let remaining = (contract.expires_at - clock.elapsed_secs()).max(0.0);
    let offer_age = (clock.elapsed_secs() - contract.offered_at).max(0.0);
    let controls = match contract.state {
        TowState::Offered => "A accept | D decline",
        TowState::Accepted => "Select truck, then J dispatch",
        TowState::EnRoutePickup | TowState::EnRouteDropoff => "Space changes simulation speed",
        TowState::HookingUp => "Hookup in progress",
        _ => "",
    };
    let receipt = if contract.state == TowState::Completed {
        format!(
            "\nACTUAL  distance {:.1} tiles | time {:.1} min | fuel {:.2} gal\nCosts {} fuel + {} wear | contribution {}",
            contract.actuals.approach_tiles + contract.actuals.tow_tiles,
            contract.actuals.duration_secs,
            contract.actuals.fuel_gallons,
            format_money(contract.actuals.fuel_cost_cents),
            format_money(contract.actuals.wear_reserve_cents),
            format_money(contract.payout_cents - contract.actuals.operating_cost_cents()),
        )
    } else {
        String::new()
    };

    for mut output in text.iter_mut() {
        *output = Text::new(format!(
            "TOW CALL #{id} [{state}]  seed {seed:016X}\n{vehicle} | {urgency} | offered {offer_age:.0} min ago | expires in {remaining:.0} min\npickup ({px}, {py}) -> dropoff ({dx}, {dy})\nESTIMATE  {total_tiles} tiles | {duration:.1} min | {fuel:.2} gal\nPayout {payout} | costs {costs} | margin {margin}\nCompany {cash} | reputation {reputation}\n{controls}{receipt}",
            id = contract.id,
            state = contract.state.label(),
            seed = session.seed,
            vehicle = contract.vehicle.label(),
            urgency = contract.urgency.label(),
            px = contract.pickup.x,
            py = contract.pickup.y,
            dx = contract.dropoff.x,
            dy = contract.dropoff.y,
            duration = estimate.duration_secs,
            fuel = estimate.fuel_gallons,
            payout = format_money(contract.payout_cents),
            costs = format_money(estimate.fuel_cost_cents + estimate.wear_reserve_cents),
            margin = format_money(estimate.expected_margin_cents),
            cash = format_money(company.cash_cents),
            reputation = company.reputation,
        ));
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
}
