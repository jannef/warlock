# Factions and creature behavior

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08b-creatures.md. Keep it current with the code, as CLAUDE.md says.

Factions and creature behavior (spec §7 "Factions", §5 "Creature sight", §8 "Creature behavior"):
`data/factions.toml` (`content/faction.rs`: `Factions`, symmetric hostility, `Stance`; optional,
without it everyone is neutral, but then no creature may name a faction) and per creature type
`faction`, `vision`, `sight`, `combat` (`melee`/`ranged`/`caster`), `range`, `opens_doors`,
`alignment` and `race` (`docs/dev/alignment.md`; read a creature's with `Actor::nature`, never the
type's) (`content/creature.rs`, checked in `content/mod.rs`). Creature types live in `data/creatures/`,
one file per creature named by its id (`kobold.toml`; the field documentation is in its
`README.toml`), a table directory read in path order: see "Content directories" in
`docs/dev/content.md`. Stance comes from types:
`Content::stance(creature, creature)`, `World::stance(actor, actor)`, and for presentation only
`PerceptionView::stance(seen actor)` (look mode's "(hostile)"). Between the player and a creature,
the Beast-friend talent makes `animal` factions neutral until the player attacks one of the type
(`world/beast_friend.rs::befriended`, read by `World::stance`, the AI's `nearest_hostile_in_sight`,
and `PerceptionView::stance`; `docs/dev/progression.md`). The AI is `world/ai.rs`
(look → fight → search the last seen spot → look into the newest unseen noise → back to the post →
patrol), paths `world/path.rs` (A*, fixed tie-breaks), and per-creature memory `Mind` on `Actor`
(saved, hashed). Creature sight is `sight.rs`: the player's rules computed per creature, only for
cells where hostiles stand (nearest first, line of sight only out to that distance, darkvision
before light, bounce light only when needed); it must stay equal to the player's perception —
`light_tests::check_creature_sight` compares `sight::light` with the perceived light on every
visible cell in the light property tests (and under each weather in `weather_tests`). Both take
the sky from `WorldState::sky` (the weather's clouds included) and overworld sight from
`daylight::sight_under` (rain included). A creature's attack is `World::attack` (`docs/dev/combat.md`):
`Event::Attacked` (kept only when the player sees both ends), a noise (`noise::Act::Attack`), and
messages built in `World::perceive_events` (unseen ends are "something"). The app shows a lunge
(`ui/motion.rs`, `motion` switch), a shot glyph, and a creature's `on = "attack"` particle burst
(`ui/effects.rs`, `particles` switch); `Event::Died` has no effect of its own. No creature
behavior may use randomness (combat rolls are the rules', from the gameplay stream); tie-breaks
are by distance, direction order, and actor id.

Danger levels and spawning (spec §7 "Danger levels", #326): the data is `content/danger.rs`
(`DangerRule`, `Spawning`, `weight` and `draw`, the field parsing shared by `data/dungeons.toml`
and level files), reached through `Content::spawning(level id)` (a hand-made level's own, else
its branch's at the depth; `None`: danger 0). The rules are `world/spawn.rs`: `World::danger`
(computed, never stored), `LevelState::spawner` (the next attempt's time and the stream
`spawn/<level id>`, `Rng::spawn`; made with a level that has a danger rule, else absent so other
levels hash as before), `spawn_tick` (one call at the end of `run_turn`: every attempt due on the
current level), `arrive_spawner` and `sense_danger` (one line each in `travel`: frozen floors,
Sixth sense's arrival note), `attempt` (the cap by `World::stance`, the chance, the draw, the
cell out of a freshly computed `Perception`'s sight and `AWAY` cells off, the route by
`mapgen::out_and_back`, then `travel::make_creature`, the one way a creature is made from a
`CreatureSpawn`), and the dev `spawn` action. A spawn bumps `World::spawned`, so a rest perceives
after that turn. Uniques are remembered in `Progress::uniques` by type id, in `make_creature`
(however one came: a spawn, a population, a level file). A spawn sets `rolls_gear`, so
`make_creature` rolls its gear from `gear/<actor id>`. The initial population of a generated
floor is `mapgen/population.rs` (`populate`, one call in `mapgen::generate_for` after
`place_loot` and before `gear::arm`; stream `population/<level id>`, `Rng::population`): the
count by walkable cells, the draw by `danger::draw`, a cell from the free plain-floor cells in
row order (`swap_remove` of a drawn index), a route by `out_and_back`. Whom a level is made for
is `mapgen::Visit` (the luck, the player's level for a scaled danger, the uniques met;
`travel::level_template` builds it; `generate_with` is `generate_for` with only the luck).
`cargo xtask bench-turns` has a `delf` row: a fixture 120×60 cave branch of danger 8 at full
population (`xtask/src/bench.rs`); its night's rest is held to the 500 ms budget thanks to coarse
turns.

Coarse turns while the player rests (#349, spec §8 "Coarse turns while resting"): `world/stride.rs`,
one call at the top of `World::creature_turn` (`coarse_turn` → `Coarse::Strode(price)` returns at
once, `Slow` multiplies the usual action's price by `STRIDE`, `Usual` is the usual turn). `away`
(the player rests, a local level, no dialogue, and not hostile to the player within its sight
radius, nor its mind's `last_seen`/`noise`), `idle` (calm, nothing heard worth a look, no
statuses, able to move, no hostile within its sight radius; returns the hostiles within its
sight plus a stride), `stride_path` (up to `STRIDE` = 20 route steps onto plain cells: walkable,
no door, ice, trap, actor, or shunned cell, not a cell the player's last perception saw creatures
on, not within its step noise's reach of the player, none within its sight of a hostile; prices
and step noise cached per terrain), and a stride's one `make_step_noise` at its end. An idle
creature whose next route cell is taken waits `STRIDE` waits; one whose first step is not plain
takes its turn as usual. Outside a rest nothing here runs. Tests: `world/stride_tests.rs` (pace
kept, busy ones slowed, turn by turn near the player, replays, the rest's sight equals a
perception's); the rest-versus-waiting tests in `time_tests`, `cost_tests`, `hearing_tests`, and
`talk_tests` still hold (their creatures cross into view or earshot turn by turn), and
`speed_tests::resting_accounts_for_fast_creatures` compares a rest with resting turn by turn. Tests: `world/spawn_tests.rs` (the property test of the
design's §9.6, the population of a generated floor and that nothing else moves, uniques in a
population, a spawn's gear stream, frozen floors, the talents, determinism),
`content/danger.rs` (rules, counts, weights, draws).

Creature gear (spec §8 "Creature gear", #328): the data is `content/gear.rs` (`GearLineSpec`
as written, kept on `CreatureDef::gear_spec` until objects load, then `Content::load_gear`
resolves `CreatureDef::gear`: the racial weapon's line first, `Keep` defaults by the slot an
object fits, `kit::check_kept` shared with the starting kits, and the racial-weapon rank check).
The rolls are `gear.rs`: `gear::roll(content, rng, creature, item_level) -> kit::Pack` (the
kit's `Pack::put`/`equip`, `loot::make_as` for a magic piece, `variant::draw` with
`Source::Gear(creature level)` for a plain one, the burden guard against `burden::limit` of the
type's stats and size). Who rolls is `CreatureSpawn::rolls_gear` (true for a legend or vault
entry without `carry`/`wield`/`wear`, `Canvas::add_patrol`, the local areas' deer; false for a
keeper): a generated level clears it in `gear::arm` (stream `gear/<level id>`, after
`place_loot`, in both `mapgen::generate_with` and `mapgen::local::generate`), and
`world/travel.rs::placed_gear` rolls the rest as `instantiate_level` places them (stream
`gear/<actor id>`). The danger levels' paths set `rolls_gear`: the initial population of a
generated floor (rolled by `gear::arm` after it) and a spawn in play (rolled by
`make_creature`). Scavenger is one hook in `World::creature_dies`
(`TalentMods::scavenger`, the kill the player's). Tests: `world/gear_tests.rs`; the mapgen
snapshots print each spawn's gear.
