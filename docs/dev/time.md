# Time, action costs, and speed

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08a-architecture.md. Keep it current with the code, as CLAUDE.md says.

Time (spec §8 "Time") lives in `warlock-core`: `time.rs` (`GameTime` = seconds since the
calendar epoch with calendar fields, minute of day, and 16-bit day fraction; `Duration`; 12×30-day
calendar math; `DAWN`/`DUSK`), `content/calendar.rs` (month names from `data/calendar.toml`),
`content/action.rs` (action costs, below), and the turn pipeline in `world/mod.rs`.
**Action costs** (spec §8 "Time", M13): every action that takes time has a cost in energy
(`ActionKind`, costs in `data/actions.toml`: 1000 = a normal action; a creature type may give its
attacks their own `attack_cost`; `Content::action_cost`) and a base time, `BASE_TURN` (6 s) for
everything but an overworld step, whose base is the terrain's `travel_minutes`. Its energy price
is `content::price(cost, base)` = cost × base seconds; an actor gains its speed in energy per
second, so it takes `cost / 1000 × 1000 / speed × base` (1400 at speed 1000: 8.4 s; 1000 at 1111:
about 5.4 s). `TerrainDef::step_price` (asked through `LevelState::step_price`) is the one place
movement time is decided: `move_cost` percent on local levels, `travel_minutes` on the overworld.
`player_acts` returns `Spent::Turn(price)` (`World::price_of(actor, kind)`, `World::step_price`),
`run_turn` (the energy scheduler, `world/turn/schedule.rs`, spec §8 "Speed") charges the player for it
and runs the other actors on the level in time order until the player may act again, advancing
the clock (whole seconds; leftover energy carries, so a run of actions takes its exact total time,
up to the second the actor is ready, with no drift) to each action, and `rest` loops waits until
the goal or an interruption, its last turn cut short with `price_until` so it ends exactly at the
goal at any player speed. Long actions (`rest`, `dig`, meals) go through `World::keep_at(events, more,
act)`: turn after turn of `act` while `more` holds, stopping on a creature hostile to the player
(`World::disturbs_rest`: neutral and peaceful ones and allies never) coming within
`World::stop_within` (`[long_action]` of `data/rules/tuning.toml`: 5 steps, Chebyshev; owner, #1216) in view (`watched_in_view`; one farther off,
`hostiles_in_view`, may be seen and the action goes on; `Ended::Came(Came { id, seen })`,
`World::came_line` says "comes into view" or, seen farther off before, "comes close"), on a
perceived attack on the player or a shot or a spell within `stop_within` of them (`World::stirs`, so a
miss or a resisted spell stops it whatever is said), a message (`say`; `note` does not stop it;
hunger worsening, a hurt, an attack said; others' attacks and deaths stop it only within `stop_within`,
`perceive_events`, and a creature's spell only at the player or cast by or at one within
`stop_within`, its lines held and noted otherwise, `creature_cast.rs`), or death (`Ended`). The same rule holds for every long
action through `keep_at` (dig, meals, search, study, ...), and so for a wait, which is a turn of
searching (`Action::Wait` is `Spent::Search(1)`, #1326; docs/dev/searching.md); a rest's turns
are plain waits and do not search. It perceives only after turns in which
a creature that disturbs a rest could have come within `stop_within` (`out_of_view_for`, which
accounts for speed and the cheapest step and leaves out all but hostiles), in which the player
heard something, in which anything happened but creatures moving (a hostile in view but not yet
close may shoot or cast; `passes_unseen`), or in which the weather the player would see changed
(`weather_turned`); past that stretch a turn still passes unperceived when
`World::watched_in_sight` (the rules of sight, `sight::sees`, for the cells of the creatures
that disturb a rest within `stop_within`, with the player's line of sight kept while `World::reshaped`, bumped by
`set_terrain`, and their cell are unchanged) finds the same creatures in view; `None` on the
overworld, with Blood scent, and for light-blind eyes, where only a perception will do
(`stride_tests::a_rest_sees_whom_a_perception_sees`). While the player rests (`World::resting`),
creatures away from them take coarse turns (`world/turn/stride.rs`, `docs/dev/creatures.md`), and
`out_of_view_for` counts a stride's head start. On the overworld each turn of a rest or a forage rolls for an
encounter (`World::watch_roll`, `docs/dev/encounters.md` "Camping and foraging"); the line it says
stops the long action. A turn in which the player moved is
always perceived (a run's step, below; a push).

**Running** (`world/turn/run.rs`, #1325; `Action::Run(dir)`, `run:DIR`, `KeyCommand::Run`, Alt with a
movement key by default) is one more long action through `keep_at`: `act` is a plain step
(`player_moves`, with the per-step training `apply` gives a step), and `more` says whether there is
a next step and which, a pure function of the perception (what the player sees and remembers,
`PerceptionView::cell`, the creatures seen, the traps known; never ground truth), looked at after
each step's perception (a stale view, `viewer() != player_pos()`, is never judged). It is decided
in core turn by turn rather than by the client sending moves, so a run is one recorded action, its
stops are the rest's (`Ended`), and frame timing never touches it. `steppable` is a cell a run may
step onto (known walkable ground, not deep water, a closed door, or a shut chest, no creature
seen, no trap known); `ways_on(at, from)` groups the open cells (walkable or a closed door) around
`at` by neighbours round the ring and drops the group holding `from`; `way_on` is a corridor's one
way on (exactly one group, one or two cells wide; wider is a room). The first step lands in a
corridor when `way_on` has one there: the run follows it (the way's steppable cell nearest the
heading, orthogonal on a tie) and stops at an intersection, a dead end, or a room; else it holds
its direction until the next cell is not `steppable`. A first step that is not `steppable` is a
plain `Move` (`start_running`); the client asks the step's dialogs first (`ui/step_dialogs.rs::step_dialog`).
A creature coming close says "{came}; you stop running."; a run's end trains no Willpower
(`World::running`). Tests: `world/turn/run_tests.rs`.

A new action that takes time
needs an `ActionKind` (with its id and a line in `data/actions.toml`) and must return its price
there,
asked through `price_of`/`step_price`/`price_for`: those apply the actor's burden (`docs/dev/burden.md`,
`World::burdened`), so a price taken anywhere else would escape it (`price_until`, the rest's
last turn, deliberately does). Price an action that changes the load before it does (pick up,
drop). **Every attack** (a blow, a bash, a kick at a creature, a shot, a throw, a creature's
attack or spell) is priced by `World::attack_price(id, kind, with, skills)`: `World::attack_cost`
(the creature type's cost, or the `with` object's own `attack_cost`; the attack speed of what the
actor wears and wields plus the `with` object's own affixes), less the ranks' `speed`, the combat
stance's `attack_cost`, the burden. `with` is the inventory position of what the attack is made
with: the wielded weapon for a blow, the shield for a bash (never the weapon's cost or affixes),
the missile weapon for a shot, none for a kick or a throw. Reloading is `World::reload_price`
(not an attack: no attack speed, no stance); `price_of` applies no stance (a door kick is the
plain `kick`). An attack is priced **as it begins**, before it is made: a rank its marks raise, a
load it lightens (a throw), or a kill changes the next attack's price, never its own
(`weapon_skill_tests::an_attack_is_priced_as_it_begins...`).
`creature_turn` is one creature's action and returns its price: the one place creature behavior
lives (AI decisions, the noise and events an action makes; the AI is `world/creatures/ai.rs`, `docs/dev/creatures.md`).
Speed (`speed` in `data/creatures.toml`, default 1000 = the player's; any player speed works;
`Actor::speed` scales it by the talents' `speed_percent` plus the skills' `speed`, Athletics, the
skills capped so the sum stays within `MAX_SPEED`, and nowhere else: `World::slowed` does not read it again, #581; the burden, Bloated, and statuses are applied
to prices afterwards in `World::slowed`, independent of speed)
is rules-only: never put it in perception, the UI, or messages; energy is per actor in
`WorldState` (saved, hashed); creatures on other levels and the dev skip gain none.
Levels carry `kind` (no turn length of their own); terrain carries `move_cost` (percent); scenarios
carry `start_date`/`start_time`. Rest (`b`) is a message-line input mode (`InputMode::Rest`,
`input::REST_CHOICES`), not a menu screen.
