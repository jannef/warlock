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
which is #754's. Every table the probe prints was rerun for one rank per talent (#962, #963, #964,
#965; `docs/design/talent-ranks.md`), so prose under a table that quotes a figure from an earlier
PR's run may be a little off its cells.

## How to run it

```text
cargo test --release -p warlock-core balance_probe -- --ignored --nocapture
```

It takes about three minutes in release (most of it the stat training's sweep, a hundred and
seventy careers to level 60). It prints the Markdown tables of the sections from "Players" to
"Ki" below, and those of "Stat training", and the same content always prints the same tables (fixed seeds and streams).
After a PR that tunes combat data (weapon skills, creatures, classes, spells, stats, items), run
it again and replace those sections, then revisit the tuning notes. The code is
`crates/warlock-core/src/world/balance_probe.rs` (the measuring, with the method in its module
docs), `balance_probe/training.rs` (the trained careers' days), `balance_probe/ranger.rs`
(the ranger and their companion), `balance_probe/familiar.rs` (the witch's familiar),
`balance_probe/schools.rs` (the casters' school pace), and `balance_probe/tables.rs` (the tables).

## Method

- **Monsters**: every creature type with a level, shared and campaign, 400 in all. Each is made
  by `travel::make_creature`, once bare and once for each of six seeds with the gear its type
  rolls at its own level. Its side of a fight is `World::target_of` (Defence, Protection,
  resistances), its blow is `Actor::melee`, and its blows a round are its speed over its
  `World::attack_price`.
- **Finesse** (not yet in the tables below): the dagger, stiletto, short sword, and rapier aim with
  Dexterity instead of Strength. The thief's plain to-hit moved +1 at level 1, 0 at level 10, and
  -2 at level 60 (+18 to +16: they train more Strength than Dexterity), and the toughest
  weapon-carrying monsters lost under 1 to hit on average. The witch's dagger changes too.
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
  pool. The priest's prayer (#886; `docs/dev/prayer.md`) has no roll and no mana: its line a
  level gives the name the menu shows and what one prayer answers of the uses reached.
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
- **Rage** (#842): a human male berserker (great axe; axes, great blades, and the great weapon
  style at the warrior's ranks), measured for that table alone, calm and raging: the Rage pool
  granted (the dev grant: Raging and the stance it forces), and its `builds` and decay read from
  the content and the talents (Blood frenzy, Battle trance). Each foe's blows a round times the
  share of its strikes that `is_hit()` (soaked and resisted too) build `struck`; the berserker's
  blows times the share of `Strike::Hit { critical: true }` build `critical`; one over the
  rounds to kill builds `kill`.
- **Not modelled**: triggered and active weapon abilities (Rend, Riposte, Sunder, ...), stances (but the rage table's),
  sneak attacks outside the table above, a talent build, potions, resistances against spells, healing between rounds,
  and fights against several monsters at once.

## Players

Each cell: to-hit / damage a hit / Dv / Pv, the seeds' mean; trained stats.

| L | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|
| 1 | +3/8.2/16/3 | +3/8.2/16/3 | +1/3.0/13/1 | +1/3.0/13/1 | +0/3.0/12/0 | +1/5.8/12/0 | +1/5.8/12/0 |
| 5 | +4/9.2/18/4 | +5/10.9/20/4 | +3/4.3/15/1 | +3/5.3/17/2 | +0/3.0/13/0 | +2/7.0/15/2 | +3/7.8/18/2 |
| 10 | +7/11.7/22/5 | +8/12.9/25/6 | +6/7.7/20/1 | +7/9.2/21/2 | +2/4.0/15/0 | +6/10.5/17/4 | +8/12.9/20/5 |
| 15 | +8/13.0/25/9 | +12/16.3/28/10 | +9/8.8/21/2 | +11/11.3/24/4 | +2/4.0/16/1 | +8/12.5/21/5 | +10/16.5/25/5 |
| 20 | +10/15.0/28/10 | +14/20.8/33/12 | +11/10.7/23/2 | +14/12.1/27/4 | +2/5.0/18/1 | +10/14.0/22/7 | +12/16.4/27/8 |
| 30 | +12/17.3/33/12 | +21/24.4/41/18 | +13/12.7/23/2 | +20/20.3/30/8 | +3/6.3/20/1 | +11/16.3/24/8 | +19/22.1/32/15 |
| 40 | +14/21.3/36/12 | +22/27.8/45/16 | +15/14.7/25/2 | +24/19.0/32/5 | +4/7.3/21/1 | +13/20.5/27/8 | +22/26.9/36/12 |
| 50 | +15/22.3/36/12 | +20/28.2/46/17 | +16/16.2/25/2 | +22/22.7/31/7 | +4/7.5/22/1 | +14/21.3/27/8 | +20/29.2/34/13 |
| 60 | +16/22.8/36/12 | +22/28.8/45/15 | +16/16.8/25/2 | +24/21.9/32/7 | +4/7.5/23/1 | +14/21.7/28/8 | +22/26.8/37/13 |

Stats, hit points, and mana (plain gear; trained → untrained). Scores St/Dx/To/In/Wi/Pr, the seeds' mean.

| L | Warrior | Thief | Wizard | Priest | HP W/T/Wz/P | MP Wz/P | Caster level Wz/P | Blows a round W/T |
|---|---|---|---|---|---|---|---|---|
| 1 | 15/11/12/10/12/10 | 11/12/10/11/10/10 | 10/10/10/14/12/10 | 11/9/11/11/15/11 | 23→23 / 19→19 / 13→13 / 18→18 | 16/13 | 1/1 | 0.99/0.99 |
| 5 | 15/11/12/10/12/10 | 12/13/11/12/10/10 | 10/10/10/15/14/10 | 12/9/11/12/15/11 | 37→37 / 27→26 / 18→17 / 27→26 | 52/40 | 6/5 | 0.88/0.99 |
| 10 | 18/12/14/12/12/10 | 16/16/15/13/10/10 | 10/12/13/17/18/10 | 16/9/14/13/17/11 | 61→55 / 49→37 / 28→22 / 48→38 | 103/78 | 12/10 | 0.89/1.04 |
| 15 | 19/15/16/12/12/10 | 19/18/17/13/11/10 | 10/13/17/19/20/10 | 18/10/17/14/18/11 | 90→73 / 76→47 / 44→28 / 74→48 | 158/119 | 18/15 | 0.91/1.10 |
| 20 | 21/16/17/13/13/11 | 20/19/19/14/12/11 | 10/15/19/20/21/10 | 20/12/19/14/19/12 | 123→92 / 108→58 / 66→33 / 107→58 | 215/162 | 24/20 | 0.92/1.17 |
| 30 | 23/19/19/15/14/12 | 23/21/20/16/13/12 | 10/18/20/21/23/11 | 22/14/20/16/20/12 | 192→128 / 176→79 / 116→43 / 175→79 | 336/253 | 36/30 | 0.94/1.25 |
| 40 | 25/21/19/16/16/12 | 24/22/22/17/15/12 | 11/19/22/22/25/12 | 24/17/22/17/22/13 | 268→164 / 252→100 / 172→53 / 252→101 | 467/347 | 48/40 | 0.95/1.49 |
| 50 | 27/22/20/17/17/14 | 26/24/23/18/16/13 | 11/20/23/23/26/13 | 26/19/23/17/22/14 | 345→200 / 331→120 / 234→63 / 333→122 | 617/446 | 60/50 | 0.95/1.49 |
| 60 | 28/23/21/18/18/14 | 27/24/24/18/17/14 | 12/20/24/24/28/14 | 27/20/24/18/23/15 | 349→203 / 336→122 / 238→64 / 337→124 | 763/548 | 72/60 | 0.95/1.49 |

The Defence of the warrior and the thief in plain gear by category (trained → untrained), the seeds' mean.

| Class | L | Dexterity | Armour | Shield | Parry | Natural | Dodge | Deflection | Wall | total |
|---|---|---|---|---|---|---|---|---|---|---|
| warrior | 10 | 1→0 | 2→2 | 6→6 | 2→2 | 0→0 | 1→1 | 0→0 | 0→0 | 22→21 |
| warrior | 30 | 2→0 | 1→1 | 14→14 | 5→5 | 0→0 | 1→1 | 0→0 | 0→0 | 33→31 |
| warrior | 60 | 2→0 | 1→1 | 16→16 | 6→6 | 0→0 | 2→1 | 0→0 | 0→0 | 36→34 |
| thief | 10 | 3→1 | 2→2 | 0→0 | 3→3 | 0→0 | 2→2 | 0→0 | 0→0 | 20→17 |
| thief | 30 | 5→1 | 2→2 | 0→0 | 4→4 | 0→0 | 2→2 | 0→0 | 0→0 | 23→19 |
| thief | 60 | 5→1 | 2→2 | 0→0 | 6→6 | 0→0 | 2→2 | 0→0 | 0→0 | 25→21 |

## Monsters

Within two levels: the median (range) bare → the mean with gear. Damage is a hit's mean before soak.

| L (±2) | n | HP | to-hit | dmg/blow | Dv | Pv | blows a round |
|---|---|---|---|---|---|---|---|
| 1 | 54 | 11 | -0 (-5–2) → -0.5 | 2 (1–6) → 2.4 | 12 (7–14) → 11.3 | 0 (0–3) → 0.2 | 1.00 |
| 5 | 61 | 22 | 1 (-4–4) → 0.8 | 4 (1–11) → 4.3 | 12 (7–21) → 12.0 | 0 (0–2) → 0.4 | 1.00 |
| 10 | 54 | 39 | 2 (0–7) → 3.0 | 7 (2–16) → 8.5 | 11 (7–20) → 12.4 | 0 (0–5) → 0.7 | 1.00 |
| 15 | 46 | 57 | 4 (1–7) → 4.4 | 13 (4–20) → 12.2 | 12 (8–19) → 13.3 | 0 (0–7) → 1.1 | 1.00 |
| 20 | 47 | 73 | 5 (2–9) → 5.7 | 16 (4–22) → 14.5 | 13 (11–24) → 14.0 | 1 (0–6) → 1.6 | 1.00 |
| 30 | 34 | 117 | 9 (6–13) → 8.7 | 22 (10–27) → 20.9 | 16 (12–26) → 15.7 | 2 (0–6) → 2.8 | 1.02 |
| 40 | 27 | 143 | 11 (8–16) → 11.2 | 26 (11–32) → 23.0 | 17 (13–22) → 17.7 | 3 (0–11) → 3.4 | 1.07 |
| 50 | 23 | 228 | 14 (10–20) → 14.0 | 32 (16–40) → 29.4 | 19 (12–25) → 19.2 | 5 (0–13) → 4.3 | 1.05 |
| 60 | 17 | 224 | 17 (11–21) → 16.5 | 34 (18–41) → 30.9 | 21 (17–29) → 21.3 | 6 (0–16) → 6.3 | 1.16 |
| 70 | 7 | 197 | 20 (17–21) → 19.3 | 32 (22–40) → 31.5 | 26 (21–29) → 25.5 | 5 (0–11) → 4.5 | 1.35 |

## Fights

Each cell: P(player hits) / P(monster hits) % · damage a round player : monster · rounds to kill / rounds to die; the median over the monsters within two levels (trained stats, monsters with gear).

| L | Warrior kit | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain (melee) | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|---|
| 1 | 81/35 · 7.4:0.1 · 1.6 / 285 | 81/35 · 7.3:0.1 · 1.6 / 341 | 81/35 · 7.4:0.1 · 1.6 / 314 | 66/55 · 2.7:0.6 · 5.4 / 33 | 66/55 · 2.7:0.6 · 5.4 / 32 | 65/62 · 2.1:1.1 · 5.8 / 13 | 71/59 · 4.4:1.0 · 2.8 / 20 | 71/59 · 4.4:1.1 · 2.8 / 19 |
| 5 | 86/34 · 8.4:0.9 · 2.8 / 46 | 86/27 · 7.3:0.6 · 3.3 / 75 | 89/15 · 9.3:0.3 · 2.6 / 185 | 78/48 · 4.3:1.8 · 6.1 / 16 | 80/33 · 5.3:1.2 · 4.9 / 43 | 65/62 · 1.9:2.7 · 14 / 6.8 | 78/43 · 4.3:1.5 · 5.2 / 19 | 81/29 · 5.3:0.9 · 4.3 / 43 |
| 10 | 92/21 · 11.9:1.2 · 3.6 / 61 | 92/16 · 10.5:0.7 · 4.2 / 106 | 94/8 · 11.5:0.3 · 3.8 / >999 | 88/29 · 11.2:1.9 · 4.9 / 30 | 93/20 · 13.9:1.3 · 4.0 / 75 | 66/58 · 2.6:4.4 · 18 / 6.4 | 90/44 · 8.5:2.4 · 5.1 / 22 | 97/27 · 11.7:1.3 · 3.8 / 77 |
| 15 | 94/9 · 13.4:0.7 · 4.6 / 259 | 94/9 · 11.6:0.4 · 5.4 / 335 | 99/4 · 16.6:0.2 · 4.0 / >999 | 96/29 · 14.1:3.0 · 5.5 / 29 | 98/14 · 19.7:1.2 · 4.7 / 114 | 62/64 · 2.0:7.4 · 30 / 6.1 | 94/31 · 10.5:2.5 · 6.6 / 38 | 97/15 · 15.4:1.1 · 4.3 / 969 |
| 20 | 98/3 · 16.9:0.3 · 5.0 / >999 | 98/5 · 14.7:0.3 · 5.7 / 962 | 99/1 · 23.6:0.0 · 3.7 / >999 | 99/29 · 19.1:3.3 · 5.5 / 46 | 99/11 · 23.8:1.1 · 4.4 / 570 | 67/60 · 2.6:7.3 · 33 / 9.2 | 97/33 · 11.6:2.5 · 6.9 / 51 | 99/10 · 15.9:0.6 · 5.0 / >999 |
| 30 | 98/1 · 20.4:0.2 · 7.0 / >999 | 98/1 · 18.0:0.1 · 8.0 / >999 | 99/1 · 30.3:0.1 · 5.0 / >999 | 99/40 · 24.9:9.2 · 6.5 / 21 | 99/7 · 45.0:1.0 · 3.3 / 566 | 56/63 · 2.1:14.3 · 53 / 8.2 | 95/33 · 12.3:6.0 · 11 / 31 | 99/5 · 23.8:0.5 · 6.1 / >999 |
| 40 | 98/1 · 26.4:0.1 · 6.8 / >999 | 98/1 · 22.5:0.2 · 7.9 / >999 | 99/1 · 39.9:0.1 · 4.7 / >999 | 99/47 · 33.3:10.4 · 6.0 / 26 | 99/8 · 57.3:1.6 · 3.6 / >999 | 50/76 · 2.5:15.1 · 66 / 12 | 96/34 · 16.3:6.6 · 11 / 42 | 99/1 · 29.6:0.1 · 5.7 / >999 |
| 50 | 98/2 · 27.5:0.4 · 8.6 / >999 | 98/5 · 23.2:1.2 · 10 / 715 | 99/1 · 36.6:0.1 · 6.6 / >999 | 99/68 · 35.0:19.6 · 7.4 / 18 | 99/29 · 49.3:7.0 · 4.7 / 569 | 42/83 · 1.8:21.7 · 150 / 11 | 95/56 · 17.1:14.7 · 14 / 23 | 99/14 · 32.0:2.8 · 7.1 / 664 |
| 60 | 94/3 · 20.2:1.1 · 10 / 803 | 94/13 · 17.7:3.0 · 12 / 153 | 99/1 · 27.5:0.2 · 7.0 / >999 | 95/84 · 27.2:23.8 · 9.6 / 14 | 99/38 · 39.3:9.7 · 6.1 / 76 | 28/94 · 1.1:27.9 · 454 / 8.8 | 90/71 · 14.6:16.6 · 16 / 22 | 99/14 · 24.9:3.5 · 9.6 / 832 |

Out of depth (+3: the monsters one to five levels above), and untrained stats, for the warrior in plain gear.

| L | Warrior plain +3 | Warrior magic +3 | Warrior plain, untrained | Warrior magic, untrained |
|---|---|---|---|---|
| 1 | 81/42 · 7.2:0.8 · 2.8 / 31 | 81/42 · 7.2:0.8 · 2.8 / 32 | 81/35 · 7.4:0.1 · 1.7 / 311 | 81/35 · 7.4:0.1 · 1.6 / 327 |
| 5 | 86/34 · 7.3:1.5 · 4.9 / 28 | 88/20 · 9.2:0.8 · 3.9 / 66 | 86/27 · 7.3:0.5 · 3.3 / 79 | 89/15 · 9.3:0.3 · 2.7 / 197 |
| 10 | 92/25 · 9.9:1.7 · 5.9 / 45 | 94/13 · 11.0:0.7 · 5.3 / 688 | 87/19 · 8.7:0.8 · 5.2 / 74 | 89/9 · 9.7:0.4 · 4.5 / >999 |
| 15 | 94/11 · 11.7:0.8 · 6.1 / 149 | 98/5 · 16.5:0.3 · 4.5 / >999 | 88/13 · 8.9:0.6 · 7.3 / 161 | 98/6 · 14.3:0.3 · 4.7 / >999 |
| 20 | 96/8 · 14.6:0.6 · 7.7 / 297 | 99/1 · 22.6:0.1 · 5.0 / >999 | 91/7 · 10.0:0.4 · 9.0 / 470 | 97/1 · 20.4:0.0 · 4.5 / >999 |
| 30 | 98/3 · 16.6:0.4 · 8.9 / >999 | 99/1 · 28.6:0.1 · 5.5 / >999 | 87/5 · 10.5:0.6 · 13 / 518 | 99/1 · 24.7:0.1 · 6.3 / >999 |
| 40 | 99/2 · 22.8:0.2 · 8.6 / >999 | 99/1 · 40.0:0.1 · 5.1 / >999 | 81/3 · 12.7:0.7 · 13 / 830 | 97/1 · 29.1:0.1 · 6.3 / >999 |
| 50 | 97/8 · 20.9:1.6 · 10 / 554 | 99/1 · 33.8:0.1 · 6.8 / >999 | 76/12 · 10.7:2.5 · 22 / 124 | 92/1 · 24.0:0.2 · 10 / >999 |
| 60 | 92/20 · 16.9:6.1 · 14 / 59 | 98/1 · 25.7:0.2 · 9.3 / >999 | 63/27 · 7.0:5.8 · 40 / 50 | 90/1 · 16.8:0.3 · 15 / >999 |

## Protection by body

Rounds for the plain warrior (trained) to kill the monsters of their level: as shipped, then with every monster's Protection set to §8.2's body line at its level.

| L | as shipped | none (0) | hide | bony | armoured |
|---|---|---|---|---|---|
| 10 | 4.1 | 3.7 | 3.7 | 4.1 | 4.7 |
| 15 | 5.3 | 4.6 | 4.6 | 5.0 | 6.1 |
| 20 | 5.7 | 4.8 | 5.0 | 5.5 | 6.1 |
| 30 | 8.0 | 6.1 | 6.5 | 7.2 | 8.2 |
| 40 | 7.8 | 5.6 | 6.2 | 6.7 | 7.6 |
| 50 | 10 | 8.2 | 9.1 | 10 | 12 |
| 60 | 12 | 6.8 | 7.9 | 8.5 | 10 |

## Spells

Each spell that rolls (harm or healing) of the schools the trained caster knows or has an affinity with, as `Actor::spell_roll` makes it in plain gear, per cast on one target: the rung of its ladder the stats reach (the seeds' range), the school's rank the pace below reached (and the rank of the caster style it is cast in from where the caster stands), the roll (seed 1's dice, the seeds' mean) and the same roll at rank 0 (the ladder alone), its mean on a target under a hex (Malison's bite; `–` without it), and its mana (the cost table's price with no rank's cut in brackets) over the pool.

**Wizard**

| Spell (level, res) | L | rung | school rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Ray of frost (1, attack) | 10 | 1 | 5 (Sniper 5–6) | 1d9 1s again, 6.4 (rank 0: 1d8, 4.5) | – | 1 (1) / 103 |
| Ray of frost (1, attack) | 20 | 1 | 7 (Sniper 7) | 2d9 1s again, 11.0 (rank 0: 1d8, 4.5) | – | 1 (1) / 215 |
| Ray of frost (1, attack) | 30 | 1 | 8 (Sniper 8–9) | 2d9 1s and 2s again, 12.5 (rank 0: 1d8, 4.5) | – | 1 (1) / 336 |
| Ray of frost (1, attack) | 60 | 1 | 10 (Sniper 10) | 3d10 1s and 2s again, 18.9 (rank 0: 1d8, 4.5) | – | 1 (1) / 763 |
| Spark (1, auto) | 10 | 1 | 5 (Sniper 5–6) | 1d7 1s again, 5.2 (rank 0: 1d6, 3.5) | – | 1 (1) / 103 |
| Spark (1, auto) | 20 | 1 | 7 (Sniper 7) | 2d7 1s again, 9.0 (rank 0: 1d6, 3.5) | – | 1 (1) / 215 |
| Spark (1, auto) | 30 | 1 | 8 (Sniper 8–9) | 2d7 1s and 2s again, 10.2 (rank 0: 1d6, 3.5) | – | 1 (1) / 336 |
| Spark (1, auto) | 60 | 1 | 10 (Sniper 10) | 3d8 1s and 2s again, 15.8 (rank 0: 1d6, 3.5) | – | 1 (1) / 763 |
| Magic missile (3, auto) | 10 | 2 | 5 (Sniper 5–6) | 3d5 1s again, 10.8 (rank 0: 3d4, 7.5) | – | 2 (3) / 103 |
| Magic missile (3, auto) | 20 | 2 | 7 (Sniper 7) | 4d5 1s again, 13.7 (rank 0: 3d4, 7.5) | – | 2 (3) / 215 |
| Magic missile (3, auto) | 30 | 2–3 | 8 (Sniper 8–9) | 4d5 1s and 2s again, 15.3 (rank 0: 3d4, 7.8) | – | 2 (3) / 336 |
| Magic missile (3, auto) | 60 | 3 | 10 (Sniper 10) | 5d6+1 1s and 2s again, 21.8 (rank 0: 3d4+1, 8.5) | – | 1 (3) / 763 |
| Burning hands (4, area) | 10 | 2 | 5 (Sniper 5–6) | 3d5 1s again, 10.8 (rank 0: 3d4, 7.5) | – | 5 (6) / 103 |
| Burning hands (4, area) | 20 | 2 | 7 (Sniper 7) | 4d5 1s again, 13.7 (rank 0: 3d4, 7.5) | – | 5 (6) / 215 |
| Burning hands (4, area) | 30 | 2–3 | 8 (Sniper 8–9) | 4d5 1s and 2s again, 15.7 (rank 0: 3d4, 8.2) | – | 5 (6) / 336 |
| Burning hands (4, area) | 60 | 3 | 10 (Sniper 10) | 5d6+2 1s and 2s again, 22.8 (rank 0: 3d4+2, 9.5) | – | 1 (6) / 763 |
| Cause wounds (4, touch) | 10 | 1–2 | 0 (Toucher 1–2) | 2d8, 7.7 (rank 0: 2d8, 7.7) | – | 3 (3) / 103 |
| Cause wounds (4, touch) | 20 | 2 | 0 (Toucher 3) | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 3 (3) / 215 |
| Cause wounds (4, touch) | 30 | 2–3 | 0 (Toucher 4–5) | 2d8+1, 10.3 (rank 0: 2d8, 9.3) | – | 3 (3) / 336 |
| Cause wounds (4, touch) | 60 | 3 | 0 (Toucher 6) | 2d8+3, 12.0 (rank 0: 2d8+2, 11.0) | – | 1 (3) / 763 |
| Call of the wild (5, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 103 |
| Call of the wild (5, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 215 |
| Call of the wild (5, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 336 |
| Call of the wild (5, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 2 (5) / 763 |
| Arcane bolt (6, area) | 10 | 1–2 | 5 (Sniper 5–6) | 3d7 1s again, 14.4 (rank 0: 3d6, 10.8) | – | 10 (11) / 103 |
| Arcane bolt (6, area) | 20 | 1–2 | 7 (Sniper 7) | 4d7+2 1s again, 19.6 (rank 0: 3d6+2, 12.2) | – | 9 (11) / 215 |
| Arcane bolt (6, area) | 30 | 2 | 8 (Sniper 8–9) | 4d7+2 1s and 2s again, 21.6 (rank 0: 3d6+2, 12.5) | – | 9 (11) / 336 |
| Arcane bolt (6, area) | 60 | 3 | 10 (Sniper 10) | 5d8+4 1s and 2s again, 30.2 (rank 0: 3d6+4, 14.5) | – | 2 (11) / 763 |
| Frost (6, area) | 10 | 1–2 | 5 (Sniper 5–6) | 3d7 1s again, 14.4 (rank 0: 3d6, 10.8) | – | 13 (14) / 103 |
| Frost (6, area) | 20 | 1–2 | 7 (Sniper 7) | 4d7+2 1s again, 19.6 (rank 0: 3d6+2, 12.2) | – | 12 (14) / 215 |
| Frost (6, area) | 30 | 2 | 8 (Sniper 8–9) | 4d7+2 1s and 2s again, 21.6 (rank 0: 3d6+2, 12.5) | – | 12 (14) / 336 |
| Frost (6, area) | 60 | 3 | 10 (Sniper 10) | 5d8+4 1s and 2s again, 30.2 (rank 0: 3d6+4, 14.5) | – | 2 (14) / 763 |
| Blight (7, area) | 10 | 1 | 0 (Sniper 5–6) | 3d6, 10.5 (rank 0: 3d6, 10.5) | – | 17 (17) / 103 |
| Blight (7, area) | 20 | 1–2 | 0 (Sniper 7) | 3d6+3, 12.0 (rank 0: 3d6+3, 12.0) | – | 17 (17) / 215 |
| Blight (7, area) | 30 | 1–2 | 0 (Sniper 8–9) | 3d6+3, 13.0 (rank 0: 3d6+3, 13.0) | – | 17 (17) / 336 |
| Blight (7, area) | 60 | 2 | 0 (Sniper 10) | 3d6+3, 13.5 (rank 0: 3d6+3, 13.5) | – | 5 (17) / 763 |
| Scorching ray (7, attack) | 10 | 2–3 | 5 (Sniper 5–6) | 3d7 1s again, 14.8 (rank 0: 3d6, 11.1) | – | 6 (7) / 103 |
| Scorching ray (7, attack) | 20 | 2–3 | 7 (Sniper 7) | 5d7 1s again, 21.6 (rank 0: 4d6, 13.4) | – | 6 (7) / 215 |
| Scorching ray (7, attack) | 30 | 3 | 8 (Sniper 8–9) | 5d7 1s and 2s again, 24.4 (rank 0: 4d6, 14.0) | – | 6 (7) / 336 |
| Scorching ray (7, attack) | 60 | 4 | 10 (Sniper 10) | 5d8+1d12 1s and 2s again, 33.6 (rank 0: 3d6+1d10, 16.0) | – | 1 (7) / 763 |
| Call lightning (11, area) | 10 | 2–3 | 5 (Sniper 5–6) | 5d7 1s again, 19.9 (rank 0: 5d6, 15.2) | – | 19 (20) / 103 |
| Call lightning (11, area) | 20 | 3 | 7 (Sniper 7) | 6d7 1s again, 26.9 (rank 0: 5d6, 17.5) | – | 18 (20) / 215 |
| Call lightning (11, area) | 30 | 3–4 | 8 (Sniper 8–9) | 6d7 1s and 2s again, 29.4 (rank 0: 5d6, 17.8) | – | 18 (20) / 336 |
| Call lightning (11, area) | 60 | 4 | 10 (Sniper 10) | 7d8+2 1s and 2s again, 38.8 (rank 0: 5d6+2, 19.5) | – | 4 (20) / 763 |
| Fireball (11, area) | 10 | 2–3 | 5 (Sniper 5–6) | 4d7 1s again, 19.2 (rank 0: 4d6, 14.6) | – | 24 (26) / 103 |
| Fireball (11, area) | 20 | 2–3 | 7 (Sniper 7) | 6d7 1s again, 26.1 (rank 0: 5d6, 16.9) | – | 23 (26) / 215 |
| Fireball (11, area) | 30 | 3 | 8 (Sniper 8–9) | 6d7 1s and 2s again, 29.1 (rank 0: 5d6, 17.5) | – | 23 (26) / 336 |
| Fireball (11, area) | 60 | 4 | 10 (Sniper 10) | 7d8+2 1s and 2s again, 38.8 (rank 0: 5d6+2, 19.5) | – | 5 (26) / 763 |
| Lightning bolt (11, area) | 10 | 2–3 | 5 (Sniper 5–6) | 4d7 1s again, 19.2 (rank 0: 4d6, 14.6) | – | 19 (20) / 103 |
| Lightning bolt (11, area) | 20 | 2–3 | 7 (Sniper 7) | 6d7 1s again, 26.1 (rank 0: 5d6, 16.9) | – | 18 (20) / 215 |
| Lightning bolt (11, area) | 30 | 3 | 8 (Sniper 8–9) | 6d7 1s and 2s again, 29.1 (rank 0: 5d6, 17.5) | – | 18 (20) / 336 |
| Lightning bolt (11, area) | 60 | 4 | 10 (Sniper 10) | 7d8+2 1s and 2s again, 38.8 (rank 0: 5d6+2, 19.5) | – | 4 (20) / 763 |
| Flame (12, area) | 10 | 1–2 | 5 (Sniper 5–6) | 4d7 1s again, 19.2 (rank 0: 4d6, 14.6) | – | 27 (29) / 103 |
| Flame (12, area) | 20 | 1–2 | 7 (Sniper 7) | 6d7 1s again, 26.1 (rank 0: 5d6, 16.9) | – | 26 (29) / 215 |
| Flame (12, area) | 30 | 2 | 8 (Sniper 8–9) | 6d7 1s and 2s again, 29.1 (rank 0: 5d6, 17.5) | – | 26 (29) / 336 |
| Flame (12, area) | 60 | 3 | 10 (Sniper 10) | 7d8+2 1s and 2s again, 38.8 (rank 0: 5d6+2, 19.5) | – | 6 (29) / 763 |
| Call of the forest (20, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 103 |
| Call of the forest (20, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 215 |
| Call of the forest (20, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 336 |
| Call of the forest (20, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 6 (20) / 763 |
| Call the elements (35, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 103 |
| Call the elements (35, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 215 |
| Call the elements (35, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 336 |
| Call the elements (35, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 11 (35) / 763 |

**Priest**

| Spell (level, res) | L | rung | school rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Mend (1, touch) | 10 | 0–1 | 7 (Battle caster 4–5) | 2d5+2 1s again, 8.6 (rank 0: 1d4, 2.4) | – | 1 (1) / 78 |
| Mend (1, touch) | 20 | 1–2 | 10 (Battle caster 10) | 2d6+3 1s and 2s again, 11.7 (rank 0: 1d4, 2.7) | – | 1 (1) / 162 |
| Mend (1, touch) | 30 | 1–2 | 10 (Battle caster 10) | 2d6+3 1s and 2s again, 11.7 (rank 0: 1d4, 2.7) | – | 1 (1) / 253 |
| Mend (1, touch) | 60 | 1–2 | 10 (Battle caster 10) | 2d8+3 1s and 2s again, 13.1 (rank 0: 1d6, 3.3) | – | 1 (1) / 548 |
| Cure wounds (3, touch) | 10 | 0–1 | 7 (Battle caster 4–5) | 4d7+2 1s again, 19.0 (rank 0: 3d6, 9.9) | – | 1 (2) / 78 |
| Cure wounds (3, touch) | 20 | 1–2 | 10 (Battle caster 10) | 4d8+3 1s and 2s again, 24.5 (rank 0: 3d6, 11.0) | – | 2 (2) / 162 |
| Cure wounds (3, touch) | 30 | 1–2 | 10 (Battle caster 10) | 4d8+3 1s and 2s again, 24.5 (rank 0: 3d6, 11.0) | – | 2 (2) / 253 |
| Cure wounds (3, touch) | 60 | 1–2 | 10 (Battle caster 10) | 4d8+6 1s and 2s again, 26.5 (rank 0: 3d6+3, 13.0) | – | 2 (2) / 548 |
| Orison (prayer) | 10 | – | – | no roll: answers 1 of 2 (wounds, hunger) | – | 0 (0) / 78 |
| Prayer (prayer) | 20 | – | – | no roll: answers 1 of 5 (disease, poison, wounds, release, hunger) | – | 0 (0) / 162 |
| Litany (prayer) | 30 | – | – | no roll: answers 1 of 6 (disease, poison, wounds, release, hunger, restoration) | – | 0 (0) / 253 |
| Miracle (prayer) | 60 | – | – | no roll: answers all of 7 (disease, poison, wounds, release, mana, hunger, restoration) | – | 0 (0) / 548 |

The three tables are #905's run (Malison's hexed column: a damage spell's mean on a target under one hex, from the witch's level 18), on `main` after W1 (#908) and Z1 (#909).

**Witch**

| Spell (level, res) | L | rung | school rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Cause wounds (4, touch) | 10 | 1–2 | 6–7 (Toucher 6–7) | 2d6+3 1s again, 12.0 (rank 0: 2d6, 8.0) | – | 2 (3) / 90 |
| Cause wounds (4, touch) | 20 | 2 | 7–9 (Toucher 6–9) | 2d8+3 1s again, 15.5 (rank 0: 2d8, 9.0) | 18.5 | 2 (3) / 197 |
| Cause wounds (4, touch) | 30 | 2–3 | 9–10 (Toucher 6–10) | 3d8+4 1s and 2s again, 21.4 (rank 0: 2d8, 9.7) | 24.4 | 2 (3) / 312 |
| Cause wounds (4, touch) | 60 | 3 | 10 (Toucher 6–10) | 3d8+7 1s and 2s again, 23.4 (rank 0: 2d8+2, 11.0) | 26.4 | 2 (3) / 718 |
| Call of the wild (5, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 90 |
| Call of the wild (5, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 197 |
| Call of the wild (5, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 312 |
| Call of the wild (5, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 4 (5) / 718 |
| Blight (7, area) | 10 | 1 | 6–7 (Sniper 2–4) | 3d6+2 1s again, 13.8 (rank 0: 3d6, 10.5) | – | 16 (17) / 90 |
| Blight (7, area) | 20 | 1–2 | 7–9 (Sniper 5–7) | 3d6+5 1s again, 17.7 (rank 0: 3d6+3, 12.5) | 20.7 | 15 (17) / 197 |
| Blight (7, area) | 30 | 2 | 9–10 (Sniper 6–9) | 4d6+6 1s and 2s again, 23.0 (rank 0: 3d6+3, 13.5) | 26.0 | 15 (17) / 312 |
| Blight (7, area) | 60 | 2 | 10 (Sniper 9–10) | 4d6+7 1s and 2s again, 23.7 (rank 0: 3d6+3, 13.5) | 26.7 | 11 (17) / 718 |
| Call of the forest (20, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 90 |
| Call of the forest (20, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 197 |
| Call of the forest (20, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 312 |
| Call of the forest (20, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 16 (20) / 718 |
| Call the elements (35, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 90 |
| Call the elements (35, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 197 |
| Call the elements (35, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 312 |
| Call the elements (35, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 28 (35) / 718 |


## School pace

The trained caster's school and caster style as their casting trained them (`balance_probe/schools.rs`): each day of each level, one pool spent on the pace spell (of the school's spells, the most a cast among those costing at most a fifth of the pool, never one cast only under the open sky; for the priest a heal, at most one a kill), every cast landing on a monster of the caster's level through `World::practise`. Each row: the pace spell over the level before and its cost (seed 1's); casts a day from the pool, and the spell casts a day the same level's fights took (the stat training's careers, which refill the pool before every cast; the priest's one a kill); the school's rank and the caster style's at the level (the seeds' range); and the pool check, the pool over a single-target spell of the caster's own level (its base mana: the level; no rank's cut, no talent).

**Wizard** (evocation)

| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |
|---|---|---|---|---|---|
| 1 | – | 0.0 / 0.0 | 2 | – | 16 / 1 = 16.5 |
| 5 | Scorching ray (7) | 6.0 / 154.7 | 3 | Sniper 3 | 52 / 5 = 10.4 |
| 10 | Scorching ray (6) | 11.3 / 219.6 | 5 | Sniper 5–6 | 103 / 10 = 10.3 |
| 15 | Flame (27) | 5.2 / 277.8 | 6 | Sniper 6–7 | 158 / 15 = 10.5 |
| 20 | Flame (26) | 7.5 / 250.0 | 7 | Sniper 7 | 215 / 20 = 10.8 |
| 30 | Flame (26) | 12.2 / 385.6 | 8 | Sniper 8–9 | 336 / 30 = 11.2 |
| 40 | Flame (23) | 20.0 / 343.5 | 9 | Sniper 9–10 | 467 / 40 = 11.7 |
| 50 | Flame (21) | 29.2 / 462.0 | 10 | Sniper 10 | 617 / 50 = 12.3 |
| 60 | Flame (17) | 45.0 / 550.5 | 10 | Sniper 10 | 763 / 60 = 12.7 |

**Priest** (restoration)

| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |
|---|---|---|---|---|---|
| 1 | – | 0.0 / 0.0 | 1 | – | 13 / 1 = 13.0 |
| 5 | Cure wounds (2) | 16.2 / 42.0 | 3–4 | Toucher 3–4 | 40 / 5 = 7.9 |
| 10 | Cure wounds (1) | 68.3 / 73.0 | 7 | Toucher 7–8 | 78 / 10 = 7.8 |
| 15 | Cure wounds (1) | 97.4 / 98.5 | 9 | Battle caster 8–9 | 119 / 15 = 7.9 |
| 20 | Cure wounds (1) | 74.3 / 74.3 | 10 | Battle caster 10 | 162 / 20 = 8.1 |
| 30 | Cure wounds (2) | 88.0 / 88.0 | 10 | Battle caster 10 | 253 / 30 = 8.4 |
| 40 | Cure wounds (2) | 75.0 / 75.0 | 10 | Battle caster 10 | 347 / 40 = 8.7 |
| 50 | Cure wounds (2) | 83.2 / 83.2 | 10 | Battle caster 10 | 446 / 50 = 8.9 |
| 60 | Cure wounds (2) | 91.2 / 91.2 | 10 | Battle caster 10 | 548 / 60 = 9.1 |

**Witch** (necromancy)

| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |
|---|---|---|---|---|---|
| 1 | – | 0.0 / 0.0 | 1 | – | 14 / 1 = 13.7 |
| 5 | Cause wounds (3) | 11.7 / 127.8 | 3 | Toucher 3–4 | 43 / 5 = 8.7 |
| 10 | Blight (16) | 21.3 / 237.0 | 6–7 | Sniper 3–7 | 90 / 10 = 9.0 |
| 15 | Blight (15) | 26.5 / 396.9 | 7–8 | Sniper 5–9 | 142 / 15 = 9.5 |
| 20 | Blight (15) | 23.7 / 337.6 | 7–9 | Sniper 7–9 | 197 / 20 = 9.8 |
| 30 | Blight (15) | 42.7 / 417.8 | 9–10 | Sniper 6–10 | 312 / 30 = 10.4 |
| 40 | Blight (14) | 62.8 / 467.7 | 10 | Sniper 9–10 | 437 / 40 = 10.9 |
| 50 | Blight (14) | 119.8 / 639.1 | 10 | Sniper 10 | 576 / 50 = 11.5 |
| 60 | Blight (11) | 156.7 / 765.3 | 10 | Sniper 10 | 718 / 60 = 12.0 |

## Sneak attacks

The thief's main-hand blow on the monsters of their level (trained stats), aware and caught unawares. The Sneak attack value assumed, the dice it gives with the talents, and the share of the monsters within two levels with vitals (no dice and no critical on the rest, so their first blow gains only the flat-footed Defence); then each cell: P(hit) aware / unaware % · damage of a blow aware : the first blow unaware · how many times as much, the median over the monsters within two levels that have vitals.

| L | Sneak attack | dice | with vitals | Thief plain | Thief magic |
|---|---|---|---|---|---|
| 1 | 15 | 1d6 | 78% | 72/78 · 2.3:5.8 · ×2.5 | 72/78 · 2.4:5.8 · ×2.5 |
| 5 | 29 | 2d6 | 59% | 78/88 · 3.8:10.3 · ×2.7 | 80/89 · 4.7:11.5 · ×2.4 |
| 10 | 47 | 2d6 | 72% | 92/97 · 7.9:15.2 · ×1.9 | 95/98 · 9.8:17.1 · ×1.8 |
| 15 | 64 | 3d6 | 63% | 99/99 · 9.9:20.5 · ×2.1 | 99/99 · 12.9:23.3 · ×1.8 |
| 20 | 82 | 4d6 | 66% | 99/99 · 11.7:25.7 · ×2.2 | 99/99 · 14.4:28.3 · ×2.0 |
| 30 | 117 | 5d6 | 62% | 99/99 · 13.6:31.2 · ×2.3 | 99/99 · 23.9:41.3 · ×1.7 |
| 40 | 152 | 8d6 | 59% | 99/99 · 16.0:43.8 · ×2.7 | 99/99 · 22.5:50.6 · ×2.3 |
| 50 | 187 | 8d6 | 52% | 99/99 · 16.5:44.2 · ×2.7 | 99/99 · 26.9:54.6 · ×2.0 |
| 60 | 222 | 9d6 | 47% | 97/99 · 16.6:50.4 · ×3.0 | 99/99 · 24.4:58.9 · ×2.4 |

## Rage

The berserker in plain gear (a great axe; from level 5 the warrior's armour: chain mail, banded mail from 15, plate from 30), trained stats, against the monsters within two levels: calm (the stance the run starts in) and raging (Raging and the frenzy stance). The pool as the Rage ability grants it (start / most); the fight cells as in "Fights"; then the rage a round against one foe, the medians of hits taken × their build + criticals × theirs + kills × theirs − the decay (kills a round: one over the rounds to kill, so a fight's average); the same without the kills (a foe that outlasts the rage, or the turns before the first kill) and how many rounds the rage lasts from its start at that net (`grows` when it does not fall); and the net against three foes (three foes' hits, the same criticals and kills).

| L | rage | Calm | Raging | one foe: struck + crit + kill − decay = net | one foe that does not die: net · rounds the rage lasts | three foes: net |
|---|---|---|---|---|---|---|
| 1 | 12/20 | 83/71 · 8.2:0.8 · 1.5 / 34 | 94/96 · 13.6:1.1 · 0.9 / 27 | 0.99 + 0.07 + 3.47 − 1.00 = +3.53 | +0.06 · grows | +5.50 |
| 5 | 12/20 | 87/64 · 9.2:0.9 · 2.6 / 50 | 97/94 · 14.2:1.5 · 1.7 / 34 | 0.94 + 0.06 + 1.85 − 1.00 = +1.85 | -0.00 · >999 | +3.72 |
| 10 | 12/20 | 89/74 · 13.5:3.0 · 3.2 / 28 | 97/97 · 18.9:2.7 · 2.2 / 31 | 0.97 + 0.07 + 1.34 − 1.00 = +1.37 | +0.04 · grows | +3.31 |
| 15 | 12/20 | 92/84 · 16.5:4.7 · 3.8 / 25 | 98/99 · 21.9:4.4 · 2.9 / 28 | 0.99 + 0.08 + 1.08 − 1.00 = +1.14 | +0.06 · grows | +3.12 |
| 20 | 12/20 | 96/90 · 24.8:6.0 · 3.4 / 27 | 99/99 · 32.6:6.2 · 2.6 / 26 | 0.99 + 0.13 + 1.19 − 1.00 = +1.31 | +0.12 · grows | +3.29 |
| 30 | 12/20 | 93/96 · 28.8:15.0 · 4.6 / 16 | 98/99 · 38.2:13.7 · 3.9 / 18 | 0.99 + 0.14 + 0.79 − 0.94 = +0.98 | +0.19 · grows | +2.97 |
| 40 | 18/20 | 97/99 · 49.1:14.9 · 3.7 / 24 | 99/99 · 78.2:14.0 · 2.4 / 26 | 1.06 + 0.29 + 3.37 − 0.67 = +4.05 | +0.68 · grows | +6.16 |
| 50 | 18/30 | 97/99 · 57.3:22.3 · 4.2 / 22 | 99/99 · 94.1:21.0 · 2.5 / 23 | 1.04 + 0.34 + 3.19 − 0.67 = +3.90 | +0.71 · grows | +5.99 |
| 60 | 18/30 | 91/99 · 40.6:23.9 · 6.0 / 27 | 99/99 · 64.1:22.9 · 3.2 / 28 | 1.14 + 0.00 + 2.38 − 0.67 = +2.85 | +0.47 · grows | +5.12 |

This section is #842's run (the rage pool's builds); the sections above are still #754's. Against
the same table before #842 (the pool fed by nothing but Blood frenzy's kills from 40): one foe's
net was −1.00 a round at every level to 30 (the rage lasted its twelve rounds, whatever the
fight), +0.69 at 40, +0.84 at 50, and +0.62 at 60; without the kills −1.00 (12 rounds) to 30,
−0.89 (13) at 40, and −0.67 (27) at 50 and 60; and three foes fed it no more than one. The calm
and raging fight cells are unchanged (#842 changes no blow). What the run shows:

- **Hits taken alone hold the rage.** A raging berserker's Dodge −6 lets the monsters of their
  level land 93 to 99 % of their blows (the berserker design's arithmetic assumed about six in
  ten), so at their one blow a round `struck` is worth 0.9 to 1.2 rage a round: about the
  decay. A duel with a foe that will not die holds the rage level (−0.03 to +0.13 a round to 30;
  it falls only at 5, over 431 rounds), so the design's "a duel of twelve rage lasts about thirty
  turns" is, by these numbers, "a duel's rage lasts as long as the foe does". From 40 Blood
  frenzy and, where the picks took it, Battle trance's slower decay make it grow.
- **Kills feed it most** while the fight is short: raging, the berserker kills a monster of their
  level in one to four rounds, so +3 a kill is 0.8 to 3.5 rage a round over the fight.
- **Criticals are a garnish**: 0.06 to 0.25 a round (none at 60, where the median monster of the
  window takes none).
- **A crowd builds it**: three foes put +2.9 to +5.5 a round into it, so a fight against several
  ends at the cap (20; 30 with Mighty rage), and the rage burns out over 20 rounds after it (45
  with Mighty rage and Battle trance's slower decay), then Exhausted's 150 s (75 with Mighty
  rage).

If a duel's rage should run out mid-fight, the levers are `struck`'s amount or a faster decay;
#842 keeps the design's numbers.

## Ki

The ascetic bare-handed (a brawler; from level 5 leather gloves, boots, and a wool cloak, no armour), trained stats, against the monsters within two levels. The pool's maximum as Martial arts grants it (the seeds' mean: Willpower and the level; Inner calm where the picks took it) and the ki back a round; the fight cells as in "Fights", with one blow an attack (no ki) and with the flurry's second blow (−2 to hit, a point of ki an attack); then the ki the flurry spends a round against what comes back, and how many rounds a full pool keeps it up (`holds` when it comes back as fast).

| L | ki | back a round | One blow | Flurry | spent − back = net · rounds from full |
|---|---|---|---|---|---|
| 1 | 5.3 | 0.33 | 80/63 · 5.5:1.1 · 2.1 / 19 | 80/63 · 10.2:1.1 · 1.2 / 20 | 0.99 − 0.33 = 0.66 · 8.1 |
| 5 | 5.3 | 0.33 | 90/69 · 8.0:3.1 · 2.8 / 10 | 90/69 · 15.1:3.0 · 1.5 / 10 | 1.00 − 0.33 = 0.67 · 8.0 |
| 10 | 6.3 | 0.33 | 95/74 · 14.8:5.8 · 2.9 / 9.6 | 95/74 · 28.6:5.8 · 1.5 / 9.5 | 1.13 − 0.33 = 0.80 · 8.0 |
| 15 | 7.8 | 0.33 | 99/74 · 21.0:8.7 · 3.1 / 9.6 | 99/74 · 41.5:8.7 · 1.6 / 9.6 | 1.21 − 0.33 = 0.88 · 8.9 |
| 20 | 9.0 | 0.33 | 99/79 · 25.3:10.3 · 3.1 / 11 | 99/79 · 50.4:10.3 · 1.6 / 11 | 1.30 − 0.33 = 0.96 · 9.4 |
| 30 | 11.3 | 0.33 | 99/87 · 34.8:19.6 · 4.1 / 9.5 | 99/87 · 68.8:19.8 · 2.1 / 9.4 | 1.40 − 0.33 = 1.07 · 11 |
| 40 | 13.2 | 0.33 | 99/87 · 46.9:16.8 · 3.7 / 16 | 99/87 · 94.1:16.9 · 1.9 / 16 | 1.53 − 0.33 = 1.20 · 11 |
| 50 | 15.8 | 0.33 | 99/92 · 46.5:27.9 · 4.7 / 13 | 99/92 · 93.2:27.4 · 2.4 / 13 | 1.54 − 0.33 = 1.20 · 13 |
| 60 | 20.0 | 0.83 | 99/97 · 54.0:29.1 · 4.9 / 13 | 99/97 · 106.4:29.4 · 2.5 / 13 | 1.55 − 0.83 = 0.71 · 28 |

This section is #846's run (the ki tricks design's S2: the pool from level 1 by Martial arts,
its maximum `4 + the Willpower modifier + level / 6`, at least 3, and Perfect self's pace); the
other sections are unchanged by it. The same probe on the content before #846 (ki from Flurry at
18, a flat 5) printed no row below 20 (no pool, so no flurry: the "One blow" cells were the
ascetic's whole fight to level 17), and from 20 the same fight cells with a pool of 5.0 (6.3 at
60 with Inner calm) and 0.33 back a round (0.44 at 60): a full pool kept the flurry up 5.2
rounds at 20, 4.7 at 30, 4.2 at 40 and 50, and 5.8 at 60, against 9.2, 11, 11, 13, and 26 now.
What the run shows:

- **The flurry doubles the ascetic's damage** at every level (the second blow at −2 lands about
  as often as the first against the monsters of their level), so rounds to kill halve: 1.2 to
  2.3 rounds a monster of their level.
- **A full pool outlasts a fight**: eight to thirteen rounds of flurry from full against one to
  two and a half rounds a kill, so a duel never runs dry; a run of three or four fights back to
  back does, and then the flurry comes every third attack (0.33 back a round). Perfect self's
  pace (0.67, 0.83 with Inner calm) makes it every other attack or better at 60.
- **The maximum grows as the design meant**: 5.3 at 1, 8.8 at 20, 15.7 at 50, 19.3 at 60
  (Inner calm included where the picks took it), so the later tricks (S3 to S7) have points to
  draw on beside the flurry.

## Zeal

Added with the paladin's Zeal and Smite (#910; `docs/design/paladin-smites.md` §5), and rerun with
Zealot (#914) to take in Holy wrath's die, Banishing smite's doubling, and Zealot; the other
sections were not rerun for it (the probe's draws after this table shift, so a rerun moves the
"Summons" cells by noise).

The paladin in plain gear (a long sword; from level 5 a shield and the warrior's armour), trained stats, against the monsters within two levels. The pool's maximum as Smite grants it (the seeds' mean: Presence and the level) and the zeal back a round with time; the fight cells as in "Fights", with plain blows, with every blow a smite (an upper bound: a smite costs its zeal), with every blow a smite on the unholy among those monsters (the smite's die more), and from level 50 with every blow a Banishing smite on them (its dice twice over; the summoned are unmade instead, whatever their hit points, which no cell shows). From level 40 every cell has Holy wrath's die where it lands, on the unholy, and at 60 (Avatar) on every foe. Then the zeal a round of plain blows builds (the paladin's hits and the foe's, at their chances, plus the time's) against a smite's cost: the rounds of fighting a smite takes to earn, and the same with Zealot.

| L | zeal | back a round | Blow | Smite | Smite on the unholy | Banishing on the unholy | built a round · rounds a smite | with Zealot |
|---|---|---|---|---|---|---|---|---|
| 1 | 10.8 | 0.10 | 78/41 · 6.8:0.1 · 1.8 / 627 | 83/41 · 10.3:0.1 · 1.2 / 663 | 87/34 · 13.4:0.6 · 1.2 / 35 | - | 1.29 · 2.3 | 1.70 · 1.8 |
| 5 | 10.8 | 0.10 | 84/34 · 7.0:0.7 · 3.4 / 53 | 88/34 · 10.0:0.6 · 2.4 / 52 | 92/36 · 13.0:0.7 · 2.2 / 49 | - | 1.16 · 2.6 | 1.50 · 2.0 |
| 10 | 11.8 | 0.10 | 91/27 · 9.8:1.2 · 4.4 / 49 | 94/27 · 13.2:1.2 · 3.3 / 47 | 93/16 · 16.5:1.1 · 3.2 / 55 | - | 1.16 · 2.6 | 1.42 · 2.1 |
| 15 | 12.0 | 0.10 | 94/14 · 11.5:0.7 · 5.4 / 151 | 97/14 · 15.1:0.7 · 4.1 / 159 | 96/8 · 19.7:0.6 · 3.0 / 172 | - | 1.06 · 2.8 | 1.20 · 2.5 |
| 20 | 13.2 | 0.10 | 98/7 · 13.9:0.4 · 6.2 / 576 | 99/7 · 21.2:0.4 · 3.9 / 523 | 99/7 · 24.8:0.6 · 3.3 / 272 | - | 1.03 · 2.9 | 1.10 · 2.7 |
| 30 | 14.5 | 0.10 | 98/6 · 16.3:0.9 · 8.8 / 346 | 99/6 · 27.3:0.9 · 5.3 / 262 | 99/5 · 31.5:0.9 · 3.2 / 318 | - | 1.01 · 3.0 | 1.08 · 2.8 |
| 40 | 15.8 | 0.10 | 98/5 · 20.1:1.0 · 9.0 / 372 | 99/5 · 34.8:1.1 · 5.4 / 891 | 98/5 · 40.8:1.1 · 3.2 / 459 | - | 1.01 · 3.0 | 1.05 · 2.8 |
| 50 | 17.0 | 0.10 | 98/7 · 19.6:1.7 · 11 / 366 | 99/7 · 38.1:1.7 · 6.0 / 296 | 99/20 · 44.9:4.6 · 5.3 / 79 | 99/20 · 66.5:4.9 · 3.6 / 75 | 1.04 · 2.9 | 1.11 · 2.7 |
| 60 | 18.8 | 0.10 | 94/12 · 18.3:2.5 · 12 / 208 | 98/12 · 38.4:2.8 · 6.1 / 261 | 97/12 · 45.7:4.3 · 3.8 / 94 | 97/12 · 70.3:4.8 · 2.5 / 83 | 1.06 · 2.8 | 1.18 · 2.5 |

What the table shows; none of it is tuned, tuning is the owner's call (Zealot, Smite's dice, and
every cost are data):

- **A smite every third blow is what the fight pays for**: a round of plain blows builds about
  one zeal (1.0 to 1.3: the paladin's hits nearly every blow, the foe's far fewer, a tenth from
  time), so a smite (3) comes about every three rounds, the design's "a smite every third blow".
- **A smite is worth 1.3 to 2.0 blows**: +3.5 at level 1 on 6.8, +7 at 20 on 13.8, +20 at 60 on
  19.4 (the smite's dice are rolled on top of a blow's, and Presence surer to hit), and the
  unholy die adds about 3 to 4 more. With a smite every third blow that is a sixth more damage
  than plain blows at level 1 to a third more at 60, about the design's "a quarter to a third".
- **Holy wrath is worth what its die says**: from level 40 the unholy take about 3.5 more a hit
  (41.3 on 37.7 at 40 against this table's first run, which left it out), and the "Blow" column
  moves little (its median monster is mostly not unholy) until Avatar puts the die on every foe at
  60 (19.4 on 15.8; other content has moved the cells a little since, the paladin's hit chance
  there too). It costs no zeal, so it is pure gain, and small beside a smite's dice, as the design
  meant.
- **Banishing smite is dear on the unholy**: 67.4 on 45.9 a blow at level 50, 71.9 on 46.5 at
  60: half as much again for twice a smite's zeal (6 against 3), and the pool holds about three.
  It is the most damage the paladin has, and on the summoned it does what no cell shows: they die
  whatever their hit points.
- **Zealot is worth most where the paladin is hit most**: it adds the foe's chance to hit to a
  round's zeal, so a smite comes every 1.8 rounds instead of 2.3 at level 1, 2.0 instead of 2.6
  at 5, but 2.8 instead of 2.9 at 30 and 2.8 instead of 3.0 at 40, where one foe of the
  paladin's level hits one blow in fifteen or twenty. At 50 and 60 (hit 13 to 18 in a hundred)
  it is 2.5 and 2.3 instead of 2.7. This table is one foe at a time; the design's case for
  Zealot is the doorway, several foes hitting, and each adds its own chance (Retribution, 25,
  spends what those blows build). In armour against level-mates at 20 to 40 that case is thin.
- **The pool covers a held smite and a heal**: 10.7 at level 1 to 18.5 at 60 (Presence and the
  level), so a full pool is three smites, or a smite and Lay on hands (6).


The ranger (#923) and their starting beast, in the starting kit (the short sword: the probe's blows are melee, so the bow is not measured), stats as rolled, short blades at the rank the other tables assume for the level, seeds 1 to 6 (each draws its own beast), against the monsters within two levels. Fight cells as in "Fights" (P(it hits) / P(it is hit) % · damage a round · rounds to kill / rounds to die) for the ranger alone and the companion alone (its grade is the ranger's level: its hit points, to-hit, damage, Defence and Protection raised by `[growth]`); together, the rounds to kill with both striking (the monster's hit points over the two damages added); the last column is the ranger alone's rounds over those.

| L | beasts | its hp | Ranger | Companion | Together: rounds to kill | alone over together |
|---|---|---|---|---|---|---|
| 1 | hound, boar | 17 | 73/54 · 4.5:0.6 · 2.8 / 37 | 71/71 · 4.4:1.5 · 3.6 / 11 | 1.6 | x1.8 |
| 5 | hound, boar | 24 | 79/62 · 4.6:2.3 · 5.2 / 13 | 73/77 · 3.9:3.8 · 7.8 / 6.2 | 3.1 | x1.7 |
| 10 | hound, boar | 35 | 79/68 · 6.0:4.9 · 7.3 / 8.9 | 83/78 · 6.9:6.5 · 7.8 / 5.4 | 3.8 | x1.9 |
| 15 | hound, boar | 46 | 81/69 · 7.2:7.4 · 8.8 / 7.6 | 84/79 · 10.4:9.8 · 7.1 / 4.7 | 3.9 | x2.2 |
| 20 | hound, boar | 57 | 85/74 · 7.2:9.3 · 12 / 7.6 | 92/78 · 12.7:10.0 · 7.4 / 6.0 | 4.5 | x2.6 |
| 30 | hound, boar | 79 | 80/85 · 7.9:19.1 · 15 / 5.0 | 96/80 · 19.7:17.8 · 7.3 / 4.5 | 4.9 | x3.1 |
| 40 | hound, boar | 101 | 73/92 · 9.7:17.3 · 16 / 7.2 | 98/79 · 27.4:14.8 · 6.7 / 6.8 | 4.8 | x3.4 |
| 50 | hound, boar | 124 | 66/98 · 7.5:29.5 · 32 / 4.9 | 99/79 · 34.4:23.1 · 6.6 / 5.4 | 5.5 | x5.9 |
| 60 | hound, boar | 145 | 51/99 · 3.8:29.4 · 87 / 5.0 | 99/84 · 35.0:21.4 · 7.0 / 7.2 | 6.5 | x13.5 |

What it says (2026-11-03, C8): the hound (grade the ranger's level) out-damages the melee ranger from level 5 on and by level 60 deals about nine times the short sword's damage a round (34.8 against 3.8; the ranger's real weapon is the bow, which this probe does not measure, so read the ranger column as a floor). It hits 71 to 99 % of the time against monsters of the ranger's level and kills one in about seven rounds at every level, and it lasts about five to six rounds against them, the same as the ranger does. The two together kill 1.6 to 6.3 rounds; the companion is the larger half of the pair from level 5. `grade_percent = 100` (the design's Q1) stands (the beast's level is the ranger's); whether a hound that out-damages its melee master nine to one at level 60 is too much is the owner's call (a tamed wolf is not measured here: it is the same growth from a higher base, and the probe's seeds draw only the hound and the boar).

## Summons

The druid's calls (#839), cast by a human druid (untrained stats, every score as rolled, seed 1) at each level a call comes or Pack leader (32) and Lord of the wild (60) change them: what comes of one cast (each type's count), for how many minutes (`lasting`: until slain), the mana over the pool, and each called type against the monsters within two levels of the druid, as in "Fights" (the beast's side bare: P(it hits) / P(it is hit) % · damage a round · rounds to kill / rounds to die).

| Call | L | comes | minutes | mana / pool | the called against the druid's level |
|---|---|---|---|---|---|
| Call of the wild (5) | 6 | 1 wolf | 5 | 5 / 43 (9 casts) | wolf 4: 69/69 · 3.0:4.0 · 7.4 / 4.0 |
| Call of the wild (5) | 25 | 1 wolf | 5 | 5 / 161 (32 casts) | wolf 4: 54/96 · 1.7:18.9 · 49 / 0.8 |
| Call of the forest (20) | 25 | 1 bear | 5 | 20 / 161 (8 casts) | bear 12: 81/98 · 8.4:19.3 · 13 / 3.1 |
| Call of the wild (5) | 32 | 2 wolves | 7 | 5 / 204 (41 casts) | wolf 4: 38/99 · 1.1:25.1 · 106 / 0.6 |
| Call of the forest (20) | 32 | 2 bears | 7 | 20 / 204 (10 casts) | bear 12: 69/99 · 6.7:25.4 · 18 / 2.4 |
| Call of the wild (5) | 40 | 2 wolves | 7 | 5 / 253 (51 casts) | wolf 4: 31/99 · 0.8:23.5 · 226 / 0.7 |
| Call of the forest (20) | 40 | 2 bears | 7 | 20 / 253 (13 casts) | bear 12: 62/99 · 4.9:23.1 · 28 / 2.6 |
| Call the elements (35) | 40 | 1 greater air elemental, 1 greater fire elemental | 7 | 35 / 253 (7 casts) | greater earth elemental 24: 90/99 · 14.8:16.8 · 11 / 7.3; greater water elemental 26: 75/99 · 12.2:20.2 · 12 / 4.3; greater air elemental 31: 85/75 · 28.7:13.9 · 5.4 / 6.7; greater fire elemental 38: 93/93 · 24.9:18.4 · 6.9 / 6.6 |
| Call of the wild (5) | 50 | 2 wolves | 7 | 5 / 314 (63 casts) | wolf 4: 19/99 · 0.2:32.9 · 609 / 0.5 |
| Call of the forest (20) | 50 | 2 bears | 7 | 20 / 314 (16 casts) | bear 12: 46/99 · 3.8:33.5 · 60 / 1.8 |
| Call the elements (35) | 50 | 1 greater air elemental, 1 greater fire elemental | 7 | 35 / 314 (9 casts) | greater earth elemental 24: 81/99 · 13.1:21.1 · 17 / 5.8; greater water elemental 26: 62/99 · 9.5:26.7 · 20 / 3.3; greater air elemental 31: 75/93 · 24.3:24.7 · 8.4 / 3.8; greater fire elemental 38: 85/99 · 24.5:30.3 · 8.8 / 4.0 |
| Call of the wild (5) | 60 | 2 wolves | lasting | 5 / 374 (75 casts) | wolf 4: 10/99 · 0.1:32.7 · >999 / 0.5 |
| Call of the forest (20) | 60 | 2 bears | lasting | 20 / 374 (19 casts) | bear 12: 31/99 · 1.7:34.0 · 119 / 1.8 |
| Call the elements (35) | 60 | 1 greater air elemental, 1 greater fire elemental | lasting | 35 / 374 (11 casts) | greater earth elemental 24: 69/99 · 9.8:26.1 · 22 / 4.7; greater water elemental 26: 46/99 · 6.5:29.6 · 34 / 2.9; greater air elemental 31: 62/98 · 16.6:24.9 · 13 / 3.7; greater fire elemental 38: 75/99 · 20.5:29.7 · 11 / 4.1 |

## Familiar

The witch's familiar (#929, #933), a human witch (untrained stats, every score as rolled, seed 1) with each of the five kinds a run may draw in turn, at its grade (her level). Its touch is her Evil eye and needs a blow that lands, so the table is the familiar's side bare against the monsters within two levels of the witch's (as in "Fights", the monsters' gear left out): **hit** the chance a blow lands, lowest / median / highest over the window; **blows** a round (a normal turn); **hexes** laid a round (blows × median hit); **rounds to the first** hex (one over the hexes a round); **degrees** her Evil eye lays; **eye** the mana one cast of it costs over her pool, which every landed blow saves her; the familiar's hit points, mean damage of a hit, and to-hit.

| L | familiar | hit % (low / median / high) | blows | hexes a round | rounds to the first | degrees | eye (mana / pool) | hp · mean hit · to hit |
|---|---|---|---|---|---|---|---|---|
| 1 | cat | 19 / 31 / 69 | 1.30 | 0.41 | 2.5 | 1 | 1 / 13 | 10 · 1.0 · -4 |
| 1 | toad | 19 / 31 / 69 | 0.90 | 0.28 | 3.6 | 1 | 1 / 13 | 10 · 1.0 · -4 |
| 1 | raven | 19 / 31 / 69 | 1.30 | 0.41 | 2.5 | 1 | 1 / 13 | 9 · 1.0 · -4 |
| 1 | hare | 25 / 38 / 75 | 1.60 | 0.61 | 1.6 | 1 | 1 / 13 | 10 · 1.0 · -3 |
| 1 | bat | 15 / 25 / 62 | 2.00 | 0.50 | 2.0 | 1 | 1 / 13 | 10 · 1.0 · -5 |
| 5 | cat | 1 / 38 / 75 | 1.30 | 0.50 | 2.0 | 1 | 1 / 42 | 18 · 1.0 · -3 |
| 5 | toad | 1 / 38 / 75 | 0.90 | 0.34 | 2.9 | 1 | 1 / 42 | 18 · 1.0 · -3 |
| 5 | raven | 1 / 38 / 75 | 1.30 | 0.50 | 2.0 | 1 | 1 / 42 | 17 · 1.0 · -3 |
| 5 | hare | 2 / 46 / 81 | 1.60 | 0.73 | 1.4 | 1 | 1 / 42 | 18 · 1.0 · -2 |
| 5 | bat | 1 / 31 / 69 | 2.00 | 0.62 | 1.6 | 1 | 1 / 42 | 18 · 1.0 · -4 |
| 10 | cat | 7 / 62 / 85 | 1.30 | 0.80 | 1.2 | 1 | 1 / 80 | 29 · 1.0 · -1 |
| 10 | toad | 4 / 54 / 81 | 0.90 | 0.49 | 2.1 | 1 | 1 / 80 | 29 · 1.0 · -2 |
| 10 | raven | 4 / 54 / 81 | 1.30 | 0.70 | 1.4 | 1 | 1 / 80 | 28 · 1.0 · -2 |
| 10 | hare | 7 / 62 / 85 | 1.60 | 0.99 | 1.0 | 1 | 1 / 80 | 29 · 1.0 · -1 |
| 10 | bat | 4 / 54 / 81 | 2.00 | 1.08 | 0.9 | 1 | 1 / 80 | 29 · 1.0 · -2 |
| 15 | cat | 15 / 62 / 85 | 1.30 | 0.80 | 1.2 | 1 | 1 / 118 | 40 · 2.0 · +0 |
| 15 | toad | 15 / 62 / 85 | 0.90 | 0.56 | 1.8 | 1 | 1 / 118 | 40 · 2.0 · +0 |
| 15 | raven | 15 / 62 / 85 | 1.30 | 0.80 | 1.2 | 1 | 1 / 118 | 39 · 3.0 · +0 |
| 15 | hare | 19 / 69 / 90 | 1.60 | 1.10 | 0.9 | 1 | 1 / 118 | 40 · 3.0 · +1 |
| 15 | bat | 10 / 54 / 81 | 2.00 | 1.08 | 0.9 | 1 | 1 / 118 | 40 · 2.0 · -1 |
| 20 | cat | 4 / 69 / 81 | 1.30 | 0.89 | 1.1 | 1 | 1 / 157 | 51 · 4.0 · +2 |
| 20 | toad | 2 / 62 / 75 | 0.90 | 0.56 | 1.8 | 1 | 1 / 157 | 51 · 3.0 · +1 |
| 20 | raven | 2 / 62 / 75 | 1.30 | 0.80 | 1.2 | 1 | 1 / 157 | 50 · 4.0 · +1 |
| 20 | hare | 4 / 69 / 81 | 1.60 | 1.10 | 0.9 | 1 | 1 / 157 | 51 · 4.0 · +2 |
| 20 | bat | 2 / 62 / 75 | 2.00 | 1.24 | 0.8 | 1 | 1 / 157 | 51 · 4.0 · +1 |
| 30 | cat | 10 / 81 / 93 | 1.30 | 1.05 | 1.0 | 1 | 1 / 229 | 73 · 7.5 · +6 |
| 30 | toad | 7 / 75 / 90 | 0.90 | 0.68 | 1.5 | 1 | 1 / 229 | 73 · 6.5 · +5 |
| 30 | raven | 7 / 75 / 90 | 1.30 | 0.97 | 1.0 | 1 | 1 / 229 | 72 · 8.0 · +5 |
| 30 | hare | 10 / 81 / 93 | 1.60 | 1.29 | 0.8 | 1 | 1 / 229 | 73 · 7.5 · +6 |
| 30 | bat | 7 / 75 / 90 | 2.00 | 1.50 | 0.7 | 1 | 1 / 229 | 73 · 8.0 · +5 |
| 40 | cat | 54 / 85 / 98 | 1.30 | 1.11 | 0.9 | 1 | 1 / 303 | 95 · 12.0 · +9 |
| 40 | toad | 46 / 81 / 96 | 0.90 | 0.72 | 1.4 | 1 | 1 / 303 | 95 · 11.0 · +8 |
| 40 | raven | 46 / 81 / 96 | 1.30 | 1.05 | 1.0 | 1 | 1 / 303 | 94 · 13.0 · +8 |
| 40 | hare | 54 / 85 / 98 | 1.60 | 1.37 | 0.7 | 1 | 1 / 303 | 95 · 12.0 · +9 |
| 40 | bat | 46 / 81 / 96 | 2.00 | 1.61 | 0.6 | 1 | 1 / 303 | 95 · 13.0 · +8 |
| 50 | cat | 62 / 93 / 99 | 1.30 | 1.21 | 0.8 | 1 | 1 / 377 | 117 · 16.5 · +13 |
| 50 | toad | 54 / 90 / 99 | 0.90 | 0.81 | 1.2 | 1 | 1 / 377 | 117 · 15.5 · +12 |
| 50 | raven | 54 / 90 / 99 | 1.30 | 1.16 | 0.9 | 1 | 1 / 377 | 116 · 18.0 · +12 |
| 50 | hare | 62 / 93 / 99 | 1.60 | 1.49 | 0.7 | 1 | 1 / 377 | 117 · 16.5 · +13 |
| 50 | bat | 54 / 90 / 99 | 2.00 | 1.79 | 0.6 | 1 | 1 / 377 | 117 · 18.0 · +12 |
| 60 | cat | 54 / 96 / 99 | 1.30 | 1.25 | 0.8 | 1 | 1 / 450 | 139 · 18.5 · +16 |
| 60 | toad | 46 / 93 / 99 | 0.90 | 0.84 | 1.2 | 1 | 1 / 450 | 139 · 17.5 · +15 |
| 60 | raven | 46 / 93 / 99 | 1.30 | 1.21 | 0.8 | 1 | 1 / 450 | 138 · 20.0 · +15 |
| 60 | hare | 54 / 96 / 99 | 1.60 | 1.53 | 0.7 | 1 | 1 / 450 | 139 · 18.5 · +16 |
| 60 | bat | 46 / 93 / 99 | 2.00 | 1.86 | 0.5 | 1 | 1 / 450 | 139 · 20.0 · +15 |

**Reading it.** The touch needs a blow that lands, and a grade-1 familiar is the weakest
attacker in the game: the cat is -4 to hit with a 1d2-4 scratch, so at level 1 it lands about 31 %
of its blows against level 1 to 3 monsters (19 % against the best-defended, 69 % against the
worst) and lays the hex 0.41 times a round, the first after 2.5 rounds on average (the hare 38 %
and 0.61, the bat 25 % and 0.50). The design assumed a hex that sticks; at level 1 the cat is a
slow way to lay one, and the witch's own Evil eye (1 mana of a 15 pool) is the faster. The
familiar's hit rate catches up as it grows (+1 to hit per 4 levels of grade): 62 % median at level
10, 81 % at 30, 93 % at 50. Not tuned (an owner call). The window is the monsters' bare Defence, so
a geared monster is a little harder; and the table says nothing of the familiar reaching its
target, only that it is beside it.


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
| warrior | St | 14.8/16.7 | 17.7 | 20.8 | 23.0 | 25.3 | 28.3 | +13.5 | 11.7 |
|  | Dx | 10.8/16.3 | 12.0 | 16.5 | 19.2 | 20.7 | 22.8 | +12.0 | 6.5 |
|  | To | 11.7/16.0 | 13.7 | 17.0 | 18.7 | 19.3 | 20.8 | +9.2 | 4.8 |
|  | In | 9.8/17.0 | 11.5 | 12.8 | 14.7 | 15.8 | 17.5 | +7.7 | 0.7 |
|  | Wi | 11.7/17.7 | 11.7 | 13.0 | 14.2 | 15.8 | 18.3 | +6.7 | 0.7 |
|  | Pr | 10.0/16.2 | 10.0 | 10.7 | 11.5 | 12.2 | 14.3 | +4.3 | 0.0 |
| | all six | | | | | | | **+53.3** | |
| thief | St | 11.0/16.8 | 16.3 | 20.3 | 22.7 | 24.2 | 27.2 | +16.2 | 10.3 |
|  | Dx | 12.5/16.5 | 15.8 | 19.3 | 21.2 | 22.3 | 24.0 | +11.5 | 7.5 |
|  | To | 10.5/16.8 | 15.3 | 18.8 | 20.3 | 21.5 | 23.8 | +13.3 | 7.0 |
|  | In | 11.3/17.2 | 12.7 | 14.2 | 15.8 | 17.2 | 17.8 | +6.5 | 0.7 |
|  | Wi | 10.3/17.3 | 10.5 | 12.0 | 13.3 | 15.0 | 17.2 | +6.8 | 0.2 |
|  | Pr | 9.8/16.3 | 9.8 | 10.8 | 11.5 | 12.3 | 14.0 | +4.2 | 0.0 |
| | all six | | | | | | | **+58.5** | |
| wizard | St | 10.0/16.7 | 10.0 | 10.0 | 10.3 | 11.0 | 12.0 | +2.0 | 0.0 |
|  | Dx | 10.0/16.3 | 12.2 | 15.0 | 17.5 | 18.8 | 20.3 | +10.3 | 4.0 |
|  | To | 9.5/17.3 | 12.8 | 19.0 | 20.3 | 21.7 | 23.7 | +14.2 | 6.3 |
|  | In | 14.5/16.8 | 17.3 | 19.5 | 21.0 | 22.5 | 24.5 | +10.0 | 7.7 |
|  | Wi | 12.5/17.7 | 18.0 | 21.2 | 23.2 | 25.0 | 27.7 | +15.2 | 10.0 |
|  | Pr | 9.5/16.0 | 9.5 | 10.3 | 11.0 | 12.2 | 14.0 | +4.5 | 0.0 |
| | all six | | | | | | | **+56.2** | |
| priest | St | 10.8/16.3 | 16.2 | 19.5 | 22.2 | 24.5 | 26.8 | +16.0 | 10.5 |
|  | Dx | 8.7/16.3 | 9.3 | 12.2 | 14.3 | 17.2 | 19.8 | +11.2 | 3.5 |
|  | To | 10.7/16.7 | 14.5 | 19.0 | 20.2 | 21.7 | 24.2 | +13.5 | 7.5 |
|  | In | 11.3/16.7 | 13.0 | 14.5 | 15.8 | 16.8 | 17.7 | +6.3 | 1.0 |
|  | Wi | 15.3/18.0 | 16.8 | 19.3 | 20.5 | 21.5 | 23.0 | +7.7 | 5.0 |
|  | Pr | 10.8/16.3 | 10.8 | 11.7 | 12.0 | 13.0 | 14.8 | +4.0 | 0.0 |
| | all six | | | | | | | **+58.7** | |

Marks a day the hooks gave in each stat over the level before (St/Dx/To/In/Wi/Pr, the seeds' mean), and the game day reached.

| L | Day | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| 5 | 4 | 25/6/15/22/3/3 | 64/34/64/21/3/3 | 1/27/44/51/85/3 | 28/6/43/22/17/3 |
| 10 | 14 | 76/16/29/5/3/2 | 98/68/62/5/3/2 | 1/27/43/48/112/1 | 76/8/59/5/25/1 |
| 15 | 24 | 136/41/39/6/3/2 | 160/106/55/6/3/2 | 0/13/51/62/138/2 | 131/12/76/6/31/2 |
| 20 | 39 | 222/35/27/4/3/1 | 144/101/50/3/3/1 | 0/15/43/52/128/1 | 132/16/51/3/25/1 |
| 30 | 69 | 346/66/29/3/3/1 | 219/138/72/3/3/2 | 1/23/54/83/192/1 | 250/18/101/4/29/1 |
| 40 | 109 | 331/59/28/2/3/1 | 220/87/87/2/3/1 | 0/19/52/70/174/1 | 324/18/97/2/25/1 |
| 50 | 149 | 408/75/32/2/3/1 | 274/77/98/2/3/1 | 0/23/66/98/237/1 | 230/17/124/2/28/1 |
| 60 | 189 | 439/104/60/2/3/1 | 343/78/162/2/3/1 | 0/18/92/111/282/1 | 277/24/224/2/31/1 |

The game day of each stat's first rise (the seeds' mean of those that rose by level 60; how many of the six in brackets when not all).

| Class | St | Dx | To | In | Wi | Pr |
|---|---|---|---|---|---|---|
| warrior | 7 | 12 | 7 | 6 | 21 | 33 |
| thief | 4 | 5 | 5 | 6 | 19 | 34 |
| wizard | 79 | 7 | 4 | 6 | 4 | 31 |
| priest | 4 | 15 | 6 | 5 | 8 | 34 |

The knobs (design §6.2): mean gain by level 60 with other `price_per_point` and `ramp_divisor` values (St/Dx/To/In/Wi/Pr · all six), the data's in bold.

| Price | Ramp ÷ | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| **20 + 10s** | **2** | +13.5/+12.0/+9.2/+7.7/+6.7/+4.3 · +53 | +16.2/+11.5/+13.3/+6.5/+6.8/+4.2 · +58 | +2.0/+10.3/+14.2/+10.0/+15.2/+4.5 · +56 | +16.0/+11.2/+13.5/+6.3/+7.7/+4.0 · +59 |
| 20 + 10s | 3 | +14.7/+13.0/+10.2/+8.0/+6.8/+4.2 · +57 | +17.5/+12.5/+14.2/+7.0/+7.3/+4.2 · +63 | +2.0/+11.0/+15.3/+10.8/+16.5/+4.3 · +60 | +17.7/+11.5/+14.8/+6.7/+8.5/+3.8 · +63 |
| 20 + 10s | 4 | +15.2/+13.7/+10.8/+7.7/+6.3/+3.8 · +58 | +18.3/+13.3/+15.0/+7.3/+7.3/+4.3 · +66 | +1.8/+11.7/+15.5/+11.3/+17.5/+4.7 · +62 | +18.5/+11.8/+15.3/+7.0/+8.8/+4.2 · +66 |
| 20 + 10s | 6 | +17.2/+14.7/+11.3/+8.0/+6.5/+4.2 · +62 | +20.0/+14.7/+16.0/+7.3/+7.2/+4.3 · +70 | +1.8/+12.5/+16.5/+12.8/+19.3/+4.2 · +67 | +19.5/+12.5/+16.7/+6.8/+9.7/+4.2 · +69 |
| 20 + 15s | 2 | +12.2/+11.2/+8.5/+6.7/+5.0/+3.0 · +46 | +15.3/+10.8/+12.8/+6.2/+5.8/+3.3 · +54 | +1.3/+9.5/+13.5/+8.8/+14.0/+3.2 · +50 | +15.5/+10.2/+13.0/+5.5/+7.0/+3.0 · +54 |
| 20 + 15s | 4 | +14.8/+12.7/+9.5/+6.5/+4.8/+2.8 · +51 | +17.2/+12.2/+13.8/+6.2/+5.7/+3.3 · +58 | +1.2/+10.2/+14.5/+10.3/+16.2/+3.0 · +55 | +17.5/+10.7/+14.3/+5.7/+8.0/+2.8 · +59 |
| 20 + 20s | 4 | +13.8/+11.7/+9.0/+5.5/+4.2/+2.5 · +47 | +16.0/+11.5/+13.3/+5.5/+4.8/+2.3 · +53 | +1.2/+10.0/+13.8/+9.7/+15.3/+2.5 · +52 | +16.3/+10.0/+13.7/+4.8/+7.2/+2.5 · +54 |

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

**One rank per talent** (#952). `levelup:auto` takes the first talent it may, so with the extra
ranks gone the probe's careers pick others from then on: no second or third Hardy, and Armour
training at +2. A warrior who trained reaches level 60 with 349 hit points where they had 380 (a
thief 336 for 367, a wizard 236 for 265, a priest 336 for 368), and level 30 with 192 for 205;
damage a blow is a point or so lower from level 40 (the warrior's 22.8 for 24.2 at 60), and the
rest moves by the noise of other picks and draws. Nothing else in the tables needs a tuning pass.

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
- `weapon_abilities` has 4 talents (Early mastery, Brace of steel, Practised technique,
  Drillmaster).
- `stamina` has 5 (Trained wind, Deep lungs, Steady breathing, Armour-breaker, Drillmaster;
  Second breath and Bellows were cut in #774).
- `stat_training` has 2 (Gourmand, Cauldron; #726; Hard-won and
  Gifted were cut in #774, Long fast became a `hunger` talent in #775, and Quick study of the
  body, Hard living, and Discipline were cut in #787, Hard training and Late bloomer in #791).
- `potions` has 2 (Thrifty drinker, Lasting draughts; Nose for draughts and Cauldron-wise were
  cut in #787).

No follow-up is needed.
