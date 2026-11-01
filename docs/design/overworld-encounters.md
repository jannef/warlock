# Warlock — Overworld encounters: ambush, ambushed, evade, fight; the encounter tables

Design document for the owner's review (2026-11-01, #793). Written against `origin/main` at
db88ffa3 (spell circles C3). Nothing here is implemented; §12 cuts it into issues. It builds on
what is there: the overworld and its generated local areas (`docs/dev/overworld.md`,
`world/travel.rs`, `mapgen/local/`), surprise, slipping past, and the initiative check
(`docs/dev/surprise.md`, `world/surprise.rs`, `world/slip.rs`, `world/ambush.rs`), checks
(`check.rs`), the danger levels' draw (`content/danger.rs`, `world/spawn.rs`), skills and their
marks (`docs/dev/skills.md`), the energy scheduler (`docs/dev/time.md`), the day's phases and the
weather (`daylight.rs`, `weather.rs`), quests and revealed locations (`docs/dev/quests.md`,
`world/reveal.rs`), the RNG streams (`docs/dev/mapgen.md`, `rng.rs`), and the one dialog
(`docs/dev/screens.md`). Every number is data and hidden in the game; the wiki states it (the
owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-11-01, #793): four outcomes while travelling the overworld, *ambush*,
*be ambushed*, *evade*, *fight*, decided by skill checks (evading too); rare ("they should not
happen as often as pokemon encounters in wild grass, not at all"); roads pretty safe; encounter
tables chosen by the area of the map and the tile type, affected by the time of day, and changing
during the scenario (a quest stage, a revealed place, time passing); start positions by outcome.

## Contents

1. Summary
2. Research, tied to the decisions
3. What exists, and what this builds on
4. How an encounter starts: the roll
5. The checks: who notices whom, and what the player may do
6. Where it happens: the local area, the positions, getting back
7. The table data
8. Peaceful encounters
9. Determinism, saves, replays
10. What the player sees: messages, dumps, the wiki
11. Talents and class abilities
12. Rejected ideas
13. Implementation plan (slices)
14. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **The roll** | after every overworld step onto a cell that has a local area, one draw from a new stream `encounters`: the chance is the destination landscape's **rate per hour** (`data/encounters.toml`: road 1 %, plains and grassland 3 %, forest and hills 5 %, dense forest and swamp 8 %) × the step's time × the phase of day (twilight × 1.5, night × 2). A day's march (12 h) rolls about 0.12 encounters on a road, 0.35 on open land, 0.6 in forest or hills, 1.0 in swamp or dense forest. Then the campaign's **tables** that match the cell (region, landscape, phase, weather, quest state, revealed places, days passed, player level) are merged and one entry is drawn by weight; no matching entry, nothing happens. §4 |
| **Two lookout checks** (`check::check`, the gameplay stream) | the player's: Intelligence modifier + the new `lookout` benefit (Spot, Survival) + Alert's `notice` against 13 + the party's `stalk` (its lead type's Stealth) + the land's cover + the night (halved for eyes with darkvision) + the rain. The party's: its lead's Intelligence modifier + its `spot` against 13 + the player's `stalk` (Stealth) + the cover + the night only for a lead without darkvision + the rain − a lit light the player carries. §5.1 |
| **Four outcomes** from the two checks | the player made, the party failed: **the drop** (ambush, evade, or fight, the player's choice); both made: **met** (evade or fight); the player failed, the party made: **ambushed** (no choice: the fight opens on the party's terms); both failed: **they pass each other by**, nothing happens and nothing is said. §5.2 |
| **The choice** | the one dialog (`Dialog`, `docs/dev/screens.md`), opened by the app from `PerceptionView::encounter()`: "Lie in wait" / "Slip past" / "Stand and fight". The world holds the pending encounter (`WorldState::encounter`) and refuses every action but `Action::Encounter(choice)` until it is answered; info screens stay open to the player, so they may look at their pack first. §5.3 |
| **Evade** (a check, as the owner asks) | Dexterity modifier + Survival's new `evade` benefit (+ Stealth's `stalk` while the party is unaware) against 13 + the lead's pursuit (its speed above the player's: +1 per 100) + 2 when the party is aware. Made: the party is never made, the detour costs half the step's time again ("You slip away unseen." / "You lose them in the hills."). Failed: the detour's time is spent and the fight opens with the party **three cells off**, nobody surprised ("They are onto you."). §5.4 |
| **Where** | the cell's local area `local:x,y`, generated on first visit as `>` does, persisting as any level; the party are ordinary actors made by `make_creature` with gear rolled. Positions by outcome (§6.1): **fight** the player on the arrival cell, the party seven cells in along the way; **ambush** the party five cells in with a route toward the player, each member with `Mind::missed` set (an ambush is a **slip already won**: they are unaware until they are hurt or step into the player's light) and one normal action of energy debt (the player acts first: D&D's surprise round through the scheduler); **ambushed** the party two cells off around the player, the player one action in debt, and `World::startle(player, lead)` runs the initiative check (Surprised on a failure, as a trap's); **evade failed** three cells off. The edge and the quick leave take the player back to the overworld cell as today; the party stays in the area. §6 |
| **The data** | shared `data/encounters.toml` (rates, phase factors, the checks' constants, the positions); per campaign `campaigns/<id>/data/encounters/*.toml`, `[[table]]` with `when` conditions and `[[table.entry]]` rows (`id`, `weight`, `name`, a `group` of counted types or a `draw` from a branch's population at a danger, `stance`). Tables matching a cell merge; `replaces` shuts a base table off while a quest table stands. §7 |
| **Peaceful** | yes, small: an entry's `stance = "peaceful"` (travellers with a dialogue: "Hail them" / "Pass by") or `"wildlife"` ("Hunt them" / "Let them be"); no checks for either. The Ascmearc ships a pedlar who tells the rumoured ways, villagers on the roads near the town, and deer. No travelling shop (§8). |
| **Determinism** | the rate roll, the entry, and the group's dice from `encounters` (`WorldState::encounters: Option<Rng>`, made on the first roll); the checks from the gameplay stream, two draws each, only when an encounter is rolled; marks on the training stream. Saved: the stream, the pending encounter, the party as actors. A campaign without tables draws nothing: every showcase replay hashes as before. §9 |
| **Talents** | **none new**: encounters are no system of their own; they extend `travel` (the roll, evade), `awareness` (the lookout; Wary through `startle`), `stealth` (the stalk), and `surprise`/`sneak_attack` (an ambush is a won slip, so the thief's dice land). Alert is widened to the lookout (one more read of `notice`; its tags gain `awareness`). The owner's rule 6 (#776): an overworld-only talent is niche, so none is minted. §11 |
| **Slices** | E1 the rules (data, roll, checks, pending choice, positions, dumps, the dev action), E2 the dialog and messages (text snapshots), E3 the Ascmearc's tables, the pedlar, the wiki page and spec, E4 later: encounters while camping. §13 |

**Key choices, and why**

1. *Rare by rate per hour, not per step.* A step is 20 minutes on a road and 2½ hours in a swamp; a rate per step would make roads the most dangerous ground per hour. Exposure is time spent in the open, so the roll's chance is the rate × the step's time. Roads are then safe twice over: a low rate and short steps.
2. *Both sides roll the same check, and "nobody saw anybody" is nothing.* Two independent lookout checks give the four outcomes the owner asks for with one mechanism, and the fourth cell of the grid (neither noticed) is the one honest answer: two parties that miss each other in a dark wood pass by. It also makes Stealth matter on the road without a word said: a stealthy traveller has fewer encounters, and the ones they have are on their terms.
3. *Ambush reuses slipping past; ambushed reuses `startle`.* The surprise design (#681–#688) built one definition of unaware, one place anyone is surprised, and a "slipped past" state on a creature's mind. An ambush sets that state on each member of the party, so the thief's first blow is a sneak attack by the rules that exist, and the energy scheduler gives the first action to the side that has the drop. Being ambushed calls `startle` as a hidden trap does. No new surprise rule.
4. *The fight is in the local area the cell already has.* The generators, the edge exit, the quick leave, the arrival side, and the rest all exist; an encounter only chooses where in that area each side stands. The party persists there as any creature does, so a fled ambush is still waiting in that wood.
5. *Tables merge; a quest table adds or replaces.* "The table can change during the scenario" is a condition on a table (`quest_active`, `quest_done`, `revealed`, `after_days`), not a script: a campaign writes the roads' base table once and a cultists' table that stands while the cult quest is active and replaces the base one on the fen road.
6. *No new talent.* The coverage policy (#773, #776) asks a new *system* to ship a talent and says what a system is; encounters extend four systems that have theirs. Minting a "fewer encounters" or "better lookout" talent would be exactly the niche travel talent rule 6 forbids.

## 2. Research, tied to the decisions

| Source | What it does | Taken | Left, and why |
|---|---|---|---|
| ADOM's wilderness | an encounter check per wilderness step, by terrain; roads and the plains near Terinyo nearly safe; a fight opens on a small generated map with the party placed around the player; the party can be fled by walking off the map | per-terrain rates, roads safe, a generated area, the edge as the way out (§4, §6) | ADOM's rate (every few steps in the hills) is the owner's "pokemon" rate; ours is per hour and a tenth of it |
| D&D 3.5 (Spot/Listen vs Hide/Move Silently, encounter distance, the surprise round, flat-footed) | opposed perception decides who is aware when the encounter begins; the aware side acts in a surprise round; distance by terrain | two lookout checks, the head start as a free action, the positions by outcome (§5, §6) | no encounter-distance table (positions are three numbers), no initiative order (the scheduler is exact and hidden, surprise design §11) |
| Pokémon's wild grass | an encounter every few steps in tall grass | nothing; the owner's anti-example | the rates of §4.2 give a day's march in forest about half an encounter |
| Warlock's own danger levels (campaign design §9) | the overworld has danger 0 "no wandering encounters now (a line on #196)" | the overworld keeps danger 0: nothing *spawns* on it; encounters are a different mechanism with their own tables and stream, and a `draw` entry may borrow a branch's population at a danger of its own (§7.2) | giving the overworld a danger would put kobolds in every field at 15-minute intervals |

## 3. What exists, and what this builds on

| Part | Today | Used for |
|---|---|---|
| An overworld step | `World::player_moves` (`world/travel.rs`): the step's price is the terrain's `travel_minutes` (`LevelState::step_price`, `World::step_price` with Survival's `travel` and the talents' `travel_percent`); `WorldState::overworld_step` keeps the direction | the roll's hook (one line after the move), the time that scales the chance, the arrival side |
| Local areas | `Content::local_area(campaign, local:x,y)` says whether a cell has one; `World::visit` generates it from `map/local:x,y`; `enter_local_area` arrives by `mapgen::local::arrival(side)`; `LevelState::exit` returns to the cell; `world/quick_leave.rs` refuses `<` with hostiles about | the fight's level, the arrival cell, the way out |
| Unaware, surprise, the slip | `World::unaware` (one definition), `World::startle` (the one place anyone is surprised: the initiative check, Surprised on a failure, Hair trigger), `Mind::missed` and `World::slipped_past` (a creature slipped past is unaware until `end_slip`: hurt, or the player steps into its light; lasts `[slip] lasts_turns` 60) | ambush (`missed` on the party), ambushed (`startle` on the player) |
| Checks | `check::check(rng, modifier, difficulty)`: 2d12 + modifier reaches the difficulty, a natural 2 fails, 24 succeeds; two gameplay draws; `check::ways` for the wiki | the two lookouts and evade |
| Skills | Spot (`spot`, passive noticing), Stealth (`quiet`, `slip`), Survival (`travel`, `forage`), Listen (`hearing`); creature types may carry `skills` (the footpad's `stealth = 25`); each benefit is read in one place (`docs/dev/skills.md`) | new benefits `lookout` (Spot, Survival), `stalk` (Stealth), `evade` (Survival), each read in `world/encounter.rs` only |
| Danger draws | `danger::draw(rng, own, generic, danger, odds)` with `Content::candidates`, `level_weight` (4 at D and D−1, 2 at D−2, D−3, 1 down to D−8) | a `draw` entry's party |
| The scheduler | energy per actor (`WorldState`), the player acts at energy ≥ 0, others in time order (`world/schedule.rs`) | the head start: one action's debt on the side caught out |
| Day, weather | `daylight::phase(time)` (Day, Twilight within `TWILIGHT_HALF` of 06:00 and 18:00, Night); `weather::blend`/`conditions` give the weather over the overworld; `WeatherDef::sight` already shortens overworld sight by rain | the phase factor, the concealment terms |
| Regions | `LevelTemplate::region_at(pos)`, the status bar's names | a table's `regions` condition |
| Quests, reveals | `Quests::record(id)` (stage, completed), `WorldState::revealed` | `quest_active`, `quest_done`, `revealed` conditions |
| The dialog | `Dialog::new(title, text).plain(label, Action)` (`ui/dialog_screen.rs`); the app builds one at its call site from the view, as `trap_dialog` and `sale_dialog` do | the choice |
| Refusing actions while a choice is due | `World::refuse_due` ("You have skill rolls to spend.") | the pending encounter's refusal |
| The dev spawn | `spawn` (one attempt now, whatever the cap), `Rng::spawn` per level | the pattern for the dev `meet` |

The deferred line "no wandering encounters now" (campaign design §9.1; the overworld's danger stays 0) is what this document replaces.

## 4. How an encounter starts: the roll

### 4.1 The hook

One line at the end of the overworld branch of `World::player_moves`, after the step's events and before the step's price is returned: `self.encounter_roll(step_seconds, events)` in a new `world/encounter.rs`. It runs only when

- the level is `LevelKind::Overworld`,
- the destination cell has a local area (`Content::local_area` is `Some`: a location cell, the pass, a bridge's water, nothing there),
- the run's campaign has at least one encounter table (`Content::encounter_tables(campaign)` non-empty), and
- no encounter is pending (§5.3).

The step is complete when the roll is made: the player stands on the new cell and its time has passed. The encounter is "as you arrive". Resting or waiting on the overworld rolls nothing in the first slices (§13 E4, §14 Q3).

### 4.2 The chance

`data/encounters.toml`:

```toml
# Chances in basis points (1/100 of a percent) per hour of travel on that landscape, by day.
[rate]
road = 100           # 1 %: a day's march on the road (36 cells, 12 h) rolls 0.12 encounters
plains = 300
grassland = 300
forest = 500
hills = 500
dense_forest = 800
swamp = 800

[phase]              # percent of the rate
day = 100
twilight = 150
night = 200
```

The chance of a step is `rate[landscape] × seconds × phase / (3600 × 100)` basis points, drawn as `rng.below(10_000) < chance` from the `encounters` stream (§9). `seconds` is the time the step took the player (`Spent::Turn(price)` as seconds at their speed), so a quick traveller is exposed less: Survival's `travel`, Long strider, and Pathfinder cut exposure as they cut time, which is their fiction ("you find the easy ways"); the design accepts that indirect effect and the wiki states it. The landscape is the destination terrain's `landscape` (`data/terrain.toml`), the one the local area is generated from.

| Landscape | Minutes a step | Chance a step (day) | Steps in 12 h | Expected a day (day) | Expected a day (night) |
|---|---|---|---|---|---|
| road, bridge, ford | 20 (ford 30) | 0.33 % (0.5 %) | 36 | 0.12 | 0.24 |
| plains | 45 | 2.25 % | 16 | 0.36 | 0.72 |
| grassland | 50 | 2.5 % | 14 | 0.35 | 0.70 |
| forest | 90 | 7.5 % | 8 | 0.60 | 1.2 |
| hills | 120 | 10 % | 6 | 0.60 | 1.2 |
| dense forest | 150 | 20 % | 5 | 1.0 | 2.0 |
| swamp | 150 | 20 % | 5 | 1.0 | 2.0 |

These are *rolled* encounters; the fourth outcome of §5.2 (nobody noticed anybody) turns a quarter to a half of them into nothing, so a baseline character meets about two thirds of these by day. The journeys of the campaign design (§5.6), rolled, by day: the pass to Cēaptūn 0.017 (a level-1 character on the road is as safe as the design promised); Cēaptūn to Cealdhlāw 0.25; Cēaptūn to Fennmynster 1.8 (0.16 of it on the road, the rest in the fen: the fen is the point); Cēaptūn to Fyrdelf 1.2.

### 4.3 The entry

The tables of the run's campaign whose `when` holds for the cell now (§7.3) are merged: every entry of every matching table, less those of tables another matching table `replaces`. The weights are summed and one `below(sum)` from `encounters` picks the entry; the group's count dice are rolled from the same stream. An empty merge is nothing, silently (a region's night with no night table is quiet).

## 5. The checks

### 5.1 Who notices whom

Two independent checks, `check::check` on the gameplay stream, made as the entry is drawn (never for a peaceful or wildlife entry, §8). The **party's lead** is the entry's `lead` type, default its first; its stats and `skills` are its type's (`CreatureDef`, read as `make_creature` reads them, so a party that is never made costs no actor).

| | The player's lookout | The party's lookout |
|---|---|---|
| Modifier | Intelligence modifier + `TalentMods::check(Intelligence)` + `skill_effect(Lookout)` (Spot +8 at 100, +12 at 200; Survival +4 at 100, +6 at 200; summed) + Alert's `notice` (2) | the lead's Intelligence modifier + its `spot` benefit (the same ladder Spot gives the player) |
| Difficulty | 13 + the party's **stalk** (the lead's Stealth through `stalk`: +4 at 50, +8 at 100, +12 at 200; the footpad's 25 gives +2) + **cover** + **night** (halved, rounded down, for a player whose vision profile has darkvision) + **rain** | 13 + the player's **stalk** (`skill_effect(Stalk)`: Stealth, the same ladder) + **cover** + **night** only for a lead whose `vision` profile has no darkvision + **rain** − **light**: 4 while the player carries a lit light at night or twilight (`Actor` light as `light.rs` reads it) |

`[checks]` in `data/encounters.toml`: `difficulty = 13`, `cover = { road = 0, plains = 0, grassland = 1, hills = 2, forest = 2, dense_forest = 4, swamp = 3 }`, `night = 3`, `twilight = 1`, `rain = 1`, `heavy_rain = 2`, `light = 4`. Burden adds nothing: the clatter of a loaded pack is the footsteps rule, which the overworld does not run (spec §5: "the overworld is silent").

Why Intelligence and not Willpower: the passive search that notices a secret door in passing is an Intelligence check (`world/search.rs::notice_hidden`), and a lookout is the same noticing; one stat for "noticing" keeps the wiki's story short. Why the night counts against a human but not a goblin: 238 of 294 creature types see in the dark (surprise design §3.4), and the owner's "time of day affects encounters" should mean what it does in the fiction: the night is theirs.

Odds (2d12 + modifier reaches the difficulty; `check::ways`): needing 9 is 81 %, 11 69 %, 13 54 %, 15 38 %, 17 25 %, 19 15 %, 21 7 %.

| Case | The player makes it | The party makes it | The drop | Met | Ambushed | Pass by |
|---|---|---|---|---|---|---|
| level-1 human (In 10, no Spot, no Stealth), footpads (Stealth 25, no `spot`, human eyes), road, day | needs 15: 38 % | needs 13: 54 % | 17 % | 21 % | 33 % | 29 % |
| the same, goblins (darkvision, no skills), forest, day (cover 2) | needs 15: 38 % | needs 15: 38 % | 24 % | 14 % | 24 % | 38 % |
| the same, goblins, forest, night (+3 for the human only) | needs 18: 19 % | needs 15: 38 % | 12 % | 7 % | 31 % | 50 % |
| level-6 thief (In 12, Spot 50: +4, Stealth 60: +5), goblins, forest, night | needs 13: 54 % | needs 20: 10 % | 49 % | 5 % | 5 % | 41 % |
| level-6 ranger (In 12, Survival 80: +3, Spot 40: +3, Stealth 40: +3), footpads, hills, dusk (cover 2, twilight 1) | needs 12: 62 % | needs 19: 15 % | 53 % | 9 % | 6 % | 32 % |

Marks (the training stream): the player's lookout check made or not marks Spot's `spot` and Survival's `travel` use (`Use::Lookout`, a new use both list); the stalk marks Stealth's `sneak` when the party fails its lookout. Nothing is said of a check.

### 5.2 The four outcomes

| The player | The party | Outcome | What happens |
|---|---|---|---|
| made | failed | **the drop** | the player sees them first. The dialog (§5.3): **ambush**, **evade**, or **fight** |
| made | made | **met** | each sees the other at the same moment. The dialog: **evade** or **fight** |
| failed | made | **ambushed** | no dialog: the fight opens on the party's terms (§6.1), `World::startle(player, lead)` runs the initiative check (Dexterity + Initiative + Wary against 13 + the lead's Sneak attack craft, as for a trap), failed: Surprised |
| failed | failed | **pass by** | nothing happens and nothing is said. The roll's draws are spent; no area is generated, no actor made |

The owner's "ambush, ambushed, evade, fight" are the first three rows; "pass by" is the silence between them, and the reason a stealthy traveller finds the roads quieter.

### 5.3 The choice

The step returns as usual (`Spent::Turn(step)`, the time spent), its outcome carrying `Event::Encounter { name, choices }`, and the world holds `WorldState::encounter: Option<Pending>` (§9.2). While it is `Some`, `World::player_acts` refuses every action but `Action::Encounter(choice)` and the free ones (`refuse_pending`, beside `refuse_due`: "You must decide what to do about them.", no time). The app, in `Game::applied`, reads `PerceptionView::encounter()` (an `EncounterView { title, text, choices: Vec<(label, Choice)> }`) and opens the one dialog with a `.plain(label, Action::Encounter(choice))` per row; no `cancel`: Esc closes it, the pending stands, info screens (inventory, the character sheet, the map) open as always since they are no actions, and the next action attempt reopens the dialog. On load with a pending encounter the dialog opens the same way: the view says so.

| Outcome | Title | Text | Rows (replay text) |
|---|---|---|---|
| the drop | the entry's `name`, capitalised ("Footpads") | "A band of footpads is coming along the road. They have not seen you." | "Lie in wait" (`encounter:ambush`), "Slip past" (`encounter:evade`), "Stand and fight" (`encounter:fight`) |
| met | the same | "A band of footpads sees you as you see them." | "Give them the slip" (`encounter:evade`), "Stand and fight" (`encounter:fight`) |
| peaceful | "A pedlar" | "A pedlar is coming along the road." | "Hail them" (`encounter:hail`), "Pass by" (`encounter:pass`) |
| wildlife | "Deer" | "A herd of deer grazes ahead." | "Hunt them" (`encounter:hunt`), "Let them be" (`encounter:pass`) |

`Action::Encounter(Choice)` is one action (`Choice::{Ambush, Evade, Fight, Hail, Hunt, Pass}`), refused with no pending encounter or a choice the outcome does not offer. Its costs: ambush `encounter_wait` (10 minutes: lying in cover until they come), evade the detour (§5.4), fight, hail, hunt nothing (the area is entered as `>` enters it, 6 s), pass nothing.

### 5.4 Evade

| | |
|---|---|
| Modifier | Dexterity modifier + `TalentMods::check(Dexterity)` + `skill_effect(Evade)` (Survival: +8 at 100, +12 at 200) + `skill_effect(Stalk)` (Stealth) **only while the party is unaware** (the drop) |
| Difficulty | 13 + **pursuit** (the lead's speed above 1000, +1 per 100: a warg at 1300 is +3; a flying lead +2) + 2 when the party is aware (met) |
| Time | half the step's price again, whatever comes of it (`[evade] detour_percent = 50`) |
| Made | "You slip away unseen." (the drop) / "You lose them in the hills." (met, the landscape's word); the pending clears; nothing is made. Survival's `travel` use and Stealth's `sneak` (the drop) mark |
| Failed | "They are onto you." (the drop) / "They run you down." (met); the fight opens with the party **three cells off** (§6.1), nobody surprised, nobody in debt |

A baseline level-1 (Dexterity 12) slipping past unaware footpads needs 12: 62 %; aware, 14: 46 %; the ranger above (Survival 80: +6, Stealth +3, Dexterity 14) past unaware wargs (pursuit +3) needs 5: 99 %, aware wargs 7 less 6: 97 %. A woodsman is not caught in the woods; a townsman is, now and then, and pays the detour and a closer fight.

## 6. Where it happens

### 6.1 The area and the positions

The fight opens in the cell's local area, `local:x,y`, exactly as `>` would open it: `World::visit` generates it on the first visit (stream `map/local:x,y`, unchanged by any encounter), `travel` moves the player, `Event::LevelChanged` is pushed, and the arrival cell **A** is `mapgen::local::arrival(side)` for the side the step came from (always known: the roll follows a step). The **way in** is the road if one leaves the area at A, else the straight line from A toward the area's middle; "n cells in" means the n-th walkable cell along it (`free_cell_near` for a taken one). The party is made by `make_creature` from a `CreatureSpawn` per member (`rolls_gear = true`: gear from `gear/<actor id>` as a spawn's; `faction` the type's; a `route` where the outcome says) at the positions below, each on a free walkable cell nearest its target in ring order.

| Outcome | The player | The party | Awareness and the head start |
|---|---|---|---|
| **fight** (chosen) | A | `[positions] fight = 7` cells in, spread one cell apart across the way | everyone aware (`Mind::last_seen` the player's cell, `noticed` the turn); no debt |
| **ambush** (chosen) | the cover cell nearest the way two cells in: a walkable cell beside a tree, bush, boulder, crag, or reed (the kit's cover terrain), else beside the way | `ambush = 5` cells in, with a `route` of steps back along the way to A (they walk past the player's cover) | each member `Mind::missed = Some(turn)` (slipped past: unaware until hurt or until the player's light reaches them, `slip_lapses` after `lasts_turns`) and energy `−price(1000, BASE_TURN)`: one normal action of debt, so the player acts first. The first blow or shot (within `missile_reach`) finds them unaware: `startle` on the creature, the sneak dice for a thief, Stalker's eye's mark on each |
| **ambushed** | A | `ambushed = 2` cells off, around A, the lead between the player and the way in | the player's energy `−price(1000, BASE_TURN)`; `World::startle(player, lead)` before the party acts (its initiative check; Hair trigger answers a hostile beside the player as it would a trap's attacker, that is never); the party aware |
| **evade failed** | A | `close = 3` cells off, between the player and the way in | everyone aware; no debt |
| **met**, fight chosen | as fight | as fight | as fight |
| **hail** / **hunt** | A | `fight` cells in | neutral (their faction's stance); a traveller faces the player and waits (a `post`), deer graze as the kit's deer do |

The party's AI is the ordinary one from then on (`ai.rs`: a hostile that sees the player hunts them; one with a route walks it until it sees or hears something). Nothing in the AI is new.

### 6.2 Getting back, and what persists

Leaving is what it is today: walking off any edge returns to the overworld cell (the arrival cell *is* an edge cell, so a step back is an escape, with the free blows and the dear Surprised action that implies), and `<` is the quick leave, refused while a hostile sees the player or stands within 10. The area persists as any level: its party stays where the player left it (a fled ambush waits in that wood; a return by `>` or another encounter rolled on the same cell finds it there, placed around the new arrival as above). A party never leaves its area: the overworld has no creatures and gets none.

Encounters rolled on a cell whose area already exists use it (no regeneration: the stream `map/local:x,y` is never drawn twice). A second party in an area that already holds one is placed as above; nothing caps it but the rates.

## 7. The table data

### 7.1 The files

| File | What |
|---|---|
| `assets/data/encounters.toml` (shared, optional: without it nobody meets anybody) | `[rate]`, `[phase]`, `[checks]`, `[positions]`, `[evade]`, `encounter_wait` (§4.2, §5.1, §5.4, §6.1); `content/encounter.rs` loads it into `EncounterRules`, checked at load (every landscape named, percents in range, positions 1 to 20, the difficulty 1 to 30) |
| `assets/campaigns/<id>/data/encounters/*.toml` (a table per file or several) | `[[table]]` rows, `ContentFiles::table("encounters")` as quests and dungeons are gathered (`docs/dev/campaign.md`); a campaign's table names its own or shared creature types, its overworld's regions, its quests, its hidden locations; ids unique within the campaign |
| `assets/data/encounters/` (shared tables) | none proposed: the showcase gets no tables, so nothing it does changes |

### 7.2 A table

```toml
[[table]]
id = "cealdhlaw_downs_night"
# Where and when it applies (every field optional; all given must hold).
when = { regions = ["Hlāwdūn"], landscapes = ["hills", "grassland"], phase = ["night", "twilight"],
         level = { min = 3 } }
rate_percent = 100                 # scales the landscape's rate while this table matches (default 100)
replaces = ["cealdhlaw_downs"]     # these tables' entries are left out while this one matches

[[table.entry]]
id = "downs_skeletons"
name = "a knot of skeletons"       # what the player is told; 60 characters at most
weight = 4
group = [{ creature = "skeleton", count = "1d3+1" }, { creature = "skeleton_archer", count = "1d2-1" }]

[[table.entry]]
id = "downs_ghoul"
name = "a barrow ghoul"
weight = 2
group = [{ creature = "barrow_ghoul", count = "1" }]

[[table.entry]]
id = "downs_wight"
name = "a barrow wight"
weight = 1
when = { level = { min = 5 } }     # an entry may narrow its table's conditions
group = [{ creature = "barrow_wight", count = "1" }]

[[table.entry]]
id = "downs_strays"
name = "something from the barrow"
weight = 2
draw = { pool = "cealdhlaw", danger = 3, count = "1d3" }   # a branch's population at a danger, by the danger weights
```

| Field | Meaning | Load checks |
|---|---|---|
| `when.regions` | the region names of the campaign's overworld (`region_at` of the cell) | each a `[[region]]` name of that overworld |
| `when.landscapes` / `when.terrains` | the destination cell's terrain's `landscape` / the terrain id itself | ids exist; a landscape with a `rate` |
| `when.phase` | `day`, `twilight`, `night` (`daylight::phase`) | |
| `when.weather` | `clear`, `overcast`, `rain`, `heavy_rain` (the weather over the cell now, `weather::conditions`) | |
| `when.level = { min, max }` | the player's level (`World::player_level`) | 1 to 60 |
| `when.quest_active` / `when.quest_done` / `when.quest_not` | quest ids: active (given, not completed), completed, never given | ids in the campaign's or shared quests |
| `when.revealed` / `when.hidden` | a hidden location's id is in `WorldState::revealed`, or is not | ids of the overworld's `hidden` entries |
| `when.after_days` / `when.before_days` | whole days since the run began (the scenario's `start_date` and `start_time` against `WorldState::time`) | 0 to 3600 |
| `rate_percent` | scales the rate while the table matches (several multiply) | 0 to 1000 |
| `replaces` | tables whose entries are left out while this one matches | ids exist; no cycle needed: `replaces` is one-way and never transitive |
| `entry.id`, `entry.name`, `entry.weight` | the id (the dev action, the dumps, the wiki), the words, the weight | unique id; 1 to 60 characters; weight 1 to 1000 |
| `entry.when` | narrows its table's conditions (the same fields) | as above |
| `entry.group` | counted types: `creature`, `count` (dice; a result of 0 or less is none; the party must have at least one member: the first row's dice must be at least 1) | types the campaign may use (its own or shared), `count` parses |
| `entry.draw = { pool, danger, count }` | `pool` a branch id (its `population`) or a list of types; `count` members drawn one by one by `danger::draw` at `danger` with the shared odds (strays and out-of-depth as a floor's) | the branch is the campaign's; danger 1 to 99; uniques are left out of the pool |
| `entry.lead` | the type whose eyes and Stealth the party's checks use | one of the entry's types; default the first group row's |
| `entry.stance` | `hostile` (default), `peaceful` (§8: no checks, "Hail them"), `wildlife` (no checks, "Hunt them") | a peaceful or wildlife entry's types are of factions not hostile to `player` (`Content::stance`); a hostile entry's are hostile |
| `entry.npc` | a peaceful party's dialogue (`Content::dialogue_of` prefers it, `docs/dev/npcs.md`), given to the lead | an npc dialogue id of the campaign |

### 7.3 Matching

`Content::encounter_tables(campaign)` is the campaign's tables sorted by id. For the cell: keep those whose `when` holds, drop those any kept table `replaces`, then keep each remaining table's entries whose own `when` holds. The order of the merged entries is by table id then entry file order, and the draw is `below(sum of weights)` walking that order, so the result depends on nothing iterated from a `HashMap`. The rate is the landscape's × the product of the kept tables' `rate_percent`.

### 7.4 The Ascmearc's first tables (slice E3; the exact rows are that PR's)

| Table | When | Entries (weight: name, group) |
|---|---|---|
| `roads` | `landscapes = ["road"]` | 3: a band of footpads, `footpad` 1d3+1 (+ `footpad_archer` 1d2−1 at night); 2: goblin raiders, `goblin` 1d3 + `goblin_archer` 1d2−1; 3: a pedlar (peaceful, `pedlar`, §8); 2: villagers on the road (peaceful, `villager` 1d3; `when.regions = ["Cēapdenu", "Burnadenu"]`); 2: a herd of deer (wildlife, `deer` 1d4+1) |
| `vale` | `regions = ["Cēapdenu", "Burnadenu", "Ēastmearc"]`, `landscapes = ["plains", "grassland"]` | 4: deer; 3: giant rats, `rat` 2d3; 3: kobolds, `kobold` 1d4+1 (+ `kobold_archer` 1d2−1); 2: goblin raiders; 1: a lone footpad, `footpad` 1 |
| `downs` / `downs_night` | `regions = ["Hlāwdūn"]`; the night table above `replaces` the day one | day: kobolds, goblins, barrow spiders (`barrow_spider` 1d3); night: the table in §7.2 |
| `woods` | `regions = ["Holegnwudu", "Lindwudu"]`, `landscapes = ["forest"]` | 4: deer; 3: goblins; 2: barrow spiders; 2: wargs, `warg` 1d2 (`phase = ["night", "twilight"]`); 1: a band of footpads |
| `deep_woods` | `regions = ["Dēopholt", "Deorcholt"]` (`dense_forest`) | 3: orc runts, `orc_runt` 1d4 + `orc_warrior` 1d2−1; 2: wargs 1d3; 2: a skeletal wolf pack, `skeletal_wolf` 1d3 (night); 2: draw `wulfeshol` at danger 5, 1d3 (`when.revealed = "wulfeshol"`) |
| `ascdun` | `regions = ["Ascdūn"]`, `landscapes = ["hills"]` | 3: kobolds; 2: goblins; 2: duergar prospectors, `duergar_miner` 1d2 (`level.min = 6`); 1: a hill troll, `hill_troll` 1 (night, `level.min = 10`); 2: draw `fyrdelf` at danger 6, 1d2 (`when.revealed = "fyrdelf"`) |
| `fen` | `regions = ["Micelfenn"]` | 3: giant leeches, `giant_leech` 1d3; 3: giant frogs, `giant_frog` 1d2; 2: the drowned, `drowned` 1d3 (night); 2: a boggle, `boggle` 1 (`level.min = 5`); 1: a will-o'-wisp, `will_o_wisp` 1 (night) |
| `fen_cult` | `regions = ["Micelfenn"]`, `when.quest_active = "fen_minster"` (Sister Hild's, which reveals Fennmynster; `campaigns/ascmearc/data/quests/hild.toml`); `rate_percent = 150` | 4: cultists on the fen road, `cultist` 1d3+1 + `cult_priest` 1d2−1 |
| `wanted_wystan` | `landscapes = ["road"]`, `regions = ["Cēapdenu", "Hlāwdūn"]`, `when.quest_active = "guild_wanted"` with Black Wystan drawn (the variant: a `quest_variant` condition, §14 Q6) | 5: Wystan's archers, `footpad_archer` 1d3 + `footpad` 1d2 |
| `orcs_come_down` | `regions = ["Burnadenu"]`, `when.after_days = 30` | 3: an orc raiding party, `orc_runt` 1d3 + `orc_warrior` 1d2 (+ `warg` 1 at night) |

The "changes during the scenario" cases the owner named are each a row: a quest stage (`fen_cult`, `wanted_wystan`), a revealed place (`deep_woods`'s and `ascdun`'s draws), time passing (`orcs_come_down`). Levels of the types named: rat 1, kobold 2, goblin 2, barrow spider 2, goblin archer 3, orc runt 3, skeleton 3, giant leech 4, giant frog 4, barrow ghoul 4, orc warrior 5, drowned 5, skeletal wolf 5, footpad 6, warg 7, boggle 7, footpad archer 7, duergar miner 9, hill troll 15; a level-1 character on the road meets footpads (level 6, 1d3+1 of them) a third of the roads' hostile rolls: the road's rate (0.12 a day) is what keeps that survivable, and `level.min` on the heavier rows is the second guard (§14 Q5).

## 8. Peaceful encounters

Proposed, small. The owner's four outcomes are a hostile party's; a peaceful entry has no checks and no outcomes, only a question: **"Hail them"** opens the area with the party at `fight` range, neutral, the lead given the entry's `npc` dialogue so `chat` and `ask` work as with any person (`docs/dev/npcs.md`), and **"Pass by"** is nothing (no time: the step's is spent). A **wildlife** entry is the same with "Hunt them" / "Let them be": deer at `fight` range in a glade, food for a hungry traveller (`docs/dev/hunger.md`), Beast-friend's business as always (`world/beast_friend.rs`).

The Ascmearc's first: a **pedlar** (`campaigns/ascmearc/data/creatures/pedlar.toml`, faction `merchants`, human, level 3, a pack of small wares as `carry`, a dialogue with two topics: `rumours` with a `route` to Cealdhlāw and the ford (the rumoured ways, `world/rumour.rs`: his telling marks the player's map as day sight along the road would), and `news` (a rotation of lines about the roads: the design's register for commoners), and a `mutter`), **villagers** on the roads of the vale (the existing `villager`, faction `peaceful`, a rotation of greetings), and **deer**.

Not proposed: a travelling shop. Shops are level fixtures with a floor and a keeper's post (`docs/dev/shops.md`); a cart on the road would need a shop without a floor, a new thing for one encounter. The pedlar talks; if the owner wants him to trade, that is a shop-design question (§14 Q7).

## 9. Determinism, saves, replays

### 9.1 Streams

| Stream | Label | Kept | Draws |
|---|---|---|---|
| encounters | `encounters` (`Rng::stream(seed, "encounters")`, a `Rng::encounters` constructor beside `Rng::spawn`) | `WorldState::encounters: Option<Rng>`, made on the first roll, left out of the JSON while `None` | the rate roll (one `below(10_000)` per qualifying step), the entry (one `below`), each group row's count dice, a `draw` entry's `danger::draw` |
| gameplay | as today | `WorldState::rng` | the two lookout checks (two draws each) and the evade check (two), only when an encounter is rolled and the entry is hostile; `startle`'s check when ambushed (two) |
| training | as today | `WorldState::training` | the marks |
| `map/local:x,y`, `gear/<actor id>` | as today | | the area on its first visit (never again), each member's gear |

A campaign with no tables makes no stream and draws nothing: every showcase replay and golden hashes as before. A run of the Ascmearc on `main` draws the rate roll on every overworld step with a local area from the new stream only, so its gameplay stream is untouched until an encounter is rolled; from that step its replays hash differently from `main`'s (two to six gameplay draws, new actors).

### 9.2 Saved state

| Field | Type | When present |
|---|---|---|
| `WorldState::encounters` | `Option<Rng>` | after the first roll |
| `WorldState::encounter` | `Option<Pending { entry: String, cell: Pos, side: Dir, members: Vec<CreatureId>, outcome: Outcome }>` (`Outcome::{Drop, Met, Peaceful, Wildlife}`; ambushed, evade failed, and pass by never pend) | between the step and the choice (a quit mid-dialog resumes on it) |
| the party | ordinary `Actor`s in `WorldState::actors` on the area's level, their `Mind::missed`, energy, routes | once the area opens |

Both new fields are `#[serde(default, skip_serializing_if = "Option::is_none")]`, so old saves load and runs without encounters hash as before. `World::state_hash` covers them as it covers every field. The replay format gains the action texts `encounter:CHOICE` and the dev `meet:ENTRY[:OUTCOME]` (§10.2), nothing else.

### 9.3 Invariants (the tests of E1)

- No draw from any stream on an overworld step without tables, onto a cell without a local area, or while an encounter pends; the gameplay stream is drawn only when an entry is rolled and hostile (`no_draws_without_an_encounter`, the style of `no_draws_while_nothing_is_unaware`).
- The same seed, scenario, and actions give the same encounters (a replay `--verify` through a forced encounter of each outcome).
- The overworld's danger stays 0 and `spawn_tick` never fires there; the party never leaves its area.
- The positions hold: the player on A (or a cover cell within two of the way for an ambush), the party at its distance on walkable free cells, one member per cell, none on water; an ambushed player is `startle`d exactly once; an ambushing party is unaware by `World::unaware` of the player at the first turn and each member owes one action.
- Pass by generates no level and makes no actor.
- Load checks refuse an unknown region, type, quest, hidden location, or landscape; a peaceful entry of a hostile type; a hostile entry of a peaceful one; a first row whose dice may give 0.

## 10. What the player sees

### 10.1 Messages (words, never a number; `say`, so a rest or run stops)

| Moment | Line |
|---|---|
| the drop | "You spot a band of footpads on the road ahead. They have not seen you." (the way's word: road, trail, ground) |
| met | "A band of footpads sees you as you see them." |
| ambushed | "Footpads burst from the hedgerow!" / "... from the undergrowth!" / "... from behind the rocks!" (the landscape's word), then the area, then Surprised's own "You are taken by surprise!" or "You keep your wits about you." as `startle` says them |
| pass by | nothing |
| ambush chosen | "You wait in cover." then the area; the first blow's "You catch the goblin unawares." is surprise's own line |
| evade made / failed | §5.4 |
| fight chosen | "You stand your ground." then the area |
| hail / hunt / pass | "You hail the pedlar." / "You go after the deer." / nothing |
| the status bar | nothing new; the region as today |
| look mode | nothing new: a party member is a creature like any; Stalker's eye's "unaware of you" on an ambushed one |

### 10.2 The CLI and dumps

| Where | What |
|---|---|
| action text | `encounter:ambush|evade|fight|hail|hunt|pass` (the choice); the dev `meet:ENTRY[:drop|met|ambushed|pass]` (an entry id of the run's campaign, rolled now on the player's overworld cell as the dev `spawn` forces an attempt: the checks as usual, or the outcome forced for captures and tests; any entry of the campaign may be forced whatever the cell's tables hold, that is what a dev action is for; refused, with no time, off the overworld or on a cell without a local area) |
| the header line | `encounter pending: a band of footpads (ambush, evade, fight)` while one pends |
| `actors` (dev) | each party member marked `of encounter downs_skeletons`; the energy debt shows as the existing `energy` |
| `truth` (dev) on the overworld | `encounter rate here N bp/h (day), tables: roads, vale` for the player's cell |
| JSON | `encounter` (`entry`, `name`, `outcome`, `choices`) or null; `encounter_tables` (dev: the ids matching the player's cell) |

### 10.3 The wiki (`wiki/src/encounters.md`, under The overworld in `SUMMARY.md`; generated by `xtask/src/wiki/encounters.rs`)

| Table | Rows |
|---|---|
| `encounter-rates.md` | each landscape's rate per hour by day, twilight, and night; the chance of one step; the expected number in a day's march |
| `encounter-checks.md` | the two lookouts' and evade's formula; the cover, night, rain, and light terms; `check::ways` by edge for the needed total (as `surprise-check.md`) |
| `encounter-ladders.md` | `lookout`, `stalk`, `evade` by skill value through `Benefit::at` |
| `encounter-tables-<campaign>.md` | per campaign: table, when (regions, landscapes, phase, weather, level, quests, reveals, days), rate percent, each entry's name, weight, group or draw, stance; the test world left out |
| `encounter-positions.md` | the distances by outcome, the head start, who is unaware |

The spec gains "Overworld encounters" in spec/07c-generation.md after "Leaving an outdoor area"; `docs/dev/encounters.md` is the topic file; `docs/dev/overworld.md` and `surprise.md` each get one line pointing at it; CLAUDE.md's action text and the dump layers get their lines.

## 11. Talents and class abilities

**None new.** Encounters are not a `[[system]]`: the policy's test (`docs/dev/progression.md` "Coverage policy": a system is broad, a sub-feature extends its parent) puts them under four systems that have their talents, each read where its rules already run:

| System | What encounters read of it | Its talents that act, unchanged |
|---|---|---|
| `travel` (Overworld travel and local areas) | the roll's time, the area, evade | Long strider, Pathfinder, Trailblazer, Swift tracker, Trade routes: less time, less exposure (§4.2) |
| `awareness` | the lookout; `startle` when ambushed | Wary (+4 on the initiative check), Cool head, Hair trigger, Stalker's eye (the ambushed party's mark); **Alert widened**: its `notice` (2) is added to the player's lookout, one more read of `TalentMods::notice` in `world/encounter.rs`, and its `systems` gain `awareness` (its description already says "you notice hidden things in passing") |
| `stealth` | the player's stalk | Camouflage and the footstep talents touch noise, which the overworld does not run; none reads the stalk, and none is added |
| `surprise` / `sneak_attack` | an ambush is a won slip | Ambusher, Improved sneak attack, Master thief, Patient blade, Anatomist, Long reach: the thief's and ranger's ambush is where they shine |
| `factions` | wildlife and peaceful entries | Beast-friend |

The owner adopted #776's rule 6: an overworld-only talent is niche and is not minted. A "fewer encounters" talent would be Quiet delver's twin for the road (information-free, small, niche); a "better lookout" talent is Alert, which is why Alert is widened instead. Class abilities: none; the ranger's Swift tracker and the bard's Trade routes already own the road.

## 12. Rejected ideas

| Idea | Why not |
|---|---|
| Giving the overworld a danger level and letting `spawn_tick` run there | spawns are per 15 minutes on the player's level with a population cap: the overworld would fill with kobolds the player sees coming across the map for hours. Encounters are a roll per step, resolved in a local area; the overworld keeps danger 0 (campaign design §9.1, question 13) |
| A rate per step | a road step is 20 minutes, a swamp step 150: per step, roads would be the most dangerous ground per hour. Per hour, roads are safe twice over |
| "Both failed" as a close-quarters fight | two parties that miss each other pass; and a stealthy traveller whose party never spots them would fight at arm's length most nights, which punishes Stealth. Nothing happens, and Stealth quiets the roads |
| An encounter-distance table by terrain and a visible "distance" | three constants in the data and the arrival side do the work; the player sees where everyone stands |
| A new "ambushing" status, or placing the party asleep | `Mind::missed` is the unaware-until-hurt state the slip built, and the energy debt is the free action; a sleeping party would be a den, not a march |
| An initiative order | the scheduler is exact and hidden (surprise design §11) |
| A `KeyCommand` or input mode for the choice | every question is the one dialog (the owner's rule); the pending state in the world makes the dialog reopenable and save-safe |
| A separate "encounter level" (a small arena) | the local area exists, persists, has an edge exit and a quick leave, and is the place the fiction says: the wood you were walking through |
| A cap on encounters per day, or a cooldown | no anti-farming limits (the owner); the rate is the limit, time the cost. A player who walks the fen at night to farm skeletons pays in nights |
| Weather on the rate | a rainy fen is as full of leeches; rain shortens eyes (the checks) and that is its effect |
| A travelling shop | §8 |
| Encounters on the overworld's river banks, fords, bridges by their own landscape | a ford and a bridge are `road` landscape and roll as roads; a river cell is water, no local area, no roll |

## 13. Implementation plan

| Slice | Issue | What it touches | Depends on |
|---|---|---|---|
| **E1 Rules** | encounters: the roll, the checks, the pending choice, the positions | `warlock-core`: `content/encounter.rs` (the rules file, the tables, the load checks), `Content::encounter_tables`, `rng.rs::encounters`, `world/encounter.rs` (`encounter_roll`, the lookouts, `choose`, `open_area` with the positions, evade, `refuse_pending`), one line in `travel.rs::player_moves`, `Action::Encounter`, `Event::Encounter`, the replay text (`encounter:`, `meet:`), `WorldState` fields, the three skill benefits (`Effect::{Lookout, Stalk, Evade}`, `Use::Lookout`) on Spot, Survival, Stealth in `data/skills.toml`, Alert's extra read and tag, `PerceptionView::encounter`, the dumps (§10.2), `data/encounters.toml`; a fixture campaign's tables in tests; `world/encounter_tests.rs` (§9.3), `content/encounter.rs` tests; `docs/dev/encounters.md`, CLAUDE.md's action text | — |
| **E2 UI** | the dialog and the messages | `crates/warlock`: `input.rs::encounter_dialog` from the view in `Game::applied` and on load, the labels; `ui/screen_snapshots.rs` cases `encounter-drop`, `encounter-met`, `encounter-peaceful` (text snapshots; no golden: a dialog is text); `docs/dev/screens.md` one line | E1 |
| **E3 Content** | the Ascmearc's tables, the pedlar, the wiki, the spec | `campaigns/ascmearc/data/encounters/*.toml` (§7.4), `campaigns/ascmearc/data/creatures/pedlar.toml` and its dialogue, `villager`'s rotation; `xtask/src/wiki/encounters.rs` and `wiki/src/encounters.md`, `SUMMARY.md`, `TERMS.md`; spec/07c-generation.md; `world/ascmearc_tests.rs` (every table's regions exist, a replay through one forced encounter of each outcome, the level-1 road rate) | E1 (E2 for the snapshots of its texts) |
| **E4 Later** | encounters while resting on the overworld (§14 Q3), a peaceful party's trade (Q7), a `quest_variant` condition (Q6) if not in E1 | `world/rest.rs` one hook, `world/encounter.rs` | E1, the owner's answers |

E1 and E3 add saved state and actions (E1) and content (E3): two PRs at once at most, by the integrator's rule. Each PR runs the full check; E1's snapshots are the dumps' insta files; E2's are text snapshots; no golden anywhere (nothing here is the GPU's).

## 14. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| Q1 | **The rates** (§4.2): road 1 %/h, open land 3, forest and hills 5, dense forest and swamp 8, night × 2. Too many, too few? | as written; a day's march in the wild rolls about half an encounter, the fen about one, a road one in eight days. Tune in data after a play-through |
| Q2 | **Pass by is nothing** (§5.2). The alternative is a close-quarters fight when neither side noticed. | nothing: it is what two blind parties do, and it is Stealth's reward on the road |
| Q3 | **Camping**: should a rest on the overworld roll encounters? | later (E4): a quarter of the landscape's rate per hour of rest, the player's lookout by Listen's `hearing` instead of Spot (they are asleep), failed: ambushed; made: met. Not in E1–E3 |
| Q4 | **The ambushed head start**: one normal action of debt plus the initiative check. The check alone (no debt) is softer; two actions harsher. | one action: it is D&D's surprise round and the mirror of the player's ambush |
| Q5 | **Level gates** on heavy rows (`level.min`) against letting the dice fall. The campaign design's "a careless character dies" argues for none; the road's footpads (level 6, up to four) against a level-1 argue for the rate being the guard, not a gate. | gates only on the rows named in §7.4 (wight, troll, boggle, duergar, the draws); footpads ungated: the road's 1 %/h is the promise kept |
| Q6 | **A rolled quest's variant as a condition** (`wanted_wystan`): `when.quest_variant = { quest = "guild_wanted", variant = "wystan" }` reads `QuestRecord::roll.variant`. In E1, or E4? | E1: it is one more field and the guild's chain is the campaign's spine |
| Q7 | **The pedlar trades?** A shop without a floor is new; the pedlar talks and tells the ways. | talks only; a trading traveller is a shop-design question for later |
| Q8 | **Alert widened** to the lookout (its tags gain `awareness`), or left alone? | widened: one read, no new talent, and its description already fits |
| Q9 | **Where the choice is asked**: the dialog with Esc leaving the question pending (info screens allowed first), or a dialog that cannot be closed? | pending with Esc: a player should be able to check their pack before choosing |
| Q10 | **A fled party persists in the area** (§6.2) with no cap on parties in one area. | yes; the rates cap it in practice, and a wood with two bands in it is a story |
