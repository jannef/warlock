# Warlock — Stat training: a signed pile of marks, a price per point, growth and loss by deed

Design document for the owner's review (2026-10-29). Written against `origin/main` at 65d99485 (Background B4, #717). Nothing here is implemented; §13 cuts it into issues. It rebuilds what is there: the training marks and their two checks (`docs/dev/stats.md` "Training", `content/stat_training.rs`, `world/stat_training.rs`, `data/stats.toml`; #231, #562), the potions' and elixirs' marks (`docs/dev/potions.md`), the corpses' stat raises (`docs/dev/hunger.md`, `OnEat::stat`), Hard-won (`docs/dev/background.md`), stat damage and drains (`docs/dev/stats.md` "Drain", `docs/dev/status-effects.md`), hunger and burden (`docs/dev/hunger.md`, `docs/dev/burden.md`), the skills' mark hooks (`docs/dev/skills.md`), and the balance probe (`docs/balance.md`). Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-10-29, verbatim): *"stat training and stat marks need overhauling. Stat training needs to be plugged into systems we have currently implemented (like low chance to gain some mark per action, idk). We need training paths for each stat. Gaining marks should not be too difficult, but not completely free either. While at it, let's implement a way to lose marks when doing something that should theoretically be punishing. Marks going enough to negative could just reduce a stat (and reset marks to 0). We could attach stat training & potential increases (% chance) to eating certain types of corpses (perhaps not directly ability points)."*

**Addenda** (the same day, verbatim): *"stat ups should not actually consume all marks. that is pretty feelbad if player stacks a lot of marks, and then all are gone just for 1 stat point"*; *"need better maths, design that too"*; *"the agent can reconsider if the formula is the best, and make it work with the mark consumption. Make sure it makes sense with both - and + marks"*.

## Contents

1. Summary
2. The owner's request, and what exists today
3. The audit: what today's rules do with today's numbers
4. The maths: one signed scale, a price per point, odds that pay the price
5. Training paths: every stat, every system
6. Pace: what a career yields
7. Losing marks and losing points
8. Corpses: marks and potential, not points
9. Talents and class abilities
10. What the player sees; the wiki, the spec, the docs
11. Determinism, saves, replays
12. Choices made along the way, and what is rejected
13. Implementation plan
14. Open questions, each with a recommended default

## 1. Summary

- **One signed pile per stat** (`Actor::stat_marks: PerStat<i32>`, today a `u16`). Good deeds add marks, bad deeds take them; zero is neutral. Positive marks make a **rise** likelier, negative marks a **drop**. Saves change (the integrator's per-release bump; `FORMAT_VERSION` untouched by this work).
- **A point has a price**, `P = (20 + 10 × score) × ramp`, where the ramp is 1 up to the potential and grows past it (×1.5, ×2.5, ×4, ×6, … for the first, second, third, fourth point beyond). A drop has a price too, half the base price of the point lost, `L = (20 + 10 × (score − 1)) / 2`, never ramped: points past your nature are the first to go and the hardest to win back.
- **Odds that pay the price**: at every check (06:00 and each level, the same formula, no daily factor), a stat with `m` marks rises with chance `m³ / (m³ + P³)` and the rise **costs `P` marks, never the whole pile**: a pile of 400 marks against a price of 160 buys a point and keeps 240 toward the next (§4.3). A drop is the mirror: `|m|³ / (|m|³ + L³)` and `L` marks paid back toward zero. Nothing moves while the pile is under a quarter of its price, either way: a single bad day never costs a point, a lucky tenth never buys one. All integers (§4.6).
- **Why the new maths**: today's `marks / (marks + need)` is insensitive to effort and resets the pile. With the kill curve as it stands (146 kills a level at 10, 365 at 60) a fighter earns 150 to 900 Strength marks a level, not the 8 to 15 a day the design assumed; today's rule turns that into a point nearly every day until the cap, while a stat trickling 5 marks a level still rises every two weeks. A simulation of the rules on the curve (§3.3, §6) puts a level-60 warrior at **+101 points in all today** (Strength and Dexterity at 40); the proposal puts it at **+38**, two thirds of them in the stats the warrior lives by.
- **Gaining marks is neither hard nor free**: more uses (every stat has at least five paths across the systems the game has, §5), and a **daily band** per stat: the first 15 marks a day in a stat come at the use's full chance, the next 15 at half, the next at a quarter, all beyond at an eighth. A hundred kicks are still a hundred chances, and a day of a thousand blows is worth more than a day of a hundred, but not ten times more; nothing is ever locked (the owner's no-anti-farming rule; time is the only price).
- **Losing**: deeds the game already detects take marks (§7): dawn spent Weak or worse, hit points starved away, vomiting, disease checks lost, a drain that lands, a known trap stepped on, acting Overloaded, being caught stealing, a shop debt turned Unwelcome, fainting, and a stat left wholly idle for a week. Far enough below zero the stat **drops by one** (never below 3, the drain floor; the potential stays; not a drained point, so a temple does not mend it) and the pile pays the drop's price back toward zero. The player is warned in words when a stat enters the zone where a drop is possible ("You feel your strength wasting.").
- **Corpses give marks, not points** (§8): `on_eat.train = { strength = 40 }` is 40 % of the next point's price in sure marks; `on_eat.widen = { strength = 10 }` a 10 % chance the **potential** rises by one. Defaults by creature tag cover the roster (giants Strength, trolls Toughness, casters and hags the mind, fey Presence); hand-set values stay. Rot, harm, vomiting, and kin-eating are unchanged.
- **Potions, elixirs, and Hard-won** become shares of the price too (a potion of might 5 %, an elixir 20 %, Hard-won half of the point a story took), so they mean the same at a score of 12 and of 25. **A trained rise no longer lifts the potential**: only elixirs and corpses widen it (§4.4).
- **Talents** (§9): Hard training re-pointed to the band (30 full-odds marks a day), two new talents on the price (Gifted: a chosen stat 15 % cheaper; Late bloomer: the ramp a step gentler), two on losses (Unshakeable: deeds take half; Ascetic: a hungry dawn trains Willpower instead of wasting Strength), two class abilities (the barbarian's Hard living, the monk's Discipline). No new `[[system]]` tag: losses and corpse marks are channels of `stat_training` and `hunger`.
- **Six slices** (§13): the pile and the price (state) first; then the paths, the deeds, and the corpses beside each other; then the talents; then the probe and the docs.

## 2. The owner's request, and what exists today

What the code does now, verified line by line (`docs/dev/stats.md` "Training" is current):

| Piece | Today |
|---|---|
| The pile | `Actor::stat_marks: PerStat<u16>`, saturating, never decays, left out of the JSON while all zero; saved and hashed. Only the player trains, alive. |
| Earning | 33 uses (`StatUse`), each `[[training.use]]` in `data/stats.toml` with a `chance` (1 to 100): one draw of the **training** stream per rule below 100. Five are the skills' own reports (`World::train` serves both: burdened, kick, dodge, sneak, sneak_attack, spot, study). A potion's `marks` (`drank`, `elixir`) are sure, as many as it says (potion of might 5, elixir 20; Quick study of the body doubles them). Hard-won gives **3** marks per stat point a story line took (`hard_won.toml`: `marks = 3`). |
| The level check | `World::check_stats` from `level_up`: six **levels**-stream draws always; a stat rises with `marks / (marks + need)`, `need = 20 + 3 × score`; at or above the potential the denominator is ×4; a rise sets the marks to **0**, lifts the potential with the score (`Stats::train`), and says "You feel stronger." |
| The daily check | `World::daily_stat_check` at 06:00 (`WorldState::next_stat_check`), the same formula with the need ×`daily_factor` (3; Hard training 2), six **training**-stream draws always; a rise says the same at once and stops a rest. |
| Potentials | the race's `base_potential` 15 plus adjustments, never below the start; raised by a trained rise past it and by elixirs (`Stats::widen`); corpses raise a score up to it, never past. |
| Corpses | `OnEat::stat = { toughness = 5 }`: a 5 % gameplay draw to raise the score by one, up to the potential. 273 creature types; 145 have an `on_eat`, **41** name a stat (Toughness 5 ×11, Strength 3 ×11, Willpower 3 ×5, …), 112 leave no corpse. `corpse_stats_percent` (Cauldron, Gourmand: +100) doubles the chance. |
| Losing | nothing lowers marks. Points are lost for good only to drains (`Actor::drained`, mended at a temple) and for a while to stat damage and status penalties. |
| Streams | marks on `WorldState::training`; the level check on `levels_rng`; the daily check on `training`; corpses and elixirs on the gameplay stream. |
| Shown | nothing but "You feel stronger." and the new score; the dev `progress` layer lists the marks; the wiki has `stat-training.md`, `stat-checks.md`, `stat-daily-checks.md`. |

Two things the brief assumed are not so: Hard-won gives 3 marks a point, not 12; and the design's "8 to 15 marks a game day in a trained stat" (combat progression §9.1) was arithmetic over 30 blows a day, while the curve asks for a hundred times that (§3).

## 3. The audit: what today's rules do with today's numbers

### 3.1 Mark volume on the curve

`data/experience.toml` asks for `k(n) = 213 × m / (m + 5) + 2.85 × m` kills of your own level per level (m = n − 1), discounted early: 10 kills for level 2, 42 for 5, 146 for 10, 223 for 20, 264 for 30, 365 for 60. `docs/balance.md` measures the plain warrior at 1.6 rounds a kill at level 1, 4.3 at 10, 8.3 at 30, 11 at 60, hitting 87 to 98 % of the time; monsters hit it 35 % of the time at level 1, 13 % at 10, 2 % at 30, 16 % at 60. Marks a level that this implies, under today's chances (`blow` 15, `critical` 40 on ~8 % of hits, `block` 20 on the shield's share of misses, `burdened` 15; `dodge` 30 on Dexterity's share of misses; `hurt` 20, `big_hit` 60, `killed` 30; the wizard's `cast` 30 per cast; `discovered`, a rest's `endured`, a few chats and trades for the rest), with a plausible length of a level in game days (one for levels 2 to 5, two to 15, three to 30, four beyond: 189 days in all; `docs/balance.md` assumes a flat two):

| Level | Kills | Days | Warrior St | Dx | To | In | Wi | Pr | Wizard In | Wi |
|---|---|---|---|---|---|---|---|---|---|---|
| 2 | 10 | 1 | 4 | 1 | 3 | 2 | 1 | 5 | 3 | 9 |
| 5 | 42 | 1 | 29 | 9 | 12 | 2 | 1 | 5 | 3 | 46 |
| 10 | 146 | 2 | 143 | 49 | 35 | 2 | 2 | 5 | 3 | 200 |
| 20 | 223 | 3 | 309 | 110 | 38 | 2 | 3 | 5 | 3 | 437 |
| 30 | 264 | 3 | 519 | 194 | 36 | 2 | 3 | 5 | 3 | 637 |
| 45 | 317 | 4 | 726 | 268 | 67 | 2 | 4 | 5 | 3 | 906 |
| 60 | 365 | 4 | 867 | 303 | 207 | 2 | 4 | 5 | 3 | 1316 |

(A Monte Carlo of the rules as written over this activity model, 200 careers, run for this document and not committed: T6 moves the measurement into the balance probe, on the real hooks. The probe today feeds the rules 7 Strength marks a day; the curve feeds them 70 to 220.)

### 3.2 What the formula does with that

`p = m / (m + need)` is a hyperbola: it is 21 % at a tenth of the need, 50 % at the need, 72 % at twice it, 89 % at five times. Two faults, and a third in the reset:

1. **A flood saturates.** With 300 marks a level against a need of 62 (Strength 14) the level check is 83 % and the daily check 62 %: the stat rises nearly every day until the potential, and at the ÷4 beyond it still 18 % a day, about once in six days. The pile then resets, so the surplus is thrown away: 300 marks bought what 60 would have. **Effort past the first few dozen marks is worth nothing.**
2. **A trickle is cheap.** Presence earns about 5 marks a level. At 20 marks the daily check is 20 / (20 + 150) = 12 %: the point comes within ten days, for 20 marks. A point for a trickle stat costs 15 to 30 marks; a point for the flood stat costs 300. **The price of a point varies twentyfold with how fast you earn**, in the trickle's favour (the "days to a point" table in §4.3 has both).
3. **The reset is the feel-bad the owner named**, and also the mechanism of fault 1.

The ×4 at the potential is a cliff: one point below it the chance is 46 % a day (at 300 marks), one point above 18 %. Lifting the potential with the score then makes every further point the same 18 %: nature does not slow the character further, it only taxes once.

### 3.3 The result: a career on today's rules

Mean final scores of 200 simulated careers to level 60, start/potential → level 60 (today's rules, the volumes above):

| | Warrior St 14/17 | Dx 12/15 | To 13/16 | In 10/15 | Wi 10/15 | Pr 10/15 | all six | Wizard Wi 13/16 | To 10/13 | Pr 10/15 | all six |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Today | **40** (+26) | 39 (+27) | 34 (+21) | 17 (+7) | 19 (+9) | 20 (+10) | **+101** | **40** (+27) | 39 (+29) | 21 (+11) | **+102** |

Strength and Dexterity hit the cap of 40 before level 60 (Toughness would, given one more point of monster accuracy), and the three stats the warrior never uses rise 7 to 10 points on trickles. ADOM's trained characters gain about 25 to 35 points in all over a long game (the progression design §6.4 named that as the aim). The maths, not the chances, is the fault: lowering every chance by ten would only move the warrior's three stats toward the trickle regime and leave the twentyfold price gap.

### 3.4 Paths: which stats have them

Every stat use the code reports today against every system (the hook is the one line that calls `train_stat` or `train`):

| System | Strength | Dexterity | Toughness | Intelligence | Willpower | Presence |
|---|---|---|---|---|---|---|
| Melee (`fight.rs::resolve`) | blow, critical, block | parry, dodge | hurt, big_hit, killed | — | hungry_fight | — |
| Ranged (`resolve`) | — | shot | hurt | — | — | — |
| Spells (`spend_mana`) | — | — | — | — | cast | — |
| Skills' uses (`train.rs`) | burdened, kick | dodge, sneak, sneak_attack | — | study | spot | — |
| Burden (`apply`) | burdened | — | — | — | — | — |
| Hunger (`eat.rs`, `keep_at`) | — | — | — | — | endured (a meal) | — |
| Diseases (`disease.rs`) | — | — | resisted | — | resisted_will | — |
| Exploration (`search.rs`, `read.rs`, `experience.rs`) | dig | climb | plunge | found, read, identified, discovered | endured (a search, a dig) | — |
| Rest (`rest.rs`) | — | — | — | — | endured | — |
| Social (`talk.rs`, `shop.rs`, `quest.rs`) | — | — | — | — | — | chat, trade, quest, swapped |
| Crafting (`temper.rs`) | — | — | — | — | — | — |
| Stealth and theft (`hearing.rs`, `steal.rs`) | — | sneak | — | — | — | — |
| Items (`potion.rs`) | drank, elixir | drank, elixir | drank, elixir | drank, elixir | drank, elixir | drank, elixir |

Thin: **Intelligence** has no path that a fighter walks (found, read, identified, discovered are rare events, together 2 to 3 marks a level) and no path through casting at all, though spells scale by Intelligence; **Presence** has no path through anything but talk and trade; **Toughness** nothing from travel, cold, wading, or hunger; **Strength** nothing from crafting (the forge's Strength check trains nothing), struggling, or wading; **Willpower** nothing from prayer, struggling free, or a hold; **Dexterity** nothing from picking locks, disarming traps, stealing, or an initiative check made, though all four are Dexterity checks. Nothing trains from the overworld, the guild, the beasts befriended, the stamina spent on weapon abilities, or a shield bash in particular. §5 fills the matrix.

## 4. The maths: one signed scale, a price per point, odds that pay the price

### 4.1 Goals, stated first

| # | Goal | Why |
|---|---|---|
| G1 | **A point costs about the same marks however fast they come.** A flood should buy more points than a trickle, in proportion. | fault 2 of §3.2 |
| G2 | **A rise never wipes the pile**: it pays a price; the surplus carries. | the owner's addendum |
| G3 | **Odds grow with marks without saturating early**: small piles are nearly worthless, the price is the 50 % mark, twice the price is near-certain. | fault 1 of §3.2; a lucky point from a tenth of the effort cheapens effort |
| G4 | **Nature slows growth smoothly**, no cliff at the potential; a long career outgrows it slowly, never never. | the progression design's intent; the ×4 cliff |
| G5 | **One scale for both signs**: negative marks mirror positive ones, with a drop that pays its own price back; zero is neutral; no cheap oscillation. | the owner's third addendum |
| G6 | **The daily and level checks are the same rule**: a second check moves the moment, not the total. | the ×3 factor was a patch over fault 1 |
| G7 | **A sane pace by level 60**: a main stat +8 to +12, a secondary +4 to +8, an unused stat +1 to +3; +25 to +40 in all. | the progression design §6.4; ADOM |
| G8 | **Integers only, on the existing streams**, with the draws in step whatever the marks. | `docs/dev/stats.md`'s invariants; replays |

### 4.2 The price of a point

`base(score) = price_base + price_per_point × score` = **20 + 10 × score** (data). Past the potential the price ramps: with `over = score + 1 − potential` (the point you are about to gain, counted from the potential: 1 for the first beyond it),

`P(score, potential) = base(score) × (ramp_divisor + over × (over + 1)) / ramp_divisor`, `ramp_divisor` 4, for `over ≥ 1`; `base(score)` otherwise.

So the first point past your nature costs ×1.5, the second ×2.5, the third ×4, the fourth ×6, the fifth ×8.5, the sixth ×11.5: quadratic, so a career of two hundred days ends four to six points past its nature in the stat it lives by and never reaches 40 by use alone (the sweep in §6.2: ÷2 to ÷6 moves the warrior's Strength between +9 and +12; the owner may pick). The point *reaching* the potential costs the plain base: the ramp begins where nature ends, and there is no step at it beyond the first ×1.5.

**The drop's price** is `L(score) = base(score − 1) / drop_share`, `drop_share` 2: half the base price of the point being lost, **never ramped**. Losing is easier than winning (muscle wastes faster than it builds), a point past nature is lost at the same price as any other but regained at its ramped price, so neglecting a stat you pushed past nature is costly: it falls back toward the potential first.

| Score | Base price | Past the potential: 1st | 2nd | 3rd | 4th | 5th | 6th | Drop price |
|---|---|---|---|---|---|---|---|---|
| 6 | 80 | 120 | 200 | 320 | 480 | 680 | 920 | 35 |
| 8 | 100 | 150 | 250 | 400 | 600 | 850 | 1150 | 45 |
| 10 | 120 | 180 | 300 | 480 | 720 | 1020 | 1380 | 55 |
| 12 | 140 | 210 | 350 | 560 | 840 | 1190 | 1610 | 65 |
| 14 | 160 | 240 | 400 | 640 | 960 | 1360 | 1840 | 75 |
| 16 | 180 | 270 | 450 | 720 | 1080 | 1530 | 2070 | 85 |
| 18 | 200 | 300 | 500 | 800 | 1200 | 1700 | 2300 | 95 |
| 20 | 220 | 330 | 550 | 880 | 1320 | 1870 | 2530 | 105 |
| 25 | 270 | 405 | 675 | 1080 | 1620 | 2295 | 3105 | 130 |
| 30 | 320 | 480 | 800 | 1280 | 1920 | 2720 | 3680 | 155 |
| 35 | 370 | 555 | 925 | 1480 | 2220 | 3145 | 4255 | 180 |

Why linear in the score below the potential: the modifier is linear in the score (`floor((s − 10) / 2)`), so each point is worth the same to the rules; a steeper base would only slow the high-born. Why the start is 20 + 10 × score and not today's 20 + 3 × score: today's need is a *chance* parameter; this is a *price* in marks, and the band of §4.5 sets what a day of a stat's use earns (15 to 35 marks in a stat you live by), so a price of 160 is a week to ten days of living by a stat below the potential, the pace of §6.

### 4.3 Odds at a check, and what a rise or a drop leaves behind

At a check, with `m` marks and the price `P` (for `m > 0`) or `L` (for `m < 0`):

- **Nothing happens while `4 × |m| < price`** (the quarter rule): the stat is not in play.
- Otherwise the stat moves with chance **`|m|³ / (|m|³ + price³)`**: a rise if `m > 0`, a drop if `m < 0`.
- **A rise pays `P`**: `m ← max(0, m − P)`. **A drop pays `L` back**: `m ← min(0, m + L)`. If the pile was short of the price (a lucky early move), it goes to zero: what was there is spent, and no more.
- **At most one move per stat per check.** A flood of marks (an elixir on a full pile) turns into a point a day plus one a level, the surplus waiting. There is no other cap.

| Marks held | Chance this check | Today, at a level (St 14, need 62) | Today, at 06:00 (need ×3) |
|---|---|---|---|
| under ¼ of the price | 0 % | 21 % at a tenth | 8 % at a tenth |
| ¼ | 2 % | 39 % | 18 % |
| ½ | 11 % | 56 % | 30 % |
| ¾ | 30 % | 66 % | 39 % |
| **the price** | **50 %** | 72 % | 46 % |
| 1¼ | 66 % | 76 % | 52 % |
| 1½ | 77 % | 79 % | 56 % |
| 2× | 89 % | 84 % | 63 % |
| 3× | 96 % | 89 % | 72 % |

(Today's columns put the proposal's price of 160 marks against today's need of 62, to compare at equal marks.) The cube is the S-curve of G3: a tenth of the price is nothing, the price is even, twice it is sure enough. The power is data (`odds_power`, 3): 2 is a softer curve (20 % at half the price) and favours trickles, 4 is nearer a threshold; §4.3's table below is why 3.

**G1 checked**: a stat checked daily rises when its pile first wins a check, so what it pays depends on how fast it fills. Simulated at a price of 160 (400 runs each):

| Marks a day | Days to the point | Marks it cost | Share of the price | Today: days (daily check) | Today: marks it cost |
|---|---|---|---|---|---|
| 1 | 59 | 59 | 0.37 | 17 | 17 |
| 2 | 35 | 69 | 0.43 | 12 | 25 |
| 4 | 21 | 82 | 0.51 | 9 | 34 |
| 8 | 12 | 99 | 0.62 | 6 | 49 |
| 15 | 8 | 117 | 0.73 | 5 | 71 |
| 25 | 5 | 125 | 0.78 | 4 | 93 |
| 40 | 4 | 138 | 0.86 | 3 | 120 |
| 80 | 2.4 | 150 | 0.93 | 2.2 | 178 |
| 160 | 1.6 | 160 | 1.00 | 1.8 | 288 |

A trickle still gets a discount (a mark a day pays a third of the price, because the daily check gets sixty tries at low odds), but the price gap across a 160-fold income range is **2.7×**, against today's **17×** in the trickle's favour (17 marks against 288) *and* with today's trickle reaching the point three times faster than the proposal's. With `odds_power` 4 the gap would be 1.8× and the curve a near-threshold; with 2 it is 4.5×. Three is the middle.

### 4.4 The potential

- A trained rise **no longer lifts the potential** (`Stats::train` changes: the score only). The potential is nature, set at the start; it moves only by **elixirs** (`Stats::widen`, as today) and **corpses** (§8), and by gifts (a story's, a class's: `Stats::gift` moves both, as today).
- The ramp is measured from it, so the sixth point past nature costs ×11.5 and the seventh ×15.5: a stat does not reach 40 by use alone in two hundred days (§6.2), while an elixir or a giant's heart (potential +1) makes the next point ×1 cheaper for every point after. That is what a potential is for.
- A drop never lowers the potential (§7.3): nature is not undone by a bad month.

### 4.5 The daily band: earning is neither hard nor free

Each stat counts the marks that **moved it today**, either way (`Actor::marks_today: PerStat<u16>`, reset at the 06:00 check; saved, hashed, left out while zero). A use's chance is scaled by the band the count is in: the first **15** marks of the day at the full chance (×8/8), the next 15 at half (×4/8), the next at a quarter (×2/8), everything beyond at an eighth (×1/8, the floor; never zero). A use at 100 % is a draw at 50 % in the second band. The scale is one integer compare (`below(800) < chance × factor`), one draw per rule as today.

| Blows that hit in a day, at 15 % | Marks today (unbanded) | Marks, banded |
|---|---|---|
| 20 | 3 | 3 |
| 50 | 8 | 8 |
| 100 | 15 | 14 |
| 200 | 30 | 22 |
| 400 | 60 | 34 |
| 800 | 120 | 46 |
| 2000 | 300 | 69 |

A light day is untouched; a day of a hundred fights gives half again what a day of fifty does, not twice; a day of a thousand four times, not twenty. The owner's rule holds: nothing is capped, nothing decays, the thousandth blow still has a chance, and the only price of grinding is the clock. Sure marks that are **paid for** (a potion's, an elixir's, a corpse's, a story's) bypass the band: they are not uses. Deeds that take marks (§7) share the counter: a day's losses band the same way, so one terrible day cannot carry a stat from neutral into the drop zone (§7.3 shows the numbers).

### 4.6 Integers and streams

- Prices: `u32` arithmetic in `StatTraining::price(score, potential)` and `drop_price(score)`.
- Odds: `m` clamped to `8 × price` (99.8 % there; above it the chance is treated as that), then `num = m³`, `den = m³ + price³` in `u64` (`(8 × 4255)³ ≈ 3.9 × 10¹³` fits). `Rng::below` takes a `u32`, so the odds are reduced to a 1/65 536 scale: `scaled = num × 65 536 / den` (u64), one `below(65 536)` draw, a move if `draw < scaled`. Deterministic, no floats, exact to a part in 65 536.
- The streams stay what they are: marks on `WorldState::training` (one draw per rule per use, as today; the band changes the compare, not the draw count), the level check on `levels_rng`, the daily check on `training`. **Six draws per check always**, whatever the piles (so a fight's rolls never move and the stream's position stays a function of the days and levels passed): a stat under the quarter rule still draws and ignores the result.
- Sure marks (`Drank`, `Elixir`, a corpse's `train`, Hard-won) draw nothing (as today at 100).
- A drop's hit points: `keep_hp_within_max`, never below 1; mana likewise (`drain.rs` does the same).

### 4.7 Alternatives, and why not

| Alternative | Why not |
|---|---|
| **Today's hyperbola with a price paid instead of a reset** (`p = m / (m + need)`, then `m −= need`) | keeps fault 1 (21 % at a tenth: cheap lucky points) and fault 2 (the trickle's discount is 4× not 2.7×, and it reaches the point faster than a flood per mark); the check frequency still sets the pace more than the marks do |
| **A plain ladder** (rise when `m ≥ P`, pay `P`; no odds) | G1 exactly (a point costs `P` whatever the income) and simplest; rejected because the owner asked for odds both ways, because the daily and level checks then mean nothing but a reading of the pile, and because two characters doing the same things would rise on the same dawn. It is the cube with `odds_power → ∞`; the data knob is there if the owner prefers it |
| **Marks decay** (a share a day) instead of a band | punishes the player who stops using a stat (travel, a shop day), which is the anti-farming shape the owner rules out; the band bounds a day's gain, decay bounds the pile |
| **A per-use chance to raise the stat directly** (ADOM; no pile) | no pile to pay into (potions, corpses, stories), no negative side, and a direct raise per blow at one in a thousand is a flood stat's point every two days at level 30 |
| **A logistic or exponential odds curve** | floats, or a table; the cube is the same shape with one multiply |
| **Daily factor ×3 kept** | a patch over fault 1; with a price paid, a second check per day only moves the day of the rise, and the trickle discount already comes from the daily check |
| **The potential lifted with a trained rise** (today) | makes the ramp a one-time tax (§3.2); nature should slow every further point |
| **A hard cap on the pile** | the owner's feel-bad in another coat: effort thrown away |
| **Signed marks with the drop at a fixed −N** | a fixed threshold ignores the score; the drop's price is half the point's base, so a high score is as hard to lose as it was to gain, in proportion |

## 5. Training paths: every stat, every system

The rule for a path: one existing hook (a function that already knows the event), one line, one `StatUse` variant with its `describe()` words, one `[[training.use]]` entry. Chances are first guesses in the band's regime (15 full-odds marks a day): a frequent use (hundreds a day) 5 to 15 %, an occasional one (tens) 20 to 40 %, a rare one (a few) 100 %. Existing uses keep their chances unless noted.

### 5.1 Strength

| Use | What | Chance | Hook (one line) |
|---|---|---|---|
| blow | a melee blow of yours that hits (kick, bash too) | 15 | `fight.rs::resolve` (today) |
| critical | a confirmed critical of yours up close | 40 | today |
| block | a blow on you your shield blocked | 20 | today |
| burdened | an action, not a wait, while your load slows you | 15 | `World::apply` (today) |
| dig | a turn of digging | 10 | `dig_turn` (today) |
| kick | a door kicked, a chest kicked open | 20 | `kick_door`, `chest.rs` (today, and the chest's `Use::Kick` line joins it) |
| **forged** | a tempering done at an anvil, whatever came of it | 40 | `temper.rs::tempered` (beside `Use::Temper`) |
| **struggled** | a struggle against a hold (`World::struggle`), won or not | 30 | `hold.rs::struggle` |
| **waded** | a turn wading through deep water | 10 | `disease.rs::wade` (the one place wading is known) |
| **hauled** | a corpse or stack of 20 kg or more picked up | 50 | `pick_up`, after the weight is known |

### 5.2 Dexterity

| Use | What | Chance | Hook |
|---|---|---|---|
| dodge | an attack on you your Dexterity or Dodge made miss; a known trap stepped past | 30 | `resolve`, `sidestep.rs` (today) |
| parry | a blow your Parry turned | 25 | today |
| shot | a shot or throw of yours that hits | 20 | today |
| sneak | a step within earshot of a creature that could hear it | 10 | `hearing.rs` (today) |
| sneak_attack | a sneak attack of yours that hits | 20 | `ambush.rs` (today) |
| climb | out of the water onto ice, out of a pit | 50 | today |
| **picked** | a lock picked (the attempt, won or not) | 30 | `lockpick.rs` beside `Use::Pick` |
| **disarmed** | a trap disarmed (the attempt) | 40 | `disarm.rs` beside `Use::Disarm` |
| **stole** | a theft attempted | 30 | `steal.rs` beside `Use::Steal` |
| **kept_wits** | an initiative check of yours made (not surprised) | 50 | `surprise.rs::startle`, the player's made check |
| **aimed** | an attack spell of yours that hits (rays, bolts, touches aim by Dexterity) | 15 | `resolve`'s `AttackKind::Spell` arm, beside `shot` |

### 5.3 Toughness

| Use | What | Chance | Hook |
|---|---|---|---|
| hurt | a blow that costs you hit points | 15 (from 20: a wizard's Toughness ran away in §6) | today |
| big_hit | a big hit or a critical on you | 60 | today |
| killed | a kill of yours up close, of a foe of your level or above | 30 | today |
| resisted | a disease check won (not Willpower's) | 50 | today |
| plunge | falling through the ice | 100 | today |
| **marched** | a step over the overworld | 15 | `travel.rs` beside `Use::Travel` |
| **hungry_step** | an action, not a wait, while Hungry (not yet Weak) | 3 | `World::apply`, beside `burdened` (hunger known there) |
| **shook_off** | a harmful status ending by time (poisoned, bleeding, sickness: any status with a `harm`) | 50 | `status.rs`, the one place a status ends by easing |
| **exerted** | stamina spent on a weapon ability | 20 | `spend_stamina` (`docs/dev/weapon-abilities.md`) |
| **chilled** | a daily check made while the player's temperature is below the chill line | 100 | `daily_stat_check`, reading `World::temperature` |

### 5.4 Intelligence

| Use | What | Chance | Hook |
|---|---|---|---|
| found | a search that finds something | 50 | today |
| read | a scroll read | 40 | today |
| identified | an item identified by reading | 100 | today |
| study | a spell studied from a scroll | 100 | today |
| discovered | your first kill of a kind of creature | 100 | today |
| **spell** | a spell you know cast (not a class or sign ability): the stat its `scales` damage names, else Intelligence | 20 | `cast.rs::cast_spell`, the one cast path |
| **explored** | your first arrival on a level that pays arrival XP | 100 | `experience.rs::discover_level` |
| **appraised** | an item realised by Appraise, or a shop's price read in look mode | 50 | `appraise.rs` beside `Use::Appraise` |
| **searched** | a turn of searching that found nothing | 3 | `search.rs::search_turn` beside `Use::Search` |
| **foraged** | a forage done | 30 | `forage.rs::forage_done` beside `Use::Forage` |
| **disarmed** | a trap disarmed (shared with Dexterity: knowing the mechanism) | 20 | the same line as §5.2's |

### 5.5 Willpower

| Use | What | Chance | Hook |
|---|---|---|---|
| spot | noticing something hidden in passing | 30 | today |
| endured | a long action done to its end (a rest, a dig, a search, a meal) | 100 | `keep_at` (today) |
| resisted_will | a Willpower disease check won | 50 | today |
| hungry_fight | a blow landed while Hungry or worse | 20 | today |
| cast | mana spent on an ability | 30 | `spend_mana` (today) |
| **prayed** | a prayer done at a forgotten altar | 100 | `pray.rs::prayer_answered` |
| **struggled** | a struggle against a hold (shared with Strength) | 30 | the same line as §5.1's |
| **held_out** | a drain check made | 100 | `drain.rs`, the made arm (beside `train_resisting`) |
| **steadied** | a Surprised status endured to its end without fleeing a cell | 50 | `status.rs`, Surprised's end |
| **spell** | a spell cast whose `scales` reach names Willpower (the one `spell` use, two stats) | 20 | the same line as §5.4's |

### 5.6 Presence

| Use | What | Chance | Hook |
|---|---|---|---|
| chat | a chat | 30 | today |
| trade | a purchase paid for or a sale made | 20 | today |
| quest | a quest completed | 100 | today |
| swapped | a creature making way for you | 10 | today |
| **inspired** | Inspire, Throw voice, or any class active that works on others' minds | 100 | `class_ability.rs::inspire`, `voice.rs::throw_voice` |
| **promoted** | a guild rank won | 100 | `guild.rs::promote` |
| **befriended** | a beast calmed by Beast friend (the first time per creature) | 100 | the AI's `befriended` check, where a hostile stands down |
| **hexed** | a spell cast whose `scales` duration names Presence (the hexes) | 20 | the `spell` line |
| **bargained** | a shop's price bettered by Negotiation (a deal where the skill mattered) | 30 | `shop.rs` beside `Use::Deal` |
| **greeted** | the first chat with each person (a new acquaintance) | 100 | `talk.rs::chat`, when the talk count was 0 |

Every stat now has eight to eleven paths over at least five systems; a fighter who never opens a scroll still earns Intelligence from exploring, searching, and foraging, and a wizard Strength from wading, hauling, and struggling. The `spell` use is one hook that reads `SpellScales` (`docs/dev/spells.md`): damage → Intelligence, reach → Willpower, duration → Presence, to-hit (`aimed`) → Dexterity, each 15 to 20 %, so casting trains the mind the spell actually uses and `cast` (mana spent) stays Willpower's.

## 6. Pace: what a career yields

### 6.1 Targets (G7)

| Stat's place in the build | Marks a day (banded) | Points by level 60 | Of them past the potential |
|---|---|---|---|
| lived by (a fighter's Strength, a caster's Willpower) | 15 to 35 | +8 to +12 | 4 to 6 |
| secondary (the fighter's Dexterity and Toughness) | 6 to 15 | +5 to +9 | 2 to 4 |
| touched (Presence for anyone who talks and trades) | 2 to 5 | +3 to +5 | 0 to 1 |
| unused (a warrior's Intelligence) | ~1 | +1 to +2 | 0 |
| all six | | **+25 to +40** | |

### 6.2 The simulation

The same activity model as §3 (the curve's kills, the probe's rounds and hit chances, 189 days), the proposal's rules (price 20 + 10 × score, ramp ÷4, cube odds, the quarter rule, the band at 15, a rise paying its price, both checks), 200 careers each. Mean scores at level 60:

| Character | Stat | Start/potential | Today | **Proposed** |
|---|---|---|---|---|
| warrior | St | 14/17 | 40 (+26) | **25.1 (+11.1)** |
| | Dx | 12/15 | 39 (+27) | 21.9 (+9.9) |
| | To | 13/16 | 34 (+21) | 21.0 (+8.0) |
| | In | 10/15 | 17 (+7) | 11.7 (+1.7) |
| | Wi | 10/15 | 19 (+9) | 13.2 (+3.2) |
| | Pr | 10/15 | 20 (+10) | 14.5 (+4.5) |
| | all six | | +101 | **+38.5** |
| wizard | St | 9/13 | 18 (+9) | 12.8 (+3.8) |
| | Dx | 11/15 | 30 (+19) | 19.3 (+8.3) |
| | To | 10/13 | 39 (+29) | 20.4 (+10.4) |
| | In | 15/18 | 22 (+7) | 17.1 (+2.1) |
| | Wi | 13/16 | 40 (+27) | **24.5 (+11.5)** |
| | Pr | 10/15 | 21 (+11) | 14.5 (+4.5) |
| | all six | | +102 | **+40.6** |

The wizard's Intelligence is +2 only because the model has no `spell` use yet (§5.4 gives it 20 % per cast: the wizard's casts are the warrior's blows, so Intelligence joins Willpower as a stat lived by, and the wizard's Toughness, +10 from being hit in six rounds of ten, is why `hurt` falls to 15 %). The warrior's Dexterity rides on the dodge attribution (30 % of monster misses); the probe should measure it.

**The knobs** (the sweep, 60 careers each; `ramp_divisor` × band `full`):

| Ramp ÷ | Band | W St | W Dx | W To | W In | W Wi | W Pr | Z Wi | Z To |
|---|---|---|---|---|---|---|---|---|---|
| 2 | 10 | +8.9 | +8.1 | +6.8 | +1.7 | +3.2 | +4.6 | +9.3 | +8.5 |
| 2 | 15 | +9.4 | +8.6 | +7.0 | +1.7 | +3.3 | +4.4 | +9.9 | +8.9 |
| 3 | 15 | +10.2 | +9.3 | +7.7 | +1.7 | +3.3 | +4.5 | +10.8 | +9.9 |
| **4** | **15** | **+11.1** | **+9.9** | **+8.0** | **+1.7** | **+3.3** | **+4.3** | **+11.4** | **+10.4** |
| 6 | 15 | +12.1 | +11.0 | +8.9 | +1.7 | +3.2 | +4.5 | +12.8 | +11.3 |

The ramp sets the main stats' tail; the band hardly matters past 10 (the floods are already in the eighth band). Unused stats are set by their paths, not the knobs. The probe (§13, T6) re-measures with the real hooks and the owner picks the row.

### 6.3 Early game

Below the potential a point costs 160 marks at 14; a fighter earning 20 to 25 a day meets the first rise about day 6 to 8 (level 7 to 9 on the model's days), the second around day 14, the third around day 22, then the ramp. Today's rules give the first point at level 2 or 3. ADOM's early rises come mostly from the potential-bound corpse and herb raises, which §8 keeps as marks: a kobold corpse a day is a few marks, a giant's heart is a third of a point. If the owner wants the first rise earlier, `price_base` 20 → 0 makes the first points 15 % cheaper without touching the tail.

## 7. Losing marks and losing points

### 7.1 Deeds

The rule for a deed: one existing hook that already knows the event, one `[[training.deed]]` entry: `stat`, `on`, and either `chance` (a mark taken, one draw, like a use; banded) or `share` (a share of the **drop price** `L`, taken at once; banded as a count). Deeds are things the player did or let happen, never bad luck alone: a hidden trap sprung, a blow taken, or a disease caught take nothing.

| Stat | Deed (`on`) | Takes | Hook |
|---|---|---|---|
| Strength, Toughness | **hungry_dawn**: the 06:00 check made while Weak, Fainting, or Starving | 20 % of L each | `daily_stat_check`, reading `player_hunger` |
| Toughness | **starved**: a hit point lost to starvation | chance 50 | `hunger.rs::digest`, the starving branch |
| Toughness | **vomited**: a vomit (overeating, rotten food, a corpse that disagreed) | 10 % of L | `eat.rs::vomit` |
| Toughness | **sickened**: a disease check lost (not Willpower's) | 15 % of L | `disease.rs::disease_turn`, the failed arm (the mirror of `resisted`) |
| Willpower | **broke**: a Willpower disease check lost; a drain check failed | 15 % of L | the same arm; `drain.rs`, the failed arm |
| Willpower | **fainted**: a faint | 10 % of L | `hunger.rs::faint` |
| Dexterity | **overloaded**: an action, not a wait or a drop, while Overloaded | chance 25 | `World::apply` beside `burdened` (`Burden::Overloaded`) |
| Dexterity | **strained**: the same while Strained | chance 5 | the same line |
| Intelligence | **knew_better**: a trap you knew of sprung under you | 25 % of L | `trap.rs::spring_trap`, the `known` branch for the player |
| Presence | **caught**: caught stealing | 25 % of L | `shop.rs::caught_stealing` |
| Presence | **unwelcome**: a shop debt turning you Unwelcome | 25 % of L | the status's start, the shop's one place |
| St, Dx, To | **idle**: a stat that moved no mark in 7 days running, each further dawn | 5 % of L | `daily_stat_check`, from `Actor::idle_days: PerStat<u8>` (new state; Q6) |

Not deeds, and why: **fleeing** (the game has no flee action and the Coward stance is a legitimate choice; a stance never costs marks); **being Frightened** (only creatures are, by Hound's bay); **killing peaceful folk, eating kin** (alignment already pays, and a Presence mark on top would make the evil character's Presence a second alignment: not twice); **a drain that lands** (the point is lost already; the failed check is the deed, `broke`); **resting long** (rest heals and trains Willpower; the idle rule covers a stat left wholly alone); **wearing heavy armour** (armour has `max_dex` and weight, both already paid for in Defence and burden; the Strained and Overloaded deeds are the "far too heavy" case).

### 7.2 The drop

At a check, a stat whose pile is at or below −L/4 is in play: it drops by one with `|m|³ / (|m|³ + L³)` and the pile pays L back toward zero (§4.3). Floors: never below **3** (`DRAIN_FLOOR`, the one floor for lost points), never below the stat damage on it plus one (a score carries its damage, as drains require), and never a drop in a stat the player cannot use (none: every stat is in play). Hit points and mana are cut to the new maxima, never below 1: a drop never kills.

**The potential stays.** A drop is neglect, not a change of nature, so the lost point is won back at the plain base price (below the potential) and the player who starved for a week is set back a week or two of use, not a month. A point *past* the potential that is lost is regained at its ramped price (§4.2): neglect above nature costs more, which is as it should be.

**Not a drained point.** `Actor::drained` counts drains only; a temple's mending does not restore a trained point lost. Stat damage mends as before and is untouched by marks; the effective score the rules read is the stored score (lowered by a drop) less damage and penalties, as today.

### 7.3 Numbers, and the oscillation check (G5)

- **A warning comes first.** When a stat's pile first crosses −L/4 (the zone where a drop is possible), the player is told in words (§10): "You feel your strength wasting." Simulated at score 14 (L 75): a character Weak at every dawn (−15 Strength and −15 Toughness a day) hears the warning on day 2 and loses the point on day 4.4 on average (never before day 2, at most day 9); the pile is left at −7 on average, never below −L (`settle` adds L back). Zero drops came before the warning in 400 runs.
- **Noise cancels.** A stat gaining 20 marks one day and losing 20 the next for 200 days: 0 rises and 0 drops in 200 careers. Zero is neutral, and the quarter rule keeps a pile that hovers near it out of play.
- **No cheap round trip.** Losing a point at 14 costs 75 marks of deeds (two to five days of starving oneself, or three days of acting Overloaded at the band's rates); winning it back costs 150 marks of use. The round trip nets nothing and costs 225 marks: there is nothing to farm. The band bounds a day's movement each way to about 35 marks (15 + 7 + 4 + an eighth of the rest), so a stat at zero needs at least two bad days to reach any drop odds and three to reach even odds.
- **Hard-won, elixirs, potions** pay into the pile: marks that arrive at a negative pile raise it toward zero first (an elixir is an antidote to a bad month: that is fine, it costs an elixir).

### 7.4 Saved state

`Actor::stat_marks` becomes `PerStat<i32>` (the JSON map is read as before; the `u16` default of zero is the same zero): **saves change**, with `Actor::marks_today: PerStat<u16>` and, if Q6 is approved, `Actor::idle_days: PerStat<u8>`; all three left out of the JSON while zero, so a run that never trained hashes as today. `FORMAT_VERSION` is the integrator's per-release bump and this work does not touch it.

## 8. Corpses: marks and potential, not points

### 8.1 The data

`OnEat::stat` (a chance per stat to raise the score by one, up to the potential) is replaced by two fields:

- `train = { toughness = 20 }`: sure marks, **20 % of the price of the stat's next point** (`P`, the ramp included, so a giant's heart past your nature is worth less of a point: nature resists), not banded (a corpse is paid for), through `train_stat_times`' sure path. Validated 1 to 100.
- `widen = { strength = 10 }`: a **10 % chance the potential rises by one** (one gameplay draw per stat, as the old raise was; never past 40). Validated 1 to 100. No `on_eat` names both for the same stat at more than 100 and 50: a heart cannot be a whole point and a sure widening.

The order inside `eat_fresh` stays: message, vomit, then `train` and `widen` per stat in display order, heal, harm, feed. A rotten corpse does none of it (as today). `corpse_stats_percent` (Cauldron, Gourmand: +100) scales both the share and the chance (doubling them), as it scaled the raise.

### 8.2 Defaults by tag

The roster has 273 types, 161 with corpses; writing `train` on each is the churn the tag list exists to avoid. `[corpses.by_tag]` in `data/hunger.toml`: a type's corpse takes the **largest** `train` share and `widen` chance per stat over its tags' entries, then its own `on_eat` overrides per stat (an `on_eat` with `train = { toughness = 0 }` switches a tag's gift off). The wiki table lists the resolved values per creature (`corpse-effects.md`, regenerated), so the vagueness is in the game only.

| Tag | train (share of the price) | widen (chance, %) | Why |
|---|---|---|---|
| giant | strength 40 | strength 10 | the cloud giant's Strength 3 today; a giant's heart is the fairy-tale meal |
| ogre | strength 25, toughness 10 | strength 5 | |
| troll | toughness 40 | toughness 10 | trolls regrow; their flesh is the Toughness meal |
| hag | willpower 30 | willpower 5 | the bheur hag's Willpower 5 today |
| caster, cultist | intelligence 15, willpower 10 | — | a mind eaten; the duergar mindbreaker's Willpower 3 today |
| fey | presence 20 | presence 5 | glamour |
| devil, yugoloth, fiend | willpower 15 | — | with the harm they already carry |
| orc, hobgoblin, goblin, kobold, duergar, drow | toughness 5 | — | plain meat; the drow's bitterness and the duergar's sourness stay in their `on_eat` |
| vermin | toughness 3 | — | the beetle's "Crunchy" Toughness 10 → toughness 10 by hand |
| marsh-thing | toughness 10 | — | |
| undead, spirit, skeleton, construct, elemental, animated | — | — | harm, no corpse, or no flesh: unchanged |
| beasts (no tag; by hand) | toughness 10 (the dire warg, the bear), strength 10 (the bear) | — | the dire warg's Toughness 2 today |

Every hand-set `stat` in the 41 files is carried over by one rule, reviewed in the slice: `stat = { x = c }` → `train = { x = 4 × c }` (Toughness 5 → 20 %, Strength 3 → 12 %, the beetle's Toughness 10 → 40 %), and `widen` only where the table above says. A corpse is thus worth a fifth of a point for the common gifted ones and two fifths for a giant's, instead of one in twenty of a whole point: more often something, never a jackpot that outruns the price, and a career of giants pushes the potential, which is the one thing marks cannot do.

### 8.3 Interplay

- **Rot** (`docs/dev/hunger.md`): old corpses are eaten as fresh and give the gifts; rotten ones none, and sicken. Unchanged.
- **Harm, vomit**: unchanged, and `corpse_harmless` (Herb lore, Cauldron) spares the harm as today; a vomited corpse gives no marks (it comes back up before the gifts, as today).
- **Alignment** (`ate_corpse`): eating kin or a peaceful talker shifts the score as today; the marks come all the same (hunger is hunger). No Presence deed on top (§7.1).
- **Hunger**: a corpse feeds by size as today; the marks do not depend on how hungry the eater was.
- **Cannibalism for marks**: sapient tags give small Toughness shares only, and the giants' and trolls' gifts are not kin to any race; a drow eating drow gains a drow's 5 % and an alignment hit.

### 8.4 Words

The gifts are never shown before eating (the owner's rule, `docs/dev/hunger.md`). After: a `train` share of 25 % or more says one line by stat, "Something of its strength passes into you." (nimbleness, hardiness, cunning, resolve, bearing); under 25 % nothing (the mark is quiet, as marks are); a `widen` that landed says "You feel you could grow stronger yet." (the elixir's clause, reused). The old "You feel tougher!" on a corpse goes: a corpse no longer raises a score directly.

## 9. Talents and class abilities

No new `[[system]]` tag: losses are a channel of `stat_training`, corpse marks of `hunger` (and `stat_training`, since they pay into the pile). The `stat_training` system keeps its two talents and gains more; the coverage bar is met before and after.

| Talent | Tree, needs | Effect | Hook |
|---|---|---|---|
| **Hard training** (exists) | vigour | re-pointed: the band's full-odds share is **30** a day instead of 15 (`TalentMods::band_full`) | `banded` in `train_stat_times`; its `daily_factor` effect goes with the factor |
| **Quick study of the body** (exists) | scholar | unchanged: a potion's or elixir's share doubled (`potion_marks_percent`) | as today |
| **Hard-won** (exists, heritage, creation) | | `share = 50`: half the price of each point a story line took, as marks on that stat (replaces `marks = 3`) | `World::story_stat` |
| **Gifted** (new) | choice: a stat; no needs | the chosen stat's price is 15 % lower (`TalentMods::price_percent(stat)`) | `StatTraining::price` through a talent hook in `settle` |
| **Late bloomer** (new) | level 20 | the ramp counts every point past the potential as one fewer (the first beyond nature at ×1) | the same hook, `ramp_ease` |
| **Unshakeable** (new) | grit; Willpower 12 | deeds take half (`loss_percent = −50`), and the warning comes a day earlier (at −L/8) | `train_stat_times`' deed path |
| **Ascetic** (new) | grit; Willpower 13 | a dawn spent Hungry or worse gives Willpower 10 % of its price instead of the hungry dawn's loss; Weak or worse still costs Strength and Toughness, halved | `daily_stat_check` |
| **Hard living** (barbarian class ability, granted at 18) | class | no `hungry_dawn`, `starved`, or `vomited` deed ever | the deed path reads `TalentMods::no_deed(on)` |
| **Discipline** (monk class ability, granted at 12) | class | `endured` and `prayed` give two marks; `fainted` and `broke` take nothing | the same |
| **Gourmand**, **Cauldron** (exist, hunger) | | retagged `stat_training` too; `corpse_stats_percent` scales `train` and `widen` | §8.1 |

Each is one `TalentMods` field read in one place; none adds state or an action. The abilities menu is untouched (nothing here is active).

## 10. What the player sees; the wiki, the spec, the docs

**In the game** (words only; no mark, price, chance, or potential is ever shown):

| Moment | Line |
|---|---|
| a rise (a level or 06:00) | "You feel stronger." (quicker, tougher, sharper, more resolute, more confident), as today; stops a rest |
| a stat entering the drop zone (−L/4 first crossed) | "You feel your strength wasting." / "Your hands feel less sure." / "You feel your health failing." / "Your mind feels dull." / "Your will feels brittle." / "You feel small and unregarded." A `say`: stops a rest. Said once per crossing (a flag is not needed: the crossing is the moment the pile goes from above −L/4 to at or below it) |
| a drop | "You feel weaker." / "You feel clumsier." / "You feel frailer." / "You feel duller." / "Your resolve wavers." / "You feel less sure of yourself." Stops a rest |
| a corpse's gift of 25 % or more | "Something of its strength passes into you." (§8.4); a widening "You feel you could grow stronger yet." |
| the character screen | the scores as today (effective, lowered ones marked); nothing new |

**Dev dumps**: the `progress` layer's line becomes `stat marks St 120/160 (today 15), To -30/75, Pr 12/120`: the pile, the price in force (the drop price for a negative pile), and today's count where it is not zero. JSON dumps gain nothing (dev only). `actors` unchanged.

**The wiki** (`wiki/src/stats.md` "Training" rewritten; the Food page's corpse table; the Potions page's marks words; tables from `xtask/src/wiki/stats.rs` and `hunger.rs`):

| Table | Generated from |
|---|---|
| `stat-training.md` | the uses and chances (as today, every new use described) |
| `stat-prices.md` | §4.2's table from the `[training]` keys |
| `stat-odds.md` | §4.3's odds by share of the price (the cube, with the quarter rule) |
| `stat-pace.md` | days to the next point by marks a day (a deterministic expectation computed from the odds, not a simulation) |
| `stat-losses.md` | the deeds, what each takes, the drop's price by score |
| `stat-band.md` | the band's steps and a worked day |
| `corpse-effects.md` | the resolved `train` and `widen` per creature (replacing the raise chances) |
| `stat-checks.md`, `stat-daily-checks.md` | removed (one formula now; `stat-odds.md` is it) |

**The spec**: `spec/08f-stats-and-burden.md` "Training" rewritten (the pile, the price, the odds, the band, the deeds, the drop, the potential's new rule); `spec/08h-hunger.md` "Eating a corpse" (train and widen); `spec/08g-magic-items.md` "Potions" (marks as a share). **The docs**: `docs/dev/stats.md` "Training" (the new modules and hooks, one line each), `docs/dev/hunger.md` (`OnEat`), `docs/dev/potions.md`, `docs/dev/background.md` (Hard-won), `docs/dev/progression.md` (the talents), `docs/balance.md` (the probe's stats method, §13 T6).

## 11. Determinism, saves, replays

- Draws: marks on the training stream, one per rule per use as today (the band scales the compare); deeds the same; the checks six draws each on their streams whatever the piles (§4.6). Corpse `widen` on the gameplay stream where the old raise drew (one draw per stat named, so a corpse with the same stats named draws as before; a corpse that gained a `widen` from a tag draws once more: the showcase replays that eat such a corpse are re-made and listed in the PR).
- Saves: §7.4. Potentials stored as today; a save made with a potential lifted by a past trained rise keeps it (nothing is taken away).
- Replays: a run's hashes change wherever marks were earned (the band, the signed pile, the price) and wherever a check fired; the PR re-creates the repository's replays (`replay --create`, then `--verify`) and says so. The content fingerprint is untouched (no indexed table gains an id; `StatUse` is not in it).
- Old content: `data/stats.toml` without `[training]`'s new keys loads with the defaults named here; a `[[training.use]]` with a `stat` for `drank` is refused as today; `on_eat.stat` is refused with a message naming `train` (the data pass of T4 rewrites every file).

## 12. Choices made along the way, and what is rejected

- **Odds, not a ladder**, by the owner's word (§4.7); the ladder is one data value away.
- **The cube.** Power 2 favours trickles too much (a point for 40 % of the price at a mark a day); power 4 is a threshold with a blur. Three gives 50 % at the price, 11 % at half, 2 % at a quarter, 89 % at twice.
- **The quarter rule** rather than odds all the way down: it is what makes a single bad day safe and a warning possible before any drop, and it costs a lucky point from a tenth of the effort, which is the point.
- **The band, not decay, not a cap** (§4.7).
- **A price in marks instead of a need in a chance**: the same data knobs (base, per point) with a meaning the wiki can state as "a point at 14 is about 160 marks".
- **The potential fixed under training**: the ramp needs a fixed origin; elixirs and corpses are what move nature.
- **Drop price half the base, unramped**: losing easier than winning; nature's surplus goes first.
- **Deeds are things done**, not luck suffered: no mark lost to a hidden trap, a blow, a disease caught.
- **No Presence deed for kin-eating or murder**: alignment is that system.
- **Corpses never raise a score directly**: the owner's "perhaps not directly ability points"; a widened potential is the rarer, better gift.
- **Shares of the price for every sure amount** (potions, elixirs, corpses, Hard-won, deeds): the one scale that means the same at every score.
- **No new `[[system]]`**: nothing here is a new rule set a talent could not already name; the coverage policy's letter is met by tagging.

## 13. Implementation plan

Six issues in four waves; each links to its section. "**State**" marks a PR that adds saved state; at most two State PRs run beside each other. Hot files: `data/stats.toml`, `content/stat_training.rs`, `world/stat_training.rs`, `TalentMods`, `talents.toml`, `hunger.toml`, the creature files, `xtask/src/wiki/mod.rs`; the false-conflict trick (`docs/dev/merging.md`) applies.

| Wave | Issue | What | Adds | Depends on |
|---|---|---|---|---|
| 1 | **T1 The pile and the price** (§4, §7.4, §10's dump and tables) | `stat_marks: PerStat<i32>`, `marks_today`; `[training]` keys `price_base`, `price_per_point`, `ramp_divisor`, `odds_power`, `quarter` (the deadzone divisor, 4), `band_full`, `drop_share` (the old `need_*`, `over_potential_divisor`, `daily_factor` removed); `StatTraining::price`, `drop_price`, `odds`; `World::settle` for both checks (the level's on `levels_rng`, the daily on `training`, six draws each); a rise pays; `Stats::train` leaves the potential; `Stats::untrain` (the drop, with its floors, used by T3 only); the band in `train_stat_times`; potions, elixirs, Hard-won as shares (`hard_won { share }`); Hard training re-pointed; the `progress` line; `stat-prices`, `stat-odds`, `stat-pace`, `stat-band` tables, the two old tables removed; the spec bullet; `docs/dev/stats.md`; the hash test; the replays re-made | **State** | — |
| 2 | **T2 The paths** (§5) | the new `StatUse` variants with `describe()`, one hook line each (the `spell` hook reading `SpellScales`; `Use::Kick` in `chest.rs`, `Use::Pick`, `Disarm`, `Steal`, `Appraise`, `Search`, `Forage`, `Travel`, `Temper`, `Deal` joined to stats through `StatUse::of_skill` where the skills already report), `data/stats.toml` entries by stat, `hurt` 15; `stat-training.md` regenerated; `stat_training_tests` per hook | none | T1 |
| 2 | **T3 The deeds and the drop** (§7) | `[[training.deed]]` (`stat`, `on`, `chance` or `share`), `StatDeed` with its words, the hooks of §7.1, the drop in `settle` with `Stats::untrain`, the warning line and the drop lines, `idle_days` if Q6 is approved, `stat-losses.md`, the spec, the tests of §7.3 (the warning precedes, noise cancels, the round trip's price) | **State** only with Q6 (`idle_days`); else none | T1 |
| 2 | **T4 Corpses** (§8) | `OnEat::train`/`widen` replacing `stat`, `[corpses.by_tag]`, the data pass over the 41 files and the tag table, `corpse_stats_percent` re-pointed, the words, `corpse-effects.md`, the spec and `docs/dev/hunger.md`, `eat_tests`; the replays that eat a corpse re-made | none (content) | T1 |
| 3 | **T5 Talents and class abilities** (§9) | Gifted (a `choice` of stat), Late bloomer, Unshakeable, Ascetic; Hard living and Discipline in the class tables; Gourmand and Cauldron retagged; `TalentMods` fields read at their one hook each; `talent_tests`; the wiki's talent rows | none | T1, T3 |
| 4 | **T6 The probe and the docs** (§6, §10) | the balance probe's stat method on the real hooks (marks from the simulated fights, not a fixed table), `docs/balance.md`'s stats section with before/after by class, the wiki's prose, a cross-check of the spec against what landed; the owner's pick of `ramp_divisor` from the measured rows | none | T1 to T5 |

**Why this order**: T1 first because everything reads the pile and the price. T2, T3, and T4 beside each other: T2 lives in hooks across `world/`, T3 in `daily_stat_check` and a few failure arms, T4 in `eat.rs` and content; they share `stats.toml` (uses in one table, deeds in another) and `docs/dev/stats.md`, the false-conflict trick's territory. T5 after T3 because its talents read the deed path. T6 last. Each issue answers the feature template's "Talents and class abilities": T1 (Hard training, Quick study, Hard-won re-pointed), T2 (none new: paths extend the system), T3 (Unshakeable, Ascetic, Hard living, Discipline), T4 (Gourmand, Cauldron retagged), T5 (the rest), T6 (none).

## 14. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| Q1 | **Odds or a ladder?** The cube (50 % at the price) or a plain threshold (`odds_power` → ∞) | **the cube, power 3** (§4.3): the owner asked for odds; the ladder is one data value away |
| Q2 | **The ramp past the potential**: ÷4 (the first point beyond at ×1.5, the sixth at ×11.5; the warrior's Strength +11 by 60) or ÷2 (+9) or ÷6 (+12) | **÷4**; T6 measures with the real hooks and the owner picks the row |
| Q3 | **A rise pays its price** (`m −= P`, floor 0) and a lucky early rise spends what is there: or should a short pile go into debt (negative) so that luck is repaid? | **floor at zero**: debt would put a lucky stat into the drop zone, the oscillation §4.1 G5 forbids |
| Q4 | **One move per stat per check**, the surplus waiting? | **yes**: a flood is a point a day plus one a level; nothing is lost |
| Q5 | **The potential no longer lifts with a trained rise**? | **yes** (§4.4): the ramp needs its origin; elixirs and corpses move nature |
| Q6 | **The idle deed** (a stat moved by nothing for 7 days loses 5 % of L a dawn), which needs `idle_days: PerStat<u8>` saved? | **yes, but at 7 days and St/Dx/To only**: it is the owner's "long inactivity", it never fires on a character who plays, and it is small; drop it if the owner finds the state not worth it |
| Q7 | **Deeds share the daily band** with the uses (one counter)? | **yes**: one terrible day cannot carry a stat into the drop zone (§7.3) |
| Q8 | **The drop price is half the base, unramped** (losing easier than winning; surplus past nature goes first)? | **yes** (§4.2) |
| Q9 | **A drop leaves the potential alone** and is not a drained point (no temple mending)? | **yes** (§7.2) |
| Q10 | **The warning line** at −L/4 (the first moment a drop is possible)? | **yes**, and Unshakeable earlier; a drop with no warning first is a feel-bad |
| Q11 | **Corpses: shares and a potential chance, never a direct point**; the `stat` field replaced by `train` and `widen`; tag defaults | **yes** (§8); the hand-set values carried over ×4 |
| Q12 | **The `spell` use** trains the stats a spell's `scales` name (damage In, reach Wi, duration Pr, to-hit Dx) at 15–20 %, beside `cast`'s Willpower for the mana? | **yes**: one hook, and a caster's Intelligence finally has a path |
| Q13 | **`hurt` 20 → 15 %** (the wizard's Toughness +10 in §6.2)? | **yes**; T6 re-measures |
| Q14 | **Potions', elixirs', and Hard-won's amounts as shares of the price** (the data's `marks` 5 and 20 read as percent; `hard_won.share` 50)? | **yes**: the same meaning at every score |
| Q15 | **Hard training re-pointed to the band** (30 full-odds marks a day) rather than a price cut? | **yes**: "you drill every day" is about a day's effort counting |
| Q16 | **Early pace**: the first rise about a week in (level 7–9), against level 2–3 today? | **keep**; `price_base` 20 → 0 if the owner wants the first points quicker, without touching the tail |
| Q17 | **No new `[[system]]` tag** for losses or corpse marks? | **none**: channels of `stat_training` and `hunger`; the talents of §9 carry both tags |
| Q18 | **The checks' six draws stay** whatever the piles (the streams in step)? | **yes**: `docs/dev/stats.md`'s invariant; a stat under the quarter rule draws and ignores |
