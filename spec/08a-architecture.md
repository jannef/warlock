# Warlock specification — 8. Architecture

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

## 8. Architecture

```
crates/
  warlock-core/   plain Rust, no Bevy: map, actors, actions, turns, time, fov, light,
                  perception, rng, levels, mapgen, stats, skills, weapon skills, dice,
                  character, burden, hunger, status, checks, diseases, magic, save
  warlock-cli/    no Bevy: scenarios, replay, dump, verify
  warlock/        Bevy app: render, input, UI, effects, platform (ECS lives here)
xtask/            screenshot capture, golden-image diff, CI helpers
assets/           fonts, levels, scenarios, vaults, definitions, shaders
```

- **Pure core**: `warlock-core` has no Bevy dependency, no platform I/O inside the rules, and no
  wall-clock time. It builds and tests in seconds and is insulated from Bevy upgrades.
- **Data layout**: dense grids for terrain, light, and memory; a simple entity store with stable ids
  for actors; objects as stacks (object id and count, and what a magic item rolled) in sparse
  piles by position, remembered
  piles beside them, and an inventory, its equipment (slot → inventory stack;
  `warlock-core/src/equipment.rs`), and stats on each actor. ECS is used in the presentation
  layer, where it helps with effects, particles, and animation.
- **Perception snapshot**: after each turn, the core produces what the player perceives. The
  renderer, UI, and dumps read only this snapshot, so hidden information cannot leak by construction
  (a dev "reveal" mode excepted).
- **Turn phases**, in order: player action → the other actors on the level act in time order
  until the player may act again, the clock advancing to the moment of each action (the energy
  scheduler, "Speed" below) → environment → perception (line of sight, light, memory, heard
  sounds and their messages) → emit presentation events. An action that makes noise is heard by
  every actor in earshot as it happens (§5 "Hearing"); a creature drops what it heard as it
  acts, the player once they have acted. As the clock moves (and as perception begins, after a
  skip or an arrival), the ice on the player's level is brought up to it (§5 "Ice"), so whoever
  steps next meets the ice of the moment.
- **Time** (`warlock-core/src/time.rs`): world time is whole seconds since the calendar epoch
  (year 1, day 1 of the first month, 00:00), kept in the world state (saved, covered by the state
  hash). The calendar has 12 months of 30 days (a 360-day year) and 24-hour days; month names are
  content (Frostmere, Thawmonth, Seedtide, Rainmoon, Greenleaf, Highsun, Emberheat, Harvestide,
  Leaffall, Mistmoon, Darkening, Deepwinter). `World::time()` returns a `GameTime` with year,
  month (number and index; the name from `Content::calendar`), day, hour, minute, second, minute
  of day (the index for per-minute tables such as the sun's path, §5), and the day fraction in
  16-bit fixed point. All
  integer arithmetic; no wall-clock time.
  - **Costs** (M13): the action decides how long it takes, and the actor's speed scales
    it. Every action that takes time has a **cost** in energy, 1000 for a normal action, given in
    `data/actions.toml` (`step`, `wait`, `open_door`, `close_door`, `unlock`, `kick`, `light`,
    `hood`, `stairs`, `pick_up`, `drop`, `melee`, `shoot`, `throw`, `reload`, `cast`, `wield`,
    `wear`, `body_armor`, `ready`, `eat`, `dig`, `chat`, `throw_voice`; 100 to
    10 000; an action left out, or every action without the file, costs 1000; unknown ids are
    errors), and a **base time**: 6 s
    (`BASE_TURN`) for every action but a step on the overworld. An action of cost *c* takes an
    actor of speed *v* `c / 1000 × 1000 / v × 6 s`: a normal action takes 6 s at normal speed
    (the player's), one of cost 1400 (two-weapon fighting, say) 8.4 s, and a normal action at
    speed 1111 about 5.4 s, at speed 900 about 6.7 s. All built-in actions cost 1000 for now but
    `body_armor` (3000: 18 s); a crossbow shot takes `shoot` and `reload` (12 s), and opening a
    locked door with its key `unlock` and `open_door` (12 s).
    - Equipment (§6 "Equipment"): every thing put on or taken off costs the action of the slot it
      goes in or comes out of (a two-handed weapon: the first hand's), taking off what putting on
      does, and one action pays for all of them: a swap costs both, a quarterstaff that puts away
      a sword and a shield three, dropping a worn jerkin `body_armor` plus `drop`.
    - A step costs `step` scaled by the destination terrain's `move_cost` percent (water 200: 12 s
      at normal speed); on the overworld its base is the destination terrain's `travel_minutes`
      instead of 6 s (scaled by speed like everything else: twice the speed, half the journey),
      less the percent the actor's skills take off a journey (survival, "Skills" below; rounded
      up), in `World::step_price`.
      `TerrainDef::step_price` (through `LevelState::step_price`) is the one place movement time
      is decided.
    - `stairs` covers taking stairs, entering a location or a local area from the overworld, and
      walking off a local area's edge: 6 s each at normal speed, on the overworld too, where
      waiting and lighting a lantern take 6 s as well. Bumping a closed door costs `open_door` (a locked
      one, with its key, `unlock` as well); moving into a hostile creature, `melee`; a kick,
      `kick`.
    - A creature's action costs the same: a step onto the destination terrain, else its action's
      cost (`melee`, `shoot` (and `reload`), `cast` for attacks, `open_door` (and `unlock` for a
      locked door it has the key to), `wait` when it waits or is blocked). Chatting costs `chat`;
      swapping places with someone costs the player's step (§8 "Displacement"). A creature type may give its attacks their own `attack_cost` (§7), so a future
      attack or ability is one more entry. Free and refused actions take no time.
    - Weapons (M29, #149): a blow with a wielded weapon that has an `attack_cost` (§7) costs that
      instead of `melee` (a shot from a missile weapon with one, instead of `shoot`), unless the
      creature type gives its attacks a cost of its own: a great sword 1200 (7.2 s), a stiletto
      900 (5.4 s). Then magic items (§8 "Magic items") take a percent off: attacks (`melee`,
      `kick`, `shoot`, `throw`, `cast`) by the attack speed of the affixes of what the attack is
      made with (a blow: the wielded weapon's; a bash: the shield's; a shot: the missile weapon's;
      a kick or a throw: nothing's) and of what the actor wears and wields (global), steps by its
      moving speed, together at most 50 % off and never below 100. A shield bash costs what a blow
      with the shield costs (`melee`), never the wielded weapon's `attack_cost` or affixes. One
      function decides an action's cost, `World::cost_of`, and one an attack's,
      `World::attack_cost`; burden then slows the price as below.
    - **Every attack is priced the same way** (`World::attack_price`): its cost, less its weapon
      skill's and fighting style's `speed` (§8 "Weapon skills"), as dear as the attacker's combat
      stance makes attacks (§8 "Stances"), slowed by its burden; reloading is no attack (its own
      `reload` saving, no stance). And every attack is priced **as it begins**, before it is
      made: a rank its marks raise, a load it lightens (a throw), or a kill it makes changes the
      price of the next attack, never its own.
    - The scheduler (below) prices an action at its cost × its base in seconds; an actor gains its
      speed in energy per second, so it takes cost × base / speed seconds. The clock keeps whole
      seconds and leftover energy carries, so a run of actions takes exactly its total time up to
      the whole second the actor is ready, never drifting: ten 5.4 s steps (packed earth at 90 %)
      take exactly 54 s, five 8.4 s attacks exactly 42 s, 1111 steps at speed 1111 exactly
      6000 s; ten steps at speed 1111 (5.4005 s each) take 55 s, the first whole second after
      54.005 s. Single actions show as whole seconds (8.4 s as 9, 8, 9, 8, 8).
    - An actor's burden ("Burden" below) raises every price it pays, as a lower speed would:
      ×1.25 Burdened, ×1.5 Strained or Overloaded (rounded up to a whole unit of energy; every
      built-in price scales exactly), for every action alike, overworld journeys included (a
      20-minute road takes 25 minutes Burdened). It is applied in one place, the scheduler's
      prices (`World::price_of`, `World::step_price`, `World::attack_price`), at the burden the
      actor has as the
      action begins (picking up and dropping are priced before the load changes). A burdened
      normal action takes 7.5 s, so eight take exactly 60 s; a rest still ends exactly at its
      goal. Being Bloated (§8 "Hunger") raises them again the same way, after the burden: ×1.25
      (`[bloated] slow`), so four normal actions take 30 s (`World::bloated`).
    - A meal (§8 "Hunger") is several turns of `eat`, one per turn of the food's `turns`.
  - **Speed** (`warlock-core/src/world/schedule.rs`), ADOM-like: every creature type has a
    `speed` (§7), 1000 being normal and the player's; twice the speed is twice as many actions in
    the same game time. An energy scheduler, integer only: each actor on the current level gains
    its speed in energy per second of game time; an action costs its price, cost × base seconds
    ("Costs" above: 6000 for a normal action); an actor may act while its energy is at least 0,
    and leftover energy carries over, so action counts stay exactly proportional to speed and
    cost over any span while the clock keeps whole seconds. The player's speed may differ from
    1000 too (none of the rules assume it does not): an actor's talents' `speed_percent`
    (Fleet-footed, Quick, Ponderous) and its skills' `speed` (Athletics, #548: +5 % at 100, +10 %
    at 200; §8 "Skills") add up, the skills never taking the sum past 50 %, and scale its speed
    (`Actor::speed`); the burden, Bloated, and status effects then raise each price it pays
    ("Costs" above), independently of its speed. After the player acts, the other actors act
    in time order until the player may act again: next is whoever reaches 0 soonest, the clock moves to that moment, and ties go to
    the player first, then by ascending actor id. With equal speeds every creature acts once per
    player turn, in id order, right after the player; a speed-2000 creature acts exactly twice
    per normal turn and a speed-1500 one three times in two. A creature's action is one function
    (`World::creature_turn`, returning its price): creature behavior lives there.
    Creatures on other levels are frozen (they gain no energy), so coming back to a level never
    sets off a burst of actions. Energy is part of the world state (saved, covered by the state
    hash). Speed is hidden from the player: it is not in the perception snapshot, the status bar,
    look mode, or messages; only ground-truth dev dumps (the world state) show energy.
  - **Rest** passes turn after turn through the normal pipeline: the player waits (a `wait`
    each turn), the others act, perception runs; the last turn is cut short so the rest ends
    exactly at its goal at any speed of the player (it costs all the energy the player has and
    gains until the goal; one hour, the next 06:00, or the next 18:00; at most a day). It stops after a turn in which a
    creature came into view (one not seen the turn before; never one who talks and fights no one,
    a creature with a dialogue whose faction is hostile to nobody, as the showcase's people: the
    player sees who it is) or a message was logged that stops a
    rest (all but weather news other than rain starting or growing heavier, §5 "Weather", and
    muttering, "Dialogue" below), and
    logs how long the player rested. The player heals twice as fast all the while ("Healing"
    below), and the player's status effects beginning and ending in the turns that pass without
    perception are said all the same. Digging ("Digging" below) runs through the same loop
    (`World::keep_at`). Turns that end before any creature on the level that could stop the rest
    could come within view, in which the player heard nothing, in which the weather the
    player would see did not change, and in which no trap went off (a teleport moves a creature
    farther than its steps; §8 "Traps"), pass without
    perception: a creature is seen only within the view range, and before its
    first step that could bring it there it needs the energy for every step that cannot (each at
    least the level's cheapest step), so a fast creature shortens the stretch. Those turns cannot
    change when the rest stops (sounds carry at most 30 cells, less than the view range, so none
    is missed), and a night's rest stays fast on large levels
    (`cargo xtask bench-turns` times one; budget 500 ms; also one beside a fight in the
    arena out of the player's sight and hearing, which goes on until one side is dead, and one
    on a generated floor at full population, 80 creatures). Past that stretch a turn still
    passes without perception when all that happened was creatures moving (and the events kept
    for the next perception) and the creatures that could stop the rest whom the player sees,
    by the rules of sight for their cells alone, are those seen before: the same answer a
    perception would give (not on the overworld, nor with Blood scent or light-blind eyes,
    where a perception sees by more than line of sight and light).
  - **Coarse turns while resting** (#349; `world/stride.rs`): while the player rests, a creature
    on a local level that has no dialogue and is **away** from them takes its turns coarsely.
    Away: it means the player no harm, or neither the player nor a spot its mind is set on
    (where it last saw a hostile, a noise it goes to look into) is within its sight radius of
    the player. An away creature that is **idle** (going about its routine: nothing seen,
    heard, or set on, no status effects, able to move, no hostile within its sight radius)
    **strides**: one action of up to 20 steps of its patrol route at once, priced as those steps
    are, each onto a plain cell (walkable, no door, ice, or trap, nobody there, not one it shuns)
    on which the player did not see creatures when they last looked, from which no hostile is
    within its sight radius, and from which its step could not be heard by the player; it stops
    before the first that is not, and makes one step's noise where it ends. Without a route, or
    with someone standing on its next step, it waits 20 waits at once. Its pace stays its own
    (the same energy for the same steps), so a patrol still walks into view and stops a rest, and
    near the player (in view, in earshot, within its sight of them) it goes turn by turn. An away
    creature that is **busy** (a fight with another creature, a noise to look into, the way back
    to its post) acts as usual at 20 times the price: far from the player the rest's world runs
    that much slower for it. So an away creature notices a hostile or a noise up to a stride
    later, its steps within a stride are not heard (by other creatures; the player is out of
    earshot), and fights and searches far from the player go slower; nothing changes outside a
    rest. The rest counts a stride's head start in the stretch above. Deterministic: no
    randomness, creatures in the scheduler's order (a stride reads the perception the rest last
    computed, which is part of the run).
  - **Dev skip** (`F6` in dev builds, replay text `skip:1h`) moves the clock ahead at once;
    nobody acts or gains energy and no turn passes, but it is recorded like any action.
