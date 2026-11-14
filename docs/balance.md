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
"Starters"; the stat training of the first levels moves a little with the shorter fights), and
again after #1263 (the schools of magic no longer trained skills: a spell's roll and aim come of
its ladder, the talents, the items, and the caster style alone; "Spells" and "Caster style pace"
below), and again after #1264 (the mana floor of 4: every spell of level 0 to 4 priced as a
level-4 spell, so a cantrip costs 4 and Burning hands 6, which "Spells" shows in its mana column
and "Casters" in the monsters' casts: a kobold shaman's pool buys one Spark where it bought five,
a goblin wizard's five Burning hands where it bought eleven; spell ranks are not in the probe,
which casts every spell at rank 1), and again after #1328 (a skill rolls by itself when its
marks reach 100: the trained careers' days now raise their busiest skills between levels, so
the trained careers' numbers move a little (against the same code with `auto_at = 0`); every
table was regenerated, so they also take in the drift of the PRs since the last run), and again
after #1468 (every level a third cheaper in XP: a level is two thirds of the kills it was, so
the trained careers fight, and train, a third less between levels, and their stats, weapon
ranks, and caster styles come later by level; the prose quoting earlier runs is not rewritten),
and again after #1662 (every gameplay draw keyed by its rule and moment instead of one shared
stream: the probe's own streams did not change, but the trained careers' fights roll on the new
keys, so their marks, stats, and ranks by level move a little and every table with them; the
hand-kept tables, "Before #1200" under Starters and the design's under Stat training, stay).

## How to run it

```text
cargo test --release -p warlock-core balance_probe -- --ignored --nocapture
```

It takes about three minutes in release (most of it the stat training's sweep, a hundred and
seventy careers to level 60). It prints the Markdown tables of the sections from "Players" to
"Ki" below, and those of "Stat training", and the same content always prints the same tables (fixed seeds and streams).
After a PR that tunes combat data (weapon skills, creatures, classes, spells, stats, items), run
it again and replace those sections, then revisit the tuning notes. The code is
`crates/warlock-core/src/world/combat/balance_probe.rs` (the measuring, with the method in its module
docs), `balance_probe/training.rs` (the trained careers' days), `balance_probe/ranger.rs`
(the ranger and their companion), `balance_probe/familiar.rs` (the witch's familiar),
`balance_probe/caster_pace.rs` (the casters' caster style pace), `balance_probe/exploits.rs`
(the thief's Exploit and the openings a fight yields), `balance_probe/spread.rs` (the spread of rounds to kill and to die), and `balance_probe/tables.rs` (the tables).

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
  and the witch's dagger two ranks lower. A caster style is not set by level: it is trained by
  casting ("Caster style pace").
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
  reach, their caster style's rank, the talents' and items' hooks; nothing holds the roll
  (decision 16). Beside it the same roll with the style at rank 0, and the mean of a roll counts its rerolls exactly. No casts to kill: what a
  spell is worth against a monster is the content's question; the probe's is the roll and the
  pool. The priest's prayer (#886; `docs/dev/prayer.md`) has no roll and no mana: its line a
  level gives the name the menu shows and what one prayer answers of the uses reached.
- **Caster style pace**: before each level, each of its days the caster spends one pool on the
  spell of their school (as a category) they cast most (of the most mean roll among those
  costing at most a fifth of the pool; none cast only under the open sky; the priest's a heal,
  at most one a kill), and every cast lands on a monster of their level through
  `World::practise` (the caster style it is cast in): the real `[marks]` and curve. The ranks it
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
- **Exploits** (#1464, `balance_probe/exploits.rs`): the thief's Exploit of 1, 3, and 5
  openings on the same monsters, aware of them: the foe keeps its Defence, the profile is
  `ExploitMods::profile` (Low blow's to-hit, Master thief's threat and sure critical, as
  `World::exploit_profile`), then the sneak attack's own lower threat, and a hit lands the sneak
  dice and n × the ladder's dice; from level 18 the thrown exploit of 3 (#1465) on the first
  throwing weapon carried, the same ladder on its thrown profile. Beside it the openings a fight against one of those monsters
  yields under a feint-when-aware policy: the first blow (unaware) builds one on a hit, the
  thief then feints until the foe falls for it (its Willpower check, `feint.rs`'s difficulty),
  each feint made builds one and sets up a flat-footed sneak attack that builds one more on a
  hit, and the mark's kill gives one. Only the inflow: spending is not modelled.
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
- **Outside the probe**: the profane gifts (`docs/dev/profane-gifts.md`; design
  `docs/design/profane-gifts.md` §12). Eating a keeper's corpse is optional and late, its draws
  are a gamble by design, and a "with every signature" column is not worth the probe's time:
  the design's §12 judges each keeper's gift against the depth it sits at instead.

## Players

Each cell: to-hit / damage a hit / Dv / Pv, the seeds' mean; trained stats.

| L | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|
| 1 | +3/8.2/14/3 | +3/8.2/14/3 | +1/3.2/13/1 | +1/3.2/13/1 | +0/3.0/12/0 | +1/5.8/11/0 | +1/5.8/11/0 |
| 5 | +4/8.2/16/4 | +5/9.2/17/4 | +2/4.2/14/1 | +2/5.3/16/2 | +0/3.0/13/0 | +2/5.8/13/2 | +3/6.8/15/2 |
| 10 | +6/10.7/19/5 | +7/11.8/23/6 | +6/6.7/20/1 | +7/7.5/21/1 | +2/4.2/15/0 | +4/8.8/15/4 | +6/10.6/18/5 |
| 15 | +7/12.2/20/9 | +11/15.1/23/10 | +9/8.8/21/2 | +11/10.6/24/4 | +2/4.2/15/1 | +7/11.2/17/5 | +9/15.7/21/5 |
| 20 | +9/14.2/22/10 | +11/17.8/27/12 | +10/9.5/22/2 | +12/10.6/25/4 | +2/4.2/16/1 | +8/12.8/17/7 | +10/14.8/22/8 |
| 30 | +11/16.3/26/12 | +17/20.8/32/14 | +13/12.2/24/2 | +16/17.9/29/4 | +3/5.7/18/1 | +11/15.2/19/8 | +15/19.9/24/11 |
| 40 | +12/18.0/26/12 | +16/22.8/33/15 | +15/14.2/25/2 | +21/17.4/30/3 | +4/7.3/19/1 | +12/18.0/20/8 | +17/22.6/27/11 |
| 50 | +14/20.0/29/12 | +16/23.6/35/15 | +16/15.7/26/2 | +20/20.6/31/6 | +5/7.8/21/1 | +13/20.3/20/8 | +17/24.5/26/11 |
| 60 | +15/22.3/31/12 | +18/25.8/38/14 | +17/16.5/26/2 | +21/18.5/32/5 | +5/7.8/22/1 | +14/21.0/22/8 | +18/24.6/29/10 |

Stats, hit points, and mana (plain gear; trained → untrained). Scores St/Dx/To/In/Wi/Pr, the seeds' mean. Decipher script (learning): the trained Decipher script and the learning it gives; the auto-rolls spend every level-up roll on the first trained skill (Concentration for both), so it stays where it starts.

| L | Warrior | Thief | Wizard | Priest | HP W/T/Wz/P | MP Wz/P | Decipher script (learning) Wz/P | Blows a round W/T |
|---|---|---|---|---|---|---|---|---|
| 1 | 15/11/12/10/12/10 | 11/12/10/11/10/10 | 10/10/10/14/12/10 | 11/9/11/11/15/11 | 23→23 / 19→19 / 13→13 / 18→18 | 16/13 | 20 (3)/17 (1) | 0.99/0.99 |
| 5 | 15/11/12/10/12/10 | 11/13/10/11/10/10 | 10/10/10/14/13/10 | 11/9/11/11/15/11 | 37→37 / 26→26 / 17→17 / 26→26 | 49/39 | 20 (3)/17 (1) | 0.82/0.89 |
| 10 | 17/11/13/12/12/10 | 14/15/14/13/11/10 | 10/11/12/16/16/10 | 15/10/14/12/16/11 | 60→56 / 45→37 / 26→22 / 47→38 | 96/76 | 20 (3)/17 (1) | 0.83/1.04 |
| 15 | 18/14/16/13/12/10 | 18/18/17/14/11/10 | 10/12/14/18/18/10 | 18/11/17/13/18/11 | 88→73 / 71→47 / 36→28 / 74→48 | 148/116 | 20 (3)/17 (1) | 0.84/1.10 |
| 20 | 20/16/18/13/13/11 | 20/19/18/14/12/10 | 10/13/17/18/20/10 | 19/13/19/14/19/11 | 122→92 / 101→58 / 56→33 / 106→58 | 204/158 | 20 (3)/17 (1) | 0.85/1.10 |
| 30 | 23/18/20/15/14/12 | 22/20/20/16/14/11 | 11/15/20/20/22/11 | 22/15/21/15/20/12 | 195→128 / 170→79 / 99→43 / 177→79 | 324/246 | 20 (3)/17 (1) | 0.87/1.26 |
| 40 | 24/20/21/16/16/12 | 24/22/21/17/15/12 | 11/16/20/21/23/12 | 24/17/23/16/21/13 | 276→164 / 243→100 / 151→53 / 258→101 | 448/338 | 20 (3)/17 (1) | 0.88/1.36 |
| 50 | 26/20/22/17/17/13 | 25/22/22/18/16/13 | 12/18/21/22/24/13 | 25/18/24/17/22/14 | 363→200 / 320→120 / 204→63 / 346→122 | 577/434 | 20 (3)/17 (1) | 0.88/1.49 |
| 60 | 27/21/23/18/18/14 | 26/23/24/18/18/14 | 12/18/22/23/25/14 | 26/18/25/18/22/15 | 368→203 / 325→122 / 208→64 / 350→124 | 713/534 | 20 (3)/17 (1) | 0.88/1.49 |

The Defence of the warrior and the thief in plain gear by category (trained → untrained), the seeds' mean.

| Class | L | Dexterity | Armour | Shield | Parry | Natural | Dodge | Deflection | Wall | total |
|---|---|---|---|---|---|---|---|---|---|---|
| warrior | 10 | 0→0 | 2→2 | 4→4 | 2→2 | 0→0 | 1→1 | 0→0 | 0→0 | 19→19 |
| warrior | 30 | 2→0 | 1→1 | 8→8 | 4→4 | 0→0 | 1→1 | 0→0 | 0→0 | 26→24 |
| warrior | 60 | 2→0 | 1→1 | 10→10 | 6→6 | 0→0 | 2→1 | 0→0 | 0→0 | 31→28 |
| thief | 10 | 2→1 | 2→2 | 0→0 | 4→2 | 0→0 | 2→2 | 0→0 | 0→0 | 20→17 |
| thief | 30 | 5→1 | 2→2 | 0→0 | 6→4 | 0→0 | 2→2 | 0→0 | 0→0 | 24→18 |
| thief | 60 | 5→1 | 2→2 | 0→0 | 8→4 | 0→0 | 2→2 | 0→0 | 0→0 | 26→19 |

## Monsters

Within two levels: the median (range) bare → the mean with gear. Damage is a hit's mean before soak.

| L (±2) | n | HP | to-hit | dmg/blow | Dv | Pv | blows a round |
|---|---|---|---|---|---|---|---|
| 1 | 71 | 9 | 0 (-5–3) → -0.5 | 2 (1–8) → 2.3 | 11 (5–14) → 11.0 | 0 (0–3) → 0.1 | 1.00 |
| 5 | 65 | 18 | 1 (-3–5) → 1.2 | 4 (1–11) → 4.4 | 12 (7–21) → 12.0 | 0 (0–2) → 0.4 | 1.00 |
| 10 | 59 | 37 | 3 (-1–10) → 4.0 | 8 (1–18) → 8.5 | 11 (6–18) → 12.5 | 0 (0–5) → 0.8 | 1.00 |
| 15 | 47 | 56 | 6 (0–10) → 6.2 | 13 (2–20) → 12.6 | 12 (6–21) → 13.3 | 0 (0–6) → 1.1 | 1.00 |
| 20 | 51 | 72 | 7 (1–12) → 7.5 | 15 (2–21) → 14.9 | 13 (8–23) → 14.4 | 1 (0–6) → 1.6 | 1.00 |
| 30 | 39 | 112 | 11 (0–19) → 10.8 | 22 (2–27) → 20.3 | 15 (6–21) → 15.0 | 2 (0–8) → 3.1 | 1.00 |
| 40 | 29 | 150 | 16 (0–21) → 13.9 | 26 (2–32) → 23.9 | 17 (6–23) → 17.7 | 3 (0–11) → 3.7 | 1.05 |
| 50 | 26 | 237 | 18 (0–31) → 16.5 | 32 (2–38) → 29.3 | 19 (6–25) → 18.1 | 5 (0–13) → 4.9 | 1.08 |
| 60 | 18 | 222 | 20 (12–33) → 20.5 | 33 (18–43) → 31.4 | 22 (17–27) → 21.4 | 6 (0–16) → 5.9 | 1.17 |
| 70 | 8 | 200 | 24 (15–35) → 25.6 | 35 (26–48) → 34.7 | 24 (12–29) → 23.5 | 5 (0–11) → 4.9 | 1.26 |

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
| 1 | 86/43 · 7.7:0.1 · 1.2 / 296 | 86/43 · 7.7:0.1 · 1.2 / 340 | 86/43 · 7.6:0.1 · 1.2 / 353 | 72/55 · 3.0:0.5 · 3.3 / 38 | 72/55 · 3.2:0.5 · 3.3 / 37 | 72/61 · 2.3:1.1 · 4.1 / 14 | 77/66 · 4.7:1.2 · 2.1 / 18 | 77/66 · 4.7:1.2 · 2.0 / 18 |
| 5 | 86/51 · 7.5:1.2 · 2.7 / 37 | 86/44 · 6.1:0.9 · 3.3 / 51 | 88/33 · 7.2:0.7 · 2.8 / 86 | 77/52 · 3.7:1.9 · 7.1 / 16 | 77/41 · 4.6:1.2 · 4.7 / 30 | 65/61 · 1.8:2.9 · 13 / 6.5 | 77/60 · 3.6:1.9 · 5.9 / 16 | 81/49 · 4.5:1.4 · 4.7 / 27 |
| 10 | 88/44 · 10.4:2.9 · 4.3 / 24 | 88/36 · 8.2:1.8 · 5.3 / 41 | 91/20 · 9.3:1.0 · 4.7 / 149 | 89/34 · 9.3:2.6 · 5.3 / 23 | 92/28 · 10.7:2.2 · 4.6 / 46 | 68/68 · 2.4:5.5 · 22 / 5.0 | 83/66 · 5.9:4.0 · 7.5 / 13 | 91/44 · 8.3:2.4 · 5.3 / 28 |
| 15 | 94/52 · 11.6:5.2 · 5.4 / 18 | 94/49 · 9.4:3.5 · 6.5 / 28 | 98/32 · 13.6:2.2 · 4.9 / 120 | 97/39 · 13.9:4.5 · 5.9 / 21 | 98/19 · 16.9:1.9 · 5.0 / 51 | 67/79 · 2.0:9.7 · 34 / 3.9 | 92/70 · 8.0:6.6 · 7.7 / 11 | 97/43 · 13.3:4.1 · 5.0 / 51 |
| 20 | 96/38 · 14.6:4.8 · 6.2 / 29 | 96/41 · 11.7:3.6 · 7.6 / 42 | 98/15 · 16.6:1.1 · 5.9 / 200 | 97/44 · 14.4:5.9 · 7.7 / 19 | 99/25 · 16.6:3.2 · 6.1 / 62 | 61/85 · 1.6:12.9 · 52 / 4.6 | 95/78 · 8.7:8.5 · 9.2 / 13 | 97/47 · 11.5:4.3 · 7.8 / 34 |
| 30 | 98/40 · 16.9:8.3 · 8.9 / 24 | 98/46 · 14.0:6.9 · 11 / 30 | 99/14 · 20.5:1.8 · 8.0 / 234 | 99/56 · 21.6:11.4 · 8.1 / 16 | 99/27 · 36.5:5.3 · 4.6 / 57 | 63/92 · 2.0:20.6 · 66 / 4.8 | 97/89 · 10.0:15.4 · 13 / 12 | 99/62 · 17.5:9.0 · 8.4 / 23 |
| 40 | 93/55 · 18.2:12.6 · 10 / 25 | 93/64 · 14.9:10.6 · 13 / 30 | 98/21 · 25.1:3.3 · 8.4 / 251 | 98/68 · 25.9:16.9 · 8.3 / 17 | 99/39 · 41.0:8.7 · 5.5 / 69 | 46/96 · 1.8:24.2 · 113 / 6.3 | 91/95 · 12.4:19.5 · 16 / 14 | 99/62 · 22.5:10.7 · 7.8 / 29 |
| 50 | 95/47 · 21.8:15.1 · 12 / 25 | 95/63 · 17.7:16.0 · 14 / 23 | 98/21 · 25.9:4.8 · 10 / 90 | 98/74 · 30.2:23.1 · 8.7 / 14 | 99/49 · 40.8:14.7 · 6.5 / 46 | 48/97 · 2.0:28.7 · 149 / 7.1 | 93/98 · 15.7:28.2 · 16 / 12 | 98/80 · 22.2:20.9 · 12 / 19 |
| 60 | 92/44 · 22.7:14.1 · 10 / 28 | 92/60 · 18.8:14.8 · 13 / 26 | 96/12 · 26.6:2.6 · 9.6 / 191 | 94/82 · 29.0:29.3 · 9.3 / 12 | 99/51 · 35.5:17.6 · 7.2 / 45 | 29/98 · 1.0:34.2 · 230 / 6.1 | 85/98 · 14.1:30.1 · 17 / 12 | 95/73 · 21.1:19.9 · 11 / 21 |

Out of depth (+3: the monsters one to five levels above), and untrained stats, for the warrior in plain gear.

| L | Warrior plain +3 | Warrior magic +3 | Warrior plain, untrained | Warrior magic, untrained |
|---|---|---|---|---|
| 1 | 81/51 · 7.2:1.0 · 2.1 / 32 | 81/51 · 7.2:1.0 · 2.1 / 34 | 86/43 · 7.6:0.1 · 1.2 / 315 | 86/43 · 7.7:0.1 · 1.2 / 331 |
| 5 | 86/55 · 5.9:2.5 · 5.3 / 16 | 88/43 · 7.0:2.0 · 4.6 / 28 | 86/44 · 6.1:0.9 · 3.3 / 49 | 88/33 · 7.2:0.6 · 2.7 / 87 |
| 10 | 88/51 · 7.7:3.9 · 7.9 / 16 | 91/30 · 8.9:2.3 · 6.9 / 69 | 83/40 · 6.9:2.1 · 6.4 / 33 | 86/22 · 7.8:1.1 · 5.9 / 134 |
| 15 | 92/55 · 9.0:4.5 · 7.6 / 21 | 98/36 · 13.3:2.9 · 5.4 / 92 | 87/56 · 6.7:3.7 · 9.5 / 21 | 97/35 · 11.1:2.3 · 5.9 / 86 |
| 20 | 95/54 · 11.2:5.8 · 10 / 26 | 98/24 · 15.6:2.3 · 7.3 / 86 | 87/47 · 7.6:4.0 · 13 / 26 | 94/16 · 13.1:1.3 · 7.5 / 127 |
| 30 | 96/50 · 13.0:9.1 · 12 / 22 | 98/15 · 19.6:2.4 · 8.4 / 139 | 86/60 · 7.4:8.6 · 19 / 15 | 95/21 · 15.0:2.7 · 11 / 111 |
| 40 | 93/63 · 14.9:11.7 · 14 / 25 | 98/21 · 25.8:3.2 · 8.8 / 228 | 72/72 · 7.6:12.1 · 26 / 14 | 88/26 · 14.8:3.8 · 14 / 132 |
| 50 | 92/57 · 15.9:11.9 · 15 / 31 | 97/16 · 24.8:3.3 · 10 / 199 | 71/75 · 7.9:19.2 · 37 / 11 | 82/29 · 13.8:7.0 · 19 / 38 |
| 60 | 91/72 · 15.6:18.3 · 15 / 21 | 95/21 · 21.0:6.1 · 12 / 83 | 62/79 · 7.0:20.9 · 33 / 9.9 | 75/27 · 12.3:5.8 · 23 / 55 |

## Spread

Rounds to kill and to die over the monsters within two levels, trained stats in plain gear: the median (fastest tenth–slowest tenth) ×the tenths over the median. The target: ×0.5 and ×2.

| L | n | Warrior kills | Warrior dies | Thief dies | Wizard dies | Priest dies |
|---|---|---|---|---|---|---|
| 1 | 71 | 1.2 (0.6–1.9) ×0.46/×1.5 | 332 (25–>999) ×0.08/×7.0 | 40 (8.9–>999) ×0.22/×46.7 | 14 (4.6–486) ×0.33/×34.7 | 18 (6.2–655) ×0.35/×37.2 |
| 5 | 65 | 3.2 (1.7–6.4) ×0.54/×2.0 | 50 (14–>999) ×0.27/×49.2 | 16 (5.7–135) ×0.37/×8.7 | 6.7 (2.8–36) ×0.42/×5.4 | 16 (5.7–506) ×0.35/×31.4 |
| 10 | 59 | 5.1 (2.9–11) ×0.56/×2.2 | 39 (10–508) ×0.27/×13.1 | 24 (6.5–121) ×0.27/×5.1 | 4.9 (2.3–17) ×0.48/×3.5 | 13 (5.2–89) ×0.40/×6.8 |
| 15 | 47 | 6.6 (3.6–12) ×0.55/×1.7 | 27 (14–67) ×0.50/×2.5 | 21 (10–54) ×0.48/×2.6 | 3.9 (2.6–6.1) ×0.66/×1.6 | 12 (6.4–21) ×0.56/×1.8 |
| 20 | 51 | 7.6 (3.4–11) ×0.45/×1.5 | 39 (11–217) ×0.29/×5.5 | 20 (6.9–68) ×0.35/×3.5 | 4.6 (2.4–8.6) ×0.53/×1.9 | 14 (6.1–42) ×0.45/×3.1 |
| 30 | 39 | 11 (3.9–14) ×0.36/×1.3 | 30 (11–150) ×0.36/×5.0 | 16 (6.0–100) ×0.37/×6.2 | 4.8 (2.7–9.9) ×0.56/×2.0 | 11 (6.3–23) ×0.55/×2.0 |
| 40 | 29 | 13 (5.8–17) ×0.45/×1.3 | 30 (13–128) ×0.43/×4.3 | 17 (9.1–80) ×0.54/×4.7 | 6.3 (4.6–10) ×0.72/×1.6 | 14 (9.4–23) ×0.68/×1.7 |
| 50 | 26 | 14 (9.4–20) ×0.66/×1.4 | 22 (12–127) ×0.53/×5.7 | 15 (8.6–68) ×0.59/×4.7 | 7.1 (4.5–12) ×0.64/×1.7 | 13 (9.1–22) ×0.73/×1.8 |
| 60 | 18 | 13 (6.0–30) ×0.48/×2.4 | 25 (8.3–101) ×0.34/×4.1 | 12 (5.5–37) ×0.45/×3.0 | 6.1 (3.4–8.8) ×0.57/×1.4 | 12 (6.7–17) ×0.57/×1.4 |

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
| 1 | 1 | 43 | 1.2 / 850 · 0 % / 0 % | 3.2 / 133 · 0 % / 7 % | 4.0 / 15 · 5 % / 30 % | 2.0 / 18 · 0 % / 5 % |
| 1 | 2 | 16 | 1.0 / 53 · 0 % / 0 % | 3.2 / 24 · 0 % / 25 % | 4.0 / 11 · 12 % / 56 % | 1.7 / 13 · 0 % / 19 % |
| 1 | 3 | 12 | 1.5 / 118 · 0 % / 0 % | 4.4 / 25 · 8 % / 33 % | 4.6 / 12 · 25 % / 58 % | 2.3 / 16 · 8 % / 25 % |
| 1 | 4 | 13 | 2.0 / 38 · 0 % / 0 % | 6.0 / 19 · 23 % / 54 % | 6.2 / 7.4 · 46 % / 77 % | 3.1 / 9.0 · 8 % / 46 % |
| 1 | 5 | 13 | 2.8 / 14 · 0 % / 31 % | 12 / 6.6 · 62 % / 77 % | 12 / 3.6 · 62 % / 100 % | 4.9 / 4.5 · 54 % / 62 % |
| 5 | 1 | 43 | 1.5 / >999 · 0 % / 0 % | 2.4 / 249 · 0 % / 0 % | 4.9 / 22 · 5 % / 26 % | 2.4 / 943 · 0 % / 0 % |
| 5 | 2 | 16 | 1.1 / 203 · 0 % / 0 % | 2.2 / 43 · 0 % / 0 % | 4.5 / 17 · 6 % / 44 % | 2.0 / 46 · 0 % / 6 % |
| 5 | 3 | 12 | 1.7 / 725 · 0 % / 0 % | 3.1 / 50 · 0 % / 17 % | 5.6 / 19 · 25 % / 50 % | 2.6 / 58 · 0 % / 17 % |
| 5 | 4 | 13 | 2.2 / 84 · 0 % / 0 % | 4.5 / 36 · 0 % / 23 % | 7.3 / 11 · 38 % / 69 % | 3.4 / 37 · 0 % / 15 % |
| 5 | 5 | 13 | 3.2 / 31 · 0 % / 0 % | 7.4 / 11 · 8 % / 62 % | 15 / 4.8 · 62 % / 92 % | 5.8 / 11 · 8 % / 54 % |

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
| gorgon | 20 | warrior | 122→92 | 14 (11%→15%) | 12.6→12.9 | 12.7→13.0 | 9.6→7.0 | 9.4→14 |
| gorgon | 20 | thief | 101→58 | 14 (14%→24%) | 16.9→18.4 | 16.6→18.0 | 6.2→3.4 | 9.0→34 |
| gorgon | 20 | wizard | 56→33 | 14 (25%→43%) | 19.4→19.3 | 18.9→18.8 | 3.1→2.0 | 104→130 |
| gorgon | 20 | priest | 106→58 | 14 (13%→24%) | 15.9→15.5 | 15.8→15.3 | 6.8→3.9 | 13→28 |
| gorgon | 30 | warrior | 195→128 | 14 (7%→11%) | 9.8→10.4 | 10.2→10.8 | 19→12 | 7.2→13 |
| gorgon | 30 | thief | 170→79 | 14 (8%→18%) | 14.9→18.1 | 14.9→17.8 | 12→4.6 | 5.4→13 |
| gorgon | 30 | wizard | 99→43 | 14 (14%→33%) | 19.1→19.0 | 18.6→18.6 | 5.4→2.5 | 51→83 |
| gorgon | 30 | priest | 177→79 | 14 (8%→18%) | 14.9→14.9 | 14.8→14.9 | 12→5.4 | 9.8→21 |
| fire drake | 15 | warrior | 88→73 | 43 (49%→59%) | 25.8→27.4 | 28.3→29.7 | 2.8→2.1 | 14→21 |
| fire drake | 15 | thief | 71→47 | 43 (61%→92%) | 31.8→44.3 | 33.4→44.1 | 1.9→1.1 | 11→36 |
| fire drake | 15 | wizard | 36→28 | 43 (118%→157%) | 49.8→52.0 | 48.8→50.7 | 0.9→0.6 | 116→155 |
| fire drake | 15 | priest | 74→48 | 43 (58%→91%) | 38.2→40.5 | 38.9→40.8 | 1.8→1.1 | 17→43 |
| fire drake | 20 | warrior | 122→92 | 43 (35%→47%) | 19.3→20.4 | 22.7→23.6 | 5.2→3.4 | 10→17 |
| fire drake | 20 | thief | 101→58 | 43 (43%→74%) | 28.6→44.3 | 30.7→44.1 | 3.1→1.3 | 9.4→36 |
| fire drake | 20 | wizard | 56→33 | 43 (78%→132%) | 49.2→51.4 | 48.3→50.3 | 1.2→0.8 | 105→149 |
| fire drake | 20 | priest | 106→58 | 43 (41%→74%) | 34.2→35.8 | 35.5→36.9 | 2.9→1.5 | 14→31 |
| Hrīmwyrm the White | 30 | warrior | 195→128 | 57 (29%→45%) | 26.3→31.5 | 30.7→35.1 | 6.3→3.2 | 14→27 |
| Hrīmwyrm the White | 30 | thief | 170→79 | 57 (34%→72%) | 42.2→63.9 | 44.3→62.9 | 3.9→1.3 | 11→31 |
| Hrīmwyrm the White | 30 | wizard | 99→43 | 50 (51%→117%) | 66.8→71.3 | 64.4→68.3 | 1.7→0.9 | 166→277 |
| Hrīmwyrm the White | 30 | priest | 177→79 | 57 (32%→72%) | 51.2→54.1 | 52.1→54.5 | 3.4→1.4 | 20→50 |
| Hrīmwyrm the White | 40 | warrior | 276→164 | 57 (21%→35%) | 23.8→27.3 | 28.5→31.5 | 9.1→5.0 | 11→23 |
| Hrīmwyrm the White | 40 | thief | 243→100 | 57 (23%→57%) | 37.7→62.7 | 40.4→61.9 | 6.2→1.7 | 8.0→22 |
| Hrīmwyrm the White | 40 | wizard | 151→53 | 50 (33%→94%) | 63.6→69.6 | 61.6→66.8 | 2.6→1.0 | 88→169 |
| Hrīmwyrm the White | 40 | priest | 258→101 | 57 (22%→57%) | 48.5→49.9 | 49.8→50.9 | 5.2→1.9 | 15→44 |
| Nīðdraca the Hate-Wyrm | 60 | warrior | 368→203 | 95 (26%→47%) | 141.2→141.4 | 133.5→133.7 | 2.9→1.8 | 33→99 |
| Nīðdraca the Hate-Wyrm | 60 | thief | 325→122 | 95 (29%→78%) | 164.9→166.1 | 153.2→154.2 | 2.4→1.2 | 38→397 |
| Nīðdraca the Hate-Wyrm | 60 | wizard | 208→64 | 95 (46%→149%) | 168.5→168.4 | 156.2→156.2 | 1.7→0.7 | >999→>999 |
| Nīðdraca the Hate-Wyrm | 60 | priest | 350→124 | 95 (27%→77%) | 150.9→150.5 | 141.6→141.2 | 2.7→1.2 | 72→679 |

## Casters

Each caster (#1102) against the trained players of the first probe level from its own, in plain gear (the seeds' mean). **Spell** its first damage spell and its roll at its stats; **casts** how many of it its full pool pays for; **a cast** its mean damage (an `attack` spell's times its hit chance); **a round** that times its casts a round, against **melee** its blows a round once the mana is spent; **die** the player's rounds until they fall, the spells first, then its melee.

| Caster | L | Spell (roll) | casts | Player | HP | a cast | a round | melee | die |
|---|---|---|---|---|---|---|---|---|---|
| arcanaloth | 46 | chain lightning (12d6+3) | 8 | warrior 50 | 363 | 45.0 | 49.5 | 7.8 | 8.5 |
| arcanaloth | 46 | chain lightning (12d6+3) | 8 | wizard 50 | 204 | 45.0 | 49.5 | 25.8 | 4.1 |
| archlich | 54 | sunfire (16d6+4) | 3 | warrior 60 | 368 | 60.0 | 42.0 | 9.1 | 25 |
| archlich | 54 | sunfire (16d6+4) | 3 | wizard 60 | 208 | 60.0 | 42.0 | 15.7 | 6.2 |
| ash wraith | 15 | scorching ray (6d6) | 5 | warrior 15 | 88 | 11.6 | 12.8 | 3.0 | 16 |
| ash wraith | 15 | scorching ray (6d6) | 5 | wizard 15 | 36 | 18.4 | 20.2 | 5.4 | 1.8 |
| barrow king | 42 | grave spear (10d6+2) | 5 | warrior 50 | 363 | 37.0 | 40.7 | 13.4 | 18 |
| barrow king | 42 | grave spear (10d6+2) | 5 | wizard 50 | 204 | 37.0 | 40.7 | 18.8 | 5.7 |
| bheur hag | 42 | rime (8d6+2) | 7 | warrior 50 | 363 | 30.0 | 33.0 | 6.4 | 31 |
| bheur hag | 42 | rime (8d6+2) | 7 | wizard 50 | 204 | 30.0 | 33.0 | 17.2 | 6.3 |
| dark wizard | 24 | fireball (8d6) | 4 | warrior 30 | 195 | 28.0 | 28.0 | 0.0 | >999 |
| dark wizard | 24 | fireball (8d6) | 4 | wizard 30 | 99 | 28.0 | 28.0 | 2.6 | 3.8 |
| drow archmage | 40 | chain lightning (12d6+3) | 8 | warrior 40 | 276 | 45.0 | 49.5 | 7.4 | 5.6 |
| drow archmage | 40 | chain lightning (12d6+3) | 8 | wizard 40 | 151 | 45.0 | 49.5 | 22.7 | 3.0 |
| drow high priestess | 44 | leech (8d6+2) | 13 | warrior 50 | 363 | 27.2 | 29.9 | 7.1 | 14 |
| drow high priestess | 44 | leech (8d6+2) | 13 | wizard 50 | 204 | 29.8 | 32.8 | 25.2 | 6.2 |
| drow mage | 22 | lightning bolt (10d6) | 4 | warrior 30 | 195 | 35.0 | 35.0 | 0.7 | 87 |
| drow mage | 22 | lightning bolt (10d6) | 4 | wizard 30 | 99 | 35.0 | 35.0 | 8.7 | 2.8 |
| drow matron | 50 | chain lightning (12d6+3) | 8 | warrior 50 | 363 | 45.0 | 49.5 | 10.1 | 8.2 |
| drow matron | 50 | chain lightning (12d6+3) | 8 | wizard 50 | 204 | 45.0 | 49.5 | 27.8 | 4.1 |
| drow priestess | 24 | leech (8d6) | 9 | warrior 30 | 195 | 17.9 | 17.9 | 1.5 | 40 |
| drow priestess | 24 | leech (8d6) | 9 | wizard 30 | 99 | 27.2 | 27.2 | 12.0 | 3.6 |
| dryad | 19 | acid arrow (6d6) | 10 | warrior 20 | 122 | 10.5 | 10.5 | 1.6 | 26 |
| dryad | 19 | acid arrow (6d6) | 10 | wizard 20 | 56 | 18.9 | 18.9 | 9.4 | 2.9 |
| duergar mindbreaker | 29 | flay (2d8) | 9 | warrior 30 | 195 | 9.0 | 9.0 | 5.2 | 32 |
| duergar mindbreaker | 29 | flay (2d8) | 9 | wizard 30 | 99 | 9.0 | 9.0 | 15.8 | 10 |
| duergar priest | 19 | flay (2d8) | 6 | warrior 20 | 122 | 9.0 | 9.0 | 0.0 | >999 |
| duergar priest | 19 | flay (2d8) | 6 | wizard 20 | 56 | 9.0 | 9.0 | 2.6 | 7.1 |
| ghost lord | 53 | grave spear (10d6+2) | 6 | warrior 60 | 368 | 37.0 | 51.8 | 13.9 | 15 |
| ghost lord | 53 | grave spear (10d6+2) | 6 | wizard 60 | 208 | 37.0 | 51.8 | 29.6 | 4.1 |
| giant shaman | 36 | rime (8d6) | 7 | warrior 40 | 276 | 28.0 | 28.0 | 7.2 | 19 |
| giant shaman | 36 | rime (8d6) | 7 | wizard 40 | 151 | 28.0 | 28.0 | 20.5 | 5.4 |
| goblin wizard | 8 | burning hands (2d6) | 5 | warrior 10 | 60 | 7.0 | 7.0 | 0.1 | 380 |
| goblin wizard | 8 | burning hands (2d6) | 5 | wizard 10 | 26 | 7.0 | 7.0 | 1.6 | 3.7 |
| green hag | 18 | blight (6d6+3) | 3 | warrior 20 | 122 | 24.0 | 24.0 | 1.2 | 50 |
| green hag | 18 | blight (6d6+3) | 3 | wizard 20 | 56 | 24.0 | 24.0 | 8.3 | 2.3 |
| hobgoblin warcaster | 14 | frost (6d6) | 3 | warrior 15 | 88 | 21.0 | 21.0 | 0.0 | >999 |
| hobgoblin warcaster | 14 | frost (6d6) | 3 | wizard 15 | 36 | 21.0 | 21.0 | 0.7 | 1.7 |
| kobold shaman | 4 | spark (2d4) | 1 | warrior 5 | 37 | 5.0 | 4.0 | 0.0 | >999 |
| kobold shaman | 4 | spark (2d4) | 1 | wizard 5 | 17 | 5.0 | 4.0 | 0.2 | 465 |
| lich | 30 | chain lightning (12d6+3) | 7 | warrior 30 | 195 | 45.0 | 31.5 | 3.3 | 6.2 |
| lich | 30 | chain lightning (12d6+3) | 7 | wizard 30 | 99 | 45.0 | 31.5 | 8.6 | 3.1 |
| mummy king | 56 | flame (10d6+2) | 6 | warrior 60 | 368 | 37.0 | 42.6 | 10.0 | 20 |
| mummy king | 56 | flame (10d6+2) | 6 | wizard 60 | 208 | 37.0 | 42.6 | 33.4 | 4.9 |
| mummy lord | 36 | blight (6d6+3) | 6 | warrior 40 | 276 | 24.0 | 25.2 | 3.4 | 49 |
| mummy lord | 36 | blight (6d6+3) | 6 | wizard 40 | 151 | 24.0 | 25.2 | 16.4 | 6.2 |
| mummy priest | 20 | cause wounds (4d8+2) | 27 | warrior 20 | 122 | 11.5 | 11.5 | 2.7 | 11 |
| mummy priest | 20 | cause wounds (4d8+2) | 27 | wizard 20 | 56 | 18.6 | 18.6 | 7.3 | 3.0 |
| necromancer | 18 | leech (8d6) | 7 | warrior 20 | 122 | 14.0 | 14.0 | 0.0 | >999 |
| necromancer | 18 | leech (8d6) | 7 | wizard 20 | 56 | 25.1 | 25.1 | 0.7 | 2.2 |
| night hag | 33 | leech (8d6+2) | 10 | warrior 40 | 276 | 22.1 | 23.2 | 2.6 | 35 |
| night hag | 33 | leech (8d6+2) | 10 | wizard 40 | 151 | 29.7 | 31.1 | 14.4 | 4.8 |
| nymph | 22 | magic missile (6d4+1) | 11 | warrior 30 | 195 | 16.0 | 22.4 | 0.5 | 55 |
| nymph | 22 | magic missile (6d4+1) | 11 | wizard 30 | 99 | 16.0 | 22.4 | 9.6 | 4.4 |
| ogre mage | 18 | rime (8d6) | 4 | warrior 20 | 122 | 28.0 | 28.0 | 2.5 | 8.3 |
| ogre mage | 18 | rime (8d6) | 4 | wizard 20 | 56 | 28.0 | 28.0 | 11.4 | 2.0 |
| orc high shaman | 24 | fireball (8d6) | 3 | warrior 30 | 195 | 28.0 | 28.0 | 0.1 | >999 |
| orc high shaman | 24 | fireball (8d6) | 3 | wizard 30 | 99 | 28.0 | 28.0 | 3.8 | 7.4 |
| orc shaman | 10 | burning hands (2d6) | 2 | warrior 10 | 60 | 7.0 | 7.0 | 0.0 | >999 |
| orc shaman | 10 | burning hands (2d6) | 2 | wizard 10 | 26 | 7.0 | 7.0 | 0.4 | 39 |
| pixie | 4 | spark (2d4) | 1 | warrior 5 | 37 | 5.0 | 7.0 | 0.4 | 89 |
| pixie | 4 | spark (2d4) | 1 | wizard 5 | 17 | 5.0 | 7.0 | 1.8 | 7.9 |
| rakshasa | 52 | flame (10d6+2) | 6 | warrior 60 | 368 | 37.0 | 51.8 | 13.6 | 15 |
| rakshasa | 52 | flame (10d6+2) | 6 | wizard 60 | 208 | 37.0 | 51.8 | 37.7 | 4.0 |
| troll shaman | 21 | rime (8d6) | 5 | warrior 30 | 195 | 28.0 | 28.0 | 1.6 | 43 |
| troll shaman | 21 | rime (8d6) | 5 | wizard 30 | 99 | 28.0 | 28.0 | 11.6 | 3.5 |
| will-o'-wisp | 7 | spark (2d4) | 2 | warrior 10 | 60 | 5.0 | 7.0 | 0.1 | 671 |
| will-o'-wisp | 7 | spark (2d4) | 2 | wizard 10 | 26 | 5.0 | 7.0 | 0.3 | 63 |
| wormlord | 47 | leech (8d6+2) | 13 | warrior 50 | 363 | 27.2 | 19.0 | 4.9 | 22 |
| wormlord | 47 | leech (8d6+2) | 13 | wizard 50 | 204 | 29.8 | 20.9 | 16.2 | 9.8 |
| barrow wight | 6 | grave breath (4d4) | 2 | warrior 10 | 60 | 10.0 | 10.0 | 1.6 | 28 |
| barrow wight | 6 | grave breath (4d4) | 2 | wizard 10 | 26 | 10.0 | 10.0 | 3.0 | 3.9 |
| bog hag | 11 | acid arrow (6d6) | 6 | warrior 15 | 88 | 8.3 | 8.3 | 0.3 | 158 |
| bog hag | 11 | acid arrow (6d6) | 6 | wizard 15 | 36 | 16.4 | 16.4 | 4.2 | 2.3 |
| cult priest | 9 | grave breath (4d4) | 3 | warrior 10 | 60 | 10.0 | 10.0 | 0.0 | >999 |
| cult priest | 9 | grave breath (4d4) | 3 | wizard 10 | 26 | 10.0 | 10.0 | 0.2 | 2.6 |

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

The wizard is measured on two careers (#1299, the miscasts design §1.2): Spellcraft first, every level's skill rolls on Spellcraft (#1288), and Concentration first, the same wizard with its rolls where `levelup:auto` puts them, so its Spellcraft stays at its level-1 value (the design's neglecter). The priest, the witch, and the paladin (a half-caster) spend theirs where `levelup:auto` puts them. Re-measured at #1299, after Spellcraft's `cast` curve was slowed past 100 (+22 at 200, +40 at 999, was +34 at 200 and flat after): the Spellcraft-first wizard reaches 139 at level 10 and 293 at 20, so on the old curve it cast every spell up to level 19 at the floor (only the 1 and 1 miscasts) from level 10 and every spell in the game from level 15, and every roll after 200 was wasted. Now it is at the floor for spells up to level 15 at level 10, 23 at level 20, and 28 at level 30; at level 10 a level-30 spell still miscasts four times in five at rank 1 (one miscast in twenty-five severe) and one in five at rank 10, those all protected;and every roll counts to the end.Spellcraft up to 100 is unchanged, so the half-casters, the neglecter, and every creature caster below 100 are too; the creature casters above 100 lose up to 12 points of bonus and stay at the floor but for the lich's Chain lightning (6.9 %) and the archlich's Sunfire (4.2 %). The other sections are as their last PRs left them.

Each class's Spellcraft, the bonus of its check on a cast and its ward (trained, plain gear, the seeds' mean, the bonus rounded), and the chance of a miscast for a spell of level 1, of level 11, and of the highest level it knows (that level in brackets), at rank 1. The wizard twice: every skill roll on Spellcraft, and the same wizard with its rolls on Concentration (its Spellcraft stays where it started).

| Class | L | Spellcraft | Bonus | Ward | Level 1 | Level 11 | Highest known |
|---|---:|---:|---:|---:|---:|---:|---:|
| wizard | 1 | 20 | +3.7 | 2 | 4.2 % | 61.8 % | 4.2 % (1) |
| wizard | 5 | 55 | +8.8 | 5 | 0.7 % | 25.0 % | 0.7 % (3) |
| wizard | 10 | 126 | +18.5 | 12 | 0.7 % | 0.7 % | 0.7 % (8) |
| wizard | 15 | 189 | +24.3 | 19 | 0.7 % | 0.7 % | 0.7 % (8) |
| wizard | 20 | 256 | +26.7 | 20 | 0.7 % | 0.7 % | 0.7 % (8) |
| wizard | 30 | 442 | +32.0 | 20 | 0.7 % | 0.7 % | 0.7 % (20) |
| wizard | 40 | 664 | +37.0 | 20 | 0.7 % | 0.7 % | 0.7 % (30) |
| wizard | 50 | 924 | +43.7 | 20 | 0.7 % | 0.7 % | 0.7 % (30) |
| wizard | 60 | 999 | +46.2 | 20 | 0.7 % | 0.7 % | 0.7 % (30) |
| wizard (Concentration first) | 1 | 20 | +3.7 | 2 | 4.2 % | 61.8 % | 4.2 % (1) |
| wizard (Concentration first) | 5 | 20 | +3.7 | 2 | 4.2 % | 61.8 % | 10.4 % (3) |
| wizard (Concentration first) | 10 | 20 | +4.3 | 2 | 4.2 % | 61.8 % | 38.2 % (8) |
| wizard (Concentration first) | 15 | 20 | +5.2 | 2 | 2.1 % | 54.2 % | 31.2 % (8) |
| wizard (Concentration first) | 20 | 20 | +5.5 | 2 | 0.7 % | 45.8 % | 25.0 % (8) |
| wizard (Concentration first) | 30 | 20 | +6.5 | 2 | 0.7 % | 38.2 % | 93.1 % (20) |
| wizard (Concentration first) | 40 | 20 | +6.7 | 2 | 0.7 % | 38.2 % | 99.3 % (30) |
| wizard (Concentration first) | 50 | 20 | +7.3 | 2 | 0.7 % | 38.2 % | 99.3 % (30) |
| wizard (Concentration first) | 60 | 20 | +7.7 | 2 | 0.7 % | 31.2 % | 99.3 % (30) |
| priest | 1 | 22 | +2.2 | 2 | 10.4 % | 75.0 % | 10.4 % (1) |
| priest | 5 | 22 | +2.2 | 2 | 10.4 % | 75.0 % | 19.4 % (3) |
| priest | 10 | 28 | +3.7 | 2 | 4.2 % | 61.8 % | 54.2 % (10) |
| priest | 15 | 59 | +8.8 | 6 | 0.7 % | 25.0 % | 25.0 % (11) |
| priest | 20 | 85 | +13.0 | 8 | 0.7 % | 6.9 % | 19.4 % (14) |
| priest | 30 | 132 | +18.5 | 13 | 0.7 % | 0.7 % | 0.7 % (14) |
| priest | 40 | 171 | +22.2 | 17 | 0.7 % | 0.7 % | 0.7 % (14) |
| priest | 50 | 220 | +25.0 | 20 | 0.7 % | 0.7 % | 0.7 % (14) |
| priest | 60 | 270 | +26.5 | 20 | 0.7 % | 0.7 % | 0.7 % (14) |
| witch | 1 | 21 | +2.7 | 2 | 6.9 % | 68.8 % | 6.9 % (1) |
| witch | 5 | 21 | +2.7 | 2 | 6.9 % | 68.8 % | 14.6 % (3) |
| witch | 10 | 58 | +8.8 | 6 | 0.7 % | 25.0 % | 10.4 % (8) |
| witch | 15 | 126 | +17.3 | 12 | 0.7 % | 0.7 % | 0.7 % (8) |
| witch | 20 | 167 | +21.3 | 16 | 0.7 % | 0.7 % | 0.7 % (11) |
| witch | 30 | 302 | +26.7 | 20 | 0.7 % | 0.7 % | 4.2 % (24) |
| witch | 40 | 503 | +31.5 | 20 | 0.7 % | 0.7 % | 0.7 % (24) |
| witch | 50 | 800 | +38.7 | 20 | 0.7 % | 0.7 % | 0.7 % (24) |
| witch | 60 | 999 | +43.8 | 20 | 0.7 % | 0.7 % | 0.7 % (24) |
| paladin | 1 | 10 | -0.3 | 0 | 19.4 % | 85.4 % | — |
| paladin | 5 | 10 | -0.3 | 0 | 19.4 % | 85.4 % | — |
| paladin | 10 | 10 | +0.3 | 0 | 19.4 % | 85.4 % | 31.2 % (3) |
| paladin | 15 | 10 | +1.0 | 0 | 14.6 % | 80.6 % | 54.2 % (7) |
| paladin | 20 | 10 | +1.5 | 0 | 10.4 % | 75.0 % | 54.2 % (8) |
| paladin | 30 | 10 | +2.3 | 0 | 10.4 % | 75.0 % | 54.2 % (8) |
| paladin | 40 | 10 | +3.0 | 0 | 6.9 % | 68.8 % | 45.8 % (8) |
| paladin | 50 | 10 | +3.7 | 0 | 4.2 % | 61.8 % | 38.2 % (8) |
| paladin | 60 | 10 | +4.0 | 0 | 4.2 % | 61.8 % | 38.2 % (8) |

Where a wizard's miscast lands (the miscasts design's bands: harmless 1–16, mild 17–40, annoying 41–70, severe 71–90, deadly 91–110, catastrophe 111+; below 71 is the protected half), out of armour, in the neutral stance: the chance of a miscast, each band's share of the miscasts in percent, and the worst total.

| Wizard | L | Bonus | Ward | Spell (level, rank) | Miscast | Harmless | Mild | Annoying | Severe | Deadly | Catastrophe | Worst |
|---|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Spellcraft first | 1 | +4 | 2 | 11, r1 | 61.8 % | 9.6 | 53.1 | 34.4 | 2.9 | 0.0 | 0.0 | 87 |
| Spellcraft first | 1 | +4 | 2 | 20, r1 | 99.3 % | 1.2 | 15.0 | 40.4 | 25.4 | 14.3 | 3.6 | 132 |
| Spellcraft first | 1 | +4 | 2 | 20, r5 | 89.6 % | 3.7 | 31.0 | 45.5 | 15.8 | 4.0 | 0.0 | 112 |
| Spellcraft first | 1 | +4 | 2 | 30, r1 | 99.3 % | 0.0 | 0.0 | 2.5 | 13.7 | 24.9 | 58.9 | 182 |
| Spellcraft first | 1 | +4 | 2 | 30, r10 | 99.3 % | 0.3 | 11.4 | 37.1 | 27.9 | 17.1 | 6.2 | 137 |
| Spellcraft first | 10 | +19 | 12 | 11, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 12 |
| Spellcraft first | 10 | +19 | 12 | 20, r1 | 19.4 % | 55.8 | 43.4 | 0.8 | 0.0 | 0.0 | 0.0 | 47 |
| Spellcraft first | 10 | +19 | 12 | 20, r5 | 4.2 % | 87.2 | 12.8 | 0.0 | 0.0 | 0.0 | 0.0 | 27 |
| Spellcraft first | 10 | +19 | 12 | 30, r1 | 85.4 % | 16.9 | 41.2 | 34.7 | 7.0 | 0.2 | 0.0 | 97 |
| Spellcraft first | 10 | +19 | 12 | 30, r10 | 25.0 % | 50.6 | 46.6 | 2.8 | 0.0 | 0.0 | 0.0 | 52 |
| Spellcraft first | 20 | +27 | 20 | 11, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 20 | +27 | 20 | 20, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 20 | +27 | 20 | 20, r5 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 20 | +27 | 20 | 30, r1 | 31.2 % | 67.9 | 31.1 | 1.0 | 0.0 | 0.0 | 0.0 | 49 |
| Spellcraft first | 20 | +27 | 20 | 30, r10 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 30 | +32 | 20 | 11, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 30 | +32 | 20 | 20, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 30 | +32 | 20 | 20, r5 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 30 | +32 | 20 | 30, r1 | 6.9 % | 96.7 | 3.3 | 0.0 | 0.0 | 0.0 | 0.0 | 24 |
| Spellcraft first | 30 | +32 | 20 | 30, r10 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 60 | +46 | 20 | 11, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 60 | +46 | 20 | 20, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 60 | +46 | 20 | 20, r5 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 60 | +46 | 20 | 30, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 60 | +46 | 20 | 30, r10 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Concentration first | 1 | +4 | 2 | 11, r1 | 61.8 % | 9.6 | 53.1 | 34.4 | 2.9 | 0.0 | 0.0 | 87 |
| Concentration first | 1 | +4 | 2 | 20, r1 | 99.3 % | 1.2 | 15.0 | 40.4 | 25.4 | 14.3 | 3.6 | 132 |
| Concentration first | 1 | +4 | 2 | 20, r5 | 89.6 % | 3.7 | 31.0 | 45.5 | 15.8 | 4.0 | 0.0 | 112 |
| Concentration first | 1 | +4 | 2 | 30, r1 | 99.3 % | 0.0 | 0.0 | 2.5 | 13.7 | 24.9 | 58.9 | 182 |
| Concentration first | 1 | +4 | 2 | 30, r10 | 99.3 % | 0.3 | 11.4 | 37.1 | 27.9 | 17.1 | 6.2 | 137 |
| Concentration first | 10 | +4 | 2 | 11, r1 | 61.8 % | 9.6 | 53.1 | 34.4 | 2.9 | 0.0 | 0.0 | 87 |
| Concentration first | 10 | +4 | 2 | 20, r1 | 99.3 % | 1.2 | 15.0 | 40.4 | 25.4 | 14.3 | 3.6 | 132 |
| Concentration first | 10 | +4 | 2 | 20, r5 | 89.6 % | 3.7 | 31.0 | 45.5 | 15.8 | 4.0 | 0.0 | 112 |
| Concentration first | 10 | +4 | 2 | 30, r1 | 99.3 % | 0.0 | 0.0 | 2.5 | 13.7 | 24.9 | 58.9 | 182 |
| Concentration first | 10 | +4 | 2 | 30, r10 | 99.3 % | 0.3 | 11.4 | 37.1 | 27.9 | 17.1 | 6.2 | 137 |
| Concentration first | 20 | +6 | 2 | 11, r1 | 45.8 % | 12.3 | 58.9 | 28.5 | 0.4 | 0.0 | 0.0 | 77 |
| Concentration first | 20 | +6 | 2 | 20, r1 | 95.8 % | 2.4 | 22.2 | 45.1 | 20.6 | 9.0 | 0.7 | 122 |
| Concentration first | 20 | +6 | 2 | 20, r5 | 80.6 % | 5.4 | 41.1 | 41.9 | 10.7 | 0.9 | 0.0 | 102 |
| Concentration first | 20 | +6 | 2 | 30, r1 | 99.3 % | 0.0 | 0.0 | 8.0 | 19.3 | 29.4 | 43.4 | 172 |
| Concentration first | 20 | +6 | 2 | 30, r10 | 97.9 % | 1.8 | 18.5 | 43.3 | 23.0 | 11.6 | 1.8 | 127 |
| Concentration first | 30 | +7 | 2 | 11, r1 | 38.2 % | 13.3 | 62.3 | 24.4 | 0.0 | 0.0 | 0.0 | 72 |
| Concentration first | 30 | +7 | 2 | 20, r1 | 93.1 % | 3.0 | 26.4 | 45.8 | 18.2 | 6.4 | 0.2 | 117 |
| Concentration first | 30 | +7 | 2 | 20, r5 | 75.0 % | 6.5 | 45.8 | 39.5 | 7.9 | 0.2 | 0.0 | 97 |
| Concentration first | 30 | +7 | 2 | 30, r1 | 99.3 % | 0.0 | 0.0 | 11.7 | 22.1 | 30.2 | 36.0 | 167 |
| Concentration first | 30 | +7 | 2 | 30, r10 | 95.8 % | 2.4 | 22.2 | 45.1 | 20.6 | 9.0 | 0.7 | 122 |
| Concentration first | 60 | +8 | 2 | 11, r1 | 31.2 % | 14.6 | 65.8 | 19.6 | 0.0 | 0.0 | 0.0 | 67 |
| Concentration first | 60 | +8 | 2 | 20, r1 | 89.6 % | 3.7 | 31.0 | 45.5 | 15.8 | 4.0 | 0.0 | 112 |
| Concentration first | 60 | +8 | 2 | 20, r5 | 68.8 % | 7.9 | 49.9 | 37.0 | 5.2 | 0.0 | 0.0 | 92 |
| Concentration first | 60 | +8 | 2 | 30, r1 | 99.3 % | 0.0 | 0.2 | 16.0 | 24.9 | 29.7 | 29.3 | 162 |
| Concentration first | 60 | +8 | 2 | 30, r10 | 93.1 % | 3.0 | 26.4 | 45.8 | 18.2 | 6.4 | 0.2 | 117 |

Creature casters: the chance of a miscast for each spell of their lists (their Spellcraft and Intelligence as their type makes them).

| Creature | L | Spellcraft | Bonus | Spells |
|---|---:|---:|---:|---|
| arcanaloth | 46 | 153 | +29 | fire cloak 0.7 %, chain lightning 0.7 %, fireball 0.7 %, magic missile 0.7 % |
| archlich | 54 | 177 | +33 | haste 0.7 %, silence 0.7 %, sunfire 4.2 %, chain lightning 0.7 %, grave spear 0.7 % |
| ash wraith | 15 | 50 | +9 | scorching ray 6.9 %, grave breath 0.7 % |
| barrow king | 42 | 141 | +27 | fear 0.7 %, grave spear 0.7 %, rime 0.7 % |
| bheur hag | 42 | 141 | +27 | curse of sloth 0.7 %, rime 0.7 %, frost 0.7 % |
| dark wizard | 24 | 98 | +17 | haste 0.7 %, fireball 0.7 %, lightning bolt 0.7 %, magic missile 0.7 % |
| deathless king | 66 | 213 | +37 | haste 0.7 %, fear 0.7 %, sunfire 0.7 %, grave spear 0.7 %, chain lightning 0.7 % |
| demilich | 70 | 225 | +37 | silence 0.7 %, sunfire 0.7 %, chain lightning 0.7 %, leech 0.7 %, grave spear 0.7 % |
| drow archmage | 40 | 115 | +25 | haste 0.7 %, web 0.7 %, chain lightning 0.7 %, fireball 0.7 % |
| drow high priestess | 44 | 123 | +26 | cure wounds 0.7 %, silence 0.7 %, web 0.7 %, leech 0.7 % |
| drow mage | 22 | 79 | +16 | web 0.7 %, lightning bolt 0.7 %, magic missile 0.7 % |
| drow matron | 50 | 135 | +28 | cure wounds 0.7 %, silence 0.7 %, web 0.7 %, chain lightning 0.7 %, leech 0.7 % |
| drow priestess | 24 | 83 | +18 | cure wounds 0.7 %, silence 0.7 %, leech 0.7 % |
| dryad | 19 | 72 | +15 | barkskin 0.7 %, bind 0.7 %, acid arrow 0.7 % |
| duergar mindbreaker | 29 | 102 | +18 | fear 0.7 %, bind 0.7 %, flay 0.7 % |
| duergar priest | 19 | 72 | +12 | cure wounds 0.7 %, blessing 0.7 %, flay 2.1 %, cause wounds 0.7 % |
| ghost lord | 53 | 174 | +31 | fear 0.7 %, slumber 0.7 %, grave spear 0.7 % |
| giant shaman | 36 | 123 | +24 | cure wounds 0.7 %, rime 0.7 %, lightning bolt 0.7 % |
| goblin wizard | 8 | 34 | +6 | burning hands 0.7 %, spark 0.7 % |
| green hag | 18 | 69 | +14 | curse of sloth 0.7 %, mesmerise 0.7 %, blight 0.7 % |
| hobgoblin warcaster | 14 | 57 | +10 | frost 2.1 %, arcane bolt 2.1 %, magic missile 0.7 % |
| kobold shaman | 4 | 25 | +3 | mend 6.9 %, spark 6.9 % |
| lich | 30 | 105 | +22 | haste 0.7 %, chain lightning 6.9 %, fireball 0.7 %, grave spear 0.7 % |
| mummy king | 56 | 183 | +35 | fear 0.7 %, flame 0.7 %, grave spear 0.7 % |
| mummy lord | 36 | 123 | +24 | fear 0.7 %, blight 0.7 %, grave spear 0.7 % |
| mummy priest | 20 | 75 | +16 | fear 0.7 %, cause wounds 0.7 %, blight 0.7 % |
| necromancer | 18 | 73 | +12 | hex of weakness 0.7 %, leech 6.9 %, blight 0.7 % |
| night hag | 33 | 114 | +23 | slumber 0.7 %, leech 0.7 %, blight 0.7 % |
| nymph | 22 | 81 | +17 | mesmerise 0.7 %, magic missile 0.7 % |
| ogre mage | 18 | 69 | +12 | rime 4.2 %, frost 0.7 %, magic missile 0.7 % |
| orc high shaman | 24 | 87 | +15 | cure wounds 0.7 %, blessing 0.7 %, fireball 2.1 % |
| orc shaman | 10 | 45 | +7 | mend 0.7 %, burning hands 0.7 %, spark 0.7 % |
| pixie | 4 | 27 | +4 | evil eye 2.1 %, spark 4.2 % |
| rakshasa | 52 | 171 | +31 | glamour 0.7 %, mesmerise 0.7 %, flame 0.7 %, lightning bolt 0.7 % |
| troll shaman | 21 | 78 | +12 | cure wounds 0.7 %, rime 4.2 %, lightning bolt 10.4 % |
| will-o'-wisp | 7 | 30 | +4 | spark 4.2 % |
| wormlord | 47 | 156 | +29 | fear 0.7 %, leech 0.7 %, blight 0.7 % |
| barrow wight | 6 | 35 | +6 | hex of weakness 4.2 %, grave breath 0.7 % |
| bog hag | 11 | 60 | +11 | hex of weakness 0.7 %, acid arrow 0.7 %, grave breath 0.7 % |
| cult priest | 9 | 45 | +7 | mend 0.7 %, evil eye 0.7 %, grave breath 0.7 % |
| Līgetdrȳ the Storm-Wright | 61 | 195 | +36 | haste 0.7 %, silence 0.7 %, chain lightning 0.7 %, lightning bolt 0.7 % |

## Protection by body

Rounds for the plain warrior (trained) to kill the monsters of their level: as shipped, then with every monster's Protection set to §8.2's body line at its level.

| L | as shipped | none (0) | hide | bony | armoured |
|---|---|---|---|---|---|
| 10 | 5.3 | 4.4 | 4.5 | 4.9 | 5.6 |
| 15 | 6.6 | 5.8 | 5.6 | 6.3 | 7.6 |
| 20 | 7.6 | 5.9 | 6.4 | 7.0 | 7.7 |
| 30 | 11 | 7.3 | 7.9 | 8.9 | 11 |
| 40 | 13 | 8.8 | 9.9 | 12 | 14 |
| 50 | 14 | 11 | 12 | 14 | 17 |
| 60 | 13 | 7.4 | 8.2 | 9.2 | 11 |

## Spells

Each spell that rolls (harm or healing) of the schools the trained caster knows or has an affinity with, as `Actor::spell_roll` makes it in plain gear, per cast on one target: the rung of its ladder the stats reach (the seeds' range), the caster style it is cast in and the rank the pace below reached (`–` for a spell cast in no style), the roll (seed 1's dice, the seeds' mean) and the same roll at style rank 0, its mean on a target under a hex (Malison's bite; `–` without it), and its mana (the cost table's price with no talent in brackets) over the pool.

**Wizard**

| Spell (level, res) | L | rung | style rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Burning hands (1, area) | 10 | 1–2 | – | 4d4, 9.5 (rank 0: 4d4, 9.5) | – | 6 (6) / 96 |
| Burning hands (1, area) | 20 | 2 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 6 (6) / 204 |
| Burning hands (1, area) | 30 | 2 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 6 (6) / 324 |
| Burning hands (1, area) | 60 | 3 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 2 (6) / 713 |
| Grave breath (1, area) | 10 | 1–2 | – | 4d4, 8.5 (rank 0: 4d4, 8.5) | – | 6 (6) / 96 |
| Grave breath (1, area) | 20 | 2 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 6 (6) / 204 |
| Grave breath (1, area) | 30 | 2–3 | – | 4d4+2, 11.3 (rank 0: 4d4+2, 11.3) | – | 6 (6) / 324 |
| Grave breath (1, area) | 60 | 3 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 2 (6) / 713 |
| Ray of frost (1, attack) | 10 | 0–1 | Sniper 2–3 | 2d8, 8.7 (rank 0: 2d8, 8.7) | – | 4 (4) / 96 |
| Ray of frost (1, attack) | 20 | 1 | Sniper 2–3 | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 4 (4) / 204 |
| Ray of frost (1, attack) | 30 | 1 | Sniper 2–3 | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 4 (4) / 324 |
| Ray of frost (1, attack) | 60 | 1 | Sniper 2–3 | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 1 (4) / 713 |
| Spark (1, auto) | 10 | 0–1 | – | 2d6, 6.7 (rank 0: 2d6, 6.7) | – | 4 (4) / 96 |
| Spark (1, auto) | 20 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 4 (4) / 204 |
| Spark (1, auto) | 30 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 4 (4) / 324 |
| Spark (1, auto) | 60 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 1 (4) / 713 |
| Acid splash (3, area) | 10 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 7 (7) / 96 |
| Acid splash (3, area) | 20 | 1–2 | – | 2d6, 8.0 (rank 0: 2d6, 8.0) | – | 7 (7) / 204 |
| Acid splash (3, area) | 30 | 2 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 7 (7) / 324 |
| Acid splash (3, area) | 60 | 2 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 2 (7) / 713 |
| Magic missile (3, auto) | 10 | 1–2 | – | 6d4, 14.2 (rank 0: 6d4, 14.2) | – | 4 (4) / 96 |
| Magic missile (3, auto) | 20 | 2 | – | 6d4, 15.0 (rank 0: 6d4, 15.0) | – | 4 (4) / 204 |
| Magic missile (3, auto) | 30 | 2 | – | 6d4, 15.0 (rank 0: 6d4, 15.0) | – | 4 (4) / 324 |
| Magic missile (3, auto) | 60 | 3 | – | 6d4+1, 16.0 (rank 0: 6d4+1, 16.0) | – | 1 (4) / 713 |
| Cause wounds (4, touch) | 10 | 1 | Toucher 0 | 4d6, 14.0 (rank 0: 4d6, 14.0) | – | 3 (3) / 96 |
| Cause wounds (4, touch) | 20 | 1–2 | Toucher 0 | 4d8, 16.7 (rank 0: 4d8, 16.7) | – | 3 (3) / 204 |
| Cause wounds (4, touch) | 30 | 2 | Toucher 0 | 4d8, 18.0 (rank 0: 4d8, 18.0) | – | 3 (3) / 324 |
| Cause wounds (4, touch) | 60 | 3 | Toucher 0 | 4d8+2, 20.0 (rank 0: 4d8+2, 20.0) | – | 1 (3) / 713 |
| Acid arrow (5, attack) | 10 | 1–2 | Sniper 2–3 | 6d6, 19.8 (rank 0: 6d6, 19.8) | – | 5 (5) / 96 |
| Acid arrow (5, attack) | 20 | 2 | Sniper 2–3 | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 5 (5) / 204 |
| Acid arrow (5, attack) | 30 | 2 | Sniper 2–3 | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 5 (5) / 324 |
| Acid arrow (5, attack) | 60 | 3 | Sniper 2–3 | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | – | 2 (5) / 713 |
| Call of the wild (5, auto) | 10 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 96 |
| Call of the wild (5, auto) | 20 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 204 |
| Call of the wild (5, auto) | 30 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 324 |
| Call of the wild (5, auto) | 60 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 2 (5) / 713 |
| Arcane bolt (6, area) | 10 | 1 | – | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 11 (11) / 96 |
| Arcane bolt (6, area) | 20 | 1–2 | – | 6d6, 21.7 (rank 0: 6d6, 21.7) | – | 11 (11) / 204 |
| Arcane bolt (6, area) | 30 | 2 | – | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | – | 11 (11) / 324 |
| Arcane bolt (6, area) | 60 | 3 | – | 6d6+4, 25.0 (rank 0: 6d6+4, 25.0) | – | 3 (11) / 713 |
| Frost (6, area) | 10 | 1 | – | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 14 (14) / 96 |
| Frost (6, area) | 20 | 1–2 | – | 6d6, 21.7 (rank 0: 6d6, 21.7) | – | 14 (14) / 204 |
| Frost (6, area) | 30 | 2 | – | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | – | 14 (14) / 324 |
| Frost (6, area) | 60 | 3 | – | 6d6+4, 25.0 (rank 0: 6d6+4, 25.0) | – | 4 (14) / 713 |
| Blight (7, area) | 10 | 0–1 | – | 6d6, 17.5 (rank 0: 6d6, 17.5) | – | 17 (17) / 96 |
| Blight (7, area) | 20 | 1 | – | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 17 (17) / 204 |
| Blight (7, area) | 30 | 1–2 | – | 6d6+3, 23.0 (rank 0: 6d6+3, 23.0) | – | 17 (17) / 324 |
| Blight (7, area) | 60 | 2 | – | 6d6+3, 24.0 (rank 0: 6d6+3, 24.0) | – | 5 (17) / 713 |
| Scorching ray (7, attack) | 10 | 1–2 | Sniper 2–3 | 6d6, 19.8 (rank 0: 6d6, 19.8) | – | 7 (7) / 96 |
| Scorching ray (7, attack) | 20 | 2–3 | Sniper 2–3 | 6d6, 23.3 (rank 0: 6d6, 23.3) | – | 7 (7) / 204 |
| Scorching ray (7, attack) | 30 | 3 | Sniper 2–3 | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 7 (7) / 324 |
| Scorching ray (7, attack) | 60 | 4 | Sniper 2–3 | 6d6+2d10, 32.0 (rank 0: 6d6+2d10, 32.0) | – | 2 (7) / 713 |
| Thunderclap (7, area) | 10 | 0 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 13 (13) / 96 |
| Thunderclap (7, area) | 20 | 0 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 13 (13) / 204 |
| Thunderclap (7, area) | 30 | 0 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 13 (13) / 324 |
| Thunderclap (7, area) | 60 | 2 | – | 4d6+2, 16.0 (rank 0: 4d6+2, 16.0) | – | 4 (13) / 713 |
| Flay (8, auto) | 10 | 0–1 | – | 2d6, 6.0 (rank 0: 2d6, 6.0) | – | 8 (8) / 96 |
| Flay (8, auto) | 20 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 8 (8) / 204 |
| Flay (8, auto) | 30 | 1–2 | – | 2d8, 8.3 (rank 0: 2d8, 8.3) | – | 8 (8) / 324 |
| Flay (8, auto) | 60 | 2 | – | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 2 (8) / 713 |
| Pitch fire (8, area) | 10 | 1–2 | – | 4d6, 13.4 (rank 0: 4d6, 13.4) | – | 14 (14) / 96 |
| Pitch fire (8, area) | 20 | 2 | – | 4d6, 14.0 (rank 0: 4d6, 14.0) | – | 14 (14) / 204 |
| Pitch fire (8, area) | 30 | 2 | – | 4d6, 14.0 (rank 0: 4d6, 14.0) | – | 14 (14) / 324 |
| Pitch fire (8, area) | 60 | 3 | – | 4d6+2, 16.0 (rank 0: 4d6+2, 16.0) | – | 4 (14) / 713 |
| Shatter (8, area) | 10 | 1 | – | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 14 (14) / 96 |
| Shatter (8, area) | 20 | 1–2 | – | 6d6, 21.7 (rank 0: 6d6, 21.7) | – | 14 (14) / 204 |
| Shatter (8, area) | 30 | 2 | – | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | – | 14 (14) / 324 |
| Shatter (8, area) | 60 | 3 | – | 6d6+4, 25.0 (rank 0: 6d6+4, 25.0) | – | 4 (14) / 713 |
| Rime (9, area) | 10 | 1–2 | – | 8d6, 24.5 (rank 0: 8d6, 24.5) | – | 14 (14) / 96 |
| Rime (9, area) | 20 | 2 | – | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 14 (14) / 204 |
| Rime (9, area) | 30 | 2–3 | – | 8d6+2, 29.3 (rank 0: 8d6+2, 29.3) | – | 14 (14) / 324 |
| Rime (9, area) | 60 | 3 | – | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | – | 4 (14) / 713 |
| Leech (10, attack) | 10 | 1–2 | Sniper 2–3 | 8d6, 24.5 (rank 0: 8d6, 24.5) | – | 10 (10) / 96 |
| Leech (10, attack) | 20 | 2 | Sniper 2–3 | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 10 (10) / 204 |
| Leech (10, attack) | 30 | 2 | Sniper 2–3 | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 10 (10) / 324 |
| Leech (10, attack) | 60 | 3 | Sniper 2–3 | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | – | 3 (10) / 713 |
| Call lightning (11, area) | 10 | 1–2 | – | 8d6, 24.5 (rank 0: 8d6, 24.5) | – | 20 (20) / 96 |
| Call lightning (11, area) | 20 | 2–3 | – | 10d6, 32.7 (rank 0: 10d6, 32.7) | – | 20 (20) / 204 |
| Call lightning (11, area) | 30 | 3 | – | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 20 (20) / 324 |
| Call lightning (11, area) | 60 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 6 (20) / 713 |
| Fireball (11, area) | 10 | 1–2 | – | 8d6, 26.8 (rank 0: 8d6, 26.8) | – | 26 (26) / 96 |
| Fireball (11, area) | 20 | 2–3 | – | 8d6, 30.3 (rank 0: 8d6, 30.3) | – | 26 (26) / 204 |
| Fireball (11, area) | 30 | 3 | – | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 26 (26) / 324 |
| Fireball (11, area) | 60 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 8 (26) / 713 |
| Lightning bolt (11, area) | 10 | 1–2 | – | 8d6, 26.8 (rank 0: 8d6, 26.8) | – | 20 (20) / 96 |
| Lightning bolt (11, area) | 20 | 2–3 | – | 8d6, 30.3 (rank 0: 8d6, 30.3) | – | 20 (20) / 204 |
| Lightning bolt (11, area) | 30 | 3 | – | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 20 (20) / 324 |
| Lightning bolt (11, area) | 60 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 6 (20) / 713 |
| Soulglass (11, attack) | 10 | 1–2 | Sniper 2–3 | 5d6, 15.8 (rank 0: 5d6, 15.8) | – | 11 (11) / 96 |
| Soulglass (11, attack) | 20 | 2–3 | Sniper 2–3 | 6d6, 19.8 (rank 0: 6d6, 19.8) | – | 11 (11) / 204 |
| Soulglass (11, attack) | 30 | 3 | Sniper 2–3 | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 11 (11) / 324 |
| Soulglass (11, attack) | 60 | 4 | Sniper 2–3 | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | – | 3 (11) / 713 |
| Flame (12, area) | 10 | 0–1 | – | 8d6, 26.8 (rank 0: 8d6, 26.8) | – | 29 (29) / 96 |
| Flame (12, area) | 20 | 1–2 | – | 8d6, 30.3 (rank 0: 8d6, 30.3) | – | 29 (29) / 204 |
| Flame (12, area) | 30 | 2 | – | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 29 (29) / 324 |
| Flame (12, area) | 60 | 3 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 9 (29) / 713 |
| Reek (13, area) | 10 | 0–1 | – | 4d4, 7.5 (rank 0: 4d4, 7.5) | – | 31 (31) / 96 |
| Reek (13, area) | 20 | 1 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 31 (31) / 204 |
| Reek (13, area) | 30 | 1–2 | – | 4d4+2, 11.3 (rank 0: 4d4+2, 11.3) | – | 31 (31) / 324 |
| Reek (13, area) | 60 | 2 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 10 (31) / 713 |
| Grave spear (14, area) | 10 | 1–2 | – | 8d6, 24.5 (rank 0: 8d6, 24.5) | – | 25 (25) / 96 |
| Grave spear (14, area) | 20 | 2–3 | – | 10d6, 32.7 (rank 0: 10d6, 32.7) | – | 25 (25) / 204 |
| Grave spear (14, area) | 30 | 3 | – | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 25 (25) / 324 |
| Grave spear (14, area) | 60 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 8 (25) / 713 |
| Rebuke (14, area) | 10 | 1–2 | – | 8d6, 24.5 (rank 0: 8d6, 24.5) | – | 34 (34) / 96 |
| Rebuke (14, area) | 20 | 2–3 | – | 10d6, 32.7 (rank 0: 10d6, 32.7) | – | 34 (34) / 204 |
| Rebuke (14, area) | 30 | 3 | – | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 34 (34) / 324 |
| Rebuke (14, area) | 60 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 10 (34) / 713 |
| Balefire (16, area) | 10 | 1–2 | – | 8d6, 26.8 (rank 0: 8d6, 26.8) | – | 29 (29) / 96 |
| Balefire (16, area) | 20 | 2 | – | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 29 (29) / 204 |
| Balefire (16, area) | 30 | 2 | – | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 29 (29) / 324 |
| Balefire (16, area) | 60 | 3 | – | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | – | 9 (29) / 713 |
| Call of the forest (20, auto) | 10 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 96 |
| Call of the forest (20, auto) | 20 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 204 |
| Call of the forest (20, auto) | 30 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 324 |
| Call of the forest (20, auto) | 60 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 6 (20) / 713 |
| Chain lightning (20, auto) | 10 | 1–2 | – | 10d6, 33.8 (rank 0: 10d6, 33.8) | – | 20 (20) / 96 |
| Chain lightning (20, auto) | 20 | 2–3 | – | 10d6, 37.3 (rank 0: 10d6, 37.3) | – | 20 (20) / 204 |
| Chain lightning (20, auto) | 30 | 3 | – | 12d6, 42.0 (rank 0: 12d6, 42.0) | – | 20 (20) / 324 |
| Chain lightning (20, auto) | 60 | 4 | – | 12d6+3, 45.0 (rank 0: 12d6+3, 45.0) | – | 6 (20) / 713 |
| Sunfire (30, area) | 10 | 0–1 | – | 12d6, 40.8 (rank 0: 12d6, 40.8) | – | 90 (90) / 96 |
| Sunfire (30, area) | 20 | 1–2 | – | 12d6, 44.3 (rank 0: 12d6, 44.3) | – | 90 (90) / 204 |
| Sunfire (30, area) | 30 | 2 | – | 14d6, 49.0 (rank 0: 14d6, 49.0) | – | 90 (90) / 324 |
| Sunfire (30, area) | 60 | 4 | – | 16d6+4, 60.0 (rank 0: 16d6+4, 60.0) | – | 29 (90) / 713 |
| Call the elements (35, auto) | 10 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 96 |
| Call the elements (35, auto) | 20 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 204 |
| Call the elements (35, auto) | 30 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 324 |
| Call the elements (35, auto) | 60 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 11 (35) / 713 |

**Priest**

| Spell (level, res) | L | rung | style rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Mend (1, touch) | 10 | 0–1 | Battle caster 0 | 1d4, 2.4 (rank 0: 1d4, 2.4) | – | 3 (3) / 76 |
| Mend (1, touch) | 20 | 1 | Battle caster 5–6 | 1d4, 2.5 (rank 0: 1d4, 2.5) | – | 4 (3) / 158 |
| Mend (1, touch) | 30 | 1–2 | Battle caster 6 | 1d4, 2.7 (rank 0: 1d4, 2.7) | – | 4 (3) / 246 |
| Mend (1, touch) | 60 | 1–2 | Battle caster 7–8 | 1d4, 3.2 (rank 0: 1d4, 3.2) | – | 4 (3) / 534 |
| Cure wounds (3, touch) | 10 | 0–1 | Battle caster 0 | 3d6, 9.9 (rank 0: 3d6, 9.9) | – | 3 (3) / 76 |
| Cure wounds (3, touch) | 20 | 1 | Battle caster 5–6 | 3d6, 10.5 (rank 0: 3d6, 10.5) | – | 4 (3) / 158 |
| Cure wounds (3, touch) | 30 | 1–2 | Battle caster 6 | 3d6, 11.0 (rank 0: 3d6, 11.0) | – | 4 (3) / 246 |
| Cure wounds (3, touch) | 60 | 1–2 | Battle caster 7–8 | 3d6, 12.5 (rank 0: 3d6, 12.5) | – | 4 (3) / 534 |
| Benison (12, area) | 10 | 0–1 | Battle caster 0 | 3d6, 9.9 (rank 0: 3d6, 9.9) | – | 24 (22) / 76 |
| Benison (12, area) | 20 | 1 | Battle caster 5–6 | 3d6, 10.5 (rank 0: 3d6, 10.5) | – | 28 (22) / 158 |
| Benison (12, area) | 30 | 1–2 | Battle caster 6 | 3d6, 11.0 (rank 0: 3d6, 11.0) | – | 29 (22) / 246 |
| Benison (12, area) | 60 | 1–2 | Battle caster 7–8 | 3d6, 12.5 (rank 0: 3d6, 12.5) | – | 29 (22) / 534 |
| Rebuke (14, area) | 10 | 1–2 | Battle caster 0 | 8d6, 26.8 (rank 0: 8d6, 26.8) | – | 37 (34) / 76 |
| Rebuke (14, area) | 20 | 2–3 | Battle caster 5–6 | 8d6, 32.7 (rank 0: 8d6, 32.7) | – | 44 (34) / 158 |
| Rebuke (14, area) | 30 | 2–3 | Battle caster 6 | 10d6, 33.8 (rank 0: 10d6, 33.8) | – | 45 (34) / 246 |
| Rebuke (14, area) | 60 | 3 | Battle caster 7–8 | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 44 (34) / 534 |
| Orison (prayer) | 10 | – | – | no roll: answers 1 of 2 (wounds, hunger) | – | 0 (0) / 76 |
| Prayer (prayer) | 20 | – | – | no roll: answers 1 of 5 (disease, poison, wounds, release, hunger) | – | 0 (0) / 158 |
| Litany (prayer) | 30 | – | – | no roll: answers 1 of 6 (disease, poison, wounds, release, hunger, restoration) | – | 0 (0) / 246 |
| Miracle (prayer) | 60 | – | – | no roll: answers all of 7 (disease, poison, wounds, release, mana, hunger, restoration) | – | 0 (0) / 534 |

**Witch**

| Spell (level, res) | L | rung | style rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Grave breath (1, area) | 10 | 2 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 6 (6) / 84 |
| Grave breath (1, area) | 20 | 2–3 | – | 4d4+2, 10.7 (rank 0: 4d4+2, 10.7) | 13.7 | 6 (6) / 188 |
| Grave breath (1, area) | 30 | 2–3 | – | 4d4+2, 11.3 (rank 0: 4d4+2, 11.3) | 14.3 | 6 (6) / 302 |
| Grave breath (1, area) | 60 | 3 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | 15.0 | 5 (6) / 676 |
| Cause wounds (4, touch) | 10 | 1–2 | Toucher 3 | 4d8, 14.7 (rank 0: 4d8, 14.7) | – | 3 (3) / 84 |
| Cause wounds (4, touch) | 20 | 2 | Toucher 3 | 4d8, 18.0 (rank 0: 4d8, 18.0) | 21.0 | 3 (3) / 188 |
| Cause wounds (4, touch) | 30 | 2 | Toucher 3 | 4d8, 18.0 (rank 0: 4d8, 18.0) | 21.0 | 3 (3) / 302 |
| Cause wounds (4, touch) | 60 | 3 | Toucher 3 | 4d8+2, 20.0 (rank 0: 4d8+2, 20.0) | 23.0 | 2 (3) / 676 |
| Call of the wild (5, auto) | 10 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 84 |
| Call of the wild (5, auto) | 20 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 188 |
| Call of the wild (5, auto) | 30 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 302 |
| Call of the wild (5, auto) | 60 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 4 (5) / 676 |
| Blight (7, area) | 10 | 1 | – | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 17 (17) / 84 |
| Blight (7, area) | 20 | 1–2 | – | 6d6+3, 22.0 (rank 0: 6d6+3, 22.0) | 25.0 | 17 (17) / 188 |
| Blight (7, area) | 30 | 1–2 | – | 6d6+3, 23.0 (rank 0: 6d6+3, 23.0) | 26.0 | 17 (17) / 302 |
| Blight (7, area) | 60 | 2 | – | 6d6+3, 24.0 (rank 0: 6d6+3, 24.0) | 27.0 | 14 (17) / 676 |
| Flay (8, auto) | 10 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 8 (8) / 84 |
| Flay (8, auto) | 20 | 1–2 | – | 2d8, 7.7 (rank 0: 2d8, 7.7) | 10.7 | 8 (8) / 188 |
| Flay (8, auto) | 30 | 1–2 | – | 2d8, 8.3 (rank 0: 2d8, 8.3) | 11.3 | 8 (8) / 302 |
| Flay (8, auto) | 60 | 2 | – | 2d8, 9.0 (rank 0: 2d8, 9.0) | 12.0 | 6 (8) / 676 |
| Leech (10, attack) | 10 | 2 | Sniper 2 | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 10 (10) / 84 |
| Leech (10, attack) | 20 | 2 | Sniper 3–4 | 8d6, 28.0 (rank 0: 8d6, 28.0) | 31.0 | 10 (10) / 188 |
| Leech (10, attack) | 30 | 2 | Sniper 3–4 | 8d6, 28.0 (rank 0: 8d6, 28.0) | 31.0 | 10 (10) / 302 |
| Leech (10, attack) | 60 | 3 | Sniper 3–4 | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | 33.0 | 8 (10) / 676 |
| Soulglass (11, attack) | 10 | 2–3 | Sniper 2 | 6d6, 18.1 (rank 0: 6d6, 18.1) | – | 11 (11) / 84 |
| Soulglass (11, attack) | 20 | 3 | Sniper 3–4 | 6d6, 21.0 (rank 0: 6d6, 21.0) | 24.0 | 11 (11) / 188 |
| Soulglass (11, attack) | 30 | 3 | Sniper 3–4 | 6d6, 21.0 (rank 0: 6d6, 21.0) | 24.0 | 11 (11) / 302 |
| Soulglass (11, attack) | 60 | 4 | Sniper 3–4 | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | 26.0 | 9 (11) / 676 |
| Reek (13, area) | 10 | 1 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 31 (31) / 84 |
| Reek (13, area) | 20 | 1–2 | – | 4d4+2, 10.7 (rank 0: 4d4+2, 10.7) | 13.7 | 31 (31) / 188 |
| Reek (13, area) | 30 | 1–2 | – | 4d4+2, 11.3 (rank 0: 4d4+2, 11.3) | 14.3 | 31 (31) / 302 |
| Reek (13, area) | 60 | 2 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | 15.0 | 25 (31) / 676 |
| Grave spear (14, area) | 10 | 2–3 | – | 10d6, 29.2 (rank 0: 10d6, 29.2) | – | 25 (25) / 84 |
| Grave spear (14, area) | 20 | 3 | – | 10d6, 35.0 (rank 0: 10d6, 35.0) | 38.0 | 25 (25) / 188 |
| Grave spear (14, area) | 30 | 3 | – | 10d6, 35.0 (rank 0: 10d6, 35.0) | 38.0 | 25 (25) / 302 |
| Grave spear (14, area) | 60 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | 40.0 | 20 (25) / 676 |
| Call of the forest (20, auto) | 10 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 84 |
| Call of the forest (20, auto) | 20 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 188 |
| Call of the forest (20, auto) | 30 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 302 |
| Call of the forest (20, auto) | 60 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 16 (20) / 676 |
| Call the elements (35, auto) | 10 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 84 |
| Call the elements (35, auto) | 20 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 188 |
| Call the elements (35, auto) | 30 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 302 |
| Call the elements (35, auto) | 60 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 28 (35) / 676 |

The three tables are #1263's run (Malison's hexed column as #905 made it: a damage spell's mean on a target under one hex, from the witch's level 18). With the schools of magic gone as trained skills, a spell's roll is its ladder's rung with the talents' and the items' hooks (none in plain gear), so the roll and its style-rank-0 roll are the same but for Toucher's point on a touch from rank 4. The style rank column is the caster style a spell is cast in: Sniper for a ray or a bolt that rolls to hit, Toucher for a touch that does, Battle caster in hindering armour, `–` for the rest. Against #1251's run, at level 60: the wizard's Fireball rolls 10d6+2, 37.0 (it was 11d7+2 with 1s and 2s again, 54.6, at evocation 7) and Scorching ray 6d6+2d10, 32.0 (47.4); the witch's Grave spear 10d6+2, 37.0 (50.8) and Cause wounds 4d8+2, 20.0 (31.2); the priest's Cure wounds 3d6+3, 13.0 (23.4). Every roll is now its rank-0 roll of before: the schools' ranks were the only thing between the two columns. The mana is the cost table's price with the talents' cuts and the armour's extra (no rank's cut).

## Caster style pace

The trained caster's caster style as their casting trained it (`balance_probe/caster_pace.rs`): each day of each level, one pool spent on the pace spell (of the school's spells, the most a cast among those costing at most a fifth of the pool, never one cast only under the open sky; for the priest a heal, at most one a kill), every cast landing on a monster of the caster's level through `World::practise`. Each row: the pace spell over the level before and its cost (seed 1's); casts a day from the pool, and the spell casts a day the same level's fights took (the stat training's careers, which refill the pool before every cast; the priest's one a kill); the caster style's rank at the level (the seeds' range); and the pool check, the pool over a single-target spell of the caster's own level (its base mana: the level; no talent).

**Wizard** (evocation)

| L | pace spell (mana) | casts a day: pool / fights | style rank | pool / level-L casts |
|---|---|---|---|---|
| 1 | Burning hands (6) | 2.2 / 0.0 | – | 16 / 1 = 16.5 |
| 5 | Scorching ray (7) | 5.5 / 57.8 | Sniper 1–2 | 49 / 5 = 9.9 |
| 10 | Rime (14) | 5.8 / 98.1 | – | 96 / 10 = 9.6 |
| 15 | Chain lightning (20) | 6.5 / 144.5 | – | 148 / 15 = 9.9 |
| 20 | Chain lightning (20) | 9.2 / 134.6 | – | 204 / 20 = 10.2 |
| 30 | Chain lightning (20) | 15.0 / 197.6 | – | 324 / 30 = 10.8 |
| 40 | Sunfire (81) | 5.3 / 171.5 | – | 448 / 40 = 11.2 |
| 50 | Sunfire (65) | 8.5 / 255.5 | – | 577 / 50 = 11.5 |
| 60 | Sunfire (65) | 10.5 / 255.4 | – | 713 / 60 = 11.9 |

**Priest** (restoration)

| L | pace spell (mana) | casts a day: pool / fights | style rank | pool / level-L casts |
|---|---|---|---|---|
| 1 | Mend (3) | 4.0 / 0.0 | – | 13 / 1 = 13.0 |
| 5 | Cure wounds (3) | 10.7 / 28.0 | – | 39 / 5 = 7.9 |
| 10 | Cure wounds (3) | 22.5 / 49.0 | – | 76 / 10 = 7.7 |
| 15 | Cure wounds (3) | 30.0 / 65.5 | Battle caster 4–5 | 116 / 15 = 7.7 |
| 20 | Benison (25) | 5.5 / 49.7 | Battle caster 5–6 | 158 / 20 = 7.9 |
| 30 | Benison (28) | 8.0 / 58.7 | Battle caster 6 | 246 / 30 = 8.2 |
| 40 | Benison (29) | 11.0 / 50.0 | Battle caster 6–7 | 338 / 40 = 8.4 |
| 50 | Benison (29) | 14.2 / 55.5 | Battle caster 7 | 434 / 50 = 8.7 |
| 60 | Benison (29) | 17.7 / 60.8 | Battle caster 7–8 | 534 / 60 = 8.9 |

**Witch** (necromancy)

| L | pace spell (mana) | casts a day: pool / fights | style rank | pool / level-L casts |
|---|---|---|---|---|
| 1 | Grave breath (6) | 1.8 / 0.0 | – | 14 / 1 = 13.7 |
| 5 | Cause wounds (3) | 11.0 / 51.8 | Toucher 2 | 41 / 5 = 8.2 |
| 10 | Leech (10) | 6.8 / 130.1 | Sniper 2 | 84 / 10 = 8.3 |
| 15 | Grave spear (25) | 7.0 / 176.6 | – | 136 / 15 = 9.1 |
| 20 | Grave spear (25) | 6.7 / 138.6 | – | 188 / 20 = 9.4 |
| 30 | Grave spear (25) | 11.2 / 229.2 | – | 302 / 30 = 10.1 |
| 40 | Grave spear (25) | 17.3 / 259.6 | – | 419 / 40 = 10.5 |
| 50 | Grave spear (20) | 26.0 / 396.6 | – | 544 / 50 = 10.9 |
| 60 | Grave spear (20) | 32.7 / 395.8 | – | 676 / 60 = 11.3 |

The level 1 row has no level before it and practises nothing: it says what a starting caster's first pool buys of the spells they know (`caster_pace::first_spell`: an area before a single target, then the most mean roll; a healer's heal), #979.

These tables are #1263's run (the schools of magic no longer trained: the pace spell trains only the caster style it is cast in, on the `caster` curve, #1251's `school` curve renamed). The style column is the pace spell's: `–` when it is cast in no style (an area such as Rime or Chain lightning, a heal out of hindering armour), so a Sniper rank the wizard trained at level 5 is not shown once the pace spell becomes an area. The pace spell's cost no longer falls with a rank (Sunfire 65 at level 60, 58 before; Grave spear 20, 18), and the priest stays on Cure wounds to level 15, Benison from 20.

Casts that land on a foe at least as strong as the rank (3 marks each) to reach a rank of a caster style: rank 1 10, rank 4 150, rank 10 7,470.

## Weapon pace

The trained fighters' weapon skill and fighting style as their blows trained them (#1095; `balance_probe/training.rs`): every blow of the stat training's fights, each level's kills against the monsters within two levels of theirs, earns its marks through `World::practise` (the real `[marks]`, curves, and talents), the main hand's skill and style for both hands. Each cell: the level at which the skill first stood at the rank (the range over the seeds that fought with it; `>60` where a seed had not reached it by level 60). The untrained careers take these ranks from their trained twin; the skills the fights do not train (a shield, a second weapon skill) are set by level (`rank`, the warrior's pace).

**Warrior**

| rank | Long blades | Sword and Board |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 4–5 | 4–5 |
| 4 | 6–7 | 6–7 |
| 5 | 8–9 | 8–9 |
| 6 | 10–12 | 11–12 |
| 7 | 16–19 | 17–19 |
| 8 | 24–29 | 26–29 |
| 9 | 33–41 | 38–41 |
| 10 | 43–53 | 49–53 |

**Thief**

| rank | Daggers and knives | Duelist (3 of 6 seeds) | Twin Blades (3 of 6 seeds) |
|---|---|---|---|
| 1 | 2 | 2 | 2 |
| 2 | 2 | 2 | 2 |
| 3 | 2–4 | 3–4 | 2–4 |
| 4 | 5–6 | 5–6 | 5–6 |
| 5 | 6–7 | 7 | 6–7 |
| 6 | 8–9 | 9 | 8–9 |
| 7 | 14–15 | 14–15 | 14–15 |
| 8 | 21–23 | 21–23 | 21–22 |
| 9 | 30–33 | 30–33 | 30–32 |
| 10 | 40–43 | 40–43 | 40–41 |

**Priest**

| rank | Maces and clubs | Sword and Board |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 4 | 4 |
| 4 | 6 | 6 |
| 5 | 8–9 | 8–9 |
| 6 | 10–11 | 10–11 |
| 7 | 16–18 | 16–18 |
| 8 | 25–27 | 25–27 |
| 9 | 35–39 | 35–39 |
| 10 | 45–51 | 45–51 |

**Berserker**

| rank | Axes | Great Weapon |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 4–5 | 4–5 |
| 4 | 6–7 | 6–7 |
| 5 | 9–10 | 9–10 |
| 6 | 12–13 | 12–13 |
| 7 | 20–22 | 20–22 |
| 8 | 30–34 | 30–34 |
| 9 | 45–50 | 45–50 |
| 10 | 59–>60 | 59–>60 |

**Ascetic**

| rank | Long blades (1 of 6 seeds) | Staves (3 of 6 seeds) | Unarmed (2 of 6 seeds) | Duelist (1 of 6 seeds) | Brawler (2 of 6 seeds) | Staff Fighter (3 of 6 seeds) |
|---|---|---|---|---|---|---|
| 1 | 2 | 2 | 2 | 2 | 2 | 2 |
| 2 | 3 | 3 | 2 | 3 | 2 | 3 |
| 3 | 5 | 4 | 4 | 5 | 4 | 4 |
| 4 | 7 | 6 | 6 | 7 | 6 | 6 |
| 5 | 10 | 8 | 8 | 10 | 8 | 8 |
| 6 | 13 | 10–11 | 11 | 13 | 11 | 10–11 |
| 7 | 21 | 16–17 | 18–19 | 21 | 18–19 | 16–17 |
| 8 | 31 | 25–26 | 27–30 | 31 | 27–30 | 25–26 |
| 9 | 44 | 35–36 | 39–43 | 44 | 39–43 | 35–36 |
| 10 | 56 | 46 | 51–57 | 56 | 51–57 | 46 |

**Paladin**

| rank | Long blades | Sword and Board |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 4–5 | 4–5 |
| 4 | 6–7 | 6–7 |
| 5 | 8–9 | 8–9 |
| 6 | 11–12 | 11–12 |
| 7 | 18–19 | 18–19 |
| 8 | 27–29 | 27–29 |
| 9 | 38–42 | 38–42 |
| 10 | 48–53 | 48–53 |

**Reading it** (#1095). The warrior, the priest, and the paladin land on the issue's aim: rank 6
about level 10, rank 8 about 21 to 26, rank 10 about 37 to 47. With the cap at 100 % but the
old exponential curves (15, 30, … 7680) the same warrior stood at rank 6 by level 6 and rank 10
by 13 to 15 (measured on the way); with marks up to 200 % before #1095, sooner still (the
issue's level-13 warrior at rank 9). A fighter with two blows a round
trains faster (the thief's daggers, both hands, rank 10 by 32 to 37), one with slow, heavy blows
slower (the berserker's great axe, rank 10 by 52 to 57). The warrior's Weapon drill (+25 % marks
in melee skills) is in these numbers: each rank takes a fifth fewer blows, worth more levels now
that the last ranks take a career, so it still earns its place. Practised technique asks for rank 7,
now about level 15 instead of 7. The caster styles keep a gentler curve of their own
(`caster`, once `school`, flatter than doubling from rank 8): with the 100 % cap the wizard's evocation is
rank 4 at level 10, 6 at 20, 8 at 40, and 9 at 60 (Sniper 9 to 10; #1095's run, before #1263 removed the schools), where before
#1095 it was 5, 7, 9, and 10.

## Sneak attacks

The thief's main-hand blow on the monsters of their level (trained stats), aware and caught unawares. The Sneak attack value assumed, the dice it gives with the talents, and the share of the monsters within two levels with vitals (no dice and no critical on the rest, so their first blow gains only the flat-footed Defence); then each cell: P(hit) aware / unaware % · damage of a blow aware : the first blow unaware · how many times as much, the median over the monsters within two levels that have vitals.

| L | Sneak attack | dice | with vitals | Thief plain | Thief magic |
|---|---|---|---|---|---|
| 1 | 15 | 1d6 | 82% | 72/78 · 2.7:6.0 · ×2.2 | 72/78 · 2.7:6.0 · ×2.2 |
| 5 | 29 | 2d6 | 62% | 77/86 · 3.6:10.1 · ×2.8 | 77/86 · 4.6:11.3 · ×2.4 |
| 10 | 47 | 2d6 | 75% | 92/97 · 6.9:14.0 · ×2.0 | 94/97 · 8.0:15.1 · ×1.9 |
| 15 | 64 | 3d6 | 64% | 98/99 · 9.8:20.5 · ×2.1 | 99/99 · 11.2:21.8 · ×1.9 |
| 20 | 82 | 4d6 | 65% | 98/99 · 9.8:23.8 · ×2.4 | 99/99 · 11.4:25.3 · ×2.2 |
| 30 | 117 | 5d6 | 67% | 99/99 · 12.9:30.5 · ×2.4 | 99/99 · 20.6:38.0 · ×1.8 |
| 40 | 152 | 8d6 | 62% | 99/99 · 14.5:42.5 · ×2.9 | 99/99 · 20.2:47.7 · ×2.4 |
| 50 | 187 | 8d6 | 58% | 99/99 · 15.6:43.8 · ×2.8 | 99/99 · 22.9:50.8 · ×2.2 |
| 60 | 222 | 9d6 | 50% | 96/99 · 16.1:50.3 · ×3.1 | 99/99 · 19.6:53.6 · ×2.7 |

## Exploits

The thief's Exploit (plain gear, trained stats) on the monsters of their level with vitals, aware of them: the openings' blow keeps the foe's Defence and lands the sneak dice and n × the ladder's on a hit. The ladder (per opening spent) as the talents give it; then each cell: P(hit) % · damage, misses included, the median over the monsters within two levels that have vitals. Plain: the blow on the aware; Unaware: the first blow on a foe caught unawares (flat-footed); Thrown 3: the thrown exploit of three openings (from level 18) with the first throwing weapon carried (a dagger), within the sneak dice's reach.

| L | ladder an opening | Plain | Unaware | Exploit 1 | Exploit 3 | Exploit 5 | Thrown 3 |
|---|---|---|---|---|---|---|---|
| 1 | 1d6 | 72 · 2.7 | 78 · 5.9 | 72 · 8.2 | 72 · 13.4 | 72 · 18.6 | - |
| 5 | 1d6 | 77 · 3.6 | 86 · 10.0 | 77 · 11.6 | 77 · 17.0 | 77 · 22.6 | - |
| 10 | 1d6 | 92 · 6.9 | 97 · 14.0 | 92 · 16.3 | 92 · 22.6 | 92 · 29.0 | - |
| 15 | 1d6, +1 hit, stun | 98 · 9.7 | 99 · 20.4 | 99 · 23.2 | 99 · 30.7 | 99 · 37.8 | - |
| 20 | 1d6, +1 hit, stun | 98 · 9.8 | 99 · 23.8 | 99 · 27.2 | 99 · 34.4 | 99 · 41.3 | 97 · 30.5 |
| 30 | 1d6, +1 hit, stun | 99 · 13.0 | 99 · 30.6 | 99 · 33.9 | 99 · 40.9 | 99 · 48.0 | 95 · 32.0 |
| 40 | 2d6, +1 hit, stun | 99 · 14.6 | 99 · 42.5 | 99 · 49.2 | 99 · 63.1 | 99 · 77.2 | 89 · 51.0 |
| 50 | 2d6, +1 hit, stun | 99 · 15.8 | 99 · 43.8 | 99 · 50.4 | 99 · 64.6 | 99 · 78.5 | 84 · 47.5 |
| 60 | 2d6, +1 hit, stun, threat −4, sure at 5 | 96 · 16.1 | 99 · 50.3 | 98 · 61.2 | 99 · 80.0 | 99 · 94.5 | 71 · 47.3 |

The openings a fight against one of those monsters yields (the inflow; Exploit's spending is not modelled), the thief feinting whenever the foe is aware and not yet fooled, then striking it flat-footed: the chance a feint fools it (its Willpower check against 10 + the thief's Presence modifier + Bluff), the first blow's opening (the chance it hits), the feints' openings with the sneak attack each fooling sets up, the mark's kill (1), the whole fight's, and its turns, the median over the monsters. A thief who cannot feint (untrained in Bluff) only strikes.

| L | feint fools | first blow | feints | the kill | a fight | turns |
|---|---|---|---|---|---|---|
| 1 | 12% | 0.78 | 0.5 | 1 | 2.2 | 3.9 |
| 5 | 9% | 0.86 | 0.9 | 1 | 2.8 | 7.1 |
| 10 | 6% | 0.97 | 1.9 | 1 | 3.9 | 13.7 |
| 15 | 4% | 0.99 | 2.1 | 1 | 4.0 | 19.2 |
| 20 | 1% | 0.99 | 2.1 | 1 | 4.1 | 38.4 |
| 30 | 0% | 0.99 | 3.1 | 1 | 5.1 | 97.2 |
| 40 | 0% | 0.99 | 2.4 | 1 | 4.4 | 140.5 |
| 50 | 0% | 0.99 | 5.6 | 1 | 7.6 | 406.1 |
| 60 | 0% | 0.99 | 3.6 | 1 | 5.6 | 267.1 |

Read with the openings design §10's first pass (#1464's run): Exploit is close to the design's
numbers below 32 (an exploit of 3 at 30 is 40.7 against its 41.9), and from 32 the second die
makes an exploit of 5 about five times the plain blow (76.9 against 14.3 at 40); at 60 the sure
critical lifts an exploit of 5 to 94.3 (the design's ≈ 103). The feint is the weak build: the
monsters' Willpower beats a feint of difficulty 10 + Presence + Bluff (the probe's thief spends
no level-up rolls on Bluff, `levelup:auto`) nearly always, 12 % fooled at level 1 and none from
30, so a fight's openings come from the first blow and the kill (about 2 a fight), not from
feints, and the "turns" column shows a feint-first policy is a loss past level 10. Not tuned
here: whether Feint's difficulty or Bluff's `feint` points should rise is a later balance
change.

## Rage

The berserker in plain gear (a great axe; from level 5 the warrior's armour: chain mail, banded mail from 15, plate from 30), trained stats, against the monsters within two levels: calm (the stance the run starts in) and raging (Raging and the frenzy stance). The pool as the Rage ability grants it (start / most); the fight cells as in "Fights"; then the rage a round against one foe, the medians of hits taken × their build + criticals × theirs + kills × theirs − the decay (kills a round: one over the rounds to kill, so a fight's average); the same without the kills (a foe that outlasts the rage, or the turns before the first kill) and how many rounds the rage lasts from its start at that net (`grows` when it does not fall); and the net against three foes (three foes' hits, the same criticals and kills).

| L | rage | Calm | Raging | one foe: struck + crit + kill − decay = net | one foe that does not die: net · rounds the rage lasts | three foes: net |
|---|---|---|---|---|---|---|
| 1 | 12/20 | 87/70 · 8.6:0.7 · 1.1 / 39 | 97/96 · 13.9:0.9 · 0.6 / 30 | 0.97 + 0.07 + 4.73 − 1.00 = +4.78 | +0.05 · grows | +6.72 |
| 5 | 12/20 | 83/65 · 7.6:1.0 · 2.6 / 58 | 94/94 · 13.1:1.4 · 1.5 / 38 | 0.96 + 0.06 + 2.01 − 1.00 = +2.04 | +0.02 · grows | +3.96 |
| 10 | 12/20 | 88/78 · 10.8:3.2 · 4.0 / 26 | 97/98 · 16.7:3.3 · 2.5 / 28 | 0.98 + 0.07 + 1.23 − 1.00 = +1.27 | +0.04 · grows | +3.23 |
| 15 | 12/20 | 89/91 · 12.1:5.4 · 5.4 / 21 | 98/99 · 17.5:4.7 · 3.4 / 25 | 0.98 + 0.06 + 0.92 − 1.00 = +0.95 | +0.04 · grows | +2.91 |
| 20 | 12/20 | 90/95 · 13.3:6.7 · 7.3 / 24 | 97/99 · 19.6:6.4 · 4.8 / 24 | 1.00 + 0.07 + 0.66 − 1.00 = +0.73 | +0.06 · grows | +2.72 |
| 30 | 12/20 | 92/99 · 17.0:14.3 · 8.9 / 17 | 99/99 · 23.4:12.8 · 6.4 / 19 | 0.99 + 0.09 + 0.48 − 0.89 = +0.67 | +0.19 · grows | +2.66 |
| 40 | 18/20 | 93/99 · 25.6:16.6 · 7.3 / 22 | 99/99 · 45.9:15.7 · 4.5 / 24 | 1.04 + 0.16 + 1.78 − 0.67 = +2.31 | +0.53 · grows | +4.40 |
| 50 | 18/30 | 93/99 · 36.1:25.3 · 7.3 / 19 | 99/99 · 66.0:23.5 · 3.9 / 21 | 1.05 + 0.19 + 2.08 − 0.67 = +2.65 | +0.57 · grows | +4.74 |
| 60 | 18/30 | 86/99 · 31.4:26.5 · 7.5 / 25 | 97/99 · 60.4:24.6 · 4.2 / 27 | 1.18 + 0.12 + 1.92 − 0.67 = +2.55 | +0.63 · grows | +4.91 |

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
| 1 | 5.3 | 0.33 | 86/63 · 5.8:1.1 · 1.6 / 21 | 86/63 · 11.0:1.1 · 0.9 / 20 | 0.99 − 0.33 = 0.66 · 8.1 |
| 5 | 5.3 | 0.33 | 89/71 · 7.3:3.1 · 2.8 / 10 | 89/71 · 13.8:3.2 · 1.5 / 10 | 1.00 − 0.33 = 0.67 · 8.0 |
| 10 | 6.3 | 0.33 | 94/81 · 12.8:6.6 · 3.4 / 7.5 | 94/81 · 24.6:6.6 · 1.8 / 7.6 | 1.10 − 0.33 = 0.77 · 8.2 |
| 15 | 7.7 | 0.33 | 99/85 · 18.2:10.5 · 3.6 / 7.3 | 99/85 · 35.9:10.6 · 1.8 / 7.2 | 1.18 − 0.33 = 0.85 · 9.1 |
| 20 | 8.8 | 0.33 | 99/89 · 20.5:13.2 · 4.2 / 8.0 | 99/89 · 40.5:13.1 · 2.2 / 8.1 | 1.21 − 0.33 = 0.88 · 10 |
| 30 | 11.3 | 0.33 | 99/98 · 24.0:22.6 · 5.4 / 7.5 | 99/98 · 47.7:22.3 · 2.7 / 7.7 | 1.30 − 0.33 = 0.97 · 12 |
| 40 | 13.3 | 0.33 | 99/96 · 32.5:24.6 · 5.7 / 9.9 | 99/96 · 64.2:25.1 · 3.0 / 9.8 | 1.39 − 0.33 = 1.06 · 13 |
| 50 | 15.7 | 0.33 | 99/98 · 41.2:29.8 · 6.2 / 11 | 99/98 · 82.6:29.9 · 3.1 / 11 | 1.48 − 0.33 = 1.14 · 14 |
| 60 | 20.0 | 0.83 | 99/99 · 51.3:36.4 · 5.2 / 8.9 | 99/99 · 99.8:36.1 · 2.7 / 9.0 | 1.52 − 0.83 = 0.69 · 29 |

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
| 1 | 10.8 | 0.10 | 84/49 · 7.0:0.1 · 1.3 / 690 | 87/49 · 10.7:0.1 · 0.9 / 805 | 87/41 · 13.4:0.9 · 0.8 / 35 | - | 1.42 · 2.1 | 1.90 · 1.6 |
| 5 | 10.8 | 0.10 | 84/50 · 5.5:1.0 · 3.7 / 36 | 87/50 · 8.4:1.0 · 2.4 / 38 | 91/55 · 11.1:1.5 · 1.7 / 22 | - | 1.27 · 2.4 | 1.77 · 1.7 |
| 10 | 11.8 | 0.10 | 89/51 · 7.8:2.6 · 5.3 / 23 | 91/51 · 10.9:2.7 · 3.9 / 23 | 91/46 · 14.0:2.6 · 2.3 / 22 | - | 1.32 · 2.3 | 1.83 · 1.6 |
| 15 | 11.8 | 0.10 | 94/61 · 9.0:4.0 · 7.0 / 20 | 96/61 · 12.1:4.0 · 5.2 / 20 | 94/60 · 15.9:5.2 · 3.3 / 16 | - | 1.46 · 2.1 | 2.07 · 1.5 |
| 20 | 12.8 | 0.10 | 96/55 · 11.1:4.7 · 8.2 / 27 | 98/55 · 17.2:4.5 · 5.3 / 27 | 97/56 · 20.2:6.3 · 4.5 / 19 | - | 1.42 · 2.1 | 1.97 · 1.5 |
| 30 | 14.3 | 0.10 | 98/66 · 12.1:9.6 · 12 / 20 | 99/66 · 22.2:10.2 · 6.4 / 19 | 98/69 · 25.4:11.8 · 3.8 / 16 | - | 1.55 · 1.9 | 2.21 · 1.4 |
| 40 | 15.8 | 0.10 | 96/76 · 15.2:13.4 · 13 / 21 | 99/76 · 29.0:13.5 · 6.4 / 21 | 95/86 · 32.3:21.3 · 4.1 / 13 | - | 1.64 · 1.8 | 2.40 · 1.2 |
| 50 | 17.0 | 0.10 | 96/70 · 15.8:17.3 · 16 / 21 | 99/70 · 33.1:17.4 · 7.6 / 21 | 99/82 · 38.4:22.2 · 7.5 / 16 | 99/82 · 59.3:22.2 · 5.0 / 16 | 1.58 · 1.9 | 2.27 · 1.3 |
| 60 | 18.7 | 0.10 | 92/60 · 17.5:15.0 · 13 / 25 | 98/60 · 36.2:15.4 · 6.5 / 25 | 96/96 · 41.1:37.1 · 4.0 / 9.8 | 96/96 · 64.7:36.8 · 2.6 / 9.9 | 1.47 · 2.0 | 2.07 · 1.4 |

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
| 1 | hound, boar | 9 | 79/54 · 4.6:0.5 · 2.0 / 42 | 77/70 · 4.5:1.5 · 2.5 / 6.0 | 1.1 | x1.8 |
| 5 | hound, boar | 16 | 79/63 · 4.6:2.4 · 4.5 / 14 | 73/78 · 4.1:3.8 · 6.5 / 4.4 | 2.6 | x1.7 |
| 10 | hound, boar | 27 | 80/75 · 5.6:5.9 · 7.7 / 7.4 | 83/84 · 6.6:6.9 · 8.6 / 3.9 | 4.1 | x1.9 |
| 15 | hound, boar | 38 | 84/79 · 6.8:9.2 · 9.4 / 6.0 | 87/87 · 10.6:11.1 · 7.8 / 3.4 | 4.3 | x2.2 |
| 20 | hound, boar | 49 | 80/88 · 5.8:13.3 · 15 / 5.1 | 91/87 · 12.6:13.0 · 8.6 / 3.7 | 5.4 | x2.7 |
| 30 | hound, boar | 71 | 79/96 · 5.1:21.5 · 24 / 4.3 | 98/94 · 18.8:21.0 · 7.8 / 3.4 | 5.9 | x4.1 |
| 40 | hound, boar | 93 | 66/99 · 6.3:25.6 · 29 / 4.9 | 98/94 · 25.1:22.0 · 7.0 / 4.3 | 5.6 | x5.2 |
| 50 | hound, boar | 116 | 67/99 · 7.0:32.1 · 39 / 4.4 | 99/92 · 33.8:28.7 · 7.6 / 4.1 | 6.4 | x6.1 |
| 60 | hound, boar | 137 | 51/99 · 3.9:35.4 · 53 / 4.1 | 99/89 · 36.0:28.0 · 6.5 / 5.0 | 5.8 | x9.1 |

What it says (2026-11-03, C8): the hound (grade the ranger's level) out-damages the melee ranger from level 5 on and by level 60 deals about nine times the short sword's damage a round (34.8 against 3.8; the ranger's real weapon is the bow, which this probe does not measure, so read the ranger column as a floor). It hits 71 to 99 % of the time against monsters of the ranger's level and kills one in about seven rounds at every level, and it lasts about five to six rounds against them, the same as the ranger does. The two together kill 1.6 to 6.3 rounds; the companion is the larger half of the pair from level 5. `grade_percent = 100` (the design's Q1) stands (the beast's level is the ranger's); whether a hound that out-damages its melee master nine to one at level 60 is too much is the owner's call (a tamed wolf is not measured here: it is the same growth from a higher base, and the probe's seeds draw only the hound and the boar).

## Summons

The druid's calls (#839), cast by a human druid (untrained stats, every score as rolled, seed 1) at each level a call comes or Pack leader (32) and Lord of the wild (60) change them: what comes of one cast (each type's count), for how many minutes (`lasting`: until slain), the mana over the pool, and each called type against the monsters within two levels of the druid, as in "Fights" (the beast's side bare: P(it hits) / P(it is hit) % · damage a round · rounds to kill / rounds to die).

| Call | L | comes | minutes | mana / pool | the called against the druid's level |
|---|---|---|---|---|---|
| Call of the wild (5) | 6 | 1 wolf | 5 | 5 / 42 (8 casts) | wolf 4: 69/75 · 1.8:3.8 · 10 / 2.4 |
| Call of the wild (5) | 25 | 1 wolf | 5 | 5 / 156 (31 casts) | wolf 4: 54/99 · 0.9:20.2 · 112 / 0.4 |
| Call of the forest (20) | 25 | 1 bear | 5 | 20 / 156 (8 casts) | bear 12: 93/99 · 9.8:20.6 · 11 / 2.9 |
| Call of the wild (5) | 32 | 2 wolves | 7 | 5 / 198 (40 casts) | wolf 4: 46/99 · 0.5:25.8 · 286 / 0.3 |
| Call of the forest (20) | 32 | 2 bears | 7 | 20 / 198 (10 casts) | bear 12: 90/99 · 8.4:26.0 · 18 / 2.3 |
| Call of the wild (5) | 40 | 2 wolves | 7 | 5 / 246 (49 casts) | wolf 4: 31/99 · 0.2:24.7 · 889 / 0.4 |
| Call of the forest (20) | 40 | 2 bears | 7 | 20 / 246 (12 casts) | bear 12: 81/99 · 6.8:24.5 · 26 / 2.5 |
| Call the elements (35) | 40 | 2 greater air elementals | 7 | 35 / 246 (7 casts) | greater earth elemental 24: 93/99 · 15.8:17.0 · 13 / 11; greater water elemental 26: 93/99 · 14.3:22.0 · 13 / 4.0; greater air elemental 31: 99/93 · 37.8:20.3 · 4.9 / 3.0; greater fire elemental 38: 99/99 · 27.4:22.5 · 7.7 / 4.0 |
| Call of the wild (5) | 50 | 2 wolves | 7 | 5 / 306 (61 casts) | wolf 4: 22/99 · 0.1:32.7 · >999 / 0.3 |
| Call of the forest (20) | 50 | 2 bears | 7 | 20 / 306 (15 casts) | bear 12: 72/99 · 5.9:33.9 · 41 / 1.8 |
| Call the elements (35) | 50 | 2 greater air elementals | 7 | 35 / 306 (9 casts) | greater earth elemental 24: 88/99 · 14.6:21.0 · 17 / 8.7; greater water elemental 26: 88/99 · 13.4:27.5 · 18 / 3.2; greater air elemental 31: 99/98 · 35.8:25.7 · 7.1 / 2.4; greater fire elemental 38: 99/99 · 27.5:29.1 · 10 / 3.1 |
| Call of the wild (5) | 60 | 2 wolves | lasting | 5 / 366 (73 casts) | wolf 4: 10/99 · 0.1:38.8 · >999 / 0.2 |
| Call of the forest (20) | 60 | 2 bears | lasting | 20 / 366 (18 casts) | bear 12: 54/99 · 3.5:38.6 · 56 / 1.6 |
| Call the elements (35) | 60 | 2 greater air elementals | lasting | 35 / 366 (10 casts) | greater earth elemental 24: 75/99 · 11.8:24.0 · 19 / 7.6; greater water elemental 26: 75/99 · 10.8:29.8 · 22 / 2.9; greater air elemental 31: 96/99 · 31.6:27.5 · 7.6 / 2.3; greater fire elemental 38: 96/99 · 27.1:32.0 · 8.6 / 2.8 |

## Familiar

The witch's familiar (#929, #933), a human witch (untrained stats, every score as rolled, seed 1) with each of the five kinds a run may draw in turn, at its grade (her level). Its touch is her Evil eye and needs a blow that lands, so the table is the familiar's side bare against the monsters within two levels of the witch's (as in "Fights", the monsters' gear left out): **hit** the chance a blow lands, lowest / median / highest over the window; **blows** a round (a normal turn); **hexes** laid a round (blows × median hit); **rounds to the first** hex (one over the hexes a round); **degrees** her Evil eye lays; **eye** the mana one cast of it costs over her pool, which every landed blow saves her; the familiar's hit points, mean damage of a hit, and to-hit.

| L | familiar | hit % (low / median / high) | blows | hexes a round | rounds to the first | degrees | eye (mana / pool) | hp · mean hit · to hit |
|---|---|---|---|---|---|---|---|---|
| 1 | cat | 19 / 38 / 81 | 1.30 | 0.50 | 2.0 | 1 | 4 / 13 | 4 · 1.0 · -4 |
| 1 | toad | 19 / 38 / 81 | 0.90 | 0.34 | 2.9 | 1 | 4 / 13 | 4 · 1.0 · -4 |
| 1 | raven | 19 / 38 / 81 | 1.30 | 0.50 | 2.0 | 1 | 4 / 13 | 4 · 1.0 · -4 |
| 1 | hare | 25 / 46 / 85 | 1.60 | 0.73 | 1.4 | 1 | 4 / 13 | 4 · 1.0 · -3 |
| 1 | bat | 15 / 31 / 75 | 2.00 | 0.62 | 1.6 | 1 | 4 / 13 | 4 · 1.0 · -5 |
| 5 | cat | 4 / 38 / 75 | 1.30 | 0.50 | 2.0 | 1 | 4 / 41 | 12 · 1.0 · -3 |
| 5 | toad | 4 / 38 / 75 | 0.90 | 0.34 | 2.9 | 1 | 4 / 41 | 12 · 1.0 · -3 |
| 5 | raven | 4 / 38 / 75 | 1.30 | 0.50 | 2.0 | 1 | 4 / 41 | 12 · 1.0 · -3 |
| 5 | hare | 4 / 46 / 81 | 1.60 | 0.73 | 1.4 | 1 | 4 / 41 | 12 · 1.0 · -2 |
| 5 | bat | 4 / 31 / 69 | 2.00 | 0.62 | 1.6 | 1 | 4 / 41 | 12 · 1.0 · -4 |
| 10 | cat | 15 / 62 / 90 | 1.30 | 0.80 | 1.2 | 1 | 4 / 78 | 23 · 1.0 · -1 |
| 10 | toad | 10 / 54 / 85 | 0.90 | 0.49 | 2.1 | 1 | 4 / 78 | 23 · 1.0 · -2 |
| 10 | raven | 10 / 54 / 85 | 1.30 | 0.70 | 1.4 | 1 | 4 / 78 | 23 · 1.0 · -2 |
| 10 | hare | 15 / 62 / 90 | 1.60 | 0.99 | 1.0 | 1 | 4 / 78 | 23 · 1.0 · -1 |
| 10 | bat | 10 / 54 / 85 | 2.00 | 1.08 | 0.9 | 1 | 4 / 78 | 23 · 1.0 · -2 |
| 15 | cat | 7 / 62 / 93 | 1.30 | 0.80 | 1.2 | 1 | 4 / 114 | 34 · 2.0 · +0 |
| 15 | toad | 7 / 62 / 93 | 0.90 | 0.56 | 1.8 | 1 | 4 / 114 | 34 · 2.0 · +0 |
| 15 | raven | 7 / 62 / 93 | 1.30 | 0.80 | 1.2 | 1 | 4 / 114 | 34 · 3.0 · +0 |
| 15 | hare | 10 / 69 / 96 | 1.60 | 1.10 | 0.9 | 1 | 4 / 114 | 34 · 3.0 · +1 |
| 15 | bat | 4 / 54 / 90 | 2.00 | 1.08 | 0.9 | 1 | 4 / 114 | 34 · 2.0 · -1 |
| 20 | cat | 7 / 69 / 93 | 1.30 | 0.89 | 1.1 | 1 | 4 / 153 | 45 · 4.0 · +2 |
| 20 | toad | 4 / 62 / 90 | 0.90 | 0.56 | 1.8 | 1 | 4 / 153 | 45 · 3.0 · +1 |
| 20 | raven | 4 / 62 / 90 | 1.30 | 0.80 | 1.2 | 1 | 4 / 153 | 45 · 4.0 · +1 |
| 20 | hare | 7 / 69 / 93 | 1.60 | 1.10 | 0.9 | 1 | 4 / 153 | 45 · 4.0 · +2 |
| 20 | bat | 4 / 62 / 90 | 2.00 | 1.24 | 0.8 | 1 | 4 / 153 | 45 · 4.0 · +1 |
| 30 | cat | 38 / 81 / 99 | 1.30 | 1.05 | 1.0 | 1 | 4 / 225 | 67 · 7.5 · +6 |
| 30 | toad | 31 / 75 / 99 | 0.90 | 0.68 | 1.5 | 1 | 4 / 225 | 67 · 6.5 · +5 |
| 30 | raven | 31 / 75 / 99 | 1.30 | 0.97 | 1.0 | 1 | 4 / 225 | 67 · 8.0 · +5 |
| 30 | hare | 38 / 81 / 99 | 1.60 | 1.29 | 0.8 | 1 | 4 / 225 | 67 · 7.5 · +6 |
| 30 | bat | 31 / 75 / 99 | 2.00 | 1.50 | 0.7 | 1 | 4 / 225 | 67 · 8.0 · +5 |
| 40 | cat | 46 / 85 / 99 | 1.30 | 1.11 | 0.9 | 1 | 4 / 298 | 89 · 12.0 · +9 |
| 40 | toad | 38 / 81 / 99 | 0.90 | 0.72 | 1.4 | 1 | 4 / 298 | 89 · 11.0 · +8 |
| 40 | raven | 38 / 81 / 99 | 1.30 | 1.05 | 1.0 | 1 | 4 / 298 | 89 · 13.0 · +8 |
| 40 | hare | 46 / 85 / 99 | 1.60 | 1.37 | 0.7 | 1 | 4 / 298 | 89 · 12.0 · +9 |
| 40 | bat | 38 / 81 / 99 | 2.00 | 1.61 | 0.6 | 1 | 4 / 298 | 89 · 13.0 · +8 |
| 50 | cat | 62 / 93 / 99 | 1.30 | 1.21 | 0.8 | 1 | 4 / 371 | 111 · 16.5 · +13 |
| 50 | toad | 54 / 90 / 99 | 0.90 | 0.81 | 1.2 | 1 | 4 / 371 | 111 · 15.5 · +12 |
| 50 | raven | 54 / 90 / 99 | 1.30 | 1.16 | 0.9 | 1 | 4 / 371 | 111 · 18.0 · +12 |
| 50 | hare | 62 / 93 / 99 | 1.60 | 1.49 | 0.7 | 1 | 4 / 371 | 111 · 16.5 · +13 |
| 50 | bat | 54 / 90 / 99 | 2.00 | 1.79 | 0.6 | 1 | 4 / 371 | 111 · 18.0 · +12 |
| 60 | cat | 69 / 96 / 99 | 1.30 | 1.25 | 0.8 | 1 | 4 / 444 | 133 · 18.5 · +16 |
| 60 | toad | 62 / 93 / 99 | 0.90 | 0.84 | 1.2 | 1 | 4 / 444 | 133 · 17.5 · +15 |
| 60 | raven | 62 / 93 / 99 | 1.30 | 1.21 | 0.8 | 1 | 4 / 444 | 133 · 20.0 · +15 |
| 60 | hare | 69 / 96 / 99 | 1.60 | 1.53 | 0.7 | 1 | 4 / 444 | 133 · 18.5 · +16 |
| 60 | bat | 62 / 93 / 99 | 2.00 | 1.86 | 0.5 | 1 | 4 / 444 | 133 · 20.0 · +15 |

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
  pace spell ("Caster style pace": of evocation's damage spells, the most damage a cast among those
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
| warrior | St | 14.8/16.7 | 17.0 | 20.2 | 22.8 | 24.5 | 27.3 | +12.5 | 10.7 |
|  | Dx | 10.7/16.2 | 11.3 | 15.5 | 18.0 | 19.5 | 21.3 | +10.7 | 5.2 |
|  | To | 11.7/16.0 | 13.2 | 17.5 | 19.5 | 21.0 | 23.3 | +11.7 | 7.3 |
|  | In | 10.0/17.2 | 11.7 | 13.3 | 14.8 | 16.2 | 17.5 | +7.5 | 0.3 |
|  | Wi | 11.7/17.7 | 11.8 | 13.2 | 14.2 | 15.8 | 18.0 | +6.3 | 0.3 |
|  | Pr | 10.0/16.2 | 10.0 | 10.8 | 11.5 | 12.2 | 14.2 | +4.2 | 0.0 |
| | all six | | | | | | | **+52.8** | |
| thief | St | 11.0/16.8 | 14.5 | 19.8 | 22.0 | 23.7 | 26.3 | +15.3 | 9.5 |
|  | Dx | 12.5/16.5 | 15.3 | 18.8 | 20.3 | 21.7 | 23.0 | +10.5 | 6.5 |
|  | To | 10.5/16.8 | 13.8 | 18.5 | 20.0 | 21.2 | 23.7 | +13.2 | 6.8 |
|  | In | 11.3/17.2 | 12.7 | 14.2 | 15.7 | 16.8 | 18.2 | +6.8 | 1.0 |
|  | Wi | 10.3/17.3 | 10.7 | 11.8 | 13.5 | 15.0 | 17.5 | +7.2 | 0.3 |
|  | Pr | 9.8/16.3 | 9.8 | 10.5 | 11.3 | 12.0 | 14.0 | +4.2 | 0.0 |
| | all six | | | | | | | **+57.2** | |
| wizard | St | 10.0/16.7 | 10.3 | 10.3 | 10.8 | 11.2 | 12.2 | +2.2 | 0.0 |
|  | Dx | 10.0/16.3 | 11.0 | 12.7 | 14.7 | 16.2 | 18.3 | +8.3 | 2.0 |
|  | To | 9.5/17.3 | 12.0 | 17.2 | 19.5 | 20.3 | 22.5 | +13.0 | 5.2 |
|  | In | 14.5/16.8 | 16.3 | 18.3 | 20.2 | 21.0 | 22.7 | +8.2 | 5.8 |
|  | Wi | 12.5/17.7 | 15.8 | 19.5 | 21.5 | 22.8 | 25.2 | +12.7 | 7.5 |
|  | Pr | 9.5/16.0 | 9.5 | 10.5 | 11.0 | 11.8 | 14.0 | +4.5 | 0.0 |
| | all six | | | | | | | **+48.8** | |
| priest | St | 10.8/16.3 | 14.8 | 19.0 | 21.7 | 23.5 | 25.7 | +14.8 | 9.3 |
|  | Dx | 8.7/16.3 | 9.7 | 12.7 | 14.8 | 16.8 | 18.5 | +9.8 | 2.2 |
|  | To | 10.7/16.7 | 14.2 | 18.7 | 21.3 | 23.0 | 25.3 | +14.7 | 8.7 |
|  | In | 11.3/16.7 | 12.3 | 14.2 | 15.2 | 16.3 | 17.5 | +6.2 | 0.8 |
|  | Wi | 15.3/18.0 | 16.5 | 18.8 | 20.0 | 21.0 | 22.2 | +6.8 | 4.2 |
|  | Pr | 10.8/16.3 | 10.8 | 11.3 | 12.0 | 12.8 | 14.8 | +4.0 | 0.0 |
| | all six | | | | | | | **+56.3** | |

Marks a day the hooks gave in each stat over the level before (St/Dx/To/In/Wi/Pr, the seeds' mean), and the game day reached.

| L | Day | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| 5 | 4 | 11/4/9/14/3/3 | 22/18/19/15/4/3 | 1/11/11/28/34/3 | 13/5/23/15/10/2 |
| 10 | 14 | 68/17/27/5/3/1 | 66/52/38/6/3/1 | 1/4/21/6/51/1 | 52/9/58/5/17/1 |
| 15 | 24 | 87/25/51/7/3/1 | 124/76/44/6/3/2 | 1/5/35/34/78/2 | 85/11/81/6/23/2 |
| 20 | 39 | 153/24/36/4/3/1 | 105/70/41/4/3/1 | 0/6/27/32/71/1 | 77/14/61/4/17/1 |
| 30 | 69 | 233/43/58/4/3/1 | 162/87/58/4/3/2 | 0/7/36/43/101/1 | 167/10/121/3/22/1 |
| 40 | 109 | 236/32/97/2/3/2 | 159/57/70/2/3/1 | 1/6/30/39/90/1 | 214/7/157/2/19/1 |
| 50 | 149 | 322/41/111/2/3/1 | 237/63/81/2/3/1 | 0/8/40/55/130/1 | 207/7/183/2/21/1 |
| 60 | 189 | 215/47/150/2/3/1 | 229/57/111/2/3/1 | 1/7/48/51/130/1 | 137/9/253/2/22/1 |

The game day of each stat's first rise (the seeds' mean of those that rose by level 60; how many of the six in brackets when not all).

| Class | St | Dx | To | In | Wi | Pr |
|---|---|---|---|---|---|---|
| warrior | 9 | 14 | 8 | 7 | 19 | 34 |
| thief | 7 | 6 | 7 | 8 | 17 | 33 |
| wizard | 59 | 11 | 7 | 8 | 6 | 30 |
| priest | 7 | 11 | 6 | 10 | 10 | 37 |

The knobs (design §6.2): mean gain by level 60 with other `price_per_point` and `ramp_divisor` values (St/Dx/To/In/Wi/Pr · all six), the data's in bold.

| Price | Ramp ÷ | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| **20 + 10s** | **2** | +12.5/+10.7/+11.7/+7.5/+6.3/+4.2 · +53 | +15.3/+10.5/+13.2/+6.8/+7.2/+4.2 · +57 | +2.2/+8.3/+13.0/+8.2/+12.7/+4.5 · +49 | +14.8/+9.8/+14.7/+6.2/+6.8/+4.0 · +56 |
| 20 + 10s | 3 | +13.3/+11.5/+12.7/+8.0/+6.7/+4.5 · +57 | +16.3/+11.3/+14.0/+7.0/+7.2/+4.2 · +60 | +2.3/+8.5/+13.8/+9.2/+13.8/+4.2 · +52 | +16.0/+10.2/+16.0/+6.5/+7.7/+4.3 · +61 |
| 20 + 10s | 4 | +14.0/+11.7/+13.3/+7.8/+6.7/+4.2 · +58 | +17.5/+12.3/+14.7/+7.0/+7.3/+4.2 · +63 | +2.2/+9.2/+14.5/+9.8/+14.8/+3.8 · +54 | +16.5/+10.5/+16.8/+6.5/+8.2/+3.7 · +62 |
| 20 + 10s | 6 | +15.3/+12.7/+14.5/+8.0/+6.5/+4.0 · +61 | +18.3/+13.5/+15.7/+7.5/+7.0/+4.2 · +66 | +2.2/+9.2/+14.8/+10.3/+16.0/+4.2 · +57 | +18.0/+10.7/+17.5/+7.0/+8.7/+4.0 · +66 |
| 20 + 15s | 2 | +11.8/+9.8/+11.2/+6.5/+5.0/+2.8 · +47 | +13.8/+10.0/+12.0/+5.8/+5.7/+3.5 · +51 | +1.5/+7.5/+12.3/+7.5/+12.2/+3.0 · +44 | +14.3/+8.8/+14.2/+5.7/+6.5/+2.7 · +52 |
| 20 + 15s | 4 | +13.2/+11.0/+12.3/+6.5/+5.3/+3.2 · +52 | +15.8/+11.3/+13.2/+6.3/+5.5/+3.2 · +55 | +1.5/+8.0/+13.2/+8.5/+13.8/+3.2 · +48 | +15.5/+9.5/+15.7/+5.8/+7.2/+2.7 · +56 |
| 20 + 20s | 4 | +12.0/+10.3/+11.7/+5.5/+4.3/+2.5 · +46 | +15.3/+10.5/+12.8/+5.5/+5.0/+3.0 · +52 | +1.3/+7.2/+12.8/+8.0/+12.8/+2.7 · +45 | +15.2/+8.3/+14.8/+5.0/+6.2/+2.2 · +52 |

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

(The spell paragraph is #754's, against the spell scaling design's §12.3 and §8.4, when the
schools of magic were trained skills; #1263 removed them, so its ranks and their rolls are
history: "Spells" above has the rolls now. The others are #569's, against the combat progression
design.)

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
`world/fixtures/trap_rung_tests.rs::the_lowest_rung_never_kills_a_fresh_character` checks it, so a race,
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
   top rung (Fireball 10d6+2, 37.0 a cast, since #1263 took the schools' ranks away); what grows
   a spell past that is higher-level content (`spells/*.toml`), talents, and items.
5. **The priest has no damage spell of its own.** Its schools (restoration, abjuration,
   divination) have no damage spell. Smite and its class actives are its offence, and the probe
   does not model them. Its heals are measured one a kill ("Caster style pace").
6. **Monster blows can drop with gear.** As in the report, some types deal less damage with the
   weapon they roll than with their natural attack. The geared mean of damage a blow was below the
   bare median from level 15 up (`gear.toml`, the creature files); since #1169's ranks it is
   about level with it (above at 15, 30, and 70, below at 20, 40, and 50).

## Talents coverage

Every system the design added meets the coverage bar (`assets/data/talents/systems/`, `min_talents`
1 since #773, checked when the game loads):
- `weapon_abilities` has 4 talents (Early mastery, Brace of steel, Practised technique,
  Drillmaster).
- `stamina` has 5 (Charge, which replaced Trained wind in #1466, Deep lungs, Steady breathing, Armour-breaker, Drillmaster;
  Second breath and Bellows were cut in #774).
- `stat_training` has 2 (Gourmand, Cauldron; #726; Hard-won and
  Gifted were cut in #774, Long fast became a `hunger` talent in #775, and Quick study of the
  body, Hard living, and Discipline were cut in #787, Hard training and Late bloomer in #791).
- `potions` has 2 (Thrifty drinker, Lasting draughts; Nose for draughts and Cauldron-wise were
  cut in #787).

No follow-up is needed.
