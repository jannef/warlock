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
#965; `docs/design/talent-ranks.md`), and again for the slower weapon marks (#1095: the scale
capped at 100 %, steeper curves, and weapon ranks trained by the fights instead of set by level,
"Weapon pace"), so prose under a table that quotes a figure from an earlier PR's run may be a
little off its cells. The tables are a fresh run on `main` after #1093 to #1104 (the Shield blocking
skill removed, sure hit at 22+, the gatekeeper, doubled spell dice, the dragons' breath, monster
to-hit L/4, starting scrolls), and again after the dread slices D0 to D8 and D11 and the
variance pass (#1134: "Spread").

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
`balance_probe/schools.rs` (the casters' school pace), `balance_probe/spread.rs` (the spread of rounds to kill and to die), and `balance_probe/tables.rs` (the tables).

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
- **Players**: a human male warrior (long sword, sword and board), thief (dagger,
  twin blades), wizard (staff; evocation), and priest (mace, sword and board;
  restoration), six seeds each, and a witch (dagger; necromancy) measured for the spell tables
  alone. Each is made by `World::start_run` with the kit and levelled one level at a time with
  `xp:` and `levelup:auto`, which takes the first talent it may, not one chosen for a build.
  Weapon skill ranks come from use (#1095, "Weapon pace"): a trained career's fights train the
  main hand's weapon skill and style blow by blow through `World::practise` (the real `[marks]`,
  curves, and talents), and an untrained career takes its trained twin's ranks. The skills no
  fight trains (a shield, the berserker's great blades, the wizard's staff and the witch's dagger,
  which fight with spells) are set by level at the trained warrior's measured pace: rank 2 at
  level 1, 4 at 5, 5 at 7, 6 at 10, 7 at 15, 8 at 23, 9 at 32, and 10 from 42; the wizard's staff
  and the witch's dagger two ranks lower. A school and a caster style are not set by level: they
  are trained by casting ("School pace").
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
| 1 | +3/8.2/14/3 | +3/8.2/14/3 | +1/3.0/13/1 | +1/3.0/13/1 | +0/3.0/12/0 | +1/5.8/11/0 | +1/5.8/11/0 |
| 5 | +4/8.8/16/4 | +5/10.6/19/4 | +3/4.2/15/1 | +3/5.2/17/2 | +0/3.0/13/0 | +2/7.0/14/2 | +3/7.9/16/2 |
| 10 | +6/11.3/20/5 | +8/12.9/23/6 | +6/7.7/20/1 | +8/9.0/22/2 | +2/4.2/15/0 | +6/10.0/15/4 | +7/11.9/18/5 |
| 15 | +7/12.2/20/9 | +12/15.8/24/10 | +9/9.0/22/2 | +12/11.5/25/4 | +2/4.2/15/1 | +8/12.5/18/5 | +11/17.2/23/5 |
| 20 | +9/14.2/22/10 | +14/20.4/27/12 | +12/10.7/24/2 | +14/11.8/28/4 | +2/4.2/16/1 | +9/13.2/17/7 | +12/15.6/23/8 |
| 30 | +11/16.3/26/12 | +20/23.2/35/18 | +14/12.8/25/2 | +20/20.0/31/8 | +3/5.3/18/1 | +11/15.5/19/8 | +19/21.4/26/15 |
| 40 | +14/19.3/28/12 | +22/26.3/37/16 | +16/14.8/26/2 | +24/19.2/34/5 | +4/7.3/20/1 | +12/18.8/20/8 | +22/25.9/30/12 |
| 50 | +15/22.2/30/12 | +20/28.1/40/17 | +17/16.3/26/2 | +23/23.2/33/7 | +4/7.5/21/1 | +14/21.2/21/8 | +20/29.1/28/13 |
| 60 | +16/22.8/30/12 | +22/29.2/39/15 | +17/16.8/26/2 | +25/22.2/34/7 | +5/7.8/22/1 | +14/21.8/22/8 | +21/26.5/31/13 |

Stats, hit points, and mana (plain gear; trained → untrained). Scores St/Dx/To/In/Wi/Pr, the seeds' mean. Spellcraft (learning): the trained Spellcraft and the learning it gives; the auto-rolls spend every level-up roll on the first trained skill (Concentration for both), so it stays where it starts.

| L | Warrior | Thief | Wizard | Priest | HP W/T/Wz/P | MP Wz/P | Spellcraft (learning) Wz/P | Blows a round W/T |
|---|---|---|---|---|---|---|---|---|
| 1 | 15/11/12/10/12/10 | 11/12/10/11/10/10 | 10/10/10/14/12/10 | 11/9/11/11/15/11 | 23→23 / 19→19 / 13→13 / 18→18 | 16/13 | 20 (4)/22 (2) | 0.99/0.99 |
| 5 | 15/11/12/10/12/10 | 12/13/11/12/10/10 | 10/10/10/15/13/10 | 11/9/12/12/15/11 | 37→37 / 27→26 / 17→17 / 27→26 | 51/40 | 20 (4)/22 (2) | 0.88/0.96 |
| 10 | 18/12/14/12/12/10 | 16/16/14/13/10/10 | 10/11/12/16/16/10 | 16/10/15/13/16/11 | 61→55 / 46→37 / 25→22 / 51→38 | 99/77 | 20 (4)/22 (2) | 0.89/1.04 |
| 15 | 19/15/16/12/12/10 | 19/18/17/14/11/10 | 10/12/15/17/19/10 | 18/11/18/14/18/11 | 91→73 / 74→47 / 36→28 / 80→48 | 151/117 | 20 (4)/22 (2) | 0.91/1.10 |
| 20 | 20/17/18/13/13/10 | 20/20/19/14/12/11 | 10/12/17/19/20/10 | 20/13/20/14/19/11 | 126→92 / 106→58 / 54→33 / 114→58 | 206/160 | 20 (4)/22 (2) | 0.92/1.17 |
| 30 | 23/19/20/15/14/12 | 23/21/20/16/13/11 | 10/14/19/20/22/11 | 22/16/22/16/20/12 | 201→128 / 172→79 / 99→43 / 186→79 | 330/249 | 20 (4)/22 (2) | 0.94/1.25 |
| 40 | 25/20/21/16/16/12 | 24/23/21/17/15/12 | 11/16/20/21/24/12 | 25/18/23/17/21/13 | 283→164 / 246→100 / 149→53 / 270→101 | 458/342 | 20 (4)/22 (2) | 0.95/1.49 |
| 50 | 27/22/22/17/17/13 | 26/24/22/18/16/13 | 11/17/22/22/25/13 | 26/19/25/18/22/14 | 370→200 / 326→120 / 205→63 / 361→122 | 590/441 | 20 (4)/22 (2) | 0.95/1.49 |
| 60 | 28/22/23/18/18/14 | 27/24/24/18/17/14 | 12/18/22/23/26/14 | 27/19/26/18/23/15 | 374→203 / 330→122 / 209→64 / 366→124 | 731/542 | 20 (4)/22 (2) | 0.95/1.49 |

The Defence of the warrior and the thief in plain gear by category (trained → untrained), the seeds' mean.

| Class | L | Dexterity | Armour | Shield | Parry | Natural | Dodge | Deflection | Wall | total |
|---|---|---|---|---|---|---|---|---|---|---|
| warrior | 10 | 1→0 | 2→2 | 4→4 | 2→2 | 0→0 | 1→1 | 0→0 | 0→0 | 20→19 |
| warrior | 30 | 2→0 | 1→1 | 8→8 | 4→4 | 0→0 | 1→1 | 0→0 | 0→0 | 26→24 |
| warrior | 60 | 2→0 | 1→1 | 10→10 | 6→6 | 0→0 | 1→1 | 0→0 | 0→0 | 30→28 |
| thief | 10 | 2→1 | 2→2 | 0→0 | 4→3 | 0→0 | 2→2 | 0→0 | 0→0 | 20→17 |
| thief | 30 | 5→1 | 2→2 | 0→0 | 6→4 | 0→0 | 2→2 | 0→0 | 0→0 | 25→19 |
| thief | 60 | 5→1 | 2→2 | 0→0 | 8→6 | 0→0 | 2→2 | 0→0 | 0→0 | 26→21 |

## Monsters

Within two levels: the median (range) bare → the mean with gear. Damage is a hit's mean before soak.

| L (±2) | n | HP | to-hit | dmg/blow | Dv | Pv | blows a round |
|---|---|---|---|---|---|---|---|
| 1 | 56 | 11 | 0 (-5–3) → -0.5 | 2 (1–8) → 2.4 | 12 (7–14) → 11.3 | 0 (0–3) → 0.2 | 1.00 |
| 5 | 61 | 19 | 1 (-3–6) → 1.2 | 4 (1–13) → 4.3 | 12 (7–21) → 12.0 | 0 (0–2) → 0.4 | 1.00 |
| 10 | 55 | 39 | 3 (-1–10) → 4.0 | 7 (1–16) → 8.5 | 11 (7–19) → 12.5 | 0 (0–5) → 0.7 | 1.00 |
| 15 | 46 | 56 | 6 (2–11) → 5.9 | 13 (3–20) → 12.2 | 12 (8–19) → 13.3 | 0 (0–7) → 1.0 | 1.00 |
| 20 | 49 | 76 | 7 (2–13) → 7.0 | 16 (4–21) → 14.4 | 13 (11–20) → 14.3 | 1 (0–6) → 1.7 | 1.00 |
| 30 | 35 | 119 | 10 (6–19) → 10.9 | 22 (11–27) → 20.8 | 15 (12–25) → 15.6 | 2 (0–6) → 2.7 | 1.01 |
| 40 | 28 | 144 | 16 (8–21) → 13.3 | 26 (11–32) → 23.3 | 17 (13–22) → 17.7 | 3 (0–11) → 3.4 | 1.06 |
| 50 | 24 | 267 | 16 (10–31) → 16.6 | 32 (16–40) → 29.5 | 19 (12–25) → 19.2 | 5 (0–13) → 4.3 | 1.05 |
| 60 | 18 | 226 | 21 (12–32) → 19.9 | 32 (18–43) → 30.3 | 22 (17–29) → 21.4 | 6 (0–16) → 6.3 | 1.27 |
| 70 | 8 | 192 | 24 (13–34) → 24.6 | 34 (22–48) → 32.9 | 27 (17–30) → 24.2 | 4 (0–11) → 5.0 | 1.25 |

## Fights

Each cell: P(player hits) / P(monster hits) % · damage a round player : monster · rounds to kill / rounds to die; the median over the monsters within two levels (trained stats, monsters with gear).

| L | Warrior kit | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain (melee) | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|---|
| 1 | 81/41 · 7.4:0.2 · 1.6 / 492 | 81/41 · 7.4:0.2 · 1.6 / 505 | 81/41 · 7.4:0.2 · 1.6 / 508 | 66/54 · 2.7:0.6 · 4.9 / 56 | 66/54 · 2.7:0.6 · 5.0 / 55 | 65/61 · 2.1:1.1 · 5.7 / 14 | 71/66 · 4.4:1.2 · 2.8 / 18 | 71/66 · 4.4:1.2 · 2.7 / 18 |
| 5 | 86/48 · 8.1:1.3 · 2.7 / 46 | 86/41 · 7.1:0.9 · 3.1 / 63 | 89/26 · 8.9:0.5 · 2.5 / 134 | 79/48 · 4.0:2.0 · 6.8 / 16 | 80/35 · 5.1:1.2 · 5.0 / 43 | 65/64 · 1.8:3.1 · 15 / 5.8 | 78/55 · 4.4:2.0 · 5.1 / 17 | 82/40 · 5.4:1.3 · 4.0 / 35 |
| 10 | 91/39 · 11.4:2.1 · 3.9 / 34 | 91/32 · 9.8:1.5 · 4.4 / 53 | 94/18 · 11.6:0.7 · 3.7 / 192 | 89/33 · 11.0:2.2 · 5.0 / 25 | 92/24 · 13.7:1.6 · 4.0 / 69 | 66/68 · 2.6:5.3 · 18 / 4.9 | 88/65 · 7.8:3.4 · 5.5 / 16 | 94/45 · 10.5:2.1 · 4.1 / 36 |
| 15 | 92/44 · 12.0:4.4 · 5.2 / 23 | 92/44 · 10.4:2.9 · 6.0 / 32 | 99/26 · 15.6:1.8 · 4.3 / 257 | 96/35 · 14.5:3.6 · 5.7 / 30 | 98/16 · 19.3:1.5 · 4.6 / 80 | 64/77 · 2.2:8.8 · 29 / 4.2 | 94/60 · 10.4:5.6 · 6.2 / 15 | 98/30 · 16.0:2.8 · 4.2 / 114 |
| 20 | 97/29 · 15.5:3.2 · 5.4 / 45 | 97/36 · 13.2:2.7 · 6.4 / 61 | 99/11 · 22.8:0.7 · 3.6 / 406 | 99/27 · 19.6:3.4 · 5.6 / 69 | 99/12 · 23.1:1.4 · 4.5 / 156 | 62/79 · 1.7:10.7 · 51 / 5.2 | 96/70 · 10.2:6.7 · 7.8 / 19 | 98/35 · 14.2:3.0 · 5.7 / 110 |
| 30 | 97/21 · 17.8:4.6 · 8.9 / 48 | 97/29 · 15.8:4.9 · 10 / 42 | 99/4 · 27.1:0.6 · 6.3 / 482 | 99/38 · 24.9:9.1 · 7.7 / 28 | 99/10 · 43.4:1.5 · 3.9 / 172 | 57/83 · 1.6:18.0 · 74 / 5.5 | 96/77 · 11.4:15.3 · 12 / 12 | 99/28 · 23.2:4.1 · 6.6 / 75 |
| 40 | 97/25 · 23.2:4.6 · 9.0 / 71 | 97/38 · 20.6:5.1 · 10 / 62 | 99/4 · 36.9:0.7 · 5.9 / 488 | 99/48 · 34.3:9.9 · 6.8 / 36 | 99/12 · 56.6:2.0 · 4.3 / 239 | 43/88 · 2.3:17.5 · 82 / 8.6 | 93/87 · 14.8:14.4 · 13 / 19 | 99/27 · 27.5:3.4 · 6.3 / 144 |
| 50 | 98/36 · 27.1:11.6 · 10 / 34 | 98/55 · 23.6:14.7 · 12 / 26 | 99/8 · 34.6:1.8 · 8.1 / 366 | 99/73 · 35.6:22.8 · 8.4 / 15 | 99/39 · 54.0:11.9 · 5.5 / 136 | 42/97 · 1.7:28.3 · 168 / 7.3 | 95/97 · 16.8:26.7 · 16 / 14 | 99/61 · 31.9:15.3 · 8.6 / 34 |
| 60 | 93/35 · 24.1:9.9 · 11 / 52 | 93/56 · 20.6:11.5 · 12 / 37 | 99/9 · 36.1:1.9 · 7.7 / 412 | 95/74 · 27.4:22.0 · 9.4 / 18 | 99/38 · 42.2:10.3 · 6.1 / 138 | 30/96 · 1.3:29.8 · 343 / 7.2 | 89/95 · 14.9:24.2 · 16 / 16 | 98/54 · 25.2:10.8 · 9.4 / 68 |

Out of depth (+3: the monsters one to five levels above), and untrained stats, for the warrior in plain gear.

| L | Warrior plain +3 | Warrior magic +3 | Warrior plain, untrained | Warrior magic, untrained |
|---|---|---|---|---|
| 1 | 81/50 · 7.1:0.9 · 2.8 / 30 | 81/50 · 7.1:0.9 · 2.8 / 32 | 81/41 · 7.4:0.2 · 1.6 / 516 | 81/41 · 7.5:0.2 · 1.6 / 493 |
| 5 | 86/50 · 6.9:2.0 · 4.4 / 21 | 88/33 · 8.9:1.3 · 3.6 / 41 | 86/41 · 7.1:0.9 · 3.2 / 67 | 89/26 · 9.0:0.5 · 2.6 / 149 |
| 10 | 91/42 · 9.5:3.6 · 6.3 / 18 | 94/25 · 10.8:1.9 · 5.5 / 81 | 85/38 · 8.4:1.7 · 5.3 / 37 | 89/21 · 9.5:0.9 · 4.4 / 143 |
| 15 | 91/45 · 10.4:3.3 · 6.9 / 30 | 99/28 · 15.6:2.0 · 4.8 / 172 | 84/50 · 7.8:3.4 · 9.0 / 23 | 97/29 · 13.2:2.0 · 5.2 / 182 |
| 20 | 93/43 · 13.2:5.2 · 9.6 / 25 | 98/14 · 22.2:1.6 · 5.8 / 144 | 88/38 · 8.6:2.9 · 10 / 45 | 97/10 · 19.3:0.6 · 4.4 / 310 |
| 30 | 97/38 · 14.5:5.9 · 11 / 39 | 99/6 · 26.6:0.8 · 6.7 / 393 | 84/40 · 8.9:6.8 · 18 / 19 | 99/5 · 21.5:0.7 · 8.2 / 319 |
| 40 | 97/38 · 19.7:8.2 · 11 / 35 | 99/5 · 37.1:0.8 · 6.4 / 418 | 77/48 · 10.6:6.2 · 19 / 29 | 96/4 · 26.6:0.6 · 8.4 / 357 |
| 50 | 96/48 · 20.8:9.6 · 13 / 42 | 99/7 · 33.3:1.2 · 8.5 / 496 | 76/67 · 11.3:17.0 · 28 / 12 | 92/11 · 23.5:2.4 · 13 / 179 |
| 60 | 92/63 · 17.1:21.4 · 15 / 17 | 99/13 · 26.7:4.1 · 9.4 / 165 | 63/71 · 6.9:15.1 · 39 / 14 | 88/15 · 20.2:3.1 · 12 / 176 |

## Spread

Rounds to kill and to die over the monsters within two levels, trained stats in plain gear: the median (fastest tenth–slowest tenth) ×the tenths over the median. The target: ×0.5 and ×2.

| L | n | Warrior kills | Warrior dies | Thief dies | Wizard dies | Priest dies |
|---|---|---|---|---|---|---|
| 1 | 56 | 1.6 (1.2–2.6) ×0.77/×1.6 | 511 (25–>999) ×0.05/×4.5 | 71 (8.6–>999) ×0.12/×26.2 | 14 (4.6–487) ×0.33/×35.3 | 18 (6.1–657) ×0.35/×37.4 |
| 5 | 61 | 3.1 (2.0–5.6) ×0.63/×1.8 | 65 (18–>999) ×0.28/×42.1 | 17 (6.4–563) ×0.39/×33.8 | 5.8 (2.9–42) ×0.49/×7.2 | 18 (6.5–>999) ×0.36/×55.0 |
| 10 | 55 | 4.4 (2.6–8.9) ×0.58/×2.0 | 48 (14–405) ×0.28/×8.5 | 27 (8.8–69) ×0.33/×2.6 | 5.0 (2.5–12) ×0.50/×2.5 | 16 (6.2–78) ×0.39/×4.8 |
| 15 | 46 | 6.1 (3.4–10) ×0.56/×1.7 | 31 (15–112) ×0.47/×3.6 | 28 (13–60) ×0.46/×2.2 | 4.3 (2.7–6.0) ×0.63/×1.4 | 16 (8.7–29) ×0.55/×1.8 |
| 20 | 49 | 6.4 (3.0–12) ×0.47/×1.8 | 61 (12–289) ×0.19/×4.8 | 69 (12–156) ×0.17/×2.2 | 5.3 (2.3–8.3) ×0.44/×1.6 | 20 (6.7–48) ×0.34/×2.5 |
| 30 | 35 | 10 (3.7–13) ×0.36/×1.3 | 41 (12–434) ×0.28/×10.5 | 29 (7.0–98) ×0.24/×3.4 | 5.5 (2.9–10) ×0.53/×1.9 | 12 (7.0–38) ×0.58/×3.1 |
| 40 | 28 | 10 (3.7–13) ×0.36/×1.3 | 62 (14–213) ×0.22/×3.4 | 39 (9.7–117) ×0.25/×3.0 | 8.8 (4.5–11) ×0.51/×1.2 | 19 (9.7–31) ×0.51/×1.6 |
| 50 | 24 | 12 (6.6–16) ×0.54/×1.3 | 26 (13–182) ×0.50/×6.9 | 15 (11–73) ×0.73/×5.0 | 7.2 (5.4–12) ×0.74/×1.6 | 14 (11–27) ×0.78/×1.9 |
| 60 | 18 | 12 (5.1–22) ×0.42/×1.8 | 40 (10–88) ×0.25/×2.2 | 18 (6.9–36) ×0.39/×2.0 | 7.1 (4.2–8.9) ×0.58/×1.2 | 16 (8.7–20) ×0.56/×1.3 |

What it says (#1134, the variance pass; the dread design §7.3): before the pass (main after
D0 to D8 and D11) the warrior's rounds to kill ran ×0.56 to ×0.78 of the median at the fastest
tenth and ×1.2 to ×1.6 at the slowest, every level: a row of the same fight. After it (to-hit
by role, threat 19 for the strikers and ×3 on 24 for the brutes, hit points 0.5 to 2.0 by role
with every brute in the top of its ±35 %, the speed rule) the fastest tenth is ×0.36 to ×0.63
from level 5 and the slowest ×1.7 to ×2.0 to level 20 and at 60; at 30 to 50 the slow tail
stays at ×1.3, where the windows are small (24 to 35 types) and hold few brutes (the giants
and most of the deep brutes there are armed or uniques, outside the role factors). Rounds to
die keep their wide spread (the thief's, the wizard's, and the priest's columns: ×0.2 to ×0.5
fast, ×1.3 to ×5 slow from level 10); a dread brute is a death, not a wall. A further pass
for 30 to 50 is a content question (more unarmed brutes there), not an offset.

## Dragons

Each dragon (#1099) against the players of the probe's levels around its own, in plain gear, trained → untrained (the seeds' mean). **HP** the player's; **breath** its mean after the player's resistance, and that over their hit points; **melee** the claws and bite of one action, misses included; **a round** the dragon's damage an action over a cooldown's cycle (one breath, then a melee action for each degree of its cooldown); **die** the dragon's actions until the player falls, breathing first; **kill** the player's rounds to kill it.

| Dragon | L | Class | HP | breath (of HP) | melee | a round | die | kill |
|---|---|---|---|---|---|---|---|---|
| fire drake | 15 | warrior | 91→73 | 43 (48%→59%) | 25.2→26.9 | 27.8→29.3 | 2.9→2.1 | 13→19 |
| fire drake | 15 | thief | 74→47 | 43 (58%→92%) | 31.4→43.7 | 33.1→43.6 | 2.0→1.1 | 10→34 |
| fire drake | 15 | wizard | 36→28 | 43 (119%→157%) | 50.2→50.9 | 49.2→49.8 | 0.8→0.6 | 106→129 |
| fire drake | 15 | priest | 80→48 | 43 (54%→91%) | 36.4→39.0 | 37.3→39.6 | 2.0→1.1 | 14→31 |
| fire drake | 20 | warrior | 126→92 | 43 (34%→47%) | 19.8→20.1 | 23.1→23.4 | 5.2→3.4 | 9.0→15 |
| fire drake | 20 | thief | 106→58 | 43 (41%→74%) | 22.6→41.4 | 25.6→41.7 | 4.4→1.4 | 7.2→20 |
| fire drake | 20 | wizard | 54→33 | 43 (80%→132%) | 49.2→51.4 | 48.3→50.2 | 1.2→0.8 | 97→129 |
| fire drake | 20 | priest | 114→58 | 43 (38%→74%) | 33.0→36.0 | 34.5→37.0 | 3.1→1.4 | 13→30 |
| Hrīmwyrm the White | 30 | warrior | 201→128 | 57 (28%→45%) | 25.3→31.2 | 29.8→34.9 | 6.6→3.3 | 13→24 |
| Hrīmwyrm the White | 30 | thief | 172→79 | 57 (33%→72%) | 40.4→61.3 | 42.8→60.7 | 4.1→1.4 | 10→29 |
| Hrīmwyrm the White | 30 | wizard | 99→43 | 50 (51%→117%) | 67.4→70.6 | 64.9→67.7 | 1.7→0.9 | 185→234 |
| Hrīmwyrm the White | 30 | priest | 186→79 | 57 (31%→72%) | 50.7→53.4 | 51.6→53.9 | 3.6→1.4 | 18→50 |
| Hrīmwyrm the White | 40 | warrior | 283→164 | 57 (20%→35%) | 20.1→23.6 | 25.4→28.4 | 10→5.5 | 9.2→17 |
| Hrīmwyrm the White | 40 | thief | 246→100 | 57 (23%→57%) | 34.3→58.9 | 37.6→58.7 | 7.0→1.7 | 6.7→18 |
| Hrīmwyrm the White | 40 | wizard | 149→53 | 50 (34%→94%) | 62.6→70.4 | 60.8→67.5 | 2.6→1.0 | 85→155 |
| Hrīmwyrm the White | 40 | priest | 270→101 | 57 (21%→57%) | 49.1→50.7 | 50.3→51.6 | 5.3→1.9 | 13→41 |
| Nīðdraca the Hate-Wyrm | 60 | warrior | 374→203 | 95 (25%→47%) | 142.1→142.4 | 134.2→134.5 | 3.0→1.8 | 29→90 |
| Nīðdraca the Hate-Wyrm | 60 | thief | 330→122 | 95 (29%→78%) | 165.1→165.3 | 153.4→153.5 | 2.4→1.2 | 36→504 |
| Nīðdraca the Hate-Wyrm | 60 | wizard | 209→64 | 95 (45%→149%) | 169.2→167.9 | 156.8→155.8 | 1.7→0.7 | >999→>999 |
| Nīðdraca the Hate-Wyrm | 60 | priest | 366→124 | 95 (26%→77%) | 149.9→150.8 | 140.7→141.5 | 2.8→1.2 | 65→685 |

## Casters

Each caster (#1102) against the trained players of the first probe level from its own, in plain gear (the seeds' mean). **Spell** its first damage spell and its roll at its stats; **casts** how many of it its full pool pays for; **a cast** its mean damage (an `attack` spell's times its hit chance); **a round** that times its casts a round, against **melee** its blows a round once the mana is spent; **die** the player's rounds until they fall, the spells first, then its melee.

| Caster | L | Spell (roll) | casts | Player | HP | a cast | a round | melee | die |
|---|---|---|---|---|---|---|---|---|---|
| arcanaloth | 46 | chain lightning (12d6+3) | 8 | warrior 50 | 370 | 45.0 | 49.5 | 5.6 | 9.9 |
| arcanaloth | 46 | chain lightning (12d6+3) | 8 | wizard 50 | 205 | 45.0 | 49.5 | 25.1 | 4.1 |
| archlich | 54 | sunfire (16d6+4) | 3 | warrior 60 | 374 | 60.0 | 42.0 | 9.4 | 25 |
| archlich | 54 | sunfire (16d6+4) | 3 | wizard 60 | 209 | 60.0 | 42.0 | 15.5 | 6.2 |
| ash wraith | 15 | scorching ray (6d6) | 5 | warrior 15 | 91 | 11.3 | 12.5 | 2.8 | 18 |
| ash wraith | 15 | scorching ray (6d6) | 5 | wizard 15 | 36 | 18.4 | 20.3 | 5.5 | 1.8 |
| barrow king | 42 | grave spear (10d6+2) | 5 | warrior 50 | 370 | 37.0 | 40.7 | 10.3 | 23 |
| barrow king | 42 | grave spear (10d6+2) | 5 | wizard 50 | 205 | 37.0 | 40.7 | 18.7 | 5.7 |
| bheur hag | 42 | rime (8d6+2) | 7 | warrior 50 | 370 | 30.0 | 33.0 | 5.4 | 37 |
| bheur hag | 42 | rime (8d6+2) | 7 | wizard 50 | 205 | 30.0 | 33.0 | 17.4 | 6.4 |
| dark wizard | 24 | fireball (8d6) | 4 | warrior 30 | 201 | 28.0 | 28.0 | 0.0 | >999 |
| dark wizard | 24 | fireball (8d6) | 4 | wizard 30 | 99 | 28.0 | 28.0 | 2.7 | 3.8 |
| drow archmage | 40 | chain lightning (12d6+3) | 8 | warrior 40 | 283 | 45.0 | 49.5 | 5.9 | 5.7 |
| drow archmage | 40 | chain lightning (12d6+3) | 8 | wizard 40 | 149 | 45.0 | 49.5 | 22.8 | 3.0 |
| drow high priestess | 44 | leech (8d6+2) | 13 | warrior 50 | 370 | 27.2 | 29.9 | 6.5 | 15 |
| drow high priestess | 44 | leech (8d6+2) | 13 | wizard 50 | 205 | 29.8 | 32.8 | 26.0 | 6.3 |
| drow mage | 22 | lightning bolt (10d6) | 4 | warrior 30 | 201 | 35.0 | 35.0 | 0.6 | 113 |
| drow mage | 22 | lightning bolt (10d6) | 4 | wizard 30 | 99 | 35.0 | 35.0 | 8.7 | 2.8 |
| drow matron | 50 | chain lightning (12d6+3) | 8 | warrior 50 | 370 | 45.0 | 49.5 | 8.9 | 8.9 |
| drow matron | 50 | chain lightning (12d6+3) | 8 | wizard 50 | 205 | 45.0 | 49.5 | 28.4 | 4.1 |
| drow priestess | 24 | leech (8d6) | 9 | warrior 30 | 201 | 18.2 | 18.2 | 1.4 | 38 |
| drow priestess | 24 | leech (8d6) | 9 | wizard 30 | 99 | 27.4 | 27.4 | 12.5 | 3.6 |
| dryad | 19 | acid arrow (6d6) | 10 | warrior 20 | 126 | 11.6 | 11.6 | 1.1 | 26 |
| dryad | 19 | acid arrow (6d6) | 10 | wizard 20 | 54 | 18.9 | 18.9 | 8.1 | 2.9 |
| duergar mindbreaker | 29 | flay (2d8) | 9 | warrior 30 | 201 | 9.0 | 9.0 | 5.0 | 34 |
| duergar mindbreaker | 29 | flay (2d8) | 9 | wizard 30 | 99 | 9.0 | 9.0 | 16.2 | 10 |
| duergar priest | 19 | flay (2d8) | 6 | warrior 20 | 126 | 9.0 | 9.0 | 0.0 | >999 |
| duergar priest | 19 | flay (2d8) | 6 | wizard 20 | 54 | 9.0 | 9.0 | 2.5 | 6.8 |
| ghost lord | 53 | grave spear (10d6+2) | 6 | warrior 60 | 374 | 37.0 | 51.8 | 17.0 | 13 |
| ghost lord | 53 | grave spear (10d6+2) | 6 | wizard 60 | 209 | 37.0 | 51.8 | 28.9 | 4.1 |
| giant shaman | 36 | rime (8d6) | 7 | warrior 40 | 283 | 28.0 | 28.0 | 6.0 | 22 |
| giant shaman | 36 | rime (8d6) | 7 | wizard 40 | 149 | 28.0 | 28.0 | 20.2 | 5.3 |
| goblin wizard | 8 | burning hands (2d6) | 11 | warrior 10 | 61 | 7.0 | 7.0 | 0.0 | 8.7 |
| goblin wizard | 8 | burning hands (2d6) | 11 | wizard 10 | 25 | 7.0 | 7.0 | 0.7 | 3.6 |
| green hag | 18 | blight (6d6+3) | 3 | warrior 20 | 126 | 24.0 | 24.0 | 0.4 | 212 |
| green hag | 18 | blight (6d6+3) | 3 | wizard 20 | 54 | 24.0 | 24.0 | 5.3 | 2.2 |
| hobgoblin warcaster | 14 | frost (6d6) | 3 | warrior 15 | 91 | 21.0 | 21.0 | 0.0 | >999 |
| hobgoblin warcaster | 14 | frost (6d6) | 3 | wizard 15 | 36 | 21.0 | 21.0 | 1.8 | 1.7 |
| kobold shaman | 4 | spark (2d4) | 5 | warrior 5 | 37 | 5.0 | 4.0 | 0.0 | >999 |
| kobold shaman | 4 | spark (2d4) | 5 | wizard 5 | 17 | 5.0 | 4.0 | 0.2 | 4.3 |
| lich | 30 | chain lightning (12d6+3) | 7 | warrior 30 | 201 | 45.0 | 31.5 | 3.4 | 6.4 |
| lich | 30 | chain lightning (12d6+3) | 7 | wizard 30 | 99 | 45.0 | 31.5 | 9.0 | 3.1 |
| mummy king | 56 | flame (10d6+2) | 6 | warrior 60 | 374 | 37.0 | 42.6 | 12.9 | 17 |
| mummy king | 56 | flame (10d6+2) | 6 | wizard 60 | 209 | 37.0 | 42.6 | 33.5 | 4.9 |
| mummy lord | 36 | blight (6d6+3) | 6 | warrior 40 | 283 | 24.0 | 25.2 | 2.6 | 60 |
| mummy lord | 36 | blight (6d6+3) | 6 | wizard 40 | 149 | 24.0 | 25.2 | 16.2 | 6.1 |
| mummy priest | 20 | cause wounds (4d8+2) | 27 | warrior 20 | 126 | 12.5 | 12.5 | 4.0 | 10 |
| mummy priest | 20 | cause wounds (4d8+2) | 27 | wizard 20 | 54 | 18.7 | 18.7 | 8.7 | 2.9 |
| necromancer | 18 | leech (8d6) | 7 | warrior 20 | 126 | 15.5 | 15.5 | 0.0 | >999 |
| necromancer | 18 | leech (8d6) | 7 | wizard 20 | 54 | 25.2 | 25.2 | 1.8 | 2.1 |
| night hag | 33 | leech (8d6+2) | 10 | warrior 40 | 283 | 21.5 | 22.5 | 1.9 | 47 |
| night hag | 33 | leech (8d6+2) | 10 | wizard 40 | 149 | 29.7 | 31.1 | 14.3 | 4.8 |
| nymph | 22 | magic missile (6d4+1) | 15 | warrior 30 | 201 | 16.0 | 22.4 | 1.6 | 9.0 |
| nymph | 22 | magic missile (6d4+1) | 15 | wizard 30 | 99 | 16.0 | 22.4 | 14.3 | 4.4 |
| ogre mage | 18 | rime (8d6) | 4 | warrior 20 | 126 | 28.0 | 28.0 | 2.5 | 9.7 |
| ogre mage | 18 | rime (8d6) | 4 | wizard 20 | 54 | 28.0 | 28.0 | 11.5 | 1.9 |
| orc high shaman | 24 | fireball (8d6) | 3 | warrior 30 | 201 | 28.0 | 28.0 | 0.0 | >999 |
| orc high shaman | 24 | fireball (8d6) | 3 | wizard 30 | 99 | 28.0 | 28.0 | 2.8 | 8.4 |
| orc shaman | 10 | burning hands (2d6) | 4 | warrior 10 | 61 | 7.0 | 7.0 | 0.0 | >999 |
| orc shaman | 10 | burning hands (2d6) | 4 | wizard 10 | 25 | 7.0 | 7.0 | 1.3 | 3.6 |
| pixie | 4 | spark (2d4) | 5 | warrior 5 | 37 | 5.0 | 7.0 | 0.3 | 42 |
| pixie | 4 | spark (2d4) | 5 | wizard 5 | 17 | 5.0 | 7.0 | 1.9 | 2.5 |
| rakshasa | 52 | flame (10d6+2) | 6 | warrior 60 | 374 | 37.0 | 51.8 | 13.0 | 16 |
| rakshasa | 52 | flame (10d6+2) | 6 | wizard 60 | 209 | 37.0 | 51.8 | 37.9 | 4.0 |
| troll shaman | 21 | rime (8d6) | 5 | warrior 30 | 201 | 28.0 | 28.0 | 1.5 | 45 |
| troll shaman | 21 | rime (8d6) | 5 | wizard 30 | 99 | 28.0 | 28.0 | 11.9 | 3.5 |
| will-o'-wisp | 7 | spark (2d4) | 8 | warrior 10 | 61 | 5.0 | 7.0 | 0.1 | 310 |
| will-o'-wisp | 7 | spark (2d4) | 8 | wizard 10 | 25 | 5.0 | 7.0 | 0.3 | 3.6 |
| wormlord | 47 | leech (8d6+2) | 13 | warrior 50 | 370 | 27.2 | 19.0 | 3.2 | 25 |
| wormlord | 47 | leech (8d6+2) | 13 | wizard 50 | 205 | 29.8 | 20.9 | 16.1 | 9.8 |
| barrow wight | 6 | grave breath (4d4) | 5 | warrior 10 | 61 | 10.0 | 10.0 | 0.4 | 40 |
| barrow wight | 6 | grave breath (4d4) | 5 | wizard 10 | 25 | 10.0 | 10.0 | 1.2 | 2.5 |
| bog hag | 11 | acid arrow (6d6) | 6 | warrior 15 | 91 | 8.1 | 8.1 | 0.3 | 160 |
| bog hag | 11 | acid arrow (6d6) | 6 | wizard 15 | 36 | 16.4 | 16.4 | 4.1 | 2.2 |
| cult priest | 9 | grave breath (4d4) | 6 | warrior 10 | 61 | 10.0 | 10.0 | 0.1 | 59 |
| cult priest | 9 | grave breath (4d4) | 6 | wizard 10 | 25 | 10.0 | 10.0 | 1.6 | 2.5 |

Read (#1102, first lists): the spells are the casters' real threat now. Up to level 10 a caster
spends a handful of sparks, flames, or grave breaths (5 to 10 a cast) and then fights with its
fists, which plate shrugs off; a trained warrior outlasts most of them (the goblin wizard's
eleven burning hands and the barrow wight's breath excepted), a wizard of its level falls in two
and a half to five rounds if they stand there. From level 18 one area spell (rime, fireball,
lightning bolt: 28 to 35) takes a third to a half of a wizard's hit points, and the deep
casters' chain lightning, grave spear, flame, and sunfire (37 to 60 a cast, 3 to 8 casts) kill
a plain-geared wizard of their level in two to five rounds and a warrior in four to twenty.
Silence, resistances, and closing in (a caster holds back an area that would catch a friend)
are the counters. Attack spells (acid arrow, leech, cause wounds, scorching ray) land on a
plate-clad warrior about half as often as on a wizard.

## Protection by body

Rounds for the plain warrior (trained) to kill the monsters of their level: as shipped, then with every monster's Protection set to §8.2's body line at its level.

| L | as shipped | none (0) | hide | bony | armoured |
|---|---|---|---|---|---|
| 10 | 4.5 | 3.8 | 3.8 | 4.4 | 5.0 |
| 15 | 6.1 | 5.2 | 5.1 | 5.8 | 7.0 |
| 20 | 6.2 | 5.1 | 5.4 | 6.0 | 6.6 |
| 30 | 10 | 6.6 | 7.2 | 8.4 | 9.5 |
| 40 | 10 | 6.6 | 7.4 | 8.1 | 9.5 |
| 50 | 12 | 10 | 11 | 13 | 15 |
| 60 | 12 | 6.8 | 7.8 | 8.3 | 10 |

## Spells

Each spell that rolls (harm or healing) of the schools the trained caster knows or has an affinity with, as `Actor::spell_roll` makes it in plain gear, per cast on one target: the rung of its ladder the stats reach (the seeds' range), the school's rank the pace below reached (and the rank of the caster style it is cast in from where the caster stands), the roll (seed 1's dice, the seeds' mean) and the same roll at rank 0 (the ladder alone), its mean on a target under a hex (Malison's bite; `–` without it), and its mana (the cost table's price with no rank's cut in brackets) over the pool.

**Wizard**

| Spell (level, res) | L | rung | school rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Burning hands (1, area) | 10 | 1–2 | 4–5 (Sniper 4–5) | 4d4 1s again, 11.2 (rank 0: 4d4, 9.0) | – | 2 (3) / 99 |
| Burning hands (1, area) | 20 | 2 | 6 (Sniper 6) | 5d5 1s again, 17.0 (rank 0: 4d4, 10.0) | – | 2 (3) / 206 |
| Burning hands (1, area) | 30 | 2 | 7 (Sniper 7–8) | 5d5 1s again, 17.2 (rank 0: 4d4, 10.0) | – | 2 (3) / 330 |
| Burning hands (1, area) | 60 | 3 | 9 (Sniper 9–10) | 6d6+2 1s and 2s again, 27.0 (rank 0: 4d4+2, 12.0) | – | 1 (3) / 731 |
| Grave breath (1, area) | 10 | 1–2 | 0 (Sniper 4–5) | 4d4, 8.5 (rank 0: 4d4, 8.5) | – | 3 (3) / 99 |
| Grave breath (1, area) | 20 | 2–3 | 0 (Sniper 6) | 4d4, 10.3 (rank 0: 4d4, 10.3) | – | 3 (3) / 206 |
| Grave breath (1, area) | 30 | 2–3 | 0 (Sniper 7–8) | 4d4+2, 11.3 (rank 0: 4d4+2, 11.3) | – | 3 (3) / 330 |
| Grave breath (1, area) | 60 | 3 | 0 (Sniper 9–10) | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 1 (3) / 731 |
| Ray of frost (1, attack) | 10 | 0–1 | 4–5 (Sniper 4–5) | 2d8 1s again, 10.1 (rank 0: 2d8, 8.3) | – | 1 (1) / 99 |
| Ray of frost (1, attack) | 20 | 1 | 6 (Sniper 6) | 3d9 1s again, 16.3 (rank 0: 2d8, 9.0) | – | 1 (1) / 206 |
| Ray of frost (1, attack) | 30 | 1 | 7 (Sniper 7–8) | 3d9 1s again, 16.5 (rank 0: 2d8, 9.0) | – | 1 (1) / 330 |
| Ray of frost (1, attack) | 60 | 1 | 9 (Sniper 9–10) | 4d10 1s and 2s again, 25.2 (rank 0: 2d8, 9.0) | – | 1 (1) / 731 |
| Spark (1, auto) | 10 | 0–1 | 4–5 (Sniper 4–5) | 2d6 1s again, 7.9 (rank 0: 2d6, 6.3) | – | 1 (1) / 99 |
| Spark (1, auto) | 20 | 1 | 6 (Sniper 6) | 3d7 1s again, 13.3 (rank 0: 2d6, 7.0) | – | 1 (1) / 206 |
| Spark (1, auto) | 30 | 1 | 7 (Sniper 7–8) | 3d7 1s again, 13.4 (rank 0: 2d6, 7.0) | – | 1 (1) / 330 |
| Spark (1, auto) | 60 | 1 | 9 (Sniper 9–10) | 4d8 1s and 2s again, 21.0 (rank 0: 2d6, 7.0) | – | 1 (1) / 731 |
| Magic missile (3, auto) | 10 | 1–2 | 4–5 (Sniper 4–5) | 6d4 1s again, 16.2 (rank 0: 6d4, 13.3) | – | 2 (3) / 99 |
| Magic missile (3, auto) | 20 | 2 | 6 (Sniper 6) | 7d5 1s again, 23.8 (rank 0: 6d4, 15.0) | – | 2 (3) / 206 |
| Magic missile (3, auto) | 30 | 2 | 7 (Sniper 7–8) | 7d5 1s again, 24.0 (rank 0: 6d4, 15.0) | – | 2 (3) / 330 |
| Magic missile (3, auto) | 60 | 3 | 9 (Sniper 9–10) | 8d6+1 1s and 2s again, 34.3 (rank 0: 6d4+1, 16.0) | – | 1 (3) / 731 |
| Cause wounds (4, touch) | 10 | 1 | 0 (Toucher 0–1) | 4d6, 14.0 (rank 0: 4d6, 14.0) | – | 3 (3) / 99 |
| Cause wounds (4, touch) | 20 | 1–2 | 0 (Toucher 2) | 4d8, 17.3 (rank 0: 4d8, 17.3) | – | 3 (3) / 206 |
| Cause wounds (4, touch) | 30 | 2 | 0 (Toucher 3–4) | 4d8, 18.2 (rank 0: 4d8, 18.0) | – | 3 (3) / 330 |
| Cause wounds (4, touch) | 60 | 3 | 0 (Toucher 5–6) | 4d8+3, 21.0 (rank 0: 4d8+2, 20.0) | – | 1 (3) / 731 |
| Acid arrow (5, attack) | 10 | 1–2 | 4–5 (Sniper 4–5) | 6d6 1s again, 22.0 (rank 0: 6d6, 18.7) | – | 4 (5) / 99 |
| Acid arrow (5, attack) | 20 | 2 | 6 (Sniper 6) | 7d7 1s again, 31.0 (rank 0: 6d6, 21.0) | – | 4 (5) / 206 |
| Acid arrow (5, attack) | 30 | 2 | 7 (Sniper 7–8) | 7d7 1s again, 31.3 (rank 0: 6d6, 21.0) | – | 4 (5) / 330 |
| Acid arrow (5, attack) | 60 | 3 | 9 (Sniper 9–10) | 8d8+2 1s and 2s again, 44.0 (rank 0: 6d6+2, 23.0) | – | 1 (5) / 731 |
| Call of the wild (5, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 99 |
| Call of the wild (5, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 206 |
| Call of the wild (5, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 330 |
| Call of the wild (5, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 2 (5) / 731 |
| Arcane bolt (6, area) | 10 | 1 | 4–5 (Sniper 4–5) | 6d6 1s again, 24.8 (rank 0: 6d6, 21.0) | – | 10 (11) / 99 |
| Arcane bolt (6, area) | 20 | 1–2 | 6 (Sniper 6) | 7d7+2 1s again, 32.3 (rank 0: 6d6+2, 22.3) | – | 10 (11) / 206 |
| Arcane bolt (6, area) | 30 | 2 | 7 (Sniper 7–8) | 7d7+2 1s again, 33.3 (rank 0: 6d6+2, 23.0) | – | 9 (11) / 330 |
| Arcane bolt (6, area) | 60 | 3 | 9 (Sniper 9–10) | 8d8+4 1s and 2s again, 46.0 (rank 0: 6d6+4, 25.0) | – | 2 (11) / 731 |
| Frost (6, area) | 10 | 1 | 4–5 (Sniper 4–5) | 6d6 1s again, 24.8 (rank 0: 6d6, 21.0) | – | 13 (14) / 99 |
| Frost (6, area) | 20 | 1–2 | 6 (Sniper 6) | 7d7+2 1s again, 32.3 (rank 0: 6d6+2, 22.3) | – | 13 (14) / 206 |
| Frost (6, area) | 30 | 2 | 7 (Sniper 7–8) | 7d7+2 1s again, 33.3 (rank 0: 6d6+2, 23.0) | – | 12 (14) / 330 |
| Frost (6, area) | 60 | 3 | 9 (Sniper 9–10) | 8d8+4 1s and 2s again, 46.0 (rank 0: 6d6+4, 25.0) | – | 2 (14) / 731 |
| Blight (7, area) | 10 | 0–1 | 0 (Sniper 4–5) | 6d6, 17.5 (rank 0: 6d6, 17.5) | – | 17 (17) / 99 |
| Blight (7, area) | 20 | 1–2 | 0 (Sniper 6) | 6d6, 21.5 (rank 0: 6d6, 21.5) | – | 17 (17) / 206 |
| Blight (7, area) | 30 | 1–2 | 0 (Sniper 7–8) | 6d6+3, 23.0 (rank 0: 6d6+3, 23.0) | – | 17 (17) / 330 |
| Blight (7, area) | 60 | 2 | 0 (Sniper 9–10) | 6d6+3, 24.0 (rank 0: 6d6+3, 24.0) | – | 5 (17) / 731 |
| Scorching ray (7, attack) | 10 | 1–2 | 4–5 (Sniper 4–5) | 6d6 1s again, 22.0 (rank 0: 6d6, 18.7) | – | 6 (7) / 99 |
| Scorching ray (7, attack) | 20 | 2–3 | 6 (Sniper 6) | 9d7 1s again, 36.9 (rank 0: 8d6, 25.7) | – | 6 (7) / 206 |
| Scorching ray (7, attack) | 30 | 3 | 7 (Sniper 7–8) | 9d7 1s again, 40.3 (rank 0: 8d6, 28.0) | – | 6 (7) / 330 |
| Scorching ray (7, attack) | 60 | 4 | 9 (Sniper 9–10) | 8d8+2d12 1s and 2s again, 56.7 (rank 0: 6d6+2d10, 32.0) | – | 1 (7) / 731 |
| Thunderclap (7, area) | 10 | 0 | 4–5 | 2d6 1s again, 8.7 (rank 0: 2d6, 7.0) | – | 12 (13) / 99 |
| Thunderclap (7, area) | 20 | 0 | 6 | 3d7 1s again, 13.3 (rank 0: 2d6, 7.0) | – | 12 (13) / 206 |
| Thunderclap (7, area) | 30 | 0 | 7 | 3d7 1s again, 13.4 (rank 0: 2d6, 7.0) | – | 11 (13) / 330 |
| Thunderclap (7, area) | 60 | 2 | 9 | 6d8+2 1s and 2s again, 33.5 (rank 0: 4d6+2, 16.0) | – | 2 (13) / 731 |
| Flay (8, auto) | 10 | 0–1 | 0 (Sniper 4–5) | 2d6, 6.0 (rank 0: 2d6, 6.0) | – | 8 (8) / 99 |
| Flay (8, auto) | 20 | 1–2 | 0 (Sniper 6) | 2d6, 7.3 (rank 0: 2d6, 7.3) | – | 8 (8) / 206 |
| Flay (8, auto) | 30 | 1–2 | 0 (Sniper 7–8) | 2d8, 8.3 (rank 0: 2d8, 8.3) | – | 8 (8) / 330 |
| Flay (8, auto) | 60 | 2 | 0 (Sniper 9–10) | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 2 (8) / 731 |
| Shatter (8, area) | 10 | 1 | 4–5 (Sniper 4–5) | 6d6 1s again, 24.8 (rank 0: 6d6, 21.0) | – | 13 (14) / 99 |
| Shatter (8, area) | 20 | 1–2 | 6 (Sniper 6) | 7d7+2 1s again, 32.3 (rank 0: 6d6+2, 22.3) | – | 13 (14) / 206 |
| Shatter (8, area) | 30 | 2 | 7 (Sniper 7–8) | 7d7+2 1s again, 33.3 (rank 0: 6d6+2, 23.0) | – | 12 (14) / 330 |
| Shatter (8, area) | 60 | 3 | 9 (Sniper 9–10) | 8d8+4 1s and 2s again, 46.0 (rank 0: 6d6+4, 25.0) | – | 2 (14) / 731 |
| Rime (9, area) | 10 | 1–2 | 4–5 (Sniper 4–5) | 8d6 1s again, 28.8 (rank 0: 8d6, 24.5) | – | 13 (14) / 99 |
| Rime (9, area) | 20 | 2–3 | 6 (Sniper 6) | 9d7 1s again, 40.2 (rank 0: 8d6, 28.3) | – | 13 (14) / 206 |
| Rime (9, area) | 30 | 2–3 | 7 (Sniper 7–8) | 9d7+2 1s again, 41.6 (rank 0: 8d6+2, 29.3) | – | 12 (14) / 330 |
| Rime (9, area) | 60 | 3 | 9 (Sniper 9–10) | 10d8+2 1s and 2s again, 54.5 (rank 0: 8d6+2, 30.0) | – | 2 (14) / 731 |
| Leech (10, attack) | 10 | 1–2 | 0 (Sniper 4–5) | 8d6, 24.5 (rank 0: 8d6, 24.5) | – | 10 (10) / 99 |
| Leech (10, attack) | 20 | 2 | 0 (Sniper 6) | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 10 (10) / 206 |
| Leech (10, attack) | 30 | 2 | 0 (Sniper 7–8) | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 10 (10) / 330 |
| Leech (10, attack) | 60 | 3 | 0 (Sniper 9–10) | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | – | 3 (10) / 731 |
| Call lightning (11, area) | 10 | 1–2 | 4–5 (Sniper 4–5) | 8d6 1s again, 28.8 (rank 0: 8d6, 24.5) | – | 19 (20) / 99 |
| Call lightning (11, area) | 20 | 2–3 | 6 (Sniper 6) | 11d7 1s again, 47.2 (rank 0: 10d6, 33.8) | – | 19 (20) / 206 |
| Call lightning (11, area) | 30 | 3 | 7 (Sniper 7–8) | 11d7 1s again, 49.2 (rank 0: 10d6, 35.0) | – | 18 (20) / 330 |
| Call lightning (11, area) | 60 | 4 | 9 (Sniper 9–10) | 12d8+2 1s and 2s again, 65.0 (rank 0: 10d6+2, 37.0) | – | 4 (20) / 731 |
| Fireball (11, area) | 10 | 1–2 | 4–5 (Sniper 4–5) | 8d6 1s again, 30.0 (rank 0: 8d6, 25.7) | – | 24 (26) / 99 |
| Fireball (11, area) | 20 | 2–3 | 6 (Sniper 6) | 11d7 1s again, 45.8 (rank 0: 10d6, 32.7) | – | 24 (26) / 206 |
| Fireball (11, area) | 30 | 3 | 7 (Sniper 7–8) | 11d7 1s again, 49.2 (rank 0: 10d6, 35.0) | – | 23 (26) / 330 |
| Fireball (11, area) | 60 | 4 | 9 (Sniper 9–10) | 12d8+2 1s and 2s again, 65.0 (rank 0: 10d6+2, 37.0) | – | 5 (26) / 731 |
| Lightning bolt (11, area) | 10 | 1–2 | 4–5 (Sniper 4–5) | 8d6 1s again, 30.0 (rank 0: 8d6, 25.7) | – | 19 (20) / 99 |
| Lightning bolt (11, area) | 20 | 2–3 | 6 (Sniper 6) | 11d7 1s again, 45.8 (rank 0: 10d6, 32.7) | – | 19 (20) / 206 |
| Lightning bolt (11, area) | 30 | 3 | 7 (Sniper 7–8) | 11d7 1s again, 49.2 (rank 0: 10d6, 35.0) | – | 18 (20) / 330 |
| Lightning bolt (11, area) | 60 | 4 | 9 (Sniper 9–10) | 12d8+2 1s and 2s again, 65.0 (rank 0: 10d6+2, 37.0) | – | 4 (20) / 731 |
| Flame (12, area) | 10 | 0–1 | 4–5 (Sniper 4–5) | 8d6 1s again, 30.0 (rank 0: 8d6, 25.7) | – | 27 (29) / 99 |
| Flame (12, area) | 20 | 1–2 | 6 (Sniper 6) | 11d7 1s again, 45.8 (rank 0: 10d6, 32.7) | – | 27 (29) / 206 |
| Flame (12, area) | 30 | 2 | 7 (Sniper 7–8) | 11d7 1s again, 49.2 (rank 0: 10d6, 35.0) | – | 26 (29) / 330 |
| Flame (12, area) | 60 | 3 | 9 (Sniper 9–10) | 12d8+2 1s and 2s again, 65.0 (rank 0: 10d6+2, 37.0) | – | 6 (29) / 731 |
| Reek (13, area) | 10 | 0–1 | 0 (Sniper 4–5) | 4d4, 7.5 (rank 0: 4d4, 7.5) | – | 31 (31) / 99 |
| Reek (13, area) | 20 | 1–2 | 0 (Sniper 6) | 4d4, 10.3 (rank 0: 4d4, 10.3) | – | 31 (31) / 206 |
| Reek (13, area) | 30 | 1–2 | 0 (Sniper 7–8) | 4d4+2, 11.3 (rank 0: 4d4+2, 11.3) | – | 31 (31) / 330 |
| Reek (13, area) | 60 | 2 | 0 (Sniper 9–10) | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 10 (31) / 731 |
| Grave spear (14, area) | 10 | 1–2 | 0 (Sniper 4–5) | 8d6, 24.5 (rank 0: 8d6, 24.5) | – | 25 (25) / 99 |
| Grave spear (14, area) | 20 | 2–3 | 0 (Sniper 6) | 10d6, 33.8 (rank 0: 10d6, 33.8) | – | 25 (25) / 206 |
| Grave spear (14, area) | 30 | 3 | 0 (Sniper 7–8) | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 25 (25) / 330 |
| Grave spear (14, area) | 60 | 4 | 0 (Sniper 9–10) | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 8 (25) / 731 |
| Rebuke (14, area) | 10 | 1–2 | 0 (Sniper 4–5) | 8d6, 24.5 (rank 0: 8d6, 24.5) | – | 34 (34) / 99 |
| Rebuke (14, area) | 20 | 2–3 | 0 (Sniper 6) | 10d6, 33.8 (rank 0: 10d6, 33.8) | – | 34 (34) / 206 |
| Rebuke (14, area) | 30 | 3 | 0 (Sniper 7–8) | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 34 (34) / 330 |
| Rebuke (14, area) | 60 | 4 | 0 (Sniper 9–10) | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 10 (34) / 731 |
| Balefire (16, area) | 10 | 1–2 | 4–5 (Sniper 4–5) | 8d6 1s again, 30.0 (rank 0: 8d6, 25.7) | – | 27 (29) / 99 |
| Balefire (16, area) | 20 | 2 | 6 (Sniper 6) | 9d7 1s again, 39.9 (rank 0: 8d6, 28.0) | – | 27 (29) / 206 |
| Balefire (16, area) | 30 | 2 | 7 (Sniper 7–8) | 9d7 1s again, 40.3 (rank 0: 8d6, 28.0) | – | 26 (29) / 330 |
| Balefire (16, area) | 60 | 3 | 9 (Sniper 9–10) | 10d8+2 1s and 2s again, 54.5 (rank 0: 8d6+2, 30.0) | – | 6 (29) / 731 |
| Call of the forest (20, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 99 |
| Call of the forest (20, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 206 |
| Call of the forest (20, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 330 |
| Call of the forest (20, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 6 (20) / 731 |
| Chain lightning (20, auto) | 10 | 1–2 | 4–5 (Sniper 4–5) | 10d6 1s again, 38.0 (rank 0: 10d6, 32.7) | – | 19 (20) / 99 |
| Chain lightning (20, auto) | 20 | 2–3 | 6 (Sniper 6) | 13d7 1s again, 54.6 (rank 0: 12d6, 39.7) | – | 19 (20) / 206 |
| Chain lightning (20, auto) | 30 | 3 | 7 (Sniper 7–8) | 13d7 1s again, 58.2 (rank 0: 12d6, 42.0) | – | 18 (20) / 330 |
| Chain lightning (20, auto) | 60 | 4 | 9 (Sniper 9–10) | 14d8+3 1s and 2s again, 76.5 (rank 0: 12d6+3, 45.0) | – | 4 (20) / 731 |
| Sunfire (30, area) | 10 | 0–1 | 4–5 (Sniper 4–5) | 12d6 1s again, 46.0 (rank 0: 12d6, 39.7) | – | 85 (90) / 99 |
| Sunfire (30, area) | 20 | 1–2 | 6 (Sniper 6) | 15d7 1s again, 63.5 (rank 0: 14d6, 46.7) | – | 85 (90) / 206 |
| Sunfire (30, area) | 30 | 2 | 7 (Sniper 7–8) | 15d7 1s again, 67.1 (rank 0: 14d6, 49.0) | – | 81 (90) / 330 |
| Sunfire (30, area) | 60 | 4 | 9 (Sniper 9–10) | 18d8+4 1s and 2s again, 98.5 (rank 0: 16d6+4, 60.0) | – | 18 (90) / 731 |
| Call the elements (35, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 99 |
| Call the elements (35, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 206 |
| Call the elements (35, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 330 |
| Call the elements (35, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 11 (35) / 731 |

**Priest**

| Spell (level, res) | L | rung | school rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Mend (1, touch) | 10 | 0–1 | 7 (Battle caster 4) | 2d5+2 1s again, 8.5 (rank 0: 1d4, 2.3) | – | 1 (1) / 77 |
| Mend (1, touch) | 20 | 1 | 8–9 (Battle caster 7–8) | 2d5+2 1s and 2s again, 9.4 (rank 0: 1d4, 2.5) | – | 1 (1) / 160 |
| Mend (1, touch) | 30 | 1–2 | 8–9 (Battle caster 7–8) | 2d5+2 1s and 2s again, 9.8 (rank 0: 1d4, 2.7) | – | 1 (1) / 249 |
| Mend (1, touch) | 60 | 1–2 | 10 (Battle caster 10) | 2d8+3 1s and 2s again, 13.1 (rank 0: 1d6, 3.3) | – | 1 (1) / 542 |
| Cure wounds (3, touch) | 10 | 0–1 | 7 (Battle caster 4) | 4d7+2 1s again, 18.2 (rank 0: 3d6, 9.3) | – | 1 (2) / 77 |
| Cure wounds (3, touch) | 20 | 1 | 8–9 (Battle caster 7–8) | 4d7+2 1s and 2s again, 21.2 (rank 0: 3d6, 10.5) | – | 2 (2) / 160 |
| Cure wounds (3, touch) | 30 | 1–2 | 8–9 (Battle caster 7–8) | 4d7+2 1s and 2s again, 21.7 (rank 0: 3d6, 11.0) | – | 2 (2) / 249 |
| Cure wounds (3, touch) | 60 | 1–2 | 10 (Battle caster 10) | 4d8+6 1s and 2s again, 26.5 (rank 0: 3d6+3, 13.0) | – | 2 (2) / 542 |
| Benison (12, area) | 10 | 0–1 | 7 (Battle caster 4) | 4d7+2 1s again, 18.2 (rank 0: 3d6, 9.3) | – | 20 (22) / 77 |
| Benison (12, area) | 20 | 1 | 8–9 (Battle caster 7–8) | 4d7+2 1s and 2s again, 21.2 (rank 0: 3d6, 10.5) | – | 24 (22) / 160 |
| Benison (12, area) | 30 | 1–2 | 8–9 (Battle caster 7–8) | 4d7+2 1s and 2s again, 21.7 (rank 0: 3d6, 11.0) | – | 25 (22) / 249 |
| Benison (12, area) | 60 | 1–2 | 10 (Battle caster 10) | 4d8+6 1s and 2s again, 26.5 (rank 0: 3d6+3, 13.0) | – | 22 (22) / 542 |
| Rebuke (14, area) | 10 | 1–2 | 0 (Battle caster 4) | 8d6, 25.7 (rank 0: 8d6, 25.7) | – | 36 (34) / 77 |
| Rebuke (14, area) | 20 | 2–3 | 0 (Battle caster 7–8) | 10d6, 33.8 (rank 0: 10d6, 33.8) | – | 42 (34) / 160 |
| Rebuke (14, area) | 30 | 2–3 | 0 (Battle caster 7–8) | 10d6, 33.8 (rank 0: 10d6, 33.8) | – | 44 (34) / 249 |
| Rebuke (14, area) | 60 | 3 | 0 (Battle caster 10) | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 42 (34) / 542 |
| Orison (prayer) | 10 | – | – | no roll: answers 1 of 2 (wounds, hunger) | – | 0 (0) / 77 |
| Prayer (prayer) | 20 | – | – | no roll: answers 1 of 5 (disease, poison, wounds, release, hunger) | – | 0 (0) / 160 |
| Litany (prayer) | 30 | – | – | no roll: answers 1 of 6 (disease, poison, wounds, release, hunger, restoration) | – | 0 (0) / 249 |
| Miracle (prayer) | 60 | – | – | no roll: answers all of 7 (disease, poison, wounds, release, mana, hunger, restoration) | – | 0 (0) / 542 |

The three tables are #905's run (Malison's hexed column: a damage spell's mean on a target under one hex, from the witch's level 18), on `main` after W1 (#908) and Z1 (#909).

**Witch**

| Spell (level, res) | L | rung | school rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Grave breath (1, area) | 10 | 2 | 5 (Sniper 3–4) | 4d4+2 1s again, 13.5 (rank 0: 4d4, 10.0) | – | 2 (3) / 86 |
| Grave breath (1, area) | 20 | 2–3 | 6 (Sniper 6) | 4d4+2 1s again, 13.8 (rank 0: 4d4, 10.3) | 16.8 | 2 (3) / 188 |
| Grave breath (1, area) | 30 | 2–3 | 7 (Sniper 7) | 4d4+4 1s again, 15.2 (rank 0: 4d4+2, 11.7) | 18.2 | 2 (3) / 302 |
| Grave breath (1, area) | 60 | 3 | 10 (Sniper 10) | 5d4+6 1s and 2s again, 21.0 (rank 0: 4d4+2, 12.0) | 24.0 | 2 (3) / 679 |
| Cause wounds (4, touch) | 10 | 1–2 | 5 (Toucher 3–4) | 4d8+3 1s again, 19.9 (rank 0: 4d8, 15.3) | – | 2 (3) / 86 |
| Cause wounds (4, touch) | 20 | 2 | 6 (Toucher 3–4) | 4d8+3 1s again, 22.6 (rank 0: 4d8, 18.0) | 25.6 | 2 (3) / 188 |
| Cause wounds (4, touch) | 30 | 2 | 7 (Toucher 3–4) | 4d8+3 1s again, 22.6 (rank 0: 4d8, 18.0) | 25.6 | 2 (3) / 302 |
| Cause wounds (4, touch) | 60 | 3 | 10 (Toucher 6) | 5d8+7 1s and 2s again, 33.2 (rank 0: 4d8+2, 20.0) | 36.2 | 2 (3) / 679 |
| Call of the wild (5, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 86 |
| Call of the wild (5, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 188 |
| Call of the wild (5, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 302 |
| Call of the wild (5, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 4 (5) / 679 |
| Blight (7, area) | 10 | 1 | 5 (Sniper 3–4) | 6d6+2 1s again, 25.5 (rank 0: 6d6, 21.0) | – | 16 (17) / 86 |
| Blight (7, area) | 20 | 1–2 | 6 (Sniper 6) | 6d6+2 1s again, 26.0 (rank 0: 6d6, 21.5) | 29.0 | 16 (17) / 188 |
| Blight (7, area) | 30 | 1–2 | 7 (Sniper 7) | 6d6+5 1s again, 28.0 (rank 0: 6d6+3, 23.5) | 31.0 | 15 (17) / 302 |
| Blight (7, area) | 60 | 2 | 10 (Sniper 10) | 7d6+7 1s and 2s again, 36.2 (rank 0: 6d6+3, 24.0) | 39.2 | 11 (17) / 679 |
| Flay (8, auto) | 10 | 1 | 5 (Sniper 3–4) | 2d6+2 1s again, 9.8 (rank 0: 2d6, 7.0) | – | 7 (8) / 86 |
| Flay (8, auto) | 20 | 1–2 | 6 (Sniper 6) | 2d6+2 1s again, 10.2 (rank 0: 2d6, 7.3) | 13.2 | 7 (8) / 188 |
| Flay (8, auto) | 30 | 1–2 | 7 (Sniper 7) | 2d8+2 1s again, 11.5 (rank 0: 2d8, 8.7) | 14.5 | 7 (8) / 302 |
| Flay (8, auto) | 60 | 2 | 10 (Sniper 10) | 3d8+4 1s and 2s again, 19.8 (rank 0: 2d8, 9.0) | 22.8 | 5 (8) / 679 |
| Leech (10, attack) | 10 | 2 | 5 (Sniper 3–4) | 8d6+2 1s again, 33.3 (rank 0: 8d6, 28.0) | – | 9 (10) / 86 |
| Leech (10, attack) | 20 | 2 | 6 (Sniper 6) | 8d6+2 1s again, 33.3 (rank 0: 8d6, 28.0) | 36.3 | 9 (10) / 188 |
| Leech (10, attack) | 30 | 2 | 7 (Sniper 7) | 8d6+2 1s again, 33.3 (rank 0: 8d6, 28.0) | 36.3 | 9 (10) / 302 |
| Leech (10, attack) | 60 | 3 | 10 (Sniper 10) | 9d6+6 1s and 2s again, 43.5 (rank 0: 8d6+2, 30.0) | 46.5 | 6 (10) / 679 |
| Reek (13, area) | 10 | 1 | 5 (Sniper 3–4) | 4d4+2 1s again, 13.5 (rank 0: 4d4, 10.0) | – | 29 (31) / 86 |
| Reek (13, area) | 20 | 1–2 | 6 (Sniper 6) | 4d4+2 1s again, 13.8 (rank 0: 4d4, 10.3) | 16.8 | 29 (31) / 188 |
| Reek (13, area) | 30 | 1–2 | 7 (Sniper 7) | 4d4+4 1s again, 15.2 (rank 0: 4d4+2, 11.7) | 18.2 | 27 (31) / 302 |
| Reek (13, area) | 60 | 2 | 10 (Sniper 10) | 5d4+6 1s and 2s again, 21.0 (rank 0: 4d4+2, 12.0) | 24.0 | 21 (31) / 679 |
| Grave spear (14, area) | 10 | 2–3 | 5 (Sniper 3–4) | 10d6+2 1s again, 35.9 (rank 0: 10d6, 30.3) | – | 23 (25) / 86 |
| Grave spear (14, area) | 20 | 3 | 6 (Sniper 6) | 10d6+2 1s again, 41.2 (rank 0: 10d6, 35.0) | 44.2 | 23 (25) / 188 |
| Grave spear (14, area) | 30 | 3 | 7 (Sniper 7) | 10d6+2 1s again, 41.2 (rank 0: 10d6, 35.0) | 44.2 | 22 (25) / 302 |
| Grave spear (14, area) | 60 | 4 | 10 (Sniper 10) | 11d6+6 1s and 2s again, 51.8 (rank 0: 10d6+2, 37.0) | 54.8 | 17 (25) / 679 |
| Call of the forest (20, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 86 |
| Call of the forest (20, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 188 |
| Call of the forest (20, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 302 |
| Call of the forest (20, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 16 (20) / 679 |
| Call the elements (35, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 86 |
| Call the elements (35, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 188 |
| Call the elements (35, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 302 |
| Call the elements (35, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 28 (35) / 679 |

## School pace

The trained caster's school and caster style as their casting trained them (`balance_probe/schools.rs`): each day of each level, one pool spent on the pace spell (of the school's spells, the most a cast among those costing at most a fifth of the pool, never one cast only under the open sky; for the priest a heal, at most one a kill), every cast landing on a monster of the caster's level through `World::practise`. Each row: the pace spell over the level before and its cost (seed 1's); casts a day from the pool, and the spell casts a day the same level's fights took (the stat training's careers, which refill the pool before every cast; the priest's one a kill); the school's rank and the caster style's at the level (the seeds' range); and the pool check, the pool over a single-target spell of the caster's own level (its base mana: the level; no rank's cut, no talent).

The level 1 row has no level before it and practises nothing: it says what a starting caster's first pool buys of the spells they know (`schools::first_spell`: an area before a single target, then the most mean roll; a healer's heal), #979. These level 1 rows and the Burning hands and Grave breath rows of "Spells" are #979's run; the other rows of these tables are left as they were, so as not to mix that PR's numbers with what other PRs moved since the last full update.

**Wizard** (evocation)

| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |
|---|---|---|---|---|---|
| 1 | Burning hands (3) | 5.8 / 0.0 | 2 | Sniper 1 | 16 / 1 = 16.5 |
| 5 | Scorching ray (7) | 5.8 / 79.3 | 3 | Sniper 3 | 51 / 5 = 10.2 |
| 10 | Rime (13) | 5.8 / 122.4 | 4–5 | Sniper 4–5 | 99 / 10 = 9.9 |
| 15 | Chain lightning (19) | 7.0 / 179.2 | 5 | Sniper 5 | 151 / 15 = 10.1 |
| 20 | Chain lightning (19) | 9.8 / 138.9 | 6 | Sniper 6 | 206 / 20 = 10.3 |
| 30 | Chain lightning (18) | 16.8 / 231.7 | 7 | Sniper 7–8 | 330 / 30 = 11.0 |
| 40 | Sunfire (72) | 5.8 / 184.7 | 8 | Sniper 8 | 458 / 40 = 11.5 |
| 50 | Sunfire (58) | 9.8 / 274.6 | 8 | Sniper 8–9 | 590 / 50 = 11.8 |
| 60 | Sunfire (54) | 13.2 / 262.6 | 9 | Sniper 9–10 | 731 / 60 = 12.2 |

**Priest** (restoration)

| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |
|---|---|---|---|---|---|
| 1 | Mend (1) | 13.0 / 0.0 | 1 | Toucher 1 | 13 / 1 = 13.0 |
| 5 | Cure wounds (2) | 16.2 / 42.0 | 3–4 | Toucher 3–4 | 40 / 5 = 7.9 |
| 10 | Cure wounds (1) | 68.0 / 73.0 | 7 | Toucher 7 | 77 / 10 = 7.7 |
| 15 | Benison (20) | 20.0 / 98.5 | 8 | Battle caster 6–8 | 117 / 15 = 7.8 |
| 20 | Benison (21) | 6.8 / 74.3 | 8–9 | Battle caster 7–8 | 160 / 20 = 8.0 |
| 30 | Benison (24) | 9.5 / 88.0 | 8–9 | Battle caster 7–8 | 249 / 30 = 8.3 |
| 40 | Benison (25) | 13.0 / 75.0 | 9 | Battle caster 8–9 | 342 / 40 = 8.6 |
| 50 | Benison (24) | 17.7 / 83.2 | 9–10 | Battle caster 9–10 | 441 / 50 = 8.8 |
| 60 | Benison (22) | 23.7 / 91.2 | 10 | Battle caster 10 | 542 / 60 = 9.0 |

**Witch** (necromancy)

| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |
|---|---|---|---|---|---|
| 1 | Grave breath (3) | 4.0 / 0.0 | 1 | Sniper 0 | 14 / 1 = 13.7 |
| 5 | Cause wounds (3) | 11.7 / 82.5 | 3 | Toucher 3 | 43 / 5 = 8.5 |
| 10 | Leech (9) | 8.0 / 139.1 | 5 | Sniper 3–4 | 86 / 10 = 8.7 |
| 15 | Grave spear (23) | 7.5 / 205.4 | 5–6 | Sniper 5 | 136 / 15 = 9.1 |
| 20 | Grave spear (23) | 7.3 / 172.3 | 6 | Sniper 6 | 188 / 20 = 9.4 |
| 30 | Grave spear (22) | 12.5 / 308.2 | 7 | Sniper 7 | 302 / 30 = 10.1 |
| 40 | Grave spear (22) | 19.7 / 292.0 | 8 | Sniper 8–9 | 422 / 40 = 10.5 |
| 50 | Grave spear (18) | 29.3 / 442.1 | 9–10 | Sniper 9–10 | 547 / 50 = 10.9 |
| 60 | Grave spear (17) | 38.8 / 445.5 | 10 | Sniper 10 | 679 / 60 = 11.3 |

## Weapon pace

The trained fighters' weapon skill and fighting style as their blows trained them (#1095; `balance_probe/training.rs`): every blow of the stat training's fights, each level's kills against the monsters within two levels of theirs, earns its marks through `World::practise` (the real `[marks]`, curves, and talents), the main hand's skill and style for both hands. Each cell: the level at which the skill first stood at the rank (the range over the seeds that fought with it; `>60` where a seed had not reached it by level 60). The untrained careers take these ranks from their trained twin; the skills the fights do not train (a shield, a second weapon skill) are set by level (`rank`, the warrior's pace).

**Warrior**

| rank | Long blades | Sword and Board |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 3–4 | 3–4 |
| 4 | 5–6 | 5–6 |
| 5 | 7–8 | 7–8 |
| 6 | 9–11 | 10–11 |
| 7 | 14–16 | 15–16 |
| 8 | 20–25 | 23–25 |
| 9 | 28–35 | 31–35 |
| 10 | 36–45 | 41–45 |

**Thief**

| rank | Daggers and knives | Duelist (3 of 6 seeds) | Twin Blades (3 of 6 seeds) |
|---|---|---|---|
| 1 | 2 | 2 | 2 |
| 2 | 2 | 2 | 2 |
| 3 | 2–3 | 3 | 2–3 |
| 4 | 3–5 | 4–5 | 3–4 |
| 5 | 6 | 6 | 6 |
| 6 | 7–8 | 8 | 7–8 |
| 7 | 12–13 | 12–13 | 12–13 |
| 8 | 17–20 | 17–20 | 19 |
| 9 | 25–28 | 25–28 | 26–27 |
| 10 | 32–36 | 32–36 | 34–35 |

**Priest**

| rank | Maces and clubs | Sword and Board |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 3 | 3 |
| 4 | 5 | 5 |
| 5 | 7 | 7 |
| 6 | 9–10 | 9–10 |
| 7 | 14–15 | 14–15 |
| 8 | 21–23 | 21–23 |
| 9 | 30–33 | 30–33 |
| 10 | 38–42 | 38–42 |

**Berserker**

| rank | Axes | Great Weapon |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 4 | 4 |
| 4 | 6 | 6 |
| 5 | 8–9 | 8–9 |
| 6 | 11–12 | 11–12 |
| 7 | 17–19 | 17–19 |
| 8 | 26–29 | 26–29 |
| 9 | 38–42 | 38–42 |
| 10 | 50–55 | 50–55 |

**Ascetic**

| rank | Long blades (1 of 6 seeds) | Staves (3 of 6 seeds) | Unarmed (2 of 6 seeds) | Duelist (1 of 6 seeds) | Brawler (2 of 6 seeds) | Staff Fighter (3 of 6 seeds) |
|---|---|---|---|---|---|---|
| 1 | 2 | 2 | 2 | 2 | 2 | 2 |
| 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| 3 | 4 | 3 | 3 | 4 | 3 | 3 |
| 4 | 6 | 5 | 5 | 6 | 5 | 5 |
| 5 | 8 | 7 | 7 | 8 | 7 | 7 |
| 6 | 11 | 9–10 | 10 | 11 | 10 | 9–10 |
| 7 | 17 | 14–15 | 15–16 | 17 | 15–16 | 14–15 |
| 8 | 25 | 22–23 | 23–25 | 25 | 23–25 | 22–23 |
| 9 | 36 | 30–31 | 33–37 | 36 | 33–37 | 30–31 |
| 10 | 46 | 38–39 | 43–48 | 46 | 43–48 | 38–39 |

**Paladin**

| rank | Long blades | Sword and Board |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 3–4 | 3–4 |
| 4 | 5–6 | 5–6 |
| 5 | 7–8 | 7–8 |
| 6 | 10 | 10 |
| 7 | 15–16 | 15–16 |
| 8 | 23–25 | 23–25 |
| 9 | 32–35 | 32–35 |
| 10 | 41–45 | 41–45 |

**Reading it** (#1095). The warrior, the priest, and the paladin land on the issue's aim: rank 6
about level 10, rank 8 about 21 to 26, rank 10 about 37 to 47. With the cap at 100 % but the
old exponential curves (15, 30, … 7680) the same warrior stood at rank 6 by level 6 and rank 10
by 13 to 15 (measured on the way); with marks up to 200 % before #1095, sooner still (the
issue's level-13 warrior at rank 9). A fighter with two blows a round
trains faster (the thief's daggers, both hands, rank 10 by 32 to 37), one with slow, heavy blows
slower (the berserker's great axe, rank 10 by 52 to 57). The warrior's Weapon drill (+25 % marks
in melee skills) is in these numbers: each rank takes a fifth fewer blows, worth more levels now
that the last ranks take a career, so it still earns its place. Practised technique asks for rank 7,
now about level 15 instead of 7. The schools and caster styles keep a gentler curve of their own
(`school`, flatter than doubling from rank 8): with the 100 % cap the wizard's evocation is
rank 4 at level 10, 6 at 20, 8 at 40, and 9 at 60 (Sniper 9 to 10; "School pace"), where before
#1095 it was 5, 7, 9, and 10.

## Sneak attacks

The thief's main-hand blow on the monsters of their level (trained stats), aware and caught unawares. The Sneak attack value assumed, the dice it gives with the talents, and the share of the monsters within two levels with vitals (no dice and no critical on the rest, so their first blow gains only the flat-footed Defence); then each cell: P(hit) aware / unaware % · damage of a blow aware : the first blow unaware · how many times as much, the median over the monsters within two levels that have vitals.

| L | Sneak attack | dice | with vitals | Thief plain | Thief magic |
|---|---|---|---|---|---|
| 1 | 15 | 1d6 | 79% | 72/78 · 2.5:5.8 · ×2.3 | 72/78 · 2.4:5.8 · ×2.4 |
| 5 | 29 | 2d6 | 59% | 79/88 · 3.6:10.2 · ×2.8 | 80/89 · 4.6:11.3 · ×2.4 |
| 10 | 47 | 2d6 | 73% | 91/97 · 7.9:15.3 · ×1.9 | 94/98 · 9.6:16.9 · ×1.8 |
| 15 | 64 | 3d6 | 63% | 98/99 · 10.0:20.6 · ×2.1 | 99/99 · 12.6:23.0 · ×1.8 |
| 20 | 82 | 4d6 | 65% | 99/99 · 11.8:25.8 · ×2.2 | 99/99 · 14.0:27.9 · ×2.0 |
| 30 | 117 | 5d6 | 63% | 99/99 · 13.9:31.3 · ×2.3 | 99/99 · 23.6:40.9 · ×1.7 |
| 40 | 152 | 8d6 | 61% | 99/99 · 16.3:43.9 · ×2.7 | 99/99 · 22.8:50.5 · ×2.2 |
| 50 | 187 | 8d6 | 54% | 99/99 · 16.8:44.8 · ×2.7 | 99/99 · 26.9:54.5 · ×2.0 |
| 60 | 222 | 9d6 | 50% | 97/99 · 16.6:50.4 · ×3.0 | 99/99 · 24.6:59.0 · ×2.4 |

## Rage

The berserker in plain gear (a great axe; from level 5 the warrior's armour: chain mail, banded mail from 15, plate from 30), trained stats, against the monsters within two levels: calm (the stance the run starts in) and raging (Raging and the frenzy stance). The pool as the Rage ability grants it (start / most); the fight cells as in "Fights"; then the rage a round against one foe, the medians of hits taken × their build + criticals × theirs + kills × theirs − the decay (kills a round: one over the rounds to kill, so a fight's average); the same without the kills (a foe that outlasts the rage, or the turns before the first kill) and how many rounds the rage lasts from its start at that net (`grows` when it does not fall); and the net against three foes (three foes' hits, the same criticals and kills).

| L | rage | Calm | Raging | one foe: struck + crit + kill − decay = net | one foe that does not die: net · rounds the rage lasts | three foes: net |
|---|---|---|---|---|---|---|
| 1 | 12/20 | 83/70 · 8.3:0.8 · 1.4 / 65 | 94/96 · 13.8:1.1 · 0.9 / 48 | 0.98 + 0.07 + 3.58 − 1.00 = +3.64 | +0.06 · grows | +5.61 |
| 5 | 12/20 | 83/66 · 8.3:1.0 · 2.8 / 58 | 94/94 · 13.1:1.5 · 1.7 / 36 | 0.95 + 0.05 + 1.89 − 1.00 = +1.89 | +0.00 · grows | +3.78 |
| 10 | 12/20 | 87/77 · 11.6:3.1 · 3.8 / 26 | 97/98 · 16.7:3.5 · 2.7 / 24 | 0.99 + 0.07 + 1.18 − 1.00 = +1.24 | +0.06 · grows | +3.21 |
| 15 | 12/20 | 87/90 · 13.2:5.3 · 4.9 / 23 | 96/99 · 18.4:4.4 · 3.4 / 28 | 0.98 + 0.06 + 0.92 − 1.00 = +0.97 | +0.05 · grows | +2.93 |
| 20 | 12/20 | 93/92 · 17.1:6.2 · 4.8 / 26 | 99/99 · 23.0:6.4 · 3.4 / 25 | 0.99 + 0.08 + 0.91 − 1.00 = +0.98 | +0.08 · grows | +2.97 |
| 30 | 12/20 | 94/98 · 25.3:15.3 · 6.3 / 16 | 98/99 · 33.4:13.7 · 5.0 / 18 | 1.01 + 0.14 + 0.61 − 0.94 = +0.81 | +0.20 · grows | +2.83 |
| 40 | 18/20 | 95/99 · 31.1:15.6 · 6.8 / 24 | 99/99 · 54.1:14.4 · 3.9 / 26 | 1.05 + 0.20 + 2.08 − 0.67 = +2.67 | +0.58 · grows | +4.78 |
| 50 | 18/30 | 94/99 · 42.9:24.1 · 6.9 / 21 | 99/99 · 76.3:22.8 · 3.8 / 22 | 1.03 + 0.24 + 2.17 − 0.67 = +2.77 | +0.60 · grows | +4.83 |
| 60 | 18/30 | 91/99 · 44.3:23.8 · 5.5 / 28 | 98/99 · 76.8:22.9 · 3.5 / 29 | 1.13 + 0.17 + 2.29 − 0.67 = +2.92 | +0.63 · grows | +5.19 |

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
| 1 | 5.3 | 0.33 | 80/63 · 5.5:1.2 · 2.1 / 21 | 80/63 · 10.2:1.1 · 1.1 / 21 | 0.99 − 0.33 = 0.66 · 8.1 |
| 5 | 5.3 | 0.33 | 89/72 · 7.7:3.5 · 2.8 / 9.4 | 89/72 · 14.5:3.5 · 1.5 / 9.5 | 1.00 − 0.33 = 0.67 · 8.0 |
| 10 | 6.3 | 0.33 | 95/77 · 14.7:6.1 · 3.0 / 8.9 | 95/77 · 28.6:6.0 · 1.5 / 8.9 | 1.13 − 0.33 = 0.80 · 8.0 |
| 15 | 7.7 | 0.33 | 98/82 · 20.1:9.3 · 3.2 / 8.8 | 98/82 · 39.6:9.3 · 1.7 / 8.8 | 1.20 − 0.33 = 0.87 · 8.8 |
| 20 | 9.0 | 0.33 | 99/81 · 21.0:11.0 · 3.7 / 10 | 99/81 · 41.4:11.1 · 1.9 / 10 | 1.22 − 0.33 = 0.89 · 10 |
| 30 | 11.5 | 0.33 | 99/89 · 26.1:19.7 · 5.3 / 9.0 | 99/89 · 50.8:19.8 · 2.7 / 9.0 | 1.32 − 0.33 = 0.99 · 12 |
| 40 | 13.3 | 0.33 | 99/91 · 37.2:19.2 · 4.7 / 13 | 99/91 · 74.6:18.8 · 2.5 / 14 | 1.43 − 0.33 = 1.10 · 12 |
| 50 | 15.5 | 0.33 | 99/97 · 46.3:29.0 · 5.8 / 11 | 99/97 · 92.2:29.1 · 2.9 / 11 | 1.54 − 0.33 = 1.20 · 13 |
| 60 | 19.3 | 0.78 | 99/96 · 55.9:30.0 · 5.2 / 11 | 99/96 · 105.8:30.1 · 2.7 / 11 | 1.55 − 0.78 = 0.77 · 25 |

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
| 1 | 10.8 | 0.10 | 78/48 · 6.9:0.1 · 1.8 / 760 | 83/48 · 10.4:0.1 · 1.1 / 740 | 87/42 · 13.3:0.8 · 1.2 / 37 | - | 1.36 · 2.2 | 1.84 · 1.6 |
| 5 | 10.8 | 0.10 | 84/46 · 6.7:1.0 · 3.5 / 44 | 87/46 · 9.8:1.0 · 2.3 / 49 | 91/51 · 12.7:1.2 · 1.9 / 30 | - | 1.28 · 2.3 | 1.75 · 1.7 |
| 10 | 11.8 | 0.10 | 91/47 · 10.0:2.1 · 4.4 / 29 | 94/47 · 13.5:2.1 · 3.3 / 29 | 94/45 · 16.9:2.8 · 2.8 / 23 | - | 1.35 · 2.2 | 1.82 · 1.6 |
| 15 | 11.8 | 0.10 | 92/52 · 10.2:3.5 · 6.1 / 26 | 95/52 · 13.6:3.5 · 4.5 / 26 | 93/52 · 17.8:4.9 · 3.2 / 19 | - | 1.42 · 2.1 | 1.95 · 1.5 |
| 20 | 13.0 | 0.10 | 97/44 · 12.2:3.2 · 6.9 / 47 | 98/44 · 19.2:3.3 · 4.2 / 51 | 98/48 · 22.4:5.4 · 3.6 / 24 | - | 1.38 · 2.2 | 1.82 · 1.7 |
| 30 | 14.2 | 0.10 | 97/47 · 14.0:9.0 · 11 / 22 | 98/47 · 24.4:8.9 · 6.2 / 22 | 97/76 · 28.5:14.0 · 3.3 / 14 | - | 1.42 · 2.1 | 1.89 · 1.6 |
| 40 | 15.8 | 0.10 | 97/57 · 18.0:8.3 · 11 / 36 | 99/57 · 32.8:8.2 · 6.0 / 37 | 97/82 · 38.7:21.3 · 3.6 / 13 | - | 1.52 · 2.0 | 2.10 · 1.4 |
| 50 | 17.2 | 0.10 | 98/59 · 19.4:15.6 · 14 / 25 | 99/59 · 38.2:15.2 · 7.3 / 25 | 99/76 · 45.2:20.3 · 7.7 / 19 | 99/76 · 67.3:20.6 · 5.2 / 18 | 1.56 · 1.9 | 2.15 · 1.4 |
| 60 | 18.7 | 0.10 | 93/54 · 18.7:10.9 · 13 / 42 | 98/54 · 39.4:10.9 · 6.2 / 43 | 97/96 · 45.1:33.0 · 4.0 / 12 | 97/96 · 69.5:33.3 · 2.7 / 11 | 1.47 · 2.0 | 2.00 · 1.5 |

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


## Ranger and companion

The ranger (#923) and their starting beast, in the starting kit (the short sword: the probe's blows are melee, so the bow is not measured), stats as rolled, short blades at the rank the other tables assume for the level, seeds 1 to 6 (each draws its own beast), against the monsters within two levels. Fight cells as in "Fights" (P(it hits) / P(it is hit) % · damage a round · rounds to kill / rounds to die) for the ranger alone and the companion alone (its grade is the ranger's level: its hit points, to-hit, damage, Defence and Protection raised by `[growth]`); together, the rounds to kill with both striking (the monster's hit points over the two damages added); the last column is the ranger alone's rounds over those.

| L | beasts | its hp | Ranger | Companion | Together: rounds to kill | alone over together |
|---|---|---|---|---|---|---|
| 1 | hound, boar | 17 | 73/53 · 4.4:0.6 · 2.7 / 78 | 71/70 · 4.4:1.5 · 3.4 / 12 | 1.5 | x1.8 |
| 5 | hound, boar | 24 | 79/64 · 4.4:2.7 · 5.1 / 13 | 73/78 · 4.0:4.1 · 7.8 / 6.1 | 3.1 | x1.7 |
| 10 | hound, boar | 35 | 79/74 · 6.0:5.4 · 7.8 / 8.3 | 82/83 · 6.8:6.9 · 8.4 / 5.1 | 4.1 | x1.9 |
| 15 | hound, boar | 46 | 82/77 · 7.2:8.3 · 9.1 / 6.7 | 85/85 · 10.4:10.3 · 8.0 / 4.5 | 4.3 | x2.1 |
| 20 | hound, boar | 57 | 80/84 · 5.8:11.0 · 14 / 6.2 | 92/82 · 12.7:11.2 · 8.3 / 5.1 | 5.1 | x2.6 |
| 30 | hound, boar | 79 | 76/89 · 5.5:20.2 · 24 / 4.7 | 96/84 · 19.6:20.1 · 8.1 / 4.0 | 6.1 | x4.0 |
| 40 | hound, boar | 101 | 66/97 · 7.0:21.0 · 27 / 6.0 | 98/86 · 27.4:17.2 · 6.6 / 6.0 | 5.3 | x5.1 |
| 50 | hound, boar | 124 | 66/99 · 7.5:30.9 · 39 / 4.7 | 99/91 · 34.4:27.4 · 8.0 / 4.6 | 6.7 | x5.9 |
| 60 | hound, boar | 145 | 52/99 · 3.9:30.4 · 65 / 4.8 | 99/84 · 35.4:22.3 · 6.5 / 6.7 | 5.9 | x11.0 |

What it says (2026-11-03, C8): the hound (grade the ranger's level) out-damages the melee ranger from level 5 on and by level 60 deals about nine times the short sword's damage a round (34.8 against 3.8; the ranger's real weapon is the bow, which this probe does not measure, so read the ranger column as a floor). It hits 71 to 99 % of the time against monsters of the ranger's level and kills one in about seven rounds at every level, and it lasts about five to six rounds against them, the same as the ranger does. The two together kill 1.6 to 6.3 rounds; the companion is the larger half of the pair from level 5. `grade_percent = 100` (the design's Q1) stands (the beast's level is the ranger's); whether a hound that out-damages its melee master nine to one at level 60 is too much is the owner's call (a tamed wolf is not measured here: it is the same growth from a higher base, and the probe's seeds draw only the hound and the boar).

## Summons

The druid's calls (#839), cast by a human druid (untrained stats, every score as rolled, seed 1) at each level a call comes or Pack leader (32) and Lord of the wild (60) change them: what comes of one cast (each type's count), for how many minutes (`lasting`: until slain), the mana over the pool, and each called type against the monsters within two levels of the druid, as in "Fights" (the beast's side bare: P(it hits) / P(it is hit) % · damage a round · rounds to kill / rounds to die).

| Call | L | comes | minutes | mana / pool | the called against the druid's level |
|---|---|---|---|---|---|
| Call of the wild (5) | 6 | 1 wolf | 5 | 5 / 43 (9 casts) | wolf 4: 69/75 · 1.6:4.2 · 12 / 2.9 |
| Call of the wild (5) | 25 | 1 wolf | 5 | 5 / 161 (32 casts) | wolf 4: 54/98 · 0.9:19.8 · 103 / 0.6 |
| Call of the forest (20) | 25 | 1 bear | 5 | 20 / 161 (8 casts) | bear 12: 93/99 · 10.0:19.5 · 12 / 3.1 |
| Call of the wild (5) | 32 | 2 wolves | 7 | 5 / 204 (41 casts) | wolf 4: 42/99 · 0.5:25.6 · 255 / 0.5 |
| Call of the forest (20) | 32 | 2 bears | 7 | 20 / 204 (10 casts) | bear 12: 88/99 · 8.5:25.3 · 17 / 2.4 |
| Call of the wild (5) | 40 | 2 wolves | 7 | 5 / 253 (51 casts) | wolf 4: 28/99 · 0.3:24.7 · 552 / 0.5 |
| Call of the forest (20) | 40 | 2 bears | 7 | 20 / 253 (13 casts) | bear 12: 78/99 · 7.3:24.0 · 23 / 2.5 |
| Call the elements (35) | 40 | 1 greater air elemental, 1 greater fire elemental | 7 | 35 / 253 (7 casts) | greater earth elemental 24: 91/99 · 16.1:17.0 · 10 / 11; greater water elemental 26: 88/99 · 14.4:20.6 · 12 / 3.1; greater air elemental 31: 99/83 · 28.0:14.1 · 6.0 / 4.4; greater fire elemental 38: 99/97 · 27.1:18.0 · 7.5 / 6.8 |
| Call of the wild (5) | 50 | 2 wolves | 7 | 5 / 314 (63 casts) | wolf 4: 22/99 · 0.1:33.3 · >999 / 0.4 |
| Call of the forest (20) | 50 | 2 bears | 7 | 20 / 314 (16 casts) | bear 12: 72/99 · 5.6:33.4 · 42 / 1.8 |
| Call the elements (35) | 50 | 1 greater air elemental, 1 greater fire elemental | 7 | 35 / 314 (9 casts) | greater earth elemental 24: 88/99 · 14.2:21.6 · 17 / 8.5; greater water elemental 26: 83/99 · 12.0:26.0 · 21 / 2.5; greater air elemental 31: 99/97 · 27.2:21.0 · 10 / 2.9; greater fire elemental 38: 99/99 · 27.3:29.9 · 9.9 / 4.1 |
| Call of the wild (5) | 60 | 2 wolves | lasting | 5 / 374 (75 casts) | wolf 4: 7/99 · 0.0:35.6 · >999 / 0.3 |
| Call of the forest (20) | 60 | 2 bears | lasting | 20 / 374 (19 casts) | bear 12: 46/99 · 3.3:35.8 · 62 / 1.7 |
| Call the elements (35) | 60 | 1 greater air elemental, 1 greater fire elemental | lasting | 35 / 374 (11 casts) | greater earth elemental 24: 69/99 · 11.1:23.0 · 21 / 8.1; greater water elemental 26: 62/99 · 8.7:30.2 · 25 / 2.2; greater air elemental 31: 93/99 · 23.2:25.3 · 11 / 2.5; greater fire elemental 38: 96/99 · 27.0:29.7 · 8.7 / 4.1 |

## Familiar

The witch's familiar (#929, #933), a human witch (untrained stats, every score as rolled, seed 1) with each of the five kinds a run may draw in turn, at its grade (her level). Its touch is her Evil eye and needs a blow that lands, so the table is the familiar's side bare against the monsters within two levels of the witch's (as in "Fights", the monsters' gear left out): **hit** the chance a blow lands, lowest / median / highest over the window; **blows** a round (a normal turn); **hexes** laid a round (blows × median hit); **rounds to the first** hex (one over the hexes a round); **degrees** her Evil eye lays; **eye** the mana one cast of it costs over her pool, which every landed blow saves her; the familiar's hit points, mean damage of a hit, and to-hit.

| L | familiar | hit % (low / median / high) | blows | hexes a round | rounds to the first | degrees | eye (mana / pool) | hp · mean hit · to hit |
|---|---|---|---|---|---|---|---|---|
| 1 | cat | 19 / 31 / 69 | 1.30 | 0.41 | 2.5 | 1 | 1 / 13 | 10 · 1.0 · -4 |
| 1 | toad | 19 / 31 / 69 | 0.90 | 0.28 | 3.6 | 1 | 1 / 13 | 10 · 1.0 · -4 |
| 1 | raven | 19 / 31 / 69 | 1.30 | 0.41 | 2.5 | 1 | 1 / 13 | 9 · 1.0 · -4 |
| 1 | hare | 25 / 38 / 75 | 1.60 | 0.61 | 1.6 | 1 | 1 / 13 | 10 · 1.0 · -3 |
| 1 | bat | 15 / 25 / 62 | 2.00 | 0.50 | 2.0 | 1 | 1 / 13 | 10 · 1.0 · -5 |
| 5 | cat | 4 / 38 / 75 | 1.30 | 0.50 | 2.0 | 1 | 1 / 42 | 18 · 1.0 · -3 |
| 5 | toad | 4 / 38 / 75 | 0.90 | 0.34 | 2.9 | 1 | 1 / 42 | 18 · 1.0 · -3 |
| 5 | raven | 4 / 38 / 75 | 1.30 | 0.50 | 2.0 | 1 | 1 / 42 | 17 · 1.0 · -3 |
| 5 | hare | 4 / 46 / 81 | 1.60 | 0.73 | 1.4 | 1 | 1 / 42 | 18 · 1.0 · -2 |
| 5 | bat | 4 / 31 / 69 | 2.00 | 0.62 | 1.6 | 1 | 1 / 42 | 18 · 1.0 · -4 |
| 10 | cat | 10 / 62 / 85 | 1.30 | 0.80 | 1.2 | 1 | 1 / 82 | 29 · 1.0 · -1 |
| 10 | toad | 7 / 54 / 81 | 0.90 | 0.49 | 2.1 | 1 | 1 / 82 | 29 · 1.0 · -2 |
| 10 | raven | 7 / 54 / 81 | 1.30 | 0.70 | 1.4 | 1 | 1 / 82 | 28 · 1.0 · -2 |
| 10 | hare | 10 / 62 / 85 | 1.60 | 0.99 | 1.0 | 1 | 1 / 82 | 29 · 1.0 · -1 |
| 10 | bat | 7 / 54 / 81 | 2.00 | 1.08 | 0.9 | 1 | 1 / 82 | 29 · 1.0 · -2 |
| 15 | cat | 15 / 62 / 85 | 1.30 | 0.80 | 1.2 | 1 | 1 / 118 | 40 · 2.0 · +0 |
| 15 | toad | 15 / 62 / 85 | 0.90 | 0.56 | 1.8 | 1 | 1 / 118 | 40 · 2.0 · +0 |
| 15 | raven | 15 / 62 / 85 | 1.30 | 0.80 | 1.2 | 1 | 1 / 118 | 39 · 3.0 · +0 |
| 15 | hare | 19 / 69 / 90 | 1.60 | 1.10 | 0.9 | 1 | 1 / 118 | 40 · 3.0 · +1 |
| 15 | bat | 10 / 54 / 81 | 2.00 | 1.08 | 0.9 | 1 | 1 / 118 | 40 · 2.0 · -1 |
| 20 | cat | 19 / 69 / 81 | 1.30 | 0.89 | 1.1 | 1 | 1 / 157 | 51 · 4.0 · +2 |
| 20 | toad | 15 / 62 / 75 | 0.90 | 0.56 | 1.8 | 1 | 1 / 157 | 51 · 3.0 · +1 |
| 20 | raven | 15 / 62 / 75 | 1.30 | 0.80 | 1.2 | 1 | 1 / 157 | 50 · 4.0 · +1 |
| 20 | hare | 19 / 69 / 81 | 1.60 | 1.10 | 0.9 | 1 | 1 / 157 | 51 · 4.0 · +2 |
| 20 | bat | 15 / 62 / 75 | 2.00 | 1.24 | 0.8 | 1 | 1 / 157 | 51 · 4.0 · +1 |
| 30 | cat | 15 / 81 / 93 | 1.30 | 1.05 | 1.0 | 1 | 1 / 230 | 73 · 7.5 · +6 |
| 30 | toad | 10 / 75 / 90 | 0.90 | 0.68 | 1.5 | 1 | 1 / 230 | 73 · 6.5 · +5 |
| 30 | raven | 10 / 75 / 90 | 1.30 | 0.97 | 1.0 | 1 | 1 / 230 | 72 · 8.0 · +5 |
| 30 | hare | 15 / 81 / 93 | 1.60 | 1.29 | 0.8 | 1 | 1 / 230 | 73 · 7.5 · +6 |
| 30 | bat | 10 / 75 / 90 | 2.00 | 1.50 | 0.7 | 1 | 1 / 230 | 73 · 8.0 · +5 |
| 40 | cat | 54 / 85 / 98 | 1.30 | 1.11 | 0.9 | 1 | 1 / 304 | 95 · 12.0 · +9 |
| 40 | toad | 46 / 81 / 96 | 0.90 | 0.72 | 1.4 | 1 | 1 / 304 | 95 · 11.0 · +8 |
| 40 | raven | 46 / 81 / 96 | 1.30 | 1.05 | 1.0 | 1 | 1 / 304 | 94 · 13.0 · +8 |
| 40 | hare | 54 / 85 / 98 | 1.60 | 1.37 | 0.7 | 1 | 1 / 304 | 95 · 12.0 · +9 |
| 40 | bat | 46 / 81 / 96 | 2.00 | 1.61 | 0.6 | 1 | 1 / 304 | 95 · 13.0 · +8 |
| 50 | cat | 62 / 93 / 99 | 1.30 | 1.21 | 0.8 | 1 | 1 / 377 | 117 · 16.5 · +13 |
| 50 | toad | 54 / 90 / 99 | 0.90 | 0.81 | 1.2 | 1 | 1 / 377 | 117 · 15.5 · +12 |
| 50 | raven | 54 / 90 / 99 | 1.30 | 1.16 | 0.9 | 1 | 1 / 377 | 116 · 18.0 · +12 |
| 50 | hare | 62 / 93 / 99 | 1.60 | 1.49 | 0.7 | 1 | 1 / 377 | 117 · 16.5 · +13 |
| 50 | bat | 54 / 90 / 99 | 2.00 | 1.79 | 0.6 | 1 | 1 / 377 | 117 · 18.0 · +12 |
| 60 | cat | 54 / 93 / 99 | 1.30 | 1.21 | 0.8 | 1 | 1 / 450 | 139 · 18.5 · +16 |
| 60 | toad | 46 / 90 / 99 | 0.90 | 0.81 | 1.2 | 1 | 1 / 450 | 139 · 17.5 · +15 |
| 60 | raven | 46 / 90 / 99 | 1.30 | 1.16 | 0.9 | 1 | 1 / 450 | 138 · 20.0 · +15 |
| 60 | hare | 54 / 93 / 99 | 1.60 | 1.49 | 0.7 | 1 | 1 / 450 | 139 · 18.5 · +16 |
| 60 | bat | 46 / 90 / 99 | 2.00 | 1.79 | 0.6 | 1 | 1 / 450 | 139 · 20.0 · +15 |

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
| warrior | St | 14.8/16.7 | 17.5 | 20.3 | 23.0 | 25.2 | 28.2 | +13.3 | 11.5 |
|  | Dx | 10.8/16.3 | 12.3 | 16.8 | 19.3 | 20.5 | 22.5 | +11.7 | 6.2 |
|  | To | 11.7/16.0 | 14.0 | 18.0 | 19.7 | 20.8 | 23.2 | +11.5 | 7.2 |
|  | In | 9.8/17.0 | 11.5 | 13.0 | 14.7 | 15.8 | 17.8 | +8.0 | 0.8 |
|  | Wi | 11.7/17.7 | 11.7 | 13.0 | 14.3 | 15.7 | 18.3 | +6.7 | 0.7 |
|  | Pr | 10.0/16.2 | 10.0 | 10.5 | 11.5 | 12.2 | 13.8 | +3.8 | 0.0 |
| | all six | | | | | | | **+55.0** | |
| thief | St | 11.0/16.8 | 16.5 | 20.5 | 22.8 | 24.5 | 27.3 | +16.3 | 10.5 |
|  | Dx | 12.5/16.5 | 15.7 | 19.7 | 21.3 | 22.7 | 24.2 | +11.7 | 7.7 |
|  | To | 10.5/16.8 | 14.3 | 18.7 | 19.8 | 21.3 | 23.7 | +13.2 | 6.8 |
|  | In | 11.3/17.2 | 12.7 | 14.3 | 16.0 | 17.3 | 18.3 | +7.0 | 1.2 |
|  | Wi | 10.3/17.3 | 10.3 | 11.8 | 13.2 | 14.8 | 17.2 | +6.8 | 0.2 |
|  | Pr | 9.8/16.3 | 9.8 | 10.8 | 11.3 | 12.5 | 13.8 | +4.0 | 0.0 |
| | all six | | | | | | | **+59.0** | |
| wizard | St | 10.0/16.7 | 10.2 | 10.2 | 10.3 | 11.2 | 12.0 | +2.0 | 0.0 |
|  | Dx | 10.0/16.3 | 10.8 | 12.3 | 14.2 | 16.2 | 18.3 | +8.3 | 2.0 |
|  | To | 9.5/17.3 | 11.5 | 17.2 | 19.2 | 20.3 | 22.3 | +12.8 | 5.0 |
|  | In | 14.5/16.8 | 16.0 | 18.8 | 19.8 | 21.0 | 22.8 | +8.3 | 6.0 |
|  | Wi | 12.5/17.7 | 16.2 | 20.2 | 22.3 | 23.7 | 26.0 | +13.5 | 8.3 |
|  | Pr | 9.5/16.0 | 9.5 | 10.5 | 10.8 | 12.0 | 13.8 | +4.3 | 0.0 |
| | all six | | | | | | | **+49.3** | |
| priest | St | 10.8/16.3 | 15.5 | 19.7 | 22.3 | 24.7 | 27.0 | +16.2 | 10.7 |
|  | Dx | 8.7/16.3 | 9.8 | 13.0 | 15.5 | 17.7 | 19.2 | +10.5 | 2.8 |
|  | To | 10.7/16.7 | 15.3 | 19.5 | 21.5 | 23.3 | 26.5 | +15.8 | 9.8 |
|  | In | 11.3/16.7 | 12.8 | 14.5 | 16.0 | 16.8 | 17.8 | +6.5 | 1.2 |
|  | Wi | 15.3/18.0 | 16.3 | 19.3 | 20.3 | 21.3 | 22.8 | +7.5 | 4.8 |
|  | Pr | 10.8/16.3 | 10.8 | 11.2 | 12.0 | 13.0 | 14.7 | +3.8 | 0.0 |
| | all six | | | | | | | **+60.3** | |

Marks a day the hooks gave in each stat over the level before (St/Dx/To/In/Wi/Pr, the seeds' mean), and the game day reached.

| L | Day | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| 5 | 4 | 23/8/16/20/3/3 | 49/36/40/21/4/2 | 0/12/20/35/44/3 | 32/8/53/21/14/2 |
| 10 | 14 | 75/21/35/6/3/2 | 103/69/48/5/3/1 | 0/4/25/13/64/2 | 71/11/65/6/24/1 |
| 15 | 24 | 132/39/61/6/3/2 | 170/113/51/6/3/2 | 1/4/37/41/95/2 | 110/12/103/6/33/2 |
| 20 | 39 | 209/31/48/4/3/1 | 145/102/40/4/3/1 | 0/4/27/34/72/1 | 106/17/86/4/24/1 |
| 30 | 69 | 326/62/74/4/3/1 | 232/117/59/4/3/1 | 0/5/39/49/118/2 | 170/14/156/4/30/1 |
| 40 | 109 | 322/47/78/2/3/1 | 242/88/61/2/3/1 | 0/6/30/40/98/1 | 293/12/161/2/25/1 |
| 50 | 149 | 428/61/114/2/3/1 | 342/92/104/2/3/1 | 0/6/41/58/142/1 | 200/10/209/2/27/1 |
| 60 | 189 | 313/60/154/2/3/1 | 356/89/133/2/3/1 | 0/7/40/54/133/1 | 205/11/293/2/29/1 |

The game day of each stat's first rise (the seeds' mean of those that rose by level 60; how many of the six in brackets when not all).

| Class | St | Dx | To | In | Wi | Pr |
|---|---|---|---|---|---|---|
| warrior | 7 | 11 | 7 | 6 | 24 | 36 |
| thief | 4 | 5 | 6 | 6 | 18 | 29 |
| wizard | 71 | 10 | 7 | 6 | 5 | 29 |
| priest | 6 | 10 | 4 | 5 | 10 | 44 |

The knobs (design §6.2): mean gain by level 60 with other `price_per_point` and `ramp_divisor` values (St/Dx/To/In/Wi/Pr · all six), the data's in bold.

| Price | Ramp ÷ | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| **20 + 10s** | **2** | +13.3/+11.7/+11.5/+8.0/+6.7/+3.8 · +55 | +16.3/+11.7/+13.2/+7.0/+6.8/+4.0 · +59 | +2.0/+8.3/+12.8/+8.3/+13.5/+4.3 · +49 | +16.2/+10.5/+15.8/+6.5/+7.5/+3.8 · +60 |
| 20 + 10s | 3 | +14.5/+12.7/+12.7/+7.8/+6.5/+4.0 · +58 | +17.7/+12.8/+14.2/+7.2/+6.8/+4.5 · +63 | +2.2/+8.8/+13.5/+9.7/+14.5/+4.2 · +53 | +16.8/+10.7/+16.7/+6.7/+8.2/+3.7 · +63 |
| 20 + 10s | 4 | +15.5/+12.8/+13.3/+8.0/+6.3/+4.3 · +60 | +18.5/+13.2/+14.8/+7.0/+7.0/+4.0 · +64 | +2.0/+8.5/+14.3/+9.8/+15.3/+4.5 · +54 | +17.3/+11.0/+17.5/+6.8/+8.7/+3.7 · +65 |
| 20 + 10s | 6 | +16.5/+14.0/+15.0/+8.0/+6.7/+3.8 · +64 | +20.2/+14.8/+15.8/+7.3/+7.3/+4.2 · +70 | +2.0/+9.3/+15.2/+10.8/+16.2/+4.5 · +58 | +18.7/+11.7/+18.8/+7.3/+9.7/+4.2 · +70 |
| 20 + 15s | 2 | +12.0/+11.0/+11.0/+6.7/+5.3/+3.3 · +49 | +15.5/+10.7/+12.2/+6.0/+5.8/+3.3 · +54 | +1.7/+7.3/+12.2/+7.8/+12.2/+3.2 · +44 | +15.2/+9.3/+14.2/+5.8/+6.8/+3.2 · +54 |
| 20 + 15s | 4 | +14.2/+12.2/+12.3/+6.7/+5.5/+2.8 · +54 | +17.7/+12.5/+14.0/+6.2/+6.2/+3.3 · +60 | +1.5/+7.8/+13.3/+8.7/+14.5/+3.5 · +49 | +16.7/+10.0/+16.3/+6.2/+7.8/+3.2 · +60 |
| 20 + 20s | 4 | +13.3/+11.0/+11.5/+5.7/+4.2/+2.5 · +48 | +16.5/+11.3/+13.2/+5.5/+4.8/+2.7 · +54 | +1.2/+7.0/+12.5/+8.3/+13.2/+2.7 · +45 |

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

## Traps

Not in the probe: traps are not fights. Their damage grows with the danger of the level they lie
on (#1165, `[trap.grows]` in `data/traps.toml`, the wiki's trap damage table). The base, at
danger 0 to 2, is held under the fewest hit points a level-1 character starts with: 6 in the test's draws (a kobold
wizard, over every race and class), against the worst base going off of 4 (a fireball or a
lightning bolt at 1d4, an arrow's or a spear's 1d2 doubled by a critical).
`world/trap_rung_tests.rs::the_lowest_rung_never_kills_a_fresh_character` checks it, so a race,
class, or talent that lowers starting hit points, or a stronger base, fails there first. The old
dice (the fireball's 3d6, the lightning's 4d6) come at danger 10, and twice or more of them at
30.

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
