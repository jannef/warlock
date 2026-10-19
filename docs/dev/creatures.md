# Factions and creature behavior

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08b-creatures.md. Keep it current with the code, as CLAUDE.md says.

Factions and creature behavior (spec §7 "Factions", §5 "Creature sight", §8 "Creature behavior"):
`data/factions.toml` (`content/faction.rs`: `Factions`, symmetric hostility, `Stance`; optional,
without it everyone is neutral, but then no creature may name a faction) and per creature type
`faction`, `vision`, `sight`, `combat` (`melee`/`ranged`/`caster`), `range`, `opens_doors`
(`content/creature.rs`, checked in `content/mod.rs`). Stance comes from types:
`Content::stance(creature, creature)`, `World::stance(actor, actor)`, and for presentation only
`PerceptionView::stance(seen actor)` (look mode's "(hostile)"). The AI is `world/ai.rs`
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
