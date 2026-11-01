# Combat balance: the player against the monsters of their level

The balance probe measures how a fight between the player and the monsters of their level goes,
level by level. It reads every number through the game's own code. It was first run, and not
kept, for the balance report of 2026-10-27 (the combat progression design,
`docs/design/combat-progression.md` §2). It is kept now as a test (#569, the design's §15 D1)
so that the first-guess numbers in the data can be judged again after every tuning PR. It was
first kept at #569, after every slice of that design had landed: the new rank tables (W1),
weapon abilities and stamina (W2 to W4, SP1), spell scales (S1), monster Protection by body (P1),
stat training and the daily check (T1, T2), hit points to level 50 (H1), and potions and elixirs
(Q1 to Q3). #727 (stat training T6) made it measure trained stats through the game's training
hooks and added the "Stat training" section. The tables below are its output on `main` at the
time of #754 (the spell scaling design's S7, after every other slice of that design but the
Magic page, a screen): the ladders, the school skills, the caster styles, and the school
abilities, with the casters' school ranks trained by their casting ("School pace") instead of
set by level. The sections after them ("Against the design's predictions" and "Tuning notes")
were written against #569's run, and their figures are that run's, except the spell paragraph,
which is #754's.

## How to run it

```text
cargo test --release -p warlock-core balance_probe -- --ignored --nocapture
```

It takes about three minutes in release (most of it the stat training's sweep, a hundred and
seventy careers to level 60). It prints the Markdown tables of the sections from "Players" to
"Sneak attacks" below, and those of "Stat training", and the same content always prints the same tables (fixed seeds and streams).
After a PR that tunes combat data (weapon skills, creatures, classes, spells, stats, items), run
it again and replace those sections, then revisit the tuning notes. The code is
`crates/warlock-core/src/world/balance_probe.rs` (the measuring, with the method in its module
docs), `balance_probe/training.rs` (the trained careers' days), `balance_probe/schools.rs`
(the casters' school pace), and `balance_probe/tables.rs` (the tables).

## Method

- **Monsters**: every creature type with a level, shared and campaign, 400 in all. Each is made
  by `travel::make_creature`, once bare and once for each of six seeds with the gear its type
  rolls at its own level. Its side of a fight is `World::target_of` (Defence, Protection,
  resistances), its blow is `Actor::melee`, and its blows a round are its speed over its
  `World::attack_price`.
- **Players**: a human male warrior (long sword, sword and board, shield block), thief (dagger,
  twin blades), wizard (staff; evocation), and priest (mace, sword and board, shield block;
  restoration), six seeds each, and a witch (dagger; necromancy) measured for the spell tables
  alone. Each is made by `World::start_run` with the kit and levelled one level at a time with
  `xp:` and `levelup:auto`, which takes the first talent it may, not one chosen for a build.
  Weapon skill ranks are set by level, because ranks come from use: rank 2 at level 1, 4 at 5, 6
  at 10, 7 at 15, 8 at 20, 9 at 30, and 10 from 40. The wizard's staff and the witch's dagger
  are two ranks lower. A school and a caster style are not set by level: they are trained by
  casting ("School pace").
- **Trained stats** (#727; "Stat training" below has the method and the results): before each
  level come its game days, and each day its share of the level's kills is fought blow by blow
  through the game's training hooks, then the day's other uses, then the daily check at 06:00
  (`World::daily_stat_check`); the level's own check follows. **Untrained** players get no
  marks.
- **Gear**: the kit; *plain* level-appropriate pieces (the warrior goes from chain mail to banded
  mail at 15 and plate at 30, with a small shield, then a large one from 15); or *magic*, the
  same pieces made magic from level 3 and rare from 30, with an amulet and two rings from 15.
  Magic is a generous ceiling: every slot is magic.
- **A fight**: a round is one normal turn. Each side's damage a round is its blows a round times
  the mean of 300 `combat::strike`s against the other's target (the off hand adds its blow). The
  hit chance is `combat::hit_ways`, exactly. Rounds to kill are the monster's hit points over the
  player's damage a round; rounds to die are the player's hit points over the monster's damage a
  round. Each cell is the median over the monsters within two levels of the player, and each
  player seed is paired with one gear roll of each monster.
- **Spells** (the spell scaling design §12.4, S7): the trained caster's own roll of every spell
  that rolls (harm or healing) of the schools they know or have an affinity with,
  `Actor::spell_roll` in plain gear from where they stand: the rung of its ladder their stats
  reach, their school's and caster style's ranks, the talents' and items' hooks; nothing holds
  the roll (decision 16). Beside it the same roll with the school and the style at rank 0 (the
  ladder alone), and the mean of a roll counts its rerolls exactly. No casts to kill: what a
  spell is worth against a monster is the content's question; the probe's is the roll and the
  pool.
- **School pace**: before each level, each of its days the caster spends one pool on the
  spell of their school they cast most (of the most mean roll among those costing at most a
  fifth of the pool; none cast only under the open sky; the priest's a heal, at most one a
  kill), and every cast lands on a monster of their level through `World::practise` (the
  school, then the caster style it is cast in): the real `[marks]` and curve. The ranks it
  reaches are the ones the spell tables and the trained careers' fights cast at.
- **Sneak attacks** (#688): the thief's main-hand blow on a monster that has not seen them,
  against the same blow on one that has. The foe is flat-footed (`World::flat_footed`: nothing
  of its Dexterity or Dodge), the dice are `World::sneak_dice` (the Sneak attack skill and the
  thief's talents, Improved sneak attack from 32) added to each hit, Master thief's threat is 2
  sooner, and a monster with no vitals takes no dice and no critical. Sneak attack rises by use,
  and `levelup:auto` spends the level-up rolls on another skill, so the probe sets it by level:
  12 plus 3.5 a level (15 at 1, 117 at 30, 222 at 60), the design's "about 100 by level 25 to
  30". Only the first blow is measured: the initiative check's Surprised, which strips the
  Defence for the next blow too, is not.
- **Not modelled**: triggered and active weapon abilities (Rend, Riposte, Sunder, ...), stances,
  sneak attacks outside the table above, a talent build, potions, resistances against spells, healing between rounds,
  and fights against several monsters at once.

## Players

Each cell: to-hit / damage a hit / Dv / Pv, the seeds' mean; trained stats.

| L | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|
| 1 | +3/8.2/16/3 | +3/8.2/16/3 | +0/3.2/13/1 | +0/3.2/13/1 | +0/3.0/12/0 | +1/6.0/12/0 | +1/6.0/12/0 |
| 5 | +4/9.2/18/4 | +5/10.9/20/4 | +2/4.7/15/1 | +2/5.7/17/2 | +0/3.0/13/0 | +2/7.2/15/2 | +3/7.9/18/2 |
| 10 | +7/11.7/22/5 | +8/12.9/25/6 | +6/8.0/20/1 | +8/9.5/21/2 | +2/4.0/16/0 | +6/10.7/17/4 | +8/12.9/20/5 |
| 15 | +8/13.2/25/9 | +13/16.5/28/10 | +10/9.3/21/2 | +11/11.8/24/4 | +2/4.0/16/1 | +8/12.7/21/5 | +10/16.8/26/5 |
| 20 | +10/15.3/28/10 | +14/21.1/33/12 | +12/11.2/23/2 | +14/12.6/27/4 | +2/5.0/18/1 | +10/14.7/22/7 | +13/17.2/27/8 |
| 30 | +13/17.7/33/12 | +22/24.9/41/18 | +14/13.2/23/2 | +20/20.7/30/8 | +4/7.0/21/1 | +11/16.7/25/8 | +19/22.4/32/15 |
| 40 | +15/21.3/36/12 | +23/28.0/45/16 | +17/15.3/25/2 | +26/19.7/32/5 | +4/7.0/22/1 | +14/20.3/27/8 | +23/26.8/36/12 |
| 50 | +16/22.0/36/12 | +21/28.1/46/17 | +18/16.7/25/2 | +23/23.3/31/7 | +4/7.0/22/1 | +14/22.3/27/8 | +20/30.2/34/13 |
| 60 | +17/24.7/36/12 | +22/30.8/45/15 | +19/18.8/25/2 | +26/23.9/32/7 | +5/8.2/23/1 | +15/23.7/27/8 | +22/28.8/36/13 |

Stats, hit points, and mana (plain gear; trained → untrained). Scores St/Dx/To/In/Wi/Pr, the seeds' mean.

| L | Warrior | Thief | Wizard | Priest | HP W/T/Wz/P | MP Wz/P | Caster level Wz/P | Blows a round W/T |
|---|---|---|---|---|---|---|---|---|
| 1 | 15/11/12/10/12/10 | 11/12/10/11/10/10 | 10/10/10/14/12/10 | 11/9/11/11/15/11 | 23→23 / 18→18 / 13→13 / 18→18 | 16/13 | 1/1 | 0.99/0.99 |
| 5 | 15/11/12/10/12/10 | 12/13/11/12/10/10 | 10/10/10/15/13/10 | 12/9/11/12/15/11 | 37→37 / 27→26 / 18→17 / 26→26 | 51/39 | 6/5 | 0.88/0.99 |
| 10 | 18/12/14/11/12/10 | 17/16/16/13/10/10 | 10/13/14/18/19/10 | 16/9/15/12/17/11 | 60→54 / 48→36 / 27→22 / 47→37 | 103/78 | 12/10 | 0.89/1.04 |
| 15 | 19/15/16/12/12/10 | 20/18/18/14/11/10 | 10/14/18/19/20/10 | 18/11/18/13/19/11 | 89→73 / 77→47 / 48→28 / 79→48 | 158/118 | 18/15 | 0.91/1.10 |
| 20 | 21/18/18/13/13/11 | 21/20/19/14/12/10 | 10/16/19/20/22/10 | 20/13/20/14/20/12 | 128→92 / 113→58 / 74→33 / 121→58 | 216/164 | 24/20 | 0.92/1.17 |
| 30 | 24/20/19/15/14/12 | 23/22/21/16/13/12 | 11/18/21/22/24/11 | 23/16/21/15/21/13 | 209→129 / 197→79 / 137→43 / 196→79 | 342/256 | 36/30 | 0.94/1.24 |
| 40 | 27/22/20/16/16/13 | 25/24/22/18/15/13 | 11/20/22/23/26/12 | 26/19/23/17/22/14 | 294→166 / 282→100 / 202→53 / 284→101 | 478/357 | 48/40 | 0.95/1.43 |
| 50 | 28/23/21/17/17/14 | 27/24/24/18/16/14 | 12/21/24/25/28/14 | 27/21/24/18/24/15 | 381→202 / 373→120 / 273→63 / 376→122 | 623/460 | 60/50 | 0.95/1.50 |
| 60 | 30/24/22/18/18/15 | 29/25/26/19/18/15 | 12/22/25/26/30/15 | 29/21/26/18/24/16 | 391→205 / 384→122 / 283→64 / 386→124 | 777/569 | 72/60 | 0.95/1.56 |

The Defence of the warrior and the thief in plain gear by category (trained → untrained), the seeds' mean.

| Class | L | Dexterity | Armour | Shield | Parry | Natural | Dodge | Deflection | Wall | total |
|---|---|---|---|---|---|---|---|---|---|---|
| warrior | 10 | 1→0 | 2→2 | 6→6 | 2→2 | 0→0 | 1→1 | 0→0 | 0→0 | 22→21 |
| warrior | 30 | 2→0 | 1→1 | 14→14 | 5→5 | 0→0 | 1→1 | 0→0 | 0→0 | 33→31 |
| warrior | 60 | 2→0 | 1→1 | 16→16 | 6→6 | 0→0 | 1→1 | 0→0 | 0→0 | 36→34 |
| thief | 10 | 3→1 | 2→2 | 0→0 | 3→3 | 0→0 | 2→2 | 0→0 | 0→0 | 20→17 |
| thief | 30 | 5→1 | 2→2 | 0→0 | 4→4 | 0→0 | 2→2 | 0→0 | 0→0 | 23→19 |
| thief | 60 | 5→1 | 2→2 | 0→0 | 6→6 | 0→0 | 2→2 | 0→0 | 0→0 | 25→21 |

## Monsters

Within two levels: the median (range) bare → the mean with gear. Damage is a hit's mean before soak.

| L (±2) | n | HP | to-hit | dmg/blow | Dv | Pv | blows a round |
|---|---|---|---|---|---|---|---|
| 1 | 41 | 11 | -1 (-5–2) → -0.3 | 2 (1–6) → 2.3 | 11 (7–14) → 11.0 | 0 (0–3) → 0.2 | 1.00 |
| 5 | 56 | 20 | 1 (-4–4) → 0.6 | 4 (1–11) → 4.1 | 12 (7–21) → 11.9 | 0 (0–2) → 0.4 | 1.00 |
| 10 | 53 | 39 | 2 (0–7) → 2.8 | 7 (2–16) → 8.5 | 11 (7–18) → 12.5 | 0 (0–5) → 0.7 | 1.00 |
| 15 | 46 | 57 | 4 (1–9) → 4.3 | 13 (4–20) → 12.2 | 12 (8–19) → 13.3 | 0 (0–5) → 1.0 | 1.00 |
| 20 | 47 | 73 | 5 (2–10) → 5.7 | 16 (4–22) → 14.4 | 13 (11–19) → 14.1 | 1 (0–9) → 1.6 | 1.00 |
| 30 | 34 | 116 | 8 (6–13) → 8.8 | 22 (10–28) → 21.0 | 15 (12–21) → 15.6 | 2 (0–11) → 2.8 | 1.02 |
| 40 | 27 | 143 | 11 (8–16) → 11.3 | 26 (11–32) → 23.1 | 17 (14–24) → 17.6 | 3 (0–11) → 3.4 | 1.07 |
| 50 | 23 | 228 | 14 (10–18) → 14.2 | 32 (16–40) → 29.5 | 19 (12–25) → 19.3 | 4 (0–13) → 4.5 | 1.05 |
| 60 | 17 | 225 | 17 (12–21) → 16.9 | 34 (18–41) → 30.9 | 21 (15–27) → 22.0 | 6 (0–16) → 6.0 | 1.15 |
| 70 | 7 | 197 | 20 (19–23) → 20.1 | 32 (20–40) → 32.1 | 25 (12–29) → 25.0 | 5 (0–12) → 4.9 | 1.35 |

## Fights

Each cell: P(player hits) / P(monster hits) % · damage a round player : monster · rounds to kill / rounds to die; the median over the monsters within two levels (trained stats, monsters with gear).

| L | Warrior kit | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain (melee) | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|---|
| 1 | 85/35 · 7.4:0.1 · 1.6 / 320 | 85/35 · 7.4:0.1 · 1.6 / 348 | 85/35 · 7.4:0.1 · 1.6 / 325 | 67/55 · 2.7:0.5 · 5.9 / 36 | 67/55 · 2.8:0.5 · 5.9 / 35 | 71/62 · 2.3:1.1 · 5.6 / 14 | 76/59 · 4.8:1.0 · 2.7 / 19 | 76/59 · 4.7:1.0 · 2.7 / 19 |
| 5 | 86/34 · 8.4:0.8 · 2.8 / 52 | 86/27 · 7.3:0.5 · 3.3 / 89 | 89/15 · 9.3:0.2 · 2.6 / 256 | 76/49 · 4.5:1.6 · 5.9 / 18 | 77/35 · 5.3:1.0 · 4.7 / 52 | 65/60 · 1.9:2.4 · 15 / 7.5 | 78/43 · 4.7:1.2 · 4.8 / 23 | 81/29 · 5.6:0.8 · 3.9 / 51 |
| 10 | 92/20 · 11.8:1.2 · 3.6 / 57 | 92/15 · 10.3:0.7 · 4.1 / 94 | 94/7 · 11.5:0.3 · 3.7 / >999 | 90/25 · 11.8:1.9 · 4.6 / 32 | 94/17 · 14.8:1.3 · 3.8 / 111 | 65/51 · 2.6:4.3 · 18 / 6.9 | 90/42 · 8.6:2.5 · 5.0 / 22 | 96/25 · 12.2:1.4 · 3.5 / 80 |
| 15 | 96/7 · 13.5:0.6 · 4.5 / 243 | 96/8 · 11.8:0.4 · 5.2 / 310 | 99/3 · 16.8:0.1 · 3.9 / >999 | 97/27 · 15.1:2.8 · 5.3 / 31 | 99/12 · 20.7:1.0 · 4.6 / 125 | 65/58 · 2.0:6.8 · 31 / 7.6 | 95/26 · 10.6:2.2 · 6.5 / 51 | 98/13 · 16.2:1.1 · 4.1 / 914 |
| 20 | 98/2 · 17.4:0.2 · 4.8 / >999 | 98/4 · 15.3:0.2 · 5.5 / >999 | 99/1 · 24.1:0.0 · 3.6 / >999 | 99/25 · 20.6:2.7 · 5.1 / 56 | 99/10 · 25.3:0.9 · 4.2 / 504 | 65/58 · 2.5:6.9 · 33 / 11 | 98/32 · 12.3:2.5 · 6.5 / 61 | 99/10 · 16.7:0.6 · 4.8 / >999 |
| 30 | 99/1 · 21.2:0.2 · 6.8 / >999 | 99/1 · 18.8:0.2 · 7.8 / >999 | 99/1 · 31.6:0.1 · 4.7 / >999 | 99/42 · 26.1:9.1 · 6.3 / 26 | 99/8 · 45.5:1.1 · 3.3 / 493 | 64/62 · 2.8:13.4 · 41 / 10 | 96/33 · 12.8:5.5 · 11 / 37 | 99/5 · 24.2:0.5 · 6.1 / >999 |
| 40 | 99/1 · 27.0:0.1 · 6.9 / >999 | 99/1 · 23.1:0.1 · 7.9 / >999 | 99/1 · 42.0:0.1 · 4.5 / >999 | 99/47 · 34.6:11.2 · 5.9 / 28 | 99/8 · 56.9:1.1 · 3.7 / 999 | 51/70 · 2.4:14.8 · 70 / 14 | 97/34 · 16.3:6.8 · 10 / 46 | 99/1 · 29.1:0.1 · 5.6 / >999 |
| 50 | 99/1 · 27.2:0.6 · 8.8 / >999 | 99/7 · 23.5:1.3 · 10 / 739 | 99/1 · 36.4:0.1 · 6.6 / >999 | 99/68 · 37.6:19.6 · 7.1 / 20 | 99/30 · 51.7:7.8 · 4.6 / 425 | 42/86 · 1.6:22.6 · 165 / 12 | 97/56 · 17.7:15.1 · 13 / 26 | 99/14 · 32.1:3.0 · 7.3 / >999 |
| 60 | 95/4 · 22.7:1.3 · 9.4 / >999 | 95/16 · 19.9:4.9 · 10 / 86 | 99/1 · 30.4:0.1 · 6.4 / >999 | 99/84 · 35.7:27.4 · 7.2 / 15 | 99/38 · 48.0:11.4 · 5.0 / 56 | 28/92 · 1.4:29.7 · 449 / 9.7 | 90/77 · 18.6:21.1 · 13 / 19 | 99/19 · 27.6:5.3 · 9.0 / 204 |

Out of depth (+3: the monsters one to five levels above), and untrained stats, for the warrior in plain gear.

| L | Warrior plain +3 | Warrior magic +3 | Warrior plain, untrained | Warrior magic, untrained |
|---|---|---|---|---|
| 1 | 81/35 · 7.0:0.6 · 2.9 / 42 | 81/35 · 7.0:0.6 · 2.9 / 42 | 85/35 · 7.4:0.1 · 1.6 / 310 | 85/35 · 7.4:0.1 · 1.6 / 329 |
| 5 | 86/34 · 7.2:1.5 · 4.9 / 27 | 88/20 · 9.1:0.8 · 3.9 / 67 | 86/27 · 7.4:0.5 · 3.3 / 83 | 89/15 · 9.2:0.2 · 2.7 / 233 |
| 10 | 92/23 · 9.9:1.5 · 6.0 / 44 | 94/11 · 11.0:0.7 · 5.5 / 512 | 86/17 · 8.6:0.9 · 5.5 / 74 | 89/8 · 9.6:0.4 · 4.5 / >999 |
| 15 | 94/11 · 11.8:0.7 · 6.2 / 162 | 99/5 · 16.8:0.3 · 4.4 / >999 | 89/11 · 8.5:0.6 · 7.7 / 191 | 98/5 · 13.7:0.2 · 4.8 / >999 |
| 20 | 97/7 · 14.9:0.6 · 7.5 / 292 | 99/1 · 23.0:0.1 · 5.0 / >999 | 90/7 · 10.1:0.4 · 9.2 / 625 | 97/1 · 20.1:0.0 · 4.5 / >999 |
| 30 | 98/3 · 17.1:0.4 · 8.6 / >999 | 99/1 · 30.5:0.1 · 5.2 / >999 | 88/5 · 11.1:0.6 · 12 / 582 | 99/1 · 25.1:0.1 · 6.2 / >999 |
| 40 | 99/2 · 23.0:0.3 · 8.6 / >999 | 99/1 · 42.0:0.1 · 4.9 / >999 | 81/3 · 12.8:0.6 · 13 / >999 | 97/1 · 29.0:0.1 · 6.3 / >999 |
| 50 | 98/8 · 20.6:1.5 · 10 / 458 | 99/1 · 33.2:0.1 · 6.8 / >999 | 76/14 · 11.0:2.7 · 22 / 140 | 92/1 · 24.3:0.2 · 10 / >999 |
| 60 | 86/29 · 15.8:8.4 · 15 / 54 | 98/2 · 27.0:0.3 · 8.7 / >999 | 55/27 · 7.0:7.7 · 42 / 30 | 87/1 · 16.6:0.3 · 15 / >999 |

## Protection by body

Rounds for the plain warrior (trained) to kill the monsters of their level: as shipped, then with every monster's Protection set to §8.2's body line at its level.

| L | as shipped | none (0) | hide | bony | armoured |
|---|---|---|---|---|---|
| 10 | 4.1 | 3.7 | 3.7 | 4.1 | 4.6 |
| 15 | 5.2 | 4.5 | 4.5 | 5.0 | 5.9 |
| 20 | 5.5 | 4.7 | 4.9 | 5.4 | 5.9 |
| 30 | 7.8 | 5.7 | 6.2 | 6.9 | 7.8 |
| 40 | 7.9 | 5.5 | 6.1 | 6.7 | 7.7 |
| 50 | 10 | 8.2 | 9.2 | 10 | 12 |
| 60 | 10 | 7.1 | 7.9 | 8.6 | 9.6 |

## Spells

Each spell that rolls (harm or healing) of the schools the trained caster knows or has an affinity with, as `Actor::spell_roll` makes it in plain gear, per cast on one target: the rung of its ladder the stats reach (the seeds' range), the school's rank the pace below reached (and the rank of the caster style it is cast in from where the caster stands), the roll (seed 1's dice, the seeds' mean) and the same roll at rank 0 (the ladder alone), and its mana (the cost table's price with no rank's cut in brackets) over the pool.

**Wizard**

| Spell (level, res) | L | rung | school rank | roll | mana / pool |
|---|---|---|---|---|---|
| Ray of frost (1, attack) | 10 | 1 | 5 (Sniper 5–6) | 1d9 1s again, 6.4 (rank 0: 1d8, 4.5) | 1 (1) / 103 |
| Ray of frost (1, attack) | 20 | 1 | 7 (Sniper 7) | 2d9 1s again, 11.0 (rank 0: 1d8, 4.5) | 1 (1) / 216 |
| Ray of frost (1, attack) | 30 | 1 | 8 (Sniper 8) | 2d9 1s and 2s again, 12.5 (rank 0: 1d8, 4.5) | 1 (1) / 342 |
| Ray of frost (1, attack) | 60 | 1 | 10 (Sniper 10) | 3d10 1s and 2s again, 18.9 (rank 0: 1d8, 4.5) | 1 (1) / 777 |
| Spark (1, auto) | 10 | 1 | 5 (Sniper 5–6) | 1d7 1s again, 5.2 (rank 0: 1d6, 3.5) | 1 (1) / 103 |
| Spark (1, auto) | 20 | 1 | 7 (Sniper 7) | 2d7 1s again, 9.0 (rank 0: 1d6, 3.5) | 1 (1) / 216 |
| Spark (1, auto) | 30 | 1 | 8 (Sniper 8) | 2d7 1s and 2s again, 10.2 (rank 0: 1d6, 3.5) | 1 (1) / 342 |
| Spark (1, auto) | 60 | 1 | 10 (Sniper 10) | 3d8 1s and 2s again, 15.8 (rank 0: 1d6, 3.5) | 1 (1) / 777 |
| Magic missile (3, auto) | 10 | 2 | 5 (Sniper 5–6) | 3d5 1s again, 10.8 (rank 0: 3d4, 7.5) | 2 (3) / 103 |
| Magic missile (3, auto) | 20 | 2 | 7 (Sniper 7) | 4d5 1s again, 13.7 (rank 0: 3d4, 7.5) | 2 (3) / 216 |
| Magic missile (3, auto) | 30 | 2–3 | 8 (Sniper 8) | 4d5+1 1s and 2s again, 15.7 (rank 0: 3d4+1, 8.2) | 2 (3) / 342 |
| Magic missile (3, auto) | 60 | 3 | 10 (Sniper 10) | 5d6+1 1s and 2s again, 21.8 (rank 0: 3d4+1, 8.5) | 1 (3) / 777 |
| Burning hands (4, area) | 10 | 2 | 5 (Sniper 5–6) | 3d5 1s again, 10.8 (rank 0: 3d4, 7.5) | 5 (6) / 103 |
| Burning hands (4, area) | 20 | 2 | 7 (Sniper 7) | 4d5 1s again, 13.7 (rank 0: 3d4, 7.5) | 5 (6) / 216 |
| Burning hands (4, area) | 30 | 2–3 | 8 (Sniper 8) | 4d5+2 1s and 2s again, 16.3 (rank 0: 3d4+2, 8.8) | 5 (6) / 342 |
| Burning hands (4, area) | 60 | 3 | 10 (Sniper 10) | 5d6+2 1s and 2s again, 22.8 (rank 0: 3d4+2, 9.5) | 1 (6) / 777 |
| Cause wounds (4, touch) | 10 | 1–2 | 0 (Toucher 1–2) | 2d8, 8.3 (rank 0: 2d8, 8.3) | 3 (3) / 103 |
| Cause wounds (4, touch) | 20 | 2 | 0 (Toucher 3) | 2d8, 9.0 (rank 0: 2d8, 9.0) | 3 (3) / 216 |
| Cause wounds (4, touch) | 30 | 2–3 | 0 (Toucher 4) | 2d8+3, 11.0 (rank 0: 2d8+2, 10.0) | 3 (3) / 342 |
| Cause wounds (4, touch) | 60 | 3 | 0 (Toucher 6) | 2d8+3, 12.0 (rank 0: 2d8+2, 11.0) | 1 (3) / 777 |
| Arcane bolt (6, area) | 10 | 1 | 5 (Sniper 5–6) | 3d7 1s again, 14.0 (rank 0: 3d6, 10.5) | 10 (11) / 103 |
| Arcane bolt (6, area) | 20 | 2 | 7 (Sniper 7) | 4d7+2 1s again, 19.9 (rank 0: 3d6+2, 12.5) | 9 (11) / 216 |
| Arcane bolt (6, area) | 30 | 2 | 8 (Sniper 8) | 4d7+2 1s and 2s again, 21.6 (rank 0: 3d6+2, 12.5) | 9 (11) / 342 |
| Arcane bolt (6, area) | 60 | 3 | 10 (Sniper 10) | 5d8+4 1s and 2s again, 30.2 (rank 0: 3d6+4, 14.5) | 2 (11) / 777 |
| Frost (6, area) | 10 | 1 | 5 (Sniper 5–6) | 3d7 1s again, 14.0 (rank 0: 3d6, 10.5) | 13 (14) / 103 |
| Frost (6, area) | 20 | 2 | 7 (Sniper 7) | 4d7+2 1s again, 19.9 (rank 0: 3d6+2, 12.5) | 12 (14) / 216 |
| Frost (6, area) | 30 | 2 | 8 (Sniper 8) | 4d7+2 1s and 2s again, 21.6 (rank 0: 3d6+2, 12.5) | 12 (14) / 342 |
| Frost (6, area) | 60 | 3 | 10 (Sniper 10) | 5d8+4 1s and 2s again, 30.2 (rank 0: 3d6+4, 14.5) | 3 (14) / 777 |
| Blight (7, area) | 10 | 1 | 0 (Sniper 5–6) | 3d6, 10.5 (rank 0: 3d6, 10.5) | 17 (17) / 103 |
| Blight (7, area) | 20 | 1–2 | 0 (Sniper 7) | 3d6+3, 12.5 (rank 0: 3d6+3, 12.5) | 17 (17) / 216 |
| Blight (7, area) | 30 | 2 | 0 (Sniper 8) | 3d6+3, 13.5 (rank 0: 3d6+3, 13.5) | 17 (17) / 342 |
| Blight (7, area) | 60 | 2 | 0 (Sniper 10) | 3d6+3, 13.5 (rank 0: 3d6+3, 13.5) | 6 (17) / 777 |
| Scorching ray (7, attack) | 10 | 2 | 5 (Sniper 5–6) | 3d7 1s again, 14.0 (rank 0: 3d6, 10.5) | 6 (7) / 103 |
| Scorching ray (7, attack) | 20 | 3 | 7 (Sniper 7) | 5d7 1s again, 22.4 (rank 0: 4d6, 14.0) | 6 (7) / 216 |
| Scorching ray (7, attack) | 30 | 3 | 8 (Sniper 8) | 5d7 1s and 2s again, 24.4 (rank 0: 4d6, 14.0) | 6 (7) / 342 |
| Scorching ray (7, attack) | 60 | 4 | 10 (Sniper 10) | 5d8+1d12 1s and 2s again, 33.6 (rank 0: 3d6+1d10, 16.0) | 1 (7) / 777 |
| Call lightning (11, area) | 10 | 2–3 | 5 (Sniper 5–6) | 5d7 1s again, 21.4 (rank 0: 5d6, 16.3) | 19 (20) / 103 |
| Call lightning (11, area) | 20 | 3 | 7 (Sniper 7) | 6d7 1s again, 26.9 (rank 0: 5d6, 17.5) | 18 (20) / 216 |
| Call lightning (11, area) | 30 | 3–4 | 8 (Sniper 8) | 6d7+2 1s and 2s again, 30.1 (rank 0: 5d6+2, 18.5) | 18 (20) / 342 |
| Call lightning (11, area) | 60 | 4 | 10 (Sniper 10) | 7d8+2 1s and 2s again, 38.8 (rank 0: 5d6+2, 19.5) | 5 (20) / 777 |
| Fireball (11, area) | 10 | 2 | 5 (Sniper 5–6) | 4d7 1s again, 18.5 (rank 0: 4d6, 14.0) | 24 (26) / 103 |
| Fireball (11, area) | 20 | 3 | 7 (Sniper 7) | 6d7 1s again, 26.9 (rank 0: 5d6, 17.5) | 23 (26) / 216 |
| Fireball (11, area) | 30 | 3 | 8 (Sniper 8) | 6d7 1s and 2s again, 29.1 (rank 0: 5d6, 17.5) | 23 (26) / 342 |
| Fireball (11, area) | 60 | 4 | 10 (Sniper 10) | 7d8+2 1s and 2s again, 38.8 (rank 0: 5d6+2, 19.5) | 6 (26) / 777 |
| Lightning bolt (11, area) | 10 | 2 | 5 (Sniper 5–6) | 4d7 1s again, 18.5 (rank 0: 4d6, 14.0) | 19 (20) / 103 |
| Lightning bolt (11, area) | 20 | 3 | 7 (Sniper 7) | 6d7 1s again, 26.9 (rank 0: 5d6, 17.5) | 18 (20) / 216 |
| Lightning bolt (11, area) | 30 | 3 | 8 (Sniper 8) | 6d7 1s and 2s again, 29.1 (rank 0: 5d6, 17.5) | 18 (20) / 342 |
| Lightning bolt (11, area) | 60 | 4 | 10 (Sniper 10) | 7d8+2 1s and 2s again, 38.8 (rank 0: 5d6+2, 19.5) | 5 (20) / 777 |
| Flame (12, area) | 10 | 1 | 5 (Sniper 5–6) | 4d7 1s again, 18.5 (rank 0: 4d6, 14.0) | 27 (29) / 103 |
| Flame (12, area) | 20 | 2 | 7 (Sniper 7) | 6d7 1s again, 26.9 (rank 0: 5d6, 17.5) | 26 (29) / 216 |
| Flame (12, area) | 30 | 2 | 8 (Sniper 8) | 6d7 1s and 2s again, 29.1 (rank 0: 5d6, 17.5) | 26 (29) / 342 |
| Flame (12, area) | 60 | 3 | 10 (Sniper 10) | 7d8+2 1s and 2s again, 38.8 (rank 0: 5d6+2, 19.5) | 7 (29) / 777 |

**Priest**

| Spell (level, res) | L | rung | school rank | roll | mana / pool |
|---|---|---|---|---|---|
| Mend (1, touch) | 10 | 0–1 | 7 (Battle caster 4–5) | 2d5+2 1s again, 8.6 (rank 0: 1d4, 2.4) | 1 (1) / 78 |
| Mend (1, touch) | 20 | 1 | 10 (Battle caster 10) | 2d6+3 1s and 2s again, 11.3 (rank 0: 1d4, 2.5) | 1 (1) / 164 |
| Mend (1, touch) | 30 | 1–2 | 10 (Battle caster 10) | 2d6+3 1s and 2s again, 12.4 (rank 0: 1d4, 3.0) | 1 (1) / 256 |
| Mend (1, touch) | 60 | 2 | 10 (Battle caster 10) | 2d8+3 1s and 2s again, 13.5 (rank 0: 1d6, 3.5) | 1 (1) / 569 |
| Cure wounds (3, touch) | 10 | 0–1 | 7 (Battle caster 4–5) | 4d7+2 1s again, 19.0 (rank 0: 3d6, 9.9) | 1 (2) / 78 |
| Cure wounds (3, touch) | 20 | 1 | 10 (Battle caster 10) | 4d8+3 1s and 2s again, 24.0 (rank 0: 3d6, 10.5) | 2 (2) / 164 |
| Cure wounds (3, touch) | 30 | 1–2 | 10 (Battle caster 10) | 4d8+3 1s and 2s again, 25.5 (rank 0: 3d6, 12.0) | 2 (2) / 256 |
| Cure wounds (3, touch) | 60 | 2 | 10 (Battle caster 10) | 4d8+6 1s and 2s again, 27.0 (rank 0: 3d6+3, 13.5) | 2 (2) / 569 |

**Witch**

| Spell (level, res) | L | rung | school rank | roll | mana / pool |
|---|---|---|---|---|---|
| Cause wounds (4, touch) | 10 | 1–2 | 6–7 (Toucher 6–7) | 2d8+3 1s again, 12.7 (rank 0: 2d8, 8.7) | 2 (3) / 91 |
| Cause wounds (4, touch) | 20 | 2 | 7–8 (Toucher 6–8) | 2d8+3 1s again, 14.2 (rank 0: 2d8, 9.0) | 2 (3) / 199 |
| Cause wounds (4, touch) | 30 | 2–3 | 9–10 (Toucher 6–10) | 3d8+4 1s and 2s again, 21.2 (rank 0: 2d8, 10.0) | 2 (3) / 318 |
| Cause wounds (4, touch) | 60 | 3 | 10 (Toucher 6–10) | 3d8+7 1s and 2s again, 23.1 (rank 0: 2d8+2, 11.0) | 2 (3) / 736 |
| Blight (7, area) | 10 | 1 | 6–7 (Sniper 2–4) | 3d6+2 1s again, 13.8 (rank 0: 3d6, 10.5) | 16 (17) / 91 |
| Blight (7, area) | 20 | 1–2 | 7–8 (Sniper 7) | 3d6+5 1s again, 17.2 (rank 0: 3d6+3, 13.0) | 15 (17) / 199 |
| Blight (7, area) | 30 | 2 | 9–10 (Sniper 8) | 4d6+6 1s and 2s again, 22.8 (rank 0: 3d6+3, 13.5) | 15 (17) / 318 |
| Blight (7, area) | 60 | 2 | 10 (Sniper 8–10) | 4d6+7 1s and 2s again, 23.7 (rank 0: 3d6+3, 13.5) | 14 (17) / 736 |

## School pace

The trained caster's school and caster style as their casting trained them (`balance_probe/schools.rs`): each day of each level, one pool spent on the pace spell (of the school's spells, the most a cast among those costing at most a fifth of the pool, never one cast only under the open sky; for the priest a heal, at most one a kill), every cast landing on a monster of the caster's level through `World::practise`. Each row: the pace spell over the level before and its cost (seed 1's); casts a day from the pool, and the spell casts a day the same level's fights took (the stat training's careers, which refill the pool before every cast; the priest's one a kill); the school's rank and the caster style's at the level (the seeds' range); and the pool check, the pool over a single-target spell of the caster's own level (its base mana: the level; no rank's cut, no talent).

**Wizard** (evocation)

| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |
|---|---|---|---|---|---|
| 1 | – | 0.0 / 0.0 | 2 | – | 16 / 1 = 16.5 |
| 5 | Scorching ray (7) | 5.8 / 122.5 | 3–4 | Sniper 3–4 | 51 / 5 = 10.3 |
| 10 | Scorching ray (6) | 11.3 / 217.8 | 5 | Sniper 5–6 | 103 / 10 = 10.3 |
| 15 | Flame (27) | 5.2 / 268.2 | 6 | Sniper 6 | 158 / 15 = 10.5 |
| 20 | Flame (26) | 7.2 / 239.6 | 7 | Sniper 7 | 216 / 20 = 10.8 |
| 30 | Flame (26) | 12.2 / 385.6 | 8 | Sniper 8 | 342 / 30 = 11.4 |
| 40 | Flame (23) | 20.2 / 343.5 | 9 | Sniper 9–10 | 478 / 40 = 11.9 |
| 50 | Flame (21) | 28.5 / 460.7 | 10 | Sniper 10 | 623 / 50 = 12.5 |
| 60 | Flame (21) | 35.8 / 552.2 | 10 | Sniper 10 | 777 / 60 = 13.0 |

**Priest** (restoration)

| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |
|---|---|---|---|---|---|
| 1 | – | 0.0 / 0.0 | 1 | – | 13 / 1 = 13.0 |
| 5 | Cure wounds (2) | 16.2 / 42.0 | 3–4 | Toucher 3–4 | 39 / 5 = 7.9 |
| 10 | Cure wounds (1) | 68.0 / 73.0 | 7 | Toucher 7–8 | 78 / 10 = 7.8 |
| 15 | Cure wounds (1) | 97.4 / 98.5 | 9 | Battle caster 8–9 | 118 / 15 = 7.9 |
| 20 | Cure wounds (1) | 74.3 / 74.3 | 10 | Battle caster 10 | 164 / 20 = 8.2 |
| 30 | Cure wounds (2) | 88.0 / 88.0 | 10 | Battle caster 10 | 256 / 30 = 8.5 |
| 40 | Cure wounds (2) | 75.0 / 75.0 | 10 | Battle caster 10 | 357 / 40 = 8.9 |
| 50 | Cure wounds (2) | 83.2 / 83.2 | 10 | Battle caster 10 | 460 / 50 = 9.2 |
| 60 | Cure wounds (2) | 91.2 / 91.2 | 10 | Battle caster 10 | 569 / 60 = 9.5 |

**Witch** (necromancy)

| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |
|---|---|---|---|---|---|
| 1 | – | 0.0 / 0.0 | 1 | – | 13 / 1 = 13.3 |
| 5 | Cause wounds (3) | 11.5 / 126.7 | 3 | Toucher 3–4 | 43 / 5 = 8.6 |
| 10 | Blight (16) | 21.3 / 235.3 | 6–7 | Sniper 3–7 | 91 / 10 = 9.1 |
| 15 | Blight (15) | 8.5 / 393.8 | 7–8 | Sniper 5 | 143 / 15 = 9.5 |
| 20 | Blight (15) | 11.8 / 357.3 | 7–8 | Sniper 7 | 199 / 20 = 9.9 |
| 30 | Blight (15) | 42.3 / 447.5 | 9–10 | Sniper 8–10 | 318 / 30 = 10.6 |
| 40 | Blight (14) | 61.5 / 472.5 | 10 | Sniper 9–10 | 448 / 40 = 11.2 |
| 50 | Blight (14) | 82.0 / 683.2 | 10 | Sniper 10 | 588 / 50 = 11.8 |
| 60 | Blight (14) | 102.8 / 818.2 | 10 | Sniper 10 | 736 / 60 = 12.3 |

## Sneak attacks

The thief's main-hand blow on the monsters of their level (trained stats), aware and caught unawares. The Sneak attack value assumed, the dice it gives with the talents, and the share of the monsters within two levels with vitals (no dice and no critical on the rest, so their first blow gains only the flat-footed Defence); then each cell: P(hit) aware / unaware % · damage of a blow aware : the first blow unaware · how many times as much, the median over the monsters within two levels that have vitals.

| L | Sneak attack | dice | with vitals | Thief plain | Thief magic |
|---|---|---|---|---|---|
| 1 | 15 | 1d6 | 71% | 75/75 · 2.7:5.7 · ×2.1 | 75/75 · 2.7:5.7 · ×2.1 |
| 5 | 29 | 2d6 | 55% | 76/86 · 4.0:10.5 · ×2.6 | 77/86 · 4.9:11.5 · ×2.4 |
| 10 | 47 | 2d6 | 72% | 93/98 · 8.4:15.8 · ×1.9 | 95/98 · 10.2:17.6 · ×1.7 |
| 15 | 64 | 3d6 | 63% | 99/99 · 10.4:21.1 · ×2.0 | 99/99 · 13.6:24.0 · ×1.8 |
| 20 | 82 | 4d6 | 66% | 99/99 · 12.4:26.3 · ×2.1 | 99/99 · 15.1:28.9 · ×1.9 |
| 30 | 117 | 5d6 | 62% | 99/99 · 14.4:31.9 · ×2.2 | 99/99 · 24.4:41.8 · ×1.7 |
| 40 | 152 | 8d6 | 59% | 99/99 · 16.8:44.5 · ×2.7 | 99/99 · 23.8:51.5 · ×2.2 |
| 50 | 187 | 8d6 | 52% | 99/99 · 17.1:45.1 · ×2.6 | 99/99 · 27.8:55.3 · ×2.0 |
| 60 | 222 | 9d6 | 47% | 99/99 · 19.1:53.2 · ×2.8 | 99/99 · 26.7:61.3 · ×2.3 |

## Stat training

The stat training design (`docs/design/stat-training.md`, slices T1 to T6, #722 to #727) set
its pace with a model of a career outside the game (§3.1, §6.2). Since T6 the probe measures the
pace through the game's own code, so the model's guesses about combat are replaced by the
fights the probe already measures.

**Method** (`balance_probe/training.rs`):
- **Days.** A level takes the design's days (§3.1): one for levels 2 to 5, two to 15, three to
  30, four beyond, 189 days to level 60. Each day has its share of the level's kills, the
  experience curve's kills of the player's own level (`Experience::kills_for`: 10 for level 2,
  146 for 10, 365 for 60).
- **Fights.** Each kill is a fight against a monster within two levels of the player, the
  types in turn, each with one seed's gear. It is fought blow by blow with `combat::strike`
  (the player's blows a round as above, their off hand too) until the monster's hit points are
  gone or 40 rounds have passed. Every blow either way goes through `World::train_fight` (blow,
  critical, block, parry, hurt, big_hit); a monster's miss by Dexterity or Dodge is a `dodge`;
  a kill of a monster of the player's level or above is `killed`, the first of its type
  `discovered`. The thief's first blow of a fight is a sneak attack. The wizard fights with its
  pace spell ("School pace": of evocation's damage spells, the most damage a cast among those
  costing a fifth of its pool or less, none cast only under the open sky): each
  cast spends its mana through `World::spend_mana` (cast, pool refilled first), trains through
  `World::train_spell` (spell), and an attack spell's roll goes through `train_fight` (aimed).
  The priest spends mana on an ability once a fight (a heal, a blessing).
- **The rest of a day** (first guesses; `training::COMMON` and each build's `extras`): 3 long
  actions done to the end (a rest and two meals), 2 chats, 1 trade, 1 creature making way,
  30 steps within earshot of something (the thief 100), 20 turns of searching that find
  nothing, 10 steps over the overworld, 2 through deep water, and a few rarer uses (a find, a
  scroll read, a level explored every three days, a lock picked, a trap disarmed, a quest every
  other level, ...); the warrior spends stamina on a weapon ability 5 times, the thief 3, the
  priest 2; the thief picks a lock and steals; the wizard reads a scroll; the priest prays at an
  altar every fifth day. A load that slows the player (the dressed player's, without the kit's
  pieces the gear replaced) trains `burdened` on every round and step; none of the four is
  slowed in plain gear. These go through `World::train_stat_times`, each use at its own chance.
- **Checks.** Each day ends with the daily check (`World::daily_stat_check`); each level with
  the level's own (`levelup:auto`). Nothing else touches the piles: no potions, no elixirs, no
  corpses, no deeds (the probe's player is never hungry), and talents only as `levelup:auto`
  and the random creation picks give them.
- **The knobs.** The same trained careers again with other `price_per_point` and
  `ramp_divisor` values in `data/stats.toml` (the design's §6.2 sweep), scores at level 60.

The hooks give far more marks than the design's model in a stat a build lives by: a warrior
earns about 340 Strength marks a day at level 30 and 500 at 60 (the model: 170 and 220),
mostly from blocks, as monsters swing at a warrior who is hard to hit (tuning note 1) for 9 to
14 rounds a kill (the model: 8 to 11). The wizard's Willpower comes to 240 a day at 30 and 590
at 60 (the model: 210 and 330). The stats a build does not use get 2 to 3 marks a day from the
day's other uses, about what the model gave.

The probe's tables, with the data as it is (`ramp_divisor` 2):

Scores of the trained careers (plain gear; the seeds' mean), start/potential at level 1, and the gain and the points past the potential at 60.

| Class | Stat | Start/potential | L10 | L20 | L30 | L40 | L60 | Gain | Past potential |
|---|---|---|---|---|---|---|---|---|---|
| warrior | St | 14.8/16.7 | 17.7 | 21.2 | 24.2 | 27.0 | 30.2 | +15.3 | 13.5 |
|  | Dx | 10.8/16.3 | 11.8 | 17.7 | 20.0 | 21.8 | 24.5 | +13.7 | 8.2 |
|  | To | 11.7/16.0 | 13.5 | 17.8 | 19.0 | 20.3 | 22.3 | +10.7 | 6.3 |
|  | In | 9.8/17.0 | 11.2 | 13.0 | 14.7 | 16.3 | 18.2 | +8.3 | 1.2 |
|  | Wi | 11.7/17.7 | 11.7 | 13.2 | 14.3 | 15.7 | 18.3 | +6.7 | 0.8 |
|  | Pr | 10.0/16.2 | 10.0 | 10.8 | 11.8 | 12.8 | 14.8 | +4.8 | 0.0 |
| | all six | | | | | | | **+59.5** | |
| thief | St | 11.0/16.8 | 16.8 | 21.0 | 23.2 | 25.2 | 28.8 | +17.8 | 12.0 |
|  | Dx | 12.5/16.5 | 16.3 | 20.0 | 22.0 | 23.5 | 25.3 | +12.8 | 8.8 |
|  | To | 10.5/16.8 | 15.5 | 19.3 | 20.8 | 22.3 | 25.7 | +15.2 | 8.8 |
|  | In | 11.3/17.2 | 12.7 | 14.2 | 15.8 | 17.5 | 19.2 | +7.8 | 2.0 |
|  | Wi | 10.3/17.3 | 10.5 | 12.2 | 13.3 | 15.0 | 17.5 | +7.2 | 0.5 |
|  | Pr | 9.8/16.3 | 9.8 | 10.5 | 11.7 | 12.8 | 15.0 | +5.2 | 0.0 |
| | all six | | | | | | | **+66.0** | |
| wizard | St | 10.0/16.7 | 10.0 | 10.0 | 11.0 | 11.0 | 12.3 | +2.3 | 0.0 |
|  | Dx | 10.0/16.3 | 12.7 | 16.0 | 18.5 | 20.3 | 21.8 | +11.8 | 5.5 |
|  | To | 9.5/17.3 | 13.7 | 19.0 | 20.8 | 22.5 | 25.2 | +15.7 | 7.8 |
|  | In | 14.5/16.8 | 17.5 | 20.0 | 21.8 | 23.2 | 26.3 | +11.8 | 9.5 |
|  | Wi | 12.5/17.7 | 18.7 | 21.7 | 24.0 | 26.0 | 29.5 | +17.0 | 11.8 |
|  | Pr | 9.5/16.0 | 9.5 | 10.3 | 11.3 | 12.5 | 15.2 | +5.7 | 0.0 |
| | all six | | | | | | | **+64.3** | |
| priest | St | 10.8/16.3 | 16.0 | 20.3 | 22.8 | 25.8 | 28.7 | +17.8 | 12.3 |
|  | Dx | 8.7/16.3 | 9.2 | 13.0 | 16.5 | 19.2 | 21.2 | +12.5 | 4.8 |
|  | To | 10.7/16.7 | 14.7 | 19.5 | 20.8 | 22.8 | 26.5 | +15.8 | 9.8 |
|  | In | 11.3/16.7 | 12.5 | 14.0 | 15.3 | 16.8 | 18.3 | +7.0 | 1.7 |
|  | Wi | 15.3/18.0 | 16.7 | 19.7 | 21.2 | 22.5 | 24.2 | +8.8 | 6.2 |
|  | Pr | 10.8/16.3 | 10.8 | 11.7 | 12.7 | 13.8 | 15.7 | +4.8 | 0.0 |
| | all six | | | | | | | **+66.8** | |

Marks a day the hooks gave in each stat over the level before (St/Dx/To/In/Wi/Pr, the seeds' mean), and the game day reached.

| L | Day | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| 5 | 4 | 26/7/15/21/3/3 | 65/34/64/21/3/3 | 1/22/29/44/66/3 | 37/5/36/20/17/3 |
| 10 | 14 | 93/19/38/6/3/1 | 117/87/68/6/3/2 | 1/35/56/69/157/2 | 89/11/64/5/29/2 |
| 15 | 24 | 179/58/51/6/3/2 | 187/132/61/7/3/2 | 1/17/66/80/192/2 | 174/20/97/6/45/2 |
| 20 | 39 | 324/52/39/4/3/1 | 202/140/66/4/3/2 | 1/22/52/77/182/2 | 175/24/66/4/37/2 |
| 30 | 69 | 404/98/42/3/3/2 | 314/184/96/4/3/2 | 1/37/75/118/300/2 | 300/35/135/4/43/2 |
| 40 | 109 | 475/88/37/2/3/1 | 299/126/120/2/3/2 | 1/34/70/107/262/2 | 292/28/141/2/38/1 |
| 50 | 149 | 476/114/46/2/3/1 | 397/117/141/2/3/1 | 1/27/99/146/345/2 | 332/25/178/2/40/2 |
| 60 | 189 | 464/133/93/2/3/2 | 443/96/211/2/3/2 | 1/26/139/172/421/1 | 343/21/300/2/44/2 |

The game day of each stat's first rise (the seeds' mean of those that rose by level 60; how many of the six in brackets when not all).

| Class | St | Dx | To | In | Wi | Pr |
|---|---|---|---|---|---|---|
| warrior | 6 | 11 | 6 | 6 | 21 | 34 |
| thief | 4 | 6 | 4 | 5 | 19 | 32 |
| wizard | 62 | 6 | 5 | 6 | 4 | 25 |
| priest | 4 | 14 | 5 | 8 | 8 | 30 |

The knobs (design §6.2): mean gain by level 60 with other `price_per_point` and `ramp_divisor` values (St/Dx/To/In/Wi/Pr · all six), the data's in bold.

| Price | Ramp ÷ | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| **20 + 10s** | **2** | +15.3/+13.7/+10.7/+8.3/+6.7/+4.8 · +60 | +17.8/+12.8/+15.2/+7.8/+7.2/+5.2 · +66 | +2.3/+11.8/+15.7/+11.8/+17.0/+5.7 · +64 | +17.8/+12.5/+15.8/+7.0/+8.8/+4.8 · +67 |
| 20 + 10s | 3 | +16.3/+14.5/+11.5/+8.5/+6.5/+5.3 · +63 | +19.3/+14.3/+15.8/+8.0/+7.0/+5.0 · +70 | +2.7/+12.8/+16.7/+12.5/+18.2/+5.5 · +68 | +18.8/+13.2/+16.7/+7.5/+9.8/+4.8 · +71 |
| 20 + 10s | 4 | +17.2/+15.3/+11.8/+8.3/+7.0/+5.2 · +65 | +20.5/+15.0/+16.7/+7.7/+7.2/+5.3 · +72 | +2.8/+13.7/+17.3/+13.3/+19.7/+5.2 · +72 | +20.3/+13.2/+17.0/+7.3/+10.2/+5.0 · +73 |
| 20 + 10s | 6 | +18.3/+16.3/+12.8/+8.5/+6.5/+5.0 · +68 | +21.8/+16.2/+18.0/+8.0/+7.3/+5.3 · +77 | +2.3/+14.7/+18.2/+14.8/+21.7/+5.2 · +77 | +21.3/+13.8/+18.3/+7.8/+11.3/+4.8 · +78 |
| 20 + 15s | 2 | +14.2/+12.8/+9.7/+6.7/+5.8/+4.0 · +53 | +16.7/+12.0/+14.2/+6.7/+6.2/+3.8 · +60 | +2.0/+11.3/+14.8/+10.3/+15.3/+4.0 · +58 | +17.0/+11.5/+14.7/+6.5/+8.0/+4.2 · +62 |
| 20 + 15s | 4 | +16.2/+14.2/+11.2/+7.0/+5.7/+3.5 · +58 | +18.7/+13.7/+15.5/+6.8/+5.7/+4.2 · +64 | +2.0/+11.8/+16.5/+12.3/+17.8/+4.3 · +65 | +18.5/+12.3/+15.8/+6.5/+9.2/+4.2 · +66 |
| 20 + 20s | 4 | +14.8/+13.7/+10.8/+5.7/+4.3/+3.0 · +52 | +18.0/+13.2/+14.7/+6.2/+4.8/+3.2 · +60 | +1.7/+11.2/+15.7/+11.3/+17.3/+3.5 · +61 | +17.7/+11.7/+15.7/+5.3/+8.7/+2.7 · +62 |

**Before and after, by class.** Gain of all six stats by level 60 (the main stat's score at 60
in brackets). *Before*: the probe as it was until T6, two days of fixed marks a level (the
warrior St 7, Dx 3, To 4, the rest 1; §9.1 of the combat design), on T1 to T5's rules. *Design*:
§6.2's model. *Hooks, ÷4*: this method with the data as T1 shipped it (`ramp_divisor` 4).
*Hooks, ÷2*: with `ramp_divisor` 2, as #727 set it.

| Class (main stat) | Before | Design (§6.2) | Hooks, ÷4 | **Hooks, ÷2** |
|---|---|---|---|---|
| warrior (St) | +22 (20) | +51 (28.8) | +65 (32.3) | **+59 (29.7)** |
| thief (St, Dx) | +28 (18, 19) | | +72 (31.5, 27.3) | **+66 (29.2, 25.2)** |
| wizard (Wi) | +22 (19) | +66 (29.0) | +74 (33.0) | **+66 (30.2)** |
| priest (St, Wi) | +24 (18, 20) | | +73 (30.8, 25.6) | **+68 (29.2, 24.5)** |

The fixed table fed the rules a tenth of what play gives, so the old probe's careers ended at
+22 to +28. On the hooks with `ramp_divisor` 4, three of the four classes passed the design's
G7 (+45 to +70 in all, a main stat +12 to +17) and every main stat gained 17 to 21. The sweep
below puts the ramp at 2 inside G7 in all for every class (+59 to +68), main stats at +15 to
+18, 29 to 30 at level 60, and nothing near 40; `price_per_point` moves the trickle stats and
the main ones less (15 with ÷4 lands about where 10 with ÷2 does, but leaves the main stats at
+16 to +19). So **`ramp_divisor` is 2** since #727: the first point past the potential costs
twice the base, the second 4 ×, the third 7 ×. `price_per_point` stays 10: the early pace
it sets is as the design meant (the first rise in a stat a build lives by on day 4 to 6, §6.3:
"about day 5 or 6").

Still above the design's targets: a stat the build does not use gains +7 to +9 in Intelligence
and Willpower, not +2 to +5. Those marks come from the day's guessed uses, not the fights: a
first kill of each of some 400 creature types (`discovered`, sure, 400 Intelligence marks over
a career) and three long actions a day (`endured`, sure, 570 Willpower marks). They are the next
dials if the unused stats should grow less; #727 left them, as the day's activity they
rest on is a guess.

**The thief and the priest live by Strength.** Both gain more Strength than anything else
(+18), the thief more than Dexterity (+13): every melee blow that lands trains Strength
(`blow`), a dagger's as a mace's, while Dexterity comes only from dodges, parries, sneaking, and
picking locks. Whether a light weapon's blow should train Dexterity instead is a design question
for its own issue.

**Deeds, measured.** A faint is a draw every turn while Fainting or Starving (`[fainting]`
5 %), some 25 faints an hour of play. At T3's 10 Willpower marks a faint, an afternoon of
fainting took about a thousand marks: fifteen drops' worth at Willpower 12 (drop price 65), so
one fast meant a Willpower point lost at nearly every check for two weeks after it ended. `fainted`
is now a mark at 20 % a faint (about 5 marks an hour; measured with `warlock-cli dump`: 28
marks over six hours of fainting and starving to the edge of death), so an afternoon of it
warns and a day or two of it costs a point, like a few dawns met Weak (15 a dawn).

**Corpses.** The resolved table (`corpse-effects.md`) has one outlier: the fire drake, 150
Strength and 150 Toughness marks. It is the Ascmearc's unique drake at the bottom of Fyrdelf,
eaten once a run at about level 19, where a warrior earns 200 Strength marks a day: about a
point in each, a fitting meal for the campaign's last quest, and left as it is. Every other
corpse gives 60 marks or less in a stat.

**Creation picks.** The probe's players are random human characters, so their creation talents
are Random's picks, drawn from the talents offered; the culls (#774, #787) took Gifted and the
other filler out of those, so the seeds' picks, starts, and potentials moved again (the tables
above are the probe's after #787).

## Against the design's predictions

(The spell paragraph is #754's, against the spell scaling design's §12.3 and §8.4; the others
are #569's, against the combat progression design.)

**§3 (the new ranks, with training).** The design predicted that a warrior with a long sword in
plain gear and trained Strength would have +6 / 10.5 at level 10 (93 %, 4 rounds to kill), +11 /
15.5 at 30 (93 %, 8 rounds), and +15 / 20.5 at 60 (93 %, 11 rounds). The probe measures +6 /
11.3 (90 %, 4.2), +12 / 16.8 (95 %, 7.4), and +15 / 23.0 (80 %, 9.6). To-hit and damage land
as predicted. The hit chance at level 60 is lower because the monsters' Defence with gear
averages 23, where the design assumed 21. Trained Strength reaches 27 at level 60 (the design
assumed 26).

**§9.4 (hit points past level 20).** For untrained characters, the probe matches the design's
"after" column: at level 60 the warrior has 215 hit points (predicted 187), the thief 122 (120),
the wizard 60 (62), and the priest 122 (120). Trained characters have about twice that: 400,
340, 180, and 321. The table assumed creation Toughness, but trained Toughness raises the
modifier that every level up to 50 adds.

**§8.4 (Protection by body).** The design predicted that the trained plain warrior would kill a
hide, bony, and armoured monster in 8 / 9 / 11 rounds at level 30, and in 13 / 16 / 22 at 60. The
probe measures 6.5 / 7.1 / 8.3 and 8.7 / 9.5 / 12. The monsters as shipped sit at about the bony
line (7.6 at 30, 9.5 at 60), and the armoured body adds about a quarter to a third to a fight,
not a wall. The prediction was higher because the measured damage is higher (23 a hit at 60,
not 20.5).

**Spell scaling §12.3 and §8.4 (#754, S7).** The design's numbers came from a script over the
rules as written, for a trained wizard of Intelligence 16 / 20 / 23 at levels 10 / 30 / 60 and
the ranks §8.4 estimated (5 / 8 / 10); the probe measures them on the game's code, with the
ranks the casting really reaches.
- **The pace** ("School pace"): the wizard's evocation is rank 3 at level 5, 5 at 10, 6 at 15,
  7 at 20, 8 at 30, 9 at 40, and 10 from 50. §8.4 estimated rank 5 at about 8 to 10, 7 by 20, 8
  by 30, and 10 toward 60: the missile curve lands where the design put it, rank 10 a little
  early. A pool buys 5 to 12 casts of the wizard's pace spell a day to level 30 (Scorching ray,
  then Flame at 26 mana) and 20 to 36 from 40 (the ranks' mana cut and a growing pool), and
  every landed cast against a monster of the caster's level earns the most marks (`most`, 200 %)
  until the rank nears the level. The witch is quicker (necromancy 6 at 10, 7 to 9 at 20, 9 to
  10 at 30, 10 from 40): Blight is cheaper than Flame, so a pool buys 15 to 26 casts. The
  priest's restoration is the quickest (7 at 10, 10 from 20) and measures the model more than
  the curve: one heal a kill, and the experience curve's kills (73 to 98 a day) are fewer than
  the pool allows at Cure wounds' 1 to 2 mana; a priest heals only what is wounded. **No school
  row or curve is tuned**: the schools stay on the missile curve (the design's open question
  (b), the owner's default), as the wizard's measured pace is the estimate's.
- **Rank 0** (the ladder alone): within a die of §12.3 everywhere. Ray of frost 1d8 (4.5) and
  Magic missile 3d4 to 3d4+1 (7.1 to 8.5) as predicted; Fireball 4d6 (14.0) at 10 and 5d6 (17.5)
  at 30 as predicted, 5d6 already at 20 (the design: one die lower there; trained Intelligence
  now averages 19.5 at level 20, against the design's 18) and 5d6+2 (19.5) at 60 (the design
  5d6: Intelligence averages 25.7 at 60, not 23, and reaches the next rung); Scorching ray 3d6 / 4d6 at 10 / 30 as predicted and 3d6+1d10 (16.0) at 60.
- **With the ranks**: Fireball 17.7 / 29.1 / 38.8 at levels 10 / 30 / 60 (§12.3: 17.7 / 28.3 /
  36.8; the extra is the rung the trained Intelligence reaches); Ray of frost 6.4 / 12.5 / 18.9
  (5.4 / 11.6 / 18.9), Magic missile 10.2 / 15.5 / 21.8 (10.2 / 14.4 / 21.8), Scorching ray
  13.3 / 24.4 / 33.6 (13.3 / 23.6 / 31.5), Arcane bolt and Frost 14.0 / 21.6 / 30.2 (13.3 /
  20.9 / 28.2), Call lightning 20.7 / 29.1 / 38.8 (17.7 / 28.3 / 38.8). The witch's and the
  priest's are higher at 10 and 30 than §12.3, as their ranks run ahead of §8.4's: Cause wounds
  12.4 / 21.2 / 23.8 (9.8 / 18.8 / 21.8), Blight 13.8 / 22.8 / 23.7 (13.8 / 19.7 / 23.7), Cure
  wounds 19.0 / 25.0 / 27.0 (14.3 / 20.9 / 27), Mend 8.6 / 12.1 / 13.5 (4.4 / 9.2 / 13.5); all
  meet the design at 60, where every school is rank 10. A rank-10 school still about doubles a
  spell over its ladder (Fireball 38.8 against 19.5).
- **The pool** (§12.4: about ten casts of a spell of the caster's own level, the ranks' mana
  off): the wizard's pool is 10.2 to 12.7 casts of a level-L spell at every level from 5 to 60
  (102 at 10, 336 at 30, 764 at 60); the priest's 7.8 to 9.4, the witch's 8.5 to 12.0. Passes.
- **Casting in the stat training's fights**: those careers fight every kill with the pace spell
  and refill the pool before every cast, so they cast 125 to 700 spells a day where the pool
  buys 5 to 36: the Intelligence and Willpower marks of a caster's fights ("Stat training") are
  a ceiling. The school pace does not use them.

**§2 (the report's findings), revisited.**
- Monster to-hit still grows faster than the player's unaided to-hit, but the player's now grows
  too: a plain warrior goes from +3 to +15, not +4 to +8.
- The magic-armour gap is unchanged, as the owner chose (§17 q11). A warrior, thief, or priest
  fully in magic gear is hit 1 to 9 % from level 20 to 60, and a monster needs hundreds of rounds
  to kill them.
- Hit points grow to level 50 (above).
- Monster Protection now matters: the median goes from 0 at level 15 to 6 at 60, not 0 to 2.
- The wizard's spells keep growing to level 60 (#754 above): a rank-10 school doubles a spell
  over its ladder, and there is no cap. There is still no spell above level 12 (higher-level
  spells are the content's to add).

## Tuning notes

These are findings for their own PRs (data changes are not made here). Each says what the probe
shows and where the number lives.

1. **The shield stack makes a plain warrior almost unhittable from level 20 to 40.** The Shield
   category alone gives 14 to 16 Defence at levels 30 to 60: the large shield, plus shield block
   rank 10 (+6), plus sword and board rank 10 (+6). With Parry +6 and trained Dexterity, the
   total is Dv 33 to 36, against monster to-hit of +9 to +18. The monsters hit 1 to 5 % from level
   20 to 40 and 18 % at 60, and the warrior's rounds to die are in the hundreds. The design (§3)
   expected "dead in 5 → about 10" at level 60. Even untrained, the warrior is hit 5 % from 30 to
   40 (Dv 31 to 34). The levers are `shield` in `weapon_skills.toml` (`shield_block`,
   `sword_and_board`) and the shields' Defence.
2. **Trained Toughness doubles hit points.** A trained character has about twice §9.4's hit
   points at level 60 (warrior 400 against 215 untrained). Hit points per level are `class hp ÷
   2 + the Toughness modifier`, paid to `hp_level` 50, so every point of trained modifier pays up
   to 50 times. With note 1, the fighter's side of the fight is now far ahead from level 20.
   The levers are `hp_level` (`experience.toml`), the Toughness uses' chances (`stats.toml`), and
   its `[training]` prices ("Stat training").
3. **The thief in plain gear still falls behind late.** At Dv 25 and 340 hit points, the plain
   thief is hit 76 to 84 % at levels 50 to 60 and dies in 11 to 15 rounds, while killing in 9. Magic
   gear fixes it (hit 24 %). This is the report's finding 1 (gear decides), now limited to the
   light-armoured classes.
4. **The highest spell is level 12** (#754; the cap and the lift are gone since the spell
   scaling design's S1 and S2). A level-60 wizard's best burst is a level-11 or 12 spell at its
   top rung and evocation 10 (38.8 a cast); what grows a spell past that is higher-level
   content (`spells/*.toml`), not a lever in the schools. The cantrips now grow by the school
   (Ray of frost 4.5 at rank 0, 18.9 at evocation 10).
5. **The priest has no damage spell of its own.** Its schools (restoration, abjuration,
   divination) have no damage spell. Smite and its class actives are its offence, and the probe
   does not model them. Its restoration is measured on heals, one a kill (#754's "School pace"),
   which overstates its pace: rank 10 by level 20.
6. **Monster blows can drop with gear.** As in the report, some types deal less damage with the
   weapon they roll than with their natural attack. The geared mean of damage a blow is below the
   bare median from level 15 up (`gear.toml`, the creature files).

## Talents coverage

Every system the design added meets the coverage bar (`assets/data/talents.toml`, `min_talents`
1 since #773, checked when the game loads):
- `weapon_abilities` has 5 talents (Early mastery, Second blade's rend, Brace of steel, Practised
  technique, Drillmaster).
- `stamina` has 5 (Trained wind, Deep lungs, Steady breathing, Armour-breaker, Drillmaster;
  Second breath and Bellows were cut in #774).
- `stat_training` has 4 (Hard training, Late bloomer, Gourmand, Cauldron; #726; Hard-won and
  Gifted were cut in #774, Ascetic became a `hunger` talent in #775, and Quick study of the
  body, Hard living, and Discipline were cut in #787).
- `potions` has 2 (Thrifty drinker, Lasting draughts; Nose for draughts and Cauldron-wise were
  cut in #787).

No follow-up is needed.
