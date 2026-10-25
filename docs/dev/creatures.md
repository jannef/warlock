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
`gear/<actor id>`). A spawn path added later (the danger levels' population and spawns) sets
`rolls_gear` or calls `gear::roll` itself. Scavenger is one hook in `World::creature_dies`
(`TalentMods::scavenger`, the kill the player's). Tests: `world/gear_tests.rs`; the mapgen
snapshots print each spawn's gear.
