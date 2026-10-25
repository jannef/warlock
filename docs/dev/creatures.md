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
after that turn. Uniques are remembered in `Progress::uniques` by type id. The initial population
of a generated floor and its `bench-turns` entry are the follow-up (C2), and a spawn's gear comes
with #M's `gear::roll`. Tests: `world/spawn_tests.rs` (the property test of the design's §9.6,
frozen floors, the talents, determinism), `content/danger.rs` (rules, counts, weights, draws).
