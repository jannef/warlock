# Warlock — The priest: weak in a fight, master of roguelike housekeeping

Design document for the owner's review (2026-11-02, #820). Written against `origin/main` at
f3846261 (encounters E5). Nothing here is implemented; §11 cuts it into issues. It builds on
what is there: the class tables and the coverage policy (`docs/dev/progression.md`, rules 1–9 of
#776), the spell data and the one cast path (`docs/dev/spells.md`, `docs/dev/spell-scaling.md`
§7.1's hooks, the restoration school's rows, `docs/dev/magic-skills.md`), statuses, diseases,
and treating oneself (`docs/dev/status-effects.md`), drain and stat damage (`docs/dev/stats.md`
"Drain"), hunger and rot (`docs/dev/hunger.md`), the temple's services (`docs/dev/npcs.md`,
`world/service.rs`), special rooms and Consecrate (`docs/dev/special-rooms.md`), and the gaps
report (draft PR #802, `docs/design/class-ability-gaps.md`). Every number is data and hidden in
the game but where the menu already shows it (a spell's mana and dice, as a weapon's); the wiki
states it (the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-11-02, #820, verbatim): "lean into it being generally pretty shit,
but having tools to heal/cure/get rid of poison/create food. Kinda having the tools to overcome
all the traditional rogue like housekeeping issues, but not any extra ability to fight."

## Contents

1. Summary
2. Research, tied to the decisions
3. The housekeeping of Warlock: what bites, and what answers it today
4. What exists: the priest's table and class file
5. The proposal: the table, level by level
6. The new spells and the new active, as data
7. Determinism, saves, replays
8. What the player sees: the menu, messages, dumps, the wiki
9. Talents and class abilities (the coverage policy)
10. Rejected ideas
11. Implementation plan (slices)
12. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **The fantasy** | The priest is a poor fighter by numbers (Strength −1, Dexterity −1, 4 hit points a level, a mace and a buckler) and the table gives them **nothing for a fight**: no blow, no attack spell, no buff to hit. What it gives is an answer to every housekeeping problem a roguelike has: wounds, bleeding, poison, hexes, disease, stat damage and the permanent drain of the undead, hunger, cursed places, and a safe place to rest. Light is already theirs (the kit's scroll of light teaches Light for certain at level 1); identification is the merchant's (the owner). §5 |
| **Goes** | **Smite** (40): a blow is a fight ability, and it is the paladin's (#823 owns smiting). **Blessing** (3): +1 to hit is a fight buff; the paladin and the bard keep it, and a priest may still learn it from its scroll (abjuration affinity +2). **Purify** (25) as an entry of its own: its effect moves into Daily bread's talent (one rotten-food rule beside the spell that makes rot a non-issue). §5, Q1 |
| **Stays** | Mend (1), Cure wounds (3), Consecrate (9), Remove disease (12), Purity of body (32), Miracle (50), Saint (60). The kit, stats, skills, and weapon ranks as they are (Q6). §4, §5 |
| **New** | **Daily bread** (6): a utility spell that makes a wafer of waybread (1000 nutrition, worth nothing) for 6 mana, and Purify's rule. **Balm** (18): a cleanse that ends bleeding, poison, every hex, and whatever holds or webs the caster. **Restoration** (25): a minute's rite for 40 mana that mends all stat damage and restores every drained point: the temple's mending as a spell. **Sanctuary** (40): a minute's rite for 30 mana that hallows the ground within 2 cells for an hour: no hostile steps in, wounds and mana come back at a sanctuary's pace, and the peace breaks if the priest strikes from inside it. §5, §6 |
| **The table** | 1 Mend · 3 Cure wounds · 6 Daily bread · 9 Consecrate · 12 Remove disease · 18 Balm · 25 Restoration · 32 Purity of body · 40 Sanctuary · 50 Miracle · 60 Saint. Every standard level of the cadence (1, 6, 12, 18, 25, 32, 40, 50, 60) filled; the off-cadence 3 and 9 kept, as the gaps report notes they are on purpose. §5 |
| **Spells, not cooldowns** | The three new spells are `[[spell]]` files of the restoration school cast by the one cast path: they cost mana, train restoration (whose rows already make heals cheaper and quicker), have ladders and scrolls, and reach the hooks the talents and items use. Sanctuary is a class active built as Consecrate is (a rite with `mana` and `time`), since it lays a place, not a status. §6 |
| **Data kinds** | `utility` gains `creates = { object, count }` beside `identify`; `cleanse` gains `mends` (stat damage) and `restores` (drained points) beside `cures` and `diseases`; a new talent effect `sanctuary { mana, time, radius, lasts }` and `WorldState::hallows`. One read each. §6, §7 |
| **Talents** | **None new**: every entry extends a system that has its talents (`hit_points`, `status`, `diseases`, `hunger`, `special_rooms`, `spells`, `mana`). The one new rule (a hallowed place) is a sub-feature of `special_rooms` (a sanctuary the player lays), no tag of its own. §9 |
| **Slices** | P1 the three spells and the table (rules, data, scrolls, dumps, tests); P2 Sanctuary (the hallow, the AI hook, the pace hook, the break); P3 the wiki, the spec, the balance probe's priest column. §11 |

**Key choices, and why**

1. *Housekeeping is the class, so every entry answers a problem the player would otherwise solve with gold, luck, or a walk to town.* §3 lists what bites in Warlock today and what answers it; the table gives the priest one spell per row, in the order the problems arrive (hunger by day two, disease by the first rat bite, drain by the barrows, the long rest by the deep floors).
2. *Spells, because the machinery is there.* A spell has a cost, a school that trains, ladders for growth, a scroll others may find, and the six hooks; a bespoke active has none of that. Only Sanctuary is an active, because it makes a place, and the cast path makes rolls and statuses.
3. *The temple's services stay the non-priest's way, and the priest's way is cheaper only in gold.* Restoration costs 40 mana and a minute; the temple's mending 40 gold and a walk. Both exist; neither is removed (no economy capstone, rule 1).
4. *Sanctuary is a rest, not a fortress.* It is the one entry that touches creatures, and it does so by keeping them out, not by hurting them; striking from inside ends it, so it cannot be a turret. It passes rule 1 (a capstone changes what the character can do in a dungeon).
5. *Nothing to fight with, down to Blessing.* The owner's "not any extra ability to fight" is read literally: Smite goes, and Blessing (+1 to hit) is proposed out too, as a question (Q1). What the priest can do in a fight is what any character can: swing the mace, read a scroll, run.

## 2. Research, tied to the decisions

| Source | What it does | Taken | Left, and why |
|---|---|---|---|
| **NetHack's Priest** (nethackwiki.com/wiki/Priest, fetched) | starts with a blessed mace, a small shield, four potions of holy water, garlic, wolfsbane, two clerical spellbooks; **always knows an item's beatitude** ("Priests have inherent knowledge of any item's beatitude"); special spell Remove curse; `#turn` undead ("leaves the Priest immobilized for a few turns"); intrinsics warning (15) and fire resistance (20). The role's strength is curse housekeeping: holy water "allows reliable uncursing" | the mace-and-buckler start (kept, §4); the idea that the class's power is *knowing and undoing the dungeon's nuisances*, not hitting (§5); a cleanse of hexes as the nearest thing to Remove curse (Balm, §6.2) | Warlock has no cursed items and no beatitude, so no B/U/C sight; identification is the merchant's (the owner); turn undead is a fight ability |
| **DCSS's Elyvilon** (crawl-ref `ability.txt`, fetched) | Purification "purges your body of poison, confusion, slowness, petrification, weakness, and sickness" and restores drained maximum health; Heal Self; the god pacifies and saves | **one cleanse of everything that ails** (Balm's list: bleeding, poison, every hex, holds; §6.2) and **a rite that undoes drain** (Restoration, §6.3) | pacifying is a fight tool; the life-saving is Miracle already |
| **DCSS's Zin** (`ability.txt`, `gods.txt`, fetched) | Vitalisation "temporarily provides protection from poison, confusion, petrification, and sickness"; **Sanctuary** "transforms the area around the user into a domain of unbreakable law for a short duration" (in play: monsters will not enter, and attacking from within ends it) | **Sanctuary at 40** (§6.4): a hallowed radius hostiles will not enter, broken by the priest's own blow | the tithe (gold is not a class mechanic here); Recite and Imprison (fight tools) |
| **DCSS's trend** (0.24 removed remove-curse scrolls, 0.26 removed hunger) | the modern answer to housekeeping is to delete it | nothing: Warlock keeps hunger, rot, disease, drain, and unholy rooms as the spec has them, and gives one class the tools instead. That is the owner's call in #820 | — |
| **ADOM's Priest and Healer** (ancardia wiki unreachable, HTTP 402; from memory, unverified) | the Priest starts with Healing and knows an altar's alignment; its class powers lean on prayer and piety (cheaper prayers early, turn undead later). The Healer is ADOM's housekeeping class: Healing from the start, faster regeneration, and class powers that neutralise poison and cure sickness and disease at fixed levels | the Healer's shape, not the Priest's: fixed-level answers to poison, disease, and regeneration (§5), and the Heal skill from level 1 (the priest's `heal = 2d6+10`, already there) | prayer and piety: Warlock's prayer is the forgotten altar's outcome table for everyone (`data/outcomes/forgotten_altar.toml`); the priest gets no private piety |
| **D&D 3.5's cleric list** (recalled) | Purify Food and Drink 0; Sanctuary 1st; Delay Poison 2nd; Create Food and Water, Remove Disease, Remove Curse 3rd (cleric 5); Neutralize Poison, Restoration 4th (cleric 7); Greater Restoration 7th | **the order**: food and disease early, poison and hexes next, restoration after (§5); Sanctuary as a cleric's, not a wizard's | raise dead: permadeath |
| **Warlock's own temple** (`campaigns/ascmearc/data/dialogue/ceaptun.toml`) | Sister Hild's `cure` (60 gold: every disease and poison) and `mend` (40 gold: all stat damage, every drained point) | Restoration does exactly `World::mend` (§6.3), Remove disease exactly `cure`; the priest carries the temple | the prices stay: the non-priest's way |

## 3. The housekeeping of Warlock: what bites, and what answers it today

Read from the data on `main`. The right column is what the proposal adds.

| Problem | How bad it is | Answers today | The priest's answer (level) |
|---|---|---|---|
| **Wounds** | 1 hit point back per max(60, 360 − 12 × Toughness) s (Toughness 10: 4 minutes), × 2 resting, × 3 in a sanctuary; none while Starving | rest; potions of healing (2d8, level 1, frequency 60); Second wind (talent); the altar's boons | Mend (1, 1d3 to 1d6), Cure wounds (3, 2d6 to 3d6+3, two casts of time), Saint (60) |
| **Bleeding** | 1 hp per 40 s a degree, up to 5 degrees, eases a degree a minute; from slashing and piercing big hits and criticals | the Heal skill (an Intelligence check against 6 + 2 a degree, 2.5 turns, retry at will), the bandage means | the Heal skill from level 1 (`heal = 2d6+10`, known for); Balm (18) ends it at once |
| **Poison** | 1 hp per 2 min a degree, up to 10, eases a degree per 2 min, Strength and Dexterity −1 from 4 degrees, healing and mana at half pace; 26 of 295 creature types poison | potion of antidote (60 gold, level 2); the Heal skill once per dose; Divine grace and Nature's ward (eases twice as fast) | Remove disease (12), Balm (18) |
| **Hexes and holds** | Ill-wished, Weakened (Strength and Dexterity −2), Sluggish (×1.5 time), Drowsy (×3), Webbed and Held (no step): from altar banes, mishaps, spiders, the drowning grip | wait them out (60 s to 2 min a degree); a Strength check against a hold | Balm (18) |
| **Disease** | 9 diseases; 26 creature types carry one; rotten food carries grave rot (never leaves by itself, kills); marsh ague from wading; a failed check a day takes 1d3 to 1d6 off a stat; Diseased halves healing and mana | angelica root (30 gold, forage), potion of cure disease (150 gold, level 4), the temple's cure (60 gold), the Heal skill's try; Purity of body at 32 | Remove disease (12), Purity of body (32) |
| **Stat damage** | a disease's; mends a point a day once nothing shows | the temple's mend (40 gold) | Restoration (25) |
| **Drain** | **for good**: 36 of 295 creature types drain a stat on a hit (wights, wraiths, mummies, the hags: Toughness, Strength, Intelligence, Presence, 1 or 2 points at a time against a check of 16 to 24); the score falls to a floor of 3, hit points and mana with it | the temple's mend (the only restore); Unbroken soul, Divine grace, Iron will (the checks) | Restoration (25) |
| **Hunger** | a human burns 1440 a day, starts at 3600 (hungry after a day and a half); Weak takes Strength and Dexterity; Starving takes 1 hp per 10 min and stops healing; food rots (bread 5 days, a ration 30) | buy food (a ration 5 gold, 1500), forage (Survival), eat corpses (rot, disease), the altar's "sated" | **Daily bread (6)**: a wafer (1000) for 6 mana |
| **Rot** | rotten food: 50 % vomit, else 1d4 degrees of poison and a disease check (grave rot) | Iron stomach, Herb lore, Purify (priest 25), Cauldron | Purify's rule, in Daily bread's talent (6) |
| **Cursed places** | cursed rooms raise the dead you leave, haunted rooms bring undead every hour of a rest, draining rooms (and whole levels) take a point of mana every 20 s | Consecrate (priest 9, paladin 12) | Consecrate (9) stays |
| **The long rest** | a floor with a danger spawns a creature every 15 min at 60 %; a rest stops when one comes into view; a haunted room's hour | a sanctuary room (shunned, × 3), Quiet delver, Consecrate on the haunt | **Sanctuary (40)** |
| **Light** | every race without darkvision starts with a lantern (never runs dry; snuffed by an altar bane); Light is a level-0 spell (`lit`, half an hour a degree) | the lantern, the scroll of light | the kit's scroll of light: studied at level 1 (a level-0 spell at caster level 1 is certain), the priest knows Light for 1 mana. Nothing added (§10) |
| **Identification** | unknown scrolls, potions, wands, magic items | Lore talents, the scroll and spell of identify, the appraiser (100 gold a kind) | nothing: the merchant's (the owner, #776 rule 8 and the Eye for quality line) |
| **Death** | permadeath | Miracle (50), Rise from the ashes (a birth sign) | Miracle (50) stays |

Not in Warlock, so not here: cursed items, blindness, confusion, lycanthropy, level drain, item destruction by fire or water, running out of lamp oil.

## 4. What exists: the priest's table and class file

`assets/data/classes/03-priest.toml`: Strength −1, Dexterity −1, Intelligence +1, Willpower 1d3+1, Presence 1d2; `hp = 4`, `mana = 9`, `casting = 100`; affinity restoration +3, abjuration +2, divination +1, necromancy −2; skills Heal 2d6+10, Knowledge 2d6+10, Concentration 2d4+4, Decipher script 2d4+4; weapon skills maces 2, shield block 1, restoration 1; styles sword and board 1, toucher 1; the kit a mace, a buckler, a wool cloak, an amber amulet, a scroll of light, three candles, bread, 10 gold; bonus talents from Vigour and Grit at 10, 20, 30, 45, 55.

| Level | Today | Effect (data) | Fate |
|---:|---|---|---|
| 1 | Mend | spell `mend`: touch heal 1d3 (Willpower 16: 1d4, 22: 1d6), 1 mana | **stays** |
| 3 | Blessing | spell `blessing`: Blessed 10 min a degree (+1 to hit, Willpower +1), 3 mana | **goes** from the priest's table (Q1); the paladin (6) and bard (6) keep it; the scroll of blessing teaches it |
| 3 | Cure wounds | spell `cure_wounds`: touch heal 2d6 (16: 3d6, 22: 3d6+3), two casts, 2 mana | **stays** |
| 9 | Consecrate | active, 15 mana, a minute: lifts an unholy room's effect for good (the level's if it pervades), `alignment min = kind`, Consecrated +20 | **stays** |
| 12 | Remove disease | spell `remove_disease`: cleanse, every disease and Poisoned, 11 mana | **stays** |
| 25 | Purify | `purify`: rotten food neither sickens nor exposes | **moves** into Daily bread's talent at 6; the entry's level goes to Restoration |
| 32 | Purity of body | `disease_immune` (shared with the paladin 18 and the monk 32) | **stays** |
| 40 | Smite | a blow of holy wrath (shared with the paladin 12) | **goes**: a fight ability, and #823's |
| 50 | Miracle | `reprieve` once: a killing blow leaves 1 hp, Worn thin a day | **stays** |
| 60 | Saint | `healing_percent` +100 | **stays** |

Gaps (the report): 6 (never had one), 18 (Ward of faith, Deflection +1, cut by #798 as a weak defence passive). Both were defence or nothing; the proposal fills them with housekeeping.

## 5. The proposal: the table, level by level

Bonus talents (Vigour, Grit) at 10, 20, 30, 45, 55 unchanged. Skill rolls 2 unchanged.

| Lv | id | Name | Kind | What it does (the game's words) | Data | Problem (§3) |
|---:|---|---|---|---|---|---|
| 1 | `mend` | Mend | spell (restoration, touch heal) | "A touch that closes a scratch: a few hit points back, yours or another's beside you." | as today | wounds |
| 3 | `cure_wounds` | Cure wounds | spell (restoration, touch heal) | as today | as today | wounds |
| 6 | `daily_bread` | Daily bread | spell (restoration, utility) **+ `purify`** | "Pray, and bread is given (a spell, paid in mana): a wafer of waybread in your pack. And rotten food neither sickens you nor carries a disease into you." | spell level 6, `creates = { object = "waybread", count = 1 }`, count ladder Willpower 18: 2, 26: 3; talent effects `[{ kind = "spell", spell = "daily_bread" }, { kind = "purify" }]` | hunger, rot |
| 9 | `consecrate` | Consecrate | active | as today | as today | cursed places |
| 12 | `remove_disease` | Remove disease | spell (restoration, cleanse) | as today | as today | disease, poison |
| 18 | `balm` | Balm | spell (restoration, cleanse) | "A balm of prayer on body and mind (a spell, paid in mana): bleeding stops, poison and every hex leave you, and whatever holds or webs you lets go." | spell level 7, `cures = ["bleeding", "poisoned", "ill_wished", "weakened", "sluggish", "drowsy", "webbed", "held"]` | bleeding, poison, hexes, holds |
| 25 | `restoration` | Restoration | spell (restoration, cleanse) | "A long rite of restoring (a spell, paid in much mana; it takes a minute to say): what sickness took from your body is mended, and what the dead drank from you is given back." | spell level 20, `mana = 40`, `casts = 6`, `mends = true`, `restores = true` | stat damage, drain |
| 32 | `purity_of_body` | Purity of body | passive | as today | as today | disease |
| 40 | `sanctuary` | Sanctuary | active (a rite) | "Kneel and hallow the ground about you (an ability, paid in mana; a minute's rite): for an hour no foe will set foot within two paces of where you knelt, and your wounds and your mind mend there as in a sanctuary. The peace breaks if you strike from within it." | `{ kind = "sanctuary", mana = 30, time = "1m", radius = 2, lasts = "1h" }` | the long rest |
| 50 | `miracle` | Miracle | passive | as today | as today | death |
| 60 | `saint` | Saint | passive | as today | as today | wounds |

**Why these levels.**

- *6, Daily bread*: hunger is the first problem (a day and a half in), and a level-6 priest's pool (about 49 mana, `docs/dev/spells.md` "Identify") buys eight wafers from a full pool: 8 000 nutrition, five and a half days for a human. The spell is a day's bread for about twenty minutes of regeneration (6 points at a point per 204 s at Willpower 13), and the restoration rows cut its mana from rank 4. A wafer is worth 0 gold (`waybread.toml`), so a shop buys none: no gold from bread. No cap and no cooldown (the owner: no anti-farming limits; time is the price; a stack of wafers weighs 100 g each, the burden is the only limit).
- *18, Balm*: the gap. Bleeding and poison have the Heal skill from level 1 and Remove disease from 12; what has nothing until here is the hexes and the holds, and a sure, one-turn end to bleeding in a fight (the Heal skill's try is a check and 2.5 turns). Its level is 7 (7 mana, a scroll of it teaches at caster level 7): the class gives it at 18, late for its power, which is the shape of a class whose table is all housekeeping and whose owner wants it "pretty shit".
- *25, Restoration*: drain is the housekeeping problem Warlock makes worst (36 types, for good, a floor of 3) and answers least (one sister in one town). A priest of 25 has about 180 mana (interpolating the probe's 90 at 12 and 216 at 30): 40 is a fifth of it, and a minute kneeling in a dungeon is a risk. Earlier would make the barrows' wights toothless to a priest of 12; later would be past the levels where a drained Toughness ends runs.
- *40, Sanctuary*: the capstone-grade entry (rule 1: it changes what the character can do in a dungeon, and only there), from Zin. Rest is the priest's whole economy (mana and hit points come back with time), and the deep floors make resting the dangerous part. 30 mana and a minute for an hour of peace with no spawn reaching the circle; the regen × 3 is the sanctuary room's own number.
- *3 and 9 kept off-cadence*: the report counts them as deliberate; moving Cure wounds to 6 would leave a level-3 priest with Mend alone and push Daily bread to 12, past the first hungry days.

**What the priest cannot do**, so the owner sees the shape: no attack spell of their own (Spark, Magic missile, Ray of frost are the others' tables'; scrolls may teach them, at necromancy ↔ restoration −4 and no evocation affinity); no blow (Smite gone); no buff to hit (Blessing gone, Q1); no summon (the druid's, #821); no stealth, no pet, no voice. In a fight they have a mace at rank 2, a buckler, 4 hit points a level, and the Toucher style, which is for Mend and Cure wounds on someone beside them.

## 6. The new spells and the new active, as data

### 6.1 Daily bread

```toml
# assets/data/spells/daily_bread.toml
[[spell]]
id = "daily_bread"
name = "Daily bread"
description = "Pray, and bread is given: a wafer of waybread in your pack."
level = 6
school = "restoration"
kind = "utility"
shape = "self"
resolution = "auto"
effect = { creates = { object = "waybread", count = 1 } }

[spell.grows]
count = "willpower 18 2, 26 3"
```

| Field | Rule |
|---|---|
| `effect.creates = { object, count }` | new on `SpellEffect` (`creates: Option<(ObjectId, u8)>`); allowed for `utility` only (`spell_def::effect`'s `only`); the object must exist and be `food` (a load check in `spell_def::resolve`, as `identify` is checked); `count` 1 to 10. A `utility` spell needs `identify` or `creates`, exactly one |
| `grows.count` | a new ladder dimension, integer rungs (`RungValue::Count`, the `Cells` rule: strictly ascending from the base, at most 10); `Grows::count`; `spell_def::grows` allows it only with `creates`; `Actor::spell_count(content, spell)` climbs it as `spell_degrees` does (`degree_rungs` and `degrees` of `SpellMods` apply to it too: a *of Lingering* ring gives a wafer more; the owner may strike that, Q4) |
| The cast | `World::cast_create` (in `world/cast.rs` beside `cast_identify`): pays, then `count` stacks of the object are made with `rot::made_at(now)` (fresh) and put into the caster's pack (`carry::give`, as a quest gift is; a pack that cannot take them drops them at the caster's feet with "… falls at your feet."); "Bread is given." The `Outcome` is `landed` (restoration trains, `practise_cast`) and `level` the caster's own |
| Draws | none: no die but the count's, which is a ladder |
| `waybread` | unchanged: nutrition 1000, 2 turns to eat, keeps 60 days, value 0, 100 g |
| The scroll | `scroll_of_daily_bread` (level 4, frequency 30, 130 gold: the level 6–7 convention), `read` kind `spell`, `teaches` it; reading one gives a wafer |
| Cost | 6 (self, auto: 100 %); the restoration rows' `mana` take 5 % off from rank 4, 10 from 7, 15 at 10; Frugal caster −20 % |

### 6.2 Balm

```toml
[[spell]]
id = "balm"
name = "Balm"
description = "A balm of prayer on body and mind: bleeding stops, poison and every hex leave you, and whatever holds or webs you lets go."
level = 7
school = "restoration"
kind = "cleanse"
shape = "self"
resolution = "auto"
effect = { cures = ["bleeding", "poisoned", "ill_wished", "weakened", "sluggish", "drowsy", "webbed", "held"] }
```

Nothing new in the engine: `cleanse` with `cures` is Remove disease's kind, through `World::ended` for each status (a hold ends as `loosen_holds` ends it: the holder lets go). The cast is refused with nothing to cure ("You have nothing to cure.", no time, no mana: the cleanse's existing refusal). A status not on the list (Staggered, Surprised, Pinned, which ease by actions; the places' Silenced, Unwelcome, Ebbing, which refresh while one stands inside; the cooldowns) stays. Scroll: `scroll_of_balm` (level 4, frequency 30, 130 gold). Cost 7.

### 6.3 Restoration

```toml
[[spell]]
id = "restoration"
name = "Restoration"
description = "A long rite of restoring: what sickness took from your body is mended, and what the dead drank from you is given back."
level = 20
school = "restoration"
kind = "cleanse"
shape = "self"
resolution = "auto"
casts = 6
mana = 40
effect = { mends = true, restores = true }
```

| Field | Rule |
|---|---|
| `effect.mends`, `effect.restores` | new on `SpellEffect`, `cleanse` only; a cleanse needs at least one of `cures`, `diseases`, `mends`, `restores` |
| The cast | `World::cast_cleanse` gains two lines: `mends` → `StatDamage::mend_all` (as `World::mend` does for the temple, `world/service.rs`); `restores` → `World::restore_drained` (the same). Refused with nothing to do ("You are whole.", no time, no mana) when no stat is damaged and nothing is drained; a disease still showing is no bar (the temple refuses a mend while one shows; the rite mends what it can and the disease takes again tomorrow: Q5) |
| `casts = 6` | a minute of `cast` actions (10 s each at normal speed), interruptible as any multi-cast spell is: a creature coming into view stops it, the mana unspent (`pay_cast` pays when the cast is sure) |
| `mana = 40` | explicit, as Identify's 60: the level (20) is how hard a scroll of it is to learn (certain at caster level 20; a struggle for a priest of 12), the cost stands apart; the restoration rows still take their percent off |
| The scroll | `scroll_of_restoration`: level 8, frequency 10, 300 gold (beyond the README's convention, which stops at level 12: a new row for it), so a warrior may find the rite the temple sells |
| Marks | the Willpower mark of every spend (`StatUse::Cast`), restoration's `landed` when anything mended |

### 6.4 Sanctuary

A class active like Consecrate (`TalentEffect::Consecrate { mana, time }` → `ClassActive::Consecrate`, `world/consecrate.rs`): `TalentEffect::Sanctuary { mana, time, radius, lasts }` → `ClassActive::Sanctuary`, `world/sanctuary.rs`. `use:sanctuary` in action text; the abilities menu's Class group.

```toml
# assets/data/talents/sanctuary.toml
[[talent]]
id = "sanctuary"
name = "Sanctuary"
tree = "class"
granted = true
description = "Kneel and hallow the ground about you (an ability, paid in mana; a minute's rite): for an hour no foe will set foot within two paces of where you knelt, and your wounds and your mind mend there as in a sanctuary. The peace breaks if you strike from within it."
effects = [{ kind = "sanctuary", mana = 30, time = "1m", radius = 2, lasts = "1h", regen = 300 }]
systems = ["special_rooms", "hit_points", "mana"]
```

| Part | Rule |
|---|---|
| The rite | `use_class` → `begin_sanctuary`: refused (no time, no mana) on the overworld, in water, inside a shop, or where a hallow already stands within `radius`; else `spend_mana` and `Spent::Sanctuary(until)`, kept at through `keep_at` as Consecrate is (a creature coming into view, a message, or a blow interrupts: "You rise from your knees.", the mana spent, nothing laid: time is the price) |
| The hallow | `WorldState::hallows: Vec<Hallow { level: LevelId, at: Pos, radius: u8, until: GameTime, regen: u16 }>`; laid at the end of the rite on the priest's cell; "The ground about you is hallowed." |
| Who keeps out | `World::shuns(id)` (`world/ai.rs`, the one place a hostile creature keeps out of a sanctuary room) gains one clause: a hostile not already inside the hallow treats every cell within `radius` (Chebyshev, as rooms are cells) as shunned. A creature inside when it is laid may move within and leave (the sanctuary room's rule); one that is not never steps in, opens no door into it, and waits (`creature_steps`' existing wait). `spawn.rs` leaves hallowed cells out (its `shunned` clause, one more condition). Missiles and spells cross it as they cross a sanctuary room: the hallow keeps feet out, not arrows |
| The pace | `World::healing_rate` and `World::mana_rate` take `× regen / 100` for an actor standing in a hallow on the current level, beside the room's `regen` (the larger of the two counts, never both: a hallow laid in a sanctuary room is × 3, not × 9) |
| The break | `World::resolve` (a blow, shot, or throw by the player) and `spell_strikes`/`cast_area_hex` (a harmful cast by the player) call `World::break_hallow()` when the player stands inside one: it ends ("The peace is broken.", `Event::Hallow(End)`), whoever it kept out is free to come. Casting Mend, Balm, or Daily bread inside breaks nothing (no target harmed). Being hit inside breaks nothing: the one who struck from outside with an arrow owes nothing |
| The end | `World::live`'s event loop takes `hallow_until` as a moment (so splitting time never changes the result) and drops hallows past `until` ("The hallowing fades.", a `note`: a rest goes on); leaving the level keeps it (it is on its level; it fades at its hour wherever the priest is) |
| Alignment | none: a priest of any band hallows (the words the same; an unholy priest's hallow is as holy to the goblins, who keep out). Q3 |
| What it is not | not a room effect (rooms are generated; a hallow is laid on cells), not a status (it stays when the priest walks away), not a wall (nothing is unwalkable: the player, a neutral, a beast, and a hostile already inside walk it freely; only a hostile outside chooses not to), not a turret (the break) |

Numbers as data: `radius` 1 to 5, `lasts` 10 minutes to 12 hours, `regen` 100 to 1000, `mana` 1 to 500, `time` 10 s to an hour (load checks on the talent effect).

### 6.5 Costs against the pool

The probe's priest pools (`docs/dev/spells.md` "Balance": 27 at 3, 42 at 5, 49 at 6–7, 63 at 8, 90 at 12, 216 at 30, 426 at 60; 18 and 25 interpolated).

| Level | Pool | Mend (1) | Cure wounds (2) | Daily bread (6) | Remove disease (11) | Balm (7) | Restoration (40) | Sanctuary (30) |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 3 | 27 | 27 casts | 13 | — | — | — | — | — |
| 6 | 49 | 49 | 24 | 8 wafers | — | — | — | — |
| 12 | 90 | 90 | 45 | 15 | 8 | — | — | — |
| 18 | 132 | 132 | 66 | 22 | 12 | 18 | — | — |
| 25 | 180 | 180 | 90 | 30 | 16 | 25 | 4 | — |
| 40 | 300 | 300 | 150 | 50 | 27 | 42 | 7 | 10 |

A rest to full is under an hour at level 3, about 3 hours at 12, 7 at 30 (the probe). The restoration school's `mana` row (5 % from rank 4, 10 from 7, 15 at 10) and its `speed` row (a tenth quicker from rank 3, a third at 9) make every row above cheaper and quicker as the priest casts; rank 5 comes in the first fortnight of a caster who casts daily (the spell scaling design §8.4), and a priest who makes bread, mends, and cleanses casts daily.

## 7. Determinism, saves, replays

| What | Rule |
|---|---|
| Draws | Daily bread none (a ladder count), Balm none, Restoration none, Sanctuary none. The three spells' only dice are Mend's and Cure wounds', unchanged. A run without a priest hashes as before |
| Saved | `WorldState::hallows` (`#[serde(default, skip_serializing_if = "Vec::is_empty")]`), hashed in `state_hash`; nothing else new (`Actor::talents` holds the table's talents as today; the wafers are ordinary stacks with `died`) |
| Old priest saves | `class::grant` on `from_state` gives what the grown table has and the save lacks (Daily bread, Balm, Restoration, Sanctuary at their levels) and takes nothing away: a saved priest keeps Smite, Blessing, and Purify (`docs/dev/progression.md`: "nothing is taken away"). Purify's talent file is removed (its effect lives in Daily bread's); a save naming `purify` loads without it (`Talents::drop_unknown`, said in `retiered`) |
| Content fingerprint | changes (three spell files, three scroll objects, one talent file removed and two added, the class file): saves of every class note the new fingerprint as every content PR does; `FORMAT_VERSION` untouched |
| Replays | `use:daily_bread`, `use:balm`, `use:restoration` (spells by id, as today), `use:sanctuary` (a class active); no new syntax |
| Tests | `world/caster_tests.rs` (the priest's table: each entry at its level, the two cut not given, Blessing not given (Q1)); `world/spell_list_tests.rs`: the full caster's count bound stays `4..=6` with Blessing out (mend, cure_wounds, daily_bread, remove_disease, balm, restoration), or widens to `4..=7` with it in; the per-spell checks (level ≤ class level at the grant: 6 ≤ 6, 7 ≤ 18, 20 ≤ 25) hold; `content/spell_def.rs` (`creates`, `count`, `mends`, `restores`, the refusals); `world/bread_tests.rs` (a wafer in the pack, fresh, the count ladder, a full pack, the scroll, no draw); `world/balm_tests.rs` (each status ended, a hold let go, the refusal with nothing); `world/restoration_tests.rs` (stat damage mended, drained points back with the hit points and mana they carry, the refusal, interrupted at the third cast with the mana unspent); `world/sanctuary_tests.rs` (a kobold outside never steps in and waits; one inside leaves; the pace × 3; a blow from inside breaks it; an arrow from outside does not; it fades at the hour whoever is where; refused on the overworld and over another; the save and a replay) |

## 8. What the player sees

| Where | What |
|---|---|
| The abilities menu | the Spells group lists Daily bread ("makes a wafer of waybread"; with the count at 2 or more, "two wafers"), Balm ("cures bleeding, poison, hexes, holds"), Restoration ("mends your body; a minute") with their mana as every spell; the Class group lists Sanctuary with Consecrate's pattern (greyed `NoMana`, `While` nothing: it has no cooldown) |
| Messages | "Bread is given." / "A wafer of waybread falls at your feet." · Balm: the statuses' own `end` lines ("The bleeding stops.", "The poison wears off.", "The web falls away.") · Restoration: "You feel whole again." (the temple's words, `World::mend`'s event) · Sanctuary: "You kneel and pray." … "The ground about you is hallowed." / "You rise from your knees." / "The peace is broken." / "The hallowing fades." · a creature kept out says nothing (as at a sanctuary room's door) |
| Look mode | a hallowed cell: "… on hallowed ground" (as a special room names its floor to one who knows it; the player knows their own hallow); nothing on a creature |
| The status bar | nothing: a hallow is a place, not a status; the mana shows as today |
| Dumps | `spells` layer: `Daily bread (level 6, 6 mana): 1 wafer`, `Balm (level 7, 7 mana)`, `Restoration (level 20, 40 mana)`; `abilities`: `Sanctuary (class): here`; `truth` (dev): `hallow at (12, 5) r2 until 09:15:00`; JSON `hallows` (`pos`, `radius`, `until`) for the current level |
| The wiki | `wiki/src/classes.md`'s priest row (the generated `class-levels.md` picks the table up); `spell-list.md` and the scroll tables pick the spells up; a new paragraph under Special rooms → "Sanctuary" with the numbers (radius, hour, × 3, the break), generated from the talent's effect by `xtask/src/wiki/talents.rs`'s `effect_text` (one arm); the housekeeping table of §3 as a new page `wiki/src/housekeeping.md` ("What ails you, and what cures it": every status, disease, and drain with its cures, the priest's spell and level among them), generated by a new `xtask/src/wiki/housekeeping.rs` from `status.toml`, `diseases.toml`, the objects' `cures`, the services, and the class tables |
| The spec | spec/08i-character.md "Class abilities": the priest's row; spec/07d-shops-and-rooms.md "Special rooms": the hallowed ground paragraph |

## 9. Talents and class abilities

**No new talent, no new system.** The coverage policy's test (`docs/dev/progression.md`): a system is broad; a sub-feature extends its parent and mints nothing.

| Entry | System(s) it reads | Its parent's talents, unchanged |
|---|---|---|
| Daily bread | `hunger` (feeds), `spells`, `mana` | Lean, Ascetic, Iron stomach, Gourmand, Herb lore; Purify's rule rides in this talent |
| Balm | `status` (ends statuses), `spells`, `mana` | Clotting, Poison resistance, Divine grace, Nature's ward, Cool head |
| Restoration | `diseases` (stat damage and drain are its tag: `drain` folded into `diseases` in #773), `spells`, `mana` | Unbroken soul, Iron will, Fever-proof, Strong constitution |
| Sanctuary | `special_rooms` (a sanctuary the player lays; the room effect's `shunned` and `regen` are its rules), `hit_points`, `mana` | Pious, Old delver; Healthy, Fast healer; Meditation, Clear mind |

Against the bar, entry by entry: none is information only (rule 8); none is a small percentage on a niche mechanic; none duplicates a talent's effect (Restoration is the temple's service, which is no talent; Balm's list is no talent's; Pious cheapens the temple and is untouched) (rule 3); none chains (rule 4); every one works underground (rule 6: Daily bread anywhere, Sanctuary refused only on the overworld); no hidden marks (rule 7); no gold, favour, XP, or quest effect anywhere, and the 40 changes what a dungeon rest is (rule 1); nothing on shop favour (rule 9). Tags match hooks: Daily bread reads `hunger` (it feeds) and `spells`/`mana` (it is a cast); Sanctuary reads `special_rooms` (the `shunned` and `regen` rules), and `hit_points`/`mana` for the pace it changes.

Class abilities of other classes: none touched but the two shared entries the priest's table stops naming (Smite stays the paladin's, Blessing the paladin's and bard's); their talent files are unchanged.

## 10. Rejected ideas

| Idea | Why not |
|---|---|
| **Turn undead** (NetHack, ADOM) | a fight ability: the undead flee or die. Against the owner's "not any extra ability to fight" |
| **Keep Smite for a priest of the good side** | it is a blow; and #823 (the paladin, modes of smiting) owns it |
| **Keep Blessing** as "a priest without a prayer is odd" | +1 to hit is a fight buff; Blessing is the paladin's and bard's table entry and its scroll teaches it (abjuration +2 affinity: a struggle at caster level 1, easy by 3). Asked as Q1 |
| **Light as a table entry** (the owner's example) | the kit's scroll of light teaches Light for certain at level 1 (a level-0 spell at caster level 1), and every race without darkvision has a lantern: a table entry would spend a level on what the first turn gives. The candles stay flavour (Q6) |
| **Remove curse** on items | Warlock has no cursed items; Balm takes the hexes, Consecrate the places |
| **Identify** | the merchant's (the owner); the scroll and spell of identify exist for all |
| **Purify as its own entry at 25** | small beside a spell that makes bread; folded into Daily bread's talent so the effect kind stays in use and the level goes to Restoration |
| **Create water, holy water** | there is no thirst, and holy water's uses (NetHack) are curse uses |
| **A private piety or favour pool** (ADOM) | the altar's prayer is everyone's (`forgotten_altar.toml`); a second pool for one class is a system for one class, and the resources system says a pool joins with its own talents (policy step 4). Mana is the priest's pool |
| **Sanctuary as a buff status on the priest** (hostiles shun the priest's cell) | a status that moves with the priest is an escort; the owner's "no fighting" and Zin's shape both want a *place* one kneels in and leaves; and a status would show a spell circle about a priest walking through a goblin den untouched |
| **Sanctuary as a generated room effect laid at run time** | room effects are the generator's (`mapgen/special.rs`), per room, in the fingerprint; a hallow is cells on any level |
| **Sanctuary that also stops spawning on the level** | Quiet delver's territory, and a level-wide effect for a kneel; the hallow keeps spawns out of its cells only |
| **A ward (Deflection, Protection) at 18**, as the cut Ward of faith | a defence passive is a fight ability by another door, and #798 cut it as weak (#776's bar) |
| **Faster mana regeneration as the capstone** (Arcane mastery's twin) | a percent on a pace, the wizard's already; Saint stays as approved |
| **Resurrection, a second Miracle** | permadeath; Miracle is once in a life by the owner's design |
| **Raising Saint to healing and mana × 2** | the owner approved Saint; a change is not asked for. Left as a line for the owner if the capstone feels thin (Q7) |
| **Daily bread making a ration or bread** (sellable) | 5 and 2 gold a cast is a gold trickle: rule 1's spirit (no economy ability); the wafer is worth 0 |
| **A hunger-stilling status instead of food** (D&D's *sustain*) | hunger is a count the player never sees; food is what the system has, and a wafer is a thing in the pack that spoils in 60 days, drops, and weighs |

## 11. Implementation plan

| Slice | Issue | What it touches | Depends on |
|---|---|---|---|
| **P1 The spells and the table** | priest: Daily bread, Balm, Restoration; Smite, Blessing (Q1), Purify out | `warlock-core`: `content/spell_def.rs` (`creates`, `count` ladder, `mends`, `restores`, the checks), `content/spell_ladder.rs` (`RungValue::Count`), `world/cast.rs` (`cast_create`, `cast_cleanse`'s two lines), `world/spell_rung.rs` (`spell_count`), `world/spell_words.rs` (the menu's words), dumps; data: `spells/daily_bread.toml`, `balm.toml`, `restoration.toml`, `objects/scroll_of_*.toml` (three), `talents/daily_bread.toml`, `balm.toml`, `restoration.toml` (one spell a talent, `tree = "class"`, `granted`), `talents/purify.toml` removed, `classes/03-priest.toml`; tests §7; `docs/dev/spells.md`, `docs/dev/progression.md` (the priest's line), CLAUDE.md's action text if any (none: `use:ID` covers it) | — |
| **P2 Sanctuary** | priest 40: the hallowed ground | `content/talent.rs` (`TalentEffect::Sanctuary`), `talent.rs` (`TalentMods::sanctuary`), `content/class.rs` (`ClassActive::Sanctuary`), `world/sanctuary.rs` (the rite, `Spent::Sanctuary`, `keep_at`, the hallow, `break_hallow`, the fade), one clause in `ai.rs::shuns` and `spawn.rs`, one term in `healing_rate` and `mana_rate`, one call in `fight.rs::resolve` and `spell.rs::spell_strikes`, `WorldState::hallows`, perception (`PerceptionView::hallowed(pos)` for look mode), dumps; data: `talents/sanctuary.toml`, the class file's 40; tests §7; `docs/dev/special-rooms.md` (a section "Hallowed ground"), `docs/dev/screens.md` nothing (the menu's existing Class group) | P1 (the class file) |
| **P3 Wiki, spec, probe** | the priest's pages | `xtask/src/wiki/housekeeping.rs` and `wiki/src/housekeeping.md`, `SUMMARY.md`, `TERMS.md`; the priest's row in `classes.md`; spec/08i-character.md and spec/07d-shops-and-rooms.md; `world/balance_probe.rs`'s priest column gains the new spells (no roll: a line each); `ui/screen_snapshots.rs` a case `abilities-priest-40` (the menu over a level-40 priest: text) | P1, P2 |

P1 and P2 each add saved state or actions (P2 the state, P1 none but content): by the integrator's rule they may run beside one other such PR. No golden anywhere (nothing here is the GPU's: a hallow draws no light, Q8).

## 12. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| Q1 | **Blessing** (3): out of the priest's table (a +1 to hit is a fight buff), or kept as the one prayer a priest says over themselves? | **out**: the owner's words are "not any extra ability to fight"; the scroll of blessing teaches it to a priest who wants it (abjuration +2), and the spell count test's `4..=6` holds without it |
| Q2 | **Daily bread at 6 and Balm at 18**, or Balm at 6 (bleeding and poison in a fight from the start) and Daily bread at 18? | **bread first**: hunger comes by day two and has no sure answer but gold; bleeding has the Heal skill from level 1 and poison Remove disease at 12 |
| Q3 | **Sanctuary and alignment**: holy gifts (Consecrate, Smite, Lay on hands) do nothing outside the good bands; should a hallow? | **no alignment field**: the priest's whole table is housekeeping, and an evil priest who cannot rest is a class that cannot be played evil. Consecrate keeps its band (it was approved so) |
| Q4 | **Daily bread's count on the degree hooks** (`degree_rungs`, `degrees`: *of Lingering*, the transmutation rows): a wafer more from a duration item is odd. | **no**: `spell_count` climbs its own ladder and reads no hook; the restoration rows' `mana` and `speed` are its growth (a note in the spells README) |
| Q5 | **Restoration while a disease still shows**: refused (the temple's rule), or mends what it can? | **mends**: 40 mana and a minute buy the mend now; the disease takes its point tomorrow and Remove disease (12) is the priest's own answer to that. The temple keeps its refusal |
| Q6 | **The kit and the weapon ranks** (maces 2, shield block 1, sword and board 1; three candles that do nothing): unchanged, or the mace to rank 1 and the candles to a burning light? | **unchanged**: the ranks are a starting skill, not an ability, and NetHack's mace is the fantasy; the candles are rite flavour. A burning candle is a light-system issue, not the priest's |
| Q7 | **Saint** (60): healing × 2 as approved, or healing and mana × 2 (a capstone that feels like one)? | **as approved**; the owner may widen it in one line of data (`mana_regen_percent` 100) if the table reads thin at the top |
| Q8 | **Should a hallow shed light** (as a shrine's stone does), so the player and the goblins see its edge? | **no light** (no golden, no glare for light-blind eyes, no beacon to the floor); look mode says "hallowed ground", and a creature that stops two cells off and waits is the edge made visible. If the owner wants it drawn, a steady dim preset (`circle_restoration`'s luma, radius 2) is one line and one golden |
| Q9 | **Restoration's scroll** (a level-8, 300-gold scroll that undoes drain for any class): ship it, or keep the rite the priest's and the temple's? | **ship it**, rare (frequency 10): a warrior drained to the floor far from Cēaptūn should have a dice-roll's hope, and "every spell has a scroll" is the magic design's rule |
| Q10 | **Purity of body at 32** with Remove disease at 12: keep (a convenience: no mana, no rest interrupted by a fever), or replace with something that answers a problem the table still lacks? | **keep**: it is the owner's approved shared entry, and the table lacks nothing in §3 by 32; immunity is the right shape for a priest past the barrows |
