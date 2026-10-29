# Warlock — Stat training: a signed pile of marks, a price per point, growth and loss by deed

Design document for the owner's review (2026-10-29), revised with the owner's decisions the same day (§14). Written against `origin/main` at 65d99485 (Background B4, #717), then against 13b76ebc (v0.9.0; Hard-won at 12 marks a point, #719). Nothing here is implemented; §13 cuts it into issues. It rebuilds what is there: the training marks and their two checks (`docs/dev/stats.md` "Training", `content/stat_training.rs`, `world/stat_training.rs`, `data/stats.toml`; #231, #562), the potions' and elixirs' marks (`docs/dev/potions.md`), the corpses' stat raises (`docs/dev/hunger.md`, `OnEat::stat`), Hard-won (`docs/dev/background.md`), stat damage and drains (`docs/dev/stats.md` "Drain", `docs/dev/status-effects.md`), hunger and burden (`docs/dev/hunger.md`, `docs/dev/burden.md`), the skills' mark hooks (`docs/dev/skills.md`), and the balance probe (`docs/balance.md`). Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-10-29, verbatim): *"stat training and stat marks need overhauling. Stat training needs to be plugged into systems we have currently implemented (like low chance to gain some mark per action, idk). We need training paths for each stat. Gaining marks should not be too difficult, but not completely free either. While at it, let's implement a way to lose marks when doing something that should theoretically be punishing. Marks going enough to negative could just reduce a stat (and reset marks to 0). We could attach stat training & potential increases (% chance) to eating certain types of corpses (perhaps not directly ability points)."*

**Addenda** (the same day, verbatim): *"stat ups should not actually consume all marks. that is pretty feelbad if player stacks a lot of marks, and then all are gone just for 1 stat point"*; *"need better maths, design that too"*; *"the agent can reconsider if the formula is the best, and make it work with the mark consumption. Make sure it makes sense with both - and + marks"*.

**The decisions** (the same day, on the first draft's questions): marks are a plain signed sum, inspected now and then, with no daily band and no idle loss ("marks are marks … do not make it too complicated"); stats grow moderately liberally, rises may be quick but never on a schedule; losses are few and deliberate ("we rather see stats go up than down … stat lowering is in the game so players can manipulate the stats"); a rise pays its price and the surplus waits; the potential moves only by other means than training. §14 has them verbatim and what each changed.

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
14. Decisions (owner, 2026-10-29)

## 1. Summary

- **One signed pile per stat** (`Actor::stat_marks: PerStat<i32>`, today a `u16`), a plain cumulative sum. Good deeds add marks, a few bad ones take them; zero is neutral. Positive marks make a **rise** likelier, negative marks a **drop**. Nothing else is counted: no daily band, no idle days, no decay, no cap (§4.5). Saves change (the integrator's per-release bump; `FORMAT_VERSION` untouched by this work).
- **A point has a price**, `P = (20 + 10 × score) × ramp`, where the ramp is 1 up to the potential and grows past it (×1.5, ×2.5, ×4, ×6, … for the first, second, third, fourth point beyond). A drop has a price too, half the base price of the point lost, `L = (20 + 10 × (score − 1)) / 2`, never ramped: points past your nature are the first to go and the hardest to win back.
- **Odds that pay the price**: at every check (06:00 and each level, the same formula), a stat with `m` marks rises with chance `m³ / (m³ + P³)` and the rise **costs `P` marks, never the whole pile**: a pile of 400 marks against a price of 160 buys a point and keeps 240 toward the next (§4.3). A drop is the mirror: `|m|³ / (|m|³ + L³)` and `L` marks paid back toward zero. Nothing moves while the pile is under a quarter of its price, either way. All integers (§4.6). The odds, not a threshold, decide (decision 1): two characters who did the same do not rise on the same dawn.
- **Why the new maths**: today's `marks / (marks + need)` is insensitive to effort and resets the pile. With the kill curve as it stands (146 kills a level at 10, 365 at 60) a fighter earns 150 to 900 Strength marks a level, not the 8 to 15 a day the design assumed; today's rule turns that into a point nearly every day until the cap, while a stat trickling 5 marks a level still rises every two weeks. A simulation of the rules on the curve (§3.3, §6) puts a level-60 warrior at **+100 points in all today** (Strength and Dexterity at 40); the proposal puts it at **+51**, two thirds of them in the stats the warrior lives by, and a wizard at **+66** over its two minds and the body that takes the blows.
- **Gaining marks is neither hard nor free**: more uses (every stat has at least five paths across the systems the game has, §5), each a chance per event. What bounds a flood of marks is the **price**, not a counter: the ramp past the potential is quadratic, so the career's 29 000 Strength marks buy twelve points and the thirteenth would cost 8 000 more (§4.5). Nothing is ever locked (the owner's no-anti-farming rule; time is the only price).
- **Losing** (§7): a **small set of deliberate deeds** take marks: a dawn met Weak or worse, hit points starved away, vomiting, fainting, acting while Overloaded, a known trap stepped on, being caught stealing. Nothing suffered by luck alone takes a mark. Far enough below zero the stat **drops by one** (never below 3, the drain floor; **the potential stays**; not a drained point, so a temple does not mend it) and the pile pays the drop's price back toward zero. The player is warned in words when a stat enters the zone where a drop is possible ("You feel your strength wasting."). Rises dominate by construction: a fighter's Strength earns 70 to 220 marks a day, and the only way to lose it is to starve or overload oneself for days, which is the roguelike staple of lowering a stat on purpose (decision 8).
- **Marks are marks** (decision 14): every sure amount is a plain number of marks, the same unit the uses pay in. A potion of might gives 15, an elixir 60, Hard-won 60 a point the story took, a corpse what its data says (a giant's heart 60 Strength marks, a kobold 8 Toughness). **Corpses give marks, not points** (§8): `on_eat.train = { strength = 60 }` is sure marks; `on_eat.widen = { strength = 5 }` a 5 % chance the **potential** rises by one. Defaults by creature tag cover the roster (giants Strength, trolls Toughness, casters and hags the mind, fey Presence); hand-set values stay. Rot, harm, vomiting, and kin-eating are unchanged.
- **The potential is nature** (decision 5): a trained rise no longer lifts it; only elixirs, corpses (rare), and whatever later systems the owner adds widen it; a drop never lowers it (§4.4).
- **Talents** (§9): Hard training re-pointed (every use gives a mark half again as often), two new talents on the price (Gifted: a chosen stat 15 % cheaper; Late bloomer: the ramp a step gentler), one on a loss (Ascetic: a hungry dawn trains Willpower instead of wasting Strength), two class abilities (the barbarian's Hard living, the monk's Discipline). No new `[[system]]` tag: losses and corpse marks are channels of `stat_training` and `hunger`.
- **Six slices** (§13): the pile and the price (the one PR with saved state) first; then the paths, the deeds, and the corpses beside each other; then the talents; then the probe and the docs.

## 2. The owner's request, and what exists today

What the code does now, verified line by line (`docs/dev/stats.md` "Training" is current):

| Piece | Today |
|---|---|
| The pile | `Actor::stat_marks: PerStat<u16>`, saturating, never decays, left out of the JSON while all zero; saved and hashed. Only the player trains, alive. |
| Earning | 33 uses (`StatUse`), each `[[training.use]]` in `data/stats.toml` with a `chance` (1 to 100): one draw of the **training** stream per rule below 100. Five are the skills' own reports (`World::train` serves both: burdened, kick, dodge, sneak, sneak_attack, spot, study). A potion's `marks` (`drank`, `elixir`) are sure, as many as it says (potion of might 5, elixir 20; Quick study of the body doubles them). Hard-won gives **12** marks per stat point a story line took (`hard_won.toml`: `marks = 12` since #719; 3 before, which gave about 6 % at the first level check). |
| The level check | `World::check_stats` from `level_up`: six **levels**-stream draws always; a stat rises with `marks / (marks + need)`, `need = 20 + 3 × score`; at or above the potential the denominator is ×4; a rise sets the marks to **0**, lifts the potential with the score (`Stats::train`), and says "You feel stronger." |
| The daily check | `World::daily_stat_check` at 06:00 (`WorldState::next_stat_check`), the same formula with the need ×`daily_factor` (3; Hard training 2), six **training**-stream draws always; a rise says the same at once and stops a rest. |
| Potentials | the race's `base_potential` 15 plus adjustments, never below the start; raised by a trained rise past it and by elixirs (`Stats::widen`); corpses raise a score up to it, never past. |
| Corpses | `OnEat::stat = { toughness = 5 }`: a 5 % gameplay draw to raise the score by one, up to the potential. 273 creature types; 145 have an `on_eat`, **41** name a stat (Toughness 5 ×11, Strength 3 ×11, Willpower 3 ×5, …), 112 leave no corpse. `corpse_stats_percent` (Cauldron, Gourmand: +100) doubles the chance. |
| Losing | nothing lowers marks. Points are lost for good only to drains (`Actor::drained`, mended at a temple) and for a while to stat damage and status penalties. |
| Streams | marks on `WorldState::training`; the level check on `levels_rng`; the daily check on `training`; corpses and elixirs on the gameplay stream. |
| Shown | nothing but "You feel stronger." and the new score; the dev `progress` layer lists the marks; the wiki has `stat-training.md`, `stat-checks.md`, `stat-daily-checks.md`. |

One thing the earlier designs assumed is not so: the "8 to 15 marks a game day in a trained stat" (combat progression §9.1) was arithmetic over 30 blows a day, while the curve asks for a hundred times that (§3).

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

(The wizard's other stats are derived: Toughness half the warrior's Strength volume (it is hit more), Dexterity 25 a level, Strength 4, Presence 5. A Monte Carlo of the rules as written over this activity model, 200 careers, run for this document and not committed: T6 moves the measurement into the balance probe, on the real hooks. The probe today feeds the rules 7 Strength marks a day; the curve feeds them 70 to 220.)

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
| Today | **40** (+26) | 40 (+28) | 38 (+25) | 13 (+3) | 17 (+7) | 20 (+10) | **+100** | **40** (+27) | 40 (+30) | 20 (+10) | **+101** |

Strength and Dexterity hit the cap of 40 before level 60 (Toughness nearly), and the three stats the warrior never uses rise 3 to 10 points on trickles. ADOM's trained characters gain about 25 to 35 points in all over a long game (the progression design §6.4 named that as the aim; the owner now asks for moderately more, decision 2). The maths, not the chances, is the fault: lowering every chance by ten would only move the warrior's three stats toward the trickle regime and leave the twentyfold price gap.

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
| G2 | **A rise never wipes the pile**: it pays a price; the surplus carries. | the owner's addendum; decisions 3 and 4 |
| G3 | **Odds grow with marks without saturating early**: small piles are nearly worthless, the price is the 50 % mark, twice the price is near-certain. | fault 1 of §3.2; a lucky point from a tenth of the effort cheapens effort |
| G4 | **Nature slows growth smoothly**, no cliff at the potential; a long career outgrows it slowly, never never. | the progression design's intent; the ×4 cliff |
| G5 | **One scale for both signs**: negative marks mirror positive ones, with a drop that pays its own price back; zero is neutral; no cheap oscillation. | the owner's third addendum |
| G6 | **The daily and level checks are the same rule**: a second check moves the moment, not the total. | the ×3 factor was a patch over fault 1 |
| G7 | **A moderately liberal pace by level 60** (decision 2): a main stat +12 to +17, a secondary +8 to +13, an unused stat +2 to +5; +45 to +70 in all; no stat at 40 by use alone. Quick is fine; a schedule is not (decision 16). | the owner |
| G8 | **Integers only, on the existing streams**, with the draws in step whatever the marks. | `docs/dev/stats.md`'s invariants; replays |
| G9 | **Nothing but the pile is counted** (decision 7): no daily counter, no idle days, no decay. The pile is a signed sum the checks read. | "marks are marks … do not make it too complicated" |

### 4.2 The price of a point

`base(score) = price_base + price_per_point × score` = **20 + 10 × score** (data). Past the potential the price ramps: with `over = score + 1 − potential` (the point you are about to gain, counted from the potential: 1 for the first beyond it),

`P(score, potential) = base(score) × (ramp_divisor + over × (over + 1)) / ramp_divisor`, `ramp_divisor` 4, for `over ≥ 1`; `base(score)` otherwise.

So the first point past your nature costs ×1.5, the second ×2.5, the third ×4, the fourth ×6, the fifth ×8.5, the sixth ×11.5: quadratic, so a career of two hundred days ends nine to twelve points past its nature in the stat it lives by and never reaches 40 by use alone (§6.2; the sweep there has ÷2 to ÷8, and ÷4 is the pick, decision 2). The point *reaching* the potential costs the plain base: the ramp begins where nature ends, and there is no step at it beyond the first ×1.5.

**The drop's price** is `L(score) = base(score − 1) / drop_share`, `drop_share` 2: half the base price of the point being lost, **never ramped** (decision 8). Losing is easier than winning (muscle wastes faster than it builds), a point past nature is lost at the same price as any other but regained at its ramped price, so neglecting a stat you pushed past nature is costly: it falls back toward the potential first.

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

Why linear in the score below the potential: the modifier is linear in the score (`floor((s − 10) / 2)`), so each point is worth the same to the rules; a steeper base would only slow the high-born. Why the start is 20 + 10 × score and not today's 20 + 3 × score: today's need is a *chance* parameter; this is a *price* in marks, set against what the curve pays (§3.1: 70 to 220 marks a day in a stat you live by at mid levels, 15 to 30 in the early levels), so a price of 160 is about a week of living by a stat at level 5 and a day or two at level 20, where the ramp has long taken over (§4.5, §6).

### 4.3 Odds at a check, and what a rise or a drop leaves behind

At a check, with `m` marks and the price `P` (for `m > 0`) or `L` (for `m < 0`):

- **Nothing happens while `4 × |m| < price`** (the quarter rule): the stat is not in play.
- Otherwise the stat moves with chance **`|m|³ / (|m|³ + price³)`**: a rise if `m > 0`, a drop if `m < 0`. The odds decide, never a threshold (decision 1).
- **A rise pays `P`**: `m ← max(0, m − P)`. **A drop pays `L` back**: `m ← min(0, m + L)`. If the pile was short of the price (a lucky early move), it goes to zero: what was there is spent, and no more; no debt (decision 3).
- **At most one move per stat per check.** The surplus waits for the next chance (decision 4): a flood of marks (an elixir on a full pile) turns into a point a day plus one a level. There is no other cap.

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

(Today's columns put the proposal's price of 160 marks against today's need of 62, to compare at equal marks.) The cube is the S-curve of G3: a tenth of the price is nothing, the price is even, twice it is sure enough. The power is data (`odds_power`, 3): 2 is a softer curve (20 % at half the price) and favours trickles, 4 is nearer a threshold; the table below is why 3.

**G1 checked**: a stat checked daily rises when its pile first wins a check, so what it pays depends on how fast it fills. Simulated at a price of 160 (400 runs each):

| Marks a day | Days to the point | Marks it cost | Share of the price | Today: days (daily check) | Today: marks it cost |
|---|---|---|---|---|---|
| 1 | 63 | 63 | 0.39 | 17 | 17 |
| 2 | 36 | 71 | 0.44 | 13 | 26 |
| 4 | 21 | 83 | 0.52 | 9 | 35 |
| 8 | 12.5 | 100 | 0.62 | 6 | 50 |
| 15 | 8 | 119 | 0.74 | 5 | 72 |
| 25 | 5.5 | 129 | 0.80 | 4 | 97 |
| 40 | 4 | 139 | 0.87 | 3 | 125 |
| 80 | 2.5 | 151 | 0.94 | 2.4 | 188 |
| 160 | 1.6 | 160 | 1.00 | 1.8 | 292 |
| 320 | 1.1 | 160 | 1.00 | 1.5 | 470 |

A trickle still gets a discount (a mark a day pays four tenths of the price, because the daily check gets sixty tries at low odds), but the price gap across a 160-fold income range is **2.6×**, against today's **17×** in the trickle's favour (17 marks against 292) *and* with today's trickle reaching the point three times faster than the proposal's. Past the price a point costs exactly the price: the flood's extra marks stay in the pile for the next. With `odds_power` 4 the gap would be 1.8× and the curve a near-threshold; with 2 it is 4.5×. Three is the middle.

### 4.4 The potential

- A trained rise **no longer lifts the potential** (`Stats::train` changes: the score only; decision 5). The potential is nature, set at the start; it moves only by **elixirs** (`Stats::widen`, as today), **corpses** (§8, rarely), and whatever the owner adds later, and by gifts (a story's, a class's: `Stats::gift` moves both, as today).
- The ramp is measured from it, so the sixth point past nature costs ×11.5 and the twelfth ×40: a stat does not reach 40 by use alone in two hundred days (§6.2), while an elixir or a giant's heart (potential +1) makes the next point one step cheaper for every point after. That is what a potential is for.
- A drop never lowers the potential (§7.2, decision 9): nature is not undone by a bad month.

### 4.5 Earning: the pile is the whole state

A use adds a mark with its chance, every time, however many times a day (decision 7). There is no daily band, no counter of marks today, no idle count, no decay, no cap: `Actor::stat_marks` is the only training state, and the first draft's band and idle-days counter are gone (§14, decisions 6 and 7). What keeps a flood from buying a point a day is the price:

| A career's marks (the model, §3.1 with §5's paths) | Buys at the price, from the start to level 60 | The next point would cost |
|---|---|---|
| warrior Strength, 14/17: ~29 000 | 14 → 26 (+12; cumulative 23 200: 160, 170, 180 to the potential, then 285, 500, 840, 1 320, 1 955, 2 760, 3 750, 4 940, 6 345) | 7 980 |
| wizard Willpower, 13/16: ~38 000 | 13 → 27 (+14; cumulative 31 300) | 10 220 |
| warrior Presence, 10/15: ~470 | 10 → 13 (+3; 120, 130, 140) | 150 |

(The daily check's discount, §4.3: a pile that fills over several days buys its point for 75 to 90 % of the price, which is the two or three more points §6.2 measures.) So a day of a thousand blows is worth ten times a day of a hundred, exactly, and both are honest: the quadratic ramp means the tenth point past nature costs as much as the first nine together. The owner's rule holds in its simplest form: nothing is capped, nothing decays, the thousandth blow has the same chance as the first, and the only price of grinding is the clock. Sure marks (a potion's, an elixir's, a corpse's, a story's) are plain marks into the same pile.

### 4.6 Integers and streams

- Prices: `u32` arithmetic in `StatTraining::price(score, potential)` and `drop_price(score)`.
- Odds: `m` clamped to `8 × price` (99.8 % there; above it the chance is treated as that), then `num = m³`, `den = m³ + price³` in `u64` (`(8 × 4255)³ ≈ 3.9 × 10¹³` fits). `Rng::below` takes a `u32`, so the odds are reduced to a 1/65 536 scale: `scaled = num × 65 536 / den` (u64), one `below(65 536)` draw, a move if `draw < scaled`. Deterministic, no floats, exact to a part in 65 536.
- The streams stay what they are: marks on `WorldState::training` (one draw per rule per use, as today), the level check on `levels_rng`, the daily check on `training`. **Six draws per check always**, whatever the piles (so a fight's rolls never move and the stream's position stays a function of the days and levels passed; decision 18): a stat under the quarter rule still draws and ignores the result.
- Sure marks (`Drank`, `Elixir`, a corpse's `train`, Hard-won) draw nothing (as today at 100).
- A drop's hit points: `keep_hp_within_max`, never below 1; mana likewise (`drain.rs` does the same).

### 4.7 Alternatives, and why not

| Alternative | Why not |
|---|---|
| **Today's hyperbola with a price paid instead of a reset** (`p = m / (m + need)`, then `m −= need`) | keeps fault 1 (21 % at a tenth: cheap lucky points) and fault 2 (the trickle's discount is 4× not 2.6×, and it reaches the point faster than a flood per mark); the check frequency still sets the pace more than the marks do |
| **A plain ladder** (rise when `m ≥ P`, pay `P`; no odds) | G1 exactly (a point costs `P` whatever the income) and simplest; rejected by the owner (decision 1): the odds decide, so two characters doing the same things do not rise on the same dawn (decision 16). It is the cube with `odds_power → ∞`; the data knob is there |
| **A daily band on a stat's marks** (the first draft: the first 15 marks a day at full odds, then half, a quarter, an eighth) | rejected by the owner (decision 7): a second counter per stat, saved and hashed, for a bound the ramp already gives; "marks are marks" |
| **Marks decay** (a share a day) | punishes the player who stops using a stat (travel, a shop day), which is the anti-farming shape the owner rules out |
| **An idle loss** (a stat moved by nothing for a week loses marks) | rejected by the owner (decision 6): not asked for, and more state |
| **A per-use chance to raise the stat directly** (ADOM; no pile) | no pile to pay into (potions, corpses, stories), no negative side, and a direct raise per blow at one in a thousand is a flood stat's point every two days at level 30 |
| **A logistic or exponential odds curve** | floats, or a table; the cube is the same shape with one multiply |
| **Daily factor ×3 kept** | a patch over fault 1; with a price paid, a second check per day only moves the day of the rise, and the trickle discount already comes from the daily check |
| **The potential lifted with a trained rise** (today) | makes the ramp a one-time tax (§3.2); nature should slow every further point (decision 5) |
| **A hard cap on the pile** | the owner's feel-bad in another coat: effort thrown away |
| **Signed marks with the drop at a fixed −N** | a fixed threshold ignores the score; the drop's price is half the point's base, so a high score is as hard to lose as it was to gain, in proportion |
| **Sure amounts as shares of the price** (the first draft: a potion 5 % of the next point, a corpse 20 %) | rejected by the owner (decision 14): one unit, the mark; a potion is worth more to a low score than a high one, which is how potions have always worked |

## 5. Training paths: every stat, every system

The rule for a path: one existing hook (a function that already knows the event), one line, one `StatUse` variant with its `describe()` words, one `[[training.use]]` entry. Chances are first guesses against the price (§4.2): a frequent use (hundreds a day) 5 to 15 %, an occasional one (tens) 20 to 40 %, a rare one (a few) 100 %. Existing uses keep their chances unless noted; T6's probe measures and the data is the dial.

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
| hurt | a blow that costs you hit points | 15 (from 20: a wizard's Toughness ran away in §6; decision 13) | today |
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
| **spell** | a spell you know cast (not a class or sign ability): the stat its `scales` damage names, else Intelligence | 20 | `cast.rs::cast_spell`, the one cast path (decision 12) |
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

| Stat's place in the build | Marks a day | Points by level 60 | Of them past the potential |
|---|---|---|---|
| lived by (a fighter's Strength, a caster's Willpower and Intelligence) | 15 at level 5, 70 to 220 past level 10 | +12 to +17 | 9 to 14 |
| secondary (the fighter's Dexterity and Toughness, the caster's Toughness) | 5 to 80 | +8 to +13 | 5 to 10 |
| touched (Presence for anyone who talks and trades) | 2 to 4 | +4 to +6 | 0 to 1 |
| unused (a warrior's Intelligence, with §5's paths) | 1 to 2 | +2 to +5 | 0 |
| all six | | **+45 to +70** | |

Moderately liberal (decision 2): about half of today's +100, double the first draft's +38, and no stat at 40 by use alone. The rises come when the odds say, never on a schedule (decision 16): with 100 marks a day against a price of 285 a point comes in two to five days, and a check at the price is a coin toss.

### 6.2 The simulation

The same activity model as §3 (the curve's kills, the probe's rounds and hit chances, 189 days), with §5's paths at their first-guess yields (the wizard's `spell` use on its casts at 20 %, `aimed` on half of them at 15 %; `hurt` 15; the other new paths 3 to 10 marks a level by stat), the proposal's rules (price 20 + 10 × score, ramp ÷4, cube odds, the quarter rule, a rise paying its price, both checks, no band), 200 careers each. Mean scores at level 60:

| Character | Stat | Start/potential | Today | **Proposed** |
|---|---|---|---|---|
| warrior | St | 14/17 | 40 (+26) | **28.8 (+14.8)** |
| | Dx | 12/15 | 40 (+28) | 23.6 (+11.6) |
| | To | 13/16 | 38 (+25) | 21.6 (+8.6) |
| | In | 10/15 | 13 (+3) | 14.7 (+4.7) |
| | Wi | 10/15 | 17 (+7) | 15.1 (+5.1) |
| | Pr | 10/15 | 20 (+10) | 15.9 (+5.9) |
| | all six | | +100 | **+50.7** |
| wizard | St | 9/13 | 19 (+10) | 14.1 (+5.1) |
| | Dx | 11/15 | 30 (+19) | 23.8 (+12.8) |
| | To | 10/13 | 40 (+30) | 22.2 (+12.2) |
| | In | 15/18 | 20 (+5) | 29.2 (+14.2) |
| | Wi | 13/16 | 40 (+27) | **29.0 (+16.0)** |
| | Pr | 10/15 | 20 (+10) | 15.9 (+5.9) |
| | all six | | +101 | **+66.2** |

The wizard gains more than the warrior because it lives by two stats (the `spell` use makes Intelligence a flood beside Willpower's `cast`) and is hit in six rounds of ten (its Toughness, +12 even with `hurt` at 15 %). The warrior's Dexterity rides on the dodge attribution (30 % of monster misses); the probe should measure it.

**The knobs** (the sweep, 100 careers each; the price is `price_base + price_per_point × score`, the ramp `ramp_divisor`):

| Price | Ramp ÷ | W St | W Dx | W To | W In | W Wi | W Pr | W all | Z Wi | Z In | Z To | Z all |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 20 + 10s | 2 | +12.5 | +10.0 | +7.4 | +4.7 | +5.0 | +5.8 | +45 | +13.5 | +12.1 | +10.3 | +58 |
| **20 + 10s** | **4** | **+14.7** | **+11.7** | **+8.7** | **+4.7** | **+5.1** | **+5.9** | **+51** | **+16.1** | **+14.1** | **+12.3** | **+66** |
| 20 + 10s | 6 | +16.3 | +12.7 | +9.4 | +4.6 | +5.2 | +5.9 | +54 | +17.7 | +15.6 | +13.4 | +72 |
| 20 + 10s | 8 | +17.5 | +13.6 | +10.0 | +4.6 | +5.1 | +6.1 | +57 | +18.9 | +16.8 | +14.3 | +77 |
| 20 + 15s | 4 | +13.4 | +10.6 | +7.9 | +3.4 | +4.1 | +4.9 | +44 | +14.6 | +12.9 | +11.1 | +60 |
| 20 + 20s | 4 | +12.4 | +9.8 | +7.1 | +2.8 | +3.3 | +4.1 | +40 | +13.6 | +12.0 | +10.4 | +55 |

The ramp sets the main stats' tail (two points of Strength per step of the divisor); the price per point sets the trickle stats (the unused ones lose a point per 5 of it) and the first rise's day. ÷4 at 10 a point is the pick (decision 2: moderately liberal; the writer's call): the warrior's Strength at 29 and the wizard's Willpower at 29 by level 60, nothing at 40, the unused stats +3 to +6. The probe (§13, T6) re-measures with the real hooks; both knobs are data.

### 6.3 Early game

Below the potential a point costs 160 marks at 14; a fighter earning 15 to 30 Strength marks a day at levels 4 to 7 meets the first rise about **day 5 or 6** (level 6 on the model's days; today's rules give it at level 2 or 3), the second and third within the next week or two as the curve's volume climbs, and is at its potential around level 10, where the ramp takes over: the fourth point (×1.5, 285 marks) a few days later, the fifth (500) a week after that, and from there each point takes longer than the one before while the character keeps getting faster at earning. Quick early, slowing with nature, never on a date (decision 16). The wizard's Willpower runs the same course a level or two later (46 marks a level at 5, 200 at 10).

## 7. Losing marks and losing points

### 7.1 Deeds

Few, and deliberate (decision 8): the game is not a death by a thousand cuts, and losing a stat is here so that the player *can* lower one, the roguelike staple. The rule for a deed: one existing hook that already knows the event, one `[[training.deed]]` entry: `stat`, `on`, and either `chance` (a mark taken, one draw, like a use) or `marks` (that many taken at once). Deeds are things the player did or let happen, never bad luck alone: a hidden trap sprung, a blow taken, a disease caught or a check lost, a drain that lands take nothing.

| Stat | Deed (`on`) | Takes | Hook |
|---|---|---|---|
| Strength, Toughness | **hungry_dawn**: the 06:00 check made while Weak, Fainting, or Starving | 15 marks each | `daily_stat_check`, reading `player_hunger`, before the check's draw |
| Toughness | **starved**: a hit point lost to starvation | chance 50 | `hunger.rs::digest`, the starving branch |
| Toughness | **vomited**: a vomit (overeating, rotten food, a corpse that disagreed) | 10 marks | `eat.rs::vomit` |
| Willpower | **fainted**: a faint | 10 marks | `hunger.rs::faint` |
| Dexterity | **overloaded**: an action, not a wait or a drop, while Overloaded | chance 10 | `World::apply` beside `burdened` (`Burden::Overloaded`) |
| Intelligence | **knew_better**: a trap you knew of sprung under you | 20 marks | `trap.rs::spring_trap`, the `known` branch for the player |
| Presence | **caught**: caught stealing | 20 marks | `shop.rs::caught_stealing` |

Seven deeds, every stat with one lever: to lower Strength or Toughness, starve; Dexterity, carry too much; Intelligence, walk onto traps you know; Presence, get caught; Willpower, faint. Each is a choice the player makes with open eyes and keeps making for days (§7.3), so a player who plays normally never sees a drop.

Not deeds, and why: **a disease check lost, a drain check failed, Strained** (the first draft had them; luck and ordinary play, struck for decision 8); **a shop debt turned Unwelcome** (the shop already pays it back); **fleeing** (the game has no flee action and the Coward stance is a legitimate choice; a stance never costs marks); **being Frightened** (only creatures are, by Hound's bay); **killing peaceful folk, eating kin** (alignment already pays, and a Presence mark on top would make the evil character's Presence a second alignment: not twice); **resting long, leaving a stat alone** (rest heals and trains Willpower; no idle loss, decision 6); **wearing heavy armour** (armour has `max_dex` and weight, both already paid for in Defence and burden; Overloaded is the "far too heavy" case).

### 7.2 The drop

At a check, a stat whose pile is at or below −L/4 is in play: it drops by one with `|m|³ / (|m|³ + L³)` and the pile pays L back toward zero (§4.3). Floors: never below **3** (`DRAIN_FLOOR`, the one floor for lost points), never below the stat damage on it plus one (a score carries its damage, as drains require). Hit points and mana are cut to the new maxima, never below 1: a drop never kills.

**The potential stays** (decision 9). A drop is neglect, not a change of nature, so the lost point is won back at the plain base price (below the potential) and the player who starved for a week is set back a week or two of use, not a month. A point *past* the potential that is lost is regained at its ramped price (§4.2): neglect above nature costs more, which is as it should be.

**Not a drained point.** `Actor::drained` counts drains only; a temple's mending does not restore a trained point lost. Stat damage mends as before and is untouched by marks; the effective score the rules read is the stored score (lowered by a drop) less damage and penalties, as today.

### 7.3 Numbers, and the oscillation check (G5)

- **A warning comes first** (decision 10). When a deed takes a stat's pile from above −L/4 to at or below it (the zone where a drop is possible), the player is told in words (§10): "You feel your strength wasting." The deed lands before the check's draw, so the warning always precedes the drop, if only by a moment. Simulated at score 14 (L 75): a character Weak at every dawn (−15 Strength and −15 Toughness a day) hears the warning on day 2 and loses the point on day 4.5 on average (never before day 2, at most day 10); the pile is left at −5 on average, never below −L (`settle` adds L back). Zero drops came before the warning in 400 runs.
- **Noise cancels.** A stat gaining 20 marks one day and losing 20 the next for 200 days: 0 rises and 0 drops in 200 careers. Zero is neutral, and the quarter rule keeps a pile that hovers near it out of play.
- **No cheap round trip.** Losing a point at 14 costs 75 marks of deeds (five dawns Weak, or some 750 actions while Overloaded); winning it back costs 150 marks of use. The round trip nets nothing and costs 225 marks: there is nothing to farm. A player who wants a stat down (to fit a race's cap, to stay under a threshold, to play a weakling: the staple) can have it in a few days of deliberate misery, and rises dominate for everyone else.

### 7.4 Saved state

`Actor::stat_marks` becomes `PerStat<i32>` (the JSON map is read as before; the `u16` default of zero is the same zero): **saves change**, and nothing else is added: no counter of marks today, no idle days (decisions 6 and 7). The field stays left out of the JSON while all zero, so a run that never trained hashes as today. `FORMAT_VERSION` is the integrator's per-release bump and this work does not touch it.

## 8. Corpses: marks and potential, not points

### 8.1 The data

`OnEat::stat` (a chance per stat to raise the score by one, up to the potential) is replaced by two fields (decision 11):

- `train = { toughness = 30 }`: that many **sure marks** on the stat (marks are marks, decision 14), through `train_stat_times`' sure path. Validated 1 to 200.
- `widen = { strength = 5 }`: a **5 % chance the potential rises by one** (one gameplay draw per stat, as the old raise was; never past 40). Validated 1 to 20: potential points are rare (decision 11). No `on_eat` names a `widen` without a `train` on the same stat.

The order inside `eat_fresh` stays: message, vomit, then `train` and `widen` per stat in display order, heal, harm, feed. A rotten corpse does none of it (as today). `corpse_stats_percent` (Cauldron, Gourmand: +100) scales both the marks and the chance (doubling them), as it scaled the raise.

### 8.2 Defaults by tag

The roster has 273 types, 161 with corpses; writing `train` on each is the churn the tag list exists to avoid. `[corpses.by_tag]` in `data/hunger.toml`: a type's corpse takes the **largest** `train` and `widen` per stat over its tags' entries, then its own `on_eat` overrides per stat (an `on_eat` with `train = { toughness = 0 }` switches a tag's gift off). The wiki table lists the resolved values per creature (`corpse-effects.md`, regenerated), so the vagueness is in the game only.

| Tag | train (marks) | widen (chance, %) | Why |
|---|---|---|---|
| giant | strength 60 | strength 5 | the cloud giant's Strength 3 today; a giant's heart is the fairy-tale meal |
| ogre | strength 40, toughness 15 | strength 2 | |
| troll | toughness 60 | toughness 5 | trolls regrow; their flesh is the Toughness meal |
| hag | willpower 40 | willpower 3 | the bheur hag's Willpower 5 today |
| caster, cultist | intelligence 20, willpower 15 | — | a mind eaten; the duergar mindbreaker's Willpower 3 today |
| fey | presence 30 | presence 3 | glamour |
| devil, yugoloth, fiend | willpower 20 | — | with the harm they already carry |
| orc, hobgoblin, goblin, kobold, duergar, drow | toughness 8 | — | plain meat; the drow's bitterness and the duergar's sourness stay in their `on_eat` |
| vermin | toughness 4 | — | the beetle's "Crunchy" Toughness 10 → toughness 60 by hand |
| marsh-thing | toughness 15 | — | |
| undead, spirit, skeleton, construct, elemental, animated | — | — | harm, no corpse, or no flesh: unchanged |
| beasts (no tag; by hand) | toughness 15 (the dire warg, the bear), strength 15 (the bear) | — | the dire warg's Toughness 2 today |

Every hand-set `stat` in the 41 files is carried over by one rule, reviewed in the slice: `stat = { x = c }` → `train = { x = 6 × c }` (Toughness 5 → 30, Strength 3 → 18, the beetle's Toughness 10 → 60), and `widen` only where the table above says. Against a price of 160 at 14, a common gifted corpse is thus about a fifth of a point and a giant's heart two fifths, instead of one in twenty of a whole point: more often something, never a jackpot that outruns the price, and a career of giants pushes the potential a few times, which is the one thing marks cannot do (§4.4).

### 8.3 Interplay

- **Rot** (`docs/dev/hunger.md`): old corpses are eaten as fresh and give the gifts; rotten ones none, and sicken. Unchanged.
- **Harm, vomit**: unchanged, and `corpse_harmless` (Herb lore, Cauldron) spares the harm as today; a vomited corpse gives no marks (it comes back up before the gifts, as today) and costs the `vomited` deed's 10 Toughness marks (§7.1).
- **Alignment** (`ate_corpse`): eating kin or a peaceful talker shifts the score as today; the marks come all the same (hunger is hunger). No Presence deed on top (§7.1).
- **Hunger**: a corpse feeds by size as today; the marks do not depend on how hungry the eater was.
- **Cannibalism for marks**: sapient tags give small Toughness marks only, and the giants' and trolls' gifts are not kin to any race; a drow eating drow gains a drow's 8 and an alignment hit.

### 8.4 Words

The gifts are never shown before eating (the owner's rule, `docs/dev/hunger.md`). After: a `train` of 30 marks or more says one line by stat, "Something of its strength passes into you." (nimbleness, hardiness, cunning, resolve, bearing); under 30 nothing (the mark is quiet, as marks are); a `widen` that landed says "You feel you could grow stronger yet." (the elixir's clause, reused). The old "You feel tougher!" on a corpse goes: a corpse no longer raises a score directly.

## 9. Talents and class abilities

No new `[[system]]` tag (decision 17): losses are a channel of `stat_training`, corpse marks of `hunger` (and `stat_training`, since they pay into the pile). The `stat_training` system keeps its three talents and gains more; the coverage bar is met before and after.

| Talent | Tree, needs | Effect | Hook |
|---|---|---|---|
| **Hard training** (exists) | vigour | re-pointed (decision 15: no band to point it at): every use's chance is half again (`TalentMods::train_percent` +50; a use at 100 stays sure), so marks come half again as fast in every stat; its `daily_factor` effect goes with the factor | the chance compare in `train_stat_times` |
| **Quick study of the body** (exists) | scholar | unchanged: a potion's or elixir's marks doubled (`potion_marks_percent`) | as today |
| **Hard-won** (exists, heritage, creation) | | `marks = 60` for each point a story line took (from 12, #719: 12 marks are under the quarter rule at every price; 60 are 7 % at the first check against a price of 140 and two fifths of the way to the point) | `World::story_stat`, as today |
| **Gifted** (new) | choice: a stat; no needs | the chosen stat's price is 15 % lower (`TalentMods::price_percent(stat)`) | `StatTraining::price` through a talent hook in `settle` |
| **Late bloomer** (new) | level 20 | the ramp counts every point past the potential as one fewer (the first beyond nature at ×1) | the same hook, `ramp_ease` |
| **Ascetic** (new) | grit; Willpower 13 | a dawn met Hungry or worse gives Willpower 15 marks, and the `hungry_dawn` deed takes nothing: the fast is a discipline | `daily_stat_check`, the deed's one place |
| **Hard living** (barbarian class ability, granted at 18) | class | no `hungry_dawn`, `starved`, `vomited`, or `fainted` deed ever | the deed path reads `TalentMods::no_deed(on)` |
| **Discipline** (monk class ability, granted at 12) | class | `endured` and `prayed` give two marks; `fainted` takes nothing | the same, and the use's count |
| **Gourmand**, **Cauldron** (exist, hunger) | | retagged `stat_training` too; `corpse_stats_percent` scales `train` and `widen` | §8.1 |

Each is one `TalentMods` field read in one place; none adds state or an action. The abilities menu is untouched (nothing here is active). The first draft's Unshakeable (deeds take half) is dropped: with deeds few and deliberate (§7.1) a talent that softens them would only slow the player who wants the drop.

Potions and elixirs, the other sure sources, are re-pointed in the same unit (decision 14): a potion's `marks` 5 → **15**, an elixir's 20 → **60** (the elixir's main gift stays the widening), so a potion of wit is a tenth of a fighter's next Intelligence point and an elixir a third of a point at mid scores.

## 10. What the player sees; the wiki, the spec, the docs

**In the game** (words only; no mark, price, chance, or potential is ever shown):

| Moment | Line |
|---|---|
| a rise (a level or 06:00) | "You feel stronger." (quicker, tougher, sharper, more resolute, more confident), as today; stops a rest |
| a stat entering the drop zone (−L/4 first crossed) | "You feel your strength wasting." / "Your hands feel less sure." / "You feel your health failing." / "Your mind feels dull." / "Your will feels brittle." / "You feel small and unregarded." A `say`: stops a rest. Said once per crossing (a flag is not needed: the crossing is the moment the pile goes from above −L/4 to at or below it) |
| a drop | "You feel weaker." / "You feel clumsier." / "You feel frailer." / "You feel duller." / "Your resolve wavers." / "You feel less sure of yourself." Stops a rest |
| a corpse's gift of 30 marks or more | "Something of its strength passes into you." (§8.4); a widening "You feel you could grow stronger yet." |
| the character screen | the scores as today (effective, lowered ones marked); nothing new |

**Dev dumps**: the `progress` layer's line becomes `stat marks St 120/160, To -30/75, Pr 12/120`: the pile and the price in force (the drop price for a negative pile). JSON dumps gain nothing (dev only). `actors` unchanged.

**The wiki** (`wiki/src/stats.md` "Training" rewritten; the Food page's corpse table; the Potions page's marks; tables from `xtask/src/wiki/stats.rs` and `hunger.rs`):

| Table | Generated from |
|---|---|
| `stat-training.md` | the uses and chances (as today, every new use described) |
| `stat-prices.md` | §4.2's table from the `[training]` keys |
| `stat-odds.md` | §4.3's odds by share of the price (the cube, with the quarter rule) |
| `stat-pace.md` | days to the next point by marks a day (a deterministic expectation computed from the odds, not a simulation) |
| `stat-losses.md` | the deeds, what each takes, the drop's price by score |
| `corpse-effects.md` | the resolved `train` and `widen` per creature (replacing the raise chances) |
| `stat-checks.md`, `stat-daily-checks.md` | removed (one formula now; `stat-odds.md` is it) |

**The spec**: `spec/08f-stats-and-burden.md` "Training" rewritten (the pile, the price, the odds, the deeds, the drop, the potential's new rule); `spec/08h-hunger.md` "Eating a corpse" (train and widen); `spec/08g-magic-items.md` "Potions" (the marks); `spec/08i-character.md` (Hard-won's marks). **The docs**: `docs/dev/stats.md` "Training" (the new modules and hooks, one line each), `docs/dev/hunger.md` (`OnEat`), `docs/dev/potions.md`, `docs/dev/background.md` (Hard-won), `docs/dev/progression.md` (the talents), `docs/balance.md` (the probe's stats method, §13 T6).

## 11. Determinism, saves, replays

- Draws: marks on the training stream, one per rule per use as today; deeds with a `chance` the same, deeds with `marks` none; the checks six draws each on their streams whatever the piles (§4.6). Corpse `widen` on the gameplay stream where the old raise drew (one draw per stat named, so a corpse with the same stats named draws as before; a corpse that gained a `widen` from a tag draws once more: the showcase replays that eat such a corpse are re-made and listed in the PR).
- Saves: §7.4. Potentials stored as today; a save made with a potential lifted by a past trained rise keeps it (nothing is taken away).
- Replays: a run's hashes change wherever marks were earned (the signed pile, the price) and wherever a check fired; the PR re-creates the repository's replays (`replay --create`, then `--verify`) and says so. The content fingerprint is untouched (no indexed table gains an id; `StatUse` is not in it).
- Old content: `data/stats.toml` without `[training]`'s new keys loads with the defaults named here; a `[[training.use]]` with a `stat` for `drank` is refused as today; `on_eat.stat` is refused with a message naming `train` (the data pass of T4 rewrites every file).

## 12. Choices made along the way, and what is rejected

- **Odds, not a ladder**, by the owner's word (decision 1; §4.7); the ladder is one data value away.
- **The cube.** Power 2 favours trickles too much (a point for 40 % of the price at a mark a day); power 4 is a threshold with a blur. Three gives 50 % at the price, 11 % at half, 2 % at a quarter, 89 % at twice.
- **The quarter rule** rather than odds all the way down: one compare, and it is what makes the warning line exact (no drop before it) and costs a lucky point from a tenth of the effort, which is the point. It is part of the odds, not a counter: the pile is still the only state (G9).
- **No band, no decay, no cap, no idle loss** (decisions 6 and 7; §4.5, §4.7): the ramp bounds a flood by itself.
- **A price in marks instead of a need in a chance**: the same data knobs (base, per point) with a meaning the wiki can state as "a point at 14 is about 160 marks".
- **The potential fixed under training** (decision 5): the ramp needs a fixed origin; elixirs and corpses (and later systems) are what move nature.
- **Drop price half the base, unramped** (decision 8): losing easier than winning; nature's surplus goes first.
- **Deeds are few, and things done** (decision 8): no mark lost to a hidden trap, a blow, a disease, a failed check; seven deeds, one lever per stat.
- **No Presence deed for kin-eating or murder**: alignment is that system.
- **Corpses never raise a score directly**: the owner's "perhaps not directly ability points"; a widened potential is the rarer, better gift, and rare (decision 11).
- **Marks are marks** (decision 14): every sure amount (potions, elixirs, corpses, Hard-won, deeds) is a count of marks, the uses' unit; no shares of a price.
- **No new `[[system]]`** (decision 17): nothing here is a new rule set a talent could not already name; the coverage policy's letter is met by tagging.

## 13. Implementation plan

Six issues in four waves; each links to its section. "**State**" marks a PR that adds or changes saved state; only T1 does. Hot files: `data/stats.toml`, `content/stat_training.rs`, `world/stat_training.rs`, `TalentMods`, `talents.toml`, `hunger.toml`, the creature files, `xtask/src/wiki/mod.rs`; the false-conflict trick (`docs/dev/merging.md`) applies.

| Wave | Issue | What | Adds | Depends on |
|---|---|---|---|---|
| 1 | **T1 The pile and the price** (§4, §7.4, §10's dump and tables) | `stat_marks: PerStat<i32>`; `[training]` keys `price_base`, `price_per_point`, `ramp_divisor`, `odds_power`, `quarter` (the deadzone divisor, 4), `drop_share` (the old `need_*`, `over_potential_divisor`, `daily_factor` removed); `StatTraining::price`, `drop_price`, `odds`; `World::settle` for both checks (the level's on `levels_rng`, the daily on `training`, six draws each); a rise pays; `Stats::train` leaves the potential; `Stats::untrain` (the drop, with its floors, used by T3 only); potions 15, elixirs 60, Hard-won 60 (`hard_won { marks }` as today, the value); Hard training re-pointed to `train_percent`; the `progress` line; `stat-prices`, `stat-odds`, `stat-pace` tables, the two old tables removed; the spec bullets; `docs/dev/stats.md`; the hash test; the replays re-made | **State** | — |
| 2 | **T2 The paths** (§5) | the new `StatUse` variants with `describe()`, one hook line each (the `spell` hook reading `SpellScales`; `Use::Kick` in `chest.rs`, `Use::Pick`, `Disarm`, `Steal`, `Appraise`, `Search`, `Forage`, `Travel`, `Temper`, `Deal` joined to stats through `StatUse::of_skill` where the skills already report), `data/stats.toml` entries by stat, `hurt` 15; `stat-training.md` regenerated; `stat_training_tests` per hook | none | T1 |
| 2 | **T3 The deeds and the drop** (§7) | `[[training.deed]]` (`stat`, `on`, `chance` or `marks`), `StatDeed` with its words, the seven hooks of §7.1, the drop in `settle` with `Stats::untrain`, the warning line and the drop lines, `stat-losses.md`, the spec, the tests of §7.3 (the warning precedes, noise cancels, the round trip's price) | none | T1 |
| 2 | **T4 Corpses** (§8) | `OnEat::train`/`widen` replacing `stat`, `[corpses.by_tag]`, the data pass over the 41 files and the tag table, `corpse_stats_percent` re-pointed, the words, `corpse-effects.md`, the spec and `docs/dev/hunger.md`, `eat_tests`; the replays that eat a corpse re-made | none (content) | T1 |
| 3 | **T5 Talents and class abilities** (§9) | Gifted (a `choice` of stat), Late bloomer, Ascetic; Hard living and Discipline in the class tables; Gourmand and Cauldron retagged; `TalentMods` fields read at their one hook each; `talent_tests`; the wiki's talent rows | none | T1, T3 |
| 4 | **T6 The probe and the docs** (§6, §10) | the balance probe's stat method on the real hooks (marks from the simulated fights, not a fixed table), `docs/balance.md`'s stats section with before/after by class, the wiki's prose, a cross-check of the spec against what landed; `ramp_divisor` and `price_per_point` re-read against the measured rows (§6.2) | none | T1 to T5 |

**Why this order**: T1 first because everything reads the pile and the price. T2, T3, and T4 beside each other: T2 lives in hooks across `world/`, T3 in `daily_stat_check` and six hooks in hunger, burden, traps, and the shop, T4 in `eat.rs` and content; they share `stats.toml` (uses in one table, deeds in another) and `docs/dev/stats.md`, the false-conflict trick's territory. T5 after T3 because its talents read the deed path. T6 last. Each issue answers the feature template's "Talents and class abilities": T1 (Hard training, Quick study, Hard-won re-pointed), T2 (none new: paths extend the system), T3 (Ascetic, Hard living, Discipline, built in T5), T4 (Gourmand, Cauldron retagged), T5 (the rest), T6 (none).

## 14. Decisions (owner, 2026-10-29)

The owner's answers to the first draft's open questions, verbatim, with the reading taken where an answer needed one. The body above is revised to them; the first draft's daily band (`marks_today`), its idle deed (`idle_days`), its shares of the price, and five of its deeds are gone.

| # | Question | Answer | Taken as |
|---|---|---|---|
| 1 | Odds or a ladder? | "odds" | the cube decides a rise and a drop (§4.3), never a threshold; `odds_power` stays data; the quarter rule stays as the odds' floor, not a counter (§12) |
| 2 | The ramp past the potential: ÷2, ÷4, or ÷6? | "you decide. we want stats to grow moderately liberally" | **÷4 at 10 marks a point**, the writer's call from the re-run sweep without the band (§6.2): a warrior +51 and a wizard +66 by level 60, main stats at 29, nothing at 40; G7 rewritten to it (§4.1). T6 re-reads it on the real hooks |
| 3 | A rise pays its price (floor 0), or a short pile goes into debt? | "pays its price" | `m ← max(0, m − P)`; a lucky early rise spends what is there, no debt (§4.3) |
| 4 | One move per stat per check, the surplus waiting? | "surplus marks wait for next chance" | kept (§4.3): a point a day plus one a level at most, nothing lost |
| 5 | The potential no longer lifts with a trained rise? | "potential needs to raise using some other method that stat training (elixirs, eating corpses or something else later)" | kept and said so (§4.4): elixirs, corpses (rare), and later systems; never a trained rise, never a drop |
| 6 | The idle deed (a stat unused for a week loses marks; `idle_days` saved)? | "i said nothing about long inactivity. perhaps some brief agent did it? Remove this." | **removed**: no idle deed, no `idle_days`, no state for it (§7.1, §7.4, §13); listed under the rejected alternatives (§4.7) |
| 7 | Deeds share the daily band with the uses? | "what? marks are marks, there is no daily protection or anything. it's just a cumulative sum that gets inspected every now and then. do not make it too complicated" | **the band is removed entirely** (no `marks_today`, no `band_full`, no band table or wiki page); the pile is a plain signed sum, read at the checks (G9, §4.5); the ramp alone bounds a flood |
| 8 | The drop price half the base, unramped? | "we rather see stats go up than down. misery comes from permadeath, not from death by thousand cuts. stat lowering is in the game so players can manipulate the stats - it's staple in roguelikes." | kept (§4.2), and the deeds **cut to seven deliberate ones** (§7.1): a disease check lost, a drain check failed, Strained, Unwelcome, and idle struck; rises dominate by construction (§1, §7.3) |
| 9 | A drop leaves the potential alone and is not a drained point? | "drop does nothing to potential" | kept (§7.2) |
| 10 | The warning line at −L/4? | "sure, you warning about potential drops and raises are good flavour" | kept (§7.3, §10); it always precedes a drop |
| 11 | Corpses: marks and a potential chance, never a direct point; tag defaults? | "sure, or at least potential points need to be rare" | kept, with `widen` **capped at 20 % in the data and 2 to 5 % in the tags** (§8.1, §8.2), and the marks as plain counts (decision 14) |
| 12 | The `spell` use trains the stats a spell's `scales` name? | "sure" | kept (§5.4 to §5.6) |
| 13 | `hurt` 20 → 15 %? | "sure" | kept (§5.3); T6 re-measures |
| 14 | Potions', elixirs', and Hard-won's amounts as shares of the price? | "marks are marks. all stat training that is not just +n to stats is done with marks." | **no shares**: every sure source gives a plain count of marks, the uses' unit; a potion 15, an elixir 60, Hard-won 60 a point (from 12, #719), a corpse its `train` count, a deed its `marks` (§1, §4.7, §8.1, §9); `widen` is the one direct +1, on the potential |
| 15 | Hard training re-pointed to the band? | "fuck bands. this is a needles complication?" | the band is gone (decision 7); Hard training becomes **every use's chance half again** (`train_percent` +50; §9) |
| 16 | Early pace: the first rise about a week in? | "the raises can be quite rapid, i do not mind. as long as they are not guaranteed on schedule" | the first rise about day 5 or 6 with no band (§6.3), the next within the fortnight, then the ramp; the odds, never a threshold, set the day (decision 1) |
| 17 | No new `[[system]]` tag? | "ok" | kept (§9) |
| 18 | The checks' six draws stay whatever the piles? | "sure" | kept (§4.6) |

Also folded in: `main` moved Hard-won from 3 to 12 marks a point while the questions were out (#719, v0.9.0), so §2 reads 12 and §9 re-points it from there; the simulation was re-run without the band and with §5's paths in the model, and every pace number in §1, §3.3, §4.3, §6, and §7.3 is from that run.
