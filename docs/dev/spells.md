# Spells

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08e-blood-and-status.md ("Spells"), the approved magic design `docs/design/magic.md`, and
the spell scaling design `docs/design/spell-scaling.md` (its slices S0 to S6, tracker #759).
Keep it current with the code, as CLAUDE.md says.

The spell system is built in waves (design §13). This file covers what is built: the **data
foundation** (#394: the cost table, the spell files, the tags; spell levels in
place of the tier table since S1, #748), the **one cast path** (#395: cost, reach, dice,
resolution, the shapes, and the casters' placeholder spells migrated onto it), **learning**
(#396: the learned set, study, the check, mishaps; "Learning" below), the **first spell list**
(#397: the spells, their scrolls, the classes' guaranteed spells; "The first list" below),
**wands** (#398; "Wands" below), **rods** (#399; "Rods" below), the **Spellcraft talents**
(#400; "Talents" below), and the cross-checking pass (#401; "Balance" below). Not to be
confused with `content/spell.rs`, the bursts and bolts scrolls and traps cast, whose code the
cast path shares.

- **Spell levels** (S1, #748; the spell scaling design §5, decisions 1, 9, 10, 16): every spell
  has `level = N`, 0 to `MAX_SPELL_LEVEL` (70), one number with two meanings: its **base mana**
  (`max(1, level)`) and the **learning at which a scroll of it teaches for certain**. A
  spell of level 0 or 1 is a cantrip (a spell's own `mana` still replaces the base: Burning
  hands and Grave breath, level 1 for the class tables' guard, cost 3, #979). There is no table of levels, no ceiling, no cap, and no
  load-time check on how much a spell does, now or in any later slice (decision 16): authors
  write the strength they intend. Each tier column became a rule of the level (§5.2): the
  comfortable level is the spell's level; the cost is the level; the overreach surcharge,
  `ceiling`, `max_lift`, the resolutions' `damage` percent, and the run-time cap are gone; a
  wand's charges are its own; study time and difficulty are by levels (`data/study.toml`).
- **Ladders and hooks** (S2, #749; the spell scaling design §4, §6, §7, decisions 2 to 8, 12, 14
  to 16): a spell's growth is written in its own file as **ladders** (`[spell.grows]`,
  `content/spell_ladder.rs`), and talents and items act on it through **six named hooks**
  (`content/spell_mod.rs`'s `SpellMods`, summed by `Actor::spell_mods`, `world/spell_mods.rs`;
  read by `world/spell_rung.rs` and the cast path). The S1 bridge (dice by the caster's
  level, the lift, `SpellDice`, `[spell.scales]`, `max_range`/`max_length`/`max_radius`,
  `spell_scale.rs`) is gone. Nothing of the caster's level grows a spell.
- **Data**:
  - `assets/data/spells.toml` (`SpellRules`, `content/spell_def.rs`): the cost table:
    `[shapes]` (cost percents: `ray`, `bolt`, `touch`, `self`, `cone`, `line`, and `burst` a
    list by radius whose length is the largest radius, ascending, at most `BURST_RADII`'s end);
    `[resolution]` (`attack`, `auto`, `area`, `touch`, each `{ cost }` in percent; a `damage`
    key, the old cap's share, is refused). `[tiers]` and `overreach_cost` are refused (unknown
    fields). Every check refuses with a `ContentError`.
  - `assets/data/spells/` (`SpellData`), one `[[spell]]` file each, the format, the ladder
    grammar, and the authoring conventions (templates, not checks: §6.4) in its `README.toml` (a
    comment-only file, like the classes'). Read by `ContentFiles::table("spells")`
    minus `data/spells.toml` itself, so campaigns may add spells; a spell without the cost table
    is an error. Ids are `SpellId` indices in path order and enter the content fingerprint (saves
    name them: `Actor::spells`); a spell file added, removed, or renamed changes saves (S1
    edits the 26 in place: `tier = N` became `level = N`, path order kept).
  - Arcane training, the wizard's, is `skill { spellcraft, 10 }` (#988).
  - `mana_level = 60` in `data/experience.toml` (the last level), and the
    damage types `force` and `necrotic` at the end of `data/damage_types.toml` (non-physical:
    `Choice::Element` offers them, so Elemental ward may be taken for them).
- **Base rolls** (`spell_def::base_roll`, for `damage` and a heal's `[spell.effect] heal`): a
  `Roll` (`dice/roll.rs`, S0) that can do something (`min` ≥ 0, `max` ≥ 1). The old table form
  is refused naming the string form. Nothing bounds what they roll (decision 16).
- **Ladders** (`content/spell_ladder.rs`: `Ladder { stat, rungs: Vec<(u8, RungValue)> }`,
  `RungValue::{Roll, Cells, Degrees}`, `Ladder::parse(text, Unit)`, `rung_at(score)` (how many
  rungs are at or below the score), `value(rung)` (`None` at 0, the base), `words()` (the wiki's
  `Intelligence 13: 3d6, 16: 4d6`), `Display` as data writes it, `MAX_RUNGS` 8, `MAX_SCORE` 40):
  one string, the stat's id then `AT VALUE` rungs separated by commas, `AT` strictly
  ascending. `SpellData::grows: Grows { damage, heal, count, jumps, range, length, radius, degrees }`, each
  optional (`Grows::roll`, `reach`, `named`). `spell_def::grows` checks them against the
  spell's `Bases` (§6.3): a ladder only for a dimension the spell has (`damage` a damage roll,
  `heal` a heal, `range` a ray, bolt, or burst, `length` a cone or line, `radius` a burst,
  `degrees` a buff or a hex); rolls that never weaken (`min` and `max` non-decreasing from the
  base, each rung different from the one before); cells and degrees strictly ascending from
  the base and within the engine's limits (`MAX_REACH` 20, a cone `MAX_CONE` 5, a line 20, a
  radius the burst table's length, degrees the status's `max` and `MAX_SEVERITY`). **No check
  reads how much a rung does**, and none is to be added (decision 16). The wand's and the
  scroll's `rungs` are checked against the roll ladder's length where they load
  (`content/wand.rs`, `content/scroll.rs`).
- **Other checks** (`spell_def::resolve`): `level` present and 0 to 70; the retired fields
  refused, each naming its replacement (`retired`: `tier`, `max_range`, `max_length`,
  `max_radius`, `[spell.scales]`, `[spell.scroll]`); `Resolution::fits(shape)` (`area` exactly
  for the area shapes, `touch` only for touch, `attack` for ray and bolt, `auto` for any single
  target); the base reach each shape has exactly (`range` 1 to `MAX_REACH` for ray, bolt,
  burst; `radius` 1 to the burst table's length; `length` 1 to `MAX_CONE` for a cone, to
  `MAX_REACH` for a line), required; `casts` 1 to `MAX_CASTS`; a `damage` spell has `damage`
  and a `damage_type`; `aim` (a stat id, default Dexterity) only on an `attack` or `touch`
  damage spell; `[spell.effect]` holds only its kind's fields (heal: `heal`; buff, hex, and
  damage: `status`, `severity` 1 to `MAX_SEVERITY`; cleanse: `cures`, `diseases`), the
  statuses known; `[spell.cast]` (`SpellLook`): `effect` only for a shape cast at a distance
  (none: nothing drawn), `noun` (default the name in lower case) and `verb` (default the damage
  type's) not empty, a `noise` loudness 1 to `MAX_LOUDNESS`, `blast`/`heat`/`frost` only for a
  burst and checked by `Burst::new`; its effect against `data/effects.toml` once effects load
  (`spell_def::check_effects`: burst and cone a burst's, ray, bolt, line a bolt's).
- **Pure helpers**: `SpellRules::shape_cost(shape, radius)`, `ratio`, `mana(spell, armour)` (the
  cost, below); `SpellData::roll` (the base roll, harm or healing), `roll_at(rung)` (the
  ladder's roll at a rung, before any hook; the wiki's scroll and wand columns), `tags`,
  `aimed`, `base_range`, `trained_stats` (its ladders' stats, Intelligence when its roll has
  none: what a cast trains).
- **The hooks** (`content/spell_mod.rs`, `world/spell_mods.rs`; the spell scaling design §7.1):
  `SpellMods { rungs, scroll_rungs, wand_rungs, reach_rungs, degree_rungs, sides, dice, bonus,
  reroll, riders, cells, radius_cells, degrees, cost }`, `SpellMods::add` (riders of a status
  merged, `merge_rider`). `SpellMod { mods, when }` is one source with its tag filter
  (`fits(tags)`: any tag, or every spell without one). Talents: `TalentEffect::SpellMod(SpellModSpec)`
  (`{ kind = "spell_mod", rungs = 1 }`), checked by `SpellModSpec::check` (something to do, the
  rider's status known and its severity 1 to `MAX_SEVERITY`, `when` tags known schools, kinds,
  shapes, or damage types, `cost` a percent, and on a talent chosen for a school or an element
  no `when`: the choice is its filter), resolved into `TalentMods::spell`
  (`TalentMods::spell_mods(tags)` sums them). Items: `Effect::{SpellRungs, SpellBonus,
  SpellSides, SpellDice, SpellReroll, WandRungs}` and `SpellReach` (cells), `SpellDuration`
  (degrees), worn, unidentified too, with no tag filter (no shipped affix needs one; one is a
  field on the affix when it does). `Actor::spell_mods(content, spell)` is the one sum, with
  the school's rank as its third source (`Benefit::spell_mods`, docs/dev/magic-skills.md). `cost` is the one field beside the six: School
  focus's percent, summed into `spell_mana_percent` in `Actor::spell_base` (the decision of this
  slice: the design's §7.2 writes School focus as a `spell_mod` with a cost, so the struct
  carries it rather than a seventh effect kind).
- **The rung** (`world/spell_rung.rs`): `CastBy::{Own, Scroll(rungs), Wand(rungs)}` (`pub`, for
  the menu's S4); `Actor::spell_rung(content, spell, by, mods)`: an own cast `climb`s (the
  rungs the effective score reaches, `Actor::effective_stats`, plus `mods.rungs`); a scroll's
  or a wand's is its own rungs plus `scroll_rungs` or `wand_rungs`, no stats; never below 0 nor
  past the ladder's length. `Actor::spell_roll(content, spell, by)` is `roll_at(rung)` with
  `grow(sides).more(dice).plus(bonus)` (`PerceptionView::own_spell_roll`, the dumps; the menu in
  S4); `spell_reach` (the base range or length climbed by stats plus `reach_rungs`, then
  `cells`; the radius on its own ladder plus `radius_cells`; clamped only to `MAX_REACH`, a
  cone `MAX_CONE`, the burst table's length; a touch 1); `spell_degrees` (the base severity
  climbed, plus `degrees`, at least 1; the status's `max` holds it in `Effects::add`);
  `spell_to_hit` (the `aim` stat's modifier plus the worn `spell_to_hit`, of the Magus),
  `item_spell_to_hit` (a scroll's or a wand's: Dexterity's alone), `spell_threat` (22 less
  Baleful's, never below `combat::THREATS`' floor). The dev `progress` layer's `fireball rung
  2/4 (In 16; +1 Empower)` (`Actor::spell_rung_sources`).
- **The one cast path** (`world/cast.rs`, `World::cast_spell` from `Action::Cast`; decisions the
  design left open are under "Choices" below):
  - **Cost**, the one hook `Actor::spell_cost(content, spell)` (decisions 1 and 10): `SpellRules::mana` (`max(1, level)`, or the spell's own `mana`, ×
    `shape_cost(shape, its own radius)` × the resolution's `cost` ×
    (100 + `Actor::armour_spell_cost`, the summed `spell_cost_percent` of what is worn, each
    piece's less its own Mage-wrought `spell_ease`, never below 0: `Actor::worn_spell_cost`) / 100³,
    rounded to the nearest once, at least 1), then `TalentMods::spell_mana_with`:
    `spell_mana_percent` (Archmage −50, Empower −10) plus School focus's cost for a spell of
    its tag (`TalentMods::focus(spell.tags)`), rounded down, at least 1 (`Actor::spell_base`,
    what `World::spend_mana` is given), then `Actor::mana_cost` (Frugal caster and the worn
    `mana_cost`, of Thrift, summed, at most 50 % off) inside
    `spend_mana`. `ability::spell_unavailable(actor, content, spell)` is
    `Unavailable::NoMana` against it; `PerceptionView::abilities` and the cast ask it. Mana is
    spent only once the cast is sure to go off.
  - **Settled once** (`Cast::settle`): the hooks summed once (`Actor::spell_mods`), then the
    roll at the rung of who casts it (`Pay::Mana` an own cast, `Pay::Scroll(item, rungs)`,
    `Pay::Wand(item, rungs)`), its `reroll`, its `riders`, the caster's `Reach` and `degrees`.
    Nothing holds any of them (decisions 4 and 16).
  - **Reach**: `Actor::spell_reach` → `Reach { range, radius }` (above). A burst still costs by
    its own base `radius`.
  - **Dice**: `Cast::roll`, drawn with `Cast::reroll` (`Roll::roll(rng, reroll)`: a die at or
    below it drawn once more, that draw standing): a burst's and a line's through
    `Burst::reroll` and `Bolt::reroll` (0 for a trap's), a cone's directly, a single target's
    through `reflect::Shot { dice, reroll }` into `Profile::reroll` (0 for every blow) so a
    critical repeats the whole roll stage. A heal rolls the same way. The mishap backfire casts
    the spell's base, no hooks (`learn.rs::backfire_spell`).
  - **Kinds**: buff (its status on the caster), heal (self: the caster, refused when whole;
    touch: the creature at the adjacent cell, or the caster), cleanse (as the placeholder's),
    hex (the hostile the player sees at the cell, `hex_target`), utility (Identify, below;
    any other: nothing yet), damage
    (by shape: `Shape::Burst` → `burst_center` + `cast_burst`; `Line` → `bolt_line` +
    `cast_bolt`; `Cone` → `cast_cone`; `Ray`/`Bolt`/`Touch` → `cast_single`). A damage spell's
    `effect.status` and the cast's riders (merged, a status of both its severities added) fall
    on everyone it hit who lives (`spell_lands`); nothing on a miss. `Cause::Spell(SpellId)` for
    a death by one's own spell.
  - **Resolution**: area through `World::spell_hurts` (Evasion via `World::evaded`, then
    `World::spell_strikes`); auto through `spell_strikes` with a plain hit; attack and touch roll
    `combat::strike` with `Profile::bare(Attack { damage: roll, to_hit: the `aim` stat's modifier
    for an own cast (`Actor::spell_to_hit`), the Dexterity modifier for a scroll or a wand,
    threat `Actor::spell_threat` for an own cast (22 less Baleful's, carried by
    `reflect::Shot::threat`, a reflection keeping it) and 22 for a scroll or a wand, crit ×2
    }).with(benefit)` against
    `combat::Target::bare(target_against(target, missile).defence)` (no Protection; the strike's
    damage, crit included, is then resisted in `spell_strikes`, nothing holding it). The
    `benefit` (#571) is the spell's school's and the caster style's (`Actor::spell_skills`:
    the school's skill, docs/dev/magic-skills.md; the style of where it is cast from, decided
    once a cast, `world/spell_aim.rs`: Battle caster in armour that hinders casting, else
    Toucher at a creature beside, Sniper farther; `Benefit::at_distance` adds Sniper's
    far benefits from `[spell_attacks] far`), for a cast paid in mana of a spell that
    `rolls_to_hit` only; then `World::spell_practised` fires their triggered abilities
    (`resolve_triggers`); the marks are the cast's, once, at its end (`World::practise_cast`). A scroll or a wand
    keeps the bare roll (docs/dev/weapon-skills.md "Caster styles"). `spell_strikes` is the one
    place a spell's blow lands: the event (a miss too), provoking, the resistance, blood, the
    hurt, the wound. `cast_single` settles the caster's side once (`reflect::Shot`: dice,
    to-hit, effect, spell skills) and strikes through `spell_attack(by, target, source, shot)`, the one place a
    single target's roll is made.
  - **Spell reflection** (#566, `world/reflect.rs`): an attack or touch roll whose miss
    `Defence::why_missed` put down to the target's **Shield** raises the `spell_blocked`
    trigger for the target (`World::spell_blocked`, through `weapon_trigger`). A
    `reflect_spell` effect (Spell reflection, Sword and Board 10) marks the reflector while its
    `per_turn` count allows (`Reflections`, transient, cleared in `spend_actions`), and
    `cast_damage` then casts the same `Shot` back (`World::turn_back`): said first, from the
    reflector's cell back along the cells the spell flew through (they were empty), its own
    attack roll against the caster's Defence, the caster's roll, reroll, and to-hit, the
    spell's status and the caster's riders on a hit (`spell_lands`, by the reflector); no mana again, nothing for
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
    a confirmation) and the damage dice for a single target, one draw a die in group order and
    one more for each die the reroll hook sends back; one roll for an area, its targets in
    ascending actor id. A buff, hex, cleanse, or rider draws nothing.
  - **Saves**: nothing new in #395 (costs and reaches are derived); a spell is a `SpellId`
    (an index, in the content fingerprint) since #397. Old replays that cast verify only if the spell's cost is the same as
    the placeholder's; the regenerated costs change the state hash of any that cast.
- **Choices** (design §14 and the issue's open points, decided here):
  - The attack roll's skill: rays, bolts, and touches aim with their `aim` stat (Dexterity unless
    the spell names another; none does), threat and critical at the blows' defaults, plus since
    #571 the school's skill (Spell attacks until the schools of magic) and a caster style;
    Spellcraft does not aim spells (its
    benefit is the study check's, #396).
  - A critical rolls the damage again, and nothing holds it (decision 4).
  - S2's decisions (the issue's open points): **School focus's cost** is a `cost` field on the
    one `SpellMods` (the design §7.2 writes it so), not a separate effect; the hooks are six
    (rung, dice, roll, rider, reach, degrees), `cost` the seventh field beside them and not a
    hook of growth. **Affix names** as the issue lists them (`spell_rungs`, `spell_bonus`,
    `spell_sides`, `spell_dice`, `spell_reroll`, `wand_rungs`; `Effect::SpellDice` is free since
    the `SpellDice` struct went). **No tag filter on affixes** yet: no shipped affix needs one,
    and items sum into one table (`Mods`), so one is added with its first user. **The hex
    wands** (Sleep, Web) have no roll ladder, so their `rungs` range is 0 to 0 (the design's
    "the seven ship with `rungs = { min = 0, max = 2 }`" holds for the five that roll). **The
    reach a talent adds** is held only by the engine's limits (a cone at `MAX_CONE`): School
    focus on Burning hands is a cone of 4, where S1 held it at its cap of 3. **Trained stats**
    are the ladders' stats (Intelligence when the roll has none): a spell with no reach ladder
    (Burning hands) no longer trains Willpower by the old default. **An old Potent** was held to
    the nearest tier's range at a lenient read until #840 pinned saves to their version.
  - A ray/bolt flies to the first creature on its line; a hex (`ray`, `auto`) keeps the
    placeholder's rule, the creature seen at the cell aimed at.
  - The migrated tiers (#395, history: S1 turned them into levels, §5.8): the lowest tier
    whose cap holds the placeholder's dice (damage spells), else one near the placeholder's mana, never above the tier the class reaches when
    it grants the spell (so none costs overreach when granted). Costs and the data, old in
    brackets: Blessing 0 (1; 2), Cure wounds 2 (6; 4, now 2d6 for 1d8 + level/2), Remove
    disease 3 (11; 10), Call lightning 4 (29; 8, a line), Flame 3 (26; 10), Frost 2 (14; 10),
    Arcane bolt 2 (11; 3, a line of force), Fire burst 3 (26; 9), Evil eye 1 (3; 2), Hex of
    weakness 1 (3; 3), Curse of sloth 1 (3; 4), Blight 2 (14; 8). Armour: chain 10, scale 20,
    banded 30, plate 40, great helm 10, large shield 10, tower shield 20. #397 retiered some
    of them for the first list (below): Blessing 1, Cure wounds 1 (a touch), Evil eye 0, Call
    lightning 3 (2d6 to 3d6), and Fire burst folded into Fireball.
- **Spells aimed at a corpse** (#999; the spell types design §7 entries 2, 29, 30;
  `world/corpse_spell.rs`, tests `world/corpse_spell_tests.rs`): `[spell.effect] corpse =
  "consumed" | "kept"` (`CorpseUse`) on a burst of harm, a touch that heals, or a `self` summon with
  `rise` (`spell_def::corpse_fits`, refused otherwise). `SpellData::aims_corpse`, so `aimed()` holds
  whatever the shape; `Aim::Corpse` ("a corpse you see"; `Ability::action_at` as a cell's);
  `Reading::aimed` for the read list, since a `self` spell's scroll has no range. The rules:
  `World::corpse_in_sight` (the player sees the cell, by the perception; the pile's top-most corpse
  stack; **no stage read**, decision Q10), `use_corpse` (one of the stack gone when `consumed`).
  Presentation: `PerceptionView::{corpse_at, corpses_in_sight}`, read by the aiming mode (the cursor
  starts on the nearest corpse, Tab cycles them, no last target, `Line::no_corpse` says "There is no
  corpse there."). Refused, with no time and no mana, with no corpse.
  - **Charnel burst** (`cast_damage`'s corpse arm): `within_reach` as its reach check (the caster's
    own cell allowed: no one is spared), then the burst at the cell with `origin = center` (no flight,
    no `burst_center`), `World::spill_corpse` (a kill's spatter by the corpse's creature).
  - **Grave feast** (`cast_feast`, ahead of `cast_heal`): `effect.mana`, a roll to 500, on a heal.
    The caster's own: nobody on the cell is touched. Refused beyond a touch and when neither pool has
    room. The heal's roll, then the mana's, on the gameplay stream.
  - **Rouse the dead** (`cast_rise`, ahead of `cast_summon`; `docs/dev/allies.md` "The risen"):
    `effect.rise = { most, minutes, hit_points, berserk, berserk_faction, berserk_message }`,
    refusals in this order: no allies data, the overworld, no corpse, a `unique`'s, no free cell, the
    cap. Then one `below(100)` against `berserk`, whatever it is. `Grows::degrees` and
    `base_degrees` read `rise.minutes`. `count` and `roll()` are none: one corpse, one creature. No
    alignment deed, no caster level.
- **The first list** (#397; design §7, §8; tests `world/spell_list_tests.rs`):
  - **Spells are data**: `SpellId` (an index in path order) is the one spell type; the
    twelve-variant `ClassSpell` of #395 is gone. `Action::Cast(String, _)` names the spell by its
    data id (`use:ID[:X,Y]`: any `use:ID` that is no birth sign or class active, parsed without
    content; an unknown one is refused by the rules, "You have no such ability."). The
    `Ability` methods that need a spell's shape or id take the content (`aim`, `action`,
    `action_at`), as does the aiming mode's `Aim::action`. `TalentEffect::Spell { spell }` holds
    the id string (checked against `data/spells/` at load, resolved in `TalentMods::spells`).
    `Content::spells_by_level` is the menu's order (spell level, then path), `Ability::every`'s
    too.
  - **The list** (33 spells, by spell level, the spell scaling design §5.8; Water walking 5, Brand
    7, and Fire cloak 12, #1002, are buffs of status fields, docs/dev/status-effects.md): 0 Evil eye, Light
    (self buff: Lit, whose `light` the light cache sheds about the caster; #589, the scroll of
    light's); 1 Burning hands (cone 3, `mana = 3`, #979), Grave breath (the same, necrotic, #979), Mend
    (touch heal), Ray of frost (ray, attack), Spark (bolt, auto); 3 Barkskin
    (buff), Blessing, Cure wounds (touch heal 2d6), Curse of sloth, Hex of weakness, Magic
    missile (bolt, auto, force, 1d4 to 2d4); 4 Cause wounds (touch,
    necrotic); 6 Arcane bolt, Frost, Identify (#592, below), Sleep (area hex, burst radius 1);
    7 Blight, Scorching ray (ray, attack, 1d6 to 3d6, the design's "one to three rays" as
    dice), Web (area hex); 11 Call lightning, Fireball (2d6 to 3d6), Haste (buff), Lightning
    bolt (line), Remove disease; 12 Flame. The druid's calls (#839; summons, `docs/dev/allies.md`
    "The druid's calls"): 5 Call of the wild, 20 Call of the forest, 35 Call the elements, each
    with a scroll at the README's convention carried past level 12 (item level about half the
    spell's: 3, 10, 18; frequency 35, 10, 5; value 100, 320, 500) and a `study_difficulty` (8,
    10, 12). Six costs moved with the levels: Cause wounds 2 → 3,
    Burning hands 5 → 6, Scorching ray 6 → 7, Web 11 → 13, Blight 14 → 17, Flame 26 → 29. New statuses: `drowsy` (cost 300), `webbed`
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
    (frequency 300, no level; `spell_list_tests` leaves it out of the by-level loot order). In the
    game the abilities menu opens the identify list (`IdentifyBy::Spell`) on choosing it.
    - **Its cost is explicit**: `mana` on a spell (`SpellData::mana`, 1 to `MAX_SPELL_MANA`
      500) replaces its level's price in `SpellRules::mana` (no shape or resolution; the
      armour's percent still added), and the talents' percents and
      Frugal caster still apply (`Actor::spell_base`, `Actor::mana_cost`). Identify's is **60**
      (the owner, 2026-10-28: a large cost, not coupled with its difficulty). Measured with
      `warlock-cli dump --character human,male,CLASS --actions "xp:N levelup:auto"`: a wizard
      has 47 mana at level 5, 55 at 6, 63 at 7, 71 at 8; a priest 42 at 5, 49 at 6–7, 63 at 8;
      a druid 47 at 6. So no caster of level 1 to 6 can cast it, a wizard can from level 7
      and a priest or druid from about 8. Its level, 6, is how hard it is to learn (study's
      overreach).
  - **Area hexes** (`World::cast_area_hex`): a hex of shape burst flies as a burst
    (`burst_center`), and its status, at the caster's degrees, falls on every actor in
    `burst_cells` but the caster, friend or foe; each seen is said ("The rat is drowsy."). It
    goes off whether or not anyone is there. No draws.
  - **Scrolls of spells**: one object per spell (`scroll_of_ID`, `read` kind `spell`;
    docs/dev/scrolls.md), the three older ones too since S2 (fireball `rungs = 1`, lightning 2,
    frost 0: their burst and bolt readings and `read.rs::read_spell` went); every scroll teaches
    a spell (#592). Read: `World::read_cast`, the cast path with `Pay::Scroll(item, rungs)` (the
    `Cast` struct's `pay`; `World::pay_cast` is the one place a cast pays, mana or scroll, once
    it is sure to go off) at the scroll's `rungs` plus the reader's `scroll_rungs`.
    Level, frequency, and value are each scroll's own; the objects' README states the
    convention (S1, §5.7): spell levels 0–1 (1, 60, 40 gold), 3–4 (2, 45, 80), 6–7 (4, 30, 130),
    11–12 (6, 20, 200). Shops draw their goods by loot frequency (`mapgen/shop.rs`, `pick_weighted`).
  - **Guaranteed spells**: class abilities in the class tables, one talent file per spell
    (`assets/data/talents/`, `tree = "class"`, `granted`, systems `spells` first), shared by
    the classes that give it. A spell a class gives is its holder's at that level, whatever
    their Spellcraft: its level is the author's call (`docs/design/no-caster-level.md` §4). The
    content test (`every_class_has_mana_and_its_guaranteed_spells`) holds a full caster (its
    `FULL` list: druid, priest, witch, wizard) to a cantrip (level 0 or 1) at 1 and three to twelve spells (the priest's three:
    their rite is the Orison, `docs/dev/prayer.md`), and the others to at
    most two boons of level 5 or less. Every class has `mana`
    at least 1 (the berserker and the warrior gained theirs); races keep their own.
  - **Choices** (design §14, the defaults): q1, q2, q3, q5, q7 as above (interrupted study
    keeps the scroll; Dexterity aims; no talent lifts a cap (since S1 there is none); `force` and `necrotic` stay; Scroll savant
    never keeps a studied scroll); q4: the guaranteed lists as §8, the paladin's and ranger's boons Blessing and Barkskin at 6, the bard's Mend and
    Blessing; q6: the mishap table unchanged. Sleep has no level limit (the design's "creatures
    of low level"): it slows rather than stops, so any creature may be caught.
- **Presentation**: `own_spell_cost(spell)` and
  `own_spell_reach(spell)` (the menu's mana and the description's reach, the aiming mode's
  reach), `own_spell_roll(spell)` (a `SpellRoll`, `world/spell_words.rs`: `Actor::spell_roll`
  for an own cast, its damage type or "healed", and the text, `4d6 fire`; the menu's row and the
  dump `spells` layer's `Fireball (level 11, 26 mana): 4d6 fire, range 11, radius 2`; a test
  holds it to `Actor::spell_roll`), and `own_spell_reroll(spell)` (Sure hand's faces, the
  description's "Your 1s are rolled again."). `spell_words.rs` also says a reach
  (`reach_words`: `range 11`, `length 3`, `radius 2`) and a reroll (`reroll_words`) for the
  menu, the wand's description, and the dump. The character screen does not show spells (its
  attacks column holds three).
- **UI**: the abilities menu's **Spells** group (`Source::Spell`) with the roll now, its type,
  and the mana (`4d6 fire, 26 mana, aimed`; the spell scaling design decision 11, the owner's
  exception to hidden numbers; `ui/abilities_screen.rs`, snapshots `abilities-menu-spells` and
  `abilities-menu-spells-grown`); a wand's description, once its type is known, its dice at the
  wand's rung and the user's wand rungs and its radius (`Holds Fireball: 4d6 fire, radius 2`,
  `describe::appraised_by` through `PerceptionView::describe`; snapshot `wand-description`);
  no "your spell grows" line anywhere: the menu shows the new dice when it next opens; the
  aiming mode by shape
  (`ui/target.rs`: `cast_spell`, `Aim::spell_line`; a cone's dots are its cells).
- **Learning** (#396; `world/learn.rs`, `content/learning.rs`, tests `world/learn_tests.rs`
  and `content/learning.rs`):
  - **Known spells**: `Actor::spells` (`BTreeSet<SpellId>`, saved by index, left out while
    empty, so actors without hash as before; saves change) holds the learned ones;
    `Actor::knows_spell` is that or a class ability's `spell` effect (`TalentMods::has_spell`);
    `Actor::known_spells` lists both in menu order; `Ability::Spell(s).had_by` asks
    `knows_spell`. `Content::spell_of(SpellId)` is a spell's data whoever has it;
    `Content::class_spell` stays "a class ability gives it".
  - **Data**: `data/study.toml` (`StudyRules`: `base` 12, `per_overreach` 1, `known_per_point`
    20, `turns_base` 100, `turns_per_level` 20, `bad_from` 6, `terrible_from` 15 (at most 70),
    all in spell levels since S1, and the words' thresholds `easy`,
    `struggle` in ways of 144); `data/mishaps/` (`Mishap`, one `[[mishap]]` a file, its
    `README.toml` the format: `band` slight/bad/terrible/lucky, `weight`, `message`, `effect`
    by `kind`: nothing, status, backfire, mana, teleport, lose_mark, drain, summon, learn, mark;
    only its kind's fields; `learn` and `mark` only lucky); `teaches` in a spell scroll's `read`
    table (`ScrollSpell::teaches`, a `SpellId`, checked as objects load; `Reading::teaches`
    is what every caller asks, a `spell` reading teaching its own);
    a class's `affinity` (`ClassDef::affinity`, school → −10 to 10 study points); the skills'
    `study` benefit (`skills::Effect::Study`, Spellcraft's since #1024, a curve to 20 at
    200) and `study` mark (`Use::Study`, which is also `StatUse::Study`: Intelligence 100 %);
    `ActionKind::Study` (`study` in `data/actions.toml`, a turn); Raging blocks `study`.
  - **The action** (`Action::Study { item }`, `study:b`; `World::start_study` →
    `Spent::Study(Lesson)` → `World::study`, through `keep_at` like a meal): refusals as the
    spec lists them; `study_turns(rules, level, percent)` turns (`turns_base + turns_per_level ×
    level`, Quick study's
    `TalentMods::study_percent`); interrupted (`Ended::Came`/`Said`): "…you stop studying.",
    time lost, scroll kept (design §14 q1, the default). Done: the scroll is found again by
    object (positions may shift), the `study` use is trained, the odds are taken now, the
    scroll used up (`use_scroll`, never Scroll savant's keeping: §14 q7, decided no), then the check.
  - **Learning, the number** (`Actor::learning(content) -> Option<i32>`, #987,
    `docs/design/no-caster-level.md` §2): the `learn` benefit (`skills::Effect::Learn`, Spellcraft's
    in `data/skills.toml`; ceiling `MAX_SPELL_LEVEL`) through `Actor::skill_effect`, so skill
    affixes and the talents' `skill` bonuses count, at least 1; `None` when
    `Skills::trains_effect(Effect::Learn)` is false, whatever bonuses are worn. `None` means no
    learning at all: `study_odds` gives none, `PerceptionView::studies` lists nothing,
    `PerceptionView::can_learn` is false (the study list says `world::UNTRAINED` instead of its
    empty words), `start_study` refuses with `UNTRAINED` after the other refusals and before any
    time. The dev `progress` layer prints `learning 36 (Spellcraft 108)` (the trained value,
    before what items and talents add) or `cannot learn: not trained in Spellcraft`. Study is no
    ability, so the `abilities` layer and `Unavailable` have no row for it. A class's granted spells (`TalentMods::has_spell`) never read this.
    Starting Spellcraft is the class files' (`docs/dev/character-creation.md`).
  - **Odds** (`study_odds(actor, content, character, spell) -> Option<Odds>`, pure;
    `World::player_study_odds`): `Odds::overreach` = the spell's level − the learning; at 0
    or less `check: None` (certain, no draw), unless the spell's `study_difficulty`
    (`SpellData::study_difficulty`, −20 to 40, default 0; the owner's Q7 of #828: the calls'
    scrolls are very hard to learn) is above 0, which makes it a check at any learning with
    the overreach counted as at least 0; else `(bonus, difficulty)`: Intelligence
    modifier + `skill_effect(Effect::Study)` + the class's affinity with the school +
    `TalentMods::study_bonus`, against `StudyRules::difficulty(overreach′, known levels)`, where
    overreach′ is the overreach × (100 + `TalentMods::study_overreach_percent`) / 100, rounded
    down (Overreach's −33), and the known levels `Actor::known_levels`, the sum of the levels
    of the spells known, then + `study_difficulty`; the mishap's band reads the plain overreach;
    `Odds::ways()` of 144 (`check::ways`). `StudyRules::words` turns ways into "certain",
    "easy", "a struggle", "far beyond you".
  - **Mishaps** (`World::draw_mishap`, `World::mishap`): one `rng.below(total weight)` among
    `drawn_among(mishaps, band)` (the band's and the lucky ones, in path order); the message,
    then the effect through existing code: `afflict`; backfire (the scroll's own `Cast`:
    `cast_burst` centred on the reader, or `cast_bolt` on a path of their own cell, nothing
    holding the roll; `say_spell`;
    a death is `Cause::Scroll`); mana cut directly (no Frugal caster, no training); the teleport
    trap's `teleport` (made `pub(super)`); a stat mark off one drawn among the marked stats; a
    `Drain` (Willpower check); `travel::make_creature` at the first free cell around the reader
    (faction its type's, `spawned` bumped); `learn_spell`; a stat mark. Choices: the design's
    "learned anyway, one tier lower in power" is learned as is (no per-spell power yet);
    "a wild surge casts another spell" is not a mishap yet.
  - **Presentation**: `PerceptionView::own_spells`, `studies()` (`OwnStudy { item, spell,
    chance }`: identified scrolls teaching an unknown spell, pack order); the dump `spells`
    layer and JSON `spells`/`studies`; the dev `progress` layer's `study c frost: overreach 5,
    +0 against 17, 36/144` (overreach in spell levels). The study list (`ui/object_screen.rs`, `ObjectList::Study`, the
    `Study` key command `L`) and its dialog; snapshots `scroll-study-list`,
    `scroll-study-dialog`, `scroll-study-none`.
  - **Wiki**: `xtask/src/wiki/learning.rs` (`study-chances.md`, `study-times.md`,
    `study-affinities.md`, `mishaps.md`) on `wiki/src/magic.md`'s "Learning spells from
    scrolls".
  - **Talents**: Quick study (`quick_study`, the new Spellcraft tree's first talent:
    `TalentEffect::Study { bonus, percent, overreach_percent }`, +2 and −50 %), systems
    `spells` and `scrolls`; Overreach is the same effect's `overreach_percent` (S1).
- **Wands** (#398; design §9; `content/wand.rs`, `world/zap.rs`, tests `world/zap_tests.rs`):
  - **Data**: `[object.wand]` (`WandSpec` → `Wand { spell, charges, min_rung, max_rung }` on
    `ObjectDef::wand`): an aimed spell (`SpellData::aimed`), `charges` 1 to `MAX_WAND_CHARGES`
    (99; required since S1: the wand's own, the seven shipped with the old tier defaults, 12,
    12, 9, 9, 9, 7, 7), `rungs = { min, max }` (S2: default `min` 0, `max` the spell's roll
    ladder's length; within it, `min` at most `max`; `caster_level` refused naming it). The
    five wands that roll ship with `rungs = { min = 0, max = 2 }`, Sleep and Web with none (no
    roll ladder). A wand has a `slot`, is not `stackable`, and has no attack, `launches`, range,
    food, `read`, or `kind`; it may have `level` and `frequency` (loot). Class `wands`
    (`item_classes.toml`). Seven wands in `data/objects/wand_of_*.toml`; general stores stock
    and buy them.
  - **Instances**: `Stack::charges` (private `Charges { used, known, carried }`; `None` = as
    made: full, unknown; saved and hashed, left out while `None`, so other stacks save as
    before; saves change). `charges_left` (rules, dev), `known_charges` / `charges_known`
    (presentation), `use_charge`, `identify_charges`; `carry_for` (Lore) counts a
    wand's unknown charges too; `is_valid_for` holds `used` within the wand's charges.
    `Stack::wand_rung` (S5, #752; private `Option<u8>`, `None` for everything but a wand;
    saved and hashed, left out while `None`; saves change): drawn by
    `Stack::draw_wand_rung(content, rng)` evenly in `min_rung..=max_rung` (one `Rng::range`,
    no draw for a one-rung range or a non-wand, nothing drawn twice) wherever a wand is made,
    from the maker's own stream: `magic::loot::make_from` (last, after the ornament and
    material: loot, drops, chests, hoards, room loot, kits' and gear's magic lines), `kit::roll`'s
    plain lines, `gear::make`, `mapgen/shop.rs`'s stock, `World::leave_objects` (outcomes,
    `gameplay`), and as a fallback for wands that come from content without one: a level's piles
    and chests in `instantiate_level` (`Rng::objects(seed, level id)`, piles in cell order,
    then chests), a creature's inventory in `make_creature` (`objects/actor/<id>`), and the
    scenario's pack in `World::begin` (`objects/pack`). `wand_rung(content)` (rules, dev; a
    wand without one reads 0, as a save from before S5 does), `known_wand_rung(lore)`
    (presentation: with the type, `Lore::is_identified`); `is_valid_for` refuses a rung on
    anything but a wand. `describe_truth` adds `(rung N)` (the `actors` layer), the
    `inventory` layer and JSON (`wand_rung`) only once the type is known.
  - **Use**: `World::fire` hands a wand in a `ready` slot (`Actor::wand_item`) to
    `World::zap`: empty → "Nothing happens.", no time, charges then known; else
    `World::wand_cast` (the cast path, `Pay::Wand(item)`: `pay_cast` → `World::use_charge`,
    "You zap the oak wand.", and the type learned through `learn_scroll`, "It is a wand of
    fireball.") at `CastBy::Wand(stack.wand_rung)` (plus the user's `wand_rungs`, Wand mastery
    and of Wands, in `Actor::spell_rung`), the user's reach, degrees, and
    other hooks; priced as `cast` × `casts`. A death by it is `Cause::Spell`, as a mana
    cast's.
  - **Labels**: `scroll_label::Labels::draw` draws wand labels (`data/wand_labels.toml`,
    `Content::wand_labels`, the `ScrollLabelDef` format) after the scrolls', from the same
    `identification` stream, so scrolls keep their labels; `Lore` names and colours a wand by
    its type's def (`label_def`), `Lore::knows` asks `charges_known` too, and `Lore::describe`
    adds "(7 charges)" / "(empty)" once known. `WorldState::known_scrolls` holds known wand
    types as well.
  - **Presentation**: `PerceptionView::ready_wand`; the aiming mode's `AimKind::Zap(item)`
    (`t` with a wand ready; the spell's line and reach once the type is known, a plain line
    before; sends `Action::Fire`; the prompt names what the wand does once its type is known,
    "Zap a wand of fireball (4d6 fire, radius 2)", `PerceptionView::wand_does`);
    `describe::appraised_for`'s wand lines through `PerceptionView::describe_item` (once the
    type is known, `describe::wand_does`: the player's `spell_roll` at `CastBy::Wand(rung)`,
    the damage type's name or "healed", their burst radius, "Holds Fireball: 4d6 fire, radius
    2"; the spell in words for a wand that rolls nothing, or with no user, as plain
    `describe::appraised` gives it; the charges once identified). Snapshots
    `wand-description`, `wand-aim`.
  - **Choices** (design §14 and the issue): the attack roll aims with Dexterity, as every
    spell's (there is no Missile skill); only aimed spells may be wands; a wand type is
    identified by use (as a scroll by reading), its charges only by identification; an empty
    wand's emptiness is learned by trying it; the zap takes the `cast` action's time; reach is
    the user's (Willpower), as a scroll's; Wand mastery is in Weaponcraft (system `spells`).
  - **Talents**: Wand mastery (`wand_mastery`, `spell_mod { wand_rungs = 1 }`).
  - **Wiki**: `xtask/src/wiki/spells.rs` `wands.md` (wand, spell, spell level, charges, its
    strength, the rolls at the range's ends, `roll_at(min_rung)` to `roll_at(max_rung)`, item
    level, frequency, value) on `wiki/src/magic.md`'s "Wands".
- **Rods** (#399; design §9; `content/rod.rs`, `world/rod.rs`, tests `world/rod_tests.rs` and
  `content/rod.rs`):
  - **Effects** (the registry, distinct from spells): `data/rod_effects/` (`RodEffect`, one
    `[[rod_effect]]` a file, its `README.toml` the format; `Content::rod_effects`, an index is
    the id, `rod_effect_index`), loaded after spells (they name statuses) and before objects.
    `shape` (`RodShape`: line, bolt, burst; each kind fixes its shape), `range` 1 to
    `MAX_ROD_REACH` (20), `grows.range` (`RodEffect::grows: Option<Ladder>`, S2's
    `content/spell_ladder.rs` in `Unit::Cells`, rungs rising strictly from `range` to at most
    20; the spell scaling design §7.5), `radius` 1 to `MAX_ROD_RADIUS` (3) for a burst only,
    optional `noise`, and `kind` with only its fields (`RodDoes`): `dig` (`power` 1 to
    `MAX_ROD_POWER`), `teleport`, `status` (`status`, `severity` 1 to its max, `up_to_level`, a
    creature level of 1 or more). The retired `max_range`, `per_levels`, and
    `up_to_caster_level` are refused naming their replacements. `RodEffect::reach(stats)` = the
    ladder's value at the rung the score in its stat reaches, `range` below the first;
    `Actor::rod_reach` reads it at `effective_stats`. Nothing of the user's level is read
    anywhere in rods (decision 10). No visual effect yet (no `Event::Spell`).
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
    `Actor::rod_reach`: `rod_dig` (`from.ray(at)` for the reach: `break_wall(p, Breach::Dug)`
    where the terrain breaks below `power` and `breaks_at`; stops at anything else opaque or off
    the map), `rod_teleport` (`bolt_line`, the first actor; `World::teleport`, the teleport
    trap's hash; then `provoke`, `provoke_court`, `break_empathy`; "The bolt strikes nothing."
    when empty, still paid), `rod_status` (`burst_center` and `burst_cells` as an area hex;
    `afflict` with the user as the giver; a creature above the effect's `up_to_level` "shrugs
    it off"). Priced as one `cast`. No draws.
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
    (there are no allies to summon, and a hostile summon is a curse, not a rod); the user's
    Willpower sets reach (a ladder, S3), and sleep and terror take creatures to their own
    `up_to_level` (20), never by the user's level; the cost is fixed per rod; a rod type is identified by use or identification, as a wand's;
    an unknown rod still costs its mana when used; teleport other provokes as a blow does, the
    status bursts do not (as an area hex); a use takes the `cast` action's time.
- **Talents** (#400; design §10; the spell scaling design §7.2; tests
  `world/spell_talent_tests.rs`): the `spells` system in `data/talents.toml`, held to the bar.
  The Spellcraft tree: Quick study (#396), Overreach (level 6, `study { overreach_percent = -33
  }` since S1), School focus (`choice = "school"`, `Choice::School`: the seven schools then the
  non-physical damage types; `spell_mod { cost = -10, cells = 1 }`, one pick per choice, foci on
  the same spell add up), Keen mind (the same choice, `spell_mod { sides = 1 }`), and S2's three
  (each hook ships with a user, but the rider: "Spell types" below took Numbing cold's): Sure
  hand (level 12, `spell_mod { reroll = 1 }`), Brimming (level 25, `spell_mod { dice = 1 }` and `spell_mana_percent` +10; systems `spells`
  and `mana`). The wizard's class talents: Scroll adept (at 1, reading −50 % and `spell_mod {
  scroll_rungs = 1 }`: scrolls only, decision 12), Arcane training (at 1: Spellcraft +10), Empower (at 32, `spell_mod { rungs = 1 }`, mana −10 %), Archmage
  (at 60, `spell_mod { rungs = 2 }`, mana −50 %); its bonus talents from Scholar and
  Spellcraft. Empower, Archmage, and Frugal caster name `spells` beside `mana` (Empower and
  Archmage no longer touch scrolls, so no longer `scrolls`); Scroll savant (which took in
  Careful reader in #775) stays `scrolls` only. Wand mastery is in #398's wands; Channeler
  (`rod_cost_percent`, level 5) is in #399's rods.
- **Balance** (#401; design §12, run against the data on 2026-10-27; `warlock-cli dump
  --character human,male,CLASS --actions "xp:N levelup:auto" --format json`, N the human
  threshold times the class's experience factor):
  1. **Pools** (the stats the seed rolls, so a +1 or +2 mental modifier; `levelup:auto` may
     take Deep well): wizard 33 / 106 / 254 / 502 at levels 3 / 12 / 30 / 60, priest, druid,
     and witch about 27 / 90 / 216 / 426,
     bard and aristocrat 17 / 45–53 / 103–125 / 201–246, ascetic, paladin, and ranger 9–14 /
     27–50 / 63–122 / 123–242, the 25 % classes 4 / 13 / 31 / 61. A full caster's pool is about
     ten casts of a spell of the caster's own level at every level (the design's aim; the spell
     scaling design §12.4): 106 at level 12 against a level-11 spell's 11 mana, 254 at 30
     against 29, 502 at 60 against 56.
  2. **Regeneration** (a point per `max(60, 360 − 12 × Willpower)` seconds, resting at 200 %):
     a rest to full is under an hour at level 3, about 3 hours at level 12, 7 at 30, and 10 to
     14 at 60 (Willpower 13 to 16). Passes to level 30; at 60 it is a night's sleep, not the
     design's "a few hours": left for the owner (raising `[healing] mana_resting` would shorten
     it at every level).
  3. **Ceilings against hit points** (history: S1 deleted the ceilings, decision 16; common
     and uncommon creatures, levels within each tier's): the median creature takes 1.3 to 3.1 casts at a tier's ceiling and
     2.4 to 5.7 at an average roll; none of a tier's median dies to one cast. Passes.
  4. **Weapon parity**: a level-12 warrior's great axe does 1d12 + Strength + the Axes rank
     (about 12 on average, 18 at most) a blow; a level-12 wizard's best single target is
     Scorching ray (2d6 then, at most 12) for 6 mana, about 17 casts from the
     pool, and an area (Fireball, 26 mana) four. Passes: spells cost real mana. No spell
     above tier 3 exists yet, so a caster past level 18 reached no higher ceiling.
  5. **The probe** (the spell scaling design §12.4, S7, #754; `docs/balance.md` "Spells" and
     "School pace"; `world/balance_probe.rs`, `balance_probe/schools.rs`): the trained
     wizard's, priest's, and witch's roll of every spell that rolls, `Actor::spell_roll` in
     plain gear (`| Spell (level, res) | L | rung | school rank | roll | mana / pool |`, the roll
     beside its rank-0 roll, a reroll's mean exact), at the school and caster-style ranks their
     casting reaches: each day of a level, one pool on the school's pace spell, every cast
     through `World::practise` as `practise_cast` does. The pool check by level: the pool over a
     single-target spell of the caster's own level (base mana = the level, the ranks' mana
     off), 10 to 13 for the wizard from level 5 to 60, 8 to 9 the priest, 8.5 to 12 the witch.
     Measured pace (the wizard): evocation 5 at level 10, 7 at 20, 8 at 30, 10 from 50, §8.4's
     estimate; the schools stay on the missile curve (no row tuned). No casts-to-kill column.
  6. **Tests**: since S1 `the_shipped_cost_table_reads_as_the_design_has_it` and
     `the_shipped_table_makes_areas_dearer_and_touches_cheaper` (the cap check at load and the
     tier table's went with the table, and nothing replaces them: decision 16).
  - **Data changed**: the scrolls of fireball and lightning are worth 200 gold (were 150) and
    frost 130 (was 100), the tier-3 and tier-2 values: #397 gave the three older scrolls their
    tier's item level and frequency but kept their M29 values, so a scroll of fireball sold for
    less than a scroll of flame, which does the same.
- **Wiki**: `xtask/src/wiki/spells.rs` (`spell-shapes.md`, `spell-resolutions.md` (resolution,
  roll, cost), `spell-list.md` (spell, level, school, shape and resolution,
  the base reach, what the base does, **Grows with**: each ladder in words, `grows_with`, the
  roll's first and unnamed: `Intelligence 13: 3d6, 16: 4d6; range Willpower 14: 11, 18: 12`,
  mana, casts), `spell-armour.md`; `does` is shared with the talents' table) for
  `wiki/src/magic.md`: the list by spell level, then name; `class-spells.md` "(level 11, from
  12)"; `scrolls.rs`'s "Casts at" (the roll at the scroll's rung); `talents.rs`'s
  `spell_mod_text` (each hook a talent uses, in the cast's order); `magic.rs`'s affix words and
  the talents beside each affix; `learning.rs`'s `times` and `chances` one row per sampled
  spell level (0, 3, 6, 11, 16, 22, 29, 37, 46, 56).
- **Tests**: `content/spell_def.rs` (the shipped cost table, the refused tables (`[tiers]`,
  `overreach_cost`, a resolution's `damage`), the design's first spell list, no check on how
  much a spell does (any roll, bonus, heal, or rung loads), malformed spells (a level of 71, a
  `tier` key, no level, each retired field naming its replacement), the ladders' checks (a rung
  below its base, a descending ladder, a ladder for a dimension the spell lacks, an unknown
  stat, a radius past the burst table, a degree past the status's max, nine rungs, `aim` on an
  area), the cost by level, shape, resolution, and armour, the 26 shipped spells at §5.8's
  levels, costs, and base rolls, and every shipped ladder as §12 writes it),
  `content/spell_ladder.rs` (the grammar, `rung_at`, the refusals), `content/learning.rs` (the
  study rules in levels, the bands at 6 and 15), `world/zap_tests.rs` (a wand without `charges`,
  with `caster_level`, or with `rungs` past its ladder refused; a zap at the stack's own rung,
  Wand mastery a rung more; the draw even over the range, once, the same for the same seed; a
  non-wand drawing nothing and saving as before; a wand saved without a rung read as 0; the
  rung shown with the type and not before), `content/mod.rs` (spells in the
  fingerprint), `content/experience.rs` (`mana_level` 60), `world/mana_tests.rs` (mana to
  level 60),
  `world/cast_tests.rs` (an attack can miss and critical with nothing holding it, auto and area
  never critical, Empower's rung, Evasion halving an area only, line against bolt, the cone,
  touch, armour, Willpower's reach ladder at 13, 14, 17, 18, no draws without a roll; S2: the
  rung by score and by stat damage, a cast at every rung drawing that rung's dice and nothing
  more, a rung's boundary changing the roll and nothing else, Sure hand's reroll drawing once
  more per die and never twice, `aim` by Intelligence), `world/affix_spell_tests.rs` (the Magus, Potent's
  bonus, Lingering, Farcasting past the ladder, Baleful, the old Potent read),
  `world/affix_draught_tests.rs` (of Wands' rungs beside Wand mastery, never past the ladder),
  `world/weapon_ability_tests.rs` (reflection carries the caster's climbed roll),
  `world/caster_tests.rs` (the casters' tables and spells), `world/spell_talent_tests.rs`
  (#400: Overreach on the study check and never on mana, School focus by tag, Arcane
  training, every combination of the spell talents casting; S2: Empower and Archmage to the
  ladder's last rung and the other hooks after, Keen mind on its school only, Brimming's die
  and cost, Sure hand's hook, Scroll adept's scroll rung past the scroll's
  own roll), `world/learn_tests.rs` (study turns by level and Quick study, certain at or below the
  learning, the difficulty by levels and known levels, the dumps), `world/spell_list_tests.rs` (#397:
  scrolls of spells read, the area hexes, Haste, one scroll a spell, the scrolls' loot by spell
  level, the classes' guaranteed spells, mana for every class),
  `magic/tests.rs` (`spell_scrolls_drop_by_level`),
  `world/spell.rs` (cone geometry),
  `ui/target.rs` (aiming by shape).

## Zones

An area that lies on the floor for a time (`world/zone.rs`; the design's §7 "Zones", #1000).
Balefire, Reek, Mirk, and Slick (scroll only) lay one with `[spell.effect] zone = { kind, turns }`
(`SpellZone`, `ZoneKind`; `content/spell_def.rs::check_zone`: a burst or a line; `turns` 1 to
`MAX_ZONE_TURNS`, of 6 s; `fire` a `damage` spell of fire, `fouled` of poison, `dark` and `slick` a
`hex`, whose `status` is then not asked).

- **State**: `LevelState::zones: Vec<Zone>` (`kind`, `cells`, `until`, `by`, `harm`, `light`,
  `spell`), saved, hashed, left out while empty, and `LevelState::zone_memory`, what the player saw.
- **Rules**: fixed cells, never spreading (owner, Q3); a new zone takes its cells from the old
  (`lay_zone`); `by` is whose kill it is, never who it spares (Q14); past `until` it does nothing,
  and `zones_fade` drops it at the end of the turn, on the current level, quietly. No zone on the
  overworld (`can_loose_here`).
- **Hooks**, one line each: `zone_turn_start` first in `ai.rs::creature_turn` (before a sleeper's
  rest and a stride) and in `schedule.rs::run_turn` as the player's next turn begins (`fire`,
  `fouled`); `zone_tread` in `ice.rs::after_tread`, every step's one hook (`fire` onto, `slick` onto
  or off); `LevelState::darkened` in `LightCache::sources` and `LevelState::shade` in the light pass,
  the player's (`perception.rs`) and a creature's (`sight.rs::light`) alike (`dark`); the fire's cells
  are sources in `LightCache::sources`, each keyed by `zone_key`, a light of the fire type's
  `fire.light`. A stride stops short of a zone's cell (`stride.rs`), so a coarse turn never crosses one
  unburnt. Bounce light is not cut: light landing on a dark cell may still bounce faintly beside it.
- **The cast**: `cast_damage` takes the cells before the cast (`zone_cells`: a burst's centre moves
  when who stood in its way falls) and lays after `spell_lands`; `cast_hex` hands a hex with a zone
  to `cast_zone`. A fire's `harm` is rolled once as it is laid (the spell's `Cast::roll`), so nothing
  draws at a burn; ice under a fire melts as it is laid; `fire` and `slick` lie only on walkable ground
  out of open water, `fouled` and `dark` on anything sight passes through.
- **Clearing**: `LevelState::clear_zone_cells(cells, kinds)`. Cold's `douses` reads it in
  `spell_type.rs::douse` (`[Fire]`). Gale's `clears` reads it in `spell_rule.rs::cast_gale`
  (`[Fire, Fouled, Dark]` from its data) on the cells of its line.
- **What the player knows**: `Perception::zones`, the zones on visible cells; `remember` keeps
  them in `zone_memory` as terrain is; `PerceptionView::zone_at` is visible, remembered, or nothing
  on a hidden cell. A dark cell is not visible without darkvision, so it shows by the dark itself.
  The words are `ui/look.rs::zone_words`, the tint `ui/zone_tint.rs`.
- **Determinism**: a fire's roll at the cast; Slick's check (two draws) at the step; the type's
  rider's chance per creature harmed; nothing else, and a level with no zone only reads an empty list.
- **Dev**: `zone:KIND@X,Y,R` (`zone_dev`: by the player, ten turns, a fire's harm 4, no time); the
  `truth` layer has a line a zone, JSON `--state` a `zones` list.
- **Tests**: `world/zone_tests.rs`, `ui/look.rs` (the words' snapshot), `ui/zone_tint.rs`.

## Spell types

What every spell of a damage type does beyond its damage (the design:
`docs/design/spell-types-and-new-spells.md` §3 to §5, §10).

- **Data**: `[types]` in `assets/data/spells.toml`, one entry a damage type, documented there;
  `content/spell_type.rs` parses and checks it (`SpellRules::types`; `Content::spell_type(kind)`, none
  for a type without an entry). `spell_type::parse` runs apart from `SpellRules::parse`, which
  skips the table, because it needs the statuses and lights loaded. A new rule is a field on
  `TypeSpec` and `SpellType`, a hook, and a row in the table of `xtask/src/wiki/spells.rs::types`.
- **Hooks** (`world/spell_type.rs`, one line at each place on the cast path):
  - `spell_type_lands` in `cast.rs::spell_lands`, after the cast's own status and riders: the
    `rider`, then `ends`. It walks the `harmed` list: `Strike::Hit`, not the `reached` list its
    own status falls on, which counts `Strike::Resisted` (one that resisted it all takes no burn
    and makes no draw). A scroll's, a wand's, a reflected, and a mishap's cast all pass here
    (`learn.rs::backfire_spell` goes through `spell_lands` for it, so the scroll's own status
    falls on the reader too).
  - `cast_burst` (`world/spell.rs`) for `Caster::Actor` only: the ice (`burst.ice`, a spell's own
    `heat` or `frost`, or else the type's), then `spell_type_burst` (`shatter`, `douses`, `fire`).
    With a `blast`, `doors_blasted` lists the shut doors before `World::blast`, so a blast never
    reaches a door through a wall it breaks, and `blast_doors` breaks them after it. `shatter` reads
    the burst's `blast` × `SHATTERS_UM` (100 µm a point: Shatter's 1000 is ice under 10 cm, what
    bears a person; `blast` as µm would be a 1 mm skin that bears no one).
  - `spell_type_bite` in `spell_strikes` (`world/spell.rs`), after Poison lore's bite and before
    `resisted`: `bite` reads the target's hit points now.
  - `spell_type_shove` in `spell_strikes`, after `wounded`, on one who lives: `combat::big_hit` on
    the hit points taken, or a confirmed critical. Away from `source.0` (a burst's centre, a ray's
    caster), else from the caster for the one at the centre; the caster there has no way and
    stays. A status blocking `Step` holds it (`Effects::blocking`). `weapon_active.rs::drive_back`
    moves it and pushes `Event::ActorMoved` to `made`; `read.rs::say_spell` reports it after the blow
    (`knock_back`'s direct `say` would come before it) and a blast's `Event::DoorBroken`, seen, too.
  - `immune_tags` has no hook on the cast path: `spell_type::lay_immunities` at load (after
    `load_creatures`) sets the type to `IMMUNE` on the `resist` of every creature type of the tag,
    so `Actor::resistances`, Knowledge's `known_resistances`, the creature's description, and the
    wiki all read it, and no creature file names it. It stands against any harm of the type, as a
    `resist` line does: a Withering blade too, though no blow reads the rest of the table.
    `Caster::Trap` and `Caster::Place` keep their own data and read no type. Blaze is not on this
    path.
  - `spell_type_cell` in `cast.rs`: at radius 0, on the cell of the creature a single-target spell
    struck, hit or missed; with no creature on the cell aimed at, if the ray got there (the
    design's "last cell of the line" would freeze the far end of a ray, past the aim, and it
    flies its whole range: no bridge could be built), else the last it reached. A line's arm in
    `cast_damage` calls it on the line's last cell: not `cast_bolt`, which Fire breath shares. A
    cone has no world rule.
  - `spell_type_spreads` (`world/spell_arc.rs`) in `cast_damage`, once, after the shape's arm and
    before `spell_lands`, so `say_spell`, the riders, and the marks read every strike alike. Each
    arm hands back what it rolled (`cast_burst`, `cast_bolt`, `cast_cone`, `cast_single`,
    `spell_attack`), and each spread strike is `spell_strikes` at that number: no dice, no roll to
    hit, no Evasion (`spell_hurts` is an area's). It walks `made` by index as it grows, so a creature
    harmed, however it was struck, conducts and then arcs once; `Strike::Hit` only, on the cell
    in its event (a dead one's too). `conducts` for every shape, `arc` for a ray, bolt, touch, or
    line. `World::water_body` fills 8 neighbours of `LevelState::is_open_water` into a
    `BTreeSet`, one per body per cast; a `retinue` never stands anywhere. Fire breath shares
    `cast_bolt` and a mishap `cast_burst`, but only `cast_damage` calls this, and the turned-back
    flight runs after it, so a reflected spell spreads nothing. The event drawn is the first
    flight only: an arc is told by `Event::Attacked`, as the owner's Q4 and the design's
    "Perception" have it, with no preview on aiming.
  - `SpellData::effect.arcs` and the `count` key of `[spell.grows]` on such a spell
    (`Grows::jumps`, whole rungs, 1 to 12; `Grows::count` stays a summon's): `Actor::spell_reach`
    gives them as `Reach::jumps`, climbed as a reach ladder, and `reach_words` says `jumps 3`.
    With `arcs` the reach is `ARC_REACH` (2) for every jump, certain or by the draw, and a jump
    needs a line clear of what is opaque (the design says only "within 2 cells"; a chain through
    a wall would strike what no one could see). `arcs` is refused at load off those four shapes
    and on a type with no `arc`.
  - `TalentEffect::SpellType { arc_chance, when }` (`TalentMods::arc_chance`, the highest): read
    in `spell_type_spreads`, by the caster's talents. Conductor's. It needs no `alike` line.
- **`World::lay_fire`** (`spell_type.rs`): the one laying of a fire on a walkable cell, longest
  wins; Blaze's `kindle` calls it, and a burning missile's `come_down` keeps its own (it adds to a
  pile). `douses` goes through `shoot.rs::put_out`, burning objects there with it.
- **Burning**: `assets/data/status.toml`; the light is the status light (`StatusDef::light`,
  `ember`, radius 1: `LightCache` sheds it for every actor). `quenched = true` is the status field
  for "water ends it": `World::quench` from `ice.rs::after_tread` (every step's one hook, the
  player's, a creature's, a displaced one's) on open water, and from `fall_in`. A knock-back or a
  teleport into water does not (no step).
- **Words**: `SpellType::words`, appended in `spell_words.rs::roll_words`, the one place the menu,
  the `spells` dump, and a wand's description read.
- **Numbing cold** is gone (its rider is cold's). The `rider` hook on `spell_mod` stays, with no
  shipped user, for a talent that wants one (`when = ["fire"]` works beside the type's own).
- **Determinism**: one `below(100)` per living creature harmed, only for a `chance` below 100,
  in `spell_lands`' order; before those, one per creature harmed with someone to arc to, past the
  certain jumps and below a chance of 100, in the order struck; the rest draw nothing. **Saves**: nothing new: a status is data, a fire
  is `LevelState::fires`, a broken door terrain.
- **Tests**: `world/spell_type_tests.rs` (the draw by seed, the fire-immune, the light, water, the
  numbing on hit and miss, cold ending Burning, a ray's one cell, a burst by its type, a spell's
  own field, the fire left and doused, a place's burst reading nothing, the rider hook, the words,
  a mishap), `world/spell_arc_tests.rs` (the arc's draw by seed, the jump back at the caster and an
  ally, a chain's jumps and end, the wall, the pond in id order, Conductor, ice, a burst, the
  `arcs` load checks), `world/spell_type_force_tests.rs` (a big hit and a small, a critical, the held, a
  burst outward and its centre, the report's order, doors locked and trapped, chests, Fireball's
  doors, thin ice and a plunge, poison and Blight's two, the bite's edge, the undead in the dump,
  Corroded's ticks), `content/spell_type.rs` (the load checks).

## Small rules

Ten spells with one rule each (#1001; the design `docs/design/spell-types-and-new-spells.md` §7
entries 4, 6, 11, 18, 21, 31, 32, 33, 35, 36; §8 for their class rows and scrolls). Every one has
its scroll (`objects/scroll_of_<id>.toml`) and a class row (`talents/<id>.toml`, granted: wizard 6
Blink, 9 Shatter; priest 18 Rebuke, 24 Benison; druid 9 Bramble, 20 Gale, 30 Cloudburst; witch 24
Leech; bard 12 Thunderclap, 20 Sense the quick). The rules live in `world/spell_rule.rs`, each
behind one line of `cast.rs`; the fields in `[spell.effect]` (`spell_def::effect`, which now takes
the shape too and refuses each field off its kind and shape).

- **`range = 0` on a burst** (Thunderclap, Benison): `SpellData::about_caster`. Allowed on a
  `damage` or a `heal` burst only, with no `range` ladder; `aimed()` is false (the menu's
  `Aim::Nothing`, a scroll read at once), `base_range()` and `Actor::spell_reach`'s range `None`
  (so no "range" in the words), `standing_aim` and `spell_aim` say `Myself`. `cast_damage` takes the
  caster's cell for the missing aim and skips `within_reach`. No cell is left out: the caster is
  in it and takes the centre's full roll (the edge's half is everyone else's at radius 1) and its
  status (Thunderclap's Stunned 1; the owner's Q14). Force's shove has no away for the one at the
  centre. Scroll savant's `spares_self` still spares a reader, as it does any own burst.
- **A heal over an area** (Benison) is only a burst about its caster (`spell_def` refuses any other
  area heal): `cast_heal_about`, one roll for the caster and every ally of theirs
  (`Ally::master`, not in the retinue) in the burst who is hurt, in actor id order, each through
  `World::spell_heals` (the touch heal's tail, said). Refused, no mana, no time, when none is hurt.
- **`leech`** (Leech; 1 to 100, a single-target damage spell's): `spell_strikes` now returns the hit
  points it took (after resistance and the bite; a kill's too), and `spell_attack`, the one place
  a single target's roll is made, gives that share, rounded down, back to the striker through
  `World::leech`, never past their maximum. Read there and not in `spell_strikes` (the design's
  words) because `spell_strikes` does not know the spell. A reflected Leech mends the reflector, who
  is its striker then. The menu's roll says `half of it yours` (`all`, `part`).
- **`only_tags`** (Rebuke; a damage or hex burst's, each tag one some creature carries,
  `spell_def::check_only_tags` once creatures load): `World::is_of_tags`. `cast_burst_at` is
  `cast_burst` with the filter (the traps', the scrolls', and Blaze's calls are unchanged); the
  untagged are not struck, so no event, no provoking. The burst's walls, ice, and type rule are its
  own all the same (Rebuke's fire on its centre). An undead ally is struck.
- **`only_terrain = "plants"`** (Bramble; an area hex's): `World::stands_on`, the `plants` flag in
  `data/terrain.toml` (grass, tall and dry grass, marsh, reeds, flowers, buttercups, bluebells,
  bush, forest floor, fungus).
- **`push`, `douses`, and `clears`** (Gale; a line hex's, `push` 1 to 3): `cast_gale`, from `cast_hex` when the
  spell has a `push` (a hex needs `status`, `push`, or `zone`). The creatures on `bolt_line` are listed in
  path order first, then each driven once, nearest first, `push` times `drive_back` straight away
  from the caster (`Dir::toward`): one with another behind it stays, one a status roots
  (`World::rooted`, shared with force's shove) stays. Each is provoked (`provoke`,
  `provoke_court`), said "driven back" where seen. Then its fires go out (`put_out`), and the zones of
  the kinds in `clears` (a line hex's only) lose the line's cells (`clear_zone_cells`). The line
  stops at an opaque cell, so a shut door stands.
- **`blink = { cells, ends }`** (Blink; a utility on `self`, `cells` 1 to 10): `cast_blink`. The free
  cells (walkable, no trap, no ice, no actor, not the caster's) within `cells` (a round reach,
  `radius2`) in cell order, through walls or not, one `rng.below(n)`; refused, no mana, with none.
  `World::teleport_to` (the teleport trap's move, split out of `teleport`), then each of `ends`
  ends (`World::ended`: the end words come with the events). Held, webbed, and pinned block `step`,
  not `cast`.
- **`weather = { kind, hours }`** (Cloudburst; a utility on `self`, 1 to 24 hours): `cast_weather`
  sets `WorldState::weather_override` to `weather::Override::lasting(kind, now, hours)`, refused
  without the open sky when `open_sky`. A wet kind (rain, heavy rain) then puts out every fire on
  an unroofed cell of a sky-exposed level and quenches (`World::quench`) everyone on one.
- **`senses = { reach, kind }`** on a status (Quick-sensed, Sense the quick's buff; `reach` 1 to 20):
  `World::heartbeats` at the end of `run_turn` pushes a `Heard` of `kind` at the player's own
  hearing threshold (so faint: up to 2 cells off) for every creature on the level within `reach`
  that is not undead, a construct (by tag), or an animated thing. Perception's `noise::perceive`
  leaves out the ones the player sees and places the rest roughly; `Exact::All` (Tracker) never
  places a heartbeat exactly. The deaf feel nothing. `SoundKind::Heartbeat`, "a heartbeat", `q` in
  the sound layer. The circled status (`data/circles.toml`, divination) sheds its circle's light,
  as every circled buff does.
- **Shatter**: data only on force's rules, with the `shatter` burst effect (`data/effects.toml`:
  white and brief); Thunderclap's look too.
- **The class test** (`spell_list_tests.rs`): a full caster's book is three to twelve spells (Q13),
  and a hybrid has at most four (the bard's Mend, Thunderclap, Sense the quick); the "low boons"
  clause is gone, as a class row is not what a spell is.
- **Determinism**: Blink's one draw, Benison's one roll, Leech's none (its strike's); the rest draw
  nothing. **Saves**: `weather::Override` gains `since` (when it was given), read by `Override::at`
  in place of `until` less three hours.
- **Tests**: `world/spell_rule_tests.rs` (the leech's half and its cap, the undead's nothing, the
  words; the blink's draw by seed, never onto a creature, out of held, webbed, and pinned, nowhere
  to go; the clap on all in it, the caster too, no aim; the heal on allies only, refused; the
  rebuke on an undead ally and foe and not the living; the bramble sparing stone; the gale's order,
  the held, the door; the rain for an hour and what it puts out; the heartbeat rough and not for
  the undead, past its reach, or in sight; Shatter on a door), `content/spell_def.rs` (the fields
  refused off their kinds and shapes), `weather.rs` (an override's window), `noise/tests.rs`.
