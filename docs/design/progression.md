# Warlock — Progression: experience, levels, class tables, skill rolls, stat training, talents, mana, resources

Design document for the owner's review (2026-10-20). Written against `origin/main` at 64b3713. Nothing here is implemented; §11 cuts it into issues.

## Contents

1. Summary
2. Research, tied to the decisions
3. Experience and levels (curve, factors, sources, hit points, the schedule, mana)
4. Class level tables (all fifteen classes, 1 to 60; the features that become resources)
5. Skill increases at level-up
6. Attribute growth (stat training by use, checked at level-up)
7. Talents (name, picks, prerequisites, ranks, the list by tree)
8. Coverage matrix and the policy for new systems
9. What the player sees (messages, screens, character creation, keys, wiki)
10. Data and code sketch (files, state, hooks, determinism)
11. Implementation plan (issues and waves)
12. Decisions (the owner's answers and the rest)

## 1. Summary

**What the owner asked for**: a level cap of 60 on a steep curve; race and class multipliers on the XP a level needs (long-lived races and patient classes pay more, the poor choices level faster); a per-class table of what each level gives; skill increases tied to level-ups "something like ADOM"; a decision on attribute growth; and a talent system with picks at creation and at levels, prerequisite trees, stat/skill/race/vision requirements, and a list of talents that interact with the systems that exist, with a policy so no system is ever "woefully unsupported". This revision folds in the owner's review of the first draft (PR #220): more kills per level (3 000 to level 20, 15 000 to 60), ADOM-like skill rolls fed by marks, stats trained by use and checked at level-up with no player choice, a lucky class with an extra talent, and no banking of any level-up choice.

**The proposal in one table:**

| Piece | Decision |
|---|---|
| Curve | written in kills of your own level: `k(n) = 213·m/(m+5) + 2.85·m` (m = n−1), XP for the level = `k(n) × (5n + 10)`, summed; **3 000** such kills to level 20, **15 000** to level 60 (2 863 615 XP for a human warrior), 38 for level 2, 365 for level 60. §3 |
| Factors | `experience` percent per race (elf 140 … half-orc 90) and class (wizard 125 … commoner 80), multiplied onto the thresholds (Angband's `expfact`; ADOM has the same idea). 72 % to 175 %. §3.3 |
| Sources | kills and quests as today; plus small, draw-free **discovery** XP: the first kill of each creature type, the first arrival on a level. §3.4 |
| Every level: skills | **skill rolls** (2 by class, thief and ranger 3, bard 4, humans +1, +1 per 4 Intelligence from 14), each on a trained skill of the player's choice, its die set by the skill's **marks** on ADOM's ladder (+1 … 4d5), one step up for a class's known-for skill, +1 for a class's or race's other skills; a roll consumes half the marks. Marks no longer grow skills by themselves. §5 |
| Every level: stats | **stat training**: 24 uses on existing hooks earn marks per stat (fighting Strength, being hurt Toughness, reading Intelligence …); at each level every stat is checked with chance `marks / (marks + 20 + 3·score)`, a rise consuming the marks; the potential a soft cap (chance ÷ 4 above it, lifted by a rise). No choice, no numbers. §6 |
| Every 3rd level | **a talent**; 2 at creation (humans 3, bards 3); class bonus picks (8 or 5) from the class's trees. 27–31 by level 60 from ~100 talents in 10 trees plus racial ones, prerequisites at most two deep (talents, stat scores, skill values, ranks, level, race, eyes). §7 |
| Class tables | a signature at 1, abilities at 6, 12, 18, 25, 32, 40, 50 (ADOM's levels), a capstone at 60, bonus talents between: 135 entries, 27 of them active (in the one abilities menu), every one on an existing hook; casters get placeholder actives on the scrolls' shared cast code until the spell system, paid in mana. §4 |
| Mana | the owner's new resource (its own PR): race and class `mana` (wizard 12 … warrior 0, the aristocrat oddly 7), a per-level gain beside the hit points, caster actives cost mana instead of cooldowns, spending it trains Willpower, five talents, a coverage row. §3.7 |
| No banking | the level-up screen opens with the level and cannot be left until its rolls and picks are spent; the rules refuse every other action while a choice is due, so the CLI and replays spend levels the same way; several levels in a turn are one sequence; a level stops a long action. §9.2 |
| Coverage | a `systems` tag on every talent and class ability, a closed list in data, a content test that fails when a listed system has fewer than 2 talents and 1 class ability (and when a tree has no prerequisite-free talent), and a "Talents" section in the feature issue template. §8 |
| UI | level-up messages; one `LevelUp` screen on the stack (skills → talent, the one paged list); a **Talents** info page (page 7); a Talents step in character creation with a Random row; everything rebindable; words in the game, numbers in the generated wiki tables. §9 |
| Data and code | `data/talents/<id>.toml`, `data/classes/<id>.toml` (directories: churning tables), `data/stats.toml` (stat training), settings in `talents.toml`/`experience.toml`/`skills.toml`; `Progress`, `stat_marks`, and a `levels` random stream in the world state, `Actor::talents`, one `TalentMods` aggregation read at the hooks the affixes already use; picks as replayed actions; no level-up draw from the gameplay stream. §10 |
| Resources | the owner's alternative resources (#225, after this milestone): barbarian Rage (decays, refilled by kills, ends at zero), ranger Focus, thief Openings, monk Ki, bard Inspiration; each class feature ships in its simple form first and migrates in a follow-up PR; four class talents; a coverage row. §4.18 |
| Plan | 28 issue-ready items in six waves; the critical path is three (curve → talents core → screens); two items wait for the mana PR, five for #225. §11 |

**Key decisions, and why**

1. *Steepness is measured in kills, not XP*, and the owner set the kills: a ramp to about 160 even-level kills a level by level 10, then a slope to 365 at 60; the XP table follows from a linear creature XP guideline.
2. *Three channels that never compete and never wait*: talents (chosen), skill rolls (chosen, dice by use), stats (by use, checked at the level). The research is unanimous that the disliked systems are the ones where one pool buys two things (5e's feat-or-stat); here nothing is traded against anything.
3. *Use charges, the level cashes in* (ADOM's shape): skill marks set the dice and are spent by the roll; stat marks set the chance and are spent by the rise. No grind buys a point outside a level-up, and no level-up gives a point to something never used, except the trained skill's guaranteed +1.
4. *Talents are never taxes, never level-1-only, never gates on ordinary actions, never flat numbers that stop mattering*: at most two-step chains, every link worth having, scaling effects, everything available whenever its prerequisites hold, and a prerequisite-free talent in every tree so a pick is never impossible.
5. *Classes differ in kind, talents in degree.* ADOM's praised part is class powers that change what you can do; its unloved part is talents that add +2. Here the class tables carry the identity (abilities), and talents the build.
6. *Nothing is banked.* A level is spent when it comes, in the rules as in the screen, so a save, a replay, and a dump all agree on what a level-7 character is.
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
| `threshold(n) = linear·(n−1) + quadratic·(n−1)²` (10, 10: 24 500 at 50) | a curve written in **kills of your own level**, `k(n) = 213·m/(m+5) + 2.85·m`, times the creature XP guideline, summed (2 863 615 at 60; 3 000 kills to 20, 15 000 to 60) |
| one curve for everyone | the curve times the character's **race × class experience factor** (percent) |
| XP from the player's own kills (WoW scaling, ZD bands to level 45) and quest rewards | the same, ZD bands extended to 60; plus a small **discovery** channel (first kill of a type, first arrival on a level) |
| `hp_level = 20` | unchanged; a **mana** gain per level beside it, to the mana PR's `mana_level` (§3.7) |
| a level gives hit points only | a level gives **skill rolls** (every level, dice by the skill's marks), **a training check on every stat** (every level, by its marks; no choice), a **talent** (every 3rd), and the **class's gain** at its table's levels (§4–§7); all of it spent as it comes, nothing banked |

Nothing else moves: `kill_xp` keeps its formula, kills by traps and creatures keep giving nothing, and the player keeps seeing their level, XP, and the XP of the next level (`PerceptionView::own_experience`).

### 3.2 The curve

**Measured in kills.** Creature XP grows linearly with creature level (the guideline below, which the existing creatures follow), so the curve is written as *how many kills of your own level a level costs*, `k(n)`, and the XP table follows from it. The owner's targets: about **3 000** such kills to reach level 20 and **15 000** to reach level 60 (ADOM's scale: a level-50 winner has killed ten to twenty thousand things).

**Formula** (data `[kills]` in `experience.toml`; integer math in hundredths): with `m = n − 1`,

`k(n) = plateau × m / (m + ramp) + slope × m` = `213 × m / (m + 5) + 2.85 × m`

`xp_for(n) = round(k(n) × guideline(n))`, `guideline(n) = 5 × n + 10`, and `threshold(n) = Σ xp_for(2..n)`.

The first term ramps quickly (38 kills for level 2, 106 for level 5, 163 for level 10) toward a plateau; the second keeps every later level costlier than the last (223 at 20, 300 at 40, 365 at 60), so the curve never flattens. A pure geometric or cubic curve cannot meet both targets (their ratio, 5, forces an early ramp and a late slope), which is why the old `linear`/`quadratic` fields are replaced by `plateau`, `ramp`, and `slope_per_100`. The level 60 threshold is 2 863 615 XP for the baseline character; the steepest (elf wizard) 5 011 326, well inside `u32`.

| Level | XP today | XP proposed | XP for this level | Kills of your own level for this level | Even-level kills so far |
|---:|---:|---:|---:|---:|---:|
| 2 | 20 | 767 | 767 | 38 | 38 |
| 3 | 60 | 2 431 | 1 664 | 67 | 105 |
| 4 | 120 | 5 084 | 2 653 | 88 | 193 |
| 5 | 200 | 8 797 | 3 713 | 106 | 299 |
| 6 | 300 | 13 628 | 4 831 | 121 | 420 |
| 8 | 560 | 26 840 | 7 212 | 144 | 698 |
| 10 | 900 | 45 064 | 9 758 | 163 | 1 014 |
| 12 | 1 320 | 68 601 | 12 450 | 178 | 1 363 |
| 15 | 2 100 | 114 464 | 16 739 | 197 | 1 935 |
| 18 | 3 060 | 173 769 | 21 314 | 213 | 2 559 |
| 20 | 3 800 | 221 186 | 24 517 | 223 | 3 000 |
| 25 | 6 000 | 369 068 | 33 048 | 245 | 4 181 |
| 30 | 8 700 | 561 820 | 42 316 | 264 | 5 465 |
| 35 | 11 900 | 803 095 | 52 311 | 283 | 6 843 |
| 40 | 15 600 | 1 096 517 | 63 029 | 300 | 8 309 |
| 45 | 19 800 | 1 445 689 | 74 467 | 317 | 9 860 |
| 50 | 24 500 | 1 854 206 | 86 623 | 333 | 11 493 |
| 55 | 29 700 | 2 325 653 | 99 496 | 349 | 13 207 |
| 60 | — | 2 863 615 | 113 085 | 365 | 15 000 |

("Kills of your own level" is the idealised count: a kill above you is worth up to 20 % more, below you less, grey ones nothing, as today, so real kill counts run higher. The first levels are shortened in practice by discovery (§3.4: every new creature type pays its XP once more) and by quests; a level-1 character who kills the hub quest's three rats and a few new things is a third of the way to level 2.)

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
| Human warrior (100 %) | 767 | 45 064 | 221 186 | 561 820 | 1 096 517 | 1 854 206 | 2 863 615 |
| Halfling thief (90 %) | 692 | 40 670 | 199 620 | 507 043 | 989 607 | 1 673 421 | 2 584 413 |
| Dwarf priest (132 %) | 1 014 | 59 597 | 292 518 | 743 007 | 1 450 144 | 2 452 187 | 3 787 131 |
| Elf wizard (175 %) | 1 342 | 78 862 | 387 076 | 983 185 | 1 918 905 | 3 244 861 | 5 011 326 |
| Half-orc commoner (72 %) | 552 | 32 446 | 159 254 | 404 510 | 789 492 | 1 335 028 | 2 061 803 |

What the player sees: their XP and the next threshold as today (`XP 85 / 767`); the character creation step for the race and the class says "Levels come slowly" / "quickly" in words, and the wiki's Races and Classes tables show the percent (a rule written in data, like stat adjustments).

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

Unchanged: each level up to `hp_level` (20) adds `max(1, class hp ÷ 2 + the Toughness modifier)`, so pools stay linear and a level-60 character's strength is in talents, skills, and stats, not in a runaway pool. Toughness trained up by use (§6) feeds this through `raised_stats` as it does for items; mana grows beside hit points (§3.7). The Vigour talents (Hardy, §7) are the way to buy more hit points, and they scale with level so the complaint that a flat +3 means nothing late (ADOM's Hardy) does not apply. `hp_level` stays at 20 (decided, §12).

### 3.6 What a level gives, all together

| Every level | Every 3rd (3, 6, …, 60) | Class table levels |
|---|---|---|
| hit points (to 20); **mana** (to the mana PR's `mana_level`, from the class's and the race's `mana` and the mental stats, as hit points come from Toughness: §3.7); **skill rolls**, spent at once (§5); **a training check on every stat**, no choice (§6) | a **talent** pick (§7): 20 by level 60, plus 2 at creation (humans and bards 3, a human bard 4) | a class ability, or a bonus talent from named trees (§4) |

The schedule is data (`[levels] talent_every = 3` in `data/experience.toml`), so it can be tuned without code. Every level is spent on the spot (§9.2: the level-up screen must be completed; nothing is banked), and every level gives at least its skill rolls and its stat checks; 20 give a talent, and the class tables fill most of the rest.

### 3.7 Mana

The owner has asked for **mana** as a resource with regeneration rules, built in its own PR (the mana PR): a current value on `Actor`, a maximum derived like `max_hp` from the race's and the class's `mana` in data plus a gain per level up to a `mana_level`, driven by the mental stats; one pace hook like `World::healing_rate` inside `World::live`; and hooks in hunger, statuses, diseases, special rooms, and the magic item affixes. This design gives it its place in progression and leaves its formula to that PR:

| Race | `mana` | Class | `mana` | Class | `mana` |
|---|---:|---|---:|---|---:|
| Elf | 3 | Wizard | **12** | Paladin | 3 |
| Gnome | 3 | Witch | **10** | Monk | 4 |
| Half-elf | 2 | Priest | **9** | Ranger | 2 |
| Human | 1 | Druid | **9** | Thief | 1 |
| Halfling | 1 | Bard | 5 | Warrior | 0 |
| Dwarf | 0 | **Aristocrat** | **7** (the oddly high one: idle minds, old blood) | Barbarian | 0 |
| Half-orc | 0 | Merchant | 2 | Commoner, Craftsperson | 1 |

- **Per level**: as hit points gain `max(1, class hp ÷ 2 + the Toughness modifier)` up to `hp_level`, mana gains `max(0, class mana ÷ 2 + the better of the Intelligence and Willpower modifiers)` up to `mana_level` (the mana PR's; 30 is the suggestion, so casters keep growing after hit points stop), through `World::level_hp`'s twin in `level_up`. A warrior with class mana 0 and In 10 gains none.
- **Class actives that cast** (the caster placeholders of §4 and the paladin's, bard's, and priest's gifts) **cost mana instead of a cooldown status**: mana comes back with time, so "time is the penalty" holds without a second timer; a cast with too little mana is `Unavailable::NoMana` ("not enough mana"), the cost is in the ability's data (`mana = 6`), and the martial actives (second wind, rage, smite, whirlwind, volley, quivering palm) keep their statuses, since they are not magic. §4's tables say which.
- **Stat training**: spending mana is a `cast` use that trains **Willpower** (§6.2); Intelligence already trains from reading and identifying.
- **Talents**: five in the Scholar tree on the mana PR's hooks (§7.5), and mana is a system in the coverage list (§8).

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

Plus the shared skeleton (§3.6) for everyone: skill rolls every level (the class decides how many, §5), a training check on every stat every level (§6), a talent every third level. A level not in a class's table gives the shared gains only.

**Rules for the entries.** Every entry below names what system it reads or changes (its *hook*), so it can be built now; the few that need a system still to come say so (`#151` damage types; `spell` for the caster placeholders, see below; `action` for a new small action, §9). Active abilities (**A**) register in `warlock_core::ability` under `Source::Class`, with an action of their own (replay text `use:ID[:DIR|:X,Y]`), an `ActionKind` cost, and no default key; passive ones (**P**) are rules read at their hook. Numbers are the first guess, kept in the class's data file (§10) to be tuned by play.

**Building blocks** most entries stand on (each is one PR in §11):

- **B1 Buff and cooldown statuses.** `data/status.toml` grows optional combat fields a status applies while it lasts: `to_hit`, `damage` (as the stances have), `dodge`, `parry`, `cost` (percent of every action's price: a slow), `regen` (exists), `blocks = ["read"]` (actions refused while it lasts), and `follows = "exhausted"` (a status given as this one ends). `penalty` already takes positive numbers (−10 to 10), so a status may *raise* scores. An ability that gives the player a status is `World::afflict(player, status, 1, player)`; a **cooldown** is a status (`winded`, `drained`, `spent`, `hoarse`, `exhausted`) with an `eases` and no tick, during which the ability is `Unavailable::While(status)` ("you are winded") — a visible, honest, time-based limit, which keeps the owner's rule: time is the only price, and nothing counts uses. Statuses on creatures exist already (`Actor::effects`), so hexes work the same way.
- **B2 The shared cast code** (`World::cast_burst`, `World::cast_bolt` with `Caster::Actor`) that scrolls use: a class ability may carry a burst or a bolt in data exactly as a scroll's `read` table does, aimed by the targeting UI. These are **placeholders for casters** until the spell system (magic is out of the first phase); they let the four caster classes have live tables now, and the spell system replaces them (decided, §12). They **cost mana** (§3.7, the mana PR) rather than a cooldown status: mana returns with time, so the one timer is enough; a cast without the mana is `Unavailable::NoMana`. The martial actives keep their statuses (and some become resources with #225, §4.18).
- **B3 Free blows.** `World::second_blow` becomes the one place every extra blow is decided: the off hand's, Sword and Board's bash, and now a *cleave* (after a melee kill, at another adjacent hostile), a *riposte* (after a miss credited to Parry, `why_missed`), and an *opportunist's* blow (after a creature's fleeing step, `World::flees`). Each is a normal attack with its own events and marks.
- **B4 Effective eyes.** `WorldState::vision` stays the base profile; `World::eyes()` applies talent and class changes (threshold percent, darkvision radius, `sees_uv`) and is what perception and creature sight read. Creatures keep their types' profiles.
- **B5 Defence by attack kind.** `Actor::defence` takes the attack's kind, so a bonus can count against missiles only (`deflect_missiles`).
- **B6 Kill tally for all types.** The quests' per-level tally (`Quests::kills`) counts every type, not only those a quest asks for (`favoured_prey`, and discovery's first kill).
- **B7 Small new actions** that skills have waited for: `disarm:DIR` (Disable device), `pick:DIR` (Open lock), `steal:DIR` (Steal), `throw_voice:X,Y`, `hood` (the lantern). Each is its own issue (§11); the class entries that need one say `action`.

### 4.2 Warrior

Bonus talents (Weaponcraft, Guard) at 4, 10, 16, 22, 28, 34, 46, 58. Skill rolls 2.

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

Bonus talents (Might, Vigour) at 10, 20, 30, 45, 55. Skill rolls 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `rage` | Rage (#225) | A | Raging for 5 min: St +2, To +2, In −2, cannot read; then Exhausted 10 min (St −1), during which no rage | B1 (`penalty` positive, `blocks`, `follows`); cost `wait` |
| 6 | `thick_hide` | Thick hide | P | Natural +2 while wearing no body armour | `Actor::defence` |
| 12 | `fast_movement` | Fast movement | P | steps 10 % cheaper while Unburdened and in no body armour | `World::step_price` |
| 18 | `brutal_critical` | Brutal critical | P | critical multiplier +1 with two-handed weapons | `Profile` |
| 25 | `hardened` | Hardened | P | from now on every level adds hit points again (as levels up to 20 do), +1 | `World::level_hp`, `hp_level` per class |
| 32 | `mighty_thews` | Mighty thews | P | Strength +2 and Toughness +2, scores and potentials (ADOM's 32) | `Stats` |
| 40 | `blood_frenzy` | Blood frenzy (#225) | P | a kill while raging extends the rage by a minute | `Effects::add` refresh on `Event::Died` |
| 50 | `mighty_rage` | Mighty rage (#225) | P | Raging gives St +4, To +4; Exhausted lasts 5 min | data |
| 60 | `unstoppable` | Unstoppable (#225) | P | while raging, bleeding and poison tick nothing, and once per rage a blow cannot take your last hit point | `World::live`, `hurt` |

### 4.4 Paladin

Bonus talents (Guard, Vigour) at 10, 20, 30, 45, 55. Skill rolls 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `lay_on_hands` | Lay on hands | A | regain 2 HP per level; costs 6 mana | mana (§3.7); cost `cast` |
| 6 | `divine_grace` | Divine grace | P | +2 on disease checks; poison eases twice as fast | `check` modifier, `Effect::until_eases` |
| 12 | `smite` | Smite | A | a melee blow in a direction with +1d6 damage per 10 levels (at least 1d6); Spent for 5 min | `attack_with` with an extra `Profile` die; cost `melee` |
| 18 | `divine_health` | Divine health | P | immune to diseases | `World::expose` refuses |
| 25 | `shield_of_faith` | Shield of faith | P | Shield +1; Sword and Board's bash chance +10 | `Actor::defence`, `second_blow` |
| 32 | `steadfast` | Steadfast | P | Burdened costs nothing (as a dwarf's) | `Load::penalty` |
| 40 | `holy_radiance` | Holy radiance | A | a toggle: you shed a faint light (`item_glow`), seen by all; off again from the menu | worn-light hook (`LightCache` carried source) |
| 50 | `bulwark` | Bulwark | P | Protection +2 | protection sum |
| 60 | `avatar` | Avatar | P | Lay on hands heals fully and cures every status and disease, for half the mana | data, `World::cure` |

### 4.5 Monk

Bonus talents (Finesse, Vigour) at 10, 20, 30, 45, 55. Skill rolls 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `martial_arts` | Martial arts | P | bare hands 1d6 (kicks too) | bare-hands dice in `World::attack`/`player_kicks` |
| 6 | `fleet` | Fleet | P | steps 10 % cheaper in no body armour and with no shield | `World::step_price` |
| 12 | `deflect_missiles` | Deflect missiles | P | Dodge +2 against shots, throws, and trap arrows | B5 |
| 18 | `flurry` | Flurry (#225) | P | unarmed attacks strike twice (the second at −2) | B3 |
| 25 | `still_mind` | Still mind | P | +2 on Willpower checks; status penalties to In and Wi never apply | `check`, `effective_stats` |
| 32 | `purity_of_body` | Purity of body | P | immune to diseases | `World::expose` |
| 40 | `diamond_soul` | Diamond soul | P | Deflection +2 | `Actor::defence` |
| 50 | `quivering_palm` | Quivering palm (#225) | A | an unarmed blow in a direction; a hit that wounds makes the target bleed 3 degrees; Spent 10 min | `attack_with` + `afflict`; cost `melee` |
| 60 | `perfect_self` | Perfect self | P | bare hands 1d8; unarmed threat +1 and critical multiplier +1 | `Profile` |

### 4.6 Ranger

Bonus talents (Weaponcraft, Wayfarer) at 10, 20, 30, 45, 55. Skill rolls 3.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `favoured_prey` | Favoured prey | P | +1 to hit and +2 damage against the creature type you have killed most (ties: the first reached) | B6, `Profile` |
| 6 | `tracker` | Tracker | P | you hear sounds 3 fainter, and place every sound you hear exactly | `Actor::hearing`, `PerceptionView::sounds` |
| 12 | `rapid_shot` | Rapid shot (#225) | P | shots 10 % cheaper | `World::attack_price` |
| 18 | `woodland_stride` | Woodland stride | P | forest floor, bushes, reeds, tall grass, and marsh cost you as plain ground (no extra time; their noise stays) | `World::step_price` |
| 25 | `swift_tracker` | Swift tracker | P | overworld journeys 20 % quicker | `World::step_price` |
| 32 | `evasion` | Evasion | P | bursts and bolts (traps, scrolls, spells) do you half damage | `cast_burst`/`cast_bolt` victim hook |
| 40 | `camouflage` | Camouflage | P | under the open sky your footsteps are half as loud | `noise::noise` (`is_sky_exposed`) |
| 50 | `volley` | Volley (#225) | A | three shots at the target for the price of one (three missiles leave the stack) | `shoot.rs::loose` ×3; cost `shoot` |
| 60 | `master_hunter` | Master hunter | P | bows and crossbows threat +1; favoured prey +2 to hit, +4 damage | `Profile` |

### 4.7 Thief

Bonus talents (Shadow, Finesse, Delver) at 4, 10, 16, 22, 28, 34, 46, 58. Skill rolls 3.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `sneak_attack` | Sneak attack | P | +1d6 damage (melee, shot, throw) against a creature that has not seen you (its mind is calm) | `Profile` from `Mind` |
| 6 | `trapfinding` | Trapfinding | P + A | searching +4 and hidden traps 2 easier for you; **Disarm** (a direction): a Dexterity check plus Disable device against the trap's difficulty takes it away; a failure by 5 or more springs it at you | `search_modifier`; `action` disarm |
| 12 | `pick_lock` | Pick lock | A | open a locked door without a key: a Dexterity check plus Open lock against the lock's difficulty (data on locks, default 14); it takes `unlock` whatever comes of it | `action` pick; `door.rs` |
| 18 | `evasion` | Evasion | P | as the ranger's | cast hooks |
| 25 | `uncanny_dodge` | Uncanny dodge | P | the burden's cap on Dexterity in Defence is 2 higher at every level | `Burden::max_dex` |
| 32 | `improved_sneak_attack` | Improved sneak attack (#225) | P | sneak attack +2d6 (3d6 in all) | `Profile` |
| 40 | `light_fingers` | Light fingers | A | **Steal** (a direction): take one stack from a creature that is not hostile; a Dexterity check plus Steal against its Willpower check; caught, it turns on you (as a robbed keeper does) | `action` steal; `faction_override` |
| 50 | `hide_in_plain_sight` | Hide in plain sight | P | your footsteps are silent while Unburdened, your attacks half as loud | `noise::noise` |
| 60 | `master_thief` | Master thief (#225) | P | sneak attacks threaten a critical from a natural 20 | `Profile` |

### 4.8 Bard

Bonus talents (Scholar, Delver) at 10, 20, 30, 45, 55. Skill rolls 4 (ADOM: bards get more); one extra talent at creation.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `lore` | Lore | P | a magic item you have carried for a day is identified | `World::identify` from `perceive` (as `rot_away` runs) |
| 6 | `wide_learning` | Wide learning | P | +1 skill roll every level (the bard's 4 become 5) | §5 rolls |
| 12 | `song_of_rest` | Song of rest | P | resting heals half again as fast | `World::healing_rate` while `resting` |
| 18 | `inspire` | Inspire (#225) | A | Inspired for 5 min: +1 to hit, Dodge +1; costs 3 mana (or 3 Inspiration, §4.18) | B1, mana; cost `chat` |
| 25 | `jack_of_all_trades` | Jack of all trades | P | every trained skill counts 5 higher | `Actor::skill_effect` |
| 32 | `throw_voice` | Throw your voice | A | a voice (loudness 50) sounds at a cell you see within 6: creatures that hear it go and look there | `action` throw_voice; `World::sound_at`, the AI's noise step |
| 40 | `silver_tongue` | Silver tongue | P | shop favour +15 %, quest gold ×1.25 | `world::favour`, quest reward |
| 50 | `legend` | Legend | P | quest XP ×1.5; Presence +1 | quest reward, `Stats` |
| 60 | `virtuoso` | Virtuoso | P | Inspired gives +2/+2 for 10 min and costs no mana | data |

### 4.9 Priest

Bonus talents (Vigour, Grit) at 10, 20, 30, 45, 55. Skill rolls 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `blessing` | Blessing | A | Blessed for 10 min: +1 to hit, Wi +1; costs 2 mana | B1, mana; cost `cast` |
| 6 | `cure_wounds` | Cure wounds | A | regain 1d8 + 1 per 2 levels; costs 4 mana | mana; cost `cast` ×2 |
| 12 | `remove_disease` | Remove disease | A | cures every disease and poison; costs 10 mana | `World::cure(remove_disease)`, status end, mana |
| 18 | `shield_of_faith` | Shield of faith | P | Deflection +1 | `Actor::defence` |
| 25 | `purify` | Purify | P | rotten food neither sickens you nor exposes you to disease | `eat.rs::finish` |
| 32 | `divine_health` | Divine health | P | immune to diseases | `World::expose` |
| 40 | `smite` | Smite | A | a melee blow in a direction with +2d6 damage; Spent 5 min | `attack_with`; cost `melee` |
| 50 | `miracle` | Miracle | P | once in a run, a blow that would kill you leaves you at 1 hit point, Drained for a day ("A miracle!") | `hurt`/`dies`; a flag in the progress state |
| 60 | `saint` | Saint | P | hit points always come back twice as fast | `World::healing_rate` |

### 4.10 Druid

Bonus talents (Wayfarer, Vigour) at 10, 20, 30, 45, 55. Skill rolls 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `wild_empathy` | Wild empathy | P | creatures of animal factions (`animal = true` in `factions.toml`: vermin, deer) are neutral to you until you attack one of that type | `World::stance`, a flag per type in the progress state |
| 6 | `woodland_stride` | Woodland stride | P | as the ranger's | `step_price` |
| 12 | `weather_sense` | Weather sense | P | you know the weather of the next three hours: the status bar's weather segment shows the coming change ("rain soon") | perception, `weather::weather_at(now + 3 h)` |
| 18 | `natures_ward` | Nature's ward | P | wading never gives you a disease; poison eases twice as fast | `World::wade`, status easing |
| 25 | `winters_friend` | Winter's friend | P | ice holds you as if you carried nothing; a plunge never hurts; climbing out costs nothing | `World::tread`, `after_tread`, `tread_price` |
| 32 | `call_lightning` | Call lightning | A | under the open sky: a bolt (4d6, 8 cells, a thunderclap) at a target; costs 8 mana | B2 `cast_bolt`; `spell` placeholder; mana; cost `cast` |
| 40 | `sun_and_moon` | Sun and moon | P | under the sun hit points come back half again as fast; at night your overworld sight is one cell longer | `healing_rate` (`sun().is_up`), `daylight::overworld_sight` |
| 50 | `flame_and_frost` | Flame and frost | A | two abilities: a fireball burst (3d6, radius 2, breaks walls, melts ice) or a frost burst (2d6, freezes water); 10 mana each | B2 `cast_burst`; `spell` placeholder; mana |
| 60 | `timeless_body` | Timeless body | P | hunger burns half as fast; diseases never progress (they still show until shaken off or cured) | `hunger`, `disease_turn` |

### 4.11 Wizard

Bonus talents (Scholar) at 10, 20, 30, 45, 55. Skill rolls 2. The actives are **placeholders** on the shared cast code until the spell system exists (decided, §12), paid in mana (§3.7).

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `scroll_adept` | Scroll adept | P | reading costs half; your spell scrolls add 1 to every die | `read` price, `cast_*` |
| 6 | `arcane_lore` | Arcane lore | P | a magic item you have carried for an hour is identified | identify hook |
| 12 | `spell_recall` | Spell recall | P | one read in four, the scroll is not used up | `read.rs` (one gameplay draw per read) |
| 18 | `arcane_bolt` | Arcane bolt | A | a bolt (2d6, 6 cells) at a target; costs 3 mana | B2; `spell`; mana; cost `cast` |
| 25 | `mage_armour` | Mage armour | P | Deflection +2 while wearing no body armour | `Actor::defence` |
| 32 | `empower` | Empower | P | your spell scrolls and your bolts roll one die more | `cast_*` |
| 40 | `fire_burst` | Fire burst | A | the fireball's burst (3d6, radius 2, blast, heat) at a target; costs 9 mana | B2; `spell`; mana |
| 50 | `arcane_mastery` | Arcane mastery | P | mana comes back twice as fast | `World::mana_rate` (the mana PR) |
| 60 | `archmage` | Archmage | P | your spells add 2 to every die and cost half the mana | data |

### 4.12 Witch

Bonus talents (Scholar, Grit) at 10, 20, 30, 45, 55. Skill rolls 2.

| Lv | id | Name | A/P | Effect | Hook |
|---:|---|---|---|---|---|
| 1 | `herb_lore` | Herb lore | P | rotten food harms you half as much; a corpse's own harm (`on_eat` harm) never; the angelica root heals 1d6 too | `eat.rs::finish` |
| 6 | `evil_eye` | Evil eye | A | Hexed for 1 min on a creature you see in a direction: −2 to hit; costs 2 mana | B1 on creatures; mana; cost `cast` |
| 12 | `poison_lore` | Poison lore | P | your Poisoned eases twice as fast; +2 damage against poisoned targets | status, `Profile` |
| 18 | `hex_of_weakness` | Hex of weakness | A | Weakened 2 min on the target: St −2, Dx −2; costs 3 mana | B1, mana |
| 25 | `second_sight` | Second sight | P | you notice hidden things in passing (as if trained in Spot), +2; secret doors 2 easier | `search.rs` |
| 32 | `curse_of_sloth` | Curse of sloth | A | Slowed 1 min on the target: its actions cost half again as much; costs 4 mana | B1 (`cost` field), mana |
| 40 | `blight` | Blight | A | a burst (2d6, radius 2) that also poisons 2 degrees everyone it reaches; costs 8 mana (poison-typed with #151) | B2 + `afflict`; `spell`; mana |
| 50 | `cauldron` | Cauldron | P | corpses never harm you; their stat chances are doubled; a fresh corpse heals 1d4 | `eat.rs` |
| 60 | `hags_bargain` | Hag's bargain | P | once in a run, a death leaves you at 1 hit point, Drained a day, and Presence −2 for good | `hurt`/`dies` |

### 4.13 Aristocrat

Bonus talents (Weaponcraft, Delver) at 10, 20, 30, 45, 55. Skill rolls 2.

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

Bonus talents (**any tree**: the commoner's gift is versatility) at 10, 20, 30, 45, 55. Skill rolls 2.

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

Bonus talents (Weaponcraft, Guard, Delver) at 10, 20, 30, 45, 55. Skill rolls 2.

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

Bonus talents (Delver, Scholar) at 10, 20, 30, 45, 55. Skill rolls 2.

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

15 classes × 9 entries = 135 class abilities, of which 27 are active (28 menu entries: `flame_and_frost` is two) and 108 passive; 4 need a new small action (disarm, pick, steal, throw_voice), 9 are caster placeholders on the shared cast code (`spell`), 14 cost mana (§3.7: the casters', the paladin's lay on hands, the bard's inspire, the priest's three; they wait for the mana PR), 1 (`blight`) wants #151 for its type but works untyped, and 10 (marked #225) move onto alternative resources after this milestone (§4.18). Shared entries (`evasion`, `woodland_stride`, `divine_health`, `second_wind`, `smite`, `shield_of_faith`, the worth-showing ones) are one implementation each, named in several tables.

### 4.18 Class features that become resources (#225)

The owner has asked for **alternative resources** (#225: integer pools beside hit points and mana that regenerate, decay, build from events, are spent, or end at zero and take their bar with them; a character may hold several), to be built **after** this milestone. The class features below are the ones that move onto them; each ships first in the simple form its table gives (a status, a passive) so the class is playable now, and migrates in a follow-up PR that depends on #225 (§11, R1–R5). The parameters are first guesses in `data/resources.toml`.

| Class | Resource | Granted by | Max | Start | Over time | Built by | Spent by | At zero | Ships first as | Mana |
|---|---|---|---:|---:|---|---|---|---|---|---|
| Barbarian | **Rage** | the `rage` ability (level 1) | 20 (30 with `mighty_rage`) | 12 | **decays** 1 per base turn (6 s), resting too | +5 per kill once `blood_frenzy` (40) is known; nothing before | `unstoppable` (60): 10 rage to keep the last hit point | **ends**: the bar goes, Exhausted follows (10 min; 5 with `mighty_rage`), and the rage's benefits (St +2, To +2, In −2; cannot read) leave with it | the `raging` status of B1 (5 min, refreshed a minute by a kill); the move is "the status's timer becomes the pool" | none while raging: a barbarian cannot read, and mana abilities are refused |
| Ranger | **Focus** | `rapid_shot` (12) | 10 | 10 | **regenerates** 1 per base turn, not while resting | — | `volley` (50): 6; with #225 `rapid_shot` becomes "a shot that costs 2 focus is 10 % cheaper" instead of always | never ends: the bar stays once granted | `rapid_shot` as the passive the table gives; `volley` as three free shots | separate; nothing converts |
| Thief | **Openings** | `improved_sneak_attack` (32) | 5 | 0 | **decays** 1 per 20 turns | +1 per sneak attack that hits | a **finisher** added to `master_thief` (60): spend 5 for a blow that is a critical without a confirmation roll | never ends; sits at 0 | the passives as the table gives; the finisher arrives with the migration | separate |
| Monk | **Ki** | `flurry` (18) | 5 | 5 | **regenerates** 1 per 3 base turns, resting too | — | `flurry`: 1 per second blow (no ki, no flurry); `quivering_palm` (50): 5 | never ends | `flurry` always on (B3); `quivering_palm` with its Spent status | separate: a monk's mana (4) stays for talents' sake |
| Bard | **Inspiration** | `inspire` (18) | 5 | 0 | **decays** 1 per 30 turns | +1 per kill of the bard's own | `inspire`: 3 inspiration **in place of** its 3 mana when it has them (the one place a resource touches mana) | never ends | `inspire` costing mana | spent instead of mana when full enough |

Everything else in the tables stays mana, a status, or a passive. Creatures get no resources (their rage would never show anyway). Four talents on these resources are in §7.5, marked `#225`; "Resources" is a row of the coverage matrix (§8) and joins the systems list when #225 lands.

## 5. Skill increases at level-up

### 5.1 Today, and the ADOM reference

Skills grow today only from **training marks** (`world/train.rs`: a chance per meaningful use, drawn from the `training` stream; `need(value) = marks + value / per`, ×3 past the soft cap 100), a rule the spec marks **provisional** pending exactly this decision. ADOM: skills train "passively as the PC successfully uses them, and actively on level-up"; a level-up gives a number of increases (3 at least, plus one per 5 points of Learning over 10, more for humans and bards), and each skill rolls its own die from a ladder (+1, 1d3, 1d5, 2d4, 3d3, 3d4, 3d5, 4d4, 4d5) whose step is **raised by successful use and degrades every ten points of advancement**. The owner wants this shape: marks decide how many and how large the dice are, a roll consumes marks and shrinks the dice, the class decides the count, humans get one more, high Intelligence more, and classes and races favour some skills.

### 5.2 The rule: marks are a reservoir, spent as dice at level-up

**One channel.** Marks no longer turn into points by themselves: the provisional `[increase]` rule goes, and a skill grows **only at level-up**, by rolls whose dice are set by the marks it has gathered since. The reasons: one curve to tune instead of two that interact; no double counting (a used skill is not rewarded twice); the Skills screen's meter gets a plain meaning ("how big your next roll is"); and the ADOM feel the owner asked for (use charges the skill, the level cashes it in). What is lost, growth between levels, matters less now that every level gives rolls and a trained skill always rolls at least +1.

| Rule | Value | Data |
|---|---|---|
| **The ladder** | a skill's die is set by its marks: 0 marks `+1`; 5 `1d3`; 12 `1d5`; 25 `2d4`; 50 `3d3`; 100 `3d4`; 200 `3d5`; 400 `4d4`; 800 `4d5` (ADOM's ladder; the steps roughly double) | `[ladder] dice = [...], marks = [0, 5, 12, 25, 50, 100, 200, 400, 800]` in `skills.toml` (replaces `[increase]`) |
| **A roll** | adds the die to the value; past 100 the roll is halved, rounded up (the soft cap) | `Skills::roll` |
| **It consumes marks** | half the skill's marks, rounded up, so the next roll on the same skill is one or two steps lower (50 → 25 → 12 → 6 → 3: 3d3, 2d4, 1d5, 1d3, +1) | the same |
| **Rolls per level** | the class's `rolls` (§4: 2 for most; 3 for the thief and the ranger; **4 for the bard**, whose identity it is), **+1 for humans**, **+1 for every 4 points of Intelligence from 14** (14: +1, 18: +2, 22: +3 …; a score rule the player can read off the character screen, never the modifier) | `data/classes/*.toml`, `races.toml`, `[rolls] intelligence_from = 14, intelligence_every = 4` |
| **Affinities** | a skill the class is **known for** (`known_for`, its `2d6+10` list) rolls its die **one step up the ladder**; a skill the class trains besides, or the **race** trains, adds **+1** to the roll (both stack: a halfling thief's Stealth is known-for and racial: one step up and +1) | `known_for`, `skills` on the class and the race; an optional explicit `affinity` table for later tuning |
| **A skill with no marks** | still rolls (`+1`), so a trained skill never stands still, as in ADOM | |
| **Where rolls go** | any **trained** skill, chosen by the player, all of them before the screen closes (no banking, §9.2); nothing stops every roll going into one skill, the shrinking dice are the only brake | `Action::Advance(skill)` |
| **Earning marks** | unchanged (`mark` rules in `skills.toml`, the `training` stream); the Studious talent raises their chance | `world/train.rs` |
| **What is said** | "Your stealth improves." (never the die or the marks) | the level-up screen |
| **Randomness** | the dice draw from the new **`levels` stream** in the world state (like `training`); each `advance:SKILL` is an action recorded in the replay | `WorldState::levels_rng` |

### 5.3 Numbers in play

- A thief (3 rolls, 4 with Intelligence 14) who sneaks a lot gathers 30–60 Stealth marks a level (`sneak` 15 % per step within earshot): the first roll is `2d4` or `3d3` one step up (`3d3`/`3d4`, average 6–7.5) +1 racial if a halfling; a second roll on it half that. Spending two rolls on Stealth and two elsewhere gives Stealth about 10 a level: 25 at creation, 60 by level 5, 100 near level 10, 150 by level 25 (halved past 100). Spread over four skills, each about 4–5 a level.
- A warrior (2 rolls) with no marks in Athletics still gets `+1` or, being known for it, `1d3` a level; with a few door kicks (`kick` 20 %) `1d5`–`2d4`.
- A skill never used (no marks) grows by 1–2 a level if the player keeps rolling it: 60 levels buys about 90 points, a mastery for the patient.
- Humans (+1 roll) and bards (4) are the fast learners; an elf wizard (2 rolls, In 16: +1) learns broadly but slowly, as its experience factor intends.

The pace is set by three knobs in data: the ladder's mark steps, the class `rolls`, and the halving. If play shows masteries come too early, raise the mark steps; too late, lower them.

### 5.4 The level-up screen's skills step

A paged list of the trained skills (the one item-list widget, §9): each row the skill's name, its value, and a dim four-cell meter of its ladder step (how practised it is: `━━──` at `2d4`), the frame saying "Skill rolls: 2 left"; a letter rolls one on that row (`advance:stealth`, no time, said at once: "Your stealth improves."), the meter dropping as the marks are spent. The step cannot be left until every roll is spent (Esc does nothing but say so); with no trained skill at all (a scenario without a character) the step is skipped and the rolls are lost. No dice, no mark counts: the wiki's Skills page carries the generated ladder and the Classes page each class's rolls and known-for skills.

## 6. Attribute growth

### 6.1 The options, and the owner's call

| Model | Where | What players say |
|---|---|---|
| None at level-up; potions and training by use | Angband (potions), ADOM (training: St by staying Burdened, To by herbs, Le by reading; checked periodically and "guaranteed on level-up", never past the potential; capped at St 15 "regardless of your potential", the verified complaint) | the grind is the complaint, the potentials the mystery |
| A chosen point every few levels | D&D 3.5/Pathfinder (4, 8, …), DCSS (every 3–5 by species), Qud | accepted; 5e's feat-or-stat tension the only disliked form |
| Several points every level | ToME (3) | stat inflation |

The owner does not want the player picking stats. The design below is ADOM's idea made deterministic and tied to the level: **stats earn training marks from use, and at every level-up each stat has a chance to rise by its marks; a rise consumes them.**

### 6.2 Stat training marks

Each of the six stats has a mark count on the player (`Actor::stat_marks: PerStat<u16>`, saved and hashed; creatures earn none). Marks come from existing hooks, each with a chance in percent, one draw of the **`training`** stream per use (exactly as skill marks do: never the gameplay stream, so a fight's rolls never move). Data: `data/stats.toml`, `[[training]] stat, on, chance`.

| Stat | Use (`on`) | Where it is reported today | Chance |
|---|---|---|---|
| **Strength** | `burdened`: an action other than waiting while your load slows you (Balance's use) | `World::apply` | 10 |
| | `blow`: a melee blow of yours that hits (a kick, a bash too) | `fight.rs::resolve` | 5 |
| | `dig`: a turn of digging | `dig_turn` | 10 |
| | `kick`: a kick at a door (Athletics' use) | `kick_door` | 20 |
| **Dexterity** | `dodge`: an attack on you that missed because of your Dexterity or Dodge (Tumble's use) | `fight.rs::resolve` | 20 |
| | `shot`: a shot or throw of yours that hits | `missile_hits` | 10 |
| | `sneak`: a step within earshot that stealth kept quiet (Stealth's use) | `make_noise` | 5 |
| | `climb`: climbing out onto ice, a pit's climb | `tread_price`, `spring_trap` | 50 |
| **Toughness** | `hurt`: a blow that costs you hit points | `hurt` | 10 |
| | `big_hit`: a big hit or a critical on you | `after_blow` | 50 |
| | `resisted`: a disease check won (catching or a progression) | `disease.rs` | 50 |
| | `plunge`: falling through ice | `after_tread` | 100 |
| **Intelligence** | `found`: a search that finds something (a secret door, a trap) | `search.rs::reveal` | 50 |
| | `read`: a scroll read | `read.rs` | 30 |
| | `identified`: an item identified | `World::identify` | 100 |
| | `discovered`: the first kill of a creature type (§3.4) | `earn_kill` | 100 |
| **Willpower** | `spot`: noticing something hidden in passing (Spot's use) | `notice_hidden` | 30 |
| | `endured`: a long action completed to its goal (a rest to dawn, a wall dug through, a search for a while) | `keep_at` | 100 |
| | `resisted_will`: a Willpower check won (brain fever) | `disease.rs` | 50 |
| | `hungry_fight`: a blow landed while Hungry or worse | `fight.rs::resolve` | 10 |
| | `cast`: mana spent on an ability (the mana PR; §3.7) | the ability's action | 20 |
| **Presence** | `chat`: a chat or a topic asked | `talk.rs` | 20 |
| | `trade`: a purchase paid or a sale made | `shop.rs` | 20 |
| | `quest`: a quest completed | `quest.rs` | 100 |
| | `swapped`: a creature making way for you | `swap_places` | 10 |

Every `on` is a `stats::Use` variant reported at the one place the thing happens, like `skills::Use`; the two reports share the call (`World::train(Use)`). There is no cap on marks and no decay: time is the only price, and the owner's no-anti-farming rule holds (kicking a door a hundred times is a hundred chances at a mark, and a hundred bangs every creature on the level hears).

### 6.3 The level-up check

At every level gained, **each stat in display order** gets one check, drawn from the **`levels`** stream (never gameplay): it rises by 1 with chance

`p = marks / (marks + need)`, `need = 20 + 3 × score` (score 10: 50; 14: 62; 20: 80; 30: 110)

so 20 marks at Strength 14 is a 24 % chance, 60 marks 49 %, and no marks is no chance. **A rise consumes all the stat's marks**; a failed check keeps them (they go on growing toward the next level). At most one point per stat per level. The **potential** stays a soft cap: at or above it the chance is divided by 4, and a rise there lifts the potential with the score, so a long career outgrows nature slowly rather than never, and no sudden stop betrays the hidden number (a chance that merely fell cannot be told from bad luck). The rise says "You feel stronger." (quicker, tougher, sharper, more resolute, more confident); the character screen shows the new score; nothing else is shown: no marks, no chance, no potential.

Hit points follow a Toughness rise through `raised_stats`, carrying limits a Strength one, as for items.

### 6.4 Expected pace

A fighting character lands 150–300 blows a level at the new curve: 8–15 Strength marks from blows plus some from burden and digging, so Strength rises about every third level early and every other level later (as marks per level grow with the kills per level), Toughness about as often from being hit, Dexterity a little less, and the mental stats only as the character reads, searches, talks, and trades: a half-orc barbarian who never opens a scroll keeps its Intelligence. Over 60 levels that is roughly **+25 to +35 points in all**, most in the two or three stats the character lives by (St 14 → 26 for a warrior who fights every level), comparable to ADOM's trained characters and to the 15 chosen points of the earlier draft doubled, since nothing is chosen. The `need` formula and the chances are the knobs; the wiki describes the uses in words ("fighting trains Strength, being hurt Toughness, reading Intelligence…"), never the chances.

**Not proposed:** player-chosen raises (the owner's call), stat talents, potions (later, as items), decay of marks.

## 7. Talents

### 7.1 The name

**Talents.** ADOM's word, and the owner's; "feats" carries D&D's rules baggage (and 3.5's reputation for feat taxes), "perks" is Fallout's. In the code: `Talent`, `TalentId`, `data/talents/`. The abilities menu's groups keep their names: a talent that is active appears there under a new `Source::Talent` group ("Talents"), between Class and Skills.

### 7.2 How many, and when

| When | Picks | Notes |
|---|---|---|
| Character creation | **2**; humans 3 (their racial `versatile`); **bards 3** (the lucky class: ADOM gives bards, merchants, and farmers a bonus talent, and a bard's trade is many small gifts); a human bard 4 | from the talents whose prerequisites the drafted character meets |
| Levels 3, 6, 9, …, 60 | **1 each: 20** | ADOM: every 3rd to 50 (16); 3.5: every 3rd (7 by 20) |
| Class bonus picks (§4) | 8 (warrior, thief) or 5 (the rest) | from the class's named trees only; Pathfinder 2e's lesson: separate pools never compete |
| In all by level 60 | **27–31** | of about 100 talents, so a career takes a third of the list, and no two builds need to look alike |

Every pick is made **as it comes**, on the level-up screen that opens with the level (§9.2); nothing is banked, and there is no respec (ADOM, DCSS: a choice is a choice). So that a pick is never impossible, **every tree has at least one talent without prerequisites** (the content test checks it: Healthy, Mighty kick, Opportunist, Far shot, Armour training, Keen ears, Long strider, Apprentice, Lean, Tunneler), and a class's bonus pick falls back to any tree if its trees have nothing the character may take.

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

Notation: **Needs** as in §7.3; **R** ranks (blank = 1); **A/P** active (an entry in the abilities menu, `Source::Talent`) or passive; **Hook**: the one rule that reads it. `#151` marks talents that wait for damage types, `mana` those that wait for the mana PR (§3.7), `#225` those that wait for alternative resources (§4.18); `action` ones that need a small new action (§4.1 B7). The four `#225` talents are the only class-limited ones: a resource belongs to the class that has it. Numbers are first guesses in data.

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
| `mighty_kick` | Mighty kick | | — | kicks at creatures 1d6 instead of 1d3; door kicks as if 4 stronger | `player_kicks`, `kick_door` | P |
| `breaker` | Breaker | | → mighty_kick, St 14 | door kicks as if 4 stronger again; digging ×1.25 | `kick_door`, `dig_work` | P |
| `iron_grip` | Iron grip | | St 13, a two-handed weapon skill at 3 | blows with two-handed weapons 10 % cheaper | `World::attack_price` | P |
| `battle_trance` | Battle trance | | class barbarian, Lv 6 | your rage starts 6 higher and decays a third slower | `data/resources.toml` Rage | P `#225` |
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
| `inner_calm` | Inner calm | | class monk, Lv 18 | ki +2 at most, and it comes back half again as fast | Ki | P `#225` |
| `evasion` | Evasion | | Dx 15, Lv 15 | bursts and bolts (traps, scrolls, spells) do you half damage | `cast_*` victim hook | P |
| `riposte` | Riposte | | Dx 13, a melee skill at 5, Lv 12 | a blow that misses you by your Parry gives a free blow back | B3, `why_missed` | P |
| `opportunist` | Opportunist | | — | a creature that steps away from beside you (flees) takes a free blow | B3, `World::flees` | P |

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
| `steady_hand` | Steady hand | | class ranger, Lv 12 | focus comes back half again as fast | Focus | P `#225` |
| `rapid_reload` | Rapid reload | | crossbows 3 | reloading 25 % cheaper | `attack_price` | P |

#### Guard (armour, Protection, standing firm)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `armour_training` | Armour training | 2 | — / St 13 | worn armour's `max_dex` +1 per rank | `Actor::defence` | P |
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
| `patient_blade` | Patient blade | | class thief, Lv 32 | openings decay half as fast, and a sneak attack that kills gives two | Openings | P `#225` |
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
| `quick_learner` | Quick learner | | In 14 | +1 skill roll every level | §5 | P |
| `apprentice` | Apprentice (*skill*) | 3 | — | train a skill you are untrained in, at 1d6+4; or +1d6+4 to one you are trained in | `Actor::skills` (dice from the `levels` stream) | P |
| `deep_well` | Deep well | 3 | In 12 or Wi 12 / Lv 10 / Lv 25 | +1 maximum mana per 4 character levels per rank (at least +2) | `Actor::max_mana` | P `mana` |
| `clear_mind` | Clear mind | | Wi 12 | mana comes back 25 % faster | `World::mana_rate` | P `mana` |
| `meditation` | Meditation | | → clear_mind | resting regains mana half again as fast | `mana_rate` while `resting` | P `mana` |
| `frugal_caster` | Frugal caster | | In 14, Lv 9 | abilities cost a fifth less mana (never below 1) | the ability's spend | P `mana` |
| `blood_magic` | Blood magic | | Wi 13, Lv 12 | give 5 hit points (never the last) for 3 mana | `hurt`, the mana PR's gain | **A** `mana` |

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
| `tunneler` | Tunneler | | — | digging 50 % faster | `dig_work` | P |
| `silver_tongue` | Silver tongue | | Pr 12 | shop favour +10 % | `world::favour` | P |
| `haggler` | Haggler | | → silver_tongue, appraise 20 | favour +10 % more, and the cap is 60 % | `favour`, `MAX_FAVOUR` | P |
| `well_liked` | Well liked | | Pr 13 | quest rewards' gold and XP +25 % | quest reward | P |

#### Racial (each needs its race; `versatile` is granted, not picked)

| id | Name | R | Needs | Effect | Hook | A/P |
|---|---|---|---|---|---|---|
| `elven_accuracy` | Elven accuracy | | race elf, bows 3 | threat +1 with bows | `Profile` | P |
| `ageless` | Ageless | | race elf, Lv 9 | hunger burns 20 % slower; diseases progress every second day | `digest`, `disease_turn` | P |
| `dual_heritage` | Dual heritage | | race half_elf | +1 skill roll every level | §5 | P |
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

**Count**: 10 + 9 + 9 + 11 + 9 + 11 + 9 + 14 + 6 + 8 + 15 = **111 talents** (2 of them `#151`, 5 `mana`, 4 `#225`: those wait for their systems), 4 active (`second_wind`, `hooded_light`, `tinker`, `blood_magic`), 6 with a choice, 13 with ranks. Trimming to 80 for the first release is easy (the last two of each tree), but the list is written so every tree has a path in it and every system in §8 has its two.

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
| **Mana** (the mana PR; §3.7) | deep_well, clear_mind, meditation, frugal_caster, blood_magic | the 14 mana-costing actives, arcane_mastery, archmage, virtuoso | pending: in the list once the mana PR lands |
| **Resources** (#225; §4.18) | battle_trance, inner_calm, steady_hand, patient_blade | rage, blood_frenzy, mighty_rage, unstoppable, rapid_shot, volley, improved_sneak_attack, master_thief, flurry, quivering_palm, inspire | pending: in the list once #225 lands |
| Stat training (§6) | — (every talent that makes a use happen more often trains the stat behind it) | — | not a system of its own: the training channel the level-up reads |

The rows at the bar are the systems with the least surface (stances are four rows of data; carried light is a lantern and item glow). The policy below keeps every row at or above it.

### 8.2 Policy for new systems

Concrete, and checked by a test rather than by memory:

1. **Every talent and class ability carries `systems = ["traps", "searching"]`** in its data, from a closed list of system tags kept in `data/talents.toml` (`[systems]`, one line per system with a name for the wiki). A content test fails when a tag in the list has fewer than **2 talents** or fewer than **1 class ability**, and when a talent names a tag not in the list. Adding a system without touching the list passes (nothing requires it); adding it to the list without talents fails. So the rule is: **a system is "in the list" when it ships with its talents.**
2. **The issue template** (`.github/ISSUE_TEMPLATE/feature.md`, or the feature issue body convention in `spec/13-github-workflow.md`) gets a **"Talents and class abilities"** section: *"Which talents and class entries does this system ship with (at least two talents; name the trees), or why none?"* A feature PR that adds a system adds its tag and its talents in the same PR, as it updates the spec, CLAUDE.md, and the wiki today.
3. **The wiki's Talents page** is generated per tree with a column of systems, and a **"By system"** table (system → talents and class abilities) comes from the same tags, so a gap is visible to players and the owner alike.
4. **The coverage test's thresholds are data** (`min_talents = 2`, `min_class = 1`), so the bar can rise as the list grows.

Pending systems with their first talents already named here: mana (the mana PR: five Scholar talents and the casters' costs), alternative resources (#225: four class talents and ten class features, §4.18), damage types (#151: `elemental_ward`, `crusher`, `blight`'s type), the spell system (replaces the caster placeholders and gets a Spellcraft tree), pits and climbing (#128: `light_step` rank 2, Athletics), food freshness (#196: `forager` would go here), potions (a Vigour/Scholar talent each).

## 9. What the player sees

### 9.1 Messages

In the order they come, after the kill's own messages (as today's "You feel more experienced." / "Welcome to level 3."):

| Event | Message |
|---|---|
| XP gained from a quest or discovery | "You gain 15 experience." (quests say it today); "You learn something of kobolds." (first kill, with the XP); "You gain 20 experience for finding your way here." (arrival) |
| a level | "You feel more experienced." "Welcome to level 3." then the stat rises, if any ("You feel stronger."), then the level-up screen opens for what must be chosen |
| the class table | "You have learned to cleave." (passive) / "You can now rage." (active: it is in the abilities menu) |
| a skill rolled | "Your stealth improves." (the marks' phrase) |
| a talent chosen | "You learn Cleave." / "You learn Hardy (2)." |
| an ability on cooldown | "You are still winded." (refused, no time) |
| any other action while a choice is due | "You must choose a talent first." / "You have skill rolls to spend." (refused, no time: the rules enforce it, §10.2) |

Never: a die, a modifier, a potential, a mark, a chance, a speed.

### 9.2 The level-up screen

`ScreenName::LevelUp`, one screen on the one stack (`ui/level_up_screen.rs`), **opened by `screens::on_level_up`** at the end of the turn in which a level came (as `on_death` opens the death screen; the death screen wins if both apply) and nowhere else: there is nothing to come back for, since nothing is banked. No time passes on it. It walks the level's choices in order, each the one paged list (§6 "Item lists"), skipping steps with nothing to choose; the frame says which step and what is left ("Level 12 · Skill rolls 3 · Talent 1"):

1. **Roll skills**: the trained skills as in §5.4; one letter per roll, until none is left.
2. **Choose a talent**: the talents the character may take now, grouped by tree under headers (as item lists group by class), each row the name, its rank if ranked ("Hardy 2/3"), and a short effect in words; talents whose prerequisites are not met are **not listed** (the Talents page lists every talent with what it needs, so planning happens there and the pick list stays short); a choice talent opens a second list (the weapon skills, the styles, the elements) in its place (`ScreenAction::Replace`). The selected row's full description shows below. A bonus pick from the class's trees is its own step with only those trees.

Every letter is an `Act(Action)` (`advance:SKILL`, `talent:ID[:choice]`), applied at once, recorded in the replay, and the screen refreshes with the result (`screens::refresh`), like the equipment screen. **It cannot be left** while a choice is due: Esc and the screen's own key say "You have 2 skill rolls to spend." in the message line and stay; when the last choice is made it closes by itself. (The stat checks of §6 need no step: they are rolled by the rules as the level is gained and only say their results.)

**Several levels in one turn** (one kill worth two levels): the rules hand out every level's gains at once (`Progress::claimed` catches up level by level, each level's rolls and talent counted), and the screen walks them as one sequence, its frame naming the level each step belongs to ("Level 4 of 3–4 · Talent"). **A level during a long action** (a rest, a dig, a search, a meal; a creature bleeding to death from the player's wound mid-rest): the level's messages are `say`s, so `World::keep_at` stops the action as any message stops it, the turn ends, and the screen opens; the action is not resumed by itself. **In the headless CLI and replays** the same holds through the rules alone: while a choice is due, `World::apply` refuses every action but the choices ("You must choose a talent first."), so a replay carries `advance:…` and `talent:…` steps right after the kill, and `warlock-cli dump --actions` must spend them too (a `levelup:auto` dev action takes the first eligible choice of each kind, for tests and benchmarks).

Keys: the list commands (letters, `<`/`>` pages, arrows, Enter on the `>` row), all in the existing rebindable contexts; no new key.

### 9.3 The Talents info page

`ScreenName::Talents`, **page 7** of the info pages (`PAGES`: Character, Inventory, Equipment, Skills, Weapon skills, Quests, **Talents**; tab label "Talents"), reached by `Page(7)` and Tab in the pages context and by a rebindable `KeyCommand::Talents` of its own (id `talents`), unbound by default like `Back`, so no new default key is taken in play. It shows:

- the character's talents, grouped by tree, each with its rank and its effect in words (from data: `description`);
- below them, dimmer, **every other talent** of the game by tree with what it needs (`Needs: Dexterity 13, Dodge`), what is met in the normal colour and what is not dimmed, so the player can plan a build without the wiki; 72 wide like the character screen, scrolling with ↑↓ where it does not fit, the selected talent's full description below.
- The class's coming abilities are **not** listed (the wiki has them: in the game the class table is learned by levelling, like ADOM's).

The abilities menu lists active talents under "Talents" and class actives under "Class", each greyed with its cooldown's reason while one lasts ("you are winded").

### 9.4 Character creation

A new step **Talents** after Class and before the Summary (`ScreenName::ChargenTalents`, a `ChoiceScreen` like the others, but a multi-pick: 2 rows marked, 3 for a human or a bard, 4 for a human bard): the list of talents whose prerequisites the drafted character meets (the start is computed as the summary computes it: seed, choices, start date), grouped by tree, each with its one-line effect; a **Random** row (as the other steps have) draws the picks from the stream `character/talents/<race>,<gender>,<class>`; Esc/Back returns to Class with the picks kept in the draft (`ScreenStack::draft`). The Summary gains a `Talents  Hardy, Alert` line. On the command line, `--talents hardy,alert` beside `--character`; **without it the Random row's picks are taken** (they are a pure function of the seed and the spec), so every scenario, replay, and golden entry starts with its talents and nothing waits. Replays record them in `character` (`{race, gender, class, talents}`); an old replay without the key gets the Random picks, which is what it would have got then too. The race and class steps add a line in words for the experience factor ("Levels come slowly." for 115 and above, "quickly" for 95 and below) and for the rolls ("Learns quickly." for 3 or more).

### 9.5 The character screen and the Skills screen

The character screen's `Lv  Level  3` / `XP` rows stay; a `Talents  7` row is added after them. The Skills screen's four-cell meter now shows the skill's ladder step (how practised it is), with the same glyphs. The status bar gets nothing new (it is full).

### 9.6 What stays hidden

The rules the talents stand on keep their numbers where the game hides them today: speed (Quick says "you move a little quicker"), checks and modifiers (Strong constitution says "you shake off disease more easily"), potentials and stat marks (a rise says "You feel stronger." and nothing of why), skill marks and dice (the meter shows a step, never a number), to-hit (Weapon focus says "you hit more often with long blades"). Where the game **shows** a quantity (Defence, Protection, hit points, carrying limit, prices in shops, XP), the talent says the number ("Dodge +1", "+10 % carrying", "XP +5 %"). The in-game text is the talent's `description` in data; the **wiki** tables show the rules in full (the +2 on disease checks, the percents, the ladder, the stat uses in words without chances), as the wiki shows skill benefits and stance numbers today. Creatures' talents (none yet) would never show.

### 9.7 Keys

All new commands are rebindable with stable ids in `settings/keys.rs`: `talents` (the page, unbound; `Page(7)` reaches it), the class and talent abilities (each an unbound `KeyCommand` with its ability's id, as the shield bash is); the list commands are the existing ones. No new fixed key, and no key for the level-up screen: it opens itself.

### 9.8 Wiki

| Page | Content | Generated tables (`xtask/src/wiki/`) |
|---|---|---|
| Experience (existing) | the curve to 60 in kills and XP, the factors, discovery | `experience-levels.md` (to 60, with the factor examples), `experience-factors.md` (races and classes), `experience-discovery.md`, ZD table to 60 |
| Classes (existing) | the level tables, rolls and known-for skills | `class-levels/<class>.md` per class (level, name, what it does), `class-rolls.md` |
| Races (existing) | the experience factor, racial talents, the extra roll | `races.md` gains the columns; `talents-racial.md` |
| **Talents** (new, under Your character) | the rules of §7.2–7.4, then the trees | `talents-<tree>.md` per tree (name, ranks, needs, effect as a rule, systems), `talents-by-system.md` |
| Skills (existing) | the ladder and the level-up rolls | `skill-ladder.md` |
| Stats (existing) | what trains each stat, in words | `stat-training.md` (uses only, no chances) |
| Abilities (existing) | the class and talent actives | `abilities.md` (from the registry, as today) |

## 10. Data and code sketch

### 10.1 Data files

| File | Change |
|---|---|
| `data/experience.toml` | `max_level = 60`; the curve as `[kills] plateau = 213, ramp = 5, slope_per_100 = 285` and `guideline = { per_level = 5, base = 10 }` (`linear` and `quadratic` go; a test pins the generated table's hash, as the sun's table is pinned); `[levels] talent_every = 3, creation_talents = 2`; `[discovery] first_kill = 100, arrival = 5, identify = 10`; `[[zero_difference]]` to 60 |
| `data/races.toml` | per race: `experience = 140` (percent), `talents = ["versatile"]` (granted), `rolls = 1` (humans); `mana` (the mana PR's field, values in §3.7) |
| **`data/classes/<id>.toml`** (one file per class; `classes.toml` goes) | what `[[class]]` has today, plus `experience = 125`, `mana = 12` (the mana PR's field), `rolls = 2`, `known_for = [...]`, `creation_talents = 1` (the bard), `bonus_talents = { levels = [10, 20, 30, 45, 55], trees = ["guard", "vigour"] }`, and `[[level]]` rows: `level = 6, ability = "shield_wall"` with the ability's own table (`[[ability]] id, name, description, kind = "passive" or "active", aim, cost, effect = {...}, cooldown = "winded", systems = [...]`, and for casters a `burst`/`bolt` table as scrolls have). Classes are numbered in path order like objects (README.toml) |
| `data/skills.toml` | `[increase]` goes; `[ladder] dice = ["1", "1d3", "1d5", "2d4", "3d3", "3d4", "3d5", "4d4", "4d5"], marks = [0, 5, 12, 25, 50, 100, 200, 400, 800]`, `[rolls] intelligence_from = 14, intelligence_every = 4, spend = 50` (percent of the marks a roll consumes) |
| **`data/stats.toml`** (new) | `[training] need_base = 20, need_per_point = 3, over_potential_divisor = 4`; `[[training.use]] stat = "strength", on = "blow", chance = 5` … (the table of §6.2) |
| **`data/talents.toml`** | settings: `[systems]` (the closed tag list with names), `[coverage] min_talents = 2, min_class = 1, free_per_tree = 1`, `max_ranks = 3`, the trees (`[[tree]] id, name, description`, in display order) |
| **`data/talents/<id>.toml`** (one file per talent, README.toml, numbered in path order like affixes) | `[[talent]] id, name, tree, ranks, choice, description` (words, in game), `needs = { talents = ["dodge"], stats = { dexterity = 13 }, skills = { tumble = 25 }, weapon_skills = { bows = 3 }, level = 9, race = "dwarf", eyes = "darkvision", class = "warrior" }` (per rank: `[[talent.rank]] needs = {...}, effect = {...}`), `effect = { kind = "...", ... }` (a typed enum, §10.3), `systems = ["traps"]`, `active = { aim = "nothing", cost = "wait", cooldown = "winded", mana = 0 }` |
| `data/status.toml` | B1: optional `to_hit`, `damage`, `dodge`, `parry`, `cost`, `blocks`, `follows`; the new statuses `raging`, `exhausted`, `winded`, `drained`, `spent`, `blessed`, `inspired`, `hoarse`, `hexed`, `weakened`, `slowed`, `staggered` |
| `data/factions.toml` | `animal = true` on the vermin (and the deer's) faction for `wild_empathy` and `beast_friend` |

Why a directory for talents and classes: both are long tables that churn (every system adds a talent; class tables are 15 × 9 rows and will be retuned), the repo's rule for churning tables is one file per entity (objects, affixes, creatures, materials: `docs/dev/content.md`), and parallel PRs adding talents then never conflict. `talents.toml`, `stats.toml`, and `experience.toml` stay single files: settings, not tables.

### 10.2 State

```
WorldState
  experience: u32                      (exists)
  progress: Progress                   (new; left out of the JSON while default)
    talents: Vec<Taken { id: String, rank: u8, choice: Option<String> }>   by id, like affixes: adding talents keeps saves
    due: Due { rolls: u8, talents: u8, bonus_trees: Vec<Vec<TreeId>>, level: u8 }   what this level-up still asks for; never carried past it
    claimed: u8                        the level whose gains have been handed out (a kill that spans levels hands out each)
    once: BTreeSet<String>             one-per-run flags: "miracle", "hags_bargain"
    discovered: Discovered { kills: BTreeSet<CreatureId>, affixes: BTreeSet<String> }
    empathy_broken: BTreeSet<CreatureId>   wild_empathy, beast_friend
  levels_rng: Rng                      (new) the `levels` stream: skill rolls, Apprentice's dice, the stat checks
Character (saved, in WorldState)
  talents: Vec<String>                 the creation picks (the spec, for replays and `--talents`)
Actor
  talents: Talents                     (new) the player's taken talents, resolved to TalentIds for the rules; creatures' empty (a creature type may list `talents` later with no new code)
  skills: Skills                       (exists) marks now a reservoir: value, marks; no `progress` toward a point
  stat_marks: PerStat<u16>             (new) stat training marks, the player's
  effects: Effects                     (exists; cooldowns and buffs live here)
```

**No banking in the rules**: while `progress.due` asks for anything, `World::apply` refuses every action but `Action::Advance`, `Action::Talent`, and the dev `levelup:auto` ("You must choose a talent first."), so the CLI, replays, and the UI all spend a level on the spot. `FORMAT_VERSION` bumps once for the lot (the thresholds alone change what level a saved XP total is; a save at a level whose gains were never handed out, from before this change, hands them out on load at the first turn, once). The class table's passive abilities are **not stored**: they follow from the class and the level, read through the same aggregation as talents (`Actor::talents` is the player's picks plus the class entries at or below their level, computed on load and on level-up). The state hash covers all of it.

### 10.3 Where the rules hook in

One aggregation, read where the affixes already are: `Talents::mods(content) -> TalentMods` (summed typed effects: `dodge: i32`, `natural: i32`, `protection: i32`, `max_hp_per_levels`, `healing_percent`, `resting_percent`, `step_percent`, `noise_percent`, `hearing: i32`, `favour_percent`, `xp_percent`, `check: PerStat<i32>`, `vision: VisionMods`, flags…), and per-attack ones through `Actor::attack_benefit`-style readers (`TalentMods::for_attack(kind, skill, style, target)`). Each existing hook gains one read:

| Hook (exists) | Reads |
|---|---|
| `Actor::effective_stats`, `raised_stats` | class stat gifts and level-up rises (stored in `Stats`, no read) |
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
| `World::stance`, `World::refusal` | wild_empathy, beast_friend, noblesse |
| `hurt`/`dies` | immune_to_pain, unstoppable, miracle, hags_bargain |
| **`World::train(Use)`** (the one mark reporter, skills' and stats') | the uses of §6.2 at the places named there: `apply` (burdened), `resolve` (blow, dodge, hungry_fight), `missile_hits` (shot), `make_noise` (sneak), `tread_price`/`spring_trap` (climb), `hurt` (hurt), `after_blow` (big_hit), `after_tread` (plunge), `disease.rs` (resisted, resisted_will), `search.rs` (found, spot), `read.rs` (read), `identify` (identified), `earn_kill` (discovered), `keep_at` (endured), `talk.rs` (chat, swapped), `shop.rs` (trade), `quest.rs` (quest), `dig_turn` (dig), `kick_door` (kick) |
| **`World::level_up`** (new, in `world/level_up.rs`, from `grant_xp`) | hands out the level's hit points and mana, class entries, due rolls and talents, and runs the six stat checks (`levels` stream), level by level |
| the mana PR's hooks (`Actor::max_mana`, `World::mana_rate`, the ability's spend) | `deep_well`, `clear_mind`, `meditation`, `frugal_caster`, `blood_magic`; the casters' costs; `arcane_mastery`, `archmage`, `virtuoso` |

New modules: `content/talent.rs` (defs, prerequisites, effects, the systems list, the coverage test), `talents.rs` (the runtime: `Talents`, `TalentMods`, `may_take`), `content/class.rs` (grows the level table and abilities), `content/stat_training.rs` (`data/stats.toml`), `world/level_up.rs` (the hand-out, `Action::Talent`, `Advance`, the due-choice refusal, the stat checks, messages), `world/experience.rs` (the curve in kills, the factor, discovery), `ability.rs` (`Source::Talent`, class actives, `Unavailable::While`), `ui/level_up_screen.rs`, `ui/talents_screen.rs`, `ui/chargen_screen.rs` (the step), `xtask/src/wiki/{talents,classes,stats}.rs`, and `warlock-cli` dump layers `talents` (the player's, with ranks and choices) and `progress` (due choices, discovered, stat marks; dev).

### 10.4 Determinism

| Draw | Stream | Note |
|---|---|---|
| skill rolls, Apprentice's dice, **the stat checks** | **`levels`** (new in `WorldState`, like `training`) | never the gameplay stream: a level-up moves no fight. The stat checks draw six times per level in stat order, always, so the stream stays in step whatever the marks |
| skill and stat **marks** | `training` (exists) | one draw per reported use with a chance below 100 |
| creation picks' Random row, and the picks of a run started without `--talents` | `character/talents/<spec>` | the picks are recorded in `character` either way |
| talents with a chance in play (`lucky`, `spell_recall`, `light_step`'s check, `staggering_blow` through the critical) | the **gameplay** stream, drawn only when the condition arises | the same rule as checks: a run without the talent draws nothing extra |
| discovery XP, class hand-outs, the thresholds | no draw | |
| loot talents (`treasure_hunter`, `gem_lore`, `jeweller`) | the `loot/<level>` and `drop/<actor>` streams, as today | they change what is rolled, never the map; a level generated before the talent was taken keeps its loot |
| cooldowns | game time (statuses) | |

Every pick is an action with replay text (`talent:hardy`, `talent:weapon_focus:long_blades`, `advance:stealth`), `Spent::NoTime`, refused with a message when nothing is due or the prerequisites fail ("You cannot take Cleave: you need Strength 13."). The hash covers `progress`, `levels_rng`, `Actor::talents`, `stat_marks`, and the skills' marks.

## 11. Implementation plan

Issue-ready items. Each becomes one GitHub issue (title as given), one branch, one PR, with the spec, CLAUDE.md/`docs/dev/progression.md`, and wiki updated in the same PR as the repo's rule is. "Size" is S (a day), M (two to three), L (avoid: split). Items in the same wave touch no common file, so they run in parallel under the seven-agent cap; the merge order inside a wave is free. The critical path is **A → D → G** (three PRs) for a playable loop; everything else widens it.

| # | Issue title | Implements | Depends on | Size | Parallel with |
|---|---|---|---|---|---|
| **A** | Experience to 60: the curve in kills, race and class experience factors, discovery XP; the per-level mana gain once the mana PR has landed (else a follow-up line in H6) | §3.1–3.5, §3.7 | — (the mana PR for the mana line only) | S | B, C |
| **B** | Status effects as buffs and cooldowns: combat fields, `blocks`, `follows`, `Unavailable::While` | §4.1 B1 | — | S–M | A, C |
| **C** | Free blows: `second_blow` for kill, parry, and flee triggers | §4.1 B3 | — | S | A, B |
| **D** | Talents core: data model and loader, `Actor::talents` and `TalentMods`, `Progress` and the `levels` stream, `talent:`/`advance:` actions, due choices refused in `apply`, the coverage test, CLI layers, saves and replays; the first 12 modifier talents (`healthy`, `hardy`, `dodge`, `brawn`, `silent_feet`, `keen_ears`, `pack_mule`, `long_strider`, `lean`, `good_learner`, `alert`, `thick_skin`) | §7.1–7.4, §10 | A | M | — |
| **E** | Skill rolls at level-up: the ladder replaces `[increase]`, marks as a reservoir, class `rolls` and `known_for`, race `rolls`, the Intelligence thresholds, `data/classes/` split from `classes.toml` | §5 | D | M | F, H1, H2, J1–J4 |
| **F** | Stat training: `data/stats.toml`, `stat_marks`, `World::train(Use)` for stats at the 24 hooks, the level-up checks on the `levels` stream, messages | §6 | D | M | E, H1, H2, J1–J4 |
| **G** | Level-up screen, Talents info page (page 7), the character creation Talents step, `--talents` | §9 | D, E (the rolls step), F (nothing to choose, but the messages) | M | H3, H4, I1–I4 |
| **H1** | Talents: Vigour, Might, Finesse | §7.5 (26 talents) | D; C for `cleave`, `riposte`, `opportunist` | M | E, F, H2 |
| **H2** | Talents: Weaponcraft, Guard | §7.5 (19) | D | S–M | E, F, H1 |
| **H3** | Talents: Shadow and Wayfarer, with effective eyes (B4) and Defence by attack kind (B5) | §7.5 (20), §4.1 B4–B5 | D | M | G, H4 |
| **H4** | Talents: Scholar, Grit, Delver | §7.5 (23) | D | M | G, H3 |
| **H5** | Racial talents and `versatile` | §7.5 (15) | D, G | S | I1–I4 |
| **H6** | Mana talents: `deep_well`, `clear_mind`, `meditation`, `frugal_caster`, `blood_magic`; the `cast` stat use | §7.5, §6.2, §3.7 | D, F, **the mana PR** | S | I1–I4 |
| **I1** | Class tables: warrior, barbarian, paladin, monk | §4.2–4.5 | D, B, C | M | I2–I4 |
| **I2** | Class tables: ranger, thief, bard, with the kill tally for every type (B6) | §4.6–4.8, §4.1 B6 | D, B; J1–J4 for the actives that need them (else shipped greyed "not usable yet") | M | I1, I3, I4 |
| **I3** | Class tables: priest, druid, wizard, witch, the caster placeholders on the shared cast code, costing mana | §4.9–4.12, §3.7 | D, B, **the mana PR** | M | I1, I2, I4 |
| **I4** | Class tables: aristocrat, commoner, craftsperson, merchant | §4.13–4.16 | D, B, C | S–M | I1–I3 |
| **J1** | Disarm traps: the Disable device action | §4.1 B7 | B | S | J2–J4 |
| **J2** | Pick locks: the Open lock action and lock difficulties | §4.1 B7 | B | S | J1, J3, J4 |
| **J3** | Steal: the Steal action | §4.1 B7 | B | S | J1, J2, J4 |
| **J4** | Throw your voice, and hood the lantern | §4.1 B7 | B | S | J1–J3 |
| **K** | Coverage policy: the issue template section, `docs/dev/progression.md`, spec and CLAUDE.md bullets, the wiki's "by system" table | §8.2 | D | S | anything after D |
| **L** | Damage-type talents: `elemental_ward`, `crusher`, `blight`'s type | §7.5 `#151` | #151, D | S | R1–R5 |
| **R1** | Resources: barbarian rage as a Rage pool (from the Raging status), blood frenzy, mighty rage, unstoppable | §4.18 | **#225**, I1 | S–M | R2–R5 |
| **R2** | Resources: ranger Focus, rapid shot and volley on it | §4.18 | **#225**, I2 | S | R1, R3–R5 |
| **R3** | Resources: thief Openings and the master thief's finisher | §4.18 | **#225**, I2 | S | R1, R2, R4, R5 |
| **R4** | Resources: monk Ki, flurry and quivering palm on it; bard Inspiration in place of mana | §4.18 | **#225**, I1, I2 | S | R1–R3, R5 |
| **R5** | Resource talents: `battle_trance`, `inner_calm`, `steady_hand`, `patient_blade` | §7.5 `#225` | **#225**, D | S | R1–R4 |

**Waves** (each within the seven-agent cap): **1** A, B, C · **2** D · **3** E, F, H1, H2, J1–J4 (seven) · **4** G, H3, H4, I1, I2, I4 · **5** I3, H5, H6, K, L · **6** R1–R5, after #225 (the owner: after all current issues). **The mana PR** gates I3 and H6 (and A's mana line); if it lands before wave 4, I3 joins wave 4. Everything else is independent of it: the class `mana` values of §3.7 are data the mana PR may take as given.

**Splitting the content-heavy items.** The talent waves are cut by tree so each PR is 20–26 one-file talents plus their hooks, and no two PRs touch the same hook file where it can be helped (H1 owns `fight.rs`'s profile reads and `second_blow`, H2 `defence` and the protection sum, H3 `noise.rs`, `sight`/`eyes`, `step_price`, H4 `search.rs`, `shop.rs`, `read.rs`, `eat.rs`); where two must touch one hook (`healing_rate`: H1's `fast_healer`, H3's `sun_blessed`) the first to land adds the read and the second only its talent. Class tables are cut by four classes of a kind so each PR is one data file per class and about 36 entries, and shared entries (`evasion`, `woodland_stride`, `divine_health`, `second_wind`, `smite`) are implemented by the first PR that needs them and named by the others. D ships enough talents to prove every hook kind (a flat Defence bonus, a percent, a per-level scaler, a noise percent, a check modifier), so the waves after it are data plus one read each.

**Tests each item carries** (the repo's bar): unit tests in `warlock-core` for the rules; a replay created and verified for determinism (every level-up in it spent by `advance:`/`talent:` steps); the content coverage test from D; insta snapshots where the generators are touched (none expected); golden entries for the three new screens (G), advisory while #166 stands; `cargo xtask wiki --check`.

## 12. Decisions

The owner reviewed the first draft (PR #220) and approved it to implement with the changes now folded in above. For the record:

**Answered by the owner**

1. **Kills, not XP, and more of them**: about 3 000 even-level kills to level 20 and 15 000 to level 60 (§3.2, the new curve).
2. **Skill increases lean on ADOM**: marks set the dice, a roll consumes marks and shrinks them, the class sets the rolls (one class with "many rolls" as its identity: the bard), humans +1, Intelligence more, class and race affinities (§5).
3. **No player-chosen stat raises**: stats train by use through marks and are checked at every level-up, a rise consuming the marks (§6).
4. **Talent counts stand**; one lucky class gets another: the bard, at creation (§7.2).
5. **Class signature abilities at fixed levels stand**; the tables are not set in stone and may favour some classes later; some classes are allowed to be memes (the druid's early entries stay as they are).
6. **No banking**: every level-up choice is made at once (§9.2, enforced by the rules, §10.2).

**Decided on the recommendations, under the owner's "looks good"**

7. Discovery XP: the first kill of a type and the first arrival on a level now; affix identification later.
8. `hp_level` stays 20; Hardy and the barbarian's `hardened` buy more hit points.
9. Caster classes get placeholder actives through the scrolls' shared cast code with cooldown statuses, marked `spell` in data for the spell system to replace.
10. Cooldowns on the martial actives are visible, time-based statuses (Winded, Drained, Spent, Exhausted); the casting actives cost mana instead (§3.7).
11. No respec.
12. Talents have up to 3 ranks and may take a choice (Weapon focus (long blades)).
13. Creation picks: 2 (humans and bards 3), a Talents step before the Summary; `--talents` on the command line, the Random picks without it.
14. The level-up screen opens by itself at the end of the turn and cannot be left until its choices are made; several levels in one turn are one sequence; a level stops a long action as a message does.
15. Class bonus picks: 8 for the warrior and the thief, 5 for the rest, the commoner's from any tree.
16. The four small actions (disarm, pick lock, steal, throw voice; hooding the lantern with them) ride on this milestone as separate issues.
17. `Actor::talents` on every actor, so creatures may get talents from data later.
18. All ~100 talents ship (the `#151` ones with #151).

**Decided in this revision, flagged for a nod**

19. Skill marks feed **only** the level-up rolls; the by-use point gain (the provisional `[increase]`) is removed (§5.2: one channel, one curve, no double counting). The alternative, keeping both, is a data switch away if play misses growth between levels.
20. Potentials stay a **soft** cap on stat training: the level-up chance is quartered at or above the potential and a rise there lifts it (§6.3), so the hidden number never shows as a wall.
21. The expected stat growth is about +25 to +35 points over 60 levels, by use, with `need = 20 + 3 × score` as the knob (§6.4).
22. **Mana** (the owner's new request, built in its own PR): class and race `mana` values with the aristocrat as the oddly gifted meme class; a per-level gain to a `mana_level` beside the hit points; the casters' actives cost mana **instead of** cooldown statuses (mana comes back with time), the martial actives keep theirs; spending mana trains Willpower; five Scholar talents on the mana hooks; mana in the coverage list (§3.7, §7.5, §8).
23. **Alternative resources** (#225, after this milestone): rage, focus, openings, ki, and inspiration as pools with the parameters of §4.18; each feature ships first as the status or passive its table gives and migrates in R1–R5 of §11; four class-limited talents; a coverage row.
