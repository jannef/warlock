# Warlock — No caster level: learning by Spellcraft alone

**Built; this file is history.** Current behaviour is in spec §8 "Spells" and `docs/dev/spells.md`.

Design document for the owner's review (#977), revised after the owner's approval of draft PR #981 (§9 records the decisions: no learning without Spellcraft). Written 2026-11-05 against `origin/main` at 6f43a36a. Nothing here is implemented; §8 cuts it into issues. Every number below was read from the files named or computed from them by a throwaway script (the 2d12 ways as `check::ways` counts them, the curves as `Benefit::at` rounds them); nothing was built or run in the game, and anything not checked is marked *(unchecked)*.

**The owner's request** (#977): *"abolish the caster level concept. Spell learning should only come from the Spellcraft skill. Give casters appropriate starting points in that."* The decision is final: caster level goes. This document keeps it under no other name and computes no hidden value from the character level.

## Contents

1. Summary: the new rule, and the issue's nine items
2. The learning rule, with worked numbers
3. Starting points per class, and how Spellcraft grows
4. The spells a class gives at a level
5. Inventory: everything that reads caster level, each with its verdict
6. Training Spellcraft
7. Interactions: Skill focus (#963), the skills investigation (#973), one rank per talent (#969)
8. Implementation issues
9. Decisions

## 1. Summary

| | Today (`world/caster_level.rs`, `world/learn.rs`, `data/study.toml`) | The design |
|---|---|---|
| What decides what a scroll teaches | **caster level** = character level × the class's `casting` % ÷ 100, + talents (Arcane training) + worn items (of the Adept, half again with Attuned); at least 1 with any mana | **learning** = a benefit of the Spellcraft skill, `learn`, a curve in `data/skills.toml` from the skill's value (with what affixes and talents add to it); at least 1 for a character trained in Spellcraft; **a character not trained in it cannot study a scroll at all** (refused, no time), cantrips included |
| Who may study | anyone with a scroll and mana | only a character **trained in Spellcraft** (the skill in their trained set; an item's or talent's bonus alone does not train it); anyone else is refused, with no time spent. Reading a scroll to cast it, and the spells a class grants, need no Spellcraft |
| Certain | a spell of level ≤ caster level | a spell of level ≤ learning |
| Above it | overreach = level − caster level; a 2d12 check: Intelligence modifier + Spellcraft ÷ 10 + Decipher script ÷ 10 + the class's affinity + Quick study, against 12 + overreach (Overreach: a third fewer) + known levels ÷ 20 (+ `study_difficulty`) | overreach = level − learning; the same check, Spellcraft's tenth kept |
| Who has Spellcraft at the start | wizard and witch (2d6+10, known for); the priest one time in four (2d4); nobody else | every class that casts: the four full casters 2d6+10, the bard, paladin, and ranger 2d4+4, the aristocrat and ascetic by chance (§3); those without it cannot learn spells |
| The number in the game | hidden; the dumps' `Caster level 3`, `cl 8` | hidden; Spellcraft on the Skills screen as any skill; the dev `progress` layer's `learning 36 (Spellcraft 108)`; the wiki states the curve |

**The issue's nine items, and where each is settled:**

| # | Item | Where | In one line |
|---|---|---|---|
| 1 | The new rule | §2 | learning is Spellcraft's `learn` curve, `[[10, 0], [60, 10], [100, 30], [150, 70]]`: 0 below 10, 10 at 60, 30 at the soft cap 100, the last spell level (70) at 150; certain at or below it, the check above it as today; no learning at all untrained |
| 2 | Starting points | §3 | the dice per class, race (none), and birth sign (none), and the growth by level-up rolls, with the odds at levels 1, 10, 30, 60 |
| 3 | Spells classes grant | §4 | a class's spell is the player's at the level its table names, whatever their Spellcraft; the load-time test that held a given spell to the class's caster level goes, its count checks stay |
| 4 | Things that give caster level | §5.1–5.2 | **of the Adept** deleted (of the Scholar is the affix for learning; the Eorcanstān re-pointed); **Arcane training** → Spellcraft +10 (`skill`, an existing kind); **Attuned** deleted with its kind; **Overreach** and **Quick study** unchanged; no creature gives or reads one |
| 5 | Creatures that cast | §5.3 | they read nothing today but the dev dump's `cl N`, which goes; their `spellcraft` values stay data nothing reads (a #973 row) |
| 6 | Training | §6 | study alone is too thin: a `cast` mark at 25 % as Concentration's, the one line in `skills.toml` |
| 7 | Everything else that reads it | §5.4 | dumps, JSON, the probe, spec, docs, wiki, tests, one row each |
| 8 | Interactions | §7 | Skill focus on Spellcraft is +25 → +5 to +20 learning by where on the curve; #973 reads this document's Spellcraft row; #969 changes nothing here |
| 9 | The split | §8 | N1 the rule and the starting points; N2 the talents, the affix, the unique; N3 caster level itself (code, `casting`, dumps, probe, docs); N4 the cast mark |

## 2. The learning rule

### 2.1 The rule

| Piece | Today | The design | Where |
|---|---|---|---|
| Learning | caster level (above) | for a character trained in Spellcraft, `Actor::skill_effect(content, Effect::Learn)` (`world/actor.rs` 466: the skill's value plus skill affixes and the talents' `skill` bonuses, read through `Skills::effect_above`), **at least 1**; for a character not trained in Spellcraft there is no learning: `study_odds` gives no odds and the study is refused (the row below) | `world/learn.rs::study_odds`, the one read; it drops its `caster_level` argument |
| Certain | level ≤ caster level, and `study_difficulty` ≤ 0 | level ≤ learning, and `study_difficulty` ≤ 0 | `study_odds` |
| Untrained | no such state: caster level is at least 1 with any mana | **refused, no time spent, for a character not trained in Spellcraft**, cantrips too, in words the player can understand (N1 fixes them, in the vein of "You have no training in Spellcraft; you cannot make sense of how to learn a spell from a scroll."). "Trained" is membership of the trained set: Of the Scholar, Arcane training, or Skill focus on an untrained character add to a skill they do not have and do not open learning | the study's start (`learn.rs` 165), beside its other refusals |
| Cantrips | anyone with mana has caster level ≥ 1, so a level-0 or -1 spell is certain | for the trained, learning is at least 1 (the floor above), so a level-0 or -1 spell is certain; for the untrained, refused with the rest | `study_odds` |
| Overreach | level − caster level | level − learning (Overreach's third off as today) | `Odds::overreach`; the mishap band reads it as today |
| The check | 2d12 + Intelligence modifier + `Effect::Study` (Spellcraft ÷ 10 and Decipher script ÷ 10, each to +20) + affinity + Quick study ≥ 12 + overreach + known ÷ 20 + `study_difficulty` | the same | `study_odds`, `StudyRules::difficulty` (`content/learning.rs` 120) |
| The data | `data/skills.toml` Spellcraft: `benefit = [{ effect = "study", curve = [[0, 0], [200, 20]] }]` | `benefit = [{ effect = "learn", curve = [[10, 0], [60, 10], [100, 30], [150, 70]] }, { effect = "study", curve = [[0, 0], [200, 20]] }]` | `skills.rs`: `Effect::Learn`, its ceiling `MAX_SPELL_LEVEL` (70) as `Effect::Study` has `MAX_STUDY_BONUS` (`skills.rs` 63, 267) |
| Hard spells | `study_difficulty` above 0: a check at any caster level, overreach counted as at least 0 | the same, at any learning | `study_odds` 127–134 |

**Why Spellcraft keeps its tenth in the check as well.** Spellcraft is the whole of learning: the curve says what is certain, the tenth eases what is not. Dropping the tenth would make every hard spell (the druid's calls, `study_difficulty` 8, 10, 12) and every spell above the reach harder than today for everyone, and the trained wizard who learns Call the elements at 69 % today (§2.3) would fall to 15 %. Two benefits on one skill is what Survival (forage, lookout, evade) and Stealth (quiet, slip, stalk) already are; each is read in one place.

**Why a curve, and why this shape.** Spell levels run 0 to 70; a skill runs 0 to 100 at its ordinary pace, past it at half pace and with affixes (`Skills::roll`, `skills.rs` 789–796; `MAX_VALUE` 999). A straight line (`[[10, 0], [150, 70]]`, a level per 2 points) was computed first and rejected: it hands a level-10 wizard learning 17 to 27 where caster level gives 12, and a level-30 priest 45 where it gives 30, because a skill grows by whole dice a level while caster level grew by one. The curve bends the other way: the first fifty points buy ten levels (the cantrips and the first list's spells, 0 to 12), the next forty buy twenty (the soft cap 100 reaches 30), and the fifty past the cap buy forty (150 reaches the last spell level). Read with `Benefit::at` (a line between the points, rounded down, level after the last):

| Spellcraft | 9 | 17 | 22 | 27 | 35 | 50 | 60 | 75 | 90 | 100 | 110 | 125 | 140 | 150+ |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Learning | 0 (→ 1) | 1 | 2 | 3 | 5 | 8 | 10 | 17 | 25 | 30 | 38 | 50 | 62 | 70 |

150 is the soft cap plus Skill focus (+25, #963) plus a mid-tier *of the Scholar* (+25 to +29 at item level 6; `affixes/of_the_scholar.toml`), or the cap plus the best Scholar (+45 to +50 at 12). Everything about the number is one data line; §9 records the owner's acceptance of the shape.

### 2.2 The worked numbers: what moves

The chance of learning from a scroll, today and under the design, for a low, a middle, a high, and a very high spell, at levels 1, 10, 30, 60, per class. **Today** uses the class's caster level at the level (`level × casting ÷ 100`, at least 1; the wizard + 1 per 5 levels for Arcane training) and the probe's mean Intelligence (`docs/balance.md` 126–134: wizard 14 / 17 / 21 / 24, priest 11 / 12 / 16 / 18; the others taken as modifier 0 / +1 / +2 / +3), the wizard's and witch's Spellcraft at 17 + 2.5 a level, the known levels 1 / 7 / 20 / 40 (the class tables' spells, roughly). **The design** applies to characters trained in Spellcraft; an untrained one is refused at every cell. It uses the starting value of §3 (the wizard's with Arcane training's +10), grown at two paces, *steady* and *focused* (§3.3: one roll a level at the ladder's first steps, or two rolls a level fed by casting's marks), and the same Intelligence. A cell is the chance in percent, or *certain*; the design's cell is *steady / focused*.

| Class, level | Today: caster level | Design: Spellcraft → learning, steady / focused | Magic missile (3) | Fireball (11) | Call the elements (35, difficulty 12) | A level-56 spell |
|---|---|---|---|---|---|---|
| Wizard 1 | 1 | 27 → 3 / 27 → 3 | 81 → certain / certain | 25 → 46 / 46 | 1 → 1 / 1 | 1 → 1 / 1 |
| Wizard 10 | 12 | 54 → 8 / 90 → 25 | certain → certain / certain | certain → 96 / certain | 1 → 1 / 7 | 1 → 1 / 1 |
| Wizard 30 | 36 | 108 → 36 / 167 → 70 | certain | certain | 69 → 81 / 99 | 19 → 31 / certain |
| Wizard 60 | 72 | 153 → 70 / 272 → 70 | certain | certain | 99 → 99 / 99 | certain → certain / certain |
| Priest 1 | 1 | 17 → 1 / 17 → 1 | 46 → 54 / 54 | 4 → 7 / 7 | 1 → 1 / 1 | 1 → 1 / 1 |
| Priest 10 | 10 | 35 → 5 / 71 → 15 | certain | 62 → 46 / certain | 1 → 1 / 1 | 1 → 1 / 1 |
| Priest 30 | 30 | 75 → 17 / 146 → 66 | certain | certain | 1 → 1 / 85 | 1 → 1 / certain |
| Priest 60 | 60 | 118 → 44 / 236 → 70 | certain | certain | 4 → 69 / 99 | certain → 69 / certain |
| Druid 1 | 1 | 17 → 1 / 17 → 1 | 54 → 62 / 62 | 7 → 10 / 10 | 1 → 1 / 1 | 1 → 1 / 1 |
| Druid 10 | 10 | 35 → 5 / 71 → 15 | certain | 69 → 54 / certain | 1 → 1 / 1 | 1 → 1 / 1 |
| Druid 30 | 30 | 75 → 17 / 146 → 66 | certain | certain | 1 → 1 / 85 | 1 → 1 / certain |
| Druid 60 | 60 | 118 → 44 / 236 → 70 | certain | certain | 4 → 69 / 99 | certain → 69 / certain |
| Witch 1 | 1 | 17 → 1 / 17 → 1 | 54 → 54 / 54 | 7 → 7 / 7 | 1 → 1 / 1 | 1 → 1 / 1 |
| Witch 10 | 10 | 35 → 5 / 71 → 15 | certain | 81 → 46 / certain | 1 → 1 / 1 | 1 → 1 / 1 |
| Witch 30 | 30 | 75 → 17 / 146 → 66 | certain | certain | 19 → 1 / 90 | 1 → 1 / certain |
| Witch 60 | 60 | 118 → 44 / 236 → 70 | certain | certain | 96 → 75 / 99 | certain → 62 / certain |
| Bard 1 | 1 | 9 → 1 / 9 → 1 | 46 → 46 / 46 | 4 → 4 / 4 | 1 | 1 |
| Bard 10 | 7 | 27 → 3 / 63 → 11 | certain | 38 → 25 / certain | 1 | 1 |
| Bard 30 | 22 | 67 → 13 / 144 → 65 | certain | certain | 1 → 1 / 90 | 1 → 1 / certain |
| Bard 60 | 45 | 114 → 41 / 234 → 70 | certain | certain | 7 → 75 / 99 | 4 → 38 / certain |
| Paladin, ranger 1 | 1 | 9 → 1 / 9 → 1 | 46 → 46 / 46 | 4 → 4 / 4 | 1 | 1 |
| Paladin, ranger 10 | 5 | 27 → 3 / 63 → 11 | certain | 25 → 25 / certain | 1 | 1 |
| Paladin, ranger 30 | 15 | 67 → 13 / 144 → 65 | certain | certain | 1 → 1 / 81 | 1 → 1 / certain |
| Paladin, ranger 60 | 30 | 114 → 41 / 234 → 70 | certain | certain | 1 → 62 / 99 | 1 → 38 / certain |
| Warrior (no Spellcraft) 1 / 10 / 30 / 60 | 1 / 2 / 7 / 15 | none: cannot learn at all | 46 / 62 / certain / certain → refused | 4 / 10 / 38 / certain → refused | 1 → refused | 1 → refused |
| Aristocrat 1, with Spellcraft (one in three) / without | 1 | 9 → 1 / cannot learn | 46 → 46 / refused | 4 → 4 / refused | 1 / refused | 1 / refused |
| Ascetic 1, with Spellcraft (one in five) / without | 1 | 5 → 1 / cannot learn | 38 → 38 / refused | 2 → 2 / refused | 1 / refused | 1 / refused |

(The aristocrat's and ascetic's rows are the hybrids' at level 1 (Spellcraft 9 and 5; their cells use modifier 0, *unchecked*), growing as the hybrids' do once trained. The paladin's affinities differ from the ranger's, but neither has one with evocation or enchantment, so their rows are the same.)

**Where the outcome moves, and why:**

| Move | Why |
|---|---|
| Level 1 is the same or a little kinder for everyone who has Spellcraft | learning 1 to 3 against caster level 1; the wizard's Arcane training now pays at once (§5.2) |
| The middle game (10 to 30) is **slower at the steady pace** for every full caster but the wizard, and for the hybrids: Fireball at level 10 is a struggle (46 %) for a priest where it was 62 %, certain for a wizard only at the focused pace | caster level came free with every character level; learning now costs level-up rolls, and the steady pace (one roll a level, the ladder's small dice) gives a priest 35 at level 10 and 75 at 30, learning 5 and 17 |
| The middle game is **faster at the focused pace** than today | two rolls a level fed by casting's marks (§6) put a full caster at the soft cap by level 15 to 20 and at 150 before 30: learning 66 at 30 where caster level gave 30 or 36. The price is every other skill of the class (Concentration's pool and pace, Heal, Knowledge) standing still, and the gates left are the scrolls' loot levels (`objects/README.toml`: a level-11 spell's scroll is item level 6) and the mana (a level-35 spell costs 35 a cast) |
| The high game (60) ends at 70 for anyone who kept at it | today a level-60 wizard learns level 72 and a priest 60 for certain; now learning tops at 70 (the last spell level) from Spellcraft 150, which the steady wizard reaches by 60 and a steady priest does not (118: learning 44, a level-56 spell at 69 %) |
| The hybrids can get there at all | a level-60 paladin today learns a level-30 spell for certain and nothing above 35 in practice (1 %); with Spellcraft they walk the same ladder, slower |
| A class without Spellcraft cannot learn spells at all (the owner's decision, §9) | a warrior studies nothing, cantrips included (§3.2), where caster level crept to 15 by 60 and made Fireball certain. Scrolls still work to cast, and a class's own spells still arrive |

The pace is the player's: these are the two ends, and the probe will measure where `levelup:auto` lands (§8, N3).

### 2.3 Today's values the table replaces

The check and its words are unchanged, so the wiki's `study-chances.md` (difficulty by overreach, the chance at each bonus) stays as generated; its intro loses "caster level". A new generated table, `learning.md`, states the curve: Spellcraft at the sampled values above → learning (`xtask/src/wiki/learning.rs`, one function and one line in `generate()`).

## 3. Starting points per class

### 3.1 The dice

The class `skills` entries (`data/classes/*.toml`; `docs/dev/skills.md` 31–32: rolled at creation after every other roll, so nothing else moves; the seed's stream, `character/optional-skills/<spec>`, for the optional ones). The two shapes a class uses today are 2d6+10 (its known-for skills) and 2d4+4 (its others), and 2d4 by chance (`classes/README.toml` 44–52); the design uses no new shape.

| Class | `casting` today (goes) | Spellcraft today | The design | Range (mean) | Learning at the start, min / mean / max | Why |
|---|---|---|---|---|---|---|
| Wizard | 100 | `spellcraft = "2d6+10"`, known for | unchanged; + Arcane training's +10 (§5.2) | 22–32 (27) with the talent | 2 / 3 / 4 | the learner: Arcane training and the known-for step keep it ahead |
| Witch | 100 | `"2d6+10"`, known for | unchanged | 12–22 (17) | 0 / 1 / 2 | |
| Priest | 100 | optional, 25 %: `2d4` | `spellcraft = "2d6+10"` in `skills` (the optional entry removed) | 12–22 (17) | 0 / 1 / 2 | a full caster: today's one-in-four start with 2 to 8 points would leave three priests in four unable to learn past cantrips for certain |
| Druid | 100 | none | `spellcraft = "2d6+10"` | 12–22 (17) | 0 / 1 / 2 | a full caster whose own spells reach 35 |
| Bard | 75 | none | `spellcraft = "2d4+4"` | 6–12 (9) | 0 (→ 1) | a hybrid: Mend at 1 is the class's; scrolls are a check until the rolls come |
| Paladin | 50 | none | `spellcraft = "2d4+4"` | 6–12 (9) | 0 (→ 1) | a hybrid with Blessing at 6 |
| Ranger | 50 | none | `spellcraft = "2d4+4"` | 6–12 (9) | 0 (→ 1) | a hybrid with Barkskin at 6 |
| Aristocrat | 75 | none | `optional_skills`: `spellcraft = { chance = 35, dice = "2d4+4" }` | 6–12 (9), one in three | 0 (→ 1) when trained; cannot learn otherwise (two in three) | mana 7 and no spell of its own: a tutored noble may have read of magic; today's 75 % share bought nothing a scroll could not |
| Ascetic | 50 | none | `optional_skills`: `spellcraft = { chance = 20, dice = "2d4" }` | 2–8 (5), one in five | 0 (→ 1) when trained; cannot learn otherwise (four in five) | mana 4, no spell of its own |
| Berserker, thief, warrior, commoner, craftsperson, merchant | 25 | none | none | — | cannot learn | mana 1 or 2; the 25 % share gave a level-60 warrior learning 15 for spells he cannot pay for; now they cannot study (§3.2) |

`known_for` stays as it is (the wizard's and the witch's name Spellcraft; the priest's and druid's lists are empty today, so their rolls on it get the class's +1 and not the step: `skill_roll.rs` 126–137). Not proposed: a new dice shape between 2d4+4 and 2d6+10 for the bard, or `known_for` for the priest and druid; both are one data word if the owner wants the middle classes faster.

**Races and birth signs give none** (`data/races.toml`: the twelve races' `skills` are Listen, Spot, Bluff, Metalcraft, Search, Stealth, Appraise, Intimidate, Detect traps, Disable device, Survival, Swim; `data/horoscope.toml` grants talents only, none of them a skill), and no racial or sign talent gives caster level (`grep caster_level assets/data/talents/`: Arcane training, Attuned, Overreach, the README). Two background lines train it (`data/background/04-youth.toml` 74: a witch's or druid's cunning-woman teacher, Spellcraft 1d4; 235: a winter in a dead wizard's library, 2d4 and Intelligence +1, for the wizard, witch, priest, and bard), through `Skills::add` after the class roll (docs/dev/background.md), so they add to a trained skill or start it. Nothing else starts a skill once Apprentice goes (#963): a class without Spellcraft has none for the run, which §2.2's last row accepts.

### 3.2 What the rule costs those without it

A warrior, thief, berserker, commoner, craftsperson, or merchant (and two aristocrats in three, four ascetics in five) **cannot learn a spell from a scroll at all**, cantrips included: the study is refused, with no time spent, in words that say they lack the training. The owner decided this on approving the design (§9: "No spell craft skill, no learning spells. That is fine."). They lose what caster level gave them for nothing (7 at level 30 and 15 at 60); they keep reading a scroll to cast it from the scroll (that is not learning), every spell their class grants at a level (§4), and every ability and potion. They can still gain Spellcraft through the two background lines of §3.1 (a cunning-woman teacher's 1d4, and the winter in a dead wizard's library, 2d4 and Intelligence +1: each starts the skill for a character of the classes it names who rolls it) and through the bard's Jack of all trades (for a bard, who has it by class, it trains another skill; for others it is a way to a skill they do not have, as the talent's file states, *unchecked*). **A quest that teaches Spellcraft as its reward is a deferred idea, on #196**; nothing here builds it. The owner's rule that no roll locks anything out holds for those with the skill: a failed study costs the scroll and a mishap, and the next scroll is a new roll. If the owner wants a door sooner, the cheapest is a `spellcraft` optional entry at a low chance on the 25 % classes, one line each, and #973's report may propose a general way to take up a skill.

### 3.3 How Spellcraft grows

Growth is level-up rolls only (docs/dev/skills.md 15–28; `world/skill_roll.rs`): each level gives `rolls` (class 2, the bard's 4, the ranger's and thief's 3; + the human's 1; + 1 at Intelligence 14, 2 at 18, 3 at 22: `[rolls]`, `data/skills.toml` 99–103), each spent on one trained skill (`advance:SKILL`), adding the ladder's die at the step the skill's marks reach, a step up for a known-for skill, +1 for one of the class's other skills, halved from 100 (`Skills::roll`), spending half the marks (`spend = 50`).

| Marks | 0 | 5 | 12 | 25 | 50 | 100 | 200 | 400 | 800 |
|---|---|---|---|---|---|---|---|---|---|
| Die (`[ladder]`, `skills.toml` 90–92) | 1 | 1d3 | 1d5 | 2d4 | 3d3 | 3d4 | 3d5 | 4d4 | 4d5 |
| Mean | 1 | 2 | 3 | 5 | 6 | 7.5 | 9 | 10 | 12 |

Marks come from use (docs/dev/skills.md 6–12; `world/train.rs`): Spellcraft's today is `study` at 100 % (`skills.toml` 367), one mark a scroll studied. A caster studies perhaps a scroll a level, and each roll spends half the marks, so the step hovers at 0 or 1: the die is 1 or 1d3 (the wizard and witch, known for it, 1d3 or 1d5). That is the **steady pace** of §2.2: +2 a level (the wizard +3), one roll a level. With a `cast` mark (§6) the marks pile up with every spell paid in mana (the probe's wizard casts a pool a day, ten casts or so: two or three marks a day at 25 %), the step climbs to 3 or 4 (2d4, 3d3), and a player who spends two rolls a level on it gets the **focused pace**, +6 a level (the wizard +7). The halving from 100 slows both.

| Class (start) | Steady: Spellcraft → learning at 10 / 30 / 60 | Focused: at 10 / 30 / 60 | Caster level today at 10 / 30 / 60 |
|---|---|---|---|
| Wizard (27 with Arcane training) | 54 → 8, 108 → 36, 153 → 70 | 90 → 25, 167 → 70, 272 → 70 | 12, 36, 72 |
| Witch, priest, druid (17) | 35 → 5, 75 → 17, 118 → 44 | 71 → 15, 146 → 66, 236 → 70 | 10, 30, 60 |
| Bard (9) | 27 → 3, 67 → 13, 114 → 41 | 63 → 11, 144 → 65, 234 → 70 | 7, 22, 45 |
| Paladin, ranger (9) | 27 → 3, 67 → 13, 114 → 41 | 63 → 11, 144 → 65, 234 → 70 | 5, 15, 30 |

**Each class keeps up with what its table and kit expect.** Every spell a class gives comes by its table (§4) and needs no learning. The kits' scrolls (`kit` in `data/classes/`): the wizard's scroll of light (level 0: certain), scroll of identify (6: a check, 75 % at learning 3 with Arcane training's ten against today's 54 % at caster level 1), and one of fireball, frost, or lightning (11, 6, 11: 46 %, 81 %, 46 % against 25 %, 62 %, 25 % today; the probe script again, a level-1 wizard of Intelligence 14 knowing Ray of frost); the priest's scroll of light. Nothing a kit holds is harder to learn than today. What a class's own spells ask is mana, which is untouched. The learning the table above gives is for scrolls found, and a full caster at the steady pace learns the first list's spells (levels 0 to 12) for certain by level 20 to 25 (Spellcraft 55 to 65 → learning 9 to 11; Flame, 12, a little later), the hybrids by 30; at the focused pace a full caster has them by level 10 to 12.

## 4. The spells a class gives at a level

| | Today | The design |
|---|---|---|
| How a class's spell is known | a class ability in the class table, a talent file with `spell { spell }` (`TalentMods::has_spell`; `Actor::knows_spell`, `learn.rs` 83–85): held at the table's level, no study, no check | the same: **no caster level is in the rule today**, only in a test |
| The tie | `spell_list_tests.rs` 218–241 `every_class_has_mana_and_its_guaranteed_spells_within_its_caster_level`: a given spell's level ≤ `level × casting ÷ 100` (at least 1) at the level it comes; the spell scaling design §5.6; `spec/08e` 325; `wiki/src/magic.md` 13 ("A spell a class gives never comes above your caster level") | the clause goes with `casting`. A table's level is the author's call, as decision 16 leaves a spell's strength the author's. The test keeps its other clauses: a full caster's first spell is a cantrip and it has three to six, a hybrid at most two boons of level 5 or less, every class has mana (`spell_list_tests.rs` 243–262) |
| What the player is told | nothing | nothing; the wiki's line becomes "A spell your class gives is yours at that level, whatever your Spellcraft" |
| The tables' spells by level (for the record) | wizard: Ray of frost (1) at 1, Magic missile (3) at 3, Burning hands (4) at 12, Arcane bolt (6) at 18; priest: Mend (1) at 1, Blessing and Cure wounds (3) at 3; druid: Spark (1) at 1, Barkskin (3) at 3, Call of the wild (5) at 6, Call lightning (11) at 12, Call of the forest (20) at 25, Call the elements (35) at 40; witch: Evil eye (0) at 1, Hex of weakness (3) at 3, Cause wounds (4) at 12, Curse of sloth (3) at 32, Blight (7) at 40; bard: Mend at 1; paladin: Blessing at 6; ranger: Barkskin at 6 (`data/classes/`; the levels from `data/spells/`) | unchanged |

Not proposed: a check or a Spellcraft need on a class's own spell. The class teaches it; that is what the table is.

## 5. Inventory: every reader of caster level

`grep -ril 'caster.level\|caster_level' crates xtask assets spec docs wiki/src` lists 106 files; the design records under `docs/design/` (14 of them) are untouched, and the 28 object files and 3 data READMEs name it only in comments. The rest, grouped, each with its verdict.

### 5.1 The item affix

| | of the Adept (`assets/data/affixes/of_the_adept.toml`) |
|---|---|
| Today | suffix, `effect = "caster_level"`, `category = "caster_level"`, bases weapon, helmet, ring, amulet, weight 50, tiers +1 at item level 2, +2 at 7, +3 at 12; `Effect::CasterLevel` (`content/affix.rs` 209–210, 300, 363, 430), read in `Actor::caster_level` (`caster_level.rs` 30), described "+N to caster level" (`magic/describe.rs` 383), the wiki's "caster level" (`xtask/src/wiki/magic.rs` 199); the one affix of its category |
| Verdict | **deleted**, with `Effect::CasterLevel` and the `caster_level` category. **Of the Scholar** (`of_the_scholar.toml`: `effect = "skill"`, `skill = "spellcraft"`, helmet, ring, amulet, weight 60, +8–12 at level 1 to +45–50 at 12) is already the affix that raises learning; a second affix on the same skill is a duplicate. Learning it gives, to a character trained in Spellcraft (worn by an untrained one it adds to a skill they do not have and opens nothing): +8 at the start (Spellcraft 17 → 25–29: learning 1 → 3), +45–50 at the top (108 → 153+: 36 → 70) |
| Weapons | the Adept was the one way a staff lent learning. Proposed with it, one line: `"weapon"` added to of the Scholar's `bases`, so a staff of the Scholar is possible; strike it to keep learning off weapons |
| Loot tables | an affix is drawn by `weight` among those whose `bases` hold the item's base and whose tier `level` fits (`magic/loot.rs` 10–20, `pick_weighted`); no object, kit, chest, or shop names an affix. Deleting it re-weights the suffix draw on weapons, helmets, rings, and amulets among the rest (its 50 shared out); nothing else changes. Every seed's affix draws on those bases shift, so a replay with magic loot may hash differently (saves are pinned to the version; the repository holds no replay files; the golden manifests build theirs from action text, and a golden showing a magic item's name would need the finisher's usual pass) |
| The unique | the Eorcanstān (`assets/campaigns/ascmearc/data/uniques/eorcanstan.toml` 10: `"of_the_adept:3"` among six affixes; the comment "at a higher caster level") → `"of_the_scholar:6"` (+25–29 Spellcraft: learning +5 to +13 by where the wearer is on the curve; the Adept's +3, +5 with Attuned, in the same band), the comment "learning deeper" |
| Tests | `magic/tests.rs` 726–729 (the map from each affix effect to the file that reads it: the entry goes), `world/creature_magic_tests.rs` 11–20 and 61–90 (a test affix of `caster_level` on a creature: the caster-level third of the test goes, the mana and pace thirds stay), `world/affix_talent_tests.rs` (Attuned's: deleted whole, §5.2), the UI snapshot `item-description-spell-affix.snap` 25 ("Suffix of the Adept: +2 to caster level") re-pointed to of the Scholar's description |
| Docs | `spec/08g-magic-items.md` 93–94, 128; `docs/dev/magic-items.md` 54; `wiki/src/magic-items.md` 141; `docs/design/affixes-systems.md` (a record, untouched) |

### 5.2 The talents

| Talent | Today | Verdict | Coverage |
|---|---|---|---|
| **Arcane training** (`talents/arcane_training.toml`; the wizard's at 1, `09-wizard.toml` 37) | `caster_level { value = 1, per_levels = 5 }`: +1 caster level per 5 character levels (`TalentMods::caster_level_at`, `talent.rs` 1123–1130; +0 at 1, +2 at 10, +6 at 30, +12 at 60) | **replaced**: `effects = [{ kind = "skill", skill = "spellcraft", value = 10 }]`, the existing kind ("that skill counts that much higher", `talents/README.toml` 238; `TalentEffect::Skill`, read in `Actor::skill_effect_above`, `world/actor.rs` 473–490, so it counts in the curve and the tenth alike). Worth, for the wizard, who is always trained: learning +2 at the start (17 → 27: 1 → 3), +4 at 54, +5 at 75, +8 at 108, 0 past 150. Description: "Your learning outruns your training: you read spells as one ten points deeper in Spellcraft." The alternative is to drop it and raise the wizard's dice by the same ten; §9 records the owner's decision to keep it | stays a `spells` class ability; not alike to Skill focus (#963: `skill { value = 25 }` on a chosen trained skill, a pick) by the clone rule, which reads the fields (`talent_alike.rs`; Weapon focus and Deadly aim coexist the same way) |
| **Attuned** (`talents/attuned.toml`; Scholar, Intelligence 12, level 6) | `attuned_percent { value = 50 }`: the caster level magic items give counts half again, rounded up (`TalentMods::attuned`, `talent.rs` 1132; `Actor::caster_level` 30) | **deleted**, with the `attuned_percent` kind (`content/talent.rs` 292–293 and its check, `talent.rs` 251, `xtask/src/wiki/talents.rs` 679, README). No item gives caster level any more, so it has no effect; re-pointing it at the skill affixes ("your items' skill bonuses count half again") would be a new kind for a niche of item builds, which the coverage policy forbids. The `magic_items` system keeps seven talents (Hoard sense, Father's sword, Old lore, Dragon hoard, Spirit weapon, Merchant's eye, Treasure hunter), `spells` keeps the Spellcraft tree and the casters', so the bar holds | the Scholar tree loses one pick (11 → 10 after #969's count); no talent `needs` it (`grep attuned assets/data/talents/`: itself) |
| **Overreach** (`talents/overreach.toml`; Spellcraft tree, level 6) | `study { overreach_percent = -33 }`: the levels beyond the caster level count a third fewer | **unchanged**: the overreach is now beyond the learning; its description already says "a spell beyond your learning is a third less beyond you". The file comment, `spec/08i` 980, the wiki words (`talents.rs` 928) say "learning" | |
| **Quick study**, **School focus**, Keen mind, Sure hand, Numbing cold, Brimming, Scroll adept, Empower, Archmage, Magister, Schooled, Devotee, Wand mastery, Channeler | read no caster level (`docs/dev/spells.md` 526–542; the hooks are `SpellMods`) | unchanged | |
| The `caster_level` effect kind | `TalentEffect::CasterLevel { value, per_levels }` (`content/talent.rs` 284–290, 2208; `talent.rs` 242–249, 1377; `xtask/src/wiki/talents.rs` 671–676; README 454–455; `talents.toml` 200–205 "Spells and caster level") | **removed**; the system is named "Spells and learning" | |

### 5.3 Creatures

| | Today | Verdict |
|---|---|---|
| What a creature reads | **nothing in play.** `World::caster_level(id)` (`caster_level.rs` 44–56) gives a creature its type's level and 100 %, plus what it wears; its only caller for a creature is the dev `actors` dump's `cl N` (`dump.rs` 1698–1702). Creatures cast as intents (`combat = "caster"` on the type, `ai.rs`; `kobold_shaman.toml` 17), never by spell id, and never study (spec/08b 345; `docs/dev/mana.md` 98: "its caster level is read by nothing in play yet") | `cl N` goes from the `actors` layer; `World::caster_level` goes with it (§5.4). No creature loses a thing it could do |
| Creature Spellcraft | 42 creature types carry `skills = { concentration = N, spellcraft = N + 5 }` (`data/creatures/`: kobold shaman 25, lich 105, demilich 225; the Ascmearc's cult priest 45, bog hag 60), read by nothing: `Effect::Study` and the new `Effect::Learn` are read in `study_odds` for the player alone, and a creature's `Skills::effect` is read only by the encounter lookouts (`docs/dev/skills.md` 154–157) | **stay as data** (the wiki's creature tables show them); nothing new reads them. No check "a creature's Spellcraft reaches its spells" is possible: creature spells have no levels. A row for #973 (a skill value nothing reads) |
| Summoners, animated casters, the drow and duergar priests | the same | the same; `docs/dev/animated.md` 161, `wiki/src/animated.md` 85 and `mana.md` 14 lose "and caster level" |

### 5.4 Everything else

| Reader | Where | What happens |
|---|---|---|
| The derived number | `world/caster_level.rs` (`casting_percent`, `Actor::caster_level`, `World::caster_level`), `world/mod.rs` 106, 257, 1558 | **deleted** |
| The class share | `ClassDef::casting`, `CASTING`, `least_casting` (`content/class.rs` 42–46, 78–79, 277–279, 401–405, 493); `casting = N` in the 15 class files; `classes/README.toml` 28–29; `spec/07e` 152 | **removed**; its intent is the starting dice (§3.1). The wiki's `spell-casting.md` table (`xtask/src/wiki/spells.rs` 68–77, `wiki/mod.rs` 323, the assertion at 1448 "Wizard 100 %") goes; `wiki/src/magic.md` 69 drops the include |
| The study | `learn.rs` 60–69 (`Odds` docs), 116–147 (`study_odds`), 157–163 (`player_study_odds`); `action.rs` 158 | `study_odds(actor, content, character, spell)`: the learning read inside (§2.1), none for a character not trained in Spellcraft; `player_study_odds` and the study's start refuse on it (the words, no time); docs say "learning" |
| Perception | `PerceptionView::own_caster_level` (`perception.rs` 2010–2015), `studies()` 1869 | the method goes; `studies` reads through `study_odds` as before |
| Dumps | `dump.rs` 58, 848, 874 (`Caster level N` in the `stats` layer), 1698–1702 (`cl N`), 2112 (JSON `caster_level`), 2435 (a test's expected line); `spec/09` 113, 181; CLAUDE.md's dump notes (`Caster level 3`, `cl 8`, JSON `caster_level`) | the lines go; the dev `progress` layer gains `learning 36 (Spellcraft 108)` beside its `study c frost: overreach 5, …`; JSON gains nothing (Spellcraft is in `skills`) |
| The Magic page, the character screen | `ui/magic_screen.rs` (schools, caster styles, spells), `ui/character_screen.rs` | **show no caster level today** (`docs/dev/spells.md` 336–337); nothing changes. Spellcraft is on the Skills screen as every skill |
| The study list and dialog | `ui/object_screen.rs` `ObjectList::Study`; the words "certain / easy / a struggle / far beyond you" | unchanged for the trained (the words come from `Odds::ways`); for the untrained the study command says why it is refused (the refusal's words; a text snapshot) |
| The refusal and its dumps | the study's start (`learn.rs` 165–213); `dump.rs` 1219–1262 (the `progress` layer's `study c …` lines, the `spells` layer's scrolls "they could study") and 1298 (JSON `spells`' studyable scrolls); `ability.rs` 88 (the scrolls one could study); `PerceptionView::studies` | `studies()` lists nothing for an untrained character, so the `spells` layer and JSON have no study lines for them; the `progress` layer prints `cannot learn: not trained in Spellcraft` where `learning N (Spellcraft N)` would be; the `abilities` layer says the same wherever it lists a study (the reason's id `untrained`); a replay's `study:c` is refused with no time. N1 |
| The balance probe | `balance_probe.rs` 571 (a comment), 662 and 975 (`caster_level` per level); `balance_probe/tables.rs` 56 and 90 (the "Caster level Wz/P" column); `docs/balance.md` 124–134 | the column becomes "Spellcraft Wz/P" with the learning in brackets, as the probe's auto-rolls leave it (`advance_auto` spends every roll on the first trained skill, Concentration for both: the column will show the start plus the known-for and class steps, and the table says so); the rerun is N3's |
| Spells data and docs | `data/spells.toml` 6, 14; `spells/README.toml` 12, 41; `content/spell_def.rs` 8, 331, 566, 599; `study.toml` 1–17 | "the caster level at which a scroll of it teaches for certain" → "the learning at which" |
| Rods and wands | `content/rod.rs` 5, 180, 312–314, 457–458 (`up_to_caster_level` refused naming `up_to_level`); `content/wand.rs` 33, 61–63 (`caster_level` refused naming `rungs`); `world/rod.rs` 12; `content/object.rs` 442; `ui/target.rs` 178; `rod_effects/README.toml` 16–17; `rod_of_digging/sleep/terror.toml` comments; `spec/08g` 357, 411–412; `wiki/src/magic.md` 270 | the refusals of the two retired fields stay (old content is refused as before); the comments and docs lose the phrase. `rod_of_sleep.toml` 2 ("no stronger than your caster level") is stale already (rods read `up_to_level`) and is fixed |
| Scroll objects | the 25 `scroll_of_*.toml` comments ("at its own caster level": `scroll_of_light.toml` 2) | comments: "at its own rungs" |
| `cast.rs` 6, 126; `mana.md` 51, 84; `spells.toml` 14 | "no caster level in the cost" | the reassurance goes with the thing |
| Tests | `world/caster_level_tests.rs` (152 lines: by class and level, no mana, talents and a ring, a creature, the dumps; `mana_grows_with_every_level_to_60` moves to the mana tests); `affix_talent_tests.rs` (deleted, §5.2); `creature_magic_tests.rs` (§5.1); `learn_tests.rs` 115, 220 (the cases by Spellcraft value: a priest at 17, a wizard at 108, the floor of 1, a hard spell at any learning); `spell_talent_tests.rs` 30, 93–110 (Arcane training's +10 in the curve and the tenth); `spell_list_tests.rs` 218–241 (§4); `summon_spell_tests.rs` 308 (renamed "at any learning"); `rod_tests.rs` 89, 161–166, 198 (assertions that caster level is 1 or unread: dropped); `zap_tests.rs` 452–461 (the refused field: stays) | as listed |
| Spec | `spec/02-scope.md` 86; `07e` 152; `08b` 345; `08e` 275–277, 293–294, 325, 351, 423, 467–476 (the "Caster level" entry), 498–500; `08g` 93–94, 128, 357, 411–412; `08i` 980, 1283; `09` 113, 181 | "Learning" replaces the "Caster level" entry: the curve, the floor, the one read |
| `docs/dev/` | `spells.md` (the bullets at 19–34, 49–53, 306–313, 320–337, 376–388, 409–411, 609–629), `magic-skills.md` (none), `skills.md` 91–93 (the `learn` read beside `study`), `mana.md` 51, 84, 90–98, 125, `magic-items.md` 54, `animated.md` 161, `progression.md` 165–166, 327, `character-creation.md` (the classes' Spellcraft), CLAUDE.md | as the code |
| Wiki | `wiki/src/magic.md` 6, 13, 51, 56–67 (the "Caster level" section → "Learning": the curve's table, the floor, Arcane training, of the Scholar, Skill focus), 69, 75, 199, 270; `features.md` 81; `magic-items.md` 141; `mana.md` 14; `animated.md` 85; `classes.md` (the casting share, *unchecked* whether prose names it); `xtask/src/wiki/learning.rs` (the new `learning.md`), `magic.rs` 199, `talents.rs` 671–679, 928, `weapon_abilities.rs` 204 (a comment) | as the code; `cargo xtask wiki --check-links` for the `#caster-level` anchor's five referrers |

## 6. Training Spellcraft

| | Today | The design |
|---|---|---|
| Marks | `mark = [{ on = "study", chance = 100 }]` (`skills.toml` 367): one a scroll studied (`World::study`, `Use::Study`, also `StatUse::Study`) | `mark = [{ on = "study", chance = 100 }, { on = "cast", chance = 25 }]`: a spell paid in mana is practice, as Concentration's `cast` mark at 25 % already is (`skills.toml` 160; reported by `World::spend_mana`, docs/dev/skills.md 99–101). One data line; the `Use::Cast` hook exists |
| Why | §3.3: with study alone the die stays at 1 or 1d3, a point or two a roll, and a priest who spends a roll a level on it reaches 75 by level 30 (learning 17). A skill the whole of learning hangs on cannot train only by the thing it gates | with casting's marks an active caster's step climbs to 2d4 or 3d3 and a roll is worth five or six |
| Farming | casting cantrips for marks is allowed (the owner: no anti-farming limits, time is the penalty; Concentration is farmed the same way today); the marks only raise the die a level-up roll throws, and each roll spends half of them | |
| Not proposed | a mark for reading a scroll (`Use` for `read` does not exist; a read teaches nothing), for scribing (the wizard's, a stat use only), for identifying by spell; a second use of the skill (a Spellcraft check anywhere else): #973 is the place, and this design's row for it is "effect: learning and the study tenth; marks: study and cast" | |

## 7. Interactions

| With | What it does here |
|---|---|
| **Skill focus** (#963, implementing `docs/design/talent-ranks.md` §2: Scholar, no needs, `skill { value = 25 }` on a chosen trained skill, past 100 too) | taken for Spellcraft (offered for a trained skill only, so it never opens learning for an untrained character) it counts in the curve and the tenth (`Actor::skill_effect_above` sums the talents' `skills`): learning +5 at Spellcraft 27 (3 → 8), +11 at 56 (9 → 20), +13 at 75 (17 → 30), +20 at 108 (36 → 56), nothing past 150. A wizard's pick at creation (it is offered, having no needs) starts at 52: learning 8 for a level-1 character, Fireball (11) at 81 %. Fine under "unbalanced choices are fine"; it is one of the wizard's two creation picks and does nothing else. #963 is not waited for: this design reads the `skill` kind as it exists today (`TalentEffect::Skill`, `README.toml` 238), and Arcane training uses it with `skill` named, which needs nothing from #963's "`skill` optional" change |
| **The skills investigation** (#973: a report on every skill's effect, training, holders, and verdict) | Spellcraft's row reads this document: effect, learning (the curve) and the study tenth; training, study and (N4) cast; holders, the nine classes of §3.1 and two background lines, with every other class unable to learn spells; verdict after N4, fine. Its "no way to take up a skill" finding (sharper now: Spellcraft is the one gate to learning), if it makes one, is where §3.2's door for the 25 % classes belongs. Nothing here waits for it: N4 lands as proposed unless #973's merged report says otherwise first |
| **One rank per talent** (#969, in flight) | no talent here has ranks; Attuned's deletion and Arcane training's new effect are single-rank files in either file shape. The talent-rank machinery (#966) is not used |
| **Of the Scholar's loot** | unchanged; it is now the one affix of learning and worth more than its tier said (a level-12 ring is +20 learning at the top of the curve). Its weight (60) is not retuned here; the balance rerun in N3 says what the probe's "magic" gear draws |
| **The druid's calls** (`study_difficulty` 8, 10, 12; a check at any learning) | unchanged in kind; §2.2 shows Call the elements: a steady level-60 priest 69 %, a focused one 99 %, a steady level-30 priest 1 % as today |

## 8. Implementation issues

Rules first, then the content that uses them (CLAUDE.md "Keep PRs small"); each PR runs the full check list, replays, and `cargo xtask wiki --check-links`. N1 before N2 before N3; N4 any time after N1.

| # | PR | Contents | Depends on |
|---|---|---|---|
| N1 | **Learning is Spellcraft** | `Effect::Learn` (`skills.rs`: the variant, its ceiling 70, its id `learn`, its words); Spellcraft's `learn` curve in `data/skills.toml`; `study_odds` reading the learning (the floor of 1 for the trained) and dropping its `caster_level` argument, `player_study_odds` and `PerceptionView::studies` with it; **the refusal** (a study by a character not trained in Spellcraft is refused with no time spent, in plain words; `studies()` lists nothing for them; the `abilities` and `spells` dump lines, §5.4); the `progress` layer's `learning N (Spellcraft N)`; the nine classes' Spellcraft dice (§3.1: priest, druid, bard, paladin, ranger in `skills`; aristocrat and ascetic in `optional_skills`; the priest's optional entry removed); `learn_tests.rs` by Spellcraft value, with the refusal (a warrior; an untrained character wearing of the Scholar; the trained floor of 1; a scroll still read to cast; a class's granted spell still arriving) and a text snapshot of the words, `summon_spell_tests.rs` 308; `study.toml`'s comments, `spells/README.toml` 12 and 41; `spec/08e` "Learning", `docs/dev/spells.md` "Learning" and `skills.md`'s one line, `character-creation.md`; the wiki's "Learning" section and `learning.md` (`xtask/src/wiki/learning.rs`), `study-chances.md`'s intro. After it `World::caster_level` exists and nothing but the dumps and the probe read it | nothing |
| N2 | **The talents, the affix, the unique** | `arcane_training.toml` → `skill { skill = "spellcraft", value = 10 }` (§5.2; `spell_talent_tests.rs` 93–110 rewritten); `attuned.toml` deleted with `attuned_percent` (`content/talent.rs`, `talent.rs`, README, `wiki/talents.rs` 679) and `affix_talent_tests.rs`; `of_the_adept.toml` deleted with `Effect::CasterLevel` (`content/affix.rs`, `describe.rs` 383, `wiki/magic.rs` 199, `magic/tests.rs` 726), `creature_magic_tests.rs` trimmed, the `item-description-spell-affix` snapshot re-pointed; `"weapon"` on of the Scholar's bases (§5.1, or struck); the Eorcanstān `of_the_scholar:6`; `spec/08g` 93–94, 128, `08i` 980, 1283, `docs/dev/magic-items.md` 54, `progression.md` 165–166, `wiki/src/magic-items.md` 141, `talents.md`, `cargo xtask wiki`. After it `Actor::caster_level` is `level × casting ÷ 100` alone | N1 (so Arcane training's new effect is read by a rule) |
| N3 | **Caster level removed** | `world/caster_level.rs` and `caster_level_tests.rs` deleted (`mana_grows_with_every_level_to_60` moved); `ClassDef::casting`, `CASTING`, the 15 `casting` lines, `classes/README.toml` 28–29; `own_caster_level`, the `stats` line, JSON `caster_level`, `cl N`, `dump.rs` 2435; `spell_list_tests.rs` 218–241 (§4); `rod_tests.rs` 89, 161–166, 198; the probe's column (§5.4) and the rerun with `docs/balance.md` updated; `talents.toml` 200–205 and `caster_level` kind removed (`content/talent.rs` 284–290, 2208, 3830, `talent.rs` 242–249, 1123–1130, 1377, `wiki/talents.rs` 671–676, README 454–455); the `spell-casting.md` table and `wiki/mod.rs` 1448; every remaining comment and doc line of §5.4 (spec, `docs/dev/`, CLAUDE.md, the wiki's five `#caster-level` referrers, the object comments); `cargo xtask wiki --check-links` | N1, N2 |
| N4 | **Spellcraft trains by casting** | `{ on = "cast", chance = 25 }` on Spellcraft; a test in `world/skills_tests.rs` (a cast paid in mana marks it, a scroll read does not); `docs/dev/skills.md`, the wiki's skills table (generated) | N1; after #973's report if that merges first, else as is |

N1 is what the owner sees; N2 and N3 are the removals; N4 is one line. The balance probe runs once, in N3.

## 9. Decisions

Approved by the owner on draft PR #981, with the comment: *"Go with mostly the recommendations. No spell craft skill, no learning spells. That is fine. Add to deferred ideas a quest to teach spellcraft skill as reward."* There are no open questions.

1. **The curve** `[[10, 0], [60, 10], [100, 30], [150, 70]]`: accepted as recommended (§2.1).
2. **Arcane training** as Spellcraft +10: accepted as recommended (§5.2).
3. **Marks on casting now**, at Concentration's 25 % (N4): accepted as recommended (§6).
4. **Spellcraft is required to learn.** A character not trained in it cannot study a scroll to learn a spell, cantrips included; the floor of 1 applies to the trained only (§2.1, §3.2). Reading a scroll to cast it and the spells a class grants are unaffected.
5. **A quest that teaches Spellcraft as a reward** is deferred, on #196; not built here.
