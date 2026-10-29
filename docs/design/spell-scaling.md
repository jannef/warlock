# Warlock — Spell scaling: a ladder in every spell's own words

Design document for the owner's review (2026-10-29). Written against `origin/main` at 7833b9c2 (the stat training design, #721). Nothing here is implemented; §11 cuts it into issues. It audits the player wiki's magic pages (`wiki/src/magic.md`, `scrolls.md`, `mana.md`, `magic-items.md`, the generated `spell-list.md`, `spell-tiers.md`, `wands.md`, `rods.md`, `class-levels.md`) against the rules behind them (`docs/dev/spells.md`, spec §8 "Spells", `assets/data/spells.toml`, `assets/data/spells/*.toml`, `content/spell_def.rs`, `content/spell_scales.rs`, `world/spell_scale.rs`, `world/cast.rs`, `world/caster_level.rs`, the talents and affixes that touch spells), and rebuilds how a spell grows. Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (verbatim): *"auditor reads the magic documentation from wiki, and makes sure the documentation is as clear as possible. It also identifies mechanics that are too complicated to be put in simple words. Then it makes a change proposal, that aims to streamline the system. Just fucking look at it: `cap: Intelligence 10 %, Willpower 5 % of the cap a point; reach: Willpower 50 % of a cell a point`. Damage scaling must be entirely per-spell in its spell text. It must be clearly written. Tools we have at our disposal: this is not played with real dice. 2d6 spell can scale to 2d7 -- dice can change; we can add extra dice; we can add static bonus; not all damage die must be same. Some ideas, let's make few: one stat increases range, duration, radius etc; one adds damage per dice based breakpoints; one increases the size of dice, based on breakpoints. If you choose to go with these universal rules, you just need to write 3 or more rules, and spell becomes very simple unless we choose to add riders."* Earlier: the magic mechanics feel *"too complicated to be fun ... not complicated, but fucking janky"*.

**Addendum** (the same day, verbatim): *"i am not against getting rid of concept of caster level as spell scaling. perhaps that is just learning difficulty thing?"* §4.4 weighs it and recommends it.

## Contents

1. Summary
2. What exists today
3. The audit: the wiki read as a player
4. The proposal: ladders
5. The data format and its checks
6. Talents, items, scrolls, wands, rods, traps
7. What the player sees; the wiki, the spec, the docs
8. Migration: every spell, with before and after
9. Determinism, saves, replays, fingerprints
10. Choices made along the way, and what is rejected
11. Implementation plan
12. Open questions, each with a recommended answer

## 1. Summary

- **One idea replaces six.** A spell's growth is a **ladder** written in its own file and printed verbatim in its wiki row: `damage = "intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2"` reads *"Fireball: 2d6 fire; with Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2."* Your score reaches the number, you roll that. The last rung is the most the spell ever does, for anyone. Nothing else is computed: no cap, no percent of a cap per modifier point, no `max_lift` by tier, no Keen mind widening the most, no half-rounding, no dice "room" formula (§4.2, §4.3).
- **Any dice shape on any rung** (the owner's tools): more dice (`3d6` → `4d6`), bigger dice (`1d6` → `1d8`), a static bonus (`5d6+2`), mixed dice (`3d6+1d10`). A new `Roll` type (a sum of dice groups and a bonus) carries them; `Dice` stays for everything else (§5.2).
- **Breakpoints are stat scores**, not modifiers: "Intelligence 16" is the number on the character screen (§4.3). One stat per ladder. Reach (`range`, `length`, `radius`) and a buff's or hex's `degrees` are ladders of the same kind, in whole cells and whole degrees: `range = "willpower 14 11, 18 12"` (§4.3).
- **Caster level stops growing spells** (the addendum, §4.4). It keeps its other jobs, which are all about *how hard a spell is*: which tier is comfortable (mana and overreach), the study check, what a class table grants when. `per_levels`, `max_dice`, the scroll's cast level, and the wand's caster level go. What keeps spells relevant at level 30 to 60 is what the combat progression design already said no scaling replaces: the tier 4 to 9 spells, plus stat rungs that run to score 25 and beyond, and the class talents that give rungs (§4.5).
- **Rungs are the one unit for everything outside the spell** (§6): Empower is *a rung up*, Archmage *two rungs*, Keen mind *a rung up for its school*, Potent items *one or two rungs*, Farcasting *rungs up the reach ladder*, Lingering *rungs up the degrees ladder*, Wand mastery *a rung for wands*. Past the top rung a rung does nothing. The old scrolls of fireball, lightning, and frost become plain spell scrolls that read at a rung (`rungs = 2` for lightning's 4d6), so every scroll is one kind of thing.
- **Reading one spell row today takes 17 concepts** for the damage cell and 6 for the reach cell; the proposal takes 3 and 2 (§3.3).
- **Balance holds** (§8.3): the ladders are set so the trained wizard's and priest's spells land within a die of today's at levels 10, 30, and 60 (Fireball 4d6 / 5d6 / 5d6 against today's 3d6 / 5d6 / 5d6; Scorching ray 3d6 / 4d6 / 4d6 against 3d6 / 4d6 / 4d6; Cure wounds 3d6 / 3d6 / 3d6+3 against 3d6 / 3d6 / 3d6). Where it shifts, it is said: the two attack cantrips, Blight, Cause wounds, and Mend gain one or two rungs where today they never grow.
- **Saves do not change**; the content fingerprint does (every spell file changes), as with any content change, covered by the release's bump; `FORMAT_VERSION` untouched (§9).
- **Six slices** (§11): the `Roll` type alone first; then the ladders with every spell migrated, the talents and items re-pointed, and the wiki generator in one PR; then rods, the prose, and the rung message beside each other; the balance probe last.

## 2. What exists today

Verified line by line against the code (`docs/dev/spells.md` is current):

| Piece | Today |
|---|---|
| A spell's damage | `[spell.damage] dice = "2d6", per_levels = 4, max_dice = 3, bonus = 0`: the dice at the tier's **comfortable caster level**, a die more per `per_levels` caster levels above it, never more than `max_dice` (`SpellDice::at`). A heal's `heal` table the same. |
| Caster level | derived: character level × the class's `casting` percent (100 wizard, witch, priest, druid; 75 bard, aristocrat; 50 paladin, ranger, monk; 25 the rest), plus Arcane training (+1 per 5 levels), plus *of the Adept* items (1 to 3), half again with Attuned; at least 1 with any mana (`Actor::caster_level`). A level-60 wizard casts at 72. It feeds learning (`tier_of`), overreach cost, dice growth, the scroll's cast level, the wand's level, and the rods' reach and sleep/terror gate. |
| The cap | `ceiling[tier] × resolution.damage %` (tier 3 area: 24 × 80 % = 19). A load-time check refuses a spell written over it; every roll is held to it at run time (`spell::capped`), criticals and talents included. |
| The lift (#556) | `[spell.scales] damage = { intelligence = 10, willpower = 5 }`: Σ weight × modifier is the percent the cap is lifted, never below 0, at most the tier's `max_lift` (40, 50, 60, 70, then 80) plus Keen mind's 10 for its school and *Potent* items' 4 to 15; the lifted cap is `cap × (100 + lift) / 100` rounded down; the dice gain `(lifted − bonus) / sides − (cap − bonus) / sides` more (`SpellDice::lifted`). A scroll read or a wand used is unlifted. |
| Reach | `range` (`length`, `radius`) grown by the `reach` scales (default Willpower 50 %: a cell per 2 modifier points, a half rounded down), plus School focus's cell and *of Farcasting*'s cells on the range or length, never past `max_range` (`max_length`, `max_radius`). A burst costs by its own `radius` however far it spreads. |
| Duration | a buff's or hex's `severity` grown by the `duration` scales (default Presence 50 %; the hexes Presence 50 and Willpower 25) plus *of Lingering*'s degrees, up to the status's `max` (3 for Blessed, Hexed, Weakened, Sluggish, Drowsy, Hasted, Barkskin, Lit; 6 for Webbed). |
| To-hit | the `to_hit` scales (default Dexterity 100 %), a half toward zero, plus *of the Magus*; threat 22 less *Baleful*; the Spell attacks skill and a caster style. |
| Talents on dice | Scroll adept +1 to every die, Empower a die more, Archmage +2 to every die (`Dice::empowered`), then held to the lifted cap. They work on spell scrolls read too, held to the taught spell's cap or the scroll's own most. |
| Scrolls, wands, rods | a spell scroll casts at `[spell.scroll] level` (default the comfortable level), free of mana; the three older scrolls (fireball 3d6, lightning 4d6, frost 2d6) are bursts and bolts of their own, uncapped. A wand casts at its own `caster_level` (default the comfortable level) plus Wand mastery's 2 and *of Wands*' 1 to 2. A rod's reach is `range + caster level / per_levels` up to `max_range`; sleep and terror take creatures of level at most the caster level. |
| Shown in the game | the abilities menu's "tier 3, 26 mana, aimed" and the aiming mode's reach. Nothing of dice, caps, or lifts. |
| Shown in the wiki | the row quoted in the request; "The lifted cap" (a 20-line section with a worked example); the tier table's "Most lift" column; `class-levels.md` and `talents.md` repeat the row text for every granted spell. |

## 3. The audit: the wiki read as a player

### 3.1 Sentences and cells that are unclear, with rewrites

Each row is a sentence or a cell a player meets, why it fails, and what the proposal prints instead (generated from the data where it is a table).

| # | Where | Today | Why it fails | Rewrite |
|---|---|---|---|---|
| 1 | `spell-list.md`, "Grows with" | `cap: Intelligence 10 %, Willpower 5 % of the cap a point; reach: Willpower 50 % of a cell a point` | four numbers, three units (percent of a cap, modifier points, cells), and none of them is a number the player rolls or a score they have | `Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2; range Willpower 14: 11, 18: 12` |
| 2 | `spell-list.md`, "What it does" | `2d6, a die more per 4 caster levels above 12, to 3d6 fire damage ... (never more than 19; your stats may lift it to 32, more with Keen mind)` | asks the reader to compute their caster level, then holds up three numbers (19, 32, "more") of which the one that matters, what they roll, is absent | `2d6 fire damage` (the ladder is the next column; its last rung is the most) |
| 3 | `spell-list.md`, "Reach" | `range 10 (to 12), radius 2` | "to 12" is a limit with no way to know when it is reached | `range 10 (Willpower 14: 11, 18: 12), radius 2` |
| 4 | `spell-list.md`, Frost | `it freezes the water it reaches into ice 10 cm thick (never more than 14; your stats may lift it to 22, more with Keen mind)` | the owner's example: the ice and the cap share one parenthesis as if 14 and 22 were centimetres | `2d6 cold damage; it freezes the water it reaches into ice 10 cm thick` |
| 5 | `magic.md` "Tiers" | "Each tier also has a **damage ceiling**: the most a single-target spell of that tier can do in one cast at its highest roll, and a **most lift**: how far a caster's stats may raise a spell's ceiling" | two tier-wide limits a player never applies: each spell's row already says what it does; the ceiling is an author's check (§4.6) | delete; the tier table keeps comfortable caster level, base mana, and wand charges |
| 6 | `magic.md` "The lifted cap" | the whole section: "each by a percent of the cap per point of its modifier ... never more than the tier's most lift (40 % at tier 0 ...) ... rolls as many more dice as the lift makes room for: a die more for each whole die's worth the cap rose (for a d6 spell, the lifted cap divided by 6, rounded down, less the unlifted cap divided by 6, rounded down)" | a six-step computation with two roundings, explained with a worked example because it cannot be said plainly | delete; replaced by "How a spell grows" (§7): *"Each spell's row lists its ladder: a stat and the scores at which the spell changes. Your score reaches the number, you cast that rung. The last rung is the most the spell does, for anyone."* |
| 7 | `magic.md` "Shapes" | "Your Willpower lets a spell reach farther: a cell more of range or length, and a cell more of a burst's radius, for every 2 points of its modifier, but never past the spell's own limit (the "to" in the list). (A spell may name other stats for its reach: its "Grows with" says so; none does yet.)" | modifier arithmetic, a parenthetical about a thing that does not exist, and a limit told as "to" | *"How far a spell reaches is in its row, and so is how far your Willpower takes it."* |
| 8 | `magic.md` "Shapes" | "The hexes also grow with your Willpower: their degrees are half your Presence modifier plus a quarter of your Willpower modifier, added up and rounded to the nearest, a half down" | two stats, two fractions, a rounding rule | *"A hex's row says how long your Presence makes it last."* (one stat, §12 Q8) |
| 9 | `magic.md` "Damage and its cap" | "Spell damage is always rolled dice, with a small fixed bonus at most a quarter of the ceiling, and rarely any. A spell's dice may grow with your caster level (a die more every few levels above its tier's comfortable level), up to its own most." | the quarter rule is an author's rule; "every few levels" is vague because the real number is per spell and depends on a derived level | *"Spell damage is rolled dice, as written in the spell's row; what grows it is its ladder, nothing else."* |
| 10 | `magic.md` "Damage and its cap" | "Scroll adept adds 1 to every die, Empower rolls a die more, Archmage adds 2 to every die; they bring a spell to its most sooner, and nothing lifts it past" | three different arithmetic effects, each then clamped | *"Scroll adept and Empower take your spells a rung up their ladder; Archmage two. Past the last rung nothing helps."* |
| 11 | `magic.md` "Caster level" | "Your caster level is how strong a caster you are." | under the proposal it is how *learned* a caster you are: it never changes what a spell does | *"Your caster level is how hard a spell you can handle: which tiers are comfortable to learn and cast. It never changes what a spell does."* |
| 12 | `magic.md` "Wands" | "cast at the **wand's own caster level**, never yours ... Its damage still stops at the spell's cap." | a wand's level is invisible and its only effect was dice growth | *"A wand casts its spell at the foot of its ladder, however little magic you have; Wand mastery takes it a rung up."* |
| 13 | `magic.md` "Rods" | "its reach ... grows by a cell per so many caster levels, up to its limit" | "so many" hides a per-rod number that the table then states differently ("3 + 1 per 3 caster levels, at most 12") | *"A rod's row says how far it reaches and how far your Willpower takes it."* |
| 14 | `scrolls.md` "Spell scrolls" | "at the scroll's own caster level (its tier's comfortable level)" | a scroll has no level a player can see | *"as written in the spell's row, at the foot of its ladder: your stats do not grow a scroll's spell, but your talents' rungs count (Scroll adept)"* |
| 15 | `class-levels.md`, `talents.md` | every granted spell repeats `(never more than 14; your stats may lift it to 21, more with Keen mind)` | the row text leaks into two more tables | fixed by the one generator (`wiki/spells.rs::does`) |
| 16 | `spell-tiers.md` | the "Most lift" column | a per-tier number that modifies another per-tier number | removed |
| 17 | `spell-resolutions.md` | "Most damage, of the ceiling: attack 100 %, auto 75 %, area 80 %, touch 120 %" | a factor on an author's check | the column goes; the table keeps the roll (attack, auto, area, touch) and the cost |
| 18 | `magic.md` "Spellcraft talents" | "Keen mind ... lets your stats lift that school's or element's spells 10 points higher than their tier's most lift" | "points" of a percent of a cap | *"Keen mind: that school's spells stand a rung higher for you."* |
| 19 | `magic-items.md` | "*Potent* ... raise the most lift of every spell by their number" | as 18 | *"Potent: your spells stand one (or two) rungs higher."* |

What is already clear and stays: the shapes' descriptions, the resolutions (attack, auto, area, touch), the mana paragraph (long, but every factor is a thing the player chose: tier, shape, armour, talents), learning and mishaps, the wand and rod paragraphs apart from the lines above.

### 3.2 Mechanics too complicated to be said in one plain sentence, ranked

1. **The lift** (§2): percent of a cap per modifier point, two stats summed, clamped to a per-tier most widened by a talent and by items, times the cap, rounded down, turned into dice by a difference of two floors. Six operations, three of them roundings or clamps. The wiki needs a worked example; the owner could not predict a stat point's effect. **Removed.**
2. **Dice by caster level**: a derived level (four inputs) against a per-tier comfortable level, divided by a per-spell `per_levels`, capped by `max_dice`. Four concepts before the spell's own two. **Removed** (§4.4).
3. **Reach and duration by percent of a modifier**: "50 % of a cell a point, a half rounded down", a cap told as "to 12", School focus and Farcasting cells added after, a burst that costs by a radius it no longer has. **Replaced** by cell ladders.
4. **Talent dice then a clamp**: a die, +1 a die, +2 a die, each then held to a lifted cap, so what a talent is worth depends on where the cap sits. **Replaced** by rungs.
5. **The hexes' two-stat duration** (Presence 50 %, Willpower 25 %, rounded to the nearest, a half down). **Replaced** by one stat.
6. **The three older scrolls**: bursts and bolts of their own with their own dice and no cap, beside the spell scrolls of the same spells, with a special rule for talents (`max(taught cap, own most)`). **Folded** into spell scrolls at a rung.
7. The mana cost (seven factors) is long but not janky: each factor is a choice the player made (tier, shape, armour, talents). **Unchanged**, outside this document.

### 3.3 Concept count: what a player must hold to read one spell row

Fireball's row, today and under the proposal. A concept is a number or rule the reader must know or compute that is not printed in the cell itself.

| Cell | Today | Proposed |
|---|---|---|
| Damage | 1 tier · 2 comfortable caster level · 3 caster level (3a level × casting %, 3b Arcane training, 3c *of the Adept*, 3d Attuned) · 4 `per_levels` · 5 `max_dice` · 6 the tier's ceiling · 7 the resolution's percent · 8 the cap as a clamp · 9 the stat modifier · 10 percent of the cap per point · 11 two stats summed · 12 `max_lift` by tier · 13 Keen mind's +10 · 14 Potent's +N · 15 the lifted cap rounded down · 16 the dice "room" formula · 17 talent dice then the clamp — **17** (20 with 3a–d) | 1 the roll · 2 the stat's score · 3 the rung reached (and 4 a talent's or item's rungs, when the player has any) — **3** |
| Reach | 1 the modifier · 2 50 % of a cell a point · 3 a half rounded down · 4 the "to" cap · 5 School focus's cell · 6 Farcasting's cells — **6** | 1 the score · 2 the rung — **2** |
| Duration | 1 the modifier · 2 a degree per 2 points · 3 a second stat at a quarter · 4 the rounding · 5 the status's max · 6 Lingering's degrees — **6** | 1 the score · 2 the rung — **2** |

## 4. The proposal: ladders

### 4.1 Goals

| # | Goal | Why |
|---|---|---|
| G1 | **A spell's growth is written in its file and printed as written.** No global pipeline between the data and the wiki row. | the owner: "entirely per-spell in its spell text ... clearly written" |
| G2 | **One lookup, no arithmetic**: the player compares a score to a number. No percents, no modifiers, no rounding, no clamps. | "nobody can predict what a stat point does" |
| G3 | **Any dice shape**: count, size, bonus, mixed dice, per rung. | the owner's tools |
| G4 | **One driver per dimension**: one stat for damage, one for reach, one for duration. | "one stat increases range ..."; §3.2 item 5 |
| G5 | **One unit for everything outside the spell**: a rung. A talent, an item, a wand's make, an old scroll's strength are all "rungs up". | §3.2 item 4 and 6 |
| G6 | **Caster level is difficulty, not power.** | the addendum, §4.4 |
| G7 | **Hidden in the game, exact in the wiki**: no number in the game's text; every rung in the wiki. | the owner's rule |
| G8 | **Integers only, deterministic, derived at cast time, nothing saved.** | `docs/dev/spells.md`'s invariants |
| G9 | **Balance within a die of today** at levels 10, 30, 60 for the trained caster; shifts named. | the migration must not re-tune the game by accident |

### 4.2 The rule, as the wiki will state it

> Every spell has a **base**, what it does for anyone who casts it, and may have **ladders**: for its damage (or healing), its reach, and how long its effect lasts. A ladder names one stat and lists scores with what the spell does at each: *Fireball: 2d6 fire; Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2*. When your score reaches a number, you cast that rung. **The last rung is the most the spell ever does, for anyone.** Talents and magic items that strengthen spells take you **rungs up** a ladder; past the last rung they do nothing. A scroll read and a wand used cast the base, plus their own rungs.

That is the whole rule. §4.3 is the same thing in the precision the code needs.

### 4.3 Rungs, precisely

- A **ladder** is a stat and a list of **rungs** `(at, value)`, `at` strictly ascending scores in 1 to 40 (the stats' range), at most `MAX_RUNGS` (8). The spell's **base** value is rung 0. The **value** is a `Roll` for `damage` and `heal`, whole cells for `range`, `length`, and `radius`, whole degrees for `degrees`.
- **The rung reached by stats**: `by_stats = |{ rung : at ≤ score }|`, `score` the caster's **effective score** in the ladder's stat (`Actor::effective_stats`: what the character screen shows, with statuses, stat damage, drains, and worn items). A score below the first rung is rung 0. Nothing is ever below the base: a low score never shrinks a spell (today's `max(0)`, kept as a consequence, not a rule).
- **Rungs from outside** (`more`): the sum of the caster's talents' and items' rungs for that dimension (§6.1), 0 or more. For a scroll read or a wand used, `by_stats` is 0 for damage and heal and `more` is the reader's talents' rungs plus the item's own (§6.4); reach and degrees are the reader's as for an own cast.
- **The rung cast**: `rung = min(by_stats + more, rungs.len())`. **The value cast** is `rungs[rung − 1].value`, or the base at rung 0. The last rung is reached and never passed.
- **What the dimensions mean**: `damage` and `heal` are the roll made (one draw per die, in group order, the bonus added; an area's one roll for all as today); `range` is the farthest cell a ray, bolt, or burst goes; `length` a cone's or line's; `radius` a burst's; `degrees` the severity a buff or hex is given (the status's own `max` still holds it in `Effects::add`, and the data check keeps every rung within it, so the clamp never fires). A burst **costs by its base radius** (the one place `shape_cost` reads a radius), as today.
- **No cap at run time.** `spell::capped`, `Cast::cap`, `Burst::cap`, `Bolt::cap`, `Actor::spell_cap`, `Actor::spell_lift`, `SpellRules::lift`, `lifted_cap`, `grown`, `lengthened`, `max_bonus`, `SpellDice::at`, `SpellDice::lifted` are deleted. A **critical** (attack and touch resolutions only, as today) rolls the damage twice, as a blow's does, and is not held to anything (§12 Q4).
- **To-hit**: `aim = "dexterity"` (default) names the one stat whose **modifier** is added to an attack or touch spell's roll, as a blow adds Dexterity's; a scroll's or a wand's always Dexterity. *Of the Magus*, *Baleful*, the Spell attacks skill, and the caster styles are unchanged. (A modifier, not a ladder: to-hit shares the blows' rule, and a ladder would make it the one place scores and modifiers mix.)
- **Integers**: a comparison, a count, a minimum, a lookup. No division anywhere in growth.

### 4.4 Caster level stops growing spells (the addendum)

**What caster level does today**, and what it would do:

| Reader of caster level | Today | Proposed |
|---|---|---|
| `tier_of` → the comfortable tier → **mana** (overreach ×1.5 a tier) | yes | **kept**: this is "how hard a spell is for you" |
| the **study check** (`learn.rs::study_odds`) | yes | **kept** |
| what a class table **grants** when (the content test: a granted spell within the tier reached) | yes | **kept** |
| a spell's **dice** (`per_levels`, `max_dice`) | yes | **gone**: the ladder's stat |
| a **scroll's** cast level (`[spell.scroll] level`) | yes | **gone**: a scroll casts the base plus `rungs` (§6.4) |
| a **wand's** level (`Wand::caster_level`, Wand mastery, *of Wands*) | yes | **gone**: a wand casts the base plus its `rungs` (§6.4) |
| a **rod's** reach (`range + level / per_levels`) | yes | **gone**: a Willpower ladder, as a spell's (§6.4) |
| a rod's sleep and terror **gate** (a creature of level at most the caster level) | yes | **kept** (§12 Q10): "no stronger than your magic" is a difficulty, not growth |
| the dumps (`Caster level N`, `cl N`), the balance probe | yes | kept |

**For retiring it** (the recommendation):

1. **One driver per spell.** With both level and stats growing a spell, every row needs two tables and a reader must hold both. The ladder is one.
2. **Caster level is itself a pipeline** (level × casting % + Arcane training + *of the Adept* × Attuned): four concepts a player must compute before the spell's own rule applies (§3.3). Stats are a number on the character screen.
3. **It makes the stats matter.** The stat training design (#721, §5) gives Intelligence a path through casting and Willpower through spending mana; a caster who trains the mind now grows their spells by it. Today Intelligence lifts a cap by a percent nobody can feel; a rung at 16 is felt.
4. **The level keeps every other reward.** A caster's level already brings mana (the pool, which decides how often a spell is cast), new tiers to learn, the class table's granted spells and talents (Empower at 32, Archmage at 60 are level-driven rungs), and comfort (cheaper casting). Dice were the one thing level gave that stats could also give.
5. **The owner's instinct** ("perhaps that is just learning difficulty thing") matches what the magic design built caster level for: the tier table, overreach, and study. Dice growth was bolted on in §5 of that design and the lift on top of it in #556.

**Against it**, honestly:

- A caster with a poor mind never grows a spell. Accepted ("unbalanced choices are fine"); the wiki says exactly where the rungs are, and a potion of wit, an elixir, or the stat training paths are the way up. A level-60 wizard with Intelligence 12 casts a level-1 Fireball; today they cast a 3d6 one. This is the one real loss, and it is the design's point.
- Arcane training, *of the Adept*, and Attuned lose their effect on power and keep only comfort and learning. They stay worth taking (a tier-3 spell comfortable at level 10 instead of 12 costs 11 mana rather than 16; a tier-5 scroll learnable without a mishap sooner), but their descriptions change (§6.2).
- The balance probe's spell method changes (it reads caster level for dice today; §11 S5).

**What it changes elsewhere in this document**: the concept count drops by four (§3.3); the migration table has no `per_levels` or `max_dice` column, every spell's base is what it rolled at its comfortable level (§8); `docs/dev/spells.md`'s tier table keeps `comfortable`, `cost`, `ceiling` (redefined, §4.6), `wand_charges`, and loses `max_lift`; the wiki's tier table loses "Most lift" and its "Comfortable caster level" column is explained as comfort, not power.

### 4.5 What keeps spells relevant at level 30 to 60

Without dice by level, three things, all already in the game or already planned:

1. **Higher tiers.** The combat progression design (§7.3) showed that even with the lift a level-60 caster needs the tier 4 to 9 spells: "a level-60 caster needs the tier 4 to 9 spells to exist, which is content the magic design planned and no scaling replaces". A tier-5 burst written at base 5d6 with a ladder to 8d6+4 (within the tier's top of 72 × 80 % = 57) kills the level-60 median monster (225 hit points, `docs/balance.md`) in 6 casts for 53 mana of a 684 pool; a tier-7 one at 10d8 (top 110 × 80 % = 88) in 4. The tier content is the progression, as the design meant.
2. **Rungs that run on.** The trained wizard's Intelligence reaches 20 at level 30 and 23 at 60 (`docs/balance.md`; the stat training design puts a main stat at +12 to +17 by 60); ladders are written to 25 and may go higher for the high tiers, so a caster keeps climbing after the dice-by-level curve would have flattened (today `max_dice` is reached by caster level 24 on every tier-3 spell: a level-20 wizard's Fireball is a level-60 wizard's Fireball but for the lift).
3. **Rungs from the class table.** Empower (wizard 32), Archmage (60), Keen mind, Potent items: level-gated rungs, each worth exactly one step on every ladder the player can read.

§8.3 shows the trained wizard's casts to kill at 10, 20, 30, and 60 today and proposed; they match within a die at 30 and 60 because today's growth had already stopped there.

### 4.6 The ceiling: an author's check, not a rule

The magic design's strict cap (its §5, "caps, not curves") had one job: no 10d6 fireballs. Under ladders the top rung is in every row and reviewed in every PR, which does most of that job; a content check does the rest cheaply. `ceiling` stays in `spells.toml`, **redefined as the most a spell of the tier may be written to do at its top rung** (single target), and set to today's lifted maxima so no spell migrated below loses a rung: `[8, 18, 28, 40, 56, 72, 90, 110, 136, 162]` (today's ceiling × (1 + `max_lift`), rounded). The check: every rung's `Roll::max` (the base's too) ≤ `ceiling[tier] × resolution.damage / 100`, harm or healing. `resolution.damage` keeps that one job. The quarter-of-the-ceiling bonus rule goes (a bonus is a rung's own; the ceiling bounds it). **Nothing at run time reads the ceiling**, and the player wiki does not print it (§12 Q9): the fact a player needs, the most a spell does, is the last rung of its own row.

### 4.7 Criticals, resolutions, the area's edge

Unchanged but for the clamp: attack and touch spells roll 2d12 + the aim stat's modifier against Defence and may critical (the damage rolled twice); auto never misses and never criticals; area strikes everyone with one roll, half beyond radius − 1, Evasion halving. The resolutions' cost factors are unchanged.

## 5. The data format and its checks

### 5.1 `[spell.grows]`

```toml
[[spell]]
id = "fireball"
tier = 3
# ...
range = 10
radius = 2
damage = "2d6"                        # the base roll, a Roll (§5.2); was { dice, per_levels, max_dice, bonus }

[spell.grows]                         # all optional; a dimension left out never grows
damage = "intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2"
range  = "willpower 14 11, 18 12"
```

A ladder is one string: the stat's id, then rungs separated by commas, each `AT VALUE` with one space. `AT` is a score, 1 to 40, strictly ascending. `VALUE` is a `Roll` for `damage` and `heal`, an integer for `range`, `length`, `radius`, `degrees`. One string, so the wiki prints it and a reviewer reads it as the player will. The keys:

| Key | For | Base field | Value | Bounds |
|---|---|---|---|---|
| `damage` | a `damage` spell | `damage` | a `Roll` | each rung's `max` ≥ the one before, `min` ≥ the one before, not identical; the base's and every rung's `max` ≤ the tier's ceiling × the resolution's percent |
| `heal` | a `heal` spell | `effect.heal` | a `Roll` | as `damage` |
| `range` | ray, bolt, burst | `range` | cells | strictly ascending from the base, 1 to `MAX_REACH` (20) |
| `length` | cone, line | `length` | cells | strictly ascending from the base, a cone to `MAX_CONE` (5), a line to 20 |
| `radius` | burst | `radius` | cells | strictly ascending from the base, to the burst table's length (3) |
| `degrees` | buff, hex | `effect.severity` | degrees | strictly ascending from the base, to the status's `max` and `MAX_SEVERITY` |

`aim = "dexterity"` (a stat id; default Dexterity) on an `attack` or `touch` damage spell replaces `[spell.scales] to_hit`.

Retired fields: `[spell.damage] dice, per_levels, max_dice, bonus` (now `damage = "2d6"`), `max_range`, `max_length`, `max_radius` (the top rung is the most; a spell without a ladder reaches its base), `[spell.scales]` (all four), `[spell.scroll] level`, `[tiers] max_lift`. `README.toml` and `spells.toml`'s comments are rewritten with the grammar above and §5.4's conventions.

### 5.2 `Roll`: dice of more than one size

`Dice` is one group (`count`, `sides`, `bonus`) and stays what it is for stats, weapons, and everything else. A **`Roll`** (`dice.rs`, beside it) is a sum of `Dice` groups and one bonus: text `3d6+1d10`, `5d6+2`, `2d8`, `+3`; groups joined by `+`, at most `MAX_GROUPS` (4), the bonus last and once; `min`, `max`, `mean_x2` (twice the mean, an integer), `roll(rng)` (each group in order, one draw a die, as `Dice::roll`), `Display` as written; `From<Dice>`. `Attack::damage` in `combat` becomes a `Roll` (a `Dice` converts), so `combat::strike` rolls a spell's mixed dice as it rolls a blow's; `reflect::Shot::dice` the same. Nothing else changes type: a weapon's `Dice` becomes a one-group `Roll` at the one place a `Profile` is built. S0 (§11) lands this alone, with no behaviour change, so S1 is about spells only.

### 5.3 Validation (`spell_def::resolve`, a `ContentError` naming the spell)

1. A ladder only for a dimension the spell has (as today's `scales` check): `damage` for a damage roll, `heal` for a heal, `range`/`length`/`radius` for the shape that has them, `degrees` for a buff or hex. `aim` only for an attack or touch damage spell.
2. The stat is one of `Stat::ALL`; at least one rung; at most `MAX_RUNGS` (8); scores 1 to 40, strictly ascending.
3. Values as the table in §5.1: `Roll`s that can do harm (`min` ≥ 0, `max` ≥ 1) and never weaken (`max` and `min` non-decreasing, each rung different from the one before); cells and degrees strictly ascending from the base and within the shape's or status's bounds.
4. The ceiling (§4.6) on the base and every damage or heal rung.
5. A wand's `rungs` (§6.4) at most the spell's damage ladder length; a spell scroll's `rungs` the same.
6. Tests: the shipped data passes; a rung below its base, a descending ladder, a stat the spell has no dimension for, an unknown stat, a rung past the ceiling, a radius past the table, a degree past the status's max, nine rungs, a wand with too many rungs are each refused with the words of the error.

### 5.4 Authoring conventions (templates, not rules)

The owner's three universal shapes become the **recommended ways to write a ladder**, stated once in `README.toml`, so most spells read alike without the player having to learn them as rules (the row is still the truth):

| Shape | Writes as | Typical use |
|---|---|---|
| **[A] cells or degrees, one more a rung**, Willpower for reach at scores 14, 18, 22, 26; Presence for degrees at 14, 18 | `range = "willpower 14 11, 18 12"`, `degrees = "presence 14 2, 18 3"` | every aimed spell, every buff and hex |
| **[B] a die more a rung**, the spell's stat at 13, 16, 19, 22, 25 | `damage = "intelligence 13 3d6, 16 4d6, 19 5d6"` | the body of a damage ladder |
| **[C] a bigger die a rung** | `damage = "willpower 19 2d8"`, `"intelligence 16 1d8"` | cantrips; a rung where another die would pass the ceiling |
| **[D] a static bonus at the top** | `25 5d6+2` | the last rung, where a die would pass the ceiling |
| **mixed dice** | `25 3d6+1d10` | sparingly, where a spell's fiction wants it (a hotter core) |

The stat by school, unless the spell's text says otherwise: evocation, transmutation, divination **Intelligence**; necromancy, abjuration, restoration **Willpower**; enchantment **Presence**; reach **Willpower** always; degrees **Presence** always. Rung scores are on the same grid for every spell (13/16/19/22/25 for dice; 14/18/22/26 for cells; 14/18 for degrees) so a player who learns one row knows where to look in the next.

### 5.5 Code sketch: what goes, what comes

| Goes | Comes |
|---|---|
| `content/spell_scales.rs` (`Weights`, `SpellScales`, `ScalesSpec`, `Has`), `MAX_SCALE_WEIGHT` | `content/spell_ladder.rs`: `Ladder { stat, rungs: Vec<(u8, RungValue)> }`, `RungValue::{Roll(Roll), Cells(u8), Degrees(u16)}`, the parser, `Ladder::rung_at(score) -> usize`, `Ladder::value(rung) -> &RungValue`, `MAX_RUNGS` |
| `SpellDice` (`count`, `sides`, `per_levels`, `max_dice`, `bonus`, `at`, `lifted`) | `SpellData::damage: Option<Roll>`, `effect.heal: Option<Roll>`, `SpellData::grows: Grows { damage, heal, range, length, radius, degrees: Option<Ladder> }`, `SpellData::aim: Stat` |
| `SpellRules::{lift, lifted_cap, grown, lengthened, max_bonus}`, `max_lift` | `SpellRules::ceiling_for(tier, resolution)` (the check only) |
| `world/spell_scale.rs` (`spell_lift`, `spell_cap`, `spell_reach_more`, `spell_degrees_more`) | `world/spell_rung.rs`: `Actor::spell_rung(content, spell, dim, extra: Extra) -> usize`, `Actor::spell_roll(...) -> Roll`, `Actor::spell_reach(...) -> Reach`, `Actor::spell_degrees(...) -> u16`, `Actor::spell_to_hit` (the `aim` stat's modifier + *of the Magus*) |
| `Cast::cap`, `spell::capped`, `Burst::cap`, `Bolt::cap`, `cast.rs::damage_dice` | `Cast::roll: Roll` |
| `TalentMods::{spell_dice, spell_per_die, focus_lift}`, `Dice::empowered` | `TalentMods::{spell_rungs, reach_rungs, degree_rungs, wand_rungs}`, `focus_rungs(tags)` (§6.2) |
| `ItemEffect::SpellLift`, `Wand::caster_level`, `Actor::wand_caster_level`, `[spell.scroll] level`, `ScrollSpell::level` | `ItemEffect::SpellRungs`, `Wand::rungs`, `ScrollSpell::rungs` |
| `max_range`, `max_length`, `max_radius`, `SpellData::base_reach` | the base fields alone; `Reach` from the ladders |

The cast path (`world/cast.rs`) changes at four lines: the roll (`Actor::spell_roll` for a mana cast; the base plus item and talent rungs for a scroll or wand), the reach (`Actor::spell_reach`), the degrees (`Actor::spell_degrees`), and the removal of every `capped`. `reflect::Shot` carries a `Roll`. The mishap backfire casts the base. `learn.rs`'s `read_spell` (the three old scrolls) goes with them (§6.4).

## 6. Talents, items, scrolls, wands, rods, traps

### 6.1 Rungs: the one unit outside the spell

Every talent and affix that touched the lift, the cap, the dice, the reach, or the duration becomes **rungs on a ladder**: `spell_rungs` (the damage and heal ladders), `reach_rungs` (range, length, radius), `degree_rungs` (degrees), `wand_rungs` (a wand's damage). Rungs add; the ladder's length is the most. A spell with no ladder in a dimension is not helped in it (its row shows no ladder, so nothing is hidden). The player's sentence: *"Empower: your spells stand a rung higher on their ladders."*

### 6.2 Talents (old → new)

| Talent | Today | Proposed | Wiki row |
|---|---|---|---|
| Empower (wizard 32) | `spell_power { dice = 1 }`, −10 % mana | `spell_power { rungs = 1 }`, −10 % mana | your spells stand a rung higher, and cost a little less |
| Archmage (wizard 60) | `spell_power { per_die = 2 }`, −50 % mana | `spell_power { rungs = 2 }`, −50 % mana | two rungs higher, half the mana |
| Scroll adept (wizard 1) | `spell_power { per_die = 1 }`, reading −50 % | `spell_power { rungs = 1 }`, reading −50 % | your spells and the spell scrolls you read stand a rung higher (§12 Q12) |
| Keen mind (Spellcraft, a school or element) | `school_focus { lift = 10 }` | `school_focus { rungs = 1 }` | that school's spells stand a rung higher |
| School focus (Spellcraft, a school or element) | `school_focus { cost = -10, reach = 1 }` | `school_focus { cost = -10, reach_rungs = 1 }` | cost 10 % less and reach a rung farther |
| Wand mastery (Weaponcraft) | `wand_level { value = 2 }` | `wand_rungs { value = 1 }` | the wands you use cast a rung higher |
| Arcane training (wizard 1) | `caster_level +1 per 5` | unchanged; description: "your mastery of spells grows faster than your years: each tier comes comfortable sooner, and costs you less sooner" | learning and cost only |
| Attuned (Scholar) | `attuned_percent 50` on items' caster level | unchanged; its items' caster level now eases learning and cost only | its description says so |
| Overreach, Channeler, Frugal caster, Quick study, Spell recall, Scroll savant, Careful reader | — | unchanged | — |
| `SpellDamagePercent` (the wards against bursts and bolts) | on traps' and scrolls' bursts | unchanged (it is the target's, not the caster's) | — |

No new `[[system]]` tag: these are the `spells` system's talents re-pointed; the coverage bar is unchanged (`docs/dev/progression.md` "Coverage policy").

### 6.3 Affixes (old → new)

| Affix | Today | Proposed |
|---|---|---|
| Potent (weapon, ring, amulet) | `spell_lift` 4–6 / 6–8 / 8–10 / 9–11 / 11–13 / 13–15 by level 3 to 13 | `spell_rungs`: `{ level 3: 1 }`, `{ level 9: 2 }` (two tiers; a rung is worth a lift of about 15 to 20 %, so the old low tiers collapse into one) |
| of Farcasting (cloak, ring, amulet) | `spell_reach` 1 / 1–2 / 2 | `reach_rungs`, the same numbers |
| of Lingering (helmet, ring, amulet) | `spell_duration` 1 / 1–2 / 2 | `degree_rungs`, the same numbers |
| of Wands (gloves, ring) | `wand_level` 1 / 1–2 / 2 | `wand_rungs`, the same numbers |
| of the Adept (caster level 1 to 3) | learning, cost, dice, scroll and rod levels | learning and cost only; unchanged data |
| of the Magus, Baleful, of Thrift, Mage-wrought | — | unchanged |

Affixes are saved by id, so a save that holds a Potent ring loads: its value (4 to 15) is read as rungs and must be clamped at the one read (`min(value, 2)`) until the next release's bump, or the loader maps the old tiers (value ≤ 8 → 1, else 2) at load. The PR says which (the former is one line).

### 6.4 Scrolls, wands, rods, traps, creatures

- **Spell scrolls** (`read = { kind = "spell", spell = "fireball" }`): a read casts the spell's base (damage rung `0 + rungs + the reader's talents' rungs`; reach and degrees the reader's, as today), free of mana. `rungs` (default 0, at most the ladder's length) is new on the `read` table: **the three older scrolls become spell scrolls at a rung**: fireball `rungs = 1` (3d6, as its burst rolled), lightning `rungs = 2` (4d6), frost `rungs = 0` (2d6). Their `burst`/`bolt` readings, `Burst::cap`, `Bolt::cap`, `learn.rs::read_spell`, and the "max of the taught cap and the scroll's own most" rule go. Careful reader, Spell recall, Scroll savant are unchanged. `docs/dev/scrolls.md` and `wiki/src/scrolls.md` lose the paragraph that sets the three apart (they still fly and burst as they did: the spell's shape does that).
- **Wands**: `[object.wand] rungs` (default 0) replaces `caster_level`; a zap casts the base plus `rungs + Wand mastery + of Wands` (never the user's stats or Potent), reach and degrees the user's. The seven wands ship at 0. The wiki's wand table shows "Rungs" instead of "Caster level".
- **Rods**: `range`, `max_range`, `per_levels` on a rod effect become `range` and `grows.range = "willpower 14 ..., 18 ..."` (the same grammar, `RodEffect::grows`), so the wiki's rod row reads as a spell's: digging `3; Willpower 14: 4, 18: 5, 22: 6, 26: 7` (today 3 + 1 per 3 caster levels to 12: a level-30 wizard reaches 12 today, 6 proposed; §8.2 says why that is right). Sleep's and terror's `up_to_caster_level` gate stays. Channeler unchanged.
- **Traps** (`data/traps.toml` bursts and bolts): untouched; `content/spell.rs` keeps `Burst` and `Bolt` for them (the `cap` fields removed).
- **Mishap backfire**: the taught spell's base, as today's "held to the taught spell's cap" becomes "its base".
- **Spell reflection**: the `Shot` carries the caster's `Roll`; the reflector casts it back as is.
- **Creatures**: creature spells are intents, not the cast path (`docs/dev/spells.md`); their caster level (#525) still feeds their summons' comfort. Nothing changes.

## 7. What the player sees; the wiki, the spec, the docs

**In the game** (words only, no numbers; G7):

- The abilities menu's Spells rows stay "tier 3, 26 mana, aimed"; the aiming mode's reach is the rung reached. The menu's description line for a spell gains a generated clause after the author's text: *"It grows with your Intelligence, and reaches farther with your Willpower."* (the ladders' stats, nothing more; `PerceptionView::own_spell_grows`).
- **One line when a known spell climbs a rung** through a lasting score (a stat rise at the 06:00 or level check, an elixir, a potion of wit's mark turned point; never a status): *"Your fireball burns hotter."* (`[spell.cast] grown`, optional; default *"Your {noun} grows stronger."*; for reach *"... reaches farther."*, for degrees *"... lasts longer."*). Computed by comparing rungs before and after the score changes, for the spells the player knows (`World::spell_rungs_changed`, called from `Stats::train`'s callers). A talent or item taken says nothing: its own line already does.
- The dump `spells` layer adds the rung in words: `Fireball (tier 3, 26 mana): 4d6, rung 2 of 4`; dev `progress` adds `fireball rung 2/4 (In 16; +1 Empower)`; JSON `spells[].rung`, `rungs`. The `--screen abilities` snapshot covers the menu line.

**In the wiki** (exact; the generator `xtask/src/wiki/spells.rs`):

- `spell-list.md`: "What it does" prints the base (`2d6 fire damage; it melts ...`), "Grows with" prints each ladder as written in words (`Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2; range Willpower 14: 11, 18: 12`), "Reach" prints the base (`range 10, radius 2`). A spell without ladders shows `—`.
- `spell-tiers.md`: Tier, Comfortable caster level, Base mana cost, Wand charges (no ceiling, no most lift). `spell-resolutions.md`: Resolution, Roll, Cost. `wands.md`: Rungs for Caster level. `rods.md`: Reach as a spell's. `class-levels.md` and `talents.md` inherit `does`.
- `magic.md`: "The spells" explains the ladder in the words of §4.2; "Caster level" is rewritten as difficulty (§3.1 row 11); "Spellcraft talents" and "Damage and its cap" are replaced by **"How a spell grows"** (the rule, the rungs from talents and items, scrolls and wands at the base, criticals as blows); "The lifted cap" is deleted; "Shapes" loses its Willpower and Presence paragraphs (one sentence each pointing at the rows); "Wands" and "Rods" as §3.1 rows 12 and 13. `scrolls.md` row 14. `magic-items.md`'s Potent, Farcasting, Lingering, of Wands, of the Adept lines. `stats.md`'s Intelligence, Willpower, Presence entries say "grows the spells whose ladders name it".

**The spec and the docs**: spec §8 "Spells" bullets **Spells**, **The cap**, **The lift**, **Casting: Reach, Dice**, **Caster level**, **Scrolls of spells**, **Wands**, **Rods** rewritten to the proposal (the lift bullet deleted, a **Ladders** bullet added); `docs/dev/spells.md`'s Data, The cap, Pure helpers, The caster's stats, The one cast path (Reach, Dice), Choices (the lift), Wands, Rods, Talents, Balance, Wiki, Tests sections; `docs/dev/scrolls.md`, `docs/dev/magic-items.md` (the affixes), `docs/dev/progression.md` (the talents), `docs/balance.md` (§8.4); `README.toml` and `spells.toml`'s comments.

## 8. Migration: every spell, with before and after

### 8.1 Damage and healing ladders (all 15 spells that roll)

The base is what the spell rolled at its tier's comfortable caster level today (its `dice`), so a scroll or a wand is unchanged. Rungs follow §5.4's grid and are chosen so the trained wizard's and priest's casts land within a die of today's at levels 10, 30, and 60 (§8.3). "Top" is the last rung's most, against the author's ceiling (§4.6).

| Spell (tier, resolution) | Today | Proposed `damage` / `heal` | Top ≤ ceiling | Shift from today |
|---|---|---|---|---|
| Ray of frost (0, attack) | 1d6, no growth, cap 6 | base `1d6`; `intelligence 16 1d8` | 8 ≤ 8 | grows once (today never) |
| Spark (0, auto) | 1d4, no growth, cap 4 | base `1d4`; `intelligence 16 1d6` | 6 ≤ 6 | grows once |
| Mend (0, touch heal) | 1d3, no growth, cap 7 | base `1d3`; `willpower 16 1d4, 22 1d6` | 6 ≤ 9 | grows twice |
| Magic missile (1, auto) | 1d4 +1 die per 3 to 2d4; lift In 10 Wi 5, cap 9 → 13 | base `1d4`; `intelligence 13 2d4, 16 3d4, 22 3d4+1` | 13 ≤ 13 | same at 10 to 40; +1 at 60 |
| Burning hands (1, area) | as Magic missile | base `1d4`; `intelligence 13 2d4, 16 3d4, 22 3d4+2` | 14 ≤ 14 | same at 10 to 40; +2 at 60 |
| Cause wounds (1, touch) | 1d6 +1 per 4 to 2d6; no lift; cap 14 | base `1d6`; `willpower 13 2d6, 19 2d8, 25 2d8+2` | 18 ≤ 21 | grows at 30 and 60 (today flat from level 7) |
| Cure wounds (1, touch heal) | 2d6; lift Wi 15, cap 14 → 21 | base `2d6`; `willpower 16 3d6, 22 3d6+3` | 21 ≤ 21 | same to 30; +3 at 40 on |
| Scorching ray (2, attack) | 1d6 +1 per 6 to 3d6; lift; cap 18 → 28 | base `1d6`; `intelligence 13 2d6, 16 3d6, 19 4d6, 25 3d6+1d10` | 28 ≤ 28 | 2d6 from In 13 (today 1d6 to caster level 12); −1 die at 20; same at 10, 30, 60 |
| Arcane bolt (2, area) | 2d6 flat; lift; cap 14 → 22 | base `2d6`; `intelligence 13 3d6, 19 3d6+2, 25 3d6+4` | 22 ≤ 22 | same to 20; +2 at 30 on |
| Frost (2, area) | as Arcane bolt | base `2d6`; `intelligence 13 3d6, 19 3d6+2, 25 3d6+4` | 22 ≤ 22 | as Arcane bolt |
| Blight (2, area) | 2d6 flat; no lift; cap 14 | base `2d6`; `willpower 16 3d6, 22 3d6+3` | 21 ≤ 22 | grows (today never): +1 die from Wi 16 |
| Fireball (3, area) | 2d6 +1 per 4 to 3d6; lift; cap 19 → 32 | base `2d6`; `intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2` | 32 ≤ 32 | +1 die at 10; −1 at 20; same at 30, 60 |
| Flame (3, area) | 3d6 flat; lift; cap 19 → 32 | base `3d6`; `intelligence 16 4d6, 19 5d6, 25 5d6+2` | 32 ≤ 32 | same at 10, 30, 60; −1 at 20 |
| Lightning bolt (3, area) | as Fireball | base `2d6`; `intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2` | 32 ≤ 32 | as Fireball |
| Call lightning (3, area) | as Fireball | base `2d6`; `willpower 13 3d6, 16 4d6, 19 5d6, 25 5d6+2` | 32 ≤ 32 | as Fireball; Willpower (the druid's stat, §12 Q3) |

The `aim` of Ray of frost, Scorching ray, Cause wounds, Mend, and Cure wounds stays Dexterity (today's default; no spell names another). Identify, Remove disease, Light, Barkskin, Blessing, Haste, Evil eye, Curse of sloth, Hex of weakness, Sleep, Web roll nothing.

### 8.2 Reach and degrees ladders (every aimed spell, every buff and hex, the rods)

Today: a cell per 2 points of Willpower's modifier up to `max_range` (Willpower 14: +1, 18: +2), a degree per 2 points of Presence's up to the status's max (the hexes with a quarter of Willpower's on top, dropped: §12 Q8). The ladders below are **exactly today's values** at the scores where today's rounding changed them, so no reach or duration moves for anyone.

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
| Rods of sleep and terror (burst 5, +1 per 5, to 10) | caster level | `range = 5`, `grows.range = "willpower 14 6, 18 7, 22 8"` |

The rods lose reach at high level (a level-30 wizard digs 12 cells today, 6 proposed): their reach was the one place a rod grew with the user at all, and a rod "the same in anyone's hands" (the wiki's own words) fits a flat-ish reach better; the rods' value is the effect, not the range. If the owner wants the long reach back, the ladders may run to 26 with larger steps (§12 Q10).

### 8.3 Before and after: the numbers

Computed for this document by a script over the rules as written (today: `SpellDice::at` and `lifted` with the lift, the roll's expectation held to the lifted cap, exact; proposed: the ladder's rung at the score), for the **trained wizard** of `docs/balance.md` (Intelligence 16 / 18 / 20 / 23 and Willpower 16 / 18 / 20 / 25 at levels 10 / 20 / 30 / 60; caster level 12 / 24 / 36 / 72 with Arcane training) against the **median monster's hit points** of their level (39 / 73 / 116 / 225), no talents; "casts" are expected casts to kill at the mean, per target, hits assumed. The script is not committed: S5 moves the measurement into the balance probe on the real code.

| Spell (tier, res) | L | Today: dice, held to | avg | casts | Proposed: rung | avg | casts |
|---|---|---|---|---|---|---|---|
| Ray of frost (0, attack) | 10 / 30 / 60 | 1d6 → 6 | 3.5 | 11 / 33 / 64 | 1d8 (In 16 / 20 / 23) | 4.5 | 8.7 / 26 / 50 |
| Spark (0, auto) | 10 / 30 / 60 | 1d4 → 4 | 2.5 | 16 / 46 / 90 | 1d6 | 3.5 | 11 / 33 / 64 |
| Magic missile (1, auto) | 10 / 20 / 30 / 60 | 3d4 → 13 | 7.5 | 5.2 / 9.7 / 15 / 30 | 3d4 / 3d4 / 3d4 / 3d4+1 | 7.5 / 7.5 / 7.5 / 8.5 | 5.2 / 9.7 / 15 / 26 |
| Burning hands (1, area) | 10 / 20 / 30 / 60 | 3d4 → 13 | 7.5 | 5.2 / 9.7 / 15 / 30 | 3d4 / 3d4 / 3d4 / 3d4+2 | 7.5 / 7.5 / 7.5 / 9.5 | 5.2 / 9.7 / 15 / 24 |
| Cause wounds (1, touch) | 10 / 20 / 30 / 60 | 2d6 → 14 | 7.0 | 5.6 / 10 / 17 / 32 | 2d6 / 2d6 / 2d8 / 2d8+2 (Wi) | 7.0 / 7.0 / 9.0 / 11 | 5.6 / 10 / 13 / 20 |
| Scorching ray (2, attack) | 10 / 20 / 30 / 60 | 3d6 → 26 / 4d6 → 28 / 4d6 / 4d6 | 10.5 / 14 / 14 / 14 | 3.7 / 5.2 / 8.3 / 16 | 3d6 / 3d6 / 4d6 / 4d6 | 10.5 / 10.5 / 14 / 14 | 3.7 / 7.0 / 8.3 / 16 |
| Arcane bolt, Frost (2, area) | 10 / 20 / 30 / 60 | 3d6 → 20 / 22 / 22 / 22 | 10.5 | 3.7 / 7.0 / 11 / 21 | 3d6 / 3d6 / 3d6+2 / 3d6+2 | 10.5 / 10.5 / 12.5 / 12.5 | 3.7 / 7.0 / 9.3 / 18 |
| Blight (2, area) | 10 / 20 / 30 / 60 | 2d6 → 14 | 7.0 | 5.6 / 10 / 17 / 32 | 3d6 / 3d6 / 3d6 / 3d6+3 (Wi) | 10.5 / 10.5 / 10.5 / 13.5 | 3.7 / 7.0 / 11 / 17 |
| Fireball, Lightning bolt (3, area) | 10 / 20 / 30 / 60 | 3d6 → 27 / 5d6 → 30 / 5d6 → 32 / 5d6 → 32 | 10.5 / 17.5 / 17.5 / 17.5 | 3.7 / 4.2 / 6.6 / 13 | 4d6 / 4d6 / 5d6 / 5d6 | 14 / 14 / 17.5 / 17.5 | 2.8 / 5.2 / 6.6 / 13 |
| Flame (3, area) | 10 / 20 / 30 / 60 | 4d6 → 27 / 5d6 / 5d6 / 5d6 | 14 / 17.5 / 17.5 / 17.5 | 2.8 / 4.2 / 6.6 / 13 | 4d6 / 4d6 / 5d6 / 5d6 | 14 / 14 / 17.5 / 17.5 | 2.8 / 5.2 / 6.6 / 13 |
| Call lightning (3, area) | 10 / 20 / 30 / 60 | as Fireball | | 3.7 / 4.2 / 6.6 / 13 | 4d6 / 4d6 / 5d6 / 5d6+2 (Wi 16 / 18 / 20 / 25) | 14 / 14 / 17.5 / 19.5 | 2.8 / 5.2 / 6.6 / 12 |

Heals, the **trained priest** (Willpower 17 / 21 / 26 at 10 / 30 / 60): Mend today 1d3 (2.0) at every level, proposed 1d4 / 1d4 / 1d6 (2.5 / 2.5 / 3.5); Cure wounds today 3d6 → 20 or 21 (10.5) at every level, proposed 3d6 / 3d6 / 3d6+3 (10.5 / 10.5 / 13.5).

**Reading it**: at levels 10, 30, and 60 every tier-2 and tier-3 spell is within a die of today, and at 30 and 60 the tier-3 spells are identical (today's growth had already stopped: `max_dice` 3 by caster level 24, the lift at its most by Intelligence 18). Level 20 is one die lower on Scorching ray, Fireball, Flame, and Lightning bolt (today's lift reached 60 % there with two stats summed; the ladder waits for Intelligence 19 at level 25 or so), and level 10 one die higher on Fireball and Lightning bolt. The spells that never grew (the cantrips, Blight, Cause wounds, Mend) now grow once or twice, which is the one deliberate buff: a witch's Blight and a priest's Mend should answer their stats as a wizard's Fireball does. Nothing moves by more than a die anywhere, and the level-60 picture (13 casts of a tier-3 burst against the median monster) is the same as today's, which §4.5 answers with tiers, not scaling.

### 8.4 Mana and the probe

Mana costs are untouched: the tier table's `cost`, overreach, shape, resolution, armour, and the talents' cuts are as they are; a burst costs by its base radius. The balance probe's "Spells" method (`world/balance_probe.rs`, `docs/balance.md` "Spells") reads `Actor::spell_cap` and `SpellDice::lifted` today; S5 rewrites it on `Actor::spell_roll` and prints the rung reached beside the dice (`| Spell | L | rung | dice | casts to kill | mana / pool |`), and re-runs the §7.3 cross-check paragraph against the rows above.

## 9. Determinism, saves, replays, fingerprints

- **Draws**: a cast draws its attack roll (if any) and its damage, one draw per die of the `Roll` in group order; an area's one roll for everyone, in ascending actor id; a buff, hex, or cleanse draws nothing. Where today's roll was one group, the proposed draws are the same count for the same rung, so a replay with no stat change over a rung verifies if the dice are the same.
- **Nothing new is saved.** Rungs are derived from `Actor::effective_stats`, the talents, and the items at cast time; `Actor::spells` (spell ids) is unchanged as long as no spell file is added, removed, or renamed (none is: 26 files edited in place, path order kept). Wands' `Stack::charges` unchanged. **Saves do not change**; `FORMAT_VERSION` untouched.
- **The content fingerprint changes** (every spell file, `spells.toml`, the talents' and affixes' files, the rod effects, the three scrolls): saves of 0.9.0 do not load until the integrator's per-release bump, as with any content change (`docs/dev/saves.md`). A Potent affix in a save reads its old value as rungs: clamped at the read (§6.3).
- **Replays** that cast re-hash (`cast_tests`, `spell_talent_tests`, `spell_list_tests`, `affix_spell_tests`, `caster_style_tests` fixtures; the showcase's scenarios cast nothing in their golden walks, `tests/golden/manifest/`) and are re-made in S1. The state hash (`World::state_hash`) is unchanged in form.
- **Determinism tests** carried over: an attack can miss and critical, auto and area never; Evasion halves an area only; no draws without a roll; a cast at every rung draws the dice of that rung; the rung reached at a score boundary (15 → 16) changes the roll and nothing else.

## 10. Choices made along the way, and what is rejected

| Choice | Taken | Rejected, and why |
|---|---|---|
| The primitive | an **explicit ladder** per spell: every rung spelled out | **three universal rules** ([A] cells, [B] a die a rung, [C] a size step) that a spell opts into with a short line: shorter data, but the player must learn three rules and their interplay (what if B and C fall on the same score), and a bonus or a mixed rung needs a fourth; the ladder is the three rules' *output*, and the wiki prints it, so nothing is learned. The three shapes survive as the authoring templates (§5.4) |
| Breakpoints | **scores** | **modifiers**: one more concept (`floor((s − 10) / 2)`), and "Intelligence +3" is not on the character screen; also the to-hit's unit, so the two would mix in a row |
| Drivers | **stats only** for growth; caster level for difficulty | **both** (today): two tables per row; **caster level only**: the stats would do nothing for a caster, against the stat training design that just gave them paths |
| Talents and items | **rungs** | **a flat die / cell** (the owner's alternative): a die of what size on a mixed roll, and a flat die passes the top rung unless clamped, which is the clamp again; rungs respect every spell's own text and need no unit of their own. Cells and degrees as rungs too, for one unit (a reach rung is a cell on every shipped ladder anyway) |
| The ceiling | kept as a **load-time author's check**, re-valued, out of the wiki | **deleted**: cheap insurance against a careless rung in a content PR; **kept in the wiki**: a per-tier number nobody applies |
| Criticals | **rolled twice, unclamped**, as a blow's | **held to the top rung** (today): a clamp the player must know; attack and touch spells pay for the critical with the roll to hit, as the resolution table already prices (§12 Q4) |
| Reach for scrolls and wands | **the reader's** (as today) | **the base**: "you aim it, your will carries it" is intuitive and costs nothing; damage at the base because the magic is the item's |
| The three older scrolls | **spell scrolls at a rung** | **kept apart**: the one special case in reading, with its own talent rule |
| The hexes' Willpower quarter | **dropped** | a second stat at a fraction: §3.2 item 5 |
| Cantrips | **one rung** each (Ray of frost 1d8, Spark 1d6) | **flat** (today): the ceiling at 8 allows a rung, a wizard's first spell should answer the first stat point, and talents' rungs then mean something to a cantrip; the magic design's "a cantrip is a poor weapon beside a crossbow" still holds at 1d8 |
| A spell's stat | **one**, by school with named exceptions | **two weighted** (today): §3.2 item 5 |
| A talent on the rung unit | **none new** | "Mastery: a chosen spell a rung higher": a `choice = "spell"` kind does not exist; deferred (#196) |

## 11. Implementation plan

Six issues in four waves; each links to its section. "**State**" marks a PR that adds or changes saved state; none does. "**Fingerprint**" marks one that changes the content fingerprint. Hot files: `spells.toml`, `spell_def.rs`, `cast.rs`, `talent.rs`'s `TalentEffect`, `affix.rs`'s `Effect`, `wiki/spells.rs`, `docs/dev/spells.md`, spec §8 "Spells"; the false-conflict trick (`docs/dev/merging.md`) applies. Sizes: S a day, M two or three, L a week.

| Wave | Issue | What | Size | State / fingerprint | Depends on |
|---|---|---|---|---|---|
| 1 | **S0 The `Roll` type** (§5.2) | `dice::Roll` (groups + bonus, text, `roll`, `min`, `max`, `mean_x2`, `From<Dice>`, serde); `combat::Attack::damage: Roll` and `reflect::Shot::dice: Roll` with every builder converting; no behaviour change (the hash test and every replay verify as before); `docs/dev/combat.md` one line | S | none / none | — |
| 2 | **S1 Ladders** (§4, §5, §6, §8.1, §8.2 but the rods, §7's generator and dumps) | `content/spell_ladder.rs`; `SpellData::{damage: Roll, grows, aim}`; the retired fields refused with a message naming the new one; `ceiling` re-valued, `max_lift` gone; `world/spell_rung.rs`; the cast path on rungs, every `capped` gone; `TalentEffect::SpellPower { rungs }`, `SchoolFocus { cost, reach_rungs, rungs }`, `WandRungs`; `Effect::{SpellRungs, ReachRungs, DegreeRungs, WandRungs}` with the affix files and the Potent read clamp; `Wand::rungs`; `ScrollSpell::rungs` and the three scrolls folded; the 26 spell files; `README.toml`, `spells.toml` comments; `wiki/spells.rs` (`list`, `does`, `reach`, `grows_with`, `tiers`, `resolutions`, `wands`); the dump `spells`/`progress` lines; `docs/dev/spells.md`, `scrolls.md`, `magic-items.md`, `progression.md`; the spec bullets; the tests of §5.3 and §9; replays re-made | L | none / **fingerprint** | S0 |
| 3 | **S2 Rods** (§6.4, §8.2) | `RodEffect::grows` on the same parser; `per_levels`, `max_range` gone; the four rod files; `rods.md`; `docs/dev/spells.md` "Rods"; `rod_tests` | S | none / fingerprint | S1 |
| 3 | **S3 The prose** (§7, §3.1) | `wiki/src/magic.md` ("How a spell grows", "Caster level", "Shapes", "Wands", "Rods", "Spellcraft talents"), `scrolls.md`, `magic-items.md`, `stats.md`; the spec's prose paragraphs; `docs/design/README.md`'s entry | S | none / none | S1 |
| 3 | **S4 The rung line** (§7) | `World::spell_rungs_changed` at the stat rises; `[spell.cast] grown`; `own_spell_grows` in the menu's description line; screen snapshot `abilities-menu-spells` | S | none / none (a message) | S1 |
| 4 | **S5 The probe** (§8.4) | the balance probe's spell method on `Actor::spell_roll`, the rung column; `docs/balance.md` "Spells" re-run and the §7.3 paragraph rewritten; `docs/dev/spells.md` "Balance" | M | none / none | S1, S2 |

**Why this order**: S0 first because it is a type change across `combat` with no spell in it, so it merges clean before the hot spell files move. S1 is one PR because the old hooks (`spell_lift`, `SpellDice`, `max_range`) and the talents that feed them cannot be half-replaced: every spell file, the generator, and the talents' effects change in the same commit or the content fails to load and the wiki fails to build. S2, S3, S4 beside each other: a rod file, a wiki page, a message. S5 last.

**Talents and class abilities** (the feature template's section, per slice): S0 none (a type). S1 re-points Empower, Archmage, Scroll adept, Keen mind, School focus, Wand mastery, Potent, of Farcasting, of Lingering, of Wands onto rungs (§6.2, §6.3) and re-words Arcane training, Attuned, and of the Adept; no new `[[system]]` tag (the `spells` system's bar is unchanged). S2 Channeler unchanged, none new. S3, S4, S5 none.

## 12. Open questions, each with a recommended answer

For the owner to approve line by line.

| # | Question | Recommended answer |
|---|---|---|
| Q1 | Retire caster level from spell growth: it decides only which tiers are comfortable (mana, overreach), the study check, and the class tables' grants? | **Yes** (§4.4). The one loss, a high-level caster with a poor mind, is the design's point. |
| Q2 | Breakpoints on **scores** (Intelligence 16), not modifiers? | **Yes**: the number on the character screen. |
| Q3 | **One stat per ladder**, by school (evocation, transmutation, divination Intelligence; necromancy, abjuration, restoration Willpower; enchantment Presence), with Call lightning on Willpower as the druid's spell? The druid's Flame, Frost, and Spark then grow with Intelligence, the druid's weaker stat. | **Yes**; unbalanced choices are fine, and the druid's own spells can be re-stated on Willpower in S1 if the owner prefers (a one-word change each). |
| Q4 | Criticals **unclamped**: an attack or touch spell's critical rolls the dice twice, as a blow's? | **Yes**: the last clamp goes; attack and touch pay for it with the roll to hit. Alternative: "a critical never passes the last rung's most" (one more sentence). |
| Q5 | Talents and items as **rungs** (Empower 1, Archmage 2, Scroll adept 1, Keen mind 1, Potent 1 or 2, Farcasting and Lingering their numbers, Wand mastery 1), never flat dice or cells? | **Yes** (§6.1). |
| Q6 | **Cantrips grow one rung** (Ray of frost 1d8, Spark 1d6)? | **Yes**. |
| Q7 | **Blight, Cause wounds, and Mend get ladders** (Willpower), where today they never grow? | **Yes**: a witch's and a priest's spells should answer their stats as a wizard's do. |
| Q8 | Hexes last by **Presence alone** (today's quarter of Willpower dropped)? | **Yes** (§3.2 item 5). |
| Q9 | The tier **ceiling stays as an author's check** on the top rung, re-valued to today's lifted maxima, and leaves the player wiki? | **Yes** (§4.6). |
| Q10 | **Rods reach by Willpower ladders** (digging 3 to 7, not 3 to 12), and sleep's and terror's "no stronger than your magic" gate stays on caster level? | **Yes**; if the long dig is wanted, the ladder runs `14 5, 18 7, 22 9, 26 12`. |
| Q11 | **One line** when a known spell climbs a rung through a lasting score ("Your fireball burns hotter."), and the menu's description says what a spell grows with, in words? | **Yes** (§7); no numbers in the game. |
| Q12 | Scroll adept's rung on the wizard's **own spells too**, not only on scrolls read? | **Yes**, as today. |
| Q13 | Wands cast at **rung 0** by default (the base), plus their `rungs`, Wand mastery's 1, and of Wands' 1 to 2; the three old scrolls become spell scrolls at rungs 1 / 2 / 0? | **Yes** (§6.4). |
| Q14 | The rung **grid** as a convention (dice at 13 / 16 / 19 / 22 / 25; cells at 14 / 18 / 22 / 26; degrees at 14 / 18), stated in `README.toml`, not checked? | **Yes** (§5.4): a reader learns one row and knows where to look in the next. |
| Q15 | Mixed dice used **once** in the shipped list (Scorching ray's top rung `3d6+1d10`) to show the tool, else plain? | **Yes**; drop it to `4d6+4` if the owner finds it precious. |
