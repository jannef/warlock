# Creature subsystem

**Rule: a roster PR adds or edits ONLY its own `docs/dev/creatures/<tribe>.md`; it never edits
this file.** This file is the stable overview of the creature subsystem (factions, the AI,
sight, spawning, gear, incorporeal creatures, coarse turns); every tribe's or wave's roster
notes, per-creature facts, and test-row lists live in the per-tribe files below. The spec's
creature and faction lists are likewise not edited by a roster (`spec/07b-data.md` points at
`assets/data/creatures/` and `assets/data/factions.toml`). A new tribe adds a new file and one
line here only if it truly is a new family, in the same PR as nothing else shares.

Per-tribe notes:
- `docs/dev/creatures/undead.md`: the undead roster (#383; batches, population lines, holds, drains)
- `docs/dev/creatures/campaign-waves.md`: the barrow, fen, and delf waves (the non-undead notes)
- `docs/dev/creatures/yugoloth.md`: the yugoloths (#446; faction, tags, riders)
- `docs/dev/creatures/hags.md`: the hags (#451; faction, tag, riders)
- `docs/dev/creatures/hobgoblins.md`: the hobgoblins (#454; the goblin tribe's second tier)
- `docs/dev/creatures/orcs.md`: the orc tribe and its wargs (#455; faction, tags, formulas, populations)
- `docs/dev/creatures/giants.md`: the giants and ogres (#457; faction, tags, throws)

## Overview

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08b-creatures.md. Keep it current with the code, as CLAUDE.md says.

Factions and creature behavior (spec §7 "Factions", §5 "Creature sight", §8 "Creature behavior"):
`data/factions.toml` (`content/faction.rs`: `Factions`, symmetric hostility, `Stance`; optional,
without it everyone is neutral, but then no creature may name a faction) and per creature type
`faction`, `vision`, `sight`, `combat` (`melee`/`ranged`/`caster`), `range`, `opens_doors`,
`alignment` and `race` (`docs/dev/alignment.md`; read a creature's with `Actor::nature`, never the
type's) (`content/creature.rs`, checked in `content/mod.rs`). Its `tags` (`content/tag.rs`, #441) are plain singular
words for quests and rules (`tag:goblin`, `docs/dev/quests.md` "Creature tags"), never the
faction: **tag every new creature** of a group or role listed in the README (an undead of the
`undead` faction must carry `undead`: `content::tag`'s test). Creature types live in `data/creatures/`,
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

Incorporeal creatures (spec §8 "Incorporeal creatures", #381): the data is a creature type's
`incorporeal = { wall_defence }` (`content/creature.rs::Incorporeal`, checked in `Content::load`
beside `natural`; `Content::incorporeal(type)`). The rules are `world/incorporeal.rs`:
`LevelState::phaseable` (solid terrain, neither walkable nor transparent, off the level's edge:
walls, rock, closed doors), `World::in_wall`, `World::wall_defence` (read by
`World::defence_of`: the Wall category, `combat::Category::Wall`), and `World::landing` (where
a dying creature's things fall: `creature_dies` moves the dead actor's cell there before its
pile and corpse). The hooks, one each: `ai.rs::route` (a solid cell costs `1 + WALL` and may be
a goal), `ai.rs::creature_steps` (through a solid cell, a closed door left shut; no `tread`, no
`step_onto_trap`), `travel.rs::player_moves` (a seen hostile inside a solid cell is struck at,
`World::strike_at`), and `shoot.rs` (a missile reaching an opaque cell attacks whoever stands
inside it). Sight is unchanged: the player's perception already sees a wall cell (and so an
actor in it) by its lit face or darkvision; `sight::sees` sees opaque cells only within
darkvision. The message for a miss the wall turned is `fight::attack_message`'s `Category::Wall`
arm. The example type is `stone_haunt` (placed by no level). Tests:
`world/incorporeal_tests.rs`, `shoot_tests::a_shade_in_a_wall_is_shot_there_...`,
`spawn_tests::an_incorporeal_creature_is_never_placed_inside_a_wall`.

Danger levels and spawning (spec §7 "Danger levels", #326): the data is `content/danger.rs`
(`DangerRule`, `Spawning`, `level_weight`, `weight` and `draw`, the field parsing shared by
`data/dungeons.toml` and level files), reached through `Content::spawning(level id)` (a hand-made level's own, else
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
Rarity (#373): `content/creature_rarity.rs` parses `data/creature_rarities.toml` (tiers in file
order, each a `factor` in percent of a common type's weight, 1 to 1000; `common` required, the
only tier without the file); `CreatureDef::rarity` is an index into `Content::creature_rarities`
(a type's `rarity`, default `common`; an unknown tier is a load error). A draw's candidates are
`danger::Candidate`s `(type, level, factor)`, built by `Content::candidates` (the run's met
uniques left out) for `World::draw_spawn` and `mapgen::population::populate`, and
`danger::weight(level, factor, danger)` is `level_weight × factor`.
Strays and exploding out-of-depth draws (#375): `danger::draw(rng, own, generic, danger, odds)`
rolls, in order, the stray chance (`odds.stray`: the pick goes to `generic`), the out-of-depth
chance, and each further step (`danger::deeper`: `D + 1`, then `odds.deeper` % again and again
one more, at most `odds.deeper_cap`), then one `below(total)` per pick that finds something
(`danger::pick`): at the deeper danger the chosen pool, else the generic list (the own folk all
too weak), then at `D` the chosen pool, else the own folk. `danger::Odds` is
`Spawning::odds`: each of `stray`, `out_of_depth`, `deeper`, `deeper_cap` as a branch or level
gives it, else `data/dungeons.toml`'s `[defaults]` (`danger::DefaultsFile`, read into
`Content::draw_odds` before the levels; a `danger` without it is a load error, and `[defaults]`
in any other dungeons file is refused). The generic creature list is derived, not flagged:
`Content::fill_generic` (after `load_dungeons`) gives every `Spawning::generic` the union of
the populations of the areas with a danger of its campaign and of no campaign, uniques left out
(so people, keepers, hand-placed guardians, quest creatures, and test fixtures' types never
stray); `populate` builds the generic candidates once a floor, the own ones per creature (a
unique leaves them). Hand-placed creatures (level legends, vaults,
scenario people, shopkeepers) are never drawn, whatever their tier.
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
`content/danger.rs` (rules, counts, weights, draws, a rare type drawn less often, out-of-depth
draws by rarity), `content/creature_rarity.rs` (the tier file and its refusals), and
`content::tests::every_drawn_creature_has_a_rarity` (shipped content).

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

## Creature waves

The campaign's creature waves (docs/design/campaign.md §10.1; #333 the barrow wave, #334 the fen
wave, #335 the delf wave): content only, on the fields above. A wave adds
- one file per type: the common ones in `data/creatures/` (shared), the campaign's own people and
  bosses in `campaigns/<id>/data/creatures/` (numbered after the shared block, only that
  campaign's files may name them; `docs/dev/campaign.md`), the Ascmearc's in
  `campaigns/ascmearc/data/creatures/`. Each file starts with a comment naming its wave; `xp` follows
  `5 × level + 10`; a humanoid has `racial_weapon` with a rank in its skill and `[[creature.gear]]`
  lines (a second weapon of another skill wants a rank there too, so it benefits);
- its factions in `data/factions.toml`, inserted by group (after the kobolds), never at the end.
  Four test fixtures write their own neutral factions file over the shipped one
  (`world/{equip,magic,object,read}_tests.rs`): every shipped faction id must be listed there
  too, or the shipped types naming it fail to load;
- its rows in `world/bestiary_tests.rs`'s `WAVES` (the type and its owning campaign), and its
  stances in that file's faction test.
