# Warlock — Miscasts, revision 2

Design for the owner's line-by-line review (revision 2, 2026-11-09). Revision 1 (2026-11-08) was
a review of the miscast system #1275 built; the owner answered its seven questions and gave a
direction and a list of new entries. This revision is the design that adopts them, written
against `main` at 1eb04a6b (ranks #1276 and the rank tables #1280–#1286 landed since revision 1).
Nothing here is implemented. Numbers come from an exact enumeration of the rules (2d12 × 2d12,
§2.4); the review's findings that still hold are summarised in §1.2, not repeated.

## Brief for the owner

**What the owner decided** (PR #1277 comments): the margin **orders** the outcome; **ranks** are
the lever; Concentration stays mana-only (**Spellcraft is the miscast skill**); half-casters at
1 in 3 to 1 in 7 is intended; **Surge goes**; a healer's backfire **heals the nearest hostile**;
the words **vary**; the table grows to **60–125 rows**; entries below about 70 **cannot kill
outright**; and six new entries: a **rank lost** (the last rank lost forgets the spell), **stat
drain**, the **gate itself as a rooted monster** that keeps summoning on a cooldown, **every
kind of fiend** (not demons alone), the fiends **friendly only to each other**, and a **portal
into an infernal dungeon** with a boss on its last floor.

**The design in five lines.**
1. The check stays 2d12 + Intelligence + Spellcraft's `cast` against 8 + spell level + armour,
   **less the spell's rank above 1**. Spell level against Spellcraft is the axis; nothing else
   moves it (§2.1).
2. The total becomes **2d12 + 5 × the miss − ward − stance**: the miss carries it, the die is the
   small variance. A near miss lands in the first dozen rows; a spell twenty levels too high
   lands past 100 (§2.2, §2.4).
3. **One table of 65 rows, two totals each**, ordered from a glimmer to oblivion (§3). Rows
   1–70 are **protected**: a harm there cannot take the caster below 1 hit point, and no row
   there summons, drains for good, backfires, forgets a spell, or opens a pit; the loader
   refuses one that would (§4).
4. New effects: **rank loss and forgetting** (§5.1), **stat damage** (mends) below 70 and the
   permanent **drain** above (§5.2), the **rebound heal** and the other backfire fallbacks
   (§5.3), **noise and zones** in Surge's place (§5.4); the **gates**: nine rooted gate
   monsters of a new faction hostile to everyone but itself, summoning fiends by tier (§6); and
   **the Pit**, Hellegeat: a shared five-floor fiend branch opened by stairs the miscast makes,
   its warden on the last floor, whose death seals it (§7).
5. **Fourteen small PRs** in five phases; the first (the rules and the protected half of the
   table) stands alone and is nearly all data (§10).

## Contents

1. Decisions taken
2. The check and the total
3. The table
4. Death protection
5. New effects: ranks, stats, backfires, chaos
6. The gates (fiend portals)
7. The Pit: Hellegeat
8. Words
9. Creatures' miscasts
10. Implementation plan
11. Docs, wiki, tests
12. Open questions for the owner

## 1. Decisions taken

### 1.1 The owner's answers

| Revision 1 asked | Answer | Where it lands |
|---|---|---|
| Flat or ordered? | ordered: higher totals, worse effects | §2.2 (2d12 + 5 × miss), §3 (the ordered rows) |
| Ranks as the lever (A1)? | yes | §2.1 (`rank_ease`) |
| Concentration wards (A2)? | no: mana only; Spellcraft is the miscast skill | nothing added to Concentration; `ward` stays Spellcraft's |
| Half-casters at 1 in 3 to 1 in 7? | intended: "better get that spellcraft up" | §2.4 (their odds, all in the protected rows), the wiki sentence (§11) |
| Surge? | "needs replacing" | gone; §5.4 takes its slots with noise and zones |
| Healers' backfires? | rebound to heal hostiles | §5.3 |
| The words? | vary them | §8 |
| (direction) | penalise casting high-level spells without Spellcraft; overhaul freely | §2, §3 |
| (direction) | 60–125 rows, small variance, no instant death below ~70 | §3 (65 rows), §4 (`safe_below`) |
| (entries) | rank loss, forgetting; stat drain; gate monsters with a cooldown; every fiend kind; fiends friendly only to each other; the infernal portal | §5.1, §5.2, §6, §7 |

### 1.2 What revision 1 found that still shapes this

- **Spellcraft grows only at level-up** (rolls on it; casting only banks marks). The owner
  accepts that: the player who wants fewer miscasts spends rolls on Spellcraft, and between
  level-ups deepens the spell's rank (§2.1). The balance probe should spend the wizard's rolls on
  Spellcraft so its table shows a career that plays the system (§11).
- **2d12 against a flat number is a cliff**: 1 in 144 to 99 % over 14 points of
  `difficulty − bonus`. That is the game's check and stays; the rank ease lets the player move
  along it spell by spell.
- **Positions lied** (Senseless above Backlash): the new table is built from what each effect
  does, and Senseless is gone (Staggered covers it).
- **Defensive stance is a free −5**: kept, an unbalanced choice the owner allows; the wiki says
  so.
- **Scribe sidesteps the check** at three times the mana: kept; the wiki says so.

### 1.3 What stays as built

The spell always resolving first and in full; the check after the cast's own draws; a natural 2
failing and a natural 24 making; one ordered table in data, one entry a file, tiling checked at
load; the mishaps' effect machinery shared; `seen` lines with `{who}`; creatures checking with
their own Spellcraft; `Cause::Miscast` and its graveyard line; Spellcraft's `cast` and `ward`
curves; the stance's `miscast`; armour's +1 per 10 % of spell cost.

## 2. The check and the total

### 2.1 The check

```
bonus      = Intelligence modifier + talents' Intelligence checks + Spellcraft `cast`
difficulty = base 8 + per_level 1 × spell level + armour % ÷ 10 − rank_ease 1 × (rank − 1)
2d12 + bonus ≥ difficulty makes; a natural 2 fails; a natural 24 makes
```

- **New: `rank_ease = 1`** in `data/miscast.toml` (`MiscastRules::rank_ease`), one term in
  `MiscastRules::difficulty`, which gains a `rank` argument (`Actor::cast_difficulty` reads
  `Actor::spell_rank`; a scroll or wand never reaches the check, and a creature's spells are rank
  1, so nothing else changes). Rank 10 casts as a spell nine levels lower.
- Why ranks and nothing else: the owner's one sentence, "penalise casting high-level spells
  without Spellcraft", is `spell level − Spellcraft` as the axis. The rank is the player's own
  move along it inside a run (rereading the grimoire: `rank × (100 + 20 × level)` turns and a
  Decipher script check, no farming limit, #1264), and it makes the spells a caster has invested
  in both stronger and safer, which is what mastering a spell should feel like. Nothing in the
  `spellcraft` talent tree touches the check; no item does (no Spellcraft affix exists); the
  review's A3 affix stays shelved.
- Everything else as built: `base 8`, `per_level 1`, `armour_percent 10`, Intelligence for every
  spell.

### 2.2 The total

```
total = 2d12 + per_margin 5 × miss − Spellcraft `ward` − stance `miscast`, at least 1
miss  = difficulty − natural roll − bonus, at least 0 (a natural 2 with a bonus to spare misses by 0)
```

- **The die changes** from an exploding 1d66 to **2d12** (`die = "2d12"`, a dice expression:
  `MiscastRules::die` becomes a `Roll`; the explosion goes). The review showed a uniform 66 drowns
  the margin: every row from Sparks to Displaced came up 7.6–9.1 % for anyone who could cast
  the spell at all. 2d12 has a mean of 13 and a spread of 2–24, so **neighbouring rows differ
  little and the miss decides the band**, which is the owner's "ordered" and "small variance".
- **`per_margin` 3 → 5**: each point missed moves the total two rows and a half. A miss of 10 is
  the edge of the protected half (§4); a miss of 20 the catastrophes.
- `ward` (Spellcraft: 10 at 100, 20 at 200) and the stance (Defensive 5, Aggressive −5, Frenzy
  −10) are unchanged. With the total ordered they now mean what they say: a master's natural 2
  is `2d12 − 20`, the first two rows; Frenzy's +10 is two bands up.

### 2.3 Why not retune the `cast` curve

Revision 1's direction B (a steeper curve) would move every caster along the cliff at once and
leave the neglecter where they are. The owner's answer 4 says the neglecter *should* be where
they are. The curve stays; if play shows dedicated wizards reach the floor too early or too late,
it is one line in `skills.toml` later.

### 2.4 Numbers

The chance of a miscast (% of casts) and, **given one**, where on the table it lands, for
representative casters. Bands are §3's: *harmless* rows 1–16 (nothing and boons), *mild* 17–40,
*annoying* 41–70 (the top of the protected half), *severe* 71–90, *deadly* 91–110, *catastrophe*
111+. Intelligence and Spellcraft are the balance probe's (`docs/balance.md` "Miscasts",
"Players").

| Caster | Spell, rank | Bonus vs difficulty | Miscast | harmless | mild | annoying | severe | deadly | catastrophe | Worst total |
|---|---|---|---|---|---|---|---|---|---|---|
| Wizard L1 (Spellcraft 20, Int 14) | Spark 1, r1 | +3 vs 9 | 6.9 % | 18 | 81 | 1 | 0 | 0 | 0 | 44 |
| the same | Fireball 11 from a found grimoire, r1 | +3 vs 19 | 68.8 % | 5 | 49 | 39 | 6.5 | 0.1 | 0 | 94 |
| the same | Fireball, r3 | +3 vs 17 | 54.2 % | 7 | 56 | 35 | 1.9 | 0 | 0 | 84 |
| the same | Fireball, r5 | +3 vs 15 | 38.2 % | 9 | 63 | 28 | 0.1 | 0 | 0 | 74 |
| the same | Fireball, r10 | +3 vs 10 | 10.4 % | 16 | 81 | 3 | 0 | 0 | 0 | 49 |
| Wizard L20, Spellcraft 100 (+14), Int 18, ward 10 | Chain lightning 20, r1 | +17 vs 28 | 31.2 % | 40 | 52 | 8 | 0 | 0 | 0 | 59 |
| the same | Chain lightning, r5 | +17 vs 24 | 10.4 % | 61 | 39 | 0 | 0 | 0 | 0 | 39 |
| Wizard L20 who never rolled Spellcraft (20: +2), Int 18, ward 2 | Chain lightning 20, r1 | +5 vs 28 | 97.9 % | 2 | 18 | 43 | 23 | 12 | 1.8 | 127 |
| the same | Chain lightning, r5 | +5 vs 24 | 85.4 % | 4 | 36 | 44 | 13 | 2 | 0 | 107 |
| Priest L10, Spellcraft 22 (+2), Int 13, ward 2 | a level-10 spell, r1 | +3 vs 18 | 61.8 % | 10 | 53 | 34 | 3 | 0 | 0 | 87 |
| the same | r4 | +3 vs 15 | 38.2 % | 13 | 62 | 24 | 0 | 0 | 0 | 72 |
| Ranger L6, Spellcraft 9 (+0), Int 12 | Barkskin 3, r1 | +1 vs 11 | 25.0 % | 10 | 71 | 18 | 0 | 0 | 0 | 64 |
| Bard L1, Spellcraft 9, Int 12 | Mend 1, r1 | +1 vs 9 | 14.6 % | 13 | 79 | 7 | 0 | 0 | 0 | 54 |
| Master, Spellcraft 200 (+34), Int 22, ward 20, Defensive | Sunfire 30 | +39 vs 38 | 0.7 % | 100 | 0 | 0 | 0 | 0 | 0 | 1 |
| Wizard L1 forcing Sunfire 30 from a grimoire | r1 | +3 vs 38 | 99.3 % | 0 | 0 | 0.6 | 10 | 21 | 69 | 189 |
| Wizard L1, Fireball, **Frenzy** (+10) | r1 | +3 vs 19 | 68.8 % | 0 | 33 | 51 | 14 | 1.5 | 0 | 104 |
| Kobold shaman (Spellcraft 25, +3, ward 2) | Spark | +3 vs 9 | 6.9 % | 28 | 72 | 0 | 0 | 0 | 0 | 42 |
| Troll shaman (78, +12, ward 7) | Lightning bolt 11 | +12 vs 19 | 10.4 % | 47 | 53 | 0 | 0 | 0 | 0 | 42 |
| Lich (105, +23, ward 10) | Chain lightning 20 | +23 vs 28 | 4.2 % | 79 | 21 | 0 | 0 | 0 | 0 | 29 |

Reading:
- **A caster within reach of a spell never leaves the protected half.** The new wizard on Spark,
  the dedicated level-20 wizard on Chain lightning, every half-caster on their borrowed spells,
  every shipped creature caster: the worst total is below 71. Their miscasts are a nuisance
  with a boon one time in five to one in two.
- **Spell level against Spellcraft is the axis, and it bites.** The level-20 wizard who put no
  roll in Spellcraft and casts a level-20 spell miscasts 98 % of casts, a quarter of them severe
  (a gate, a permanent drain, the spell forgotten) and one in eight deadly. The same wizard at
  rank 5 is at 85 % with 2 % deadly. The owner's "they better get that Spellcraft up" is exactly
  what the table says.
- **Ranks are a real lever.** The new wizard with a Fireball grimoire goes from 69 % with 6.5 %
  severe at rank 1 to 10 % and nothing worse than annoying at rank 10, for nine rereadings.
- **The hopeless case is hopeless.** A level-1 wizard forcing Sunfire from a grimoire has the
  Pit open under them one cast in four. That is intended: they were told in the read list that
  the spell was far above them.
- **Half-casters** at 1 in 4 to 1 in 7 with Spellcraft 9: the whole miscast lands in rows 1–64.
  The wiki tells them a roll or two on Spellcraft moves it.

## 3. The table

65 rows, two totals each from 1 to 128, the last open (`129+`), one `[[miscast]]` file each in
`data/miscasts/`, tiling checked at load as now. **Miss needed** is the smallest miss whose best
die (24) reaches the row's first total, and in brackets the miss at which the row is the mean
outcome: how far above one's Spellcraft a spell must be to see the row at all, and to see it
often. Effect kinds are the existing `MishapEffect` kinds unless §5 or §6 says they are new
(`noise`, `zone`, `rank_loss`, `forget`, `stat_damage`, `portal`; `drain` gains `times`;
`backfire` gains fallbacks; `surge` goes). "Capped" is §4's rule. "The spell's type" is the
damage type of the spell cast (force for one with none, as `harm` does now).

| Range | Id | Effect | Band | Miss needed |
|---|---|---|---|---|
| 1–2 | `glimmer` | nothing | — | 0 (0) |
| 3–4 | `motes` | nothing | — | 0 (0) |
| 5–6 | `refund` | refund: the cast's mana back | boon | 0 (0) |
| 7–8 | `hum` | noise 30: a low hum those near hear | — | 0 (0) |
| 9–10 | `echo` | twice: the spell again at its target, free | boon | 0 (0) |
| 11–12 | `sparks` | harm 1d2 fire (capped) | mild | 0 (0) |
| 13–14 | `blessing` | boon blessed 1: on the caster or someone near | boon | 0 (0) |
| 15–16 | `quickened` | status hasted 1 on the caster | boon | 0 (1) |
| 17–18 | `fizz` | mana 10 % of what is left | mild | 0 (1) |
| 19–20 | `singe` | harm 1d3 of the spell's type (capped) | mild | 0 (1) |
| 21–22 | `wobble` | status staggered 1 | mild | 0 (2) |
| 23–24 | `crack` | noise 60: a sharp report | mild | 0 (2) |
| 25–26 | `leak` | mana 25 % | mild | 1 (3) |
| 27–28 | `lash` | harm 1d4 of the spell's type (capped) | mild | 1 (3) |
| 29–30 | `flinch` | status frightened 1 | mild | 1 (3) |
| 31–32 | `shiver` | status weakened 1 | mild | 2 (4) |
| 33–34 | `fog` | zone smoke, radius 1, 5 turns, at the caster | chaos | 2 (4) |
| 35–36 | `slip` | swap: places with someone near (a creature's never with the player) | chaos | 3 (5) |
| 37–38 | `drowse` | status drowsy 1 | mild | 3 (5) |
| 39–40 | `thunder` | noise 100: a thunderclap heard across the level | chaos | 3 (5) |
| 41–42 | `scorch` | harm 2d4 of the spell's type (capped) | mild | 4 (6) |
| 43–44 | `gloom` | zone dark, radius 2, 10 turns | chaos | 4 (6) |
| 45–46 | `seep` | mana 50 % | bad | 5 (7) |
| 47–48 | `slick` | zone slick, radius 1, 10 turns | chaos | 5 (7) |
| 49–50 | `slur` | status silenced 1: no casting for a few heartbeats | bad | 5 (7) |
| 51–52 | `stumble` | rank_loss 1: the spell a rank lower (never below 1) | bad | 6 (8) |
| 53–54 | `burn` | harm 2d6 of the spell's type (capped) | bad | 6 (8) |
| 55–56 | `ache` | stat_damage 1 to a random stat (mends a point a day) | bad | 7 (9) |
| 57–58 | `displaced` | teleport: to a free cell of the level | chaos | 7 (9) |
| 59–60 | `lapse` | lose_mark: a stat training mark gone | mild | 7 (9) |
| 61–62 | `chill` | status sluggish 2 | bad | 8 (10) |
| 63–64 | `falter` | rank_loss 2 | bad | 8 (10) |
| 65–66 | `dry` | mana 100 %: every point left | bad | 9 (11) |
| 67–68 | `blast` | harm 3d6 force (capped) | bad | 9 (11) |
| 69–70 | `racked` | stat_damage 2 to a random stat | bad | 9 (11) |
| 71–72 | `backlash` | backfire at the base roll (§5.3: a heal on the nearest hostile, a buff on them, a summon hostile, else (level ÷ 3 + 1)d6 force) | severe | 10 (12) |
| 73–74 | `withering` | drain 1 of a random stat for good unless Willpower vs 16 | severe | 10 (12) |
| 75–76 | `rift` | summon the `rift` gate (lesser demons) | severe | 11 (13) |
| 77–78 | `hellmouth` | summon the `hellmouth` gate (lesser devils) | severe | 11 (13) |
| 79–80 | `gloom_gate` | summon the `gloom_gate` (lesser yugoloths and fiends) | severe | 11 (13) |
| 81–82 | `forgetting` | forget 1: the spell is forgotten (a class spell: back to rank 1) | severe | 12 (14) |
| 83–84 | `rend` | harm 4d6 force | severe | 12 (14) |
| 85–86 | `twofold_withering` | drain 1 of each of two random stats, Willpower vs 16 each | severe | 13 (15) |
| 87–88 | `abyssal_gate` | summon the `abyssal_gate` (greater demons) | severe | 13 (15) |
| 89–90 | `rebound` | backfire at the cast's full roll (the same fallbacks) | deadly | 13 (15) |
| 91–92 | `iron_gate` | summon the `iron_gate` (greater devils) | deadly | 14 (16) |
| 93–94 | `black_gate` | summon the `black_gate` (greater yugoloths and fiends) | deadly | 14 (16) |
| 95–96 | `unravelling` | harm 6d10 force | deadly | 15 (17) |
| 97–98 | `deep_withering` | drain 2 of a random stat, Willpower vs 18 | deadly | 15 (17) |
| 99–100 | `amnesia` | forget 2: the spell and one more learned spell | deadly | 15 (17) |
| 101–102 | `dire_gate` | summon the `dire_gate` (fiends of level 36 to 44) | deadly | 16 (18) |
| 103–104 | `harrowing` | harm 8d10 force | deadly | 16 (18) |
| 105–106 | `pit` | portal: stairs down into Hellegeat, and a `rift` at their mouth | deadly | 17 (19) |
| 107–108 | `threefold_withering` | drain 1 of each of three random stats, Willpower vs 18 | deadly | 17 (19) |
| 109–110 | `ruin` | harm 10d10 force | deadly | 17 (19) |
| 111–112 | `arch_gate` | summon the `arch_gate` (fiends of level 46 to 58) | catastrophe | 18 (20) |
| 113–114 | `pit_yawning` | portal with an `abyssal_gate` at its mouth | catastrophe | 18 (20) |
| 115–116 | `blank` | forget all: every learned spell | catastrophe | 19 (21) |
| 117–118 | `unmaking` | harm 12d10 force | catastrophe | 19 (21) |
| 119–120 | `ruinous_withering` | drain 2 of each of two random stats, Willpower vs 20 | catastrophe | 19 (21) |
| 121–122 | `pit_roaring` | portal with a `dire_gate` at its mouth | catastrophe | 20 (22) |
| 123–124 | `lords_gate` | summon the `lords_gate` (a balor or a pit fiend) | catastrophe | 20 (22) |
| 125–126 | `annihilation` | harm 15d10 force | catastrophe | 21 (23) |
| 127–128 | `pit_and_lords` | portal with a `lords_gate` at its mouth | catastrophe | 21 (23) |
| 129+ | `oblivion` | harm 20d10 force | catastrophe | 21 (23) |

Notes on the order:
- **Rows 1–24 are the near miss** (miss 0: the natural 2 of a caster with the bonus, or a miss
  by nothing). Two of every five near misses are nothing or a boon; the rest cost a few hit
  points, a tenth of the mana, or an action. That keeps the review's "a miscast is good news
  about one time in four" where it belongs: for the caster who nearly made it.
- **Boons stop at 16.** Past a miss of 2 nothing good happens; past 10 nothing is cheap.
- **Harm climbs in small steps** (1d2, 1d3, 1d4, 2d4, 2d6, 3d6) and is capped the whole way to
  70; from 71 it is uncapped and climbs 4d6, 6d10, 8d10, 10d10, 12d10, 15d10, 20d10 (means 14,
  33, 44, 55, 66, 83, 110 against wizard hit points 13 / 26 / 58 / 104 at levels 1 / 10 / 20 /
  30). The deadly rows kill the caster who should not have cast the spell, which is the point.
- **The gate rows are interleaved with the harms and the drains** so that three near-equal totals
  give three different disasters: a flavour of the "small variance" the owner asked for, and so
  no single row has a quarter of the deadly band.
- **The same-named families** (`withering` ×5, the gates ×9, the pit ×4, `forget` ×3) are the
  "rows with small variance": each step up is the same thing, a little worse.
- Senseless and Surge are gone; Dread is `flinch` at a single degree; Scorch is below Burn.

## 4. Death protection

**Rule**: `safe_below = 71` in `data/miscast.toml`. An entry whose `to` is below it is
**protected**: it can never kill the caster, nor cost them anything they cannot get back.

How it is guaranteed:
- **Harm is capped**: a protected `harm` takes at most `hp − 1` (`amount.min(hp − 1)`; nothing
  at 1 hit point). One line in the `Harm` arm of `World::miscast`, reading the entry. The
  `Cause::Miscast` death line can then only come from rows 71 and up.
- **Stat damage, not drain**: below 71 a stat loss is `stat_damage` (§5.2), which `StatDamage::take`
  never takes below a score of 1 and which mends a point a day; hit points and mana are cut to
  the new maxima **never below 1** (the drain's own rule, `drain.rs`).
- **Statuses that tick are refused**: a protected `status` must be one whose definition has no
  `damage` or `mana_damage` tick (Staggered, Frightened, Weakened, Drowsy, Sluggish, Silenced,
  Hasted are such; Burning, Bleeding, Poisoned are not). The loader checks it against
  `data/status.toml`.
- **Kinds refused below 71**: `backfire`, `drain`, `summon`, `forget`, `portal`, and a `zone` of
  kind `fire`. `rank_loss` is allowed but **never forgets** (it stops at rank 1). The loader
  (`content/miscast.rs`) refuses a protected entry with any of them, so a campaign's own table
  cannot break the promise either.
- The rule is about the miscast itself: a caster left at 1 hit point in a fight, silenced, or
  swapped into a corridor of foes can still die of what follows. That is the game.

What the number means in play: a protected total needs a miss of at most 9 on the best die; a
miss of 10 is a spell ten levels above what one's Spellcraft and rank carry. Nobody dies of
their own spell unless they cast one that far beyond them.

## 5. New effects: ranks, stats, backfires, chaos

### 5.1 Rank loss and forgetting

- **`rank_loss { ranks = N }`** (new kind): the spell cast loses N ranks, never below 1:
  `Actor::set_spell_rank(spell, max(1, rank − N))`. A class-granted spell that was never deepened
  is already at 1: nothing more. Said in words ("The words slip from you: you know Fireball less
  well.") and shown on the Magic page's rank column. Getting it back is the deepening reading
  (#1264): `rank × (100 + 20 × level)` turns and a check, which is what makes the loss hurt
  without being final (and no farming limit, as the house rule says).
- **`forget { spells = N | all = true }`** (new kind): the spell cast is **forgotten**: removed
  from `Actor::spells` (a new `Actor::forget_spell`, the second writer beside
  `set_spell_rank`), so `knows_spell` is false, the Magic page and the abilities menu lose it,
  and casting it is refused. `spells = 2` forgets one more **learned** spell drawn at random
  (`rng.below` over the learned ones in menu order); `all` forgets every learned spell. A spell a
  **class grants** (`TalentMods::has_spell`) **cannot be forgotten**: it falls to rank 1 instead
  (what one's class taught is in the bones; also the only thing the data model allows without a
  "forgotten" exclusion set). Learned again from its grimoire as any spell is (`learn_spell`,
  a study at rank 1), **paying no experience the second time**: a new `Actor::forgotten:
  BTreeSet<SpellId>` (saved, left out while empty) records what was once known and
  `learn_spell` skips `spell_xp` for a member; without it, forget-and-relearn would mint
  experience. Open question 1 asks the owner to confirm the class-spell rule.
- **A creature's** rank loss and forgetting do nothing (its spells are the type's list, at rank
  1); the `seen` line still speaks.
- Where they sit: `stumble` 51 (1 rank), `falter` 63 (2), `forgetting` 81, `amnesia` 99,
  `blank` 115.

### 5.2 Stat damage and drain

Two existing mechanics, one for each half of the table:
- **Below 71: `stat_damage { stat = "random", points = N }`** (new kind) uses
  [`StatDamage::take`](../../crates/warlock-core/src/stats.rs) on `Actor::damage`, the diseases'
  stat damage (spec §8 "Stats"): the score is read lower at once, never below 1; hit points and
  mana are cut to the new maxima, never below 1; it **mends a point a day** once no disease shows
  (`disease.rs`'s mending loop already runs for any actor with damage; the PR confirms it runs
  with no disease at all). Said in words ("You feel weaker": the diseases' lines by stat), never
  the points. `ache` 55 (1 point), `racked` 69 (2).
- **From 71: the existing `drain`** (`World::drain`: a Willpower check against `difficulty`, else
  `points` off the score **for good**, `Actor::drained` counting them for a temple's mending;
  never a kill). New field **`times = N`** (default 1): the draw and the check made N times, each
  for a random stat (so two stats may be the same). `withering` 73 (1, vs 16),
  `twofold_withering` 85 (1 × 2, vs 16), `deep_withering` 97 (2, vs 18), `threefold_withering`
  107 (1 × 3, vs 18), `ruinous_withering` 119 (2 × 2, vs 20).
- The old `withering` (Willpower 16, 1 point) keeps its id and numbers.

### 5.3 Backfire fallbacks: the rebound heal

`backfire` (base roll) and `backfire { full = true }` (the cast's roll) today do nothing for a
heal, a buff, a summon, a cleanse, or a utility. New fallbacks, in `miscast_backfire`:

| The spell's kind | Backfire |
|---|---|
| damage | as now: a burst (its radius, else 1) on the caster |
| hex | as now: on the caster at the cast's degrees |
| **heal** | **the heal lands on the nearest hostile the caster sees** (`hex_target`'s pick by distance, then actor id), for the roll (base or full); the player is told whom it mended. No hostile in sight: the fallback harm |
| **buff** | the buff on that nearest hostile at the cast's degrees (Haste on the orc); none in sight: the fallback harm |
| **summon** | the creatures come as the summon would bring them, but **hostile**: of their type's own faction, no `ally`, no `summoner`. (Not the `loosed` faction: a backfired wolf is a wolf) |
| cleanse, utility, and any heal or buff with no hostile in sight | **fallback harm**: `(level ÷ 3 + 1)d6` force at base, `(level ÷ 2 + 1)d6` at full (Cure wounds 3: 2d6 / 2d6; Haste 11: 4d6 / 6d6; Benison 12: 5d6 / 7d6) |

The owner's words were "rebound to heal hostiles"; a heal that mends the thing you are fighting
is the cleanest punishment a priest can read. `backlash` 71 and `rebound` 89 are both above the
protected line, so the fallback harm may kill, as the damage spell's burst always could.

### 5.4 In Surge's place: noise and zones

Surge (a random damage spell, Sunfire among them, bursting at someone within 6) is removed:
the `surge` kind and `surge.toml` go. Its "chaos" slots are taken by two small new kinds that
reuse engine pieces and cannot kill:
- **`noise { loudness = N }`**: `World::sound_at(loudness, caster's cell)` as the thunderclap
  sound (`t`); 30 is a hum those beside hear, 60 a report heard down the corridor, 100 the
  thunderclap heard across the level that wakes sleepers and brings what hunts by ear
  (`docs/dev/hearing.md`). `hum` 7, `crack` 23, `thunder` 39. For a creature's miscast the same:
  its own kin come to see.
- **`zone { kind, radius, turns }`**: the dev `zone:` machinery (`docs/dev/spells.md` "Zones"),
  by the caster, centred on them: `fog` 33 (smoke 1, 5 turns: sight cut), `gloom` 43 (dark 2,
  10 turns), `slick` 47 (slick 1, 10 turns: footing). `fire` is refused below 71 by the loader
  (§4); no row uses it.

Swap (`slip` 35), Displaced (`displaced` 57), Echo, Refund, Boon, Quickened stay as the review
proposed, with the one change that **a creature's swap never picks the player** (a kobold's
one-in-144 should not drag the player into its friends).

## 6. The gates (fiend portals)

The owner: the rift or gate is itself a tough, rooted monster that keeps summoning on a cooldown;
every kind of fiend, not demons alone; what comes through is friendly only to each other and
attacks everything else.

### 6.1 The `loosed` faction

A new faction in `data/factions.toml`, inserted after `yugoloth`:

```toml
# What a miscast lets through (spec §8 "Spells": "Miscasts"): the gates and every fiend they
# call. Loyal to one another and to nothing else: they fall on the player, the kobolds, the
# townsfolk, and the fiends of the ordered hells alike.
[[faction]]
id = "loosed"
name = "the loosed"
hostile_to_others = true
```

- **`hostile_to_others`** (new flag in `content/faction.rs`, beside `hostile_to_all`): fills the
  faction's row and column of the hostility matrix **but not its diagonal**, so the loosed are
  neutral to each other and hostile to every other faction, the player's and the demons'
  included. `hostile_to_all` (the berserk) stays as it is: hostile to each other too.
  `Factions::has_enemies` and `disturbs_rest` treat it as `hostile_to_all` is treated (its own
  quarrels only: being on the loosed's list makes no one a fighter).
- The four test fixtures that write their own `data/factions.toml` list it (every shipped
  faction id must be); `fixture_factions.rs` strips the flag as it strips the berserk's.
- Why not reuse `berserk`: the owner wants the fiends friendly to each other. Why not the
  fiends' own factions: the devils and demons leave the kobolds be, and a vrock that walks past
  the kobolds to reach the player is not what "attack anything else" means.

### 6.2 The gate monsters

Nine shared creature types in `assets/data/creatures/`, faction `loosed`, tags `["summoner",
"fiend", "gate"]`, **rooted**, with a `[creature.summon]` each. A gate has no attack of its own,
no corpse, darkvision, `sight` 8, high Protection and resistances (fire 100, poison 100, cold
50), a `cry` (a roar or a scream every minute or two) so it is heard, `rarity = "unique"`-like
exclusion from every population and stray list (`stray = false`; a gate is never drawn by a
level, only made by a miscast or a Pit's mouth), and `xp` by level like any creature. Killing
the gate is the fix: what it called remains, but no more come.

| Id | Name | Level | hp | Prot. | Calls (its pool, `tag`-free ids) | Count, cooldown, `most` | Row |
|---|---|---|---|---|---|---|---|
| `rift` | a rift | 10 | 30 | 4 | dretch 8, quasit 12 | 1d2 every 60 s, most 4 | 75 |
| `hellmouth` | a hellmouth | 10 | 30 | 4 | lemure 5, imp 9 | 1d2 every 60 s, most 4 | 77 |
| `gloom_gate` | a gate of gloom | 16 | 40 | 5 | hell hound 13, nightmare 21, mezzoloth 22 | 1 every 60 s, most 4 | 79 |
| `abyssal_gate` | an abyssal gate | 30 | 90 | 8 | balgura 24, chasme 28, hezrou 30, vrock 34 | 1 every 90 s, most 4 | 87 |
| `iron_gate` | an iron gate | 30 | 90 | 8 | bearded devil 20, barbed devil 26, bone devil 32, erinyes 38 | 1 every 90 s, most 4 | 91 |
| `black_gate` | a black gate | 32 | 90 | 8 | incubus 27, succubus 27, canoloth 29, night hag 33, nycaloth 36 | 1 every 90 s, most 4 | 93 |
| `dire_gate` | a dire gate | 42 | 130 | 10 | nycaloth 36, erinyes 38, cambion 39, marilith 40, chain devil 42, glabrezu 44 | 1 every 120 s, most 3 | 101 |
| `arch_gate` | an arch-gate | 52 | 160 | 12 | arcanaloth 46, horned devil 48, nalfeshnee 50, rakshasa 52, ice devil 55, ultroloth 58 | 1 every 180 s, most 3 | 111 |
| `lords_gate` | the lords' gate | 64 | 200 | 14 | balor 62, pit fiend 66 | 1 every 300 s, most 2 | 123 |

- **Every fiend named exists** (`assets/data/creatures/`: the demons #449, devils #448,
  yugoloths #446, the other fiends #450). Nothing new is needed below the Pit's warden (§7).
  The pools are written as id lists, which `SummonSpec` allows for shared types; the `tag` form
  (`tag = "fiend"` within `levels`) would also work and is the alternative if the owner prefers
  one line a gate.
- **The summons are `loosed`**: `creature_summons` already passes the summoner's faction as the
  spawn's `faction` when it differs from the type's, so a `rift` calling a `dretch` makes a
  `loosed` dretch with `summoner = the rift`. Banishing smite (#913) unmakes them as it does any
  summoned thing.
- **Cooldown** is the existing `summoning` status (6 s a degree, max 100: up to 600 s), **cost**
  none (a breeder's rule: `mana` left out), **cap** `most` live summons. The AI's condition "a
  hostile it sees within range" is met by anything in sight, since everything is hostile to it;
  so a gate in an empty room waits, and a gate beside a fight pours. The first summon comes on
  the gate's first turn (the status is not given at birth).
- **Rooted** (new creature flag `rooted = true`, `CreatureDef::rooted`): it never moves, never
  gives way (as `budge = "never"`), is never pushed or teleported, never follows on stairs, and
  its AI turn is: summon if it may, else wait. One arm in `creature_turn` before the AI (as the
  target dummy's is), one check in the places that move an actor (`teleport_to`, the swap,
  displacement, `stair_followers`), and the `truth` dump marks it `rooted`. The gate's rows in
  the wiki's creature tables come for free.
- **Where it stands**: the first free cell around the caster (`miscast_summon` as now). A
  caster's own miscast gate is hostile to the caster too: a kobold shaman's rift eats the
  kobolds first.
- **Death**: "The rift shudders and closes." (its death line); no corpse, no drop; experience
  by level. What it called stays, now without a summoner (the record stands, as summoning.md
  says).
- **Not timed out**: a gate lives until killed or the level is left and comes back to; it does
  not close on its own. Open question 4 offers the alternative.

### 6.3 Who reaches a gate

From §2.4: never a caster within ten levels of their Spellcraft and rank. The new wizard with a
rank-1 Fireball grimoire: about 3 % of casts bring a lesser gate (`rift`, `hellmouth`,
`gloom_gate` at 75–80 are a third of that case's "severe" 6.5 %). The level-20 wizard with
Spellcraft 20 on Chain lightning: a gate on about one cast in nine, a greater one on one in
twenty-five. The level-1 wizard forcing Sunfire: `lords_gate` or a Pit with lords at its mouth
on one cast in six.

## 7. The Pit: Hellegeat

The owner: a portal to an infernal dimension creates stairs down to a fiend-themed dungeon, with a
boss on its last floor.

### 7.1 The branch (data)

A new bounded branch, **Hellegeat** (Old English *helle-geat*, hell-gate; the levels "Hellegeat
1" to "Hellegeat 5"), shared: `assets/data/dungeons/hellegeat.toml`. The loader already gives a
branch outside a campaign `campaign = None` (`load_dungeons`: `campaign::of_path`), and
`has_level` finds a branch level by id whatever the campaign; the first PR confirms a shared
branch passes every check (vault pinning for a shared vault, the stray list), else one copy per
campaign in `campaigns/<id>/data/dungeons/` is the fallback (two files today).

```toml
[[dungeon]]
id = "hellegeat"
name = "Hellegeat"
arrival = "The heat hits you like a door. The air stinks of brimstone."
generator = "caves"
depth = 5
size = [80, 40]
temperature = 45
ambient = [40, 8, 4, 0]           # a dull red glow everywhere
traps = ["fireball", "pit", "pit"]
traps_per_level = [3, 5]
loot = [2, 5]
chests = { chance = 40, count = [1, 2] }
# Danger 12 on the first floor, +4 a floor, and half the player's level past 10, at most +30:
# a level-1 fool meets dretches and lemures; a level-40 one meets hezrous and bone devils.
danger = { base = 12, per_depth = 4, player = { percent = 50, from = 10, max = 30 } }
population = ["lemure", "dretch", "imp", "quasit", "hell_hound", "bearded_devil", "nightmare",
  "mezzoloth", "balgura", "barbed_devil", "incubus", "succubus", "chasme", "canoloth", "hezrou",
  "bone_devil", "night_hag", "vrock", "nycaloth", "erinyes"]
stray = 0
out_of_depth = 10
shops = { chance = 0 }
room_effects = { chance = 0 }    # or a fire-only table, a later PR

[[dungeon.levels]]
from = 5
vaults = ["wardens_hall"]
```

- **Why a branch and not a hand-made level**: the Pit can open at any character level, and
  `danger`'s `player` term is the one place in the data that scales with the player (the
  README: "an area that grows with the player"). Everything else (generation on first visit,
  persistence, the `<` of the first floor, the `>` down, loot, chests, traps, danger) is the
  existing branch machinery.
- **The population is `loosed`?** No: the Pit's own inhabitants keep their factions (demons,
  devils, yugoloths, fiends), so the Blood War is fought on its floors (demons and devils are
  hostile both ways) and the player can slip past a brawl. Only what a gate calls is `loosed`.
  The two gates a miscast makes at the Pit's mouth are `loosed`.
- **Temperature 45 °C**: the temperature system reads a branch's `temperature`; what heat does to
  the player is the existing rules' business (nothing yet beyond the °C shown; a later issue).

### 7.2 The portal (engine)

**`portal { gate = "<gate id>" }`** (new kind): on the caster's level,
1. the free cell around the caster nearest the first free one becomes **stairs down**
   (`LevelState::terrain.set(stairs)` and `LevelState::stairs.push(StairsLink { Down, to:
   "hellegeat:1", name: Some("the Pit") })`: exactly what `World::open_lair` does for a layer
   boss's lair, `world/layer_boss.rs`), and
2. the `gate` comes to the next free cell (`miscast_summon`).
3. Said: "The floor splits, and a stair goes down into red dark. Something waits at its head."

Refused, the gate alone made, and said so, when: the level is the **overworld** (no cell to
dig a stair in; the gate stands on the map cell), an **endless floor** (Grundlēas forgets the
floor left: the Pit's `<` would lead to a floor that is gone), or the Pit is **sealed** (§7.3). A
town, a hand-made level, a branch floor, the showcase: all take it, townsfolk and all.

**Travel**: `World::travel` already resolves `hellegeat:1` (a branch level), generates it on the
first visit with `entered_from = the level the stairs are on`, and arrives on its `<`; climbing
out arrives on the `>` whose `to` is `hellegeat:1`. One engine change: **a later portal from
another level** arrives at the same first floor, whose `<` still points at the first level. On
arrival into a branch's first floor from a level that is not where its `<` leads, **retarget
the `<`** (`stairs[up].to = from`; one line in `travel` after `visit`, for a branch marked
`portal = true` in its `[[dungeon]]`, so no other branch changes). Several levels may then hold
a `>` into Hellegeat; the Pit has one mouth at a time, the last one used.

**Saves**: nothing new but the stairs on the level (already saved in `LevelState::stairs`) and
the warden's death in `Quests::slain` (already saved).

### 7.3 The warden and the sealing

- **The Warden of the Gate** (`gatewarden`; a name in the campaigns' register comes with the
  roster PR, e.g. *Hellweard*): a **unique fiend of level 50**, faction `loosed`, a horned
  devil's body (hp 180, Protection 10, 3d10 claws, a `held` whip, fire 100, poison 100), with a
  `summon` of its own (`rift` and `hellmouth` gates, 1 every 120 s, most 2: the warden opens
  gates), in the vault `wardens_hall` on floor 5 (a hall of basalt with a hoard: chests of the
  floor's band and `loot` cells at item level 30). The vault is pinned by floor 5's `vaults`, as
  Fyrdelf's drake is.
- **Its death seals the Pit** (`Quests::slain` records it, as every unique's death is): from then
  on `portal` makes the gate alone ("The ground shudders, but the Pit is sealed."), and every
  `>` into `hellegeat:1` **closes when the player next stands on its level** (arrival from the
  Pit included: "The stair crumbles to rubble behind you."): the stairs cell becomes floor and
  the link is removed (`close_pit`, the reverse of `open_lair`, one line in `travel`'s arrival).
  Closing on arrival rather than at the kill is what keeps the player from being sealed in.
  Hellegeat's floors persist like any branch's; nothing leads to them again.
- **Experience**: the warden's kill pays as a unique's does; `discover_level` pays the five
  floors; the hoard is the loot. Sealing the Pit is the reward in the fiction and could later be
  a quest's objective (not in this design).

### 7.4 What is new engine work, what is data

| Piece | Kind |
|---|---|
| `hellegeat.toml`, the `wardens_hall` vault, the `gatewarden` type | data |
| the `portal` kind: stairs + gate, the refusals | code, small: `open_lair`'s body reused |
| `portal = true` on `[[dungeon]]` and the `<` retarget on arrival | code, one field and one line |
| `close_pit` on arrival once the warden is slain | code, one function and one line |
| the `truth` dump showing the Pit's stairs, `gen --level hellegeat:3` | free (branch machinery) |
| the scenario `--scenario hellegeat` (the first floor, a dev scenario of the showcase) | data |

## 8. Words

The owner: vary them. The register for a system line is plain modern English; the lines borrow
the world's voice sparingly ("the weave" is a word of the world and stays in some) and never a
number. Every row's player line and `seen` line (`{who}` the creature). The `seen` lines of the
gates and the Pit are what the player sees happen, not what the creature feels.

| Id | Player | Seen |
|---|---|---|
| glimmer | The air glimmers for a moment and is still. | The air glimmers about {who}. |
| motes | Motes of colour drift from your hands and fade. | Motes of colour drift from {who}'s hands. |
| refund | The weave slackens, and the spent power runs back into you. | {who} brightens for a moment. |
| hum | A low hum hangs in the air after the spell. | A low hum hangs about {who}. |
| echo | The spell rings like a struck bell and sounds twice. | {who}'s spell sounds twice! |
| sparks | Sparks leap from your fingers and bite. | Sparks bite {who}'s fingers. |
| blessing | Stray power settles on someone as a blessing. | A stray blessing settles near {who}. |
| quickened | Power hums through your limbs, and they quicken. | {who} quickens. |
| fizz | The spell fizzes, and a little of your power with it. | {who}'s spell fizzes. |
| singe | The weave snaps back and singes you. | {who}'s own spell singes it. |
| wobble | The world lurches, and you with it. | {who} reels. |
| crack | The spell ends with a crack like a whip. | {who}'s spell ends with a crack. |
| leak | Your power leaks away through a fraying in the weave. | {who}'s light gutters. |
| lash | The spell turns and lashes you. | {who}'s spell lashes it. |
| flinch | Something cold looks back at you through the spell. | {who} flinches from something unseen. |
| shiver | A shiver runs through you and leaves you weak. | {who} shivers and sags. |
| fog | Grey fog boils from the spell and wraps you. | Grey fog boils up about {who}. |
| slip | Space knots, and you are standing where another stood. | {who} and another change places in a blink. |
| drowse | A great weariness settles over you. | {who}'s eyes droop. |
| thunder | The spell breaks with a clap of thunder. | {who}'s spell breaks with a clap of thunder! |
| scorch | The weave snaps back and scores you. | {who}'s own spell scores it. |
| gloom | The spell drinks the light, and dark pools about you. | Dark pools about {who}. |
| seep | Half your power drains out through the broken weave. | {who} dims. |
| slick | The ground about you weeps and turns slick. | The ground about {who} turns slick. |
| slur | Your tongue stumbles, and no word of power will come. | {who} mouths words, and nothing comes. |
| stumble | The words slip from you: you know this spell less well than you did. | {who} looks puzzled. |
| burn | The weave bursts in your hands and burns you. | {who}'s spell bursts in its hands. |
| ache | A deep ache settles in your bones. | {who} stoops. |
| displaced | The world folds, and you are flung through it. | {who} is flung through a fold in the air. |
| lapse | Something you had learned slips your mind. | {who} shakes its head. |
| chill | Cold sinks into your limbs and slows them. | {who} slows, as if wading. |
| falter | The spell's shape falters in your mind: two ranks of it are gone. | {who} falters. |
| dry | Every drop of your power is torn out of you. | {who} goes grey. |
| blast | The spell bursts, and the blast hurls you back. | {who}'s spell bursts, and the blast hurls it back. |
| racked | You are racked, and something of you is bent out of true. | {who} is racked. |
| backlash | The spell turns and breaks over your own head! | {who}'s spell breaks over its own head! |
| withering | The weave feeds on you, and something of you goes with it. | {who} withers. |
| rift | A rift tears open beside you, and the stink of the Abyss pours out! | A rift tears open beside {who}! |
| hellmouth | The ground splits into a mouth of brass and fire! | The ground splits beside {who} into a mouth of brass and fire! |
| gloom_gate | A gate of gloom stands open where there was none, and something bays beyond it. | A gate of gloom opens beside {who}. |
| forgetting | The spell is gone from you: you cannot recall a word of it. | {who} stares at its hands. |
| rend | The raw power rends you. | {who} is rent by its own power. |
| twofold_withering | The weave feeds on you twice over. | {who} withers before your eyes. |
| abyssal_gate | A gate yawns open beside you, and the Abyss looks through! | A gate yawns open beside {who}! |
| rebound | The spell rebounds, and its full force falls on you! | {who}'s spell falls on it with its full force! |
| iron_gate | An iron gate grinds open beside you, and the hells are behind it! | An iron gate grinds open beside {who}! |
| black_gate | A black gate opens beside you, and things that serve for pay look out. | A black gate opens beside {who}. |
| unravelling | The weave unravels through you, and the raw power burns you from within! | {who} burns from within! |
| deep_withering | The spell feeds deep, and takes what will not come back. | {who} withers horribly. |
| amnesia | The spell tears loose from your mind and takes another with it. | {who} clutches its head. |
| dire_gate | A dire gate rips open, and what steps through has a name. | A dire gate rips open beside {who}! |
| harrowing | You are harrowed by your own power. | {who} is harrowed by its own power! |
| pit | The floor splits, and a stair goes down into red dark. Something waits at its head. | The floor splits beside {who}, and a stair goes down into red dark! |
| threefold_withering | The weave feeds on you thrice over. | {who} shrivels. |
| ruin | Ruin pours through the broken weave. | Ruin pours through {who}! |
| arch_gate | An arch-gate opens beside you, wide enough for a lord of the hells. | An arch-gate opens beside {who}! |
| pit_yawning | The floor yawns open, and a stair goes down into red dark. A great gate stands at its head! | The floor yawns open beside {who}, and a great gate stands at the stair's head! |
| blank | Every spell you ever learned is torn from you. | {who} stares, empty-eyed. |
| unmaking | The power unmakes you from within. | {who} is unmade from within! |
| ruinous_withering | The weave feeds on you until there is little left. | {who} is wasted to the bone. |
| pit_roaring | The floor splits with a roar, and a stair goes down into red dark. A dire gate stands at its head! | The floor splits beside {who} with a roar, and a dire gate stands at the stair's head! |
| lords_gate | A gate opens for a lord of the hells. | A gate opens beside {who} for a lord of the hells! |
| annihilation | The power annihilates everything it touches, and it touches you. | {who} is torn apart by its own power! |
| pit_and_lords | The floor splits, a stair goes down into red dark, and a lord of the hells is coming up it! | The floor splits beside {who}, and a lord of the hells is coming up the stair! |
| oblivion | Oblivion. | {who} is unmade. |

The messages for the new effect kinds' second lines (whom the heal mended, which stat aches,
which spell was forgotten, the gate's death) are the engine's and named in §5–§6.

## 9. Creatures' miscasts

Creatures keep checking with their own Spellcraft and Intelligence; every shipped caster is at
or near the floor (§2.4: the worst shipped total is 44, a `scorch`; the troll shaman's lightning
bolt is the exception at 10 % of casts, with nothing worse than `gloom`). So the table past 70
is, in shipped data, the player's alone; a campaign's creature with a borrowed spell could reach
it. Per kind:

| Kind | A creature's miscast |
|---|---|
| nothing, harm, mana, status, teleport, refund, twice, boon, noise, zone, stat_damage, drain | as the player's (a creature's drain checks Willpower as any check) |
| swap | never with the player (§5.4) |
| rank_loss, forget | nothing (its spells are the type's list); the `seen` line speaks |
| lose_mark, mark | nothing for a creature (marks are the player's training); already so |
| backfire | as the player's; a creature's heal rebounds on the nearest actor hostile **to it** (the player, often) |
| summon (a gate) | the gate is `loosed`: hostile to the creature and its kin too |
| portal | the gate alone: a creature never opens the Pit |

The `seen` lines are rare flavour and may be a little longer than the player's, as §8 writes
them.

## 10. Implementation plan

One feature per PR, each with its tests, spec, `docs/dev/` and wiki updates; `cargo xtask wiki`
regenerates the table. Order within a phase is free; phases depend on the one before where
marked.

**Phase 1: the rules and the protected half** (no new creatures, no new kinds beyond the four
smallest).
1. **The check and the total**: `rank_ease`, `die` as a `Roll` (2d12), `per_margin` 5,
   `safe_below` 71 with the harm cap and the loader's whitelist; `surge` kind and file removed;
   Senseless removed; the first 35 rows (1–70) laid out with the existing kinds, and `noise` and
   `zone` (§5.4); the old upper rows re-cut onto 71+ with existing kinds only (`backlash`,
   `withering`, `rend`, `rebound`, `unravelling`, `harrowing`, `ruin`, `unmaking`,
   `annihilation`, `oblivion`), the gate and pit rows standing in as `harm` of the neighbour's
   dice until phases 3–4 fill them (the tiling holds at every step). Wiki tables regenerate;
   `docs/balance.md`'s probe spends the wizard's rolls on Spellcraft. *Data mostly; code: the
   `Roll` die, `rank_ease`, the cap, the whitelist, `noise`, `zone`, the creature-swap
   exclusion.*
2. **Rank loss and forgetting** (§5.1): `rank_loss`, `forget`, `Actor::forget_spell`,
   `Actor::forgotten`; rows `stumble`, `falter`, `forgetting`, `amnesia`, `blank`. *Code and
   data.*
3. **Stat damage and the drain's `times`** (§5.2): `stat_damage`; `drain { times }`; the five
   withering rows and the two aches. *Code, small; data.*
4. **Backfire fallbacks** (§5.3): the rebound heal, the buff on a hostile, the hostile summon, the
   fallback harm. *Code.*

**Phase 2: the words** (§8): one pass over every file once the rows are final; the wiki page's
prose (Scribe, the stance, the half-casters, the protected half, ranks). *Data and docs.* Can run
beside phase 1's later PRs.

**Phase 3: the gates** (§6), after phase 1.
5. **`hostile_to_others` and the `loosed` faction**: `content/faction.rs`, `factions.toml`, the
   fixtures. *Code, small; data.*
6. **`rooted`**: the flag, the `creature_turn` arm, the move guards, the dump. *Code.*
7. **The nine gate types** and their rows (`summon` of a gate; the stand-ins from PR 1 replaced).
   *Data; a roster-style PR with `docs/dev/creatures/gates.md`.*

**Phase 4: the Pit** (§7), after phase 3 (the Pit's mouth is a gate).
8. **Hellegeat the branch**, reachable by a dev scenario and `gen --level hellegeat:N`; the
   shared-branch question settled. *Data, and a loader fix if a shared branch fails a check.*
9. **The `portal` kind**: stairs + gate, the refusals, `portal = true` and the `<` retarget; the
   four pit rows. *Code, small; data.*
10. **The warden and the sealing**: the `gatewarden` type, `wardens_hall`, `close_pit`. *Data;
    code, small.*

**Phase 5: polish**, any time after phase 1.
11. **Balance pass**: rerun the probe's Miscasts table (the wizard's rolls on Spellcraft; a
    Spellcraft-first and a Concentration-first career), adjust `per_margin` or a row's dice if
    the numbers in §2.4 drift. *Data.*
12. **The wiki's miscast page** rewritten around the new table: the protected half, the gates,
    the Pit (a world fact, not a walkthrough). *Docs.*

Fourteen numbered pieces; PRs 1–4 and 11–12 touch no creature or level, PRs 5–10 are the fiends.

## 11. Docs, wiki, tests

- **Spec**: `spec/08e-blood-and-status.md` "Miscasts" (the total, the protected rule, the new
  kinds, the gates and the Pit in a sentence each); `spec/07b-data.md` points at the new faction
  flag and the `rooted` field; spec §7 gains "The Pit" beside "The endless branch".
- **`docs/dev/spells.md` "Miscasts"**: the data (`rank_ease`, `die` as a `Roll`, `safe_below`),
  the kinds, the fallbacks, the whitelist; **`docs/dev/summoning.md`**: gates as summoners with
  no cost, `rooted`; **`docs/dev/creatures.md`**: `hostile_to_others`, `rooted`, and a line for
  `docs/dev/creatures/gates.md`; **`docs/dev/mapgen.md`**: Hellegeat under the branches, the
  `portal` flag and the `<` retarget, `close_pit` beside `open_lair`; **`docs/dev/stats.md`**:
  `stat_damage` as a second source of `Actor::damage`, `drain`'s `times`;
  **`docs/dev/skills.md`**: nothing (Concentration unchanged).
- **Wiki**: `miscasts.md` (the generated table grows to 65 rows with the protected half marked;
  the chances table gains the rank column: a sentence "each rank above the first takes 1 off the
  difficulty"), `creatures.md`/`creature-powers.md` (the gates' rows and summons come from the
  data), a `the-pit.md` or a section of `magic.md` for Hellegeat as a world fact, `stances.md`
  (Defensive's −5 on the total), `scrolls.md` (Scribe makes no check). Terms linked
  (`wiki/TERMS.md`); `cargo xtask wiki --check-links`.
- **Tests** (`world/miscast_tests.rs` and new files): the rank ease in the difficulty; the 2d12
  total and the margin's weight; the cap (a 1-hp caster takes nothing; a 5-hp one is left at
  1); the loader's whitelist (a protected `summon`, `drain`, `backfire`, ticking status, fire
  zone refused; `rank_loss` allowed); rank loss stopping at 1, forgetting removing the spell and
  `forgotten` skipping the experience; `stat_damage` mending; `drain { times = 2 }` drawing
  twice; the rebound heal on the nearest hostile and the fallback harm; `noise` heard, `zone`
  laid; a creature's swap never the player; the `loosed` matrix (neutral on the diagonal,
  hostile everywhere else); a rooted gate that never moves and summons `loosed` dretches to its
  cap and cooldown; the Pit opening (stairs and gate), refused on the overworld and an endless
  floor, the `<` retargeted on a second mouth, `close_pit` on arrival once the warden is slain;
  `mapgen/hellegeat_tests.rs` (the warden's hall on floor 5 over ten seeds, the first-floor
  snapshot); determinism by `replay --verify` for a run that miscasts into the Pit and back.
- **The balance probe** (`world/balance_probe/miscasts.rs`): the wizard's rolls on Spellcraft
  (or both careers), the band distribution per miscast as §2.4 prints it.

## 12. Open questions for the owner

Only what is still undecided; each with a recommendation.

1. **Class-granted spells and forgetting.** `forget` cannot remove a spell the class grants
   (it is not in `Actor::spells`); the design makes such a spell fall to rank 1 instead (§5.1).
   The alternative is a saved exclusion set so a wizard can lose Magic missile too.
   **Recommendation**: rank 1 only; what the class taught stays.
2. **The die.** 2d12 + 5 × miss (§2.2), or keep the exploding d66 with a bigger `per_margin`?
   The d66 keeps a 1-in-66 chance of any near miss landing anywhere, which the owner's "ordered"
   argues against. **Recommendation**: 2d12; the explosion goes.
3. **Where the Pit may open.** Anywhere but the overworld and an endless floor (§7.2), towns
   included: a Sunfire miscast in Cēaptūn's market puts a hellmouth among the stalls.
   **Recommendation**: yes, towns included; the gate at the mouth is `loosed` and the
   townsfolk pay, which is the story the owner's fiends tell.
4. **Do gates close on their own?** The design: never, until killed (§6.2). The alternative is
   a lifetime (an hour of game time) after which the gate closes and what it called stays.
   **Recommendation**: never; a gate left behind is a reason to come back with a bigger sword.
5. **The Pit is one per run, sealed for good by the warden's death** (§7.3), later portals then
   making the gate alone. The alternative is a Pit that can be reopened unsealed (a new warden
   each time). **Recommendation**: sealed for good; a run has one descent into the hells.
6. **Shared branch or per campaign.** Hellegeat as the first shared branch
   (`assets/data/dungeons/hellegeat.toml`) if the loader's checks allow it (§7.1), else one copy
   in each campaign. **Recommendation**: shared, fixing the loader if one check refuses it; the
   Pit belongs to no world in particular.
