# Warlock — Skill uses: what each skill does, how it trains, and what the gaps need

Design document for the owner's review (#973). Written 2026-11-05 against `origin/main` at 24dc95dc. An investigation, not a feature: nothing here is implemented, no rule or data changed. Every effect and every mark below was checked in the code, not read off the skill's description: the reading function and the firing hook are named with file and line. Where a number was measured, §5 says how. What could not be established is marked *(not established)*.

**The owner's request** (#973): *"investigate whether we have enough ways to train skills, identify which skills have no uses yet, and propose uses for skills."*

**Merging this PR approves the proposals the owner keeps**; each becomes an implementation issue linking to its entry in §6.

## Contents

1. Summary: every skill in one table
2. How a skill grows, in numbers (what "too few" means)
3. Which skills have no use
4. Are there enough ways to train?
5. Measurements: how the numbers were produced
6. Proposals, ranked
7. Work in flight that this touches
8. Open questions

## 1. Summary

29 skills (`assets/data/skills.toml`). The verdicts: **4 no use** (and the same 4 have no way to train), **7 too few ways to train**, **3 niche**, **15 fine**.

Columns: *Effect* is what the game reads today (the function, file:line); a value no rule reads is "nothing". *Training* is each `mark` rule with its chance and the hook that fires it (`World::train(Use::…)`, file:line); "typical" is the marks a run earns per character level by §5's sessions and counts (the ladder: 5 marks a level rolls `1d3`, 12 `1d5`, 25 `2d4`, 50 `3d3`; under 5 a level the skill rolls `+1`, as an untrained-by-use skill does). *Who* is who starts with it: races (`races.toml`), classes (`classes/*.toml`; **bold** = `known_for`), `opt` = a class's optional skill, `bg` = a background line (`data/background/`). Every bard also takes up every skill but Sneak attack at `1d4+1` at level 12 (Jack of all trades, `02-bard.toml:44`).

| Skill | Effect (read in) | Training (fired in) · typical marks a level | Who | Verdict |
|---|---|---|---|---|
| Appraise | `identify`: each game hour, 2–20 % to tell one unidentified carried thing (`world/appraise.rs:32` trained-at-all, `:41`) | `appraise` 100 % per thing told (`appraise.rs:61`) · 1–3 (one per unidentified item found, if carried an hour or more) | gnome; bard, thief, craftsperson, **merchant**; bg ×5 | niche |
| Athletics | `kick`: +0–12 to a door kick's Strength (`world/door.rs:166`); `speed`: +0–10 % quicker (`world/actor.rs:394`) | `kick` 20 % (`door.rs:181`, `chest.rs:150`); `burdened` 5 % (`world/mod.rs:1256`); `travel` 5 % (`travel.rs:217`, `encounter.rs:995`) · 3–8 (measured: 4 per 100 overworld steps) | **berserker**, ascetic, ranger, **warrior**, commoner; bg ×1 | fine |
| Balance | `burden`: 0–80 % less of the burden's slowdown (`world/schedule.rs:272`) | `burdened` 5 % (`mod.rs:1256`) · 0 unless the player carries too much on purpose (5 per 100 slowed actions) | **ascetic**, warrior | too few ways to train |
| Bluff | **nothing** (no `benefit`; the abilities menu lists it "not usable yet", `ability.rs` the `Ability::Skill` row) | **none** (no `mark`) · 0 | half-elf; **bard**, aristocrat, **merchant**; bg ×1 | no use |
| Concentration | `mana_regen`: mana back 0–60 % faster (`world/mana.rs:78`); `max_mana`: +0–8, at most a fifth (`world/concentration.rs:33`) | `cast` 25 % per mana spent (`mana.rs:106`) · 10–30 for a caster (a cast or two a fight) | priest, druid, ascetic, paladin, **wizard**, witch; bg ×2 | fine |
| Decipher script | `study`: 0–20 off a study check (`world/learn.rs:137`) — **the same benefit as Spellcraft's**, summed | `study` 50 % per scroll studied (`learn.rs:250`; a scriptorium doubles it, `room_effects.toml:416`) · 0–2 (a spell scroll found a level at most; 30 scroll types exist) | bard, priest, wizard; bg ×2 | too few ways to train (and a duplicate) |
| Detect traps | `detect_traps`: +0–12 to finding hidden traps, searching and in passing (`world/search.rs:157`); trained at all, you notice traps in passing (`search.rs:191`) | `find_trap` 50 % per hidden trap found (`search.rs:272`, `:303`) · 1–2 (2–5 hidden traps a generated level, §5.3; half are sprung, not found) | kobold; **thief**; bg ×1 | too few ways to train |
| Disable device | `disarm`: +0–12 to the Dexterity check (`world/disarm.rs:86`); trained at all, the Disarm ability (`ability.rs:509`) | `disarm` 20 % per try (`disarm.rs:55`; retries allowed, `disarm.rs:14`) · 2–6 (a few tries per known trap) | kobold; **thief** | fine |
| Escape artist | **nothing** (no `benefit`; the pits hook of #128 is still "pending", `docs/dev/skills.md:162`) | **none** · 0 | ascetic, thief | no use |
| Handle animal | `tame`: +0–20 to the taming check (`world/tame.rs:181`); trained at all, the Tame ability (`ability.rs:522`) | `handle` 50 % per order given (`ally.rs:485`, `sic.rs:83`), ally's kill (`fight.rs:1180`), taming try (`tame.rs:209`) · 10–40 with a companion (its kills) | **druid**, paladin, ranger, commoner; bg ×1 | fine |
| Heal | `heal`: +0–12 to the treatment check (`world/heal.rs:103`); trained at all, the Heal ability (`ability.rs:523`) | `heal` 30 % per try (`heal.rs:116`) · 2–6 (bleeding after most fights; a poison or disease once each, `heal.rs:24–26`) | **priest**, druid, paladin, **witch**; bg ×5 | fine |
| Initiative | `initiative`: +0–12 to the check when caught unawares (`world/surprise.rs:225`); `surprise_ease`: 0–75 % of the failed check's extra cost off (`surprise.rs:287`) | `surprise` 100 % per such check (`surprise.rs:234`) · 2–8 (an ambush or a sprung hidden trap each; never on purpose) | berserker, bard, **ascetic**, **ranger**, thief, warrior; bg ×1 | fine |
| Intimidate | `intimidate`: +0–12 to the Willpower check a war cry or a threat asks (`world/war_cry.rs:68`, `world/intimidate.rs:79`); trained at all, the Intimidate ability (`ability.rs:524`) | `intimidate` 100 % per cry or threat (`war_cry.rs:48`, `intimidate.rs:73`) · as often as used: 10–50 | half-orc; berserker, **paladin**, witch, warrior, aristocrat; bg ×2 | fine |
| Knowledge | `resist`: +0–4 to the player's check against a creature's drain, disease, or hold, read only *above the creature's level* (`world/knowledge.rs:54` → `Actor::skill_effect_above`) | `discover` 100 % per first kill of a kind or paying arrival (`knowledge.rs:46` ← `experience.rs:148,170`); `resist` 25 % per such check (`knowledge.rs:38`) · 5–15 (first kills come steadily) | bard, **priest**, **wizard**, **aristocrat**, merchant; bg ×2 | niche (the effect: +4 at most, and nothing against a creature of your level or above) |
| Listen | `hearing`: hear sounds one fainter per 10 points (`world/actor.rs:844`; a resting lookout, `encounter.rs:854`) | `hear` 30 % per sound that is **news** (`perceive.rs:443`; `encounter.rs:870`): the same kind near the same place within 10 turns is not news (`noise.rs:622`, `RECALL_TURNS`) · 0–3 (measured: 0 in 120 steps and a night among kobolds, 0 in 200 hub turns, 2 in a 12-turn fight) | elf, halfling, gnome, goblin, ratfolk; berserker, druid, ascetic, ranger | too few ways to train |
| Metalcraft | `temper`: +0–12 to the forge's Strength check (`world/temper.rs:148`) | `temper` 100 % per item worked (`temper.rs:171`) · 0–3 (a forge is a special room of weight 3 beside the altar's 8, `room_effects.toml:331`; no other anvil) | dwarf, duergar; **craftsperson**; bg ×2 | too few ways to train |
| Music | `song_degrees`: +1 degree at 50 and 100, then per 100 (`world/song.rs:61`); `song_craft`: +0–8 to the hearers' check (`song.rs:63`) | `song` 100 % per song (`song.rs:92`) · as often as sung: 10–40 | **bard**; bg ×1 | fine |
| Negotiation | `bargain`: 0–30 % better shop prices (`world/shop.rs:222`, `:300`); services never haggle (`service.rs:24`) | `deal` 25 % per purchase or sale (`shop.rs:295`) · 1–5 (shops are in towns; one visit a level or less) | bard, thief, craftsperson, **merchant**; bg ×3 | niche |
| Open lock | `pick`: +0–12 to the lock-picking check (`world/lockpick.rs:41`); trained at all, Pick lock (`ability.rs:514`) | `pick` 20 % per try (`lockpick.rs:74`) · 1–5 (0–3 locked doors and 0–1 chests a generated level, §5.3; retries allowed) | **thief** | fine |
| Search | `search`: +0–12 to a search turn (`search.rs:164`); `search_reach`: a cell farther from 75 (`search.rs:260`) | `search` 10 % per search turn (`search.rs:277`) · 10–30 on purpose (measured: 27 per 300 turns) | dwarf; thief | fine |
| Sneak attack | `sneak_dice`: 1d6 trained, up to 7d6 at 200 (`world/ambush.rs:67`); `surprise`: +0–12 to the victim's initiative difficulty (`surprise.rs:271`) | `sneak_attack` 50 % per sneak attack that hits (`ambush.rs:137`) · 10–30 for a thief | **thief**; drow (opt 40 %) | fine |
| Spellcraft | `study`: 0–20 off a study check (`learn.rs:137`) | `study` 100 % per scroll studied (`learn.rs:250`) · 0–3 (as Decipher script's) | **wizard**, **witch**; priest (opt); bg ×2 | too few ways to train (#977 asks the same, its question 6) |
| Spot | `spot`: +0–12 to noticing secret doors, hollows, mimics in passing (`search.rs:170`); trained at all, you notice them at all (`search.rs:191`); `lookout`: +0–12 on the overworld (`encounter.rs:812`). A creature's Spot is read against the player's slip (`slip.rs:74`) | `spot` 50 % per thing noticed in passing (`search.rs:307`); `lookout` 50 % per encounter lookout (`encounter.rs:827`) · 0–2 (0–2 secret doors a level, §5.3; measured 0 in every session) | elf, half-elf, drow; **ranger**, thief; bg ×2 | too few ways to train |
| Steal | `steal`: +0–12 to Dexterity against Willpower (`world/steal.rs:80`); trained at all, Steal (`ability.rs:518`) | `steal` 50 % per try (`steal.rs:96`; retries allowed, `steal.rs:23`) · as often as tried | **thief**; bg ×1 | fine |
| Stealth | `quiet`: 0–75 % less noise (`noise.rs:305`); `slip`: +0–12 past darkvision (`world/slip.rs:72`); `stalk`: +0–12 on the overworld (`encounter.rs:819`, `:968`) | `sneak` 15 % per step within earshot of something (`world/hearing.rs:50`; `encounter.rs:830`, `:998`) · 20–60 (measured: 16 per 120 steps among creatures; the progression design's 30–60) | halfling, goblin, drow, duergar, ratfolk; **thief**; bg ×1 | fine |
| Survival | `travel`: 0–20 % shorter journeys (`schedule.rs:228`); `forage`: +0–30 % to a forage (`world/forage.rs:119`), trained at all, Forage (`ability.rs:521`); `lookout` +0–6 and `evade` +0–12 (`encounter.rs:812`, `:973`) | `travel` 10 % per overworld step (`travel.rs:217`); `lookout` 10 %; `forage` 25 % per forage (`forage.rs:138`) · 5–15 (measured: 11 per 100 overworld steps) | lizardfolk; **berserker**, **druid**, **ranger**; bg ×2 | fine |
| Swim | **nothing** (no `benefit`; wading is the birth sign's and Light step's: `schedule.rs:217–224`) | **none** · 0 | lizardfolk; berserker, ranger, warrior; bg ×2 | no use |
| Tumble | `dodge`: +1 Defence (Dodge) at 20, 40, 60, 80, 100, then per 25 (`world/actor.rs:673`) | `dodge` 40 % per attack that missed **by the Dexterity or Dodge category** (`fight.rs:881`) or known trap sidestepped (`sidestep.rs:58`) · 5–20 in fights; **by the code, 0 until Dexterity is 12 or Tumble 20** (§4, the attribution catch) | bard, **ascetic**, **thief**; bg ×2 | fine, with a catch |
| Woodcraft | **nothing** (no `benefit`; "not usable yet" in the menu) | **none** · 0 | **ranger**, **craftsperson** | no use |

The rows' "bg ×N" are approximate; the exact starts beyond the guaranteed lists are these (classes' `optional_skills`, a chance in percent of `2d4`, `classes/*.toml`; background lines, `grep -o 'skill = {[^}]*}' assets/data/background/*.toml`, 54 grants):

| Skill | Optional (class: chance %) | Background lines |
|---|---|---|
| Appraise | wizard 20, aristocrat 20 | 5 |
| Athletics | paladin 25, craftsperson 25 | 2 |
| Balance | thief 20 | 0 |
| Bluff | thief 30, witch 25 | 2 |
| Concentration | — | 2 |
| Decipher script | merchant 25 | 3 |
| Detect traps | — | 1 |
| Escape artist | — | 0 |
| Handle animal | berserker 20, witch 25, aristocrat 30, merchant 20 | 2 |
| Heal | ranger 15, ascetic 25, warrior 15, commoner 15 | 6 |
| Initiative | — | 1 |
| Intimidate | — | 2 |
| Knowledge | paladin 25, witch 20 | 2 |
| Listen | priest 15, bard 30, thief 30, merchant 15 | 3 |
| Metalcraft | warrior 15, commoner 10 | 2 |
| Music | aristocrat 25 | 1 |
| Negotiation | — | 4 |
| Open lock | — | 1 |
| Search | ranger 20, craftsperson 20, wizard 15 | 1 |
| Sneak attack | drow (race) 40 | 0 |
| Spellcraft | priest 25 | 2 |
| Spot | bard 30, druid 20, berserker 30 | 2 |
| Steal | bard 15 | 1 |
| Stealth | ranger 30, ascetic 20 | 3 |
| Survival | warrior 30, commoner 30 | 6 |
| Swim | druid 20, ascetic 20, commoner 20 | 4 |
| Tumble | — | 3 |
| Woodcraft | druid 35, commoner 20 | 2 |

## 2. How a skill grows, in numbers

Growth is level-up rolls only (`docs/dev/skills.md:15–28`): each level gives the class's `rolls` (2; thief and ranger 3; bard 4) + 1 for a human + 1 per 4 Intelligence from 14; each roll on a skill throws the die its marks set on the ladder (`skills.toml:90–92`: 0 marks `+1`, 5 `1d3`, 12 `1d5`, 25 `2d4`, 50 `3d3`, 100 `3d4`, 200 `3d5`), one step up if the class is known for it, +1 if the class or race trains it, and spends half the marks. So:

- a skill earning **under 5 marks a level** rolls `+1` (`1d3` known-for): about 1–2 points a level, 30–60 over a career, whatever the player does. That is the line for "too few ways to train";
- **25–50 marks a level** is where the design wanted a practised skill (`docs/design/progression.md:535`: a sneaking thief's 30–60 Stealth marks, Stealth 100 near level 10);
- a skill with **no `mark` rule** can never leave `+1`; a skill with no `benefit` gives nothing at any value.

The marks are a reservoir spent by the roll, so a skill used ten times a level and one used a hundred times differ by two ladder steps (about `1d5` against `3d3`), not tenfold: the chances in `skills.toml` were set so that often-used skills do not race ahead (`skills.toml:19–20`). The gaps below are therefore the skills under the 5-mark line, not the ones under 50.

## 3. Which skills have no use

Four skills have **no `benefit` and no `mark`**: nothing reads them, nothing trains them, and a roll spent on them buys nothing but a number on the Skills screen. Each is in the abilities menu as a skill "not usable yet" (`ability.rs`, the `Ability::Skill` rows; the menu's test lists `bluff` and `woodcraft`, `ability.rs:1219–1230`).

| Skill | Who starts with it today | What the data file says it is for |
|---|---|---|
| **Bluff** | half-elf `1d4+2`; bard `2d6+10` (known for), merchant `2d6+10` (known for), aristocrat `2d4+4`; a thief 30 % and a witch 25 % of the time; two background lines | "Lying with a straight face, feinting, and talking your way out." |
| **Escape artist** | ascetic `2d4+4`, thief `2d4+4`; no optional, no background line | "Slipping out of bonds, grabs, and tight places." The pits hook (#128) is pending (`skills.toml:206`) |
| **Swim** | lizardfolk `1d6+4`; berserker, ranger, warrior `2d4+4`; a druid, ascetic, or commoner 20 % of the time; four background lines | "Staying afloat and making headway in deep water." Deep water (`lake`, `terrain.toml:1044`) is `walkable = false` for everyone |
| **Woodcraft** | ranger `2d6+10` (known for), craftsperson `2d6+10` (known for); a druid 35 % and a commoner 20 % of the time; two background lines | "Working wood: carving, carpentry, and the making of bows." |

A **bard** and a **merchant** are known for Bluff, and a **ranger** and a **craftsperson** for Woodcraft: four classes have one of their two or three signature skills doing nothing. The thief's `levelup:auto` and a player's rolls can be sunk into them unknowingly.

**Declared but never fired**: none. Every `Use` variant (`skills.rs:292–353`) has at least one `train(Use::…)` call, and every `mark.on` in the data names a fired use. **Read but never trained**: none either. The two halves line up; the holes are whole skills, not dangling ends.

## 4. Are there enough ways to train?

**For 15 skills, yes**: a character who wants it high can practise it on purpose, and time is the only price (search turns, sneaking steps, casts, songs, cries, tries at a lock or a trap or a pocket, orders to a beast). **For 11, no**, in three ways:

1. **Nothing to practise** (4): Bluff, Escape artist, Swim, Woodcraft (§3).
2. **The use exists but comes too rarely to matter** (7, under 5 marks a level in §5's sessions):
   - **Spot** and **Detect traps** train only when something hidden is *found*: 0–2 secret doors and 2–5 hidden traps per generated level (§5.3), and a sprung trap teaches nothing. Spot earned 0 marks in every session. A player cannot go looking: searching trains Search, not these.
   - **Listen** trains only on sounds that are news (`noise.rs:622`): a creature walking turn after turn is one mark's chance, not thirty. A night's rest among kobolds, a night in Cealdhlāw, and 200 turns in the hub earned 0; a fight 2. Nobody can listen harder.
   - **Metalcraft** trains at a forge's anvil only, a special room drawn at weight 3 (`room_effects.toml:331`), so many runs never meet one; there is no anvil in a town.
   - **Spellcraft** and **Decipher script** train only by studying a scroll (`learn.rs:250`), one a level at best, and share one benefit (`study`). #977 asks whether that is enough once learning depends on Spellcraft alone; it is not.
   - **Balance** trains only while the player's load slows them (`mod.rs:1256`). It *can* be practised (carry a boulder, walk), but at 5 % per action it is the slowest grind in the game and nobody is told it works that way.
3. **Cannot be practised on purpose by design** (not counted as gaps): **Initiative** (being caught unawares) and the passive halves of Knowledge (`resist`) and Appraise (hourly). Their marks are certain (100 %) when they fire, which is the right shape for a passive skill.

**One hidden catch in a "fine" skill**: Tumble's mark fires only when a miss is *attributed* to the Dexterity or Dodge category (`fight.rs:873–881`). A miss is attributed by a draw weighted by the defender's positive bonuses (`combat.rs:555–559`, `Defence::why_missed`, `combat.rs:224`): a target with **no** Dexterity bonus (Dexterity 11 or less) and Tumble under 20 (no Dodge bonus) has weight 0 → every miss is "plain" → **Tumble never trains**. The code is the proof (§5.2); the sessions agree without proving it: a human thief (Dx 11, Tumble 12) earned 0 marks in a fight where a halfling ascetic (Dx 15, no armour) and an elf thief (Dx 19) earned 5 each, but so did a halfling thief at Dx 14, and a twelve-turn fight is too short to tell luck from the rule. And an armoured fighter's misses are drawn as "armour" in proportion to its bonus (the jerkin's 1 against Dexterity's 2 for that halfling), so a warrior in mail trains Tumble rarely however nimble. Proposal Tu1 in §6.14 fixes the first.

**Growth itself is not the problem.** The roll machinery, the ladder, and the known-for steps are as designed; what is missing is uses, and for the 15 fine skills the chances are well set (measured rates match the data: Search 9 % of turns, Stealth 13 % of steps, Survival 11 % and Athletics 4 % of overworld steps).

## 5. Measurements: how the numbers were produced

No code was changed. Every number comes from the built `warlock-cli` of this commit, the `actors` dump layer (dev-only), which prints each trained skill with its marks and die (`stealth 17 (16 1d5)`), over scripted sessions; and from `gen` for level counts. The sessions (the script is not committed; each is one `dump --scenario … --character … --actions … --layers actors`):

### 5.1 Passive marks over scripted play

| Session | Actions | Marks earned |
|---|---|---|
| `circles`, human thief: 120 steps among the hall's kobolds (`move:E*3 move:W*3` ×20) | 120 steps within earshot | Stealth **16** (13 %; data 15 %). Spot, Search, Listen (halfling thief, same walk): 0 |
| `circles`, halfling thief: `rest:dusk rest:dawn` | 13,000 turns | Listen **0**, everything else 0 |
| `showcase` hub, halfling thief: `wait*200`; human thief: `rest:dusk rest:dawn` | 200 turns; a night | Listen 0; all 0 |
| `showcase` hub, human thief: `search:100` ×3 | 300 search turns | Search **27** (9 %; data 10 %) |
| `cealdhlaw`, human thief: `search:50 rest:dusk rest:dawn search:50` | 100 search turns, a night on Cealdhlāw 1 | Search 9; Listen not trained; Spot, Detect traps 0 |
| `world`, human ranger: 120 overworld steps east and west (`encounter:pass` between, none pended; the character starved on the third day) | ~100 journeys | Survival **11** (travel 10 % + lookout 10 %: no encounter came), Athletics **4** (5 %); Spot 0 (no lookout) |
| `arena`, halfling ascetic (Dx 15, Defence 15, no armour, Tumble 16): `move:NE*8 move:E*4 wait*60`, dead on turn 24 | ~12 turns beset by kobolds and rats | Tumble **5** (`1d3`), Listen 2, Stealth 1, Initiative 0 |
| `arena`, elf thief (Dx 19, Defence 17, Tumble 19): the same | ~12 turns | Tumble **5**, Stealth 2 |
| `arena`, halfling thief (Dx 14, Defence 13, a leather jerkin, Tumble 16): the same, dead on turn 19 | ~10 turns | Tumble **0**, Listen 2, Stealth 1 |
| `arena`, human thief (Dx 11, Defence 12, a leather jerkin, Tumble 12): the same | ~12 turns, dead | Tumble **0**, Stealth 0 |
| `arena`, dwarf warrior (no Tumble): the same, 72 turns to 1 hp | ~60 turns of fighting | **0 marks in any skill** (Athletics, Balance, Initiative, Intimidate, Metalcraft, Search, Swim: none fires in a plain fight) |

The fights were short because the arena's pack kills a level-1 character; the per-turn rates are the point. "Typical marks a level" in §1 scales these by a level's play (the balance probe's day model, `balance_probe/training.rs:42–68`: 30 sneaking steps, 20 search turns, 10 journeys, 0.5 kicks, 0.3 finds, 0.5 spots a day; one to four days a level, `training.rs:25–31`). Those day counts are the probe's first guesses, not measurements: treat the "typical" column as an order of magnitude.

### 5.2 The Tumble attribution catch

The rule, from the code: `combat.rs:555`: `if natural == FUMBLE || total < BASE_DEFENCE || weight == 0 { None }`; `reason_weight` (`combat.rs:216`) sums the positive category bonuses (`stats::modifier` is `(score − 10) ÷ 2`, so Dexterity 11 gives 0), and `why_missed` (`combat.rs:224`) picks a category in proportion; `fight.rs:873–881` marks Tumble only for `Some(Dexterity | Dodge)`. So a character with Dexterity 11 or less and Tumble under 20 has weight 0 unless armour or a shield gives one, and then every attributable miss is the armour's. The sessions above are consistent with it and no more: the Dx-14 halfling thief's 0 is luck or the jerkin's share, not the rule.

### 5.3 Opportunities per generated level

`gen --level L --seed warlock`, counting `^` (traps, hidden too), `&` (locked doors), `S` (secret doors), `⊟` (chests):

| Level | Traps | Locked doors | Secret doors | Chests |
|---|---|---|---|---|
| depths 1 / 2 / 3 / 4 / 5 | 3 / 5 / 4 / 4 / 4 | 0 / 0 / 0 / 0 / 0 | 1 / 2 / 0 / 0 / 0 | 0 |
| cealdhlaw 1 / 2 / 3 | 3 / 4 / 3 | 0 / 3 / 0 | 1 / 2 / 0 | 0 |
| fennmynster 1 / 2 | 4 / 2 | 1 / 0 | 2 / 2 | 0 |
| fyrdelf 1 / 2 / 3 | 4 / 3 / 2 | 1 / 1 / 2 | 2 / 0 / 1 | 1 / 0 / 1 |

One seed; the generators draw differently per seed, so these are a sample, not the mean. Detect traps' and Spot's marks are bounded by the first and third columns; Open lock's by the second and fourth (plus the vault chests of hand-made levels).

### 5.4 Counts in the content

- Creatures with an ability Knowledge resists (`grep -l` over `assets/data/creatures/*.toml`, 289 files): 37 mention a drain, 26 a disease, 23 a hold (files overlap).
- Scroll types: 31 (`assets/data/objects/scroll_*.toml`), 28 of them spell scrolls to study.
- Rooms: forge weight 3, scriptorium 5, spring 4, altar 8 (`room_effects.toml`).
- Terrain that slows a step (`terrain.toml`, `move_cost`): water and running water 200, bush 200, shallow water, marsh, reeds, rubble 150, ice (thin, plain, thick) 120; deep water (`lake`) is not walkable at all.

*(not established)*: marks over a whole real run (a player exploring, fighting, and resting for ten levels). No bot plays the game, and a hand script through a generated dungeon breaks with the generator (CLAUDE.md "Golden screenshots" says the same of walks). The sessions above isolate each hook's rate instead; the "typical a level" column multiplies them by a guessed day.

## 6. Proposals, ranked

For each skill not judged *fine* (and Tumble's catch), one to three proposals: what the player does, what the skill changes, how it earns marks, the size (**data** = `skills.toml` and a data file; **small rule** = a new `Use` or `Effect` with its one hook, or a new action on existing machinery; **new system** = state, AI, or terrain rules that do not exist). House rules kept throughout: time is the only penalty, nothing limits retrying, no lock-out on a failed roll, every active use is a row in the one abilities menu (`Ability::use_of`), numbers hidden in the game and stated in the wiki. Concepts borrowed from elsewhere are named so the owner can decide on them.

### 6.1 The top ten, across skills

| # | Proposal | Skill | Size | Why first |
|---|---|---|---|---|
| 1 | **Feint** (§6.2 B1) | Bluff | small rule | gives the two classes known for Bluff a combat use with the thief's Sneak attack behind it; the Surprised status and `flat_footed` already exist |
| 2 | **Struggle free** (§6.3 E1) | Escape artist | data + 2 hooks | the hold check exists and already takes Knowledge the same way; 23 creature files hold |
| 3 | **Fletching** (§6.5 W1) | Woodcraft | small rule | the ranger's signature skill makes the ranger's ammunition; arrows break (`arrows.toml:14`) so the use repeats |
| 4 | **Wading** (§6.4 S1) | Swim | data + 2 hooks | water is everywhere (marsh, streams, flooded rooms) and costs double; the birth sign's `swimmer` shows the hook |
| 5 | **A plain miss by a nimble target is a dodge** (§6.14 Tu1) | Tumble | small rule | repairs the chicken-and-egg: Tumble cannot start under Dexterity 12 |
| 6 | **Sighting trains Spot** (§6.11 Sp1) | Spot | data + 1 hook | the Stealth pattern (a passive mark from plain play) for the skill that has none |
| 7 | **A sprung trap teaches** (§6.8 Dt1) | Detect traps | data + 1 hook | most traps are met by springing them; marks on what actually happens |
| 8 | **Casting trains Spellcraft** (§6.10 Sc1) | Spellcraft | data only | one line; #977 needs an answer to its question 6 either way |
| 9 | **Listen at a door** (§6.9 L1) | Listen | small rule | a deliberate use for a skill that has only accidents |
| 10 | **An anvil in town** (§6.7 M1) | Metalcraft | content only | the cheapest fix there is: a place to practise on purpose |

Then: fold Decipher script into Spellcraft (§6.6 D1), Balance eases Staggered and rough ground (§6.13 Ba1), Wedge a door (§6.5 W2), and the rest by skill below.

### 6.2 Bluff — no use

**B1. Feint** *(small rule)*. Ability **Feint** in the abilities menu (Bluff's row becomes it, as Steal is Steal's), aimed at a direction: `feint:DIR`, a turn. A Bluff check: the player's Presence modifier + Bluff's `feint` points (curve `[[0, 0], [100, 8], [200, 12]]`, at most 20) against the foe's Willpower check, made as Intimidate's threat is (`world/intimidate.rs`, `check::check`). Made: the foe is **Surprised** (`status.toml:786`, `max = 1`: one degree, one blow) — the player's next blow finds it flat-footed (`World::flat_footed`, `surprise.rs:185`): Sneak attack's dice for a thief, and the surprise craft for anyone. Failed: the turn is spent, the foe told nothing new ("The kobold is not fooled."); try again. A creature without a mind (an animated thing, `Content::has_vitals` false) cannot be feinted. Marks: `Use::Feint` 50 % per feint, whatever came of it. Borrowed: "feint" as a Bluff check in D&D 3.5 (the mechanic, not the numbers); the name is plain English. Data `data/feint.toml` (difficulty, the status). Who gains: bard and merchant (known for), half-elf, aristocrat; a thief who takes Bluff up. Not a talent: a skill's own active, as Steal is.

**B2. Talk your way out when caught** *(small rule)*. In `World::steal` when the victim catches the player (`steal.rs:86`, before `caught_stealing`): a Bluff check (as B1's) against the victim's Willpower. Made: "You talk your way out of it": the victim keeps its thing, nothing turns hostile, and the player may try again later (the owner's rule: a victim that caught you may be tried again). Failed: as today. Marks: `Use::Feint` 100 % per such check (the same use: a lie told). Trained only: without Bluff there is no check, as today. The stat side: a Presence mark on a made check (`StatUse::Chat`'s kind, *not established* which; the stat-training design's matrix has Presence thin).

**B3. Talk past a band on the overworld** *(small rule + data)*. A new `Choice::Bluff` ("Talk your way past") on a hostile party met face to face (`encounter.rs:116`, the `Met` outcomes beside Fight and Evade), offered only when the player is trained. A Bluff check against the party's lead type's Willpower (as the lookout reads the lead type's Spot, `encounter.rs:812`): made, the band lets you pass (`How::Pass`); failed, it falls on you as `fight` with the party having noticed you (no worse than fighting would have been). Marks 100 % per try. Rank: after B1 and B2; it depends on the encounter tables having talking bands.

### 6.3 Escape artist — no use

**E1. Struggle free** *(data + 2 hooks)*. `Effect::Escape` (curve `[[0, 0], [100, 8], [200, 12]]`, at most 20) added to the held player's struggle check (`world/hold.rs:84`, `World::struggle`: Strength's modifier plus talents plus Knowledge's `resist_bonus`, against the holder's grip) exactly where Knowledge's bonus is added; `Use::Struggle` 100 % per struggle, whatever came of it (the player's step while held). Drowning grip (`lucky.rs`) is a hold and gets it free. Escape artist's check is Strength's today; a question (§8) is whether a Dexterity-flavoured skill should let the player struggle by the better of Strength and Dexterity when trained.

**E2. Out of the pit** *(small rule)*. The pit's climb costs an action's energy on top of the step (`sidestep.rs:30`, `trap.rs:183`). Trained Escape artist halves the climb's cost from 50 and skips it from 100 (`Effect::Climb`); `Use::Climb` 100 % per climb out. This is #128's pending hook (`docs/dev/skills.md:162`), which names Athletics too: give Athletics the same `climb` mark at 50 % and no effect (the strength to climb is Athletics' `speed`), or the effect too and let both sum, as study does. One hook in the pit's arm.

**E3. Wriggle through** *(small rule; rank last)*. A step into a cell held by a *neutral* creature that would block the way (an NPC in a corridor, a beast that does not move) swaps places with it when trained, by an Escape artist check against its Willpower, as `talk.rs::swap_places` (`talk.rs:498`) already moves the player past a talked-to NPC. Marks 50 %. A convenience with a fitting name; the owner may find it dull, which is why it is last.

### 6.4 Swim — no use

**S1. Wading** *(data + 2 hooks)*. Wading terrain (water, running water, shallow water: `stars::wading`, `stars.rs:39`; marsh by its disease) costs 150–200 % of a step (`terrain.toml`). `Effect::Wade` (percent of the surcharge off: curve `[[0, 0], [100, 60], [200, 100]]`) read in `World::step_price` beside the `swimmer`/`wades_free` branch (`schedule.rs:217–224`: those zero the surcharge; Swim sits under them and never past them); `Use::Wade` 10 % per wading step (the same rate as `travel`), reported where `World::wade` reports the marsh's exposure (`disease.rs:116`) or in `player_moves`. Lizardfolk, berserkers, rangers, and warriors have it; every flooded room and ford trains it.

**S2. Deep water** *(new system)*. Trained Swim makes `lake` (and a campaign's rivers, if any are made `landscape = "lake"`) enterable: each step a Swim check (Toughness's modifier + Swim's points against the water's difficulty + 2 per burden level above Unburdened); made, the step; failed, the turn is spent treading water and the cold takes 1 hit point (never the last, as the plunge's rule, `ice.rs:12–14`). Strained or worse refuses ("You would sink."). Creatures never follow into deep water (aquatic monsters are deferred, #196), so deep water is a refuge and a shortcut: an owner call on whether that is wanted. The overworld's lakes would need the same thought at map scale. Marks 25 % per step. Rank below S1: S1 is a week's work less and proves the hooks.

**S3. The plunge** *(data + 1 hook)*. A Swim check where the player plunges through thin ice (`ice.rs`, `PLUNGE`): made, half the plunge's hit points; marks 100 %. A frill on S1; worth it only with S1.

### 6.5 Woodcraft — no use

**W1. Fletching** *(small rule + data)*. Ability **Fletch** (Woodcraft's row), anywhere, 10 minutes (`cost` in `data/fletch.toml`), needing a blade in hand (any `short_blades` weapon wielded) and a **wooden missile weapon readied** (so the kind follows the launcher: arrows for a bow, bolts for a crossbow; a thrower gets nothing). Makes `count` missiles (dice `1d4` + Woodcraft/25, data) of the plain kind; under Woodcraft 25, `brittle_arrow` instead (an object that exists). A Woodcraft check (Dexterity's modifier + its points) leans the count as Temper's margin leans its outcome (`data/outcomes/fletch.toml`: a bundle, a few, one, a wasted hour). Marks 100 % per batch. Arrows break 30 % of shots (`arrows.toml:14`), so the ranger keeps fletching all run. Borrowed: fletching is common roguelike fare (ADOM's Fletchery is the same idea); named so the owner can say.

**W2. Wedge a door** *(small rule; a door flag in the save, pinned to the version as the policy says)*. Ability **Wedge**, a direction, a turn, on a closed door beside the player: the door is wedged. A creature that opens doors (`docs/dev/doors.md`) cannot; it must kick or break it, against the lock-kick difficulty + Woodcraft's `wedge` points (curve `[[0, 0], [100, 8], [200, 12]]`), as a locked door is kicked (`door.rs:166`). The player opens it as usual (pulls the wedge; the wedge is not an item). A rest tool and a stealth tool, which is what a carpenter's skill should be in a dungeon. Marks 100 % per wedge. Rank after W1.

**W3. Whittle** *(data + a reuse of `temper.rs`; rank last)*. Temper's twin for wood: anywhere, with a blade, 10 minutes, a wooden weapon (bows, staves, clubs, spear shafts: the `materials` of the object, `arrows.toml:7` shows the shape) leaned by Woodcraft through `data/outcomes/whittle.toml` (improved, nothing, flawed, ruined). A duplicate of the forge's shape; cheap, but only worth it if the owner wants wood and metal symmetrical.

### 6.6 Decipher script — too few ways to train, a duplicate

**D1. Fold it into Spellcraft** *(data only; recommended)*. Its one benefit is Spellcraft's own, summed (`learn.rs:137` reads both through `study`); its one mark is a half-chance copy of Spellcraft's. With #977 making Spellcraft the one learning skill, a second skill adding to the same check is noise the player cannot see through. Cut the skill; re-point the bard's, priest's, and wizard's `2d4+4` to Spellcraft (the bard and priest gain a study skill they lack), the two background lines (`04-youth.toml:65`, `:235`) and the scriptorium's `train` (`room_effects.toml:416`) likewise. Old replays' hashes change (a content change; the policy allows it).

**D2. Decipher a labelled scroll** *(small rule; the alternative if the skill stays)*. Ability **Decipher** on a labelled scroll carried (a scroll of a type not yet identified, the list `World::unidentified` already builds for the appraisal service, `service.rs:63`): 10 minutes; an Intelligence check + the skill's points against the type's spell level / 5 + 10 (data); made, the type is learned (`World::learn_scroll`) **without reading it**; failed, time, try again. Marks 100 %. Gives non-casters a way to know a scroll besides reading it blind, Appraise's hourly chance, or a shop's service. Overlap: Appraise's `identify` already covers scrolls (the same list) by the hour; D2 is the on-purpose version. If D2 is kept, D1 is not done and Decipher script's `study` benefit is dropped so it stops duplicating Spellcraft.

**D3. Maps** *(new system; from #196 "alternative unlocks": "rumours at the inn, maps")*. A found map object, deciphered (as D2's check), reveals a hidden location (`World::reveal`, the dev `reveal:` path). Needs the map objects, their loot placement, and the campaign's choice of which locations maps may reveal. Noted for the owner; not ranked.

### 6.7 Metalcraft — too few ways to train

**M1. An anvil in town** *(content only)*. Put an anvil (the `anvil` terrain the forge room uses) in Cēaptūn's smithy and in the Hall's yard: the Temper ability is offered by the terrain to anyone (`room_effects.toml:334`, `offers`), so a dwarf who wants Metalcraft high can go and practise, at ten minutes and a Strength check a try, and a flawed outcome risked. No rule changes. With #975 (walking into an anvil offers Temper) the player finds it by walking.

**M2. Mend** *(data + small rule)*. A second anvil use: an item Temper left **flawed** is made whole again (the outcome's reverse), by the same check; a ruined one never. Marks 100 %. Gives the forge a reason to return to and softens Temper's gamble without removing it (no lock-out: a flaw is undone by time and a check). Depends on how a flaw is stored on the item *(not established here)*.

**M3. Merge with Woodcraft** *(cut; the fallback)*. One **Craft** skill for both if neither earns its place after W1–W3 and M1–M2: dwarves and craftspeople keep their dice. Not recommended while W1 is on the table: a ranger who fletches is not a smith.

### 6.8 Detect traps — too few ways to train

**Dt1. A sprung trap teaches** *(data + 1 hook)*. `Use::Sprung` 100 % when a hidden trap goes off under the player, and 50 % when one goes off under a creature the player sees (`trap.rs:144`, `World::spring_trap`; the victim and the sight are known there). Trained only, as every mark. Most hidden traps are met this way (§5.3: 2–5 a level), so this alone lifts the skill over the 5-mark line.

**Dt2. Taking one apart teaches what it looks like** *(data only)*. `{ on = "disarm", chance = 30 }` on Detect traps: `Use::Disarm` already fires per try (`disarm.rs:55`). One line.

**Dt3. Looking hard counts a little** *(data only)*. `{ on = "search", chance = 3 }`: a search turn marks Detect traps 3 % whatever it finds. One line; keeps searching on purpose from being useless for it. Lower than Dt1 and Dt2 because it is a trickle from an action that is already Search's.

### 6.9 Listen — too few ways to train

**L1. Listen at a door** *(small rule)*. Ability **Listen**, a direction, a turn, at a closed door beside the player: for that turn the player's hearing threshold through that door is lowered by Listen's value / 10 + 2 (the same scale as `hearing`), and what moves beyond is heard as sound markers (the `sound` layer's `f`, `x`, `v` …, the perception snapshot's existing path). Marks 100 % per listen, whatever is heard. A thief's reason to stop at a door. Borrowed: "listen at the door" is in D&D's Listen skill and in most dungeon fiction; the mechanic here is the existing hearing threshold.

**L2. Listening while resting** *(data + 1 hook)*. While the player rests or waits, a sound heard that is not news marks Listen at 5 % a turn (today only news marks, `perceive.rs:443`). A night among wandering creatures then earns a few marks, which a night of nothing does not. The hook is one line beside the news check; the rate is low on purpose (a rest is 13,000 turns).

### 6.10 Spellcraft — too few ways to train

**Sc1. Casting trains it** *(data only)*. `{ on = "cast", chance = 10 }` on Spellcraft: `Use::Cast` already fires per mana spent (`mana.rs:106`), where Concentration takes 25 %. A caster who casts learns to read spells. One line; **#977 should decide it** (its question 6), and this document only puts the number on the table.

**Sc2. Reading a spell scroll trains it** *(data + 1 hook)*. `Use::ReadSpell` 50 % per spell scroll read (`world/scroll.rs:40`, `World::read`, where a scroll's spell resolves). Scrolls are read more than studied; a non-caster reading scrolls gains a little Spellcraft, which under #977 is the way to learning.

### 6.11 Spot — too few ways to train

**Sp1. Sighting trains Spot** *(data + 1 hook)*. `Use::Sight` 25 % the first time the player sees a creature that has not noticed them (`perceive.rs`, where `mark_noticed` runs after `Perception::compute`, `docs/dev/surprise.md:32`; the creature's `unaware` is `World::unaware`, `surprise.rs:100`). The passive counterpart of Stealth's `sneak`: plain play trains it, a watchful player who sees things first trains it more. The one hook is where the sighting is already known; no draw of the gameplay stream (the training stream, as every mark).

**Sp2. Searching counts a little** *(data only)*. `{ on = "search", chance = 3 }` on Spot, as Dt3 for Detect traps. A trickle; rank below Sp1.

### 6.12 Knowledge — niche

**K1. What you know of it** *(small rule, information)*. Look mode's description of a creature says, in words, what it resists and what it is immune to, when Knowledge stands above its level (the same read as `resist`, `Actor::skill_effect_above`): "You know the troll fears fire." Numbers stay hidden; words as the wiki's resistances table says them. No marks (knowing is not practising). The policy against info-only *talents* does not bar a skill's information read; the owner may still think it dull.

**K2. Raise the curve** *(data only)*. `resist` is `[[0, 0], [5, 1], [40, 4]]`, at most 6: a wizard with Knowledge 60 gets +2 against a level-20 drainer and nothing against anything of level 60. Make it `[[0, 0], [5, 2], [40, 6]]` (the cap stands), or read it at the value and let the creature's level subtract half. Tuning, not a use.

### 6.13 Balance — too few ways to train

**Ba1. Rough ground and Staggered** *(data + 2 hooks)*. Two things in its own description ("bad ground"): (a) `Use::RoughStep` 10 % per step onto ground that slows but is not water (bush, reeds, rubble, ice: `move_cost > 100` and not `wading`), with `Effect::Footing` (percent of the surcharge off, as Swim's wading, S1, for land) read in `step_price` beside it; (b) Staggered (`status.toml:415`, `cost = 150`): `Effect::SteadyEase` takes Balance's percent off the status's extra cost, as Initiative's `surprise_ease` does for Surprised (`surprise.rs:287`), and `Use::Staggered` 20 % per action taken while Staggered. Both are the burden pattern again, on things that happen in every dungeon.

**Ba2. Merge into Athletics** *(cut; the fallback)*. If Ba1 is not wanted, Balance's one effect is a burden ease that Athletics' `speed` already resembles; fold `burden` into Athletics and re-point the ascetic's and warrior's dice. Not recommended with Ba1 on the table: the ascetic is known for Balance.

### 6.14 Tumble — fine, with a catch

**Tu1. A plain miss by a nimble target is a dodge** *(small rule)*. In `combat::strike` (`combat.rs:555`): when `total >= BASE_DEFENCE` and `weight == 0`, attribute the miss to `Category::Dexterity` instead of `None` (the attack would have hit a motionless target; nothing but the target's movement made it miss). Then `fight.rs:881` fires `Use::Dodge` for a Dx-11 thief with Tumble 12, and the Dexterity stat mark with it (`StatUse::Dodge`, `stat_training.rs:188`), which the stat-training design wanted ("Dexterity nothing from …" is its thin row). A natural 2 stays a plain miss. Nothing else reads the attribution but the messages ("You dodge" rather than a plain miss: a wording change to check in the combat snapshots).

### 6.15 Appraise and Negotiation — niche, by design

**A1. Freshness** *(data + 1 hook; from #196's first line, owner 2026-10-19: "some skill (and spells?) to be able to reveal more information")*. `Effect::Freshness` on **Appraise** ("what a thing is made of") and on **Survival** ("living off the land"), summed where both are trained: the eat list and look mode say a corpse's or food's stage in words (`rot::stage`, which the player is never shown today) when a check of the sum against the food's age passes; the dev `actors` dump already prints the stage. Marks: `Use::Judged` 10 % per food looked at, Appraise's. The one piece of the deferred idea that is a skill's; the spell half belongs to #980.

**N1. Services haggle a little** *(data + 1 hook)*. `favour` applies to a healer's and a sage's price (`service.rs:24` keeps temples flat), and `Use::Deal` fires on a service paid. Negotiation then trains in every town visit, not only the shop's.

Neither is urgent: both skills do what their classes need and nothing more, and *niche* is a verdict the owner may keep.

## 7. Work in flight that this touches

| Work | What here depends on it |
|---|---|
| **#977** (no caster level; Spellcraft the one learning skill) | §6.10 Sc1 and Sc2 are its question 6's answer; §6.6 D1 (folding Decipher script into Spellcraft) is easier before its starting points are set, so its design should take D1 as given or refuse it. Nothing here reads caster level |
| **#980** (spell types and new spells) | A1's spell half (a spell that tells freshness) is its; a web or hold spell on the player would be E1's. Nothing else |
| **#963** (Skill focus, +25 to a trained skill) | every effect read through `skill_effect` gains it; the four no-use skills gain nothing from it until they have a benefit, which is one more reason to give them one (a Skill focus on Bluff is a wasted talent today) |
| **#975** (walking into an altar, spring, or anvil offers its action) | M1's anvil in town is found by walking; the `offers` data is the one place both read |
| **#952 / #966** (one rank per talent) | no talent here; a proposal needing one says so (none does) |
| **#196** (deferred ideas) | A1 takes the freshness line; D3 the maps line; S2 is bounded by the aquatic-monsters line |

## 8. Open questions

1. **Cut or build the four?** Bluff, Escape artist, Swim, and Woodcraft could instead be removed, their dice re-pointed (bard → Negotiation or Music; ranger → Survival; thief → Tumble; warrior → Athletics). **Recommendation: build B1, E1, S1, W1** (the top of §6.1); they are a small rule each on hooks that exist, and four classes are known for two of them.
2. **Decipher script: fold (D1) or decipher (D2)?** **Recommendation: fold**, unless the owner wants a scroll-reading use for non-casters, in which case D2 without the `study` benefit.
3. **Deep water (S2)?** A refuge no creature follows into, until aquatic monsters exist. **Recommendation: not yet**; S1 first, S2 when #196's aquatic line is picked up.
4. **Escape artist's stat.** The struggle is a Strength check (`hold.rs:84`). Should a trained Escape artist struggle by the better of Strength and Dexterity? **Recommendation: yes** (the skill is about slipping, not breaking), as one line in E1.
5. **Heal's one-try rule.** `heal.rs:24–26`: a poison or a disease "may not be tried again". That is a lock-out on one failed roll, which the house rule forbids elsewhere (steal, disarm, pick). Not this document's to change; **recommendation: raise it as its own issue** (a retry after a rest, say).
6. **The "typical marks a level" column** is the probe's day model times measured rates, not a played run. **Recommendation: accept it as an order of magnitude**; a bot that plays is a bigger tool than this question is worth.
7. **Tu1's wording.** Attributing a plain miss to Dexterity changes the miss message for unarmoured targets. **Recommendation: do it**, and read the combat snapshot diff.
8. **Information reads (K1).** Is a skill's information use (what a creature resists, in words) wanted, given the owner's rule that the game stays vague? **Recommendation: yes in words, never numbers**; the owner decides.
