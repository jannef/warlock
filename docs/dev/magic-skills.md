# Schools of magic

Developer notes for the schools of magic as skills (the spell scaling design
`docs/design/spell-scaling.md` §8, decision 17; slice S8, #755). The rules are in
spec/08i-character.md, "Weapon skills": "Schools of magic". Keep it current with the code, as
CLAUDE.md says.

## The rows

- `data/weapon_skills.toml`: seven rows of `role = "school"`, after the shield skills and
  before the fighting styles, each with `school = "<id>"` (a `content::School`, the spells'
  `school`). `content/weapon_skill.rs`: `Role::School` (not implicit: one skill a school, not
  one a role), `WeaponSkillDef::school` (`Some` exactly for a school), `WeaponSkillTable::of_school`.
  The loader refuses a school row without `school`, a `school` on another role, a school with
  two rows (`of_school` would read only the first), and an unknown school (serde).
  `Content::load` refuses a spell whose school has no row (once there are weapon skills at all).
- A school row is a weapon skill in everything else: its own `school` curve (#1095), `[marks]`,
  `Training { rank, marks }` on `Actor::weapon_skills` (saved by index), `train`, `raise`,
  `progress`, `ranks_news` ("You feel more skilled at evocation."), the dev `rank:evocation:5`.
  A rank in one school raises no other.
- `Role::section` is `None` for a school: the weapon skills screen lists none, nor the caster
  styles; the Magic page (`ui/magic_screen.rs`, page 6 of the info pages, `docs/dev/screens.md`)
  lists the schools, the caster styles (Battle caster marked from `own_styles_in_use`), and the
  spells known with their rolls. `Choice::attacks` leaves schools out of `choice = "weapon_skill"`.

## What a rank gives

`Benefit` gained the spell fields, each an array of 11 and allowed for a school only (the
per-role check in `SkillSpec::benefits`): `reroll`, `sides`, `dice`, `cells`, `degrees`,
`bonus`, `mana` (those held in a `u8` 0 to 255; no other range, #1205). `speed` is allowed for a
school (still refused for the caster grips), `to_hit` and `threat` are the existing fields;
`damage` and `crit` are refused for a school (decision 16: nothing clamps a spell).

Read in one place each:

| What | Where |
|---|---|
| `reroll`, `sides`, `dice`, `bonus`, `cells`, `degrees` | `Benefit::spell_mods`, added in `Actor::spell_mods` (`world/spell_mods.rs`) after the talents' and items', unfiltered (a school's rank is for its own spells) |
| `mana` | the same `SpellMods`, as `cost = -mana` (the cost hook's percent, beside School focus's and `spell_mana_percent`; the floor of 1 holds) |
| `speed` | `World::cast_price` (`world/schedule.rs`): `Benefit::saving` off the `cast` action, times the spell's `casts`, then the burden |
| `to_hit`, `threat` | `Actor::spell_skills(content, spell, aim)` → `AttackSkills { skill: the school's, style }`, `benefit_of` in `cast.rs::spell_attack` (a spell that rolls to hit, an own cast) |

The rank is read as every weapon skill's (`Actor::benefit`, through `Actor::school_benefit`):
Schooled's and Magister's `ranks` raise a trained school's benefits; an untrained school stays
at 0. Since the hooks are `Cast::settle`'s for every cast, a scroll read or a wand used by a
ranked caster rolls with the rank's dice too (the design's §8.7, unfiltered); they train
nothing.

## How one casts: the caster styles

`world/cast_style.rs` (S10, #757; reworked by #1251, the design's §10.2 is history).
`Actor::spell_grip(content, spell)` is the caster style's grip, decided by the spell and the
armour alone, never by where the spell is aimed: `ArmouredSpell` (`worn_spell_cost` above 0:
Battle caster, whatever the spell), else for a spell that rolls to hit
(`SpellData::rolls_to_hit`: a damage spell resolved by the attack roll) `TouchSpell` (shape
`touch`: Toucher) or `RangedSpell` (a ray, a bolt: Sniper), else `None` (an area, a bolt that
never misses, a heal, a buff, a hex, a touch heal: no style). The player's casts (`cast_spell`,
`read_cast`, `wand_cast`) and the creatures' (`creature_cast.rs`) are decided alike in
`Cast::settle`; the `Cast` keeps `skills: AttackSkills` (`Actor::spell_skills(content, spell)`:
the school's skill and `style_for(spell_grip)`) for the whole cast: its marks, a rolling
spell's to-hit and its `hit` and `critical` triggers (`cast_single` takes it for an own cast),
S11's `spell_lands`.

- **The style's benefit**: `Actor::spell_mods_in_style(content, spell)` is `spell_mods` (the
  talents', the items', the school's) plus the style's `Benefit::spell_mods()` (read through
  `Actor::benefit`, so Stylist's ranks count). Sniper's `cells` (rank 5 one, rank 9 two) and
  Toucher's `bonus` (rank 4 one, 7 two, 10 three) are its only spell fields, so they reach only
  the spells that roll to hit; the loader allows `cells` for `ranged_spell` and `bonus` for
  `touch_spell` (beside a school), refuses either on any other grip, and `speed` on every caster
  grip. `Cast::settle` sums the mods once and hands them to `spell_reach_with`,
  `spell_roll_with`, and `spell_degrees_with`. The cast, the menu, `PerceptionView::own_spell_reach`,
  the dumps, and the aiming mode all read the same sum, so what is shown is what is cast.
- **A touch's reach is fixed** (#1251): `spell_reach_with` gives a `touch` spell a range of 1
  whatever the hooks say (School focus's `cells`, Sniper's, a school rank's, an item's), and no
  touch spell has a radius or arcs.
- **In use**: `Actor::styles_in_use` (`own_styles_in_use`) marks Battle caster while the armour
  worn hinders casting, never Sniper or Toucher (each spell cast decides them).
- No caster style but Battle caster teaches an ability (Spell guard, rank 4); Sniper's and
  Toucher's (Pinning ray, Staggering touch, Withering touch) were removed by #1251.
- The grips keep their ids, so no row moved.

## Marks

`World::practise_cast(cast)` (`world/cast.rs`), at the end of `cast_spell` once the cast went
off, for `Pay::Mana` only (a scroll or a wand trains nothing), never once the player is dead:
it practises the spell's school with `practise(school, landed, level)` (the one function every
weapon skill's marks go through), then the caster style.

- **The `Outcome`** (`Cast::outcome`, a `Cell`): `landed` and `level` (the strongest creature
  struck at, `None`: nobody, so the player's own level). Every kind reports through one
  function, `World::spell_landed(cast, other, landed, events)`: a buff, a cleanse, and Identify land on
  the player; a heal on whom it healed (landed when it rolled a point or more); a hex on each
  creature it fell on (landed when the status is on them after `afflict`); a damage spell on
  each creature its events say it struck at (`spell_lands`, `Event::Attacked` by the caster:
  a hit lands, a miss is an attempt weighed by the one missed; reported after the spell's
  messages and riders, in the order struck). An area is one practise, weighed by the strongest
  struck. A reflected spell reports nothing (`spell_lands`' `cast` is `None` for it).
- **The school abilities' trigger** (S11): `spell_landed` also raises `Trigger::SpellLands`
  through `World::weapon_trigger` for every landing of a `Pay::Mana` cast on a creature that
  lives, with the cast's `skills` (the school and the caster style it was cast in); see
  `docs/dev/weapon-abilities.md` "School abilities" (Thunderstruck, Wasting, Aegis, Lulling).
- **The style**: the cast's `skills.style` (above), practised after the school by the same
  outcome and level, lands or not, as a blow's style; none for a spell that rolls no attack out
  of hindering armour. Battle caster earns full marks (an attempt and a success
  as the cast landed) like the others. `spell_practised` keeps only `resolve_triggers`.
- **Talents**: `TalentMods::marks(Role::School, n)` adds `school_marks_percent` (Devotee) and
  leaves a role-less `weapon_marks` (Forge-born) out of the schools.
- No daily band, no limit, no draw: marks are counts.
- **The pace** (#1251): the schools and the caster styles train along `[curves] school`, three
  times the marks it first asked (`[30, 60, 120, 240, 480, 960, 1920, 3600, 6000, 9000]`; 450
  marks to rank 4 and 22,410 to rank 10). A cast that lands on a foe whose level is at least
  the rank earns 3 marks (`[marks]`: an attempt and a hit), so rank 1 takes 10 such casts, rank 4
  150, and rank 10 7,470 (it was 4, 50, and 2,490). Weapon skills keep their own curves.

## Talents (the `schools` system)

`[[system]] id = "schools"` after `spells` in `data/talents.toml`, `min_talents` 2: Schooled
(`choice = "school_skill"`, `content/talent.rs`'s `Choice::SchoolSkill`, the seven school rows;
`needs = { chosen = 2 }`, `Need::Chosen` reads the chosen school's rank;
`weapon { ranks = 1 }`), Devotee (level 9, `school_marks_percent = 50`,
`TalentEffect::SchoolMarksPercent`), and the wizard's Magister (level 45, `weapon { role =
"school", ranks = 1 }`). `Choice::is_weapon_skill` is the one test for the three choices of a
weapon skill table row (weapon skill, style, school skill).

## Classes

The classes' `spell_attack` grants became school grants: wizard evocation 2, druid evocation 1,
priest and bard restoration 1, witch necromancy 1, paladin abjuration 1. The ranger starts
untrained; no race or creature gives one.

## Dumps

`schools` layer (`dump.rs::school_rows`: every school, then the caster styles,
`Evocation 5 (40%)`), JSON `schools` and `caster_styles` (`id`, `rank`, `progress`); the
`weapon_skills` layer and JSON leave the schools out; the dev `progress` layer adds
`evocation marks 312/630` for each school with marks or a rank.

## Saves

Saves change: a `Training` per school on `Actor::weapon_skills` (by index), the styles moved an
index, a save's Spell attacks training dropped (the content fingerprint refuses such a save).

## Tests

`content/weapon_skill.rs` (`the_builtin_table_loads`, `rank_ten_holds_the_family_tables` with
the seven schools, `the_caster_ladder_loads_and_is_checked`: the rows, `spell_mods`, the
refusals), `world/school_skill_tests.rs` (marks per kind of cast, a miss, an area,
scrolls and wands, a rank's roll, mana, and time, Schooled, Devotee, Magister, the
dumps and `rank:evocation:5`, a school forgotten), `world/caster_style_tests.rs` (the styles
by the spell: a ray trains Sniper wherever it is aimed, a Fireball, Magic missile, and a Blessing
none, a touch that rolls to hit Toucher and a touch heal none, every spell in hindering armour
Battle caster with full marks; Sniper's cells and Toucher's bonus in the reach and the roll; a
touch's reach fixed at 1; Battle caster in use in such armour only; Sniper and Toucher teach no
ability, Spell guard; the armour's cost), `content/weapon_skill.rs`'s refusals of
`cells` and `bonus` on other grips, `world/school_ability_tests.rs` (the school abilities).
Wiki: `xtask/src/wiki/schools.rs` (`school-skills.md`, `school-benefits.md`,
`school-abilities.md`, `caster-style-benefits.md`) and `weapon_abilities.rs`
(`caster-style-abilities.md`) for `wiki/src/schools.md`, whose "Caster styles" section the
fighting styles page points to (`Term::CasterStyle`, `Term::CasterStyleAbility`: the caster
styles' anchors are `schools.md#style-ID`).
