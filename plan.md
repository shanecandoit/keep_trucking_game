# Keep Trucking Gameplay Plan

## Product Direction

The player begins as an owner-operator with one worn tow truck, a tiny garage, and a telephone. At first, the player answers calls, chooses jobs, drives the truck, and pays outside shops for repairs. Growth should replace work the player has already learned with employees and better infrastructure:

1. Answer calls and drive the tow truck personally.
2. Buy another truck and hire a driver.
3. Hire a dispatcher so calls no longer require constant player attention.
4. Hire an in-house mechanic and maintain a small fleet.
5. Expand the garage, office, fleet, territory, and road network.
6. Operate within a living city whose residents create traffic and service demand.

The core fantasy is not passive number growth. It is seeing a company and city run better because the player chose the right people, trucks, routes, maintenance, and infrastructure.

## Planning Rules

- [ ] Each phase must end in a playable milestone.
- [ ] New automation should replace a task the player previously performed manually.
- [ ] Every failure should have a visible cause: traffic, bad dispatching, wear, staffing, route choice, or insufficient capacity.
- [ ] Keep Tier 1 towing useful after the company expands.
- [ ] Simulate important nearby actors in detail and distant actors at a cheaper level of detail.
- [ ] Keep simulation logic independent from rendering so headless tests can exercise contracts, traffic, staffing, and finances.
- [ ] Use deterministic random seeds for cities and simulation tests.
- [ ] Avoid deep dialogue and relationship systems until the operating loop is proven.

## Phase 0 — Prototype Foundation

Goal: establish a readable isometric world with selectable, road-constrained vehicles.

- [x] Render a square grid through an isometric projection.
- [x] Convert mouse coordinates back into top-down grid coordinates.
- [x] Render arid terrain tiles.
- [x] Render Tier 1 gravel roads.
- [x] Reserve a road tier for later concrete roads.
- [x] Render simple three-face trucks.
- [x] Render simple gray buildings with shop and person doors.
- [x] Select a truck with the mouse.
- [x] Show the selected truck's tile with a cyan outline.
- [x] Show the hovered tile with a cyan light pointer.
- [x] Route a truck along connected road tiles.
- [x] Route building clicks to road-facing entrances.
- [x] Separate background, background UI, foreground, and foreground UI draw passes.
- [x] Test isometric projection round trips.
- [x] Replace the temporary cross-shaped road layout with map data.
- [x] Add a fixed simulation tick separate from frame rendering.
- [x] Add pause, normal speed, and fast-forward controls.
- [x] Add a seeded game-session resource.
- [x] Add an in-game debug panel for simulation time, selected entity, route, and current task.
- [x] Anchor changing diagnostic readouts in fixed-width screen-space Bevy UI panels.

Exit condition: the player can select a truck, choose a reachable destination, and understand the terrain, road, selection, and building layers.

## Phase 1 — Owner-Operator First Day

Goal: the player answers the phone, accepts a tow call, drives the starting truck, completes the tow, and gets paid.

### Time and Day

- [x] Add a game clock with day, hour, and minute; start Day 1 at 08:00 and advance one game minute per simulation second.
- [ ] Define business hours and after-hours calls.
- [x] Add fixed bottom-left pause, 1x, and 3x controls to foreground UI.
- [x] Make contract deadlines use simulation time rather than wall-clock time.
- [ ] End the day with a simple income-and-expense summary.

### Starting Company

- [ ] Create a `Company` resource with cash, reputation, employees, trucks, and property.
- [ ] Start with one small building, one tow truck, and no employees.
- [ ] Give the starting building two truck parking spaces.
- [ ] Give the starting building one mechanic bay.
- [ ] Give the starting office two workspaces.
- [ ] Reserve one office workspace for the player while manually answering calls.
- [ ] Prevent purchases and hires that exceed building capacity.
- [ ] Show current capacity in the building panel.

### Map Commands and Selected-Truck Clipboard

Architecture decision: direct map commands operate on one selected truck. Keep
`Focus.selected` as a single selection; use the dispatch board and garage UI for
fleet-wide work instead of adding RTS-style group selection without a concrete
fleet action that requires it.

- [ ] Replace the unconditional hover cone with a command-target preview that appears only while an idle, manually controllable truck is selected.
- [ ] Resolve the cursor tile once into a shared `CommandTarget`: road tile or building entrance, reachable route, and an explicit blocked reason.
- [ ] Use the same `CommandTarget` for both preview rendering and click execution so the displayed destination cannot disagree with the assigned route.
- [ ] Draw the cone at a building's resolved road entrance rather than its building tile.
- [ ] Hide the command cone while the selected truck has an active tow assignment; keep contract pickup, route, and destination markers visible instead.
- [ ] Show a small invalid/unreachable marker, or no command marker, when clicking would not issue a route.
- [ ] Add an onscreen clipboard card when a truck is selected and remove it when selection is cleared.
- [ ] Make the clipboard status-first: truck/unit identity, player or assigned driver, availability/current task, fuel, aggregate wear, odometer, active job, next target, and ETA.
- [ ] Show `Dispatch to Job` on the clipboard only when an accepted unassigned job exists and the selected truck is eligible.
- [ ] Explain disabled clipboard actions with the actual reason: already assigned, insufficient fuel, incompatible capacity, unreachable pickup, or unavailable driver.
- [ ] Keep the clipboard scoped to one truck; later fleet summaries belong in the garage and dispatch queue rather than expanding this card into a fleet window.

### Telephone and Dispatch

- [ ] Generate incoming tow calls at a basic random rate.
- [x] Give each call an origin, destination, vehicle type, urgency, payout, and expiration time.
- [ ] Ring the telephone and show an obvious unanswered-call state.
- [ ] Let the player answer or miss a call.
- [x] Show the first call's locations, vehicle, expected distance/time/cost, and quoted payout in the debug job panel.
- [x] Let the player accept or decline the first debug job with keyboard controls.
- [ ] Limit the number of active jobs the player can hold.
- [x] Expire the current unanswered or unaccepted debug call using simulation time.
- [ ] Apply a small reputation consequence for accepted jobs that are abandoned.
- [ ] Log why a call was created, accepted, missed, completed, or failed.

### Manual Tow Job

- [x] Add a `TowContract` state machine covering offered, accepted, pickup travel, hookup, towing, completion, decline, expiration, and visible failure reasons.
- [ ] Spawn a disabled customer vehicle at the call origin.
- [ ] Highlight the call origin after accepting the job.
- [x] Let the player dispatch the selected starting truck to the call.
- [x] Show the truck's planned route.
- [x] Detect arrival at the correct roadside service tile.
- [x] Add a short hookup timer.
- [ ] Attach the customer vehicle visually behind the tow truck.
- [x] Route the tow truck to the requested destination after hookup.
- [ ] Detach and deliver the customer vehicle.
- [x] Pay the player only after successful delivery.
- [x] Apply fuel use and aggregate truck wear from actual distance traveled.
- [x] Show a debug completion receipt with payout, distance, time, fuel cost, wear reserve, and contribution margin.

### First-Day Tutorial

- [ ] Introduce answering the phone.
- [ ] Introduce accepting a job.
- [ ] Introduce selecting and routing the truck.
- [ ] Introduce hooking up and delivering a vehicle.
- [ ] Introduce fuel, wear, and outside repair costs.
- [ ] Avoid modal tutorial text when the dispatch board or telephone can teach the action visually.

Exit condition: a new player can complete one full tow loop without developer controls.

## Phase 2 — A Small Tow Company

Goal: the player buys additional trucks and hires people, shifting from doing every job personally to managing a small operation.

### Trucks and Fleet Ownership

- [ ] Define truck identity, model, purchase price, towing capacity, fuel economy, mileage, condition, and service history.
- [ ] Add a used-truck market with a small rotating inventory.
- [ ] Let the player inspect a truck before purchasing it.
- [ ] Make cheap used trucks carry visible wear and maintenance risk.
- [ ] Add truck parking assignments.
- [ ] Prevent buying a truck when no parking space is available.
- [ ] Add truck availability states: `Parked`, `Assigned`, `Driving`, `Towing`, `Refueling`, `Maintenance`, `BrokenDown`.
- [ ] Show fleet state at a glance from the garage.
- [ ] Allow selling a truck with value based on mileage and condition.

### Drivers

- [ ] Add named driver candidates.
- [ ] Give drivers a small readable stat set: speed, care, stamina, and wage.
- [ ] Add driver traits that change job suitability without hidden dice rolls.
- [ ] Add hiring cost and recurring payroll.
- [ ] Assign one driver to one truck at a time.
- [ ] Let the player remain the driver of one truck.
- [ ] Let hired drivers execute assigned routes automatically.
- [ ] Make driver care affect fuel use and wear.
- [ ] Make stamina affect shift length and fatigue.
- [ ] Make speed affect travel time without allowing traffic-rule violations by default.
- [ ] Add rest, shift start, and shift end states.
- [ ] Show why a driver is unavailable.

### Dispatchers

- [ ] Add named dispatcher candidates.
- [ ] Require an office workspace for each dispatcher.
- [ ] Let the player continue answering calls manually when desired.
- [ ] Let a dispatcher answer calls automatically.
- [ ] Give dispatchers a configurable acceptance policy.
- [ ] Let the player set minimum payout, maximum distance, territory, and urgency rules.
- [ ] Make dispatcher skill affect response time and assignment quality.
- [ ] Show which dispatcher accepted and assigned each job.
- [ ] Allow manual reassignment before a truck departs.
- [ ] Make one dispatcher sufficient for the first few trucks but not an unlimited fleet.

### Small-Company Management

- [ ] Add daily payroll.
- [ ] Add fuel expenses.
- [ ] Add insurance and property overhead.
- [ ] Add a basic profit-and-loss summary.
- [ ] Forecast today's committed payroll and likely contract income.
- [ ] Warn about insufficient cash before payroll.
- [ ] Add reputation changes based on response time and completion quality.
- [ ] Add repeat customers and preferred-provider bonuses.
- [ ] Add a dispatch queue view for offered, active, and completed jobs.

Exit condition: the player can operate three trucks with one dispatcher and one or two hired drivers while still choosing to drive personally.

## Phase 3 — Maintenance and the In-House Mechanic

Goal: maintenance becomes a profitable capacity decision rather than a passive health bar.

### Wear and Condition

- [ ] Track wear by subsystem: engine, tires, brakes, hydraulics, and tow equipment.
- [ ] Derive overall condition from subsystem wear instead of storing a second mutable condition value.
- [ ] Accumulate wear from mileage, load, road quality, driver care, and deferred maintenance.
- [ ] Make symptoms visible before failure: smoke, braking distance, slow hookup, noise, or fuel use.
- [ ] Avoid unexplained random breakdowns.
- [ ] Let ignored visible symptoms progress into predictable failures.
- [ ] Record service and failure history per truck.

### Outside Mechanics

- [ ] Let the player tow or drive a truck to an outside repair shop.
- [ ] Charge market labor and parts prices.
- [ ] Add travel and queue time before outside service begins.
- [ ] Let outside shops specialize in different work.
- [ ] Show the expected price and completion time before approving work.
- [ ] Make outside service the correct early-game choice, not merely a punishment.

### Hiring a Mechanic

- [ ] Add named mechanic candidates using the shared employee identity model.
- [ ] Give mechanics skill, speed, care, specialties, and wage.
- [ ] Require one mechanic workspace and one service bay.
- [ ] Limit the starting garage to one mechanic and one active repair.
- [ ] Let the mechanic inspect trucks and produce a visible diagnosis.
- [ ] Let the player approve preventive maintenance or repairs.
- [ ] Deduct parts inventory and labor time.
- [ ] Make mechanic skill affect diagnosis accuracy, service speed, and repeat-work risk.
- [ ] Keep failures legible rather than rolling hidden catastrophic chances.

### Preventive Maintenance

- [ ] Add mileage- and condition-based service intervals.
- [ ] Let the player schedule service around expected demand.
- [ ] Reward early maintenance with lower parts cost, less downtime, and better reliability.
- [ ] Make over-maintenance wasteful enough that timing still matters.
- [ ] Let dispatch avoid assigning trucks due for immediate service.
- [ ] Show upcoming maintenance on the dispatch calendar.
- [ ] Make pristine fleet condition visible on trucks and in reputation.

### Employee and Fleet Morale

- [ ] Track driver satisfaction from truck condition, workload, schedule stability, and pay.
- [ ] Make drivers prefer safe, maintained trucks.
- [ ] Let poor fleet condition reduce driver retention and recruiting quality.
- [ ] Let a good mechanic improve trust and reduce driver complaints.
- [ ] Keep morale effects small, visible, and attributable.
- [ ] Add short workplace barks instead of a deep relationship-dialogue system.

Exit condition: hiring one mechanic can improve uptime and profit, but consumes scarce garage capacity and payroll.

## Phase 4 — Garage, Office, and Property Expansion

Goal: the physical property becomes the main capacity constraint for fleet growth.

- [ ] Represent parking spaces, mechanic bays, office desks, fuel storage, and parts storage explicitly.
- [ ] Let the player inspect occupied and available capacity.
- [ ] Add upgrades to the starting building rather than immediately replacing it.
- [ ] Add a second mechanic bay upgrade.
- [ ] Add additional truck parking.
- [ ] Add a larger dispatch office.
- [ ] Add a parts room and on-site fuel tank.
- [ ] Give upgrades construction cost and completion time.
- [ ] Temporarily block affected capacity during construction.
- [ ] Add larger properties in other parts of the city.
- [ ] Let location affect response time, property cost, and nearby demand.
- [ ] Require road access for every garage entrance and parking space.
- [ ] Let the player move trucks and staff between properties.
- [ ] Add property taxes, rent, utilities, and maintenance.
- [ ] Add a clear building footprint and expansion-preview overlay.

Exit condition: the player must choose between more trucks, more office automation, and more maintenance capacity.

## Phase 5 — Procedural City Foundation

Goal: generate a deterministic, connected city that supports believable traffic and tycoon decisions.

### City Seed and Data Model

- [ ] Add a city seed shown in new-game setup.
- [ ] Generate the same city from the same seed.
- [ ] Separate city data from render entities.
- [ ] Define districts, parcels, buildings, roads, intersections, lanes, and addresses.
- [ ] Store a road graph for routing and a lane graph for traffic movement.
- [ ] Validate that all generated districts connect to the main road network.
- [ ] Validate that service buildings have reachable entrances.
- [ ] Add a headless city-generation test suite.
- [ ] Save generated city data rather than relying on regeneration alone.

### Terrain and Districts

- [ ] Generate an arid terrain field with subtle color variation.
- [ ] Reserve terrain for highways, local roads, buildings, and undeveloped parcels.
- [ ] Generate residential, commercial, industrial, civic, and highway-service districts.
- [ ] Place higher-density districts near major roads.
- [ ] Keep industrial and repair businesses accessible to trucks.
- [ ] Add district-level demand modifiers for towing and deliveries.
- [ ] Add landmarks that make generated cities readable.

### Road Generation

- [ ] Generate one connected arterial backbone.
- [ ] Add local street branches and loops.
- [ ] Avoid excessive dead ends unless they serve a clear neighborhood.
- [ ] Classify roads as gravel, local paved road, arterial, or highway.
- [ ] Start the player's property on a Tier 1 gravel-access road.
- [ ] Reserve concrete and highway construction for progression.
- [ ] Generate legal intersections and turning connections.
- [ ] Mark speed limits and vehicle restrictions.
- [ ] Add shoulders or service areas where tow jobs can safely occur.
- [ ] Ensure every generated building entrance connects to a road lane.
- [ ] Add route-cost weights for distance, speed, congestion, road quality, and truck restrictions.

### Building Generation

- [ ] Generate houses, apartments, workplaces, stores, repair shops, fuel stations, and civic buildings.
- [ ] Assign every building a type, capacity, entrance, parking, and address.
- [ ] Give workplaces job capacity.
- [ ] Give homes household capacity.
- [ ] Give shops inventory categories and opening hours.
- [ ] Give outside mechanics service capacity and specialties.
- [ ] Keep building visuals modular and cheap to render.
- [ ] Add distinct silhouettes for home, shop, office, factory, garage, and fuel station.

Exit condition: a seed generates a connected city with valid roads, entrances, districts, and route queries.

## Phase 6 — Traffic Simulation

Goal: civilian and company vehicles share roads, obey signals, and create understandable congestion.

### Lane Movement

- [ ] Move vehicles along lane centerlines instead of tile centers.
- [ ] Give vehicles acceleration, braking, maximum speed, and safe following distance.
- [ ] Prevent vehicles from overlapping or passing through each other.
- [ ] Add lane occupancy and look-ahead queries.
- [ ] Support turns at intersections.
- [ ] Support driveways and building entrances.
- [ ] Add parking and despawning at trip destinations.
- [ ] Make tow trucks obey ordinary traffic rules by default.
- [ ] Reserve emergency priority behavior for a later upgrade or specific contract type.

### Traffic Lights and Intersections

- [ ] Add stop lines and signal groups.
- [ ] Generate compatible traffic-light phases.
- [ ] Add green, yellow, and red timing.
- [ ] Prevent conflicting vehicle movements.
- [ ] Let vehicles queue at red lights.
- [ ] Allow legal right and left turns according to the intersection definition.
- [ ] Add protected-turn phases where needed.
- [ ] Add all-way stop signs for small intersections.
- [ ] Log invalid intersection or phase data during generation.
- [ ] Add deterministic tests for red-light stopping and phase transitions.

### Routing and Congestion

- [ ] Route vehicles through the road and lane graphs.
- [ ] Update travel-time estimates from recent congestion.
- [ ] Let drivers choose between short congested routes and longer fast routes.
- [ ] Keep route decisions stable enough to avoid constant rerouting.
- [ ] Show traffic as a map overlay.
- [ ] Make congestion affect tow response times and contract feasibility.
- [ ] Create accidents and breakdown demand from visible traffic conditions, not arbitrary spawning alone.

### Simulation Scale

- [ ] Fully simulate nearby visible vehicles.
- [ ] Use simplified movement for vehicles outside the camera area.
- [ ] Aggregate distant district traffic when individual vehicles do not matter.
- [ ] Preserve trip completion and demand statistics across level-of-detail transitions.
- [ ] Set a target number of visible vehicles and profile it.
- [ ] Add counters for active, simplified, and aggregated vehicles.

Exit condition: vehicles complete road trips, stop at red lights, queue without overlapping, and contribute to measurable congestion.

## Phase 7 — Residents and Daily City Life

Goal: residents generate predictable trips from home to work, shops, and home again.

### Population

- [ ] Generate households and assign them to homes.
- [ ] Generate workers and assign them to workplaces with available jobs.
- [ ] Give residents simple needs: work, groceries, fuel, services, and leisure.
- [ ] Avoid simulating personality data that does not affect gameplay.
- [ ] Give households access to zero or more civilian vehicles.
- [ ] Allow walking or abstract transit later without blocking the vehicle MVP.

### Daily Schedules

- [ ] Create morning home-to-work trips.
- [ ] Create midday work-to-shop or service trips.
- [ ] Create evening work-to-shop-to-home trips.
- [ ] Vary schedules enough to avoid one synchronized traffic spike.
- [ ] Respect workplace and shop opening hours.
- [ ] Return vehicles to valid home parking overnight.
- [ ] Carry unfinished trips across simulation speed changes.
- [ ] Add weekend or special-event schedules later.

### Trip Demand

- [ ] Choose destinations based on distance, capacity, price, and preference.
- [ ] Prevent destinations from accepting more visitors than capacity allows.
- [ ] Record trip origin, destination, purpose, route, and travel time.
- [ ] Use actual trips to drive fuel demand, repair demand, congestion, and roadside failures.
- [ ] Generate tow calls from stranded resident vehicles.
- [ ] Generate accident calls from traffic incidents.
- [ ] Generate impound and illegal-parking jobs later.

Exit condition: residents visibly travel from homes to work and shops, return home, and create towing demand through the same city simulation.

## Phase 8 — Tycoon Economy and Dynamic Demand

Goal: city growth, traffic, staffing, and fleet condition combine into a sustainable management game.

- [ ] Derive tow demand from population, mileage, vehicle age, weather, traffic, and road quality.
- [ ] Keep some baseline calls so unlucky seeds cannot starve the player.
- [ ] Add customer types: residents, businesses, police, insurers, and fleet accounts.
- [ ] Add contract pricing based on distance, urgency, load, risk, and reputation.
- [ ] Add recurring commercial and municipal accounts.
- [ ] Add competitors that consume some demand without simulating every competitor vehicle initially.
- [ ] Add territory reputation and preferred-provider status.
- [ ] Add loans with visible repayment schedules.
- [ ] Add bankruptcy protection or recovery options for early balancing.
- [ ] Add taxes, licenses, insurance, payroll, fuel, parts, utilities, and property expenses.
- [ ] Add monthly statements and category trends.
- [ ] Show why profit changed instead of only showing the final number.
- [ ] Add economic difficulty settings through starting cash, demand, and expense multipliers.
- [ ] Add city growth that creates new districts and demand.
- [ ] Let the player improve roads to reduce travel time and unlock heavier work.
- [ ] Unlock concrete roads after the gravel-road phase is understood.

Exit condition: the player can grow or fail based on understandable operational decisions over multiple in-game weeks.

## Phase 9 — Content Progression

Goal: expand beyond towing without making the original tow business obsolete.

### Tier 2 — Garbage and Recurring Routes

- [ ] Unlock garbage trucks through reputation and licensing.
- [ ] Add recurring pickup contracts.
- [ ] Add bin capacity and disposal trips.
- [ ] Add route-order optimization.
- [ ] Generate garbage demand from homes and businesses.
- [ ] Let garbage operations create breakdown and tow demand.

### Tier 3 — Cement and the Concrete Clock

- [ ] Unlock cement trucks and quarry contracts.
- [ ] Add concrete freshness and delivery deadlines.
- [ ] Let degraded concrete produce a reduced outcome instead of total failure.
- [ ] Make road quality and congestion central to feasibility.
- [ ] Require better maintenance for heavy cement work.

### Tier 4 — Road Construction

- [ ] Let the player build and upgrade roads.
- [ ] Add gravel-to-concrete upgrades.
- [ ] Add road construction vehicles and crews.
- [ ] Temporarily reroute traffic around construction.
- [ ] Make player-built roads affect every company route and city trip.
- [ ] Use infrastructure access to unlock new territory.

Exit condition: each new tier adds a distinct operating problem and feeds demand back into earlier tiers.

## Phase 10 — Persistence, Balance, and Release Readiness

- [ ] Save company, employees, trucks, contracts, city, traffic, residents, and random-generator state.
- [ ] Load without changing deterministic simulation outcomes.
- [ ] Add rotating autosaves and manual save slots.
- [ ] Version save data and provide migrations.
- [ ] Add a headless multi-day simulation test.
- [ ] Add economy telemetry for income, expenses, utilization, response time, and failures.
- [ ] Profile city generation time.
- [ ] Profile traffic update time.
- [ ] Profile rendering with target vehicle and building counts.
- [ ] Add accessibility options for color, text size, pause behavior, and speed controls.
- [ ] Add remappable input.
- [ ] Add audio for phone calls, dispatch radio, engines, garage work, and traffic.
- [ ] Add a first-hour onboarding playtest.
- [ ] Balance the first truck purchase and first employee hire.
- [ ] Balance outside repair against hiring a mechanic.
- [ ] Verify the starting garage creates meaningful capacity choices.
- [ ] Verify automation reduces workload without removing player decisions.
- [ ] Verify failures always expose their cause and recovery path.

## Immediate Vertical Slice

These are the next tasks that turn the current visual prototype into gameplay:

- [x] Add a fixed simulation clock with pause and speed controls.
- [x] Add minimum `Company`, `Truck`, and `TowContract` vertical-slice domain models.
- [ ] Add the `Employee` domain model and expand company property/capacity state.
- [ ] Give the starting gray building two parking spaces, one mechanic bay, and two office workspaces.
- [ ] Make the route-target cone conditional on a selected, eligible truck and derive preview/click behavior from one shared `CommandTarget`.
- [ ] Add the selected-truck clipboard card with status, resources, current job, next target, ETA, and contextual dispatch action.
- [x] Seed one deterministic passenger-car tow offer and expose it through the debug job panel.
- [ ] Replace the debug offer controls with a ringing telephone and accept/decline UI.
- [ ] Add one disabled civilian vehicle as a tow target.
- [x] Implement the first tow-contract state machine.
- [x] Let the player accept the debug call and dispatch the selected truck to it.
- [x] Add hookup, towing, delivery, payout, and a debug receipt.
- [x] Add fuel and simple aggregate wear based on actual travel.
- [ ] Add one outside mechanic destination and repair invoice.
- [ ] Add an end-of-day financial summary.
- [ ] Save and load the vertical-slice state.

Definition of done: start a new game, answer the phone, accept a tow, drive to the customer, tow the vehicle to an outside mechanic, receive payment, pay operating costs, and save the game.
