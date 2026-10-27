# Summoning

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08b-creatures.md ("Summoning", #520). Keep it current with the code, as CLAUDE.md says.

Summoning is an **enemy-only creature ability**: a creature type with `[creature.summon]` calls
forth creatures of its side. The player has no summoning (the owner, #520), so there is no
ability in the abilities menu, no action, no key, and no talents (#520's "Talents and class
abilities").

- **Data** (`content/summon.rs`): `SummonSpec` as written (`CreatureSpec::summon`, checked by
  `SummonSpec::check` as the type loads), kept on `CreatureDef::summon_spec` until every type is
  known; `Content::load_summons` (one step after `load_throws` in `Content::load`) resolves it
  with `summon::resolve` into `CreatureDef::summon: Option<Summon>`: the pool (ids, or a tag
  within `levels`, uniques and the summoner left out, in creature order; a shared summoner may
  call only shared types, a campaign's its own too), `count` (dice, 1 to `MAX_COUNT` = 12),
  `cooldown` and the degrees of the cooldown status it gives (`ceil(cooldown / eases)`), `mana`,
  `range` (default 8), `spread` (default 2), `most` (default 6), `verb` (default "summons").
  Refused: no pool or both kinds, `tag` without `levels`, out-of-range numbers, an unknown or
  another campaign's creature, an empty pool, a summoner without `mana`, and a missing
  `summoning` status (`summon::STATUS`) or one that ticks or never eases.
- **The cooldown is a status** (`data/status.toml` `summoning`: no tick, eases 6 s a degree, max
  100, refresh, no `seen`): the house rule for ability limits (`ability.rs` module docs). It is
  saved with the actor's `effects`, as every status is; nothing counts uses.
- **State**: `Actor::summoner: Option<ActorId>` (`serde(default, skip_serializing_if =
  "Option::is_none")`): who called it. Old saves load and hash as before; no `FORMAT_VERSION`
  bump. `World::live_summons(id)` counts the actors that name `id`, on any level.
- **Rules** (`world/summon.rs`, one line in `ai.rs::fight`, before throws, shots, and spells,
  and one arm in `creature_turn`): `World::summons_at` (the conditions: a hostile it sees within
  `range`, casting not blocked, the status off, `Actor::mana_cost` of the cost at hand, fewer
  than `most` alive, a free cell), `summon_cells` (the free cells within `spread`, in its line
  of sight (`LineOfSight` to `radius_for(2·spread²)`), walkable, no actor, not `World::shuns`;
  sorted by Chebyshev distance, squared distance, y, x), `creature_summons` (`spend_mana`,
  `afflict` the status, `make_noise(Act::Attack(Spell))`, `count.roll` on `state.rng`, capped
  by `most − live`, one `rng.below(pool.len())` per creature, `travel::make_creature` with
  `rolls_gear` and the summoner's faction as `faction` when it differs from the type's, then
  `summoner` set and `World::spawned` bumped), and `say_summoning` (the words from the last
  turn's `perception.sees_actors_at`). The draws come from the gameplay stream, after the AI
  decided without drawing.
- **Experience**: nothing special: `World::creature_dies` pays a summoned creature's kill like
  any other's (first-kill bonus included).
- **Dumps**: the dev `actors` layer adds `; summoned by #N` to a summoned actor.
- **Wiki**: `xtask/src/wiki/creatures.rs::creature_summons` (`creature-summons.md`, the
  "Summoning" section of `wiki/src/creature-numbers.md`); the tag in `creature-tags.md`.
- **Fixture**: the showcase's summoning hall (`levels/summoning_hall.toml`, scenario
  `summoners`), one summoner to a room behind a door.
- **Tests**: `world/summon_tests.rs` (on a fixture content: the summoned around it on its side,
  the mana paid and the status given, the message; the cooldown; no mana; the cap; Silence;
  walls; the tag pool; the load refusals; the save round trip; determinism; and on the shipped
  content, the goblin wizard in the summoning hall and a summoned kill paying XP),
  `content/summon.rs` (the spec checks), `content/summoner_roster_tests.rs` (every summoning
  goes with the `summoner` tag and another tribe's, and the roster's numbers).

Adding a summoner: give the type `mana`, a `[creature.summon]`, and the tag `summoner` besides
its own tribe's; the roster tests check the rest. The drow priestess (the drow roster) gets her
table in a follow-up.
