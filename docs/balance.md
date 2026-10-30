# Combat balance: the player against the monsters of their level

The balance probe measures how a fight between the player and the monsters of their level goes,
level by level. It reads every number through the game's own code. It was first run, and not
kept, for the balance report of 2026-10-27 (the combat progression design,
`docs/design/combat-progression.md` §2). It is kept now as a test (#569, the design's §15 D1)
so that the first-guess numbers in the data can be judged again after every tuning PR. It was
first kept at #569, after every slice of that design had landed: the new rank tables (W1),
weapon abilities and stamina (W2 to W4, SP1), spell scales (S1), monster Protection by body (P1),
stat training and the daily check (T1, T2), hit points to level 50 (H1), and potions and elixirs
(Q1 to Q3). The tables below are its output on `main` at the time of #727 (stat training T6), which
measures trained stats through the game's training hooks and added the "Stat training"
section (the trained players' scores, and so their side of every fight, moved with it); the
sections after them ("Against the design's predictions" and
"Tuning notes") were written against #569's run, and their figures are that run's.

## How to run it

```text
cargo test --release -p warlock-core balance_probe -- --ignored --nocapture
```

It takes two to three minutes in release (most of it the stat training's sweep, a hundred and
seventy careers to level 60). It prints the Markdown tables of the sections from "Players" to
"Sneak attacks" below, and those of "Stat training", and the same content always prints the same tables (fixed seeds and streams).
After a PR that tunes combat data (weapon skills, creatures, classes, spells, stats, items), run
it again and replace those sections, then revisit the tuning notes. The code is
`crates/warlock-core/src/world/balance_probe.rs` (the measuring, with the method in its module
docs), `balance_probe/training.rs` (the trained careers' days), and `balance_probe/tables.rs`
(the tables).

## Method

- **Monsters**: every creature type with a level, shared and campaign, 400 in all. Each is made
  by `travel::make_creature`, once bare and once for each of six seeds with the gear its type
  rolls at its own level. Its side of a fight is `World::target_of` (Defence, Protection,
  resistances), its blow is `Actor::melee`, and its blows a round are its speed over its
  `World::attack_price`.
- **Players**: a human male warrior (long sword, sword and board, shield block), thief (dagger,
  twin blades), wizard (staff, spell attack), and priest (mace, sword and board, shield block),
  six seeds each. Each is made by `World::start_run` with the kit and levelled one level at a time
  with `xp:` and `levelup:auto`, which takes the first talent it may, not one chosen for a build.
  Weapon skill ranks are set by level, because ranks come from use: rank 2 at level 1, 4 at 5, 6
  at 10, 7 at 15, 8 at 20, 9 at 30, and 10 from 40. The wizard's staff and the priest's spell
  attack are two ranks lower.
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
- **Spells**: the trained caster's own casts at their caster level, with the stats' lift
  (`Actor::spell_lift`) and without it. The dice are those of `SpellDice::lifted` and the
  talents; nothing holds the roll (the spell scaling design S1, decision 16). Attack spells
  strike the monster's Defence against missiles, and areas always hit.
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
| 1 | +3/8.2/16/3 | +3/8.2/16/3 | +0/3.0/13/1 | +0/3.0/13/1 | +0/3.0/12/0 | +1/6.0/12/0 | +1/6.0/12/0 |
| 5 | +4/9.2/18/4 | +5/10.9/20/4 | +2/4.3/15/1 | +2/5.3/17/2 | +0/3.0/13/0 | +2/7.2/15/2 | +3/7.9/18/2 |
| 10 | +7/11.7/22/5 | +8/12.9/25/6 | +6/7.7/20/1 | +8/9.2/21/2 | +2/4.0/16/0 | +6/10.5/17/4 | +8/12.8/20/5 |
| 15 | +8/13.0/25/9 | +12/16.3/28/10 | +9/8.8/21/2 | +11/11.3/24/4 | +2/4.0/17/1 | +8/12.7/21/5 | +10/16.7/25/5 |
| 20 | +10/15.3/28/10 | +14/20.9/33/12 | +12/10.8/23/2 | +14/12.2/27/4 | +2/5.0/18/1 | +10/14.5/23/7 | +13/16.9/28/8 |
| 30 | +13/17.7/33/12 | +21/24.7/41/18 | +14/13.0/23/2 | +20/20.5/30/8 | +3/6.8/20/1 | +11/16.8/26/8 | +19/22.6/34/15 |
| 40 | +15/20.7/36/12 | +22/26.8/45/16 | +17/15.2/25/2 | +26/19.7/32/5 | +4/7.0/21/1 | +14/20.3/28/8 | +23/26.4/37/12 |
| 50 | +16/21.7/36/12 | +21/28.2/46/17 | +18/16.3/25/2 | +22/24.2/31/7 | +4/7.0/22/1 | +14/21.2/28/8 | +20/29.1/36/13 |
| 60 | +17/22.7/36/12 | +22/28.7/44/15 | +19/17.0/25/2 | +26/22.2/32/7 | +6/8.0/22/1 | +15/22.0/28/8 | +23/27.2/37/13 |

Stats, hit points, and mana (plain gear; trained → untrained). Scores St/Dx/To/In/Wi/Pr, the seeds' mean.

| L | Warrior | Thief | Wizard | Priest | HP W/T/Wz/P | MP Wz/P | Caster level Wz/P | Blows a round W/T |
|---|---|---|---|---|---|---|---|---|
| 1 | 15/11/12/10/12/10 | 11/12/10/11/10/10 | 10/10/10/14/12/10 | 11/9/11/11/15/11 | 22→22 / 19→19 / 13→13 / 18→18 | 16/13 | 1/1 | 0.99/0.99 |
| 5 | 15/11/12/10/12/10 | 12/13/11/12/10/10 | 10/11/10/15/13/10 | 11/9/11/12/15/11 | 36→36 / 27→26 / 18→17 / 26→26 | 52/40 | 6/5 | 0.88/0.96 |
| 10 | 18/12/14/11/12/10 | 16/16/15/13/10/10 | 10/13/14/17/18/10 | 16/9/14/12/17/11 | 58→53 / 48→36 / 27→22 / 45→36 | 104/78 | 12/10 | 0.89/1.04 |
| 15 | 19/14/16/12/13/10 | 19/18/17/14/11/10 | 10/15/18/19/20/10 | 18/11/17/14/18/11 | 86→70 / 74→46 / 44→27 / 72→46 | 159/120 | 18/15 | 0.91/1.10 |
| 20 | 21/17/18/13/13/11 | 21/20/19/14/12/11 | 10/16/19/20/22/10 | 20/13/18/14/20/11 | 118→88 / 104→56 / 66→32 / 102→55 | 217/163 | 24/20 | 0.92/1.17 |
| 30 | 24/19/19/15/14/11 | 23/22/20/16/13/11 | 11/18/21/21/24/11 | 23/15/20/16/21/12 | 185→127 / 171→79 / 114→43 / 181→79 | 341/256 | 36/30 | 0.94/1.24 |
| 40 | 26/21/20/16/16/12 | 25/23/22/17/15/12 | 11/20/22/23/26/12 | 26/19/22/17/22/14 | 282→168 / 264→100 / 195→53 / 272→101 | 476/353 | 48/40 | 0.95/1.42 |
| 50 | 28/22/21/17/18/14 | 28/24/24/18/17/14 | 12/21/24/24/28/14 | 27/20/24/18/23/15 | 374→204 / 361→120 / 271→63 / 361→122 | 622/456 | 60/50 | 0.95/1.49 |
| 60 | 30/24/22/18/19/15 | 29/25/25/19/18/15 | 13/22/26/26/30/15 | 29/21/26/18/24/16 | 385→208 / 372→122 / 282→64 / 372→124 | 779/566 | 72/60 | 0.95/1.56 |

The Defence of the warrior and the thief in plain gear by category (trained → untrained), the seeds' mean.

| Class | L | Dexterity | Armour | Shield | Parry | Natural | Dodge | Deflection | Wall | total |
|---|---|---|---|---|---|---|---|---|---|---|
| warrior | 10 | 1→0 | 2→2 | 6→6 | 2→2 | 0→0 | 1→1 | 0→0 | 0→0 | 22→21 |
| warrior | 30 | 2→0 | 1→1 | 14→14 | 5→5 | 0→0 | 1→1 | 0→0 | 0→0 | 33→31 |
| warrior | 60 | 2→0 | 1→1 | 16→16 | 6→6 | 0→0 | 1→1 | 0→0 | 0→0 | 36→34 |
| thief | 10 | 3→1 | 2→2 | 0→0 | 3→3 | 0→0 | 2→2 | 0→0 | 0→0 | 20→17 |
| thief | 30 | 5→1 | 2→2 | 0→0 | 4→4 | 0→0 | 2→2 | 0→0 | 0→0 | 23→19 |
| thief | 60 | 5→1 | 2→2 | 0→0 | 6→6 | 0→0 | 2→2 | 0→0 | 0→0 | 25→20 |

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
| 50 | 23 | 228 | 14 (10–18) → 14.3 | 32 (16–40) → 29.4 | 19 (12–25) → 19.3 | 4 (0–13) → 4.5 | 1.05 |
| 60 | 17 | 225 | 17 (12–21) → 16.9 | 34 (18–41) → 30.9 | 21 (15–27) → 22.0 | 6 (0–16) → 6.0 | 1.15 |
| 70 | 7 | 197 | 20 (19–23) → 20.1 | 32 (20–40) → 32.1 | 26 (12–29) → 25.0 | 5 (0–12) → 4.9 | 1.35 |

## Fights

Each cell: P(player hits) / P(monster hits) % · damage a round player : monster · rounds to kill / rounds to die; the median over the monsters within two levels (trained stats, monsters with gear).

| L | Warrior kit | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain (melee) | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|---|
| 1 | 85/35 · 7.4:0.1 · 1.6 / 306 | 85/35 · 7.4:0.1 · 1.6 / 332 | 85/35 · 7.4:0.1 · 1.6 / 306 | 67/55 · 2.6:0.5 · 6.1 / 37 | 67/55 · 2.6:0.5 · 6.0 / 36 | 71/62 · 2.3:1.1 · 5.6 / 14 | 76/59 · 4.8:1.0 · 2.7 / 19 | 76/59 · 4.7:1.0 · 2.7 / 19 |
| 5 | 86/34 · 8.4:0.8 · 2.8 / 51 | 86/27 · 7.3:0.5 · 3.3 / 87 | 89/15 · 9.3:0.2 · 2.6 / 250 | 75/46 · 3.8:1.6 · 6.7 / 18 | 75/32 · 4.6:1.0 · 5.3 / 54 | 65/59 · 1.8:2.3 · 15 / 8.2 | 78/43 · 4.6:1.2 · 4.9 / 22 | 81/29 · 5.4:0.8 · 4.1 / 52 |
| 10 | 92/20 · 11.6:1.3 · 3.6 / 57 | 92/15 · 10.4:0.8 · 4.1 / 95 | 94/7 · 11.5:0.3 · 3.8 / >999 | 89/25 · 11.1:2.0 · 4.7 / 31 | 93/17 · 13.9:1.3 · 3.9 / 103 | 65/50 · 2.6:4.1 · 18 / 6.8 | 89/42 · 8.4:2.6 · 5.1 / 20 | 95/25 · 11.4:1.4 · 3.7 / 77 |
| 15 | 95/9 · 13.4:0.7 · 4.6 / 213 | 95/8 · 11.7:0.4 · 5.4 / 309 | 99/4 · 16.9:0.2 · 4.0 / >999 | 96/27 · 13.9:2.8 · 5.4 / 30 | 98/12 · 19.4:1.1 · 4.7 / 121 | 65/56 · 2.0:6.5 · 30 / 7.3 | 95/27 · 10.6:2.3 · 6.5 / 43 | 98/12 · 15.5:1.0 · 4.4 / 987 |
| 20 | 98/3 · 17.2:0.3 · 4.8 / >999 | 98/5 · 15.2:0.2 · 5.5 / >999 | 99/1 · 23.0:0.0 · 3.7 / >999 | 99/27 · 19.8:2.8 · 5.3 / 49 | 99/10 · 24.1:1.0 · 4.3 / 516 | 65/59 · 2.5:6.8 · 34 / 9.9 | 98/26 · 12.0:2.0 · 6.6 / 71 | 99/7 · 16.3:0.4 · 5.0 / >999 |
| 30 | 99/1 · 21.2:0.2 · 6.8 / >999 | 99/2 · 18.4:0.2 · 7.8 / >999 | 99/1 · 30.3:0.1 · 5.0 / >999 | 99/42 · 25.4:9.3 · 6.4 / 21 | 99/7 · 45.3:1.0 · 3.3 / 540 | 62/64 · 2.7:13.3 · 45 / 8.8 | 96/26 · 12.9:4.3 · 10 / 46 | 99/2 · 25.5:0.2 · 5.8 / >999 |
| 40 | 98/1 · 25.8:0.1 · 7.0 / >999 | 98/1 · 21.7:0.1 · 8.1 / >999 | 99/1 · 38.7:0.1 · 4.9 / >999 | 99/47 · 33.8:11.1 · 6.0 / 26 | 99/8 · 53.7:1.3 · 3.8 / >999 | 51/72 · 2.4:15.2 · 70 / 13 | 97/27 · 16.2:5.7 · 11 / 55 | 99/1 · 28.6:0.1 · 5.6 / >999 |
| 50 | 99/2 · 26.3:0.5 · 9.1 / >999 | 99/8 · 23.1:1.3 · 10 / 640 | 99/1 · 36.6:0.1 · 6.6 / >999 | 99/68 · 35.9:19.5 · 7.2 / 19 | 99/31 · 51.8:8.0 · 4.1 / 115 | 42/85 · 1.6:22.6 · 165 / 12 | 97/49 · 15.8:12.6 · 15 / 30 | 99/10 · 30.8:2.2 · 7.7 / >999 |
| 60 | 95/5 · 20.7:1.7 · 9.9 / 847 | 95/16 · 17.8:4.4 · 11 / 108 | 99/1 · 28.6:0.2 · 6.9 / >999 | 99/84 · 31.1:27.7 · 7.9 / 14 | 99/38 · 44.0:12.1 · 5.5 / 54 | 30/95 · 1.4:31.0 · 351 / 9.4 | 91/71 · 15.2:21.0 · 15 / 19 | 99/15 · 24.8:4.2 · 10 / 421 |

Out of depth (+3: the monsters one to five levels above), and untrained stats, for the warrior in plain gear.

| L | Warrior plain +3 | Warrior magic +3 | Warrior plain, untrained | Warrior magic, untrained |
|---|---|---|---|---|
| 1 | 81/35 · 7.0:0.6 · 2.9 / 40 | 81/35 · 6.9:0.6 · 2.9 / 37 | 85/35 · 7.5:0.1 · 1.6 / 333 | 85/35 · 7.4:0.1 · 1.6 / 306 |
| 5 | 86/34 · 7.3:1.5 · 4.9 / 26 | 88/20 · 9.1:0.8 · 3.9 / 65 | 86/27 · 7.4:0.5 · 3.3 / 84 | 89/15 · 9.3:0.2 · 2.6 / 275 |
| 10 | 92/23 · 10.0:1.5 · 6.1 / 45 | 94/11 · 10.9:0.7 · 5.5 / 525 | 86/17 · 8.5:0.9 · 5.2 / 74 | 89/8 · 9.6:0.4 · 4.5 / >999 |
| 15 | 94/11 · 11.6:0.7 · 6.2 / 150 | 99/5 · 16.4:0.3 · 4.5 / >999 | 89/11 · 8.5:0.6 · 7.6 / 171 | 98/5 · 13.7:0.3 · 4.9 / >999 |
| 20 | 97/8 · 14.9:0.6 · 7.5 / 318 | 99/1 · 22.4:0.1 · 5.0 / >999 | 90/7 · 9.5:0.4 · 9.3 / 578 | 97/1 · 19.3:0.0 · 4.7 / >999 |
| 30 | 98/3 · 17.0:0.4 · 8.7 / 964 | 99/1 · 29.7:0.1 · 5.4 / >999 | 88/5 · 9.8:0.6 · 13 / 629 | 99/1 · 23.7:0.1 · 6.5 / >999 |
| 40 | 99/2 · 21.9:0.4 · 8.9 / >999 | 99/1 · 38.2:0.1 · 5.3 / >999 | 81/3 · 12.5:0.6 · 14 / >999 | 97/1 · 27.9:0.1 · 6.7 / >999 |
| 50 | 97/8 · 20.3:1.5 · 11 / 487 | 99/1 · 33.6:0.2 · 6.8 / >999 | 76/14 · 10.9:2.7 · 23 / 131 | 93/1 · 24.6:0.1 · 10 / >999 |
| 60 | 86/31 · 14.6:8.8 · 16 / 53 | 98/2 · 24.7:0.4 · 9.4 / >999 | 55/27 · 6.8:8.2 · 43 / 28 | 87/1 · 16.7:0.3 · 14 / >999 |

## Protection by body

Rounds for the plain warrior (trained) to kill the monsters of their level: as shipped, then with every monster's Protection set to §8.2's body line at its level.

| L | as shipped | none (0) | hide | bony | armoured |
|---|---|---|---|---|---|
| 10 | 4.1 | 3.7 | 3.7 | 4.2 | 4.7 |
| 15 | 5.3 | 4.6 | 4.5 | 5.0 | 6.0 |
| 20 | 5.5 | 4.6 | 4.9 | 5.3 | 5.8 |
| 30 | 7.8 | 5.8 | 6.1 | 6.9 | 7.9 |
| 40 | 8.1 | 5.9 | 6.5 | 6.9 | 8.0 |
| 50 | 10 | 8.3 | 9.4 | 11 | 13 |
| 60 | 11 | 7.6 | 8.7 | 9.5 | 11 |

## Spells

Each damage spell as the trained caster casts it (plain gear): the lift, the dice, and the expected casts to kill the median monster of their level (an attack spell's miss chance included; per target), lifted (unlifted in brackets); mana per cast of the pool.

**Wizard**

| Spell (level, resolution) | L | lift % | dice | casts to kill | mana / pool |
|---|---|---|---|---|---|
| Ray of frost (1, Attack) | 10 | 0 | 1d6+1, 4.5 | 9.1 (9.2) | 1 / 104 |
| Ray of frost (1, Attack) | 30 | 0 | 1d6+1, 4.5 | 24 (24) | 1 / 341 |
| Ray of frost (1, Attack) | 60 | 0 | 2d6+6, 13.0 | 18 (18) | 1 / 779 |
| Spark (1, Auto) | 10 | 0 | 1d4+1, 3.5 | 11 (11) | 1 / 104 |
| Spark (1, Auto) | 30 | 0 | 1d4+1, 3.5 | 34 (34) | 1 / 341 |
| Spark (1, Auto) | 60 | 0 | 2d4+6, 11.0 | 21 (21) | 1 / 779 |
| Magic missile (3, Auto) | 10 | 65 | 3d4+3, 10.5 | 4.3 (5.6) | 3 / 104 |
| Magic missile (3, Auto) | 30 | 85 | 3d4+3, 10.5 | 11 (17) | 3 / 341 |
| Magic missile (3, Auto) | 60 | 135 | 5d4+15, 27.5 | 8.3 (14) | 1 / 779 |
| Burning hands (4, Area) | 10 | 65 | 3d4+3, 10.5 | 4.3 (5.5) | 6 / 104 |
| Burning hands (4, Area) | 30 | 85 | 3d4+3, 10.5 | 11 (17) | 6 / 341 |
| Burning hands (4, Area) | 60 | 135 | 5d4+15, 27.5 | 8.3 (14) | 2 / 779 |
| Cause wounds (4, Touch) | 10 | 0 | 2d6+2, 9.0 | 5.0 (4.9) | 3 / 104 |
| Cause wounds (4, Touch) | 30 | 0 | 2d6+2, 9.0 | 13 (12) | 3 / 341 |
| Cause wounds (4, Touch) | 60 | 0 | 3d6+9, 19.5 | 14 (14) | 1 / 779 |
| Arcane bolt (6, Area) | 10 | 65 | 3d6+3, 13.5 | 3.3 (4.3) | 11 / 104 |
| Arcane bolt (6, Area) | 30 | 85 | 3d6+3, 13.5 | 8.7 (13) | 11 / 341 |
| Arcane bolt (6, Area) | 60 | 135 | 5d6+15, 32.5 | 7.1 (12) | 4 / 779 |
| Frost (6, Area) | 10 | 65 | 3d6+3, 13.5 | 3.3 (4.3) | 14 / 104 |
| Frost (6, Area) | 30 | 85 | 3d6+3, 13.5 | 8.7 (13) | 14 / 341 |
| Frost (6, Area) | 60 | 135 | 5d6+15, 32.5 | 7.1 (12) | 5 / 779 |
| Blight (7, Area) | 10 | 0 | 2d6+2, 9.0 | 4.3 (4.3) | 17 / 104 |
| Blight (7, Area) | 30 | 0 | 2d6+2, 9.0 | 13 (13) | 17 / 341 |
| Blight (7, Area) | 60 | 0 | 3d6+9, 19.5 | 12 (12) | 6 / 779 |
| Scorching ray (7, Attack) | 10 | 65 | 2d6+2, 9.0 | 4.6 (9.2) | 7 / 104 |
| Scorching ray (7, Attack) | 30 | 85 | 5d6+5, 22.5 | 4.8 (7.9) | 7 / 341 |
| Scorching ray (7, Attack) | 60 | 135 | 8d6+24, 52.0 | 4.9 (9.0) | 2 / 779 |
| Call lightning (11, Area) | 10 | 65 | 3d6+3, 13.5 | 2.9 (4.3) | 20 / 104 |
| Call lightning (11, Area) | 30 | 85 | 5d6+5, 22.5 | 5.2 (8.7) | 20 / 341 |
| Call lightning (11, Area) | 60 | 135 | 8d6+24, 52.0 | 4.9 (8.8) | 8 / 779 |
| Fireball (11, Area) | 10 | 65 | 3d6+3, 13.5 | 2.9 (4.3) | 26 / 104 |
| Fireball (11, Area) | 30 | 85 | 5d6+5, 22.5 | 5.2 (8.7) | 26 / 341 |
| Fireball (11, Area) | 60 | 135 | 8d6+24, 52.0 | 4.9 (8.8) | 10 / 779 |
| Lightning bolt (11, Area) | 10 | 65 | 3d6+3, 13.5 | 2.9 (4.3) | 20 / 104 |
| Lightning bolt (11, Area) | 30 | 85 | 5d6+5, 22.5 | 5.2 (8.7) | 20 / 341 |
| Lightning bolt (11, Area) | 60 | 135 | 8d6+24, 52.0 | 4.9 (8.8) | 8 / 779 |
| Flame (12, Area) | 10 | 65 | 4d6+4, 18.0 | 2.1 (2.9) | 29 / 104 |
| Flame (12, Area) | 30 | 85 | 5d6+5, 22.5 | 5.2 (8.8) | 29 / 341 |
| Flame (12, Area) | 60 | 135 | 8d6+24, 52.0 | 4.9 (8.9) | 11 / 779 |

**Priest**

| Spell (level, resolution) | L | lift % | dice | casts to kill | mana / pool |
|---|---|---|---|---|---|
| none: no damage spell it knows or of a school it has an affinity with ||||||

## Sneak attacks

The thief's main-hand blow on the monsters of their level (trained stats), aware and caught unawares. The Sneak attack value assumed, the dice it gives with the talents, and the share of the monsters within two levels with vitals (no dice and no critical on the rest, so their first blow gains only the flat-footed Defence); then each cell: P(hit) aware / unaware % · damage of a blow aware : the first blow unaware · how many times as much, the median over the monsters within two levels that have vitals.

| L | Sneak attack | dice | with vitals | Thief plain | Thief magic |
|---|---|---|---|---|---|
| 1 | 15 | 1d6 | 71% | 75/75 · 2.6:5.6 · ×2.2 | 75/75 · 2.6:5.6 · ×2.2 |
| 5 | 29 | 2d6 | 55% | 75/85 · 3.7:10.1 · ×2.7 | 75/85 · 4.6:11.0 · ×2.4 |
| 10 | 47 | 2d6 | 72% | 92/98 · 8.0:15.4 · ×1.9 | 95/98 · 9.9:17.1 · ×1.7 |
| 15 | 64 | 3d6 | 63% | 99/99 · 9.8:20.6 · ×2.1 | 99/99 · 13.0:23.3 · ×1.8 |
| 20 | 82 | 4d6 | 66% | 99/99 · 12.1:26.0 · ×2.1 | 99/99 · 14.4:28.3 · ×2.0 |
| 30 | 117 | 5d6 | 62% | 99/99 · 14.2:31.5 · ×2.2 | 99/99 · 24.2:41.6 · ×1.7 |
| 40 | 152 | 8d6 | 59% | 99/99 · 16.4:44.5 · ×2.7 | 99/99 · 24.0:51.5 · ×2.1 |
| 50 | 187 | 8d6 | 52% | 99/99 · 16.8:44.5 · ×2.7 | 99/99 · 28.9:56.7 · ×2.0 |
| 60 | 222 | 9d6 | 47% | 99/99 · 16.9:50.4 · ×3.0 | 99/99 · 24.6:59.0 · ×2.4 |

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
  `discovered`. The thief's first blow of a fight is a sneak attack. The wizard fights with the
  damage spell of the most damage a cast among those costing a fifth of its pool or less: each
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
| warrior | St | 14.8/16.7 | 17.7 | 21.0 | 23.7 | 26.0 | 29.7 | +14.8 | 13.0 |
|  | Dx | 10.8/16.3 | 11.7 | 16.7 | 19.0 | 21.0 | 24.2 | +13.3 | 7.8 |
|  | To | 11.7/16.0 | 13.7 | 17.5 | 18.7 | 19.8 | 22.0 | +10.3 | 6.0 |
|  | In | 9.8/17.0 | 11.3 | 13.0 | 14.8 | 16.3 | 18.3 | +8.5 | 1.3 |
|  | Wi | 11.7/17.7 | 11.8 | 13.2 | 14.3 | 16.3 | 18.7 | +7.0 | 1.0 |
|  | Pr | 10.0/16.2 | 10.0 | 10.7 | 11.3 | 12.0 | 14.7 | +4.7 | 0.0 |
| | all six | | | | | | | **+58.7** | |
| thief | St | 11.0/16.8 | 16.3 | 20.7 | 23.2 | 25.3 | 29.2 | +18.2 | 12.3 |
|  | Dx | 12.5/16.5 | 16.3 | 19.7 | 21.7 | 23.0 | 25.2 | +12.7 | 8.7 |
|  | To | 10.5/16.8 | 15.0 | 18.8 | 20.3 | 22.0 | 25.3 | +14.8 | 8.5 |
|  | In | 11.3/17.2 | 12.7 | 14.2 | 15.8 | 17.2 | 18.8 | +7.5 | 1.7 |
|  | Wi | 10.3/17.3 | 10.3 | 11.8 | 13.0 | 15.0 | 18.2 | +7.8 | 0.8 |
|  | Pr | 9.8/16.3 | 9.8 | 10.7 | 11.3 | 12.5 | 14.8 | +5.0 | 0.0 |
| | all six | | | | | | | **+66.0** | |
| wizard | St | 10.0/16.7 | 10.0 | 10.0 | 10.8 | 11.0 | 13.0 | +3.0 | 0.0 |
|  | Dx | 10.0/16.3 | 12.8 | 16.2 | 18.2 | 20.0 | 21.8 | +11.8 | 5.5 |
|  | To | 9.5/17.3 | 13.8 | 19.2 | 20.8 | 22.5 | 25.7 | +16.2 | 8.3 |
|  | In | 14.5/16.8 | 17.2 | 19.7 | 21.3 | 23.2 | 26.0 | +11.5 | 9.2 |
|  | Wi | 12.5/17.7 | 18.3 | 21.7 | 24.0 | 26.3 | 30.2 | +17.7 | 12.5 |
|  | Pr | 9.5/16.0 | 9.5 | 10.3 | 11.0 | 12.5 | 14.8 | +5.3 | 0.0 |
| | all six | | | | | | | **+65.5** | |
| priest | St | 10.8/16.3 | 15.8 | 19.8 | 22.8 | 25.7 | 29.2 | +18.3 | 12.8 |
|  | Dx | 8.7/16.3 | 9.3 | 12.7 | 15.3 | 18.7 | 21.2 | +12.5 | 4.8 |
|  | To | 10.7/16.7 | 14.3 | 18.5 | 20.5 | 22.0 | 26.2 | +15.5 | 9.5 |
|  | In | 11.3/16.7 | 12.5 | 14.2 | 15.5 | 16.7 | 18.5 | +7.2 | 1.8 |
|  | Wi | 15.3/18.0 | 16.8 | 19.5 | 21.0 | 22.2 | 24.5 | +9.2 | 6.5 |
|  | Pr | 10.8/16.3 | 10.8 | 11.3 | 12.2 | 13.5 | 16.0 | +5.2 | 0.3 |
| | all six | | | | | | | **+67.8** | |

Marks a day the hooks gave in each stat over the level before (St/Dx/To/In/Wi/Pr, the seeds' mean), and the game day reached.

| L | Day | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| 5 | 4 | 25/6/13/20/3/3 | 60/32/57/21/3/3 | 1/34/46/61/98/2 | 37/5/36/20/17/3 |
| 10 | 14 | 76/17/30/6/3/2 | 98/75/61/5/3/2 | 0/30/58/62/142/2 | 76/10/60/5/24/2 |
| 15 | 24 | 138/37/40/6/3/2 | 152/108/48/6/3/2 | 0/19/56/76/168/2 | 127/15/73/6/32/2 |
| 20 | 39 | 220/30/28/3/3/1 | 145/96/46/4/3/1 | 1/20/51/69/155/1 | 126/16/43/3/26/1 |
| 30 | 69 | 342/65/29/3/3/1 | 221/129/67/3/3/1 | 1/30/64/98/245/1 | 207/24/79/3/29/1 |
| 40 | 109 | 494/91/38/2/3/2 | 308/124/119/3/3/2 | 1/37/80/125/305/2 | 385/28/126/2/38/2 |
| 50 | 149 | 573/115/48/2/3/2 | 407/119/144/2/3/1 | 1/33/123/173/420/2 | 369/30/179/2/43/2 |
| 60 | 189 | 511/140/98/2/3/2 | 508/103/239/2/3/2 | 1/26/172/204/506/2 | 402/26/331/2/45/1 |

The game day of each stat's first rise (the seeds' mean of those that rose by level 60; how many of the six in brackets when not all).

| Class | St | Dx | To | In | Wi | Pr |
|---|---|---|---|---|---|---|
| warrior | 6 | 12 | 6 | 6 | 18 | 41 |
| thief | 4 | 5 | 4 | 6 | 22 | 31 |
| wizard | 64 | 4 | 4 | 5 | 4 | 36 |
| priest | 5 | 13 | 5 | 8 | 8 | 40 |

The knobs (design §6.2): mean gain by level 60 with other `price_per_point` and `ramp_divisor` values (St/Dx/To/In/Wi/Pr · all six), the data's in bold.

| Price | Ramp ÷ | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| **20 + 10s** | **2** | +14.8/+13.3/+10.3/+8.5/+7.0/+4.7 · +59 | +18.2/+12.7/+14.8/+7.5/+7.8/+5.0 · +66 | +3.0/+11.8/+16.2/+11.5/+17.7/+5.3 · +66 | +18.3/+12.5/+15.5/+7.2/+9.2/+5.2 · +68 |
| 20 + 10s | 3 | +16.3/+14.0/+11.5/+8.3/+7.3/+4.8 · +62 | +19.5/+14.2/+15.7/+7.7/+7.7/+5.3 · +70 | +2.3/+12.7/+17.0/+12.8/+19.0/+5.3 · +69 | +19.5/+13.2/+16.5/+7.0/+9.7/+5.3 · +71 |
| 20 + 10s | 4 | +17.5/+14.8/+12.0/+8.7/+7.3/+4.8 · +65 | +20.5/+14.8/+16.3/+8.0/+7.8/+5.0 · +72 | +2.8/+13.3/+17.5/+13.7/+20.2/+5.5 · +73 | +20.0/+13.5/+17.2/+7.3/+10.3/+5.0 · +73 |
| 20 + 10s | 6 | +19.0/+16.2/+13.0/+8.7/+7.5/+5.2 · +70 | +21.7/+16.2/+18.0/+8.2/+7.8/+4.8 · +77 | +2.8/+14.3/+18.8/+15.0/+21.3/+5.2 · +78 | +22.2/+14.0/+18.5/+8.0/+11.2/+5.2 · +79 |
| 20 + 15s | 2 | +14.0/+12.5/+9.7/+7.2/+5.7/+4.2 · +53 | +16.3/+12.0/+14.0/+6.5/+6.2/+3.8 · +59 | +2.0/+11.0/+15.3/+10.8/+16.0/+4.3 · +60 | +17.2/+11.8/+14.5/+6.0/+8.0/+3.5 · +61 |
| 20 + 15s | 4 | +16.2/+14.0/+11.0/+7.3/+5.5/+3.8 · +58 | +18.7/+13.5/+15.7/+6.7/+6.2/+4.0 · +65 | +1.8/+12.2/+17.0/+12.7/+18.3/+4.3 · +66 | +19.3/+12.3/+15.8/+6.8/+9.2/+4.0 · +68 |
| 20 + 20s | 4 | +15.0/+13.2/+10.2/+6.5/+4.8/+3.0 · +53 | +18.0/+13.0/+14.7/+6.3/+5.2/+3.3 · +60 | +1.5/+11.7/+15.8/+11.7/+17.5/+3.7 · +62 | +18.3/+11.5/+15.3/+5.7/+8.5/+3.0 · +62 |

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
are Random's picks; Gifted (T5) is one of them now (a seed's warrior takes it for Intelligence,
a seed's thief for Toughness), and the six seeds' starts and potentials moved with it (the
table's first column).

## Against the design's predictions

(The spell findings below predate the spell scaling design's S1, #748, which removed the tiers,
their ceilings, and the run-time cap: the spell tables above are S1's, the prose the cap's.)

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

**§7.3 (spell scaling).** The design predicted, at level 10, Scorching ray at 4d6 (2.7 casts,
3.6 today) and Fireball at 4d6 (2.7). The probe measures 3d6+3 (3.6 casts with misses, 4.7
unlifted) and 3d6+3 (2.8 casts, 4.2). At 30 the design predicted Fireball at 5d6 (6.4 casts,
10.7); the probe measures 5d6+5 held to 31 (5.3, 8.5). At 60 it predicted 12.6 casts (21.0); the
probe measures 6.9 (12). The lift does what the design meant: about a third fewer casts. The
level-60 wizard does better than predicted for two reasons: the median monster of their level
has 217 hit points, not 221, and Archmage's three points a die push every roll of a tier-3 spell
to the cap.

**§2 (the report's findings), revisited.**
- Monster to-hit still grows faster than the player's unaided to-hit, but the player's now grows
  too: a plain warrior goes from +3 to +15, not +4 to +8.
- The magic-armour gap is unchanged, as the owner chose (§17 q11). A warrior, thief, or priest
  fully in magic gear is hit 1 to 9 % from level 20 to 60, and a monster needs hundreds of rounds
  to kill them.
- Hit points grow to level 50 (above).
- Monster Protection now matters: the median goes from 0 at level 15 to 6 at 60, not 0 to 2.
- The wizard's spells keep up to level 60 per target (7 casts of a tier-3 burst), but only
  because Archmage binds the cap on every roll. There is still no spell above tier 3 (§14).

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
4. **Archmage is wasted against the cap** (before S1, which retired the cap: Archmage's points
   now all count). At level 60, Fireball rolls 6d6+18 (mean 39) and is
   held to 32 on nearly every cast, so the per-die bonus does almost nothing for tier 0 to 3
   spells. This is by design ("caps, not curves"), but it means the tier 4 to 9 spells (§14) are
   the missing content, not more lift. Cantrips (Ray of frost, Spark) have no `[spell.scales]`
   and do not grow with stats (data: `spells/*.toml`).
5. **The priest has no damage spell of its own.** Its schools (restoration, abjuration,
   divination) have no damage spell. Smite and its class actives are its offence, and the probe
   does not model them.
6. **Monster blows can drop with gear.** As in the report, some types deal less damage with the
   weapon they roll than with their natural attack. The geared mean of damage a blow is below the
   bare median from level 15 up (`gear.toml`, the creature files).

## Talents coverage

Every system the design added meets the coverage bar (`assets/data/talents.toml`, `min_talents`
2, checked when the game loads):
- `weapon_abilities` has 5 talents (Early mastery, Second blade's rend, Brace of steel, Practised
  technique, Drillmaster).
- `stamina` has 6 (Trained wind, Deep lungs, Steady breathing, Second breath, Armour-breaker,
  Drillmaster).
- `stat_training` has 10 (Hard training, Quick study of the body, Hard-won, Gifted, Late bloomer,
  Ascetic, Gourmand, Cauldron, and the class abilities Hard living and Discipline; #726).
- `potions` has 4 (Thrifty drinker, Nose for draughts, Lasting draughts, Cauldron-wise).

No follow-up is needed.
