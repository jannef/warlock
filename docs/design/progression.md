# Warlock — Progression: experience, levels, class tables, skill advances, stat raises, talents

Design document for the owner's review (2026-10-20). Written against `origin/main` at 64b3713. Nothing here is implemented; §11 cuts it into issues.

## Contents

1. Summary
2. Research, tied to the decisions
3. Experience and levels (curve, factors, sources, hit points, the schedule)
4. Class level tables (all fifteen classes, 1 to 60)
5. Skill increases at level-up
6. Attribute growth
7. Talents (name, picks, prerequisites, ranks, the list by tree)
8. Coverage matrix and the policy for new systems
9. What the player sees (messages, screens, character creation, keys, wiki)
10. Data and code sketch (files, state, hooks, determinism)
11. Implementation plan (issues and waves)
12. Open questions for the owner

## 1. Summary

**What the owner asked for**: a level cap of 60 on a steep curve; race and class multipliers on the XP a level needs (long-lived races and patient classes pay more, the poor choices level faster); a per-class table of what each level gives; skill increases tied to level-ups "something like ADOM"; a decision on attribute growth; and a talent system with picks at creation and at levels, prerequisite trees, stat/skill/race/vision requirements, and a list of talents that interact with the systems that exist, with a policy so no system is ever "woefully unsupported".

**The proposal in one table:**

| Piece | Decision |
|---|---|
| Curve | `threshold(n) = 20·(n−1) + 10·(n−1)² + 3·(n−1)³`, cap 60 (652 127 XP for a human warrior). A cubic (as Caves of Qud) rather than a geometric curve because creature XP grows linearly with creature level; so the *kills of your own level* a level costs grow linearly from 2 to 100, which is what "steep" should mean in a game whose XP comes from kills. §3 |
| Factors | `experience` percent per race (elf 140 … half-orc 90) and class (wizard 125 … commoner 80), multiplied onto the thresholds (Angband's `expfact`; ADOM has the same idea). 72 % to 175 %. §3.3 |
| Sources | kills and quests as today; plus small, draw-free **discovery** XP: the first kill of each creature type, the first arrival on a level. §3.4 |
| Every level | **skill advances** (2–3 by class, +1 at In 16, humans +1) spent on trained skills with class dice (`1d4+2` known-for, `1d4`, `1d3`), a used skill rolling twice; marks stay as the by-use channel. §5 |
| Every 4th level | **+1 to a chosen stat**, lifting its potential if it is there (no hidden cap ever refuses a pick). §6 |
| Every 3rd level | **a talent**; 2 at creation (humans 3); class bonus picks (8 or 5) from the class's trees. 27–30 by level 60 from ~100 talents in 10 trees plus racial ones, prerequisites at most two deep (talents, stat scores, skill values, ranks, level, race, eyes). §7 |
| Class tables | a signature at 1, abilities at 6, 12, 18, 25, 32, 40, 50 (ADOM's levels), a capstone at 60, bonus talents between: 135 entries, 27 of them active (in the one abilities menu), every one on an existing hook; casters get placeholder actives on the scrolls' shared cast code until the spell system. §4 |
| Coverage | a `systems` tag on every talent and class ability, a closed list in data, a content test that fails when a listed system has fewer than 2 talents and 1 class ability, and a "Talents" section in the feature issue template. §8 |
| UI | level-up messages; one `LevelUp` screen on the stack (stat → skills → talent, the one paged list, Esc banks); a **Talents** info page (page 7); a Talents step in character creation with a Random row; everything rebindable; words in the game, numbers in the generated wiki tables. §9 |
| Data and code | `data/talents/<id>.toml`, `data/classes/<id>.toml` (directories: churning tables), settings in `talents.toml`/`experience.toml`; `Progress` in the world state, `Actor::talents`, one `TalentMods` aggregation read at the hooks the affixes already use; a new `levels` random stream for advance dice, picks as replayed actions, no level-up draw from the gameplay stream. §10 |
| Plan | 20-odd PR-sized issues; the critical path is three (curve → talents core → screens), the rest widens in parallel waves. §11 |

**Key decisions, and why**

1. *Steepness is measured in kills, not XP.* With linear creature XP the cubic is the curve under which each level costs more fights than the last, all the way to 60, without making 60 unreachable (about 3 000 even-level kills, 300 to level 20).
2. *Three separate budgets that never compete*: talents, stat raises, skill advances. The research is unanimous that the disliked systems are the ones where one pool buys two things (5e's feat-or-stat) and the liked ones keep pools apart (Pathfinder 2e, ToME).
3. *Talents are never taxes, never level-1-only, never gates on ordinary actions, never flat numbers that stop mattering*: at most two-step chains, every link worth having, scaling effects, everything available whenever its prerequisites hold. These are the four complaints ADOM's and D&D's communities repeat.
4. *Classes differ in kind, talents in degree.* ADOM's praised part is class powers that change what you can do; its unloved part is talents that add +2. Here the class tables carry the identity (abilities), and talents the build (modifiers and a few actives).
5. *Both skill channels, one skill.* Marks reward doing; advances reward surviving; a used skill gets the kinder die, so neither channel punishes the other, and the count of advances never exposes a hidden modifier (a score threshold instead).
6. *Stats grow at level-up, modestly, and the potential moves with them*, so the hidden ceiling never has to say no.
7. *Cooldowns are statuses.* Class actives that heal or buff are limited by a visible, time-based status (Winded, Drained), the one kind of limit compatible with "time is the penalty".
8. *Coverage is enforced by a test, not a promise.*

## 2. Research: what players praise and complain about, tied to the decisions

(Three research passes: ADOM and the D&D family; Angband, ToME 4, DCSS, Sil, Caves of Qud, Cogmind, Brogue, classic WoW; and player and designer sentiment. Sources in brackets; full notes are in the research agents' reports.)

| Finding | Source | Decision it drives |
|---|---|---|
| ADOM gives a talent at creation and one every third level (16 by 50), up to 7 at creation with bonuses; class powers at 6, 12, 18, 25, 32, 40, 50, "fixed for each class", of four kinds (one-time, per-level, passive, activated) | ADOM wiki *Talent*, *Class power* | §4's levels and the 1/60 additions; §7.2's every-3rd |
| ADOM's community rates its talents as "not particularly significant"; the trash tier is the flat +2s ("useless except as prerequisites"), Affinities ("bonuses to-hit are easy to come by"), the magic tree ("mostly a waste of time"); Treasure Hunter's two worthless prerequisites were "one of the most widely read" disputes; level-1-only talents (Heir, Alert) lock builds before the player knows the game; later patches made Healthy and Strong Legs "the single best talent" by making them scale | adom.de "Talent Ratings" (2009, archived), ADOM wiki *Treasure Hunter Debate*, *Version History*; r/ADOM via snippets | §7.3 two-step chains, §7.4 scaling effects, §7.6 no stat or level-1-only talents |
| ADOM skill increases: at least 3 per level plus one per 5 Learning over 10 (humans +2, bards +2); per-skill dice on a ladder (1 … 4d5) that degrade as the skill rises and are raised back by successful use; the guidebook's advice is "don't waste level advances" on skills that train by use | ADOM wiki *Skill*, *Yergius*; ADOM Guidebook | §5: a class count plus a visible score threshold (never Learning's modifier), a kinder die for used skills instead of a degrading ladder |
| ADOM attributes: hidden potentials; training by use capped at St 15 "regardless of your potential" (a verified complaint); herb and burden grinding; 1.2.0 removed most use-training; no fixed stat on level-up | ADOM wiki *Attribute potential*, *Attributes*; Steam ADOM thread | §6: no training by use; a raise that lifts the potential so the cap never refuses |
| ADOM has race and class XP modifiers (gnomes about −25 %, trolls much more; monks, paladins, rangers, barbarians about +25 %, farmers and merchants −10 %); Angband 3.x `expfact` = race (100–200) + class (0–35); Angband 4.2 flattened it to Human 100 / High-Elf 145 / others 120 and dropped class factors; DCSS keeps a ±2 "xp aptitude" | ADOM wiki *Experience*; angband `p_race.txt`, `player-birth.c`, 4.2.0 notes; crawl `species/*.yaml` | §3.3: multiplicative factors, kept modest (72–175 %) and shown in words at creation |
| Angband's `player_exp` table is geometric (10, 70, 380, 4 400, 75 000, 1 000 000, 4 500 000 at 2/5/10/20/30/40/50); DCSS exponential early, quadratic late (1 059 325 at 27); Qud `15·x³ + 100`; ToME near-quadratic; WoW quadratic (4 084 700 total to 60, mob XP `5·level + 45`, the ZD table to 60: 15/16/17 from 50/55/60) | angband `player.c`; crawl `player.cc`; Qud wiki *Level*; te4 `load.lua`; warcraft.wiki.gg *Experience to level*, *Mob experience* | §3.2 the cubic and the linear creature guideline; §3.2 the ZD bands to 60 |
| AD&D's racial level limits were the disliked form ("too arbitrarily cruel"; a quarter of tables ignored them); 3e's principle "consequences, not restrictions"; XP multipliers are tolerated when small and shown at birth | EN World *Racial Level Limits* poll, *Tweet streamlining 3e* | §3.3 no caps by race, factors shown |
| D&D 3.5: a feat at 1 and every 3rd, fighter bonus feats every 2nd; the chains (Dodge → Mobility → Spring Attack; Power Attack → Cleave → Great Cleave) are "a tax" whose middle links "you aren't really using"; "feats should be optional upgrades, not mandatory patches"; anything that gates an ordinary action "arbitrarily steals that ability from everyone who doesn't take the feat" | d20srd feats; paizo *Combat Feat Chains*; EN World *Feat Tax*; Papers & Pencils *The Problem with Feats* | §7.3, §7.6; §4.1 bonus picks as the fighter's |
| 5e's feat-or-ASI tension: "most of the feats are never even considered", "the obviously correct choice is boring"; Pathfinder 2e's separate pools (class, skill, general, ancestry feats; boosts at 5/10/15/20) "never compete" | EN World *Hate ASI's?*, *Playtest 6: Feats*; aonprd | §1 decision 2: three budgets |
| ToME 4: 1 class + 1 generic talent point a level, 3 stat points, category points at 10/20/34, 5 ranks per talent with stat and level gates; praised for "build variety is crazy", criticised as "overwhelming", "point-intensive, with many talents … receiving just a single point to unlock them", generic points the chronic bottleneck | te4 wiki *Talents*, `Actor.lua`; Steam and r/ToME4 threads | §7.4 at most 3 ranks, ~30 picks of ~99 (never enough points to take everything, never so many that picks are filler) |
| DCSS: no feats; "the crusade against no-brainers"; "avoidance of grinding": low-risk, slow, rewarded activity "encourages players to bore themselves"; use-based training was removed because a caster "had to stop casting spells and instead try to ineptly fight things" (victory dancing) and replaced by XP-funded training weighted to recent use; a stat point every 3–5 levels by species | crawl manual *Philosophy*; crawl 0.9/0.10 release notes; `species/*.yaml` | §5's kinder die for used skills (the hybrid DCSS landed on), §6's cadence |
| Sil: no levels; XP for the first sight and first kill of a type (decaying), the first time at a depth, and identification; abilities bought with XP behind skill-point and ability prerequisites in data (`ability.txt`: `requires N points`, `requires ability X`) | NarSil manual (Sil's rules), sil-q `ability.txt` | §3.4 discovery XP; §7.3 prerequisites as data |
| Caves of Qud: a stat event every 3 levels (alternating +1 chosen / +1 all), skill points by Intelligence spent on trees with attribute minimums and power prerequisites; "perfect balance is not a goal … many viable and expressive builds" | Qud wiki *Level*, *Attributes*, *Skills*; cavesofqud.com roadmap | §6, §7.3; the owner's "unbalanced is fine" has company |
| Brogue removed levels (1.7) for rationed potions of life and strength and enchant scrolls ("grinding encourages repetitive gameplay rather than exploration"); Cogmind has no XP ("grinding is pointless"; slots by area) | Brogue wiki; Grid Sage Games *You, Cogmind* | not adopted (the owner wants levels), but the no-anti-farming rule and discovery XP are the same instinct |
| Dead levels: Pathfinder's explicit goal "every class gets something every level"; Qud gives something every level; ToME past 50 is "significantly diminished returns" | paizo *Dead Levels*; Qud wiki; Steam ToME | §3.6: every level gives advances, 45 of 60 more |
| Level-up pick menus: liked when "it's not only about what you pick, but what you don't"; disliked as "decision stress" that breaks the dungeon's rhythm | r/roguelikes thread (snapshot); Steam Brogue thread | §9.2: one screen, three short steps, Esc banks, no `--more--` |
| Exploration XP pitfalls: DCSS clamped exploration piety for "the Temple, deep water, or lava" (auto-explore income); Sil's depth XP is one-shot per depth; ID-XP invites hoarding | crawl changelog; Sil manual | §3.4: arrival XP one-shot per level, none for the overworld or local areas; identification XP last and small |
| "Linear fighter, quadratic wizard": give martials options, not only numbers | EN World thread | §4: every martial class has 2–3 actives and free-blow mechanics |

## 3. Experience and levels

### 3.1 What exists, and what changes

| Today (`data/experience.toml`, `content/experience.rs`, `world/experience.rs`) | Proposed |
|---|---|
| `max_level = 50` | **60** |
| `threshold(n) = linear·(n−1) + quadratic·(n−1)²` (10, 10: 24 500 at 50) | add a **`cubic`** term: `20·(n−1) + 10·(n−1)² + 3·(n−1)³` (652 127 at 60) |
| one curve for everyone | the curve times the character's **race × class experience factor** (percent) |
| XP from the player's own kills (WoW scaling, ZD bands to level 45) and quest rewards | the same, ZD bands extended to 60; plus a small **discovery** channel (first kill of a type, first arrival on a level) |
| `hp_level = 20` | unchanged (open question 4) |
| a level gives hit points only | a level gives **skill advances** (every level), a **stat raise** (every 4th), a **talent** (every 3rd), and the **class's gain** at its table's levels (§4–§7) |

Nothing else moves: `kill_xp` keeps its formula, kills by traps and creatures keep giving nothing, and the player keeps seeing their level, XP, and the XP of the next level (`PerceptionView::own_experience`).

### 3.2 The curve

**Formula**: `threshold(n) = 20·(n−1) + 10·(n−1)² + 3·(n−1)³` XP to *reach* level `n` (data: `linear = 20`, `quadratic = 10`, `cubic = 3`, integer math as today). Caves of Qud uses a pure cubic (`15·x³ + 100`); Angband and DCSS use hand-tuned geometric tables. The cubic was chosen over a geometric curve for one reason: **steepness should be measured in kills, not XP.** With creature XP growing linearly with creature level (the guideline below, which the existing creatures already follow), a cubic threshold makes the number of even-level kills a level costs grow *linearly* from about 2 at level 2 to about 100 at level 60; a geometric curve (×1.15 a level) would make it grow a thousandfold, which is not a cap anyone reaches. The last ten levels alone cost 42 % of all the XP in the game.

| Level | XP today | XP proposed | XP for this level | Kills of your own level for this level | Even-level kills so far |
|---:|---:|---:|---:|---:|---:|
| 2 | 20 | 33 | 33 | 1.6 | 2 |
| 3 | 60 | 104 | 71 | 2.8 | 4 |
| 5 | 200 | 432 | 201 | 5.7 | 14 |
| 8 | 560 | 1 659 | 531 | 10.6 | 42 |
| 10 | 900 | 3 177 | 841 | 14.0 | 67 |
| 15 | 2 100 | 10 472 | 1 931 | 22.7 | 160 |
| 20 | 3 800 | 24 567 | 3 471 | 31.6 | 304 |
| 25 | 6 000 | 47 712 | 5 461 | 40.5 | 486 |
| 30 | 8 700 | 82 157 | 7 901 | 49.4 | 717 |
| 40 | 15 600 | 193 947 | 14 131 | 67.3 | 1 310 |
| 50 | 24 500 | 377 937 | 22 161 | 85.2 | 2 080 |
| 60 | — | 652 127 | 31 991 | 103.2 | 3 032 |

("Kills of your own level" assumes the creature XP guideline below; a kill of a creature above you is worth up to 20 % more, below you less, grey ones nothing, as today. Discovery and quests shorten the early levels most.)

**Creature XP guideline** (for new creature data; the wiki shows it, a content test warns past ×3 either way): `xp ≈ 5 × level + 10`. The existing creatures sit on it already: deer 2 and bat 3 (level 1, vermin), rat 5, cave beetle 8 (2), kobold 15 (2), kobold archer 25 (3), kobold shaman 40 (4), shopkeeper 400 (18, a deliberate outlier: 100 on the guideline). No change to them.

**ZD bands** (`[[zero_difference]]`): extend with WoW's values past 45: `from = 50, zd = 15`, `from = 55, zd = 16`, `from = 60, zd = 17`.

### 3.3 Race and class experience factors

Each race and each class gets `experience`, a percent of the XP a level needs (default 100). A character's factor is the product: `threshold(n, character) = (threshold(n) × race × class + 5 000) / 10 000`, Angband-style (Angband 3.x: race 100–200 plus class 0–35; 4.2 flattened it to Human 100, High-Elf 145, others 120; ADOM: gnomes about −25 %, trolls "massively more", monks, paladins, rangers, and barbarians about +25 %). Long-lived races pay more (their `age` range is the guide), as do the classes whose fantasy is patience; the poor choices level fast, which is their consolation.

| Race | Age | `experience` | Reason |
|---|---|---:|---|
| Elf | 90–300 | **140** | long-lived, low-light eyes, the best Dexterity potential |
| Gnome | 40–150 | **120** | long-lived, low-light eyes |
| Dwarf | 40–120 | **115** | long-lived, darkvision, ignores Burdened, stone sense |
| Half-elf | 20–60 | **105** | some of the elves' years |
| Human | 16–35 | **100** | the baseline |
| Halfling | 20–50 | **95** | small and weak (St −3): levels a little faster |
| Half-orc | 14–30 | **90** | short-lived, poor In and Pr |

| Class | `experience` | Reason |
|---|---:|---|
| Wizard | **125** | the scholar's slow road (and no spells yet) |
| Monk | **120** | discipline |
| Priest | **115** | devotion |
| Druid | **115** | patience |
| Paladin | **110** | the oath |
| Ranger | **105** | the long watch |
| Warrior | **100** | baseline |
| Bard | **100** | baseline |
| Thief | **95** | lives fast |
| Barbarian | **90** | fury, no patience |
| Craftsperson | **90** | everyday folk |
| Aristocrat | **90** | everyday folk |
| Merchant | **90** | everyday folk |
| Commoner | **80** | no training to speak of: learns fastest of all |

Range: a half-orc commoner needs 72 % of the baseline, an elf wizard 175 %.

| Character | Lv 2 | Lv 10 | Lv 20 | Lv 30 | Lv 40 | Lv 50 | Lv 60 |
|---|---:|---:|---:|---:|---:|---:|---:|
| Human warrior (100 %) | 33 | 3 177 | 24 567 | 82 157 | 193 947 | 377 937 | 652 127 |
| Halfling thief (90 %) | 30 | 2 867 | 22 172 | 74 147 | 175 037 | 341 088 | 588 545 |
| Dwarf priest (132 %) | 44 | 4 202 | 32 490 | 108 653 | 256 495 | 499 822 | 862 438 |
| Elf wizard (175 %) | 58 | 5 560 | 42 992 | 143 775 | 339 407 | 661 390 | 1 141 222 |
| Half-orc commoner (72 %) | 24 | 2 287 | 17 688 | 59 153 | 139 642 | 272 115 | 469 531 |

What the player sees: their XP and the next threshold as today (`XP 85 / 120`); the character creation step for the race and the class says "Levels come slowly" / "quickly" in words, and the wiki's Races and Classes tables show the percent (a rule written in data, like stat adjustments).

### 3.4 Sources of XP

| Source | Rule | Status |
|---|---|---|
| **Kills** | the player's own (melee, shot, throw, a wound of theirs, their scroll's spell), `kill_xp` with the ZD scaling | exists, unchanged |
| **Quests** | the reward's `xp` | exists, unchanged |
| **First kill of a creature type** (new, Sil's "first kill") | once per type, the type's full `xp` again, **unscaled** by level difference (meeting something new always teaches a little), said as "You learn something of kobolds." + the XP | proposed |
| **First arrival on a level** (new, Sil's "new depth") | `arrival × item level` XP (5 × the depth of a branch level; 5 on a hand-made level; the start level, the overworld, and local areas give none, so walking the land cannot be farmed) | proposed |
| **First identification of an affix** (optional) | `identify = 10` XP the first time an affix id is identified by the player | proposed, lowest priority |
| Discovery of creatures by sight (Sil) | not proposed: sighting is easy to farm with stairs, and our kill bonus covers "learning" | — |
| Rested XP (WoW) | not proposed | — |

Discovery is small by design: about 60 creature types (one day) × their XP plus the levels is a few thousand XP, under a tenth of the road to level 30 and nothing against level 60; it exists to make the first visit to a level, and the first kobold, feel like progress, and it needs no draw of any random stream. Data: `[discovery] first_kill = 100` (percent of the type's XP), `arrival = 5`, `identify = 10`; state: the set of creature ids killed and affix ids identified (level arrivals need no state: a level state exists once visited, so "new" is "the level state was created by this travel"). Left out of the JSON while empty.

Quest XP and discovery XP are *not* scaled by the race × class factor (the factor is on the thresholds, so everything is, once).

### 3.5 Hit points per level

Unchanged: each level up to `hp_level` (20) adds `max(1, class hp ÷ 2 + the Toughness modifier)`, so pools stay linear and a level-60 character's strength is in talents, skills, and stats, not in a runaway pool. Toughness raised at level 4, 8, … (§6) feeds this through `raised_stats` as it does for items. The Vigour talents (Hardy, §7) are the way to buy more hit points, and they scale with level so the complaint that a flat +3 means nothing late (ADOM's Hardy) does not apply. Open question 4 asks whether `hp_level` should follow the cap to 30.

### 3.6 What a level gives, all together

| Every level | Every 3rd (3, 6, …, 60) | Every 4th (4, 8, …, 60) | Class table levels |
|---|---|---|---|
| hit points (to 20); **skill advances** (§5) | a **talent** pick (§7): 20 by level 60, plus 2 at creation (humans 3) | a **stat raise** (§6): 15 by level 60 | a class ability, a bonus talent from named trees, or a weapon skill rank (§4) |

The schedule is data (`[levels] talent_every = 3`, `stat_every = 4` in `data/experience.toml`), so it can be tuned without code. A level never passes silently: every level gives at least skill advances; 30 of the 60 give a talent or a stat raise, and the class tables fill most of the rest.

## 4. Class level tables

### 4.1 The shape

ADOM gives every class seven fixed powers at levels 6, 12, 18, 25, 32, 40, and 50, and players praise them because they differ *in kind* between classes (a barbarian's mighty blow, a paladin's cure, a thief's shop theft) rather than in size. We keep that rhythm, add a **signature at level 1** (so the class is felt from the first turn: today a class is only numbers) and a **capstone at 60**, and between those a class gets **bonus talent picks** from the trees that fit it (D&D's fighter bonus feats), so no class has a dead stretch and the talents (§7) do the breadth while the class abilities do the identity:

| Level | Every class | Martial pattern (warrior, thief) | Common pattern (the rest) |
|---|---|---|---|
| 1 | signature ability | | |
| 4, 10, 16, 22, 28, 34, 46, 58 | | bonus talent from the class's trees | |
| 10, 20, 30, 45, 55 | | | bonus talent from the class's trees |
| 6, 12, 18, 25, 32, 40, 50 | class ability | | |
| 60 | capstone | | |

Plus the shared skeleton (§3.6) for everyone: skill advances every level (the class decides how many, §5), a talent every third level, a stat raise every fourth. A level not in a class's table gives the shared gains only.

**Rules for the entries.** Every entry below names what system it reads or changes (its *hook*), so it can be built now; the few that need a system still to come say so (`#151` damage types; `spell` for the caster placeholders, see below; `action` for a new small action, §9). Active abilities (**A**) register in `warlock_core::ability` under `Source::Class`, with an action of their own (replay text `use:ID[:DIR|:X,Y]`), an `ActionKind` cost, and no default key; passive ones (**P**) are rules read at their hook. Numbers are the first guess, kept in the class's data file (§10) to be tuned by play.

**Building blocks** most entries stand on (each is one PR in §11):

- **B1 Buff and cooldown statuses.** `data/status.toml` grows optional combat fields a status applies while it lasts: `to_hit`, `damage` (as the stances have), `dodge`, `parry`, `cost` (percent of every action's price: a slow), `regen` (exists), `blocks = ["read"]` (actions refused while it lasts), and `follows = "exhausted"` (a status given as this one ends). `penalty` already takes positive numbers (−10 to 10), so a status may *raise* scores. An ability that gives the player a status is `World::afflict(player, status, 1, player)`; a **cooldown** is a status (`winded`, `drained`, `spent`, `hoarse`, `exhausted`) with an `eases` and no tick, during which the ability is `Unavailable::While(status)` ("you are winded") — a visible, honest, time-based limit, which keeps the owner's rule: time is the only price, and nothing counts uses. Statuses on creatures exist already (`Actor::effects`), so hexes work the same way.
- **B2 The shared cast code** (`World::cast_burst`, `World::cast_bolt` with `Caster::Actor`) that scrolls use: a class ability may carry a burst or a bolt in data exactly as a scroll's `read` table does, aimed by the targeting UI. These are **placeholders for casters** until the spell system (magic is out of the first phase); they let the four caster classes have live tables now, and the spell system replaces them (open question 6).
- **B3 Free blows.** `World::second_blow` becomes the one place every extra blow is decided: the off hand's, Sword and Board's bash, and now a *cleave* (after a melee kill, at another adjacent hostile), a *riposte* (after a miss credited to Parry, `why_missed`), and an *opportunist's* blow (after a creature's fleeing step, `World::flees`). Each is a normal attack with its own events and marks.
- **B4 Effective eyes.** `WorldState::vision` stays the base profile; `World::eyes()` applies talent and class changes (threshold percent, darkvision radius, `sees_uv`) and is what perception and creature sight read. Creatures keep their types' profiles.
- **B5 Defence by attack kind.** `Actor::defence` takes the attack's kind, so a bonus can count against missiles only (`deflect_missiles`).
- **B6 Kill tally for all types.** The quests' per-level tally (`Quests::kills`) counts every type, not only those a quest asks for (`favoured_prey`, and discovery's first kill).
- **B7 Small new actions** that skills have waited for: `disarm:DIR` (Disable device), `pick:DIR` (Open lock), `steal:DIR` (Steal), `throw_voice:X,Y`, `hood` (the lantern). Each is its own issue (§11); the class entries that need one say `action`.

### 4.2 Warrior

Bonus talents (Weaponcraft, Guard) at 4, 10, 16, 22, 28, 34, 46, 58. Skill advances 2 a level.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `weapon_drill` | Weapon drill | P | marks in melee weapon skills ×1.25 | `World::resolve` (`practise`) |
| 6 | `shield_wall` | Shield wall | P | Shield +1 while holding a shield | `Actor::defence` |
| 12 | `veterans_wind` | Veteran's wind | A | regain 1 HP per level (at least 10) at once; Winded for 10 min (cannot use it again) | B1; cost `wait` |
| 18 | `cleaving_strike` | Cleaving strike | P | a melee blow that kills gives a free blow at another adjacent hostile (with the Cleave talent: on and on while they die) | B3 |
| 25 | `armour_drill` | Armour drill | P | worn armour's `max_dex` +1 and Protection +1 | `Actor::defence`, protection sum |
| 32 | `weapon_mastery` | Weapon mastery | P | threat +1 with any melee weapon whose skill is at rank 8 or more | `Profile` |
| 40 | `iron_stance` | Iron stance | P | the aggressive stance costs no Dodge; the defensive stance no to-hit | `StanceDef::apply` |
| 50 | `whirlwind` | Whirlwind | A | one blow at every adjacent hostile for the price of one attack (one exertion) | `fight.rs::attack_with` per target; cost `melee` |
| 60 | `champion` | Champion | P | critical multiplier +1 (never past ×5) with melee weapons | `Profile` |

### 4.3 Barbarian

Bonus talents (Might, Vigour) at 10, 20, 30, 45, 55. Skill advances 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `rage` | Rage | A | Raging for 5 min: St +2, To +2, In −2, cannot read; then Exhausted 10 min (St −1), during which no rage | B1 (`penalty` positive, `blocks`, `follows`); cost `wait` |
| 6 | `thick_hide` | Thick hide | P | Natural +2 while wearing no body armour | `Actor::defence` |
| 12 | `fast_movement` | Fast movement | P | steps 10 % cheaper while Unburdened and in no body armour | `World::step_price` |
| 18 | `brutal_critical` | Brutal critical | P | critical multiplier +1 with two-handed weapons | `Profile` |
| 25 | `hardened` | Hardened | P | from now on every level adds hit points again (as levels up to 20 do), +1 | `World::level_hp`, `hp_level` per class |
| 32 | `mighty_thews` | Mighty thews | P | Strength +2 and Toughness +2, scores and potentials (ADOM's 32) | `Stats` |
| 40 | `blood_frenzy` | Blood frenzy | P | a kill while raging extends the rage by a minute | `Effects::add` refresh on `Event::Died` |
| 50 | `mighty_rage` | Mighty rage | P | Raging gives St +4, To +4; Exhausted lasts 5 min | data |
| 60 | `unstoppable` | Unstoppable | P | while raging, bleeding and poison tick nothing, and once per rage a blow cannot take your last hit point | `World::live`, `hurt` |

### 4.4 Paladin

Bonus talents (Guard, Vigour) at 10, 20, 30, 45, 55. Skill advances 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `lay_on_hands` | Lay on hands | A | regain 2 HP per level; Drained for 1 h | B1; cost `cast` |
| 6 | `divine_grace` | Divine grace | P | +2 on disease checks; poison eases twice as fast | `check` modifier, `Effect::until_eases` |
| 12 | `smite` | Smite | A | a melee blow in a direction with +1d6 damage per 10 levels (at least 1d6); Spent for 5 min | `attack_with` with an extra `Profile` die; cost `melee` |
| 18 | `divine_health` | Divine health | P | immune to diseases | `World::expose` refuses |
| 25 | `shield_of_faith` | Shield of faith | P | Shield +1; Sword and Board's bash chance +10 | `Actor::defence`, `second_blow` |
| 32 | `steadfast` | Steadfast | P | Burdened costs nothing (as a dwarf's) | `Load::penalty` |
| 40 | `holy_radiance` | Holy radiance | A | a toggle: you shed a faint light (`item_glow`), seen by all; off again from the menu | worn-light hook (`LightCache` carried source) |
| 50 | `bulwark` | Bulwark | P | Protection +2 | protection sum |
| 60 | `avatar` | Avatar | P | Lay on hands heals fully and cures every status and disease; Drained halved | data, `World::cure` |

### 4.5 Monk

Bonus talents (Finesse, Vigour) at 10, 20, 30, 45, 55. Skill advances 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `martial_arts` | Martial arts | P | bare hands 1d6 (kicks too) | bare-hands dice in `World::attack`/`player_kicks` |
| 6 | `fleet` | Fleet | P | steps 10 % cheaper in no body armour and with no shield | `World::step_price` |
| 12 | `deflect_missiles` | Deflect missiles | P | Dodge +2 against shots, throws, and trap arrows | B5 |
| 18 | `flurry` | Flurry | P | unarmed attacks strike twice (the second at −2) | B3 |
| 25 | `still_mind` | Still mind | P | +2 on Willpower checks; status penalties to In and Wi never apply | `check`, `effective_stats` |
| 32 | `purity_of_body` | Purity of body | P | immune to diseases | `World::expose` |
| 40 | `diamond_soul` | Diamond soul | P | Deflection +2 | `Actor::defence` |
| 50 | `quivering_palm` | Quivering palm | A | an unarmed blow in a direction; a hit that wounds makes the target bleed 3 degrees; Spent 10 min | `attack_with` + `afflict`; cost `melee` |
| 60 | `perfect_self` | Perfect self | P | bare hands 1d8; unarmed threat +1 and critical multiplier +1 | `Profile` |

### 4.6 Ranger

Bonus talents (Weaponcraft, Wayfarer) at 10, 20, 30, 45, 55. Skill advances 3.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `favoured_prey` | Favoured prey | P | +1 to hit and +2 damage against the creature type you have killed most (ties: the first reached) | B6, `Profile` |
| 6 | `tracker` | Tracker | P | you hear sounds 3 fainter, and place every sound you hear exactly | `Actor::hearing`, `PerceptionView::sounds` |
| 12 | `rapid_shot` | Rapid shot | P | shots 10 % cheaper | `World::attack_price` |
| 18 | `woodland_stride` | Woodland stride | P | forest floor, bushes, reeds, tall grass, and marsh cost you as plain ground (no extra time; their noise stays) | `World::step_price` |
| 25 | `swift_tracker` | Swift tracker | P | overworld journeys 20 % quicker | `World::step_price` |
| 32 | `evasion` | Evasion | P | bursts and bolts (traps, scrolls, spells) do you half damage | `cast_burst`/`cast_bolt` victim hook |
| 40 | `camouflage` | Camouflage | P | under the open sky your footsteps are half as loud | `noise::noise` (`is_sky_exposed`) |
| 50 | `volley` | Volley | A | three shots at the target for the price of one (three missiles leave the stack) | `shoot.rs::loose` ×3; cost `shoot` |
| 60 | `master_hunter` | Master hunter | P | bows and crossbows threat +1; favoured prey +2 to hit, +4 damage | `Profile` |

### 4.7 Thief

Bonus talents (Shadow, Finesse, Delver) at 4, 10, 16, 22, 28, 34, 46, 58. Skill advances 3.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `sneak_attack` | Sneak attack | P | +1d6 damage (melee, shot, throw) against a creature that has not seen you (its mind is calm) | `Profile` from `Mind` |
| 6 | `trapfinding` | Trapfinding | P + A | searching +4 and hidden traps 2 easier for you; **Disarm** (a direction): a Dexterity check plus Disable device against the trap's difficulty takes it away; a failure by 5 or more springs it at you | `search_modifier`; `action` disarm |
| 12 | `pick_lock` | Pick lock | A | open a locked door without a key: a Dexterity check plus Open lock against the lock's difficulty (data on locks, default 14); it takes `unlock` whatever comes of it | `action` pick; `door.rs` |
| 18 | `evasion` | Evasion | P | as the ranger's | cast hooks |
| 25 | `uncanny_dodge` | Uncanny dodge | P | the burden's cap on Dexterity in Defence is 2 higher at every level | `Burden::max_dex` |
| 32 | `improved_sneak_attack` | Improved sneak attack | P | sneak attack +2d6 (3d6 in all) | `Profile` |
| 40 | `light_fingers` | Light fingers | A | **Steal** (a direction): take one stack from a creature that is not hostile; a Dexterity check plus Steal against its Willpower check; caught, it turns on you (as a robbed keeper does) | `action` steal; `faction_override` |
| 50 | `hide_in_plain_sight` | Hide in plain sight | P | your footsteps are silent while Unburdened, your attacks half as loud | `noise::noise` |
| 60 | `master_thief` | Master thief | P | sneak attacks threaten a critical from a natural 20 | `Profile` |

### 4.8 Bard

Bonus talents (Scholar, Delver) at 10, 20, 30, 45, 55. Skill advances 3 (ADOM: bards get more).

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `lore` | Lore | P | a magic item you have carried for a day is identified | `World::identify` from `perceive` (as `rot_away` runs) |
| 6 | `wide_learning` | Wide learning | P | +1 skill advance every level | §5 advances |
| 12 | `song_of_rest` | Song of rest | P | resting heals half again as fast | `World::healing_rate` while `resting` |
| 18 | `inspire` | Inspire | A | Inspired for 5 min: +1 to hit, Dodge +1; then Hoarse 10 min (no song) | B1; cost `chat` |
| 25 | `jack_of_all_trades` | Jack of all trades | P | every trained skill counts 5 higher | `Actor::skill_effect` |
| 32 | `throw_voice` | Throw your voice | A | a voice (loudness 50) sounds at a cell you see within 6: creatures that hear it go and look there | `action` throw_voice; `World::sound_at`, the AI's noise step |
| 40 | `silver_tongue` | Silver tongue | P | shop favour +15 %, quest gold ×1.25 | `world::favour`, quest reward |
| 50 | `legend` | Legend | P | quest XP ×1.5; Presence +1 | quest reward, `Stats` |
| 60 | `virtuoso` | Virtuoso | P | Inspired gives +2/+2 for 10 min and never leaves you Hoarse | data |

### 4.9 Priest

Bonus talents (Vigour, Grit) at 10, 20, 30, 45, 55. Skill advances 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `blessing` | Blessing | A | Blessed for 10 min: +1 to hit, Wi +1; Spent 10 min | B1; cost `cast` |
| 6 | `cure_wounds` | Cure wounds | A | regain 1d8 + 1 per 2 levels; Drained 5 min | B1; cost `cast` ×2 |
| 12 | `remove_disease` | Remove disease | A | cures every disease and poison; Drained a day | `World::cure(remove_disease)`, status end |
| 18 | `shield_of_faith` | Shield of faith | P | Deflection +1 | `Actor::defence` |
| 25 | `purify` | Purify | P | rotten food neither sickens you nor exposes you to disease | `eat.rs::finish` |
| 32 | `divine_health` | Divine health | P | immune to diseases | `World::expose` |
| 40 | `smite` | Smite | A | a melee blow in a direction with +2d6 damage; Spent 5 min | `attack_with`; cost `melee` |
| 50 | `miracle` | Miracle | P | once in a run, a blow that would kill you leaves you at 1 hit point, Drained for a day ("A miracle!") | `hurt`/`dies`; a flag in the progress state |
| 60 | `saint` | Saint | P | hit points always come back twice as fast | `World::healing_rate` |

### 4.10 Druid

Bonus talents (Wayfarer, Vigour) at 10, 20, 30, 45, 55. Skill advances 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `wild_empathy` | Wild empathy | P | creatures of animal factions (`animal = true` in `factions.toml`: vermin, deer) are neutral to you until you attack one of that type | `World::stance`, a flag per type in the progress state |
| 6 | `woodland_stride` | Woodland stride | P | as the ranger's | `step_price` |
| 12 | `weather_sense` | Weather sense | P | you know the weather of the next three hours: the status bar's weather segment shows the coming change ("rain soon") | perception, `weather::weather_at(now + 3 h)` |
| 18 | `natures_ward` | Nature's ward | P | wading never gives you a disease; poison eases twice as fast | `World::wade`, status easing |
| 25 | `winters_friend` | Winter's friend | P | ice holds you as if you carried nothing; a plunge never hurts; climbing out costs nothing | `World::tread`, `after_tread`, `tread_price` |
| 32 | `call_lightning` | Call lightning | A | under the open sky: a bolt (4d6, 8 cells, a thunderclap) at a target; Drained 10 min | B2 `cast_bolt`; `spell` placeholder; cost `cast` |
| 40 | `sun_and_moon` | Sun and moon | P | under the sun hit points come back half again as fast; at night your overworld sight is one cell longer | `healing_rate` (`sun().is_up`), `daylight::overworld_sight` |
| 50 | `flame_and_frost` | Flame and frost | A | two abilities: a fireball burst (3d6, radius 2, breaks walls, melts ice) or a frost burst (2d6, freezes water); Drained 10 min | B2 `cast_burst`; `spell` placeholder |
| 60 | `timeless_body` | Timeless body | P | hunger burns half as fast; diseases never progress (they still show until shaken off or cured) | `hunger`, `disease_turn` |

### 4.11 Wizard

Bonus talents (Scholar) at 10, 20, 30, 45, 55. Skill advances 2. The actives are **placeholders** on the shared cast code until the spell system exists (open question 6).

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `scroll_adept` | Scroll adept | P | reading costs half; your spell scrolls add 1 to every die | `read` price, `cast_*` |
| 6 | `arcane_lore` | Arcane lore | P | a magic item you have carried for an hour is identified | identify hook |
| 12 | `spell_recall` | Spell recall | P | one read in four, the scroll is not used up | `read.rs` (one gameplay draw per read) |
| 18 | `arcane_bolt` | Arcane bolt | A | a bolt (2d6, 6 cells) at a target; Drained 2 min | B2; `spell`; cost `cast` |
| 25 | `mage_armour` | Mage armour | P | Deflection +2 while wearing no body armour | `Actor::defence` |
| 32 | `empower` | Empower | P | your spell scrolls and your bolts roll one die more | `cast_*` |
| 40 | `fire_burst` | Fire burst | A | the fireball's burst (3d6, radius 2, blast, heat) at a target; Drained 10 min | B2; `spell` |
| 50 | `arcane_mastery` | Arcane mastery | P | Drained eases twice as fast | status easing |
| 60 | `archmage` | Archmage | P | your spells add 2 to every die and cost half | data |

### 4.12 Witch

Bonus talents (Scholar, Grit) at 10, 20, 30, 45, 55. Skill advances 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `herb_lore` | Herb lore | P | rotten food harms you half as much; a corpse's own harm (`on_eat` harm) never; the angelica root heals 1d6 too | `eat.rs::finish` |
| 6 | `evil_eye` | Evil eye | A | Hexed for 1 min on a creature you see in a direction: −2 to hit; Spent 1 min | B1 on creatures; cost `cast` |
| 12 | `poison_lore` | Poison lore | P | your Poisoned eases twice as fast; +2 damage against poisoned targets | status, `Profile` |
| 18 | `hex_of_weakness` | Hex of weakness | A | Weakened 2 min on the target: St −2, Dx −2; Spent 2 min | B1 |
| 25 | `second_sight` | Second sight | P | you notice hidden things in passing (as if trained in Spot), +2; secret doors 2 easier | `search.rs` |
| 32 | `curse_of_sloth` | Curse of sloth | A | Slowed 1 min on the target: its actions cost half again as much; Spent 2 min | B1 (`cost` field) |
| 40 | `blight` | Blight | A | a burst (2d6, radius 2) that also poisons 2 degrees everyone it reaches; Drained 10 min (poison-typed with #151) | B2 + `afflict`; `spell` |
| 50 | `cauldron` | Cauldron | P | corpses never harm you; their stat chances are doubled; a fresh corpse heals 1d4 | `eat.rs` |
| 60 | `hags_bargain` | Hag's bargain | P | once in a run, a death leaves you at 1 hit point, Drained a day, and Presence −2 for good | `hurt`/`dies` |

### 4.13 Aristocrat

Bonus talents (Weaponcraft, Delver) at 10, 20, 30, 45, 55. Skill advances 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `well_dressed` | Well dressed | P | shop favour +10 %; quest gold ×1.25 | `favour`, quest reward |
| 6 | `fencing` | Fencing | P | Duelist's benefits read one rank higher | `Actor::attack_benefit` |
| 12 | `noblesse` | Noblesse | P | anyone not hostile makes way for you, keepers at their posts too | `World::refusal` |
| 18 | `riposte` | Riposte | P | a blow that misses you by your Parry gives a free blow back | B3 |
| 25 | `connoisseur` | Connoisseur | P | item descriptions show what things are worth; magic items carried a day are identified | perception, identify hook |
| 32 | `iron_will` | Iron will | P | +2 on Willpower checks; Presence +1 | `check`, `Stats` |
| 40 | `commanding` | Commanding | P | Intimidate counts 20 higher; keepers let you leave owing up to 50 gold (the debt stays on the bill) | `skill_effect`, `World::turned_back` |
| 50 | `peerless` | Peerless | P | your criticals confirm at +2 | `combat::strike` |
| 60 | `legacy` | Legacy | P | XP +10 %; Presence +1 | `grant_xp` |

### 4.14 Commoner

Bonus talents (**any tree**: the commoner's gift is versatility) at 10, 20, 30, 45, 55. Skill advances 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `hard_work` | Hard work | P | Burdened slows you ×1.10 instead of ×1.25 (what Balance 100 gives), on top of Balance | `World::burdened` |
| 6 | `hearty` | Hearty | P | hunger burns 10 % slower | `World::digest` |
| 12 | `stubborn` | Stubborn | P | +1 on Toughness checks; poison never lowers your scores | `check`, `Effects::penalty` |
| 18 | `big_hands` | Big hands | P | +1 damage with maces, clubs, and bare hands | `Profile` |
| 25 | `thick_skull` | Thick skull | P | Protection +1 | protection sum |
| 32 | `second_wind` | Second wind | A | as the talent: regain 1 HP per level, Winded 15 min | B1 |
| 40 | `salt_of_the_earth` | Salt of the earth | P | hit points come back a quarter faster, resting another quarter | `healing_rate` |
| 50 | `folk_hero` | Folk hero | P | quest XP and gold ×1.5 | quest reward |
| 60 | `unlikely_hero` | Unlikely hero | P | +1 to every stat (and its potential) | `Stats` |

### 4.15 Craftsperson

Bonus talents (Weaponcraft, Guard, Delver) at 10, 20, 30, 45, 55. Skill advances 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `keen_appraisal` | Keen appraisal | P | item descriptions show what things are worth | perception (`Stack::value`) |
| 6 | `whetstone` | Whetstone | P | weapons you wield +1 damage | `Mods::local` |
| 12 | `fletching` | Fletching | P | missiles you shoot or throw break half as often | `shoot.rs` `breaks` |
| 18 | `sturdy_work` | Sturdy work | P | the body armour you wear +1 Protection | protection sum |
| 25 | `mining` | Mining | P | digging ×1.5 | `dig_work` |
| 32 | `tinker` | Tinker | A | Disarm traps (the thief's action) | `action` disarm |
| 40 | `jeweller` | Jeweller | P | gems you find are one cut finer | `gem::find` (loot stream, deterministic) |
| 50 | `masterwork` | Masterwork | P | weapons you wield threat +1 | `Mods::local` |
| 60 | `artisan` | Artisan | P | every piece of armour you wear +1 Protection; weapons +2 damage | mods |

### 4.16 Merchant

Bonus talents (Delver, Scholar) at 10, 20, 30, 45, 55. Skill advances 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `haggler` | Haggler | P | shop favour +15 % | `favour` |
| 6 | `deep_pockets` | Deep pockets | P | carrying limit +10 kg; coins weigh nothing | `burden::load`, `Stack::weight` |
| 12 | `eye_for_quality` | Eye for quality | P | an item you buy or sell is identified | `shop.rs::pay`, `trade` |
| 18 | `trade_routes` | Trade routes | P | overworld journeys 20 % quicker | `step_price` |
| 25 | `credit` | Credit | P | keepers let you leave owing up to 100 gold; the debt stays on their bill until you pay | `World::turned_back` |
| 32 | `appraiser` | Appraiser | P | item descriptions show worth; gems' names show theirs | perception |
| 40 | `insured` | Insured | P | shops pay half again as much for what they buy (never more than they ask) | `Shops::offer` |
| 50 | `magnate` | Magnate | P | quest gold ×2; the favour cap is 60 % | quest, `MAX_FAVOUR` |
| 60 | `golden_tongue` | Golden tongue | P | Presence +2; every keeper gives you 10 % off | `Stats`, `favour` |

### 4.17 Counting

15 classes × 9 entries = 135 class abilities, of which 27 are active (28 menu entries: `flame_and_frost` is two) and 108 passive; 4 need a new small action (disarm, pick, steal, throw_voice), 9 are caster placeholders on the shared cast code (`spell`), and 1 (`blight`) wants #151 for its type but works untyped. Shared entries (`evasion`, `woodland_stride`, `divine_health`, `second_wind`, `smite`, `shield_of_faith`, the worth-showing ones) are one implementation each, named in several tables.

## 5. Skill increases at level-up

### 5.1 Today, and the ADOM reference

Skills grow today only from **training marks** (`world/train.rs`: a chance per meaningful use, drawn from the `training` stream; `need(value) = marks + value / per`, ×3 past the soft cap 100), a rule the spec marks **provisional** pending exactly this decision. ADOM does both: skills train "passively as the PC successfully uses them, and actively on level-up", where the level-up gives a number of increases (3 at least, plus one per 5 points of Learning over 10, more for humans and bards) and each skill rolls its own die (a ladder from +1 to +4d5) that *degrades* as the skill climbs and is *raised back by successful use*. Its known faults: the count leaks a hidden stat (Learning), and the guidebook's advice is to "not waste level advances" on skills that train by themselves, which makes the level-up a puzzle of what not to pick.

### 5.2 The proposal: two channels, one skill

Keep the marks as they are (they are the "by use" channel, already tuned, already deterministic), and add a **level-up channel** of **skill advances** that is coarse, quick to spend, and favours the skills the player actually used:

| Rule | Value | Where |
|---|---|---|
| Advances per level | the class's `advances` (2 for most classes; **3** for the thief, ranger, and bard; see §4), **+1 while Intelligence is 16 or more** (a score, which the player sees; never the modifier), **+1 for humans** (their "room to grow"), +1 for a bard from level 6 (`wide_learning`), +1 with the Quick learner talent | `data/classes/*.toml`, `races.toml`, the talent |
| What an advance may go into | any **trained** skill (an untrained skill does not exist for the character; the Apprentice talent (§7) and class tables are the ways to train a new one) | `Actor::skills` |
| How much it gives | the class's die for that skill: **`1d4+2`** for a skill the class is known for (its `2d6+10` list, now named `known_for`), **`1d4`** for any other skill the class or the race trains, **`1d3`** for a skill trained any other way; **past 100** the roll is halved, rounded up (the soft cap, as for marks) | `data/classes/*.toml` `[advance]` |
| Use makes the die kinder | a skill that **gained at least one point from marks since the last level-up** rolls its die **twice and keeps the better** (ADOM's "raised by successful use", without its degrading ladder) | `Skills::used_since_level` flag per skill, cleared at level-up |
| Unspent advances | **bank**, without limit or expiry (nothing forces a pick: the owner's rule) | `Progress::pending.advances` |
| What is said | "Your stealth improves." (the marks' phrase; never a number beyond the value on the Skills screen) | the level-up screen |
| Randomness | the dice draw from a new **`levels` stream** in the world state (like `training`), never the gameplay stream, so spending advances never moves a fight; each `advance:SKILL` is an action recorded in the replay, so the result is reproducible | `WorldState::levels_rng` |

**Why both.** Marks reward doing; advances reward surviving. A thief who never kicks a door still becomes a better burglar by levelling, as ADOM's does, but one who sneaks all day gets more out of each advance in Stealth. The two channels never double count: the "used" flag is only a reroll, not extra points.

**Numbers in play.** A thief (3 a level, say 2.5 with the odd In bonus) spends about 150 advances by level 50, each about 4 points in a known-for skill (4.5 average, more with the reroll): roughly 600 points, enough for three skills at 100 and the rest in the 40s, on top of what marks bring. A warrior (2 a level) spends 100: two masteries. With the halving past 100 the way from 100 to 150 costs as much as from 0 to 100.

### 5.3 Marks keep their rule

The provisional `[increase]` block stays as it is (`marks = 4, per = 10, soft_cap = 100, past_cap = 3`): nothing here needs a change to it, and the Studious talent is the one knob on it. If play shows the two channels together grow skills too fast, the first knob to turn is the class dice, not the marks.

### 5.4 The level-up screen's skills step

A paged list of the trained skills (the one item-list widget, §8): each row the skill's name, its value, a `•` where it was used since the last level (the kinder die), and "+N" after a spend this session; the frame says "Skill advances: 2 left"; a letter spends one on that row (`Action::Advance(skill)`, no time, said at once: "Your stealth improves."); Esc leaves the rest banked. No dice, no mark counts, no "good/other" labels: the wiki's Classes page carries the generated dice table.

## 6. Attribute growth

### 6.1 The options

| Model | Where | What players say | Fit with Warlock |
|---|---|---|---|
| **None at level-up**; potions and training by use (Angband, ADOM) | ADOM trains St by staying Burdened (capped at 15 whatever the potential), To by herbs, Le by reading; Angband by potions to 18/100 | ADOM's training is the canonical "stat pumping" grind, retreated from in 1.2.0; a verified complaint is that the cap bites "regardless of your potential" | we already have corpses raising scores toward the potential; adding use-training would add the grind the owner's "time is the penalty" rule tolerates but nobody praises |
| **A point every few levels** (D&D 3.5 and Pathfinder: +1 at 4, 8, 12, 16, 20; DCSS: one every 3–5 levels by species; Qud: one every 3) | chosen by the player | accepted everywhere; the 5e "feat or stat" tension is the one disliked form, and only because the two compete | cheap, transparent, data-driven; a stat is one of six, so 15 points over 60 levels is an average of +2.5 a stat |
| **Several points every level** (ToME: 3) | chosen | "number go up" but the "stat inflation" complaint (scores ten times the start) | our scores run 1–40 with D&D modifiers: inflation would break every hidden modifier's meaning |

### 6.2 Recommendation

**Yes, modestly: the player raises one stat of their choice by 1 at levels 4, 8, 12, …, 60** (15 points in all; `stat_every = 4` in `data/experience.toml`), chosen on the level-up screen (`Action::RaiseStat(stat)`, replay text `raise:strength`, no time; "You feel stronger." / "quicker" / "tougher" / "smarter" / "more resolute" / "more confident"). Unspent raises bank.

**Potentials.** A level-up raise **raises the score and, where the score has reached it, the potential too**. Two reasons: a raise that could be refused by a hidden cap would reveal the potential at that moment (the owner's hidden-numbers rule), and the growth of a long career is the one thing that should outrun one's nature. Corpses and magic items keep their rules (never past the potential; items raise the effective score only). The potentials' role stays what it is: the ceiling for the cheap sources.

**What this does to the numbers.** A human warrior with St 14 at level 1 who puts every raise into Strength reaches 29 at level 60 (+9 modifier); spread evenly, each stat gains 2–3. Hit points follow Toughness raises through `raised_stats`, as they do for items, and so do carrying limits and checks. No other D&D concept comes with it: no ability checks, no saving throws.

**Not proposed (and why):** training by use (grind); stat talents (+1 St as a talent pick is the dull pick every review warns of; the budget above is separate so talents never compete with stats, the lesson of 5e); ADOM's class stat gifts are kept to two class entries (`mighty_thews`, `unlikely_hero`) and `legend`/`golden_tongue`'s Presence, which are flavour, not the plan.

## 7. Talents

### 7.1 The name

**Talents.** ADOM's word, and the owner's; "feats" carries D&D's rules baggage (and 3.5's reputation for feat taxes), "perks" is Fallout's. In the code: `Talent`, `TalentId`, `data/talents/`. The abilities menu's groups keep their names: a talent that is active appears there under a new `Source::Talent` group ("Talents"), between Class and Skills.

### 7.2 How many, and when

| When | Picks | Notes |
|---|---|---|
| Character creation | **2** (humans 3: their racial `versatile`) | from the talents whose prerequisites the drafted character meets; ADOM gives 1 (up to 7 with bonuses), D&D 1 (humans 2) |
| Levels 3, 6, 9, …, 60 | **1 each: 20** | ADOM: every 3rd to 50 (16); 3.5: every 3rd (7 by 20) |
| Class bonus picks (§4) | 8 (warrior, thief) or 5 (the rest) | from the class's named trees only; Pathfinder 2e's lesson: separate pools never compete |
| In all by level 60 | **27–30** | of about 90 talents, so a career takes a third of the list, and no two builds need to look alike |

A pick never expires and is never forced: the level-up screen can be left and the picks stay banked (the status line of the Talents page says "1 talent to choose"). There is no respec (ADOM, DCSS: a choice is a choice); open question 8.

### 7.3 Prerequisites

Each talent lists what it needs, all of which must hold when it is chosen (D&D's rule that a prerequisite lost later disables the talent is **not** adopted: a Strength lowered by disease does not switch Cleave off; the owner's rule that hidden numbers never show rules out the ambiguity that would make):

| Kind | Written as | Read from | Shown to the player |
|---|---|---|---|
| another talent (trees and chains) | `→ dodge`, `→ hardy ×2` (rank 2 of it) | the character's talents | yes |
| a stat **score** | `St 13` | the stored score (never the effective one, never a modifier), as it stands at the pick | yes (scores are shown) |
| a skill **value** | `tumble 25` (trained, at least that) | `Actor::skills` | yes (values are shown) |
| a weapon skill or style **rank** | `bows 3` | `Actor::weapon_skills` | yes |
| a character **level** | `Lv 9` | the level | yes |
| the **race** | `race dwarf` | `Character` | yes |
| the **eyes** (vision profile) | `eyes darkvision` | the race's profile (the base, not the talents' changes) | yes |
| the **class** | `class warrior` | `Character` | yes (rare: classes get their own tables) |

Prerequisites are deliberately shallow: **at most two steps in a chain**, and every link useful on its own (the paizo threads' "feat tax" and ADOM's "two prerequisite talents of minimal value" for Treasure Hunter are the failure both research reports flag). Stat prerequisites use round, visible scores (12–16) so a character can plan toward them with the level-up raises.

### 7.4 Ranks and choices

- **Ranks**: a talent may have up to **3 ranks** (`ranks = 3`); each rank is a pick and may carry its own prerequisites (`Hardy`: To 11 / 14 / 17); shown as "Hardy (2/3)". Chains are preferred where the steps read as different things (Dodge → Deflect missiles); ranks where they are plainly "more of it" (ADOM's Hardy/Very Hardy/Extremely Hardy becomes one talent).
- **Choices**: a talent may take a parameter (`choice = "weapon_skill"`, `"style"`, `"element"`): Weapon focus (long blades) and Weapon focus (bows) are two picks of the same talent with different choices; replay text `talent:weapon_focus:long_blades`.
- **Scaling**: numeric effects scale with level or with what they modify wherever that reads naturally (+1 HP per 4 levels, percent of healing, "one rank higher"), so a talent taken at level 3 still matters at 50 (the ADOM complaint "3 HP from Hardy is negligible for a PC with over 100 health").

### 7.5 The list

Notation: **Needs** as in §7.3; **R** ranks (blank = 1); **A/P** active (an entry in the abilities menu, `Source::Talent`) or passive; **Hook**: the one rule that reads it. `#151` marks talents that wait for damage types; `action` ones that need a small new action (§9). Numbers are first guesses in data.

#### Vigour (hit points, healing, constitution)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `healthy` | Healthy | | — | hit points come back 25 % faster | `World::healing_rate` | P |
| `hardy` | Hardy | 3 | To 11 / 14 / 17 | +1 max HP per 4 character levels per rank, at least +3 per rank, now and at every level after | `Actor::max_hp` | P |
| `fast_healer` | Fast healer | | → healthy, To 13 | resting heals half again as fast (×3 in all) | `healing_rate` while `resting` | P |
| `second_wind` | Second wind | | To 13, Lv 6 | regain 1 HP per level (at least 8) at once; Winded for 15 min, during which no second wind | B1 status; cost `wait` | **A** |
| `iron_stomach` | Iron stomach | | To 12 | rotten food never makes you vomit and harms you half as much; overeating never makes you vomit | `eat.rs::finish` | P |
| `poison_resistance` | Poison resistance | | To 14 | Poisoned eases twice as fast and never lowers your scores | `Effect::until_eases`, `Effects::penalty` | P |
| `clotting` | Clotting | | To 12 | a big hit gives you no bleeding (a critical one degree less); bleeding eases twice as fast | `World::after_blow`, easing | P |
| `strong_constitution` | Strong constitution | | To 13 | +2 on disease checks (catching and progressions) | `check` modifier in `disease.rs` | P |
| `fever_proof` | Fever-proof | | → strong_constitution, Lv 15 | one check won shakes a disease off (`recovery` one less, never below 1; grave rot still never) | `disease_turn` | P |
| `unbowed` | Unbowed | | → hardy ×2, Lv 20 | status effects' penalties to your scores are halved (toward 0) | `Effects::penalty` | P |

#### Might (Strength, blows, breaking things)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `brawn` | Brawn | 2 | St 12 / 15 | +1 melee damage per rank | `Profile::melee` | P |
| `cleave` | Cleave | | St 13, a melee skill at 3 | a melee blow that kills gives a free blow at another adjacent hostile | B3 `second_blow` | P |
| `great_cleave` | Great cleave | | → cleave, Lv 18 | cleaving goes on while the blows kill (at most once per adjacent hostile a turn) | B3 | P |
| `mighty_kick` | Mighty kick | | St 13 | kicks at creatures 1d6 instead of 1d3; door kicks as if 4 stronger | `player_kicks`, `kick_door` | P |
| `breaker` | Breaker | | → mighty_kick, St 14 | door kicks as if 4 stronger again; digging ×1.25 | `kick_door`, `dig_work` | P |
| `iron_grip` | Iron grip | | St 13, a two-handed weapon skill at 3 | blows with two-handed weapons 10 % cheaper | `World::attack_price` | P |
| `staggering_blow` | Staggering blow | | St 15, Lv 12 | a critical you confirm in melee leaves the target Staggered: its next action costs half again as much | B1 status on creatures (`cost`) | P |
| `crusher` | Crusher | | St 13, maces or unarmed at 3 | bludgeoning blows ignore 2 Protection (`penetration` +2) | `Profile` | P `#151` |

#### Finesse (Dexterity, dodging, quickness)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `dodge` | Dodge | | Dx 13 | Dodge +1 | `Actor::defence` | P |
| `nimble` | Nimble | | Dx 13 | the burden's cap on Dexterity in Defence is 1 higher at every level | `Burden::max_dex` | P |
| `light_step` | Light step | 2 | Dx 12 / Dx 14, Lv 10 | a trap you step onto gets a Dexterity check against its difficulty: won, it does not go off under you (it stays, found); rank 2: pits never catch you | `spring_trap` for the player | P |
| `quick` | Quick | 2 | Dx 14, Lv 9 / Dx 16, Lv 24 | you move and act a little quicker (speed +5 % per rank; the number is hidden, as speed is) | `Actor::speed` | P |
| `deflect_missiles` | Deflect missiles | | → dodge, Dx 15, Lv 12 | Dodge +2 against shots, throws, and trap arrows | B5 | P |
| `evasion` | Evasion | | Dx 15, Lv 15 | bursts and bolts (traps, scrolls, spells) do you half damage | `cast_*` victim hook | P |
| `riposte` | Riposte | | Dx 13, a melee skill at 5, Lv 12 | a blow that misses you by your Parry gives a free blow back | B3, `why_missed` | P |
| `opportunist` | Opportunist | | Dx 12, Lv 9 | a creature that steps away from beside you (flees) takes a free blow | B3, `World::flees` | P |

#### Weaponcraft (weapon skills, styles, missiles)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `weapon_focus` | Weapon focus (*skill*) | | choice: a weapon skill at 2 | +1 to hit with it | `Profile` | P |
| `weapon_specialization` | Weapon specialization (*skill*) | | → weapon_focus (same), rank 5 | +2 damage with it | `Profile` | P |
| `improved_critical` | Improved critical (*skill*) | | a weapon skill at 7 | threat +1 with it | `Profile` | P |
| `stylist` | Stylist (*style*) | | choice: a style at 3 | the style's benefits read one rank higher (never past 10) | `Actor::attack_benefit` | P |
| `two_weapon_training` | Two-weapon training | | Dx 13, twin_blades 2 | both two-weapon penalties eased by 2 more | `TwoWeapons::eased` | P |
| `shield_expert` | Shield expert | | shield_block 2 | Shield +1; Sword and Board's free bash 10 more likely | `Actor::defence`, `second_blow` | P |
| `deadly_aim` | Deadly aim | | Dx 13, a launcher skill at 3 | +2 damage with shots | `Profile::ranged` | P |
| `far_shot` | Far shot | | St 12 | launchers reach 2 cells farther, throws 1 | `launcher_range`, `throw_range` | P |
| `thrower` | Thrower | | thrown 2 | thrown weapons +1 damage, throws 10 % cheaper | `Profile::thrown`, `attack_price` | P |
| `rapid_reload` | Rapid reload | | crossbows 3 | reloading 25 % cheaper | `attack_price` | P |

#### Guard (armour, Protection, standing firm)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `armour_training` | Armour training | 2 | To 12 / St 13 | worn armour's `max_dex` +1 per rank | `Actor::defence` | P |
| `heavy_armour_mastery` | Heavy armour mastery | | → armour_training ×2, Lv 12 | Protection +1 while the body armour worn has Protection 3 or more | protection sum | P |
| `thick_skin` | Thick skin | 3 | To 12 / 15 / 18 | Natural armour +1 per rank (ADOM's Tough, Iron, Steel Skin) | `Actor::defence` (Natural) | P |
| `shield_wall` | Shield wall | | shield_block 3 | Protection +1 while holding a shield | protection sum | P |
| `deflection` | Deflection | | Wi 13, Lv 9 | Deflection +1 | `Actor::defence` | P |
| `stalwart` | Stalwart | | → armour_training | the defensive stance costs you no to-hit | `StanceDef::apply` | P |
| `unflinching` | Unflinching | | Wi 12 | the coward stance's to-hit penalty is halved, and its fleeing steps are 10 % cheaper again | `StanceDef::apply`, `World::flees` | P |
| `immune_to_pain` | Immune to pain | | → thick_skin ×2, To 16, Lv 20 | every hit on you does 1 less after the soak (never below 1) | `hurt` | P |
| `elemental_ward` | Elemental ward (*element*) | 2 | To 12 / Lv 20 | 25 % resistance per rank to fire, cold, or electricity | resistances | P `#151` |

#### Shadow (stealth, hearing, eyes, light)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `silent_feet` | Silent feet | 2 | Dx 12 / stealth 25 | footsteps 20 % quieter per rank; rank 2: the doors you open and close are silent | `noise::noise` | P |
| `keen_ears` | Keen ears | 2 | — / listen 20 | you hear sounds 2 fainter per rank | `Actor::hearing` | P |
| `night_eyes` | Night eyes | | eyes normal | your eyes need half the light (threshold 40 → 20) | B4 | P |
| `dark_adaptation` | Dark adaptation | | eyes darkvision | darkvision 2 cells farther | B4 | P |
| `owl_eyes` | Owl eyes | | eyes low_light, Lv 9 | darkvision 2 cells | B4 | P |
| `rune_sight` | Rune sight | | In 14, Lv 15 | you see UV light | B4 (`sees_uv`) | P |
| `hooded_light` | Hooded light | | — | a toggle: your lantern burns hooded (half its radius; a dim preset), so others see you from less far | `action` hood; the lantern's preset | **A** |
| `lamplighter` | Lamplighter | | — | your lantern reaches one cell farther (a brighter preset) | the lantern's preset | P |
| `ambusher` | Ambusher | | stealth 30, Lv 9 | +1d6 damage on a blow or shot at a creature that has not seen you | `Profile` from `Mind` | P |
| `blood_scent` | Blood scent | | eyes darkvision | you see bleeding creatures in the dark within 4 cells | perception (status `seen`) | P |

#### Wayfarer (the land, water, ice, weather)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `long_strider` | Long strider | | — | overworld journeys 15 % quicker | `step_price` | P |
| `pathfinder` | Pathfinder | | → long_strider, survival 20 | forest, hills, and swamp cost you as plains do on the overworld | `step_price` (`travel_minutes`) | P |
| `swimmer` | Swimmer | | swim trained | wading costs you as a plain step; a plunge through ice never hurts you | `step_price`, `after_tread` | P |
| `winter_born` | Winter-born | | To 12 | ice holds you as if you carried nothing; climbing out onto ice costs half | `World::tread`, `tread_price` | P |
| `beast_friend` | Beast-friend | | Wi 13 | creatures of animal factions (vermin, deer) are neutral to you until you attack one of that type (what the druid has from level 1) | `World::stance`, the progress flags | P |
| `marsh_walker` | Marsh walker | | To 12 | wading never gives you a disease; marsh and reeds cost you as plain ground | `World::wade`, `step_price` | P |
| `weather_wise` | Weather-wise | | Wi 12 | you know the weather of the next three hours (the status bar's weather segment says what is coming) | perception, `weather_at` | P |
| `sun_blessed` | Sun-blessed | | — | under the open sun hit points come back half again as fast | `healing_rate`, `sun().is_up` | P |
| `moon_eyed` | Moon-eyed | | eyes low_light, or race elf | your overworld sight at night is one cell longer | `daylight::overworld_sight` | P |

#### Scholar (items, scrolls, gems, learning)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `lore` | Lore | | In 12 | a magic item you have carried for a day is identified | `World::identify` from `perceive` | P |
| `keen_appraiser` | Keen appraiser | | appraise 15 | item descriptions show what things are worth (gems too) | `PerceptionView` (value) | P |
| `scroll_savant` | Scroll savant | | In 13 | reading a scroll costs half | `read` price | P |
| `careful_reader` | Careful reader | | → scroll_savant | your burst scrolls never catch you | `cast_burst` | P |
| `spell_recall` | Spell recall | | In 15, Lv 12 | one read in four, the scroll is not used up (one gameplay draw per read) | `read.rs` | P |
| `treasure_hunter` | Treasure hunter | | In 12 or appraise 20 | loot you find and drops roll magic half again as often, rare too (the weights; from the loot streams, so maps never change) | `magic::loot::roll` rarity weights | P |
| `good_learner` | Good learner | 2 | In 12 / In 16 | XP +5 % per rank | `grant_xp` | P |
| `quick_learner` | Quick learner | | In 14 | +1 skill advance every level | §5 | P |
| `apprentice` | Apprentice (*skill*) | 3 | — | train a skill you are untrained in, at 1d6+4; or +1d6+4 to one you are trained in | `Actor::skills` (dice from the `levels` stream) | P |

#### Grit (hunger, burden, will)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `lean` | Lean | | — | hunger burns 15 % slower; exertion costs nothing | `World::digest`, `exert` | P |
| `pack_mule` | Pack mule | 2 | St 11 / St 14 | carrying limit +10 % per rank | `burden::load` | P |
| `porter` | Porter | | → pack_mule ×2, To 12 | Burdened costs you nothing (as a dwarf's) | `Load::penalty` | P |
| `iron_will` | Iron will | | Wi 13 | +2 on Willpower checks | `check` | P |
| `gourmand` | Gourmand | | To 12 | a corpse's chances to raise a stat are doubled; a meal never leaves you Bloated (what does not fit is lost) | `eat.rs::finish` | P |
| `sure_footed` | Sure-footed | | Dx 12 | Bloated and the burden's slowdown are halved for your steps | `World::burdened` (steps) | P |

#### Delver (traps, searching, doors, digging, shops, talk)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `alert` | Alert | | Wi 12 | you notice hidden things in passing (as if trained in Spot), +2 | `search.rs::notices` | P |
| `keen_searcher` | Keen searcher | | search 15 | searching +4 and it reaches one cell farther | `search_modifier`, reach | P |
| `trap_sense` | Trap sense | | → alert | hidden traps and secret doors are 3 easier for you; a trap you set off does you half damage | `difficulty_of`, `trap_deals` | P |
| `lockbreaker` | Lockbreaker | | St 12 | door kicks as if 6 stronger, half as loud | `kick_door`, `noise` | P |
| `tunneler` | Tunneler | | St 12 | digging 50 % faster | `dig_work` | P |
| `silver_tongue` | Silver tongue | | Pr 12 | shop favour +10 % | `world::favour` | P |
| `haggler` | Haggler | | → silver_tongue, appraise 20 | favour +10 % more, and the cap is 60 % | `favour`, `MAX_FAVOUR` | P |
| `well_liked` | Well liked | | Pr 13 | quest rewards' gold and XP +25 % | quest reward | P |

#### Racial (each needs its race; `versatile` is granted, not picked)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `elven_accuracy` | Elven accuracy | | race elf, bows 3 | threat +1 with bows | `Profile` | P |
| `ageless` | Ageless | | race elf, Lv 9 | hunger burns 20 % slower; diseases progress every second day | `digest`, `disease_turn` | P |
| `dual_heritage` | Dual heritage | | race half_elf | +1 skill advance every level | §5 | P |
| `diplomat` | Diplomat | | race half_elf, Pr 11 | shop favour +10 %; quest gold ×1.25 | `favour`, quests | P |
| `versatile` | Versatile | | race human | **granted at creation**: one more talent pick at creation and at level 30 | `Progress::pending` | P |
| `stone_sense` | Stone sense | | race dwarf | noticing what is hidden in stone +4; digging ×1.25 | `search_modifier`, `dig_work` | P |
| `dwarven_resilience` | Dwarven resilience | | race dwarf, To 14 | poison eases twice as fast; +2 on disease checks | status, `check` | P |
| `stout` | Stout | | race dwarf, Lv 12 | Strained costs you what Burdened costs others | `Load::penalty` | P |
| `lucky` | Lucky | | race halfling | a natural 2 on your attack roll is rolled again, once (a gameplay draw only then) | `combat::strike` | P |
| `slinger` | Slinger | | race halfling, slings 2 | slings +1 damage and 2 cells of range | `Profile`, `launcher_range` | P |
| `small_target` | Small target | | race halfling, Dx 13 | Dodge +1 against creatures larger than you | B5 (by attacker size) | P |
| `tinker` | Tinker | | race gnome, In 12 | Disarm traps (the Disable device action) | `action` disarm | **A** |
| `gem_lore` | Gem lore | | race gnome | gems' names show their worth; you find gems twice as often | perception, `gem::find` chance | P |
| `ferocity` | Ferocity | | race half_orc | below a third of your hit points, +2 damage | `Profile` | P |
| `orcish_hide` | Orcish hide | | race half_orc, To 13 | Protection +1 | protection sum | P |

**Count**: 10 + 8 + 8 + 10 + 9 + 10 + 9 + 9 + 6 + 8 + 15 = **102 talents** (2 of them `#151`), 3 active (`second_wind`, `hooded_light`, `tinker`), 6 with a choice, 12 with ranks. Trimming to 80 for the first release is easy (the last two of each tree), but the list is written so every tree has a path in it and every system in §8 has its two.

### 7.6 What was left out, on purpose

- **Stat talents** (+2 St): ADOM's "useless except as prerequisites"; stats grow on the level-up (§6).
- **Talents that gate ordinary actions** (D&D's Power Attack, Weapon Finesse): everyone can already kick, bash, throw, search, dig; talents make a thing better, never possible for the first time, except where the action does not exist yet (disarm, pick lock, steal) and the talent *is* the action's arrival for the few who would have it.
- **Level-1-only talents** (ADOM's Heir, Alert, the +2s): every talent can be taken whenever its prerequisites hold, so a build is never locked out at a screen the player has not learned to read.
- **Long chains**: no chain is longer than two steps.
- **Spell and mana talents**: with the spell system, as its coverage (§8.2).

## 8. Coverage

### 8.1 The matrix

Systems (rows) against what touches them: talents (§7) and class abilities (§4). The owner's worry is ADOM's: whole systems with nothing that cares about them. Every rule system that exists today has at least two talents and one class entry (the bar the test in §8.2 enforces); the rows nearest the bar are marked.

| System | Talents | Class abilities | Thin? |
|---|---|---|---|
| Combat: to-hit, damage, criticals | weapon_focus, weapon_specialization, improved_critical, brawn, deadly_aim, thrower, ferocity, elven_accuracy, lucky | weapon_mastery, champion, brutal_critical, big_hands, masterwork, whetstone, peerless, favoured_prey, master_hunter, perfect_self, martial_arts | |
| Combat: Defence categories (Dodge, Parry, Shield, Natural, Deflection) | dodge, deflect_missiles, thick_skin, shield_expert, shield_wall, deflection, small_target, riposte | shield_wall, shield_of_faith, thick_hide, diamond_soul, mage_armour, deflect_missiles, uncanny_dodge | |
| Protection (Pv), penetration | heavy_armour_mastery, shield_wall, immune_to_pain, orcish_hide, crusher (#151) | armour_drill, bulwark, thick_skull, sturdy_work, artisan | |
| Free blows (`second_blow`) | cleave, great_cleave, riposte, opportunist | cleaving_strike, flurry, riposte | |
| Weapon skills and marks | weapon_focus…, rapid_reload, two_weapon_training | weapon_drill, fencing, master_hunter | |
| Fighting styles | stylist, two_weapon_training, shield_expert | fencing, shield_of_faith | |
| Stances | stalwart, unflinching | iron_stance | at the bar |
| Ranged combat, missiles | deadly_aim, far_shot, thrower, rapid_reload, slinger, deflect_missiles | rapid_shot, volley, fletching, deflect_missiles | |
| Kicks, unarmed | mighty_kick, crusher | martial_arts, big_hands, flurry, quivering_palm, perfect_self | |
| Hit points, healing, regeneration | healthy, hardy, fast_healer, second_wind, sun_blessed | veterans_wind, hardened, song_of_rest, cure_wounds, lay_on_hands, salt_of_the_earth, saint, sun_and_moon | |
| Status effects (bleeding, poison; buffs) | poison_resistance, clotting, unbowed, staggering_blow | rage, blessing, inspire, evil_eye, hex_of_weakness, curse_of_sloth, poison_lore, quivering_palm, stubborn, unstoppable | |
| Diseases and checks | strong_constitution, fever_proof, iron_will, marsh_walker, dwarven_resilience, ageless | divine_grace, divine_health, purity_of_body, remove_disease, natures_ward, still_mind, timeless_body | |
| Hunger, food, corpses, rot | iron_stomach, lean, gourmand, ageless | hearty, purify, herb_lore, cauldron, timeless_body | |
| Burden and carrying | nimble, pack_mule, porter, sure_footed, stout | steadfast, uncanny_dodge, deep_pockets, hard_work, fast_movement | |
| Stealth and noise | silent_feet, ambusher | hide_in_plain_sight, camouflage, sneak_attack | |
| Hearing and sounds | keen_ears | tracker, throw_voice | |
| Light (carried, presets) | hooded_light, lamplighter | holy_radiance | at the bar |
| Vision profiles, darkvision, UV | night_eyes, dark_adaptation, owl_eyes, rune_sight, blood_scent, moon_eyed | sun_and_moon (night sight) | |
| Creature sight and awareness (`Mind`) | ambusher | sneak_attack, improved_sneak_attack, master_thief, throw_voice | |
| Traps | light_step, trap_sense, evasion, tinker | trapfinding, evasion, tinker | |
| Searching and secret doors | alert, keen_searcher, trap_sense, stone_sense | second_sight, trapfinding | |
| Doors, locks, kicking | lockbreaker, mighty_kick, breaker | pick_lock | |
| Digging | tunneler, breaker, stone_sense | mining | |
| Ice and cold water | winter_born, swimmer | winters_friend | |
| Wading and water terrain | swimmer, marsh_walker | woodland_stride, natures_ward | |
| Overworld travel, local areas | long_strider, pathfinder | swift_tracker, trade_routes, woodland_stride | |
| Weather, sun, moon | weather_wise, sun_blessed, moon_eyed | weather_sense, sun_and_moon, call_lightning | |
| Shops and favour | silver_tongue, haggler, keen_appraiser, treasure_hunter | haggler, credit, insured, magnate, golden_tongue, well_dressed, commanding, eye_for_quality | |
| Quests and talk, NPCs, displacement | well_liked, diplomat | silver_tongue, legend, folk_hero, noblesse, throw_voice | |
| Magic items and identification | lore, treasure_hunter, keen_appraiser | lore, arcane_lore, connoisseur, eye_for_quality | |
| Scrolls and the shared cast code | scroll_savant, careful_reader, spell_recall, evasion | scroll_adept, spell_recall, empower, the caster placeholders | |
| Gems | keen_appraiser, gem_lore | jeweller, appraiser | |
| Blood | blood_scent | — | exempt: blood is presentation state no rule reads (`blood_scent` reads the Bleeding status, not the stains); it is not in the systems list |
| Skills and training | apprentice, quick_learner, dual_heritage | wide_learning, jack_of_all_trades, commanding | |
| Experience | good_learner, well_liked | legend, folk_hero, legacy | |
| Stats and potentials | — (by design, §6) | mighty_thews, unlikely_hero, legend, golden_tongue | |
| Speed (hidden) | quick | fast_movement, fleet | |
| Factions and a creature's stance toward the player | beast_friend, well_liked (NPCs) | wild_empathy, noblesse, light_fingers (outrage) | at the bar |
| Damage types and resistances (#151) | elemental_ward, crusher | blight | pending |

The rows at the bar are the systems with the least surface (stances are four rows of data; carried light is a lantern and item glow). The policy below keeps every row at or above it.

### 8.2 Policy for new systems

Concrete, and checked by a test rather than by memory:

1. **Every talent and class ability carries `systems = ["traps", "searching"]`** in its data, from a closed list of system tags kept in `data/talents.toml` (`[systems]`, one line per system with a name for the wiki). A content test fails when a tag in the list has fewer than **2 talents** or fewer than **1 class ability**, and when a talent names a tag not in the list. Adding a system without touching the list passes (nothing requires it); adding it to the list without talents fails. So the rule is: **a system is "in the list" when it ships with its talents.**
2. **The issue template** (`.github/ISSUE_TEMPLATE/feature.md`, or the feature issue body convention in `spec/13-github-workflow.md`) gets a **"Talents and class abilities"** section: *"Which talents and class entries does this system ship with (at least two talents; name the trees), or why none?"* A feature PR that adds a system adds its tag and its talents in the same PR, as it updates the spec, CLAUDE.md, and the wiki today.
3. **The wiki's Talents page** is generated per tree with a column of systems, and a **"By system"** table (system → talents and class abilities) comes from the same tags, so a gap is visible to players and the owner alike.
4. **The coverage test's thresholds are data** (`min_talents = 2`, `min_class = 1`), so the bar can rise as the list grows.

Pending systems with their first talents already named here: damage types (#151: `elemental_ward`, `crusher`, `blight`'s type), the spell system (replaces the caster placeholders and gets a Spellcraft tree), pits and climbing (#128: `light_step` rank 2, Athletics), food freshness (#196: `forager` would go here), potions (a Vigour/Scholar talent each).

## 9. What the player sees

### 9.1 Messages

In the order they come, after the kill's own messages (as today's "You feel more experienced." / "Welcome to level 3."):

| Event | Message |
|---|---|
| XP gained from a quest or discovery | "You gain 15 experience." (quests say it today); "You learn something of kobolds." (first kill, with the XP); "You gain 20 experience for finding your way here." (arrival) |
| a level | "You feel more experienced." "Welcome to level 3." then one line for what waits: "You may choose a talent." / "You have 2 skill advances." / "You may raise a stat." (joined: "You may choose a talent and raise a stat, and you have 2 skill advances.") |
| the class table | "You have learned to cleave." (passive) / "You can now rage." (active: it is in the abilities menu) / "You may choose a bonus talent." |
| a stat raised | "You feel stronger." (St), "quicker" (Dx), "tougher" (To), "sharper" (In), "more resolute" (Wi), "more confident" (Pr) |
| a skill advanced | "Your stealth improves." (the marks' phrase) |
| a talent chosen | "You learn Cleave." / "You learn Hardy (2)." |
| an ability on cooldown | "You are still winded." (refused, no time) |

Never: a die, a modifier, a potential, a speed, a chance, a mark count.

### 9.2 The level-up screen

`ScreenName::LevelUp`, one screen on the one stack (`ui/level_up_screen.rs`), **opened by `screens::on_level_up`** at the end of the turn in which something came pending (like `on_death` opens the death screen), unless another screen is open (then after it closes), and **from the Talents page** (Enter on its "Choose…" row) or the pause menu while anything is pending. No time passes on it. It walks three steps, each the one paged list (§6 "Item lists"), skipping steps with nothing pending; the frame says which step and what is left ("Level 12 · Talent 1 of 2 · Skill advances 3 · Stat raise 1"):

1. **Raise a stat**: six rows (`St  Strength  14`), a letter raises one (`raise:strength`), said at once. Scores only, never potentials; a stat at its potential shows nothing different (the raise lifts both, §6).
2. **Advance skills**: the trained skills as in §5.4; one letter per advance.
3. **Choose a talent**: the talents the character may take now, grouped by tree under headers (as item lists group by class), each row the name, its rank if ranked ("Hardy 2/3"), and a short effect in words; talents whose prerequisites are not met are **not listed here** (the Talents page lists every talent with what it needs, so planning happens there, and the pick list stays short); a choice talent opens a second list (the weapon skills, the styles, the elements) in its place (`ScreenAction::Replace`). The selected row's full description shows below the list. A bonus pick from the class's trees is its own step with only those trees.

Every step's letter is an `Act(Action)` (`talent:ID[:choice]`, `advance:SKILL`, `raise:STAT`), applied at once, recorded in the replay, and the screen refreshes (`screens::refresh`) with the result, like the equipment screen. **Esc leaves** with the rest banked; a `MenuChoice` is never needed. Keys: the list commands (letters, `<`/`>` pages, arrows, Enter on the `>` row), all in the existing rebindable contexts; one new rebindable command, `KeyCommand::LevelUp` ("Level up: spend what is pending", id `level_up`), unbound by default (`UNBOUND`), so a player may bind it.

### 9.3 The Talents info page

`ScreenName::Talents`, **page 7** of the info pages (`PAGES`: Character, Inventory, Equipment, Skills, Weapon skills, Quests, **Talents**; tab label "Talents"), reached by `Page(7)` and Tab in the pages context and by a rebindable `KeyCommand::Talents` of its own (id `talents`), unbound by default like `Back`, so no new default key is taken in play. It shows:

- at the top, while anything is pending: `Pending: 1 talent, 3 skill advances, 1 stat raise · Enter to choose`;
- the character's talents, grouped by tree, each with its rank and its effect in words (from data: `description`);
- below them, dimmer, **every other talent** of the game by tree with what it needs (`Needs: Dexterity 13, Dodge`), what is met in the normal colour and what is not dimmed, so the player can plan a build without the wiki; 72 wide like the character screen, scrolling with ↑↓ where it does not fit, the selected talent's full description below.
- The class's coming abilities are **not** listed (the wiki has them: in the game the class table is learned by levelling, like ADOM's).

The abilities menu lists active talents under "Talents" and class actives under "Class", each greyed with its cooldown's reason while one lasts ("you are winded").

### 9.4 Character creation

A new step **Talents** after Class and before the Summary (`ScreenName::ChargenTalents`, a `ChoiceScreen` like the others, but a multi-pick: 2 rows marked, 3 for a human): the list of talents whose prerequisites the drafted character meets (the start is computed as the summary computes it: seed, choices, start date), grouped by tree, each with its one-line effect; a **Random** row (as the other steps have) draws the picks from the stream `character/talents/<race>,<gender>,<class>`, so a seed's random character is always the same; Esc/Back returns to Class with the picks kept in the draft (`ScreenStack::draft`). The Summary gains a `Talents  Hardy, Alert` line. The command line gets `--talents hardy,alert` beside `--character` (a `CharacterSpec` field: `dwarf,female,warrior` stays valid, no talents = none picked, which is allowed: a pick never expires, so they wait on the Talents page), and replays record them in `character` (`{race, gender, class, talents}`; old replays without the key load as before). The race and class steps add a line in words for the experience factor ("Levels come slowly." for 115 and above, "quickly" for 95 and below).

### 9.5 The character screen and the status bar

The character screen's `Lv  Level  3` / `XP` rows stay; a `Talents  7 (1 to choose)` row is added after them, and the Skills row count stays as is. The status bar gets nothing new (it is full), except that `Lv` already shows; a pending pick does not nag there.

### 9.6 What stays hidden

The rules the talents stand on keep their numbers where the game hides them today: speed (Quick says "you move a little quicker"), checks and modifiers (Strong constitution says "you shake off disease more easily"), potentials (the stat raise never mentions them), to-hit (Weapon focus says "you hit more often with long blades"). Where the game **shows** a quantity (Defence, Protection, hit points, carrying limit, prices in shops, XP), the talent says the number ("Dodge +1", "+10 % carrying", "XP +5 %"). The in-game text is the talent's `description` in data; the **wiki** tables show the rules in full (the +2 on disease checks, the percents), as the wiki shows skill benefits and stance numbers today. Creatures' talents (none yet) would never show.

### 9.7 Keys

All new commands are rebindable with stable ids in `settings/keys.rs`: `level_up` (unbound), `talents` (the page, unbound; `Page(7)` reaches it), the class and talent abilities (each an unbound `KeyCommand` with its ability's id, as the shield bash is), and the list commands are the existing ones. No new fixed key.

### 9.8 Wiki

| Page | Content | Generated tables (`xtask/src/wiki/`) |
|---|---|---|
| Experience (existing) | the curve to 60, the factors, discovery | `experience-levels.md` (to 60, with the factor examples), `experience-factors.md` (races and classes), `experience-discovery.md`, ZD table to 60 |
| Classes (existing) | the level tables | `class-levels/<class>.md` per class (level, name, what it does), `class-advances.md` (advances a level and the dice) |
| Races (existing) | the experience factor, racial talents | `races.md` gains the column; `talents-racial.md` |
| **Talents** (new, under Your character) | the rules of §7.2–7.4, then the trees | `talents-<tree>.md` per tree (name, ranks, needs, effect as a rule, systems), `talents-by-system.md` |
| Skills (existing) | the advances section | `skill-advances.md` |
| Your character (existing) | stat raises | — |
| Abilities (existing) | the class and talent actives | `abilities.md` (from the registry, as today) |

## 10. Data and code sketch

### 10.1 Data files

| File | Change |
|---|---|
| `data/experience.toml` | `max_level = 60`, `linear = 20`, `quadratic = 10`, **`cubic = 3`**; `[levels] talent_every = 3, stat_every = 4, creation_talents = 2`; `[discovery] first_kill = 100, arrival = 5, identify = 10`; `[[zero_difference]]` to 60; `creature_xp_guideline = [5, 10]` (slope, offset; the test's and the wiki's) |
| `data/races.toml` | per race: `experience = 140` (percent), `talents = ["versatile"]` (granted), `advances = 1` (humans, half-elves get theirs as talents) |
| **`data/classes/<id>.toml`** (one file per class; `classes.toml` goes) | what `[[class]]` has today, plus `experience = 125`, `known_for = [...]`, `[advance] per_level = 2, known = "1d4+2", trained = "1d4", other = "1d3"`, `bonus_talents = { levels = [10, 20, 30, 45, 55], trees = ["guard", "vigour"] }`, and `[[level]]` rows: `level = 6, ability = "shield_wall"` with the ability's own table (`[[ability]] id, name, description, kind = "passive" or "active", aim, cost, effect = {...}, cooldown = "winded", systems = [...]`, and for casters a `burst`/`bolt` table as scrolls have). Classes are numbered in path order like objects (README.toml) |
| **`data/talents.toml`** | settings: `[systems]` (the closed tag list with names), `[coverage] min_talents = 2, min_class = 1`, `max_ranks = 3`, the trees (`[[tree]] id, name, description`, in display order) |
| **`data/talents/<id>.toml`** (one file per talent, README.toml, numbered in path order like affixes) | `[[talent]] id, name, tree, ranks, choice, description` (words, in game), `needs = { talents = ["dodge"], stats = { dexterity = 13 }, skills = { tumble = 25 }, weapon_skills = { bows = 3 }, level = 9, race = "dwarf", eyes = "darkvision", class = "warrior" }` (per rank: `[[talent.rank]] needs = {...}, effect = {...}`), `effect = { kind = "...", ... }` (a typed enum, §10.3), `systems = ["traps"]`, `active = { aim = "nothing", cost = "wait", cooldown = "winded" }` |
| `data/status.toml` | B1: optional `to_hit`, `damage`, `dodge`, `parry`, `cost`, `blocks`, `follows`; the new statuses `raging`, `exhausted`, `winded`, `drained`, `spent`, `blessed`, `inspired`, `hoarse`, `hexed`, `weakened`, `slowed`, `staggered` |
| `data/factions.toml` | `animal = true` on the vermin (and the deer's) faction for `wild_empathy` |
| `data/skills.toml` | nothing: the `[increase]` block stays |

Why a directory for talents and classes: both are long tables that churn (every system adds a talent; class tables are 15 × 9 rows and will be retuned), the repo's rule for churning tables is one file per entity (objects, affixes, creatures, materials: `docs/dev/content.md`), and parallel PRs adding talents then never conflict. `talents.toml` and `experience.toml` stay single files: settings, not tables.

### 10.2 State

```
WorldState
  experience: u32                      (exists)
  progress: Progress                   (new; left out of the JSON while default)
    talents: Vec<Taken { id: String, rank: u8, choice: Option<String> }>   by id, like affixes: adding talents keeps saves
    pending: Pending { talents: u8, bonus: Vec<(u8 level)>, advances: u16, raises: u8 }
    claimed: u8                        the level whose gains have been handed out (a kill that spans levels hands out each)
    used_since_level: BTreeSet<SkillId>    the kinder-die flag (§5)
    once: BTreeSet<String>             one-per-run flags: "miracle", "hags_bargain"
    discovered: Discovered { kills: BTreeSet<CreatureId>, affixes: BTreeSet<String> }
    empathy_broken: BTreeSet<CreatureId>   wild_empathy
  levels_rng: Rng                      (new) the `levels` stream: advance dice, Apprentice's dice
Character (saved, in WorldState)
  talents: Vec<String>                 the creation picks (the spec, for replays and `--talents`)
Actor
  talents: Talents                     (new) the player's taken talents, resolved to TalentIds for the rules; creatures' empty (a creature type may list `talents` later with no new code)
  effects: Effects                     (exists; cooldowns and buffs live here)
```

`FORMAT_VERSION` bumps once for the lot (the thresholds alone change what level a saved XP total is). The class table's passive abilities are **not stored**: they follow from the class and the level, read through the same aggregation as talents (`Actor::talents` is the player's picks plus the class entries at or below their level, computed on load and on level-up). The state hash covers all of it.

### 10.3 Where the rules hook in

One aggregation, read where the affixes already are: `Talents::mods(content) -> TalentMods` (summed typed effects: `dodge: i32`, `natural: i32`, `protection: i32`, `max_hp_per_levels`, `healing_percent`, `resting_percent`, `step_percent`, `noise_percent`, `hearing: i32`, `favour_percent`, `xp_percent`, `check: PerStat<i32>`, `vision: VisionMods`, flags…), and per-attack ones through `Actor::attack_benefit`-style readers (`TalentMods::for_attack(kind, skill, style, target)`). Each existing hook gains one read:

| Hook (exists) | Reads |
|---|---|
| `Actor::effective_stats`, `raised_stats` | class stat gifts (stored in `Stats`, no read) |
| `Actor::max_hp` | `hardy` |
| `Actor::defence(kind, attacker)` | dodge, natural, deflection, parry, shield, burden cap (`nimble`, `uncanny_dodge`), per-kind (`deflect_missiles`, `small_target`) |
| protection sum (`combat::soak` caller) | `heavy_armour_mastery`, `shield_wall`, `orcish_hide`, class Pv |
| `Profile` builders (`melee`, `ranged`, `thrown`, `bash`, `kick`) | focus, specialization, threat, crit, brawn, deadly_aim, sneak/ambush (needs the target's `Mind`), favoured prey, ferocity |
| `World::attack_price`, `step_price`, `price_of` | iron_grip, rapid_shot, thrower, rapid_reload, fast_movement, fleet, woodland_stride, long_strider, swimmer, marsh_walker, quick (speed) |
| `World::second_blow` (B3) | cleave, great_cleave, riposte, opportunist, flurry, cleaving_strike |
| `World::healing_rate` | healthy, fast_healer, song_of_rest, sun_blessed, saint, salt_of_the_earth |
| `World::afflict` / `after_blow` / `Effect::until_eases` / `Effects::penalty` | clotting, poison_resistance, unbowed, stubborn, still_mind, staggering_blow, the buffs |
| `check` callers in `disease.rs`, `search.rs`, `door.rs` (checks only where they exist) | strong_constitution, iron_will, divine_grace, dwarven_resilience, still_mind |
| `World::expose`, `wade`, `disease_turn` | divine_health, purity_of_body, natures_ward, marsh_walker, fever_proof, ageless, timeless_body |
| `noise::noise`, `Actor::hearing` | silent_feet, keen_ears, camouflage, hide_in_plain_sight, tracker, lockbreaker |
| `World::eyes()` (B4) | night_eyes, dark_adaptation, owl_eyes, rune_sight; `daylight::overworld_sight`: moon_eyed, sun_and_moon |
| `search.rs` (`search_modifier`, `notices`, `difficulty_of`) | alert, keen_searcher, trap_sense, stone_sense, second_sight, trapfinding |
| `spring_trap`, `trap_deals`, `cast_burst`/`cast_bolt` | light_step, trap_sense, evasion, careful_reader, scroll_adept, empower |
| `kick_door`, `dig_work`, `tread`/`tread_price`/`after_tread` | mighty_kick, breaker, lockbreaker, tunneler, mining, winter_born, winters_friend, swimmer |
| `burden::load`, `Load::penalty`, `World::burdened` | pack_mule, porter, stout, steadfast, hard_work, sure_footed, deep_pockets |
| `World::digest`, `exert`, `eat.rs::finish` | lean, hearty, iron_stomach, gourmand, purify, herb_lore, cauldron, ageless |
| `world::favour`, `Shops::offer`, `turned_back`, quest reward | silver_tongue, haggler, well_liked, the merchant's and aristocrat's |
| `grant_xp`, `earn_kill`, `travel` (arrival), `identify` | good_learner, legacy, discovery |
| `magic::loot::roll`, `gem::find` | treasure_hunter, gem_lore, jeweller (from the loot streams: the map is unchanged) |
| `World::identify` from `perceive` | lore, arcane_lore, connoisseur (a `Stack::carried_since` is needed: one new field on the stack, saved) |
| `World::stance`, `World::refusal` | wild_empathy, noblesse |
| `hurt`/`dies` | immune_to_pain, unstoppable, miracle, hags_bargain |

New modules: `content/talent.rs` (defs, prerequisites, effects, the systems list, the coverage test), `talents.rs` (the runtime: `Talents`, `TalentMods`, `may_take`), `content/class.rs` (grows the level table and abilities), `world/level_up.rs` (pending budgets on level gain, `Action::Talent`, `Advance`, `RaiseStat`, the class table's hand-outs and messages), `world/experience.rs` (the factor, discovery), `ability.rs` (`Source::Talent`, class actives, `Unavailable::While`), `ui/level_up_screen.rs`, `ui/talents_screen.rs`, `ui/chargen_screen.rs` (the step), `xtask/src/wiki/{talents,classes}.rs`, and `warlock-cli` dump layers `talents` (the player's, with ranks and choices) and `progress` (pending, discovered; dev).

### 10.4 Determinism

| Draw | Stream | Note |
|---|---|---|
| skill advance dice, Apprentice's dice | **`levels`** (new in `WorldState`, like `training`) | never the gameplay stream: spending advances moves no fight |
| creation picks' Random row | `character/talents/<spec>` | the picks themselves are recorded in `character`, so the stream is the UI's only |
| talents with a chance in play (`lucky`, `spell_recall`, `light_step`'s check, `staggering_blow` through the critical) | the **gameplay** stream, drawn only when the condition arises | the same rule as checks: a run without the talent draws nothing extra |
| discovery XP, class hand-outs, stat raises, the thresholds | no draw | |
| loot talents (`treasure_hunter`, `gem_lore`, `jeweller`) | the `loot/<level>` and `drop/<actor>` streams, as today | they change what is rolled, never the map; a level generated before the talent was taken keeps its loot |
| cooldowns | game time (statuses) | |

Every pick is an action with replay text (`talent:hardy`, `talent:weapon_focus:long_blades`, `advance:stealth`, `raise:strength`), `Spent::NoTime`, refused with a message when nothing is pending or the prerequisites fail ("You cannot take Cleave yet: you need Strength 13."). The hash covers `progress`, `levels_rng`, and `Actor::talents`; a replay without picks banks them, so old replays play as before but for the thresholds (their hashes change with the format bump anyway).

## 11. Implementation plan

PR-sized issues, in dependency order; letters are issues, arrows dependencies. Items in the same row can run in parallel (the owner's cap of seven agents and the merge-conflict rules in `docs/dev/merging.md` apply: no two issues in flight touch the same class or talent files).

| # | Issue | Depends on | Size | Notes |
|---|---|---|---|---|
| A | **Experience to 60**: `cubic`, the race × class `experience` factors, ZD to 60, discovery XP, the creature XP guideline test, wiki tables | — | S | changes saves (format bump); `--character`'s words on the race/class steps |
| B | **Buff and cooldown statuses** (B1): the status fields, `blocks`, `follows`, the new statuses, `Unavailable::While`; `afflict`-self abilities in the registry (`Source::Class` plumbing with no class entries yet) | — | S–M | parallel with A |
| C | **Free blows** (B3): `second_blow` generalised for kill, parry, flee triggers (no consumer yet but tests) | — | S | parallel with A, B |
| D | **Talents core**: data model, loader, `Talents`/`TalentMods`, `Actor::talents`, `Progress`, the `levels` stream, actions `talent`/`advance`/`raise`, pending budgets on level gain, messages, CLI layers, the coverage test (with the systems list), save and replay | A | M | the first 12 pure-modifier talents ship with it (dodge, hardy, healthy, brawn, silent_feet, keen_ears, pack_mule, long_strider, lean, good_learner, alert, thick_skin) to prove every hook kind |
| E | **Skill advances at level-up**: class dice data (`data/classes/` split from `classes.toml`), `known_for`, the kinder die, the `advances` counts | D | S–M | moving classes to a directory is this PR's housekeeping |
| F | **Stat raises** every 4th level, the potential rule | D | S | |
| G | **Level-up screen, Talents page, chargen step, `--talents`** | D, E, F | M | UI only; golden entries for the three screens |
| H1–H4 | **Talent waves**: H1 Vigour + Might + Finesse; H2 Weaponcraft + Guard; H3 Shadow (B4 effective eyes, B5 defence by kind) + Wayfarer; H4 Scholar + Grit + Delver | D (H2 also C; H3 brings B4/B5) | S–M each | parallel with each other and with G |
| H5 | **Racial talents** and `versatile` | D, G | S | |
| I1–I4 | **Class tables**: I1 warrior, barbarian, paladin, monk; I2 ranger, thief, bard; I3 priest, druid, wizard, witch (the caster placeholders through the cast code); I4 aristocrat, commoner, craftsperson, merchant | D, B, C (I1), B6 kill tally (I2) | M each | one file per class, so all four run in parallel |
| J1–J4 | **Small new actions**: J1 disarm (Disable device), J2 pick lock (Open lock; lock difficulty data), J3 steal, J4 throw voice and hood the lantern | B | S each | parallel; I2 and H5's `tinker` wait for J1–J3 or ship their entries greyed "not usable yet", as skills are today |
| K | **Coverage policy**: issue template section, `docs/dev/talents.md`, CLAUDE.md and spec bullets, the wiki's "by system" table | D | S | |
| L | **#151 talents** (`elemental_ward`, `crusher`, `blight`'s type) | #151, D | S | after #151 lands |

Critical path: A → D → G (three PRs) puts the whole loop in the game with a dozen talents; everything else widens it in parallel. Suggested waves: **wave 1** A, B, C (three agents); **wave 2** D; **wave 3** E, F, H1, H2, J1–J4 (within the cap); **wave 4** G, H3, H4, I1–I4; **wave 5** H5, K, L.

Each PR updates the spec bullets (§8 "Experience", a new §8 "Talents", "Level-up", the class bullet of §7), CLAUDE.md's pointers, `docs/dev/experience.md` → `docs/dev/progression.md`, and the wiki, as the repo's rule is.

## 12. Open questions for the owner

1. **The curve.** Cubic (`3·(n−1)³ + …`: even-level kills per level grow linearly from 2 to 100, 652 k XP at 60) rather than geometric. *Recommendation: cubic, with the creature XP guideline `5·level + 10`; revisit the constant once the Depths have level-10+ creatures.*
2. **The factors' range.** Races 90–140, classes 80–125, multiplied (72 % to 175 %). *Recommendation: as proposed; Angband 4.2 and DCSS both narrowed theirs, so keep the spread modest and show it in words at creation.*
3. **Discovery XP.** First kill of a type (its XP once more, unscaled), first arrival on a level (5 × depth), optionally first identification of an affix (10). *Recommendation: the first two now, identification later with item knowledge.*
4. **Hit points past 20.** Keep `hp_level = 20` (pools stay linear; Hardy and the barbarian's `hardened` buy more) or raise it to 30 with the cap at 60. *Recommendation: keep 20.*
5. **Stat raises.** +1 to a chosen stat every 4 levels (15 by 60), lifting the potential with it so no hidden cap ever refuses a pick. *Recommendation: yes; the alternative, respecting potentials and greying the capped stat, leaks them.*
6. **Caster classes now.** Give priest, druid, wizard, witch active placeholders through the scrolls' shared cast code with cooldown statuses, to be replaced by the spell system; or leave their tables to bonus talents until magic exists. *Recommendation: placeholders, clearly marked `spell` in data so the spell system can swap them; a wizard with nothing to do until M-magic is the ADOM "unsupported" feeling in reverse.*
7. **Cooldowns as statuses.** Winded/Drained/Spent are visible, time-based limits on class actives. They are not use counts, so "time is the penalty" holds, but they are a limit. *Recommendation: accept; the alternative (no limit, only the action's time and hunger) makes every heal a free full heal between fights, which only moves the limit to tedium.*
8. **Respec.** None proposed (a choice is a choice; nothing anti-farming is involved). *Recommendation: none; if ever, a rare consumable, never a menu.*
9. **Ranks and choices.** Up to 3 ranks per talent and parameterised talents (Weapon focus (long blades)). *Recommendation: both; they keep the list at ~90 names instead of ~140.*
10. **Skill advances.** 2 a level (3 for thief, ranger, bard), +1 at Intelligence 16 (a score threshold, not the modifier), humans and half-elves +1, dice `1d4+2` / `1d4` / `1d3`, the used-skill reroll, banking without limit. *Recommendation: as proposed; the one number to watch is the thief's 3.*
11. **Creation picks.** 2 (humans 3), chosen on a new step before the Summary; `--talents` on the command line. *Recommendation: yes; 1 would make the step feel like a tax, 3 like homework.*
12. **The level-up screen opens by itself** at the end of the turn a level comes, Esc banks everything. *Recommendation: yes, with the Talents page as the way back; never a `--more--`.*
13. **Class bonus picks.** 8 for warrior and thief (martial pattern), 5 for the rest; the commoner's from any tree. *Recommendation: as proposed.*
14. **New small actions** (disarm, pick lock, steal, throw voice, hood the lantern) ride on this milestone because the thief's and bard's tables want them. *Recommendation: yes, as four small issues; the class entries ship greyed "not usable yet" if they lag, as skill actions do today.*
15. **`Actor::talents` on every actor** (creatures could get talents from data later, with no new code) or the player only. *Recommendation: every actor; it costs nothing and keeps one aggregation.*
16. **Trim the list?** 99 talents are written; 80 is a comfortable first release. *Recommendation: ship all but `#151`'s now, since each is one data file and one hook, and the coverage test wants the breadth.*
