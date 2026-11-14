# Warlock — The craftsperson: re-enabled, the Crafts trades, and a new level table

**Built; this file is history.** Current behaviour is in spec §8 "Talents" ("Crafts", "Patron") and "Professions", and `docs/dev/crafting.md` and `docs/dev/progression.md`.

Design document (2026-11-13; #1415), **approved by the owner on 2026-11-13** with the answers in §13. Written against `origin/main` at 926d4d874 (version 0.16.0). Nothing here is implemented; §12 maps it onto implementation issues, one PR each. It builds on what is there: the class tables and bonus picks (`docs/dev/progression.md` "Class tables"), creation picks and the heritage hand-outs (`docs/dev/progression.md` "Creation picks", `world/heirloom.rs`), the professions (`docs/design/professions.md`, `docs/design/recipes.md`, `docs/dev/crafting.md`: stock, recipes, the check and the grades, salvage, Cut and Set, brewing, Smelt), Temper and `Worked` (`world/temper.rs`, `magic/worked.rs`, `data/temper.toml`), item damage (`world/item_damage.rs`), traps (`docs/dev/traps.md`), the towns' class helpers (`docs/dev/npcs.md` "The aristocrat's servant", "The merchant's business partner"), and rolled quests (`docs/dev/quests.md`). Enchanting's design (#1414, `docs/design/enchanting.md`, in progress beside this one) is referenced where this class touches it, and depended on nowhere: every hook here stands without it.

Every number below is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate). Numbers are a first pass; balance is not a goal, and the everyday folk are poor fighters on purpose. The aim: a craftsperson plays differently from a warrior who happens to know Metalcraft, not because their blows are better (they are not), but because their hands do things no one else's do.

## Contents

1. Summary
2. The owner's brief, and what exists today
3. The Crafts tree: seven trades
4. Who may take a trade, and the race rule
5. The class revisited: stats, skills, kit, rolls, bonus trees
6. The level table
7. The abilities, one by one
8. The Enchanting hooks
9. What the player sees; the dumps; the wiki
10. Determinism, saves, and replays
11. Docs and tests
12. Implementation plan: the phases
13. Owner's answers (2026-11-13)
14. Choices made along the way

## 1. Summary

| Piece | Proposal |
|---|---|
| The trades | A new talent tree **Crafts** (`crafts`, after Heritage), seven talents, one per profession skill: **Smith** (Metalcraft), **Wright** (Woodcraft), **Gemcutter** (Jewelcraft), **Brewster** (Alchemy), **Pitman** (Mining), **Wortcunner** (Herbalism), **Spellwright** (Enchanting). Each trains its skill at **3d6** (rolled from its own stream, added to whatever the character has), hands down **the tools of the trade** (three steel ingots, four billets and a sinew, an uncut gem, herbs, a pick-axe, a scroll of identify), and makes the skill a **trade of yours**: a level-up roll on it throws the die one step up the ladder, as a class's known-for skill does. §3 |
| Who | **The craftsperson at level 1 only** (the owner, 2026-11-13): `needs = { creation = true, class = "craftsperson" }`, so never another class's creation pick, never a level's pick, never a bonus-tree pick. **Not** a trade the character's race teaches everyone: a new prerequisite `race_lacks = "metalcraft"` (`Need::RaceLacks`), read from `races.toml`'s guaranteed `skills` (never `optional_skills`), its words derived from data: "not to be a dwarf or a duergar". §4 |
| The craftsperson's two | The class gains `creation_bonus = { count = 2, trees = ["crafts"] }`: two **bonus picks at level 1** from the Crafts tree, on top of everyone's two creation picks, through the bonus-pick machinery that exists (`Due::bonus`, the trees' refusal words). The creation Talents step asks for the trades first; Random draws them; `--talents smith,wright,hardy,healthy` names them. §4, §5 |
| The class | Offered again (`hidden` removed). Stats unchanged. `skills` lose Metalcraft and Woodcraft (the trades are the choice now): Appraise and Negotiation `2d4+4`; no known-for (the trades carry it); the profession optionals go; **3 skill rolls** a level (a crafter lives by skill values); a **knife** in the kit; bonus trees **Grit, Delver, Scholar** (a worker, not a fighter). The showcase's forge and quarry characters name their trades in their scenario files. §5 |
| The table | Sturdy work, Masterwork, and Artisan go. In their place: **1** Time served and Eye for the make, **6** Waste not, **12** Patron, **18** Make do and mend, **25** Trapwright, **32** Field forge, **40** Quick hands, **50** Master's hand, **60** Maker's mark. Three actives (Make do and mend, Trapwright, Field forge), one town helper (Patron), five passives. §6, §7 |
| Enchanting | Spellwright is Enchanting's trade; Waste not's salvage yield covers whatever #1414 gives salvage; Maker's mark's capacity rule is a one-line hook for #1414's capacity; Patron's pool admits enchanting recipes when they exist. Nothing here waits on #1414. §8 |
| Phases | P1 the Crafts tree (a craftsperson at level 1 may take a trade; the class stays hidden until P2), P2 the class re-enabled with its two trades, Time served, and Eye for the make, P3 Waste not, P4 Patron, P5 Make do and mend, P6 Trapwright, P7 Field forge, P8 Quick hands and Master's hand, P9 Maker's mark. §12 |

## 2. The owner's brief, and what exists today

The brief (#1415, 2026-11-13): re-enable the craftsperson at character creation; redesign the whole class ("all the current abilities are shite": Sturdy work 18, Masterwork 50, Artisan 60 use equipment better and nothing more); **two free talents at level 1** from a new talent category **Crafts**, one per profession skill (Metalcraft, Woodcraft, Jewelcraft, Alchemy, Mining, Herbalism, Enchanting: settle the list), each starting the character at about **3d6** in that skill, and **none for a skill the race gets unconditionally**; abilities more interesting than using equipment better, built around the professions and around Enchanting as #1414 designs it.

| Part | Today |
|---|---|
| The class | `assets/data/classes/14-craftsperson.toml`: `hidden = true`; Strength +1, Dexterity 1d3, Intelligence +1, Presence −1d2; hp 3, mana 1, experience 90 %; `skills` Metalcraft and Woodcraft `2d6+10` (known for), Appraise and Negotiation `2d4+4`; optional Athletics 25 %, Jewelcraft 30 %, Mining 25 %, Search 20 %; 2 rolls; axes 1; table Sturdy work 18, Masterwork 50, Artisan 60; bonus picks at 10, 20, 30, 45, 55 from Weaponcraft, Guard, Delver; kit a hand axe, gloves, apron, boots, one of rope / boots / an iron helm, two candles, a ration, 25 gold. Usable through `--character`, scenarios (the showcase's `forge` and `quarry`), tests, and the screen snapshots |
| Class tables | `ClassDef::abilities` rows `{ level, talent }` of `class`-tree talents (`granted`) or ordinary talents handed out; `class::grant` gives them as the level comes and on a restore; `bonus_talents = { levels, trees }` adds `Due::bonus` picks, refused outside the trees ("You must choose a talent of Weaponcraft or Delver."); `held_already` turns a held talent into a pick (`docs/dev/progression.md` "Class tables") |
| Creation picks | `[levels] creation_talents` 2 (+ the class's `creation_talents`, the bard's 1; + Versatile's 1): a `Due { level: 1, talents }` opened by `World::begin` and spent by `take_creation` (the spec's picks, else `take_random` on `character/talents/<spec>`); `creation_offers` passes `bonus: 0` (`world/creation.rs:137`). A talent with `needs = { creation = true }` is met only at level 1 (Heritage). Hand-outs (`TalentEffect::is_handed_down`: `starting_gold`, `starting_gems`, `heirloom`) are given once by `World::hand_down` from their own streams |
| Talent needs | `Need::{Talent, Stat, Skill, WeaponSkill, AnyWeaponSkill, Chosen, Level, Race, Eyes, Class, Animal, Element, Lucky, Creation, Either}`; words by `Need::words`; `Candidate` carries `race: Option<&str>` |
| Races' skills | `RaceDef::skills` (guaranteed dice): dwarf and duergar Metalcraft and Mining; gnome Jewelcraft; halfling Herbalism; kobold Mining (and Detect traps, Disable device); no race the others. `optional_skills` (the dwarf's Jewelcraft 20 %) are chance, not a guarantee |
| Professions | Metalcraft (anvil), Woodcraft (a knife), Jewelcraft (Cut, Set, jewellery), Alchemy (brews): `data/recipes/<skill>/`, every recipe its skill's (#1387); the check `2d12 + stat modifier + skill_effect_of(skill, Craft) + TalentMods::check(stat)` against `difficulty + work`; grades Fine 6, Marvellous 12, Masterwork 20 (`data/recipes.toml`); a failure gives half the stock back (`yield` 50); Mining digs faster and finds ore (`data/mining.toml`), Smelt turns it to ingots at an anvil; Herbalism gathers (`data/herbs.toml`); Enchanting tells magic items by the hour (`identify_magic`) and crafts nothing yet |
| Temper | `world/temper.rs`, `data/temper.toml`: ten minutes at an anvil, `13 + 2 × attempts`; made, an improvement (to three) or a repair; **failed, the item is damaged and loses every improvement, and a damaged one is destroyed** |
| Item damage | `Worked::damaged` by acid, rust, fire, or a failed attempt (`ItemHarm`); a damaged item is a point worse until mended; only Temper repairs it; `World::item_protected` guards quest objects, uniques, of Proof |
| Salvage | one ability for everyone: a check per material, `yield` 50 % of its grams, a failed gem chipped, a natural 2 shatters it |
| Traps | `data/traps.toml`: shot, strike, burst, bolt, pit, alarm, teleport; `Trap { kind, known, charges, dir }` in `LevelState::traps`; whoever steps on one springs it, creatures know nothing of traps; a trap's kill earns nothing; no one but a level file places a trap |
| Class helpers | `[[town.servant]]` in a campaign's `data/towns/`: `talent`, `level`, `at`, `creature` by gender, `speech` by race; placed when the level is made for a holder of the talent (`stipend::place_servant`); the words in `[dialogue.stipend]` / `[dialogue.partner]`; Cēaptūn's squire (Stipend) and business partner (Invest) |
| Rolled quests | `QuestDef::roll` (`Pick::{Creatures, Tags, Objects{class, loot}}`, `Variants`), drawn on `roll_quest`'s stream, the goal a template over the target; `hand_in` by object id; `quest_roll:ID:TARGET` the dev draw |
| Shops | pay 30 to 40 % of a thing's worth (`pays`); a worked item's worth is ×2, ×3, ×5 by grade (`Worked::value`) |

## 3. The Crafts tree: seven trades

**Decision: a tree, one talent per profession skill, the craftsperson's at level 1 only, each a hand-out.** `data/talents.toml` gains `[[tree]] id = "crafts", name = "Crafts", description = "A trade learned young: a craftsperson's, at character creation."`, placed after `heritage` (both are creation-only trees, Crafts the craftsperson's; the Talents page and the pick lists show the trees in file order). Seven files under `data/talents/`, one each:

| Talent (id) | Skill | Dice | The tools of the trade (handed down, carried) | Closed to (by data, §4) |
|---|---|---|---|---|
| **Smith** (`smith`) | Metalcraft | 3d6 | 3 steel ingots (`piece:steel*3`, 1.5 kg: a knife's or a dagger's worth, three for a long sword) | dwarf, duergar |
| **Wright** (`wright`) | Woodcraft | 3d6 | 4 wood billets (`piece:wood*4`, 800 g) and a length of sinew (`piece:sinew`): a short bow's worth | — |
| **Gemcutter** (`gemcutter`) | Jewelcraft | 3d6 | one uncut small garnet or zircon (`gem:garnet` / `gem:zircon`, the roughest cut: the placement grammar gains `:uncut` if it lacks it) | gnome |
| **Brewster** (`brewster`) | Alchemy | 3d6 | 2 sprigs of moonwort and 2 glowcaps (`moonwort*2`, `glowcap*2`: a potion of healing's and a potion of mana's herbs) | — |
| **Pitman** (`pitman`) | Mining | 3d6 | a pick-axe (`pick_axe`, iron; readied in the Tool slot if it is free, else carried) | dwarf, duergar, kobold |
| **Wortcunner** (`wortcunner`) | Herbalism | 3d6 | 2 sprigs of comfrey and a sprig of vervain (`comfrey*2`, `vervain`) | halfling |
| **Spellwright** (`spellwright`) | Enchanting | 3d6 | a scroll of identify (`scroll_of_identify`, known by type as everything a run starts with is) | — |

The names are the trades' English words, old where English has one: a *wright* is a maker in wood (shipwright, wheelwright); a *brewster* the Middle English brewer; a *pitman* works the pit; *wortcunning* is the Old English word for the knowledge of herbs (`wyrtcunnung`), so a *wortcunner* knows them; a *spellwright* makes spells into things. The descriptions say what the trade is and what the character starts with, never the dice: "You were apprenticed to a smith: you start trained in Metalcraft, with three steel ingots, and your smithing grows as a smith's does."

**One effect kind, `trade`**, carries all of it:

```toml
[[talent]]
id = "smith"
name = "Smith"
tree = "crafts"
description = "..."
needs = { creation = true, class = "craftsperson", race_lacks = "metalcraft" }
effects = [{ kind = "trade", skill = "metalcraft", dice = "3d6", tools = ["piece:steel*3"] }]
systems = ["skills"]
```

| Part of `trade` | Rule | Where |
|---|---|---|
| `skill`, `dice` | **Handed down once**, as the talent is taken (`TalentEffect::is_handed_down`, `World::hand_down`'s list): the dice are rolled from the stream `character/trade/<spec>/<skill>` (its own, so no other roll moves: the kit's, Family wealth's, the optional skills' stand), plus the holder's `TalentMods::trade_bonus` (§7.1, Time served: the craftsperson's, held from level 1), and added to the skill with `Skills::add`, as a race's or a story's grant is: a human craftsperson's `class_skill` plus a trade's `3d6` sums, as two grants of one skill do today | `world/trade.rs` (new): `World::learn_trade`, one line in `hand_down` |
| `tools` | handed down with it: kit lines (`content/kit.rs`'s `KitLineSpec` grammar: `"piece:steel*3"`, `{ wield = "pick_axe" }`), each made as `kit.rs::give` makes a line (one `rng.index` among `a|b` alternatives from the same stream), carried (a `wield` line into the first free slot it fits, else carried), their labelled types known (`kit::known_types`, the rule for everything a run starts with); over the carrying limit as anything picked up (nobody starts Strained: the kit test takes the heaviest trade, the pick-axe, into its worst case) | the same |
| **a trade of yours** | the one effect the sum keeps: `TalentMods::trades: Vec<SkillId>`. Read in **one place**: `world/skill_roll.rs::skill_roll`, beside `ClassDef::is_known_for`: a roll on a trade throws the die one step up the ladder, as a known-for skill's does (the two never stack: one step). So a trade grows as a class's craft would; the Skills screen and the level-up screen's dice already show the step | `skill_roll` |

Why 3d6 and not a fixed 10: the brief's, and the other starting grants are dice. Why the craftsperson's and creation-only (the owner's answer): a trade is the class's own, learned young; a level's pick in a dungeon that trains Metalcraft from nothing reads wrong, and skill rolls are the way a skill grows. Why hand-outs rather than a `skill` bonus effect: a bonus (`skill { value }`, Skill focus's kind) would make the skill *read* higher without being trained in it (every "trained at all" rule, the profession rows, the recipes' `from` would need to ask the talent too); training it once is what every other grant does. Why the tools: a smith with no steel and no anvil is a smith in name; three ingots and the town's anvil make the first hour of a run a smith's.

## 4. Who may take a trade, and the race rule

**Decision: the craftsperson at level 1 only; never a trade the race teaches everyone; their two are bonus picks at level 1.**

| Rule | Decision | Why |
|---|---|---|
| Who | **a craftsperson, at level 1 only** (the owner, 2026-11-13): `needs = { creation = true, class = "craftsperson" }` (`Need::Class`, `Need::Creation`), so the trades are offered only to a craftsperson's creation picks: the Talents step, Random, `--talents`. Never a level's pick, never a bonus-tree pick (the class's bonus picks at 10 and later: `creation` is unmet), never any other class's pick. The class's two bonus picks and its ordinary creation picks both draw on them | the owner's answer: the trades are the class's own; a dwarven warrior who salvages well takes Heritage's gifts, not a trade |
| The race rule | `needs = { race_lacks = "metalcraft" }`, a new `Need::RaceLacks(SkillId)`: met unless the character's race's **guaranteed** `skills` (`RaceDef::skills`, `RaceDef::trains`) names the skill. `optional_skills` and a human's `class_skill` never count: a 20 % chance is no guarantee. A run without a character (a fixture) meets it, though `class` then fails it | the brief: "do not offer a profession skill the race gets unconditionally"; read from data, nothing in code names a race or a skill |
| Its words | derived: "not to be a dwarf or a duergar" (every race whose `skills` names the skill, in `races.toml` order, joined with "or"); a skill no race teaches everyone makes the need always met and its words empty, so `unmet` never lists it and the Talents page shows nothing for it | the refusal and the Talents page say what a dwarf cannot be, in the game's words; the wiki lists the races per trade |
| The craftsperson's two | the class file's new `creation_bonus = { count = 2, trees = ["crafts"] }` (`ClassDef::creation_bonus: Option<CreationBonus>`; `count` 1 to 8, `trees` non-empty, each a tree; the loader checks them). `World::begin` opens level 1's `Due` with `bonus: count` beside `talents: creation_count`; `World::bonus_trees_at(1)` returns its trees (the level table's `bonus_trees` for every other level, as today). Everything else is the bonus machinery that exists: a Crafts talent spends a bonus pick before an ordinary one, any other talent is refused while a bonus pick is due ("You must choose a talent of Crafts."), a bonus pick nothing fits is dropped (`settle_due`: a dwarf may take five of the seven, so none is ever dropped in practice) | one new field, no new flow; the words come free |
| The creation step | `creation_offers` passes the due's `bonus` (not `0`) and `World::creation_due`'s trees, so `ui/chargen_talents_screen.rs` lists the Crafts tree alone while trades are due, headed "Choose a trade (2 left)", then every tree for the ordinary picks as today. `take_random` draws among the offers the due allows (the trees while `bonus > 0`): a random craftsperson gets two random trades their race allows, then two random talents. `--talents smith,wright,hardy,healthy` learns in order: `learn` spends a bonus pick for a Crafts talent while one is due, else an ordinary pick (a human craftsperson's fifth pick may be a third trade) | `World::learn` → `take_talent`'s rule, unchanged |
| A third trade | allowed as an ordinary creation pick for a craftsperson (a third trade at `3d6` for one who spends an ordinary pick on it) | no reason to forbid; the picks are the price |
| A scenario's own character | `ScenarioDef::character` gains `talents = ["smith", "gemcutter"]` (optional): the picks a run of the scenario's own character takes in `start_run`, with or without the kit, as `--talents` would, each checked (`learn`'s refusals are load errors naming the scenario). The showcase's `forge` names `["smith", "gemcutter"]` for Wulfhere (human), `quarry` `["wright", "gemcutter"]` for Bram (a dwarf: Metalcraft and Mining are his race's, so Smith and Pitman are closed to him; the quarry needs Mining, which he has by race, and his race's Metalcraft keeps him a smith) | the forge scenario's smith must still smith when the class stops training Metalcraft itself |

## 5. The class revisited: stats, skills, kit, rolls, bonus trees

```toml
[[class]]
id = "craftsperson"
name = "Craftsperson"
description = "A smith, carpenter, or weaver who knows a trade inside out: sure hands, a patient mind, and few words."
stats = { strength = 1, dexterity = "1d3", intelligence = 1, presence = "-1d2" }
hp = 3
mana = 1
experience = 90
skills = { appraise = "2d4+4", negotiation = "2d4+4" }
optional_skills = { athletics = { chance = 25, dice = "2d4" }, search = { chance = 20, dice = "2d4" } }
rolls = 3
weapon_skills = { axes = 1 }
creation_bonus = { count = 2, trees = ["crafts"] }
abilities = [
    { level = 1, talent = "time_served" },
    { level = 1, talent = "eye_for_the_make" },
    { level = 6, talent = "waste_not" },
    { level = 12, talent = "patron" },
    { level = 18, talent = "make_do_and_mend" },
    { level = 25, talent = "trapwright" },
    { level = 32, talent = "field_forge" },
    { level = 40, talent = "quick_hands" },
    { level = 50, talent = "masters_hand" },
    { level = 60, talent = "makers_mark" },
]
bonus_talents = { levels = [10, 20, 30, 45, 55], trees = ["grit", "delver", "scholar"] }
kit = [
    { wield = "hand_axe" },
    "knife",
    { wear = "leather_gloves" },
    { wear = "leather_apron" },
    { wear = "leather_boots" },
    "candle*2",
    "ration",
    "gold_coin*25",
]
```

| Change | Why |
|---|---|
| `hidden` gone | the brief. `classes_offered` lists it; its row in the class step; `class-screen-*` snapshots change |
| `skills` lose Metalcraft and Woodcraft; `known_for` gone; the Jewelcraft and Mining optionals gone | the trades are the choice: a class that trains Metalcraft `2d6+10` beside a Smith pick at `3d6` makes Smith the pick that stacks and every other trade the poor one. The trades carry the known-for step (§3). Appraise and Negotiation stay: a crafter prices and sells their work |
| `rolls` 2 → 3 | a crafter's whole progression is skill values (the recipes' `from`, the `craft` curve, the `trade` step); the thief and the ranger have 3, the bard 4. The owner approved the 3 rolls (§13) |
| `creation_bonus` | §4 |
| a knife in the kit, the "something of their own making" line gone | every hand recipe (`knife = true`) and Trapwright's stakes want a knife; the old line was flavour that gave a helm one run in three |
| bonus trees Grit, Delver, Scholar | a worker: Grit's burden line (ingots are heavy: chain mail is 22 of them), Delver's digging and shops, Scholar's items and gems (Treasure hunter, Skill focus on a trade). Weaponcraft and Guard were a fighter's |
| the old table | Sturdy work, Masterwork, and Artisan: their files deleted (saves are pinned to the version, #1187: a save holding `sturdy_work` is refused as any save of another version is). §6 |

What stays: the stats (sure hands and a patient mind: Dexterity and Intelligence; few words: Presence), hp 3, mana 1, experience 90 %, axes 1, the hand axe (a tool as much as a weapon).

A fresh human craftsperson, then: two trades at `3d6 + 6` each (§7.1: 9 to 24, mean 16.5, where today's `2d6+10` runs 12 to 22, mean 17), their tools, three ordinary creation picks (Versatile), Appraise and Negotiation, a knife and a hand axe, 3 rolls a level plus Intelligence's. A Smith and Wright: steel for a dagger, wood and sinew for a bow, and Cēaptūn's anvil a walk away.

## 6. The level table

The everyday folk's shape (1, 6, 12, 18, 25, 32, 40, 50, 60; bonus picks at 10, 20, 30, 45, 55), two at level 1 as the aristocrat and the merchant have. Kinds: **P** passive (a `TalentMods` read), **A** active (a `ClassActive` in the abilities menu's Class group, never a key of its own), **H** a town helper (placed by the talent as the squire and the partner are).

| Level | Talent (id) | Kind | In one line |
|---|---|---|---|
| 1 | **Time served** (`time_served`) | P | each trade taken at creation starts 6 higher |
| 1 | **Eye for the make** (`eye_for_the_make`) | P | the describe screen reads an item's make: what it would come apart into, how much more work it will take, how hard its stuff is to work |
| 6 | **Waste not** (`waste_not`) | P | salvage gives back three quarters, a failed recipe loses no stock, a failed temper spoils nothing |
| 12 | **Patron** (`patron`) | H | a burgher in town commissions work from you and advances the materials |
| 18 | **Make do and mend** (`make_do_and_mend`) | A | mend a damaged item anywhere, in ten minutes, without an anvil; never destroys |
| 25 | **Trapwright** (`trapwright`) | A | rig a trap on a cell beside you from arrows, a billet, or a dug pit; its kills are yours |
| 32 | **Field forge** (`field_forge`) | A | set an ingot on a heap of stones: a makeshift anvil, anywhere, in an hour |
| 40 | **Quick hands** (`quick_hands`) | P | every craft in half the time |
| 50 | **Master's hand** (`masters_hand`) | P | every grade and every cut comes four points sooner |
| 60 | **Maker's mark** (`makers_mark`) | P | what you make bears your mark: worth half again, prized by your patron, and (with #1414) room for one affix more |

Why this order: the first six levels set the character up (the trades, their tools, the eye); Waste not at 6 answers the stock-poor start (every salvage and failure returns more); Patron at 12 opens the commission loop when the character has reached Cēaptūn and can make Fine work now and then, and its advanced materials feed the trades; Make do and mend at 18 lands where acid, rust, and fire start wearing gear; Trapwright at 25 and Field forge at 32 are the deep dungeon's (a trapper's corridor, a forge from salvage); Quick hands at 40 halves the long works (mail is two hours, plate four) once they are made; Master's hand and Maker's mark are a master's. The owner approved the order (§13).

## 7. The abilities, one by one

Every effect below is a `TalentEffect` kind, a `TalentMods` field, and **one read** (`docs/dev/progression.md` "The hooks"); every active is a `ClassActive` variant whose rule lives in its own module with one line in each hot file (`class_ability.rs`'s dispatch, `ability.rs`'s matches, `action.rs`'s text). Numbers are in the talent's effect where they are the talent's alone, in a `data/*.toml` where a table is needed (rigs, the patron).

### 7.1 Time served (level 1, passive)

`effects = [{ kind = "trade_bonus", value = 6 }]`, `TalentMods::trade_bonus`, read in **one place**: `World::learn_trade` (§3) adds it to a trade's dice as the trade is handed down. Order: `World::begin` grants the class's level-1 talents before `start_run` takes the creation picks, so a craftsperson's trades find it held; no one else can take a trade, so the bonus is the craftsperson's alone. It changes nothing after creation. "You served your time at a trade: each trade you take when you are made starts well above a dabbler's."

Why a bonus on the trade rather than a class skill: the class cannot know which two trades are coming; the trade reads the bonus as it is taken.

### 7.2 Eye for the make (level 1, passive)

`effects = [{ kind = "eye_for_make" }]`, `TalentMods::eye_for_make`, read in **one place**: `magic/describe.rs::lines` (the describe screen of any item the player carries, wears, or looks at on a cell they see; `PerceptionView::describe`), which adds up to three lines for a holder:

| Line | When | From |
|---|---|---|
| "Taken apart, it would give 2 steel ingots and the ruby." / "Nothing in it is worth saving." / "It would not come apart." | always / no material a profession works / protected (`World::item_protected`) | `piece::break_up(item, yield)` with the holder's yield (Waste not's 75 when held, else `salvage.toml`'s 50), named by `piece::names`; a gem whole; the words salvage itself would use. Never the odds |
| "It will take two more improvements at the anvil." / "It can be made no finer." / "It is damaged; an anvil would make it good, or your own hands." | an item with an `Aspect` (`worked::aspects` non-empty) | `Worked::improved.len()` against `temper.toml`'s `most`; `Worked::damaged`. Never `attempts` (that stays hidden: the difficulty's secret) |
| "Mithril is hard to work." / "Tin is soft and easy to work." / nothing | the main material's `work` ≥ +4 "very hard to work", +1 to +3 "hard to work", −2 or less "soft and easy to work", else nothing | `MaterialDef::work` in words |

A perception-only passive: no rule changes, no draw. The brief's "reading an item's make". It also tells a new player what salvage is for.

### 7.3 Waste not (level 6, passive)

`effects = [{ kind = "waste_not", salvage = 75, failed = 100 }, { kind = "temper_safe" }]`:

| Field | Read in | Rule |
|---|---|---|
| `salvage` | `world/salvage.rs`, where `yield` is read (`piece::break_up(item, yield)`) | the holder's yield replaces `salvage.toml`'s when higher: 75 % of each made material's grams back (at least one piece, as ever); a quarterstaff gives 6 billets, not 4; chain mail 16 ingots, not 11. A chipped or shattered gem is unchanged: the gem's fate is the lapidary's check, not waste |
| `failed` | `world/craft_make.rs::make_out`, where `Spend::back` reads `recipes.toml`'s `yield` | a **failed recipe** gives back `failed` % of its stock and object inputs (100: all of it; `count_for` of the grams, so an odd count never gives more than was taken). A brew is unchanged: with a `failure` object the inputs are spent for the murky draught (`Spend::back` 0 before `yield` is asked). A counted recipe that made none the same (all back). "The blade comes out brittle and you break it up for the metal, every ounce." |
| `temper_safe` | `world/temper.rs::tempered`, the failure arm | a **failed Temper or Whittle damages nothing**: the item is as it was, the attempt counted (`attempts` rises, so the next is 2 harder: the difficulty climbs toward impossible on its own, and no farming is worth it). "You know when to stop before the steel spoils." A natural 2 the same. A damaged item's failed attempt no longer destroys it |

Why these three together: they are one idea, "nothing is lost in your hands but time", the house rule's spirit (time is the penalty). Why at 6: the stock-poor start is where half a long sword's steel matters.

### 7.4 Patron (level 12, a town helper)

`effects = [{ kind = "patron" }]`, `TalentMods::patron`. A **patron** is a town's class helper of the third kind (`[[town.servant]]` with `talent = "patron"`, placed by `stipend::place_servant` for a holder when the town's level is made, never twice): Cēaptūn's a well-to-do burgher at the Gæsthūs, `patron` (`campaigns/ascmearc/data/creatures/`; posh, as a person of means, then the race registers of `docs/dev/npcs.md`), speaking `campaigns/ascmearc/data/dialogue/patrons.toml` with `[dialogue.patron]` (`PatronWords`: `offer` with `{work}` and `{stock}`, `waiting` with `{work}`, `delivered` with `{gold}`, `plain` (a plain piece refused), `idle`, `address` by gender). `World::called` names them "your patron". The rules are `world/patron.rs`, the numbers `data/patron.toml` (`content/patron.rs`; shared: it names no campaign's recipes).

| Rule | Decision |
|---|---|
| A commission | a chat with the patron (`World::patron_chat`, one line in `chat` after the partner's) offers one when none is open: "I want a long sword, Fine or better. Here is the steel for it." The **work** is drawn (`world/patron.rs::draw_commission`, stream `patron/<n>`, `n` the count of commissions ever taken, saved in `WorldState::patron: Patron { taken, open: Option<Commission> }`) from the **pool**: every generic recipe (no `use`, a count of 1) of every profession the player is trained in whose `from` the player's trained value reaches and whose result has an `Aspect` (arms, armour, shields, helmets, bows, cloaks, gloves, boots: the graded things; never jewellery, potions, tools, clothing, counted goods), in path order, one `rng.index`. An empty pool (no graded recipe reachable): "Come back when your hands are worth hiring." |
| The advance | with the offer, the patron hands over the recipe's **materials at its base**: for each `"stock"` part or stock table the count `piece::count_for` of the part's grams in the material the row prices it in (`craft::stock_words`' rule: the base material, else the piece's), as pieces; each object or stock input (a sinew, an ingot for a crossbow's lock); never a gem (the ornament is the maker's). Through `World::receive` (the stipend's: what would leave the player Overloaded is set down at their feet) |
| The quest | the commission is a quest in the log while open (`Quest::Commission`, `kind = "commission"`, the goal "Bring {patron} a Fine {work} or better", giver the patron): shown as any quest is, counted as none (no XP, no guild rank, no Invest bonus, no record in `completed`: `QuestDef::counts = false`, read where quests are tallied, one place) |
| Hand-in | a chat with the patron while one is open: the first stack the player carries of the work's object with a `Worked` of at least one improvement (Fine or better) is taken; none: `waiting` ("The steel I gave you was for a long sword."); a plain one carried: `plain` ("That is a long sword, and no more; I asked for Fine work."). Whose work it is, the patron does not ask (there is no maker state before level 60: §7.10); a found Fine sword serves |
| Pay | `pays = 100` % of the delivered item's worth (`Stack::value`: the grade's ×2, ×3, ×5 counted; a Fine steel long sword 160 gold where a shop pays 48), plus `bonus = 50` % more for a piece that bears the player's Maker's mark (§7.10). Then the next may be asked for at once |
| Limits | **one commission open at a time** (the shape of a commission, not a lock-out: deliver and take the next); no cap, no cooldown, no refusal by count. The road to town and the hour at the anvil are the price |
| Never delivered | nothing: the patron waits, the stock is the player's (worth coins), no second commission until the first is done. Salvaging the advanced stock gives pieces; nothing stops it |
| The patron dies | the open commission is dropped from the log, `Patron::open` cleared (`World::patron_dies`, one line in `fight.rs::dies` beside `partner_dies`); no new patron |
| Marks | `Use::Deal` on Negotiation at a hand-in (a sale is a sale) |

Why a helper and a quest rather than a shop that buys dear: the brief's "commissions"; the advance solves the smith's first problem (stock) in the world's own voice; the quest log shows it; the servant and partner code carries nine tenths of it. Why graded things only: "Fine or better" is the commission's teeth; jewellery and potions have no grade (recipes.md §4). The owner approved it as a first version (§13).

### 7.5 Make do and mend (level 18, active)

`effects = [{ kind = "make_do", seconds = 600, difficulty = 10 }]` → `ClassActive::MakeDo`, `Aim::Item` over `ObjectList::MakeDo` (`world::mendable`: stacks carried, not worn or wielded, whose `Worked::damaged` is true), `Action::UseOn` as Merchant's eye (`use:make_do_and_mend:c`), the rule in `world/make_do.rs`:

| Step | Rule |
|---|---|
| Refused, no time | no such stack ("You carry no such thing."); not damaged ("There is nothing wrong with it."); worn or wielded ("Take it off first."); a main material no profession works (`profession_of` none: cloth, glass, wax; "You know no craft that would mend it."); protected (`item_protected`, a unique or a quest's: "You dare not work on it."); unpaid goods (`may_loose`) |
| Time | `seconds` (ten minutes; Quick hands halves it) by `keep_at`, `Spent::MakeDo`, `ActionKind::Craft` turns; interrupted, nothing, the time spent ("...; you put down your work.") |
| The check | at the end, the stack re-found: `2d12 + the main material's profession's stat modifier + skill_effect_of(that skill, Craft) + TalentMods::check(stat)` against `difficulty + work` (steel 10, mithril 14, wood 9); untrained in that profession, the stat alone (salvage's rule). Two gameplay draws |
| Made | `Worked::repair()`: `damaged` false, `by` cleared; the improvements it lost when it was damaged stay lost (that is what damage did); `attempts` untouched (a Temper later is as hard as it was). "You make good the damage: it is a steel long sword again." The item's identification unchanged |
| Failed, a natural 2 | nothing: the item stays damaged, nothing is destroyed. "You cannot make it good here; it will need an anvil." Try again: time is the penalty |
| Marks | `Use::Craft` on the profession, if trained |

Why safe where Temper is not: Temper's failure destroys because Temper also improves; mending alone is the craftsperson's and risks nothing but time. Why not at an anvil: the whole point is the field; the anvil has Temper. Why not worn items: Temper's rule, and a mend is not done on the body.

### 7.6 Trapwright (level 25, active)

`effects = [{ kind = "rigs" }]` → `ClassActive::Rig`, a new aim `Aim::Rig`: the abilities menu opens the **rig list** (`ui/rig_screen.rs`, a `PagedList` as the shape list is: each rig with what it takes and, greyed, why not now), a pick asks a **direction** (the direction prompt every `Aim::Direction` ability uses), and the action is `Action::Rig { rig, dir }`, text `rig:tripwire:E`, `ActionKind::Craft`. The rule is `world/rig.rs`, the data `data/rigs.toml` (`content/rig.rs`: `RigRules { difficulty = 10, [[rig]] }`; optional: without the file the active is "not usable yet"):

| Rig (id) | Trap placed (`data/traps.toml`, new rows, no `grows`) | Takes | Seconds | Charges | Needs |
|---|---|---|---|---|---|
| **Tripwire** (`tripwire`) | `rigged_arrows`: `shot`, object `arrow`, `to_hit` 2, `damage` "1d6" piercing, bowstring 40 | 1 to 3 arrows (the stack the player readies or carries first; as many as 3), spent | 300 | one per arrow | a wall within 2 cells of the cell (a shot comes from a slit in the nearest wall: `spring_trap`'s rule) |
| **Stakes** (`stakes`) | `rigged_stakes`: `strike`, noun "the stakes", `to_hit` 2, `damage` "1d6" piercing | a wood billet; a knife carried | 600 | 2 | — |
| **Pitfall** (`pitfall`) | `rigged_pit`: `pit`, `damage` "1d6" bludgeoning, the climb | a digging tool ready in the Tool slot (`dig::tool`, a pick-axe or a mattock) | 1200 | 3 (it fills in) | — |

| Rule | Decision |
|---|---|
| The cell | beside the player: walkable floor, free of actors, traps, chests, doors, shop floors, water, ice, and fungus; a cell of a level, never the overworld (`Unavailable::NotHere`). Refused with the reason, no time |
| The work | `seconds` by `keep_at` (`Spent::Rig`); interrupted, nothing |
| The check | at the end, the inputs re-found: `2d12 + Dexterity modifier + skill_effect_of(Woodcraft, Craft) + TalentMods::check(Dexterity)` against `difficulty` (10): a trained wright rigs better, anyone with the talent rigs. Made: the trap is placed, **known** to the player (`Trap { kind, known: true, charges, rigged: true }`), the inputs spent; "You rig a tripwire across the way." Failed: nothing placed, the inputs back, the time gone ("The rig will not hold."). **A natural 2 springs it on you**: the trap is placed and sprung at once with the rigger as its victim (`spring_trap`), then stands with a charge less ("The tripwire snaps as you set it!") |
| `Trap::rigged` | a new flag (saved, default false): a kill by a rigged trap is **the player's** (`spring_trap` passes the player as the slayer to `World::dies`: XP, the kill tally, quests, `train_kill`, as a kill by their own arrow is); its damage is the trap's own, never grown by danger; it is `known`, so stepping onto it asks and may be sidestepped (`world/sidestep.rs`) |
| Creatures and allies | creatures know nothing of traps and spring it; **allies too** (they path as creatures do): a rigged corridor is no place to call your hound through. Allies knowing the player's rigs is a later line (§13) |
| Dumps | `truth`: a rigged trap's line ends `(rigged)`; `traps` lists it as any known trap |

Why rigs and not recipes: a recipe makes a thing in the pack; a rig makes a cell; the profession screen has no direction, and Trapwright is the class's, not Woodcraft's (a Smith-and-Gemcutter rigs too, at Dexterity alone). Why these three: they come of what a crafter carries (arrows, billets, a pick), and `shot`, `strike`, and `pit` exist; a flask on a tripwire (a `burst` of the flask's throw) is a fourth row for a later issue once flasks have a throw table a trap can read. Why a check at all: a rig that fails costs the time, the one penalty; the natural 2 is the trapper's old joke.

### 7.7 Field forge (level 32, active)

`effects = [{ kind = "field_forge", seconds = 3600 }]` → `ClassActive::FieldForge`, `Aim::Direction`, `Action::Use` with a direction (`use:field_forge:E`, Smite's shape), the rule in `world/field_forge.rs`:

| Rule | Decision |
|---|---|
| Takes | one **ingot of any metal** carried (a `piece` of the `metal` group; the first stack found, as a brew takes its herbs), spent at the end |
| The cell | as a rig's (§7.6): walkable, free, no fixture, not the overworld |
| The work | an hour (`seconds`; Quick hands halves it) by `keep_at`, `Spent::FieldForge`; interrupted, nothing |
| Done | `set_terrain(cell, makeshift_anvil)` and the ingot spent: "You set the block on a heap of stones: a makeshift anvil." No check (an hour's labour, not a craft) |
| The terrain | `terrain.toml` gains `makeshift_anvil`: name "makeshift anvil", the anvil's glyph and colours a shade duller, `walkable = false`, `stone = true`, **`works_as = "anvil"`** (a new optional `TerrainDef` field: a station this terrain serves as), `hardness = 300` (a pick-axe clears it in a few minutes if it stands in the way; nothing comes of digging it: `mine` gives nothing off a hand-placed terrain, and `break_wall`'s `becomes` is the floor). Placed after `anvil` in the file |
| `works_as` | read in **one place**, the station test every recipe's `lack` and `start_making` share (`anvil_beside` → `World::station_beside(station)`: a neighbouring terrain whose id is the station **or whose `works_as` is**); `Offer::Temper`'s walk-in (`input.rs::step_screen`) asks `TerrainDef::is_station("anvil")` the same way, so walking into a makeshift anvil opens Metalcraft's screen as the real one does. Smelt works there too: a Pitman-and-Smith turns the dungeon's ore to ingots on the spot |
| It stays | a terrain of the level, saved with it, as any `set_terrain` is; the player may clear it with a pick. It blocks a cell as an anvil does |

Why an ingot: the anvil is the block; the stones are anywhere. Why an hour and not a check: setting a block is labour; the forging after it has the check. Why not a portable anvil in the pack: a station is a cell beside the maker everywhere in the recipe code; a thing in the pack would need a second notion of station. The owner accepted that one ingot and an hour leave a permanent block (§13).

### 7.8 Quick hands (level 40, passive)

`effects = [{ kind = "craft_time_percent", value = 50 }]`, `TalentMods::craft_time_percent` (the time a craft takes, in percent of the data's; Fletcher's `time_percent` convention), read in **one place**: a new `World::craft_seconds(base)` that every craft's `keep_at` loop asks for its seconds (a recipe's `seconds` in `start_making`, Temper's and Whittle's `temper.toml` `seconds`, Cut's and Set's, Salvage's, Make do's, a rig's, Field forge's, Smelt's as a recipe). Fletcher's share multiplies with it (an arrows batch by a ranger with Quick hands: a quarter of the time). Gather is not a craft (Herbalism's own rule) and keeps its time. "Your hands know the work: every craft in half the time."

Why one function: eight loops would be eight reads; the one place is the function they all call, and the test is one.

### 7.9 Master's hand (level 50, passive)

`effects = [{ kind = "grade_ease", value = 4 }]`, `TalentMods::grade_ease`, added to the **margin as it is graded** and never to the check itself (pass or fail is unchanged): two reads, `craft_make::grade(settings, natural, margin + ease)` and `cut::cut_band(gems, cut_step, natural, margin + ease)`. So for the holder Fine comes from a margin of 2, Marvellous from 8, Masterwork from 16; a gem's cut a step sooner (crude 0 to 0, fair from 1, fine from 6, exquisite from 11, flawless from 16). Set has no grade. With the wiki's odds (recipes.md §4): a master smith (+16) on a steel long sword makes a Masterwork 44 % of the time, not 25; an apprentice (+4) sees Marvellous 19 %, not 7. "A master's hand: every grade and every cut comes sooner."

### 7.10 Maker's mark (level 60, passive)

`effects = [{ kind = "makers_mark", worth_percent = 50 }]`, `TalentMods::makers_mark`, read in **one place**: `craft_make::make_out`, where a made item is built: a holder's work (a graded recipe's result, jewellery too; never a counted batch or a brew) is built with `Worked::marked = true` (a new field, saved, default false; a plain marked item carries a `Worked` with nothing else on it). What the mark does is then the mark's, read where each thing is read:

| What | Where | Rule |
|---|---|---|
| Its worth | `Worked::value` | `worth_percent` more (×1.5), on top of the grade's multiplier |
| Its words | `magic/describe.rs::lines` | "It bears Brakka's mark." (the character's name; the item's name is unchanged: "a Fine ruby-pommelled mithril long sword" is long enough) |
| The patron | `world/patron.rs` (§7.4) | `bonus` 50 % more for a marked piece |
| Enchanting (#1414) | the capacity rule, when it lands | a marked item holds **one affix more** than its quality, material, and ornament give it (one line where capacity is summed; §8) |

Why a mark and not a bigger number: it is what a master's work is in the world, a name on it; and it gives #1414 and the patron one thing to read.

## 8. The Enchanting hooks

#1414 is being designed beside this document (the owner's brief: affix capacity 1 to 6 from quality, material, and the set gem; magic crystals from salvaging magic items or a necromancy spell; recipes after the currency-crafting effects; rare materials from draining powerful foes). What this class touches, each a line in that design's terms and nothing more:

| Here | There | Dependence |
|---|---|---|
| Spellwright (§3) | Enchanting's trade: `3d6` Enchanting, a scroll of identify; the enchanting recipes list from its `from` values as any profession's | none: Enchanting is a skill today |
| Waste not's `salvage` 75 % (§7.3) | if salvage yields crystals by a percent, the holder's percent applies to crystals as to stock (the one `yield` read) | none: the read is where `yield` is |
| Patron's pool (§7.4) | admits enchanting recipes if any has a graded result (none expected: an enchantment changes an item, it makes none); otherwise untouched | none |
| Maker's mark (§7.10) | a marked item's capacity +1: one line in the capacity sum | the hook is written here; #1414's implementer adds the line; until then the mark is worth and words |
| Field forge (§7.7) | if enchanting wants a station (#1414 asks "anywhere? a station?"), `works_as` can serve a second station id on the same terrain (`works_as = ["anvil", "workbench"]`) | none |

Nothing in this class is unusable without #1414, and nothing in #1414 need read this class but the one capacity line.

## 9. What the player sees; the dumps; the wiki

**Hidden numbers.** The game never says the dice (3d6, +6), a yield percent, a difficulty, a margin, or a time percent; it says what the trades start with, that salvage gives back more, that a failure loses nothing, that a craft takes half the time (time in words, `Duration::spoken`, is shown already), that a grade comes sooner. The patron names the work and the grade. Eye for the make speaks in words (§7.2). The wiki states every number.

**The menu and the screens.** The three actives sit in the abilities menu's Class group (`Source::Class`), named by their talents, with `Aim::Item` (Make do and mend: the mendable list), `Aim::Rig` (Trapwright: the rig list, then a direction), `Aim::Direction` (Field forge); unavailable rows say why (`Unavailable::NotHere` on the overworld, `NotYet` without `rigs.toml`, a new `NothingToMend` with nothing damaged carried). The creation Talents step heads the trades "Choose a trade (2 left)"; the summary's Talents row names them with the picks. The patron is an NPC with a chat and no topics beyond the chat (the offer and the hand-in are the chat itself, as the squire's handover is).

**Dumps.** `talents`: the trades as picks (plain, in the order taken); `abilities`: `Make do and mend (class): an item`, `Trapwright (class): a rig and a direction`, `Field forge (class): a direction`; `truth`: `(rigged)` on a rigged trap's line; `quests`: the commission while open; `progress` (dev): `patron 3 taken, open long_sword`; `recipes` unchanged. JSON: `talents[]` as today; `quests[]` with `kind: "commission"`; a trap's `rigged` in `known_traps`.

**Wiki.** `wiki/src/classes.md`: the craftsperson's paragraph (offered again; the two trades; the table in words) and the "For now a new run offers…" sentence updated; `wiki/src/talents.md`: a Crafts section (what a trade is, the craftsperson's at creation only, the race rule in words) with a generated `crafts-trades.md` (`xtask/src/wiki/talents.rs::trades`: talent, skill, dice, the tools, the races it is closed to, read from `race_lacks` and `races.toml`); `wiki/src/crafting.md`: a "The craftsperson" section linking the class and stating Waste not's, Quick hands', Master's hand's, Maker's mark's numbers, Make do and mend's check, Field forge's hour and ingot; `wiki/src/traps.md`: "Rigging traps" with generated `rigs.md` (`xtask/src/wiki/traps.rs::rigs`: rig, trap, takes, time, charges, damage); a new `wiki/src/patron.md` under People (the pool rule, the pay, the advance) or a section of `quests.md`; `talents-by-system.md` picks the class abilities up by their `systems`. `wiki/TERMS.md` gains: trade, Crafts, tools of the trade, rig, Trapwright, patron, commission, makeshift anvil, Maker's mark, Make do and mend. Prose for players (#1172).

## 10. Determinism, saves, and replays

| Piece | Rule |
|---|---|
| Streams | a trade's dice and tools from `character/trade/<spec>/<skill>`; a commission's draw from `patron/<n>`; the Talents step's Random from `character/talents/<spec>` as today (its draws change for a craftsperson: two more picks); every check (Make do, a rig) from the gameplay stream in a fixed order; marks from the training stream |
| New saved state | `Trap::rigged: bool` (default false), `Worked::marked: bool` (default false), `WorldState::patron: Patron { taken: u32, open: Option<Commission { object, pays }> }` (left out while unused), the commission quest in the log, `makeshift_anvil` cells in a level's terrain. Saves are pinned to the version (#1187): no shim, no alias; a save holding `sturdy_work`, `masterwork`, or `artisan` is refused as any other version's save is (`Talents::unknown_to`) |
| Hashes that move | every craftsperson run with the kit (two more creation picks: the Random draws, the trades' skills and tools), the forge's and quarry's scenario characters (`talents` in their scenario files; the golden manifest's and the screen snapshots' craftsperson cases are re-recorded in P2), a human craftsperson's `class_skill` draw is unchanged (its own stream). Nothing else: a run without a trade or a craftsperson draws as before |
| Replays | new actions `use:make_do_and_mend:LETTER`, `rig:ID:DIR`, `use:field_forge:DIR`; `--talents` names trades; a replay's `character.talents` carries them as it carries every pick |
| Content | talents are not fingerprinted (saves name them by id); the new terrain row sits after `anvil`; `data/rigs.toml`, `data/patron.toml`, the trap rows, the dialogue file, the town's `[[town.servant]]` entry are content as any |

## 11. Docs and tests

**Docs.** `docs/dev/progression.md` (the Crafts tree, `Need::RaceLacks`, the `trade` hand-out and its one read, `creation_bonus`, every new effect's hook row, the class table), `docs/dev/character-creation.md` (level 1's bonus picks, a scenario's `talents`), `docs/dev/crafting.md` (Waste not's three reads, Make do and mend, Field forge and `works_as`, Quick hands' `craft_seconds`, Master's hand, Maker's mark), `docs/dev/traps.md` (rigs, `Trap::rigged`), `docs/dev/npcs.md` (the patron, the third helper), `docs/dev/quests.md` (commissions, `counts`), `docs/dev/objects.md` (`Worked::marked`), `docs/dev/skills.md` (a trade's known-for step), CLAUDE.md's action-syntax list and the `--talents` note, spec §8 "Class abilities" (the craftsperson's bullet), "Talents" (the Crafts tree, the race rule), "Traps" (rigs), §7 the makeshift anvil, §8 "Dialogue" (the patron).

**Tests.** `content/talent.rs` (the `trade` effect's parsing and refusals: a skill that is no skill, dice, tools that do not resolve; `race_lacks` resolving; the Crafts tree), `world/trade_tests.rs` (each trade's skill and tools, the known-for step on a roll, a dwarf refused Smith and Pitman in words and offered the other five, a gnome refused Gemcutter, the stream moving no other roll, Time served's +6, a warrior (or any non-craftsperson) refused every trade in words, a craftsperson's third trade as an ordinary pick, no trade offered as a bonus-tree pick at 10, Random's two trades first, `--talents` in and out of order, a scenario's `talents`), `world/class_craft_tests.rs` (the table by level, the bonus picks and their trees, every passive's rule, the three actives' refusals with no time and their draws, restores and replays), `world/make_do_tests.rs`, `world/rig_tests.rs` (each rig, the wall rule, the kill credited, the natural 2, the sidestep of one's own trap), `world/patron_tests.rs` (the pool by training and `from`, the advance's counts, a Fine hand-in paid and a plain one refused, one at a time, the death, a save), `world/field_forge_tests.rs` (the ingot spent, Temper and a forge at the makeshift anvil, the walk-in, clearing it), `content/rig.rs`, `content/patron.rs`, `content/terrain.rs` (`works_as` resolving), the screen snapshots (the class step with the craftsperson, the Talents step's trade heading, the rig list, the mendable list, the patron's chat), the wiki's tables.

## 12. Implementation plan: the phases

Each phase one issue and one PR, refined from the section named; the order respects the dependencies. P1 before everything; P2 before the rest; P3 to P9 in any order after P2 (P9's patron bonus needs P4).

| Phase | Builds | Sections | Files |
|---|---|---|---|
| **P1: the Crafts tree** | the `crafts` tree; the seven talent files; `Need::RaceLacks` with its derived words; `TalentEffect::Trade` (handed down: dice from its stream plus `trade_bonus`, tools as kit lines, `TalentMods::trades` and `trade_bonus`); the one read in `skill_roll`; `--talents` and Random unchanged in flow; the wiki's `crafts-trades.md` and Talents prose. A craftsperson (`needs.class`) may take a trade from this PR; the class stays hidden until P2, so tests and `--character` reach it | §3, §4 (the rule and its words) | `data/talents.toml`, `data/talents/{smith,wright,gemcutter,brewster,pitman,wortcunner,spellwright}.toml`, `content/talent.rs`, `talent.rs`, `world/trade.rs` (new), `world/heirloom.rs` (one line in `hand_down`), `world/skill_roll.rs`, `content/kit.rs` (a gem's `:uncut` if needed), `xtask/src/wiki/talents.rs`, `wiki/src/talents.md`, `wiki/TERMS.md`, `docs/dev/progression.md`, `docs/dev/skills.md`, spec §8 "Talents" |
| **P2: the craftsperson re-enabled** | the class file (§5); `ClassDef::creation_bonus` and level 1's `Due::bonus` with its trees (`begin`, `bonus_trees_at`, `creation_offers`, `take_random`, the Talents step's heading); Time served (`trade_bonus`) and Eye for the make (`describe.rs`); `ScenarioDef::character.talents` and the forge's and quarry's; Sturdy work, Masterwork, Artisan deleted; the golden manifest's and the snapshots' craftsperson runs re-recorded; the wiki's classes page | §4 (the two), §5, §7.1, §7.2 | `data/classes/14-craftsperson.toml`, `data/talents/{time_served,eye_for_the_make}.toml`, `content/class.rs`, `content/scenario.rs`, `world/creation.rs`, `world/talent.rs`, `world/class.rs`, `magic/describe.rs`, `ui/chargen_talents_screen.rs`, `campaigns/showcase/scenarios/{forge,quarry}.toml`, `ui/screen_snapshots.rs`, `tests/golden/manifest/*` (if any names the class), `xtask/src/wiki/`, `wiki/src/classes.md`, `docs/dev/character-creation.md`, `docs/dev/progression.md`, spec §8 "Class abilities" |
| **P3: Waste not** | `waste_not { salvage, failed }` and `temper_safe`, three reads | §7.3 | `data/talents/waste_not.toml`, `world/salvage.rs`, `world/craft_make.rs`, `world/temper.rs`, `talent.rs`, `docs/dev/crafting.md`, `wiki/src/crafting.md` |
| **P4: Patron** | `data/patron.toml`, `content/patron.rs`, `world/patron.rs` (the pool, the draw, the advance, the chat, the hand-in, the pay, the death), `WorldState::patron`, the commission quest (`kind = "commission"`, `counts = false`), Cēaptūn's `[[town.servant]]` and `patron` creature and `patrons.toml` dialogue (posh, then the race registers), the `progress` dump row, the wiki page | §7.4 | the above, `content/town.rs`, `world/stipend.rs` (the placing shared), `world/quest.rs`, `world/talk.rs` (one line), `fight.rs` (one line), `campaigns/ascmearc/data/{towns,creatures,dialogue}/`, `dump.rs`, `docs/dev/npcs.md`, `docs/dev/quests.md`, `wiki/src/` |
| **P5: Make do and mend** | `ClassActive::MakeDo`, `ObjectList::MakeDo`, `world/make_do.rs`, `Unavailable::NothingToMend`, `--screen make-do`, the snapshot | §7.5 | `data/talents/make_do_and_mend.toml`, `content/class.rs`, `ability.rs`, `world/class_ability.rs` (one line), `world/make_do.rs` (new), `world/mod.rs` (`Spent`), `ui/` (the list's screen name), `docs/dev/crafting.md`, CLAUDE.md |
| **P6: Trapwright** | `data/rigs.toml`, `content/rig.rs`, the three trap rows, `Trap::rigged` and the kill credited, `ClassActive::Rig`, `Aim::Rig`, `ui/rig_screen.rs`, `Action::Rig` (`rig:ID:DIR`), `world/rig.rs`, the `truth` mark, the wiki's `rigs.md` | §7.6 | the above, `data/traps.toml`, `content/trap.rs`, `world/trap.rs` (the slayer), `action.rs`, `ability.rs`, `input.rs` (the list then the direction), `dump.rs`, `xtask/src/wiki/traps.rs`, `wiki/src/traps.md`, `docs/dev/traps.md`, CLAUDE.md |
| **P7: Field forge** | `TerrainDef::works_as`, `World::station_beside`, the `makeshift_anvil` row, `ClassActive::FieldForge`, `world/field_forge.rs` | §7.7 | `data/terrain.toml`, `content/terrain.rs`, `world/craft.rs` (the station test), `input.rs` (the walk-in), `world/field_forge.rs` (new), `world/class_ability.rs` (one line), `docs/dev/crafting.md`, spec §7 |
| **P8: Quick hands and Master's hand** | `craft_time_percent` and `World::craft_seconds` (every loop calls it), `grade_ease` (two reads), the wiki's odds tables gain a "with Master's hand" column | §7.8, §7.9 | `data/talents/{quick_hands,masters_hand}.toml`, `world/craft_make.rs`, `world/temper.rs`, `world/cut.rs`, `world/set.rs`, `world/salvage.rs`, `world/make_do.rs`, `world/rig.rs`, `world/field_forge.rs`, `xtask/src/wiki/recipes.rs`, `docs/dev/crafting.md` |
| **P9: Maker's mark** | `Worked::marked`, set in `make_out`; `Worked::value`; the describe line; the patron's bonus; the capacity hook noted for #1414's implementer | §7.10, §8 | `data/talents/makers_mark.toml`, `magic/worked.rs`, `world/craft_make.rs`, `magic/describe.rs`, `world/patron.rs`, `docs/dev/objects.md`, `docs/dev/crafting.md` |

Later, unfiled: a flask on a tripwire (a `burst` rig) once flasks have a throw table a trap can read; potion and jewellery commissions ("a potion of greater healing": no grade, so the patron pays for the thing); the patron in a second town; allies that know the player's rigs (§13); a trade taught by a town's teacher for gold (Eadric teaches Metalcraft already: `train_skill`), which would make a trade learnable later without a pick.

## 13. Owner's answers (2026-11-13)

The owner approved the design with these answers; every question not named below took the document's proposed default.

1. **Who may take a trade.** **The craftsperson at level 1 only** (the proposed default, anyone at creation, was overruled): not a creation pick of any other class, not a level's pick, not a bonus-tree pick. The race rule stays. (`needs = { creation = true, class = "craftsperson" }`.)
2. **Creation-only.** A trade is never a level's pick. Default taken: **creation-only**.
3. **The seven.** Metalcraft, Woodcraft, Jewelcraft, Alchemy, Mining, Herbalism, Enchanting. Default taken: **all seven**.
4. **The dice and Time served.** `3d6`, and `+6` for a craftsperson. Default taken: **yes**.
5. **The tools of the trade.** Each trade hands down its small bundle. Default taken: **yes**.
6. **A trade is known-for.** A roll on a trade throws the die one step up. Default taken: **yes**.
7. **The class's skills and rolls.** No profession in `skills`; Appraise and Negotiation `2d4+4`; 3 rolls a level. Default taken: **as written**.
8. **Bonus trees** Grit, Delver, Scholar. Default taken: **yes**.
9. **The race rule reads guaranteed skills only.** Default taken: **guaranteed only**.
10. **The order of the table** (§6). **Approved as written** (the owner: "the level table is approved as written").
11. **The patron.** **Approved as a first version** (the owner: "just do something as first version"): the design stands as written, open to later change (the pay, the advance, the pool, a second town).
12. **Rigs.** Default taken: **as written**; allies that avoid the player's known rigs is a later line.
13. **Field forge.** **The makeshift anvil stays for good** (the owner): a permanent cell for one ingot and an hour; no reverting terrain.
14. **Waste not's third rule** (a failed Temper spoils nothing) at level 6. Default taken: **yes**.
15. **Make do and mend never destroys** and leaves `attempts` as they were. Default taken: **yes**.
16. **Quick hands at 50 %** of every craft's time, Gather excepted. Default taken: **yes**.
17. **Master's hand** moves grades and cuts by 4, never pass or fail. Default taken: **yes**.
18. **Maker's mark** as written. Default taken: **as written**.
19. **The kit**: a knife replaces "rope | boots | an iron helm". Default taken: **yes**.
20. **The old three talents** are deleted. Default taken: **delete**.

## 14. Choices made along the way

| Choice | Taken | Rejected, and why |
|---|---|---|
| The Crafts category | a talent tree, `crafts`, the craftsperson's at creation only (`needs.class`) | a class field listing skills (then not a talent: the brief says talents); a `skill_choice` of seven (no choice for the player) |
| The trade's effect | a hand-out (`trade`: dice, tools, the known-for step) | a `skill` bonus (reads higher without being trained; every "trained at all" rule would ask the talent); a `skills` grant on the class (the class cannot know the trade) |
| The two free trades | `creation_bonus` on the class, the bonus-pick machinery at level 1 | `creation_talents = 2` more (any tree: not "from Crafts"); `bonus_talents.levels = [1, 1]` (one tree set for all levels); a new flow |
| The race rule | `Need::RaceLacks(skill)`, words derived from `races.toml` | a list of races on each talent (`race_not = ["dwarf", "duergar"]`: a second copy of the races' skills, wrong the day a race changes); refusing by class skills too (a ranger may take Wright; it only stacks) |
| The craftsperson's own profession skills | none; the trades, +6 | keeping Metalcraft and Woodcraft `2d6+10` (Smith stacks to 27, every other trade is the poor pick) |
| The tools | kit lines on the talent | nothing (a smith with no steel); a bigger kit on the class (which trade?) |
| Time served | the +6 read as a trade is handed down | a class skill (unknown trade); a flat +6 on every profession skill for life (a bonus that is no training) |
| Eye for the make | a describe passive in words | showing `attempts` (the difficulty's one secret); an identify roll (Enchanting's) |
| Waste not | three "nothing lost but time" rules | a 100 % salvage yield (a free round trip); a gem that never shatters (the lapidary's risk is the gem's) |
| Patron | a town helper and a one-at-a-time commission with the materials advanced, at 100 % | a shop that buys dear (no commission, no advance); a maker check on the hand-in (no maker state before 60; a found piece serves); quest XP and guild rank (farmable by design; the gold is the pay) |
| Make do and mend | field repair of the damage alone, safe | Temper anywhere (the anvil's point); improvements back (damage took them) |
| Trapwright | a rig list then a direction, rigs as data, kills credited | rigs as Woodcraft recipes (a recipe has no cell, and it is the class's, not the skill's); no attribution (a trap's kill earns nothing: the ability would be a damage tax); traps unknown to the rigger (one's own tripwire) |
| Field forge | a `makeshift_anvil` terrain with `works_as = "anvil"` | placing the `anvil` terrain itself (indistinguishable, indestructible); an anvil in the pack (a second notion of station); a terrain that reverts (saved state and a hook on leaving for a corner case; the owner: the makeshift anvil stays for good) |
| Quick hands | one `craft_seconds` every loop calls | a read per loop (eight); recipes only (Temper and Salvage are crafts too) |
| Master's hand | the margin graded 4 higher, the check untouched | +4 on the check (pass more; grades are the master's mark, not fewer failures); lower bands in `recipes.toml` (for everyone) |
| Maker's mark | `Worked::marked`, worth, a line, the patron's bonus, a capacity hook | the name ("of Brakka's make" on every piece: the names are long); a maker's *level* on the item (state for nothing) |
| The old talents | deleted | kept as ordinary Guard picks (they were the brief's "shite": nothing to keep) |
