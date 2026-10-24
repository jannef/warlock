# Warlock specification — 7. Levels and data formats (continued): races, classes, and the rules' data

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Stat adjustments** (races, classes, the horoscope) are, per stat id, a fixed number (`2`,
  `-1`) or dice as text (`"2d3+1"`, `"1d4-1"`, `"-1d2"`; §8 "Dice"), rolled when the character
  is made; a missing stat is 0, and every result a roll can have must lie within −10 to 10.
- **Races** (`data/races.toml`, optional; §8 "Character creation"), in the order New run offers
  them: `id`, `name`, a one-line `description`, `vision` (a profile in `vision.toml`: the player's
  eyes), `age = [youngest, oldest]` (a new character's age in years, 1 to 1000), `stats`
  (adjustments to the starting scores), `potential` (adjustments to the potentials, the same way;
  hidden from the player), `hp` (its hit points, 0 to 1000, default 0; §8 "Combat"), `mana` (its
  mana, 0 to 1000, default 0; §8 "Mana"), `natural`
  (natural armour as a creature's, `{ defence, protection, name }`, each 0 to 20, not both 0;
  default none; §8 "Combat": lizardfolk's scales, Protection 1), `resist` (what the character
  resists, in percent per damage type, −100 to 100, as a creature's; default none; §8 "Damage
  types": drow poison 25), `light_blind` (`{ glare, sight, to_hit }`: eyes strong light dazzles,
  §8 "Character creation"; default none; drow 250, 10, −1), `disease_check` (points added to
  the checks against diseases, −10 to 10, default 0; §8 "Diseases": ratfolk 4), `eats_rotten`
  (default false: whether rotten food feeds the character without harm; §8 "Hunger": ratfolk),
  `alignment` (its part of a new character's alignment score, −1000 to 1000, default 0; §8
  "Alignment": elf and halfling 50, half-orc −50, goblin, kobold, lizardfolk, and ratfolk −200
  (Callous), drow and duergar −450 (Evil), the rest 0), `size` (a size
  in `data/sizes.toml`, default `medium`; §8 "Burden"), `ignores_burdened` (default false: whether
  Burdened, the first level of burden, costs the character nothing), `passive_search` (which
  hidden features the character notices in passing without the Spot skill: `"all"`, elves and
  drow, `"stone"`, dwarves and duergar, or `"traps"`, kobolds (hidden traps only); default none;
  §8 "Searching"), `search` (points added to searching and noticing alike, −10 to 10, default
  0; elves and kobolds 2), `skills` (the skills it trains,
  by skill id: dice or a number, every result 0 to 100; §8 "Skills"), `class_skill` (dice added
  to one of the class's skills, picked when the character is made; default none), `rolls` (skill
  rolls each level gives beyond the class's, 0 to 10, default 0; humans 1), `hunger` (how
  much the character needs to eat, in percent of a human, 10 to 1000, default 100; §8 "Hunger":
  elf 80, half-elf 90, human 100, dwarf 110, halfling 125, gnome 90, half-orc 120, goblin 80,
  drow 80, kobold 80, lizardfolk 90, duergar 90, ratfolk 110),
  `experience` (how much XP a level needs, in percent, 50 to 200, default 100, multiplied with
  the class's; §8 "Experience": elf 140, half-elf 105, human 100, dwarf 115, halfling 95, gnome
  120, half-orc 90, goblin 80, drow 130, kobold 75, lizardfolk 95, duergar 115, ratfolk 85;
  long-lived races pay more, and the poor choices less), and
  `weapon_skills` (starting ranks by weapon skill id, a fixed number or dice, every result 0 to
  10; §8 "Weapon skills"); and at the top
  `base_potential` (1 to 40), what every potential is before the race adjusts it (15). Races are
  mostly fixed numbers, their identity, with a little dice. The races, their eyes, ages, and hit
  points: elf (low-light, 90–300, 2), half-elf (low-light, 20–60, 3), human (normal, 16–35, 4),
  dwarf (darkvision, 40–120, 6), halfling (normal, 20–50, 2), gnome (low-light, 40–150, 3), half-orc
  (darkvision, 14–30, 6), and the evil races (#255): goblin (darkvision, 10–30, 2), drow
  (darkvision, 100–400, 2), kobold (darkvision, 6–20, 1), lizardfolk (normal, 12–50, 5), duergar
  (darkvision, 40–300, 6), ratfolk (low-light, 8–40, 2); humans adjust no score but roll `1d2` on
  every potential. Halflings, gnomes, goblins, kobolds, and ratfolk are small, the rest medium;
  dwarves and duergar ignore Burdened. Their skills (`1d6+4` each): elves listen and spot,
  half-elves bluff and spot (`1d4+2`), dwarves metalcraft and search (stonecunning), halflings
  listen and stealth, gnomes appraise and listen, half-orcs intimidate, goblins stealth
  (`1d6+6`: sneaking and hiding are one skill) and listen (`1d4+2`), drow spot and stealth
  (`1d4+2`), kobolds disable device and search (`1d4+2`), lizardfolk swim and survival
  (`1d4+2`), duergar metalcraft and stealth (`1d4+2`), ratfolk stealth and listen (`1d4+2`);
  humans none, but one extra class skill (`class_skill = "2d4+4"`).
  Weapon skills (`weapon_skills`; §8 "Weapon skills"): elf long blades 1 and bows 1, half-elf
  bows `1d2-1`, dwarf axes 1 and crossbows 1, halfling slings 1, gnome crossbows 1, half-orc
  axes 1, goblin short blades 1 and slings 1, drow short blades 1 and crossbows 1, kobold slings
  1 and spears 1, lizardfolk spears 1 and thrown 1, duergar crossbows 1 and maces 1, ratfolk
  daggers 1; humans none.
- **Status effects** (`data/status.toml`, optional; without it nothing ails anyone and resting
  heals twice as fast; §8 "Status effects", "Healing"): `[healing] resting` (how fast hit points
  come back while the player rests, in percent of the normal pace, 0 to 1000; 200) and
  `mana_resting` (the same for mana, §8 "Mana"; default 200), then the
  `[[status]]`s in order of importance (the status bar shows the first first): `id`, `name`
  (as the status bar and the character screen show it), `seen` (how look mode names it on a
  creature the player sees; none: it does not show on others), `max` (the most degrees of
  severity, 1 to 100), `stacking` (`add`, the default: a new dose's degrees add up, to `max`;
  `highest`: the stronger of the two; `refresh`: the stronger, its easing starting over),
  `tick` (seconds between ticks at one degree; `n` times as often at `n` degrees) and `damage`
  (hit points a tick costs, default 0, needing a `tick`), `eases` (seconds for it to ease by a
  degree; none: until cured), `regen` (how fast hit points come back while it lasts, in percent,
  0 to 1000, default 100), `mana_regen` (the same for mana, §8 "Mana") and `mana_damage` (mana a
  tick drains, default 0, needing a `tick`; never below 0), `penalty` (`{ from = 4, strength = -1, dexterity = -1 }`: what it
  does to scores from a severity on, each −10 to 10), `wound` (`{ big_hit = 1, critical = 2 }`:
  the degrees a big hit and a confirmed critical cause), `cured_by` (the means that end it at
  once: `bandage`), and the words: `start` and `end` (said to the player), `dies` (a creature
  the player sees dying of it: "bleeds to death"), `death` (the death screen's line: "Bled to
  death"). Built in: **bleeding** (`seen` "bleeding", max 5, a hit point every 40 s per degree,
  a degree eased a minute, no healing while it lasts, a big hit 1 degree and a critical 2, cured
  by a bandage), **poisoned** (max 10, a hit point every 2 minutes per degree, a degree eased
  every 2 minutes, healing at half the pace, Strength and Dexterity −1 from 4 degrees on; it does
  not show on others), and **diseased** (what every disease shows as, `data/diseases.toml`'s
  `status`: max 1, healing at half the pace, no words of its own, "succumbs to disease", "Died of
  disease"). The status diseases show as may have no `tick`, `eases`, `wound`, or `cured_by`
  (diseases give it and take it away), and no `on_hit` names it.
- **Diseases** (`data/diseases.toml`, optional; without it nobody falls ill; §8 "Checks",
  "Diseases"): `status` (the status every disease shows as, required with any disease:
  `diseased`), then the `[[disease]]`s in file order: `id`, `name` (as the character screen
  names it once known: `filth fever`), `difficulty` (what a check must reach to resist it, 1 to
  50), `check` (the stat whose modifier the checks add, default `toughness`), `incubation` (days
  from catching it to its onset: dice or a number, at least 1, at most 365, whatever it rolls),
  `interval` (days between progressions, default 1, 1 to 365), `damage` (stat damage a failed
  check does: `{ dexterity = "1d3", toughness = "1d3" }`, each 1 to 40 whatever it rolls), `hp`
  (hit points a failed check costs: dice, 1 to 1000; none by default; a failed check must do
  one or the other), `recovery` (checks won in a row that shake it off, default 2; 0: never by
  itself), `kills` (default false: whether its damage may take Toughness to 0 or the last hit
  point, which kills), `regen` (how fast hit points come back while it shows, in percent, when
  slower than its status's; 0 to 1000), `mana_regen` (the same for mana; the built-in brain
  fever 25), `caught` (the sources rules expose to it by name:
  `rotten_food`), `wading` (`{ terrain = ["marsh", "swamp"], per_hour = 4 }`: the terrain the
  player catches it in, a chance in percent for each hour spent crossing it, 1 to 100),
  `cured_by` (the means that cure it at once: `remove_disease`; nothing applies one yet), and the
  words, all required: `onset` (said as it shows), `symptoms` (said at the progressions that go
  badly, in turn, the first again after the last), `recovered` (said as it leaves). Built in, our
  own in the D&D 3.5 spirit (a check of 2d12 + modifier against the difficulty): **filth fever**
  (difficulty 13, `1d3` days, `1d3` Dexterity and `1d3` Toughness; from the giant rat's bite),
  **red ache** (16, `1d3` days, `1d6` Strength), **the shakes** (14, 1 day, `1d6` Dexterity),
  **marsh ague** (14, `1d4` days, `1d3` Toughness, healing at a quarter of the pace; caught
  crossing marsh (the ground of swamp areas) and the overworld's swamp, 4 % an hour), **grave
  rot** (15, 1 day, `1d4` Toughness and `1d4` Presence, never by itself, kills; from rotten food,
  later the undead), and **brain fever** (Willpower against 13, `1d3` days, `1d3` Intelligence
  and `1d3` Willpower); all recover after two checks won in a row but grave rot, and all are
  cured by `remove_disease`. Red ache, the shakes, and brain fever have no source yet.
- **Sizes** (`data/sizes.toml`, optional; §8 "Burden"), in file order, smallest first (§8
  "Displacement" compares them): `id`, `name`, `carry`
  (what it carries in percent of a medium one, 10 to 1000): small 75, medium 100; and `blood`
  (how much blood it spills in percent of a medium one, 0 to 1000, default 100; §8 "Blood"):
  small 60. `medium` is
  required (the size of every race and creature type that names none); without the file there
  is only medium.
- **Hunger** (`data/hunger.toml`, optional; §8 "Hunger"; without it nobody hungers, nothing can
  be eaten, food does not rot, and creatures leave no corpses): `burn_seconds` (game seconds per
  nutrition point burned at 100 % hunger, 1 to 86 400: 60), `start` (nutrition at the start of a
  run: 3600), `max` (the most there can be: 6500), `vomit_leaves` (the most left after
  vomiting: 1000), `sickness` (the status id, `data/status.toml`, that harm from eating gives:
  `poisoned`; needed when anything eaten harms), `[states]` (the least nutrition of `bloated` 5500, `satiated` 4000, `not_hungry` 1440,
  `hungry` 720, `weak` 240, and `fainting` 0, strictly falling and below `max`; below
  `fainting` the player is Starving; `start` and `vomit_leaves` lie from `fainting` to `max`),
  `[bloated] slow` (percent of an action's price while Bloated, 100 to 1000: 125), `[weak]
  stats` (what Weak, Fainting, and Starving take from Strength and Dexterity, only those two,
  −10 to 10: −1 each), `[fainting]` (`chance`, percent each turn, 0 to 100: 5; `turns` lost, 1
  to 100: 5), `[starving]` (`damage` hit points, 1 to 1000: 1, every `seconds` of game time, 1
  to 86 400: 600), `[exertion]` (extra points the player's `melee`, `shoot`, and `throw` burn,
  0 to 1000: 2, 1, 1; no other action), `[overeating]` (`choke`, the percent chance a meal past
  `max` chokes the eater to death: 0, off; `vomit`, else the chance they vomit: 50), `[rotten]`
  (what eating rotten food of any kind does: `vomit`, percent: 50; `harm`, dice of degrees of
  sickness, 0 to 1000: `1d4`), and `[corpses]` (optional; without it creatures leave none:
  `material` (`flesh`), and `[corpses.size]` with every size of `data/sizes.toml` and what its
  corpses give and how long they keep, `{ nutrition, turns, keeps, lasts }` as for food (§7
  "Objects"): small 400 and 3, medium 1000 and 6, both keeping `"8h"` and lasting `"36h"`). The
  wiki shows its tables (the states, the races' needs, food, corpses) generated.
- **Classes** (`data/classes/`, one file per class named `NN-id.toml` so path order is the order
  offered; optional): `id`, `name`, a one-line
  `description`, `stats` (adjustments as for races), `hp` and `mana` (as for races; half the
  class's mana comes with each level up to `mana_level`), `experience` (as for
  races, multiplied with the race's; §8 "Experience": wizard 125, monk 120, priest, druid, and
  witch 115, paladin 110, ranger 105, warrior and bard 100, thief 95, barbarian, aristocrat,
  craftsperson, and merchant 90, commoner 80), `alignment` (as for races, added to the race's:
  paladin 450, Good; thief and witch −200, Callous; the rest 0), `skills` (as for races),
  `known_for` (the skills of `skills` it is known for, its `2d6+10` ones: a level-up roll on one
  throws its die a step up the ladder), `rolls` (skill rolls each level gives, 0 to 10, default
  2: the thief and the ranger 3, the bard 4), `weapon_skills` (starting ranks as for races), `weapon_skill_choice` (two or more
  sets of such ranks, one picked at random when the character is made), and `styles` and
  `style_choice` (the same for fighting styles, §8 "Fighting styles"; `weapon_skills` names no
  style and `styles` nothing but styles), `abilities` (the level table: `{ level, talent }`
  rows, lowest level first, each a talent of `data/talents/` the class gives at that level, §8
  "Class abilities"), and `bonus_talents` (`{ levels, trees }`: a bonus talent pick at each of
  those levels from those trees, none named: any). Each class rolls
  its key stats (the warrior St `1d3+1` and To `1d3`, the wizard In `2d2+1` and To `-1d2`, the
  commoner To `1d3-1`); the rest are fixed. Adventurers: barbarian (8 hit points), bard (2),
  priest (4), druid (4), monk (5), paladin (6), ranger (5), thief (3), wizard (0), witch (1),
  warrior (7); everyday folk: aristocrat (1), commoner (3), craftsperson (3), merchant (2). A
  class adjusts the starting scores, gives hit points, sets how fast levels come, trains skills,
  gives starting ranks in weapon skills and fighting styles, and, with a level table, class
  abilities and bonus talents. Skills: a few it
  is known for (`2d6+10`) and a few more (`2d4+4`); the full table is in the wiki (Skills), e.g.
  the thief open lock, steal, stealth, and tumble well, appraise, disable device, escape artist,
  search, and spot a little; the ranger spot, survival, and woodcraft well, athletics, handle
  animal, listen, and swim a little; the wizard concentration, knowledge, and spellcraft well,
  decipher script and use magic device a little; the commoner athletics and handle animal a
  little. Weapon skills: barbarian unarmed 1 and axes 2 or great blades 2, bard short blades 1
  and daggers `1d2`, priest maces 2 and shield blocking 1, druid staves 2 and slings 1, monk
  unarmed 2 and staves 1, paladin long blades 2, shield blocking `1d2`, and shield bashing 1,
  ranger bows 2 and short blades 1, thief daggers 2 and thrown 1, wizard staves 1, witch daggers
  1 and staves 1, warrior long blades 2 and shield blocking `1d2`, aristocrat long blades 1,
  commoner maces 1, craftsperson axes 1, merchant daggers 1. Fighting styles: barbarian great
  weapon 1, bard duelist 1, priest sword and board 1, druid staff fighter 1, monk brawler 1 or
  staff fighter 1, paladin sword and board 1, ranger marksman 1, thief twin blades 1 or duelist
  1, warrior sword and board 1, aristocrat duelist 1, commoner brawler 1; the wizard, witch,
  craftsperson, and merchant none.
- **Alignment** (`data/alignment.toml`, optional; without it nobody has a score, everyone counts
  as Neutral, and nothing moves it; §8 "Alignment"): seven `[[band]]`s, best first (`id` saintly,
  good, kind, neutral, callous, evil, wicked, in that order; `from`, the least score in the band,
  each below the one before and wicked's −1000; a one-line `description` for the wiki; `rise`,
  said when the score rises into the band, every band but wicked; `fall`, said when it falls into
  it, every band but saintly), `[shift]` (what each deed is worth before it diminishes, −1000 to
  1000: `kill_peaceful` and `kill_hostile` by the victim's alignment, `{ good, neutral, evil }`;
  `kill_beast`, `attack_shopkeeper`, `theft`, `pay_debt`, `eat_kin`), and `[shops] favour` (how
  much better a good shopkeeper's prices are for each band, in percent, −50 to 50; a band not
  listed 0).
- **Skills** (`data/skills.toml`, optional; without it nobody is trained in anything, and any
  `skills` elsewhere is an error; §8 "Skills"): `[[skill]]`s in the order the Skills screen lists
  them (alphabetical), each with `id`, `name`, a one-line `description`, `kind` (`passive`,
  `action`, or `rules`: what it is for; no rule reads it), `mark` (when it earns training marks:
  `{ on = USE, chance = PERCENT }`, 1 to 100; uses `sneak`, `hear`, `dodge`, `burdened`,
  `travel`, `steal`), and `benefit` (what it gives by its value: `{ effect = EFFECT, steps = [20, 40],
  every = 25 }`, one for each value reached and one more for every `every` points past the last,
  or `{ effect = EFFECT, curve = [[0, 0], [100, 60]] }`, straight between points from value 0,
  rounded down, level after the last; effects `dodge`, `quiet`, `hearing`, `burden`, `travel`,
  `steal` (points added to stealing, at most 20; trained, it lets the player steal: §8
  "Stealing"), the percent ones within 0 to 100). At the top, `[ladder]`: `dice` and `marks`, one per step
  (the die a level-up roll throws from that many marks on; marks from 0, rising; each die adds at
  least 1; default ADOM's `+1` … `4d5` from 0 … 800 marks), and `[rolls]`:
  `intelligence_from`, `intelligence_every` (one more roll at that Intelligence and every so many
  points above; 14, 4), `spend` (percent of the marks a roll spends, 50), `soft_cap` (from this
  value on a roll adds half, 100); §8 "Skills". Adding a skill is one
  entry (saves then change: skill ids are indices). The 26 skills: appraise, athletics,
  balance, bluff, concentration, decipher script, disable device, escape artist, handle animal,
  heal, intimidate, knowledge, listen, metalcraft, music, open lock, search, spellcraft, spot,
  steal, stealth, survival, swim, tumble, use magic device, woodcraft.
- **Horoscope** (`data/horoscope.toml`, optional; §8 "Character creation"): `anchor_year`, a
  year that starts a great cycle (1: the year of the Great Race, the first of the reckoning),
  then the `[[animal]]`s in the order of the Great Race and the `[[element]]`s, each with `id`,
  `name`, a one-line `description`, and `stats` (small fixed adjustments). Their numbers must
  have no common divisor, so every pair comes once a great cycle. The twelve animals:
  Salamander, Hydra, Manticore, Unicorn, Wyvern, Basilisk, Roc, Griffin, Sphinx, Phoenix,
  Kraken, Barghest; the five elements: Wood, Fire, Earth, Metal, Water. Without the file there
  are no signs.
