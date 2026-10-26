# Spells

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08e-blood-and-status.md ("Spells") and the approved magic design `docs/design/magic.md`.
Keep it current with the code, as CLAUDE.md says.

The spell system is built in waves (design §13). This file covers what is built: the **data
foundation** (#394: the tier table, the spell files with their load-time cap check, the tags,
caster level), the **one cast path** (#395: cost, reach, dice, resolution, the shapes, and
the casters' placeholder spells migrated onto it), and **learning** (#396: the learned set,
study, the check, mishaps; "Learning" below). Not to be confused
with `content/spell.rs`, the bursts and bolts scrolls and traps cast, whose code the cast path
shares.

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
  `MAX_REACH` for a line); its base reach (`range`, `radius`, `length`: only where its cap
  is, 1 to the cap, default the cap); `casts` 1 to `MAX_CASTS`; a `damage` spell has `damage`
  and a `damage_type`; `[spell.effect]` holds only its kind's fields (heal: `heal`; buff, hex,
  and damage: `status`, `severity` 1 to `MAX_SEVERITY`; cleanse: `cures`, `diseases`), the
  statuses known; `[spell.cast]` (`SpellLook`): `effect` only for a shape cast at a distance
  (none: nothing drawn), `noun` (default the name in lower case) and `verb` (default the damage
  type's) not empty, a `noise` loudness 1 to `MAX_LOUDNESS`, `blast`/`heat`/`frost` only for a
  burst and checked by `Burst::new`; its effect against `data/effects.toml` once effects load
  (`spell_def::check_effects`: burst and cone a burst's, ray, bolt, line a bolt's);
  `[spell.scroll] level` 1 to `MAX_LEVEL`, default the tier's comfortable level.
- **Pure helpers**: `SpellRules::tier_of(caster_level)` (the highest tier reached; 0 below the
  first), `shape_cost(shape, radius)`, `ratio`, `cap`, `cap_of(spell)`, `max_bonus`,
  `mana(spell, caster_level, armour)` (the cost, below), `grown(base, cap, modifier)` (Willpower,
  `reach_step`), `lengthened(severity, modifier)` (Presence, `presence_step`);
  `SpellDice::at(caster_level, comfortable)` (the dice at a caster level: one more per
  `per_levels` above the comfortable level, never more than `max_dice`); `SpellData::tags`,
  `aimed`, `base_reach`.
- **The one cast path** (`world/cast.rs`, `World::cast_spell` from `Action::Cast`; decisions the
  design left open are under "Choices" below):
  - **Cost**, the one hook `Actor::spell_cost(content, spell, caster_level)`:
    `SpellRules::mana` (the tier's `cost` × (100 + `overreach_cost` × tiers above
    `tier_of(caster_level)`) × `shape_cost(shape, its own radius)` × the resolution's `cost` ×
    (100 + `Actor::armour_spell_cost`, the summed `spell_cost_percent` of what is worn) / 100⁴,
    rounded to the nearest once, at least 1), then Archmage's `TalentMods::spell_mana`
    (`Actor::spell_base`, what `World::spend_mana` is given), then `Actor::mana_cost` (Frugal
    caster) inside `spend_mana`. `ability::spell_unavailable(actor, content, spell, level)` is
    `Unavailable::NoMana` against it; `Ability::unavailable` alone cannot know the caster level,
    so `PerceptionView::abilities` and the cast ask the former. Mana is spent only once the cast
    is sure to go off.
  - **Reach**: `Actor::spell_reach` → `Reach { range, radius }`: base grown by the effective
    Willpower modifier (`SpellRules::grown`) up to the caps; a touch's range 1. A burst still
    costs by its own `radius`. Presence lengthens a buff's and a hex's severity
    (`SpellRules::lengthened`; the status's `max` clamps it in `Effects::add`; Blessed, Hexed
    `ill_wished`, Weakened, and Sluggish have `max = 3` for it).
  - **Dice**: `SpellDice::at(caster_level, comfortable)` then `TalentMods::spell_damage` (Scroll
    adept's and Archmage's `per_die`, Empower's `dice`); the roll is held to
    `SpellRules::cap_of` (`Burst::cap`/`Bolt::cap`, `spell::capped`; `None` for scrolls, traps,
    and gifts, which keep rolling as before). A heal rolls its dice with no talents, held the
    same way.
  - **Kinds**: buff (its status on the caster), heal (self: the caster, refused when whole;
    touch: the creature at the adjacent cell, or the caster), cleanse (as the placeholder's),
    hex (the hostile the player sees at the cell, `hex_target`), utility (nothing yet), damage
    (by shape: `Shape::Burst` → `burst_center` + `cast_burst`; `Line` → `bolt_line` +
    `cast_bolt`; `Cone` → `cast_cone`; `Ray`/`Bolt`/`Touch` → `cast_single`). A damage spell's
    `effect.status` falls on everyone it hit who lives. `Cause::Spell(ClassSpell)` for a death
    by one's own spell.
  - **Resolution**: area through `World::spell_hurts` (Evasion via `World::evaded`, then
    `World::spell_strikes`); auto through `spell_strikes` with a plain hit; attack and touch roll
    `combat::strike` with `Profile::bare(Attack { damage: dice, to_hit: Dexterity modifier,
    threat 22, crit ×2 })` against `combat::Target::bare(target_against(target, missile).defence)`
    (no Protection; the strike's damage, crit included, is then held to the cap and resisted in
    `spell_strikes`). `spell_strikes` is the one place a spell's blow lands: the event (a miss
    too), provoking, the resistance, blood, the hurt, the wound.
  - **Geometry**: `spell::cone_cells(from, toward, length, opaque)` (exported as
    `warlock_core::world::cone_cells` for the aiming mode): cells within the rounded radius, in
    line of sight from `from`, not opaque, `dot > 0` and `2·dot² ≥ |v|²·|d|²` (45° either side),
    never `from`; drawn as `SpellShape::Burst { center: caster, radius: length }` over those
    cells (the renderer draws only an event's cells). A ray or a bolt is `bolt_line` cut at the
    first actor, drawn as `SpellShape::Bolt`. A spell without an `effect` pushes no
    `Event::Spell` (`cast_burst` and `cast_bolt` skip an empty one).
  - **Determinism**: draws, only when they arise: the attack roll (two dice, a why-missed draw,
    a confirmation) and the damage dice for a single target; one roll for an area, its targets
    in ascending actor id. A buff, hex, or cleanse draws nothing.
  - **Saves**: nothing new (the twelve abilities are `ClassSpell` as before; costs and
    reaches are derived). Old replays that cast verify only if the spell's cost is the same as
    the placeholder's; the regenerated costs change the state hash of any that cast.
- **Choices** (design §14 and the issue's open points, decided here):
  - The attack roll's skill: there is no Missile skill (weapon skills are per weapon), so rays,
    bolts, and touches aim with the **Dexterity** modifier, threat and critical at the blows'
    defaults; Spellcraft has no benefit yet (#400 may give it one).
  - A critical rolls the damage again, but the result is still **held to the cap**: nothing
    passes it, criticals included (design §5's strict cap).
  - Empower and Archmage are not changed in data: their dice and per-die bonus apply, and the
    roll is clamped to the spell's tier cap, so they reach the cap sooner and never pass it.
    Scrolls keep them unclamped (scrolls are not tiered yet).
  - Willpower: a cell per 2 points of a positive modifier (`reach_step`); Presence: a degree
    per 2 (`presence_step`), on statuses whose degrees are their duration.
  - A ray/bolt flies to the first creature on its line; a hex (`ray`, `auto`) keeps the
    placeholder's rule, the creature seen at the cell aimed at.
  - The migrated tiers: the lowest tier whose cap holds the placeholder's dice (damage
    spells), else one near the placeholder's mana, never above the tier the class reaches when
    it grants the spell (so none costs overreach when granted). Costs and the data, old in
    brackets: Blessing 0 (1; 2), Cure wounds 2 (6; 4, now 2d6 for 1d8 + level/2), Remove
    disease 3 (11; 10), Call lightning 4 (29; 8, a line), Flame 3 (26; 10), Frost 2 (14; 10),
    Arcane bolt 2 (11; 3, a line of force), Fire burst 3 (26; 9), Evil eye 1 (3; 2), Hex of
    weakness 1 (3; 3), Curse of sloth 1 (3; 4), Blight 2 (14; 8). Armour: chain 10, scale 20,
    banded 30, plate 40, great helm 10, large shield 10, tower shield 20.
- **Caster level** (`world/caster_level.rs`): `Actor::caster_level(content, level, casting)`
  (`level × casting ÷ 100` rounded down, plus `TalentMods::caster_level` and the worn
  `Effect::CasterLevel`, at least 0, at least 1 while `max_mana` is above 0),
  `World::caster_level(id)` (the player at `player_level` and `casting_percent`: the class's,
  100 without a class; a creature at its type's level and 100), `casting_percent(content,
  character)`. Derived, never stored: nothing in saves or the state hash changes.
- **Presentation**: `PerceptionView::own_caster_level()`; the dump `stats` layer's
  `Caster level N` after the scores, JSON dumps' `caster_level`; `own_spell_cost(spell)` and
  `own_spell_reach(spell)` (the menu's "tier 2, 11 mana", the aiming mode's reach). The
  character screen does not show caster level yet.
- **UI**: the abilities menu's **Spells** group (`Source::Spell`) with tier and cost
  (`ui/abilities_screen.rs`, snapshot `abilities-menu-spells`); the aiming mode by shape
  (`ui/target.rs`: `cast_spell`, `Aim::spell_line`; a cone's dots are its cells).
- **Learning** (#396; `world/learn.rs`, `content/learning.rs`, tests `world/learn_tests.rs`
  and `content/learning.rs`):
  - **Known spells**: `Actor::spells` (`BTreeSet<ClassSpell>`, saved by id, left out while
    empty, so actors without hash as before; saves change) holds the learned ones;
    `Actor::knows_spell` is that or a class ability's `spell` effect (`TalentMods::has_spell`);
    `Actor::known_spells` lists both in menu order; `Ability::Spell(s).had_by` asks
    `knows_spell`. `Content::spell_of(ClassSpell)` is a spell's data whoever has it;
    `Content::class_spell` stays "a class ability gives it". The spells stay the twelve
    `ClassSpell`s (#397 brings more and may generalise the enum to `SpellId`).
  - **Data**: `data/study.toml` (`StudyRules`: `base`, `per_overreach`, `known_per_point`,
    `turns_per_tier`, `bad_from`, `terrible_from`, and the words' thresholds `easy`,
    `struggle` in ways of 144); `data/mishaps/` (`Mishap`, one `[[mishap]]` a file, its
    `README.toml` the format: `band` slight/bad/terrible/lucky, `weight`, `message`, `effect`
    by `kind`: nothing, status, backfire, mana, teleport, lose_mark, drain, summon, learn, mark;
    only its kind's fields; `learn` and `mark` only lucky); `teaches` in a spell scroll's `read`
    table (`ScrollSpell::teaches`, a `ClassSpell` with a spell file, checked as objects load);
    a class's `affinity` (`ClassDef::affinity`, school → −10 to 10 study points); the skills'
    `study` benefit (`skills::Effect::Study`, Spellcraft and Decipher script, a curve to 20 at
    200) and `study` mark (`Use::Study`, which is also `StatUse::Study`: Intelligence 100 %);
    `ActionKind::Study` (`study` in `data/actions.toml`, a turn); Raging blocks `study`.
  - **The action** (`Action::Study { item }`, `study:b`; `World::start_study` →
    `Spent::Study(Lesson)` → `World::study`, through `keep_at` like a meal): refusals as the
    spec lists them; `study_turns(rules, tier, percent)` turns (Quick study's
    `TalentMods::study_percent`); interrupted (`Ended::Came`/`Said`): "…you stop studying.",
    time lost, scroll kept (design §14 q1, the default). Done: the scroll is found again by
    object (positions may shift), the `study` use is trained, the odds are taken now, the
    scroll used up (`use_scroll`, never Spell recall: §14 q7, decided no), then the check.
  - **Odds** (`study_odds(actor, content, character, caster_level, spell) -> Odds`, pure;
    `World::player_study_odds`): overreach = tier − `SpellRules::tier_of(caster level)`; at 0
    or less `check: None` (certain, no draw); else `(bonus, difficulty)`: Intelligence
    modifier + `skill_effect(Effect::Study)` + the class's affinity with the school +
    `TalentMods::study_bonus`, against `StudyRules::difficulty(overreach, known tiers)`;
    `Odds::ways()` of 144 (`check::ways`). `StudyRules::words` turns ways into "certain",
    "easy", "a struggle", "far beyond you".
  - **Mishaps** (`World::draw_mishap`, `World::mishap`): one `rng.below(total weight)` among
    `drawn_among(mishaps, band)` (the band's and the lucky ones, in path order); the message,
    then the effect through existing code: `afflict`; backfire (the scroll's own `Cast`:
    `cast_burst` centred on the reader, or `cast_bolt` on a path of their own cell, the roll held
    to the taught spell's `cap_of`; `say_spell`;
    a death is `Cause::Scroll`); mana cut directly (no Frugal caster, no training); the teleport
    trap's `teleport` (made `pub(super)`); a stat mark off one drawn among the marked stats; a
    `Drain` (Willpower check); `travel::make_creature` at the first free cell around the reader
    (faction its type's, `spawned` bumped); `learn_spell`; a stat mark. Choices: the design's
    "learned anyway, one tier lower in power" is learned as is (no per-spell power yet);
    "a wild surge casts another spell" is not a mishap yet.
  - **Presentation**: `PerceptionView::own_spells`, `studies()` (`OwnStudy { item, spell,
    chance }`: identified scrolls teaching an unknown spell, pack order); the dump `spells`
    layer and JSON `spells`/`studies`; the dev `progress` layer's `study c frost: overreach 2,
    +0 against 20, 15/144`. The study list (`ui/object_screen.rs`, `ObjectList::Study`, the
    `Study` key command `L`) and its dialog; snapshots `scroll-study-list`,
    `scroll-study-dialog`, `scroll-study-none`.
  - **Wiki**: `xtask/src/wiki/learning.rs` (`study-chances.md`, `study-times.md`,
    `study-affinities.md`, `mishaps.md`) on `wiki/src/magic.md`'s "Learning spells from
    scrolls".
  - **Talents**: Quick study (`quick_study`, the new Spellcraft tree's first talent:
    `TalentEffect::Study { bonus, percent }`, +2 and −50 %), systems `spells` and `scrolls`.
- **Talents**: the `spells` system in `data/talents.toml`, `pending = true` until #400 ships
  the rest of the Spellcraft tree (Quick study, #396, is its first). The cast path extends `mana` and `scrolls`: Frugal caster, Archmage,
  Empower, Scroll adept act on it as above.
- **Wiki**: `xtask/src/wiki/spells.rs` (`spell-tiers.md`, `spell-shapes.md`,
  `spell-resolutions.md`, `spell-casting.md`, `spell-list.md`, `spell-armour.md`; `does` is
  shared with the talents' table) for `wiki/src/magic.md`.
- **Tests**: `content/spell_def.rs` (the shipped table, `tier_of`, the refused tables, the
  design's first spell list passing the cap, a spell over its cap, a bonus too large, malformed
  spells, the dice's growth, the cost table, Willpower and Presence, the shipped spells against
  the placeholders), `content/class.rs` (`casting`), `content/mod.rs` (spells in the
  fingerprint), `content/experience.rs` (`mana_level` 60), `world/caster_level_tests.rs` (by
  class and level, no mana, talents and a ring, a creature, the dumps, mana to level 60),
  `world/cast_tests.rs` (an attack can miss and critical under the cap, auto and area never,
  Empower to the cap, Evasion halving an area only, line against bolt, the cone, touch, armour,
  Willpower's reach, no draws without a roll; test spells replace Arcane bolt's file),
  `world/caster_tests.rs` (the twelve migrated spells), `world/spell.rs` (cone geometry),
  `ui/target.rs` (aiming by shape).
