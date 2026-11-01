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
  two rows, an unknown school (serde), and `role = "spell"` (Spell attacks, retired, with a
  message naming `school`; `SkillSpec::role` is read as text for it, `Role::from_id`).
  `Content::load` refuses a spell whose school has no row (once there are weapon skills at all).
- A school row is a weapon skill in everything else: the missile curve, `[marks]`,
  `Training { rank, marks }` on `Actor::weapon_skills` (saved by index), `train`, `raise`,
  `progress`, `ranks_news` ("You feel more skilled at evocation."), the dev `rank:evocation:5`.
  Defaults: evocation ↔ transmutation, necromancy ↔ restoration, abjuration ↔ enchantment, 4
  each; divination none. Schools default only to schools (the loader's family check: styles,
  schools, weapon skills).
- `Role::section` is `None` for a school: the weapon skills screen lists none (the Magic page,
  S9, will). `Choice::attacks` leaves schools out of `choice = "weapon_skill"`.

## What a rank gives

`Benefit` gained the spell fields, each an array of 11 and allowed for a school only (the
per-role check in `SkillSpec::benefits`): `reroll`, `sides`, `dice`, `cells`, `degrees` (0 to
3), `bonus` (0 to 6), `mana` (0 to 50). `speed` is allowed for a school (still refused for the
caster grips), `to_hit` and `threat` are the existing fields; `damage` and `crit` are refused for
a school. These ranges are the table's own field ranges (`MAX_SPELL_DICE`, `MAX_SPELL_BONUS`,
`MAX_SPELL_MANA`), not a bound on what a spell does (decision 16: nothing clamps a spell).

Read in one place each:

| What | Where |
|---|---|
| `reroll`, `sides`, `dice`, `bonus`, `cells`, `degrees` | `Benefit::spell_mods`, added in `Actor::spell_mods` (`world/spell_mods.rs`) after the talents' and items', unfiltered (a school's rank is for its own spells) |
| `mana` | the same `SpellMods`, as `cost = -mana` (the cost hook's percent, beside School focus's and `spell_mana_percent`; the floor of 1 holds) |
| `speed` | `World::cast_price` (`world/schedule.rs`): `Benefit::saving` off the `cast` action, times the spell's `casts`, then the burden |
| `to_hit`, `threat` | `Actor::spell_skills(content, spell)` → `AttackSkills { skill: the school's, style }`, `benefit_of` in `cast.rs::spell_attack` (a spell that rolls to hit, an own cast) |

The rank is read as every weapon skill's (`Actor::benefit`, through `Actor::school_benefit`):
Schooled's and Magister's `ranks` raise a trained school's benefits; an untrained school stays
at 0. Since the hooks are `Cast::settle`'s for every cast, a scroll read or a wand used by a
ranked caster rolls with the rank's dice too (the design's §8.7, unfiltered); they train
nothing.

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
  lives, with `Actor::spell_skills` (the school and the caster style); see
  `docs/dev/weapon-abilities.md` "School abilities" (Thunderstruck, Wasting, Aegis, Lulling).
- **The style until S10**: on a spell that rolls to hit, the style `spell_skills` picks, by
  the same outcome, when it struck at someone (as before: a ray at nothing trained no style); on any other cast, Battle caster when the armour worn hinders casting, an
  attempt against the player's own level (what `practise_in_armour` did; it is gone, and
  `spell_practised` keeps only `resolve_triggers`).
- **Talents**: `TalentMods::marks(Role::School, n)` adds `school_marks_percent` (Devotee) and
  leaves a role-less `weapon_marks` (Forge-born) out of the schools.
- No daily band, no limit, no draw: marks are counts.

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
`FORMAT_VERSION` untouched (the integrator's).

## Tests

`content/weapon_skill.rs` (`the_builtin_table_loads`, `rank_ten_holds_the_family_tables` with
the seven schools, `the_caster_ladder_loads_and_is_checked`: the rows, `spell_mods`, the
defaults, the refusals), `world/school_skill_tests.rs` (marks per kind of cast, a miss, an area,
scrolls and wands, a rank's roll, mana, and time, the defaults, Schooled, Devotee, Magister, the
dumps and `rank:evocation:5`, a school forgotten), `world/caster_style_tests.rs` (the styles
with the schools), `world/school_ability_tests.rs` (the school abilities). Wiki:
`xtask/src/wiki/schools.rs` (`school-skills.md`, `school-benefits.md`, `school-abilities.md`)
for `wiki/src/schools.md`.
