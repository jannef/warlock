# Warlock — Skill uses: what each skill does, how it trains, and what the gaps need

**Built; this file is history.** Current behaviour is in spec §8 "Skills" and `docs/dev/skills.md`.

Design document, revised after the owner's approval of draft PR #984 (#973; §8 records the decisions). The investigation (§1 to §5) was written 2026-11-05 against `origin/main` at 24dc95dc, and checked again at d562ceb5 for this revision. An investigation first: nothing here is implemented, and no rule or data changed. Every effect and every mark below was checked in the code, not read off the skill's description: the reading function and the firing hook are named with file and line (the line numbers of §1 to §5 are those of 24dc95dc; §6 names functions and files). Where a number was measured, §5 says how. What could not be established is marked *(not established)*.

**The owner's request** (#973): *"investigate whether we have enough ways to train skills, identify which skills have no uses yet, and propose uses for skills."*

**§1 to §5 are the state of the game before this work; §6 is what gets built**; each group in §6.1 becomes an implementation issue.

## Contents

1. Summary: every skill in one table
2. How a skill grows, in numbers (what "too few" means)
3. Which skills have no use
4. Are there enough ways to train?
5. Measurements: how the numbers were produced
6. What gets built
7. Work in flight that this touches
8. Decisions

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
| Decipher script | `study`: 0–20 off a study check (`world/learn.rs:137`) — **the same benefit as Spellcraft's**, summed | `study` 50 % per scroll studied (`learn.rs:250`; a scriptorium doubles it, `room_effects.toml:416`) · 0–2 (a spell scroll found a level at most; 30 scroll types exist) | bard, priest, wizard; bg ×2 | too few ways to train (and a duplicate, §6.6 removes it) |
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
| Spellcraft | `study`: 0–20 off a study check (`learn.rs:137`) | `study` 100 % per scroll studied (`learn.rs:250`) · 0–3 (as Decipher script's) | **wizard**, **witch**; priest (opt); bg ×2 | too few ways to train (#989 answers it: casting marks it) |
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
   - **Spellcraft** and **Decipher script** train only by studying a scroll (`learn.rs:250`), one a level at best, and share one benefit (`study`). #977 settled it for Spellcraft: casting marks it (#989).
   - **Balance** trains only while the player's load slows them (`mod.rs:1256`). It *can* be practised (carry a boulder, walk), but at 5 % per action it is the slowest grind in the game and nobody is told it works that way.
3. **Cannot be practised on purpose by design** (not counted as gaps): **Initiative** (being caught unawares) and the passive halves of Knowledge (`resist`) and Appraise (hourly). Their marks are certain (100 %) when they fire, which is the right shape for a passive skill.

**One hidden catch in a "fine" skill**: Tumble's mark fires only when a miss is *attributed* to the Dexterity or Dodge category (`fight.rs:873–881`). A miss is attributed by a draw weighted by the defender's positive bonuses (`combat.rs:555–559`, `Defence::why_missed`, `combat.rs:224`): a target with **no** Dexterity bonus (Dexterity 11 or less) and Tumble under 20 (no Dodge bonus) has weight 0 → every miss is "plain" → **Tumble never trains**. The code is the proof (§5.2); the sessions agree without proving it: a human thief (Dx 11, Tumble 12) earned 0 marks in a fight where a halfling ascetic (Dx 15, no armour) and an elf thief (Dx 19) earned 5 each, but so did a halfling thief at Dx 14, and a twelve-turn fight is too short to tell luck from the rule. And an armoured fighter's misses are drawn as "armour" in proportion to its bonus (the jerkin's 1 against Dexterity's 2 for that halfling), so a warrior in mail trains Tumble rarely however nimble. Tu1 in §6.14 addresses the first.

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

## 6. What gets built

For each skill not judged *fine* (and Tumble's catch), what the player does, what the skill changes, how it earns marks, and the size (**data** = `skills.toml` and a data file; **small rule** = a new `Use` or `Effect` with its one hook, or a new action on existing machinery; **new system** = state, AI, or terrain rules that do not exist). This section is the design as the owner approved it (§8); every entry is to be built unless it says otherwise. Time is the only penalty and retrying is open, except where the owner set otherwise (Metalcraft's rising difficulty and its destruction, §6.7). Numbers are hidden in the game and stated in the wiki. An active use is a row of the one abilities menu; a check the owner made automatic has no row.

The owner's decisions override the talent rules and the house rules where they differ. Should a load check refuse something here, the implementing PR adds an exemption for it and says so.

### 6.1 The work in order

| # | Work | Entries | Size | Comes after |
|---|---|---|---|---|
| 1 | Heal: the one-try rule goes | §6.16 H1 | small rule | — |
| 2 | Tumble: one plain miss in four is a dodge | §6.14 Tu1 | small rule | — |
| 3 | Marks from plain play: Spot, Detect traps, Listen | Sp1, Sp2, Dt1, Dt2, Dt3, L2 | data + hooks | — |
| 4 | Escape artist: the automatic check, and out of the pit | E1, E2 | small rule | — |
| 5 | Feint | B1 | small rule | — |
| 6 | Talking past on the overworld | B3 | small rule + data | 5 |
| 7 | Wading and the plunge | S1, S3 | data + hooks | — |
| 8 | Deep water: the terrain and the swimming | S2, rules | new system | 7 |
| 9 | Deep water: the generators | S2, placement | content + mapgen | 8 |
| 10 | The knife and Fletching | W1 | small rule + data | — |
| 11 | Metalcraft: the one action | M0 | new item state | — |
| 12 | An anvil in Cēaptūn | M1 | content | 11 |
| 13 | Decipher script: reading a scroll | D0 | small rule | #987 |
| 14 | The Decipher ability | D2 | small rule | 13 |
| 15 | Listen at a door | L1 | small rule | 3 |
| 16 | Wedge a door | W2 | small rule | 10 |
| 17 | Whittle | W3 | small rule | 10, 11 |
| 18 | Balance: rough ground and Staggered | Ba1 | data + hooks | 7 |
| 19 | Knowledge: words, and the curve | K1, K2 | small rule + data | — |
| 20 | Appraise and Negotiation: freshness, services | A1, N1 | data + hooks | — |
| 21 | Wriggle through | E3 | small rule | 4 |

Not built, each for a reason its entry gives: B2 (the owner), D1 and D3 (the owner, q2), M2 and M3 (replaced by M0), Ba2 (a fallback Ba1 makes needless), Sc1 and Sc2 (#989 owns Spellcraft's training).

### 6.2 Bluff — no use

**B1. Feint** *(small rule)*. Ability **Feint** in the abilities menu (Bluff's row becomes it, as Steal is Steal's), aimed at a direction: `feint:DIR`, a turn. A Bluff check: the player's Presence modifier + Bluff's `feint` points (curve `[[0, 0], [100, 8], [200, 12]]`, at most 20) against the foe's Willpower check, made as Intimidate's threat is (`world/intimidate.rs`, `check::check`). Made: the foe is **Surprised** (`status.toml`, `max = 1`: one degree, one blow), so the player's next blow finds it flat-footed (`World::flat_footed`): Sneak attack's dice for a thief, and the surprise craft for anyone. Failed: the turn is spent, the foe told nothing new ("The kobold is not fooled."); try again. A creature without a mind (an animated thing, `Content::has_vitals` false) cannot be feinted. Marks: `Use::Feint` 50 % per feint, whatever came of it. Data `data/feint.toml` (difficulty, the status). Who gains: bard and merchant (known for), half-elf, aristocrat; a thief who takes Bluff up. Not a talent: a skill's own active, as Steal is. Bluff's description in `skills.toml` loses "talking your way out": "Lying with a straight face, feinting, and talking your way past."

**B2. Talking out of being caught: not built** (the owner: "we do not want the complications that come from talking way out when caught"). No Bluff check in `World::steal`, at a shop, or anywhere a caught player answers for it. A caught thief stays caught, as today.

**B3. Talk past a band on the overworld** *(small rule + data)*. A flag on the encounter entry, `talk_past = true` in `[[table.entry]]`. The choice is offered only for a flagged entry, only in the `Met` outcome (the party has seen the player, face to face), and only to a player trained in Bluff: `Choice::Bluff`, id `bluff`, label "Talk your way past", between Evade and Fight. `Outcome::choices` is a constant today, so the list gains a step that adds it (`encounter.rs`); the dumps' `choices` and the dialog read that list. Known parties take it as any party of the entry does.

- **The check**: a contest as in B1: the player's Presence modifier, what talents add to its checks, and Bluff's `feint` points, against the party's lead type's Willpower (`World::lead_of`, as the lookouts read the lead type's Spot). Gameplay stream, as `evade`.
- **Made**: "You talk your way past." The party lets the player go as `Choice::Pass` does; the try takes a `chat` action's price.
- **Failed**: "They are not fooled." The fight opens as `Choice::Fight` opens it (`How::Fight`, `ActionKind::Stairs`'s price, the same positions): no worse than standing to fight.
- **Marks**: `Use::Bluffed` 100 % per try (a mark rule `{ on = "talk_past", chance = 100 }` beside Feint's).
- **Load checks** (`content/encounter.rs`): `talk_past` only on a hostile entry with a `group`. Never on a `draw` (what is drawn may be anything), never on a peaceful or wildlife entry.

**The entries flagged**, all in `assets/campaigns/ascmearc/data/encounters/`, conservatively: people with something to gain from letting a stranger walk on.

| Entry | Table | Why |
|---|---|---|
| `road_footpads`, `road_footpads_night`, `woods_footpads` | roads, woods | robbers on foot: a purse is what they want, and a good story is worth hearing |
| `vale_footpad` | vale | one robber, who would as soon not fight |
| `ascdun_duergar` | ascdun | prospectors at their work, not out for blood |
| `fen_cultists` | fen_cult | robed folk on a road, who may take a stranger's word that they are one of them |

**Not flagged**: `wystan_archers` (their quest's fight, set on the road for it, and hunting); kobolds, goblin raiders, and orc raiding parties (`*_kobolds`, `*_goblins`, `orc_raiders*`, `deep_orcs`: out to kill and rob whoever comes, and no talk stops them); every beast (`*_rats`, `*_wargs`, `*_spiders`, leeches, frogs); the dead and the things of the fen (skeletons, wights, ghouls, `fen_drowned`, `fen_bog_body`, `fen_wisp`, `deep_skeletal_wolves`); `ascdun_troll`; every `draw` entry (`ascdun_fyrdelf`, `deep_wulfeshol`, `downs_strays`). Peaceful and wildlife entries already pass by freely.

### 6.3 Escape artist — no use

**E1. The automatic check** *(data + one hook; the owner: "Struggle free should not be an action")*. No action, no ability row, no key. When a status the skill can fix prevents or slows what the player tries to do, the check happens as part of that attempt.

**What it can fix**: bonds, the statuses a body can slip. In `status.toml` today three block a step, and all three are that: **Held** (`hold`, its grip), **Webbed**, **Pinned**. They are marked by data: a status with a `hold` counts, and any other carries a new field `escape = DIFFICULTY`; Webbed and Pinned take `escape = 13`, Held's own 13 and its holder's `difficulty` stand. It cannot fix a slowed mind or body: Staggered (Balance's, Ba1), Stunned, Surprised (Initiative's), Transfixed, Sluggish, and Drowsy stay out. So no status that merely slows is marked today; the hook covers that case anyway because the owner named it, and a later status of bonds that slows takes only the field.

**The one hook** is `World::struggle` (`world/hold.rs`), already asked in `player_acts` before `refuse_blocked` (`world/mod.rs`). It looks at the action's kind (`kind_of`), and at the first status blocking that kind:

| Status, player | The check | Made | Failed |
|---|---|---|---|
| Held, untrained | the hold's stat (Strength), talents, Knowledge's `resisting`, against the grip: today's rule, unchanged | the hold ends, the action goes ahead at its price | a step's price, "You struggle, but the ghast holds you fast." |
| Held, trained | as above with the **better of Strength and Dexterity** (each with what talents add to that stat's checks; the owner, q4) + `Effect::Escape` | the same | the same |
| Webbed or Pinned, untrained | none: refused by `refuse_blocked`, no time, until time eases it, as today | — | — |
| Webbed or Pinned, trained | better of Strength and Dexterity + `Effect::Escape`, against its `escape` | the status ends with its own `end` line, the action goes ahead at its price | the action's price, "You struggle, but you are still caught in a web." (the status's `seen`) |
| A marked status that slows (none today), trained | as above | it ends, and the action costs its plain price | no extra cost: the action goes ahead at the slowed price it already had |

- `Effect::Escape`: curve `[[0, 0], [100, 8], [200, 12]]`, at most 20.
- **The cost**: two draws of the gameplay stream per check. A failure costs the turn because a free failure could be repeated at no time at all; it costs no more than the wait the player would otherwise have had to make.
- **The marks**: `Use::Escape` 100 % per check, whatever came of it; `StatUse::Struggled` as `struggle` fires it today.
- A second status blocking the same action asks on the next attempt; `refuse_blocked` refuses this one.

**E2. Out of the pit** *(small rule)*. The pit's climb costs an action's energy on top of the step (`sidestep.rs`, `trap.rs`). Trained Escape artist halves the climb's cost from 50 and skips it from 100 (`Effect::Climb`); `Use::Climb` 100 % per climb out, read in the pit's one arm. This is #128's pending hook (`docs/dev/skills.md`), which names Athletics too: Athletics takes the same `climb` mark at 50 % and no effect.

**E3. Wriggle through** *(small rule)*. A step into a cell held by a *neutral* creature that refuses to make way (an NPC in a corridor, a beast that does not move) swaps places with it when trained, by an Escape artist check (Dexterity's modifier + `Effect::Escape`) against its Willpower, as `talk.rs::swap_places` already moves the player past a talked-to NPC. Made, the swap; failed, the step's price. Marks: `Use::Wriggle` 50 % per try. Last in the order.

### 6.4 Swim — no use

**S1. Wading** *(data + 2 hooks)*. Wading terrain (`stars::wading`, `world/stars.rs`) costs 150–200 % of a step (`terrain.toml`). `Effect::Wade` (percent of the surcharge off: curve `[[0, 0], [100, 60], [200, 100]]`) is read in `World::step_price` beside the `swimmer` and `wades_free` branch: those zero the surcharge, Swim sits under them and never past them. `Use::Wade` 10 % per wading step, reported where `World::wade` reports the marsh's exposure (`disease.rs`). Lizardfolk, berserkers, rangers, and warriors have it; every flooded room and ford trains it.

**S2. Deep water** *(new system; the owner, q3: "implement deep water, but do not make it as common as spills we have now. always ask confirmation before diving into deep water")*.

**The terrain**: a new `deep_water` in `terrain.toml`, the darker colours of `lake`, `move_cost = 200`, `step_noise = 200`, `stains = false`, transparent, `walkable = false`, and a new field `swim = true`. `walkable = false` is what keeps it out of everything else: paths, travel, running, spawning, teleports, blinks, and landings treat it as they treat any impassable cell. `swim` lets `player_moves` take the step for the one mover it permits. The overworld's `lake` stays as it is: nobody swims at map scale. Deep water does not freeze (no `freeze`): frost neither bridges it nor traps a swimmer.

**Asking**: every step from anything else into deep water opens the one dialog first, `input::deep_water_dialog` beside `trap_dialog`: title "Deep water", "Really dive into the deep water?", "Dive in" and "Stay back", with Stay back selected first and Esc's, no time taken. No setting turns it off. A step from deep water to deep water does not ask: the player is already in. The core action `move:DIR` needs no dialog, as with a trap.

**Who**: trained in Swim only. Untrained, the step is refused with no time and no dialog ("You cannot swim."). Strained or worse is refused at the shore with no time and no dialog ("You would sink.").

**A step in it**: a Swim check, Toughness's modifier + talents + `Effect::Swim` points (curve `[[0, 0], [100, 8], [200, 12]]`, at most 20) against `difficulty = 11` + 2 per burden level above Unburdened (`data/swim.toml`). Made: the step, priced by `step_price` (S1's Wade, Swimmer, and Undertow all apply). Failed: the turn is spent treading water, the player does not move, and the cold takes 1 hit point, cold-typed, through `hurt` with `Harm::Spared` (never the last, the ice's rule). So at Toughness and burden even: 69 % unburdened, 54 % one level up; +4: 90 %, 81 %. Marks: `Use::Swim` 25 % per step in it, whatever came of it. Two draws of the gameplay stream a step.

**Drowning**: none. The cold never takes the last hit point, nothing drags the swimmer down, and a swimmer who grows Strained in the water can still swim, harder.

**Gear**: nothing is lost or dropped by going in. Burden is all it asks. In the water nothing can be dropped or picked up ("It would sink."), and resting, searching, and digging are refused with no time. Fighting, firing, casting, reading, and waiting go on as anywhere; a wait asks no check.

**Carried light**: going in puts out the ready light through `World::snuff_light` (the altar's snuff; "Your lantern goes out."); a light cannot be lit while the player is in deep water ("The water would only put it out."). Spell light and other statuses are untouched.

**Creatures**: none enter it. They path around it as around any impassable cell, wait at the shore, and shoot and cast from there; a creature beside the swimmer may strike, and the swimmer it. The player's allies stay ashore, and take up following at the shore again. So deep water is a refuge and a shortcut, as q3 was told; aquatic monsters stay deferred (#196).

**Placing it** *(rarer than today's spills: every `caves` floor has 1–3 shallow pools a screen, `caves.rs::pools`)*:

- **Caves**: one `caves` floor in six has one deep pool: a new stream `Rng::deep_water(run_seed, level)`, one `chance` per floor, so no other draw of the map stream moves. On those floors, once the map and its stairs are laid, a blob of 40–70 free floor cells becomes `water` as `pools` grows them, and every cell of it whose eight neighbours are all of it becomes `deep_water`. The shore stays wadeable, so every staircase stays reachable without swimming (a test over seeds). What `population` and the traps place later sees fewer cells on those floors only.
- **Local areas**: where an overworld `lake` borders the area (`mapgen/local/water.rs`, `setting.lakes`), still water at `depth > 3` from its shore is `deep_water`. No new draw. Ponds and rivers stay wadeable.
- Nowhere else; no vault, no special room, no hand-made level of the Ascmearc. Water that stands in a room stays shallow.

**S3. The plunge** *(data + 1 hook)*. A Swim check where the player plunges through thin ice (`ice.rs`, `PLUNGE`): the same `Effect::Swim` + Toughness against 11; made, half the plunge's hit points; marks 100 %.

### 6.5 Woodcraft — no use

**W1. Fletching** *(small rule + data; the owner's: a universal action, a knife in the pack, the ranger far better at it)*. Ability **Fletch**, `fletch` in action text: anyone may, anywhere, trained in Woodcraft or not.

*What it needs*, each the row's reason in the abilities menu when missing, and refused by the rules in the same words with no time:

- **A knife carried**, in the pack, worn or wielded or neither. `Unavailable::NoKnife`, "you have no knife". What counts is below.
- **A launcher ready** in the missile weapon slot whose `launches` is listed in `data/fletch.toml`: arrows for a bow, bolts for a crossbow. `Unavailable::NoLauncher`, "you have no bow or crossbow ready". A sling or a thrower gets nothing. The launcher's own material does not matter (a horn composite bow counts).

*What it does*: `ActionKind::Fletch`, ten minutes (`seconds = 600` in `data/fletch.toml`) by `keep_at`, as Temper's turns: interrupted, the time is spent and nothing is made. No material is consumed. Then a Woodcraft check: Dexterity's modifier + talents + `Effect::Fletch` points (curve `[[0, 0], [100, 8], [200, 12]]`) against `difficulty = 12`, whose margin scales `count` (dice `1d4` + Woodcraft's value ÷ 25):

| Margin | Made |
|---|---|
| 0 or more | the whole count |
| −1 to −3 | half, rounded up |
| −4 to −6 | one |
| below −6 | none: wasted time |

*Of what*: under Woodcraft 25, `brittle_arrow` or `brittle_bolt`; from 25, plain `arrow` or `bolt`. The brittle kinds are new objects (the design's earlier "an object that exists" was wrong: `brittle_arrow` is only a test fixture, `shoot_tests.rs`): `breaks = 100`, value 0, no `kind` or `frequency`, as `flaming_arrow` (never in loot or shops). Marks: `Use::Fletch` 100 % per batch, for those trained. An arrow breaks 30 % of shots (`arrows.toml`), so a bow's owner keeps fletching.

*The ranger's*: **no ranger ability touches fletching today** (nothing in `07-ranger.toml`, `talents/`, or `ability.rs`; `fletcher` is only a shop kind); the design's Fletching was Woodcraft's row. The owner's "change the ranger ability" therefore adds one: the talent **Fletcher**, `tree = "class"`, `granted = true`, `{ level = 1, talent = "fletcher" }` first in the ranger's table, a new effect kind `fletcher` (`TalentMods::fletcher`). Nobody else has it, and no other way gives it. With it:

- half the time: five minutes;
- twice the count, after the check's margin;
- +4 to the check;
- plain arrows and bolts at any Woodcraft, never brittle.

So a level-1 ranger (Woodcraft 12–22, Dexterity's modifier + 4) makes about five plain arrows in five minutes, and fails rarely; a level-1 warrior with no Woodcraft (62 % at Dexterity even) makes about two brittle ones in ten.

**Knives**: the data has no `knife` object and no `knife` category. A weapon's `category` is its weapon skill: `daggers`, named "Daggers and knives" (`weapon_skills.toml`), holds `dagger` and `stiletto`; `short_blades` holds `short_sword` and `scimitar`, and those do not count. `races.toml` already calls the goblin's stiletto "a sneaky knife" and the witch's kit calls her dagger "a ritual knife"; `cutlery` is a household thing with no category. So:

- **The rule**: a knife is any carried stack whose object has `category = "daggers"` (`Content::is_knife`). Splitting the category would split a weapon skill; the requirement is the category.
- **The new object**: `knife` (`objects/knife.toml`): `class = "melee_weapons"`, `category = "daggers"`, `kind = "weapon"`, `slot = "main_hand"`, `light_weapon`, `finesse`, damage `1d3`, weight 200, value 5, `materials = ["steel"]`, `made_of = ["metal", "bone", "stone"]`. The weaponsmith stocks it by its class.
- **Rangers start with it**: `"knife"` in the ranger's `kit`, carried, not wielded. Thief, bard, witch, and merchant already carry a dagger; goblins a stiletto.

**W2. Wedge a door** *(small rule; a door flag in the save, pinned to the version as the policy says)*. Ability **Wedge** (Woodcraft's, for those trained), a direction, a turn, on a closed door beside the player: the door is wedged. A creature that opens doors (`docs/dev/doors.md`) cannot; it must kick or break it, against the lock-kick difficulty + Woodcraft's `wedge` points (curve `[[0, 0], [100, 8], [200, 12]]`), as a locked door is kicked (`door.rs`). The player opens it as usual (pulls the wedge; the wedge is not an item). Marks 100 % per wedge.

**W3. Whittle** *(small rule)*. Temper's twin for wood, on the same rules so that one item has one state: M0's check, attempts, improvements, names, and value, on a weapon whose main part is `wood` (bows, staves, clubs). Ability **Whittle**, `whittle:LETTER`, anywhere, with a knife carried (`Unavailable::NoKnife`), ten minutes; Dexterity's modifier + talents + Woodcraft's `whittle` points (curve as `temper`) against M0's difficulty. Marks `Use::Whittle` 100 % per attempt. The earlier outcome table (improved, nothing, flawed, ruined) goes as Temper's does.

### 6.6 Decipher script — too few ways to train

**D0. Reading a scroll is a check for those who cannot simply read it** *(small rule; the owner, q2: "decipher script for scroll use for non-casters. casters with sufficent spellcraft can bypass the check")*. Decipher script stays; D1 (folding it into Spellcraft) is not built.

**The new rule, plainly**: reading a scroll asks one of two things. A character trained in Spellcraft whose **learning** is at least the scroll's spell level reads it at once. Everyone else makes a Decipher script check. This changes `docs/design/no-caster-level.md`, whose §1 and §4 say reading a scroll to cast works for anyone: it still needs no Spellcraft and is refused nobody, but it may now fail. The owner's words here win.

- **"Enough" Spellcraft**: learning (`Actor::skill_effect(content, Effect::Learn)`, at least 1, trained only: #987's reading) ≥ the scroll's spell level. Exactly what a character could learn for certain, they can read for certain; no draw. So a level-1 wizard (learning 2–4) reads magic missile at once and makes the check for fireball until their Spellcraft grows. A scroll with no spell (restoration) counts as level 0, so any Spellcraft bypasses it.
- **The check**: `World::read`, after its refusals and before `use_up`: Intelligence's modifier + talents + Decipher script's `decipher` points (curve `[[0, 0], [100, 8], [200, 12]]`), against `10 + spell level ÷ 5` (`data/decipher.toml`). Untrained, it is the modifier alone: a real chance, never a refusal. Two gameplay draws, only when asked.
- **Made**: the scroll reads as today (Scroll adept, Scroll savant, an unknown scroll identified).
- **Failed**: "The writing swims before your eyes." A read's price; the scroll is kept; a scroll under its label stays unknown; try again.
- **The marks**: `Use::Decipher` 100 % per check. Reading gives Spellcraft no mark (#989's own test says so).
- **What goes**: Decipher script's `study` benefit and its `study` mark (`skills.toml`), so `study_odds` reads only Spellcraft's tenth, and it no longer duplicates Spellcraft. Its `train = { decipher_script = 200 }` in the scriptorium stays: marks twice as often there.

| Spell level | Difficulty | Modifier +0 | +4 | +8 |
|---|---|---|---|---|
| 0–4 | 10 | 75 % | 93 % | 99 % |
| 11 (fireball) | 12 | 62 % | 85 % | 98 % |
| 35 | 17 | 25 % | 54 % | 81 % |
| 56 | 21 | 7 % | 25 % | 54 % |
| 70 | 24 | 1 % | 10 % | 31 % |

**D2. Decipher a labelled scroll** *(small rule)*. Ability **Decipher** (Decipher script's row, for those trained) on a labelled scroll carried (a scroll of a type not yet identified, the list `World::unidentified` already builds for the appraisal service, `service.rs`): 10 minutes; D0's check against D0's difficulty; made, the type is learned (`World::learn_scroll`) **without reading it**; failed, time, try again. Marks 100 %. Overlap: Appraise's `identify` covers scrolls by the hour; D2 is the on-purpose version.

**D3. Maps: not built.** "Hints in ancient language" and the like are the owner's later ideas, and are not designed here; #196 keeps the maps line.

### 6.7 Metalcraft — too few ways to train

**M0. Temper: one action on a metal item** *(new item state; replaces what Temper does at the anvil)*. The owner: "chance to improve or ruin metal items. first fail -> "damaged", improving "damaged" -> either completely destroys or removes damaged condition. just one action that does both."

The action keeps its names (`Action::Temper`, `temper:LETTER`, `Ability::Temper`, `Offer::Temper`, #975's walk into an anvil, `ScreenName::Temper`) and its place, time, and interruption: beside an anvil, ten minutes by `keep_at`, interrupted it does nothing to the item and counts no attempt. Everything after the ten minutes is new.

- **What may be worked**: a weapon, launcher, armour, or shield (`temperable`'s kinds) whose **main part** (the first part of `Stack::parts`, so a bone dagger is bone) is of material `group = "metal"`. Refused with no time otherwise ("Only metal can be worked at an anvil."). So leathers, wooden bows, and bone or stone arms are out (W3 has the wood). A spirit weapon is refused. The Temper list shows only what is accepted.
- **The check**, one per attempt: Strength's modifier + talents' Strength checks + Metalcraft's `temper` points (curve unchanged), against `13 + 2 × attempts` (`data/temper.toml`: `seconds = 600`, `difficulty = 13`, `per_attempt = 2`, `most = 3`). A natural 2 always fails, a natural 24 always succeeds. **Every attempt on the item, made or failed, counts, for the item's whole life**, so each is harder than the last, through damage and repair too. Two gameplay draws.

| | Item's state | Made | Failed |
|---|---|---|---|
| Improve | not damaged, under 3 improvements | one improvement: a random aspect it has | **damaged**: all its improvements are gone |
| Repair | damaged | the damaged condition is gone: a plain base item with its attempts, to start over | **destroyed**: the item is gone, as `Work::Ruin` removes it today |
| Refused, no time | 3 improvements | "It can be made no finer." | — |

**Improvements**, drawn evenly on the gameplay stream among the aspects the base item has, and the same one may come twice:

| Aspect | Who has it | Each improvement |
|---|---|---|
| Damage die | anything with an `attack` | +2 sides on its dice: `1d8` → `1d10`, `1d9` → `1d11`, `2d6` → `2d8` |
| To hit | anything with an `attack` | +1 |
| Plus damage | anything with an `attack` | +1 |
| DV | a non-zero `armour` (armour, shields, a parrying weapon) | +1 |
| PV | a non-zero `protection` | +1 |

**Names**: 1 **Fine**, 2 **Marvellous**, 3 **Masterwork**, ahead of the rest of the name (`magic/name.rs`): "a Fine long sword", "a Marvellous Sharp long sword of the Bear", "a Masterwork chain mail shirt", and **"a damaged long sword"**. They show whatever the item's identification, and tell the worker what happened. The numbers show as any item's do (`Numbers`); attempts are hidden in the game and printed by the dev `actors` dump.

**Damaged** does nothing more than the owner said: all improvements gone, and it must be repaired before it can be improved again. A damaged item fights and protects as its base does, so stopping there is a real choice. (The owner named no combat penalty; this one is decided in his direction, §8.)

**Saved state**: a new `Stack` field beside `magic` and `parts`: `worked: Option<Box<Worked>>`, `Worked { improved: Vec<Aspect>, attempts: u8, damaged: bool }`, `skip_serializing_if` none, `crate::magic::worked` for its pure rules. A stack with any is unlike any other, so a worked item never stacks; a stack of several is worked one at a time (`set_apart`, as today). Saves are pinned to the version: no shim.

**What reads it**: one place for the numbers, `Mods::local` for to-hit, plus damage, DV, and PV (the stack's local numbers, where affixes go) and the weapon's `Attack` where `melee`, `ranged`, and `Numbers::of` take it from the object, for the die. Dumps' `numbers` and every shown attack follow.

**Value** (`Stack::value`, so shops and the wiki's prices agree): Fine ×2, Marvellous ×3, Masterwork ×5, damaged ×½ (rounded up), on top of what its parts and affixes are worth; attempts are worth nothing.

**What happens to Temper's old outcomes**: all gone. `data/outcomes/forge.toml` is deleted, with `Work` (`improve`, `flaw`, `ruin`), `work_on`, the `work` hook in `content/outcome.rs` and `world/outcome.rs`, `magic/temper.rs` (no other user), `lean_for`, and `POINTS_PER_MARGIN`. So the anvil no longer adds or raises affixes, or takes one away: a magic item's affixes stay through improvement and damage, and go only when it is destroyed. `offers = { temper = "forge" }` in `room_effects.toml` becomes `offers = { temper = true }`, as Temper no longer names an outcome table (`Offer::Temper`'s parse and load check, `resolve_places`); `exhaust` goes with it: the anvil was never spent. A unique item may be destroyed, as `Work::Ruin` allowed.

**Kept**: Temper identifies what survives, whatever came of it ("This is ...!" when that says something new). Marks: `Use::Temper` 100 % per attempt, whatever came of it, and `StatUse::Forged`. Who may: anyone at an anvil, as today; Metalcraft only leans the check.

**The odds** of an attempt, by the check's modifier (Strength's modifier + Metalcraft's points; Metalcraft gives +8 at 100, +12 at 200) and the attempts already made:

| Modifier | 1st | 2nd | 3rd | 4th | 5th | 6th |
|---|---|---|---|---|---|---|
| +0 | 54 % | 38 % | 25 % | 15 % | 7 % | 2 % |
| +4 | 81 % | 69 % | 54 % | 38 % | 25 % | 15 % |
| +8 | 96 % | 90 % | 81 % | 69 % | 54 % | 38 % |
| +12 | 99 % | 99 % | 96 % | 90 % | 81 % | 69 % |

So a smith at +8 usually reaches Masterwork in three attempts (69 %); an untrained character has a 54 % chance at one Fine and then gambles harder on the second.

**M1. An anvil in town: where it can be done** *(content only)*. Forge rooms (weight 3) and the showcase's `forge` level keep theirs; one more `anvil` in Cēaptūn, in The Forge beside the smith's fire (`levels/ceaptun.toml`, the weaponsmith's area `[35, 4, 43, 8]`). `offer_at` reads the terrain, so no rule changes; #975 asks on walking into it. Free: the anvil is not the shop's service. So Metalcraft has a place to practise on purpose in every Ascmearc run.

**M2 (Mend) and M3 (merging with Woodcraft): not built.** M0's repair is M2; M3 was a fallback.

### 6.8 Detect traps — too few ways to train

**Dt1. A sprung trap teaches** *(data + 1 hook)*. `Use::Sprung` 100 % when a hidden trap goes off under the player, and 50 % when one goes off under a creature the player sees (`World::spring_trap`, `trap.rs`). Trained only, as every mark. Most hidden traps are met this way (§5.3: 2–5 a level), so this alone lifts the skill over the 5-mark line.

**Dt2. Taking one apart teaches what it looks like** *(data only)*. `{ on = "disarm", chance = 30 }` on Detect traps: `Use::Disarm` already fires per try.

**Dt3. Looking hard counts a little** *(data only)*. `{ on = "search", chance = 3 }`: a search turn marks Detect traps 3 % whatever it finds.

### 6.9 Listen — too few ways to train

**L1. Listen at a door** *(small rule)*. Ability **Listen**, a direction, a turn, at a closed door beside the player: for that turn the player's hearing threshold through that door is lowered by Listen's value ÷ 10 + 2 (the same scale as `hearing`), and what moves beyond is heard as sound markers (the `sound` layer's `f`, `x`, `v` …, the perception snapshot's existing path). Marks 100 % per listen, whatever is heard.

**L2. Listening while resting** *(data + 1 hook)*. While the player rests or waits, a sound heard that is not news marks Listen at 5 % a turn (today only news marks, `perceive.rs`). A night among wandering creatures then earns a few marks, which a night of nothing does not. The hook is one line beside the news check; the rate is low on purpose (a rest is 13,000 turns).

### 6.10 Spellcraft — too few ways to train

**#989 owns this**: casting marks Spellcraft at 25 % (`docs/design/no-caster-level.md` §6, approved). The earlier Sc1's 10 % is replaced by it, and Sc2's mark for reading a scroll is refused by that design and #989's test. Nothing here adds to it. D0's bypass reads Spellcraft and never marks it.

### 6.11 Spot — too few ways to train

**Sp1. Sighting trains Spot** *(data + 1 hook)*. `Use::Sight` 25 % the first time the player sees a creature that has not noticed them (`perceive.rs`, where `mark_noticed` runs after `Perception::compute`, `docs/dev/surprise.md`; the creature's `unaware` is `World::unaware`). The passive counterpart of Stealth's `sneak`. The training stream, as every mark.

**Sp2. Searching counts a little** *(data only)*. `{ on = "search", chance = 3 }` on Spot, as Dt3.

### 6.12 Knowledge — niche

**K1. What you know of it** *(small rule, information; the owner, q8: yes, in words, never numbers)*. Look mode's description of a creature says, in words, what it resists and what it is immune to, when Knowledge stands above its level (the same read as `resist`, `Actor::skill_effect_above`): "You know the troll fears fire." The words are those of the `stats` layer's resistances. No marks (knowing is not practising).

**K2. Raise the curve** *(data only)*. `resist` is `[[0, 0], [5, 1], [40, 4]]`, at most 6. It becomes `[[0, 0], [5, 2], [40, 6]]`, with the cap standing. The value stays read above the creature's level.

### 6.13 Balance — too few ways to train

**Ba1. Rough ground and Staggered** *(data + 2 hooks)*. (a) `Use::RoughStep` 10 % per step onto ground that slows but is not water (bush, reeds, rubble, ice: `move_cost > 100` and not `stars::wading`), with `Effect::Footing` (percent of the surcharge off, curve as S1's) read in `step_price`. (b) Staggered (`status.toml`, `cost = 150`): `Effect::SteadyEase` takes Balance's percent off the status's extra cost, as Initiative's `surprise_ease` does for Surprised, and `Use::Staggered` 20 % per action taken while Staggered.

**Ba2. Merging into Athletics: not built.** A fallback Ba1 makes needless.

### 6.14 Tumble — fine, with a catch

**Tu1. One plain miss in four is a dodge** *(small rule; the owner, q7: "Attribute plain miss to dodge only 1/4 times. Dodge DV misses are still always dodges.")*. A blow at the **player** that comes back `Strike::Missed { by: None }` is attributed to `Category::Dodge` one time in four. A miss with `by: Some(Dexterity | Dodge)` (the defender's own bonuses, `why_missed`) is a dodge as it always was. A miss by armour or a shield stays that.

- **The stream**: the **training stream** (`WorldState::training`), `chance(25)`, one draw per plain miss at the player. `combat::strike` is unchanged and draws nothing new, so **the gameplay stream does not move**; a test holds it.
- **What it disturbs**: every later training draw moves by one, so marks and state hashes differ from the plain miss on, as any new mark does: saves are pinned to the version, and replays are recreated. Blows between creatures, or at a creature from the player, draw nothing.
- **The hook**: `fight.rs`, after the blow is made (past `caught_shot`) and before `Event::Attacked`, so everything after it sees one dodge: the words ("You dodge"), `Use::Dodge` at its 40 %, `StatUse::Dodge`, `free_blow::dodged`, and whatever answers a dodge (Poised, a weapon ability's dodge trigger). An answer fired that way draws the gameplay stream as it would for any dodge: the one way this reaches a fight, and it is the point of calling it a dodge.
- All plain misses count, a natural 2 and a total under 10 too: "plain" is `by: None`, and no new information comes back from `strike`. Spells' misses (`cast.rs`) are not blows and are left out.
- So a Dx-11 thief with Tumble 12 earns Tumble at last: 10 % a plain miss. `balance_probe/training.rs` reads the same attribution and is updated with it.

### 6.15 Appraise and Negotiation — niche, by design

**A1. Freshness** *(data + 1 hook; from #196's first line)*. `Effect::Freshness` on **Appraise** and on **Survival**, summed where both are trained: the eat list and look mode say a corpse's or food's stage in words (`rot::stage`, which the player is never shown today) when a check of the sum against the food's age passes; the dev `actors` dump already prints the stage. Marks: `Use::Judged` 10 % per food looked at, Appraise's. The spell half of the idea stays with #196.

**N1. Services haggle a little** *(data + 1 hook)*. `favour` applies to a healer's and a sage's price (`service.rs` keeps temples flat), and `Use::Deal` fires on a service paid. Negotiation then trains in every town visit, not only the shop's.

### 6.16 Heal — fine, with one rule gone

**H1. No more one try** *(small rule; the owner, q5: "get rid of that rule i guess")*. A poison or a disease may be tried again, as bleeding, a lock, or a trap may: time is the price. `Effect::treated` (`status.rs`) and `Infection::treated` (`disease.rs`) go, with `heal.rs`'s `treatable` conditions on them and the lines that set them (`world/heal.rs`); saves are pinned. `Unavailable::NothingToTreat` is then only nothing there to treat, and its comment and the `heal` dump words follow. A try is unchanged otherwise: a made try eases a poison a degree, and a disease by a check won or a put-off progression; it never cures one outright. `Use::Heal` 30 % per try.

## 7. Work in flight that this touches

**Landed since this document was first written**: #975 (walking into an altar, spring, or anvil asks: M1's anvil is found by walking, and the dialog is `input::offer_dialog`, which deep water's sits beside); Skill focus (#963: every effect read through `skill_effect` gains it, so Escape, Swim, Fletch, and Decipher's points all do; a Skill focus on one of the four no-use skills stops being a waste); the caster level and spell type designs (#977, #980), merged; #970's ranged penalties and Parry (no effect here). Talent ranks are going (#952, #966): this design takes one talent, Fletcher, with one effect and no ranks.

| Issue | What here depends on it |
|---|---|
| **#987** (learning is Spellcraft) | D0 comes after it: it needs `Effect::Learn`. It keeps "Decipher script ÷ 10" in the study check; D0 then removes it. It writes Spellcraft's row in `skills.toml` and the class files (a line apart from the ranger's `kit`, W1) |
| **#988, #990** (Arcane training, caster level removed) | none on the rules; `07-ranger.toml` lines move (W1) |
| **#989** (Spellcraft trains by casting) | owns §6.10; D0 adds no Spellcraft mark, and its test stands |
| **#995–#1004** (spell types and spells) | #1001's Blink ends Held, Webbed, and Pinned, E1's three: no conflict, both remove the status. #1002 adds fields to `status.toml` where E1 adds `escape`: a textual neighbour. #1000's Slick and its `after_tread` sit on the step path deep water's branch changes in `player_moves`: a neighbour. #1000's Slick staggers, which Ba1 eases. A bind or web spell on the player is E1's by its status |
| **#985 / PR #1006** (adjacent creatures follow on stairs) | none: nobody swims to stairs |
| **#959, #958** (vaults) | `mapgen` snapshots only: S2's placement changes the caves floors that roll a deep pool and the local areas beside lakes |
| **#196** (deferred ideas) | A1 takes the freshness line; S2 is bounded by the aquatic-monsters line; maps stay |

At most about three PRs at once may add actions or save state (`CLAUDE.md`): those here are M0 (`Stack`), H1 (removes state), W2 (doors), and the actions of B1, W1, D2, L1, W2, W3.

## 8. Decisions

The owner on PR #984, in his words; §6 applies them. All earlier open questions are closed.

- **Escape artist**: "Struggle free should not be an action. It should be an automatic skill check when player action is prevented/slowed because of status it can "fix"" (E1).
- **Fletching**: "Let's change the ranger ability to make the new universal Fletching action much better. others needs to make do without that bonus. Do not require small blade in hand, just in inventory is fine. Rangers should start with it. The ability menu should say that you do not have a knife as reason. Remember that short blade is different category than knives. we probs want knife." (W1)
- **Bluff**: "we do not want the complications that come from talking way out when caught. Talking past in overworld is fine for encounters flagged with that options - be careful what to flag." (B2 cut; B3)
- **Metalcraft**: "chance to improve or ruin metal items. first fail -> "damaged", improving "damaged" -> either completely destroys or removes damaged condition. just one action that does both. Can attempt the improve multiple times, difficulty raises each time. damaged -> all improvements gone and need to start over. improves random aspect of the base item (damage die, to hit, + damage, dv, pv) perhaps max 3 improvements per item? "Fine x, Marvellous X, masterwork x)" (M0)
- **q1, the four**: "others that what was mentioned before, build as suggested".
- **q2, Decipher script**: "decipher script for scroll use for non-casters. casters with sufficent spellcraft can bypass the check. Perhaps we will have more uses later (like hints in ancient language etc)" (D0; the later uses are not designed).
- **q3, deep water**: "sure, implement deep water, but do not make it as common as spills we have now. always ask confirmation before diving into deep water." (S2)
- **q4, Escape artist's stat**: "better of the two with the skill" (E1).
- **q5, Heal's one try**: "get rid of that rule i guess." (H1)
- **q6, the "typical marks a level" column**: "i do not really care". It stays as it is, an order of magnitude.
- **q7, Tumble**: "Attribute plain miss to dodge only 1/4 times. Dodge DV misses are still always dodges." (Tu1)
- **q8, information reads**: "yeah" (K1, in words, never numbers).

**Details the revision had to settle**, decided by the owner's evident direction:

1. **There was no ranger fletching ability to change.** The ranger gets one, Fletcher, at level 1 (W1). "Rangers should start with it" then holds whether "it" is the knife or the ability.
2. **Attempts never reset**, through damage and repair: "difficulty raises each time". A damaged item has no combat penalty, as the owner named none (M0).
3. **W3 Whittle** uses M0's rules: Temper's old outcome table, which W3 copied, is gone (q1 says build it; "just one action" says not to keep a second shape).
4. **D2 is built with D0**: q1 builds what is suggested, and it gives Decipher script a way to practise on purpose.
5. **Deep water's untrained are refused**, as S2 proposed: there is nothing to confirm for one who cannot swim.
