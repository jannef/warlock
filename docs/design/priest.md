# Warlock — The priest: one prayer, answered as far as it reaches

**Built; this file is history.** Current behaviour is in spec §8 "The casters' class tables, and their spells" and `docs/dev/prayer.md`.

Design document for the owner's review (2026-11-02, #820; round 2, after the owner's answers on
PR #831, recorded in §13). Written against `origin/main` at c9201000 (the druid design). Nothing
here is implemented; §11 cuts it into issues. It builds on what is there: the class tables and the
coverage policy (`docs/dev/progression.md`, rules 1–9 of #776), the class actives and the holy
gifts' bands (`docs/dev/alignment.md`, `world/class_ability.rs`, `world/consecrate.rs`), the spell
data and the one cast path (`docs/dev/spells.md`), statuses, diseases, and treating oneself
(`docs/dev/status-effects.md`), drain and stat damage (`docs/dev/stats.md` "Drain"), hunger
(`docs/dev/hunger.md`), mana (`docs/dev/mana.md`), the resources (`docs/dev/resources.md`), the
temple's services (`world/service.rs`), special rooms and the forgotten altar's prayer
(`docs/dev/special-rooms.md`, `world/pray.rs`), and the gaps report (draft PR #802). Every number
is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki
is accurate).

**The owner's request** (#820, verbatim): "lean into it being generally pretty shit, but having
tools to heal/cure/get rid of poison/create food. Kinda having the tools to overcome all the
traditional rogue like housekeeping issues, but not any extra ability to fight." **The new
direction** (PR #831, 2026-11-02, §13): one prayer, given early and spammable as ADOM's, that fixes
the priest's worst problem from a list that grows with their level, under a hidden strain that
makes it fizzle when overused; grander names as it grows, two answers a prayer later, and at 60
**Miracle**, everything at once for one cost; evil versions of the gifts, not twins, and better if
they turn out better.

## Contents

1. Summary
2. Research, tied to the decisions
3. The housekeeping of Warlock: what bites, and what answers it today
4. What exists: the priest's table and class file
5. The proposal: the table, level by level
6. The prayer, the dark faces, Communion, and Sanctuary, as data
7. Determinism, saves, replays
8. What the player sees: the menu, messages, dumps, the wiki
9. Talents and class abilities (the coverage policy)
10. Rejected ideas
11. Implementation plan (slices)
12. Open questions, each with a recommended default
13. The owner's decisions (2026-11-02, round 1)

## 1. Summary

| Piece | Proposal |
|---|---|
| **The fantasy** | A poor fighter by numbers (Strength −1, Dexterity −1, 4 hit points a level, a mace and a buckler) with **one prayer** from level 1 that answers whatever ails them most: wounds first, then as they grow hunger, disease and poison, hexes and holds, drain and stat damage, mana; two answers a prayer from 40; at 60 **Miracle**, everything at once. The prayer costs no mana, takes a cast's time, and is limited only by a hidden **strain** that decays with time: pray too often and nothing answers for a while. §5, §6.1 |
| **Who answers** | The priest's alignment band at the moment: Neutral or better, the holy powers (the **holy face**: Orison → Prayer → Litany → Benediction → Miracle); Callous or worse, the dark ones (the **dark face**: Whisper → Invocation → Rite → Pact → Miracle). The dark face answers the same problems with its own twists, some of them better: hunger filled to the brim, a disease or poison cured *and passed to the nearest creature in sight*, a hex ended *and turned back on its caster*. One ability, one strain, two faces. §6.1, §6.2 |
| **Stays** | Mend (1), Blessing (3, the owner: in), Cure wounds (3): the mana economy beside the strain's. Consecrate (9, the paladin's shared gift, `min = kind`). The kit, stats, skills, and weapon ranks (q6). §4 |
| **Goes** | Smite (40: #823's), Purity of body (32: q10 "no"), Saint (60: Miracle takes its place, q7), the reprieve named Miracle (50: its name is the capstone's; Hag's bargain stays the witch's), Remove disease (12) and Purify (25) as entries of their own: both fold into the prayer's unlocks. §4 |
| **New** | The prayer's seven uses and five tiers (§6.1); **Communion with the dark** (9, the dark face's gift beside Consecrate: in an unholy place a prayer adds no strain and the place's drain does not touch you); **Sanctuary** (50: hallowed, or dreaded, ground for an hour: no foe steps within two paces, wounds and mind mend as in a sanctuary room, broken by the priest's own blow); **Miracle** (60). §6 |
| **The table** | 1 Mend, **Orison** · 3 Blessing, Cure wounds · 6 **Daily bread** (hunger) · 9 Consecrate / **Communion with the dark** · 12 **Prayer** (disease, poison) · 18 **Prayer of release** (hexes, holds) · 25 **Litany** (drain, stat damage) · 32 **Litany of the mind** (mana) · 40 **Benediction** (two answers) · 50 **Sanctuary** · 60 **Miracle**. Eight of the thirteen rows are the prayer. §5 |
| **The strain** | Hidden. +100 per attempt, answered or not; a point gone a minute (100 in 100 minutes); the chance a prayer fizzles is the strain over 10 in percent (100 → 10 %, 500 → 50 %, 1000 → certain); a fizzle adds its 100 and **locks** the prayer until the strain has decayed to 0. All data in `assets/data/prayer.toml`; the wiki states it. §6.1 |
| **Data kinds** | one talent effect `prayer { tier, title, uses, answers, all }`, one `communion`, one `sanctuary { mana, time, radius, lasts, regen }`; `data/prayer.toml` (the strain, the faces' bands, the priority order, the words); `Progress::prayer` (the strain) and `WorldState::hallows`. One read each. §6, §7 |
| **Talents** | **None new, no new system**: the prayer is a class ability whose uses extend systems that have their talents (`hit_points`, `status`, `diseases`, `mana`, `hunger`, `resources`, `special_rooms`); the strain is the ability's own time-refilled limit, as the policy's step 4 wants (no cooldown status, time is the price). §9 |
| **Slices** | P1 the prayer's holy face, the strain, the table and the cuts; P2 the dark face and Communion; P3 Sanctuary; P4 the wiki, the spec, the probe; beside them a small issue for the scroll of restoration (q9). §11 |

**Key choices, and why**

1. *One ability that grows, not a list of spells.* The owner's direction: the prayer is given at 1 and gains uses, a second answer, and a grander name as the priest levels, so the table reads as one thing deepening. The previous round's Daily bread, Balm, and Restoration are now **uses** of the prayer (§5); Mend, Blessing, and Cure wounds stay as spells, because a mana heal beside a strain-limited one is two economies, as ADOM's priest has spells beside prayer.
2. *ADOM's shape, NetHack's counter.* The prayer fixes the worst problem first from a fixed priority order (ADOM, §2), and its limit is a hidden number that a prayer raises and time lowers (NetHack's prayer timeout, verified in §2: 300 to start, reset by a prayer, down one a turn, unsafe above a threshold). The owner's fizzle rule (a chance by the number, a lock after a fizzle) is the Warlock turn on it.
3. *The band picks the face, never the gift.* Alignment bars nothing (`docs/dev/alignment.md`): a priest of any band has the one prayer and is always answered by someone. The dark face is not a mirror: its answers carry twists that only a dark power would grant (passing a plague on, turning a hex back, feeding to the brim), and where that is simply better, so be it (the owner, q3).
4. *Hidden numbers stay hidden.* No message names the strain, its lock, or a chance; a locked prayer fizzles in the same words as an unlucky one, as ADOM's unheard prayer does. The wiki states every number.
5. *Weak in a fight, still.* Nothing here adds a blow, an attack spell, or a defence; Blessing's +1 to hit stays because the owner called it harmless (q1). The dark twists hurt a creature only as a side of a cure.

## 2. Research, tied to the decisions

| Source | What it does | Taken | Left, and why |
|---|---|---|---|
| **ADOM's prayer** (the ADOM Guidebook, adomgb.info §0.2, fetched; the ancardia wiki unreachable: HTTP 402, then a JavaScript wall) | Verified from the Guidebook: piety "constantly decreases with time"; a prayer that finds nothing to fix "remains unheard" and costs little; "repeatedly praying will anger a deity" through a ladder of punishments; being Doomed "makes it much more difficult to pray for divine help"; crowning lets one "pray more often". Recalled, unverified: a heard prayer fixes the PC's worst trouble in a fixed order (death's door first, then starvation, sickness, poison, blindness, confusion, paralysis, a cursed item, hunger), and a hidden prayer timeout makes a second prayer soon after the first unsafe | the **spammable prayer** that fixes one problem in priority order (§6.1); the **unheard prayer is free** (a prayer with nothing to answer is refused with no time and no strain); **time lowers the counter** (the strain decays a point a minute) | piety by sacrifice (Warlock has no sacrifice, and the forgotten altar's prayer is everyone's: `data/outcomes/forgotten_altar.toml`); anger as a ladder of punishments (the fizzle and lock are the whole cost: time is the penalty) |
| **NetHack's prayer timeout** (nethackwiki.com/wiki/Prayer, /Prayer_timeout, fetched) | a hidden counter: 300 at the start, down one a turn; a successful prayer resets it to between 50 and 1000 (rnz(350), sometimes far more); praying is safe with major trouble under 201, with minor trouble under 101, with none only at 0; too high: "your god's anger is incremented by one, you lose 3 luck, and your god smites you"; a prayer fixes all major troubles first, then minor | the **one hidden counter a prayer raises and time lowers** (§6.1); **major before minor** became the owner's priority order; the thresholds became a chance by the number (the owner's q7) | the reset to a random value (the owner's +100 is flat and known to the wiki); luck and anger |
| **ADOM's Healer and Priest** (recalled) | the Priest starts with Healing and leans on prayer; the Healer is the housekeeping class: Healing, faster regeneration, powers that neutralise poison and cure sickness at fixed levels | the Heal skill from level 1 (the priest's `heal = 2d6+10`, there already); fixed-level unlocks (§5) | — |
| **DCSS's Elyvilon and Zin** (crawl-ref `ability.txt`, fetched for round 1) | Purification "purges your body of poison, confusion, slowness, petrification, weakness, and sickness" and restores drained maximum health; Zin's **Sanctuary** "transforms the area around the user into a domain of unbreakable law" (monsters do not enter; attacking from inside ends it) | the release use's list (§6.1: bleeding, poison, every hex, holds) and the restoration use; **Sanctuary at 50** (§6.4) | pacifying, Recite, Imprison (fight tools); the tithe |
| **DCSS's dark gods** (Kikubaaqudgha, Makhleb, Yredelemnul; recalled) | the dark powers give by taking: Makhleb heals on kills, Yred drains life, Kiku delivers corpses and turns pain back | the **dark face's twists**: a cure that passes the plague on, a hex turned on its caster, a feast rather than a wafer (§6.2) | heal-on-kill (a fight economy: the owner's priest fights no better) |
| **NetHack's Priest** (nethackwiki, fetched for round 1) | a mace and a small shield; sees beatitude; `#turn` undead | the mace-and-buckler start (kept); nothing of curses (Warlock has none) | turn undead (a fight ability) |
| **Warlock's own temple** (`campaigns/ascmearc/data/dialogue/ceaptun.toml`) | Sister Hild's `cure` (60 gold: every disease and poison) and `mend` (40 gold: all stat damage, every drained point) | the prayer's disease, poison, and restoration uses call the same two functions (`World::cure`'s ends, `World::mend`); the priest carries the temple | the prices stay: the non-priest's way |

## 3. The housekeeping of Warlock: what bites, and what answers it today

Read from the data on `main`. The right column is the prayer's use and the level it comes.

| Problem | How bad it is | Answers today | The prayer's answer (level) |
|---|---|---|---|
| **Wounds** | 1 hit point back per max(60, 360 − 12 × Toughness) s (Toughness 10: 4 minutes), × 2 resting, × 3 in a sanctuary; none while Starving | rest; potions of healing (2d8, level 1); Second wind; the altar's boons | **wounds** (1): bleeding stops, hit points to full. Mend (1) and Cure wounds (3) for mana beside it |
| **Bleeding** | 1 hp per 40 s a degree, up to 5, eases a degree a minute | the Heal skill (a check, 2.5 turns), bandages | **wounds** (1) |
| **Hunger** | a human burns 1440 a day, starts at 3600 (Hungry after a day and a half); Weak takes Strength and Dexterity; Starving takes 1 hp per 10 min and stops healing; food rots | buy food, forage, eat corpses, the altar's "sated" | **hunger** (6, Daily bread): fed to Satiated; the dark face to the brim. Rotten food harmless (Purify's rule rides on it) |
| **Disease** | 9 diseases, 26 creature types carry one, rotten food carries grave rot, marsh ague from wading; a failed check a day takes 1d3 to 1d6 off a stat; Diseased halves healing and mana | angelica root, potion of cure disease (150 gold), the temple's cure (60 gold), the Heal skill's try | **disease** (12): every disease gone, the showing and the brewing alike |
| **Poison** | 1 hp per 2 min a degree, up to 10; Strength and Dexterity −1 from 4 degrees; healing and mana at half pace | potion of antidote, the Heal skill once a dose, Divine grace, Nature's ward | **poison** (12) |
| **Hexes and holds** | Hexed, Weakened (St and Dx −2), Sluggish (×1.5 time), Drowsy (×3), Webbed and Held (no step), Frightened, Transfixed, Ashen (an altar's bane: −2 to every stat a day) | wait them out; a Strength check against a hold | **release** (18) |
| **Stat damage** | a disease's; mends a point a day once nothing shows | the temple's mend (40 gold) | **restoration** (25) |
| **Drain** | **for good**: 36 of 295 creature types drain a stat on a hit against a check of 16 to 24; the score falls to a floor of 3, hit points and mana with it | the temple's mend (the only restore); Unbroken soul, Divine grace, Iron will | **restoration** (25) |
| **Mana** | regenerates a point per max(60, 360 − 12 × Willpower) s; a rest to full takes about 3 hours at level 12, 7 at 30 (`docs/dev/spells.md` "Balance") | rest, Meditation, Clear mind | **mana** (32): to full |
| **Cursed places** | cursed rooms raise the dead you leave, haunted rooms bring undead every hour of a rest, draining rooms (and whole levels) take a point of mana every 20 s | Consecrate (priest 9, paladin 12) | Consecrate (9, holy); **Communion with the dark** (9, dark): the place is yours to use |
| **The long rest** | a floor with a danger spawns a creature every 15 min at 60 %; a rest stops when one comes into view | a sanctuary room (shunned, × 3), Quiet delver, Consecrate on a haunt | **Sanctuary** (50) |
| **Everything at once** | the deep floors: drained, poisoned, bleeding, hungry, and out of mana after one fight | several of the above, one by one | **Miracle** (60): every use at once, and every pool (stamina, whatever else is held) to full, for one attempt's strain |
| **Light** | every race without darkvision starts with a lantern; Light is a level-0 spell the kit's scroll teaches for certain at level 1 | the lantern, the scroll | nothing added (§10) |
| **Identification** | unknown scrolls, potions, wands, magic items | the Lore talents, the scroll and spell of identify, the appraiser | nothing: the merchant's (the owner) |

Not in Warlock, so not here: cursed items, blindness, confusion, lycanthropy, level drain, item
destruction, lamp oil.

## 4. What exists: the priest's table and class file

`assets/data/classes/03-priest.toml`: Strength −1, Dexterity −1, Intelligence +1, Willpower 1d3+1,
Presence 1d2; `hp = 4`, `mana = 9`, `casting = 100`; affinity restoration +3, abjuration +2,
divination +1, necromancy −2; skills Heal 2d6+10, Knowledge 2d6+10, Concentration 2d4+4, Decipher
script 2d4+4; weapon skills maces 2, shield block 1, restoration 1; styles sword and board 1,
toucher 1; the kit a mace, a buckler, a wool cloak, an amber amulet, a scroll of light, three
candles, bread, 10 gold; bonus talents from Vigour and Grit at 10, 20, 30, 45, 55; no `alignment`
of its own, so a priest starts at their race's score (a human's 0, an elf's 50: the Neutral band;
a drow's −450: Evil) nudged by the birth animal.

| Level | Today | Effect (data) | Fate |
|---:|---|---|---|
| 1 | Mend | spell `mend`: touch heal 1d3 (Willpower 16: 1d4, 22: 1d6), 1 mana | **stays** |
| 3 | Blessing | spell `blessing`: Blessed 10 min a degree (+1 to hit, Willpower +1), 3 mana | **stays** (q1: "in") |
| 3 | Cure wounds | spell `cure_wounds`: touch heal 2d6 (16: 3d6, 22: 3d6+3), two casts, 2 mana | **stays**: the mana heal beside the prayer's |
| 9 | Consecrate | active, 15 mana, a minute: lifts an unholy room's effect for good, `alignment min = kind`, Consecrated +20 | **stays** (shared with the paladin, untouched; Q3 asks whether its band should be `neutral`) |
| 12 | Remove disease | spell `remove_disease`: cleanse, every disease and Poisoned, 11 mana | **folds** into the prayer's disease and poison uses at 12; the spell file stays for its scroll (`scroll_of_remove_disease` teaches it to anyone) |
| 25 | Purify | `purify`: rotten food neither sickens nor exposes | **folds** into Daily bread's talent at 6 (one hunger talent) |
| 32 | Purity of body | `disease_immune` (the paladin's 18 and the monk's 32) | **goes** from the priest (q10); the others keep it |
| 40 | Smite | a blow of holy wrath (the paladin's 12) | **goes**: a fight ability, and #823's |
| 50 | Miracle | `reprieve` once: a killing blow leaves 1 hp, Worn thin a day | **goes**: the capstone takes the name (q7); the `reprieve` kind stays Hag's bargain's |
| 60 | Saint | `healing_percent` +100 | **goes**: Miracle (60) replaces it (q7) |

## 5. The proposal: the table, level by level

Bonus talents (Vigour, Grit) at 10, 20, 30, 45, 55 unchanged. Skill rolls 2 unchanged. Where a
row has a holy and a dark face, the band at the moment of use picks (§6.1): Neutral and better the
holy, Callous and worse the dark.

| Lv | id | Name (holy / dark) | Kind | What it does (the game's words) | Data |
|---:|---|---|---|---|---|
| 1 | `mend` | Mend | spell | as today | as today |
| 1 | `orison` | **Orison** / Whisper | the prayer, tier 1 | "Pray (an ability; the time of a cast, no mana): what ails you most is answered, as far as your prayers yet reach: for now, your wounds. Pray too often and nothing answers for a while." Use **wounds**: bleeding stops, hit points to full | `{ kind = "prayer", tier = 1, title = { holy = "Orison", dark = "Whisper" }, uses = ["wounds"] }` |
| 3 | `blessing` | Blessing | spell | as today | as today |
| 3 | `cure_wounds` | Cure wounds | spell | as today | as today |
| 6 | `daily_bread` | **Daily bread** | prayer use | "Your prayers feed you: hungry, you are fed. And rotten food neither sickens you nor carries a disease into you." Use **hunger**: Hungry or worse, fed to Satiated (holy) or to the brim, one short of Bloated (dark); Purify's rule on both faces | `[{ kind = "prayer", uses = ["hunger"] }, { kind = "purify" }]` |
| 9 | `consecrate` | Consecrate | active (holy, `min = kind`) | as today | as today |
| 9 | `communion` | **Communion with the dark** | passive (dark, `max = callous`) | "The unholy places are yours: in a cursed, draining, or haunted room, or on a level such a curse lies over, your prayers cost you nothing, and the place's drain does not touch you." | `{ kind = "communion" }` |
| 12 | `prayer_of_cleansing` | **Prayer** / Invocation | prayer, tier 2 | "Your prayers reach further: every disease in you, shown or brewing, is gone, and any poison with it." Uses **disease**, **poison**. Dark: each disease cured is passed to the nearest creature you see; the poison the same, at the degrees it had | `{ kind = "prayer", tier = 2, title = { holy = "Prayer", dark = "Invocation" }, uses = ["disease", "poison"] }` |
| 18 | `prayer_of_release` | **Prayer of release** | prayer use | "Whatever hexes, holds, or webs you lets go." Use **release**: every status in `prayer.toml`'s `release` list ends (Hexed, Weakened, Sluggish, Drowsy, Webbed, Held, Frightened, Transfixed, Ashen). Dark: a hex ended is laid on its caster, if you see them, at the degrees it had; a hold's holder is Webbed as long as it held you | `{ kind = "prayer", uses = ["release"] }` |
| 25 | `litany_of_restoring` | **Litany** / Rite | prayer, tier 3 | "What sickness took from your body is mended, and what the dead drank from you is given back." Use **restoration**: all stat damage mended, every drained point restored (`World::mend`, the temple's) | `{ kind = "prayer", tier = 3, title = { holy = "Litany", dark = "Rite" }, uses = ["restoration"] }` |
| 32 | `litany_of_the_mind` | **Litany of the mind** | prayer use | "Your mind is filled: mana to full." Use **mana** | `{ kind = "prayer", uses = ["mana"] }` |
| 40 | `benediction` | **Benediction** / Pact | prayer, tier 4 | "Two things that ail you are answered by one prayer." `answers = 2` | `{ kind = "prayer", tier = 4, title = { holy = "Benediction", dark = "Pact" }, answers = 2 }` |
| 50 | `sanctuary` | Sanctuary | active (a rite; the words by band) | "Kneel and hallow the ground about you (an ability, paid in mana; a minute's rite): for an hour no foe will set foot within two paces of where you knelt, and your wounds and your mind mend there as in a sanctuary. The peace breaks if you strike from within it." | `{ kind = "sanctuary", mana = 30, time = "1m", radius = 2, lasts = "1h", regen = 300 }` |
| 60 | `miracle` | **Miracle** | prayer, tier 5 | "Everything that ails you is answered at once: wounds, hunger, sickness and poison, hexes and holds, what was taken from you, your mana, and every other pool you hold, filled." `all = true` | `{ kind = "prayer", tier = 5, title = { holy = "Miracle", dark = "Miracle" }, all = true }` |

**The priority order** (which problem a prayer answers when several ail the priest and it answers
fewer than all; the owner's list, ADOM's shape; `prayer.toml`'s `[[use]]` order): **disease,
poison, wounds, release, mana, hunger, restoration**. A use not yet unlocked is skipped; a problem
the priest does not have is skipped; with none left the prayer is refused (no time, no strain).

**Why these levels.**

- *1, wounds*: the first problem, and ADOM's first fix. A level-1 priest who can pray their hit
  points full is strong out of a fight and no better in one: the prayer takes a cast's time, and
  the second prayer in the same fight already risks a fizzle (§6.1). Mend stays for a touch heal
  on someone beside them and Cure wounds for the mana economy.
- *6, hunger* ("daily bread first", q2): hunger comes by day two and has no sure answer but gold.
  Fed to Satiated (4000) from Hungry (under 1440) is a day and three quarters for a human: a wafer
  a day was the old proposal; this is "pretty good" (q4) for one attempt's strain.
- *12, disease and poison*: Remove disease's level, where the first rat bites and marsh wadings
  have shown their fevers. The brewing diseases go too (the priest is told only of the shown ones,
  so nothing hidden leaks: §8).
- *18, release*: the gap level. Hexes and holds have nothing but waiting; a sure, one-turn end to
  a web or a hold in a fight is the one answer the table gives that matters mid-fight, and it
  comes late for its power, which is the shape of a class whose owner wants it "pretty shit".
- *25, restoration*: drain is the housekeeping problem Warlock makes worst (36 types, for good, a
  floor of 3) and answers least (one sister in one town). Earlier would make the barrows' wights
  toothless to a priest of 12.
- *32, mana*: a prayer that fills the pool makes the priest's spells near free, limited by the
  strain alone. At 32 the pool is about 230 (interpolating the probe's 216 at 30): one attempt's
  strain for three hours of regeneration.
- *40, two answers* (rule 1: a capstone changes what the character can do in a dungeon): the
  priest who comes out of a fight bleeding and poisoned prays once.
- *50, Sanctuary*: the rest is the priest's whole economy (mana and strain both come back with
  time), and the deep floors make resting the dangerous part.
- *60, Miracle*: the owner's: "pretty busted". Everything, every pool, one attempt.
- *3 and 9 kept off-cadence*: the report counts them as deliberate.

**What the priest cannot do**, so the owner sees the shape: no attack spell of their own, no blow
(Smite gone), no summon (the druid's, #821), no stealth, no pet, no voice. In a fight they have a
mace at rank 2, a buckler, 4 hit points a level, Blessing's +1, and a prayer that takes a turn and
may fizzle.

## 6. The prayer, the dark faces, Communion, and Sanctuary, as data

### 6.1 The prayer

One class active, `ClassActive::Orison`, id `orison` (`use:orison` in action text at every tier:
the id never changes, the name does), carried by the tier-1 talent; the later prayer talents add
uses, answers, and titles to it through `TalentMods::prayer` (`Prayer { uses: BTreeSet<Use>,
answers: u8, all: bool, title: Option<(String, String)> }`: `uses` the union, `answers` the
largest, `all` any, `title` the highest `tier`'s). The abilities menu's Class group shows the
title of the face in force (§8).

```toml
# assets/data/prayer.toml — the priest's prayer (spec §8 "Class abilities"): the strain, the
# faces, the uses in their priority order, and the words. Every number is hidden in the game and
# stated in the wiki.
strain = 100           # added by every attempt, answered or not (1 to 1000)
decay = "1m"           # a point of strain gone every this long (10 s to 1 h)
fizzle_per_100 = 10    # percent chance a prayer fizzles per 100 points of strain (1 to 100)
holy = { min = "neutral" }   # the bands the holy powers answer
dark = { max = "callous" }   # the bands the dark powers answer
release = ["ill_wished", "weakened", "sluggish", "drowsy", "webbed", "held", "frightened", "transfixed", "ashen"]

[words]
holy = { pray = "You kneel and pray.", fizzle = "You pray, but nothing answers.", nothing = "You have nothing to pray for." }
dark = { pray = "You whisper to the dark.", fizzle = "The dark is silent.", nothing = "You have nothing to ask of the dark." }

[[use]]           # the order is the priority: the first that ails the priest is answered first
id = "disease"
holy = "The sickness leaves you."
dark = "The sickness leaves you, and finds another."
[[use]]
id = "poison"
holy = "The poison leaves you."
dark = "The poison leaves you, and finds another."
[[use]]
id = "wounds"
holy = "Your wounds close."
dark = "Your wounds close, cold to the touch."
[[use]]
id = "release"
holy = "You are released."
dark = "You are released, and it is returned."
[[use]]
id = "mana"
holy = "Your mind is filled."
dark = "Your mind is filled with a cold clarity."
[[use]]
id = "hunger"
holy = "You feel fed."
dark = "You feel glutted."
[[use]]
id = "restoration"
holy = "You feel whole again."
dark = "What was taken is taken back."
```

| Part | Rule |
|---|---|
| The uses | a closed list in code (`prayer::Use`: Disease, Poison, Wounds, Release, Mana, Hunger, Restoration); the file names each once and gives the order and the words (a use missing or twice is a `ContentError`); `release` names statuses (checked against `status.toml`; none may be a hold's holder-side status, a buff, or Diseased). Without the file the prayer is refused ("You have nothing to pray for."), so fixtures load |
| The problem of each use | disease: a disease **showing** (`Infection::shown`; a brewing one alone is no problem, so the prayer never tells the player of one: once answered, every infection goes, brewing ones too, silently); poison: Poisoned; wounds: Bleeding or `hp < max_hp`; release: any status of the list; mana: `mana < max_mana`; hunger: Hungry or worse (`Hunger < NotHungry`); restoration: any stat damage or any drained point |
| The face | `World::prayer_face()`: the player's band (`Actor::band`) within `holy` → Holy, within `dark` → Dark; without alignment rules, Holy. A band in neither (none with these defaults: `neutral` down to `callous` leaves no gap) → Holy. The two `Requirement`s are checked at load to leave no band unanswered |
| The time and the block | `Spent::Turn(price_of(player, ActionKind::Cast))` (`cast = 1000`: 10 s at normal speed, a spell's). `kind_of(Use(Orison))` is `Cast`, so Raging and Silenced block it (a silenced priest cannot pray aloud), as they block a spell |
| The refusals | no time, no strain: no prayer talent; no `prayer.toml`; no problem among the unlocked uses (the face's `nothing`) |
| The strain | `Progress::prayer: Option<Strain { value: u16, at: GameTime, locked: bool }>`; `Strain::now(rules, time) = value.saturating_sub(elapsed(at, time) / decay)` (lazy: no moment in `World::live`, no state to tick; reading never writes); at 0 the lock is off. An attempt reads `now`, then writes `value = now + strain, at = time` |
| The attempt | locked → fizzle. Else `chance = min(100, now × fizzle_per_100 / 100)`; `chance > 0` → **one gameplay draw**, `below(100) < chance` → fizzle; `chance = 0` (no strain) → no draw, answered. A fizzle: the face's `fizzle` line, `locked = true`, the strain added. Answered: the face's `pray` line, the strain added, then the first `answers` problems in priority order (`all`: every problem, and every resource pool to its maximum: `Pools`' values, through `World::grant_resource`'s cap), each through its one existing place and each said in the face's words |
| The fixes | disease → `World::end_disease` for every infection (the temple's `cure`); poison → `World::ended(poisoned)`; wounds → `ended(bleeding)`, `hp = max_hp` (not `heal_player`: no cause, no cap but the maximum); release → `ended` for each (a hold ends as `loosen_holds` ends one); mana → `mana = max_mana`; hunger → `Satiation::feed` to `states.satiated` (holy) or `states.bloated − 1` (dark), never less than it has; restoration → `World::mend(player)` (`StatDamage::mend_all`, `restore_drained`). Nothing here draws but the one fizzle die |
| Communion | `TalentMods::communion` (the dark 9, §6.3): the strain is **not** added when the player stands where `effects_at` says an unholy effect lies (a room's or the level's) and the face is Dark (the talent's `alignment = { max = "callous" }`, read as the holy gifts read theirs: outside the bands it does nothing) |
| Marks | none: the prayer is no cast (`train_spell` is the cast path's) and no stat use; the Willpower mark would make a hidden pile out of a free ability (rule 7's spirit) |
| Alignment | none: praying moves no score (`prayed_good`/`prayed_evil` are the altar's) |

Numbers against the pool: with `decay = "1m"` and `fizzle_per_100 = 10`, a prayer an hour and
forty minutes apart never risks a fizzle; two twenty minutes apart risk 8 % at the second; a
third ten minutes later 17 %; a fizzle at 270 strain locks the prayer for 370 minutes, about six
hours. ADOM's unheard prayer and NetHack's 300-turn timeout are the scale. Miracle at 60 under
the same numbers is the owner's "pretty busted": everything, every hour and forty minutes, free.

### 6.2 The dark face's twists

The dark powers answer the same problems and add, each a line in the use's fix, no draw:

| Use | The twist | Through |
|---|---|---|
| disease | each disease cured is passed to the **nearest creature the priest sees** (the least `dist2`, the lowest `ActorId` on a tie; none seen: nothing more): it incubates in them as if they had lost the check | `World::infect` (the dev `infect`'s path; no check) |
| poison | the degrees cured are laid on that creature | `World::afflict(target, poisoned, degrees, player)`: immunity and resistance apply as to any dose |
| release | each hex ended whose `by` the priest sees is laid on that `by` at the degrees ended; a hold's holder is Webbed for the degrees the hold had | `afflict` (a creature type's `immune` list refuses; a buff status never: the list is checked at load to hold no buff) |
| hunger | fed to one short of Bloated, not to Satiated | `Satiation::feed`'s target |
| wounds, mana, restoration | the words alone | — |

A kill by a passed-on poison is a death by a status the player caused: `dies` judges it as any
kill of theirs (`judge_kill`, XP as today). Nothing here is a blow: the twists ride on cures.

### 6.3 Communion with the dark (9)

```toml
# assets/data/talents/communion.toml
[[talent]]
id = "communion"
name = "Communion with the dark"
tree = "class"
granted = true
description = "The unholy places are yours: in a cursed, draining, or haunted room, or on a level such a curse lies over, your prayers cost you nothing, and the place's drain does not touch you."
effects = [{ kind = "communion" }]
systems = ["special_rooms"]
alignment = { max = "callous" }
```

Two reads: `World::orison` (no strain, §6.1) and the one place a room effect's `status` is laid
on the player (the draining room's Ebbing: skipped for a holder of the Dark face standing in an
unholy place). A holder of the Holy face (a priest risen to Neutral) has it and it does nothing,
as a holy gift outside its bands does. `TalentDef::alignment`'s load check widens from "a holy
active's talent" to "a holy active's or a band-gated passive's" (`communion` the first), the
reading of `docs/dev/alignment.md`: "would work the same way, by band at the moment of use".

### 6.4 Sanctuary (50)

As round 1 designed it (the owner: q3 "create evil versions", q8 "sure", no light), moved from 40
to 50 and given a dark face in words alone: the rule is the same for both, since the owner allows
twins and a safe rest is a safe rest.

A class active like Consecrate (`TalentEffect::Sanctuary { mana, time, radius, lasts, regen }` →
`ClassActive::Sanctuary`, `world/sanctuary.rs`; `use:sanctuary`; the Class group).

| Part | Rule |
|---|---|
| The rite | `begin_sanctuary`: refused (no time, no mana) on the overworld, in water, inside a shop, or where a hallow already stands within `radius`; else `spend_mana` and `Spent::Sanctuary(until)`, kept at through `keep_at` as Consecrate is (a creature coming into view, a message, or a blow interrupts: "You rise from your knees.", the mana spent, nothing laid) |
| The hallow | `WorldState::hallows: Vec<Hallow { level, at, radius, until, regen }>`, laid at the end of the rite on the priest's cell; holy: "The ground about you is hallowed."; dark: "A dread settles on the ground about you." |
| Who keeps out | `World::shuns(id)` (`world/ai.rs`, the one place a hostile keeps out of a sanctuary room) gains one clause: a hostile not already inside treats every cell within `radius` (Chebyshev) as shunned; one inside when it is laid may move within and leave; `spawn.rs` leaves hallowed cells out. Missiles and spells cross it: the hallow keeps feet out |
| The pace | `World::healing_rate` and `World::mana_rate` take `× regen / 100` for an actor standing in a hallow on the current level (the larger of the hallow's and the room's, never both) |
| The break | `World::resolve` (a blow, shot, or throw by the player) and the player's harmful casts call `World::break_hallow()` when the player stands inside: "The peace is broken." A dark twist of the prayer (a plague passed on from inside) breaks it too: it harms. Being hit inside breaks nothing |
| The end | `World::live`'s event loop takes `hallow_until` as a moment and drops hallows past `until` ("The hallowing fades.", a `note`); leaving the level keeps it |
| Light | none (q8) |

Numbers as data: `radius` 1 to 5, `lasts` 10 minutes to 12 hours, `regen` 100 to 1000, `mana` 1
to 500, `time` 10 s to an hour (load checks on the talent effect).

### 6.5 The spells that stay, and the test

Mend (1), Blessing (3), Cure wounds (3) are the priest's guaranteed spells: three. The full
caster's bound in `world/spell_list_tests.rs` (`4..=6`) becomes `3..=6`, and `docs/dev/spells.md`'s
"four to six spells" reads "three to six: the priest's rite is the Orison". The cantrip at 1 (Mend)
holds. `remove_disease.toml` stays a spell file (its scroll teaches it; nothing names it in a
table), as do `smite`, `purity_of_body`, and `purify`'s **effect kind** (`purify` rides on Daily
bread's talent; the `purify` talent file goes). `saint.toml` and the old `miracle.toml` go;
`miracle.toml` is written anew as the tier-5 prayer talent (the id reused: saves are pinned to
versions, §7).

## 7. Determinism, saves, replays

| What | Rule |
|---|---|
| Draws | the prayer: one gameplay draw per attempt **only while the strain is above 0 and the prayer unlocked** (a priest's first prayer, or any an hour and forty minutes after the last, draws nothing); the twists none; Sanctuary none. A run without a priest draws as before |
| Saved | `Progress::prayer` (`#[serde(default, skip_serializing_if = "Option::is_none")]`, hashed in `state_hash` while `Some`; a run that never prayed saves and hashes as before); `WorldState::hallows` (left out while empty, hashed). Nothing else new: the talents of the table are `Actor::talents` as today |
| Old saves | **none carried**: the owner pins saves to game versions; this design adds no alias for `remove_disease`, `purify`, `smite`, `purity_of_body`, `saint`, or the old `miracle`, and no shim for a save holding them. `Talents::drop_unknown` stays what it is for the ids that no longer exist in any file (`purify`, `saint`); `miracle` loads as the new talent, which is what a version-pinned save never meets |
| Content fingerprint | changes (talent files added and removed, the class file, `prayer.toml`'s use ids are not ids of the fingerprint: the uses are closed in code); `FORMAT_VERSION` untouched (the integrator's) |
| Replays | `use:orison` (every tier), `use:sanctuary`, `use:consecrate`; the spells by id as today; no new syntax |
| Time | the strain's decay is a function of `GameTime` (lazy), so cutting time differently never changes it; the hallow's end is a moment of `live` |
| Tests | `world/orison_tests.rs`: each use at its level and not before; the priority order with several problems; one answer before 40, two from 40, all at 60 and the pools filled; the refusal with nothing (no time, no strain); the strain's growth, decay by the clock however cut, the chance at 100 and 500 (mirrored draws), the lock and its end at 0; a brewing disease alone is no problem and goes with a shown one; the Holy face at Neutral, the Dark at Callous, the swap when the band falls; a silenced priest refused; a save and a replay. `world/dark_face_tests.rs`: the plague passed to the nearest seen creature, the poison at its degrees and resisted by an immune one, a hex turned on its caster and not on one unseen, a hold's holder webbed, the brim; Communion: no strain in a cursed room, Ebbing not laid, nothing for the Holy face. `world/sanctuary_tests.rs` as round 1 (a kobold outside never steps in; one inside leaves; the pace × 3; a blow from inside breaks it; an arrow from outside does not; a twist from inside breaks it; the fade; refused on the overworld; save and replay). `world/caster_tests.rs`: the priest's table (each entry at its level, the six cut not given). `content/prayer.rs`: the parse, a use missing or twice, a buff in `release`, a band unanswered by both faces |

## 8. What the player sees

| Where | What |
|---|---|
| The abilities menu | the Class group lists the prayer by the face's title (`PerceptionView::own_alignment` picks it): "Orison", later "Prayer", "Litany", "Benediction", "Miracle"; for a dark priest "Whisper", "Invocation", "Rite", "Pact", "Miracle"; its description the highest tier's talent's; greyed `While(raging)`/`While(silenced)` as a spell; never greyed for the strain (hidden: the rules refuse or fizzle). Communion is a passive: the Talents page. Sanctuary with Consecrate's pattern (`NoMana`) |
| Messages | the face's `pray` line, then one line per answer (§6.1's words; a disease's own `recovered` line is **not** said: the prayer's one line covers every disease, so the brewing ones stay unnamed); a fizzle the face's `fizzle` line, the same whether locked or unlucky; the refusal the face's `nothing`. Sanctuary: "You kneel and pray." … "The ground about you is hallowed." / "A dread settles on the ground about you." / "You rise from your knees." / "The peace is broken." / "The hallowing fades." |
| The status bar | nothing: the strain is hidden, a hallow is a place |
| Look mode | a hallowed cell: "… on hallowed ground" (dark: "… on dreaded ground"); nothing on a creature |
| The character screen | nothing new (the band shows as today) |
| Dumps | `abilities`: `Orison (class): here` (the face's title; `Unavailable` as the menu); `talents`: the prayer talents as held, `Communion with the dark (class)`; `progress` (dev): `prayer strain 180 (locked)`, the uses unlocked, the face; `truth` (dev): `hallow at (12, 5) r2 until 09:15:00`; JSON `abilities` as today, dev `prayer: { strain, locked, face, uses }`, `hallows` |
| The wiki | `wiki/src/classes.md`'s priest section (the generated `class-levels.md` picks the table up); a new page `wiki/src/prayer.md` ("The priest's prayer") with a generated table from `prayer.toml` and the class table (`xtask/src/wiki/prayer.rs`): each use, its level, its priority, what it answers on each face, the twist; the strain's numbers in words and figures (+100 an attempt, a point a minute, the chance per 100, the lock); Communion and Sanctuary with their numbers; `TERMS.md` rows for prayer, strain, face |
| The spec | spec/08i-character.md "Class abilities": the priest's row and the prayer; spec/07d-shops-and-rooms.md "Special rooms": hallowed ground |

## 9. Talents and class abilities

**No new talent, no new system.** The coverage policy's test (`docs/dev/progression.md`): a
system is broad; a sub-feature extends its parent and mints nothing; a class ability is counted
by the systems it reads. The prayer is a class ability; its limit (the strain) is the ability's
own time-refilled cost, which the policy's step 4 asks of every cost ("time refills it, 'time is
the penalty'"; "a talent that changes a cost changes the data's cost, never adds a timer").

| Entry | System(s) it reads (`systems`) | Its parent's talents, unchanged |
|---|---|---|
| Orison (wounds) | `hit_points`, `status` | Healthy, Fast healer; Clotting |
| Daily bread (hunger, Purify's rule) | `hunger`, `diseases` | Lean, Ascetic, Iron stomach, Gourmand, Herb lore |
| Communion with the dark | `special_rooms` | Pious, Old delver, Quiet delver |
| Prayer (disease, poison) | `diseases`, `status` | Fever-proof, Strong constitution, Poison resistance, Divine grace |
| Prayer of release | `status` | Cool head, Nature's ward |
| Litany (restoration) | `diseases` (drain's tag since #773) | Unbroken soul, Iron will |
| Litany of the mind (mana) | `mana` | Deep well, Clear mind, Meditation |
| Benediction (two answers) | `hit_points`, `status`, `diseases`, `mana`, `hunger` (what it answers more of) | as above |
| Sanctuary | `special_rooms`, `hit_points`, `mana` | Pious, Old delver; Healthy; Meditation |
| Miracle (all, every pool) | the five above and `resources` | Deep lungs, Steady breathing |

Against the bar, entry by entry: none is information only (rule 8); none a small percentage on a
niche mechanic; none duplicates a talent's effect (rule 3: the prayer's cures are the temple's
functions, which no talent gives; `purify` rides on one talent now that the `purify` talent goes;
Purity of body leaves the priest, so no immunity sits beside a cure); none chains (rule 4: the
tiers are table rows, not prerequisites); every one works underground (rule 6: Sanctuary refused
only on the overworld; Communion's rooms are the dungeon's); no hidden stat marks (rule 7: the
prayer trains nothing; the strain is hidden but is a cost, not a mark, and the wiki states it);
nothing on gold, favour, XP, or quests (rules 1, 2, 9), and the 40, 50, and 60 change what the
character can do in a dungeon (rule 1). Tags match hooks: each prayer talent names the systems its
uses' fixes read, Communion `special_rooms` (its two reads are a room's status and a room's
unholiness).

**Should the strain be a system with a pickable?** The policy's step 3 ("a broad rule set a talent
could change: a check, a price, a pace") fits it (a talent *Fervent*, priest-only, "the strain of
your prayers passes twice as fast", would be a real choice and visible enough by rule 7's
analogy). Not proposed: the strain is one class's limit, as Lay on hands' Drained is the
paladin's, and the owner's direction asks for the prayer, not a tree about it. Q8 leaves the
door open.

Class abilities of other classes: none touched. The paladin keeps Smite, Consecrate (its
`alignment` as today unless Q3 lowers it), and Purity of body; the monk Purity of body; the witch
Hag's bargain (the `reprieve` kind's one holder now).

## 10. Rejected ideas

| Idea | Why not |
|---|---|
| **The three spells of round 1** (Daily bread, Balm, Restoration as `[[spell]]` files) | the owner's direction folds them into the one prayer's uses: a spell has a cost in mana and a school that trains, which a prayer paid in strain does not; keeping both would be two ways to the same cure |
| **The dark powers bargain instead of fizzling** (no random fizzle; a hard wall at a cap; blood or alignment as the price) | a second cost model for one ability; the owner's fizzle is the mechanic, and the dark face's twists are where the dark differs |
| **A dark face that heals on kills** (Makhleb) | a fight economy: the priest fights no better; the twists ride on cures only |
| **A dark face that raises the dead** (an undead ally) | the druid's allies (#821) and a fight ability |
| **Turn undead** | a fight ability (the undead flee or die) |
| **Keep Smite** | a blow, and #823's |
| **Keep the reprieve at 50 under another name** | its name is the capstone's (q7); a priest who prays their hit points full needs it less; Rise from the ashes is the sign's and Hag's bargain the witch's; the table has no row to spare |
| **Keep Saint as healing × 2 beside Miracle** | Miracle fills the pool; a pace on top is a percent no one feels (rule: a small percentage) |
| **Keep Remove disease as a spell in the table** | the prayer's 12 is the same cure; the spell file and its scroll stay for everyone |
| **The prayer trains Willpower** | a hidden pile from a free ability; the policy's rule 7 spirit |
| **A prayer with nothing to answer still strains** | ADOM's unheard prayer costs little; a free refusal is kinder and leaks nothing (a priest cannot probe for a brewing disease: a brewing one alone is no problem) |
| **Telling a locked prayer from an unlucky one** ("Silence." against "Nothing answers.") | the lock is a hidden number's state; one line for both, as ADOM's |
| **Showing the strain** (a bar, a word) | hidden numbers stay hidden; the wiki states it; the fizzle is what the player learns from |
| **Cooldowns among the release list** (Spent, Drained, Winded, Worn thin, Exhausted, Hoarse) | a prayer that resets another gift's price is a cross-class reset; the list holds what ails, not what was paid |
| **A separate dark table** (two class files, or two talents a row) | twenty rows for one class; one talent a row with two faces keeps the table readable and the data in one place |
| **Light as a table entry** | the kit's scroll teaches Light for certain at level 1 |
| **Identify** | the merchant's (the owner) |
| **A hallow that sheds light** | q8: no (no golden, no glare, no beacon) |
| **Hunger answered at Not hungry** | a priest spamming bread; Hungry or worse is the problem |

## 11. Implementation plan

| Slice | Issue | What it touches | Depends on |
|---|---|---|---|
| **P1 The prayer and the table** | priest: the Orison, its uses, the strain, the cuts | `warlock-core`: `content/prayer.rs` (`PrayerRules`, the parse and checks, `Content::prayer()`), `content/talent.rs` (`TalentEffect::Prayer`), `talent.rs` (`TalentMods::prayer`), `content/class.rs` (`ClassActive::Orison`), `world/orison.rs` (the face, the problems, the strain, the attempt, the fixes; one arm in `class_ability.rs::use_class`, one in `buff.rs::kind_of`), `world/experience.rs` (`Progress::prayer`), `dump.rs` (the layers), `perception.rs` (the title); data: `prayer.toml`, `talents/orison.toml`, `daily_bread.toml` (with `purify`), `prayer_of_cleansing.toml`, `prayer_of_release.toml`, `litany_of_restoring.toml`, `litany_of_the_mind.toml`, `benediction.toml`, `miracle.toml` (rewritten), `purify.toml` and `saint.toml` removed, `classes/03-priest.toml`; `spell_list_tests`' bound; tests §7; `docs/dev/progression.md` (the priest's line, "Class actives"), a new `docs/dev/prayer.md`, `docs/dev/spells.md` (the bound), CLAUDE.md's action text (`use:orison`). The holy face only: the dark words may ship, the twists wait | — |
| **P2 The dark face and Communion** | priest: the dark powers' answers | `world/orison.rs` (the twists, §6.2), `world/dark_face.rs` (the nearest seen creature, the pass-on, the turn-back), `content/talent.rs` (`TalentEffect::Communion`, the widened `alignment` check), the room status's one place; data: `talents/communion.toml`, the class file's 9; tests §7; `docs/dev/prayer.md`, `docs/dev/alignment.md` (band-gated passives) | P1 |
| **P3 Sanctuary** | priest 50: the hallowed ground | as round 1's P2: `TalentEffect::Sanctuary`, `ClassActive::Sanctuary`, `world/sanctuary.rs`, one clause in `ai.rs::shuns` and `spawn.rs`, one term in `healing_rate` and `mana_rate`, one call in `fight.rs::resolve`, `spell.rs::spell_strikes`, and the twists, `WorldState::hallows`, perception for look mode, dumps; data: `talents/sanctuary.toml`, the class file's 50; tests §7; `docs/dev/special-rooms.md` "Hallowed ground" | P1 (the class file) |
| **P4 Wiki, spec, probe** | the priest's pages | `xtask/src/wiki/prayer.rs` and `wiki/src/prayer.md`, `SUMMARY.md`, `TERMS.md`; the priest in `classes.md`; spec/08i-character.md, spec/07d-shops-and-rooms.md; `world/balance_probe.rs`'s priest column (a line for the prayer: no roll); `ui/screen_snapshots.rs` cases `abilities-priest-1` and `abilities-priest-60` (the menu's titles: text) | P1–P3 |
| **The scroll of restoration** (q9, its own small issue, not the priest's) | a read scroll, no spell behind it (as the scroll of identify): `read` kind `cleanse` with `mends`, `restores`; level 8, frequency 10, 300 gold; so a warrior drained far from Cēaptūn has a dice-roll's hope | — |

P1 adds saved state (`Progress::prayer`) and an action, P3 state; by the integrator's rule each
may run beside one other such PR. No golden anywhere (nothing here is the GPU's).

## 12. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| Q1 | **The strain's numbers**: +100 an attempt, a point a minute, 10 % per 100? | **as written**: a prayer every hour and forty minutes is free, a second within the hour risks under 10 %, a fizzle costs about six hours; NetHack's 300 turns and ADOM's unheard prayer are the scale. All three are one line of `prayer.toml` |
| Q2 | **The bands**: the holy powers answer Neutral and better, the dark Callous and worse? | **yes**: a human or elven priest starts Neutral and is answered from the first turn; the first evil deed does not change who answers, the fall to Callous does |
| Q3 | **Consecrate's band** (`min = kind`, the paladin's shared talent): a Neutral priest's Consecrate goes unanswered until they do good. Lower it to `neutral`? | **lower it**: one line of `consecrate.toml`, the paladin (starting Good at 550) unaffected in practice, and a fresh priest's 9 answers. Communion's `max = callous` leaves the Neutral band to Consecrate |
| Q4 | **Cure wounds (3)**: keep as the mana heal beside the prayer, or cut (Orison fills the pool at 1)? | **keep**: a mana heal when the strain is high, a touch heal for someone beside you (the Toucher style), and the priest's third spell (the test's bound) |
| Q5 | **The release list** (Hexed, Weakened, Sluggish, Drowsy, Webbed, Held, Frightened, Transfixed, Ashen): Ashen in (an altar's bane, a day of −2), the cooldowns out? | **as written**: an altar's bane ails; a cooldown is a price paid |
| Q6 | **A brewing disease**: cured only with a shown one (so the prayer leaks nothing), or a problem of its own? | **only with a shown one**: the priest never learns of a disease by praying; Miracle at 60 takes everything either way |
| Q7 | **The hunger threshold**: Hungry or worse, or anything under Satiated? | **Hungry or worse**: the problem is hunger, not room for more |
| Q8 | **A pickable on the strain** (*Fervent*: the strain passes twice as fast, priest-only, Grit)? | **no**: one class's limit, as Lay on hands' Drained; the owner may ask for it later in one talent file (its hook is `Strain::now`'s divisor) |
| Q9 | **The scroll of restoration** (the owner: sure): with Restoration no longer a spell, a read scroll of its own? | **yes, its own small issue** (§11): the item game's, not the priest's |
| Q10 | **Sanctuary's dark name**: "Sanctuary" for both faces with the words differing, or a name of its own? | **one name**: the ability is the same; the lines differ |

## 13. The owner's decisions (2026-11-02, round 1)

Recorded verbatim from PR #831, with what this round did with each.

| # | The owner | Done here |
|---|---|---|
| q1 | "in. there is no need to hamstring the priest completely. +1 to hit is fucking trash ability in grand scheme of things. It's not going to make or break a priest." | Blessing stays at 3 (§4, §5) |
| q2 | "daily bread first" | the hunger use is the first unlocked after wounds (6); release (the old Balm) at 18 (§5) |
| q3 | "create evil versions of these gifts too. They do not need to share the same theme or be evil twin type of thing. if evil priest is just better, then do be it." | the dark face (§6.2: the plague passed on, the hex turned back, the brim), Communion with the dark beside Consecrate (§6.3), Sanctuary's dark words (§6.4); where the dark is better, it is |
| q4 | "you can make it pretty good. not terribly important." | the hunger answer fills to Satiated (the dark to the brim) rather than a wafer (§5) |
| q5 | "priest could just get rid of the thing entirely" | read as: no refusal and no mending around a disease: the prayer cures the disease (priority 1) and restores in its turn (priority 7); from 40 both in one prayer (§5, §6.1) |
| q6 | "sure" | the kit, the stats, the skills, and the weapon ranks unchanged (§4) |
| q7 | "capstone can be pretty busted. Let's call it 'Miracle' instead. It fills satiation, cures all ailments, fills hp pool, fills mana pool (and fills all other pools too). It takes roughly same time as casting normal spell. It tracks an invisible number (like +100 per miracle) that decays over time. Every time miracle is attempted, there is a chance based on number already in the pool that it just does not work (kinda like when out of piety in adom). If it fizzles out, +100 is added and future uses fizzle out automatically until the pool has decayed to 0." | Miracle at 60 (`all`: every use and every pool), a cast's time, the strain exactly so (§6.1); Saint and the reprieve go (§4) |
| q8 | "sure" | the hallow sheds no light (§6.4) |
| q9 | "sure" | the scroll of restoration ships, as a read scroll of its own now that Restoration is a prayer's use (§11) |
| q10 | "no." | Purity of body leaves the priest's table (§4) |
| the direction | "i think we take the idea of the miracle, and give that to priest pretty early (same as ADOM has 'spammable' pray). We just keep adding things it does (and making it's name grander) as the priest levels up. basically before lvl60, it picks from the list of ailments in priority order same as adom, based on what uses is unlocked already. like: 1. cure disease 2. cure poison 3. stop bleeding + fill hp pool 5. fill mana 6. fill satiation 7. etc etc. Then we just keep adding things it can do, then perhaps it does 2 things per prayer. Last at miracle state it just does everything at once for 1 cost." | the Orison at 1 with the strain; uses added at 6, 12, 18, 25, 32; the titles Orison → Prayer → Litany → Benediction → Miracle (the dark Whisper → Invocation → Rite → Pact → Miracle); the priority order disease, poison, wounds, release, mana, hunger, restoration; two answers at 40; everything at 60 for one attempt's strain (§5, §6.1) |
