# Combat balance: the player against the monsters of their level

The balance probe measures how a fight between the player and the monsters of their level goes,
level by level. It reads every number through the game's own code. It was first run, and not
kept, for the balance report of 2026-10-27 (the combat progression design,
`docs/design/combat-progression.md` §2). It is kept now as a test (#569, the design's §15 D1)
so that the first-guess numbers in the data can be judged again after every tuning PR. It was
first kept at #569, after every slice of that design had landed: the new rank tables (W1),
weapon abilities and stamina (W2 to W4, SP1), spell scales (S1), monster Protection by body (P1),
stat training and the daily check (T1, T2), hit points to level 50 (H1), and potions and elixirs
(Q1 to Q3). The tables below are its output on `main` at the time of #688 (Surprise S7), which
added the "Sneak attacks" table; the sections after them ("Against the design's predictions" and
"Tuning notes") were written against #569's run, and their figures are that run's.

## How to run it

```text
cargo test --release -p warlock-core balance_probe -- --ignored --nocapture
```

It takes a few seconds in release. It prints the Markdown tables of the sections from "Players"
to "Sneak attacks" below, and the same content always prints the same tables (fixed seeds and streams).
After a PR that tunes combat data (weapon skills, creatures, classes, spells, stats, items), run
it again and replace those sections, then revisit the tuning notes. The code is
`crates/warlock-core/src/world/balance_probe.rs` (the measuring, with the method in its module
docs) and `balance_probe/tables.rs` (the tables).

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
- **Trained stats**: before each level come two game days of marks, then the daily check at each
  day's 06:00 (`World::daily_stat_check`), then the level's own check. The marks a day follow
  the design's per-use arithmetic (§9.1). The warrior gets St 7, Dx 3, To 4. The thief gets St 4,
  Dx 7, To 4. The wizard gets To 2, In 3, Wi 5. The priest gets St 4, To 3, Wi 5. Every other
  stat gets 1 a day. **Untrained** players get no marks.
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
  (`Actor::spell_cap`) and without it (`SpellRules::cap_of`). The dice are those of
  `SpellDice::lifted` and the talents, held to the cap. Attack spells strike the monster's
  Defence against missiles, and areas always hit.
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
| 1 | +4/8.2/16/3 | +4/8.2/16/3 | +0/2.8/13/1 | +0/2.8/13/1 | +0/3.0/12/0 | +2/5.8/12/0 | +2/5.8/12/0 |
| 5 | +5/9.5/18/4 | +6/11.2/20/4 | +2/4.2/16/1 | +2/5.2/18/2 | +1/3.2/13/0 | +3/7.2/16/2 | +4/7.9/18/2 |
| 10 | +6/11.2/22/5 | +8/12.6/25/6 | +4/6.0/20/1 | +6/7.7/21/2 | +2/4.8/15/0 | +5/9.0/17/4 | +7/11.2/20/5 |
| 15 | +8/12.8/25/9 | +13/16.2/28/10 | +7/6.8/20/2 | +9/9.3/24/4 | +3/5.5/16/1 | +7/10.8/21/5 | +9/14.8/25/5 |
| 20 | +10/14.7/28/10 | +14/20.1/33/12 | +10/8.7/22/2 | +12/10.1/26/4 | +5/7.2/18/1 | +8/12.5/24/7 | +11/15.1/29/8 |
| 30 | +12/16.8/33/12 | +21/24.0/41/18 | +12/10.5/23/2 | +18/18.0/30/8 | +6/9.0/19/1 | +9/14.5/26/8 | +17/19.9/34/15 |
| 40 | +14/19.5/36/12 | +22/25.7/45/16 | +14/12.2/25/2 | +23/16.8/33/5 | +7/9.7/20/1 | +11/17.2/28/8 | +20/23.1/37/12 |
| 50 | +15/20.3/36/12 | +20/26.8/46/17 | +15/13.2/25/2 | +19/21.1/31/7 | +8/10.0/20/1 | +12/18.8/28/8 | +18/27.1/36/13 |
| 60 | +16/23.5/36/12 | +22/29.5/44/15 | +16/15.8/25/2 | +23/20.9/32/7 | +8/12.5/20/1 | +12/20.7/28/8 | +20/25.8/37/13 |

Stats, hit points, and mana (plain gear; trained → untrained). Scores St/Dx/To/In/Wi/Pr, the seeds' mean.

| L | Warrior | Thief | Wizard | Priest | HP W/T/Wz/P | MP Wz/P | Caster level Wz/P | Blows a round W/T |
|---|---|---|---|---|---|---|---|---|
| 1 | 15/11/12/10/12/10 | 11/12/10/11/10/10 | 10/10/10/14/12/10 | 11/9/11/11/15/11 | 23→23 / 18→18 / 13→13 / 18→18 | 16/13 | 1/1 | 0.99/0.99 |
| 5 | 16/12/13/11/12/10 | 12/14/12/12/11/10 | 10/11/10/15/14/10 | 12/9/12/12/16/11 | 39→36 / 28→26 / 18→17 / 29→26 | 52/41 | 6/5 | 0.88/0.99 |
| 10 | 17/13/15/12/13/11 | 14/16/14/13/11/11 | 11/12/12/16/16/11 | 14/10/14/13/17/12 | 64→54 / 46→36 / 24→22 / 46→37 | 99/80 | 12/10 | 0.89/1.04 |
| 15 | 18/14/16/13/14/12 | 15/17/16/14/12/12 | 12/12/13/17/18/12 | 15/11/15/14/18/13 | 90→72 / 69→47 / 33→28 / 67→48 | 149/122 | 18/15 | 0.91/1.10 |
| 20 | 20/16/17/14/15/13 | 17/19/17/14/13/13 | 14/14/15/18/18/13 | 16/12/17/15/19/14 | 123→92 / 95→57 / 45→33 / 101→58 | 201/164 | 24/20 | 0.92/1.17 |
| 30 | 22/17/19/15/17/15 | 18/21/19/16/15/15 | 16/16/17/20/20/14 | 18/14/18/16/21/16 | 207→129 / 175→79 / 94→43 / 167→79 | 312/256 | 36/30 | 0.94/1.24 |
| 40 | 24/18/20/16/18/16 | 20/22/21/18/17/16 | 17/16/19/21/22/16 | 20/15/20/18/23/17 | 291→166 / 252→100 / 141→53 / 238→101 | 429/356 | 48/40 | 0.95/1.42 |
| 50 | 25/20/22/18/19/17 | 21/24/22/19/18/17 | 18/18/19/22/24/17 | 21/17/21/18/24/18 | 379→202 / 332→120 / 191→63 / 313→122 | 554/463 | 60/50 | 0.95/1.49 |
| 60 | 27/22/24/19/20/18 | 23/26/24/19/19/18 | 18/18/21/23/25/17 | 23/17/22/19/26/19 | 390→205 / 342→122 / 201→64 / 324→124 | 684/578 | 72/60 | 0.95/1.56 |

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
| 50 | 23 | 228 | 14 (10–18) → 14.3 | 32 (16–40) → 29.4 | 19 (12–25) → 19.3 | 4 (0–13) → 4.5 | 1.05 |
| 60 | 17 | 225 | 17 (12–21) → 16.9 | 34 (18–41) → 30.9 | 21 (15–27) → 22.0 | 6 (0–16) → 6.0 | 1.15 |
| 70 | 7 | 197 | 20 (19–23) → 20.1 | 32 (20–40) → 32.1 | 26 (12–29) → 25.0 | 5 (0–12) → 4.9 | 1.35 |

## Fights

Each cell: P(player hits) / P(monster hits) % · damage a round player : monster · rounds to kill / rounds to die; the median over the monsters within two levels (trained stats, monsters with gear).

| L | Warrior kit | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain (melee) | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|---|
| 1 | 87/35 · 7.5:0.1 · 1.6 / 305 | 87/35 · 7.5:0.1 · 1.6 / 353 | 87/35 · 7.5:0.1 · 1.6 / 307 | 67/55 · 2.4:0.5 · 6.2 / 35 | 67/55 · 2.5:0.6 · 6.3 / 34 | 71/62 · 2.3:1.1 · 5.6 / 14 | 77/59 · 4.7:1.0 · 2.6 / 19 | 77/59 · 4.6:1.0 · 2.7 / 19 |
| 5 | 89/31 · 8.9:0.7 · 2.6 / 63 | 89/25 · 7.9:0.4 · 3.0 / 92 | 91/14 · 9.7:0.2 · 2.5 / 324 | 75/43 · 4.0:1.4 · 6.9 / 21 | 76/30 · 4.8:0.9 · 5.4 / 76 | 66/59 · 2.0:2.4 · 13 / 7.9 | 80/42 · 5.1:1.2 · 4.6 / 25 | 83/28 · 5.8:0.7 · 3.8 / 63 |
| 10 | 91/18 · 11.2:1.1 · 3.8 / 63 | 91/13 · 9.9:0.7 · 4.3 / 114 | 93/5 · 11.1:0.3 · 3.9 / >999 | 83/25 · 8.5:1.8 · 6.4 / 30 | 89/17 · 11.1:1.3 · 5.1 / 78 | 71/56 · 3.5:4.7 · 13 / 5.5 | 84/40 · 6.6:2.4 · 6.8 / 21 | 92/22 · 9.5:1.2 · 4.6 / 67 |
| 15 | 95/8 · 13.2:0.6 · 4.7 / 193 | 95/9 · 11.7:0.4 · 5.5 / 302 | 99/4 · 16.7:0.2 · 4.1 / >999 | 92/30 · 10.1:3.2 · 7.8 / 24 | 96/14 · 14.7:1.1 · 6.3 / 95 | 75/65 · 3.5:7.6 · 17 / 4.4 | 91/27 · 8.2:2.3 · 8.0 / 32 | 94/12 · 12.2:1.0 · 5.6 / >999 |
| 20 | 98/3 · 16.3:0.3 · 5.0 / 876 | 98/5 · 14.5:0.3 · 5.8 / 915 | 99/1 · 22.2:0.0 · 4.0 / >999 | 97/29 · 14.8:3.1 · 7.0 / 36 | 98/12 · 18.9:1.1 · 5.5 / 356 | 79/65 · 5.0:7.6 · 17 / 5.9 | 94/22 · 9.8:1.7 · 8.4 / 70 | 98/6 · 13.2:0.3 · 6.1 / >999 |
| 30 | 98/1 · 20.2:0.2 · 7.1 / >999 | 98/2 · 17.7:0.2 · 8.3 / >999 | 99/1 · 29.2:0.1 · 5.2 / >999 | 98/44 · 19.0:9.4 · 8.2 / 22 | 99/7 · 38.5:0.9 · 3.8 / 952 | 76/72 · 4.8:15.3 · 23 / 6.2 | 92/26 · 9.9:4.3 · 13 / 42 | 99/2 · 20.7:0.2 · 7.2 / >999 |
| 40 | 98/1 · 23.2:0.2 · 7.6 / >999 | 98/1 · 20.2:0.2 · 8.7 / >999 | 99/1 · 35.0:0.1 · 5.2 / >999 | 98/46 · 25.5:10.7 · 7.8 / 27 | 99/8 · 45.3:1.4 · 4.5 / 967 | 70/82 · 5.4:17.0 · 32 / 8.7 | 90/27 · 12.6:5.6 · 14 / 47 | 99/1 · 23.3:0.1 · 7.2 / >999 |
| 50 | 97/4 · 24.1:0.7 · 9.9 / >999 | 97/8 · 20.9:1.3 · 11 / 742 | 99/1 · 33.9:0.1 · 7.3 / >999 | 98/67 · 27.1:18.9 · 9.7 / 19 | 99/31 · 44.5:7.7 · 5.0 / 126 | 65/92 · 4.4:24.9 · 55 / 7.7 | 88/49 · 12.6:13.1 · 18 / 25 | 97/10 · 25.4:2.2 · 9.0 / 914 |
| 60 | 92/7 · 21.1:2.7 · 10 / 185 | 92/16 · 18.4:4.9 · 11 / 89 | 98/1 · 28.3:0.2 · 6.7 / >999 | 92/84 · 25.4:27.0 · 10 / 13 | 99/37 · 40.3:12.0 · 6.0 / 54 | 46/98 · 3.5:31.9 · 77 / 6.5 | 78/71 · 12.2:20.6 · 19 / 17 | 96/15 · 22.8:4.3 · 11 / 359 |

Out of depth (+3: the monsters one to five levels above), and untrained stats, for the warrior in plain gear.

| L | Warrior plain +3 | Warrior magic +3 | Warrior plain, untrained | Warrior magic, untrained |
|---|---|---|---|---|
| 1 | 83/35 · 7.2:0.6 · 2.9 / 42 | 83/35 · 7.2:0.6 · 2.8 / 43 | 87/35 · 7.6:0.1 · 1.6 / 334 | 87/35 · 7.5:0.1 · 1.6 / 324 |
| 5 | 89/31 · 7.8:1.3 · 4.5 / 31 | 91/19 · 9.7:0.7 · 3.7 / 79 | 87/28 · 7.4:0.5 · 3.2 / 86 | 90/16 · 9.4:0.2 · 2.6 / 304 |
| 10 | 92/21 · 9.3:1.4 · 6.4 / 51 | 93/9 · 10.5:0.6 · 5.5 / 683 | 87/18 · 8.6:0.9 · 5.1 / 73 | 90/9 · 9.7:0.4 · 4.5 / >999 |
| 15 | 93/12 · 11.5:0.7 · 6.3 / 154 | 98/5 · 16.7:0.3 · 4.6 / >999 | 90/12 · 8.7:0.6 · 7.5 / 181 | 98/5 · 13.7:0.3 · 4.9 / >999 |
| 20 | 96/8 · 14.1:0.7 · 7.9 / 326 | 99/1 · 21.5:0.1 · 5.3 / >999 | 91/8 · 10.0:0.4 · 9.0 / 512 | 98/1 · 19.8:0.0 · 4.5 / >999 |
| 30 | 98/3 · 16.4:0.4 · 9.2 / 906 | 99/1 · 28.1:0.1 · 5.6 / >999 | 89/5 · 11.3:0.7 · 12 / 422 | 99/1 · 25.7:0.1 · 6.1 / >999 |
| 40 | 98/2 · 20.1:0.3 · 9.6 / >999 | 99/1 · 34.6:0.1 · 5.7 / >999 | 83/3 · 13.2:0.7 · 13 / >999 | 98/1 · 29.2:0.1 · 6.4 / >999 |
| 50 | 95/8 · 18.4:1.8 · 12 / 519 | 99/1 · 30.3:0.1 · 7.6 / >999 | 78/14 · 11.5:2.7 · 21 / 129 | 93/1 · 25.1:0.1 · 9.8 / >999 |
| 60 | 82/31 · 14.5:8.6 · 17 / 54 | 96/2 · 25.2:0.4 · 9.4 / >999 | 58/27 · 7.2:8.1 · 41 / 28 | 88/1 · 16.8:0.4 · 14 / >999 |

## Protection by body

Rounds for the plain warrior (trained) to kill the monsters of their level: as shipped, then with every monster's Protection set to §8.2's body line at its level.

| L | as shipped | none (0) | hide | bony | armoured |
|---|---|---|---|---|---|
| 10 | 4.3 | 3.9 | 3.9 | 4.3 | 5.0 |
| 15 | 5.4 | 4.7 | 4.7 | 5.1 | 6.1 |
| 20 | 5.9 | 4.9 | 5.1 | 5.6 | 6.4 |
| 30 | 8.3 | 6.1 | 6.7 | 7.5 | 8.8 |
| 40 | 8.8 | 6.4 | 7.1 | 7.7 | 9.1 |
| 50 | 11 | 9.0 | 10 | 11 | 14 |
| 60 | 11 | 7.3 | 8.4 | 8.9 | 11 |

## Spells

Each damage spell as the trained caster casts it (plain gear): the lift and the cap it lifts, the dice, and the expected casts to kill the median monster of their level (an attack spell's miss chance included; per target), lifted (unlifted in brackets); mana per cast of the pool.

**Wizard**

| Spell (tier, resolution) | L | lift % | cap → lifted | dice | casts to kill | mana / pool |
|---|---|---|---|---|---|---|
| Ray of frost (0, Attack) | 10 | 0 | 6 → 6 | 1d6+1, 4.5 | 11 (11) | 1 / 99 |
| Ray of frost (0, Attack) | 30 | 0 | 6 → 6 | 1d6+1, 4.5 | 30 (30) | 1 / 312 |
| Ray of frost (0, Attack) | 60 | 0 | 6 → 6 | 2d6+6, 13.0 | 64 (61) | 1 / 684 |
| Spark (0, Auto) | 10 | 0 | 4 → 4 | 1d4+1, 3.5 | 12 (12) | 1 / 99 |
| Spark (0, Auto) | 30 | 0 | 4 → 4 | 1d4+1, 3.5 | 36 (36) | 1 / 312 |
| Spark (0, Auto) | 60 | 0 | 4 → 4 | 2d4+6, 11.0 | 57 (57) | 1 / 684 |
| Burning hands (1, Area) | 10 | 45 | 9 → 13 | 3d4+3, 10.5 | 4.1 (5.6) | 5 / 99 |
| Burning hands (1, Area) | 30 | 50 | 9 → 13 | 3d4+3, 10.5 | 11 (17) | 5 / 312 |
| Burning hands (1, Area) | 60 | 50 | 9 → 13 | 4d4+12, 22.0 | 18 (26) | 2 / 684 |
| Cause wounds (1, Touch) | 10 | 0 | 14 → 14 | 2d6+2, 9.0 | 5.4 (5.5) | 2 / 99 |
| Cause wounds (1, Touch) | 30 | 0 | 14 → 14 | 2d6+2, 9.0 | 15 (15) | 2 / 312 |
| Cause wounds (1, Touch) | 60 | 0 | 14 → 14 | 3d6+9, 19.5 | 29 (30) | 1 / 684 |
| Magic missile (1, Auto) | 10 | 45 | 9 → 13 | 3d4+3, 10.5 | 4.0 (5.6) | 3 / 99 |
| Magic missile (1, Auto) | 30 | 50 | 9 → 13 | 3d4+3, 10.5 | 11 (17) | 3 / 312 |
| Magic missile (1, Auto) | 60 | 50 | 9 → 13 | 4d4+12, 22.0 | 18 (26) | 1 / 684 |
| Arcane bolt (2, Area) | 10 | 45 | 14 → 20 | 3d6+3, 13.5 | 2.9 (4.3) | 11 / 99 |
| Arcane bolt (2, Area) | 30 | 60 | 14 → 22 | 3d6+3, 13.5 | 8.7 (13) | 11 / 312 |
| Arcane bolt (2, Area) | 60 | 60 | 14 → 22 | 4d6+12, 26.0 | 11 (16) | 4 / 684 |
| Blight (2, Area) | 10 | 0 | 14 → 14 | 2d6+2, 9.0 | 4.3 (4.3) | 14 / 99 |
| Blight (2, Area) | 30 | 0 | 14 → 14 | 2d6+2, 9.0 | 13 (13) | 14 / 312 |
| Blight (2, Area) | 60 | 0 | 14 → 14 | 3d6+9, 19.5 | 16 (16) | 5 / 684 |
| Frost (2, Area) | 10 | 45 | 14 → 20 | 3d6+3, 13.5 | 2.9 (4.3) | 14 / 99 |
| Frost (2, Area) | 30 | 60 | 14 → 22 | 3d6+3, 13.5 | 8.7 (13) | 14 / 312 |
| Frost (2, Area) | 60 | 60 | 14 → 22 | 4d6+12, 26.0 | 11 (16) | 5 / 684 |
| Scorching ray (2, Attack) | 10 | 45 | 18 → 26 | 3d6+3, 13.5 | 3.6 (4.9) | 6 / 99 |
| Scorching ray (2, Attack) | 30 | 60 | 18 → 28 | 4d6+4, 18.0 | 7.1 (9.8) | 6 / 312 |
| Scorching ray (2, Attack) | 60 | 60 | 18 → 28 | 5d6+15, 32.5 | 14 (21) | 2 / 684 |
| Call lightning (3, Area) | 10 | 45 | 19 → 27 | 3d6+3, 13.5 | 2.8 (4.3) | 20 / 99 |
| Call lightning (3, Area) | 30 | 70 | 19 → 32 | 5d6+5, 22.5 | 5.2 (8.7) | 20 / 312 |
| Call lightning (3, Area) | 60 | 70 | 19 → 32 | 6d6+18, 39.0 | 7.2 (12) | 8 / 684 |
| Fireball (3, Area) | 10 | 45 | 19 → 27 | 3d6+3, 13.5 | 2.8 (4.3) | 26 / 99 |
| Fireball (3, Area) | 30 | 70 | 19 → 32 | 5d6+5, 22.5 | 5.2 (8.8) | 26 / 312 |
| Fireball (3, Area) | 60 | 70 | 19 → 32 | 6d6+18, 39.0 | 7.2 (12) | 10 / 684 |
| Flame (3, Area) | 10 | 45 | 19 → 27 | 4d6+4, 18.0 | 2.1 (2.9) | 26 / 99 |
| Flame (3, Area) | 30 | 70 | 19 → 32 | 5d6+5, 22.5 | 5.2 (8.8) | 26 / 312 |
| Flame (3, Area) | 60 | 70 | 19 → 32 | 6d6+18, 39.0 | 7.2 (12) | 10 / 684 |
| Lightning bolt (3, Area) | 10 | 45 | 19 → 27 | 3d6+3, 13.5 | 2.8 (4.3) | 20 / 99 |
| Lightning bolt (3, Area) | 30 | 70 | 19 → 32 | 5d6+5, 22.5 | 5.2 (8.7) | 20 / 312 |
| Lightning bolt (3, Area) | 60 | 70 | 19 → 32 | 6d6+18, 39.0 | 7.2 (12) | 8 / 684 |

**Priest**

| Spell (tier, resolution) | L | lift % | cap → lifted | dice | casts to kill | mana / pool |
|---|---|---|---|---|---|---|
| none: no damage spell it knows or of a school it has an affinity with |||||||

## Sneak attacks

The thief's main-hand blow on the monsters of their level (trained stats), aware and caught unawares. The Sneak attack value assumed, the dice it gives with the talents, and the share of the monsters within two levels with vitals (no dice and no critical on the rest, so their first blow gains only the flat-footed Defence); then each cell: P(hit) aware / unaware % · damage of a blow aware : the first blow unaware · how many times as much, the median over the monsters within two levels that have vitals.

| L | Sneak attack | dice | with vitals | Thief plain | Thief magic |
|---|---|---|---|---|---|
| 1 | 15 | 1d6 | 71% | 75/75 · 2.4:5.4 · ×2.3 | 75/75 · 2.4:5.5 · ×2.3 |
| 5 | 29 | 2d6 | 55% | 75/85 · 3.5:9.9 · ×2.8 | 76/86 · 4.3:10.9 · ×2.5 |
| 10 | 47 | 2d6 | 72% | 87/94 · 5.8:13.0 · ×2.2 | 92/97 · 8.0:15.2 · ×1.9 |
| 15 | 64 | 3d6 | 63% | 96/99 · 7.2:18.1 · ×2.5 | 98/99 · 10.3:21.0 · ×2.0 |
| 20 | 82 | 4d6 | 66% | 99/99 · 9.3:23.4 · ×2.5 | 99/99 · 11.7:25.5 · ×2.2 |
| 30 | 117 | 5d6 | 62% | 99/99 · 11.1:28.5 · ×2.6 | 99/99 · 20.9:38.5 · ×1.8 |
| 40 | 152 | 8d6 | 59% | 98/99 · 12.7:40.6 · ×3.2 | 99/99 · 19.9:47.8 · ×2.4 |
| 50 | 187 | 8d6 | 52% | 98/99 · 12.6:40.4 · ×3.2 | 99/99 · 24.7:52.1 · ×2.1 |
| 60 | 222 | 9d6 | 47% | 96/99 · 15.0:48.9 · ×3.3 | 99/99 · 22.8:57.2 · ×2.5 |

The design (§4, "Numbers") expected the dice to "roughly triple the first blow of a fight" at
level 30. The probe measures 2.2 to 2.8 times from level 1 to 30 in plain gear and 3.2 to 3.3
from 40 (Improved sneak attack's 2d6), less in magic gear, where the blow itself is larger. The
flat-footed Defence adds little after level 15: the thief hits 96 % or more either way. Fewer
than half the monsters around level 60 have vitals (skeletons, spirits, constructs, elementals,
animated objects, the incorporeal), against about two thirds below 30: the deep is where the
thief's dice find the least to bite on.

## Against the design's predictions

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
   `daily_factor`.
3. **The thief in plain gear still falls behind late.** At Dv 25 and 340 hit points, the plain
   thief is hit 76 to 84 % at levels 50 to 60 and dies in 11 to 15 rounds, while killing in 9. Magic
   gear fixes it (hit 24 %). This is the report's finding 1 (gear decides), now limited to the
   light-armoured classes.
4. **Archmage is wasted against the cap.** At level 60, Fireball rolls 6d6+18 (mean 39) and is
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
- `stat_training` has 2 (Hard training, Quick study of the body).
- `potions` has 4 (Thrifty drinker, Nose for draughts, Lasting draughts, Cauldron-wise).

No follow-up is needed.
