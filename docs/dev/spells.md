# Spells

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08e-blood-and-status.md ("Spells") and the approved magic design `docs/design/magic.md`.
Keep it current with the code, as CLAUDE.md says.

The spell system is built in waves (design §13). This file covers what is built: the **data
foundation** (#394: the tier table, the spell files with their load-time cap check, the tags,
caster level), the **one cast path** (#395: cost, reach, dice, resolution, the shapes, and
the casters' placeholder spells migrated onto it), **learning** (#396: the learned set,
study, the check, mishaps; "Learning" below), the **first spell list** (#397: the spells,
their scrolls, the classes' guaranteed spells; "The first list" below), **wands** (#398;
"Wands" below), **rods** (#399; "Rods" below), the **Spellcraft talents** (#400; "Talents"
below), and the cross-checking pass (#401; "Balance" below). Not to be confused
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
    name them: `Actor::spells`); a spell file changes saves.
  - `casting` on classes (`ClassDef::casting`, 1 to 100, default 25, `content::CASTING`).
  - `caster_level`: the talent effect `TalentEffect::CasterLevel { value, per_levels }` (a bonus,
    ±20; flat into `TalentMods::caster_level`, or with `per_levels` 1 to 20 that much per that
    many character levels, `TalentMods::caster_level_at`: the wizard's Arcane training, +1 per
    5, #400) and the affix effect `Effect::CasterLevel` (1 to 10, after `ManaRegeneration` in
    `Effect::all`; no affix file grants it yet).
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
  `[spell.scroll] level` 1 to `MAX_LEVEL`, default the tier's comfortable level;
  `[spell.scales]` (`content/spell_scales.rs`, `SpellScales`; #556): `damage`, `to_hit`,
  `reach`, `duration`, each `{ stat = weight }` with known stats and weights 1 to
  `MAX_SCALE_WEIGHT` (200), only the dimensions the spell has (`damage`: a damage or heal roll;
  `to_hit`: `attack` or `touch`; `reach`: not touch or self; `duration`: buff or hex), the
  defaults where it is silent (`SpellScales::default`: reach Willpower 50, duration Presence 50,
  to-hit Dexterity 100, no damage). `[tiers] max_lift` (one per tier, 0 to `MAX_LIFT`, ascending
  or flat) replaces the retired `reach_step` and `presence_step`.
- **Pure helpers**: `SpellRules::tier_of(caster_level)` (the highest tier reached; 0 below the
  first), `shape_cost(shape, radius)`, `ratio`, `cap`, `cap_of(spell)`, `max_bonus`,
  `mana(spell, caster_level, armour)` (the cost, below), `grown(base, cap, more)` and
  `lengthened(severity, more)` (cells and degrees the scales give), `lift(tier, raised, more)`
  (the damage scales' sum clamped to 0 and the tier's `max_lift` plus Keen mind's),
  `lifted_cap(spell, lift)` (`cap_of × (100 + lift) / 100`, rounded down);
  `SpellDice::at(caster_level, comfortable)` (the dice at a caster level: one more per
  `per_levels` above the comfortable level, never more than `max_dice`) and
  `SpellDice::lifted(caster_level, comfortable, (cap, lifted))` (as many dice more as
  `(lifted − bonus) / sides − (cap − bonus) / sides`; equal caps change nothing);
  `Weights::sum` (Σ weight × modifier), `steps` (never below 0, over 100, a half down: the old
  floor for the 50 % defaults), `signed` (over 100, a half toward zero: a to-hit);
  `SpellData::tags`, `aimed`, `base_reach`.
- **The caster's stats** (`world/spell_scale.rs`, derived at cast time from
  `Actor::effective_stats`, nothing saved): `Actor::spell_lift` (0 without `damage` scales;
  `TalentMods::focus_lift`, Keen mind, adds to the tier's most), `spell_cap`, `spell_to_hit`,
  `spell_reach_more`, `spell_degrees_more`. What is worn joins them there (#629, global
  effects, unidentified too; `world/affix_spell_tests.rs`): `spell_lift` (Potent) beside Keen
  mind's in the tier's most, `spell_to_hit` (of the Magus) in `spell_to_hit`, `spell_duration`
  (of Lingering) in `spell_degrees_more`, and `spell_threat` (Baleful) in `Actor::spell_threat`:
  22 less it, never below `combat::THREATS`' floor.
- **The one cast path** (`world/cast.rs`, `World::cast_spell` from `Action::Cast`; decisions the
  design left open are under "Choices" below):
  - **Cost**, the one hook `Actor::spell_cost(content, spell, caster_level)`:
    `SpellRules::mana` (the tier's `cost` × (100 + `overreach_cost` × tiers above
    `tier_of(caster_level)`) × `shape_cost(shape, its own radius)` × the resolution's `cost` ×
    (100 + `Actor::armour_spell_cost`, the summed `spell_cost_percent` of what is worn, each
    piece's less its own Mage-wrought `spell_ease`, never below 0: `Actor::worn_spell_cost`) / 100⁴,
    rounded to the nearest once, at least 1; `SpellRules::mana_with` scales `overreach_cost` by
    Overreach's `TalentMods::overreach_cost_percent`, −33), then `TalentMods::spell_mana_with`:
    `spell_mana_percent` (Archmage −50, Empower −10) plus School focus's cost for a spell of
    its tag (`TalentMods::focus(spell.tags)`), rounded down, at least 1 (`Actor::spell_base`,
    what `World::spend_mana` is given), then `Actor::mana_cost` (Frugal caster and the worn
    `mana_cost`, of Thrift, summed, at most 50 % off) inside
    `spend_mana`. `ability::spell_unavailable(actor, content, spell, level)` is
    `Unavailable::NoMana` against it; `Ability::unavailable` alone cannot know the caster level,
    so `PerceptionView::abilities` and the cast ask the former. Mana is spent only once the cast
    is sure to go off.
  - **Reach**: `Actor::spell_reach` → `Reach { range, radius }`: base grown by the spell's
    `reach` scales (`Actor::spell_reach_more`, `SpellRules::grown`), then School focus's reach
    and the worn `spell_reach` (of Farcasting) on the range (a cone's or
    a line's length too, never the radius), up to the caps; a touch's range 1. A burst still
    costs by its own `radius`. The `duration` scales lengthen a buff's and a hex's severity
    (`Actor::spell_degrees_more`, `SpellRules::lengthened`; the status's `max` clamps it in `Effects::add`; Blessed, Hexed
    `ill_wished`, Weakened, and Sluggish have `max = 3` for it).
  - **Dice**: `SpellDice::lifted(caster_level, comfortable, Cast::cap)` then
    `TalentMods::spell_damage` (Scroll adept's and Archmage's `per_die`, Empower's `dice`); the
    roll is held to the lifted cap, `Cast::cap.1` (`Actor::spell_cap` for a cast paid in mana;
    a scroll read or a wand used: `SpellRules::cap_of`, unlifted; `Burst::cap`/`Bolt::cap`,
    `spell::capped`; `None` for the old scrolls, traps, and gifts, which keep rolling as
    before). A heal rolls its dice with no talents, held the same way.
  - **Kinds**: buff (its status on the caster), heal (self: the caster, refused when whole;
    touch: the creature at the adjacent cell, or the caster), cleanse (as the placeholder's),
    hex (the hostile the player sees at the cell, `hex_target`), utility (Identify, below;
    any other: nothing yet), damage
    (by shape: `Shape::Burst` → `burst_center` + `cast_burst`; `Line` → `bolt_line` +
    `cast_bolt`; `Cone` → `cast_cone`; `Ray`/`Bolt`/`Touch` → `cast_single`). A damage spell's
    `effect.status` falls on everyone it hit who lives. `Cause::Spell(SpellId)` for a death
    by one's own spell.
  - **Resolution**: area through `World::spell_hurts` (Evasion via `World::evaded`, then
    `World::spell_strikes`); auto through `spell_strikes` with a plain hit; attack and touch roll
    `combat::strike` with `Profile::bare(Attack { damage: dice, to_hit: the spell's `to_hit`
    scales for an own cast (`Actor::spell_to_hit`), the Dexterity modifier for a scroll or a wand,
    threat `Actor::spell_threat` for an own cast (22 less Baleful's, carried by
    `reflect::Shot::threat`, a reflection keeping it) and 22 for a scroll or a wand, crit ×2
    }).with(benefit)` against
    `combat::Target::bare(target_against(target, missile).defence)` (no Protection; the strike's
    damage, crit included, is then held to the cap and resisted in `spell_strikes`). The
    `benefit` (#571) is the Spell attacks skill's and the caster style's (`Actor::spell_skills`:
    Battle caster in armour that hinders casting, else Toucher for a touch, Sniper for a ray or
    a bolt; `Benefit::at_distance` adds Sniper's far benefits from `[spell_attacks] far`), for a
    cast paid in mana of a spell that `rolls_to_hit` only; then `World::spell_practised` earns
    their marks and fires their triggered abilities (`resolve_triggers`). A scroll or a wand
    keeps the bare roll (docs/dev/weapon-skills.md "Caster styles"). `spell_strikes` is the one
    place a spell's blow lands: the event (a miss too), provoking, the resistance, blood, the
    hurt, the wound. `cast_single` settles the caster's side once (`reflect::Shot`: dice, cap,
    to-hit, effect, spell skills) and strikes through `spell_attack(by, target, source, shot)`, the one place a
    single target's roll is made.
  - **Spell reflection** (#566, `world/reflect.rs`): an attack or touch roll whose miss
    `Defence::why_missed` put down to the target's **Shield** raises the `spell_blocked`
    trigger for the target (`World::spell_blocked`, through `weapon_trigger`). A
    `reflect_spell` effect (Spell reflection, Sword and Board 10) marks the reflector while its
    `per_turn` count allows (`Reflections`, transient, cleared in `spend_actions`), and
    `cast_damage` then casts the same `Shot` back (`World::turn_back`): said first, from the
    reflector's cell back along the cells the spell flew through (they were empty), its own
    attack roll against the caster's Defence, the caster's dice, lifted cap, and to-hit, the
    spell's status rider on a hit (`spell_lands`, by the reflector); no mana again, nothing for
    the reflector, and no one practises it (the `Shot` goes back without spell skills: no
    benefit, marks, or triggers). The reflected flight never raises the trigger (the mark): no ping-pong. A
    death by it is put down to the spell, as a death by one's own spell is. Creature spells are
    still intents, so only the player's spells (cast, read, or zapped) are reflected today.
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
  - **Saves**: nothing new in #395 (costs and reaches are derived); a spell is a `SpellId`
    (an index, in the content fingerprint) since #397. Old replays that cast verify only if the spell's cost is the same as
    the placeholder's; the regenerated costs change the state hash of any that cast.
- **Choices** (design §14 and the issue's open points, decided here):
  - The attack roll's skill: rays, bolts, and touches aim with their `to_hit` scales (Dexterity
    by default), threat and critical at the blows' defaults, plus since #571 the Spell attacks
    skill and a caster style (a weapon skill of role `spell` and styles of the caster grips);
    Spellcraft does not aim spells (its benefit is the study check's, #396).
  - A critical rolls the damage again, but the result is still **held to the cap**: nothing
    passes it, criticals included (design §5's strict cap).
  - Empower and Archmage (reworked in #400): their dice and per-die bonus apply, and the roll
    is clamped to the spell's tier cap, so they reach the cap sooner and never pass it; both
    lower costs (Archmage −50 %, Empower −10 %). A spell scroll read with talents that change
    its dice (`read_spell`) is held to `max(cap of the spell it teaches, the scroll's own most)`:
    the talents lift it no farther, and a scroll that rolls more of itself (lightning's 4d6
    against Lightning bolt's 19) keeps its own roll.
  - Willpower: a cell per 2 points of a positive modifier; Presence: a degree per 2, on
    statuses whose degrees are their duration: since #556 the defaults of `[spell.scales]`.
  - The lift (#556, combat progression design §7): the dice the lift adds are its room
    (`lifted / sides − cap / sides`), not `floor(lifted cap / sides)` outright, so a spell
    written under its cap and every lift of 0 roll as before; the design's worked numbers hold
    (Scorching ray 3d6 → 4d6 at 50 %, Fireball 3d6 → 5d6 at 70 %). The lift and the to-hit
    scales are the caster's own spells' (paid in mana); a scroll read and a wand used keep the
    unlifted cap and Dexterity (reach and duration stay the reader's, as before). To-hit may be
    negative, as Dexterity's was. The first authoring pass: evocation damage spells but the
    cantrips Intelligence 10 and Willpower 5; the hexes Presence 50 and Willpower 25; Cure
    wounds Willpower 15; Mend, a cantrip, and the necromancy spells none yet.
  - A ray/bolt flies to the first creature on its line; a hex (`ray`, `auto`) keeps the
    placeholder's rule, the creature seen at the cell aimed at.
  - The migrated tiers: the lowest tier whose cap holds the placeholder's dice (damage
    spells), else one near the placeholder's mana, never above the tier the class reaches when
    it grants the spell (so none costs overreach when granted). Costs and the data, old in
    brackets: Blessing 0 (1; 2), Cure wounds 2 (6; 4, now 2d6 for 1d8 + level/2), Remove
    disease 3 (11; 10), Call lightning 4 (29; 8, a line), Flame 3 (26; 10), Frost 2 (14; 10),
    Arcane bolt 2 (11; 3, a line of force), Fire burst 3 (26; 9), Evil eye 1 (3; 2), Hex of
    weakness 1 (3; 3), Curse of sloth 1 (3; 4), Blight 2 (14; 8). Armour: chain 10, scale 20,
    banded 30, plate 40, great helm 10, large shield 10, tower shield 20. #397 retiered some
    of them for the first list (below): Blessing 1, Cure wounds 1 (a touch), Evil eye 0, Call
    lightning 3 (2d6 to 3d6), and Fire burst folded into Fireball.
- **The first list** (#397; design §7, §8; tests `world/spell_list_tests.rs`):
  - **Spells are data**: `SpellId` (an index in path order) is the one spell type; the
    twelve-variant `ClassSpell` of #395 is gone. `Action::Cast(String, _)` names the spell by its
    data id (`use:ID[:X,Y]`: any `use:ID` that is no birth sign or class active, parsed without
    content; an unknown one is refused by the rules, "You have no such ability."). The
    `Ability` methods that need a spell's shape or id take the content (`aim`, `action`,
    `action_at`), as does the aiming mode's `Aim::action`. `TalentEffect::Spell { spell }` holds
    the id string (checked against `data/spells/` at load, resolved in `TalentMods::spells`).
    `Content::spells_by_tier` is the menu's order (tier, then path), `Ability::every`'s too.
  - **The list** (26 spells): tier 0 Evil eye, Light (self buff: Lit, whose `light` the
    light cache sheds about the caster; #589, the scroll of light's), Mend (touch heal), Ray of frost (ray,
    attack), Spark (bolt, auto); tier 1 Barkskin (buff), Blessing, Burning hands (cone 3),
    Cause wounds (touch, necrotic), Cure wounds (touch heal 2d6), Curse of sloth, Hex of
    weakness, Magic missile (bolt, auto, force, 1d4 to 2d4); tier 2 Arcane bolt, Blight, Frost,
    Identify (#592, below), Scorching ray (ray, attack, 1d6 to 3d6, the design's "one to three rays" as dice), Sleep and
    Web (area hexes, burst radius 1); tier 3 Call lightning, Fireball (2d6 to 3d6), Flame, Haste
    (buff), Lightning bolt (line), Remove disease. New statuses: `drowsy` (cost 300), `webbed`
    (blocks `step`), `hasted` (cost 75: `COST_PERCENT` is now 50 to 1000, and `spell_def`
    refuses a status below 100 on anything but a buff, so only the caster is hastened and the
    rest batching stays exact), `barkskin` (Dodge +2). New effects: `frost_ray`, `spark` (bolts).
  - **Identify** (#592; tests `world/identify_spell_tests.rs`): divination, `utility`, `self`,
    `[spell.effect] identify = true` (`SpellEffect::identify`, `SpellData::aims_item`; only a
    `utility` spell on `self` may have it, and a `utility` spell needs it). It asks for a stack,
    not a cell: `Action::CastOn(id, item)` (`use:identify:c`; a spell's aim without a comma is a
    stack position), `Ability::aim` → `Aim::Item` ("an item you carry" in dumps, "on something you
    carry" in the menu), `Ability::action_on`; the cast path's target is a `ReadTarget` (cell or
    stack), `cast_aim` asking for the one the spell needs. `World::cast_identify` refuses (no
    time, no mana) without the stack or for one `Lore::knows`, then pays and calls
    `World::identify` (training Intelligence by `StatUse::Identified`, as the scroll did). The
    scroll of identify is its scroll (`read:b:c`, docs/dev/scrolls.md), keeping its own loot
    (frequency 300, no level; `spell_list_tests` leaves it out of the by-tier loot order). In the
    game the abilities menu opens the identify list (`IdentifyBy::Spell`) on choosing it.
    - **Its cost is explicit**: `mana` on a spell (`SpellData::mana`, 1 to `MAX_SPELL_MANA`
      500) replaces the tier table's price in `SpellRules::mana_with` (no tier cost, overreach,
      shape, or resolution; the armour's percent still added), and the talents' percents and
      Frugal caster still apply (`Actor::spell_base`, `Actor::mana_cost`). Identify's is **60**
      (the owner, 2026-10-28: a large cost, not coupled with its difficulty). Measured with
      `warlock-cli dump --character human,male,CLASS --actions "xp:N levelup:auto"`: a wizard
      has 47 mana at level 5, 55 at 6, 63 at 7, 71 at 8; a priest 42 at 5, 49 at 6–7, 63 at 8;
      a druid 47 at 6. So no caster of level 1 to 6 can cast it, a wizard can from level 7
      and a priest or druid from about 8. Its tier, 2 (comfortable at caster level 6), is how
      hard it is to learn (study's overreach).
  - **Area hexes** (`World::cast_area_hex`): a hex of shape burst flies as a burst
    (`burst_center`), and its status, lengthened by Presence, falls on every actor in
    `burst_cells` but the caster, friend or foe; each seen is said ("The rat is drowsy."). It
    goes off whether or not anyone is there. No draws.
  - **Scrolls of spells**: one object per spell (`scroll_of_ID`, `read` kind `spell`;
    docs/dev/scrolls.md), except the three older scrolls whose burst and bolt readings stay
    (fireball teaches Fireball, frost Frost, lightning Lightning bolt); every scroll teaches a
    spell (#592). Read: `World::read_cast`,
    the cast path with `Pay::Scroll(item)` (the `Cast` struct's `pay`; `World::pay_cast` is the
    one place a cast pays, mana or scroll, once it is sure to go off) at the scroll's level.
    Level, frequency, and value by tier: 0 (1, 60, 40 gold), 1 (2, 45, 80), 2 (4, 30, 130), 3
    (6, 20, 200). Shops draw their goods by loot frequency (`mapgen/shop.rs`, `pick_weighted`).
  - **Guaranteed spells**: class abilities in the class tables, one talent file per spell
    (`assets/data/talents/`, `tree = "class"`, `granted`, systems `spells` first), shared by
    the classes that give it. The content test holds every given spell within the tier the
    class's caster level has reached at that level, a full caster to a cantrip at 1 and four to
    six spells, and the others to at most two boons of tier 1 or less. Every class has `mana`
    at least 1 (the barbarian and the warrior gained theirs); races keep their own.
  - **Choices** (design §14, the defaults): q1, q2, q3, q5, q7 as above (interrupted study
    keeps the scroll; Dexterity aims; no cap lifts; `force` and `necrotic` stay; Spell recall
    never keeps a studied scroll); q4: the casting percents as §3 and the guaranteed lists as
    §8, the paladin's and ranger's boons Blessing and Barkskin at 6, the bard's Mend and
    Blessing; q6: the mishap table unchanged. Sleep has no level limit (the design's "creatures
    of low level"): it slows rather than stops, so any creature may be caught.
- **Caster level** (`world/caster_level.rs`): `Actor::caster_level(content, level, casting)`
  (`level × casting ÷ 100` rounded down, plus `TalentMods::caster_level` and the worn
  `Effect::CasterLevel`, half again with Attuned, rounded up (`TalentMods::attuned`), at least 0, at least 1 while `max_mana` is above 0),
  `World::caster_level(id)` (the player at `player_level` and `casting_percent`: the class's,
  100 without a class; a creature at its type's level and 100, plus what it wears, #525),
  `casting_percent(content, character)`. Derived, never stored: nothing in saves or the state
  hash changes. The dev `actors` layer shows each actor's with mana as `cl N`.
- **Presentation**: `PerceptionView::own_caster_level()`; the dump `stats` layer's
  `Caster level N` after the scores, JSON dumps' `caster_level`; `own_spell_cost(spell)` and
  `own_spell_reach(spell)` (the menu's "tier 2, 11 mana", the aiming mode's reach). The
  character screen does not show caster level yet.
- **UI**: the abilities menu's **Spells** group (`Source::Spell`) with tier and cost
  (`ui/abilities_screen.rs`, snapshot `abilities-menu-spells`); the aiming mode by shape
  (`ui/target.rs`: `cast_spell`, `Aim::spell_line`; a cone's dots are its cells).
- **Learning** (#396; `world/learn.rs`, `content/learning.rs`, tests `world/learn_tests.rs`
  and `content/learning.rs`):
  - **Known spells**: `Actor::spells` (`BTreeSet<SpellId>`, saved by index, left out while
    empty, so actors without hash as before; saves change) holds the learned ones;
    `Actor::knows_spell` is that or a class ability's `spell` effect (`TalentMods::has_spell`);
    `Actor::known_spells` lists both in menu order; `Ability::Spell(s).had_by` asks
    `knows_spell`. `Content::spell_of(SpellId)` is a spell's data whoever has it;
    `Content::class_spell` stays "a class ability gives it".
  - **Data**: `data/study.toml` (`StudyRules`: `base`, `per_overreach`, `known_per_point`,
    `turns_per_tier`, `bad_from`, `terrible_from`, and the words' thresholds `easy`,
    `struggle` in ways of 144); `data/mishaps/` (`Mishap`, one `[[mishap]]` a file, its
    `README.toml` the format: `band` slight/bad/terrible/lucky, `weight`, `message`, `effect`
    by `kind`: nothing, status, backfire, mana, teleport, lose_mark, drain, summon, learn, mark;
    only its kind's fields; `learn` and `mark` only lucky); `teaches` in a spell scroll's `read`
    table (`ScrollSpell::teaches`, a `SpellId`, checked as objects load; `Reading::teaches`
    is what every caller asks, a `spell` reading teaching its own);
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
- **Wands** (#398; design §9; `content/wand.rs`, `world/zap.rs`, tests `world/zap_tests.rs`):
  - **Data**: `[object.wand]` (`WandSpec` → `Wand { spell, charges, caster_level }` on
    `ObjectDef::wand`): an aimed spell (`SpellData::aimed`), `charges` 1 to `MAX_WAND_CHARGES`
    (99; default `SpellRules::wand_charges[tier]`, the new `[tiers] wand_charges` array, never
    ascending), `caster_level` 1 to `MAX_LEVEL` (default the tier's comfortable level). A wand
    has a `slot`, is not `stackable`, and has no attack, `launches`, range, food, `read`, or
    `kind`; it may have `level` and `frequency` (loot). Class `wands` (`item_classes.toml`).
    Seven wands in `data/objects/wand_of_*.toml`; general stores stock and buy them.
  - **Instances**: `Stack::charges` (private `Charges { used, known, carried }`; `None` = as
    made: full, unknown; saved and hashed, left out while `None`, so other stacks save as
    before; saves change). `charges_left` (rules, dev), `known_charges` / `charges_known`
    (presentation), `use_charge`, `identify_charges`; `carry_for` (Connoisseur) counts a
    wand's unknown charges too; `is_valid_for` holds `used` within the wand's charges.
  - **Use**: `World::fire` hands a wand in a `ready` slot (`Actor::wand_item`) to
    `World::zap`: empty → "Nothing happens.", no time, charges then known; else
    `World::wand_cast` (the cast path, `Pay::Wand(item)`: `pay_cast` → `World::use_charge`,
    "You zap the oak wand.", and the type learned through `learn_scroll`, "It is a wand of
    fireball.") at `Actor::wand_caster_level` (the wand's level + `TalentMods::wand_level` + the
    worn `wand_level` affixes', #633, at least 1), the player's reach; priced as `cast` × `casts`. A death by it is
    `Cause::Spell`, as a mana cast's.
  - **Labels**: `scroll_label::Labels::draw` draws wand labels (`data/wand_labels.toml`,
    `Content::wand_labels`, the `ScrollLabelDef` format) after the scrolls', from the same
    `identification` stream, so scrolls keep their labels; `Lore` names and colours a wand by
    its type's def (`label_def`), `Lore::knows` asks `charges_known` too, and `Lore::describe`
    adds "(7 charges)" / "(empty)" once known. `WorldState::known_scrolls` holds known wand
    types as well.
  - **Presentation**: `PerceptionView::ready_wand`; the aiming mode's `AimKind::Zap(item)`
    (`t` with a wand ready; the spell's line and reach once the type is known, a plain line
    before; sends `Action::Fire`); `describe::appraised`'s wand lines (the spell in words once
    the type is known, the charges once identified). Snapshot `wand-description`.
  - **Choices** (design §14 and the issue): the attack roll aims with Dexterity, as every
    spell's (there is no Missile skill); only aimed spells may be wands; a wand type is
    identified by use (as a scroll by reading), its charges only by identification; an empty
    wand's emptiness is learned by trying it; the zap takes the `cast` action's time; reach is
    the user's (Willpower), as a scroll's; Wand mastery is in Weaponcraft (system `spells`).
  - **Talents**: Wand mastery (`wand_mastery`, `TalentEffect::WandLevel { value }` →
    `TalentMods::wand_level`, +2).
  - **Wiki**: `xtask/src/wiki/spells.rs` `wands.md` (wand, spell, tier, charges, caster level,
    item level, frequency, value) on `wiki/src/magic.md`'s "Wands".
- **Rods** (#399; design §9; `content/rod.rs`, `world/rod.rs`, tests `world/rod_tests.rs` and
  `content/rod.rs`):
  - **Effects** (the registry, distinct from spells): `data/rod_effects/` (`RodEffect`, one
    `[[rod_effect]]` a file, its `README.toml` the format; `Content::rod_effects`, an index is
    the id, `rod_effect_index`), loaded after spells (they name statuses) and before objects.
    `shape` (`RodShape`: line, bolt, burst; each kind fixes its shape), `range` 1 to
    `MAX_ROD_REACH` (20), `max_range` from it to 20, `per_levels` 0 to 20, `radius` 1 to
    `MAX_ROD_RADIUS` (3) for a burst only, optional `noise`, and `kind` with only its fields
    (`RodDoes`): `dig` (`power` 1 to `MAX_ROD_POWER`), `teleport`, `status` (`status`,
    `severity` 1 to its max, `up_to_caster_level`). `RodEffect::reach(caster_level)` = `range` +
    caster level / `per_levels`, at most `max_range`. No visual effect yet (no `Event::Spell`).
  - **Rods**: `[object.rod]` (`RodSpec` → `Rod { effect, mana }` on `ObjectDef::rod`): `mana` 1
    to `MAX_ROD_MANA` (100); a rod has a `slot`, is not `stackable`, and has no attack,
    `launches`, range, food, `read`, `wand`, or `kind`; it may have `level` and `frequency`
    (loot). Class `rods`. Four in `data/objects/rod_of_*.toml`; general stores stock and buy
    them; the trap hall has a pile of two (digging, sleep) at (40, 19).
  - **Use**: `World::fire` hands a rod in a `ready` slot (`Actor::rod_item`, after a wand) to
    `World::use_rod`: refused (no time, no mana) on the overworld, at oneself, a burst out of
    reach (`within_reach`), while a status blocks `cast`, and without the mana; then
    `World::spend_mana(player, Actor::rod_base)` (`rod_base` = `TalentMods::rod_cost`, Channeler's
    percent, rounded, at least 1; `spend_mana` applies Frugal caster and trains Willpower;
    `Actor::rod_cost` is both, what is shown), "You zap the pewter rod." and the type learned
    through `learn_scroll` ("It is a rod of digging."), then the effect at the user's
    `World::caster_level`: `rod_dig` (`from.ray(at)` for the reach: `break_wall(p, Breach::Dug)`
    where the terrain breaks below `power` and `breaks_at`; stops at anything else opaque or off
    the map), `rod_teleport` (`bolt_line`, the first actor; `World::teleport`, the teleport
    trap's hash; then `provoke`, `provoke_court`, `break_empathy`; "The bolt strikes nothing."
    when empty, still paid), `rod_status` (`burst_center` and `burst_cells` as an area hex;
    `afflict` with the user as the giver; a creature above the caster level with
    `up_to_caster_level` "shrugs it off"). Priced as one `cast`. No draws.
  - **Labels**: `data/rod_labels.toml` (`Content::rod_labels`), drawn in `Labels::draw` after the
    wands' from the same stream; `Lore::label_def` picks a rod's. Nothing is saved on a rod's
    stack (no charges); a known rod type is in `WorldState::known_scrolls`, as a wand's.
  - **Presentation**: `PerceptionView::ready_rod`, `own_rod(item)` (cost and reach, once the
    type is known); the aiming mode's `AimKind::Zap` covers rods (`input.rs` starts it on a
    ready wand, else a ready rod; `Aim::range` is the rod's reach); `describe::appraised`'s rod
    lines ("Holds Digging: ...", "Costs 6 mana a use, before your talents; it never runs out").
    Snapshots `rod-description`, `rod-aim`.
  - **Choices** (design §14 has none for rods; the issue's open points): four effects (dig,
    teleport other, sleep, terror) on the dig, teleport, and status hooks; summoning is left out
    (there are no allies to summon, and a hostile summon is a curse, not a rod); the user's caster
    level sets reach and, for sleep and terror, which creatures are taken; the cost is fixed per
    rod, not by caster level; a rod type is identified by use or identification, as a wand's;
    an unknown rod still costs its mana when used; teleport other provokes as a blow does, the
    status bursts do not (as an area hex); a use takes the `cast` action's time.
- **Talents** (#400; design §10; tests `world/spell_talent_tests.rs`): the `spells` system in
  `data/talents.toml`, held to the bar. The Spellcraft tree: Quick study (#396), Overreach
  (level 6, `overreach_cost_percent` −33), School focus (`choice = "school"`, `Choice::School`:
  the seven schools then the non-physical damage types; `school_focus { cost = -10, reach = 1
  }`, one pick per choice, foci on the same spell add up). The wizard's Arcane training (class,
  at 1: `caster_level` +1 per 5 levels; the other casters have none of their own yet) and its
  bonus talents from Scholar and Spellcraft. Empower, Archmage, Scroll adept, Frugal caster name
  `spells` beside `mana`/`scrolls`; Spell recall, Scroll savant, Careful reader stay `scrolls`
  only (reading, not casting). Wand mastery is in #398's wands; Channeler (`rod_cost_percent`, level 5) is in #399's rods.
- **Balance** (#401; design §12, run against the data on 2026-10-27; `warlock-cli dump
  --character human,male,CLASS --actions "xp:N levelup:auto" --format json`, N the human
  threshold times the class's experience factor):
  1. **Pools** (the stats the seed rolls, so a +1 or +2 mental modifier; `levelup:auto` may
     take Deep well): wizard 33 / 106 / 254 / 502 at levels 3 / 12 / 30 / 60 (caster level 3,
     14, 36, 72), priest, druid, and witch about 27 / 90 / 216 / 426 (caster level = level),
     bard and aristocrat 17 / 45–53 / 103–125 / 201–246, monk, paladin, and ranger 9–14 /
     27–50 / 63–122 / 123–242, the 25 % classes 4 / 13 / 31 / 61. A full caster's pool is about
     ten single-target casts of its tier's base cost at every level (the design's aim): 106
     against tier 3's 11, 254 against tier 5's 22, 502 against tier 9's 56.
  2. **Regeneration** (a point per `max(60, 360 − 12 × Willpower)` seconds, resting at 200 %):
     a rest to full is under an hour at level 3, about 3 hours at level 12, 7 at 30, and 10 to
     14 at 60 (Willpower 13 to 16). Passes to level 30; at 60 it is a night's sleep, not the
     design's "a few hours": left for the owner (raising `[healing] mana_resting` would shorten
     it at every level).
  3. **Ceilings against hit points** (common and uncommon creatures, levels within each
     tier's caster levels): the median creature takes 1.3 to 3.1 casts at a tier's ceiling and
     2.4 to 5.7 at an average roll; none of a tier's median dies to one cast. Passes.
  4. **Weapon parity**: a level-12 warrior's great axe does 1d12 + Strength + the Axes rank
     (about 12 on average, 18 at most) a blow; a level-12 wizard's best single target is
     Scorching ray (2d6 at caster level 14, at most 12) for 6 mana, about 17 casts from the
     pool, and an area (Fireball, 26 mana) four. Passes: spells cost real mana. No spell
     above tier 3 exists yet, so a caster past caster level 18 reaches no higher ceiling.
  5. **Tests**: the cap check at load, `the_shipped_tier_table_reads_as_the_design_has_it`,
     the ascending checks, and `the_shipped_table_makes_areas_dearer_and_weaker_than_single_targets`.
  - **Data changed**: the scrolls of fireball and lightning are worth 200 gold (were 150) and
    frost 130 (was 100), the tier-3 and tier-2 values: #397 gave the three older scrolls their
    tier's item level and frequency but kept their M29 values, so a scroll of fireball sold for
    less than a scroll of flame, which does the same.
- **Wiki**: `xtask/src/wiki/spells.rs` (`spell-tiers.md`, `spell-shapes.md`,
  `spell-resolutions.md`, `spell-casting.md`, `spell-list.md`, `spell-armour.md`; `does` is
  shared with the talents' table) for `wiki/src/magic.md`.
- **Tests**: `content/spell_def.rs` (the shipped table, `tier_of`, the refused tables, the
  design's first spell list passing the cap, a spell over its cap, a bonus too large, malformed
  spells, the dice's growth, the cost table, reach and degrees, the shipped spells against
  the placeholders; #556: the lift's dice, the lift's bounds, `max_lift` refused unless
  ascending or flat, the scales' defaults equal to the old steps, unknown stats and missing
  dimensions refused, the unlifted cap check), `content/class.rs` (`casting`), `content/mod.rs` (spells in the
  fingerprint), `content/experience.rs` (`mana_level` 60), `world/caster_level_tests.rs` (by
  class and level, no mana, talents and a ring, a creature, the dumps, mana to level 60),
  `world/cast_tests.rs` (an attack can miss and critical under the cap, auto and area never,
  Empower to the cap, Evasion halving an area only, line against bolt, the cone, touch, armour,
  Willpower's reach, no draws without a roll; #556: the lift by stat, `max_lift` and Keen mind,
  the lifted dice and criticals held to the lifted cap, a cantrip and the defaults unchanged,
  to-hit by Intelligence; test spells replace Arcane bolt's file),
  `world/caster_tests.rs` (the casters' tables and spells), `world/spell_talent_tests.rs`
  (#400: Overreach, School focus by tag, Arcane training, the cap with every combination of
  the spell talents, an empowered scroll at its taught cap), `world/spell_list_tests.rs` (#397:
  scrolls of spells read, the area hexes, Haste, one scroll a spell, the classes' guaranteed
  spells within their tier, mana for every class), `magic/tests.rs` (`spell_scrolls_drop_by_tier`),
  `world/spell.rs` (cone geometry),
  `ui/target.rs` (aiming by shape).
