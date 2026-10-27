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
together), `factions` (faction ids, checked once the factions load,
`RoomEffects::check_factions`: known, none twice), `hoard` (`Hoard::Items`, the default, or
`Hoard::Corpses`); it needs `needs_danger` and can never pervade), `temperature` (degrees Celsius added to
the level's, -40 to 40, default 0), `burden` (percent of the carrying limit, `BURDEN` 10 to
1000, default 100), `mirrors` (`{ images = N }`, 1 to 8, `content::Mirrors`), `warp` (percent a
step inside warps the stepper, 0 to 100, default 0), `collapse` (`{ chance, damage }`,
`content::Collapse`: chance 0 to 100, damage dice; needs `needs_below`), `rise` (`{ faction, after }`, `content::Rise`:
`after` a duration, `RISE_AFTER` 1 s to 1 h; needs `needs_danger`; the faction checked in
`RoomEffects::check_factions`), `unholy` (Consecrate lifts it), `feature` (a terrain id,
transparent and not walkable, not stairs or a door), `sides` (`{ good = LIGHT, evil = LIGHT }`: the
room draws a hidden side, its feature a light per side; the lights are resolved after they load,
`RoomEffects::resolve_places`), `offers` (`{ pray = TABLE }`, `Offer::Pray` and an outcome
table id, checked in `resolve_places` once `data/outcomes/` is loaded; `offers` and `sides` need
a `feature`), `shrine` (`{ regen, status, welcome, unwelcome }`, `content::Shrine`,
`content/room_shrine.rs`: a shrine of a side; needs `sides`; `status` resolved and held to a
room status's easing), and `hoard` (a tempting hoard, `content::Treasure`: `items` 1 to 16, `loot_level`,
`guardians = [least, most]` at most 16, `danger`; it needs `needs_danger` and can never pervade).
An effect with `level_chance` must have a
`pervade` line and be neither `shunned` nor lay a `floor` nor hold `mirrors` nor `collapse`
(`RoomEffectDef::can_pervade`; nor one with a `feature`); a level file's `effect` is held to the
same.

Features today: shadowed (10), echoing (8), hushed (8), sanctuary (6), flooded (8), black chamber
(5, `light = 0`), draining (6, `status = "ebbing"`, `level_chance` 5, #403), teeming den (8,
`needs_danger`, `min_cells` 16, `swarm`, #405), hot (5, "sweltering chamber", `temperature = 25`,
#404), cold (5, "frigid hall", `temperature = -25`), silent vault
(5, `silent`, `noise = 1`, `status = "silenced"`, #409), heavy (4, "heavy hall", `burden = 50`,
#412), mirrors (3, "hall of mirrors", `max_cells = 80`, `mirrors = { images = 2 }`, #419), unstable (4, "unstable room", `warp = 10`, `max_cells = 60`, #411), beast den (5, mark `n`, `needs_danger`, `min_cells` 16, `swarm` with
`factions = ["wildlife", "vermin", "marsh"]` and `hoard = "corpses"`, #410), collapsed (3, "collapsed
floor", mark `k` (`c` is the cold room's), `needs_below`, `floor = "cracked_floor"`, `collapse =
{ chance = 15, damage = "2d6" }`, #420), tempting hoard (5,
`hoard`, "treasure room", mark `$`, `needs_danger`, `hoard = { items = 3, loot_level = 3,
guardians = [2, 4], danger = 1 }`, #415), cursed (6, mark
`g`, `needs_danger`, `level_chance` 5, `rise = { faction = "undead", after = "2m" }`, `unholy`,
#408), altar (8, "altar room",
`min_cells = 9`, `feature = "forgotten_altar"`, `sides`, `offers = { pray = "forgotten_altar" }`,
#406), shrine (4, mark `i`, `min_cells = 9`, `feature = "shrine_stone"`, `sides` with the
altar's lights, `shrine = { regen = 200, status = "unwelcome", ... }`, #413). Draining (also
`unholy`) and cursed are the ones with a `level_chance`.

A dungeon's `room_effects` (on a branch or a `[[dungeon.levels]]` range; `content/dungeon.rs`,
`RoomOddsSpec::layer`/`resolve`, field by field: the range's, the branch's, the shared file's)
sets `chance`, `per_level`, `favoured = { id = share, ... }` (fixed shares in percent, 1 to 100,
together at most 100) and `banned = [ids]`, resolved into `RoomOdds { chance, per_level,
favoured, banned }` (`RoomOdds::shared` has neither list). Content errors: unknown ids, an id both
favoured and banned, shares over 100, every feature banned while `chance` is above 0, and the
retired `effects` list (refused, pointing to `favoured` and `banned`). Fennmynster favours flooded
35, cursed 15, hushed 15, shadowed 15, and bans hot (its ban on forges comes with them).

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

Beast dens (#410, design §9.5): a `swarm` with `factions` swaps the hostile filter for a faction
filter (`den::sleepers`: types of those factions, hostile or not, never unique; themes draw within
them). Such a kind is eligible only where the branch's own `population` holds a non-unique type of
one of them (`den::peopled`, asked by `special::pick` through `Canvas::population`, the branch's
population or empty), so a branch without beasts never draws one and a favoured share passes to the
rest. `hoard = "corpses"` (`den::corpses`): per piece (`Swarm::hoard`), one `danger::draw` over the
branch's whole population and strays (no uniques, any faction: the prey), its corpse
(`Content::corpse_of`; none for a type that leaves none) on a cell drawn anew, then one `bones`
(`data/objects/bones.toml`, junk) on another. Both lie with no `died`, so `rot::made_at` makes them
when the level is first entered: fresh then. `loot_level` is unused.

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

## Tempting hoards

Items (`mapgen/hoard.rs`, `hoard::fill`, called in `mapgen::generate_with` right after
`den::fill`, only on a branch level with a danger): every special room whose effect has a
`hoard` gets `hoard.items` items, `loot::roll_with` at the item level + `loot_level`
(`LootGrade::Any`, the visit's luck), each on a plain floor cell of the room (the palette's floor,
no stairs, light, or trap) drawn anew, from `Rng::swarm` (`swarm/<level id>`; a level with a den
and a hoard starts the stream afresh for each, which only happens with `per_level` above 1).
`population.rs` keeps its creatures out of a hoard and `special::make` refuses one where a
generator patrol walks, as for a den: it lies unguarded.

Sealing (`world/hoard.rs`): `World::lift_in_hoard(at, stacks)` is the one line in
`carry.rs::pick_up`, after the pick-up is said. Inside a room with a `hoard` that has not sealed,
`SpecialRoom::lifted` grows by the stacks lifted; once it passes the player's
`TalentMods::hoard_grace` (Hoard-wise: 1) the room is `sealed` and `seal` runs: every cell of the
room (`cells()`: a rooms level's rectangle holds its walls and doors) that is an open door with
nothing in the doorway is shut (`set_terrain` to its `closes_to`), every closed door there then
gets `Lock::new(None)` unless it is locked already (only a skeleton key fits; `lock_difficulty`
adds the branch's depth), and each makes a slam through `sound_at` (`Banging`, `KICK_NOISE`, no
maker: sleepers and creatures hear it as a trap's noise); "The doors slam shut!" (`hoard::SLAM`)
when any. No `Event::DoorClosed` is sent (the pick-up has no event list; the screen follows the
terrain). Then `guard`: on a level with a danger rule (`Content::spawning`) and a danger,
`rng.range(least, most)` guardians from the **gameplay** stream, each `danger::draw` at the
danger + `hoard.danger` over the branch's population and generic list (`Content::candidates`,
hostile to the player, not unique), placed by `travel::make_creature` (`rolls_gear: true`, awake,
no route) on a cell from `guard_cells`: the room's walkable cells with no actor, door, or stairs,
those on its edge (a neighbour outside the room or not walkable) first, among them first the
cells the last perception does not see (`Perception::sees_actors_at`), then the seen ones, then
the inner cells; each cell taken once. Saved: `lifted` and `sealed` on `SpecialRoom`, each left
out while 0/false, so other rooms hash as before. The `rooms` dev layer adds `, lifted N` and
`, sealed`.

## Forgotten altars and prayer

#406, the design's §7. Data: the `altar` effect, the terrains `forgotten_altar` (glyph `_`,
light `altar_neutral`, a `description` look mode adds after its name) and `spent_altar` (dark,
no light), the lights `altar_good`, `altar_neutral`, `altar_evil` (`data/lights.toml`), the
`pray` action cost (1000 a turn), `[shift] prayed_good`/`prayed_evil` (`data/alignment.toml`),
and the outcome table `data/outcomes/forgotten_altar.toml`.

Outcome tables (`content/outcome.rs`, `Content::outcomes`, one file each, the id its name):
`seconds`, `kinds` (boon, nothing, bane weights), `lean`, `exhaust` (default 50), `spent`
(needed while `exhaust` is above 0), and `[[outcome]]` (`kind`, `tier` minor/moderate/great,
`id`, `weight` within its kind, `message` or `messages`, `otherwise`, and `effect`, an
`OutcomeEffect`: each field one hook, applied in the struct's order). `OutcomeTable::leaned`
moves the lean; `Content::offer_at(terrain)` is what a feature cell offers (the first effect
whose `feature` it is), so a spent altar, another terrain, offers nothing.

Generation (`mapgen/special.rs`): `place` draws the side (one `below(4)` from the map stream:
0 good, 3 evil, else neutral) only for an effect with `sides`, then `make` finds the feature's
cell (`feature_cell`: plain free floor nearest the box's middle, nothing on it or on a patrol,
no door among its eight neighbours, and the room's walkable cells still 4-connected without it;
none, and the room is not made), lays it, and puts the side's light on it (`canvas.lights`).
Level files: `[[room]] side = "..."` (only for an effect with `sides`; default neutral), and the
side's light on each feature cell the level draws in the room (`content/level.rs`).

Prayer (`world/pray.rs`): `Ability::Pray` (`Source::Place`, *Here*; had by no one,
`Ability::here(level, content, pos)` lists it beside a feature that offers it, and
`PerceptionView::abilities` appends it). `Action::Pray` (`start_prayer`: refused with no altar
beside the player; `Spent::Pray`), `World::pray`: `keep_at` until the table's `seconds` have
passed (each turn the `pray` price, the last cut to `price_until`), a broken prayer says so and
nothing else; done, `prayer_answered`: `Deed::Prayed(side)`, `leaned_kinds(table, side, band,
devout)` (`world/outcome.rs`), `draw_outcome` (kind, then outcome, gameplay stream),
`befall`, then, while the altar's cell is still the feature and the player lives, the exhaustion
draw (`exhaust` above 0 only): `set_terrain(spent)` with the placed light removed.

Outcomes (`world/outcome.rs`, `World::befall`): the line first (drawn among `messages`; for an
outcome with `otherwise`, only once a hook found something to do, else `otherwise`), then the
hooks: `heal`/`mana` (dice or `full`, capped), `cure` (statuses, `ended`), `cure_by`
(`World::cure` with the means: `prayer` cures diseases), `mend` (`World::mend`), `drain`,
`status` (afflicted with a scratch event list, so the status's own start line is not said),
`hunger` (to the least nutrition of the state `hunger` states away, Satiated to Fainting),
`objects` and `loot` (at `foot_of`: the walkable cell beside the feature nearest the player and
not theirs, else theirs), `know_traps`, `identify`, `forget` (`Stack::forget`), `burn`
(`Harm::Spared`), `smite` (`Harm::Place`, can kill: `Cause::Altar`), `lose_coins`
(item class `coins`), `snuff`, `summon` (`danger::draw` over the branch's hostile non-unique
candidates at the floor's danger plus `danger`, else its deepest; at `Near::Feature` the nearest
free cell within 3, at `Near::Edge` a free cell of the room farthest from the feature, out of
the player's view if any; none on a level without a danger), `disease`, `wither`, `xp` (a share
of the span to the next level, `grant_xp`), `tithe` (the most valuable stack, worn and wielded
too), `teleport` (the trap's move), `explode` (`sound_at`, then `cast_burst` with
`Caster::Place`, whose harm is `World::place_hurts`: resisted, "The altar's fire burns you.", a
seen creature's line, `Harm::Place`), and `becomes` (the placed light removed, `set_terrain`).

Nothing new is saved but `SpecialRoom::side` (left out while `None`): the terrain is the record
of a spent or blown altar. The `rooms` dev layer adds `, side good` and `, spent`/`, destroyed`
(`dump.rs::feature_state`). The showcase's `shrine` scenario has a good, a neutral, and an evil
altar side by side. Talent: Devout (system `prayer`, `TalentMods::devout`; hook
`leaned_kinds`).

The great outcomes (#407, `world/boons.rs`; tests `world/boons_tests.rs`) are five more hooks,
applied after `becomes`: `gift` (`Stats::gift` on a random stat), `wrath` (`Stats::lessen`,
floor `DRAIN_FLOOR`, then stat damage clamped and `keep_hp_within_max`), `levels` (positive:
`gain_level`, `add_xp` to the next threshold exactly, unscaled by Good learner; negative:
`World::lose_level`), `talent` (Chosen: `talent::offers` flattened to the ways not held, one
drawn, `bestow`: `Talents::take`, `class::hand_out`, `train_chosen`, and pushed to
`Actor::boons`), and `flaw` (Forsaken: the `flaw` talents not held, by id, one drawn, given
likewise); `rarity` (`magic`/`rare`, only with `loot`) is the relic's floor, the `LootGrade`
`loot::roll_with` takes. `Actor::boons` (saved, left out while empty) is read by
`stars::grant`, which passes its ids to `Talents::regrant` beside the race's and the sign's
(the flaws, being granted, are dropped and put back there) and `Talents::hold`s the rest.
`World::lose_level` takes the experience to `threshold_for(level - 1)`, the level's
`level_hp` (up to `hp_level`), the talents' `hp_at`/`mana_at` growth, Hardened's, and
`level_mana` (up to `mana_level`) off the bases (hit points never below 1), drops `Due`s of
that level and above, and raises `Progress::peak` (saved, left out at 0): `level_up` gives a
level up to `peak` its hit points and mana again but no `Due`, no stat checks, and no class
news. Penitent (`TalentMods::penitent`) is read in `prayer_answered`: a drawn outcome of kind
bane and tier great is replaced by `draw_in(table, nothing)`. Perception's `OwnTalent::boon`
marks them: `(altar)`/`(altar, flaw)` in the talents dump, "altar" on the Talents page, left
out of the character screen's sign gifts; JSON adds `boon: true` only then.

## Shrines of a side

#413, the design's §9.3. Data: the `shrine` effect, the terrain `shrine_stone` (`∩`, no light of
its own: a shrine is never neutral, so its side's light is always placed), the status
`unwelcome` (`status.toml`, beside Silenced: `regen = 50`, `combat = { to_hit = -1 }`, `eases =
30`). Generation (`mapgen/special.rs::place`): an effect with `shrine` draws its side with one
`below(2)` (0 good, else evil) instead of the altar's `below(4)`; the feature and its light are
the altar's code. Level files (`content/level.rs`) refuse a shrine room without `side = "good"`
or `"evil"`.

Rules (`world/shrine.rs`): `LevelState::shrine_at(content, pos)` (the room's `Shrine` and its
side) and `Shrine::regard(side, Actor::nature)` (`Welcome`, `Unwelcome`, `Ignored`). Three
hooks, one line each: `World::dwell` ends with `dwell_shrine` (the status through `afflict`, as
a room's `status`; `World::dwelling_here` counts a level with a shrine, so `pass_time` and
`settle_in` call it); `World::healing_rate_at` multiplies the room's pace by `shrine_pace`
(`regen` for one welcomed, else 100) before `room_pace`, so Hallowed rest adds to it; and
`travel.rs::enter_room` says `shrine::enter_line` (`welcome`, `unwelcome`, or the effect's
`enter` for a neutral one). Nothing new is saved: the side is `SpecialRoom::side`, and the
status is saved with the actor. The showcase's `shrine` scenario has a good and an evil shrine
south of its altars.

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

## Cursed rooms

`world/curse.rs` (#408, design §5). `World::curse_corpse(dead, by_player)` is the one hook, one
line in `fight.rs::creature_dies` right after `leave_corpse` (so at the corpse's cell, after an
incorporeal creature's landing): with `Here::rise` at the cell (the room's, else the level's), a
dead type not of the rise's faction, with `corpse` and a corpse object, and not the player's kill
under Grave warden (`TalentMods::grave_warden`), `rises_as` draws the type from the gameplay
stream (`rise_candidates`: the faction's non-unique types, shared or the run's campaign's, of the
nearest level to the dead's, the lower on a tie, in id order; one `below(n)` draw, none for a
single candidate) and pushes a `Rising { at, pos, from, into }` on the dead's level
(`LevelState::risings`, saved and hashed, left out while empty; `Rising::is_valid_for` in the
save guard). The twitch line goes with the death's break lines when the player's last perception
sees the cell. `World::rise_tick` runs at the end of `run_turn`, after `spawn_tick`: the current
level's risings with `at` ≤ now, in order; each takes one corpse of `from` (by object, not by
death time) from the pile at `pos` if there is one, then `travel::make_creature` (no gear, no
route) on `free_cell_near(pos)`, bumps `spawned` (so a long action looks up at once), and says the
rising line if seen, else `make_noise_at(risen, Act::Mutter, 100)`. Levels the player is not on
are never ticked, so their due risings rise on the first turn back. `World::risings` (the current
level's) feeds the `actors` dev layer's `rising <from> at (x, y) as <into> at hh:mm:ss` rows.

Consecrate (`world/consecrate.rs`): `ClassActive::Consecrate` (`TalentEffect::Consecrate { mana,
time }`, `TalentMods::consecrate`; the `consecrate` talent, priest 9 and paladin 12, `needs =
{ requires_alignment = { min = "good" } }`). `use_class` → `begin_consecrate`: `hallowable` (the
unholy room the player stands in, else an unholy `pervading`), refused with no time when none;
then `spend_mana` and `Spent::Consecrate(until)`; `World::consecrate` keeps at it through
`keep_at` with wait prices cut to end at `until` (as a rest); done, `consecrated` removes the room
from `LevelState::rooms` or clears `pervading`, keeps only the risings still under a `rise`, says
so, and `player_did(Deed::Consecrated)` (`[shift] consecrate`, 20). Interrupted: "You stop your
prayer." (or who came into view), nothing lifted. The abilities menu greys it out without its
mana or while strayed (`Ability::unavailable`'s class branch: Consecrate's mana with Inspire's).

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

## Halls of mirrors

`world/mirrors.rs` (#419). `WorldState::images(content, level)` is the one place images are
decided (ground truth, `World::images` for the current level): for each room whose effect has
`mirrors`, in the level's room order, the free cells (`cells()` that are walkable, no door, no
actor standing there) are listed in cell order, and each actor inside, in id order, takes
`images` of them, the `index`th at `fnv1a(level id, turn, actor id, index) % free.len()`, removed
from the list (so images never share a cell; a full hall shows fewer). No stream is drawn, so
asking twice, or in perception and the rules alike, gives the same images; they move with
`WorldState::turn`.

Perception (`Perception::compute`) adds an image of each actor it sees, on a cell it sees
(`Sight::Seen`), unless Mirror-wise (`TalentMods::sees_reflections`) puts it within that many
cells (Chebyshev) of the player: a copy of the real `SeenActor` but for `id` (`image_id`: from
2^31, by actor, turn parity, and index, so never a real id and new each turn) and `pos`, with the
real one's gear and showing statuses under the image's id too; `actors` stays sorted by id.
`Perception::shows_image_at` (private `images` cells) is what the rules read of it. Anything that
looks up a perceived id in `WorldState::actors` must expect images to be missing there
(`World::creatures_in_view` leaves them out, so a rest is never disturbed by one).

The rules: `World::meets_glass` (one line in `travel.rs::player_moves`, where no real actor
stands): a move into a cell where the player sees an image says "Your blow meets only glass." and
costs the melee attack's price. `World::reflected` (in `shoot.rs::fly`): a missile aimed at an
image (the player's: one they see; a creature's: any) does not stop at the aimed cell. Bolts and
rays (`cast.rs`, `spell.rs`) already fly to the first real actor, so they need nothing.
`World::aim_at` (one line each in `shoot.rs::creature_shoots` and `hurl.rs::creature_hurls`):
a creature outside the hall shooting or throwing at the player inside picks the player or one of
their images with `rng.below(images + 1)` from the gameplay stream; inside the hall, or at a
creature, it aims true (no draw, so other fights keep their rolls). Creatures' sight and AI read
ground truth and never see images.

Look mode names the player's own image "Your reflection" (it is plain which one is the player);
any other image reads as its creature does. The `actors` dev layer adds `; images (x, y) (x, y)`
to each actor's row; `view` and the JSON dump show images as the player does (their ids are
large: a dev can tell, the game screen cannot).

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

## Collapsed floors

`world/collapse.rs` (#420). `World::collapse_step(at)` is one line at the end of
`travel.rs::player_moves`, after the step's trap and warp (`step_willingly`/`step_onto_trap`):
only the player's moves, so creatures never fall and draw nothing. It returns at once, drawing
nothing, unless `Here::collapse` (the room's only, never a pervading effect) holds at `at`, the
player still stands there, the cell is the room's laid `floor` (cracked floor; a room's other
cells never give way), the level has a down staircase whose `to` is set (`level_below`), and
the player lacks Light tread (`TalentMods::collapse_proof`). Then one gameplay draw at
`chance`, doubled while `load_of(player).penalty() >= Burdened` (capped at 100). On a fall:
`World::visit` makes the level below (as stairs would, with the visit's luck and level),
`landing` lists its cells (a flood fill from `start`, else the first staircase, through
walkable cells and doors without a lock; then walkable, no door, stairs, trap, ice, actor, or
shop floor), a second gameplay draw picks one, "The floor gives way beneath you!",
`World::travel(below, Some(cell))` (arrival lines, pervading line, `settle_in`, as stairs), an
`Event::LevelChanged`, "You fall to <name>.", `collapse.damage` through `World::hurt` with
`Harm::Fall` (#523: it can kill; the player's death then has `Cause::Fall`, "Fell through a
collapsed floor" on the death screen and in the JSON dump's `cause`), and, if they live,
Staggered (`status.toml`) at one degree. The generator (`mapgen/special.rs::make`) lays an
effect's `floor` that has a `collapse` only on the room's middle (`special::middle`: each of the
eight neighbours a walkable cell of the room and no door), so a sound rim runs round it from
every door and a way across always exists (a cave chamber's open edge counts as its rim); a room
too narrow for a middle gets no cracks, and the draws are the same as before, so only the
cracked cells of generated levels change. The
design's `Arrival::Fell` is not needed: `travel` already takes the arrival cell. Nothing is
saved beyond the level change; the cracked floor stays.

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
`World::room_pace`), Anchored mind (scholar, `room_status_immune` `ebbing`, also system
`mana`; hook `World::dwell`), Hush-foot (shadow, Stealth 10, `wake_percent` 100, also system
`stealth`; hook `World::spread_noise`'s wake check), Mirror-wise (scholar, `sees_reflections` 2; hook
`Perception::compute`'s images, which the glass reads too), Light tread (delver, Dexterity 12,
`collapse_proof`; hook `World::collapse_step`, which then draws nothing), Hoard-wise (delver,
Intelligence 11, `hoard_grace` 1; hook `World::lift_in_hoard`), and Grave warden (guard, Willpower 12,
`grave_warden`; hook `World::curse_corpse`). The class ability Consecrate (above) carries the
system too. System `prayer`: Devout (wayfarer, `devout`; hook `world/outcome.rs::leaned_kinds`) and Penitent
(grit, Willpower 12, `penitent`; hook `world/pray.rs::prayer_answered`).

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
`world/beast_den_tests.rs` (the beast den's data, unknown or twice-named factions and an unknown
hoard refused, a generated one's sleepers all beasts with bones and corpses for a hoard, and a
branch without beasts that never draws one),
`world/silent_tests.rs` (the silent vault: its data, the player's
refusals and the easing, the hundredth of noise, creatures silenced and a shaman fighting in
melee, silenced from the start, saving), `content/room_shrine_tests.rs` (the `shrine` field
parsed with its status, and its refusals), `world/shrine_tests.rs` (the shrine: its data and
the regard table, the side's light, Unwelcome on the other side and its easing, twice the
healing for its side, nothing for a neutral one, creatures by their own side, a level file's
side refused unless good or evil, saving), `mapgen/shrine_tests.rs` (generated shrines: one
stone, good or evil one to one, the side's light), `content/room_temperature_tests.rs` (the field, its range, `Here::warmth`),
`world/room_temperature_tests.rs` (hot and cold rooms: the cells' temperature, the player's and
the status bar's, no status, a cold room's pool frozen on a mild level and not beside it, ice
melting in a hot room in a frost, the pre-roll only with such a room, alike however time passes,
a year's catch-up), `content/room_burden_tests.rs` (the `burden` field, its range,
`Here::burden`), `world/heavy_hall_tests.rs` (the heavy hall: the limit halved inside for the
player and creatures, the messages in and out, perception agreeing, the step refused into
Overloaded but always allowed out), `content/room_mirrors_tests.rs` (`mirrors` parses, its range, never over a whole level), `world/mirrors_tests.rs` (images on free floor, the same however asked
and moving with the turns, none outside, the player's own, the glass, a missile flying on, the
creatures' aim, Mirror-wise, the dumps, a rest among reflections), `content/room_warp_tests.rs` (the `warp` field, its range,
`Here::warp`), `content/outcome_tests.rs` (the altar's table, `offer_at`, defaults, the lean,
refusals), `mapgen/altar_tests.rs` (one altar a room, never by a door, its side's light, the
level still joined, the sides one to two to one), `world/outcome_tests.rs` (Pray only beside an
altar not spent, a minute, a broken prayer, the lean and Devout, the draw's shares, the nothing
lines, each hook, the explosion, the deed, exhaustion near half and a spent altar after a save
and load), `world/unstable_room_tests.rs` (the unstable room: about one step in ten inside
warps the player and a creature, never outside, no draw outside, the messages, a free landing
cell, saving), `content/room_collapse_tests.rs` (the `collapse` field, its refusals, never
over a whole level, `Here::collapse`), `world/collapse_tests.rs` (the collapsed floor: its data,
about 15 % of steps on cracked floor fall and none on sound floor or outside, twice as often
Burdened, Light tread and the other no-draw cases, creatures never fall, the landing, hurt and
Staggered, a fall that kills and its `Cause::Fall`, the messages, a walk through `World::apply`,
and over a fixture branch of rooms and caves every generated collapsed floor's sound rim and a
sound way between its ways in), `world/hoard_tests.rs` (the
tempting hoard: its data and refusals, the first stack sealing it with both doors shut and locked
and two to four hostile guardians, the guardians on the edge and out of view, nothing lifted
outside counting, Hoard-wise, sealing once, an open door with something in it left open, a
doorless room, no danger, the dump, saving, and a generated hoard's items with no one inside), `world/curse_tests.rs` (the cursed room's data and refusals, rising after two
minutes as undead of the dead's level, the twitch and rising lines, not when the corpse is gone or
carried out, not outside, never the undead, the nearest-level fallback and the campaign's own,
Grave warden, a cursed level, a frozen level catching up, saving and the dump; Consecrate lifting a
room and a level, refused elsewhere with no time, asking goodness and mana, an interrupted
prayer).
