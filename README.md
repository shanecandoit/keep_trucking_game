
# Trucking Tycoon (working title) - Game Design Doc

## 1. Vibe

Blue-collar logistics sim. You start with a beat-up tow truck answering emergency calls, and end up owning the highways your entire fleet drives on. The feeling we're chasing: **competence made visible** - watching a system you built (roads, drivers, routes) run smoother because you earned it, not because you got lucky.

Tone: grounded, weathered, proud-of-your-work. Not cutesy, not corporate. Think dispatch radio chatter, coffee-stained clipboards, a mechanic who knows your engine by sound.

**Inspirations:**
- *Factorio* - legible tradeoffs, systems you can read at a glance, satisfaction from optimizing a network you built.
- *OpenTTD* - route/network design as the core fun, industries feeding industries.
- *Overcooked* - real-time tension mechanic (Concrete Clock) layered on top of a cerebral base.
- *Papers, Please* - muted industrial palette, working-class mood without being grim.
- *Pixar Cars* / *Zootopia* - anthro-character option for drivers/mechanics without compromising systems tone. [DEFER]

---

## 2. Core Loop

**Dispatch → Route → Deliver → Reputation → Unlock**

1. Contract appears on the dispatch board (diegetic corkboard/whiteboard UI).
2. Player assigns a driver + truck to the contract.
3. Truck runs the route in real time (or accelerated sim time); route quality, driver skill, and truck condition all affect outcome.
4. Delivery resolves: payout + reputation change + wear applied to truck/driver.
5. Reputation and completed contracts unlock new contract tiers, licenses, and territory.

This loop doesn't get replaced as you progress - it **scales**. Tier 1 tow contracts still exist and matter at endgame because higher tiers generate demand for them (breakdowns, quarry-adjacent service calls).

---

## 3. Progression Tiers

| Tier | Truck | New Mechanic | Genre DNA |
|---|---|---|---|
| 1 | Tow truck | Reactive dispatch, weather-triggered demand spikes | Emergency-response puzzle |
| 2 | Garbage truck | Recurring routes, TSP-style route optimization, bin capacity | OpenTTD network design |
| 3 | Cement truck | **Concrete Clock** hard timer, quarry sourcing | Real-time pressure + resource extraction |
| 4 | Highway construction | Build the roads your own trucks drive on | Factorio-style meta-infrastructure |

Tow trucks never retire - breakdowns and accidents at any tier spawn new tow contracts, keeping the starting truck class relevant forever.

---

## 4. Pillars (ranked by how load-bearing they are)

1. **Build-your-own-bottleneck** - you construct the infrastructure that later constrains and enables you. Roads you build become the only way certain high-tier contracts (esp. cement) are feasible at all. This is the signature mechanic; nothing else can be cut before this.
2. **Legible tradeoffs over RNG** - every failure (wear, missed window, reputation dip) has a visible, preventable cause. No hidden-dice breakdowns.
3. **Matching, not maxing** - driver/truck/contract *fit* matters more than raw stat growth. Prevents collapse into "biggest number wins."
4. **Old tiers stay load-bearing** - tier 1/2 content generates ongoing demand from higher tiers instead of going dead.
5. **Blue-collar texture as garnish, not system** - radio chatter, named drivers, diner stops. Deliberately non-mechanical so it can't cause scope creep.

---

## 5. Systems & Interactions

Five interacting nodes: **Drivers × Trucks × Contracts × Routes/Roads × Reputation**

- Driver skill → truck wear rate → maintenance demand → tow contracts stay relevant (loops tier 1 back in).
- Reputation → contract tier access → higher tiers need better roads → you build roads → your own route times improve (compounding return).
- Route quality → Concrete Clock feasibility → a well-built highway is the *only* way certain cement contracts are possible.
- Driver quirks → contract suitability → reckless driver + fragile cement load = bad matchup; same driver + tow rescue = ideal.

### Territory & Market Unlocks
1. **Infrastructure-gated expansion** (primary) - you can't reach a region until you've built the road there. Reuses the Build-your-own-bottleneck pillar directly.
2. **Reputation-tiered licensing** - permits (city hauling license, DOT interstate cert, quarry contractor bond) unlock via reputation threshold.
3. **Contract-chain referrals** - completing a contract in one region spawns a referral contract in an adjacent one. Organic-feeling expansion, low overhead.

### Maintenance as Pride (virtuous loop)
1. **Fleet condition as visible flex** - pristine trucks get small reputation/priority bonuses and look visibly different on the map. Pride needs a public signal, not a private stat.
2. **Mechanic-as-named-asset** - same character pattern as drivers (skill curve, quirks, banter), reacts visibly to fleet condition.
3. **Preventive maintenance rewarded, not just failure punished** - bringing a truck in early should feel *better* than reactive repair, not just "not bad."
4. **Driver-mechanic relationship loop** (polish layer) - drivers who treat their truck well build rapport with their mechanic, unlocking small perks.

Virtuous loop: pristine fleet → priority contracts → more money → better trucks → easier to stay pristine. Needs a real cost (time/money) attached to maintenance so it's a choice, not a free lunch - otherwise it becomes the single dominant strategy.

---

## 6. Visual Language: Diagrammatic-Realism

Semi-realistic trucks and environments overlaid with clean, OpenTTD-style route lines and diegetic HUD elements (dashboard gauges, bumper-sticker wear indicators).

- **Palette**: desaturated blue-collar base (asphalt gray, rust orange, safety yellow) + one saturated "systems color" (electric cyan) reserved *only* for UI overlays - routes, wear meters, contract pins.
- **Trucks**: realistic silhouettes; wear is shown environmentally (dents, mud, exhaust color) rather than only through a health bar.
- **Roads**: player-built highways look physically distinct from stock roads (fresh vs. cracked asphalt) - makes the core pillar legible without opening a menu.
- **Drivers/Mechanics**: portrait-style icons, not full 3D characters - cheap, enough personality to carry the named-asset pillar.

### Anthro Character Option
[DEFER] If pursuing the "anthro dog" idea (raised as a kid-friendly angle): apply it to **driver/mechanic portraits only**, not to the trucks themselves. Keeps the systems-legible truck/road visual language intact while giving characters warmth. Closest precedent: Overcooked-style crew games (comedic worker characters, real systems underneath). DONOT anthropomorphize the trucks themselves - fights directly against wear-as-storytelling and pulls the whole game toward a younger, Pixar-Cars-style audience.

---

## 7. UI Systems

1. **Diegetic dispatch board** - literal corkboard with pinned contracts. Tutorial-free UI metaphor.
2. **Route-overlay map** - toggleable Factorio-style layers: wear, reputation, contract, traffic.
3. **Truck dashboard close-up** - zoom into a selected truck's gauges (fuel, load integrity, driver fatigue).
4. No standalone stats-spreadsheet screen as primary interface - if it can't be shown diegetically, that's a signal the stat may be unnecessary.

---

## 8. Known Risks / Open Gaps

- **Scope creep**: four vehicle classes each need distinct mechanics - prototype tiers 1–2 only first.
- **Concrete Clock could feel gimmicky** if failure is binary. Make failed pours produce a degraded-but-usable asset instead of total loss.
- **Missing tier-1/2-native tension mechanic**: Concrete Clock only exists at tier 3. Need a tension source for early tiers - weather-driven tow demand spikes are the leading candidate.
- Comparisons to Factorio/OpenTTD are aspirational references, not scope targets - those games represent 15–20 years of depth.

---

## 9. Build Order Recommendation

1. Named drivers with 2–3 legible stats (speed / care / stamina).
2. Soft truck degradation tied visibly to route/driver choices.
3. Tier 1→2 loop with the self-built-road hook stubbed in early (validates the signature pillar cheaply).
4. Contract-tier reputation gates per client type.
5. Fleet-condition-as-visible-flex + mechanic-as-named-asset (reuses driver component).
6. Preventive-maintenance-reward layer once the base loop is proven.
7. Skip deep relationship-dialogue systems entirely; redirect budget into driver/mechanic personality (barks, quirks, radio chatter) for the same vibe payoff at a fraction of the cost.

### Engineering Notes
- Territory-by-road-reach needs a real reachability graph, not a flag-based unlock list - build this data model early.
- Fleet condition should be a *derived* value (computed from wear history) rather than a stored mutable float.
- Mechanics should reuse the Driver component (same stat shape, different role tag) rather than a parallel character system.
- Referral contract chains are just event-triggered spawns off an existing on-complete hook - cheap if the contract system already supports it.
