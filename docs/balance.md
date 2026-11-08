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
variance pass (#1134: "Spread"), and again after #1169 (armed monsters aim by their weapon
skill ranks: "Armed monsters" under "Monsters"), and again after #1188 (weapon skill defaults
removed: a rank raises no other skill, so starting characters and hand-outs lose their derived
ranks), and again after #1200 (the starters: levels 1 to 5 cut in hit points and riders,
"Starters"; the stat training of the first levels moves a little with the shorter fights).

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
| 1 | +3/8.2/14/3 | +3/8.2/14/3 | +1/3.2/13/1 | +1/3.2/13/1 | +0/3.0/12/0 | +1/5.8/11/0 | +1/5.8/11/0 |
| 5 | +4/8.3/16/4 | +5/10.2/18/4 | +3/4.5/15/1 | +3/5.7/17/2 | +0/3.0/13/0 | +2/6.7/14/2 | +3/7.6/16/2 |
| 10 | +6/11.2/20/5 | +7/12.6/23/6 | +6/7.7/20/1 | +8/9.2/22/2 | +2/4.0/15/0 | +6/10.2/15/4 | +7/11.9/18/5 |
| 15 | +7/12.3/20/9 | +12/15.7/24/10 | +9/8.8/22/2 | +12/11.0/25/4 | +2/4.0/15/1 | +8/12.5/18/5 | +10/16.7/22/5 |
| 20 | +9/14.3/22/10 | +13/20.2/27/12 | +11/10.8/24/2 | +14/11.9/28/4 | +2/4.0/16/1 | +8/13.0/17/7 | +11/15.1/23/8 |
| 30 | +11/16.5/26/12 | +21/24.2/34/18 | +14/12.8/25/2 | +20/21.0/31/8 | +3/5.5/17/1 | +11/15.5/19/8 | +19/21.9/26/15 |
| 40 | +14/19.5/28/12 | +23/26.8/37/16 | +16/15.2/26/2 | +24/19.7/34/5 | +4/7.3/19/1 | +12/18.7/20/8 | +22/25.8/29/12 |
| 50 | +15/22.2/30/12 | +20/27.8/40/17 | +17/16.3/26/2 | +23/23.5/33/7 | +4/7.5/21/1 | +14/21.2/21/8 | +20/29.1/28/13 |
| 60 | +16/22.7/31/12 | +22/29.2/40/15 | +18/16.8/26/2 | +24/22.6/34/7 | +4/7.5/22/1 | +14/21.2/22/8 | +21/26.3/31/13 |

Stats, hit points, and mana (plain gear; trained → untrained). Scores St/Dx/To/In/Wi/Pr, the seeds' mean. Spellcraft (learning): the trained Spellcraft and the learning it gives; the auto-rolls spend every level-up roll on the first trained skill (Concentration for both), so it stays where it starts.

| L | Warrior | Thief | Wizard | Priest | HP W/T/Wz/P | MP Wz/P | Spellcraft (learning) Wz/P | Blows a round W/T |
|---|---|---|---|---|---|---|---|---|
| 1 | 15/11/12/10/12/10 | 11/12/10/11/10/10 | 10/10/10/14/12/10 | 11/9/11/11/15/11 | 23→23 / 19→19 / 13→13 / 18→18 | 16/13 | 20 (4)/22 (2) | 0.99/0.99 |
| 5 | 15/11/12/10/12/10 | 12/13/11/12/10/10 | 10/10/10/15/13/10 | 11/9/11/12/15/11 | 37→37 / 27→26 / 17→17 / 27→26 | 51/39 | 20 (4)/22 (2) | 0.82/0.96 |
| 10 | 17/12/14/12/12/10 | 16/16/15/13/10/10 | 10/11/12/16/16/10 | 16/10/15/12/17/11 | 61→56 / 48→37 / 25→22 / 49→38 | 99/78 | 20 (4)/22 (2) | 0.83/1.04 |
| 15 | 19/15/16/12/12/10 | 19/18/18/13/11/10 | 10/12/15/18/19/10 | 18/11/18/13/18/11 | 92→73 / 75→47 / 36→28 / 79→48 | 152/120 | 20 (4)/22 (2) | 0.84/1.10 |
| 20 | 20/17/18/13/13/11 | 20/20/19/14/12/11 | 10/12/18/19/20/10 | 20/13/20/14/19/12 | 128→92 / 107→58 / 55→33 / 113→58 | 206/163 | 20 (4)/22 (2) | 0.85/1.17 |
| 30 | 23/19/20/14/14/11 | 23/21/20/16/13/11 | 10/14/20/20/22/11 | 22/15/22/16/20/12 | 203→128 / 174→79 / 102→43 / 186→79 | 328/253 | 20 (4)/22 (2) | 0.87/1.30 |
| 40 | 25/20/21/16/16/12 | 24/22/22/17/14/12 | 11/16/21/21/24/12 | 24/18/24/17/21/13 | 286→164 / 250→100 / 154→53 / 270→101 | 454/346 | 20 (4)/22 (2) | 0.88/1.49 |
| 50 | 27/22/23/17/18/13 | 26/23/23/18/16/13 | 12/17/22/22/24/13 | 26/18/25/18/22/14 | 376→200 / 331→120 / 209→63 / 362→122 | 585/445 | 20 (4)/22 (2) | 0.88/1.49 |
| 60 | 28/23/24/18/18/14 | 27/24/24/18/17/14 | 12/18/22/23/26/14 | 26/19/27/18/23/14 | 380→203 / 335→122 / 213→64 / 367→124 | 724/546 | 20 (4)/22 (2) | 0.88/1.49 |

The Defence of the warrior and the thief in plain gear by category (trained → untrained), the seeds' mean.

| Class | L | Dexterity | Armour | Shield | Parry | Natural | Dodge | Deflection | Wall | total |
|---|---|---|---|---|---|---|---|---|---|---|
| warrior | 10 | 1→0 | 2→2 | 4→4 | 2→2 | 0→0 | 1→1 | 0→0 | 0→0 | 20→19 |
| warrior | 30 | 2→0 | 1→1 | 8→8 | 4→4 | 0→0 | 1→1 | 0→0 | 0→0 | 26→24 |
| warrior | 60 | 2→0 | 1→1 | 10→10 | 6→6 | 0→0 | 2→1 | 0→0 | 0→0 | 31→28 |
| thief | 10 | 2→1 | 2→2 | 0→0 | 4→2 | 0→0 | 2→2 | 0→0 | 0→0 | 20→17 |
| thief | 30 | 5→1 | 2→2 | 0→0 | 6→4 | 0→0 | 2→2 | 0→0 | 0→0 | 25→18 |
| thief | 60 | 5→1 | 2→2 | 0→0 | 8→4 | 0→0 | 2→2 | 0→0 | 0→0 | 26→19 |

## Monsters

Within two levels: the median (range) bare → the mean with gear. Damage is a hit's mean before soak.

| L (±2) | n | HP | to-hit | dmg/blow | Dv | Pv | blows a round |
|---|---|---|---|---|---|---|---|
| 1 | 57 | 7 | 0 (-5–3) → -0.4 | 2 (1–6) → 2.4 | 12 (5–14) → 11.1 | 0 (0–3) → 0.2 | 1.00 |
| 5 | 63 | 19 | 1 (-3–6) → 1.2 | 4 (1–13) → 4.5 | 12 (7–21) → 12.0 | 0 (0–2) → 0.4 | 1.00 |
| 10 | 56 | 38 | 4 (-1–9) → 4.2 | 8 (1–16) → 8.9 | 12 (7–19) → 12.7 | 0 (0–7) → 0.7 | 1.00 |
| 15 | 46 | 56 | 6 (2–10) → 6.3 | 12 (3–22) → 12.7 | 12 (8–19) → 13.5 | 0 (0–7) → 1.0 | 1.00 |
| 20 | 50 | 72 | 8 (2–12) → 7.6 | 16 (4–23) → 15.1 | 13 (8–19) → 14.4 | 1 (0–7) → 1.8 | 1.00 |
| 30 | 36 | 118 | 12 (6–19) → 11.7 | 22 (11–29) → 22.0 | 16 (12–23) → 15.7 | 2 (0–8) → 2.8 | 1.00 |
| 40 | 28 | 152 | 16 (8–20) → 14.4 | 26 (13–32) → 24.6 | 17 (14–23) → 18.1 | 3 (0–11) → 3.1 | 1.05 |
| 50 | 24 | 278 | 18 (10–30) → 17.0 | 32 (20–40) → 30.2 | 19 (16–25) → 18.9 | 5 (0–13) → 4.4 | 1.05 |
| 60 | 18 | 226 | 21 (12–33) → 20.4 | 32 (18–43) → 31.4 | 22 (15–30) → 21.6 | 6 (0–16) → 5.8 | 1.26 |
| 70 | 8 | 192 | 25 (15–34) → 25.6 | 33 (22–48) → 34.7 | 23 (15–29) → 23.2 | 6 (0–11) → 4.8 | 1.23 |

### Armed monsters (#1169)

Armed types have no flat to-hit: their weapon skill (and Duelist, one-handed with no shield) is
raised to the fewest ranks that bring the weapon to `L/4` (a caster's `L/4 − L/8`), none past
the trained warrior's rank of the level (`rank` in the probe; `docs/dev/creatures.md`
"Variance by role"). Against the run on `main` just before (same content otherwise), the geared
means rise by 0.4 to 1.3 to hit and 0.3 to 1.8 damage a blow from level 10 up (to-hit 3.9 →
4.3 at 10, 6.9 → 7.6 at 20, 11.1 → 11.9 at 30, 13.2 → 14.5 at 40, 19.6 → 20.6 at 60), and stay
below the bare medians at 20, 40, and 50 to 60: the armed types close on the natural attackers
of their level and do not pass them. The fights show it most from level 30, where most deep
types are armed: the warrior in plain gear is hit 33 → 48 % at 30, 34 → 61 % at 40, and 49 →
65 % at 60, and dies in 40 → 27, 67 → 30, and 43 → 22 rounds; the rounds to kill barely move.
No retune: the armed were the ones trailing, as #1169 says.

## Fights

Each cell: P(player hits) / P(monster hits) % · damage a round player : monster · rounds to kill / rounds to die; the median over the monsters within two levels (trained stats, monsters with gear).

| L | Warrior kit | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain (melee) | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|---|
| 1 | 81/42 · 7.5:0.2 · 1.0 / 441 | 81/42 · 7.4:0.2 · 1.0 / 435 | 81/42 · 7.4:0.2 · 1.0 / 439 | 66/55 · 2.9:0.6 · 3.2 / 39 | 66/55 · 2.9:0.6 · 3.0 / 39 | 65/62 · 2.2:1.1 · 3.8 / 14 | 71/66 · 4.4:1.2 · 1.8 / 17 | 71/66 · 4.4:1.2 · 1.8 / 17 |
| 5 | 86/47 · 7.6:1.3 · 2.8 / 33 | 86/40 · 6.2:1.0 · 3.5 / 48 | 89/25 · 8.0:0.6 · 2.7 / 107 | 79/45 · 4.4:2.0 · 5.3 / 16 | 80/32 · 5.5:1.2 · 4.1 / 41 | 65/62 · 1.8:3.1 · 14 / 6.0 | 77/53 · 4.1:1.9 · 5.1 / 16 | 81/37 · 5.2:1.2 · 4.2 / 30 |
| 10 | 89/42 · 10.9:2.9 · 4.0 / 23 | 89/35 · 8.9:2.0 · 4.9 / 36 | 92/19 · 10.1:1.0 · 4.3 / 149 | 89/35 · 10.7:3.0 · 5.0 / 22 | 92/25 · 13.5:2.2 · 4.1 / 47 | 65/70 · 2.3:6.0 · 22 / 4.4 | 88/66 · 7.5:4.3 · 5.9 / 13 | 94/45 · 9.8:2.7 · 4.3 / 29 |
| 15 | 93/45 · 12.2:4.6 · 5.4 / 23 | 93/48 · 9.8:3.5 · 6.6 / 30 | 98/31 · 14.6:2.0 · 4.4 / 190 | 97/38 · 14.0:4.3 · 6.0 / 22 | 99/19 · 19.0:2.1 · 5.1 / 55 | 66/81 · 2.0:9.9 · 34 / 3.7 | 96/64 · 10.0:6.0 · 6.4 / 15 | 98/36 · 15.0:3.4 · 4.3 / 108 |
| 20 | 96/37 · 15.1:5.0 · 5.8 / 28 | 96/46 · 12.4:3.9 · 7.1 / 37 | 99/16 · 21.3:1.2 · 4.1 / 270 | 99/36 · 18.8:5.1 · 5.5 / 35 | 99/17 · 22.0:2.4 · 4.6 / 111 | 59/85 · 1.4:13.0 · 58 / 4.5 | 95/77 · 8.9:8.4 · 9.5 / 14 | 97/43 · 12.3:4.1 · 7.0 / 58 |
| 30 | 97/34 · 18.2:6.9 · 8.6 / 30 | 97/44 · 14.8:6.7 · 11 / 32 | 99/6 · 27.3:0.8 · 6.4 / 440 | 99/51 · 25.1:11.0 · 7.5 / 18 | 99/16 · 51.1:3.0 · 3.4 / 133 | 59/93 · 2.0:21.4 · 73 / 4.8 | 96/87 · 10.9:15.7 · 13 / 12 | 99/43 · 24.7:5.4 · 6.2 / 48 |
| 40 | 97/40 · 21.4:8.8 · 8.7 / 38 | 97/54 · 17.2:8.6 · 11 / 35 | 99/7 · 35.7:0.9 · 5.7 / 489 | 99/63 · 33.1:15.6 · 6.2 / 18 | 99/22 · 59.7:4.6 · 3.7 / 133 | 46/95 · 2.1:24.2 · 99 / 6.4 | 93/94 · 14.5:19.5 · 13 / 14 | 99/46 · 28.4:7.5 · 6.0 / 51 |
| 50 | 98/39 · 27.1:12.3 · 10 / 32 | 98/56 · 21.5:14.8 · 13 / 26 | 99/8 · 34.0:1.5 · 8.2 / 459 | 98/74 · 35.4:23.0 · 8.2 / 15 | 99/42 · 52.1:12.0 · 5.5 / 112 | 42/97 · 1.8:29.4 · 159 / 7.1 | 95/97 · 16.9:27.6 · 15 / 13 | 99/63 · 30.4:15.5 · 9.3 / 37 |
| 60 | 90/35 · 23.8:10.6 · 10 / 47 | 90/56 · 19.0:14.8 · 13 / 30 | 98/13 · 34.8:2.8 · 7.9 / 365 | 95/76 · 30.4:27.4 · 8.8 / 14 | 99/36 · 45.7:11.0 · 5.7 / 95 | 28/97 · 0.9:35.2 · 335 / 6.1 | 83/97 · 13.6:31.0 · 18 / 12 | 98/54 · 24.5:13.9 · 9.9 / 39 |

Out of depth (+3: the monsters one to five levels above), and untrained stats, for the warrior in plain gear.

| L | Warrior plain +3 | Warrior magic +3 | Warrior plain, untrained | Warrior magic, untrained |
|---|---|---|---|---|
| 1 | 81/50 · 7.2:0.9 · 2.1 / 32 | 81/50 · 7.2:0.9 · 2.1 / 31 | 81/42 · 7.4:0.2 · 1.0 / 438 | 81/42 · 7.5:0.2 · 1.0 / 438 |
| 5 | 85/54 · 6.1:2.5 · 5.2 / 17 | 88/37 · 7.9:1.7 · 4.1 / 30 | 86/40 · 6.2:0.9 · 3.4 / 49 | 89/25 · 8.1:0.6 · 2.7 / 100 |
| 10 | 90/48 · 8.2:3.9 · 7.2 / 17 | 92/27 · 9.7:2.0 · 6.2 / 77 | 84/40 · 7.4:2.3 · 6.2 / 28 | 88/22 · 8.5:1.1 · 5.3 / 138 |
| 15 | 91/53 · 9.4:4.6 · 8.1 / 21 | 98/34 · 14.2:2.8 · 5.4 / 156 | 86/54 · 7.1:3.8 · 10 / 21 | 97/33 · 12.0:2.1 · 5.5 / 187 |
| 20 | 96/54 · 11.8:5.6 · 10 / 24 | 98/21 · 20.6:2.1 · 6.1 / 120 | 87/47 · 7.9:4.1 · 13 / 26 | 96/14 · 16.6:1.1 · 6.2 / 192 |
| 30 | 97/47 · 13.4:8.4 · 12 / 27 | 99/7 · 26.5:0.7 · 7.0 / 481 | 85/56 · 8.0:8.7 · 19 / 15 | 99/9 · 21.8:0.9 · 8.4 / 308 |
| 40 | 97/50 · 18.0:9.8 · 12 / 31 | 99/6 · 36.5:1.0 · 6.0 / 470 | 77/62 · 8.9:10.5 · 22 / 16 | 99/6 · 24.5:0.8 · 8.2 / 331 |
| 50 | 95/53 · 19.6:12.4 · 13 / 34 | 99/7 · 32.2:1.3 · 8.5 / 476 | 76/68 · 9.7:17.7 · 29 / 12 | 93/10 · 21.3:2.1 · 13 / 218 |
| 60 | 90/69 · 16.1:20.7 · 16 / 18 | 98/13 · 26.0:6.3 · 9.6 / 134 | 58/73 · 6.1:20.3 · 33 / 11 | 85/20 · 17.7:4.0 · 14 / 175 |

## Spread

Rounds to kill and to die over the monsters within two levels, trained stats in plain gear: the median (fastest tenth–slowest tenth) ×the tenths over the median. The target: ×0.5 and ×2.

| L | n | Warrior kills | Warrior dies | Thief dies | Wizard dies | Priest dies |
|---|---|---|---|---|---|---|
| 1 | 57 | 1.0 (0.6–1.7) ×0.54/×1.6 | 433 (21–>999) ×0.05/×5.3 | 40 (8.0–>999) ×0.20/×46.3 | 14 (4.5–485) ×0.33/×35.4 | 18 (5.9–655) ×0.33/×36.8 |
| 5 | 63 | 3.5 (1.6–6.5) ×0.47/×1.9 | 48 (14–>999) ×0.29/×49.3 | 16 (6.6–264) ×0.41/×16.4 | 5.9 (2.8–34) ×0.48/×5.7 | 16 (6.6–445) ×0.41/×27.3 |
| 10 | 56 | 4.8 (2.8–9.9) ×0.58/×2.0 | 38 (12–340) ×0.30/×9.0 | 22 (7.7–103) ×0.34/×4.6 | 4.3 (2.2–11) ×0.51/×2.6 | 13 (5.7–65) ×0.44/×5.0 |
| 15 | 46 | 6.7 (3.4–11) ×0.51/×1.6 | 31 (14–67) ×0.47/×2.2 | 21 (12–59) ×0.58/×2.9 | 3.7 (2.5–5.7) ×0.68/×1.6 | 15 (7.8–26) ×0.53/×1.8 |
| 20 | 50 | 7.3 (3.3–12) ×0.45/×1.7 | 36 (11–113) ×0.31/×3.1 | 33 (9.9–107) ×0.29/×3.2 | 4.5 (2.4–6.7) ×0.53/×1.5 | 14 (6.2–30) ×0.43/×2.1 |
| 30 | 36 | 11 (3.9–15) ×0.37/×1.4 | 31 (11–155) ×0.34/×5.0 | 18 (6.1–82) ×0.34/×4.5 | 4.7 (2.8–8.1) ×0.59/×1.7 | 12 (6.5–23) ×0.55/×1.9 |
| 40 | 28 | 11 (4.2–15) ×0.39/×1.4 | 34 (15–196) ×0.43/×5.7 | 19 (9.8–111) ×0.53/×6.0 | 6.4 (4.6–10) ×0.73/×1.6 | 14 (9.8–24) ×0.70/×1.7 |
| 50 | 24 | 13 (6.7–18) ×0.52/×1.4 | 26 (14–162) ×0.52/×6.1 | 15 (11–70) ×0.71/×4.6 | 7.1 (5.5–11) ×0.78/×1.6 | 13 (11–23) ×0.81/×1.7 |
| 60 | 18 | 13 (6.0–27) ×0.48/×2.1 | 30 (8.8–105) ×0.29/×3.5 | 14 (5.8–38) ×0.42/×2.7 | 6.1 (3.5–8.8) ×0.58/×1.5 | 12 (7.1–18) ×0.58/×1.5 |

What it says (#1134, the variance pass; the dread design §7.3): before the pass (main after
D0 to D8 and D11) the warrior's rounds to kill ran ×0.56 to ×0.78 of the median at the fastest
tenth and ×1.2 to ×1.6 at the slowest, every level: a row of the same fight. After it (to-hit
by role, threat 19 for the strikers and ×3 on 24 for the brutes, hit points 0.5 to 2.0 by role
with every brute in the top of its ±35 %, the speed rule) the fastest tenth is ×0.36 to ×0.63
from level 5 and the slowest ×1.7 to ×2.0 to level 20 and at 60; at 30 to 50 the slow tail
stays at ×1.3, where the windows are small (24 to 35 types) and hold few brutes (the giants
and most of the deep brutes there are armed or uniques, outside the role factors). Rounds to
die keep their wide spread (the thief's, the wizard's, and the priest's columns: ×0.2 to ×0.5
fast, ×1.3 to ×5 slow from level 10); a dread brute is a death, not a wall. The speed review (#1167) restored speed above 1250 to the air and lightning elementals, the storm, the hunt's and grave's hounds, the nightwing and sluagh, and the ghouls (the table above is after it: the fastest tenth stays ×0.35 to ×0.58 from level 5 and the slow tail at ×1.3 to ×2.1 but 5.0 for the thief's deaths). A further pass
for 30 to 50 is a content question (more unarmed brutes there), not an offset.

## Starters

The trained player in plain gear against the monsters of one level (not a window). Each cell: rounds to kill / rounds to die (the medians) · the share of those monsters that win alone / two at once (by the means).

| Player L | Monster L | n | Warrior | Thief | Wizard (melee) | Priest |
|---|---|---|---|---|---|---|
| 1 | 1 | 30 | 0.8 / >999 · 0 % / 0 % | 2.7 / 165 · 0 % / 7 % | 3.0 / 30 · 3 % / 40 % | 1.5 / 37 · 0 % / 3 % |
| 1 | 2 | 16 | 1.0 / 91 · 0 % / 0 % | 3.1 / 24 · 0 % / 31 % | 3.5 / 9.3 · 12 % / 50 % | 1.7 / 12 · 0 % / 31 % |
| 1 | 3 | 11 | 1.5 / 437 · 0 % / 0 % | 4.4 / 32 · 9 % / 36 % | 4.8 / 10 · 27 % / 64 % | 2.3 / 14 · 9 % / 27 % |
| 1 | 4 | 13 | 1.9 / 45 · 0 % / 0 % | 5.7 / 25 · 23 % / 46 % | 6.3 / 8.6 · 38 % / 85 % | 3.1 / 11 · 8 % / 38 % |
| 1 | 5 | 13 | 3.1 / 13 · 0 % / 38 % | 13 / 5.6 · 62 % / 77 % | 12 / 3.1 · 62 % / 100 % | 5.2 / 4.3 · 62 % / 62 % |
| 5 | 1 | 30 | 0.9 / >999 · 0 % / 0 % | 1.6 / 382 · 0 % / 0 % | 3.7 / 46 · 0 % / 27 % | 1.4 / >999 · 0 % / 0 % |
| 5 | 2 | 16 | 1.1 / 195 · 0 % / 0 % | 1.6 / 63 · 0 % / 0 % | 4.4 / 14 · 6 % / 38 % | 1.7 / 56 · 0 % / 0 % |
| 5 | 3 | 11 | 1.7 / 738 · 0 % / 0 % | 2.5 / 67 · 0 % / 9 % | 5.6 / 15 · 27 % / 45 % | 2.2 / 203 · 0 % / 9 % |
| 5 | 4 | 13 | 2.2 / 120 · 0 % / 0 % | 3.1 / 61 · 0 % / 15 % | 7.4 / 12 · 38 % / 69 % | 3.2 / 48 · 0 % / 8 % |
| 5 | 5 | 13 | 3.5 / 32 · 0 % / 0 % | 5.5 / 12 · 15 % / 62 % | 15 / 4.6 · 62 % / 100 % | 5.0 / 11 · 0 % / 62 % |

Before #1200 (the same table on `main` just before):

| Player L | Monster L | n | Warrior | Thief | Wizard (melee) | Priest |
|---|---|---|---|---|---|---|
| 1 | 1 | 29 | 1.5 / >999 · 0 % / 0 % | 4.2 / 165 · 0 % / 10 % | 5.1 / 36 · 10 % / 34 % | 2.5 / 42 · 0 % / 14 % |
| 1 | 2 | 16 | 1.9 / 136 · 0 % / 0 % | 6.0 / 29 · 6 % / 38 % | 6.9 / 10 · 31 % / 62 % | 3.5 / 14 · 6 % / 31 % |
| 1 | 3 | 11 | 2.1 / 431 · 0 % / 0 % | 6.7 / 36 · 27 % / 36 % | 6.7 / 10 · 27 % / 82 % | 3.2 / 13 · 27 % / 36 % |
| 1 | 4 | 13 | 2.4 / 48 · 0 % / 0 % | 7.2 / 26 · 31 % / 54 % | 8.0 / 8.8 · 38 % / 85 % | 3.9 / 11 · 15 % / 38 % |
| 1 | 5 | 13 | 3.4 / 14 · 0 % / 54 % | 14 / 6.6 · 62 % / 77 % | 13 / 3.8 · 69 % / 100 % | 5.7 / 5.0 · 62 % / 62 % |
| 5 | 1 | 29 | 1.5 / >999 · 0 % / 0 % | 2.4 / 411 · 0 % / 0 % | 6.0 / 49 · 7 % / 28 % | 2.4 / >999 · 0 % / 0 % |
| 5 | 2 | 16 | 1.8 / 378 · 0 % / 0 % | 3.0 / 73 · 0 % / 6 % | 8.2 / 16 · 31 % / 56 % | 3.0 / 87 · 0 % / 6 % |
| 5 | 3 | 11 | 2.2 / 730 · 0 % / 0 % | 3.8 / 90 · 0 % / 27 % | 7.8 / 15 · 27 % / 73 % | 3.0 / 181 · 0 % / 27 % |
| 5 | 4 | 13 | 2.4 / 122 · 0 % / 0 % | 3.8 / 67 · 0 % / 15 % | 9.3 / 13 · 38 % / 85 % | 3.7 / 49 · 0 % / 8 % |
| 5 | 5 | 13 | 3.3 / 37 · 0 % / 0 % | 6.5 / 15 · 15 % / 62 % | 16 / 5.3 · 62 % / 100 % | 5.1 / 14 · 0 % / 62 % |

What it says (#1200, the starters: level 1 to 5 at 40 % to 90 % of the roster's hit points, their
riders at 1 % to 5 %, `content/starter.rs`): a level-1 player now kills a level-1 monster in 0.7
(warrior) to 2.7 (thief) rounds, against 1.5 to 5.1 before; the wizard fighting with the staff
loses to one level-1 monster in 3 % of them (10 % before) and to two at once in 31 % (34 %); the
priest to two in 3 % (14 %). The rounds to die do not move: no damage or to-hit was trimmed (the
monsters' blows were already small, 2.4 a hit at level 1). Level 5 is cut little (3.3 → 3.2
rounds for the warrior against level 5). The riders do not show here: the probe counts no
statuses. The beasts the player keeps are of these types too: the ranger's starting hound or boar
has 9 hit points at level 1 (17 before) and 8 fewer at every level after ("Ranger and companion"),
and the druid's wolf 13 for 16 ("Summons").

## Dragons

Each dragon (#1099) against the players of the probe's levels around its own, in plain gear, trained → untrained (the seeds' mean). **HP** the player's; **breath** its mean after the player's resistance, and that over their hit points; **melee** the claws and bite of one action, misses included; **a round** the dragon's damage an action over a cooldown's cycle (one breath, then a melee action for each degree of its cooldown); **die** the dragon's actions until the player falls, breathing first; **kill** the player's rounds to kill it.

| Dragon | L | Class | HP | breath (of HP) | melee | a round | die | kill |
|---|---|---|---|---|---|---|---|---|
| gorgon | 20 | warrior | 128→92 | 14 (11%→15%) | 13.4→13.6 | 13.4→13.6 | 9.5→6.7 | 12→19 |
| gorgon | 20 | thief | 107→58 | 14 (13%→24%) | 16.0→18.3 | 15.9→17.9 | 6.8→3.4 | 8.7→26 |
| gorgon | 20 | wizard | 55→33 | 14 (25%→43%) | 19.1→19.0 | 18.6→18.5 | 3.2→2.0 | 166→172 |
| gorgon | 20 | priest | 113→58 | 14 (12%→24%) | 15.9→15.7 | 15.7→15.5 | 7.2→3.8 | 17→38 |
| gorgon | 30 | warrior | 203→128 | 14 (7%→11%) | 10.6→11.3 | 10.9→11.6 | 18→11 | 9.6→16 |
| gorgon | 30 | thief | 174→79 | 14 (8%→18%) | 15.1→17.9 | 15.0→17.6 | 12→4.6 | 6.5→14 |
| gorgon | 30 | wizard | 102→43 | 14 (14%→33%) | 18.8→18.8 | 18.4→18.4 | 5.7→2.5 | 78→113 |
| gorgon | 30 | priest | 186→79 | 14 (8%→18%) | 15.1→15.0 | 15.0→14.9 | 13→5.4 | 13→28 |
| fire drake | 15 | warrior | 92→73 | 43 (47%→59%) | 25.3→26.5 | 27.9→28.9 | 2.9→2.1 | 13→20 |
| fire drake | 15 | thief | 75→47 | 43 (57%→92%) | 30.5→43.8 | 32.3→43.7 | 2.1→1.1 | 10→36 |
| fire drake | 15 | wizard | 36→28 | 43 (121%→157%) | 50.3→51.5 | 49.3→50.3 | 0.8→0.6 | 121→144 |
| fire drake | 15 | priest | 79→48 | 43 (55%→91%) | 36.5→38.6 | 37.5→39.2 | 2.0→1.1 | 14→31 |
| fire drake | 20 | warrior | 128→92 | 43 (34%→47%) | 20.2→20.4 | 23.4→23.7 | 5.2→3.4 | 9.4→16 |
| fire drake | 20 | thief | 107→58 | 43 (40%→74%) | 24.1→42.2 | 26.8→42.3 | 4.2→1.3 | 7.1→20 |
| fire drake | 20 | wizard | 55→33 | 43 (78%→132%) | 49.8→51.5 | 48.8→50.3 | 1.2→0.8 | 121→127 |
| fire drake | 20 | priest | 113→58 | 43 (38%→74%) | 33.5→36.1 | 34.9→37.1 | 3.1→1.4 | 14→31 |
| Hrīmwyrm the White | 30 | warrior | 203→128 | 57 (28%→45%) | 25.7→30.4 | 30.2→34.2 | 6.6→3.3 | 14→26 |
| Hrīmwyrm the White | 30 | thief | 174→79 | 57 (33%→72%) | 39.7→64.0 | 42.2→63.0 | 4.3→1.3 | 9.8→24 |
| Hrīmwyrm the White | 30 | wizard | 102→43 | 50 (49%→117%) | 67.9→70.9 | 65.3→67.9 | 1.8→0.9 | 190→289 |
| Hrīmwyrm the White | 30 | priest | 186→79 | 57 (31%→72%) | 50.9→53.8 | 51.8→54.2 | 3.5→1.4 | 19→49 |
| Hrīmwyrm the White | 40 | warrior | 286→164 | 57 (20%→35%) | 19.2→22.8 | 24.6→27.6 | 11→5.6 | 10.0→18 |
| Hrīmwyrm the White | 40 | thief | 250→100 | 57 (23%→57%) | 34.5→61.3 | 37.7→60.7 | 7.0→1.7 | 6.2→16 |
| Hrīmwyrm the White | 40 | wizard | 154→53 | 50 (33%→94%) | 63.1→69.9 | 61.2→67.0 | 2.6→1.0 | 86→172 |
| Hrīmwyrm the White | 40 | priest | 270→101 | 57 (21%→57%) | 48.9→50.0 | 50.1→51.0 | 5.4→1.9 | 13→41 |
| Nīðdraca the Hate-Wyrm | 60 | warrior | 380→203 | 95 (25%→47%) | 141.2→140.4 | 133.4→132.8 | 3.0→1.8 | 32→97 |
| Nīðdraca the Hate-Wyrm | 60 | thief | 335→122 | 95 (28%→78%) | 164.1→165.7 | 152.6→153.8 | 2.5→1.2 | 38→398 |
| Nīðdraca the Hate-Wyrm | 60 | wizard | 213→64 | 95 (45%→149%) | 168.3→168.9 | 156.1→156.6 | 1.7→0.7 | >999→>999 |
| Nīðdraca the Hate-Wyrm | 60 | priest | 367→124 | 95 (26%→77%) | 151.1→151.4 | 141.7→142.0 | 2.8→1.2 | 72→815 |

## Casters

Each caster (#1102) against the trained players of the first probe level from its own, in plain gear (the seeds' mean). **Spell** its first damage spell and its roll at its stats; **casts** how many of it its full pool pays for; **a cast** its mean damage (an `attack` spell's times its hit chance); **a round** that times its casts a round, against **melee** its blows a round once the mana is spent; **die** the player's rounds until they fall, the spells first, then its melee.

| Caster | L | Spell (roll) | casts | Player | HP | a cast | a round | melee | die |
|---|---|---|---|---|---|---|---|---|---|
| arcanaloth | 46 | chain lightning (12d6+3) | 8 | warrior 50 | 376 | 45.0 | 49.5 | 5.6 | 11 |
| arcanaloth | 46 | chain lightning (12d6+3) | 8 | wizard 50 | 209 | 45.0 | 49.5 | 25.1 | 4.2 |
| archlich | 54 | sunfire (16d6+4) | 3 | warrior 60 | 380 | 60.0 | 42.0 | 9.1 | 26 |
| archlich | 54 | sunfire (16d6+4) | 3 | wizard 60 | 213 | 60.0 | 42.0 | 15.6 | 6.5 |
| ash wraith | 15 | scorching ray (6d6) | 5 | warrior 15 | 92 | 11.3 | 12.5 | 2.8 | 18 |
| ash wraith | 15 | scorching ray (6d6) | 5 | wizard 15 | 36 | 18.5 | 20.4 | 5.5 | 1.8 |
| barrow king | 42 | grave spear (10d6+2) | 5 | warrior 50 | 376 | 37.0 | 40.7 | 10.3 | 23 |
| barrow king | 42 | grave spear (10d6+2) | 5 | wizard 50 | 209 | 37.0 | 40.7 | 18.6 | 5.9 |
| bheur hag | 42 | rime (8d6+2) | 7 | warrior 50 | 376 | 30.0 | 33.0 | 5.3 | 39 |
| bheur hag | 42 | rime (8d6+2) | 7 | wizard 50 | 209 | 30.0 | 33.0 | 17.6 | 6.5 |
| dark wizard | 24 | fireball (8d6) | 4 | warrior 30 | 203 | 28.0 | 28.0 | 0.0 | >999 |
| dark wizard | 24 | fireball (8d6) | 4 | wizard 30 | 102 | 28.0 | 28.0 | 2.8 | 3.8 |
| drow archmage | 40 | chain lightning (12d6+3) | 8 | warrior 40 | 286 | 45.0 | 49.5 | 7.1 | 5.8 |
| drow archmage | 40 | chain lightning (12d6+3) | 8 | wizard 40 | 154 | 45.0 | 49.5 | 23.7 | 3.1 |
| drow high priestess | 44 | leech (8d6+2) | 13 | warrior 50 | 376 | 27.2 | 29.9 | 6.7 | 15 |
| drow high priestess | 44 | leech (8d6+2) | 13 | wizard 50 | 209 | 29.8 | 32.8 | 26.2 | 6.4 |
| drow mage | 22 | lightning bolt (10d6) | 4 | warrior 30 | 203 | 35.0 | 35.0 | 0.6 | 112 |
| drow mage | 22 | lightning bolt (10d6) | 4 | wizard 30 | 102 | 35.0 | 35.0 | 9.0 | 2.9 |
| drow matron | 50 | chain lightning (12d6+3) | 8 | warrior 50 | 376 | 45.0 | 49.5 | 9.0 | 9.4 |
| drow matron | 50 | chain lightning (12d6+3) | 8 | wizard 50 | 209 | 45.0 | 49.5 | 28.5 | 4.2 |
| drow priestess | 24 | leech (8d6) | 9 | warrior 30 | 203 | 17.9 | 17.9 | 1.3 | 48 |
| drow priestess | 24 | leech (8d6) | 9 | wizard 30 | 102 | 27.5 | 27.5 | 12.6 | 3.7 |
| dryad | 19 | acid arrow (6d6) | 10 | warrior 20 | 128 | 11.6 | 11.6 | 1.8 | 19 |
| dryad | 19 | acid arrow (6d6) | 10 | wizard 20 | 55 | 19.0 | 19.0 | 9.4 | 2.9 |
| duergar mindbreaker | 29 | flay (2d8) | 9 | warrior 30 | 203 | 9.0 | 9.0 | 5.2 | 33 |
| duergar mindbreaker | 29 | flay (2d8) | 9 | wizard 30 | 102 | 9.0 | 9.0 | 16.3 | 10 |
| duergar priest | 19 | flay (2d8) | 6 | warrior 20 | 128 | 9.0 | 9.0 | 0.0 | >999 |
| duergar priest | 19 | flay (2d8) | 6 | wizard 20 | 55 | 9.0 | 9.0 | 2.6 | 7.4 |
| ghost lord | 53 | grave spear (10d6+2) | 6 | warrior 60 | 380 | 37.0 | 51.8 | 13.9 | 16 |
| ghost lord | 53 | grave spear (10d6+2) | 6 | wizard 60 | 213 | 37.0 | 51.8 | 29.3 | 4.2 |
| giant shaman | 36 | rime (8d6) | 7 | warrior 40 | 286 | 28.0 | 28.0 | 5.7 | 23 |
| giant shaman | 36 | rime (8d6) | 7 | wizard 40 | 154 | 28.0 | 28.0 | 20.2 | 5.5 |
| goblin wizard | 8 | burning hands (2d6) | 11 | warrior 10 | 61 | 7.0 | 7.0 | 0.0 | 8.7 |
| goblin wizard | 8 | burning hands (2d6) | 11 | wizard 10 | 25 | 7.0 | 7.0 | 0.2 | 3.6 |
| green hag | 18 | blight (6d6+3) | 3 | warrior 20 | 128 | 24.0 | 24.0 | 0.7 | 87 |
| green hag | 18 | blight (6d6+3) | 3 | wizard 20 | 55 | 24.0 | 24.0 | 6.8 | 2.3 |
| hobgoblin warcaster | 14 | frost (6d6) | 3 | warrior 15 | 92 | 21.0 | 21.0 | 0.0 | >999 |
| hobgoblin warcaster | 14 | frost (6d6) | 3 | wizard 15 | 36 | 21.0 | 21.0 | 1.9 | 1.7 |
| kobold shaman | 4 | spark (2d4) | 5 | warrior 5 | 37 | 5.0 | 4.0 | 0.0 | >999 |
| kobold shaman | 4 | spark (2d4) | 5 | wizard 5 | 17 | 5.0 | 4.0 | 0.2 | 4.3 |
| lich | 30 | chain lightning (12d6+3) | 7 | warrior 30 | 203 | 45.0 | 31.5 | 3.3 | 6.4 |
| lich | 30 | chain lightning (12d6+3) | 7 | wizard 30 | 102 | 45.0 | 31.5 | 9.1 | 3.2 |
| mummy king | 56 | flame (10d6+2) | 6 | warrior 60 | 380 | 37.0 | 42.6 | 12.5 | 18 |
| mummy king | 56 | flame (10d6+2) | 6 | wizard 60 | 213 | 37.0 | 42.6 | 33.5 | 5.0 |
| mummy lord | 36 | blight (6d6+3) | 6 | warrior 40 | 286 | 24.0 | 25.2 | 3.3 | 50 |
| mummy lord | 36 | blight (6d6+3) | 6 | wizard 40 | 154 | 24.0 | 25.2 | 17.6 | 6.3 |
| mummy priest | 20 | cause wounds (4d8+2) | 27 | warrior 20 | 128 | 12.5 | 12.5 | 5.2 | 10 |
| mummy priest | 20 | cause wounds (4d8+2) | 27 | wizard 20 | 55 | 18.7 | 18.7 | 9.6 | 3.0 |
| necromancer | 18 | leech (8d6) | 7 | warrior 20 | 128 | 15.5 | 15.5 | 0.0 | >999 |
| necromancer | 18 | leech (8d6) | 7 | wizard 20 | 55 | 25.3 | 25.3 | 1.8 | 2.2 |
| night hag | 33 | leech (8d6+2) | 10 | warrior 40 | 286 | 20.9 | 21.9 | 1.5 | 64 |
| night hag | 33 | leech (8d6+2) | 10 | wizard 40 | 154 | 29.7 | 31.1 | 14.6 | 4.9 |
| nymph | 22 | magic missile (6d4+1) | 15 | warrior 30 | 203 | 16.0 | 22.4 | 1.0 | 9.0 |
| nymph | 22 | magic missile (6d4+1) | 15 | wizard 30 | 102 | 16.0 | 22.4 | 12.5 | 4.5 |
| ogre mage | 18 | rime (8d6) | 4 | warrior 20 | 128 | 28.0 | 28.0 | 2.6 | 11 |
| ogre mage | 18 | rime (8d6) | 4 | wizard 20 | 55 | 28.0 | 28.0 | 11.5 | 2.0 |
| orc high shaman | 24 | fireball (8d6) | 3 | warrior 30 | 203 | 28.0 | 28.0 | 0.0 | >999 |
| orc high shaman | 24 | fireball (8d6) | 3 | wizard 30 | 102 | 28.0 | 28.0 | 2.8 | 9.2 |
| orc shaman | 10 | burning hands (2d6) | 4 | warrior 10 | 61 | 7.0 | 7.0 | 0.0 | >999 |
| orc shaman | 10 | burning hands (2d6) | 4 | wizard 10 | 25 | 7.0 | 7.0 | 1.3 | 3.6 |
| pixie | 4 | spark (2d4) | 5 | warrior 5 | 37 | 5.0 | 7.0 | 0.0 | 448 |
| pixie | 4 | spark (2d4) | 5 | wizard 5 | 17 | 5.0 | 7.0 | 0.8 | 2.5 |
| rakshasa | 52 | flame (10d6+2) | 6 | warrior 60 | 380 | 37.0 | 51.8 | 12.2 | 17 |
| rakshasa | 52 | flame (10d6+2) | 6 | wizard 60 | 213 | 37.0 | 51.8 | 37.8 | 4.1 |
| troll shaman | 21 | rime (8d6) | 5 | warrior 30 | 203 | 28.0 | 28.0 | 1.4 | 50 |
| troll shaman | 21 | rime (8d6) | 5 | wizard 30 | 102 | 28.0 | 28.0 | 12.1 | 3.6 |
| will-o'-wisp | 7 | spark (2d4) | 8 | warrior 10 | 61 | 5.0 | 7.0 | 0.1 | 297 |
| will-o'-wisp | 7 | spark (2d4) | 8 | wizard 10 | 25 | 5.0 | 7.0 | 0.3 | 3.6 |
| wormlord | 47 | leech (8d6+2) | 13 | warrior 50 | 376 | 27.2 | 19.0 | 4.3 | 24 |
| wormlord | 47 | leech (8d6+2) | 13 | wizard 50 | 209 | 29.8 | 20.9 | 16.5 | 10 |
| barrow wight | 6 | grave breath (4d4) | 5 | warrior 10 | 61 | 10.0 | 10.0 | 0.9 | 19 |
| barrow wight | 6 | grave breath (4d4) | 5 | wizard 10 | 25 | 10.0 | 10.0 | 2.2 | 2.5 |
| bog hag | 11 | acid arrow (6d6) | 6 | warrior 15 | 92 | 8.1 | 8.1 | 0.3 | 198 |
| bog hag | 11 | acid arrow (6d6) | 6 | wizard 15 | 36 | 16.6 | 16.6 | 4.3 | 2.2 |
| cult priest | 9 | grave breath (4d4) | 6 | warrior 10 | 61 | 10.0 | 10.0 | 0.0 | 306 |
| cult priest | 9 | grave breath (4d4) | 6 | wizard 10 | 25 | 10.0 | 10.0 | 0.6 | 2.5 |

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

## Miscasts

Each class's Spellcraft, the bonus of its check on a cast and its ward (trained, plain gear, the seeds' mean, the bonus rounded), and the chance of a miscast for a spell of level 1, of level 11, and of the highest level it knows (that level in brackets).

| Class | L | Spellcraft | Bonus | Ward | Level 1 | Level 11 | Highest known |
|---|---:|---:|---:|---:|---:|---:|---:|
| wizard | 1 | 20 | +5.2 | 2 | 6.9 % | 68.8 % | 6.9 % (1) |
| wizard | 5 | 20 | +5.3 | 2 | 6.9 % | 68.8 % | 14.6 % (3) |
| wizard | 10 | 20 | +5.8 | 2 | 4.2 % | 61.8 % | 38.2 % (8) |
| wizard | 15 | 20 | +6.7 | 2 | 2.1 % | 54.2 % | 31.2 % (8) |
| wizard | 20 | 20 | +7.2 | 2 | 2.1 % | 54.2 % | 31.2 % (8) |
| wizard | 30 | 20 | +7.8 | 2 | 0.7 % | 45.8 % | 95.8 % (20) |
| wizard | 40 | 20 | +8.3 | 2 | 0.7 % | 45.8 % | 99.3 % (30) |
| wizard | 50 | 20 | +8.8 | 2 | 0.7 % | 38.2 % | 99.3 % (30) |
| wizard | 60 | 20 | +9.2 | 2 | 0.7 % | 38.2 % | 99.3 % (30) |
| priest | 1 | 22 | +2.2 | 2 | 19.4 % | 85.4 % | 19.4 % (1) |
| priest | 5 | 22 | +2.2 | 2 | 19.4 % | 85.4 % | 31.2 % (3) |
| priest | 10 | 22 | +3.0 | 2 | 14.6 % | 80.6 % | 75.0 % (10) |
| priest | 15 | 22 | +3.2 | 2 | 14.6 % | 80.6 % | 80.6 % (11) |
| priest | 20 | 22 | +3.7 | 2 | 10.4 % | 75.0 % | 89.6 % (14) |
| priest | 30 | 22 | +4.5 | 2 | 6.9 % | 68.8 % | 85.4 % (14) |
| priest | 40 | 22 | +4.8 | 2 | 6.9 % | 68.8 % | 85.4 % (14) |
| priest | 50 | 22 | +5.0 | 2 | 6.9 % | 68.8 % | 85.4 % (14) |
| priest | 60 | 22 | +5.5 | 2 | 4.2 % | 61.8 % | 80.6 % (14) |
| witch | 1 | 18 | +2.3 | 1 | 19.4 % | 85.4 % | 19.4 % (1) |
| witch | 5 | 18 | +2.3 | 1 | 19.4 % | 85.4 % | 31.2 % (3) |
| witch | 10 | 18 | +2.8 | 1 | 14.6 % | 80.6 % | 61.8 % (8) |
| witch | 15 | 18 | +3.5 | 1 | 10.4 % | 75.0 % | 54.2 % (8) |
| witch | 20 | 18 | +3.7 | 1 | 10.4 % | 75.0 % | 61.8 % (9) |
| witch | 30 | 18 | +4.5 | 1 | 6.9 % | 68.8 % | 99.3 % (24) |
| witch | 40 | 18 | +4.5 | 1 | 6.9 % | 68.8 % | 99.3 % (24) |
| witch | 50 | 18 | +4.8 | 1 | 6.9 % | 68.8 % | 99.3 % (24) |
| witch | 60 | 18 | +4.8 | 1 | 6.9 % | 68.8 % | 99.3 % (24) |

Creature casters: the chance of a miscast for each spell of their lists (their Spellcraft and Intelligence as their type makes them).

| Creature | L | Spellcraft | Bonus | Spells |
|---|---:|---:|---:|---|
| arcanaloth | 46 | 153 | +35 | fire cloak 0.7 %, chain lightning 0.7 %, fireball 0.7 %, magic missile 0.7 % |
| archlich | 54 | 177 | +42 | haste 0.7 %, silence 0.7 %, sunfire 0.7 %, chain lightning 0.7 %, grave spear 0.7 % |
| ash wraith | 15 | 50 | +9 | scorching ray 14.6 %, grave breath 0.7 % |
| barrow king | 42 | 141 | +32 | fear 0.7 %, grave spear 0.7 %, rime 0.7 % |
| bheur hag | 42 | 141 | +32 | curse of sloth 0.7 %, rime 0.7 %, frost 0.7 % |
| dark wizard | 24 | 98 | +17 | haste 2.1 %, fireball 2.1 %, lightning bolt 2.1 %, magic missile 0.7 % |
| deathless king | 66 | 213 | +49 | haste 0.7 %, fear 0.7 %, sunfire 0.7 %, grave spear 0.7 %, chain lightning 0.7 % |
| demilich | 70 | 225 | +49 | silence 0.7 %, sunfire 0.7 %, chain lightning 0.7 %, leech 0.7 %, grave spear 0.7 % |
| drow archmage | 40 | 115 | +27 | haste 0.7 %, web 0.7 %, chain lightning 0.7 %, fireball 0.7 % |
| drow high priestess | 44 | 123 | +29 | cure wounds 0.7 %, silence 0.7 %, web 0.7 %, leech 0.7 % |
| drow mage | 22 | 79 | +16 | web 0.7 %, lightning bolt 4.2 %, magic missile 0.7 % |
| drow matron | 50 | 135 | +33 | cure wounds 0.7 %, silence 0.7 %, web 0.7 %, chain lightning 0.7 %, leech 0.7 % |
| drow priestess | 24 | 83 | +18 | cure wounds 0.7 %, silence 0.7 %, leech 0.7 % |
| dryad | 19 | 72 | +15 | barkskin 0.7 %, bind 2.1 %, acid arrow 0.7 % |
| duergar mindbreaker | 29 | 102 | +18 | fear 0.7 %, bind 0.7 %, flay 0.7 % |
| duergar priest | 19 | 72 | +12 | cure wounds 0.7 %, blessing 0.7 %, flay 6.9 %, cause wounds 0.7 % |
| ghost lord | 53 | 174 | +40 | fear 0.7 %, slumber 0.7 %, grave spear 0.7 % |
| giant shaman | 36 | 123 | +27 | cure wounds 0.7 %, rime 0.7 %, lightning bolt 0.7 % |
| goblin wizard | 8 | 34 | +6 | burning hands 4.2 %, spark 4.2 % |
| green hag | 18 | 69 | +14 | curse of sloth 0.7 %, mesmerise 0.7 %, blight 0.7 % |
| hobgoblin warcaster | 14 | 57 | +10 | frost 6.9 %, arcane bolt 6.9 %, magic missile 0.7 % |
| kobold shaman | 4 | 25 | +3 | mend 14.6 %, spark 14.6 % |
| lich | 30 | 105 | +23 | haste 0.7 %, chain lightning 10.4 %, fireball 0.7 %, grave spear 0.7 % |
| mummy king | 56 | 183 | +45 | fear 0.7 %, flame 0.7 %, grave spear 0.7 % |
| mummy lord | 36 | 123 | +27 | fear 0.7 %, blight 0.7 %, grave spear 0.7 % |
| mummy priest | 20 | 75 | +16 | fear 2.1 %, cause wounds 0.7 %, blight 0.7 % |
| necromancer | 18 | 73 | +12 | hex of weakness 0.7 %, leech 14.6 %, blight 4.2 % |
| night hag | 33 | 114 | +24 | slumber 0.7 %, leech 0.7 %, blight 0.7 % |
| nymph | 22 | 81 | +17 | mesmerise 0.7 %, magic missile 0.7 % |
| ogre mage | 18 | 69 | +12 | rime 10.4 %, frost 2.1 %, magic missile 0.7 % |
| orc high shaman | 24 | 87 | +15 | cure wounds 0.7 %, blessing 0.7 %, fireball 6.9 % |
| orc shaman | 10 | 45 | +7 | mend 2.1 %, burning hands 2.1 %, spark 2.1 % |
| pixie | 4 | 27 | +4 | evil eye 6.9 %, spark 10.4 % |
| rakshasa | 52 | 171 | +40 | glamour 0.7 %, mesmerise 0.7 %, flame 0.7 %, lightning bolt 0.7 % |
| troll shaman | 21 | 78 | +12 | cure wounds 0.7 %, rime 10.4 %, lightning bolt 19.4 % |
| will-o'-wisp | 7 | 30 | +4 | spark 10.4 % |
| wormlord | 47 | 156 | +36 | fear 0.7 %, leech 0.7 %, blight 0.7 % |
| barrow wight | 6 | 35 | +6 | hex of weakness 10.4 %, grave breath 4.2 % |
| bog hag | 11 | 60 | +11 | hex of weakness 0.7 %, acid arrow 2.1 %, grave breath 0.7 % |
| cult priest | 9 | 45 | +7 | mend 2.1 %, evil eye 0.7 %, grave breath 2.1 % |
| Līgetdrȳ the Storm-Wright | 61 | 195 | +48 | haste 0.7 %, silence 0.7 %, chain lightning 0.7 %, lightning bolt 0.7 % |

The miscast check (#1271, `data/miscast.toml`: base 10, +1 a spell level) was tuned on the
starting wizard: a bonus of about +5 miscasts a level-1 spell one cast in fifteen (6.9 %). The
probe's careers never train Spellcraft (no fight casts train it here, and `levelup:auto` spends
the skill rolls elsewhere), so its Spellcraft stays at the starting roll and the bonus grows by
Intelligence alone: the "Highest known" column is the worst case, a caster whose Spellcraft never
rose. In play every cast with one's own mana marks Spellcraft (#1270), and a caster who keeps up
with their spells (Spellcraft about 80 for a level-11 spell, +11; about 150 for a level-24 one,
+24, with Intelligence 16 to 18) miscasts them one cast in ten to fifteen. The shipped creature casters' Spellcraft keeps them
at the 1 and 1 for most of their lists; the low ones (kobold shaman, pixie, will-o'-wisp, troll
shaman's lightning bolt) miscast one cast in seven to ten.

## Protection by body

Rounds for the plain warrior (trained) to kill the monsters of their level: as shipped, then with every monster's Protection set to §8.2's body line at its level.

| L | as shipped | none (0) | hide | bony | armoured |
|---|---|---|---|---|---|
| 10 | 4.9 | 4.4 | 4.3 | 4.7 | 5.7 |
| 15 | 6.7 | 5.5 | 5.5 | 6.2 | 7.5 |
| 20 | 7.3 | 5.7 | 6.1 | 6.8 | 7.7 |
| 30 | 11 | 7.6 | 8.3 | 9.3 | 11 |
| 40 | 11 | 7.3 | 8.1 | 9.1 | 11 |
| 50 | 13 | 11 | 12 | 13 | 16 |
| 60 | 13 | 7.7 | 8.8 | 9.8 | 11 |

## Spells

Each spell that rolls (harm or healing) of the schools the trained caster knows or has an affinity with, as `Actor::spell_roll` makes it in plain gear, per cast on one target: the rung of its ladder the stats reach (the seeds' range), the school's rank the pace below reached (and the rank of the caster style it is cast in, if any), the roll (seed 1's dice, the seeds' mean) and the same roll at rank 0 (the ladder alone), its mean on a target under a hex (Malison's bite; `–` without it), and its mana (the cost table's price with no rank's cut in brackets) over the pool.

**Wizard**

| Spell (level, res) | L | rung | school rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Burning hands (1, area) | 10 | 1–2 | 3 | 4d4 1s again, 10.9 (rank 0: 4d4, 9.5) | – | 3 (3) / 99 |
| Burning hands (1, area) | 20 | 2 | 4–5 | 4d4 1s again, 12.8 (rank 0: 4d4, 10.0) | – | 2 (3) / 208 |
| Burning hands (1, area) | 30 | 2 | 6 | 5d5 1s again, 17.0 (rank 0: 4d4, 10.0) | – | 2 (3) / 331 |
| Burning hands (1, area) | 60 | 3 | 7 | 5d5+2 1s and 2s again, 20.6 (rank 0: 4d4+2, 12.0) | – | 1 (3) / 730 |
| Grave breath (1, area) | 10 | 2 | 0 | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 3 (3) / 99 |
| Grave breath (1, area) | 20 | 2–3 | 0 | 4d4+2, 10.7 (rank 0: 4d4+2, 10.7) | – | 3 (3) / 208 |
| Grave breath (1, area) | 30 | 2–3 | 0 | 4d4+2, 11.3 (rank 0: 4d4+2, 11.3) | – | 3 (3) / 331 |
| Grave breath (1, area) | 60 | 3 | 0 | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 1 (3) / 730 |
| Ray of frost (1, attack) | 10 | 0–1 | 3 (Sniper 2–3) | 2d8 1s again, 9.5 (rank 0: 2d8, 8.7) | – | 1 (1) / 99 |
| Ray of frost (1, attack) | 20 | 1 | 4–5 (Sniper 2–3) | 2d8 1s again, 11.1 (rank 0: 2d8, 9.0) | – | 1 (1) / 208 |
| Ray of frost (1, attack) | 30 | 1 | 6 (Sniper 2–3) | 3d9 1s again, 16.3 (rank 0: 2d8, 9.0) | – | 1 (1) / 331 |
| Ray of frost (1, attack) | 60 | 1 | 7 (Sniper 2–3) | 3d9 1s and 2s again, 18.3 (rank 0: 2d8, 9.0) | – | 1 (1) / 730 |
| Spark (1, auto) | 10 | 0–1 | 3 | 2d6 1s again, 7.5 (rank 0: 2d6, 6.7) | – | 1 (1) / 99 |
| Spark (1, auto) | 20 | 1 | 4–5 | 2d6 1s again, 8.9 (rank 0: 2d6, 7.0) | – | 1 (1) / 208 |
| Spark (1, auto) | 30 | 1 | 6 | 3d7 1s again, 13.3 (rank 0: 2d6, 7.0) | – | 1 (1) / 331 |
| Spark (1, auto) | 60 | 1 | 7 | 3d7 1s and 2s again, 14.9 (rank 0: 2d6, 7.0) | – | 1 (1) / 730 |
| Acid splash (3, area) | 10 | 1 | 3 | 2d6 1s again, 7.8 (rank 0: 2d6, 7.0) | – | 5 (5) / 99 |
| Acid splash (3, area) | 20 | 1–2 | 4–5 | 4d4 1s again, 10.9 (rank 0: 4d4, 8.5) | – | 4 (5) / 208 |
| Acid splash (3, area) | 30 | 2 | 6 | 5d5 1s again, 17.0 (rank 0: 4d4, 10.0) | – | 4 (5) / 331 |
| Acid splash (3, area) | 60 | 2 | 7 | 5d5 1s and 2s again, 18.6 (rank 0: 4d4, 10.0) | – | 1 (5) / 730 |
| Magic missile (3, auto) | 10 | 1–2 | 3 | 6d4 1s again, 16.3 (rank 0: 6d4, 14.2) | – | 3 (3) / 99 |
| Magic missile (3, auto) | 20 | 2 | 4–5 | 6d4 1s again, 18.9 (rank 0: 6d4, 15.0) | – | 2 (3) / 208 |
| Magic missile (3, auto) | 30 | 2 | 6 | 7d5 1s again, 23.8 (rank 0: 6d4, 15.0) | – | 2 (3) / 331 |
| Magic missile (3, auto) | 60 | 3 | 7 | 7d5+1 1s and 2s again, 26.8 (rank 0: 6d4+1, 16.0) | – | 1 (3) / 730 |
| Cause wounds (4, touch) | 10 | 1 | 0 (Toucher 0) | 4d6, 14.0 (rank 0: 4d6, 14.0) | – | 3 (3) / 99 |
| Cause wounds (4, touch) | 20 | 2 | 0 (Toucher 0) | 4d8, 18.0 (rank 0: 4d8, 18.0) | – | 3 (3) / 208 |
| Cause wounds (4, touch) | 30 | 2 | 0 (Toucher 0) | 4d8, 18.0 (rank 0: 4d8, 18.0) | – | 3 (3) / 331 |
| Cause wounds (4, touch) | 60 | 3 | 0 (Toucher 0) | 4d8+2, 20.0 (rank 0: 4d8+2, 20.0) | – | 1 (3) / 730 |
| Acid arrow (5, attack) | 10 | 1–2 | 3 (Sniper 2–3) | 6d6 1s again, 22.2 (rank 0: 6d6, 19.8) | – | 5 (5) / 99 |
| Acid arrow (5, attack) | 20 | 2 | 4–5 (Sniper 2–3) | 6d6 1s again, 25.3 (rank 0: 6d6, 21.0) | – | 4 (5) / 208 |
| Acid arrow (5, attack) | 30 | 2 | 6 (Sniper 2–3) | 7d7 1s again, 31.0 (rank 0: 6d6, 21.0) | – | 4 (5) / 331 |
| Acid arrow (5, attack) | 60 | 3 | 7 (Sniper 2–3) | 7d7+2 1s and 2s again, 35.8 (rank 0: 6d6+2, 23.0) | – | 1 (5) / 730 |
| Call of the wild (5, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 99 |
| Call of the wild (5, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 208 |
| Call of the wild (5, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 331 |
| Call of the wild (5, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 2 (5) / 730 |
| Arcane bolt (6, area) | 10 | 1 | 3 | 6d6 1s again, 23.5 (rank 0: 6d6, 21.0) | – | 11 (11) / 99 |
| Arcane bolt (6, area) | 20 | 1–2 | 4–5 | 6d6+2 1s again, 26.3 (rank 0: 6d6+2, 22.0) | – | 10 (11) / 208 |
| Arcane bolt (6, area) | 30 | 2 | 6 | 7d7+2 1s again, 33.0 (rank 0: 6d6+2, 23.0) | – | 10 (11) / 331 |
| Arcane bolt (6, area) | 60 | 3 | 7 | 7d7+4 1s and 2s again, 37.8 (rank 0: 6d6+4, 25.0) | – | 2 (11) / 730 |
| Frost (6, area) | 10 | 1 | 3 | 6d6 1s again, 23.5 (rank 0: 6d6, 21.0) | – | 14 (14) / 99 |
| Frost (6, area) | 20 | 1–2 | 4–5 | 6d6+2 1s again, 26.3 (rank 0: 6d6+2, 22.0) | – | 13 (14) / 208 |
| Frost (6, area) | 30 | 2 | 6 | 7d7+2 1s again, 33.0 (rank 0: 6d6+2, 23.0) | – | 13 (14) / 331 |
| Frost (6, area) | 60 | 3 | 7 | 7d7+4 1s and 2s again, 37.8 (rank 0: 6d6+4, 25.0) | – | 3 (14) / 730 |
| Blight (7, area) | 10 | 1 | 0 | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 17 (17) / 99 |
| Blight (7, area) | 20 | 1–2 | 0 | 6d6+3, 22.0 (rank 0: 6d6+3, 22.0) | – | 17 (17) / 208 |
| Blight (7, area) | 30 | 1–2 | 0 | 6d6+3, 23.0 (rank 0: 6d6+3, 23.0) | – | 17 (17) / 331 |
| Blight (7, area) | 60 | 2 | 0 | 6d6+3, 24.0 (rank 0: 6d6+3, 24.0) | – | 5 (17) / 730 |
| Scorching ray (7, attack) | 10 | 1–2 | 3 (Sniper 2–3) | 6d6 1s again, 22.2 (rank 0: 6d6, 19.8) | – | 7 (7) / 99 |
| Scorching ray (7, attack) | 20 | 2–3 | 4–5 (Sniper 2–3) | 8d6 1s again, 29.3 (rank 0: 8d6, 24.5) | – | 6 (7) / 208 |
| Scorching ray (7, attack) | 30 | 3 | 6 (Sniper 2–3) | 9d7 1s again, 39.9 (rank 0: 8d6, 28.0) | – | 6 (7) / 331 |
| Scorching ray (7, attack) | 60 | 4 | 7 (Sniper 2–3) | 7d7+2d11 1s and 2s again, 47.4 (rank 0: 6d6+2d10, 32.0) | – | 2 (7) / 730 |
| Thunderclap (7, area) | 10 | 0 | 3 | 2d6 1s again, 7.8 (rank 0: 2d6, 7.0) | – | 13 (13) / 99 |
| Thunderclap (7, area) | 20 | 0 | 4–5 | 2d6 1s again, 8.9 (rank 0: 2d6, 7.0) | – | 12 (13) / 208 |
| Thunderclap (7, area) | 30 | 0 | 6 | 3d7 1s again, 13.3 (rank 0: 2d6, 7.0) | – | 12 (13) / 331 |
| Thunderclap (7, area) | 60 | 2 | 7 | 5d7+2 1s and 2s again, 26.4 (rank 0: 4d6+2, 16.0) | – | 2 (13) / 730 |
| Flay (8, auto) | 10 | 1 | 0 | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 8 (8) / 99 |
| Flay (8, auto) | 20 | 1–2 | 0 | 2d8, 7.7 (rank 0: 2d8, 7.7) | – | 8 (8) / 208 |
| Flay (8, auto) | 30 | 1–2 | 0 | 2d8, 8.3 (rank 0: 2d8, 8.3) | – | 8 (8) / 331 |
| Flay (8, auto) | 60 | 2 | 0 | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 2 (8) / 730 |
| Pitch fire (8, area) | 10 | 1–2 | 3 | 4d6 1s again, 15.0 (rank 0: 4d6, 13.4) | – | 14 (14) / 99 |
| Pitch fire (8, area) | 20 | 2 | 4–5 | 4d6 1s again, 17.1 (rank 0: 4d6, 14.0) | – | 13 (14) / 208 |
| Pitch fire (8, area) | 30 | 2 | 6 | 5d7 1s again, 22.1 (rank 0: 4d6, 14.0) | – | 13 (14) / 331 |
| Pitch fire (8, area) | 60 | 3 | 7 | 5d7+2 1s and 2s again, 26.4 (rank 0: 4d6+2, 16.0) | – | 3 (14) / 730 |
| Shatter (8, area) | 10 | 1 | 3 | 6d6 1s again, 23.5 (rank 0: 6d6, 21.0) | – | 14 (14) / 99 |
| Shatter (8, area) | 20 | 1–2 | 4–5 | 6d6+2 1s again, 26.3 (rank 0: 6d6+2, 22.0) | – | 13 (14) / 208 |
| Shatter (8, area) | 30 | 2 | 6 | 7d7+2 1s again, 33.0 (rank 0: 6d6+2, 23.0) | – | 13 (14) / 331 |
| Shatter (8, area) | 60 | 3 | 7 | 7d7+4 1s and 2s again, 37.8 (rank 0: 6d6+4, 25.0) | – | 3 (14) / 730 |
| Rime (9, area) | 10 | 2 | 3 | 8d6 1s again, 31.3 (rank 0: 8d6, 28.0) | – | 14 (14) / 99 |
| Rime (9, area) | 20 | 2–3 | 4–5 | 8d6+2 1s again, 34.1 (rank 0: 8d6+2, 28.7) | – | 13 (14) / 208 |
| Rime (9, area) | 30 | 2–3 | 6 | 9d7+2 1s again, 41.2 (rank 0: 8d6+2, 29.3) | – | 13 (14) / 331 |
| Rime (9, area) | 60 | 3 | 7 | 9d7+2 1s and 2s again, 45.2 (rank 0: 8d6+2, 30.0) | – | 3 (14) / 730 |
| Leech (10, attack) | 10 | 2 | 0 (Sniper 2–3) | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 10 (10) / 99 |
| Leech (10, attack) | 20 | 2 | 0 (Sniper 2–3) | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 10 (10) / 208 |
| Leech (10, attack) | 30 | 2 | 0 (Sniper 2–3) | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 10 (10) / 331 |
| Leech (10, attack) | 60 | 3 | 0 (Sniper 2–3) | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | – | 3 (10) / 730 |
| Call lightning (11, area) | 10 | 2 | 3 | 8d6 1s again, 31.3 (rank 0: 8d6, 28.0) | – | 20 (20) / 99 |
| Call lightning (11, area) | 20 | 3 | 4–5 | 10d6 1s again, 41.6 (rank 0: 10d6, 35.0) | – | 19 (20) / 208 |
| Call lightning (11, area) | 30 | 3 | 6 | 11d7 1s again, 48.7 (rank 0: 10d6, 35.0) | – | 19 (20) / 331 |
| Call lightning (11, area) | 60 | 4 | 7 | 11d7+2 1s and 2s again, 54.6 (rank 0: 10d6+2, 37.0) | – | 5 (20) / 730 |
| Fireball (11, area) | 10 | 1–2 | 3 | 8d6 1s again, 30.0 (rank 0: 8d6, 26.8) | – | 26 (26) / 99 |
| Fireball (11, area) | 20 | 2–3 | 4–5 | 10d6 1s again, 37.5 (rank 0: 10d6, 31.5) | – | 24 (26) / 208 |
| Fireball (11, area) | 30 | 3 | 6 | 11d7 1s again, 48.7 (rank 0: 10d6, 35.0) | – | 24 (26) / 331 |
| Fireball (11, area) | 60 | 4 | 7 | 11d7+2 1s and 2s again, 54.6 (rank 0: 10d6+2, 37.0) | – | 6 (26) / 730 |
| Lightning bolt (11, area) | 10 | 1–2 | 3 | 8d6 1s again, 30.0 (rank 0: 8d6, 26.8) | – | 20 (20) / 99 |
| Lightning bolt (11, area) | 20 | 2–3 | 4–5 | 10d6 1s again, 37.5 (rank 0: 10d6, 31.5) | – | 19 (20) / 208 |
| Lightning bolt (11, area) | 30 | 3 | 6 | 11d7 1s again, 48.7 (rank 0: 10d6, 35.0) | – | 19 (20) / 331 |
| Lightning bolt (11, area) | 60 | 4 | 7 | 11d7+2 1s and 2s again, 54.6 (rank 0: 10d6+2, 37.0) | – | 5 (20) / 730 |
| Flame (12, area) | 10 | 0–1 | 3 | 8d6 1s again, 30.0 (rank 0: 8d6, 26.8) | – | 29 (29) / 99 |
| Flame (12, area) | 20 | 1–2 | 4–5 | 10d6 1s again, 37.5 (rank 0: 10d6, 31.5) | – | 27 (29) / 208 |
| Flame (12, area) | 30 | 2 | 6 | 11d7 1s again, 48.7 (rank 0: 10d6, 35.0) | – | 27 (29) / 331 |
| Flame (12, area) | 60 | 3 | 7 | 11d7+2 1s and 2s again, 54.6 (rank 0: 10d6+2, 37.0) | – | 6 (29) / 730 |
| Reek (13, area) | 10 | 1 | 0 | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 31 (31) / 99 |
| Reek (13, area) | 20 | 1–2 | 0 | 4d4+2, 10.7 (rank 0: 4d4+2, 10.7) | – | 31 (31) / 208 |
| Reek (13, area) | 30 | 1–2 | 0 | 4d4+2, 11.3 (rank 0: 4d4+2, 11.3) | – | 31 (31) / 331 |
| Reek (13, area) | 60 | 2 | 0 | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 10 (31) / 730 |
| Grave spear (14, area) | 10 | 2 | 0 | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 25 (25) / 99 |
| Grave spear (14, area) | 20 | 3 | 0 | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 25 (25) / 208 |
| Grave spear (14, area) | 30 | 3 | 0 | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 25 (25) / 331 |
| Grave spear (14, area) | 60 | 4 | 0 | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 8 (25) / 730 |
| Rebuke (14, area) | 10 | 2 | 0 | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 34 (34) / 99 |
| Rebuke (14, area) | 20 | 3 | 0 | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 34 (34) / 208 |
| Rebuke (14, area) | 30 | 3 | 0 | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 34 (34) / 331 |
| Rebuke (14, area) | 60 | 4 | 0 | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 10 (34) / 730 |
| Balefire (16, area) | 10 | 1–2 | 3 | 8d6 1s again, 30.0 (rank 0: 8d6, 26.8) | – | 29 (29) / 99 |
| Balefire (16, area) | 20 | 2 | 4–5 | 8d6 1s again, 33.4 (rank 0: 8d6, 28.0) | – | 27 (29) / 208 |
| Balefire (16, area) | 30 | 2 | 6 | 9d7 1s again, 39.9 (rank 0: 8d6, 28.0) | – | 27 (29) / 331 |
| Balefire (16, area) | 60 | 3 | 7 | 9d7+2 1s and 2s again, 45.2 (rank 0: 8d6+2, 30.0) | – | 6 (29) / 730 |
| Call of the forest (20, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 99 |
| Call of the forest (20, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 208 |
| Call of the forest (20, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 331 |
| Call of the forest (20, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 6 (20) / 730 |
| Chain lightning (20, auto) | 10 | 1–2 | 3 | 10d6 1s again, 37.9 (rank 0: 10d6, 33.8) | – | 20 (20) / 99 |
| Chain lightning (20, auto) | 20 | 2–3 | 4–5 | 12d6 1s again, 45.7 (rank 0: 12d6, 38.5) | – | 19 (20) / 208 |
| Chain lightning (20, auto) | 30 | 3 | 6 | 13d7 1s again, 57.6 (rank 0: 12d6, 42.0) | – | 19 (20) / 331 |
| Chain lightning (20, auto) | 60 | 4 | 7 | 13d7+3 1s and 2s again, 65.1 (rank 0: 12d6+3, 45.0) | – | 5 (20) / 730 |
| Sunfire (30, area) | 10 | 0–1 | 3 | 12d6 1s again, 45.7 (rank 0: 12d6, 40.8) | – | 90 (90) / 99 |
| Sunfire (30, area) | 20 | 1–2 | 4–5 | 14d6 1s again, 53.9 (rank 0: 14d6, 45.5) | – | 85 (90) / 208 |
| Sunfire (30, area) | 30 | 2 | 6 | 15d7 1s again, 66.4 (rank 0: 14d6, 49.0) | – | 85 (90) / 331 |
| Sunfire (30, area) | 60 | 4 | 7 | 17d7+4 1s and 2s again, 84.9 (rank 0: 16d6+4, 60.0) | – | 22 (90) / 730 |
| Call the elements (35, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 99 |
| Call the elements (35, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 208 |
| Call the elements (35, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 331 |
| Call the elements (35, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 11 (35) / 730 |

**Priest**

| Spell (level, res) | L | rung | school rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Mend (1, touch) | 10 | 0–1 | 5 (Battle caster 0) | 1d5+1 1s again, 4.3 (rank 0: 1d4, 2.4) | – | 1 (1) / 78 |
| Mend (1, touch) | 20 | 1 | 6–7 (Battle caster 5–6) | 1d5+2 1s again, 6.5 (rank 0: 1d4, 2.5) | – | 1 (1) / 160 |
| Mend (1, touch) | 30 | 1–2 | 6–7 (Battle caster 6–7) | 2d5+2 1s again, 8.7 (rank 0: 1d4, 2.8) | – | 1 (1) / 251 |
| Mend (1, touch) | 60 | 1–2 | 8 (Battle caster 8) | 2d7+2 1s and 2s again, 11.1 (rank 0: 1d6, 3.3) | – | 1 (1) / 546 |
| Cure wounds (3, touch) | 10 | 0–1 | 5 (Battle caster 0) | 3d7+1 1s again, 13.5 (rank 0: 3d6, 9.9) | – | 1 (2) / 78 |
| Cure wounds (3, touch) | 20 | 1 | 6–7 (Battle caster 5–6) | 3d7+2 1s again, 16.8 (rank 0: 3d6, 10.5) | – | 2 (2) / 160 |
| Cure wounds (3, touch) | 30 | 1–2 | 6–7 (Battle caster 6–7) | 4d7+2 1s again, 20.0 (rank 0: 3d6, 11.5) | – | 2 (2) / 251 |
| Cure wounds (3, touch) | 60 | 1–2 | 8 (Battle caster 8) | 4d7+5 1s and 2s again, 23.4 (rank 0: 3d6+3, 13.0) | – | 2 (2) / 546 |
| Benison (12, area) | 10 | 0–1 | 5 (Battle caster 0) | 3d7+1 1s again, 13.5 (rank 0: 3d6, 9.9) | – | 22 (22) / 78 |
| Benison (12, area) | 20 | 1 | 6–7 (Battle caster 5–6) | 3d7+2 1s again, 16.8 (rank 0: 3d6, 10.5) | – | 26 (22) / 160 |
| Benison (12, area) | 30 | 1–2 | 6–7 (Battle caster 6–7) | 4d7+2 1s again, 20.0 (rank 0: 3d6, 11.5) | – | 26 (22) / 251 |
| Benison (12, area) | 60 | 1–2 | 8 (Battle caster 8) | 4d7+5 1s and 2s again, 23.4 (rank 0: 3d6+3, 13.0) | – | 25 (22) / 546 |
| Rebuke (14, area) | 10 | 1–3 | 0 (Battle caster 0) | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 37 (34) / 78 |
| Rebuke (14, area) | 20 | 2–3 | 0 (Battle caster 5–6) | 10d6, 33.8 (rank 0: 10d6, 33.8) | – | 44 (34) / 160 |
| Rebuke (14, area) | 30 | 2–3 | 0 (Battle caster 6–7) | 10d6, 33.8 (rank 0: 10d6, 33.8) | – | 45 (34) / 251 |
| Rebuke (14, area) | 60 | 3–4 | 0 (Battle caster 8) | 10d6, 35.3 (rank 0: 10d6, 35.3) | – | 44 (34) / 546 |
| Orison (prayer) | 10 | – | – | no roll: answers 1 of 2 (wounds, hunger) | – | 0 (0) / 78 |
| Prayer (prayer) | 20 | – | – | no roll: answers 1 of 5 (disease, poison, wounds, release, hunger) | – | 0 (0) / 160 |
| Litany (prayer) | 30 | – | – | no roll: answers 1 of 6 (disease, poison, wounds, release, hunger, restoration) | – | 0 (0) / 251 |
| Miracle (prayer) | 60 | – | – | no roll: answers all of 7 (disease, poison, wounds, release, mana, hunger, restoration) | – | 0 (0) / 546 |

**Witch**

| Spell (level, res) | L | rung | school rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Grave breath (1, area) | 10 | 2 | 3 | 4d4+1 1s again, 12.5 (rank 0: 4d4, 10.0) | – | 3 (3) / 87 |
| Grave breath (1, area) | 20 | 2–3 | 4–5 | 4d4+1 1s again, 13.3 (rank 0: 4d4, 10.7) | 16.3 | 2 (3) / 189 |
| Grave breath (1, area) | 30 | 2–3 | 5 | 4d4+4 1s again, 15.2 (rank 0: 4d4+2, 11.7) | 18.2 | 2 (3) / 304 |
| Grave breath (1, area) | 60 | 3 | 8 | 5d4+5 1s and 2s again, 20.0 (rank 0: 4d4+2, 12.0) | 23.0 | 2 (3) / 692 |
| Cause wounds (4, touch) | 10 | 1–2 | 3 (Toucher 2–3) | 4d6+1 1s again, 17.3 (rank 0: 4d6, 14.7) | – | 3 (3) / 87 |
| Cause wounds (4, touch) | 20 | 2 | 4–5 (Toucher 2–3) | 4d8+1 1s again, 20.9 (rank 0: 4d8, 18.0) | 23.9 | 2 (3) / 189 |
| Cause wounds (4, touch) | 30 | 2–3 | 5 (Toucher 2–3) | 4d8+2 1s again, 22.1 (rank 0: 4d8, 18.3) | 25.1 | 2 (3) / 304 |
| Cause wounds (4, touch) | 60 | 3 | 8 (Toucher 2–3) | 5d8+5 1s and 2s again, 31.2 (rank 0: 4d8+2, 20.0) | 34.2 | 2 (3) / 692 |
| Call of the wild (5, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 87 |
| Call of the wild (5, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 189 |
| Call of the wild (5, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 304 |
| Call of the wild (5, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 4 (5) / 692 |
| Blight (7, area) | 10 | 1 | 3 | 6d6+1 1s again, 24.5 (rank 0: 6d6, 21.0) | – | 17 (17) / 87 |
| Blight (7, area) | 20 | 1–2 | 4–5 | 6d6+1 1s again, 25.7 (rank 0: 6d6, 22.0) | 28.7 | 16 (17) / 189 |
| Blight (7, area) | 30 | 1–2 | 5 | 6d6+5 1s again, 28.0 (rank 0: 6d6+3, 23.5) | 31.0 | 16 (17) / 304 |
| Blight (7, area) | 60 | 2 | 8 | 7d6+6 1s and 2s again, 35.2 (rank 0: 6d6+3, 24.0) | 38.2 | 12 (17) / 692 |
| Flay (8, auto) | 10 | 1 | 3 | 2d6+1 1s again, 8.8 (rank 0: 2d6, 7.0) | – | 8 (8) / 87 |
| Flay (8, auto) | 20 | 1–2 | 4–5 | 2d6+1 1s again, 9.7 (rank 0: 2d6, 7.7) | 12.7 | 7 (8) / 189 |
| Flay (8, auto) | 30 | 1–2 | 5 | 2d8+2 1s again, 11.5 (rank 0: 2d8, 8.7) | 14.5 | 7 (8) / 304 |
| Flay (8, auto) | 60 | 2 | 8 | 3d8+3 1s and 2s again, 18.8 (rank 0: 2d8, 9.0) | 21.8 | 6 (8) / 692 |
| Leech (10, attack) | 10 | 2 | 3 (Sniper 2) | 8d6+1 1s again, 32.3 (rank 0: 8d6, 28.0) | – | 10 (10) / 87 |
| Leech (10, attack) | 20 | 2 | 4–5 (Sniper 3) | 8d6+1 1s again, 32.5 (rank 0: 8d6, 28.0) | 35.5 | 9 (10) / 189 |
| Leech (10, attack) | 30 | 2–3 | 5 (Sniper 3) | 8d6+2 1s again, 33.7 (rank 0: 8d6, 28.3) | 36.7 | 9 (10) / 304 |
| Leech (10, attack) | 60 | 3 | 8 (Sniper 3) | 9d6+5 1s and 2s again, 42.5 (rank 0: 8d6+2, 30.0) | 45.5 | 7 (10) / 692 |
| Reek (13, area) | 10 | 1 | 3 | 4d4+1 1s again, 12.5 (rank 0: 4d4, 10.0) | – | 31 (31) / 87 |
| Reek (13, area) | 20 | 1–2 | 4–5 | 4d4+1 1s again, 13.3 (rank 0: 4d4, 10.7) | 16.3 | 29 (31) / 189 |
| Reek (13, area) | 30 | 1–2 | 5 | 4d4+4 1s again, 15.2 (rank 0: 4d4+2, 11.7) | 18.2 | 29 (31) / 304 |
| Reek (13, area) | 60 | 2 | 8 | 5d4+5 1s and 2s again, 20.0 (rank 0: 4d4+2, 12.0) | 23.0 | 22 (31) / 692 |
| Grave spear (14, area) | 10 | 2–3 | 3 | 8d6+1 1s again, 33.6 (rank 0: 8d6, 29.2) | – | 25 (25) / 87 |
| Grave spear (14, area) | 20 | 3 | 4–5 | 10d6+1 1s again, 40.3 (rank 0: 10d6, 35.0) | 43.3 | 23 (25) / 189 |
| Grave spear (14, area) | 30 | 3–4 | 5 | 10d6+2 1s again, 41.5 (rank 0: 10d6, 35.3) | 44.5 | 23 (25) / 304 |
| Grave spear (14, area) | 60 | 4 | 8 | 11d6+5 1s and 2s again, 50.8 (rank 0: 10d6+2, 37.0) | 53.8 | 18 (25) / 692 |
| Call of the forest (20, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 87 |
| Call of the forest (20, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 189 |
| Call of the forest (20, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 304 |
| Call of the forest (20, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 16 (20) / 692 |
| Call the elements (35, auto) | 10 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 87 |
| Call the elements (35, auto) | 20 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 189 |
| Call the elements (35, auto) | 30 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 304 |
| Call the elements (35, auto) | 60 | 0 | 0 | +1, 1.0 (rank 0: +1, 1.0) | – | 28 (35) / 692 |

The three tables are #1251's run (Malison's hexed column as #905 made it: a damage spell's mean on a target under one hex, from the witch's level 18): a spell is cast in a caster style only when it rolls to hit (a ray or a bolt Sniper's, a touch Toucher's) or in hindering armour (Battle caster's), so an area, a bolt that never misses, and a heal show no style, and the schools and styles train three times slower.

## School pace

The trained caster's school and caster style as their casting trained them (`balance_probe/schools.rs`): each day of each level, one pool spent on the pace spell (of the school's spells, the most a cast among those costing at most a fifth of the pool, never one cast only under the open sky; for the priest a heal, at most one a kill), every cast landing on a monster of the caster's level through `World::practise`. Each row: the pace spell over the level before and its cost (seed 1's); casts a day from the pool, and the spell casts a day the same level's fights took (the stat training's careers, which refill the pool before every cast; the priest's one a kill); the school's rank and the caster style's at the level (the seeds' range); and the pool check, the pool over a single-target spell of the caster's own level (its base mana: the level; no rank's cut, no talent).

**Wizard** (evocation)

| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |
|---|---|---|---|---|---|
| 1 | Burning hands (3) | 5.8 / 0.0 | 2 | – | 16 / 1 = 16.5 |
| 5 | Scorching ray (7) | 5.7 / 85.2 | 2–3 | Sniper 1–2 | 51 / 5 = 10.2 |
| 10 | Rime (14) | 6.0 / 125.1 | 3 | – | 99 / 10 = 9.9 |
| 15 | Chain lightning (19) | 7.2 / 197.1 | 4 | – | 152 / 15 = 10.1 |
| 20 | Chain lightning (19) | 10.2 / 164.6 | 4–5 | – | 208 / 20 = 10.4 |
| 30 | Chain lightning (19) | 16.2 / 239.9 | 6 | – | 331 / 30 = 11.0 |
| 40 | Sunfire (76) | 5.7 / 197.6 | 6 | – | 459 / 40 = 11.5 |
| 50 | Sunfire (58) | 9.7 / 287.1 | 7 | – | 590 / 50 = 11.8 |
| 60 | Sunfire (58) | 12.2 / 285.4 | 7 | – | 730 / 60 = 12.2 |

**Priest** (restoration)

| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |
|---|---|---|---|---|---|
| 1 | Mend (1) | 13.0 / 0.0 | 1 | – | 13 / 1 = 13.0 |
| 5 | Cure wounds (2) | 16.0 / 42.0 | 2 | – | 39 / 5 = 7.9 |
| 10 | Cure wounds (1) | 68.3 / 73.0 | 5 | – | 78 / 10 = 7.8 |
| 15 | Benison (21) | 35.1 / 98.5 | 6–7 | Battle caster 5–6 | 118 / 15 = 7.8 |
| 20 | Benison (23) | 6.0 / 74.3 | 6–7 | Battle caster 5–6 | 160 / 20 = 8.0 |
| 30 | Benison (25) | 9.2 / 88.0 | 6–7 | Battle caster 6–7 | 251 / 30 = 8.4 |
| 40 | Benison (26) | 12.5 / 75.0 | 7 | Battle caster 6–7 | 345 / 40 = 8.6 |
| 50 | Benison (26) | 16.3 / 83.2 | 7–8 | Battle caster 7 | 444 / 50 = 8.9 |
| 60 | Benison (25) | 20.8 / 91.2 | 8 | Battle caster 8 | 546 / 60 = 9.1 |

**Witch** (necromancy)

| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |
|---|---|---|---|---|---|
| 1 | Grave breath (3) | 4.0 / 0.0 | 1 | – | 14 / 1 = 13.7 |
| 5 | Cause wounds (3) | 11.7 / 86.7 | 2 | Toucher 2 | 43 / 5 = 8.6 |
| 10 | Leech (10) | 7.3 / 142.3 | 3 | Sniper 2 | 87 / 10 = 8.7 |
| 15 | Grave spear (23) | 7.5 / 217.2 | 4 | – | 136 / 15 = 9.1 |
| 20 | Grave spear (23) | 7.3 / 178.7 | 4–5 | – | 189 / 20 = 9.4 |
| 30 | Grave spear (23) | 12.2 / 309.9 | 5 | – | 304 / 30 = 10.1 |
| 40 | Grave spear (23) | 19.2 / 335.4 | 6–7 | – | 425 / 40 = 10.6 |
| 50 | Grave spear (18) | 29.7 / 493.3 | 7–8 | – | 556 / 50 = 11.1 |
| 60 | Grave spear (18) | 37.2 / 449.2 | 8 | – | 692 / 60 = 11.5 |

The level 1 row has no level before it and practises nothing: it says what a starting caster's first pool buys of the spells they know (`schools::first_spell`: an area before a single target, then the most mean roll; a healer's heal), #979.

These tables are #1251's run (the `school` curve at three times its marks, and a caster style trained only by the spells cast in it). The style column is the pace spell's: `–` when it is cast in no style (an area such as Rime or Chain lightning, a heal), so a Sniper rank the wizard trained at level 5 is not shown once the pace spell becomes an area. By level 60 the wizard reaches evocation 7 (it was 9), the priest restoration 8 (10), and the witch necromancy 8 (10).

Casts that land on a foe at least as strong as the rank (3 marks each) to reach a rank of a school or a caster style: rank 1 10 (was 4), rank 4 150 (was 50), rank 10 7,470 (was 2,490).

## Weapon pace

The trained fighters' weapon skill and fighting style as their blows trained them (#1095; `balance_probe/training.rs`): every blow of the stat training's fights, each level's kills against the monsters within two levels of theirs, earns its marks through `World::practise` (the real `[marks]`, curves, and talents), the main hand's skill and style for both hands. Each cell: the level at which the skill first stood at the rank (the range over the seeds that fought with it; `>60` where a seed had not reached it by level 60). The untrained careers take these ranks from their trained twin; the skills the fights do not train (a shield, a second weapon skill) are set by level (`rank`, the warrior's pace).

**Warrior**

| rank | Long blades | Sword and Board |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 4 | 4 |
| 4 | 5–6 | 5–6 |
| 5 | 7–8 | 8 |
| 6 | 9–11 | 10–11 |
| 7 | 14–16 | 15–16 |
| 8 | 20–25 | 23–25 |
| 9 | 28–35 | 32–35 |
| 10 | 37–45 | 41–45 |

**Thief**

| rank | Daggers and knives | Duelist (3 of 6 seeds) | Twin Blades (3 of 6 seeds) |
|---|---|---|---|
| 1 | 2 | 2 | 2 |
| 2 | 2 | 2 | 2 |
| 3 | 2–3 | 3–4 | 2–3 |
| 4 | 4–5 | 5 | 4–5 |
| 5 | 6–7 | 6–7 | 6–7 |
| 6 | 8–9 | 8–9 | 8 |
| 7 | 12–13 | 13 | 12–13 |
| 8 | 18–20 | 18–20 | 18–19 |
| 9 | 26–28 | 26–28 | 26–27 |
| 10 | 34–36 | 34–36 | 34–35 |

**Priest**

| rank | Maces and clubs | Sword and Board |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 3–4 | 3–4 |
| 4 | 5–6 | 5–6 |
| 5 | 7–8 | 7–8 |
| 6 | 9–10 | 9–10 |
| 7 | 14–15 | 14–15 |
| 8 | 22–23 | 22–23 |
| 9 | 30–33 | 30–33 |
| 10 | 38–43 | 38–43 |

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
| 2 | 3 | 2 | 2 | 3 | 2 | 2 |
| 3 | 4 | 3–4 | 4 | 4 | 4 | 3–4 |
| 4 | 6 | 5–6 | 5 | 6 | 5 | 5–6 |
| 5 | 8 | 7–8 | 7–8 | 8 | 7–8 | 7–8 |
| 6 | 11 | 9–10 | 10 | 11 | 10 | 9–10 |
| 7 | 17 | 14–15 | 15–16 | 17 | 15–16 | 14–15 |
| 8 | 25 | 21–22 | 23–26 | 25 | 23–26 | 21–22 |
| 9 | 36 | 30–31 | 33–37 | 36 | 33–37 | 30–31 |
| 10 | 46 | 38–39 | 43–49 | 46 | 43–49 | 38–39 |

**Paladin**

| rank | Long blades | Sword and Board |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 4 | 4 |
| 4 | 5–6 | 5–6 |
| 5 | 8 | 8 |
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
| 1 | 15 | 1d6 | 77% | 72/78 · 2.6:6.0 · ×2.3 | 72/78 · 2.6:5.9 · ×2.3 |
| 5 | 29 | 2d6 | 60% | 79/88 · 4.0:10.5 · ×2.6 | 80/89 · 5.2:11.8 · ×2.3 |
| 10 | 47 | 2d6 | 73% | 91/97 · 8.0:15.2 · ×1.9 | 94/98 · 9.9:17.1 · ×1.7 |
| 15 | 64 | 3d6 | 63% | 98/99 · 9.8:20.5 · ×2.1 | 99/99 · 12.2:22.8 · ×1.9 |
| 20 | 82 | 4d6 | 64% | 99/99 · 12.0:26.0 · ×2.2 | 99/99 · 13.7:27.7 · ×2.0 |
| 30 | 117 | 5d6 | 64% | 99/99 · 13.9:31.4 · ×2.3 | 99/99 · 25.8:43.3 · ×1.7 |
| 40 | 152 | 8d6 | 61% | 99/99 · 16.4:44.5 · ×2.7 | 99/99 · 23.5:51.4 · ×2.2 |
| 50 | 187 | 8d6 | 54% | 99/99 · 16.8:44.4 · ×2.6 | 99/99 · 27.0:54.7 · ×2.0 |
| 60 | 222 | 9d6 | 50% | 97/99 · 16.7:50.8 · ×3.0 | 99/99 · 25.4:60.0 · ×2.4 |

## Rage

The berserker in plain gear (a great axe; from level 5 the warrior's armour: chain mail, banded mail from 15, plate from 30), trained stats, against the monsters within two levels: calm (the stance the run starts in) and raging (Raging and the frenzy stance). The pool as the Rage ability grants it (start / most); the fight cells as in "Fights"; then the rage a round against one foe, the medians of hits taken × their build + criticals × theirs + kills × theirs − the decay (kills a round: one over the rounds to kill, so a fight's average); the same without the kills (a foe that outlasts the rage, or the turns before the first kill) and how many rounds the rage lasts from its start at that net (`grows` when it does not fall); and the net against three foes (three foes' hits, the same criticals and kills).

| L | rage | Calm | Raging | one foe: struck + crit + kill − decay = net | one foe that does not die: net · rounds the rage lasts | three foes: net |
|---|---|---|---|---|---|---|
| 1 | 12/20 | 83/71 · 8.3:0.7 · 0.9 / 39 | 94/96 · 13.7:1.0 · 0.5 / 31 | 0.98 + 0.07 + 5.79 − 1.00 = +5.84 | +0.06 · grows | +7.81 |
| 5 | 12/20 | 83/64 · 7.5:1.2 · 2.7 / 51 | 94/94 · 13.0:1.6 · 1.7 / 32 | 0.95 + 0.05 + 1.94 − 1.00 = +1.94 | +0.00 · grows | +3.84 |
| 10 | 12/20 | 88/79 · 11.8:3.4 · 3.8 / 25 | 97/98 · 16.6:3.5 · 2.6 / 24 | 0.98 + 0.07 + 1.22 − 1.00 = +1.26 | +0.05 · grows | +3.23 |
| 15 | 12/20 | 90/92 · 12.2:5.4 · 5.4 / 22 | 98/99 · 17.8:4.7 · 3.6 / 26 | 0.99 + 0.06 + 0.88 − 1.00 = +0.93 | +0.05 · grows | +2.91 |
| 20 | 12/20 | 93/95 · 16.1:6.8 · 5.5 / 25 | 99/99 · 23.0:6.5 · 3.7 / 25 | 1.00 + 0.08 + 0.82 − 1.00 = +0.89 | +0.07 · grows | +2.88 |
| 30 | 12/20 | 94/99 · 23.5:15.4 · 6.8 / 17 | 99/99 · 30.9:13.9 · 5.4 / 18 | 1.01 + 0.13 + 0.57 − 0.89 = +0.82 | +0.25 · grows | +2.84 |
| 40 | 18/20 | 95/99 · 27.9:17.6 · 6.7 / 22 | 99/99 · 53.2:15.9 · 3.9 / 24 | 1.05 + 0.19 + 2.15 − 0.67 = +2.72 | +0.57 · grows | +4.82 |
| 50 | 18/30 | 95/99 · 43.7:24.3 · 6.9 / 21 | 99/99 · 77.1:22.7 · 3.7 / 22 | 1.08 + 0.25 + 2.21 − 0.67 = +2.87 | +0.66 · grows | +5.04 |
| 60 | 18/30 | 88/99 · 46.3:28.1 · 5.1 / 24 | 98/99 · 77.9:26.2 · 3.5 / 26 | 1.23 + 0.17 + 2.37 − 0.67 = +3.09 | +0.73 · grows | +5.56 |

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
| 1 | 5.3 | 0.33 | 80/63 · 5.5:1.1 · 1.4 / 20 | 80/63 · 10.2:1.1 · 0.7 / 20 | 0.99 − 0.33 = 0.66 · 8.1 |
| 5 | 5.3 | 0.33 | 89/70 · 7.8:3.5 · 2.7 / 9.3 | 89/70 · 14.6:3.5 · 1.5 / 9.3 | 1.00 − 0.33 = 0.67 · 8.0 |
| 10 | 6.3 | 0.33 | 94/81 · 14.5:6.8 · 2.9 / 7.7 | 94/81 · 28.1:6.9 · 1.6 / 7.6 | 1.13 − 0.33 = 0.80 · 8.0 |
| 15 | 7.7 | 0.33 | 98/85 · 19.3:10.3 · 3.4 / 7.9 | 98/85 · 38.0:10.2 · 1.7 / 7.9 | 1.20 − 0.33 = 0.87 · 8.8 |
| 20 | 8.8 | 0.33 | 99/87 · 20.9:13.3 · 4.0 / 8.7 | 99/87 · 40.8:13.3 · 2.1 / 8.7 | 1.22 − 0.33 = 0.89 · 9.9 |
| 30 | 11.5 | 0.33 | 99/96 · 26.2:22.9 · 5.3 / 7.8 | 99/96 · 51.8:22.8 · 2.7 / 7.9 | 1.32 − 0.33 = 0.99 · 12 |
| 40 | 13.3 | 0.33 | 99/97 · 35.5:25.8 · 4.7 / 10.0 | 99/97 · 70.8:25.8 · 2.4 / 10 | 1.43 − 0.33 = 1.10 · 12 |
| 50 | 15.7 | 0.33 | 99/97 · 46.0:30.6 · 5.9 / 11 | 99/97 · 92.0:30.7 · 2.9 / 11 | 1.54 − 0.33 = 1.20 · 13 |
| 60 | 20.0 | 0.83 | 99/97 · 54.6:37.0 · 5.1 / 9.2 | 99/97 · 104.8:37.1 · 2.6 / 9.1 | 1.55 − 0.83 = 0.71 · 28 |

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
| 1 | 10.8 | 0.10 | 78/49 · 6.9:0.1 · 1.1 / 663 | 83/49 · 10.4:0.1 · 0.7 / 685 | 87/41 · 13.5:0.8 · 0.8 / 29 | - | 1.37 · 2.2 | 1.85 · 1.6 |
| 5 | 10.8 | 0.10 | 84/46 · 5.8:1.0 · 3.7 / 36 | 87/46 · 8.6:1.0 · 2.5 / 36 | 91/55 · 11.3:1.3 · 1.8 / 28 | - | 1.23 · 2.4 | 1.69 · 1.8 |
| 10 | 11.8 | 0.10 | 92/48 · 9.3:2.8 · 4.6 / 22 | 94/48 · 12.6:2.8 · 3.4 / 23 | 94/45 · 16.0:3.1 · 3.3 / 21 | - | 1.32 · 2.3 | 1.80 · 1.7 |
| 15 | 11.8 | 0.10 | 94/56 · 9.5:3.9 · 6.9 / 24 | 96/56 · 12.6:4.0 · 5.1 / 23 | 93/56 · 16.6:4.5 · 3.2 / 20 | - | 1.41 · 2.1 | 1.98 · 1.5 |
| 20 | 12.8 | 0.10 | 96/54 · 11.2:4.8 · 7.7 / 29 | 98/54 · 17.2:4.7 · 4.9 / 28 | 97/59 · 20.6:6.5 · 3.8 / 20 | - | 1.42 · 2.1 | 1.96 · 1.5 |
| 30 | 14.2 | 0.10 | 97/65 · 13.1:10.4 · 12 / 20 | 99/65 · 22.8:10.3 · 6.6 / 20 | 97/75 · 25.9:14.4 · 3.8 / 14 | - | 1.53 · 2.0 | 2.17 · 1.4 |
| 40 | 15.7 | 0.10 | 97/74 · 16.0:13.0 · 12 / 23 | 99/74 · 29.5:13.0 · 6.3 / 23 | 94/84 · 32.3:21.7 · 4.6 / 13 | - | 1.62 · 1.9 | 2.35 · 1.3 |
| 50 | 17.2 | 0.10 | 97/57 · 18.0:14.4 · 15 / 28 | 99/57 · 35.1:14.3 · 7.7 / 27 | 99/72 · 41.5:19.8 · 8.1 / 20 | 99/72 · 62.4:19.7 · 5.4 / 20 | 1.47 · 2.0 | 2.04 · 1.5 |
| 60 | 18.3 | 0.10 | 90/56 · 17.8:14.1 · 14 / 33 | 97/56 · 36.6:14.3 · 6.6 / 31 | 96/96 · 41.5:36.3 · 4.0 / 11 | 96/96 · 63.6:36.0 · 2.6 / 11 | 1.41 · 2.1 | 1.96 · 1.5 |

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
| 1 | hound, boar | 9 | 73/54 · 4.5:0.6 · 1.8 / 47 | 71/71 · 4.4:1.5 · 2.3 / 6.0 | 1.0 | x1.8 |
| 5 | hound, boar | 16 | 79/62 · 4.6:2.6 · 4.6 / 13 | 73/77 · 4.1:3.9 · 7.0 / 4.0 | 2.8 | x1.7 |
| 10 | hound, boar | 27 | 79/75 · 5.7:6.4 · 8.1 / 6.8 | 82/84 · 6.7:7.6 · 8.4 / 3.5 | 4.1 | x2.0 |
| 15 | hound, boar | 38 | 84/80 · 7.0:9.1 · 9.8 / 6.1 | 86/88 · 10.6:11.1 · 7.8 / 3.4 | 4.3 | x2.3 |
| 20 | hound, boar | 49 | 79/89 · 5.8:13.1 · 16 / 5.2 | 91/87 · 12.6:12.9 · 8.6 / 3.8 | 5.5 | x2.8 |
| 30 | hound, boar | 71 | 77/96 · 5.6:21.8 · 25 / 4.3 | 97/93 · 19.5:21.2 · 8.4 / 3.3 | 6.3 | x4.0 |
| 40 | hound, boar | 93 | 67/99 · 6.5:25.8 · 30 / 4.9 | 98/93 · 26.3:22.4 · 6.8 / 4.2 | 5.5 | x5.5 |
| 50 | hound, boar | 116 | 66/99 · 7.3:32.4 · 39 / 4.5 | 99/91 · 34.3:27.6 · 8.0 / 4.3 | 6.6 | x5.9 |
| 60 | hound, boar | 137 | 50/99 · 3.7:37.5 · 59 / 4.0 | 99/86 · 35.9:28.2 · 6.4 / 5.1 | 5.8 | x10.2 |

What it says (2026-11-03, C8): the hound (grade the ranger's level) out-damages the melee ranger from level 5 on and by level 60 deals about nine times the short sword's damage a round (34.8 against 3.8; the ranger's real weapon is the bow, which this probe does not measure, so read the ranger column as a floor). It hits 71 to 99 % of the time against monsters of the ranger's level and kills one in about seven rounds at every level, and it lasts about five to six rounds against them, the same as the ranger does. The two together kill 1.6 to 6.3 rounds; the companion is the larger half of the pair from level 5. `grade_percent = 100` (the design's Q1) stands (the beast's level is the ranger's); whether a hound that out-damages its melee master nine to one at level 60 is too much is the owner's call (a tamed wolf is not measured here: it is the same growth from a higher base, and the probe's seeds draw only the hound and the boar).

## Summons

The druid's calls (#839), cast by a human druid (untrained stats, every score as rolled, seed 1) at each level a call comes or Pack leader (32) and Lord of the wild (60) change them: what comes of one cast (each type's count), for how many minutes (`lasting`: until slain), the mana over the pool, and each called type against the monsters within two levels of the druid, as in "Fights" (the beast's side bare: P(it hits) / P(it is hit) % · damage a round · rounds to kill / rounds to die).

| Call | L | comes | minutes | mana / pool | the called against the druid's level |
|---|---|---|---|---|---|
| Call of the wild (5) | 6 | 1 wolf | 5 | 5 / 43 (9 casts) | wolf 4: 81/75 · 5.1:4.2 · 4.4 / 3.8 |
| Call of the wild (5) | 25 | 1 wolf | 5 | 5 / 161 (32 casts) | wolf 4: 69/99 · 3.1:20.1 · 32 / 0.8 |
| Call of the forest (20) | 25 | 1 bear | 5 | 20 / 161 (8 casts) | bear 12: 93/99 · 10.0:20.6 · 11 / 2.9 |
| Call of the wild (5) | 32 | 2 wolves | 7 | 5 / 204 (41 casts) | wolf 4: 62/99 · 2.3:26.0 · 58 / 0.6 |
| Call of the forest (20) | 32 | 2 bears | 7 | 20 / 204 (10 casts) | bear 12: 90/99 · 7.9:26.2 · 19 / 2.3 |
| Call of the wild (5) | 40 | 2 wolves | 7 | 5 / 253 (51 casts) | wolf 4: 42/99 · 1.5:27.6 · 106 / 0.6 |
| Call of the forest (20) | 40 | 2 bears | 7 | 20 / 253 (13 casts) | bear 12: 78/99 · 7.3:27.6 · 26 / 2.2 |
| Call the elements (35) | 40 | 1 greater air elemental, 1 greater fire elemental | 7 | 35 / 253 (7 casts) | greater earth elemental 24: 91/99 · 15.8:17.0 · 12 / 11; greater water elemental 26: 91/99 · 15.4:21.6 · 13 / 4.0; greater air elemental 31: 99/93 · 37.8:20.6 · 4.9 / 3.0; greater fire elemental 38: 99/99 · 26.9:24.8 · 7.8 / 4.9 |
| Call of the wild (5) | 50 | 2 wolves | 7 | 5 / 314 (63 casts) | wolf 4: 31/99 · 1.0:34.8 · 253 / 0.5 |
| Call of the forest (20) | 50 | 2 bears | 7 | 20 / 314 (16 casts) | bear 12: 69/99 · 6.2:34.8 · 42 / 1.7 |
| Call the elements (35) | 50 | 1 greater air elemental, 1 greater fire elemental | 7 | 35 / 314 (9 casts) | greater earth elemental 24: 85/99 · 15.3:21.2 · 18 / 8.6; greater water elemental 26: 85/99 · 13.9:28.0 · 19 / 3.1; greater air elemental 31: 99/99 · 35.2:26.7 · 7.7 / 2.3; greater fire elemental 38: 99/99 · 26.7:29.3 · 9.9 / 4.2 |
| Call of the wild (5) | 60 | 2 wolves | lasting | 5 / 374 (75 casts) | wolf 4: 15/99 · 0.3:36.0 · 714 / 0.4 |
| Call of the forest (20) | 60 | 2 bears | lasting | 20 / 374 (19 casts) | bear 12: 46/99 · 2.9:35.5 · 57 / 1.7 |
| Call the elements (35) | 60 | 1 greater air elemental, 1 greater fire elemental | lasting | 35 / 374 (11 casts) | greater earth elemental 24: 69/99 · 10.8:24.3 · 21 / 7.6; greater water elemental 26: 69/99 · 9.9:29.9 · 23 / 2.9; greater air elemental 31: 93/99 · 31.5:27.0 · 8.0 / 2.3; greater fire elemental 38: 96/99 · 26.7:31.0 · 9.1 / 3.9 |

## Familiar

The witch's familiar (#929, #933), a human witch (untrained stats, every score as rolled, seed 1) with each of the five kinds a run may draw in turn, at its grade (her level). Its touch is her Evil eye and needs a blow that lands, so the table is the familiar's side bare against the monsters within two levels of the witch's (as in "Fights", the monsters' gear left out): **hit** the chance a blow lands, lowest / median / highest over the window; **blows** a round (a normal turn); **hexes** laid a round (blows × median hit); **rounds to the first** hex (one over the hexes a round); **degrees** her Evil eye lays; **eye** the mana one cast of it costs over her pool, which every landed blow saves her; the familiar's hit points, mean damage of a hit, and to-hit.

| L | familiar | hit % (low / median / high) | blows | hexes a round | rounds to the first | degrees | eye (mana / pool) | hp · mean hit · to hit |
|---|---|---|---|---|---|---|---|---|
| 1 | cat | 19 / 31 / 81 | 1.30 | 0.41 | 2.5 | 1 | 1 / 13 | 4 · 1.0 · -4 |
| 1 | toad | 19 / 31 / 81 | 0.90 | 0.28 | 3.6 | 1 | 1 / 13 | 4 · 1.0 · -4 |
| 1 | raven | 19 / 31 / 81 | 1.30 | 0.41 | 2.5 | 1 | 1 / 13 | 4 · 1.0 · -4 |
| 1 | hare | 25 / 38 / 85 | 1.60 | 0.61 | 1.6 | 1 | 1 / 13 | 4 · 1.0 · -3 |
| 1 | bat | 15 / 25 / 75 | 2.00 | 0.50 | 2.0 | 1 | 1 / 13 | 4 · 1.0 · -5 |
| 5 | cat | 4 / 38 / 75 | 1.30 | 0.50 | 2.0 | 1 | 1 / 42 | 12 · 1.0 · -3 |
| 5 | toad | 4 / 38 / 75 | 0.90 | 0.34 | 2.9 | 1 | 1 / 42 | 12 · 1.0 · -3 |
| 5 | raven | 4 / 38 / 75 | 1.30 | 0.50 | 2.0 | 1 | 1 / 42 | 12 · 1.0 · -3 |
| 5 | hare | 4 / 46 / 81 | 1.60 | 0.73 | 1.4 | 1 | 1 / 42 | 12 · 1.0 · -2 |
| 5 | bat | 4 / 31 / 69 | 2.00 | 0.62 | 1.6 | 1 | 1 / 42 | 12 · 1.0 · -4 |
| 10 | cat | 10 / 62 / 85 | 1.30 | 0.80 | 1.2 | 1 | 1 / 82 | 23 · 1.0 · -1 |
| 10 | toad | 7 / 54 / 81 | 0.90 | 0.49 | 2.1 | 1 | 1 / 82 | 23 · 1.0 · -2 |
| 10 | raven | 7 / 54 / 81 | 1.30 | 0.70 | 1.4 | 1 | 1 / 82 | 23 · 1.0 · -2 |
| 10 | hare | 10 / 62 / 85 | 1.60 | 0.99 | 1.0 | 1 | 1 / 82 | 23 · 1.0 · -1 |
| 10 | bat | 7 / 54 / 81 | 2.00 | 1.08 | 0.9 | 1 | 1 / 82 | 23 · 1.0 · -2 |
| 15 | cat | 15 / 62 / 85 | 1.30 | 0.80 | 1.2 | 1 | 1 / 118 | 34 · 2.0 · +0 |
| 15 | toad | 15 / 62 / 85 | 0.90 | 0.56 | 1.8 | 1 | 1 / 118 | 34 · 2.0 · +0 |
| 15 | raven | 15 / 62 / 85 | 1.30 | 0.80 | 1.2 | 1 | 1 / 118 | 34 · 3.0 · +0 |
| 15 | hare | 19 / 69 / 90 | 1.60 | 1.10 | 0.9 | 1 | 1 / 118 | 34 · 3.0 · +1 |
| 15 | bat | 10 / 54 / 81 | 2.00 | 1.08 | 0.9 | 1 | 1 / 118 | 34 · 2.0 · -1 |
| 20 | cat | 25 / 69 / 93 | 1.30 | 0.89 | 1.1 | 1 | 1 / 157 | 45 · 4.0 · +2 |
| 20 | toad | 19 / 62 / 90 | 0.90 | 0.56 | 1.8 | 1 | 1 / 157 | 45 · 3.0 · +1 |
| 20 | raven | 19 / 62 / 90 | 1.30 | 0.80 | 1.2 | 1 | 1 / 157 | 45 · 4.0 · +1 |
| 20 | hare | 25 / 69 / 93 | 1.60 | 1.10 | 0.9 | 1 | 1 / 157 | 45 · 4.0 · +2 |
| 20 | bat | 19 / 62 / 90 | 2.00 | 1.24 | 0.8 | 1 | 1 / 157 | 45 · 4.0 · +1 |
| 30 | cat | 25 / 81 / 93 | 1.30 | 1.05 | 1.0 | 1 | 1 / 230 | 67 · 7.5 · +6 |
| 30 | toad | 19 / 75 / 90 | 0.90 | 0.68 | 1.5 | 1 | 1 / 230 | 67 · 6.5 · +5 |
| 30 | raven | 19 / 75 / 90 | 1.30 | 0.97 | 1.0 | 1 | 1 / 230 | 67 · 8.0 · +5 |
| 30 | hare | 25 / 81 / 93 | 1.60 | 1.29 | 0.8 | 1 | 1 / 230 | 67 · 7.5 · +6 |
| 30 | bat | 19 / 75 / 90 | 2.00 | 1.50 | 0.7 | 1 | 1 / 230 | 67 · 8.0 · +5 |
| 40 | cat | 46 / 85 / 96 | 1.30 | 1.11 | 0.9 | 1 | 1 / 304 | 89 · 12.0 · +9 |
| 40 | toad | 38 / 81 / 93 | 0.90 | 0.72 | 1.4 | 1 | 1 / 304 | 89 · 11.0 · +8 |
| 40 | raven | 38 / 81 / 93 | 1.30 | 1.05 | 1.0 | 1 | 1 / 304 | 89 · 13.0 · +8 |
| 40 | hare | 46 / 85 / 96 | 1.60 | 1.37 | 0.7 | 1 | 1 / 304 | 89 · 12.0 · +9 |
| 40 | bat | 38 / 81 / 93 | 2.00 | 1.61 | 0.6 | 1 | 1 / 304 | 89 · 13.0 · +8 |
| 50 | cat | 62 / 93 / 99 | 1.30 | 1.21 | 0.8 | 1 | 1 / 377 | 111 · 16.5 · +13 |
| 50 | toad | 54 / 90 / 98 | 0.90 | 0.81 | 1.2 | 1 | 1 / 377 | 111 · 15.5 · +12 |
| 50 | raven | 54 / 90 / 98 | 1.30 | 1.16 | 0.9 | 1 | 1 / 377 | 111 · 18.0 · +12 |
| 50 | hare | 62 / 93 / 99 | 1.60 | 1.49 | 0.7 | 1 | 1 / 377 | 111 · 16.5 · +13 |
| 50 | bat | 54 / 90 / 98 | 2.00 | 1.79 | 0.6 | 1 | 1 / 377 | 111 · 18.0 · +12 |
| 60 | cat | 46 / 93 / 99 | 1.30 | 1.21 | 0.8 | 1 | 1 / 450 | 133 · 18.5 · +16 |
| 60 | toad | 38 / 90 / 99 | 0.90 | 0.81 | 1.2 | 1 | 1 / 450 | 133 · 17.5 · +15 |
| 60 | raven | 38 / 90 / 99 | 1.30 | 1.16 | 0.9 | 1 | 1 / 450 | 133 · 20.0 · +15 |
| 60 | hare | 46 / 93 / 99 | 1.60 | 1.49 | 0.7 | 1 | 1 / 450 | 133 · 18.5 · +16 |
| 60 | bat | 38 / 90 / 99 | 2.00 | 1.79 | 0.6 | 1 | 1 / 450 | 133 · 20.0 · +15 |

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
| warrior | St | 14.8/16.7 | 17.3 | 20.5 | 23.0 | 25.0 | 28.2 | +13.3 | 11.5 |
|  | Dx | 10.8/16.3 | 12.2 | 17.0 | 19.0 | 20.5 | 22.7 | +11.8 | 6.3 |
|  | To | 11.7/16.0 | 13.8 | 18.2 | 19.8 | 21.0 | 23.8 | +12.2 | 7.8 |
|  | In | 9.8/17.0 | 11.5 | 13.2 | 14.5 | 16.0 | 17.5 | +7.7 | 0.5 |
|  | Wi | 11.7/17.7 | 11.8 | 13.3 | 14.3 | 16.0 | 18.3 | +6.7 | 0.7 |
|  | Pr | 10.0/16.2 | 10.0 | 10.7 | 11.2 | 12.3 | 14.0 | +4.0 | 0.0 |
| | all six | | | | | | | **+55.7** | |
| thief | St | 11.0/16.8 | 16.0 | 20.3 | 22.7 | 24.5 | 27.3 | +16.3 | 10.5 |
|  | Dx | 12.5/16.5 | 15.8 | 19.7 | 21.3 | 22.3 | 24.3 | +11.8 | 7.8 |
|  | To | 10.5/16.8 | 14.7 | 19.0 | 20.2 | 21.5 | 24.2 | +13.7 | 7.3 |
|  | In | 11.3/17.2 | 12.8 | 14.3 | 15.7 | 16.8 | 18.2 | +6.8 | 1.0 |
|  | Wi | 10.3/17.3 | 10.3 | 11.8 | 13.0 | 14.3 | 17.0 | +6.7 | 0.2 |
|  | Pr | 9.8/16.3 | 9.8 | 10.7 | 11.2 | 12.0 | 13.8 | +4.0 | 0.0 |
| | all six | | | | | | | **+59.3** | |
| wizard | St | 10.0/16.7 | 10.0 | 10.0 | 10.5 | 11.0 | 12.0 | +2.0 | 0.0 |
|  | Dx | 10.0/16.3 | 10.8 | 12.5 | 13.8 | 15.8 | 18.0 | +8.0 | 1.7 |
|  | To | 9.5/17.3 | 12.0 | 17.5 | 19.5 | 20.8 | 22.5 | +13.0 | 5.2 |
|  | In | 14.5/16.8 | 16.5 | 18.7 | 20.0 | 21.2 | 23.2 | +8.7 | 6.3 |
|  | Wi | 12.5/17.7 | 16.2 | 20.2 | 21.7 | 23.5 | 25.8 | +13.3 | 8.2 |
|  | Pr | 9.5/16.0 | 9.5 | 10.3 | 10.8 | 12.0 | 14.0 | +4.5 | 0.0 |
| | all six | | | | | | | **+49.5** | |
| priest | St | 10.8/16.3 | 15.8 | 19.5 | 22.3 | 24.5 | 26.3 | +15.5 | 10.0 |
|  | Dx | 8.7/16.3 | 9.8 | 13.3 | 15.3 | 17.5 | 19.2 | +10.5 | 2.8 |
|  | To | 10.7/16.7 | 14.8 | 19.7 | 21.7 | 23.5 | 27.0 | +16.3 | 10.3 |
|  | In | 11.3/16.7 | 12.5 | 14.3 | 16.0 | 17.0 | 17.8 | +6.5 | 1.2 |
|  | Wi | 15.3/18.0 | 17.0 | 19.3 | 20.5 | 21.2 | 23.2 | +7.8 | 5.2 |
|  | Pr | 10.8/16.3 | 10.8 | 11.5 | 11.8 | 12.8 | 14.5 | +3.7 | 0.0 |
| | all six | | | | | | | **+60.3** | |

Marks a day the hooks gave in each stat over the level before (St/Dx/To/In/Wi/Pr, the seeds' mean), and the game day reached.

| L | Day | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| 5 | 4 | 18/6/18/20/3/3 | 44/32/46/21/3/3 | 1/14/21/38/44/3 | 23/7/52/20/17/4 |
| 10 | 14 | 72/30/36/6/3/1 | 94/68/46/6/3/1 | 1/5/27/14/64/2 | 67/12/71/6/26/2 |
| 15 | 24 | 126/40/68/6/3/2 | 165/107/53/6/3/2 | 1/5/38/43/87/1 | 114/12/104/6/36/2 |
| 20 | 39 | 211/34/57/4/3/1 | 146/97/44/4/3/1 | 1/5/30/35/73/1 | 108/16/89/4/25/1 |
| 30 | 69 | 330/64/89/3/3/1 | 219/112/63/4/3/1 | 0/5/37/50/120/2 | 166/13/185/4/29/2 |
| 40 | 109 | 320/49/100/2/3/1 | 236/79/70/2/3/1 | 1/4/30/39/98/1 | 288/9/197/2/26/1 |
| 50 | 149 | 380/62/125/2/3/1 | 336/85/111/2/3/1 | 1/7/42/55/143/1 | 192/8/222/2/28/1 |
| 60 | 189 | 270/75/216/2/3/1 | 346/79/168/2/3/1 | 1/8/47/54/132/1 | 213/14/394/2/30/1 |

The game day of each stat's first rise (the seeds' mean of those that rose by level 60; how many of the six in brackets when not all).

| Class | St | Dx | To | In | Wi | Pr |
|---|---|---|---|---|---|---|
| warrior | 7 | 11 | 7 | 6 | 19 | 37 |
| thief | 5 | 6 | 5 | 6 | 24 | 34 |
| wizard | 68 | 11 | 7 | 6 | 5 | 32 |
| priest | 6 | 10 | 5 | 7 | 9 | 39 |

The knobs (design §6.2): mean gain by level 60 with other `price_per_point` and `ramp_divisor` values (St/Dx/To/In/Wi/Pr · all six), the data's in bold.

| Price | Ramp ÷ | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| **20 + 10s** | **2** | +13.3/+11.8/+12.2/+7.7/+6.7/+4.0 · +56 | +16.3/+11.8/+13.7/+6.8/+6.7/+4.0 · +59 | +2.0/+8.0/+13.0/+8.7/+13.3/+4.5 · +50 | +15.5/+10.5/+16.3/+6.5/+7.8/+3.7 · +60 |
| 20 + 10s | 3 | +14.2/+12.7/+13.3/+7.8/+6.5/+3.8 · +58 | +17.8/+12.7/+14.3/+7.2/+6.8/+4.2 · +63 | +1.8/+8.3/+13.7/+9.2/+14.3/+4.3 · +52 | +16.5/+10.8/+17.3/+6.8/+8.5/+3.8 · +64 |
| 20 + 10s | 4 | +15.3/+13.0/+14.2/+7.8/+6.7/+4.5 · +62 | +18.8/+13.3/+15.3/+7.2/+7.3/+4.3 · +66 | +1.8/+8.3/+14.5/+10.2/+15.0/+4.3 · +54 | +17.3/+10.8/+18.3/+6.8/+8.8/+4.2 · +66 |
| 20 + 10s | 6 | +16.5/+13.7/+15.5/+7.7/+6.3/+3.8 · +64 | +20.2/+14.2/+16.5/+7.3/+7.0/+4.3 · +70 | +1.7/+8.7/+15.0/+10.8/+16.3/+4.3 · +57 | +18.5/+11.3/+19.3/+6.7/+9.7/+3.8 · +69 |
| 20 + 15s | 2 | +12.5/+10.7/+11.5/+6.3/+5.2/+2.7 · +49 | +15.2/+10.7/+13.0/+5.8/+6.0/+3.5 · +54 | +1.0/+7.2/+12.0/+7.8/+12.0/+3.2 · +43 | +15.2/+9.2/+15.0/+5.7/+7.0/+3.2 · +55 |
| 20 + 15s | 4 | +14.2/+12.2/+13.5/+6.5/+5.0/+3.2 · +54 | +17.2/+12.5/+14.0/+5.7/+6.0/+2.8 · +58 | +1.2/+7.3/+13.3/+8.8/+13.7/+3.2 · +47 | +16.5/+10.0/+16.7/+5.8/+7.7/+2.8 · +60 |
| 20 + 20s | 4 | +12.8/+11.5/+12.2/+5.7/+4.3/+2.2 · +49 | +16.2/+11.2/+13.5/+5.5/+4.8/+2.5 · +54 | +1.2/+6.3/+13.0/+8.2/+12.8/+2.3 · +44 | +15.8/+8.7/+16.2/+5.3/+7.5/+2.3 · +56 |

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
   weapon they roll than with their natural attack. The geared mean of damage a blow was below the
   bare median from level 15 up (`gear.toml`, the creature files); since #1169's ranks it is
   about level with it (above at 15, 30, and 70, below at 20, 40, and 50).

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
