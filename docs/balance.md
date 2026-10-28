# Combat balance: the player against the monsters of their level

The balance probe measures how a fight between the player and the monsters of their level goes,
level by level. It reads every number through the game's own code. It was first run, and not
kept, for the balance report of 2026-10-27 (the combat progression design,
`docs/design/combat-progression.md` §2). It is kept now as a test (#569, the design's §15 D1)
so that the first-guess numbers in the data can be judged again after every tuning PR. The tables
below are its output on `main` at the time of #569, after every slice of the design had landed:
the new rank tables (W1), weapon abilities and stamina (W2 to W4, SP1), spell scales (S1),
monster Protection by body (P1), stat training and the daily check (T1, T2), hit points to level
50 (H1), and potions and elixirs (Q1 to Q3).

## How to run it

```text
cargo test --release -p warlock-core balance_probe -- --ignored --nocapture
```

It takes a few seconds in release. It prints the Markdown tables of the sections from "Players"
to "Spells" below, and the same content always prints the same tables (fixed seeds and streams).
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
- **Not modelled**: triggered and active weapon abilities (Rend, Riposte, Sunder, ...), stances,
  sneak attacks, a talent build, potions, resistances against spells, healing between rounds,
  and fights against several monsters at once.

## Players

Each cell: to-hit / damage a hit / Dv / Pv, the seeds' mean; trained stats.

| L | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|
| 1 | +3/8.3/16/1 | +3/8.3/16/1 | +0/2.8/13/1 | +0/2.8/13/1 | +0/3.0/12/0 | +1/5.8/13/0 | +1/5.8/13/0 |
| 5 | +4/9.7/18/4 | +6/11.3/24/5 | +2/4.2/16/1 | +2/4.5/20/3 | +1/3.2/14/0 | +3/7.2/16/2 | +5/8.5/20/3 |
| 10 | +6/11.3/22/5 | +9/13.3/27/7 | +5/6.2/20/1 | +8/8.0/23/3 | +2/4.8/15/0 | +4/9.0/18/4 | +7/11.4/23/6 |
| 15 | +8/13.2/25/9 | +13/16.7/31/10 | +7/6.8/20/2 | +9/9.8/25/4 | +3/5.3/16/1 | +6/10.8/21/5 | +8/13.6/29/6 |
| 20 | +10/14.8/28/10 | +14/20.9/36/11 | +10/8.5/22/2 | +12/10.8/28/4 | +4/6.8/17/1 | +8/12.7/24/7 | +12/15.2/30/8 |
| 30 | +12/16.8/33/12 | +22/28.2/44/16 | +12/10.5/23/2 | +21/15.5/32/8 | +5/8.8/19/1 | +9/14.5/26/8 | +19/22.2/36/15 |
| 40 | +14/19.8/36/12 | +24/24.9/48/19 | +14/12.2/25/2 | +26/17.1/34/6 | +7/9.7/20/1 | +11/17.3/28/8 | +23/23.6/39/14 |
| 50 | +14/20.8/36/12 | +24/30.2/48/18 | +15/13.2/25/2 | +23/21.8/34/7 | +7/9.7/20/1 | +11/19.7/28/8 | +22/29.7/40/13 |
| 60 | +15/23.0/36/12 | +26/28.9/48/16 | +16/15.7/25/2 | +26/25.6/35/8 | +8/12.2/20/1 | +12/20.7/28/8 | +22/28.8/39/13 |

Stats, hit points, and mana (plain gear; trained → untrained). Scores St/Dx/To/In/Wi/Pr, the seeds' mean.

| L | Warrior | Thief | Wizard | Priest | HP W/T/Wz/P | MP Wz/P | Caster level Wz/P | Blows a round W/T |
|---|---|---|---|---|---|---|---|---|
| 1 | 15/11/12/9/10/10 | 11/13/10/11/9/10 | 10/10/9/14/11/10 | 11/9/10/11/14/12 | 22→22 / 18→18 / 12→12 / 18→18 | 15/12 | 1/1 | 0.99/0.99 |
| 5 | 16/12/14/10/11/10 | 12/14/12/12/10/10 | 10/11/10/15/13/10 | 12/9/12/12/15/12 | 40→37 / 28→26 / 17→16 / 28→26 | 48/37 | 6/5 | 0.88/0.99 |
| 10 | 17/13/15/12/12/11 | 14/16/14/13/11/11 | 11/12/11/16/15/12 | 14/10/14/13/16/12 | 67→55 / 47→37 / 24→21 / 46→37 | 93/71 | 12/10 | 0.89/1.04 |
| 15 | 18/15/16/12/13/12 | 15/18/16/14/11/12 | 12/13/13/17/16/12 | 15/11/15/14/17/13 | 96→76 / 71→47 / 32→26 / 70→47 | 141/107 | 18/15 | 0.91/1.10 |
| 20 | 20/16/17/13/14/13 | 17/19/17/14/12/13 | 13/14/14/18/17/14 | 17/12/16/15/18/14 | 133→96 / 102→58 / 43→31 / 102→58 | 191/145 | 24/20 | 0.92/1.17 |
| 30 | 22/17/19/15/16/15 | 18/21/19/16/14/15 | 15/15/16/19/19/16 | 18/14/18/16/20/16 | 215→134 / 175→78 / 79→41 / 164→78 | 295/228 | 36/30 | 0.94/1.24 |
| 40 | 24/18/20/16/17/16 | 20/22/21/17/16/16 | 16/16/18/20/21/17 | 20/16/20/18/21/17 | 300→174 / 250→100 / 126→51 / 236→100 | 408/319 | 48/40 | 0.95/1.43 |
| 50 | 25/20/22/18/18/18 | 21/24/22/18/17/18 | 17/18/19/22/22/18 | 21/17/21/18/23/18 | 389→212 / 330→120 / 170→60 / 310→120 | 528/418 | 60/50 | 0.95/1.50 |
| 60 | 27/22/24/18/19/18 | 23/26/24/19/18/18 | 18/18/20/23/24/19 | 23/18/22/19/24/19 | 400→215 / 340→122 / 180→60 / 321→122 | 656/525 | 72/60 | 0.95/1.56 |

The Defence of the warrior and the thief in plain gear by category (trained → untrained), the seeds' mean.

| Class | L | Dexterity | Armour | Shield | Parry | Natural | Dodge | Deflection | Wall | total |
|---|---|---|---|---|---|---|---|---|---|---|
| warrior | 10 | 1→0 | 2→2 | 6→6 | 2→2 | 0→0 | 1→1 | 0→0 | 0→0 | 22→21 |
| warrior | 30 | 2→0 | 1→1 | 14→14 | 5→5 | 0→0 | 1→1 | 0→0 | 0→0 | 33→31 |
| warrior | 60 | 2→0 | 1→1 | 16→16 | 6→6 | 0→0 | 1→1 | 0→0 | 0→0 | 36→34 |
| thief | 10 | 3→1 | 2→2 | 0→0 | 3→3 | 0→0 | 2→2 | 0→0 | 0→0 | 20→18 |
| thief | 30 | 5→1 | 2→2 | 0→0 | 4→4 | 0→0 | 2→2 | 0→0 | 0→0 | 23→19 |
| thief | 60 | 5→1 | 2→2 | 0→0 | 6→6 | 0→0 | 2→2 | 0→0 | 0→0 | 25→21 |

## Monsters

Within two levels: the median (range) bare → the mean with gear. Damage is a hit's mean before soak.

| L (±2) | n | HP | to-hit | dmg/blow | Dv | Pv | blows a round |
|---|---|---|---|---|---|---|---|
| 1 | 38 | 11 | -1 (-5–2) → -0.4 | 2 (1–6) → 2.1 | 12 (7–14) → 11.1 | 0 (0–3) → 0.3 | 1.00 |
| 5 | 51 | 19 | 0 (-4–4) → 0.5 | 3 (1–11) → 3.8 | 12 (7–21) → 12.1 | 0 (0–2) → 0.5 | 1.00 |
| 10 | 48 | 38 | 2 (0–7) → 2.9 | 6 (2–16) → 8.2 | 12 (7–18) → 12.6 | 0 (0–5) → 0.7 | 1.00 |
| 15 | 41 | 56 | 4 (1–9) → 4.5 | 12 (4–20) → 11.8 | 12 (8–19) → 13.6 | 0 (0–5) → 1.1 | 1.00 |
| 20 | 42 | 71 | 6 (2–10) → 5.9 | 14 (4–22) → 13.9 | 14 (11–19) → 14.4 | 1 (0–9) → 1.7 | 1.00 |
| 30 | 29 | 112 | 9 (7–13) → 9.4 | 21 (10–27) → 20.6 | 16 (12–21) → 16.3 | 2 (0–11) → 2.9 | 1.04 |
| 40 | 22 | 134 | 11 (9–16) → 12.1 | 24 (11–32) → 22.2 | 18 (15–22) → 18.4 | 3 (0–11) → 3.6 | 1.09 |
| 50 | 18 | 191 | 16 (13–18) → 15.6 | 29 (16–40) → 28.3 | 19 (14–25) → 20.6 | 4 (0–13) → 4.4 | 1.11 |
| 60 | 14 | 217 | 17 (15–21) → 17.9 | 32 (18–41) → 29.5 | 21 (15–28) → 23.3 | 6 (0–16) → 6.1 | 1.22 |
| 70 | 7 | 197 | 21 (19–23) → 20.2 | 32 (20–40) → 32.0 | 25 (16–29) → 25.4 | 5 (0–12) → 5.1 | 1.33 |

## Fights

Each cell: P(player hits) / P(monster hits) % · damage a round player : monster · rounds to kill / rounds to die; the median over the monsters within two levels (trained stats, monsters with gear).

| L | Warrior kit | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain (melee) | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|---|
| 1 | 81/30 · 7.7:0.3 · 1.5 / 73 | 81/30 · 7.5:0.3 · 1.5 / 75 | 81/30 · 7.6:0.3 · 1.5 / 76 | 62/49 · 2.5:0.5 · 5.5 / 37 | 62/49 · 2.5:0.5 · 5.5 / 38 | 65/57 · 2.2:1.1 · 5.2 / 13 | 71/52 · 4.5:1.0 · 2.6 / 20 | 71/52 · 4.5:1.0 · 2.6 / 20 |
| 5 | 88/28 · 9.0:0.8 · 2.3 / 56 | 88/22 · 8.0:0.4 · 2.7 / 108 | 92/3 · 10.2:0.0 · 2.2 / >999 | 75/38 · 3.6:1.0 · 6.3 / 31 | 76/14 · 4.3:0.3 · 5.2 / 557 | 66/53 · 2.0:1.8 · 11 / 9.5 | 79/35 · 4.9:0.8 · 4.3 / 37 | 87/14 · 7.1:0.3 · 3.1 / 225 |
| 10 | 90/19 · 11.1:1.4 · 3.6 / 58 | 90/14 · 9.6:0.6 · 4.2 / 138 | 97/4 · 12.2:0.2 · 3.3 / >999 | 84/26 · 8.2:1.9 · 5.8 / 32 | 95/11 · 13.5:0.8 · 3.9 / 578 | 71/58 · 3.4:4.5 · 12 / 5.9 | 83/41 · 6.3:2.1 · 6.8 / 24 | 93/20 · 10.2:1.0 · 4.0 / 904 |
| 15 | 93/9 · 13.6:0.8 · 4.2 / 154 | 93/9 · 12.0:0.4 · 5.0 / 418 | 98/2 · 18.5:0.1 · 3.4 / >999 | 91/33 · 8.8:3.5 · 7.4 / 22 | 94/11 · 13.4:1.0 · 5.2 / 237 | 69/66 · 3.1:7.7 · 19 / 4.2 | 88/29 · 8.2:2.4 · 7.8 / 32 | 91/9 · 11.9:0.8 · 5.9 / >999 |
| 20 | 96/3 · 16.4:0.4 · 4.3 / >999 | 96/5 · 14.4:0.3 · 5.0 / >999 | 99/2 · 21.6:0.1 · 3.4 / >999 | 96/29 · 13.8:3.8 · 6.1 / 35 | 98/6 · 19.7:0.6 · 4.4 / 731 | 77/67 · 4.3:8.7 · 18 / 5.2 | 93/22 · 9.8:2.1 · 7.2 / 66 | 97/5 · 13.1:0.4 · 5.6 / >999 |
| 30 | 95/1 · 19.0:0.4 · 6.4 / >999 | 95/1 · 17.1:0.3 · 7.4 / >999 | 99/1 · 40.5:0.1 · 2.9 / >999 | 97/44 · 17.2:9.8 · 8.0 / 19 | 99/4 · 32.8:0.9 · 4.3 / >999 | 71/74 · 4.4:18.7 · 26 / 4.3 | 89/26 · 9.7:5.1 · 12 / 34 | 99/1 · 23.5:0.1 · 5.6 / >999 |
| 40 | 97/1 · 24.2:0.2 · 6.3 / >999 | 97/2 · 21.0:0.2 · 7.3 / >999 | 99/1 · 38.8:0.1 · 3.9 / >999 | 97/51 · 23.6:11.4 · 6.9 / 24 | 99/8 · 43.8:1.3 · 3.8 / >999 | 70/83 · 4.8:18.3 · 32 / 7.1 | 88/31 · 11.9:6.1 · 13 / 44 | 99/1 · 23.5:0.1 · 6.3 / >999 |
| 50 | 94/6 · 23.3:1.7 · 9.0 / 698 | 94/11 · 20.7:2.4 · 10 / 454 | 99/1 · 45.5:0.1 · 4.6 / >999 | 96/76 · 25.4:22.5 · 9.4 / 15 | 99/24 · 48.9:4.9 · 4.6 / 340 | 57/95 · 3.7:29.5 · 61 / 5.8 | 81/58 · 12.4:15.2 · 18 / 21 | 99/5 · 28.9:1.0 · 7.7 / >999 |
| 60 | 80/9 · 29.2:2.8 · 8.0 / 206 | 80/18 · 25.1:5.6 · 9.6 / 94 | 99/1 · 51.1:0.1 · 4.5 / >999 | 83/84 · 27.9:31.7 · 8.9 / 11 | 99/25 · 64.7:8.1 · 3.8 / 347 | 38/99 · 3.4:35.0 · 113 / 5.2 | 67/69 · 10.2:24.3 · 22 / 13 | 97/9 · 28.6:2.2 · 8.1 / 852 |

Out of depth (+3: the monsters one to five levels above), and untrained stats, for the warrior in plain gear.

| L | Warrior plain +3 | Warrior magic +3 | Warrior plain, untrained | Warrior magic, untrained |
|---|---|---|---|---|
| 1 | 81/34 · 7.3:0.6 · 2.5 / 43 | 81/34 · 7.4:0.7 · 2.5 / 38 | 81/30 · 7.7:0.3 · 1.5 / 72 | 81/30 · 7.6:0.3 · 1.5 / 74 |
| 5 | 83/31 · 7.7:1.3 · 4.0 / 34 | 89/6 · 9.8:0.2 · 3.2 / 489 | 86/24 · 7.5:0.4 · 2.8 / 91 | 91/3 · 9.4:0.0 · 2.4 / >999 |
| 10 | 90/22 · 9.1:1.6 · 5.9 / 51 | 97/7 · 11.6:0.5 · 4.7 / 976 | 86/19 · 8.4:0.9 · 4.8 / 84 | 95/7 · 11.0:0.3 · 3.7 / >999 |
| 15 | 93/11 · 11.6:0.7 · 5.9 / 213 | 98/2 · 17.9:0.1 · 3.7 / >999 | 87/13 · 9.0:0.7 · 6.5 / 182 | 97/3 · 15.9:0.1 · 3.8 / >999 |
| 20 | 95/7 · 13.9:0.9 · 7.0 / 288 | 98/2 · 21.2:0.3 · 4.6 / >999 | 90/8 · 10.1:0.5 · 7.6 / 498 | 96/2 · 18.5:0.1 · 4.1 / >999 |
| 30 | 95/3 · 16.2:0.5 · 8.0 / >999 | 99/1 · 40.9:0.1 · 3.2 / >999 | 85/5 · 11.0:1.0 · 12 / 594 | 99/1 · 35.6:0.1 · 3.4 / >999 |
| 40 | 94/3 · 19.6:0.5 · 8.2 / >999 | 99/1 · 39.5:0.1 · 4.0 / >999 | 81/5 · 13.0:0.8 · 13 / 648 | 99/1 · 32.0:0.1 · 5.1 / >999 |
| 50 | 89/11 · 18.7:2.8 · 11 / 281 | 99/1 · 45.1:0.1 · 4.9 / >999 | 70/20 · 10.9:4.5 · 23 / 66 | 97/1 · 32.5:0.1 · 7.1 / >999 |
| 60 | 74/28 · 23.8:8.0 · 11 / 58 | 99/1 · 49.9:0.2 · 4.9 / >999 | 48/30 · 8.7:9.4 · 43 / 30 | 95/1 · 34.4:0.2 · 6.7 / >999 |

## Protection by body

Rounds for the plain warrior (trained) to kill the monsters of their level: as shipped, then with every monster's Protection set to §8.2's body line at its level.

| L | as shipped | none (0) | hide | bony | armoured |
|---|---|---|---|---|---|
| 10 | 4.2 | 3.8 | 3.8 | 4.1 | 4.8 |
| 15 | 4.9 | 4.4 | 4.4 | 4.8 | 5.8 |
| 20 | 5.0 | 4.6 | 4.8 | 5.3 | 6.0 |
| 30 | 7.6 | 6.1 | 6.5 | 7.1 | 8.3 |
| 40 | 7.3 | 5.5 | 6.2 | 6.9 | 8.2 |
| 50 | 10 | 8.8 | 10.0 | 11 | 14 |
| 60 | 9.5 | 7.6 | 8.7 | 9.5 | 12 |

## Spells

Each damage spell as the trained caster casts it (plain gear): the lift and the cap it lifts, the dice, and the expected casts to kill the median monster of their level (an attack spell's miss chance included; per target), lifted (unlifted in brackets); mana per cast of the pool.

**Wizard**

| Spell (tier, resolution) | L | lift % | cap → lifted | dice | casts to kill | mana / pool |
|---|---|---|---|---|---|---|
| Ray of frost (0, Attack) | 10 | 0 | 6 → 6 | 1d6+1, 4.5 | 10 (10) | 1 / 93 |
| Ray of frost (0, Attack) | 30 | 0 | 6 → 6 | 1d6+1, 4.5 | 30 (29) | 1 / 295 |
| Ray of frost (0, Attack) | 60 | 0 | 6 → 6 | 2d6+6, 13.0 | 69 (70) | 1 / 656 |
| Spark (0, Auto) | 10 | 0 | 4 → 4 | 1d4+1, 3.5 | 12 (11) | 1 / 93 |
| Spark (0, Auto) | 30 | 0 | 4 → 4 | 1d4+1, 3.5 | 35 (35) | 1 / 295 |
| Spark (0, Auto) | 60 | 0 | 4 → 4 | 2d4+6, 11.0 | 55 (55) | 1 / 656 |
| Burning hands (1, Area) | 10 | 45 | 9 → 13 | 3d4+3, 10.5 | 4.2 (5.4) | 5 / 93 |
| Burning hands (1, Area) | 30 | 50 | 9 → 13 | 3d4+3, 10.5 | 11 (17) | 5 / 295 |
| Burning hands (1, Area) | 60 | 50 | 9 → 13 | 4d4+12, 22.0 | 17 (24) | 2 / 656 |
| Cause wounds (1, Touch) | 10 | 0 | 14 → 14 | 2d6+2, 9.0 | 5.4 (5.3) | 2 / 93 |
| Cause wounds (1, Touch) | 30 | 0 | 14 → 14 | 2d6+2, 9.0 | 15 (15) | 2 / 295 |
| Cause wounds (1, Touch) | 60 | 0 | 14 → 14 | 3d6+9, 19.5 | 34 (34) | 1 / 656 |
| Magic missile (1, Auto) | 10 | 45 | 9 → 13 | 3d4+3, 10.5 | 4.2 (5.4) | 3 / 93 |
| Magic missile (1, Auto) | 30 | 50 | 9 → 13 | 3d4+3, 10.5 | 11 (17) | 3 / 295 |
| Magic missile (1, Auto) | 60 | 50 | 9 → 13 | 4d4+12, 22.0 | 17 (24) | 1 / 656 |
| Arcane bolt (2, Area) | 10 | 45 | 14 → 20 | 3d6+3, 13.5 | 2.8 (4.2) | 11 / 93 |
| Arcane bolt (2, Area) | 30 | 60 | 14 → 22 | 3d6+3, 13.5 | 8.5 (13) | 11 / 295 |
| Arcane bolt (2, Area) | 60 | 60 | 14 → 22 | 4d6+12, 26.0 | 10 (16) | 4 / 656 |
| Blight (2, Area) | 10 | 0 | 14 → 14 | 2d6+2, 9.0 | 4.2 (4.1) | 14 / 93 |
| Blight (2, Area) | 30 | 0 | 14 → 14 | 2d6+2, 9.0 | 13 (13) | 14 / 295 |
| Blight (2, Area) | 60 | 0 | 14 → 14 | 3d6+9, 19.5 | 16 (16) | 5 / 656 |
| Frost (2, Area) | 10 | 45 | 14 → 20 | 3d6+3, 13.5 | 2.8 (4.1) | 14 / 93 |
| Frost (2, Area) | 30 | 60 | 14 → 22 | 3d6+3, 13.5 | 8.6 (13) | 14 / 295 |
| Frost (2, Area) | 60 | 60 | 14 → 22 | 4d6+12, 26.0 | 10 (16) | 5 / 656 |
| Scorching ray (2, Attack) | 10 | 45 | 18 → 26 | 3d6+3, 13.5 | 3.6 (4.7) | 6 / 93 |
| Scorching ray (2, Attack) | 30 | 60 | 18 → 28 | 4d6+4, 18.0 | 7.0 (9.6) | 6 / 295 |
| Scorching ray (2, Attack) | 60 | 60 | 18 → 28 | 5d6+15, 32.5 | 14 (23) | 2 / 656 |
| Call lightning (3, Area) | 10 | 45 | 19 → 27 | 3d6+3, 13.5 | 2.8 (4.2) | 20 / 93 |
| Call lightning (3, Area) | 30 | 65 | 19 → 31 | 5d6+5, 22.5 | 5.3 (8.5) | 20 / 295 |
| Call lightning (3, Area) | 60 | 70 | 19 → 32 | 6d6+18, 39.0 | 6.9 (12) | 8 / 656 |
| Fireball (3, Area) | 10 | 45 | 19 → 27 | 3d6+3, 13.5 | 2.8 (4.2) | 26 / 93 |
| Fireball (3, Area) | 30 | 65 | 19 → 31 | 5d6+5, 22.5 | 5.3 (8.5) | 26 / 295 |
| Fireball (3, Area) | 60 | 70 | 19 → 32 | 6d6+18, 39.0 | 6.9 (12) | 10 / 656 |
| Flame (3, Area) | 10 | 45 | 19 → 27 | 4d6+4, 18.0 | 2.1 (2.8) | 26 / 93 |
| Flame (3, Area) | 30 | 65 | 19 → 31 | 5d6+5, 22.5 | 5.3 (8.6) | 26 / 295 |
| Flame (3, Area) | 60 | 70 | 19 → 32 | 6d6+18, 39.0 | 6.9 (12) | 10 / 656 |
| Lightning bolt (3, Area) | 10 | 45 | 19 → 27 | 3d6+3, 13.5 | 2.8 (4.1) | 20 / 93 |
| Lightning bolt (3, Area) | 30 | 65 | 19 → 31 | 5d6+5, 22.5 | 5.3 (8.5) | 20 / 295 |
| Lightning bolt (3, Area) | 60 | 70 | 19 → 32 | 6d6+18, 39.0 | 6.9 (12) | 8 / 656 |

**Priest**

| Spell (tier, resolution) | L | lift % | cap → lifted | dice | casts to kill | mana / pool |
|---|---|---|---|---|---|---|
| none: no damage spell it knows or of a school it has an affinity with |||||||

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
