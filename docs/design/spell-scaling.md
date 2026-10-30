# Warlock — Spell scaling: a ladder in every spell's own words, spell levels for tiers, and school skills

Design document for the owner's review, **round 3** (2026-10-30). Written against `origin/main` at 38ab0bbd (the stat training slices T2 to T6, #732 to #735). Nothing here is implemented; §15 cuts it into issues. Round 1 audited the player wiki's magic pages (`wiki/src/magic.md`, `scrolls.md`, `mana.md`, `magic-items.md`, the generated `spell-list.md`, `spell-tiers.md`, `wands.md`, `rods.md`, `class-levels.md`) against the rules behind them (`docs/dev/spells.md`, spec §8 "Spells", `assets/data/spells.toml`, `assets/data/spells/*.toml`, `content/spell_def.rs`, `content/spell_scales.rs`, `world/spell_scale.rs`, `world/cast.rs`, `world/caster_level.rs`, the talents and affixes that touch spells) and rebuilt how a spell grows. Round 2 folded in the owner's answers to its fifteen questions (§16 "Decisions"): **tiers are gone, every spell has a level from 0 to 70; caster level is a learning concept and nothing else; talents and items may do anything to a spell through named hooks; the menu shows a spell's dice as it shows a weapon's; wands roll their strength when found.** Round 3 folds in the owner's two comments on round 2 (§16, decisions 16 and 17): **nothing anywhere says how strong a spell may be written: no ceiling, no cap, no author's check; a spell is its ladder** (§4, §6); and **casting trains: every school of magic is a skill of ten ranks, worked exactly like a weapon skill, each rank a leg up on casting that school's spells** (§8). The Spell attacks skill is merged into them and the caster styles become choices of where one casts from (§10); the Spells section leaves the weapon skills page for a scrollable Magic page (§11); per-spell mastery ranks were weighed and not taken (§9). Every number is data; the wiki states it (the owner's rule: the game is vague, the wiki is accurate; a spell's dice in the menu are the owner's explicit exception, as a weapon's are).

**The owner's request** (verbatim): *"auditor reads the magic documentation from wiki, and makes sure the documentation is as clear as possible. It also identifies mechanics that are too complicated to be put in simple words. Then it makes a change proposal, that aims to streamline the system. Just fucking look at it: `cap: Intelligence 10 %, Willpower 5 % of the cap a point; reach: Willpower 50 % of a cell a point`. Damage scaling must be entirely per-spell in its spell text. It must be clearly written. Tools we have at our disposal: this is not played with real dice. 2d6 spell can scale to 2d7 -- dice can change; we can add extra dice; we can add static bonus; not all damage die must be same. Some ideas, let's make few: one stat increases range, duration, radius etc; one adds damage per dice based breakpoints; one increases the size of dice, based on breakpoints. If you choose to go with these universal rules, you just need to write 3 or more rules, and spell becomes very simple unless we choose to add riders."* Earlier: the magic mechanics feel *"too complicated to be fun ... not complicated, but fucking janky"*.

**Addendum** (the same day, verbatim): *"i am not against getting rid of concept of caster level as spell scaling. perhaps that is just learning difficulty thing?"* Taken (§4.4, decision 1).

**The framing for round 2** (the owner, verbatim): *"this is a computer game, not played with real dice. There are things such as d7, d9 or whatever we need. We can also do things that would be tedious on tabletop, like rerolling 1's or whatever. Keep these things in mind when designing talents. Rules must be intuitive, executing the rules is not a concern."* It is applied throughout: the dice are any size, a talent may reroll ones or grow every die a size (§7), and every rule below is one a player can say in a sentence, however much the code does to carry it out.

**The owner's comments on round 2** (2026-10-30, verbatim; decisions 16 and 17 in §16). On round 2's §4.6: *"this one discusses a rule of 10d6 fireballs. We should purge that rule from all memories. I don't want it ever appearing anywhere."* On round 2's §4.5: *"discusses how we need 6 casts to dispatch a median monster. This is a little too much in my opinion. Let's introduce 'weapon skills' for spell schools (work like weapon skill, gain marks by casting from that school, 10 ranks) that give a leg up when casting from that school. Another thing we could do, is that each spell has its own hand authored 10-mark progression (similar to weapon skill). These new skills (along with 'Spells' section currently on weapon skill page) should probably be moved to a separate 'Magic' page (scrollable) since they will not neatly fit the weapon skills page. 'Spell attacks' skill also comes under the scope of this redesign. Also the spellcasting related fighting styles. Take a look at the existing fighting styles and spell attacks weapon skill, and deliberate how they mesh with what we are designing (i don't think it's that good match yet). another design round please."*

## Contents

1. Summary
2. What exists today
3. The audit: the wiki read as a player
4. The proposal: ladders
5. Spell levels, 0 to 70: what tiers did, and what does it now
6. The data format and its checks
7. Talents, items, scrolls, wands, rods, traps: the hooks
8. School skills: casting trains
9. Spell mastery, weighed: ranks per spell
10. Spell attacks and the caster styles: the audit and the rework
11. What the player sees; the Magic page; the wiki, the spec, the docs
12. Migration: every spell, with before and after
13. Determinism, saves, replays, fingerprints
14. Choices made along the way, and what is rejected
15. Implementation plan
16. Decisions (owner, 2026-10-29 and 2026-10-30)

## 1. Summary

- **One idea replaces six.** A spell's growth is a **ladder** written in its own file and printed verbatim in its wiki row: `damage = "intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2"` reads *"Fireball: 2d6 fire; with Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2."* Your score reaches the number, you roll that. The last rung is the most your stats take the spell. Nothing else is computed: no percent of a cap per modifier point, no `max_lift`, no half-rounding, no dice "room" formula (§4.2, §4.3). **Nothing bounds a ladder but its author** (decision 16): no ceiling, no load-time check on how much a spell may do, no column in the data that says so. A spell is as strong as its row says.
- **Any dice shape on any rung** (the owner's tools): more dice (`3d6` → `4d6`), bigger dice (`1d6` → `1d8`), a static bonus (`5d6+2`), mixed dice (`3d6+1d10`). A new `Roll` type carries them; `Dice` stays for everything else (§6.2).
- **Breakpoints are stat scores**, not modifiers: "Intelligence 16" is the number on the character screen (decision 2). One stat per ladder, the spell's own choice (decision 3). Reach (`range`, `length`, `radius`) and a buff's or hex's `degrees` are ladders of the same kind, in whole cells and whole degrees (§4.3).
- **Spell levels replace tiers** (decision 9). Every spell has a **level, 0 to 70**, one number: the caster level at which it is learned for certain, and its base mana (`Fireball: level 11, 26 mana as a burst`). No tier table, no comfortable level, no overreach surcharge; the wand charges, the study time and check, the loot conventions, and the class tables' check are each one short rule of the level (§5). The 26 spells get levels 0 to 12 (§5.8); every mana cost stays within three points of today's and eighteen of the twenty-five are unchanged.
- **Caster level is a learning concept, exclusively** (decisions 1 and 10). It decides which spells a scroll teaches for certain and how hard the others are; it never changes what a spell does, what it costs, or whether it can be cast. A spell known is a spell cast, at its level's mana: the pool is the gate. Rods and wands have no caster-level gate: a rod's sleep takes creatures up to the rod's own level (§7.5).
- **Talents and items may do anything to a spell** (decision 5): the cast path has **six named hooks** (rung, dice, roll, bonus, rider, reach and degrees), each a `SpellMods` field summed from talents and worn items, each filtered by the spell's tags (§7.1). Empower and Archmage are rungs; Keen mind grows its school's dice a size (d6 → d7); Potent adds to the roll; three new Spellcraft talents show the rest: Sure hand (a die showing 1 is rolled again), Brimming (a die more), Numbing cold (a Sluggish rider on cold spells) (§7.2). Rungs are not the only unit; they are the unit for the talents that are about the ladder itself.
- **Casting trains** (decision 17): the seven schools of magic (evocation, necromancy, abjuration, restoration, transmutation, enchantment, divination) are **school skills**, rows of `weapon_skills.toml` with `role = "school"`: the same ten ranks, the same marks (an attempt for every spell paid in mana, a success when it lands, weighed by the strongest creature it touched), the same curve as the missile skills, the same screen row, the same "You feel more skilled at evocation." Each rank's benefit is written through the hooks (§7.1): a die showing 1 rolled again, every die a size bigger, a die more, a bonus, a cell of reach, a degree of duration, mana off, a quicker cast, and the to-hit and threat of the school's spells that roll to hit (§8.3, seven hand-written tables). A rank-10 evoker's Fireball rolls 7d8 and rerolls its 1s and 2s where a novice's rolls 5d6: about twice the harm (§12.3). School abilities (a status laid by the school's spells) hang on the ranks as weapon abilities do and are listed in the one abilities menu (§8.5). Three talents and a class ability carry the new system `schools` (§8.6).
- **No ranks per spell** (§9): a ten-step table per spell was weighed (26 tables now, a few hundred steps by the time the list is full, and a fourth progression stacked on the one number the menu shows) and not taken; the school's rank is the leg up, the ladder the spell's own growth, and a per-spell table remains an additive slice if the owner wants it later.
- **Spell attacks is retired and the caster styles are reworked** (§10): the skill trained on three of twenty-six spells; its to-hit and threat rows move into every school's table, where a weapon skill's to-hit sits. Sniper, Toucher, and Battle caster become **choices of where you cast from**: a spell at a creature beside you is a Toucher's, one aimed farther a Sniper's, any spell in armour that hinders casting a Battle caster's, every one of them earning marks on every such spell, not only on the three that roll to hit.
- **A Magic page** (§11): page 6 of the info pages, scrollable, with the schools and their bars, the caster styles, and the spells known with their rolls and mana; the Spells section and the caster styles leave the Weapons page.
- **Scroll adept is a scroll talent** (decision 12): the spell scrolls you read cast a rung up, and you read in half the time; nothing on your own spells. A spell scroll casts at the rung its kind says (`rungs = 2` for the scroll of lightning's 4d6); every scroll is one kind of thing (§7.3).
- **A wand's strength is rolled when it is found** (decision 13): its rung, 0 to its ladder's length, is on the stack, saved, and shown with its type (§7.4; **saves change** in that slice).
- **The menu shows the dice** (decision 11): `Fireball   4d6 fire, 26 mana, aimed`, as the character screen shows `+4 to hit, 1d8+2 slashing`; no "your spell grows" line (§11).
- **Reading one spell row today takes 17 concepts** for the damage cell and 6 for the reach cell; the proposal takes 3 and 2, and one more, the school's rank, for a caster who has one (§3.3).
- **Balance** (§12.3): at school rank 0 the trained wizard's and priest's spells land within a die of today's at levels 10, 30, and 60 (Fireball 4d6 / 5d6 / 5d6 against today's 3d6 / 5d6 / 5d6); the cantrips, Blight, Cause wounds, and Mend gain one or two rungs where today they never grow (decisions 6 and 7). What then keeps a spell growing is the school's rank (Fireball 17.5 on average at rank 0, 22 at 5, 28 at 8, 37 at 10) and the higher-level spells the content brings (§4.5).
- **Twelve slices** (§15): `Roll` and the levels first (both small, both can merge first), the ladders and hooks as one PR, then rods, the menu's numbers, the wands' strength (saves change), and the prose beside each other; the school skills after the hooks, then the Magic page, the caster styles, and the school abilities beside each other; the probe last.

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
| 6 | `magic.md` "The lifted cap" | the whole section: "each by a percent of the cap per point of its modifier ... never more than the tier's most lift (40 % at tier 0 ...) ... rolls as many more dice as the lift makes room for ..." | a six-step computation with two roundings, explained with a worked example because it cannot be said plainly | delete; replaced by "How a spell grows" (§11): *"Each spell's row lists its ladder: a stat and the scores at which the spell changes. Your score reaches the number, you cast that rung."* |
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
| Damage | 1 tier · 2 comfortable caster level · 3 caster level (3a level × casting %, 3b Arcane training, 3c *of the Adept*, 3d Attuned) · 4 `per_levels` · 5 `max_dice` · 6 the tier's ceiling · 7 the resolution's percent · 8 the cap as a clamp · 9 the stat modifier · 10 percent of the cap per point · 11 two stats summed · 12 `max_lift` by tier · 13 Keen mind's +10 · 14 Potent's +N · 15 the lifted cap rounded down · 16 the dice "room" formula · 17 talent dice then the clamp — **17** (20 with 3a–d) | 1 the roll · 2 the stat's score · 3 the rung reached (and 4 the school's rank, one row of the school's table, once the caster has a rank; and what a talent or item adds, when the player has one, each in its own row) — **3**, 4 for a trained caster |
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
| G9 | **Balance within a die of today** at levels 10, 30, 60 for the trained caster at school rank 0; mana within three points; shifts named. | the migration must not re-tune the game by accident |
| G10 | **Casting trains.** A school's rank is a leg up on every spell of the school, worked like a weapon skill: marks by use, ten ranks, one sentence a rank, hidden numbers, a bar on a page. | decision 17: "work like weapon skill, gain marks by casting from that school, 10 ranks" |
| G11 | **No ceiling, no cap, no author's check.** Nothing in the data, the loader, or the docs says how much a spell may do; the ladder's author decides, the PR's reviewer reads the row. | decision 16 |

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

**Mana as the gate.** With no surcharge, a low-level caster who learned a high spell by a lucky study casts it at its level's mana: a level-5 wizard (47 mana) casts a level-11 Fireball (26 mana) once, then rests. The pool grows a level at a time (`mana_level = 60` in `experience.toml`, unchanged; its comment loses "tier 9's comfortable level") and is about ten casts of a spell of the caster's own level at every level (the probe's measure: 106 mana at level 12 against Fireball's 11 base, 254 at 30 against 29, 502 at 60 against 56; §12.4), so the difficulty of a spell beyond you is felt as a short day, not a refusal.

### 4.5 What keeps spells growing at level 30 to 60

Without dice by level, four things; the first two are this document's, the others already in the game or already planned:

1. **The school's rank** (§8; decision 17). Every spell paid in mana earns marks in its school, and each rank is a leg up on every spell of the school: a die showing 1 rolled again from rank 3, every die a size bigger at 5, a die more at 6, 1s and 2s rerolled at 8, a second die more at 9, a second size at 10 (evocation's table, §8.3). The trained wizard's Fireball rolls 4d6 at level 10 by its ladder; at the ranks the pace of §8.4 puts a wizard at, it averages 17.7 at level 10 (rank 5), 28.3 at 30 (rank 8), and 36.8 at 60 (rank 10), against 14, 17.5, and 17.5 at rank 0 (§12.3). A rank is **earned by casting**, like a weapon rank by fighting, so the caster who casts most grows most, in the school they cast.
2. **Higher levels.** The combat progression design (§7.3) showed that a level-60 caster needs spells above today's: "content the magic design planned and no scaling replaces". A level-22 burst written at base 5d6 with a ladder to 8d6+4 averages 32 at its top rung and 56 at evocation 10; a level-37 one at 10d8 averages 45 and 76. The content is the progression, as the design meant, and the school rank multiplies whatever the content brings.
3. **Rungs that run on.** The trained wizard's Intelligence reaches 20 at level 30 and 23 at 60 (`docs/balance.md`; the stat training design puts a main stat at +12 to +17 by 60); ladders are written to 25 and may go higher for the high levels, so a caster keeps climbing after the dice-by-level curve would have flattened (today `max_dice` is reached by caster level 24 on every tier-3 spell).
4. **The hooks from the class table and the items.** Empower (wizard 32) a rung, Archmage (60) two, Keen mind a die size, Potent items a bonus: level-gated adds, each worth exactly what its row says on every ladder the player can read, and each stacking with the school's rank through the same hooks.

§12.3 shows the trained wizard's and priest's averages at 10, 20, 30, and 60 today and proposed at rank 0, and then with the school ranks the pace predicts.

### 4.6 Criticals, resolutions, the area's edge

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
| `ceiling` (the author's check) | **gone** (decision 16): nothing says how much a spell of a level may do; its ladder does | removed |
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
| `SpellData::tier`, `SpellRules::{comfortable, cost, ceiling, max_lift, wand_charges, overreach_cost, tier_of}` | the table | `SpellData::level`; `SpellRules::{shapes, resolution}` (the resolutions' `cost` only: the `damage` percent goes with the cap) |
| `Content::spells_by_tier` (the menu's and `Ability::every`'s order) | tier, then path | `spells_by_level`: level, then path |
| `learn.rs` (`study_odds`, `study_turns`, `Odds::overreach`, `Actor::known_tiers`) | tiers | levels (§5.4); `known_levels` |
| `cast.rs` cost (`SpellRules::mana`, `mana_with`, `TalentMods::overreach_cost_percent`) | tier price and surcharge | `max(1, level)`; the surcharge and the talent mod gone (`study_overreach_percent` comes, §7.2) |
| `content/wand.rs` (`charges` default) | the tier's | required |
| `balance_probe.rs` (`comfortable`, the lift) | tiers | §12.4 |
| `dump.rs`: `spells` layer `Fireball (tier 3, 26 mana)`, `progress`'s `study c frost: overreach 2 ...` | tiers | `Fireball (level 11, 26 mana): 4d6 fire, range 11, radius 2` (§11); `overreach 5` in levels |
| `ui/abilities_screen.rs` "tier 3, 26 mana, aimed" | tiers | §11: the dice for the tier |
| the content test `spells within their tier` (`spell_list_tests`, `caster_tests`) | tiers | §5.6 |
| `xtask/src/wiki/spells.rs` (`tiers`, `list`, `does`, `wands`, `class_spells`, `wand_charges`), `learning.rs` (`times`, `chances`) | tiers | `tiers` and `wand_charges` deleted; `list` gets a Level column and sorts by it; `times` and `chances` sample spell levels 0, 3, 6, 11, 16, 22, 29, 37, 46, 56 (one row each, the levels the spells and their successors sit) |
| `wiki/src/magic.md` ("Tiers", "Mana", "Caster level", "Learning"), `scrolls.md`, `classes.md`, `features.md` (the lines naming tiers) | tiers | "Spell levels" (§11); `features.md`: "spells of levels 0 to 70" |
| spec §8 "Spells" (`spec/08e-blood-and-status.md`: **Tiers**, **Cost**, **Learning**, **Scrolls of spells**, **Wands**; `08i-character.md`'s class lists "(tier 0, at 1 ...)"; `08g-magic-items.md`'s wand bullet) | tiers | levels, the bullets rewritten (§11) |
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
| `damage` | a `damage` spell | `damage` | a `Roll` | each rung's `max` ≥ the one before, `min` ≥ the one before, not identical; nothing else: how much a rung does is its author's |
| `heal` | a `heal` spell | `effect.heal` | a `Roll` | as `damage` |
| `range` | ray, bolt, burst | `range` | cells | strictly ascending from the base, 1 to `MAX_REACH` (20) |
| `length` | cone, line | `length` | cells | strictly ascending from the base, a cone to `MAX_CONE` (5), a line to 20 |
| `radius` | burst | `radius` | cells | strictly ascending from the base, to the burst table's length (3) |
| `degrees` | buff, hex | `effect.severity` | degrees | strictly ascending from the base, to the status's `max` and `MAX_SEVERITY` |

`aim = "dexterity"` (a stat id; default Dexterity) on an `attack` or `touch` damage spell replaces `[spell.scales] to_hit`.

Retired fields: `tier` (now `level`), `[spell.damage] dice, per_levels, max_dice, bonus` (now `damage = "2d6"`), `max_range`, `max_length`, `max_radius` (the top rung is as far as stats go; a spell without a ladder reaches its base), `[spell.scales]` (all four), `[spell.scroll] level`, `[tiers]` entire. `README.toml` and `spells.toml`'s comments are rewritten with the grammar above and §6.4's conventions.

### 6.2 `Roll`: dice of more than one size

`Dice` is one group (`count`, `sides`, `bonus`) and stays what it is for stats, weapons, and everything else. A **`Roll`** (`dice.rs`, beside it) is a sum of `Dice` groups and one bonus: text `3d6+1d10`, `5d6+2`, `2d8`, `+3`; groups joined by `+`, at most `MAX_GROUPS` (4), the bonus last and once; sides 2 to `MAX_SIDES` (20; a d7 and a d9 are dice like any other); `min`, `max`, `mean_x2` (twice the mean, an integer), `roll(rng, reroll)` (each group in order, one draw a die; a die at or below `reroll` is drawn once more, §7.1), `Display` as written; `From<Dice>`; `grow(sides)` (every group's sides + n), `more(dice)` (n dice more of the first group's kind), `plus(bonus)`. `Attack::damage` in `combat` becomes a `Roll` (a `Dice` converts), so `combat::strike` rolls a spell's mixed dice as it rolls a blow's; `reflect::Shot::dice` the same. Nothing else changes type: a weapon's `Dice` becomes a one-group `Roll` at the one place a `Profile` is built. S0 (§15) lands this alone, with no behaviour change, so the ladders' PR is about spells only.

### 6.3 Validation (`spell_def::resolve`, a `ContentError` naming the spell)

1. `level` 0 to `MAX_SPELL_LEVEL`. A ladder only for a dimension the spell has: `damage` for a damage roll, `heal` for a heal, `range`/`length`/`radius` for the shape that has them, `degrees` for a buff or hex. `aim` only for an attack or touch damage spell.
2. The stat is one of `Stat::ALL`; at least one rung; at most `MAX_RUNGS` (8); scores 1 to 40, strictly ascending.
3. Values as the table in §6.1: `Roll`s that can do harm (`min` ≥ 0, `max` ≥ 1) and never weaken (`max` and `min` non-decreasing, each rung different from the one before); cells and degrees strictly ascending from the base and within the shape's or status's bounds.
4. A wand's `rungs` range (§7.4) within the spell's damage ladder length; a spell scroll's `rungs` the same.
5. Tests: the shipped data passes; a rung below its base, a descending ladder, a stat the spell has no dimension for, an unknown stat, a radius past the table, a degree past the status's max, nine rungs, a wand with too many rungs, a level of 71, a `tier` key (refused with "use `level`") are each refused with the words of the error. **No check reads how much a rung does** (decision 16): there is no test that a roll is "too big", and none is to be added.

### 6.4 Authoring conventions (templates, not rules)

The owner's three universal shapes are the **recommended ways to write a ladder**, stated once in `README.toml`, so most spells read alike without the player having to learn them as rules (the row is still the truth):

| Shape | Writes as | Typical use |
|---|---|---|
| **[A] cells or degrees, one more a rung**, Willpower for reach at scores 14, 18, 22, 26; Presence for degrees at 14, 18 | `range = "willpower 14 11, 18 12"`, `degrees = "presence 14 2, 18 3"` | every aimed spell, every buff and hex |
| **[B] a die more a rung**, the spell's stat at 13, 16, 19, 22, 25 | `damage = "intelligence 13 3d6, 16 4d6, 19 5d6"` | the body of a damage ladder |
| **[C] a bigger die a rung** | `damage = "willpower 19 2d8"`, `"intelligence 16 1d8"`, `"willpower 22 2d7"` | cantrips; a d7 or d9 where a die more is too much and a bonus too flat |
| **[D] a static bonus at the top** | `25 5d6+2` | the last rung, where the author wants a small step rather than a die |
| **mixed dice** | `25 3d6+1d10` | sparingly, where a spell's fiction wants it (a hotter core) |

The stat by school, unless the spell's text says otherwise (decision 3: the spell decides, not the caster's class): evocation, transmutation, divination **Intelligence**; necromancy, abjuration, restoration **Willpower**; enchantment **Presence**; reach **Willpower** always; degrees **Presence** always. Call lightning names Willpower (the druid's spell). Rung scores are on the same grid for every spell (13 / 16 / 19 / 22 / 25 for dice; 14 / 18 / 22 / 26 for cells; 14 / 18 for degrees; decision 14, a convention, not a check) so a player who learns one row knows where to look in the next. The level by power (§5.8): a new spell takes the level of its mana; how much its top rung does is its author's call, read in the PR as every row is (decision 16: no line, no check).

### 6.5 Code sketch: what goes, what comes

| Goes | Comes |
|---|---|
| `content/spell_scales.rs` (`Weights`, `SpellScales`, `ScalesSpec`, `Has`), `MAX_SCALE_WEIGHT` | `content/spell_ladder.rs`: `Ladder { stat, rungs: Vec<(u8, RungValue)> }`, `RungValue::{Roll(Roll), Cells(u8), Degrees(u16)}`, the parser, `Ladder::rung_at(score) -> usize`, `Ladder::value(rung) -> &RungValue`, `MAX_RUNGS` |
| `SpellDice` (`count`, `sides`, `per_levels`, `max_dice`, `bonus`, `at`, `lifted`) | `SpellData::damage: Option<Roll>`, `effect.heal: Option<Roll>`, `SpellData::grows: Grows { damage, heal, range, length, radius, degrees: Option<Ladder> }`, `SpellData::aim: Stat`, `SpellData::level: u8` |
| `SpellRules::{comfortable, cost, ceiling, max_lift, wand_charges, overreach_cost, tier_of, lift, lifted_cap, grown, lengthened, max_bonus, mana_with}`, `Resolution::damage` | `SpellRules::mana(spell, armour)` (no caster level); nothing for a ceiling |
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
| 2 | **dice** | `sides: u8` (every die a size bigger: d6 → d7), `dice: u8` (that many dice more, of the roll's first group's kind), `bonus: i16` (added to the roll's bonus) | "bigger dice", "a die more", "+2 on every spell" | to the ladder's `Roll` after the lookup, in that order: `roll.grow(sides).more(dice).plus(bonus)`; the result is **what the menu shows** (§11) |
| 3 | **roll** | `reroll: u8` (a die showing this or less is rolled once more) | "a die that shows a 1 is rolled again" | at the draw, per die, once; a critical repeats the whole roll stage, as a blow's does |
| 4 | **rider** | `riders: Vec<(StatusId, severity)>` | "what your cold spells harm is slowed a while" | after the harm, on every target the spell harmed, as the spell's own `effect.status` is laid (merged: the same status's severities add); nothing on a miss or on a heal |
| 5 | **reach** | `cells: i8` (added to range or length after the reach ladder), `radius_cells` (a burst's radius) | "your spells reach a cell farther" | after the ladder; held by `MAX_REACH` and the burst table only |
| 6 | **degrees** | `degrees: i8` | "your buffs and hexes last a degree longer" | after the ladder; held by the status's `max` only (`Effects::add`, as today) |

Cost (`spell_mana_percent`, School focus's cost, Frugal caster, *of Thrift*) and to-hit (*of the Magus*, the Spell attacks skill, the styles) and threat (*Baleful*) are hooks already and stay where they are; they are named here so an author sees the whole list in one place. **Order**: rung → dice → roll (with rerolls; twice on a critical) → the harm → riders; reach and degrees beside. **No clamp** after any of them (decisions 4 and 16): nothing bounds a roll but the dice. **Three sources** write to the one struct: the caster's talents, the items worn, and, from §8, **the school's rank and the caster style's** (`Benefit::spell_mods()`), so a rank's leg up is applied by the same five lines as a talent's. A **scroll or wand** gets the reader's `SpellMods` with `rungs` replaced by the item's own plus the reader's `scroll_rungs` or `wand_rungs` (§7.3, §7.4); the other five hooks apply as to an own cast (your Sure hand rerolls a scroll's ones too: the dice are yours to throw). The **mishap backfire** gets none.

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

The three new ones exist so that every hook ships with a user (no speculative code: `reroll`, `dice`, and `rider` would otherwise have none) and show an author the pattern; the owner strikes any that does not belong, and the hook stays tested by a test talent. Systems: all `spells` (Brimming `mana` too); no new `[[system]]` tag in this slice, the coverage bar unchanged (`docs/dev/progression.md` "Coverage policy"); the school skills' system and its talents are §8.6. `SpellDamagePercent` (the wards against bursts and bolts) is the target's and is untouched.

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

- **Rods**: `range`, `max_range`, `per_levels` on a rod effect become `range` and `grows.range = "willpower 14 ..., 18 ..."` (the same grammar, `RodEffect::grows`), so the wiki's rod row reads as a spell's: digging `3; Willpower 14: 4, 18: 5, 22: 6, 26: 7` (today 3 + 1 per 3 caster levels to 12: a level-30 wizard reaches 12 today, 6 proposed; §12.2 says why). Sleep's and terror's `up_to_caster_level` becomes **`up_to_level = 20`** on the effect (decision 10: no caster-level gate; "a rod the same in anyone's hands", the wiki's own words; today a level-30 wizard's rod took creatures to level 36, a level-10 one's to 12): a creature above it "shrugs it off", as today's wording. Channeler unchanged.
- **Traps** (`data/traps.toml` bursts and bolts): untouched; `content/spell.rs` keeps `Burst` and `Bolt` for them (the `cap` fields removed).
- **Mishap backfire**: the taught spell's base, no mods, as today's "held to the taught spell's cap" becomes "its base".
- **Spell reflection**: the `Shot` carries the caster's `Roll` and `reroll`; the reflector casts it back as is; riders are the caster's and come back with it.
- **Creatures**: creature spells are intents, not the cast path (`docs/dev/spells.md`); their caster level (#525) still feeds their summons' comfort. Nothing changes.

## 8. School skills: casting trains

Decision 17, verbatim: *"Let's introduce 'weapon skills' for spell schools (work like weapon skill, gain marks by casting from that school, 10 ranks) that give a leg up when casting from that school."* Taken literally: a school skill **is** a weapon skill in everything but what its ranks give, and what they give is written through the hooks of §7.1.

### 8.1 The rule, as the wiki will state it

> Every spell belongs to one of seven **schools**: evocation, necromancy, abjuration, restoration, transmutation, enchantment, divination. Each school is a **skill of ten ranks**, trained as a weapon skill is: every spell you cast with your own mana earns marks in its school, more when it lands, more against a strong foe, fewer as your rank rises. Each rank is a leg up on every spell of the school, said in one sentence in the school's table: a die that shows 1 is rolled again; every die a size bigger; a die more; a point on every roll; a cell more of reach; a degree longer; a little less mana; a quicker cast; and for the school's spells that roll to hit, a better aim and a wider critical range. A scroll read or a wand used trains nothing: the magic is the item's. Your rank is on the Magic page with a bar of your marks toward the next; what a rank gives, in numbers, is here and not in the game.

### 8.2 The data: `role = "school"` in `weapon_skills.toml`

Seven rows after the shield skills (where Spell attacks sat, §10) and before the fighting styles, each:

```toml
[[skill]]
id = "evocation"
name = "Evocation"
phrase = "at evocation"
role = "school"
school = "evocation"                # the spells' `school` tag it trains on and lifts
curve = "missile"
to_hit = [0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 6]   # its spells that roll to hit (§10.1)
threat = [0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3]
reroll = [0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2]   # a die showing this or less is rolled again, once
sides  = [0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2]   # every die this many sizes bigger (a d6 a d7)
dice   = [0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 2]   # this many dice more
mana   = [0, 0, 0, 0, 5, 5, 5, 10, 10, 10, 15]  # percent off the spell's mana
```

- **`Role::School`** joins the roles of `content/weapon_skill.rs`; `school` names a school of `spell_def.rs`'s list (the loader refuses an unknown one and a school with two rows; the seven are required once there are spells: a spell whose school has no row is refused, so a school cannot be forgotten). A school row is a weapon skill in every other way: `curve`, the `[marks]` rules, the defaults, `Training { rank, marks }` on `Actor::weapon_skills` (saved by index as every skill is), `train`, `raise`, `progress`, `ranks_news`, the dev `rank:evocation:5`.
- **`Benefit` gains the spell fields**, each an array of 11 and allowed for `school` rows only (checked per role as the grips' fields are): `reroll` (0 to 3), `sides` (0 to 3), `dice` (0 to 3), `bonus` (0 to 6), `cells` (0 to 3), `degrees` (0 to 3), `mana` (0 to 50, percent off), and `speed`, which exists and is **allowed for schools** (percent off the cast's time; refused for `spell` and the caster grips today because "a cast's time is the spell's": a school's practice is the one thing that may quicken it). `to_hit` and `threat` are the existing fields. `damage`, `crit`, `parry`, `reload`, `shield`, and the grips' fields are refused for a school. A higher rank is never less, as for every skill.
- **A school's `Benefit` is a `SpellMods`** (§7.1) for the fields they share: `Benefit::spell_mods()` maps `reroll`, `sides`, `dice`, `bonus`, `cells`, `degrees` one to one; `mana` goes to the mana percent and `speed` to the cast price; `to_hit` and `threat` go where Spell attacks' went (§10.1). So a rank's benefit is applied by the code that already applies a talent's: no second path.
- **Defaults** (The Riddle of Steel, as between weapon skills): `evocation` ↔ `transmutation` 4, `necromancy` ↔ `restoration` 4, `abjuration` ↔ `enchantment` 4. A rank 8 evoker is a rank 4 transmuter untried. Divination stands alone.
- **Starting ranks**: the classes' `spell_attack` grants become school grants, each class's main school (§10.1): wizard `evocation = 2`, druid `evocation = 1`, priest `restoration = 1`, witch `necromancy = 1`, paladin `abjuration = 1`, bard `restoration = 1`; the ranger starts untrained and advances by casting, as today. No race gives one. Creatures have no school ranks (their spells are intents, not the cast path).

### 8.3 The seven tables

The to-hit and threat rows are the same in every school (Spell attacks' rows, §10.1) and are listed once here: `to_hit = [0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 6]`, `threat = [0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3]`; they count for the school's spells that roll to hit (today evocation's Ray of frost and Scorching ray and necromancy's Cause wounds), and a school with none today carries them for the ones to come. Each table below is a school's own rows, ranks 0 to 10, and under it what the player feels, one sentence a rank (the wiki prints the table; the game says nothing). The pattern, as the weapon skills' families (`docs/dev/weapon-skills.md`): **the harm schools** (evocation, necromancy) grow their dice; **the mending school** (restoration) grows its heals and quickens them; **the lasting schools** (abjuration, transmutation, enchantment) lengthen and reach; **divination** cheapens and quickens, thin until it has more than Identify.

**Evocation** (Light and eleven damage spells: fire, cold, electricity, force)

| Rank | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `reroll` | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 2 | 2 | 2 |
| `sides` | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 2 |
| `dice` | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 2 | 2 |
| `mana` | 0 | 0 | 0 | 0 | 5 | 5 | 5 | 10 | 10 | 10 | 15 |

1 a surer aim; 2 a surer aim still; 3 **a die that shows 1 is rolled again**; 4 a wider critical range, a twentieth off the mana; 5 **every die a size bigger** (a d6 a d7); 6 **a die more**; 7 a tenth off the mana; 8 **a die showing 1 or 2 is rolled again**; 9 **a second die more**; 10 **every die two sizes bigger**, Thunderstruck (§8.5). A Fireball at the top rung (5d6, 17.5 on average) rolls 7d8 rerolling 1s and 2s at rank 10: 36.8. Ray of frost at its one rung (1d8, 4.5) rolls 3d10 the same way: 18.9, a cantrip for 1 mana that a master still casts.

**Necromancy** (Cause wounds, Blight, Hex of weakness)

| Rank | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `bonus` | 0 | 0 | 1 | 1 | 1 | 2 | 2 | 2 | 3 | 3 | 4 |
| `reroll` | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 2 | 2 | 2 |
| `dice` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 1 |
| `degrees` | 0 | 0 | 0 | 1 | 1 | 1 | 2 | 2 | 2 | 3 | 3 |
| `mana` | 0 | 0 | 0 | 0 | 5 | 5 | 5 | 10 | 10 | 10 | 15 |

1 a surer aim; 2 **a point more on every roll**; 3 1s rolled again, **hexes a degree longer**; 4 a wider critical range, a twentieth off the mana; 5 two points more; 6 hexes two degrees longer; 7 a tenth off; 8 **a die more**, 1s and 2s rolled again, three points more; 9 hexes three degrees longer; 10 four points more, Wasting (§8.5). Its harm bites deeper rather than wider: Cause wounds at its top (2d8+2, 11) rolls 3d8+6 rerolling 1s and 2s at rank 10: 21.8; Blight's 3d6+3 (13.5) becomes 23.7.

**Restoration** (Mend, Cure wounds, Remove disease)

| Rank | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `bonus` | 0 | 0 | 1 | 1 | 1 | 1 | 2 | 2 | 2 | 2 | 3 |
| `reroll` | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 2 | 2 | 2 |
| `sides` | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 2 | 2 |
| `dice` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 |
| `speed` | 0 | 0 | 0 | 10 | 10 | 20 | 20 | 25 | 25 | 33 | 33 |
| `mana` | 0 | 0 | 0 | 0 | 5 | 5 | 5 | 10 | 10 | 10 | 15 |

1 a surer touch; 2 **a point more healed**; 3 1s rolled again, **a tenth quicker**; 4 every die a size bigger, a twentieth off the mana; 5 a fifth quicker; 6 two points more; 7 **a die more**, a quarter quicker; 8 1s and 2s rolled again; 9 every die two sizes bigger, **a third quicker**; 10 three points more. Cure wounds' top (3d6+3, 13.5; two casts of time) heals 4d8+3 rerolling 1s and 2s in two-thirds of the time at rank 10: 27. Mend's 1d6 (3.5) heals 2d8+3 the same way: 13.5. A heal's `bonus`, `dice`, and `sides` work on the `heal` roll as on a `damage` roll (§7.1's hooks are the same hooks).

**Abjuration** (Blessing; the wards to come)

| Rank | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `degrees` | 0 | 0 | 1 | 1 | 1 | 2 | 2 | 2 | 3 | 3 | 3 |
| `speed` | 0 | 0 | 0 | 5 | 5 | 10 | 10 | 15 | 15 | 20 | 20 |
| `mana` | 0 | 0 | 5 | 5 | 10 | 10 | 15 | 15 | 20 | 20 | 25 |

1 a surer aim; 2 **its buffs a degree longer**, a twentieth off the mana; 3 a twentieth quicker; 4 a tenth off; 5 **two degrees longer**, a tenth quicker; 6 three twentieths off; 7 Aegis (§8.5), three twentieths quicker; 8 **three degrees longer**, a fifth off; 9 a fifth quicker; 10 a quarter off the mana. Blessing's degrees are held by its status's `max` as every buff's are (§4.3), so the wards' ceiling is the status's, written in `status.toml`, not here.

**Transmutation** (Barkskin, Haste, Curse of sloth, Web)

| Rank | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `degrees` | 0 | 0 | 1 | 1 | 1 | 2 | 2 | 2 | 3 | 3 | 3 |
| `cells` | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 2 | 2 | 2 |
| `speed` | 0 | 0 | 0 | 0 | 5 | 5 | 10 | 10 | 15 | 15 | 20 |
| `mana` | 0 | 0 | 0 | 0 | 5 | 5 | 5 | 10 | 10 | 10 | 15 |

1 a surer aim; 2 **its buffs and hexes a degree longer**; 3 a surer aim still; 4 **a cell farther** (Web), a twentieth quicker and a twentieth off; 5 two degrees longer; 6 a tenth quicker; 7 a tenth off; 8 **three degrees longer, two cells farther**; 9 three twentieths quicker; 10 a fifth quicker, three twentieths off.

**Enchantment** (Evil eye, Sleep)

| Rank | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `degrees` | 0 | 0 | 1 | 1 | 1 | 2 | 2 | 2 | 3 | 3 | 3 |
| `cells` | 0 | 0 | 0 | 1 | 1 | 1 | 2 | 2 | 2 | 3 | 3 |
| `mana` | 0 | 0 | 0 | 0 | 5 | 5 | 5 | 10 | 10 | 10 | 15 |

1 a surer aim; 2 **its hexes a degree longer**; 3 **a cell farther**; 4 a twentieth off the mana; 5 two degrees longer; 6 two cells farther; 7 a tenth off; 8 three degrees longer; 9 three cells farther; 10 three twentieths off, Lulling (§8.5). Evil eye at Presence 18 (Ill-wished 3, range 8) is cast at range 11 for three degrees more at rank 10, each held by the status's `max`.

**Divination** (Identify; the detections to come)

| Rank | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `mana` | 0 | 0 | 5 | 10 | 10 | 15 | 20 | 20 | 25 | 30 | 40 |
| `speed` | 0 | 0 | 0 | 10 | 10 | 20 | 20 | 30 | 30 | 40 | 50 |
| `cells` | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 2 | 2 | 2 |

1 nothing yet but marks; 2 a twentieth off the mana; 3 a tenth off, a tenth quicker; 4 a cell farther (for a detection with a range); 5 three twentieths off, a fifth quicker; 6 a fifth off; 7 three tenths quicker; 8 a quarter off, two cells farther; 9 three tenths off, two fifths quicker; 10 **two fifths off, half the time**. Identify's 60 mana is 36 at rank 10. Divination is thin because its spell list is; its table is written for the detections the content will bring, and says so in the wiki.

First guesses, as every table in `weapon_skills.toml` is; the probe (§12.4) prints the roll at the ranks the pace reaches and the owner tunes the arrays.

### 8.4 Marks: the one hook, and the pace

**One hook.** `World::practise_cast(cast, outcome)` is called once at the end of `cast_spell` for a cast paid in mana (`Pay::Mana`; a scroll or a wand trains nothing, as Spell attacks trained nothing on them), with the `Outcome` the kinds fill as they go (a `Cell` on the `Cast`, as `read` is): `landed` (a damage spell harmed at least one creature; a hex took on at least one; a heal restored at least a point; a buff, a cleanse, or a utility did its work) and `level` (the level of the strongest creature it harmed, hexed, or healed, else the caster's own: `World::level_of`). It practises **the spell's school** with `practise(school, landed, level)`, the one function every weapon skill's marks go through (`attempt` 1, `hit` 2 more, scaled by `100 + per_level × (level − rank)` percent within 0 and `most`: `[marks]`, unchanged), and then **the caster style** the cast was made in (§10.2), as `World::resolve` practises the style after the weapon skill. It replaces the two places that practise today (`spell_practised`'s `practise` loop for the spells that roll to hit, and `practise_in_armour` for Battle caster): one place, every cast. `spell_practised` keeps `resolve_triggers` (the `hit` and `critical` triggers of the rolling spells). **A miss** (an attack or touch spell that rolled under Defence) is an attempt: as a swing that misses. **An area spell** is one cast, one practise, however many it struck: the strongest struck weighs it. No draw anywhere: marks are counts, rounded as today (`practise` draws nothing).

**What it is not**: no marks for a spell cast at nothing (a Fireball into an empty corridor lands on nobody: `landed` false, an attempt's marks against one's own level), no marks for a buff cast twice in a row beyond the attempt's and success's (a Light cast ten times trains evocation ten times, at a mana each: "no anti-farming limits; time is the penalty", and mana), no daily band, no cap but `most`.

**The pace** (the missile curve, 10 … 5120 marks a rank, 10 230 in all): at parity and landing, a cast is 3 marks, so rank 3 is some 23 landed casts, rank 5 some 100, rank 7 some 420, rank 8 some 850, rank 10 some 3 400. A wizard casts ten to twenty spells a day (the pool, §4.4), nearly all evocation: rank 5 in the first fortnight (about level 8 to 10), rank 7 by level 20, rank 8 by 30, rank 10 toward 60 if the run is long enough; a priest's restoration follows the party's wounds. These are the estimates of §12.3's rank column, to be **measured by the probe** (S7, §12.4: casts per level from the pool and the pace from `[marks]`) and tuned in the data (a `spell` curve under `[curves]` is one line if the missile curve proves slow or quick). A warrior's long blades reach rank 7 in about the same number of blows; a fighter swings more often than a caster casts, and a caster's spells cost mana: the curve may need to be the quicker of the two, and the data allows it.

### 8.5 School abilities: what a rank teaches

As weapon ranks teach weapon abilities (`docs/dev/weapon-abilities.md`), school ranks teach **school abilities**: entries in `data/weapon_abilities/` with `skills = ["evocation"]` (the loader's "a weapon skill or a style" widens to "or a school"), triggered, listed in the one abilities menu under the school ("Evocation: Thunderstruck, fires by itself"), had through `Actor::ability_rank` (so Early mastery's rank counts for them too), said when the rank comes ("You feel more skilled at evocation. You have learned to strike thunderstruck."). **One new trigger**, `spell_lands` (the closed `Trigger` gains it, `defending()` false): raised once per creature a cast paid in mana **lands on** (harmed, hexed, or healed; for a buff, the caster), from the one place the kinds report a landing (`World::spell_landed(by, other, skills)`, which also fills the `Outcome` of §8.4: the two are one function), with the cast's `AttackSkills` (the school and the style). It fires for area spells too, which `hit` never did (an area has no roll). Four ship, so the trigger and the loader's widening have users:

| Ability | Skill, rank | Trigger | Effect | In the player's words |
|---|---|---|---|---|
| **Thunderstruck** | evocation 10 | `spell_lands` | `status = "staggered"` on the other | what your evocations harm is staggered, slow to act next |
| **Wasting** | necromancy 10 | `spell_lands` | `status = "weakened"` on the other | what your necromancy harms is left weak a while |
| **Aegis** | abjuration 7 | `spell_lands` | `status = "spell_ward"` on self (a buff lands on its caster) | a buff you cast wards you until you act again |
| **Lulling** | enchantment 10 | `spell_lands` | `status = "sluggish"` on the other | a creature your enchantment takes is also slow a while |

Each a status the game has (Staggered `per_action`, Weakened as Withering touch lays it, Spell ward as Spell guard's, Sluggish as Numbing cold's), each one line a creature sees ("The orc staggers."), each `systems = ["weapon_abilities", "schools", "spells", "status"]`. Restoration, transmutation, and divination teach none yet: their rank 10 is a benefit; abilities come with their spells. **Riders twice?** A talent's rider (§7.1 #4, Numbing cold) is a `SpellMods` entry laid as the spell's own status is; a school ability is a triggered ability laid by `weapon_trigger`. They coexist as a Rend ability and an extra-damage talent do: one is a talent's sentence, the other a rank's, and the menu lists the second.

### 8.6 Talents: the `schools` system

School skills are a new system (CLAUDE.md's coverage policy): `[[system]] id = "schools"`, name "Schools of magic: ranks and marks", in `talents.toml` after `spells`. The bar (`min_talents` 2) is carried by three talents and a class ability, each one sentence, each on machinery the weapon skills' talents already have:

| Talent | Tree, needs | Effect | In the player's words |
|---|---|---|---|
| **Schooled** (new) | Spellcraft, `choice = "school_skill"` (one of the seven), `needs = { chosen = 2 }` | `weapon { ranks = 1 }` on the chosen school (Stylist's effect: its benefits read a rank higher; abilities a rank sooner through `ability_rank`) | you cast the spells of your chosen school as one a rank more practised |
| **Devotee** (new) | Spellcraft, level 9 | `school_marks_percent = 50` (the marks of every school half again, as a weapon talent's `marks_percent` would; read in `practise_cast` only) | your schools learn from every cast half again as much |
| **Magister** (new, the wizard's class ability, level 45) | class | `weapon { role = "school", ranks = 1 }` (every school a rank higher: the `weapon` effect's "every skill of the role's" arm) | every school of magic answers you as one a rank more practised |
| **Early mastery** (exists) | Weaponcraft | unchanged: `ability_ranks` counts for school abilities through `ability_rank` | its description gains "and schools of magic" |

`choice = "school_skill"` is a new choice kind (the seven schools), distinct from `school` (a school or an element, Keen mind's and School focus's) because a school skill is a skill, not a tag. Systems: Schooled and Magister `schools`, `spells`; Devotee `schools`, `spells`. **Why these three**: a chosen-school rank is Weapon focus's pattern and the one most casters want; marks half again is the weapon skills' missing talent (none exists for weapons; it is a candidate for both, deferred); a class ability at a late level keeps the wizard's table owning its craft. No talent touches the curve or `[marks]`.

### 8.7 Where a rank is read: one place each

| What | Where | How |
|---|---|---|
| the hooks (`reroll`, `sides`, `dice`, `bonus`, `cells`, `degrees`) | `Actor::spell_mods(content, spell)` | adds `school_benefit(spell.school).spell_mods()` to the sum of the talents' and items', filtered by nothing (a school's benefit is for its own spells by construction); the school's rank is `ability_rank(content, school)` (Schooled's and Magister's `ranks` count) |
| `mana` | `Actor::spell_mana_percent` | one more negative term beside School focus's and Frugal caster's; the floor of 1 mana holds |
| `speed` | `World::cast_price` | `percent_of(cost, 100 − speed)`, as `attack_price` takes a weapon skill's `speed` off a blow |
| `to_hit`, `threat` | `Actor::spell_skills(content, aim)` → `AttackSkills { skill: school, style }` | the school in the slot Spell attacks held; `benefit_of(skills)` adds them as today (§10.1) |
| the marks | `World::practise_cast` | §8.4 |
| the abilities | `Actor::weapon_abilities`, `World::weapon_trigger` | §8.5 |

**What the player sees**: the Magic page (§11) with the rank and the bar; the menu's roll includes the rank's dice (§11: "4d7 fire" at evocation 5, as it includes a talent's); "You feel more skilled at evocation." as a rank comes, with what it taught. Nothing says what a rank gives. Dev: the `schools` dump layer (`Evocation 5 (40%)`, `Sniper 2 (10%)`, the caster styles with it), JSON `schools` (`id`, `rank`, `progress`) and `caster_styles`; `rank:evocation:5` sets a rank as a rank risen would (the existing dev action: a school is a skill of the table); `progress` adds `evocation marks 312/630`.

## 9. Spell mastery, weighed: ranks per spell

Decision 17's second sentence: *"Another thing we could do, is that each spell has its own hand authored 10-mark progression (similar to weapon skill)."* Weighed as an alternative and as an addition.

| | School skills (§8) | A ten-rank table per spell | Both |
|---|---|---|---|
| **Authoring** | 7 tables, about 40 rows, written once; a new spell joins its school and needs nothing | 26 tables now (260 steps), one per spell forever: a new spell is a spell file and a progression, balanced against its ladder, its school, the talents, and the items | 7 + 26 tables |
| **Learning by use** | the school you cast grows; the feel is DCSS's skills: a fire mage, not a fireball-caster | the spell you cast grows; the feel is Elona's: a favourite spell outgrows its siblings, a new spell starts cold | both |
| **The ladder** | the ladder is the spell's own growth by stats; the rank adds through the hooks; two things, each one sentence | a per-spell rank wants to add rungs (then a 4-rung ladder cannot take 10 steps) or riders (then it is a per-spell talent table) or dice (then it is the school's job again, per spell) | three progressions on one number |
| **The menu's number** (decision 11) | the roll shown is base + ladder + school rank + talents + items: four sources, one line each on the wiki | five sources | five |
| **The Magic page** | 7 bars | 26 bars and growing; a scroll for the bars alone | 33 bars |
| **The concept count** (§3.3) | 3, 4 for a trained caster | 4 | 5 |
| **A new spell's cost to the player** | it starts at the school's rank: a level-30 evoker's new evocation is strong at once | it starts at 0: a new spell is weak until cast a hundred times, which is an anti-content pressure (the player sticks with the spell they trained) | as the second |

**Recommendation: school skills only.** The second column's last row is the decider: a game that will add spells for sixty more levels should not punish a new spell with a cold start, and a school rank warms every spell of the school the day it is learned. The first column's authoring row is the second decider: the content is the progression (§4.5), and content is cheaper to write when a spell is a file and nothing else. What per-spell mastery would add that nothing else does, a favourite's edge over its siblings, is worth less than what it costs, and **the door stays open**: a per-spell table would be `[spell.mastery]` arrays of 11 on the spell file, a `Training` per known spell in `Actor::spells` (saved), the same `practise` (the spell beside the school in `practise_cast`), the same hooks; an additive slice of size M at any later time, with no change to anything in §8. Listed as deferred, not rejected, in §14.

**Worked examples**, every layer of the final design on one spell each, for the trained wizard and priest of `docs/balance.md` (Intelligence 16 / 20 / 23, Willpower 17 / 21 / 26 at levels 10 / 30 / 60) at the ranks §8.4 predicts (5 / 8 / 10). The menu shows the last line; the wiki prints each layer in its own table.

| Layer | Fireball (level 11, evocation, burst 2, 26 mana) | Cure wounds (level 3, restoration, touch heal, 2 mana, two casts of time) | Haste (level 11, transmutation, self buff, 11 mana) |
|---|---|---|---|
| **Base** (anyone) | 2d6 fire; range 10 | 2d6 healed | Hasted 1 |
| **Ladder** (the row) | `intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2`; `range willpower 14 11, 18 12` | `willpower 16 3d6, 22 3d6+3` | `degrees presence 14 2, 18 3` |
| **Rung reached** at 10 / 30 / 60 | 4d6 / 5d6 / 5d6; range 11 / 12 / 12 | 3d6 / 3d6 / 3d6+3 | the wizard's Presence 11 / 13 / 15: Hasted 1 / 1 / 2 |
| **School rank** 5 / 8 / 10 | evocation: d7, 1s again / +1 die, d7, 1s and 2s / +2 dice, d8, 1s and 2s; mana −5 / −10 / −15 % | restoration: +1, d7, 1s again, a fifth quicker / +2, +1 die, d7, 1s and 2s, a quarter quicker / +3, +1 die, d8, 1s and 2s, a third quicker | transmutation: +2 degrees, +1 cell, a twentieth quicker / +3 degrees, a tenth off, three twentieths quicker / +3 degrees, a fifth quicker |
| **Talents and items** (the hooks) | Empower (32) a rung: 5d6+2 at 60; Keen mind (evocation) +1 size; a Potent ring +2 | Sure hand: 1s again (already); *of Lingering* nothing (a heal) | *of Lingering* +1 degree; School focus (transmutation) −10 %, +1 cell (nothing to reach) |
| **The menu shows** at 10 / 30 / 60 | `4d7 fire, 25 mana, aimed` (17.7) / `6d7 fire, 23 mana, aimed` (28.3) / `7d9+4 fire, 20 mana, aimed` (Keen mind, Empower, the ring: 44.4) | `3d7+1 healed, 2 mana, aimed` (14.3) / `4d7+2 healed, 2 mana, aimed` (20.9) / `4d8+6 healed, 2 mana, aimed` (27) | `10 mana` (Hasted 3, held by the status's max 3) / `9 mana` (3) / `8 mana` (3) |
| **Per-spell mastery would have added** | a fifth source on the same line | a fifth | a fifth, on a spell already at its status's max |

(The menu line omits a reroll, as a weapon's line omits a Rend: the dice are the dice. The description line names it: "Your 1s and 2s are rolled again.")

## 10. Spell attacks and the caster styles: the audit and the rework

Decision 17: *"'Spell attacks' skill also comes under the scope of this redesign. Also the spellcasting related fighting styles. Take a look at the existing fighting styles and spell attacks weapon skill, and deliberate how they mesh with what we are designing (i don't think it's that good match yet)."*

### 10.1 Spell attacks: what it is, and why it merges

| | Today |
|---|---|
| The skill | `spell_attack`, `role = "spell"`, the missile curve; `to_hit = [0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 6]`, `threat = [0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3]`, `crit = [0 … 1, 1, 1]` (rank 8); the one skill of its role; its own `Section::Spells` on the weapon skills screen; the wiki's "Spell attacks" paragraph |
| Trains on | every spell paid in mana that **rolls to hit**: `SpellData::rolls_to_hit` = a `damage` spell with `attack` or `touch` resolution and a ray, bolt, or touch shape. Of the 26 spells that is **three**: Ray of frost, Scorching ray, Cause wounds. Heals, hexes, buffs, and every area and auto spell (Fireball, Magic missile, Lightning bolt, Sleep …) train it never and read it never |
| Classes | wizard 2, druid 1, priest 1, witch 1, paladin 1, bard 1: five of the six start ranks in a skill that **none of their granted spells trains** but the wizard's Ray of frost and the witch's Cause wounds (the priest's, the paladin's, the druid's, and the bard's granted spells roll nothing to hit) |
| Reads | `Actor::spell_skills` → `AttackSkills { skill: spell_attack, style }`; `cast_single`'s `benefit_of(skills).at_distance(...)` into the `Profile` |

**How it meshes**: badly, as the owner suspected. A weapon skill carries a weapon's to-hit *and* its damage; Spell attacks carries the to-hit of three spells and nothing of the other twenty-three, so a wizard's skill bar moves on one spell in six. Beside a school skill it would be a second skill a spell trains (its school and, if it rolls, Spell attacks), two bars for one cast, with the to-hit in one and the dice in the other: the split a weapon skill does not make. **Decision: retire it, merged into the school skills.** Every school's table carries its `to_hit` and `threat` rows (§8.3): a spell that rolls to hit aims with its school's rank, as a sword strikes with long blades', and the school trains on every spell, so the aim of Ray of frost improves as the wizard casts Fireball, which is what "practised at evocation" should mean. Its `crit` row (a multiplier at 8) is not carried over: Toucher's `crit` keeps the touch critical's edge and the attack spells' critical stays a blow's (decision 4).

**Migration**: the `spell_attack` row goes from `weapon_skills.toml` and `Role::Spell` from the loader (a `role = "spell"` row is refused naming `school`); `Section::Spells` leaves the weapon skills screen (§11); the classes' `spell_attack = N` become the school grants of §8.2; `Actor::spell_skills` returns the spell's school; `[spell_attacks] far` stays (Sniper reads it, §10.2); the wiki's "Spell attacks" paragraph on `weapon-skills.md` and its rows in the generated tables go, and the new page (§11) says where the aim lives. **Saves change**: the skills after the row (the styles) move an index, and a save's `Training` for Spell attacks is **dropped, not converted** (marks in it were marks on three spells; the class's starting school rank is the same number the class gave in Spell attacks). Said in the PR; `FORMAT_VERSION` untouched (the integrator's per-release bump already refuses the 0.9.0 saves this content change invalidates, `docs/dev/saves.md`).

### 10.2 The caster styles: what they are, and the rework

| Style | Grip today | Benefits today | Abilities | Trains on today |
|---|---|---|---|---|
| **Sniper** | `ranged_spell`: a ray or a bolt that rolls to hit, out of hindering armour | `to_hit` 0–4, `far_to_hit` 0–3 and `far_threat` 0–2 at `[spell_attacks] far` (5) cells or more | Pinning ray (4, on a critical) | the two attack rays (Ray of frost, Scorching ray) |
| **Toucher** | `touch_spell`: a touch that rolls to hit, out of hindering armour | `to_hit` 0–5, `threat` 0–2, `crit` 0–2 | Staggering touch (4, critical), Withering touch (10, hit) | the one touch that rolls (Cause wounds) |
| **Battle caster** | `armoured_spell`: a spell that rolls to hit, in armour with a `spell_cost_percent` (first) | `to_hit` 0–4, `armour` 5–50 % off what armour adds to every spell's mana | Spell guard (4, hit) | the three, in armour; and an attempt's marks on every other mana spell in armour |

**How they mesh**: a fighting style is a **choice**: one weapon or two, a shield or not, a staff. Sniper and Toucher are not choices: a ray is always a Sniper's and a touch always a Toucher's, so they are the spell's shape wearing a style's name, a hidden second rank on the same three spells. The owner's reading ("not that good match") is right. Battle caster *is* a choice (armour or not) and already trains on every spell; it is the model. **Decision: rework Sniper and Toucher into choices of where you cast from, and let all three train on every spell they are cast in.**

> **Where you cast from is your style.** A spell you aim at a creature **beside you**, a touch or anything else, is cast as a **Toucher**. A spell you aim **farther** than that, at a creature or a cell, a ray or a burst alike, is cast as a **Sniper**; from `far` cells away a Sniper is at their best. A spell cast in **armour that hinders casting** is a **Battle caster's**, wherever it is aimed. A spell on yourself out of armour is cast in no style. Every spell paid in mana trains the style it was cast in, lands or not, as every blow trains its style.

The grips keep their ids (`ranged_spell`, `touch_spell`, `armoured_spell`: no data rename, the styles' indices stand) and widen their meaning in the README and the loader: `Actor::spell_skills(content, aim)` takes the cast's **aim** (`Aim::Armoured` when `worn_spell_cost > 0`, else `Beside` when the target is a creature at distance 1, else `Afar` when the spell is aimed at a cell or creature at all, else `None`) in place of today's `touch: bool`, decided once in `cast_spell` before the kinds run, so the school and the style are one `AttackSkills` for the whole cast (the marks, §8.4; the `hit` triggers of a rolling spell; `spell_lands`). An area spell is `Afar` (a burst is aimed at a cell; a cone and a line at a direction: never "beside").

| Style | Rework | Benefits (ranks 0 to 10) | What the player feels |
|---|---|---|---|
| **Sniper** | every aimed spell at distance 2 or more, out of hindering armour | `to_hit` and `far_to_hit`, `far_threat` as today; **`cells = [0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2]`** new (a style's reach, after the school's) | your spells from afar hit better, from `far` better still; from rank 5 they reach a cell farther, from rank 9 two. Pinning ray unchanged (a critical of a rolling spell) |
| **Toucher** | every spell at a creature beside you, out of hindering armour (a touch; a ray or a hex at the creature next to you; a heal laid on the one beside you) | `to_hit`, `threat`, `crit` as today; **`bonus = [0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3]`** new (on the roll of a spell cast so, harm or healing) | up close your spells hit better and critical wider and harder; from rank 4 they do a point more, pressed home. Staggering touch and Withering touch unchanged (a rolling spell's critical and hit) |
| **Battle caster** | as today, and **full marks** on every spell in such armour (an attempt and a success as the cast landed, weighed by the strongest struck), in place of today's attempt-only on the spells that do not roll | unchanged (`to_hit`, `armour`) | as today; it simply trains at the pace of the other styles. Spell guard unchanged |

`cells` and `bonus` are `Benefit` fields allowed for the caster grips (checked per grip, as `far_to_hit` is), mapped into `SpellMods` by the same `Benefit::spell_mods()` as a school's (§8.2) and added in `Actor::spell_mods` after the school's: a style's benefit adds to the skill's, as a weapon style's does. `speed` stays refused for the caster grips (a school's practice quickens a cast; where one stands does not). The styles' defaults (`sniper` ↔ `toucher` 4, `toucher` ↔ `battle_caster` 3), the classes' starting styles (wizard and druid Sniper, witch and priest Toucher, paladin Battle caster), and Stylist's choice are unchanged. `PerceptionView::own_styles_in_use` marks **Battle caster** while the armour worn hinders casting (a standing fact, as a grip is) and never Sniper or Toucher (they are decided by the target, cast by cast); the Magic page says so in its legend.

**Rejected**: *retiring Sniper and Toucher* (only Battle caster left): simplest, but it leaves the caster with one style that most casters never wear armour for, and loses two styles' abilities (Pinning ray, the touches) that are good content; *one caster style per school*: a school is already the skill; *a style per shape* (ray, burst, touch): the spell's choice again, not the player's. **Why distance**: it is the one thing about a cast that the *player* decides every time (where to stand), as a fighter decides what to hold; it maps on the existing grips without a rename; and it makes the two styles mean something for the twenty-three spells that never roll to hit.

**Migration**: the three style rows gain their new arrays; `Grip::casts` keeps the three grips; the README's grip lines are rewritten to the rule above; the classes untouched; `caster_style_tests` rewritten (a Fireball from afar trains Sniper; a Blessing on oneself trains none; a heal on a creature beside trains Toucher; in armour every one trains Battle caster); the wiki's "Caster styles" section moves from `fighting-styles.md` to the new page (§11) and its tables (`style-benefits.md`'s caster rows, `style-abilities.md`'s) are generated there. **Saves do not change** (the rows keep their places; marks in them stand).

## 11. What the player sees; the Magic page; the wiki, the spec, the docs

**In the game** (decision 11, verbatim: *"uhm, no. Just put the damage number to the menu as you would with a weapon."*):

- **The abilities menu's Spells rows** (`ui/abilities_screen.rs`): the detail column is the spell's roll as the caster would make it now (`Actor::spell_roll`: the rung by stats, the rungs, the dice hooks; §7.1), its damage type or "healed", its mana, and "aimed" where it is: `Fireball          4d6 fire, 26 mana, aimed`, `Cure wounds       3d6 healed, 2 mana, aimed`, `Blessing          3 mana`, `Sleep             11 mana, aimed`. Short of mana the suffix is as today ("4d6 fire, 26 mana: you have 14"). No level, no tier: a known spell's level is its mana. The style is the character screen's attack line (`+2 to hit, 1d6+1 slashing or piercing`), without the to-hit: an attack spell's to-hit is one more number in a 31-column cell, and the aiming mode is where a shot is weighed; it may be added later. The **description line** for the selected spell appends the reach as it is now, in cells: *"... Range 11, radius 2."* (`PerceptionView::own_spell_reach`, which exists). The aiming mode's reach is the rung reached, as today.
- **The character screen** is unchanged: its attacks column holds three (main hand, off hand, missile) and a wizard knows six damage spells; spells have the menu.
- **No "your spell grows" line** (decision 11: no). A stat rising says what it says today; the menu shows the new dice next time it opens. A **school rank** rising says what a weapon rank says ("You feel more skilled at evocation.", with what it taught, §8.5).
- **The Magic page** (decision 17: *"should probably be moved to a separate 'Magic' page (scrollable)"*): `ui/magic_screen.rs`, `ScreenName::Magic`, **page 6** of the info pages, between Weapons and Quests, so the three pages of what the player has trained stand together (4 Skills, 5 Weapons, 6 Magic); Quests, Talents, and Story become 7, 8, 9, the ninth and last digit taken (`PAGES` in `ui/info_tabs.rs`; `WIDTH` recomputed to the longer tab line, "1 Character · … · 9 Story", about 111 cells, so a console under it shows the short names, "Char · Inv · … · Mag · … · Story", as the rule already has it). Its panel is `panel_width` wide and **one column that scrolls** (↑↓, the weapon skills screen's narrow mode, on every console: the page is a list, not a grid), sections in order:
  - **Schools**: the seven, each by name with its rank and the ten-cell marks bar, trained ones bright and untrained dim, as the weapon skills screen draws a skill (`PerceptionView::own_weapon_skills`, filtered to `Role::School`; `shows` is true for every school: a caster should see what they have not trained);
  - **Caster styles**: Sniper, Toucher, Battle caster, rank and bar, `•` on Battle caster while the armour worn hinders casting (§10.2), with the legend line "• the style you cast in now" as the weapon skills screen's;
  - **Spells**: every spell known, by level then name (the menu's order), each row as the menu's (`Fireball   4d6 fire, 26 mana, aimed`: `PerceptionView::own_spell_roll`, S4's). No cursor and no description: the page is read, not used; the abilities menu is where a spell is chosen and its description read;
  - the header line: `Mana 47/106` (what the status bar shows, `PerceptionView::own_mana`); no caster level (a learning number, §4.4, shown nowhere today but the dumps).
  A character with no mana and no spells sees the schools at rank 0, no styles, and "You know no spells." (the page exists for everyone, as Weapons does for a wizard). **Opening it**: by its number or Tab from any info page, and the Weapons page's hint line names it ("6 Magic"). It has **no opening key in play of its own**: a new rebindable `KeyCommand` is out of scope for this document (the integrator's rule for this round); whether the owner wants one (`M`, as `W` opens Weapons, which the hard rules allow a play-context command to be) is a question in §16, not a decision here.
  - **The Weapons page** loses `Section::Spells` (Spell attacks is gone, §10.1) and the caster styles (they are the Magic page's); its two columns are melee and missile on the left, shields and the weapon styles on the right, as today without the spells; `weapon-skills-screen-wizard` changes (no Spells section, no Sniper).
  - **Text snapshots** (`ui/screen_snapshots.rs`, `--screen magic`): `magic-screen` (the showcase wizard: evocation 2, Sniper 1, six spells with their rolls), `magic-screen-trained` (a `rank:evocation:5 rank:restoration:3` replay: climbed bars, a d7 in a row), `magic-screen-armoured` (a paladin in mail: Battle caster marked), `magic-screen-none` (a warrior: "You know no spells."), `magic-screen-short` (80 × 24: the scroll). The info pages' tab line changes on every page snapshot that shows it (`character-screen`, `inventory-screen`, …: "6 Magic" and the renumbered three), read as a text diff.
  - **The wiki**: a new page **"Schools of magic"** (`wiki/src/schools.md`, under Magic in `SUMMARY.md`: `- [Magic](magic.md)` gains `  - [Schools and caster styles](schools.md)`), with: the rule of §8.1; the generated `school-skills.md` (School, Spells (count of the shipped list), Starting ranks by class); `school-benefits.md` (the seven rank tables of §8.3, with the to-hit and threat rows, in `rank_table`'s format); `school-abilities.md` (§8.5); the caster styles' section moved from `fighting-styles.md` with its tables (`caster-style-benefits.md`, `caster-style-abilities.md`) rewritten to §10.2's rule; the Magic page's paragraph ("`6` from any info page ... one column that scrolls"); and the vagueness line ("The game shows your ranks and a bar; what a rank gives is here, exactly"). `weapon-skills.md` loses the Spell attacks paragraph and says "spells have a page of their own"; `fighting-styles.md` keeps a one-line pointer; `controls.md`'s info page table (6 Magic, 7 Quests, 8 Talents, 9 Story); `quests.md` "page 7", `talents.md` "page 8", `background.md` "page 9"; `classes.md`'s starting ranks column (school for Spell attacks); `magic.md` "The spells" links the schools.
- **Wands**: §7.4.
- The dump `spells` layer prints what the menu does: `Fireball (level 11, 26 mana): 4d6 fire, range 11, radius 2`; JSON `spells[]` gains `level`, `roll` (the text), `damage_type`, `reach` (`range`, `length`, `radius`, each null where none); dev `progress` adds `fireball rung 2/4 (In 16; +1 Empower)`. The `--screen abilities` snapshot `abilities-menu-spells` changes; a new case `abilities-menu-spells-grown` (a wizard at Intelligence 19 with Empower) shows a climbed ladder and a rung from a talent; `wand-description` changes (the dice line).

**In the wiki** (exact; the generator `xtask/src/wiki/spells.rs`):

- `spell-list.md`: columns Spell, **Level**, School, Shape and resolution, Reach (the base: `range 10, radius 2`), What it does (the base: `2d6 fire damage; it melts ...`), **Grows with** (each ladder as written, in words: `Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2; range Willpower 14: 11, 18: 12`; `—` without), Mana (at its level, no armour). Sorted by level, then name.
- `spell-tiers.md` deleted (the `{{#include}}` with it); `spell-resolutions.md`: Resolution, Roll, Cost. `study-times.md` and `study-chances.md` sample spell levels (§5.9). `wands.md`: Strength for Caster level, Charges as before. `rods.md`: Reach as a spell's; "Takes creatures to level 20" for sleep and terror. `class-levels.md` and `talents.md` inherit `does`; `class-levels.md` prints "(level 11, from 12)". `scrolls.md`'s spell scroll table: "Casts at".
- `magic.md`: "The spells" explains the ladder in the words of §4.2; **"Tiers" becomes "Spell levels"** (§3.1 row 5, one paragraph, no table); "Mana" loses the overreach sentence (row 12); "Caster level" is rewritten as learning (row 11); "Spellcraft talents" and "Damage and its cap" are replaced by **"How a spell grows"** (the rule; what talents and items may do, one sentence per hook with the shipped example; scrolls and wands at their rungs; criticals as blows); "The lifted cap" is deleted; "Shapes" loses its Willpower and Presence paragraphs (one sentence each pointing at the rows); "Learning" in levels (§5.4's sentence and the sample tables); "Wands" and "Rods" as rows 13 and 14. `scrolls.md` row 15. `magic-items.md`'s Potent, of Wands, of the Adept lines. `stats.md`'s Intelligence, Willpower, Presence entries say "grows the spells whose ladders name it". `classes.md` and `features.md`: "by its level" for "by its tier".

**The spec and the docs**: spec §8 "Spells" (`spec/08e-blood-and-status.md`) bullets **Spells**, **Tiers** (→ **Levels**), **The cap**, **The lift**, **Cost**, **Casting: Reach, Dice**, **Caster level**, **Learning**, **Scrolls of spells**, **Wands**, **Rods** rewritten to the proposal (the lift bullet deleted, a **Ladders** and a **Hooks** bullet added); `08i-character.md`'s class lists "(level 1, at 1 ...)"; `08g-magic-items.md`'s wand bullet; `docs/dev/spells.md`'s Data, The cap, Pure helpers, The caster's stats, The one cast path (Cost, Reach, Dice), Choices, Learning, Wands, Rods, Talents, Balance, Wiki, Tests sections; `docs/dev/scrolls.md`, `docs/dev/magic-items.md` (the affixes, `Stack::wand_rung`), `docs/dev/saves.md` (the wand rung), `docs/dev/progression.md` (the talents), `docs/balance.md` (§12.4); `README.toml` and `spells.toml`'s comments; `docs/design/README.md`'s entry for this document.

**For the school skills and the styles** (§8, §10): spec §8 "Weapon skills" (`spec/08i-character.md`: the roles gain `school`, the Spell attacks bullet goes, a **Schools of magic** bullet comes with the marks rule of §8.4), "Fighting styles" (the caster styles' bullet rewritten to §10.2's rule), "Weapon abilities" (the `spell_lands` trigger), "Talents" (the `schools` system), §6's info pages list (the Magic page); a new topic file **`docs/dev/magic-skills.md`** (the school rows, `Benefit::spell_mods`, `practise_cast`, `spell_lands`, the Magic page, the tests; one line each in `docs/dev/weapon-skills.md`, `weapon-abilities.md`, `spells.md`, `screens.md`, and CLAUDE.md's topic index); `weapon_skills.toml`'s header comment (the `school` role, the spell fields, the caster grips' rule); `data/weapon_abilities/README.toml` (the trigger, a school as a skill); `data/talents/README.toml` (`choice = "school_skill"`, `school_marks_percent`); CLAUDE.md's dump layers (`schools`) and `--screen magic`.

## 12. Migration: every spell, with before and after

### 12.1 Damage and healing ladders (all 15 spells that roll)

Unchanged from round 1 (decisions 6, 7, 14, 15: the cantrips grow once, Blight, Cause wounds, and Mend get Willpower ladders, the grid stands, Scorching ray keeps its one mixed rung). The base is what the spell rolled at its old comfortable caster level today (its `dice`), so a scroll or a wand at rung 0 is unchanged. The last rung is as far as the author took it; nothing measures it against anything (decision 16).

| Spell (level, resolution) | Today | Proposed `damage` / `heal` | Shift from today |
|---|---|---|---|
| Ray of frost (1, attack) | 1d6, no growth, cap 6 | base `1d6`; `intelligence 16 1d8` | grows once (today never) |
| Spark (1, auto) | 1d4, no growth, cap 4 | base `1d4`; `intelligence 16 1d6` | grows once |
| Mend (1, touch heal) | 1d3, no growth, cap 7 | base `1d3`; `willpower 16 1d4, 22 1d6` | grows twice |
| Magic missile (3, auto) | 1d4 +1 die per 3 to 2d4; lift In 10 Wi 5, cap 9 → 13 | base `1d4`; `intelligence 13 2d4, 16 3d4, 22 3d4+1` | same at 10 to 40; +1 at 60 |
| Burning hands (4, area) | as Magic missile | base `1d4`; `intelligence 13 2d4, 16 3d4, 22 3d4+2` | same at 10 to 40; +2 at 60 |
| Cause wounds (4, touch) | 1d6 +1 per 4 to 2d6; no lift; cap 14 | base `1d6`; `willpower 13 2d6, 19 2d8, 25 2d8+2` | grows at 30 and 60 (today flat from level 7) |
| Cure wounds (3, touch heal) | 2d6; lift Wi 15, cap 14 → 21 | base `2d6`; `willpower 16 3d6, 22 3d6+3` | same to 30; +3 at 40 on |
| Scorching ray (7, attack) | 1d6 +1 per 6 to 3d6; lift; cap 18 → 28 | base `1d6`; `intelligence 13 2d6, 16 3d6, 19 4d6, 25 3d6+1d10` | 2d6 from In 13 (today 1d6 to caster level 12); −1 die at 20; same at 10, 30, 60 |
| Arcane bolt (6, area) | 2d6 flat; lift; cap 14 → 22 | base `2d6`; `intelligence 13 3d6, 19 3d6+2, 25 3d6+4` | same to 20; +2 at 30 on |
| Frost (6, area) | as Arcane bolt | base `2d6`; `intelligence 13 3d6, 19 3d6+2, 25 3d6+4` | as Arcane bolt |
| Blight (7, area) | 2d6 flat; no lift; cap 14 | base `2d6`; `willpower 16 3d6, 22 3d6+3` | grows (today never): +1 die from Wi 16 |
| Fireball (11, area) | 2d6 +1 per 4 to 3d6; lift; cap 19 → 32 | base `2d6`; `intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2` | +1 die at 10; −1 at 20; same at 30, 60 |
| Flame (12, area) | 3d6 flat; lift; cap 19 → 32 | base `3d6`; `intelligence 16 4d6, 19 5d6, 25 5d6+2` | same at 10, 30, 60; −1 at 20 |
| Lightning bolt (11, area) | as Fireball | base `2d6`; `intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2` | as Fireball |
| Call lightning (11, area) | as Fireball | base `2d6`; `willpower 13 3d6, 16 4d6, 19 5d6, 25 5d6+2` | as Fireball; Willpower (the druid's stat, decision 3) |

The `aim` of Ray of frost, Scorching ray, Cause wounds, Mend, and Cure wounds stays Dexterity (today's default; no spell names another). Identify, Remove disease, Light, Barkskin, Blessing, Haste, Evil eye, Curse of sloth, Hex of weakness, Sleep, Web roll nothing.

### 12.2 Reach and degrees ladders (every aimed spell, every buff and hex, the rods)

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

### 12.3 Before and after: the numbers

Computed for this document by a script over the rules as written (today: `SpellDice::at` and `lifted` with the lift, the roll's expectation held to the lifted cap, exact; proposed: the ladder's rung at the score, and then the school's rank through §8.3's arrays, a reroll's mean exact; mana by §5.3), re-run for round 3 with §5.8's levels and §8.3's tables, for the **trained wizard** of `docs/balance.md` (Intelligence 16 / 18 / 20 / 23 and Willpower 16 / 18 / 20 / 25 at levels 10 / 20 / 30 / 60), no talents. The averages are **per cast on one target**; what they mean against a monster is the probe's to say on the real code (S7, §12.4), not this table's. The script is not committed.

**At school rank 0** (the migration alone: what a caster who has never cast would roll):

| Spell (level, res) | L | Today: dice, held to | avg | Proposed: rung | avg | Mana today → proposed |
|---|---|---|---|---|---|---|
| Ray of frost (1, attack) | 10 / 30 / 60 | 1d6 → 6 | 3.5 | 1d8 (In 16 / 20 / 23) | 4.5 | 1 → 1 |
| Spark (1, auto) | 10 / 30 / 60 | 1d4 → 4 | 2.5 | 1d6 | 3.5 | 1 → 1 |
| Magic missile (3, auto) | 10 / 20 / 30 / 60 | 3d4 → 13 | 7.5 | 3d4 / 3d4 / 3d4 / 3d4+1 | 7.5 / 7.5 / 7.5 / 8.5 | 3 → 3 |
| Burning hands (4, area) | 10 / 20 / 30 / 60 | 3d4 → 13 | 7.5 | 3d4 / 3d4 / 3d4 / 3d4+2 | 7.5 / 7.5 / 7.5 / 9.5 | 5 → 6 |
| Cause wounds (4, touch) | 10 / 20 / 30 / 60 | 2d6 → 14 | 7.0 | 2d6 / 2d6 / 2d8 / 2d8+2 (Wi) | 7.0 / 7.0 / 9.0 / 11 | 2 → 3 |
| Scorching ray (7, attack) | 10 / 20 / 30 / 60 | 3d6 → 26 / 4d6 → 28 / 4d6 / 4d6 | 10.5 / 14 / 14 / 14 | 3d6 / 3d6 / 4d6 / 4d6 | 10.5 / 10.5 / 14 / 14 | 6 → 7 |
| Arcane bolt, Frost (6, area) | 10 / 20 / 30 / 60 | 3d6 → 20 / 22 / 22 / 22 | 10.5 | 3d6 / 3d6 / 3d6+2 / 3d6+2 | 10.5 / 10.5 / 12.5 / 12.5 | 11 → 11, 14 → 14 |
| Blight (7, area) | 10 / 20 / 30 / 60 | 2d6 → 14 | 7.0 | 3d6 / 3d6 / 3d6 / 3d6+3 (Wi) | 10.5 / 10.5 / 10.5 / 13.5 | 14 → 17 |
| Fireball, Lightning bolt (11, area) | 10 / 20 / 30 / 60 | 3d6 → 27 / 5d6 → 30 / 5d6 → 32 / 5d6 → 32 | 10.5 / 17.5 / 17.5 / 17.5 | 4d6 / 4d6 / 5d6 / 5d6 | 14 / 14 / 17.5 / 17.5 | 26 → 26, 20 → 20 |
| Flame (12, area) | 10 / 20 / 30 / 60 | 4d6 → 27 / 5d6 / 5d6 / 5d6 | 14 / 17.5 / 17.5 / 17.5 | 4d6 / 4d6 / 5d6 / 5d6 | 14 / 14 / 17.5 / 17.5 | 26 → 29 |
| Call lightning (11, area) | 10 / 20 / 30 / 60 | as Fireball | | 4d6 / 4d6 / 5d6 / 5d6+2 (Wi 16 / 18 / 20 / 25) | 14 / 14 / 17.5 / 19.5 | 20 → 20 |

Heals, the **trained priest** (Willpower 17 / 21 / 26 at 10 / 30 / 60): Mend today 1d3 (2.0) at every level, proposed 1d4 / 1d4 / 1d6 (2.5 / 2.5 / 3.5), 1 mana both; Cure wounds today 3d6 → 20 or 21 (10.5) at every level, proposed 3d6 / 3d6 / 3d6+3 (10.5 / 10.5 / 13.5), 2 mana both.

**Reading it**: at levels 10, 30, and 60 every old tier-2 and tier-3 spell is within a die of today, and at 30 and 60 the tier-3 spells are identical (today's growth had already stopped: `max_dice` 3 by caster level 24, the lift at its most by Intelligence 18). Level 20 is one die lower on Scorching ray, Fireball, Flame, and Lightning bolt (today's lift reached 60 % there with two stats summed; the ladder waits for Intelligence 19 at level 25 or so), and level 10 one die higher on Fireball and Lightning bolt. The spells that never grew (the cantrips, Blight, Cause wounds, Mend) now grow once or twice, which is the one deliberate buff of the migration. Mana moves on seven spells by one to three points, up, where a spell sits above its old tier's first level. Nothing moves by more than a die anywhere. **Rank 0 is the floor, not the game**: a caster who casts has a rank, and the next table is what they roll.

**With the school's rank** (§8.3's tables at the ranks §8.4's pace predicts: 5 at level 10, 8 at 30, 10 at 60; the wizard's school evocation, the priest's restoration, the witch's necromancy; no talents, no items):

| Spell | L 10, rank 5 | L 30, rank 8 | L 60, rank 10 | Over rank 0 at 60 |
|---|---|---|---|---|
| Ray of frost (1d8) | 1d9, 1s again: 5.4 | 2d9, 1s and 2s again: 11.6 | 3d10, 1s and 2s again: 18.9 | × 4.2 (a cantrip, 1 mana) |
| Magic missile | 3d5, 1s again: 10.2 | 4d5, 1s and 2s: 14.4 | 5d6+1, 1s and 2s: 21.8 | × 2.6 |
| Scorching ray | 3d7, 1s again: 13.3 | 5d7, 1s and 2s: 23.6 | 6d8, 1s and 2s: 31.5 | × 2.25 |
| Arcane bolt, Frost | 3d7, 1s again: 13.3 | 4d7+2, 1s and 2s: 20.9 | 5d8+2, 1s and 2s: 28.2 | × 2.25 |
| Fireball, Lightning bolt | 4d7, 1s again: 17.7 | 6d7, 1s and 2s: 28.3 | 7d8, 1s and 2s: 36.8 | × 2.1 |
| Call lightning | 4d7, 1s again: 17.7 | 6d7, 1s and 2s: 28.3 | 7d8+2, 1s and 2s: 38.8 | × 2.0 |
| Cause wounds (the witch) | 2d6+2, 1s again: 9.8 | 3d8+3, 1s and 2s: 18.8 | 3d8+6, 1s and 2s: 21.8 | × 2.0 |
| Blight (the witch) | 3d6+2, 1s again: 13.8 | 4d6+3, 1s and 2s: 19.7 | 4d6+7, 1s and 2s: 23.7 | × 1.75 |
| Cure wounds (the priest) | 3d7+1, 1s again: 14.3 | 4d7+2, 1s and 2s: 20.9 | 4d8+6, 1s and 2s: 27 | × 2.0 |
| Mend (the priest) | 1d5+1, 1s again: 4.4 | 2d5+2, 1s and 2s: 9.2 | 2d8+3, 1s and 2s: 13.5 | × 3.9 |

**Reading it**: a rank-10 school about doubles a spell over its ladder alone, which is what a rank-10 weapon skill does to a sword (long blades' +8 damage on 1d8+2); the small spells grow most in proportion because a die more and a size more mean more to one die than to five, so a master's cantrip is a weapon and a master's Mend a real heal, which is the right shape (a level-60 wizard should not be casting a 1-mana ray for 4.5). A level-22 burst written to 8d6+4 (32 at its top rung) rolls 10d8+4 rerolling 1s and 2s at evocation 10: 56.5; a level-37 one at 10d8 (45) rolls 12d10 the same way: 75.6. The two sources multiply: the content brings the spell, the rank brings the mastery, and neither is bounded by anything but its author (decision 16). The talents stack on top through the same hooks (§9's worked Fireball: 44.4 at 60 with Keen mind, Empower, and a Potent ring). **What is not measured here** and the probe must: the ranks the pace really reaches by level (§8.4 is an estimate from casts per day), and the casts a pool buys at each level with the ranks' mana off.

### 12.4 Mana and the probe

The costs are §5.3 and §5.8; a burst costs by its base radius. The pool check (`docs/dev/spells.md` "Balance" 1: "about ten single-target casts of its tier's base cost at every level") becomes **about ten casts of a spell of the caster's own level**: the wizard's 106 at level 12 against 11, 254 at 30 against 29, 502 at 60 against 56, the same numbers read against the level instead of the tier. The balance probe's "Spells" method (`world/balance_probe.rs`, `docs/balance.md` "Spells") reads `comfortable`, `Actor::spell_cap`, and `SpellDice::lifted` today; S7 rewrites it on `Actor::spell_roll` with the probe's caster given the school rank the pace predicts for their level (`World::set_rank_dev`; the rank computed from `[marks]` and the casts a pool buys, so §8.4's estimate is checked on the real curve), prints the rung and the rank beside the roll (`| Spell (level, res) | L | rung | school rank | roll | mana / pool |`), and re-runs the §7.3 cross-check paragraph against the rows above. It prints no casts-to-kill column: what a spell is worth against a monster is the content's question (higher-level spells), and the probe's job is the roll and the pool.

## 13. Determinism, saves, replays, fingerprints

- **Draws**: a cast draws its attack roll (if any) and its damage, one draw per die of the `Roll` in group order, one more per die the reroll hook sends back (a fixed rule, so the same state draws the same count); an area's one roll for everyone, in ascending actor id; a buff, hex, or cleanse draws nothing; a rider draws nothing. Where today's roll was one group and no mods, the proposed draws are the same count for the same rung, so a replay with no stat change over a rung verifies if the dice are the same. A wand's rung is one draw from the maker's stream when the stack is made (§7.4): every replay that makes a wand re-hashes (S5).
- **Saved state**: the spell levels (S1), the ladders (S2), the talents' and items' mods, rods (S3), and the menu (S4) save **nothing new**: rungs are derived from `Actor::effective_stats`, the talents, and the items at cast time; `Actor::spells` (spell ids) is unchanged as long as no spell file is added, removed, or renamed (none is: 26 files edited in place, path order kept). **S5 adds `Stack::wand_rung`** (`Option<u8>`, left out while `None`, hashed): **saves change** in S5 and only there; a save without it reads as rung 0. `FORMAT_VERSION` untouched (the integrator's per-release bump).
- **The content fingerprint changes** in S1 (every spell file: `tier` → `level`; `spells.toml`; `study.toml`), in S2 again (every spell file, the talents' and affixes' files, the three scrolls), in S3 (the rod effects), in S5 (the wand files): saves of 0.9.0 do not load until the integrator's per-release bump, as with any content change (`docs/dev/saves.md`). A Potent affix in a save reads its old value under the new effect: clamped at the read (§7.3).
- **Replays** that cast re-hash (`cast_tests`, `spell_talent_tests`, `spell_list_tests`, `affix_spell_tests`, `caster_style_tests`, `learn_tests` fixtures; the showcase's scenarios cast nothing in their golden walks, `tests/golden/manifest/`) and are re-made in S1 (the costs) and S2 (the dice). The state hash (`World::state_hash`) is unchanged in form.
- **Determinism tests** carried over: an attack can miss and critical, auto and area never; Evasion halves an area only; no draws without a roll; a cast at every rung draws the dice of that rung; the rung reached at a score boundary (15 → 16) changes the roll and nothing else; a reroll draws once more per die it sends back and never twice; a rider lands on every target harmed and on none missed; a wand of the same seed rolls the same rung.
- **School skills** (S8, §8): a `Training` per school on `Actor::weapon_skills`, saved and hashed by index as every skill's is; marks are counts, never a draw (`practise` draws nothing; `practise_cast` once a cast); the school's rank changes the dice drawn (a d7 for a d6, a die more, a reroll's extra draw), so **a replay that casts re-hashes when the caster's rank differs**, which it does for every class that starts with a school rank (the Spell attacks grant becomes a school grant with dice behind it): the cast fixtures are re-made in S8. `spell_lands` fires in actor id order for an area, each a lookup; an ability's status is laid as a rider is (no draw unless its `turns` are dice). **Saves change in S8**: the `spell_attack` row leaves `weapon_skills.toml`, so every skill after it (the styles) moves an index and a save's ranks before the slice read against the wrong rows; the Spell attacks training is dropped, not converted (§10.1); the content fingerprint changes (`weapon_skills.toml`, the classes, the talents, the weapon abilities). S9 (the page), S10 (the styles' arrays: the rows keep their places), and S11 (the abilities) save nothing new. The integrator schedules S5 and S8 as the two state-changing PRs of this design (at most three at once, `docs/dev/merging.md`).

## 14. Choices made along the way, and what is rejected

| Choice | Taken | Rejected, and why |
|---|---|---|
| The primitive | an **explicit ladder** per spell: every rung spelled out | **three universal rules** ([A] cells, [B] a die a rung, [C] a size step) that a spell opts into with a short line: shorter data, but the player must learn three rules and their interplay, and a bonus or a mixed rung needs a fourth; the ladder is the three rules' *output*, and the wiki prints it. The three shapes survive as the authoring templates (§6.4) |
| Breakpoints | **scores** (decision 2) | **modifiers**: one more concept (`floor((s − 10) / 2)`), and "Intelligence +3" is not on the character screen; also the to-hit's unit, so the two would mix in a row |
| Drivers | **stats only** for growth; caster level for learning | **both** (today): two tables per row; **caster level only**: the stats would do nothing for a caster, against the stat training design that just gave them paths |
| Tiers | **a level per spell, 0 to 70** (decision 9); its cost is the level, its learning is the level against the caster level | **tiers** (today): a stepped level with a ten-column table between the spell and the player; **a level and a separate cost** (two numbers a spell): Identify keeps `mana` as the one override, every other spell costs its level |
| Mana at a level | **the level, at least 1** | **a curve under the level** (today's table runs 7 % under the comfortable level by tier 9): one number is the rule; the shipped levels are placed by today's costs so nothing moves far (§5.8) |
| The casting gate | **none**: a known spell is cast at its mana (decisions 1, 10) | **the overreach surcharge** (today): a casting gate by a derived level; the pool is the gate, and the study is the difficulty |
| Talents and items | **six named hooks**, each one integer or list, each its own sentence (decision 5) | **rungs as the only unit** (round 1): rejected by the owner; **a free-form effect per talent**: special cases in the cast path with nothing shared; six hooks cover every effect the owner named and every talent that exists |
| A bound on a spell's strength | **none** (decision 16): no ceiling, no cap, no load-time check, no column; a spell's top rung is its author's call, read in the PR | round 2's **load-time check by level** ("cheap insurance"): struck by the owner, who wants no such rule anywhere; **a number in the wiki**: a per-level figure nobody applies |
| What grows a spell after its ladder | **the school's rank** (§8): marks by casting, ten ranks, each a sentence through the hooks | **nothing** (round 2): left a level-60 caster with a level-11 spell at its top rung and only new content to look to; the owner: too many casts; **caster level** (today): a derived number the player cannot see, and learning only now (decision 1) |
| Ranks per school or per spell | **per school**, seven tables (§8, §9) | **per spell**, a ten-step table each: a cold start for every new spell, 26 tables and growing, a fourth progression on the menu's one number; **both**: the two costs added; per-spell stays an additive slice if wanted (§9) |
| Where the school skills live | **rows of `weapon_skills.toml`**, `role = "school"`: one `WeaponSkills`, one `practise`, one screen row, one wiki generator, defaults, the dev rank action, abilities that hang on them for free | **a table of their own** (`school_skills.toml`, a `SchoolSkills` runtime): a copy of the weapon skills' machinery for a thing the owner said should "work like weapon skill" |
| What a school rank gives | **the hooks** (§7.1) as `Benefit` fields, so a rank is applied by the code that applies a talent | **rungs on the ladder**: capped by the ladder's length (a rank-10 evoker at the top rung gets nothing); **a flat percent**: the lift again |
| Marks for an area spell | **one practise a cast**, weighed by the strongest struck | **one per creature struck**: a burst trains three times as fast as a ray for the same mana |
| Spell attacks | **retired, merged** into every school's `to_hit` and `threat` (§10.1) | **kept**: a second bar a cast moves, on three spells of twenty-six, with the to-hit in one skill and the dice in another; **reworked into a general casting skill**: that is the school skill |
| The caster styles | **reworked into where you cast from** (beside, afar, in armour), trained by every spell (§10.2) | **kept as they are**: Sniper and Toucher are the spell's shape, not the player's choice, and train on three spells; **retired**: loses Pinning ray and the touches and leaves one style; **per school or per shape**: the spell's choice again |
| The Magic page's number | **6**, after Weapons; Quests, Talents, Story move to 7, 8, 9 | **9**, after Story: no renumbering, but the craft pages apart and the last page a thing read often; **a sub-screen of Weapons**: the owner asked for a page of its own |
| The Magic page's key | **none in play**: reached by its number from any info page; a key is a question for the owner (§16) | **a new `KeyCommand`** (`M`): out of this round's scope by the integrator's rule, though the hard rules allow a play-context opener |
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
| A talent on the rung unit | **none new** | "Mastery: a chosen spell a rung higher": a `choice = "spell"` kind does not exist; deferred (#196). Per-spell mastery ranks (§9) are deferred the same way |

## 15. Implementation plan

Twelve issues in five waves; each links to its section. "**State**" marks a PR that adds or changes saved state (S5 and S8). "**Fingerprint**" marks one that changes the content fingerprint. Hot files: `spells.toml`, `spell_def.rs`, `cast.rs`, `learn.rs`, `talent.rs`'s `TalentEffect`, `affix.rs`'s `Effect`, `wiki/spells.rs`, `docs/dev/spells.md`, spec §8 "Spells"; and for the school slices `weapon_skills.toml`, `content/weapon_skill.rs`'s `Benefit`, `ui/info_tabs.rs`'s `PAGES`, `screen_snapshots.rs`, `talents.toml`'s systems; the false-conflict trick (`docs/dev/merging.md`) applies. Sizes: S a day, M two or three, L a week.

| Wave | Issue | What | Size | State / fingerprint | Depends on |
|---|---|---|---|---|---|
| 1 | **S0 The `Roll` type** (§6.2) | `dice::Roll` (groups + bonus, text, `roll(rng, reroll)`, `min`, `max`, `mean_x2`, `grow`, `more`, `plus`, `From<Dice>`, serde); `combat::Attack::damage: Roll` and `reflect::Shot::dice: Roll` with every builder converting; no behaviour change (the hash test and every replay verify as before); `docs/dev/combat.md` one line | S | none / none | — |
| 1 | **S1 Spell levels** (§5) | `level` for `tier` on the 26 spell files (§5.8; `tier` refused naming `level`); `spells.toml` keeps `[shapes]` and `[resolution]` (its `cost` column; the `damage` column goes with the cap and nothing replaces it), loses `[tiers]` and `overreach_cost`; `SpellRules` as §6.5; mana = `max(1, level)`, no surcharge; `study.toml` (§5.2) and `learn.rs` in levels, `known_levels`; Overreach → `study { overreach_percent }`; `wand.rs`: `charges` required (the seven files); `spells_by_level`; the class test (§5.6); the `spells` and `progress` dumps; the menu row "level 11, 26 mana" for "tier 3" (the dice come in S4); `wiki/spells.rs` (`tiers` and `wand_charges` out, Level column, sort), `learning.rs` (the samples); `magic.md` "Spell levels", "Mana", "Caster level", "Learning", `scrolls.md`, `classes.md`, `features.md` lines; the spec's **Tiers**, **Cost**, **Learning** bullets and `08i`'s class lists; `README.toml`, `study.toml`, `actions.toml`, the talents' README; `docs/dev/spells.md`; `the_shipped_tier_table_reads_as_the_design_has_it` deleted with the table, and the load-time cap check deleted with it (decision 16: no check on how much a spell does, in this slice or any); replays re-made (the costs) | M | none / **fingerprint** | — |
| 2 | **S2 Ladders and hooks** (§4, §6, §7.1 to §7.4 but the wand rung, §12.1, §12.2 but the rods, §11's generator and dumps) | `content/spell_ladder.rs`; `SpellData::{damage: Roll, grows, aim}`; the retired fields refused with a message naming the new one; every `capped` gone; `world/spell_mods.rs` (`SpellMods`, `Actor::spell_mods`) and `world/spell_rung.rs`; the cast path on rungs, mods, rerolls, riders; `TalentEffect::SpellMod` (`TalentMods::spell`, the scroll and wand rungs among its fields); Empower, Archmage, Keen mind, School focus, Scroll adept, Wand mastery re-pointed, Sure hand, Brimming, Numbing cold added (`assets/data/talents/`, by group); `Effect::{SpellRungs, SpellBonus, SpellSides, SpellDice, SpellReroll, WandRungs}` with Potent's and of Wands' files and the Potent read clamp; `Wand::rungs` (the range; a zap at `min` until S5); `ScrollSpell::rungs` and the three scrolls folded; the 26 spell files' ladders; `README.toml`, `spells.toml` comments; `wiki/spells.rs` (`list`, `does`, `reach`, `grows_with`, `resolutions`, `wands`, `scrolls`' "Casts at"); the dump lines; `docs/dev/spells.md`, `scrolls.md`, `magic-items.md`, `progression.md`; the spec bullets (**Ladders**, **Hooks**, **The lift** out); the tests of §6.3 and §13; replays re-made | L | none / **fingerprint** | S0, S1 |
| 3 | **S3 Rods** (§7.5, §12.2) | `RodEffect::grows` on the same parser; `per_levels`, `max_range`, `up_to_caster_level` gone, `up_to_level` in; the four rod files; `rods.md`; `docs/dev/spells.md` "Rods"; `rod_tests` | S | none / fingerprint | S2 |
| 3 | **S4 The menu's numbers** (§11) | the abilities menu's Spells rows on `Actor::spell_roll` (`PerceptionView::own_spell_roll`: the text, the damage type or "healed"), the description's reach clause; JSON `spells[].roll`; screen snapshots `abilities-menu-spells` (changed), `abilities-menu-spells-grown` (new), `wand-description` (changed: the dice line, at the wand's `min` until S5); `docs/dev/screens.md` one line | S | none / none | S2 |
| 3 | **S5 Wand strength** (§7.4) | `Stack::wand_rung` drawn at `Stack::new` from the maker's stream (loot, shops, level piles, kits), saved and hashed; `World::wand_cast` at it; the description and aiming hint; the `inventory` and `actors` dumps' `(rung N)`; `wands.md` "Strength"; `docs/dev/saves.md`, `docs/dev/magic-items.md`; `zap_tests` (the same seed the same rung; a save without it reads 0); replays that make wands re-made | S | **state (saves change)** / fingerprint (the wand files' `rungs`) | S2 |
| 3 | **S6 The prose** (§11, §3.1) | `wiki/src/magic.md` ("The spells", "How a spell grows", "Shapes", "Wands", "Rods", "Spellcraft talents" out), `scrolls.md`, `magic-items.md`, `stats.md`; the spec's prose paragraphs; `docs/design/README.md`'s entry | S | none / none | S2 |
| 3 | **S8 School skills: data and rules** (§8.1 to §8.4, §8.6, §8.7, §10.1) | `Role::School` and `school` on the row; `Benefit`'s spell fields (`reroll`, `sides`, `dice`, `bonus`, `cells`, `degrees`, `mana`; `speed` allowed for schools) checked per role; `Benefit::spell_mods()`; the seven rows of §8.3 and the three defaults; the `spell_attack` row and `Role::Spell` gone (a `role = "spell"` refused naming `school`), the classes' grants re-pointed (§8.2); `Actor::spell_skills(content, aim)` returning the school (the styles' `aim` arms come in S10: until then `Afar` for a ray or bolt, `Beside` for a touch, `Armoured` as today); `Actor::spell_mods` adding the school's benefit, `spell_mana_percent` its `mana`, `World::cast_price` its `speed`; `World::practise_cast` with the `Outcome` the kinds fill, replacing `spell_practised`'s practise loop and `practise_in_armour`; the news lines; the `schools` dump layer and JSON, `progress`'s marks line; `rank:evocation:5` (the existing action, a test that it works on a school); the `schools` system and Schooled, Devotee, Magister (`choice = "school_skill"`, `school_marks_percent`, the wizard's table at 45), Early mastery's wording; `docs/dev/magic-skills.md` (new), one line in `weapon-skills.md`, `spells.md`, `progression.md`, CLAUDE.md; the spec's Weapon skills and Talents bullets; `wiki/src/schools.md` with `school-skills.md` and `school-benefits.md` (`xtask/src/wiki/schools.rs`, one line in `generate()`), `weapon-skills.md`'s Spell attacks paragraph out, `classes.md`'s column, `SUMMARY.md`; tests: `content/weapon_skill.rs` (the rows load, the fields refused per role, a missing school refused, `rank_ten_holds_the_family_tables` widened to the schools), `world/school_skill_tests.rs` (marks per kind of cast, the strongest struck weighs an area, a miss is an attempt, a scroll and a wand train nothing, a rank changes the roll and the mana and the cast's time, the defaults, the three talents, the dump), the Weapons page's `Section::Spells` gone (its snapshots), replays re-made (the dice) | L | **state (saves change)** / **fingerprint** | S2 |
| 4 | **S9 The Magic page** (§11) | `ui/magic_screen.rs`, `ScreenName::Magic`, `PAGES` entry at 6 and `WIDTH`, the scroll, the sections, the header, the hint lines; `--screen magic`; the caster styles off the Weapons page; the snapshots `magic-screen`, `-trained`, `-armoured`, `-none`, `-short` and every page snapshot's tab line; `docs/dev/screens.md`; `wiki/src/schools.md`'s page paragraph, `controls.md`'s table, `quests.md`, `talents.md`, `background.md`, `weapon-skills.md`'s "page 5" paragraph; CLAUDE.md `--screen` | M | none / none | S4, S8 |
| 4 | **S10 The caster styles** (§10.2) | `Aim` decided once in `cast_spell` (`Beside`, `Afar`, `Armoured`, `None`), `Actor::spell_skills` on it, the grips' README lines; Sniper's `cells` and Toucher's `bonus` (allowed for the caster grips, into `SpellMods` after the school's); Battle caster's full marks through `practise_cast`; `own_styles_in_use` marking Battle caster in hindering armour; `caster_style_tests` rewritten; `fighting-styles.md`'s "Caster styles" moved to `schools.md` with `caster-style-benefits.md` and `caster-style-abilities.md`; the spec's Fighting styles bullet; `docs/dev/magic-skills.md` | M | none / fingerprint (the three rows' arrays) | S8 |
| 4 | **S11 School abilities** (§8.5) | `Trigger::SpellLands` raised in `World::spell_landed` (the one place, which also fills the `Outcome`); the loader's "a weapon skill or a style" widened to a school; Thunderstruck, Wasting, Aegis, Lulling (`data/weapon_abilities/`); the menu rows; `school-abilities.md`; `weapon_abilities/README.toml`; `docs/dev/weapon-abilities.md`'s trigger table; `weapon_ability_tests` (each on a hand-made cast: an area lands on each struck, a buff on its caster, a miss fires nothing, no draws without a rank) | S | none / fingerprint | S8 |
| 5 | **S7 The probe** (§12.4) | the balance probe's spell method on `Actor::spell_roll`, the caster at the rank the pace predicts (`set_rank_dev`), the rung and rank columns, the pool check by level; `docs/balance.md` "Spells" re-run and the §7.3 paragraph rewritten; `docs/dev/spells.md` "Balance"; §8.4's pace checked and the arrays or the curve tuned if it is off | M | none / none | S2, S3, S8 |

**Why this order**: S0 and S1 first and independent: S0 is a type change across `combat` with no spell in it, S1 touches the spell files' `tier` line and the cost path and nothing the ladders touch, so both merge clean before the hot spell files move; the integrator may run them side by side. S2 is one PR because the old hooks (`spell_lift`, `SpellDice`, `max_range`) and the talents that feed them cannot be half-replaced: every spell file, the generator, and the talents' effects change in the same commit or the content fails to load and the wiki fails to build. S3, S4, S5, S6, S8 beside each other after S2 (S8 needs `SpellMods` and `Actor::spell_mods` to exist; it touches `weapon_skills.toml`, the classes, and `cast.rs`'s practise lines, which none of the others do; S5 and S8 are the two state changes and may run together). S9, S10, S11 beside each other after S8 (S9 also after S4 for the rolls it shows): a screen, three style rows and an `Aim`, four ability files and a trigger. S7 last, on everything.

**Talents and class abilities** (the feature template's section, per slice): S0 none (a type). S1 re-points Overreach from the casting surcharge to the study check and re-words Arcane training, Attuned, and *of the Adept* as learning (§7.2); no new `[[system]]` tag. S2 re-points Empower, Archmage, Keen mind, School focus, Scroll adept, Wand mastery, Potent, *of Wands* onto the hooks and adds Sure hand, Brimming, Numbing cold (system `spells`; Brimming `mana` too), so every hook ships with a user (§7.2, §7.3); no new `[[system]]` tag (the `spells` system's bar is unchanged). S3 Channeler unchanged, none new. S4, S6, S7 none. S5 Wand mastery and *of Wands* unchanged (they add to the rolled rung), none new. **S8 adds the system `schools`** (`[[system]] id = "schools"`, after `spells`) with its coverage bar carried by Schooled and Devotee (two talents, `min_talents` met) and the wizard's class ability Magister (§8.6), and re-words Early mastery; Stylist unchanged (a caster style is still a style). S9 none (a screen). S10 none new: Stylist's choice still names the three caster styles, and the Weaponcraft talents that read a style's rank read them as before. S11 the four school abilities carry `systems = ["weapon_abilities", "schools", "spells", "status"]`; no talent (an ability is not a talent; Early mastery already reaches them).

## 16. Decisions (owner, 2026-10-29 and 2026-10-30)

The owner's answers to round 1's fifteen questions (decisions 1 to 15) and comments on round 2 (16 and 17), verbatim, with the reading taken where an answer needed one. The body above is revised to them; round 1's tiers, its "rungs are the one external unit", its "your spell grows" line, its Scroll adept rung on own spells, its fixed wand rungs, and its caster-level gate on rods are gone; round 2's ceiling line and its casts-to-kill reasoning are gone with round 3.

| # | Question (round 1) | Answer | Taken as |
|---|---|---|---|
| 1 | Retire caster level from spell growth: it decides only which tiers are comfortable (mana, overreach), the study check, and the class tables' grants? | "yes" | caster level grows nothing (§4.4); with decisions 9 and 10 it decides **only the study and the class tables' check**: the mana part went with the tiers and the overreach surcharge |
| 2 | Breakpoints on scores (Intelligence 16), not modifiers? | "mechanically the same thing. but sure, actual scores work" | scores (§4.3): the number on the character screen |
| 3 | One stat per ladder, by school, with Call lightning on Willpower as the druid's spell? | "yes, spell decides, not caster's class" | one stat, the spell's own, written in its file; the school table is a convention for authors (§6.4), Call lightning names Willpower |
| 4 | Criticals unclamped: an attack or touch spell's critical rolls the dice twice, as a blow's? | "yes. no cap" | rolled twice, nothing held: no cap anywhere at run time, on criticals or on what talents and items add (§4.3, §7.1). Round 2 kept an author's check on the ladder; round 3 strikes it (decision 16): no cap anywhere at all |
| 5 | Talents and items as rungs, never flat dice or cells? | "disagree. talents and items can do whatever their author wishes. Flat damage, different dice, a rider. This is not a concern of the magic progression per-se. Talents primarily tied to spell progression can work that way. Regardless, i want the system to mechanically be able to support riders from talents." | **six named hooks** (rung, dice, roll, rider, reach, degrees; §7.1) so any author-written effect fits without a special case; Empower, Archmage, Scroll adept, Wand mastery, *of Wands* stay rungs (talents about the ladder); Keen mind grows dice, Potent adds a bonus, School focus and *of Farcasting* and *of Lingering* keep their cells and degrees; three new talents (Sure hand, Brimming, Numbing cold) give the reroll, dice, and rider hooks a shipped user (§7.2, §7.3). The framing applied: a d7 is a die, a reroll is a rule |
| 6 | Cantrips grow one rung (Ray of frost 1d8, Spark 1d6)? | "fine" | kept (§12.1) |
| 7 | Blight, Cause wounds, and Mend get ladders (Willpower), where today they never grow? | "yes" | kept (§12.1) |
| 8 | Hexes last by Presence alone (today's quarter of Willpower dropped)? | "yes" | kept (§12.2) |
| 9 | The tier ceiling stays as an author's check on the top rung, re-valued, and leaves the player wiki? | "get rid of concept of spell tier. Assign spells a level between 0-70. (tier is just a cringe stepped spell level system anyhow). let's bypass that extra step. players are familiar with this form wow etc." | **tiers abolished** (§5): `level` 0 to 70 on every spell, its base mana and its certain-learning caster level in one number; every tier column re-expressed as a one-line rule of the level (§5.2); the 26 spells levelled (§5.8); the fingerprint changes, saves do not (§5.9). Round 2 kept the tier ceiling as a line of the level; round 3 strikes it (decision 16) |
| 10 | Rods reach by Willpower ladders (digging 3 to 7), and sleep's and terror's "no stronger than your magic" gate stays on caster level? | "sure. just get rid of the your casterlevel gate. caster level is exclusively a learning concept. items are good, because they do not require learning." | Willpower ladders kept; the caster-level gate **removed** from rods and from everything else that is not learning: the overreach mana surcharge goes (§4.4, §5.3), wands and scrolls read no caster level (§7.4), a rod's sleep and terror take creatures up to the rod's own `up_to_level` (20; §7.5) |
| 11 | One line when a known spell climbs a rung ("Your fireball burns hotter."), and the menu's description says what a spell grows with, in words? | "uhm, no. Just put the damage number to the menu as you would with a weapon." | **no line**; the abilities menu's Spells rows show the roll as the caster would make it now, the damage type, the mana, and "aimed" (`Fireball   4d6 fire, 26 mana, aimed`), the description the reach in cells (§11); the character screen unchanged; a wand's description shows its dice with its type (§7.4) |
| 12 | Scroll adept's rung on the wizard's own spells too, not only on scrolls read? | "no. why would we introduce a talent that is mandatory flat damage increase on all spells and call it scroll adept." | Scroll adept is **scrolls only**: the spell scrolls read cast a rung higher, reading in half the time (§7.2); a spell scroll casts at its kind's `rungs` (the three old ones at 1 / 2 / 0), fixed, with no talent needed to make it worth reading (§7.4) |
| 13 | Wands cast at rung 0 by default plus their `rungs`, Wand mastery's 1, and of Wands' 1 to 2; the three old scrolls become spell scrolls at rungs 1 / 2 / 0? | "wands should randomize their effective rung at loot generation." | a wand's rung is **drawn when the stack is made**, evenly in its kind's `rungs` range (0 to 2 on the seven), kept on the stack (`Stack::wand_rung`, **saves change** in S5), shown with the type as the spell's dice; Wand mastery and *of Wands* add to it (§7.4). The scrolls' part of the question stands (decision 12) |
| 14 | The rung grid as a convention (dice at 13 / 16 / 19 / 22 / 25; cells at 14 / 18 / 22 / 26; degrees at 14 / 18), stated in `README.toml`, not checked? | "sure" | kept (§6.4) |
| 15 | Mixed dice used once in the shipped list (Scorching ray's top rung `3d6+1d10`), else plain? | "whatever" | kept as written (§12.1); an author may use mixed dice where a spell's fiction wants them |
| 16 | Round 2's §4.6, "The ceiling: an author's check by level, not a rule" (a load-time check that a spell's top rung stays under a line of its level) | "this one discusses a rule of 10d6 fireballs. We should purge that rule from all memories. I don't want it ever appearing anywhere." | **no bound on a spell's strength anywhere**: the section is deleted, not reworded; the data it named, the load-time check (§6.3), the column of §12.1 that measured rungs against it, and every rule of the kind in the summary, the goals (G11), §5.2, §6.4, §6.5, §14, and §15 are gone; the rule the comment names (that a spell must stay under some size) does not exist and is not to be added: a spell is as strong as its author writes it; the magic design's own text is being edited apart from this document and is not touched here. What a spell does at any rung is its author's call, read in the PR; the wiki prints the row. The rounds' earlier reasoning about an upper limit is not referred to again |
| 17 | Round 2's §4.5, "What keeps spells relevant at level 30 to 60" (higher-level content, rungs that run on, the class talents; measured in casts to kill the median monster) | "discusses how we need 6 casts to dispatch a median monster. This is a little too much in my opinion. Let's introduce 'weapon skills' for spell schools (work like weapon skill, gain marks by casting from that school, 10 ranks) that give a leg up when casting from that school. Another thing we could do, is that each spell has its own hand authored 10-mark progression (similar to weapon skill). These new skills (along with 'Spells' section currently on weapon skill page) should probably be moved to a separate 'Magic' page (scrollable) since they will not neatly fit the weapon skills page. 'Spell attacks' skill also comes under the scope of this redesign. Also the spellcasting related fighting styles. Take a look at the existing fighting styles and spell attacks weapon skill, and deliberate how they mesh with what we are designing (i don't think it's that good match yet). another design round please." | **School skills** (§8): seven `role = "school"` rows of `weapon_skills.toml`, ten ranks on the missile curve, marks once per spell paid in mana (an attempt, a success when it lands, weighed by the strongest creature touched: §8.4), each rank's leg up written through the hooks of §7.1 (seven hand-written tables, §8.3; four school abilities, §8.5; the `schools` system with Schooled, Devotee, Magister, §8.6). **Per-spell mastery weighed and not taken** (§9: a cold start for every new spell, 26 tables and growing, a fourth source on the menu's one number; the door kept open as an additive slice). **Spell attacks retired, merged** into every school's to-hit and threat rows (§10.1: it trained on three of twenty-six spells). **The caster styles reworked** into where one casts from (beside: Toucher; farther: Sniper; in hindering armour: Battle caster), every spell paid in mana training the style it is cast in (§10.2). **The Magic page**, page 6, scrollable, with the schools, the caster styles, and the spells known; the Spells section and the caster styles leave the Weapons page (§11). §4.5 rewritten without casts to kill: the school's rank and higher-level spells are what grows a spell past its ladder; §12.3's casts columns dropped and a rank table added; the probe (S7) prints rolls and the pool, never casts to kill (§12.4). Four slices added (S8 to S11, §15) |

**Questions for round 3** (not decisions; the body assumes the first answer of each): (a) The Magic page has no opening key in play (the integrator's rule for this round: no new rebindable `KeyCommand`); does the owner want one (`M`, as `W` opens Weapons, which CLAUDE.md's hard rules allow for a play-context command), or is the page by number from any info page enough? (b) The missile curve for the schools (§8.4's pace: rank 5 in the first fortnight, rank 10 toward level 60), or a quicker `spell` curve of its own, since a caster casts fewer spells than a fighter swings blows and each costs mana? The probe (S7) measures it either way. (c) The three defaults between schools (evocation ↔ transmutation, necromancy ↔ restoration, abjuration ↔ enchantment, each 4), or none?

Also folded in: the owner's framing for round 2 (the preamble), applied to the hooks (a reroll, a die size), the dice (`MAX_SIDES` 20, a d7 and a d9 legal), and the authoring templates (§6.4 [C]); the numbers of §12.3 were re-run with §5.8's levels and carry the mana before and after, and re-run again for round 3 with §8.3's school tables.
