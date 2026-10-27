# Special rooms

Developer notes for special rooms (spec §7 "Special rooms"; the second wave's design is
`docs/design/special-rooms.md`). Moved out of `docs/dev/mapgen.md` in #402: every special-rooms
change edits this file, not the generator notes.

## Data

`data/room_effects.toml` (`content/room_effect.rs`): the shared `chance` (12 % per room) and
`per_level` (1), and one `[[effect]]` per kind of special room (a **feature**), at most 32, ids are
indices (in the save fingerprint; new effects go at the end). Fields: `id`, `name`, `mark` (the
`rooms` dev layer), `enter`, `weight` (1 to 100, default 10), `near` (optional), `level_chance`
(percent, default 0) and `pervade` (needed with it), the eligibility fields `needs_danger`,
`needs_below`, `min_cells`, `max_cells` ([`RoomEffectDef::eligible`]), and what it does: `light`,
`glow`, `noise`, `regen`, `mana_regen`, `shunned`, `floor`, `status` (a status id, resolved and
checked at load: it must ease, at least `DWELL_EASES`, 10 s a degree; `RoomEffects::parse_with`
takes the statuses' lookup, `parse` in tests none), `swarm` (a teeming den's numbers,
[`Swarm`]: `per_cells`, `min`, `max` (1 to 64), `wake` (100 to 1000), `cry` (0 to
`noise::MAX_LOUDNESS`), `loot_level`, `themes = { kind, faction }` (percents, at most 100
together; it needs `needs_danger` and can never pervade), `temperature` (degrees Celsius added to
the level's, -40 to 40, default 0), `burden` (percent of the carrying limit, `BURDEN` 10 to
1000, default 100), and `warp` (percent a step inside warps the stepper, 0 to 100, default 0). An effect with `level_chance` must have
a `pervade` line and be neither `shunned` nor lay a `floor` (`RoomEffectDef::can_pervade`); a level
file's `effect` is held to the same.

Features today: shadowed (10), echoing (8), hushed (8), sanctuary (6), flooded (8), black chamber
(5, `light = 0`), draining (6, `status = "ebbing"`, `level_chance` 5, #403), teeming den (8,
`needs_danger`, `min_cells` 16, `swarm`, #405), hot (5, "sweltering chamber", `temperature = 25`,
#404), cold (5, "frigid hall", `temperature = -25`), silent vault
(5, `silent`, `noise = 1`, `status = "silenced"`, #409), heavy (4, "heavy hall", `burden = 50`,
#412), unstable (4, "unstable room", `warp = 10`, `max_cells = 60`, #411). Draining is the
first with a `level_chance` (cursed rooms bring theirs).

A dungeon's `room_effects` (on a branch or a `[[dungeon.levels]]` range; `content/dungeon.rs`,
`RoomOddsSpec::layer`/`resolve`, field by field: the range's, the branch's, the shared file's)
sets `chance`, `per_level`, `favoured = { id = share, ... }` (fixed shares in percent, 1 to 100,
together at most 100) and `banned = [ids]`, resolved into `RoomOdds { chance, per_level,
favoured, banned }` (`RoomOdds::shared` has neither list). Content errors: unknown ids, an id both
favoured and banned, shares over 100, every feature banned while `chance` is above 0, and the
retired `effects` list (refused, pointing to `favoured` and `banned`). Fennmynster favours flooded
35, hushed 15, shadowed 15, and bans hot (its cursed share and its ban on forges come with those
rooms).

## Generation

`mapgen/special.rs`, last, from the level's map stream, so levels without a special room keep
their terrain. Per candidate (each room of a rooms level but the arrival room and vaults; each of
eight cave chambers): `roll` (the chance, while the level has fewer than `per_level`, a pervading
effect counting as one), then the room is found (a cave chamber's middle is picked first), then
`pick`: `draw` among the features not banned that the room is eligible for (its floor cells, the
level's danger: `Canvas::danger` = the level has a danger rule, `Canvas::below` = it has a level
below), then, for a feature with a `level_chance` and while nothing pervades yet, one more draw:
the feature covers the level (`Canvas::pervading`) and the room stays plain. `place` then makes
the room (`make`: its floor, no overlap, a shunned room never where a creature patrols).

`draw`: if any eligible feature is favoured, one draw in 0..100 walks the favoured ones in effect
order by their shares; past them (or with none favoured), a weighted draw by `weight` over the
eligible features that are not favoured; if that set is empty, over the favoured ones by their
shares. With no favoured feature the first draw is skipped. Adding a feature changes what
existing seeds draw (the Depths' snapshots changed once, in #402).

Fixture branches in `mapgen/tests.rs` (`FIXTURE`) check that a branch favouring only flooded rooms
makes only flooded rooms; `mapgen/special_tests.rs` checks the shares over many draws, a feature
added in a test table turning up in a dungeon that never names it, `level_chance`, and
eligibility.

## Teeming dens

Filling (`mapgen/den.rs`, `den::fill`, called in `mapgen::generate_with` right after
`population::populate`, only on a branch level with a danger): every special room whose effect has
a `swarm` gets `Swarm::sleepers(free cells)` sleepers and `Swarm::hoard(placed)` items, from the
level's own stream `Rng::swarm` (`swarm/<level id>`), so nothing else on the level moves. Free
cells: the room's cells of plain floor (the palette's) with no stairs, light, trap, or creature.
Candidates: `Content::candidates` of the branch's population and of the generic list, filtered to
types hostile to the player (`Content::stance` against the `player` creature) and not unique. The
theme (`Theme::roll`): no draw when both percents are 0; else one `below(100)`, then for a warren or
a tribe one `danger::draw` for the first type. Per sleeper: `Theme::draw` (mixed: `danger::draw`;
a warren: the type; a tribe: `danger::draw` over the faction's candidates, else the first type),
then a cell (`swap_remove` of a drawn index). Each is a `CreatureSpawn` with `asleep: true`, no
route, `rolls_gear: true` (so `gear::arm` rolls it after the population, in spawn order). The
hoard: `loot::roll_with` at the item level + `loot_level` (`LootGrade::Any`, the visit's luck), each
on a plain floor cell of the room (stairs and lights left out) drawn anew. `population.rs` keeps
its creatures and their routes out of a den (as out of a sanctuary), and `special::make` refuses a
den where a generator patrol walks. `travel.rs::make_creature` gives a spawn with `asleep` the
first status that `sleeps` (`Content::sleep_status`) at one degree.

Sleep (`world/den.rs`): a creature whose effects include a status with `sleeps` (Asleep:
`data/status.toml`, `max = 1`, no `eases`, `seen = "asleep"`) — `World::asleep`:

- `creature_turn` first asks `World::slumbers`: a sleeper's turn is a wait (× `stride::STRIDE`
  while the player rests), its `heard` cleared; it never reaches `coarse_turn` or `decide`.
- `World::unaware` (`ambush.rs`) is true for a sleeper whatever it sees: a sneak attack lands.
- `World::spread_noise` is where every noise is heard (`make_noise_at` and `sound_at` call it;
  `noise::hearers` decides who hears): each hearer keeps the `Heard`, then each sleeper among them
  wakes when `loudness × 100 ≥ threshold × percent` (`wakes_to`): `percent` its den's `wake`
  (`Here::swarm`, the room's only; 100 outside a den), × (100 + the player's
  `TalentMods::wake_percent`) / 100 for a noise whose maker is the player (Hush-foot).
- `World::hurt`: harm that leaves an actor alive (spared or not) wakes it.
- `World::wake`: removes the sleep status (not through `ended`: it says nothing to the player
  about themselves), says "The <creature> wakes up." if the player's last perception sees its
  cell, and cries: `make_noise_at(id, Act::Mutter, cry × 100 / VOICE_NOISE)` (a voice of `cry`,
  scaled by the room like any noise), which recurses into `spread_noise`. Each sleeper wakes once,
  so the cascade ends; it is deterministic (hearers in actor order, no draws).
- Sixth sense: `sense_danger` (`spawn.rs`) ends with `sense_sleepers`, "Something sleeps on this
  level." when any actor on the level sleeps.

Sleepers are ordinary hostile actors otherwise: `hostiles_here` counts them toward the spawn cap,
a kill is `kill_hostile`, the status is saved and hashed with the actor. The `bench-turns` `delf`
fixture branch favours teeming dens (`xtask/src/bench.rs`).

## State and the one question

`LevelState::rooms` (`SpecialRoom`: effect, box, a blob's cells, `entered`, `glimpsed`) and
`LevelState::pervading: Option<RoomEffectId>` (from the template: a level file's top-level
`effect`, never on the overworld; or the generator's draw), saved and hashed, each left out while
empty, so other levels hash as before. The rules ask one question, `LevelState::effects_at` (public:
`World::effects_at`), which returns a `Here { room, level }`; every reader folds it the same way
(`Here::light`, `shade`, `glow`, `noise`, `regen`, `mana_regen`, `shunned`): percents multiply,
glows add, flags OR. Readers: perception light and creature sight together (`room_shade` on every
part of a cell's light, `room_glow` in the base light; change `perception.rs` and `sight.rs`
together, `special_rooms_keep_the_light_invariants` and `light_tests::check_invariants` check they
agree), noise (`noise::noise`, `World::sound_at`), `World::healing_rate` and `World::mana_rate`
(the room's pace through `World::room_pace`, where Hallowed rest adds half again to a quickening),
and `World::shuns` (`ai.rs`, by the room only: no pervading effect is shunned). A flooded room's
slower, louder steps come from its floor terrain. `PerceptionView::light_kept` lets the renderer
dim its own transient light.

## Statuses while inside

`World::dwell(id)` (`world/rooms.rs`): every status `Here::statuses` gives at the actor's cell (the
room's, then the level's) is given at one degree through `World::afflict` (so `refresh` stacking
renews its easing; immunity to its `damage_type` refuses it). Skipped for an actor whose
`TalentMods::room_status_immune` lists it (Anchored mind) and, for a status that only touches mana
(`StatusDef::touches_only_mana`), for one whose `max_mana` is 0 (no rat shows "ebbing", and a
quiet creature stays quiet in `untouched_by`). The one call is in `World::pass_time`, before each
actor lives through a stretch of time, when `World::dwelling_here` says the current level has an
effect with a status: so it holds at every stretch, whatever cuts the time (the player's action, a
rest's turns, a creature's coarse turn while the player rests, `stride.rs`, which may cost far
more than `eases`). The design (§3.3) put it after each action; at the start of each stretch is
the same for the player and holds through the coarse turns too. Leaving, the status eases by its
own `eases` (Ebbing: 30 s). `World::settle_in` (`rooms.rs`) dwells everyone on the current
level at once as the player arrives there (`World::travel`) and as a run starts (`World::new`), so
an actor standing in such a room holds its status before its first action (else a shaman standing
in a silent vault on arrival would get one spell off). A creature holding a status is not idle, so it does not stride
(`stride.rs`): creatures in a draining room take turn-by-turn (busy, slowed) turns in a rest.

## The silent vault

`silent` (design §9.1): `noise = 1` and `status = "silenced"`. Silenced (`data/status.toml`,
beside Ebbing) has no tick and `blocks = ["cast", "read", "throw_voice"]`, so the existing block
rules do the work: `World::refuse_blocked` refuses the player's casts, scrolls, and thrown voice
with no time, the abilities menu greys out what costs them (`Unavailable::While`), and
`World::spend_mana` spends nothing while casting is blocked. Creatures: `ai.rs::fight` lets a
`Combat::Caster` cast only while its statuses do not block `cast`; silenced, it closes in and
fights in melee like any other. Not a mana-only status, so creatures without mana hold it too
(and so take turn-by-turn turns in a rest while inside, as with any status).

## Hot and cold rooms

A room's `temperature` sets its cells' temperature and nothing else (design §8, decisions 9 and
10): no status, damage, or effect on items or food; whatever the temperature does is the
environment's. `Here::warmth` adds the effects' degrees (the room's and the level's);
`WorldState::temperature_at(level, content, time, pos)` (`world/ice.rs`) is the one place a
cell's temperature is decided: `temperature_on` (the level's, outside every room) plus
`LevelState::warmth_at`. Readers: perception (`PerceptionView::temperature`, the status bar's
number, at the player's cell), `World::temperature` (the player's cell; dump headers) and
`World::temperature_at`, and the ice: `LevelState::advance_ice` groups cells by (water,
thickness, warmth), open water by (water, warmth) for each warmth the level has
(`LevelState::warmths`: outside every room and each room's), and ticks each group at the air
plus its warmth, so a year's catch-up is still a few groups. A level with a hot or a cold room
(`LevelState::has_warmth`) gets `pre_roll_ice`'s 30 days of weather when it is made, as a level
under the open sky does, so a cold room's pool is frozen when first seen; other underground
levels do not, as before. Neither may pervade (no `level_chance`: a hot level is its
`temperature`).

## Heavy halls

A room's `burden` scales every actor's carrying limit inside (design §9.9). `Here::burden`
multiplies the effects' percents; `WorldState::load` (the one place the rules ask for a load,
`docs/dev/burden.md`) passes it to `burden::load` for the actor's cell, and `burden::in_room`
scales the finished limit (after talents and magic items), rounding down to a multiple of 3 g so
the thirds stay exact. `PerceptionView::own_load` asks the player's cell the same way, so the
status bar and the inventory agree with the rules. Every penalty follows from the load, and
`burden_news` says the change after the step in or out (in `World::apply`, so before the
room's `enter` line, which `perceive` says). Nothing
is saved. `WorldState::load_at` is the load as it would be on another cell:
`World::can_step_to` (`world/travel.rs`, the player's ordinary step) refuses a step onto a cell
where the player would be Overloaded ("You carry too much to go in there.", or the usual line
when they are Overloaded where they stand too), so a step in is refused rather than leaving them
stuck, and a step out of a hall they overloaded themselves in is allowed. Edge exits, stairs, and
creatures keep the load where they stand (`can_move`, `ai.rs`).

## Unstable rooms

`world/warp.rs`: `World::warp_step(at, who)` runs after every step lands, last in
`World::step_onto_trap` (the player's step, a creature's in `ai.rs`, a creature making way in
`talk.rs::swap_places`) and in `World::step_willingly` when a known trap is sidestepped. When the
effects at `at` warp (`Here::warp`: the chance either effect leaves, 1 - (1 - a)(1 - b)) and the
stepper still stands there (a teleport trap may have moved it, a trap may have killed it), one
gameplay draw decides; a warp is `World::teleport`, the teleport trap's own move (a free cell
picked by a hash of the seed, level, turn, actor, and cell; `Event::ActorMoved`), said with the
trap's lines through `World::say` ("You are wrenched elsewhere."; "The giant rat vanishes!" when
the player sees the cell). No draw is made on any other cell, so levels without one replay as
before. An incorporeal creature (`ai.rs`, `phases`) never calls `step_onto_trap`, so it is never
warped. A creature's coarse strides in a rest (`stride.rs`) stop before a cell that warps, as
before a trap, so it takes those steps turn by turn and may warp. Nothing is saved.

## What the player is told

`world/rooms.rs`: `glimpse_rooms` (in `perceive`, before `enter_room`) marks each room of which
the player sees a cell `glimpsed` and says its `near` line, unless they are inside it;
`enter_room` (`world/travel.rs`) says `enter` once per room and marks it entered and glimpsed;
`pervade_line` is said on every arrival on a pervaded level (`World::travel`, and at the start of
a run), with Sense of place followed by what the effect is. `PerceptionView::room` names a room
once entered (with Sense of place, once glimpsed) on cells seen or remembered; look mode reads it.
`PerceptionView::pervading` is the whole level's effect (JSON dumps: `pervading`); look mode does
not name it. The `rooms` dev layer marks each room's plain ground, lists each room with
`entered`/`glimpsed`, and heads with `pervading <id>`.

## Talents

System `special_rooms` (`data/talents.toml`): Sense of place (wayfarer, `sense_of_place`; hook
`PerceptionView::room` and `pervade_line`), Hallowed rest (grit, `room_pace_percent` 50; hook
`World::room_pace`), and Anchored mind (scholar, `room_status_immune` `ebbing`, also system
`mana`; hook `World::dwell`), and Hush-foot (shadow, Stealth 10, `wake_percent` 100, also system
`stealth`; hook `World::spread_noise`'s wake check).

## Tests

`content/room_effect.rs` (fields, defaults, refusals, `Here` folding), `content/dungeon.rs`
(`favoured`/`banned` resolve and layer, the refusals), `mapgen/special_tests.rs` (the draw),
`mapgen/tests.rs` (how often levels get one, the fixture branches, the snapshots),
`mapgen/fennmynster_tests.rs`, `world/room_tests.rs` (each effect in play, messages, pervading
levels, the talents, saving), `world/dwell_tests.rs` (a room's status: Ebbing on the player and on
creatures, not on one without mana, through a rest, over a drained level, eased after leaving,
Anchored mind, saving), `world/den_tests.rs` (the den's data and refusals, sleepers take no
turn and never wake by time, quiet and loud noises and the cry cascade, a kicked door, a step,
Hush-foot, a blow, sneak attacks, the waking line and look mode's word, the spawn cap, Sixth
sense, saving, and a generated den's sleepers and hoard over a fixture branch),
`world/silent_tests.rs` (the silent vault: its data, the player's
refusals and the easing, the hundredth of noise, creatures silenced and a shaman fighting in
melee, silenced from the start, saving), `content/room_temperature_tests.rs` (the field, its range, `Here::warmth`),
`world/room_temperature_tests.rs` (hot and cold rooms: the cells' temperature, the player's and
the status bar's, no status, a cold room's pool frozen on a mild level and not beside it, ice
melting in a hot room in a frost, the pre-roll only with such a room, alike however time passes,
a year's catch-up), `content/room_burden_tests.rs` (the `burden` field, its range,
`Here::burden`), `world/heavy_hall_tests.rs` (the heavy hall: the limit halved inside for the
player and creatures, the messages in and out, perception agreeing, the step refused into
Overloaded but always allowed out), `content/room_warp_tests.rs` (the `warp` field, its range,
`Here::warp`), `world/unstable_room_tests.rs` (the unstable room: about one step in ten inside
warps the player and a creature, never outside, no draw outside, the messages, a free landing
cell, saving).
