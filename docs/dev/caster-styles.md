# Caster styles

Developer notes for the caster styles (Sniper, Toucher, Battle caster) and the Magic page. The
rules are in spec/08i-character.md, "Fighting styles". Keep it current with the code, as
CLAUDE.md says.

The schools of magic were trained skills with ranks, marks, benefits, talents (Schooled,
Devotee, Magister), and abilities (Thunderstruck, Wasting, Aegis, Lulling) until #1263 removed
them. A spell's `school` (`content::School`) is now only its category: a spell tag (Keen mind,
School focus), a class's study `affinity`, and a spell circle's colour. No weapon skill row is a
school's, and no rank anywhere lifts a school's spells.

## How one casts: the caster styles

`world/magic/cast_style.rs` (S10, #757; reworked by #1251, the design's §10.2 is history).
`Actor::spell_grip(content, spell)` is the caster style's grip, decided by the spell and the
armour alone, never by where the spell is aimed: `ArmouredSpell` (`worn_spell_cost` above 0:
Battle caster, whatever the spell), else for a spell that rolls to hit
(`SpellData::rolls_to_hit`: a damage spell resolved by the attack roll) `TouchSpell` (shape
`touch`: Toucher) or `RangedSpell` (a ray, a bolt: Sniper), else `None` (an area, a bolt that
never misses, a heal, a buff, a hex, a touch heal: no style). The player's casts (`cast_spell`,
`read_cast`, `wand_cast`) and the creatures' (`creature_cast.rs`) are decided alike in
`Cast::settle`; the `Cast` keeps `skills: AttackSkills` (`Actor::spell_skills(content, spell)`:
no weapon skill, `skill: None`, and `style_for(spell_grip)`) for the whole cast: its marks, a
rolling spell's to-hit and its `hit` and `critical` triggers (`cast_single` takes it for an own
cast), S11's `spell_lands`.

- **The style's benefit**: `Actor::spell_mods_in_style(content, spell)` is `spell_mods` (the
  talents' and the items') plus the style's `Benefit::spell_mods()` (read through
  `Actor::benefit`, so Stylist's ranks count). Sniper's `cells` (rank 5 one, rank 9 two) and
  Toucher's `bonus` (rank 4 one, 7 two, 10 three) are the only spell fields a `Benefit` has, so
  they reach only the spells that roll to hit; the loader allows `cells` for `ranged_spell` and
  `bonus` for `touch_spell`, refuses either on any other grip or role, and `speed` on every
  caster grip. `Cast::settle` sums the mods once and hands them to `spell_reach_with`,
  `spell_roll_with`, and `spell_degrees_with`. The cast, the menu,
  `PerceptionView::own_spell_reach`, the dumps, and the aiming mode all read the same sum, so
  what is shown is what is cast.
- **The aim**: a spell that rolls to hit aims with the stat, the caster style's `to_hit` and
  `threat` (`benefit_of` in `cast.rs::spell_attack`, Sniper's far benefits by distance),
  talents, and items. With the schools gone a rank-0 caster's numbers are what they were: a
  school at rank 0 gave nothing.
- **A touch's reach is fixed** (#1251): `spell_reach_with` gives a `touch` spell a range of 1
  whatever the hooks say (School focus's `cells`, Sniper's, an item's), and no touch spell has a
  radius or arcs.
- **In use**: `Actor::styles_in_use` (`own_styles_in_use`) marks Battle caster while the armour
  worn hinders casting, never Sniper or Toucher (each spell cast decides them).
- No caster style but Battle caster teaches an ability (Spell guard, rank 4); Sniper's and
  Toucher's (Pinning ray, Staggering touch, Withering touch) were removed by #1251.
- The grips keep their ids, so no row moved.
- **The time**: a cast's price is the `cast` action for each of the spell's `casts`, slowed by
  the burden (`World::cast_price`); no style quickens it.

## Marks

`World::practise_cast(cast)` (`world/magic/cast.rs`), at the end of `cast_spell` once the cast went
off, for `Pay::Mana` only (a scroll, a wand, or a flask trains nothing), never once the player is
dead: it practises the caster style the spell was cast in with `practise(style, landed, level)`
(the one function every weapon skill's marks go through); a spell cast in no style trains
nothing.

- **The `Outcome`** (`Cast::outcome`, a `Cell`): `landed` and `level` (the strongest creature
  struck at, `None`: nobody, so the player's own level). Every kind reports through one
  function, `World::spell_landed(cast, other, landed)`: a buff, a cleanse, and Identify land on
  the player; a heal on whom it healed (landed when it rolled a point or more); a hex on each
  creature it fell on (landed when the status is on them after `afflict`); a damage spell on
  each creature its events say it struck at (`spell_lands`, `Event::Attacked` by the caster:
  a hit lands, a miss is an attempt weighed by the one missed; reported after the spell's
  messages and riders, in the order struck). An area is one practise, weighed by the strongest
  struck. A reflected spell reports nothing (`spell_lands`' `cast` is `None` for it).
- Battle caster earns full marks (an attempt and a success as the cast landed) like the
  others. `spell_practised` keeps only `resolve_triggers`.
- **Talents**: `OffenceMods::marks(Role::Style, n)`: Weapon drill for its role, Forge-born's
  role-less `weapon_marks` for every row.
- No daily band, no limit, no draw: marks are counts.
- **The pace** (#1251): the caster styles train along `[curves] caster` (it was `school`),
  three times the marks it first asked (`[30, 60, 120, 240, 480, 960, 1920, 3600, 6000, 9000]`;
  450 marks to rank 4 and 22,410 to rank 10). A cast that lands on a foe whose level is at least
  the rank earns 3 marks (`[marks]`: an attempt and a hit), so rank 1 takes 10 such casts, rank 4
  150, and rank 10 7,470. Weapon skills keep their own curves. The balance probe measures it
  (`balance_probe/caster_pace.rs`, docs/balance.md "Caster style pace").

## The Magic page

`ui/magic_screen.rs`, page 6 of the info pages (`docs/dev/screens.md`): the mana, the caster
styles (Battle caster marked from `own_styles_in_use`; listed for a player who knows a spell or
has marks or a rank in one), and the spells known with their rolls. `Role::section` puts the
caster styles among the Fighting styles, but the weapon skills screen leaves out every style
whose grip casts.

## Classes

The casting classes start in a caster style: the wizard and the druid Sniper 1, the witch and the
priest Toucher 1, the paladin Battle caster 1. The bard and the ranger start untrained. No class,
race, or creature gives a school a rank (there are none).

## Dumps

The `weapon_skills` layer lists the caster styles with the other styles (`Sniper 2 (10%)`); JSON
`weapon_skills` holds them by id, and JSON `caster_styles` (`id`, `rank`, `progress`) lists the
shown caster styles alone. The `schools` layer, JSON `schools`, and the `progress` layer's
`evocation marks` lines went with the schools.

## Saves

Saves are pinned to the game version: the school rows' `Training` entries on
`Actor::weapon_skills` (saved by index) went with the rows, and the styles moved seven indexes.

## Tests

`content/weapon_skill.rs` (`the_builtin_table_loads`, `the_caster_ladder_loads_and_is_checked`:
the rows, `spell_mods`, the refusals of `cells` and `bonus` on other grips and roles),
`world/magic/caster_style_tests.rs` (the styles by the spell: a ray trains Sniper wherever it is aimed,
a Fireball, Magic missile, and a Blessing none, a touch that rolls to hit Toucher and a touch heal
none, every spell in hindering armour Battle caster with full marks; Sniper's cells and Toucher's
bonus in the reach and the roll; Toucher's ranks in the hit chance and the critical range; a
touch's reach fixed at 1; Battle caster in use in such armour only; Sniper and Toucher teach no
ability, Spell guard; the armour's cost; the classes' starting styles). Wiki:
`xtask/src/wiki/schools.rs` (`schools.md`, the schools as categories, and
`caster-style-benefits.md`) and `weapon_abilities.rs` (`caster-style-abilities.md`) for
`wiki/src/schools.md` ("Schools and caster styles"), whose "Caster styles" section the fighting
styles page points to (`Term::CasterStyle`, `Term::CasterStyleAbility`: the caster styles'
anchors are `schools.md#style-ID`; `Term::School`, `schools.md#school-ID`).
