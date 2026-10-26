# Spells

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08e-blood-and-status.md ("Spells") and the approved magic design `docs/design/magic.md`.
Keep it current with the code, as CLAUDE.md says.

The spell system is built in waves (design §13). This file covers what is built: the **data
foundation** (#394): the tier table, the spell files with their load-time cap check, the tags,
and caster level. Nothing casts these spells yet: #395 brings the one cast path (cost,
resolution, dice) and migrates the casters' placeholder spells (`content/class_spell.rs`,
`world/caster.rs`, docs/dev/progression.md "Caster spells"), which keep working until then.
Not to be confused with `content/spell.rs`, the bursts and bolts scrolls and traps cast.

- **Data**:
  - `assets/data/spells.toml` (`SpellRules`, `content/spell_def.rs`): `[tiers]` arrays of ten
    values, tier 0 first, `comfortable` (caster levels, ascending from 1), `cost` (base mana,
    ascending), `ceiling` (single-target damage at the highest roll, ascending); `overreach_cost`
    (percent per tier); `[shapes]` (cost percents: `ray`, `bolt`, `touch`, `self`, `cone`,
    `line`, and `burst` a list by radius whose length is the largest radius, ascending, at most
    `BURST_RADII`'s end); `[resolution]` (`attack`, `auto`, `area`, `touch`, each
    `{ damage, cost }` in percent). Every check refuses with a `ContentError`.
  - `assets/data/spells/` (`SpellData`), one `[[spell]]` file each, the format in its
    `README.toml` (a comment-only file, like the classes'). Read by `ContentFiles::table("spells")`
    minus `data/spells.toml` itself, so campaigns may add spells; a spell without the tier table
    is an error. Ids are `SpellId` indices in path order and enter the content fingerprint (saves
    will name them, #396); a spell file changes saves.
  - `casting` on classes (`ClassDef::casting`, 1 to 100, default 25, `content::CASTING`).
  - `caster_level`: the talent effect `TalentEffect::CasterLevel { value }` (a bonus, ±20; summed
    into `TalentMods::caster_level`) and the affix effect `Effect::CasterLevel` (1 to 10, after
    `ManaRegeneration` in `Effect::all`). Nothing grants either yet (#400's talents).
  - `mana_level = 60` in `data/experience.toml` (tier 9's comfortable caster level), and the
    damage types `force` and `necrotic` at the end of `data/damage_types.toml` (non-physical:
    `Choice::Element` offers them, so Elemental ward may be taken for them).
- **The cap** (`spell_def::dice`, for `[spell.damage]` and a heal's `[spell.effect] heal`): the
  dice are dice only (`1d6`, no bonus in the text, not negative), `max_dice` from the dice's own
  count to `MAX_DICE` (20), `per_levels` 0 to 20, `bonus` at most `SpellRules::max_bonus` (a
  quarter of the ceiling, rounded down), and `SpellDice::max` (`max_dice` × sides + bonus) at most
  `SpellRules::cap(tier, resolution)` (the ceiling × the resolution's `damage` percent, rounded
  down). The error names the spell, what it may roll, and the cap.
- **Other checks** (`spell_def::resolve`): tier 0 to 9; `Resolution::fits(shape)` (`area` exactly
  for the area shapes, `touch` only for touch, `attack` for ray and bolt, `auto` for any single
  target); the reach caps each shape has exactly (`max_range` 1 to `MAX_REACH` for ray, bolt,
  burst; `max_radius` 1 to the burst table's length; `max_length` 1 to `MAX_CONE` for a cone, to
  `MAX_REACH` for a line); a `damage` spell has `damage` and a `damage_type` and no `effect`;
  `[spell.effect]` holds only its kind's fields (heal: `heal`; buff and hex: `status`,
  `severity` 1 to `MAX_SEVERITY`; cleanse: `cures`, `diseases`), the statuses known;
  `[spell.scroll] level` 1 to `MAX_LEVEL`, default the tier's comfortable level.
- **Pure helpers** for the waves to come: `SpellRules::tier_of(caster_level)` (the highest tier
  reached; 0 below the first), `shape_cost(shape, radius)`, `ratio`, `cap`, `max_bonus`;
  `SpellDice::at(caster_level, comfortable)` (the dice at a caster level: one more per
  `per_levels` above the comfortable level, never more than `max_dice`); `SpellData::tags`.
- **Caster level** (`world/caster_level.rs`): `Actor::caster_level(content, level, casting)`
  (`level × casting ÷ 100` rounded down, plus `TalentMods::caster_level` and the worn
  `Effect::CasterLevel`, at least 0, at least 1 while `max_mana` is above 0),
  `World::caster_level(id)` (the player at `player_level` and `casting_percent`: the class's,
  100 without a class; a creature at its type's level and 100), `casting_percent(content,
  character)`. Derived, never stored: nothing in saves or the state hash changes.
- **Presentation**: `PerceptionView::own_caster_level()`; the dump `stats` layer's
  `Caster level N` after the scores, JSON dumps' `caster_level`. The character screen does not
  show it yet.
- **Talents**: the `spells` system in `data/talents.toml`, `pending = true` until #400 ships
  the Spellcraft tree.
- **Wiki**: `xtask/src/wiki/spells.rs` (`spell-tiers.md`, `spell-shapes.md`,
  `spell-resolutions.md`, `spell-casting.md`) for `wiki/src/magic.md`.
- **Tests**: `content/spell_def.rs` (the shipped table, `tier_of`, the refused tables, the
  design's first spell list passing the cap, a spell over its cap, a bonus too large, malformed
  spells, the dice's growth), `content/class.rs` (`casting`), `content/mod.rs` (spells in the
  fingerprint), `content/experience.rs` (`mana_level` 60), `world/caster_level_tests.rs` (by
  class and level, no mana, talents and a ring, a creature, the dumps, mana to level 60).
