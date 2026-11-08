# Warlock — Miscasts: a review of #1275

Design draft for the owner's review (2026-11-08), written against `main` at 700f8f71 (PR #1275,
issue #1271 and its owner comment). Nothing here is implemented; §6 lists proposals only. The
numbers come from an exact enumeration of the rules (2d12 × exploding d66, §3) and a Monte Carlo
of a wizard's career through the real skill ladder (§4.1); the script is described in §3.

## Brief for the owner

**Now.** Every cast with one's own mana checks 2d12 + Intelligence modifier + Spellcraft's `cast`
(+2 at 20, +14 at 100, +34 at 200) against 8 + spell level (+1 per 10 % armour cost); a failure
rolls a 20-entry table at d66 (exploding) + 3 × margin − `ward` − stance. The targets hold on
paper: a new wizard miscasts Spark 1 in 14, a master 1 in 144, the top of the table kills.

**Problems, most important first.**
1. **Spellcraft grows only by level-up rolls, never by casting** (casting just banks marks that
   size a roll's die). Half a wizard's rolls on it keep own-level spells at 10–30 %; none, and a
   level-20 wizard miscasts Chain lightning 96 %. Nothing done *between* level-ups moves the
   odds, and 2d12 is a cliff: 1 % to 99 % over 14 points (§4.1–4.2).
2. **The table is flat where miscasts happen**: within 8 of the difficulty every entry from Sparks
   to Displaced comes up 7.6–9.1 %; "ordered" is cosmetic, the deadly tail needs a margin of 11+
   or a 1-in-66 explosion (§4.3).
3. **Positions lie**: Senseless (84–89, above Backlash) is one slowed action (Stunned `max = 1`,
   its severity 2 clamped); Dread is −3 to hit for 12 s; both milder than Scorch (42–47) (§4.4).
4. **Surge can kill a new caster unseen**: any of 33 damage spells (Sunfire 10d6 among them) as a
   burst of radius ≥ 1 around someone within 6; an adjacent foe puts a 13-hp wizard in it (§4.5).
5. **Backfires do nothing for heals, buffs, summons, utilities**: a priest's Backlash and Rebound
   are words only (§4.6). Smaller: Defensive is a free −5; Concentration unused; half-casters at
   1 in 3 to 1 in 7; nineteen of twenty lines open "Your weave …" (§4.7–4.9).

**Directions** (not exclusive; my pick is **A + C**):
- **A. A lever inside the run** (recommended): a spell's rank (#1276) takes 1 off the difficulty
  per rank above 1, and Concentration wards. Trade-off: ranks do one more thing; rereading
  grimoires becomes how a spell is tamed, time-priced, no farming limit. §6.1.
- **B. Retune the dice**: a steeper `cast` curve. Trade-off: the dedicated wizard hits the floor
  at level 10, the neglecter is no better off, the cliff stays. Not alone. §6.2.
- **C. Fix the table** (recommended regardless): order by true severity, margin × 5, Surge
  bounded, a Backfire fallback, five new entries from existing effects, a wording pass. §6.3–6.5.
- **D. Minimal**: the bugs (Senseless, Surge, hollow Backfire) and the words. §6.6.

## Contents

1. What landed
2. The table as it stands
3. Method
4. Findings
5. What works and should stay
6. Proposals, by direction
7. Open questions for the owner

## 1. What landed

| Piece | As built (#1275) |
|---|---|
| When | after every spell cast with the caster's own mana, the player's and a creature's; never a scroll, wand, flask, or a miscast's own echo |
| The check | 2d12 + Intelligence modifier + talents' Intelligence checks + Spellcraft `cast` (curve `[[0,0],[20,2],[60,8],[100,14],[150,24],[200,34]]`) against `8 + level + armour % ÷ 10`; natural 2 always fails, natural 24 always makes |
| The total | d66 (66 → roll again and add) + 3 × margin − Spellcraft `ward` (`[[0,0],[200,20]]`) − stance `miscast`, at least 1 |
| Mitigations | `ward` (skill affixes and Skill focus count; no Spellcraft affix exists), stance: Defensive 5, Aggressive −5, Frenzy −10 |
| The spell | always goes off first, in full |
| Training | a cast marks Spellcraft 25 % (and Concentration 25 %), miscast or not; marks are cashed only by level-up rolls |
| Creatures | check with their own Spellcraft and Intelligence; the player is told through the entry's `seen` line when they see it |
| Death | `Cause::Miscast(spell)`: "Killed by their own miscast fireball" |
| Data | `data/miscast.toml` (`stat`, `base 8`, `per_level 1`, `armour_percent 10`, `die 66`, `per_margin 3`), `data/miscasts/*.toml` one entry a file, tiling 1 and up |

## 2. The table as it stands

Band is my reading of what the effect really does to a player (§4.4), not the file's position.

| Range | Id | Effect | Real weight |
|---|---|---|---|
| 1–8 | glamour | nothing | nothing |
| 9–14 | sparks | 1d2 fire | mild |
| 15–19 | refund | the cast's mana back | **boon** |
| 20–25 | dizzy | Staggered 1: the next action costs 150 % | mild |
| 26–30 | boon | Blessed 2 (+1 to hit, 20 min) on the caster or someone within 6, friend or foe | **boon/chaos** |
| 31–36 | mana_leak | 25 % of mana left | mild |
| 37–41 | echo | the spell again at its target, free | **boon** |
| 42–47 | scorch | 2d4 of the spell's own type | mild–bad at level 1 (13 hp) |
| 48–52 | swap | places with someone within 6 | chaos |
| 53–58 | dread | Frightened 2: −3 to hit for 12 s (two turns) | mild |
| 59–63 | surge | a random damage spell (33 candidates) bursts around someone within 6 | chaos, **can kill** (§4.5) |
| 64–70 | mana_drain | 75 % of mana left | bad |
| 71–76 | displaced | teleported on the level | chaos |
| 77–83 | backlash | the spell at base roll on the caster (burst radius ≥ 1; a hex on them); **nothing** for a heal, buff, summon, utility | bad / nothing |
| 84–89 | senseless | Stunned "2" → 1 (`max = 1`): one action at 150 %, flat-footed for it | **mild** |
| 90–96 | withering | 1 point of a random stat for good unless Willpower vs 16 | severe |
| 97–104 | rift | a dretch (level 8, 10 hp, claws 1d10) beside the caster | severe |
| 105–114 | rebound | the spell at the cast's full roll on the caster; nothing for spells without damage | deadly for a damage caster |
| 115–124 | gate | a vrock (level 34, 85 hp, 3d10, terror aura) | deadly |
| 125+ | unravelling | 8d10 force (mean 44; wizard hp 13 / 26 / 58 / 104 at levels 1 / 10 / 20 / 30) | deadly |

## 3. Method

A Python script (not committed; `scratchpad/miscast_model.py`, rerunnable from this document's
numbers) that:

- enumerates the 144 natural rolls of the check (`check::makes`: a natural 2 fails, a 24 makes)
  for a bonus and a difficulty, with each failure's margin `by = max(0, difficulty − natural −
  bonus)`;
- enumerates the exploding d66 exactly (three explosions deep; the rest is below 10⁻⁵) and
  `total = max(1, face + 3·by − ward)` against the table above, giving each entry's chance
  **per cast** and **given a miscast**;
- simulates a human wizard's Spellcraft over 60 levels through the real ladder (`[ladder]` dice
  `1, 1d3, 1d5, 2d4, 3d3, 3d4, 3d5, 4d4, 4d5` at marks `0, 5, 12, 25, 50, 100, 200, 400, 800`;
  `known_for` one step up; 50 % of marks spent a roll; half results from 100), with 2 class rolls
  + 1 race roll + Intelligence's (the probe's wizard Intelligence 14 → 23), the design's days a
  level (1 to level 5, 2 to 15, 3 to 30, 4 beyond), a given number of casts a day marking 25 %,
  and a given share of each level's rolls spent on Spellcraft; 400 careers averaged.

Hit points, mana, and Intelligence by level are the probe's (`docs/balance.md` "Players"), the
spell list and levels `data/spells/`, the class spell tables `data/classes/`.

## 4. Findings

### 4.1 Spellcraft is a roll-allocation question, not a casting one

Skills grow at level-up only (`skills.toml`: "Marks raise nothing by themselves"). Casting banks
marks (25 % a cast); marks only pick the die of a roll: with 5–12 banked a wizard's roll is 1d5
(mean 3), with 25–50 it is 3d3 (mean 6). The balance probe's `levelup:auto` spends every roll on
the first trained skill, Concentration, which is why its Spellcraft never leaves 20 and its
level-30 wizard miscasts Chain lightning 93 %. A player who does the same gets the same.

Mean Spellcraft, then the cast bonus and the miscast chance of the highest class spell at that
level (Ray of frost 1, Magic missile 3, Blink 6, Shatter 8, Chain lightning 20 at 24, Sunfire 30
at 36) and of a spell of the character's own level (capped at 30):

| casts a day | rolls on Spellcraft | L5 | L10 | L15 | L20 | L25 | L30 | L40 |
|---|---|---|---|---|---|---|---|---|
| 10 | none | 17 (+3): L3 15 %, L5 25 % | 17 (+4): L8 38 %, L10 54 % | 17 (+5): L15 81 % | 17 (+5): L20 98 % | 17 (+5): L20 98 % | 17 (+6): L20 96 %, L30 99 % | 17 (+6): L30 99 % |
| 10 | half | 33 (+6): L3 4 %, L5 10 % | 57 (+10): L8 7 %, L10 15 % | 82 (+15): L15 15 % | 107 (+19): L20 19 % | 125 (+22): L20 7 %, L25 31 % | 143 (+27): L30 31 % | 184 (+35): L30 1 % |
| 25 | half | 37 (+6): 4 %, 10 % | 77 (+13): 1 %, 4 % | 110 (+20): 1 % | 137 (+25): 1 % | 164 (+30): 1 % | 191 (+37): 1 % | 250 (+39): 1 % |
| 10 | all | 49 (+8): 1 %, 4 % | 93 (+15): 1 % | 125 (+23): 1 % | 161 (+30): 1 % | 197 (+37): 1 % | 234 (+39): 1 % | 310 (+39): 1 % |

(The probe's wizard casts 6–15 times a day from the pool, "Caster style pace"; 25 a day assumes
regeneration and rests between fights.)

Reading: the system has three modes and the player picks one at level-up. Spend nothing: nearly
every own-level spell miscasts. Spend half: a steady 10–30 % on own-level spells to level 30, then
the floor. Spend all: the floor from level 10, and the whole system is a 1-in-144 nuisance for the
rest of the game. Nothing the player does *between* level-ups moves the odds. Priests and druids
(Spellcraft trained but not `known_for`) roll a step smaller; their Intelligence is lower, so the
same shares land them one row worse.

### 4.2 The check is a cliff

Chance of a miscast (% of casts) by spell level and bonus, out of armour (the wiki's table):

| level | difficulty | +0 | +3 | +6 | +10 | +15 | +20 | +30 | +40 |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 9 | 19.4 | 6.9 | 0.7 | 0.7 | 0.7 | 0.7 | 0.7 | 0.7 |
| 3 | 11 | 31.2 | 14.6 | 4.2 | 0.7 | 0.7 | 0.7 | 0.7 | 0.7 |
| 6 | 14 | 54.2 | 31.2 | 14.6 | 2.1 | 0.7 | 0.7 | 0.7 | 0.7 |
| 11 | 19 | 85.4 | 68.8 | 45.8 | 19.4 | 2.1 | 0.7 | 0.7 | 0.7 |
| 20 | 28 | 99.3 | 99.3 | 95.8 | 80.6 | 45.8 | 14.6 | 0.7 | 0.7 |
| 30 | 38 | 99.3 | 99.3 | 99.3 | 99.3 | 97.9 | 80.6 | 14.6 | 0.7 |

From `difficulty − bonus = 3` (1 in 144) to `17` (99.3 %) is 14 points: three to five spell levels
either side of "comfortable" cover the whole range. That is what 2d12 against a flat number does
and it is the game's check everywhere; it is not wrong in itself, but it means **tuning the curve
cannot make miscasts a steady companion**: any caster is at the floor or far above it, with a
narrow window of 5–30 % that a few spell levels cross. Only a lever that moves per spell (§6.1)
widens where the player can stand.

The starting targets by class (Spellcraft 2d6+10 ≈ 17, level-1 spell): wizard +3 → 6.9 % (1 in
14), witch +2/3 → 6.9–10.4 %, priest +2 → 10.4 %, druid +1 (no Intelligence bonus) → 14.6 %, bard
(Spellcraft 2d4+4, Mend) +0/+1 → 14.6–19.4 %, ranger (Barkskin, level 3, at character level 6) +0
→ **31.2 %**. The issue's 1 in 10 to 1 in 20 holds for the three full casters; the half-casters
are at 1 in 3 to 1 in 7, which may be intended ("spells far above one's skill miscast often") but
is worth a sentence in the wiki.

### 4.3 Flat where it matters; the margin barely orders

Given a miscast, the entry distribution for the common cases:

| Case | miscast | nothing | mild | boon | chaos | bad | severe | deadly |
|---|---|---|---|---|---|---|---|---|
| new wizard, Spark (+3 vs 9) | 6.9 % | 3.9 | 44.7 | 22.7 | 17.8 | 9.7 | 0.3 | 0.8 |
| new wizard, Fireball from a grimoire (+3 vs 19) | 68.8 % | 1.1 | 39.1 | 17.8 | 22.1 | 16.0 | 2.8 | 1.1 |
| level-24 wizard, Spellcraft 20, Chain lightning (+6 vs 28, ward 2) | 95.8 % | 0.6 | 34.0 | 12.2 | 22.9 | 18.7 | 8.3 | 3.4 |
| level-24 wizard, Spellcraft 60 (+12 vs 28, ward 4) | 68.8 % | 2.9 | 40.8 | 19.4 | 20.8 | 13.4 | 1.7 | 0.9 |
| level-36 wizard, Spellcraft 100, Sunfire (+20 vs 38, ward 10) | 80.6 % | 5.5 | 41.5 | 19.9 | 19.2 | 11.5 | 1.5 | 0.9 |
| master, Spellcraft 200 (+40, ward 20), any spell | 0.7 % | 42.4 | 33.6 | 22.7 | 0.4 | 0.3 | 0.3 | 0.2 |

In every case but the hopeless one, each entry from Sparks (9–14) to Displaced (71–76) comes up
**7.6–9.1 %** of miscasts: a d66 is uniform, the margins of a near miss are 1–5, and 3 × 5 = 15
shifts the window by a quarter of the die. The margin only starts ordering at `by ≥ 8` (+24), by
which point the check fails 85 %+ of the time anyway. The deadly entries need a plain d66 of 66
plus `3·by ≥ 31` (by ≥ 11) for Rift and by ≥ 20 for Unravelling; below that only the 1-in-66
explosion reaches them (about 1.5 % of miscasts land on 67–132 that way, spread over the tail).
So for anyone who can cast a spell at all, the table is: three boons, five chaos, ten mild-to-bad,
each about as likely as the next; and for a master's 1-in-144 it is 42 % nothing, the rest mild or
good. The owner's "higher totals give greater effects" is true only of the hopeless caster.

Whether that is a problem is a taste question (§7 Q1): "more chaos than punishment" is met. But
the margin's weight is too small to carry the design's spine: a Warhammer-style table has the
margin matter.

### 4.4 Positions that lie

- **Senseless (84–89)**: `stunned` has `max = 1` and `per_action = true`: one action at 150 %
  cost, flat-footed for it. `severity = 2` is clamped by `Effects::add`. The entry two above
  Backlash is weaker than Dizzy (20–25), which is the same thing without flat-footed.
- **Dread (53–58)**: Frightened 2 eases 6 s a degree: −3 to hit for about two turns; `flees` is a
  creature rule. Milder than Scorch (42–47) and Sparks at level 1 (hit points 13).
- **Scorch (42–47)** is 2d4 of the spell's own type at every level: 5 of 13 hit points at level 1,
  nothing at level 30. The low end of the table hurts a new caster more than a veteran, which is
  backwards for a check the veteran passes anyway, but acceptable; it should not sit above Dread.
- **Mana drain (64–70)**: 75 % of what is left, the most painful non-lethal entry for a wizard with
  300 mana (a day's pool), as common as Sparks in the flat zone.

### 4.5 Surge: an unearned death

`surge` draws among **every** damage spell with a roll (33: from Burning hands 2d4 to Sunfire
10d6), picks someone within 6 in a clear line (not the caster), and fires the spell as a burst of
its radius **or 1** around them, at base roll. A caster fighting an adjacent foe is inside every
surge. Mean damage over the 33 is about 12; Fireball 4d6 (radius 2), Flame 6d6 (2), Chain
lightning 6d6 (1), Sunfire 10d6 (3). A level-1 wizard has 13 hit points and miscasts Spark 6.9 %;
Surge is 7.6 % of those: about 1 cast in 190 fires a spell the player has never heard of at their
own feet, and perhaps one in three of those kills ("Killed by their own miscast spark"). Companions
and familiars standing near take it too. This is the arbitrary death the brief asked about.

### 4.6 Backfire has nothing to say to half the casters

`backfire` bursts a damage spell on the caster and lays a hex on them; **any other kind does
nothing more than the entry's words**. For a priest (Mend, Cure wounds, Blessing, Benison, Silence
…), a druid's summons, a wizard's Haste or Blink, Backlash (77–83) and Rebound (105–114) are free
passes: "the spell's full force falls on you!" and nothing happens. Their deadly band is Rift,
Gate, Unravelling only. Rebound of a *heal* could be the heal on the nearest hostile; a buff's
rebound the buff on a foe; a summon's a hostile summon; a utility's a mana loss. At least a
fallback harm by spell level is needed.

### 4.7 Mitigations

- **Defensive stance** gives −5 to every miscast total for −2 to hit and −1 damage, which a caster
  seldom feels. Every caster will stand Defensive forever; that is an "unbalanced choice" the
  owner allows, and it is the only choice the player makes in the system. Fine, but note it in
  the wiki.
- **Ward** (10 at Spellcraft 100, 20 at 200) matters only for the natural-2 floor: a master's
  miscast is 42 % Glamour. A Defensive Spellcraft-100 caster takes 15 off, so a near miss never
  reaches Swap or beyond. Good.
- **Concentration** (wizard `known_for`, marked 25 % by the same casts, trained by every caster) has
  no part. It is the natural home of a ward ("keeping one's head") and would give priests and
  druids, who are not `known_for` Spellcraft, a second lever.
- **Armour**: +1 per 10 % spell cost: chain 10, scale 20, banded 30, plate 40, large shield 10,
  tower 20, great helm 10. A sword-and-board priest in scale (+3) goes 10.4 → 19.4 % on level-1
  spells; in plate with a tower shield (+6) 54 %. Reasonable; Battle caster eases it.
- **Talents**: `TalentMods::check(intelligence)` counts; nothing in the `spellcraft` tree touches
  the check or the table. The skill-tree promise ("talents, items" in the owner's comment) has no
  item yet (no Spellcraft affix exists).

### 4.8 Creatures

38 of 42 caster types sit at the 0.7 % floor (`docs/balance.md` "Miscasts"); kobold shaman, pixie,
wisp, troll shaman at 4–10 %. So the `seen` lines are rare flavour, which is fine. Three effects
cross to the player: a creature's **Swap** moves the player into the creature's cell (in a
corridor fight, into the middle of its friends); its **Echo** is a free second spell at the player;
its **summons** bring a demon hostile to all. All three are fair chaos, but a creature's natural-2
Swap with the player (a kobold shaman's one-in-144) is a swing the player can neither see coming
nor read; §6.3 suggests the player is never the swap partner of a creature's miscast.

### 4.9 Words

Nineteen of twenty player lines open "Your weave …" (slips ×6, tears ×2, frays, rings, snaps,
twists, drinks, folds, turns, bursts, feeds, rips, rebounds, unravels); Surge is the odd one. Read
in sequence over a session they blur into one message. The `seen` lines always say "it". The
register is plain modern English, right for a system line, but the lines could borrow a little of
the world's voice ("the weave" is already a word of the world). §6.5 has a set.

### 4.10 Neighbours

- **Ranks (#1276)** and miscasts do not speak: rereading a grimoire deepens a spell's power but
  not the caster's grip on it. That is the obvious in-run lever (§6.1).
- **Scribe** (wizard, level 12) sidesteps the check: a scroll makes none, at three times the mana
  and a study's time. That is a fair price and a good reason to scribe; it should be said in the
  wiki's Miscasts page.
- **The mana floor (#1276)** makes low spells 4+ mana: fewer casts a pool, fewer marks; no change
  to the odds.
- **Spellcraft's `cast` 25 % mark** and Concentration's both fire on one cast: a cast is the most
  mark-productive action in the game, which is fine as marks only size dice.

## 5. What works and should stay

- The spell always resolving first, the natural-2 floor, the natural-24 always making, the check
  after the cast's own draws (deterministic, replayable).
- One ordered table in data, one entry a file, tiling checked at load; the mishap machinery
  shared rather than duplicated; `seen` lines with `{who}`.
- The boons and the chaos (Refund, Echo, Boon, Swap, Displaced) are genuinely fun and distinct,
  and a miscast is good news about one time in four, which matches "more chaos than punishment".
- Creatures checking with their own Spellcraft, the floor keeping it rare.
- The wiki page: exact chances by level and bonus, the whole table with ranges.
- `Cause::Miscast` and its graveyard line.

## 6. Proposals, by direction

### 6.1 Direction A: a lever inside the run

**A1. Spell rank cuts the difficulty.** `difficulty = base + per_level × level + armour − rank_ease
× (rank − 1)` with `rank_ease = 1` in `miscast.toml`. Rank 10 casts like a spell nine levels
lower. A new wizard who finds a Fireball grimoire: rank 1 68.8 %, rank 5 (−4) 38.2 %, rank 10
(−9) 10.4 %. The deepening reading is time-priced and unlimited (#1276, no anti-farming), so
"practising into a spell" is rereading its book, which is what the fiction says anyway. Costs:
one line in `cast_difficulty`, a field, the wiki's chance table gains a sentence, the Magic
page's rank column gains meaning. A knock-on: ranks make a spell both stronger and safer, which
is what mastering a spell should do.

**A2. Concentration wards.** Add `{ effect = "ward", curve = [[0, 0], [100, 10], [200, 20]] }` to
Concentration; `Actor::miscast_ward` already sums `skill_effect(Effect::Ward)` over skills. A
wizard at Spellcraft 100 and Concentration 100 takes 20 off, a priest who put rolls in
Concentration gets a second lever, and Concentration gains a use beside mana. Numbers: a near miss
(by ≤ 3) with ward 20 never passes Mana leak; the floor is 60 % Glamour.

**A3 (optional). A Spellcraft affix** ("of the Weaver", amulet or ring, +10/+20 Spellcraft, rare)
so the "items" in the owner's comment exist. It also helps learning nothing (Decipher script's)
and the roll nothing, so it is a pure safety item: a good candidate for a shop's shelf.

### 6.2 Direction B: retune the dice

For the record, the curve the owner would need for "half the rolls → 5–15 % on own-level
spells": `cast = [[0,0],[20,2],[40,7],[60,12],[100,20],[150,30],[200,40]]`. Effects: the starting
targets unchanged (the 20 → 2 point stays); the half-share wizard at level 10 has Spellcraft 57
→ +11 instead of +8 → bonus +13 → level-10 spells 4 % (was 15 %); the all-share wizard is at the
floor from level 7. It narrows the window in §4.1 rather than widening it. I do not recommend it
without A.

Alternatively `per_level = 1` could become a step table (`+1 a level to 10, +1 per 2 levels
beyond`) so level-30 spells sit at difficulty 28: a level-36 wizard at Spellcraft 100 casts
Sunfire at 14.6 % instead of 80.6 %. That mostly says high spells are safer than the owner asked
("Spells far above one's skill miscast often"); it belongs to a later balance pass if A proves
too slow in play.

### 6.3 Direction C: the table

**C1. The margin's weight: `per_margin = 5`.** A miss by 3 shifts the window a quarter of the
die; by 6, half; by 13, past the plain die. Deadly entries then need by ≥ 9 (Rift) and ≥ 14
(Unravelling) on a plain 66, about as now, but the mild zone empties as the margin grows, so
"higher totals are worse" becomes true of ordinary casters too. (With 5 the explosion's +66 is
unchanged.)

**C2. Order by what the effect does, and fill the flat zone with distinct entries.** A proposed
table, 24 entries tiling 1 up; `new` marks entries that need no new effect kind unless said.
Bands as a player would feel them.

| Range | Id | Effect | Band | Note |
|---|---|---|---|---|
| 1–6 | glamour | nothing | nothing | as now |
| 7–11 | sparks | 1d2 fire | mild | as now |
| 12–16 | refund | the mana back | boon | as now |
| 17–21 | dizzy | Staggered 1 | mild | as now |
| 22–26 | report | a thunderclap (loudness 100) at the caster: the level hears it, sleepers wake | chaos | new; `sound_at`, the dev `thunderclap` sound exists; one effect kind `noise` |
| 27–31 | dread | Frightened 3 | mild | moved down (18 s) |
| 32–36 | mana_leak | 25 % of mana left | mild | as now |
| 37–41 | echo | the spell again, free | boon | as now |
| 42–46 | gloom | a dark zone, radius 2, ten turns, at the caster | chaos | new; the zone machinery (`zone:dark`) |
| 47–51 | boon | Blessed 2 on the caster or someone near | boon/chaos | as now |
| 52–56 | scorch | 2d4 of the spell's own type | mild/bad | moved up |
| 57–61 | swap | places with someone near | chaos | as now; a creature's miscast never swaps with the player |
| 62–66 | quickened | Hasted 1 on the caster | boon | new; `hasted` exists |
| 67–72 | surge | a damage spell of level ≤ the cast's + 6, as a burst at **the cast's target** (or someone near when it had none), never reaching the caster's cell | chaos | changed, §4.5; the burst's cells exclude the caster |
| 73–78 | ill_wished | Ill-wished 2 (the study's "ill omen" curse) | bad | reused from `mishaps/` |
| 79–85 | mana_drain | 75 % of mana left | bad | as now |
| 86–91 | displaced | teleported on the level | chaos | as now |
| 92–98 | backlash | the spell at base roll on the caster; **fallback** for a spell without damage: `(level ÷ 3 + 1)d6` force | bad | changed, §4.6 |
| 99–104 | tongue_tied | Silenced 2 (no casting while it lasts) | severe | new; `silenced` exists |
| 105–111 | withering | drain 1 point, Willpower vs 16 | severe | as now |
| 112–119 | rift | a dretch | severe | as now |
| 120–127 | rebound | the spell at the cast's full roll on the caster; a heal or buff rebounds onto the nearest hostile; a summon's creature comes hostile; fallback `(level ÷ 2 + 1)d6` force | deadly | changed |
| 128–137 | gate | a vrock | deadly | as now |
| 138+ | unravelling | 8d10 force | deadly | as now |

Senseless goes (it is Dizzy). Entries not taken but available if the owner wants more variety:
Drowsy 2, Sluggish 2 (the study's `swimming_head`), Weakened 1 (`backwash`), a smoke or slick
zone, a stat mark lost (`forgetting`), a stat mark gained (`hard_lesson`: a boon), one carried
unknown item identified (a boon; the identify machinery), the caster's light put out (`lit` off).
The "aged a year" of other roguelikes needs a rule for age and is left out.

Odds of the proposed table with `per_margin = 5`, given a miscast (the same enumeration as §3):
the new wizard casting Spark (6.9 % of casts miscast): nothing 1 %, boons (refund, echo, boon,
quickened) 28 %, mild 33 %, chaos (report, gloom, swap, surge, displaced) 31 %, bad 6 %, severe or
deadly 1 %. The same wizard forcing a Fireball (68.8 %): boons 21 %, mild 20 %, chaos 30 %, bad
20 %, severe 7 %, deadly 2 %. A miss by 9: bad 30 %, severe 18 %, deadly 1 %; by 15: chaos 9 %,
bad 26 %, severe 32 %, deadly 33 %. The master's floor (ward 20): nothing 39 %, mild 30 %, boons
15 %, chaos 14 %, nothing worse than Mana leak but through an explosion.

**C3. Surge, bounded** (also alone, as D): the pool is damage spells of level ≤ the cast's level +
6; the burst is centred on the cast's own target (a creature's cast: the actor it cast at; a
player's self-cast or aimless cast: someone near as now); the caster's own cell is left out of the
burst. A new wizard's Spark surge then draws among Burning hands … Blight (levels 1–7, 2d4–4d6);
Sunfire needs a level-24 cast. Allies near the target still take it: chaos, not murder.

**C4. Backfire's fallback** (also alone, as D): a spell with no damage and no hex backfires as
`(level ÷ 3 + 1)d6` force at base, `(level ÷ 2 + 1)d6` at full (Cure wounds 3: 2d6 / 2d6;
Benison 12: 5d6 / 7d6; Haste 11: 4d6 / 6d6). A heal rebounds as the heal on the nearest hostile
in sight instead, when there is one; a buff as the buff on them.

### 6.4 Creatures

- A creature's Swap never picks the player (§4.8); Echo and summons stay.
- The `seen` lines stay; since they are rare, each may be a little longer and more vivid.

### 6.5 Words

A set that keeps "the weave" but varies the opening and borrows the world's voice sparingly:

| Id | Player line | Seen |
|---|---|---|
| glamour | "The weave slips: motes of colour drift from your hands and fade." | "Motes of colour drift from {who}'s hands." |
| sparks | "Sparks leap from the weave and bite your fingers." | "Sparks bite {who}'s fingers." |
| refund | "The weave slackens, and the spent power runs back into you." | "{who} glows for a moment, and brightens." |
| dizzy | "The weave lurches, and the world with it." | "{who} reels." |
| report | "The weave snaps shut with a crack like thunder." | "{who}'s spell ends in a crack like thunder." |
| dread | "Something cold looks back through the torn weave." | "{who} cowers from something unseen." |
| mana_leak | "The weave frays, and your power bleeds out through it." | "{who}'s light gutters." |
| echo | "The weave rings like a struck bell, and the spell sounds twice." | "{who}'s spell sounds twice!" |
| gloom | "The weave drinks the light, and dark pools about you." | "Dark pools about {who}." |
| boon | "Stray power settles on someone as a blessing." | "A stray blessing settles near {who}." |
| scorch | "The weave snaps back and scores you." | "{who}'s own spell scores it." |
| swap | "The weave knots, and you are standing where another stood." | "{who} is suddenly elsewhere, and another stands in its place." |
| quickened | "The weave hums through your limbs, and they quicken." | "{who} quickens." |
| surge | "Wild power bursts from the weave!" | "Wild power bursts from {who}'s hands!" |
| ill_wished | "The broken weave leaves an ill wish on you." | "An ill wish settles on {who}." |
| mana_drain | "The weave gapes, and your power pours out." | "{who} dims and sags." |
| displaced | "The weave folds the world, and you with it." | "{who} is flung through a fold in the air." |
| backlash | "The weave turns, and the spell breaks over your own head." | "{who}'s spell breaks over its own head." |
| tongue_tied | "The weave seizes your tongue; no word of power will come." | "{who} mouths words and nothing comes." |
| withering | "The weave feeds on you, and something of you goes with it." | "{who} withers." |
| rift | "The weave tears open a rift, and something crawls through." | "A rift tears open beside {who}, and something crawls through." |
| rebound | "The weave rebounds, and the spell's full force falls on you." | "{who}'s spell falls on it with its full force." |
| gate | "The weave rips a gate wide, and something terrible steps through." | "A gate rips open beside {who}, and something terrible steps through." |
| unravelling | "The weave unravels through you, and the raw power burns you from within." | "{who} burns from within." |

### 6.6 Direction D: the minimum

Senseless → Drowsy 2 or dropped (its range given to its neighbours); C3 (Surge bounded); C4
(Backfire fallback); the creature Swap exclusion; the words. No curve change, no new entries.

### 6.7 Docs and checks any direction touches

Spec §8 "Spells": "Miscasts"; `docs/dev/spells.md` "Miscasts"; `docs/dev/skills.md` (Concentration
under A2); the wiki `miscasts.md` (the table regenerates; a sentence on ranks, stances, Scribe, and
the half-casters' odds) and `skills.md`; `docs/balance.md` "Miscasts" rerun; core tests for each
changed entry kind, the rank ease, the surge bound (no cell of the caster's, no spell above the
cap), the fallback harm; the content test for tiling. The probe should spend the wizard's rolls on
Spellcraft (or report both Concentration-first and Spellcraft-first) so its Miscasts table shows
a career that plays the system.

## 7. Open questions for the owner

1. **Flat or ordered?** Is a near miss meant to land anywhere on the lower three quarters of the
   table with equal odds (as now: pure chaos), or should the margin visibly order the outcome
   (C1)? The owner's "higher totals give greater effects" reads as the second.
2. **Ranks as the lever (A1)**: yes or no? It is the one proposal that gives the player something
   to do about miscasts between level-ups.
3. **Concentration (A2)**: should the skill ward miscasts, or stay mana-only?
4. **Half-casters at 1 in 3 to 1 in 7 for their borrowed spells**: intended?
5. **Surge** (C3): is a surge that can reach the caster wanted chaos, or the unearned death I
   read it as?
6. **Healers' backfires** (C4): a rebounding heal on the nearest hostile, or plain harm?
7. **The words** (6.5): keep the "Your weave …" litany for consistency, or vary it?
