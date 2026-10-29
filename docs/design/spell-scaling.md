# Warlock — Spell scaling: a ladder in every spell's own words, and spell levels for tiers

Design document for the owner's review, **round 2** (2026-10-29). Written against `origin/main` at 7833b9c2 (the stat training design, #721). Nothing here is implemented; §12 cuts it into issues. Round 1 audited the player wiki's magic pages (`wiki/src/magic.md`, `scrolls.md`, `mana.md`, `magic-items.md`, the generated `spell-list.md`, `spell-tiers.md`, `wands.md`, `rods.md`, `class-levels.md`) against the rules behind them (`docs/dev/spells.md`, spec §8 "Spells", `assets/data/spells.toml`, `assets/data/spells/*.toml`, `content/spell_def.rs`, `content/spell_scales.rs`, `world/spell_scale.rs`, `world/cast.rs`, `world/caster_level.rs`, the talents and affixes that touch spells) and rebuilt how a spell grows. Round 2 folds in the owner's answers to its fifteen questions (§13 "Decisions"): **tiers are gone, every spell has a level from 0 to 70; caster level is a learning concept and nothing else; talents and items may do anything to a spell through named hooks; the menu shows a spell's dice as it shows a weapon's; wands roll their strength when found.** Every number is data; the wiki states it (the owner's rule: the game is vague, the wiki is accurate; a spell's dice in the menu are the owner's explicit exception, as a weapon's are).

**The owner's request** (verbatim): *"auditor reads the magic documentation from wiki, and makes sure the documentation is as clear as possible. It also identifies mechanics that are too complicated to be put in simple words. Then it makes a change proposal, that aims to streamline the system. Just fucking look at it: `cap: Intelligence 10 %, Willpower 5 % of the cap a point; reach: Willpower 50 % of a cell a point`. Damage scaling must be entirely per-spell in its spell text. It must be clearly written. Tools we have at our disposal: this is not played with real dice. 2d6 spell can scale to 2d7 -- dice can change; we can add extra dice; we can add static bonus; not all damage die must be same. Some ideas, let's make few: one stat increases range, duration, radius etc; one adds damage per dice based breakpoints; one increases the size of dice, based on breakpoints. If you choose to go with these universal rules, you just need to write 3 or more rules, and spell becomes very simple unless we choose to add riders."* Earlier: the magic mechanics feel *"too complicated to be fun ... not complicated, but fucking janky"*.

**Addendum** (the same day, verbatim): *"i am not against getting rid of concept of caster level as spell scaling. perhaps that is just learning difficulty thing?"* Taken (§4.4, decision 1).

**The framing for round 2** (the owner, verbatim): *"this is a computer game, not played with real dice. There are things such as d7, d9 or whatever we need. We can also do things that would be tedious on tabletop, like rerolling 1's or whatever. Keep these things in mind when designing talents. Rules must be intuitive, executing the rules is not a concern."* It is applied throughout: the dice are any size, a talent may reroll ones or grow every die a size (§7), and every rule below is one a player can say in a sentence, however much the code does to carry it out.

## Contents

1. Summary
2. What exists today
3. The audit: the wiki read as a player
4. The proposal: ladders
5. Spell levels, 0 to 70: what tiers did, and what does it now
6. The data format and its checks
7. Talents, items, scrolls, wands, rods, traps: the hooks
8. What the player sees; the wiki, the spec, the docs
9. Migration: every spell, with before and after
10. Determinism, saves, replays, fingerprints
11. Choices made along the way, and what is rejected
12. Implementation plan
13. Decisions (owner, 2026-10-29)

## 1. Summary

- **One idea replaces six.** A spell's growth is a **ladder** written in its own file and printed verbatim in its wiki row: `damage = "intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2"` reads *"Fireball: 2d6 fire; with Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2."* Your score reaches the number, you roll that. The last rung is the most your stats take the spell. Nothing else is computed: no cap, no percent of a cap per modifier point, no `max_lift`, no half-rounding, no dice "room" formula (§4.2, §4.3).
- **Any dice shape on any rung** (the owner's tools): more dice (`3d6` → `4d6`), bigger dice (`1d6` → `1d8`), a static bonus (`5d6+2`), mixed dice (`3d6+1d10`). A new `Roll` type carries them; `Dice` stays for everything else (§6.2).
- **Breakpoints are stat scores**, not modifiers: "Intelligence 16" is the number on the character screen (decision 2). One stat per ladder, the spell's own choice (decision 3). Reach (`range`, `length`, `radius`) and a buff's or hex's `degrees` are ladders of the same kind, in whole cells and whole degrees (§4.3).
- **Spell levels replace tiers** (decision 9). Every spell has a **level, 0 to 70**, one number: the caster level at which it is learned for certain, and its base mana (`Fireball: level 11, 26 mana as a burst`). No tier table, no comfortable level, no overreach surcharge, no ceiling column; the wand charges, the study time and check, the loot conventions, the class tables' check, and the author's ceiling are each one short rule of the level (§5). The 26 spells get levels 0 to 12 (§5.8); every mana cost stays within three points of today's and eighteen of the twenty-five are unchanged.
- **Caster level is a learning concept, exclusively** (decisions 1 and 10). It decides which spells a scroll teaches for certain and how hard the others are; it never changes what a spell does, what it costs, or whether it can be cast. A spell known is a spell cast, at its level's mana: the pool is the gate. Rods and wands have no caster-level gate: a rod's sleep takes creatures up to the rod's own level (§7.5).
- **Talents and items may do anything to a spell** (decision 5): the cast path has **six named hooks** (rung, dice, roll, bonus, rider, reach and degrees), each a `SpellMods` field summed from talents and worn items, each filtered by the spell's tags (§7.1). Empower and Archmage are rungs; Keen mind grows its school's dice a size (d6 → d7); Potent adds to the roll; three new Spellcraft talents show the rest: Sure hand (a die showing 1 is rolled again), Brimming (a die more), Numbing cold (a Sluggish rider on cold spells) (§7.2). Rungs are not the only unit; they are the unit for the talents that are about the ladder itself.
- **Scroll adept is a scroll talent** (decision 12): the spell scrolls you read cast a rung up, and you read in half the time; nothing on your own spells. A spell scroll casts at the rung its kind says (`rungs = 2` for the scroll of lightning's 4d6); every scroll is one kind of thing (§7.3).
- **A wand's strength is rolled when it is found** (decision 13): its rung, 0 to its ladder's length, is on the stack, saved, and shown with its type (§7.4; **saves change** in that slice).
- **The menu shows the dice** (decision 11): `Fireball   4d6 fire, 26 mana, aimed`, as the character screen shows `+4 to hit, 1d8+2 slashing`; no "your spell grows" line (§8).
- **Reading one spell row today takes 17 concepts** for the damage cell and 6 for the reach cell; the proposal takes 3 and 2 (§3.3).
- **Balance holds** (§9.3): the trained wizard's and priest's spells land within a die of today's at levels 10, 30, and 60 (Fireball 4d6 / 5d6 / 5d6 against today's 3d6 / 5d6 / 5d6). The cantrips, Blight, Cause wounds, and Mend gain one or two rungs where today they never grow (decisions 6 and 7).
- **Eight slices** (§12): `Roll` and the levels first (both small, both can merge first), the ladders and hooks as one PR, then rods, the menu's numbers, the wands' strength (saves change), and the prose beside each other, the probe last.

## 2. What exists today

Verified line by line against the code (`docs/dev/spells.md` is current):

| Piece | Today |
|---|---|
| A spell's tier | `tier = 3`, 0 (a cantrip) to 9; `[tiers]` in `spells.toml` has one value per tier for `comfortable` (caster level 1, 3, 6, 12, 18, 25, 32, 40, 50, 60), `cost` (base mana 1, 3, 6, 11, 16, 22, 29, 37, 46, 56), `ceiling` (6 to 90), `max_lift` (40 to 80 %), `wand_charges` (15 to 2); `overreach_cost` 50 % a tier. `SpellRules::tier_of(caster level)` is the highest tier whose comfortable level is reached. |
| A spell's damage | `[spell.damage] dice = "2d6", per_levels = 4, max_dice = 3, bonus = 0`: the dice at the tier's **comfortable caster level**, a die more per `per_levels` caster levels above it, never more than `max_dice` (`SpellDice::at`). A heal's `heal` table the same. |
| Caster level | derived: character level × the class's `casting` percent (100 wizard, witch, priest, druid; 75 bard, aristocrat; 50 paladin, ranger, monk; 25 the rest), plus Arcane training (+1 per 5 levels), plus *of the Adept* items (1 to 3), half again with Attuned; at least 1 with any mana (`Actor::caster_level`). A level-60 wizard casts at 72. It feeds learning (`tier_of`), the overreach surcharge, dice growth, the scroll's cast level, the wand's level, and the rods' reach and sleep/terror gate. |
| The cap | `ceiling[tier] × resolution.damage %` (tier 3 area: 24 × 80 % = 19). A load-time check refuses a spell written over it; every roll is held to it at run time (`spell::capped`), criticals and talents included. |
| The lift (#556) | `[spell.scales] damage = { intelligence = 10, willpower = 5 }`: Σ weight × modifier is the percent the cap is lifted, never below 0, at most the tier's `max_lift` plus Keen mind's 10 for its school and *Potent* items' 4 to 15; the lifted cap is `cap × (100 + lift) / 100` rounded down; the dice gain `(lifted − bonus) / sides − (cap − bonus) / sides` more (`SpellDice::lifted`). A scroll read or a wand used is unlifted. |
| Mana | the tier's `cost` × (1 + 50 % per tier the spell is above the caster level's tier; Overreach −33 % of that) × the shape's percent (a burst's at its radius) × the resolution's `cost` × the armour's percent, rounded once, at least 1; then Archmage −50 %, Empower −10 %, School focus −10 %, Frugal caster and *of Thrift*. Identify's `mana = 60` replaces the tier price. |
| Reach | `range` (`length`, `radius`) grown by the `reach` scales (default Willpower 50 %: a cell per 2 modifier points, a half rounded down), plus School focus's cell and *of Farcasting*'s cells, never past `max_range` (`max_length`, `max_radius`). A burst costs by its own `radius` however far it spreads. |
| Duration | a buff's or hex's `severity` grown by the `duration` scales (default Presence 50 %; the hexes Presence 50 and Willpower 25) plus *of Lingering*'s degrees, up to the status's `max`. |
| To-hit | the `to_hit` scales (default Dexterity 100 %), a half toward zero, plus *of the Magus*; threat 22 less *Baleful*; the Spell attacks skill and a caster style. |
| Talents on dice | Scroll adept +1 to every die, Empower a die more, Archmage +2 to every die (`Dice::empowered`), then held to the lifted cap. They work on spell scrolls read too. |
| Learning | a study takes `(tier + 1) × 100` turns (Quick study half); a spell at or below the caster level's tier is learned for certain, else 2d12 + Intelligence modifier + the `study` skills + the class's affinity + Quick study's 2 against `12 + 4 × overreach tiers + known tiers / 10`; a failure draws a mishap: slight at 1 tier over, bad at 2 or 3, terrible from 4. |
| Class tables | a granted spell is held within the tier the class's caster level has reached at the level it comes (a content test). |
| Scrolls, wands, rods | a spell scroll casts at `[spell.scroll] level` (default the comfortable level), free of mana; the three older scrolls (fireball 3d6, lightning 4d6, frost 2d6) are bursts and bolts of their own, uncapped; loot `level`, `frequency`, `value` by tier (1 / 60 / 40, 2 / 45 / 80, 4 / 30 / 130, 6 / 20 / 200). A wand casts at its own `caster_level` (default the comfortable level) plus Wand mastery's 2 and *of Wands*' 1 to 2, with the tier's `wand_charges`. A rod's reach is `range + caster level / per_levels` up to `max_range`; sleep and terror take creatures of level at most the caster level. |
| Shown in the game | the abilities menu's "tier 3, 26 mana, aimed" and the aiming mode's reach. Nothing of dice, caps, or lifts. |
| Shown in the wiki | the row quoted in the request; "The lifted cap" (a 20-line section with a worked example); the tier table with "Most lift"; `class-levels.md` and `talents.md` repeat the row text for every granted spell; `study-times.md` and `study-chances.md` by tier; `wands.md` with "Caster level". |

## 3. The audit: the wiki read as a player

### 3.1 Sentences and cells that are unclear, with rewrites

Each row is a sentence or a cell a player meets, why it fails, and what the proposal prints instead (generated from the data where it is a table).

| # | Where | Today | Why it fails | Rewrite |
|---|---|---|---|---|
| 1 | `spell-list.md`, "Grows with" | `cap: Intelligence 10 %, Willpower 5 % of the cap a point; reach: Willpower 50 % of a cell a point` | four numbers, three units (percent of a cap, modifier points, cells), and none of them is a number the player rolls or a score they have | `Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2; range Willpower 14: 11, 18: 12` |
| 2 | `spell-list.md`, "What it does" | `2d6, a die more per 4 caster levels above 12, to 3d6 fire damage ... (never more than 19; your stats may lift it to 32, more with Keen mind)` | asks the reader to compute their caster level, then holds up three numbers (19, 32, "more") of which the one that matters, what they roll, is absent | `2d6 fire damage` (the ladder is the next column) |
| 3 | `spell-list.md`, "Reach" | `range 10 (to 12), radius 2` | "to 12" is a limit with no way to know when it is reached | `range 10 (Willpower 14: 11, 18: 12), radius 2` |
| 4 | `spell-list.md`, Frost | `it freezes the water it reaches into ice 10 cm thick (never more than 14; your stats may lift it to 22, more with Keen mind)` | the owner's example: the ice and the cap share one parenthesis as if 14 and 22 were centimetres | `2d6 cold damage; it freezes the water it reaches into ice 10 cm thick` |
| 5 | `magic.md` "Tiers" | "Every spell has a tier, from 0 (a cantrip) to 9. Each tier has a comfortable caster level ... Each tier also has a damage ceiling ... and a most lift" | a stepped level system with four columns the player must cross-reference before reading a spell | **"Spell levels"**: *"Every spell has a level, 0 to 70. It is the spell's base mana cost, and the caster level at which you learn it from a scroll for certain."* No table. |
| 6 | `magic.md` "The lifted cap" | the whole section: "each by a percent of the cap per point of its modifier ... never more than the tier's most lift (40 % at tier 0 ...) ... rolls as many more dice as the lift makes room for ..." | a six-step computation with two roundings, explained with a worked example because it cannot be said plainly | delete; replaced by "How a spell grows" (§8): *"Each spell's row lists its ladder: a stat and the scores at which the spell changes. Your score reaches the number, you cast that rung."* |
| 7 | `magic.md` "Shapes" | "Your Willpower lets a spell reach farther: a cell more of range or length, and a cell more of a burst's radius, for every 2 points of its modifier, but never past the spell's own limit ..." | modifier arithmetic, a parenthetical about a thing that does not exist, and a limit told as "to" | *"How far a spell reaches is in its row, and so is how far your Willpower takes it."* |
| 8 | `magic.md` "Shapes" | "The hexes also grow with your Willpower: their degrees are half your Presence modifier plus a quarter of your Willpower modifier, added up and rounded to the nearest, a half down" | two stats, two fractions, a rounding rule | *"A hex's row says how long your Presence makes it last."* (decision 8) |
| 9 | `magic.md` "Damage and its cap" | "Spell damage is always rolled dice, with a small fixed bonus at most a quarter of the ceiling, and rarely any. A spell's dice may grow with your caster level ..." | the quarter rule is an author's rule; "every few levels" is vague because the real number is per spell and depends on a derived level | *"Spell damage is rolled dice, as written in the spell's row; what grows it is its ladder, and your talents and items."* |
| 10 | `magic.md` "Damage and its cap" | "Scroll adept adds 1 to every die, Empower rolls a die more, Archmage adds 2 to every die; they bring a spell to its most sooner, and nothing lifts it past" | three different arithmetic effects, each then clamped | *"Empower takes your spells a rung up their ladders; Archmage two. Keen mind rolls its school's dice a size bigger. Each talent's row says what it does; nothing is clamped."* |
| 11 | `magic.md` "Caster level" | "Your caster level is how strong a caster you are." | under the proposal it is how *learned* a caster you are: it never changes what a spell does or costs | *"Your caster level is how far your learning reaches: a scroll of a spell at or below it teaches you for certain; above it, the study is a check. It never changes what a spell does or costs."* |
| 12 | `magic.md` "Mana" | "A spell's mana is its tier's base cost, times the overreach (half again per tier above your caster level's; with Overreach, a third less of that: 33.5 % a tier), times its shape's share ..." | a surcharge by a derived level's tier, and a talent that is a percent of a percent | *"A spell's base mana is its level (a cantrip's 1), times its shape's share and its resolution's, and more in armour. Your caster level does not enter."* |
| 13 | `magic.md` "Wands" | "cast at the **wand's own caster level**, never yours ... Its damage still stops at the spell's cap." | a wand's level is invisible and its only effect was dice growth | *"A wand casts its spell at a rung of its own, rolled when it was made and shown once you know the wand; Wand mastery takes it a rung up."* |
| 14 | `magic.md` "Rods" | "its reach ... grows by a cell per so many caster levels, up to its limit"; "creatures in it no stronger than your magic" | "so many" hides a per-rod number; "your magic" is a derived level nobody sees | *"A rod's row says how far it reaches and how far your Willpower takes it, and the strongest creature its sleep or terror takes."* |
| 15 | `scrolls.md` "Spell scrolls" | "at the scroll's own caster level (its tier's comfortable level)" | a scroll has no level a player can see | *"as written in the spell's row, at the rung the scroll's row says: your stats do not grow a scroll's spell; Scroll adept takes it a rung up"* |
| 16 | `class-levels.md`, `talents.md` | every granted spell repeats `(never more than 14; your stats may lift it to 21, more with Keen mind)` | the row text leaks into two more tables | fixed by the one generator (`wiki/spells.rs::does`) |
| 17 | `spell-tiers.md`, `study-times.md`, `study-chances.md` | tables by tier with "Most lift", "Comfortable caster level", "Turns" per tier, difficulty per tier of overreach | per-tier numbers that modify other per-tier numbers | the tier table goes; study time and difficulty are one sentence each with a sample table by spell level (§5.4) |
| 18 | `spell-resolutions.md` | "Most damage, of the ceiling: attack 100 %, auto 75 %, area 80 %, touch 120 %" | a factor on an author's check | the column goes; the table keeps the roll (attack, auto, area, touch) and the cost |
| 19 | `magic.md` "Spellcraft talents" | "Keen mind ... lets your stats lift that school's or element's spells 10 points higher than their tier's most lift" | "points" of a percent of a cap | *"Keen mind: that school's spells roll dice a size bigger for you (a d6 is a d7)."* |
| 20 | `magic-items.md` | "*Potent* ... raise the most lift of every spell by their number" | as 19 | *"Potent: 1 to 3 more on every spell's roll, harm or healing."* |

What is already clear and stays: the shapes' descriptions, the resolutions (attack, auto, area, touch), learning and mishaps as a process, the wand and rod paragraphs apart from the lines above.

### 3.2 Mechanics too complicated to be said in one plain sentence, ranked

1. **The lift** (§2): percent of a cap per modifier point, two stats summed, clamped to a per-tier most widened by a talent and by items, times the cap, rounded down, turned into dice by a difference of two floors. **Removed.**
2. **Dice by caster level**: a derived level (four inputs) against a per-tier comfortable level, divided by a per-spell `per_levels`, capped by `max_dice`. **Removed** (§4.4).
3. **Tiers**: a spell's tier, the tier's comfortable level, the caster level's tier, the gap between them as overreach, each a lookup in a ten-column table before the spell's own numbers apply. **Removed** (§5): a spell's level is one number against one number, the caster level, and only when learning.
4. **Reach and duration by percent of a modifier**: "50 % of a cell a point, a half rounded down", a cap told as "to 12", School focus and Farcasting cells added after. **Replaced** by cell ladders (the flat cells stay: §7.2).
5. **Talent dice then a clamp**: a die, +1 a die, +2 a die, each then held to a lifted cap, so what a talent is worth depends on where the cap sits. **Replaced**: each talent does one thing its row says, unclamped.
6. **The hexes' two-stat duration**. **Replaced** by one stat.
7. **The three older scrolls**: bursts and bolts of their own with their own dice and no cap, with a special rule for talents. **Folded** into spell scrolls at a rung.
8. The mana cost (seven factors) was long but not janky; it loses two factors (the tier's price, the overreach surcharge) and gains none.

### 3.3 Concept count: what a player must hold to read one spell row

Fireball's row, today and under the proposal. A concept is a number or rule the reader must know or compute that is not printed in the cell itself.

| Cell | Today | Proposed |
|---|---|---|
| Damage | 1 tier · 2 comfortable caster level · 3 caster level (3a level × casting %, 3b Arcane training, 3c *of the Adept*, 3d Attuned) · 4 `per_levels` · 5 `max_dice` · 6 the tier's ceiling · 7 the resolution's percent · 8 the cap as a clamp · 9 the stat modifier · 10 percent of the cap per point · 11 two stats summed · 12 `max_lift` by tier · 13 Keen mind's +10 · 14 Potent's +N · 15 the lifted cap rounded down · 16 the dice "room" formula · 17 talent dice then the clamp — **17** (20 with 3a–d) | 1 the roll · 2 the stat's score · 3 the rung reached (and 4 what a talent or item adds, when the player has one, each in its own row) — **3** |
| Reach | 1 the modifier · 2 50 % of a cell a point · 3 a half rounded down · 4 the "to" cap · 5 School focus's cell · 6 Farcasting's cells — **6** | 1 the score · 2 the rung — **2** |
| Duration | 1 the modifier · 2 a degree per 2 points · 3 a second stat at a quarter · 4 the rounding · 5 the status's max · 6 Lingering's degrees — **6** | 1 the score · 2 the rung — **2** |
| Mana | 1 tier · 2 the tier's cost · 3 caster level (3a–d) · 4 the caster level's tier · 5 overreach × 50 % · 6 Overreach's −33 % of that · 7 shape · 8 resolution · 9 armour — **9** (12) | 1 the level · 2 shape · 3 resolution · 4 armour — **4** |

## 4. The proposal: ladders

### 4.1 Goals

| # | Goal | Why |
|---|---|---|
| G1 | **A spell's growth is written in its file and printed as written.** No global pipeline between the data and the wiki row. | the owner: "entirely per-spell in its spell text ... clearly written" |
| G2 | **One lookup, no arithmetic**: the player compares a score to a number. No percents, no modifiers, no rounding, no clamps. | "nobody can predict what a stat point does" |
| G3 | **Any dice shape**: count, size, bonus, mixed dice, per rung; d7s and d9s where they fit. | the owner's tools; the framing |
| G4 | **One driver per dimension**: one stat for damage, one for reach, one for duration, each the spell's own choice. | "one stat increases range ..."; decision 3 |
| G5 | **Talents and items do what their author writes**, through a small set of named hooks, so a reroll, a bigger die, a flat bonus, a rider, or a rung needs no special case. | decision 5; the framing |
| G6 | **Caster level is learning, nothing else.** One number, the spell's level, says what it is hard to learn and what it costs. | decisions 1, 9, 10 |
| G7 | **Hidden in the game, exact in the wiki**, with the owner's one exception: a spell's dice, cost, and reach are shown where a weapon's are. | the owner's rule; decision 11 |
| G8 | **Integers only, deterministic, derived at cast time**; the one thing saved is a wand's rolled rung. | `docs/dev/spells.md`'s invariants; decision 13 |
| G9 | **Balance within a die of today** at levels 10, 30, 60 for the trained caster; mana within three points; shifts named. | the migration must not re-tune the game by accident |

### 4.2 The rule, as the wiki will state it

> Every spell has a **level**, 0 to 70: what it costs in mana, and how learned a caster must be to take it from a scroll for certain. Every spell has a **base**, what it does for anyone who casts it, and may have **ladders**: for its damage (or healing), its reach, and how long its effect lasts. A ladder names one stat and lists scores with what the spell does at each: *Fireball: 2d6 fire; Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2*. When your score reaches a number, you cast that rung. **The last rung is as far as your stats take it.** Your talents and magic items may add to that in their own ways, each said in its own row: a rung up, a die more, bigger dice, a bonus, a die rolled again. A scroll read and a wand used cast at a rung of their own.

That is the whole rule. §4.3 is the same thing in the precision the code needs.

### 4.3 Rungs, precisely

- A **ladder** is a stat and a list of **rungs** `(at, value)`, `at` strictly ascending scores in 1 to 40 (the stats' range), at most `MAX_RUNGS` (8). The spell's **base** value is rung 0. The **value** is a `Roll` for `damage` and `heal`, whole cells for `range`, `length`, and `radius`, whole degrees for `degrees`.
- **The rung reached by stats**: `by_stats = |{ rung : at ≤ score }|`, `score` the caster's **effective score** in the ladder's stat (`Actor::effective_stats`: what the character screen shows, with statuses, stat damage, drains, and worn items). A score below the first rung is rung 0. Nothing is ever below the base: a low score never shrinks a spell.
- **Rungs from outside** (`SpellMods::rungs`, §7.1): the sum of the caster's talents' and items' rung shifts for that dimension, 0 or more. For a scroll read or a wand used, `by_stats` is 0 for damage and heal and the rungs are the item's own plus the reader's scroll or wand rungs (§7.3, §7.4); reach and degrees are the reader's as for an own cast.
- **The rung cast**: `rung = min(by_stats + rungs, ladder.len())`. **The ladder's value** is `rungs[rung − 1].value`, or the base at rung 0. The last rung is reached and never passed *by rungs*; the other hooks (§7.1) then change the value as they say, and nothing clamps them but the engine's own limits (`MAX_REACH`, the burst table, a status's `max`).
- **What the dimensions mean**: `damage` and `heal` are the roll made (one draw per die, in group order, the bonus added; an area's one roll for all as today); `range` is the farthest cell a ray, bolt, or burst goes; `length` a cone's or line's; `radius` a burst's; `degrees` the severity a buff or hex is given (the status's own `max` still holds it in `Effects::add`). A burst **costs by its base radius** (the one place `shape_cost` reads a radius), as today.
- **No cap at run time.** `spell::capped`, `Cast::cap`, `Burst::cap`, `Bolt::cap`, `Actor::spell_cap`, `Actor::spell_lift`, `SpellRules::lift`, `lifted_cap`, `grown`, `lengthened`, `max_bonus`, `SpellDice::at`, `SpellDice::lifted` are deleted. A **critical** (attack and touch resolutions only, as today) rolls the damage twice, as a blow's does, and is not held to anything (decision 4: "no cap").
- **To-hit**: `aim = "dexterity"` (default) names the one stat whose **modifier** is added to an attack or touch spell's roll, as a blow adds Dexterity's; a scroll's or a wand's always Dexterity. *Of the Magus*, *Baleful*, the Spell attacks skill, and the caster styles are unchanged. (A modifier, not a ladder: to-hit shares the blows' rule.)
- **Integers**: a comparison, a count, a minimum, a lookup. No division anywhere in growth.

### 4.4 Caster level is learning, and nothing else

Decisions 1 and 10, with the owner's words: *"caster level is exclusively a learning concept. items are good, because they do not require learning."*

| Reader of caster level | Today | Proposed |
|---|---|---|
| the **study**: certain or a check, the mishap band | yes, by tiers | **kept, in levels**: a spell of level at most the caster level is learned for certain; above it, the gap in levels is the overreach (§5.4) |
| the **class tables'** content test (a granted spell within reach) | yes, by tiers | **kept, in levels**: a granted spell's level at most the class's caster level at the level it comes (§5.6) |
| **mana**: the overreach surcharge (×1.5 a tier over) | yes | **gone**: a known spell costs its level's mana, whoever casts it (§5.3). The Overreach talent is re-pointed to the study (§7.2) |
| a spell's **dice** (`per_levels`, `max_dice`) | yes | **gone**: the ladder's stat |
| a **scroll's** cast level | yes | **gone**: a scroll casts at its own rungs (§7.3) |
| a **wand's** level (Wand mastery, *of Wands*) | yes | **gone**: a wand casts at its rolled rung plus the user's wand rungs (§7.4) |
| a **rod's** reach (`range + level / per_levels`) | yes | **gone**: a Willpower ladder, as a spell's (§7.5) |
| a rod's sleep and terror **gate** (a creature of level at most the caster level) | yes | **gone** (decision 10): the rod's own `up_to_level` (§7.5) |
| creatures' summons (#525) | yes | kept: not the cast path |
| the dumps (`Caster level N`, `cl N`), the probe | yes | kept |

**Why it is right** (round 1's case, standing): one driver per spell; caster level is itself a four-input pipeline the player cannot see; the stat training design (#721 §5) gives Intelligence and Willpower paths that now pay in rungs; the level keeps every other reward (the pool, the grants, the class talents). **The one loss**: a caster with a poor mind never grows a spell; accepted (decision 1), the wiki says exactly where the rungs are, and potions, elixirs, and the training paths are the way up. **What Arcane training, *of the Adept*, and Attuned keep**: learning reach only (a level-10 wizard with Arcane training learns level-12 spells for certain, without it level 10). Their rows say so (§7.2); they are weaker, and that is a tuning question for later, not this document's.

**Mana as the gate.** With no surcharge, a low-level caster who learned a high spell by a lucky study casts it at its level's mana: a level-5 wizard (47 mana) casts a level-11 Fireball (26 mana) once, then rests. The pool grows a level at a time (`mana_level = 60` in `experience.toml`, unchanged; its comment loses "tier 9's comfortable level") and is about ten casts of a spell of the caster's own level at every level (the probe's measure: 106 mana at level 12 against Fireball's 11 base, 254 at 30 against 29, 502 at 60 against 56; §9.4), so the difficulty of a spell beyond you is felt as a short day, not a refusal.

### 4.5 What keeps spells relevant at level 30 to 60

Without dice by level, three things, all already in the game or already planned:

1. **Higher levels.** The combat progression design (§7.3) showed that even with the lift a level-60 caster needs spells above today's: "a level-60 caster needs the tier 4 to 9 spells to exist, which is content the magic design planned and no scaling replaces". A level-22 burst written at base 5d6 with a ladder to 8d6+4 (within the ceiling at 22: (10 + 66) × 80 % = 60) kills the level-60 median monster (225 hit points, `docs/balance.md`) in 6 casts for 53 mana of a 684 pool; a level-37 one at 10d8 (ceiling 121 × 80 % = 96) in 4. The content is the progression, as the design meant.
2. **Rungs that run on.** The trained wizard's Intelligence reaches 20 at level 30 and 23 at 60 (`docs/balance.md`; the stat training design puts a main stat at +12 to +17 by 60); ladders are written to 25 and may go higher for the high levels, so a caster keeps climbing after the dice-by-level curve would have flattened (today `max_dice` is reached by caster level 24 on every tier-3 spell).
3. **The hooks from the class table.** Empower (wizard 32) a rung, Archmage (60) two, Keen mind a die size, Potent items a bonus: level-gated adds, each worth exactly what its row says on every ladder the player can read.

§9.3 shows the trained wizard's casts to kill at 10, 20, 30, and 60 today and proposed; they match within a die at 30 and 60 because today's growth had already stopped there.

### 4.6 The ceiling: an author's check by level, not a rule

The magic design's strict cap (its §5, "caps, not curves") had one job: no 10d6 fireballs. Under ladders the top rung is in every row and reviewed in every PR, which does most of that job; a content check does the rest cheaply. With tiers gone the ceiling is **a line of the spell level**: `ceiling(level) = 10 + 3 × level` (`[ceiling] base = 10, per_level = 3` in `spells.toml`), the most a spell of that level may be written to do at its top rung to one target. It is today's lifted maxima within a tenth (today's ceiling × (1 + `max_lift`) at each tier's comfortable level: 8, 18, 28, 40, 56, 72, 90, 110, 136, 162 against the line's 10, 19, 28, 43, 58, 76, 97, 121, 148, 178 at the levels the 26 spells and their successors sit). The check: every rung's `Roll::max` (the base's too) ≤ `ceiling(level) × resolution.damage / 100`, harm or healing; §9.1 shows every shipped spell passing. `resolution.damage` keeps that one job. The quarter-of-the-ceiling bonus rule goes. **Nothing at run time reads the ceiling**, and the player wiki does not print it (decision 9 took the tier table with it): the fact a player needs, the most a spell does, is the last rung of its own row.

### 4.7 Criticals, resolutions, the area's edge

Unchanged but for the clamp: attack and touch spells roll 2d12 + the aim stat's modifier against Defence and may critical (the damage rolled twice); auto never misses and never criticals; area strikes everyone with one roll, half beyond radius − 1, Evasion halving. The resolutions' cost factors are unchanged.

## 5. Spell levels, 0 to 70: what tiers did, and what does it now

Decision 9, verbatim: *"get rid of concept of spell tier. Assign spells a level between 0-70. (tier is just a cringe stepped spell level system anyhow). let's bypass that extra step. players are familiar with this form wow etc."*

### 5.1 The level

`level = 11` on every spell, 0 to `MAX_SPELL_LEVEL` (70). It is **one number with two meanings**, both the player's: the spell's **base mana**, and the **caster level at which a scroll of it teaches for certain**. A spell of level 0 or 1 is a **cantrip** (the word stays in the wiki and the class test: it costs 1 mana and anyone with mana learns it for certain). There is no table of levels: everything a tier's columns held is a sentence of the level, below. "Spell level" in prose where "level" alone could mean the character's.

### 5.2 Everything the tier table did, as a rule of the level

| Tier column, today | Rule of the spell level, proposed | Data |
|---|---|---|
| `comfortable` (the caster level that learns the tier for certain and casts it at base cost) | **a scroll of a spell at or below your caster level teaches for certain**; above it, the gap is the overreach (§5.4). Casting never reads it | none: `Actor::caster_level` against `spell.level` |
| `cost` (base mana) | **base mana = the level, at least 1** (a cantrip's 1); `mana = N` on a spell still replaces it (Identify's 60) | none |
| `overreach_cost` 50 % a tier | **gone** (§4.4) | removed |
| `ceiling` (the author's check) | **10 + 3 × level** (§4.6) | `[ceiling] base = 10, per_level = 3` |
| `max_lift` | **gone** with the lift | removed |
| `wand_charges` by tier | **a wand's charges are its own**: `charges` required on every `[object.wand]` (the seven ship with today's numbers, §7.4) | the wand files |
| study time `(tier + 1) × 100` turns | **100 turns and 20 a level**: 10 minutes and 2 a spell level (Fireball, level 11: 32 minutes; today 40); Quick study half | `study.toml`: `turns_base = 100`, `turns_per_level = 20` |
| study difficulty `12 + 4 × overreach tiers + known tiers / 10` | **12 + 1 a level of overreach + the levels of the spells you know / 20** (§5.4) | `base = 12`, `per_overreach = 1`, `known_per_point = 20` |
| mishap bands by overreach tiers (bad from 2, terrible from 4) | **slight below 6 levels over, bad from 6, terrible from 15** | `bad_from = 6`, `terrible_from = 15` |
| the class tables' test (a grant within the tier reached) | **a granted spell's level at most the class's caster level at the level it comes** (§5.6) | the test |
| scroll and wand loot `level`, `frequency`, `value` "by tier" | **explicit per object, as they already are**; the authoring convention (§5.7) says where a new one goes | none |
| `tier_of(caster level)`, `known_tiers`, `spells_by_tier` (the menu's order) | gone; `known_levels` (the sum); the menu and the wiki list **by level, then name** | code |

### 5.3 Mana

`base = max(1, level)` unless the spell names `mana`; then × the shape's percent (a burst's at its base radius) × the resolution's `cost` × the armour's percent, rounded to the nearest once, at least 1; then Archmage, Empower, School focus, Frugal caster, *of Thrift* as today. **No caster level anywhere in it.** §5.8 lists every spell's cost today and proposed: eighteen unchanged, the rest within three mana (Blight 14 → 17 the largest), each because the spell sits above its old tier's first level (§5.8 says why). The wiki's mana paragraph loses the overreach sentence and gains none.

### 5.4 Learning

A study of a scroll takes `turns_base + turns_per_level × level` turns (Quick study's percent off), then: a spell of level at most the caster level is **learned, no draw**. Above it, `overreach = level − caster level` (in levels) and the check is 2d12 + the Intelligence modifier + the `study` skills + the class's affinity + Quick study's 2 against `difficulty = base + per_overreach × overreach' + known / known_per_point`, where `overreach'` is the overreach with the Overreach talent's third off (§7.2) and `known` the sum of the levels of the spells the caster knows. A failure draws a mishap of the band the overreach falls in. The words (certain, easy, a struggle, far beyond you) are unchanged.

What it feels like, a wizard with +6 (Intelligence 16, Spellcraft, evocation affinity): a spell 3 levels over is likely (116 of 144), 6 over a fair bet (89), 12 over a gamble (28), 16 over a long shot (6), 20 over hopeless. Today the same wizard had one tier over at 108, two at 66, three at 28: the levels make the slope smooth where tiers made it a stair, so a scroll two or three levels beyond you, which a tier could make as hard as six, is a short reach, and one fifteen beyond is nearly out of reach where three tiers still gave a fifth. First guesses, as the whole of `study.toml` is.

### 5.5 Caster level

Unchanged in how it is derived (`Actor::caster_level`: character level × the class's `casting`, Arcane training, *of the Adept*, Attuned, at least 1 with mana); changed in what reads it: §5.4 and §5.6 and nothing else (§4.4). The wiki's "Caster level" section is rewritten to §3.1 row 11.

### 5.6 Class tables

The guaranteed spells stay where they are (`assets/data/classes/*.toml`); the content test becomes **a granted spell's level ≤ the class's caster level at that level** (character level × `casting` / 100, at least 1). With §5.8's levels every grant passes: the priest's Mend (1) at 1, Blessing and Cure wounds (3) at 3, Remove disease (11) at 12; the druid's Spark (1) at 1, Barkskin (3) at 3, Call lightning (11) at 12, Flame (12) and Frost (6) at 50; the wizard's Ray of frost (1) at 1, Magic missile (3) at 3, Burning hands (4) at 12, Arcane bolt (6) at 18; the witch's Evil eye (0) at 1, Hex of weakness (3) at 3, Cause wounds (4) at 12, Curse of sloth (3) at 32, Blight (7) at 40; the bard's (75 %) Mend (1) at 1 and Blessing (3) at 6; the paladin's and ranger's (50 %) Blessing and Barkskin (3) at 6. The test's other clauses (a full caster a cantrip at 1 and four to six spells; the others at most two boons of level 5 or less, "tier 1 or less" re-read) stay. `class-levels.md` prints "(level 11, from 12)" for "(3, 12)".

### 5.7 Loot: scrolls and wands

`level`, `frequency`, and `value` are already each object's own; nothing in the code derives them from the tier, so **no file changes**. The convention for new ones, stated in the objects' `README.toml`: a scroll's item level is about **half its spell's level**, rounded up, at least 1 (today's 1 / 2 / 4 / 6 at spell levels 0–1 / 3–4 / 6–7 / 11–12), its frequency 60 at item level 1 falling to 20 at 6, its value 40 to 200 the same way; a wand's item level about **3 + half its spell's level** (fireball: 8), frequency 10 falling to 5, value 250 to 600. The `spell_scrolls_drop_by_tier` test becomes `..._by_level` (lower spell levels at lower item levels).

### 5.8 The level of every spell

Old tier → new level, by today's cost first (so mana moves little), then by power within the old tier (the stronger spell a level or two higher), with the class grants of §5.6 as hard bounds (Cure wounds and Blessing at most 3; Remove disease and Call lightning at most 12). Mana is the shape and resolution of today, no armour, before talents.

| Spell | Old tier | **Level** | Mana today → proposed | Why this level |
|---|---|---|---|---|
| Light | 0 | **0** | 1 → 1 | the first things anyone learns |
| Evil eye | 0 | **0** | 1 → 1 | as Light; the witch's at 1 |
| Spark | 0 | **1** | 1 → 1 | the attack cantrips a step above |
| Mend | 0 | **1** | 1 → 1 | |
| Ray of frost | 0 | **1** | 1 → 1 | |
| Barkskin | 1 | **3** | 3 → 3 | the ranger's and druid's at 3 and 6 |
| Blessing | 1 | **3** | 3 → 3 | the priest's at 3 |
| Hex of weakness | 1 | **3** | 3 → 3 | the witch's at 3 |
| Curse of sloth | 1 | **3** | 3 → 3 | |
| Magic missile | 1 | **3** | 3 → 3 | the wizard's at 3 |
| Cure wounds | 1 | **3** | 2 → 2 | the priest's at 3 |
| Cause wounds | 1 | **4** | 2 → 3 | a 2d8+2 top; a step above the hexes |
| Burning hands | 1 | **4** | 5 → 6 | a cone of two or three |
| Arcane bolt | 2 | **6** | 11 → 11 | |
| Sleep | 2 | **6** | 11 → 11 | |
| Identify | 2 | **6** | 60 → 60 | its own `mana` |
| Frost | 2 | **6** | 14 → 14 | |
| Web | 2 | **7** | 11 → 13 | Webbed blocks a step; above Sleep |
| Scorching ray | 2 | **7** | 6 → 7 | a 4d6 top on a single target |
| Blight | 2 | **7** | 14 → 17 | Frost's burst and two degrees of Poisoned |
| Haste | 3 | **11** | 11 → 11 | |
| Remove disease | 3 | **11** | 11 → 11 | the priest's at 12 |
| Lightning bolt | 3 | **11** | 20 → 20 | |
| Call lightning | 3 | **11** | 20 → 20 | the druid's at 12 |
| Fireball | 3 | **11** | 26 → 26 | |
| Flame | 3 | **12** | 26 → 29 | a 3d6 base to Fireball's 2d6 |

Eighteen costs unchanged; seven move by one to three mana. Levels 13 to 70 are empty, as tiers 4 to 9 were: the room the content needs (§4.5).

### 5.9 Everything else that read a tier

| Reader | Today | Proposed |
|---|---|---|
| `SpellData::tier`, `SpellRules::{comfortable, cost, ceiling, max_lift, wand_charges, overreach_cost, tier_of}` | the table | `SpellData::level`; `SpellRules::{shapes, resolution, ceiling: Ceiling { base, per_level }}`; `SpellRules::ceiling_for(level, resolution)` |
| `Content::spells_by_tier` (the menu's and `Ability::every`'s order) | tier, then path | `spells_by_level`: level, then path |
| `learn.rs` (`study_odds`, `study_turns`, `Odds::overreach`, `Actor::known_tiers`) | tiers | levels (§5.4); `known_levels` |
| `cast.rs` cost (`SpellRules::mana`, `mana_with`, `TalentMods::overreach_cost_percent`) | tier price and surcharge | `max(1, level)`; the surcharge and the talent mod gone (`study_overreach_percent` comes, §7.2) |
| `content/wand.rs` (`charges` default) | the tier's | required |
| `balance_probe.rs` (`comfortable`, the lift) | tiers | §9.4 |
| `dump.rs`: `spells` layer `Fireball (tier 3, 26 mana)`, `progress`'s `study c frost: overreach 2 ...` | tiers | `Fireball (level 11, 26 mana): 4d6 fire, range 11, radius 2` (§8); `overreach 5` in levels |
| `ui/abilities_screen.rs` "tier 3, 26 mana, aimed" | tiers | §8: the dice for the tier |
| the content test `spells within their tier` (`spell_list_tests`, `caster_tests`) | tiers | §5.6 |
| `xtask/src/wiki/spells.rs` (`tiers`, `list`, `does`, `wands`, `class_spells`, `wand_charges`), `learning.rs` (`times`, `chances`) | tiers | `tiers` and `wand_charges` deleted; `list` gets a Level column and sorts by it; `times` and `chances` sample spell levels 0, 3, 6, 11, 16, 22, 29, 37, 46, 56 (one row each, the levels the spells and their successors sit) |
| `wiki/src/magic.md` ("Tiers", "Mana", "Caster level", "Learning"), `scrolls.md`, `classes.md`, `features.md` (the lines naming tiers) | tiers | "Spell levels" (§8); `features.md`: "spells of levels 0 to 70" |
| spec §8 "Spells" (`spec/08e-blood-and-status.md`: **Tiers**, **Cost**, **Learning**, **Scrolls of spells**, **Wands**; `08i-character.md`'s class lists "(tier 0, at 1 ...)"; `08g-magic-items.md`'s wand bullet) | tiers | levels, the bullets rewritten (§8) |
| `data/spells/README.toml`, `spells.toml`, `study.toml`, `actions.toml` (the `study` line), the talents' README (`overreach_cost_percent`) | tiers | levels |
| `docs/dev/spells.md` (Data, Pure helpers, Cost, The first list, Learning, Wands, Balance, Wiki, Tests) | tiers | levels |
| traps (`data/traps.toml`), creatures (their `tier` is a roster word, not a spell's), affix tiers (`content/affix.rs`: an affix's `tiers` are item-level steps, unrelated), background tiers | not spell tiers | untouched |

**Saves**: nothing saved names a tier (`Actor::spells` holds spell ids by index; `known_tiers` was derived). **Saves do not change** in the level slice; the content fingerprint does (every spell file). `FORMAT_VERSION` untouched.

## 6. The data format and its checks

### 6.1 `[spell.grows]`

```toml
[[spell]]
id = "fireball"
level = 11                            # was tier = 3 (§5)
# ...
range = 10
radius = 2
damage = "2d6"                        # the base roll, a Roll (§6.2); was { dice, per_levels, max_dice, bonus }

[spell.grows]                         # all optional; a dimension left out never grows by stats
damage = "intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2"
range  = "willpower 14 11, 18 12"
```

A ladder is one string: the stat's id, then rungs separated by commas, each `AT VALUE` with one space. `AT` is a score, 1 to 40, strictly ascending. `VALUE` is a `Roll` for `damage` and `heal`, an integer for `range`, `length`, `radius`, `degrees`. One string, so the wiki prints it and a reviewer reads it as the player will. The keys:

| Key | For | Base field | Value | Bounds |
|---|---|---|---|---|
| `damage` | a `damage` spell | `damage` | a `Roll` | each rung's `max` ≥ the one before, `min` ≥ the one before, not identical; the base's and every rung's `max` ≤ the level's ceiling × the resolution's percent |
| `heal` | a `heal` spell | `effect.heal` | a `Roll` | as `damage` |
| `range` | ray, bolt, burst | `range` | cells | strictly ascending from the base, 1 to `MAX_REACH` (20) |
| `length` | cone, line | `length` | cells | strictly ascending from the base, a cone to `MAX_CONE` (5), a line to 20 |
| `radius` | burst | `radius` | cells | strictly ascending from the base, to the burst table's length (3) |
| `degrees` | buff, hex | `effect.severity` | degrees | strictly ascending from the base, to the status's `max` and `MAX_SEVERITY` |

`aim = "dexterity"` (a stat id; default Dexterity) on an `attack` or `touch` damage spell replaces `[spell.scales] to_hit`.

Retired fields: `tier` (now `level`), `[spell.damage] dice, per_levels, max_dice, bonus` (now `damage = "2d6"`), `max_range`, `max_length`, `max_radius` (the top rung is as far as stats go; a spell without a ladder reaches its base), `[spell.scales]` (all four), `[spell.scroll] level`, `[tiers]` entire. `README.toml` and `spells.toml`'s comments are rewritten with the grammar above and §6.4's conventions.

### 6.2 `Roll`: dice of more than one size

`Dice` is one group (`count`, `sides`, `bonus`) and stays what it is for stats, weapons, and everything else. A **`Roll`** (`dice.rs`, beside it) is a sum of `Dice` groups and one bonus: text `3d6+1d10`, `5d6+2`, `2d8`, `+3`; groups joined by `+`, at most `MAX_GROUPS` (4), the bonus last and once; sides 2 to `MAX_SIDES` (20; a d7 and a d9 are dice like any other); `min`, `max`, `mean_x2` (twice the mean, an integer), `roll(rng, reroll)` (each group in order, one draw a die; a die at or below `reroll` is drawn once more, §7.1), `Display` as written; `From<Dice>`; `grow(sides)` (every group's sides + n), `more(dice)` (n dice more of the first group's kind), `plus(bonus)`. `Attack::damage` in `combat` becomes a `Roll` (a `Dice` converts), so `combat::strike` rolls a spell's mixed dice as it rolls a blow's; `reflect::Shot::dice` the same. Nothing else changes type: a weapon's `Dice` becomes a one-group `Roll` at the one place a `Profile` is built. S0 (§12) lands this alone, with no behaviour change, so the ladders' PR is about spells only.

### 6.3 Validation (`spell_def::resolve`, a `ContentError` naming the spell)

1. `level` 0 to `MAX_SPELL_LEVEL`. A ladder only for a dimension the spell has: `damage` for a damage roll, `heal` for a heal, `range`/`length`/`radius` for the shape that has them, `degrees` for a buff or hex. `aim` only for an attack or touch damage spell.
2. The stat is one of `Stat::ALL`; at least one rung; at most `MAX_RUNGS` (8); scores 1 to 40, strictly ascending.
3. Values as the table in §6.1: `Roll`s that can do harm (`min` ≥ 0, `max` ≥ 1) and never weaken (`max` and `min` non-decreasing, each rung different from the one before); cells and degrees strictly ascending from the base and within the shape's or status's bounds.
4. The ceiling (§4.6) on the base and every damage or heal rung.
5. A wand's `rungs` range (§7.4) within the spell's damage ladder length; a spell scroll's `rungs` the same.
6. Tests: the shipped data passes; a rung below its base, a descending ladder, a stat the spell has no dimension for, an unknown stat, a rung past the ceiling, a radius past the table, a degree past the status's max, nine rungs, a wand with too many rungs, a level of 71, a `tier` key (refused with "use `level`") are each refused with the words of the error.

### 6.4 Authoring conventions (templates, not rules)

The owner's three universal shapes are the **recommended ways to write a ladder**, stated once in `README.toml`, so most spells read alike without the player having to learn them as rules (the row is still the truth):

| Shape | Writes as | Typical use |
|---|---|---|
| **[A] cells or degrees, one more a rung**, Willpower for reach at scores 14, 18, 22, 26; Presence for degrees at 14, 18 | `range = "willpower 14 11, 18 12"`, `degrees = "presence 14 2, 18 3"` | every aimed spell, every buff and hex |
| **[B] a die more a rung**, the spell's stat at 13, 16, 19, 22, 25 | `damage = "intelligence 13 3d6, 16 4d6, 19 5d6"` | the body of a damage ladder |
| **[C] a bigger die a rung** | `damage = "willpower 19 2d8"`, `"intelligence 16 1d8"`, `"willpower 22 2d7"` | cantrips; a rung where another die would pass the ceiling; a d7 or d9 where a die more is too much and a bonus too flat |
| **[D] a static bonus at the top** | `25 5d6+2` | the last rung, where a die would pass the ceiling |
| **mixed dice** | `25 3d6+1d10` | sparingly, where a spell's fiction wants it (a hotter core) |

The stat by school, unless the spell's text says otherwise (decision 3: the spell decides, not the caster's class): evocation, transmutation, divination **Intelligence**; necromancy, abjuration, restoration **Willpower**; enchantment **Presence**; reach **Willpower** always; degrees **Presence** always. Call lightning names Willpower (the druid's spell). Rung scores are on the same grid for every spell (13 / 16 / 19 / 22 / 25 for dice; 14 / 18 / 22 / 26 for cells; 14 / 18 for degrees; decision 14, a convention, not a check) so a player who learns one row knows where to look in the next. The level by power (§5.8): a new spell takes the level of its mana, and its author checks §4.6's line holds its top rung.

### 6.5 Code sketch: what goes, what comes

| Goes | Comes |
|---|---|
| `content/spell_scales.rs` (`Weights`, `SpellScales`, `ScalesSpec`, `Has`), `MAX_SCALE_WEIGHT` | `content/spell_ladder.rs`: `Ladder { stat, rungs: Vec<(u8, RungValue)> }`, `RungValue::{Roll(Roll), Cells(u8), Degrees(u16)}`, the parser, `Ladder::rung_at(score) -> usize`, `Ladder::value(rung) -> &RungValue`, `MAX_RUNGS` |
| `SpellDice` (`count`, `sides`, `per_levels`, `max_dice`, `bonus`, `at`, `lifted`) | `SpellData::damage: Option<Roll>`, `effect.heal: Option<Roll>`, `SpellData::grows: Grows { damage, heal, range, length, radius, degrees: Option<Ladder> }`, `SpellData::aim: Stat`, `SpellData::level: u8` |
| `SpellRules::{comfortable, cost, ceiling, max_lift, wand_charges, overreach_cost, tier_of, lift, lifted_cap, grown, lengthened, max_bonus, mana_with}` | `SpellRules::{ceiling: Ceiling, ceiling_for(level, resolution)}` (the check only), `SpellRules::mana(spell, armour)` (no caster level) |
| `world/spell_scale.rs` (`spell_lift`, `spell_cap`, `spell_reach_more`, `spell_degrees_more`) | `world/spell_mods.rs`: `SpellMods` (§7.1) and `Actor::spell_mods(content, spell) -> SpellMods`; `world/spell_rung.rs`: `Actor::spell_rung(...)`, `Actor::spell_roll(content, spell, pay) -> Roll` (the roll as the menu shows it), `Actor::spell_reach(...)`, `Actor::spell_degrees(...)`, `Actor::spell_to_hit` (the `aim` stat's modifier + *of the Magus*) |
| `Cast::cap`, `spell::capped`, `Burst::cap`, `Bolt::cap`, `cast.rs::damage_dice` | `Cast::roll: Roll`, `Cast::reroll: u8`, `Cast::riders: Vec<(StatusId, u16)>` |
| `TalentMods::{spell_dice, spell_per_die, focus_lift, wand_level, overreach_cost_percent}`, `Dice::empowered` | `TalentMods::spell: Vec<SpellMod>` (each a `SpellMods` with a tag filter; §7.1), `study_overreach_percent` |
| `ItemEffect::{SpellLift, WandLevel}`, `Wand::caster_level`, `Actor::wand_caster_level`, `[spell.scroll] level`, `ScrollSpell::level` | `ItemEffect::{SpellRungs, SpellBonus, SpellSides, SpellDice, SpellReroll, WandRungs}` (the `spell_reach`, `spell_duration`, `spell_to_hit`, `spell_threat` effects stay as they are), `Wand::rungs: Range`, `Stack::wand_rung: Option<u8>`, `ScrollSpell::rungs` |
| `max_range`, `max_length`, `max_radius`, `SpellData::base_reach` | the base fields alone; `Reach` from the ladders |
| `RodEffect::{max_range, per_levels, up_to_caster_level}` | `RodEffect::{grows: Option<Ladder>, up_to_level: Option<u8>}` |

The cast path (`world/cast.rs`) changes at five lines: the roll (`Actor::spell_roll` for a mana cast; the item's rung plus the user's for a scroll or wand), the reroll and riders carried on the `Cast`, the reach (`Actor::spell_reach`), the degrees (`Actor::spell_degrees`), and the removal of every `capped`. `reflect::Shot` carries a `Roll`. The mishap backfire casts the base. `learn.rs`'s `read_spell` (the three old scrolls) goes with them (§7.3).

## 7. Talents, items, scrolls, wands, rods, traps: the hooks

Decision 5, verbatim: *"disagree. talents and items can do whatever their author wishes. Flat damage, different dice, a rider. This is not a concern of the magic progression per-se. Talents primarily tied to spell progression can work that way. Regardless, i want the system to mechanically be able to support riders from talents."* Round 1's "rungs are the one external unit" is withdrawn.

### 7.1 Six hooks, one struct

Every talent and affix that touches a spell writes to one of six places, and the cast path reads them in one order. They are the fields of **`SpellMods`**, summed once per cast from the caster's talents and worn items (`Actor::spell_mods(content, spell)`; worn items unidentified too, as #629 has it), each source filtered by the spell's **tags** (its school, element, shape, kind: the `school_focus` filter that exists, `when = ["fire"]`; none = every spell):

| # | Hook | Field | What it does, in the player's words | Applied |
|---|---|---|---|---|
| 1 | **rung** | `rungs: i8` (damage and heal, own casts), `scroll_rungs` (the spell scrolls read), `wand_rungs` (the wands used), `reach_rungs`, `degree_rungs` | "your spells stand a rung higher on their ladders" | before the lookup: `rung = min(by_stats + rungs, len)` (§4.3); a scroll or wand takes the item's rung plus `scroll_rungs` or `wand_rungs` in place of `by_stats + rungs` |
| 2 | **dice** | `sides: u8` (every die a size bigger: d6 → d7), `dice: u8` (that many dice more, of the roll's first group's kind), `bonus: i16` (added to the roll's bonus) | "bigger dice", "a die more", "+2 on every spell" | to the ladder's `Roll` after the lookup, in that order: `roll.grow(sides).more(dice).plus(bonus)`; the result is **what the menu shows** (§8) |
| 3 | **roll** | `reroll: u8` (a die showing this or less is rolled once more) | "a die that shows a 1 is rolled again" | at the draw, per die, once; a critical repeats the whole roll stage, as a blow's does |
| 4 | **rider** | `riders: Vec<(StatusId, severity)>` | "what your cold spells harm is slowed a while" | after the harm, on every target the spell harmed, as the spell's own `effect.status` is laid (merged: the same status's severities add); nothing on a miss or on a heal |
| 5 | **reach** | `cells: i8` (added to range or length after the reach ladder), `radius_cells` (a burst's radius) | "your spells reach a cell farther" | after the ladder; held by `MAX_REACH` and the burst table only |
| 6 | **degrees** | `degrees: i8` | "your buffs and hexes last a degree longer" | after the ladder; held by the status's `max` only (`Effects::add`, as today) |

Cost (`spell_mana_percent`, School focus's cost, Frugal caster, *of Thrift*) and to-hit (*of the Magus*, the Spell attacks skill, the styles) and threat (*Baleful*) are hooks already and stay where they are; they are named here so an author sees the whole list in one place. **Order**: rung → dice → roll (with rerolls; twice on a critical) → the harm → riders; reach and degrees beside. **No clamp** after any of them (decision 4): the ceiling is an author's check on the ladder, not on the mods. A **scroll or wand** gets the reader's `SpellMods` with `rungs` replaced by the item's own plus the reader's `scroll_rungs` or `wand_rungs` (§7.3, §7.4); the other five hooks apply as to an own cast (your Sure hand rerolls a scroll's ones too: the dice are yours to throw). The **mishap backfire** gets none.

**Why six and not one**: a rung respects a spell's own ladder (Empower on a cantrip is 1d8, on Fireball 4d6, each as its author wrote), which makes it the right unit for talents *about the ladder* (decision 5: "talents primarily tied to spell progression"); a flat bonus is what an item does to a weapon and reads at once on a ring; a die size is the owner's d7; a reroll is the owner's "tedious on tabletop"; a rider is the owner's explicit requirement; cells and degrees are what School focus, *of Farcasting*, and *of Lingering* already do. Each is one integer or one list, one sentence, one line of code where it applies. The data: a talent effect `spell_mod { rungs = 1 }`, `spell_mod { sides = 1, when = ["evocation"] }`, `spell_mod { reroll = 1 }`, `spell_mod { rider = { status = "sluggish", severity = 1 }, when = ["cold"] }` (`TalentEffect::SpellMod`, replacing `SpellPower`, `SchoolFocus`'s `lift` and `reach`, `WandLevel`); an affix effect per field (`spell_rungs`, `spell_bonus`, `spell_sides`, `spell_dice`, `spell_reroll`; `spell_reach`, `spell_duration`, `wand_rungs` as today's names), a tag filter on the affix as on a talent.

### 7.2 Talents (old → new), and three new ones

| Talent | Today | Proposed | Hook | Wiki row |
|---|---|---|---|---|
| Empower (wizard 32) | `spell_power { dice = 1 }`, −10 % mana | `spell_mod { rungs = 1 }`, −10 % mana | rung | your spells stand a rung higher on their ladders, and cost a little less |
| Archmage (wizard 60) | `spell_power { per_die = 2 }`, −50 % mana | `spell_mod { rungs = 2 }`, −50 % mana | rung | two rungs higher, half the mana |
| Keen mind (Spellcraft, a school or element) | `school_focus { lift = 10 }` | `spell_mod { sides = 1, when = [the choice] }` | dice | that school's spells roll dice a size bigger for you: a d6 is a d7, a d8 a d9 |
| School focus (Spellcraft, a school or element) | `school_focus { cost = -10, reach = 1 }` | `spell_mod { cost = -10, cells = 1, when = [the choice] }` | cost, reach | as today: cost 10 % less and a cell farther |
| Scroll adept (wizard 1) | `spell_power { per_die = 1 }` on spells and scrolls, reading −50 % | `spell_mod { scroll_rungs = 1 }`, reading −50 % (decision 12) | rung, scrolls only | scrolls are your craft: you read them in half the time, and the spell scrolls you read cast a rung higher |
| Wand mastery (Weaponcraft) | `wand_level { value = 2 }` | `spell_mod { wand_rungs = 1 }` | rung, wands only | the wands you use cast a rung higher |
| Overreach (Spellcraft 6) | `overreach_cost_percent −33` (the casting surcharge) | `study { overreach_percent = -33 }`: the overreach counts two-thirds in the study check (rounded down) | learning | a spell beyond your learning is a third less beyond you |
| Arcane training (wizard 1) | `caster_level +1 per 5` | unchanged data; learning only | learning | your learning outruns your years: spells of a higher level come within your certain reach sooner |
| Attuned (Scholar) | `attuned_percent 50` on items' caster level | unchanged data; learning only | learning | its items' learning counts half again |
| Channeler, Frugal caster, Quick study, Spell recall, Scroll savant, Careful reader | — | unchanged | — | — |
| **Sure hand** (Spellcraft, level 12, new) | — | `spell_mod { reroll = 1 }` | roll | a spell die of yours that shows a 1 is rolled again, once |
| **Brimming** (Spellcraft, level 25, new) | — | `spell_mod { dice = 1 }`, `spell_mana_percent +10` | dice | your damage and healing spells roll a die more, and cost a tenth more |
| **Numbing cold** (Spellcraft, level 12, element cold, new) | — | `spell_mod { rider = { status = "sluggish", severity = 1 }, when = ["cold"] }` | rider | what your cold spells harm is a degree Sluggish |

The three new ones exist so that every hook ships with a user (no speculative code: `reroll`, `dice`, and `rider` would otherwise have none) and show an author the pattern; the owner strikes any that does not belong, and the hook stays tested by a test talent. Systems: all `spells` (Brimming `mana` too); no new `[[system]]` tag, the coverage bar unchanged (`docs/dev/progression.md` "Coverage policy"). `SpellDamagePercent` (the wards against bursts and bolts) is the target's and is untouched.

### 7.3 Affixes (old → new)

| Affix | Today | Proposed | Hook |
|---|---|---|---|
| Potent (weapon, ring, amulet) | `spell_lift` 4–6 / 6–8 / 8–10 / 9–11 / 11–13 / 13–15 by item level 3 to 13 | `spell_bonus` 1 / 2 / 3 at item levels 3 / 7 / 11: "+2 on every spell's roll, harm or healing" (decision 5: a ring reads as a weapon's +2 does) | dice (bonus) |
| of Farcasting (cloak, ring, amulet) | `spell_reach` 1 / 1–2 / 2 | unchanged: cells | reach |
| of Lingering (helmet, ring, amulet) | `spell_duration` 1 / 1–2 / 2 | unchanged: degrees | degrees |
| of Wands (gloves, ring) | `wand_level` 1 / 1–2 / 2 | `wand_rungs`, the same numbers | rung, wands |
| of the Adept (caster level 1 to 3) | learning, cost, dice, scroll and rod levels | learning only; unchanged data | learning |
| of the Magus, Baleful, of Thrift, Mage-wrought | — | unchanged | to-hit, threat, cost |

Affixes are saved by id with a tier and a value; a save that holds a Potent ring loads with its old value (4 to 15) under the new effect. The lenient read (`world/retier.rs`, the affix ladder's regrading) re-tiers it to the first tier that holds the value or the last, and the value is **clamped at the one read to the tier's `max`** (so an old 13 becomes a +3); one line in the loader, said in the PR.

### 7.4 Scrolls and wands

- **Spell scrolls** (`read = { kind = "spell", spell = "fireball" }`): a read casts the spell at damage rung `rungs + the reader's scroll_rungs` (Scroll adept), the reader's other mods applying (§7.1), reach and degrees the reader's, free of mana. `rungs` (default 0, at most the ladder's length) is new on the `read` table, **fixed per scroll kind** (decision 13 is about wands): **the three older scrolls become spell scrolls at a rung**: fireball `rungs = 1` (3d6, as its burst rolled), lightning `rungs = 2` (4d6), frost `rungs = 0` (2d6). Their `burst`/`bolt` readings, `Burst::cap`, `Bolt::cap`, `learn.rs::read_spell`, and the "max of the taught cap and the scroll's own most" rule go. Careful reader, Spell recall, Scroll savant are unchanged. `docs/dev/scrolls.md` and `wiki/src/scrolls.md` lose the paragraph that sets the three apart (they still fly and burst as they did: the spell's shape does that). The wiki's scroll table gets a "Casts at" column (`3d6`, from the rung).
- **Wands** (decision 13: *"wands should randomize their effective rung at loot generation"*): `[object.wand] rungs = { min = 0, max = 2 }` (default `min = 0`, `max` the spell's damage ladder length; `charges` now required, §5.2) replaces `caster_level`. **When a wand stack is made** (`Stack::new` for an object with a `wand`, reached by loot, a shop's stock, a level file's pile, and a kit alike, so every maker passes the stream it already draws from: loot's `loot`, a shop's its own, a level's its `objects`), its rung is drawn evenly in the range and kept on the stack: `Stack::wand_rung: Option<u8>` (`None` for everything but a wand), **saved and hashed, left out while `None`** so other stacks save as before: **saves change** (S5). A zap casts at `wand_rung + the user's wand_rungs` (Wand mastery, *of Wands*), the user's other mods applying (§7.1), reach and degrees the user's. **What the player sees**: a wand's type is learned by use or identification (as today); its rung comes with the type: the description's line is "Holds Fireball: 4d6 fire, radius 2" (the dice at its rung and the user's wand rungs, as the menu shows a spell's), the aiming hint the same once the type is known, the name stays "a wand of fireball". Before the type is known, nothing. Dev: the `inventory` and `actors` layers add `(rung 2)`. The seven wands ship with `rungs = { min = 0, max = 2 }` and today's charges (burning hands 12, magic missile 12, scorching ray 9, sleep 9, web 9, fireball 7, lightning bolt 7); `wands.md` shows "Strength: 2d6 to 4d6" (the rolls of the range's ends) in place of "Caster level". A save made before S5 has no `wand_rung`: read as 0 (the base), no refusal.
- **Loot**: §5.7.

### 7.5 Rods, traps, mishaps, reflection, creatures

- **Rods**: `range`, `max_range`, `per_levels` on a rod effect become `range` and `grows.range = "willpower 14 ..., 18 ..."` (the same grammar, `RodEffect::grows`), so the wiki's rod row reads as a spell's: digging `3; Willpower 14: 4, 18: 5, 22: 6, 26: 7` (today 3 + 1 per 3 caster levels to 12: a level-30 wizard reaches 12 today, 6 proposed; §9.2 says why). Sleep's and terror's `up_to_caster_level` becomes **`up_to_level = 20`** on the effect (decision 10: no caster-level gate; "a rod the same in anyone's hands", the wiki's own words; today a level-30 wizard's rod took creatures to level 36, a level-10 one's to 12): a creature above it "shrugs it off", as today's wording. Channeler unchanged.
- **Traps** (`data/traps.toml` bursts and bolts): untouched; `content/spell.rs` keeps `Burst` and `Bolt` for them (the `cap` fields removed).
- **Mishap backfire**: the taught spell's base, no mods, as today's "held to the taught spell's cap" becomes "its base".
- **Spell reflection**: the `Shot` carries the caster's `Roll` and `reroll`; the reflector casts it back as is; riders are the caster's and come back with it.
- **Creatures**: creature spells are intents, not the cast path (`docs/dev/spells.md`); their caster level (#525) still feeds their summons' comfort. Nothing changes.

## 8. What the player sees; the wiki, the spec, the docs

**In the game** (decision 11, verbatim: *"uhm, no. Just put the damage number to the menu as you would with a weapon."*):

- **The abilities menu's Spells rows** (`ui/abilities_screen.rs`): the detail column is the spell's roll as the caster would make it now (`Actor::spell_roll`: the rung by stats, the rungs, the dice hooks; §7.1), its damage type or "healed", its mana, and "aimed" where it is: `Fireball          4d6 fire, 26 mana, aimed`, `Cure wounds       3d6 healed, 2 mana, aimed`, `Blessing          3 mana`, `Sleep             11 mana, aimed`. Short of mana the suffix is as today ("4d6 fire, 26 mana: you have 14"). No level, no tier: a known spell's level is its mana. The style is the character screen's attack line (`+2 to hit, 1d6+1 slashing or piercing`), without the to-hit: an attack spell's to-hit is one more number in a 31-column cell, and the aiming mode is where a shot is weighed; it may be added later. The **description line** for the selected spell appends the reach as it is now, in cells: *"... Range 11, radius 2."* (`PerceptionView::own_spell_reach`, which exists). The aiming mode's reach is the rung reached, as today.
- **The character screen** is unchanged: its attacks column holds three (main hand, off hand, missile) and a wizard knows six damage spells; spells have the menu.
- **No "your spell grows" line** (decision 11: no). A stat rising says what it says today; the menu shows the new dice next time it opens.
- **Wands**: §7.4.
- The dump `spells` layer prints what the menu does: `Fireball (level 11, 26 mana): 4d6 fire, range 11, radius 2`; JSON `spells[]` gains `level`, `roll` (the text), `damage_type`, `reach` (`range`, `length`, `radius`, each null where none); dev `progress` adds `fireball rung 2/4 (In 16; +1 Empower)`. The `--screen abilities` snapshot `abilities-menu-spells` changes; a new case `abilities-menu-spells-grown` (a wizard at Intelligence 19 with Empower) shows a climbed ladder and a rung from a talent; `wand-description` changes (the dice line).

**In the wiki** (exact; the generator `xtask/src/wiki/spells.rs`):

- `spell-list.md`: columns Spell, **Level**, School, Shape and resolution, Reach (the base: `range 10, radius 2`), What it does (the base: `2d6 fire damage; it melts ...`), **Grows with** (each ladder as written, in words: `Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2; range Willpower 14: 11, 18: 12`; `—` without), Mana (at its level, no armour). Sorted by level, then name.
- `spell-tiers.md` deleted (the `{{#include}}` with it); `spell-resolutions.md`: Resolution, Roll, Cost. `study-times.md` and `study-chances.md` sample spell levels (§5.9). `wands.md`: Strength for Caster level, Charges as before. `rods.md`: Reach as a spell's; "Takes creatures to level 20" for sleep and terror. `class-levels.md` and `talents.md` inherit `does`; `class-levels.md` prints "(level 11, from 12)". `scrolls.md`'s spell scroll table: "Casts at".
- `magic.md`: "The spells" explains the ladder in the words of §4.2; **"Tiers" becomes "Spell levels"** (§3.1 row 5, one paragraph, no table); "Mana" loses the overreach sentence (row 12); "Caster level" is rewritten as learning (row 11); "Spellcraft talents" and "Damage and its cap" are replaced by **"How a spell grows"** (the rule; what talents and items may do, one sentence per hook with the shipped example; scrolls and wands at their rungs; criticals as blows); "The lifted cap" is deleted; "Shapes" loses its Willpower and Presence paragraphs (one sentence each pointing at the rows); "Learning" in levels (§5.4's sentence and the sample tables); "Wands" and "Rods" as rows 13 and 14. `scrolls.md` row 15. `magic-items.md`'s Potent, of Wands, of the Adept lines. `stats.md`'s Intelligence, Willpower, Presence entries say "grows the spells whose ladders name it". `classes.md` and `features.md`: "by its level" for "by its tier".

**The spec and the docs**: spec §8 "Spells" (`spec/08e-blood-and-status.md`) bullets **Spells**, **Tiers** (→ **Levels**), **The cap**, **The lift**, **Cost**, **Casting: Reach, Dice**, **Caster level**, **Learning**, **Scrolls of spells**, **Wands**, **Rods** rewritten to the proposal (the lift bullet deleted, a **Ladders** and a **Hooks** bullet added); `08i-character.md`'s class lists "(level 1, at 1 ...)"; `08g-magic-items.md`'s wand bullet; `docs/dev/spells.md`'s Data, The cap, Pure helpers, The caster's stats, The one cast path (Cost, Reach, Dice), Choices, Learning, Wands, Rods, Talents, Balance, Wiki, Tests sections; `docs/dev/scrolls.md`, `docs/dev/magic-items.md` (the affixes, `Stack::wand_rung`), `docs/dev/saves.md` (the wand rung), `docs/dev/progression.md` (the talents), `docs/balance.md` (§9.4); `README.toml` and `spells.toml`'s comments; `docs/design/README.md`'s entry for this document.

## 9. Migration: every spell, with before and after

### 9.1 Damage and healing ladders (all 15 spells that roll)

Unchanged from round 1 (decisions 6, 7, 14, 15: the cantrips grow once, Blight, Cause wounds, and Mend get Willpower ladders, the grid stands, Scorching ray keeps its one mixed rung). The base is what the spell rolled at its old comfortable caster level today (its `dice`), so a scroll or a wand at rung 0 is unchanged. "Top" is the last rung's most, against §4.6's line at the spell's new level (§5.8), times the resolution's percent.

| Spell (level, resolution) | Today | Proposed `damage` / `heal` | Top ≤ ceiling | Shift from today |
|---|---|---|---|---|
| Ray of frost (1, attack) | 1d6, no growth, cap 6 | base `1d6`; `intelligence 16 1d8` | 8 ≤ 13 | grows once (today never) |
| Spark (1, auto) | 1d4, no growth, cap 4 | base `1d4`; `intelligence 16 1d6` | 6 ≤ 9 | grows once |
| Mend (1, touch heal) | 1d3, no growth, cap 7 | base `1d3`; `willpower 16 1d4, 22 1d6` | 6 ≤ 15 | grows twice |
| Magic missile (3, auto) | 1d4 +1 die per 3 to 2d4; lift In 10 Wi 5, cap 9 → 13 | base `1d4`; `intelligence 13 2d4, 16 3d4, 22 3d4+1` | 13 ≤ 14 | same at 10 to 40; +1 at 60 |
| Burning hands (4, area) | as Magic missile | base `1d4`; `intelligence 13 2d4, 16 3d4, 22 3d4+2` | 14 ≤ 17 | same at 10 to 40; +2 at 60 |
| Cause wounds (4, touch) | 1d6 +1 per 4 to 2d6; no lift; cap 14 | base `1d6`; `willpower 13 2d6, 19 2d8, 25 2d8+2` | 18 ≤ 26 | grows at 30 and 60 (today flat from level 7) |
| Cure wounds (3, touch heal) | 2d6; lift Wi 15, cap 14 → 21 | base `2d6`; `willpower 16 3d6, 22 3d6+3` | 21 ≤ 22 | same to 30; +3 at 40 on |
| Scorching ray (7, attack) | 1d6 +1 per 6 to 3d6; lift; cap 18 → 28 | base `1d6`; `intelligence 13 2d6, 16 3d6, 19 4d6, 25 3d6+1d10` | 28 ≤ 31 | 2d6 from In 13 (today 1d6 to caster level 12); −1 die at 20; same at 10, 30, 60 |
| Arcane bolt (6, area) | 2d6 flat; lift; cap 14 → 22 | base `2d6`; `intelligence 13 3d6, 19 3d6+2, 25 3d6+4` | 22 ≤ 22 | same to 20; +2 at 30 on |
| Frost (6, area) | as Arcane bolt | base `2d6`; `intelligence 13 3d6, 19 3d6+2, 25 3d6+4` | 22 ≤ 22 | as Arcane bolt |
| Blight (7, area) | 2d6 flat; no lift; cap 14 | base `2d6`; `willpower 16 3d6, 22 3d6+3` | 21 ≤ 24 | grows (today never): +1 die from Wi 16 |
| Fireball (11, area) | 2d6 +1 per 4 to 3d6; lift; cap 19 → 32 | base `2d6`; `intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2` | 32 ≤ 34 | +1 die at 10; −1 at 20; same at 30, 60 |
| Flame (12, area) | 3d6 flat; lift; cap 19 → 32 | base `3d6`; `intelligence 16 4d6, 19 5d6, 25 5d6+2` | 32 ≤ 36 | same at 10, 30, 60; −1 at 20 |
| Lightning bolt (11, area) | as Fireball | base `2d6`; `intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2` | 32 ≤ 34 | as Fireball |
| Call lightning (11, area) | as Fireball | base `2d6`; `willpower 13 3d6, 16 4d6, 19 5d6, 25 5d6+2` | 32 ≤ 34 | as Fireball; Willpower (the druid's stat, decision 3) |

The `aim` of Ray of frost, Scorching ray, Cause wounds, Mend, and Cure wounds stays Dexterity (today's default; no spell names another). Identify, Remove disease, Light, Barkskin, Blessing, Haste, Evil eye, Curse of sloth, Hex of weakness, Sleep, Web roll nothing.

### 9.2 Reach and degrees ladders (every aimed spell, every buff and hex, the rods)

Today: a cell per 2 points of Willpower's modifier up to `max_range` (Willpower 14: +1, 18: +2), a degree per 2 points of Presence's up to the status's max (the hexes with a quarter of Willpower's on top, dropped: decision 8). The ladders below are **exactly today's values** at the scores where today's rounding changed them, so no reach or duration moves for anyone.

| Spell | Today | Proposed |
|---|---|---|
| Evil eye, Curse of sloth, Hex of weakness (ray, range 6 to 8; a hex, 1 to 3) | Willpower reach, Presence 50 + Willpower 25 duration | `range = "willpower 14 7, 18 8"`; `degrees = "presence 14 2, 18 3"` |
| Ray of frost (range 6 to 8) | Willpower | `range = "willpower 14 7, 18 8"` |
| Spark (range 5 to 6) | Willpower | `range = "willpower 14 6"` |
| Magic missile, Scorching ray, Blight, Sleep, Web (range 8 to 10) | Willpower | `range = "willpower 14 9, 18 10"` |
| Fireball, Flame, Frost (range 10 to 12, radius 2 fixed) | Willpower | `range = "willpower 14 11, 18 12"`; no `radius` ladder (today `max_radius` = `radius`) |
| Arcane bolt (line 6 to 8) | Willpower | `length = "willpower 14 7, 18 8"` |
| Call lightning (line 8 to 10) | Willpower | `length = "willpower 14 9, 18 10"` |
| Lightning bolt (line 8 to 12) | Willpower | `length = "willpower 14 9, 18 10, 22 11, 26 12"` |
| Burning hands (cone 3, fixed) | — | none |
| Light, Barkskin, Blessing, Haste (a buff, 1 to 3) | Presence | `degrees = "presence 14 2, 18 3"` |
| Sleep (Drowsy 2, max 3) | Presence + Willpower | `degrees = "presence 14 3"` |
| Web (Webbed 3, max 6) | Presence + Willpower | `degrees = "presence 14 4, 18 5, 22 6"` |
| Rod of digging (line 3, +1 per 3 caster levels, to 12) | caster level | `range = 3`, `grows.range = "willpower 14 4, 18 5, 22 6, 26 7"` |
| Rod of teleport other (bolt 4, +1 per 3, to 12) | caster level | `range = 4`, `grows.range = "willpower 14 5, 18 6, 22 7, 26 8"` |
| Rods of sleep and terror (burst 5, +1 per 5, to 10; creatures to the caster level) | caster level | `range = 5`, `grows.range = "willpower 14 6, 18 7, 22 8"`; `up_to_level = 20` |

The rods lose reach at high level (a level-30 wizard digs 12 cells today, 6 proposed): their reach was the one place a rod grew with the user at all, and a rod "the same in anyone's hands" fits a flat-ish reach better; the rods' value is the effect, not the range (decision 10: "sure").

### 9.3 Before and after: the numbers

Computed for this document by a script over the rules as written (today: `SpellDice::at` and `lifted` with the lift, the roll's expectation held to the lifted cap, exact; proposed: the ladder's rung at the score; mana by §5.3), re-run for round 2 with §5.8's levels, for the **trained wizard** of `docs/balance.md` (Intelligence 16 / 18 / 20 / 23 and Willpower 16 / 18 / 20 / 25 at levels 10 / 20 / 30 / 60; caster level 12 / 24 / 36 / 72 with Arcane training) against the **median monster's hit points** of their level (39 / 73 / 116 / 225), no talents; "casts" are expected casts to kill at the mean, per target, hits assumed. The script is not committed: S7 moves the measurement into the balance probe on the real code.

| Spell (level, res) | L | Today: dice, held to | avg | casts | Proposed: rung | avg | casts | Mana today → proposed |
|---|---|---|---|---|---|---|---|---|
| Ray of frost (1, attack) | 10 / 30 / 60 | 1d6 → 6 | 3.5 | 11 / 33 / 64 | 1d8 (In 16 / 20 / 23) | 4.5 | 8.7 / 26 / 50 | 1 → 1 |
| Spark (1, auto) | 10 / 30 / 60 | 1d4 → 4 | 2.5 | 16 / 46 / 90 | 1d6 | 3.5 | 11 / 33 / 64 | 1 → 1 |
| Magic missile (3, auto) | 10 / 20 / 30 / 60 | 3d4 → 13 | 7.5 | 5.2 / 9.7 / 15 / 30 | 3d4 / 3d4 / 3d4 / 3d4+1 | 7.5 / 7.5 / 7.5 / 8.5 | 5.2 / 9.7 / 15 / 26 | 3 → 3 |
| Burning hands (4, area) | 10 / 20 / 30 / 60 | 3d4 → 13 | 7.5 | 5.2 / 9.7 / 15 / 30 | 3d4 / 3d4 / 3d4 / 3d4+2 | 7.5 / 7.5 / 7.5 / 9.5 | 5.2 / 9.7 / 15 / 24 | 5 → 6 |
| Cause wounds (4, touch) | 10 / 20 / 30 / 60 | 2d6 → 14 | 7.0 | 5.6 / 10 / 17 / 32 | 2d6 / 2d6 / 2d8 / 2d8+2 (Wi) | 7.0 / 7.0 / 9.0 / 11 | 5.6 / 10 / 13 / 20 | 2 → 3 |
| Scorching ray (7, attack) | 10 / 20 / 30 / 60 | 3d6 → 26 / 4d6 → 28 / 4d6 / 4d6 | 10.5 / 14 / 14 / 14 | 3.7 / 5.2 / 8.3 / 16 | 3d6 / 3d6 / 4d6 / 4d6 | 10.5 / 10.5 / 14 / 14 | 3.7 / 7.0 / 8.3 / 16 | 6 → 7 |
| Arcane bolt, Frost (6, area) | 10 / 20 / 30 / 60 | 3d6 → 20 / 22 / 22 / 22 | 10.5 | 3.7 / 7.0 / 11 / 21 | 3d6 / 3d6 / 3d6+2 / 3d6+2 | 10.5 / 10.5 / 12.5 / 12.5 | 3.7 / 7.0 / 9.3 / 18 | 11 → 11, 14 → 14 |
| Blight (7, area) | 10 / 20 / 30 / 60 | 2d6 → 14 | 7.0 | 5.6 / 10 / 17 / 32 | 3d6 / 3d6 / 3d6 / 3d6+3 (Wi) | 10.5 / 10.5 / 10.5 / 13.5 | 3.7 / 7.0 / 11 / 17 | 14 → 17 |
| Fireball, Lightning bolt (11, area) | 10 / 20 / 30 / 60 | 3d6 → 27 / 5d6 → 30 / 5d6 → 32 / 5d6 → 32 | 10.5 / 17.5 / 17.5 / 17.5 | 3.7 / 4.2 / 6.6 / 13 | 4d6 / 4d6 / 5d6 / 5d6 | 14 / 14 / 17.5 / 17.5 | 2.8 / 5.2 / 6.6 / 13 | 26 → 26, 20 → 20 |
| Flame (12, area) | 10 / 20 / 30 / 60 | 4d6 → 27 / 5d6 / 5d6 / 5d6 | 14 / 17.5 / 17.5 / 17.5 | 2.8 / 4.2 / 6.6 / 13 | 4d6 / 4d6 / 5d6 / 5d6 | 14 / 14 / 17.5 / 17.5 | 2.8 / 5.2 / 6.6 / 13 | 26 → 29 |
| Call lightning (11, area) | 10 / 20 / 30 / 60 | as Fireball | | 3.7 / 4.2 / 6.6 / 13 | 4d6 / 4d6 / 5d6 / 5d6+2 (Wi 16 / 18 / 20 / 25) | 14 / 14 / 17.5 / 19.5 | 2.8 / 5.2 / 6.6 / 12 | 20 → 20 |

Heals, the **trained priest** (Willpower 17 / 21 / 26 at 10 / 30 / 60): Mend today 1d3 (2.0) at every level, proposed 1d4 / 1d4 / 1d6 (2.5 / 2.5 / 3.5), 1 mana both; Cure wounds today 3d6 → 20 or 21 (10.5) at every level, proposed 3d6 / 3d6 / 3d6+3 (10.5 / 10.5 / 13.5), 2 mana both.

**Reading it**: at levels 10, 30, and 60 every old tier-2 and tier-3 spell is within a die of today, and at 30 and 60 the tier-3 spells are identical (today's growth had already stopped: `max_dice` 3 by caster level 24, the lift at its most by Intelligence 18). Level 20 is one die lower on Scorching ray, Fireball, Flame, and Lightning bolt (today's lift reached 60 % there with two stats summed; the ladder waits for Intelligence 19 at level 25 or so), and level 10 one die higher on Fireball and Lightning bolt. The spells that never grew (the cantrips, Blight, Cause wounds, Mend) now grow once or twice, which is the one deliberate buff. Mana moves on seven spells by one to three points, up, where a spell sits above its old tier's first level; the casts a pool buys move by at most a fifth (Blight: the witch's 90 mana at level 12 buys 5 casts, not 6). Nothing moves by more than a die anywhere, and the level-60 picture (13 casts of a level-11 burst against the median monster) is the same as today's, which §4.5 answers with higher levels, not scaling. The talents' effect on these rows is each talent's own sentence (Empower a rung: Fireball 5d6 at level 32 for the trained wizard; Keen mind: 5d7, 20 on average; Sure hand on 5d6: 19.2).

### 9.4 Mana and the probe

The costs are §5.3 and §5.8; a burst costs by its base radius. The pool check (`docs/dev/spells.md` "Balance" 1: "about ten single-target casts of its tier's base cost at every level") becomes **about ten casts of a spell of the caster's own level**: the wizard's 106 at level 12 against 11, 254 at 30 against 29, 502 at 60 against 56, the same numbers read against the level instead of the tier. The balance probe's "Spells" method (`world/balance_probe.rs`, `docs/balance.md` "Spells") reads `comfortable`, `Actor::spell_cap`, and `SpellDice::lifted` today; S7 rewrites it on `Actor::spell_roll` and prints the rung reached beside the roll (`| Spell (level, res) | L | rung | roll | casts to kill | mana / pool |`), and re-runs the §7.3 cross-check paragraph against the rows above.

## 10. Determinism, saves, replays, fingerprints

- **Draws**: a cast draws its attack roll (if any) and its damage, one draw per die of the `Roll` in group order, one more per die the reroll hook sends back (a fixed rule, so the same state draws the same count); an area's one roll for everyone, in ascending actor id; a buff, hex, or cleanse draws nothing; a rider draws nothing. Where today's roll was one group and no mods, the proposed draws are the same count for the same rung, so a replay with no stat change over a rung verifies if the dice are the same. A wand's rung is one draw from the maker's stream when the stack is made (§7.4): every replay that makes a wand re-hashes (S5).
- **Saved state**: the spell levels (S1), the ladders (S2), the talents' and items' mods, rods (S3), and the menu (S4) save **nothing new**: rungs are derived from `Actor::effective_stats`, the talents, and the items at cast time; `Actor::spells` (spell ids) is unchanged as long as no spell file is added, removed, or renamed (none is: 26 files edited in place, path order kept). **S5 adds `Stack::wand_rung`** (`Option<u8>`, left out while `None`, hashed): **saves change** in S5 and only there; a save without it reads as rung 0. `FORMAT_VERSION` untouched (the integrator's per-release bump).
- **The content fingerprint changes** in S1 (every spell file: `tier` → `level`; `spells.toml`; `study.toml`), in S2 again (every spell file, the talents' and affixes' files, the three scrolls), in S3 (the rod effects), in S5 (the wand files): saves of 0.9.0 do not load until the integrator's per-release bump, as with any content change (`docs/dev/saves.md`). A Potent affix in a save reads its old value under the new effect: clamped at the read (§7.3).
- **Replays** that cast re-hash (`cast_tests`, `spell_talent_tests`, `spell_list_tests`, `affix_spell_tests`, `caster_style_tests`, `learn_tests` fixtures; the showcase's scenarios cast nothing in their golden walks, `tests/golden/manifest/`) and are re-made in S1 (the costs) and S2 (the dice). The state hash (`World::state_hash`) is unchanged in form.
- **Determinism tests** carried over: an attack can miss and critical, auto and area never; Evasion halves an area only; no draws without a roll; a cast at every rung draws the dice of that rung; the rung reached at a score boundary (15 → 16) changes the roll and nothing else; a reroll draws once more per die it sends back and never twice; a rider lands on every target harmed and on none missed; a wand of the same seed rolls the same rung.

## 11. Choices made along the way, and what is rejected

| Choice | Taken | Rejected, and why |
|---|---|---|
| The primitive | an **explicit ladder** per spell: every rung spelled out | **three universal rules** ([A] cells, [B] a die a rung, [C] a size step) that a spell opts into with a short line: shorter data, but the player must learn three rules and their interplay, and a bonus or a mixed rung needs a fourth; the ladder is the three rules' *output*, and the wiki prints it. The three shapes survive as the authoring templates (§6.4) |
| Breakpoints | **scores** (decision 2) | **modifiers**: one more concept (`floor((s − 10) / 2)`), and "Intelligence +3" is not on the character screen; also the to-hit's unit, so the two would mix in a row |
| Drivers | **stats only** for growth; caster level for learning | **both** (today): two tables per row; **caster level only**: the stats would do nothing for a caster, against the stat training design that just gave them paths |
| Tiers | **a level per spell, 0 to 70** (decision 9); its cost is the level, its learning is the level against the caster level | **tiers** (today): a stepped level with a ten-column table between the spell and the player; **a level and a separate cost** (two numbers a spell): Identify keeps `mana` as the one override, every other spell costs its level |
| Mana at a level | **the level, at least 1** | **a curve under the level** (today's table runs 7 % under the comfortable level by tier 9): one number is the rule; the shipped levels are placed by today's costs so nothing moves far (§5.8) |
| The casting gate | **none**: a known spell is cast at its mana (decisions 1, 10) | **the overreach surcharge** (today): a casting gate by a derived level; the pool is the gate, and the study is the difficulty |
| Talents and items | **six named hooks**, each one integer or list, each its own sentence (decision 5) | **rungs as the only unit** (round 1): rejected by the owner; **a free-form effect per talent**: special cases in the cast path with nothing shared; six hooks cover every effect the owner named and every talent that exists |
| The ceiling | kept as a **load-time author's check**, a line of the level, out of the wiki | **deleted**: cheap insurance against a careless rung in a content PR; **kept in the wiki**: a per-level number nobody applies |
| Criticals | **rolled twice, unclamped**, as a blow's (decision 4) | **held to the top rung** (today): a clamp the player must know |
| Reach for scrolls and wands | **the reader's** (as today) | **the base**: "you aim it, your will carries it" is intuitive and costs nothing; damage at the item's rung because the magic is the item's |
| A wand's strength | **rolled when made**, 0 to its range's top, kept on the stack (decision 13) | **fixed per kind** (round 1): the owner wants wands to vary; **rolled at each zap**: a wand that is strong one turn and weak the next is noise, and could not be shown |
| Scroll adept | **scrolls only**: a rung on the spell scrolls read (decision 12) | **a rung or a die on every spell** (today, round 1): "why would we introduce a talent that is mandatory flat damage increase on all spells and call it scroll adept" |
| The rods' gate | **the rod's own level** (20) | **the caster level** (today): no caster-level gate (decision 10); **none**: a rod of terror that frightens anything is a rod a level-1 character finds too useful |
| The three older scrolls | **spell scrolls at a rung** | **kept apart**: the one special case in reading, with its own talent rule |
| The hexes' Willpower quarter | **dropped** (decision 8) | a second stat at a fraction: §3.2 item 6 |
| Cantrips | **one rung** each (Ray of frost 1d8, Spark 1d6; decision 6) | **flat** (today): a wizard's first spell should answer the first stat point |
| A spell's stat | **one**, by school with named exceptions (decision 3) | **two weighted** (today): §3.2 item 6 |
| The menu | **the dice** in the row, the reach in the description (decision 11) | **a "your spell grows" line** (round 1): rejected; **the to-hit in the row too**: no room in 31 columns, and the aiming mode weighs a shot |
| A talent on the rung unit | **none new** | "Mastery: a chosen spell a rung higher": a `choice = "spell"` kind does not exist; deferred (#196) |

## 12. Implementation plan

Eight issues in four waves; each links to its section. "**State**" marks a PR that adds or changes saved state (S5 alone). "**Fingerprint**" marks one that changes the content fingerprint. Hot files: `spells.toml`, `spell_def.rs`, `cast.rs`, `learn.rs`, `talent.rs`'s `TalentEffect`, `affix.rs`'s `Effect`, `wiki/spells.rs`, `docs/dev/spells.md`, spec §8 "Spells"; the false-conflict trick (`docs/dev/merging.md`) applies. Sizes: S a day, M two or three, L a week.

| Wave | Issue | What | Size | State / fingerprint | Depends on |
|---|---|---|---|---|---|
| 1 | **S0 The `Roll` type** (§6.2) | `dice::Roll` (groups + bonus, text, `roll(rng, reroll)`, `min`, `max`, `mean_x2`, `grow`, `more`, `plus`, `From<Dice>`, serde); `combat::Attack::damage: Roll` and `reflect::Shot::dice: Roll` with every builder converting; no behaviour change (the hash test and every replay verify as before); `docs/dev/combat.md` one line | S | none / none | — |
| 1 | **S1 Spell levels** (§5) | `level` for `tier` on the 26 spell files (§5.8; `tier` refused naming `level`); `spells.toml` keeps `[shapes]`, `[resolution]`, gains `[ceiling]`, loses `[tiers]` and `overreach_cost`; `SpellRules` as §6.5; mana = `max(1, level)`, no surcharge; `study.toml` (§5.2) and `learn.rs` in levels, `known_levels`; Overreach → `study { overreach_percent }`; `wand.rs`: `charges` required (the seven files); `spells_by_level`; the class test (§5.6); the `spells` and `progress` dumps; the menu row "level 11, 26 mana" for "tier 3" (the dice come in S4); `wiki/spells.rs` (`tiers` and `wand_charges` out, Level column, sort), `learning.rs` (the samples); `magic.md` "Spell levels", "Mana", "Caster level", "Learning", `scrolls.md`, `classes.md`, `features.md` lines; the spec's **Tiers**, **Cost**, **Learning** bullets and `08i`'s class lists; `README.toml`, `study.toml`, `actions.toml`, the talents' README; `docs/dev/spells.md`; `the_shipped_tier_table_reads_as_the_design_has_it` replaced by the ceiling line's test; replays re-made (the costs) | M | none / **fingerprint** | — |
| 2 | **S2 Ladders and hooks** (§4, §6, §7.1 to §7.4 but the wand rung, §9.1, §9.2 but the rods, §8's generator and dumps) | `content/spell_ladder.rs`; `SpellData::{damage: Roll, grows, aim}`; the retired fields refused with a message naming the new one; `ceiling_for`; every `capped` gone; `world/spell_mods.rs` (`SpellMods`, `Actor::spell_mods`) and `world/spell_rung.rs`; the cast path on rungs, mods, rerolls, riders; `TalentEffect::SpellMod` (`TalentMods::spell`, the scroll and wand rungs among its fields); Empower, Archmage, Keen mind, School focus, Scroll adept, Wand mastery re-pointed, Sure hand, Brimming, Numbing cold added (`assets/data/talents/`, by group); `Effect::{SpellRungs, SpellBonus, SpellSides, SpellDice, SpellReroll, WandRungs}` with Potent's and of Wands' files and the Potent read clamp; `Wand::rungs` (the range; a zap at `min` until S5); `ScrollSpell::rungs` and the three scrolls folded; the 26 spell files' ladders; `README.toml`, `spells.toml` comments; `wiki/spells.rs` (`list`, `does`, `reach`, `grows_with`, `resolutions`, `wands`, `scrolls`' "Casts at"); the dump lines; `docs/dev/spells.md`, `scrolls.md`, `magic-items.md`, `progression.md`; the spec bullets (**Ladders**, **Hooks**, **The lift** out); the tests of §6.3 and §10; replays re-made | L | none / **fingerprint** | S0, S1 |
| 3 | **S3 Rods** (§7.5, §9.2) | `RodEffect::grows` on the same parser; `per_levels`, `max_range`, `up_to_caster_level` gone, `up_to_level` in; the four rod files; `rods.md`; `docs/dev/spells.md` "Rods"; `rod_tests` | S | none / fingerprint | S2 |
| 3 | **S4 The menu's numbers** (§8) | the abilities menu's Spells rows on `Actor::spell_roll` (`PerceptionView::own_spell_roll`: the text, the damage type or "healed"), the description's reach clause; JSON `spells[].roll`; screen snapshots `abilities-menu-spells` (changed), `abilities-menu-spells-grown` (new), `wand-description` (changed: the dice line, at the wand's `min` until S5); `docs/dev/screens.md` one line | S | none / none | S2 |
| 3 | **S5 Wand strength** (§7.4) | `Stack::wand_rung` drawn at `Stack::new` from the maker's stream (loot, shops, level piles, kits), saved and hashed; `World::wand_cast` at it; the description and aiming hint; the `inventory` and `actors` dumps' `(rung N)`; `wands.md` "Strength"; `docs/dev/saves.md`, `docs/dev/magic-items.md`; `zap_tests` (the same seed the same rung; a save without it reads 0); replays that make wands re-made | S | **state (saves change)** / fingerprint (the wand files' `rungs`) | S2 |
| 3 | **S6 The prose** (§8, §3.1) | `wiki/src/magic.md` ("The spells", "How a spell grows", "Shapes", "Wands", "Rods", "Spellcraft talents" out), `scrolls.md`, `magic-items.md`, `stats.md`; the spec's prose paragraphs; `docs/design/README.md`'s entry | S | none / none | S2 |
| 4 | **S7 The probe** (§9.4) | the balance probe's spell method on `Actor::spell_roll`, the rung column, the pool check by level; `docs/balance.md` "Spells" re-run and the §7.3 paragraph rewritten; `docs/dev/spells.md` "Balance" | M | none / none | S2, S3 |


**Why this order**: S0 and S1 first and independent: S0 is a type change across `combat` with no spell in it, S1 touches the spell files' `tier` line and the cost path and nothing the ladders touch, so both merge clean before the hot spell files move; the integrator may run them side by side. S2 is one PR because the old hooks (`spell_lift`, `SpellDice`, `max_range`) and the talents that feed them cannot be half-replaced: every spell file, the generator, and the talents' effects change in the same commit or the content fails to load and the wiki fails to build. S3, S4, S5, S6 beside each other: a rod file, a menu row, a stack field, a wiki page. S7 last.

**Talents and class abilities** (the feature template's section, per slice): S0 none (a type). S1 re-points Overreach from the casting surcharge to the study check and re-words Arcane training, Attuned, and *of the Adept* as learning (§7.2); no new `[[system]]` tag. S2 re-points Empower, Archmage, Keen mind, School focus, Scroll adept, Wand mastery, Potent, *of Wands* onto the hooks and adds Sure hand, Brimming, Numbing cold (system `spells`; Brimming `mana` too), so every hook ships with a user (§7.2, §7.3); no new `[[system]]` tag (the `spells` system's bar is unchanged). S3 Channeler unchanged, none new. S4, S6, S7 none. S5 Wand mastery and *of Wands* unchanged (they add to the rolled rung), none new.

## 13. Decisions (owner, 2026-10-29)

The owner's answers to round 1's fifteen questions, verbatim, with the reading taken where an answer needed one. The body above is revised to them; round 1's tiers, its "rungs are the one external unit", its "your spell grows" line, its Scroll adept rung on own spells, its fixed wand rungs, and its caster-level gate on rods are gone.

| # | Question (round 1) | Answer | Taken as |
|---|---|---|---|
| 1 | Retire caster level from spell growth: it decides only which tiers are comfortable (mana, overreach), the study check, and the class tables' grants? | "yes" | caster level grows nothing (§4.4); with decisions 9 and 10 it decides **only the study and the class tables' check**: the mana part went with the tiers and the overreach surcharge |
| 2 | Breakpoints on scores (Intelligence 16), not modifiers? | "mechanically the same thing. but sure, actual scores work" | scores (§4.3): the number on the character screen |
| 3 | One stat per ladder, by school, with Call lightning on Willpower as the druid's spell? | "yes, spell decides, not caster's class" | one stat, the spell's own, written in its file; the school table is a convention for authors (§6.4), Call lightning names Willpower |
| 4 | Criticals unclamped: an attack or touch spell's critical rolls the dice twice, as a blow's? | "yes. no cap" | rolled twice, nothing held: no cap anywhere at run time, on criticals or on what talents and items add (§4.3, §7.1); the ceiling is an author's check on the ladder only (§4.6) |
| 5 | Talents and items as rungs, never flat dice or cells? | "disagree. talents and items can do whatever their author wishes. Flat damage, different dice, a rider. This is not a concern of the magic progression per-se. Talents primarily tied to spell progression can work that way. Regardless, i want the system to mechanically be able to support riders from talents." | **six named hooks** (rung, dice, roll, rider, reach, degrees; §7.1) so any author-written effect fits without a special case; Empower, Archmage, Scroll adept, Wand mastery, *of Wands* stay rungs (talents about the ladder); Keen mind grows dice, Potent adds a bonus, School focus and *of Farcasting* and *of Lingering* keep their cells and degrees; three new talents (Sure hand, Brimming, Numbing cold) give the reroll, dice, and rider hooks a shipped user (§7.2, §7.3). The framing applied: a d7 is a die, a reroll is a rule |
| 6 | Cantrips grow one rung (Ray of frost 1d8, Spark 1d6)? | "fine" | kept (§9.1) |
| 7 | Blight, Cause wounds, and Mend get ladders (Willpower), where today they never grow? | "yes" | kept (§9.1) |
| 8 | Hexes last by Presence alone (today's quarter of Willpower dropped)? | "yes" | kept (§9.2) |
| 9 | The tier ceiling stays as an author's check on the top rung, re-valued, and leaves the player wiki? | "get rid of concept of spell tier. Assign spells a level between 0-70. (tier is just a cringe stepped spell level system anyhow). let's bypass that extra step. players are familiar with this form wow etc." | **tiers abolished** (§5): `level` 0 to 70 on every spell, its base mana and its certain-learning caster level in one number; every tier column re-expressed as a one-line rule of the level (§5.2); the 26 spells levelled (§5.8); the ceiling stays an author's check, now a line of the level (§4.6), out of the wiki; the fingerprint changes, saves do not (§5.9) |
| 10 | Rods reach by Willpower ladders (digging 3 to 7), and sleep's and terror's "no stronger than your magic" gate stays on caster level? | "sure. just get rid of the your casterlevel gate. caster level is exclusively a learning concept. items are good, because they do not require learning." | Willpower ladders kept; the caster-level gate **removed** from rods and from everything else that is not learning: the overreach mana surcharge goes (§4.4, §5.3), wands and scrolls read no caster level (§7.4), a rod's sleep and terror take creatures up to the rod's own `up_to_level` (20; §7.5) |
| 11 | One line when a known spell climbs a rung ("Your fireball burns hotter."), and the menu's description says what a spell grows with, in words? | "uhm, no. Just put the damage number to the menu as you would with a weapon." | **no line**; the abilities menu's Spells rows show the roll as the caster would make it now, the damage type, the mana, and "aimed" (`Fireball   4d6 fire, 26 mana, aimed`), the description the reach in cells (§8); the character screen unchanged; a wand's description shows its dice with its type (§7.4) |
| 12 | Scroll adept's rung on the wizard's own spells too, not only on scrolls read? | "no. why would we introduce a talent that is mandatory flat damage increase on all spells and call it scroll adept." | Scroll adept is **scrolls only**: the spell scrolls read cast a rung higher, reading in half the time (§7.2); a spell scroll casts at its kind's `rungs` (the three old ones at 1 / 2 / 0), fixed, with no talent needed to make it worth reading (§7.4) |
| 13 | Wands cast at rung 0 by default plus their `rungs`, Wand mastery's 1, and of Wands' 1 to 2; the three old scrolls become spell scrolls at rungs 1 / 2 / 0? | "wands should randomize their effective rung at loot generation." | a wand's rung is **drawn when the stack is made**, evenly in its kind's `rungs` range (0 to 2 on the seven), kept on the stack (`Stack::wand_rung`, **saves change** in S5), shown with the type as the spell's dice; Wand mastery and *of Wands* add to it (§7.4). The scrolls' part of the question stands (decision 12) |
| 14 | The rung grid as a convention (dice at 13 / 16 / 19 / 22 / 25; cells at 14 / 18 / 22 / 26; degrees at 14 / 18), stated in `README.toml`, not checked? | "sure" | kept (§6.4) |
| 15 | Mixed dice used once in the shipped list (Scorching ray's top rung `3d6+1d10`), else plain? | "whatever" | kept as written (§9.1); an author may use mixed dice where a spell's fiction wants them |

Also folded in: the owner's framing for round 2 (the preamble), applied to the hooks (a reroll, a die size), the dice (`MAX_SIDES` 20, a d7 and a d9 legal), and the authoring templates (§6.4 [C]); and the numbers of §9.3 were re-run with §5.8's levels and now carry the mana before and after.
