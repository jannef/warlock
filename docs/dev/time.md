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
`run_turn` (the energy scheduler, `world/schedule.rs`, spec §8 "Speed") charges the player for it
and runs the other actors on the level in time order until the player may act again, advancing
the clock (whole seconds; leftover energy carries, so a run of actions takes its exact total time,
up to the second the actor is ready, with no drift) to each action, and `rest` loops waits until
the goal or an interruption, its last turn cut short with `price_until` so it ends exactly at the
goal at any player speed. Long actions (`rest`, `dig`, meals) go through `World::keep_at(events, more,
act)`: turn after turn of `act` while `more` holds, stopping on a creature coming into view (but
for one who talks and fights no one, `World::disturbs_rest`), a message (`say`; `note` does not
stop it), or death (`Ended`), perceiving only after turns in which a creature that disturbs a
rest could have come into view (`out_of_view_for`, which accounts for speed and the cheapest
step and leaves out those who talk and fight no one), in which the player heard something, or in
which the weather the player would see changed (`weather_turned`); past that stretch a turn
still passes unperceived when its events are only creatures moving (and the kept kinds) and
`World::watched_in_sight` (the rules of sight, `sight::sees`, for the cells of the creatures
that disturb a rest, with the player's line of sight kept while `World::reshaped`, bumped by
`set_terrain`, and their cell are unchanged) finds the same creatures in view; `None` on the
overworld, with Blood scent, and for light-blind eyes, where only a perception will do
(`stride_tests::a_rest_sees_whom_a_perception_sees`). While the player rests (`World::resting`),
creatures away from them take coarse turns (`world/stride.rs`, `docs/dev/creatures.md`), and
`out_of_view_for` counts a stride's head start. A new action that takes time
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
lives (AI decisions, the noise and events an action makes; the AI is `world/ai.rs`, `docs/dev/creatures.md`).
Speed (`speed` in `data/creatures.toml`, default 1000 = the player's; any player speed works)
is rules-only: never put it in perception, the UI, or messages; energy is per actor in
`WorldState` (saved, hashed); creatures on other levels and the dev skip gain none.
Levels carry `kind` (no turn length of their own); terrain carries `move_cost` (percent); scenarios
carry `start_date`/`start_time`. Rest (`R`) is a message-line input mode (`InputMode::Rest`,
`input::REST_CHOICES`), not a menu screen.
