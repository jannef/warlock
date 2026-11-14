# Warlock — Professions: crafting skills, salvage, and recipe menus

**Built; this file is history.** Current behaviour is in spec §8 "Salvage" and "Professions", and `docs/dev/crafting.md`.

Design document for the owner's review (2026-11-11; #1330, the P0 of the professions epic #1337). Written against `origin/main` at b566b9609 (version 0.16.0). Nothing here is implemented; §12 maps it onto the phase issues #1331–#1336 so each can be refined from the section it builds. It builds on what is there: objects, parts, instance parts, gems, and pieces (`docs/dev/objects.md`), Temper and Whittle (`world/crafting/temper.rs`, `magic/worked.rs`, `data/temper.toml`), Fletch (`world/crafting/fletch.rs`, `data/fletch.toml`), skills (`docs/dev/skills.md`, `data/skills.toml`), the one abilities menu (`ability.rs`), and the screen stack (`docs/dev/screens.md`). Every number below is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate). Numbers are a first pass; balance beyond it is out of scope.

## Contents

1. Summary
2. The owner's brief, and what exists today
3. Material stock: the piece is the stock
4. Materials, skills, and difficulty
5. Salvage
6. Recipes as data
7. The check and the result
8. The menus: profession rows, the profession screen, the wizard
9. The five new skills and their first uses
10. Fletching with a material cost: arrowheads and arrows
11. What the player sees; determinism; saves; the wiki and the docs
12. Implementation plan: the phases
13. Open questions for the owner
14. Choices made along the way

## 1. Summary

| Piece | Proposal |
|---|---|
| Stock | **No new stock objects**: the existing generic `piece` (`objects/piece.toml`, one object made of whatever grouped material it carries, as the loose gem is) *is* the stock. Its unit grows from 100 g to **500 g of steel** (scaled by the material's `weight`), and a new `data/stock.toml` gives each material group its form word (a steel **ingot**, a hardwood **billet**, a leather **hide**) and the profession that works it. A new item class `materials` groups it on every list. How much stock an item holds is **by weight**, the math `piece::break_up` already has. §3 |
| Materials → skills | `data/stock.toml`'s `[[group]]` rows: metal → Metalcraft (Strength), wood and bone → Woodcraft (Dexterity), gems, stone, and resin → Jewelcraft (Dexterity); hide, cloth, and glass have **no profession yet** (free in recipes, lost in salvage). Each material gets one new number, `work` (default 0, steel's; a modifier on every check that works it: tin −3, mithril +4, adamantium +8, diamond +6). §4 |
| Salvage | one ability for everyone (Fletch's precedent), `salvage:LETTER`, ten minutes beside nothing in particular. For each material of the item's resolved parts that a profession works, one check by that profession against `12 + work`; made, that material's stock at 50 % of its weight (a gem whole, its size and cut kept); failed, that material is lost. The item is destroyed either way; its magic and its improvements are lost (Enchanting's hook later). Refused, no time: worn or wielded, protected (quests, uniques, of Proof), unpaid, a spirit weapon, nothing a profession works. §5 |
| Recipes | `data/recipes/<skill>/<id>.toml`, one `[[recipe]]` a file: `result`, which of its parts are chosen from stock (the result object's `made_of` lists say from what), fixed `inputs` (arrowheads), `difficulty`, `seconds`, an optional `station` (a terrain beside the player: the anvil) and `knife`, `from` (the trained value it is listed from), `anyone` (Temper, Whittle, Fletch keep their universal access). **Known by skill value**, never learned or found. §6 |
| The check | Temper's shape: 2d12 + the profession's stat modifier + its `craft` points + the talents' stat checks against `difficulty + work` of the main chosen material; a natural 2 fails, a natural 24 succeeds. Made by 6 or more: the item comes out **Fine** (one `Worked` improvement, drawn as Temper draws it); by 12 or more, **Marvellous**; so quality is Temper's ladder, and the forge's attempt counts toward Temper's rising difficulty. Failed: nothing made, half the stock back (rounded as salvage rounds), gems back whole. A counted recipe (arrows, arrowheads) scales its count by Fletch's margin bands instead. §7 |
| The menus | the Skills group of the abilities menu loses Temper (the *Here* row), Whittle, and Fletch, and gains **one row per profession**: "Metalcraft…", "Woodcraft…", "Jewelcraft…" (trained, or anyone when a recipe of it is `anyone`). A row opens the **profession screen** (`ui/profession_screen.rs`): its recipes grouped by result class, each selectable or greyed with its reason. A recipe with choices opens the **wizard**: one paged list per choice (the stock stacks that fit; an optional gem with a None row), the running choice in the header, then the one dialog to confirm, which sends `craft:ID:POS,POS,…`. Every key fixed. Scribe stays a class ability. §8 |
| The five skills | Jewelcraft (`action`: salvage of gems and stone in P3, cutting and setting later), Mining (`rules`: dig faster, ore from a broken wall; marks on walls broken), Herbalism (`passive`: a second find on a forage; marks on forages), Alchemy (`action`: tells unknown potions carried by the hour, Appraise's loop; no marks until brewing), Enchanting (`action`: tells unknown magic items the same way; no marks until its craft). Who trains them: a table. §9 |
| Fletching | P6: **arrowheads** (a Metalcraft recipe at an anvil, 20 a batch from one ingot, the head's metal theirs) and **arrows** (a Woodcraft recipe: arrowheads plus wood; the result's parts the head's metal and the shaft's wood, so steel-tipped ironwood arrows are their own goods). Without heads a batch makes brittle arrows (a sharpened shaft), with heads plain ones: `plain_from` goes, and Fletcher's "never brittle" with it. On wooded ground the wood is free ("from what wood there is"); elsewhere a billet. §10 |
| Phases | P1 the skills (data), P2 the stock, P3 Salvage, P4 the profession rows, screen, and recipe loader with Temper, Whittle, and Fletch as its first entries, P5 weaponsmithing and the wizard, P6 arrowheads and arrows. Mining, Herbalism, Alchemy, Enchanting, armour, jewellery, leatherworking later, each a section's one line. §12 |

## 2. The owner's brief, and what exists today

The brief (#1330): five new skills that do little yet (Jewelcraft, Mining, Herbalism, Alchemy, Enchanting); **salvaging** (an item broken into its materials, a check per material on its skill, "a ruby-tipped hardwood quarterstaff gives a ruby and hardwood if both checks succeed", in the abilities menu); the craft skills **leave the abilities menu** for a **profession menu** per craft listing its recipes, some with materials, some without (Temper); the proof of concept **weaponsmithing** ("Scimitar, Dagger, Long sword", then a wizard: the blade's metal from the metals carried, then extras such as a gem for the pommel); **intermediate materials** (arrowheads from the smith, arrows from arrowheads and wood by Woodcraft; fletching later gets a material cost).

| Part | Today |
|---|---|
| Parts and instance parts | an object is made of parts, each of one material with a share of the weight (`content/part.rs`, `data/parts.toml`); a stack may carry its own parts by id (`item_parts.rs`, `Stack::with_parts`, `Stack::parts`: the base parts with its own in their place; `MAX_OBJECT_PARTS` 8); names, colour, weight, worth, to-hit, damage, and Protection already follow the materials (`variant.rs`, `ObjectDef::value_with`); construction lists `made_of` per part, resolved into `ObjectDef::construction`, read by `ObjectDef::made_of(part)` (long sword: `["metal", "dragonbone"]`; dagger `["metal", "bone", "stone"]`; quarterstaff `["wood", "dragonbone"]`) |
| Raw material | **`piece`** (`objects/piece.toml`, `piece.rs`; the animated-objects design §9.2, the owner: "you do not need to invent scrap objects"): one object, class `miscellaneous`, 100 g of steel, worth 1, `made_of = ["any"]`, named "piece of tin", stacking by material; `piece::make(material, count)`; `piece::break_up(stack, percent)`: for each resolved part a gem gives its loose gem (size and cut kept), a grouped material its grams (the base part's share scaled by material weight, `grams`) summed per material, then `max(1, round(grams × percent / 100 / one piece's grams))` pieces; no magic survives. Used only by animated things' broken components |
| Gems | `data/gems.toml`: sizes and cuts, the plain ones, `[[ornament]]` per item class (melee weapons a pommel, missile weapons and shields an inlay, helmets studs, rings and amulets a setting), objects may set their own (`ornament = "tip"`: the staves); `gem.rs::ornament` draws loot's gems, never a rough cut. The loose gem's object is `gem`; a stack with one is `Stack::gem` |
| Materials | `data/materials/*.toml`, 56 files: `group` (metal 20, wood 3, bone 5, hide 4, cloth 3, stone 4, glass 1, resin 1; none for the 10 gems, paper, wax, plant, flesh, lead), `weight`, `to_hit`, `damage`, `protection`, `value`, `rarity`, `gear`, `fit`, `resists`, `gem`. No difficulty of any kind. "Properties not yet modelled: hardness, burning, and wear" (`README.toml`) |
| Temper and Whittle | `world/crafting/temper.rs`: a `Craft` (Temper: metal, Strength, `Effect::Temper`, `Use::Temper`; Whittle: wood, Dexterity, `Effect::Whittle`, `Use::Whittle`, a knife), `data/temper.toml` (`seconds` 600, `difficulty` 13, `per_attempt` 2, `most` 3), ten minutes by `keep_at` (interrupted: nothing, time spent), one check, `Worked` (`magic/worked.rs`: improvements Fine, Marvellous, Masterwork in an `Aspect` the item has; `attempts` for life; `damaged`), failure is `ItemHarm::Work` through `harm_item`, the item identified after. Temper is `Ability::Temper` in the *Here* group (`Offer::Temper`, the anvil's terrain; `input::offer_dialog` asks on walking into it), Whittle `Ability::Whittle` in the Skills group, both had by everyone, both `Aim::Item` (`ObjectList::Temper`/`Whittle`, `workable`/`workable_wood`). An anvil stands in generated forge rooms (weight 3) and in Cēaptūn's smithy |
| Fletch | `world/crafting/fletch.rs`, `data/fletch.toml`: anyone, anywhere, a knife carried and a launcher ready, ten minutes, no material; a Dexterity check + Woodcraft's `fletch` points against 12, `count` `1d4` + value ÷ 25, the margin's bands (`half_within` 3, `one_within` 6), `plain_from` 25 (below it brittle arrows, `breaks = 100`); the ranger's Fletcher (`TalentMods::fletcher`: +4, ×2, half the time, never brittle). `world::fletch_lack` is the one place that says what is missing (`Unavailable::NoKnife`, `NoLauncher`) |
| Skills | `data/skills.toml`: Metalcraft (`action`; `temper` benefit `[[0,0],[100,8],[200,12]]`; dwarf, duergar `1d6+4`; craftsperson known for; warrior 15 %, commoner 10 %; two background lines), Woodcraft (`action`; `fletch`, `whittle`, `wedge`; ranger and craftsperson known for; druid 35 %, commoner 20 %; two lines). `kind = "action"` skills without a use are `Ability::Skill` rows, "not usable yet" (`Unavailable::NotYet`); a use replaces the row (`Ability::use_of`). `Actor::skill_effect(content, Effect)` sums every skill that gives the effect (affixes and talents counted); there is no read of **one skill's** benefit |
| The abilities menu | `ability.rs`, the one registry: `Ability` (variant, `every`, `had_by`, `unavailable`, `aim`, `cost`, `action`), `Source` groups (Racial, Birth sign, Class, Spells, Talents, Skills, Weapon skills, Items, Here, Allies), `Aim` (Nothing, Direction, Cell, Corpse, Ailment, Item, Shape, Spell), `Unavailable` with its words. `ui/abilities_screen.rs` lists them; an `Aim::Item` opens an `ObjectList`; `Aim::Spell` opens `ui/scribe_screen.rs` (a `PagedList` grouped by school, fixed keys), the model for a list a row opens |
| Dialogs | one `Dialog` (`ui/dialog_screen.rs`): title, text, accept/decline/cancel options, used for every confirmation (`docs/dev/screens.md`) |
| Shops | `data/shops.toml` by item class: weaponsmith, armourer, Cēaptūn's smithy, fletcher, alchemist, general store (buys gems); no class for materials |
| Digging | `world/crafting/dig.rs`: `dig_work` per turn, the talents' `dig_percent` (Breaker, Stonewise, Stone sense +25 %) and a Delving affix; `break_wall` pushes `Event::WallBroken { pos, cause: Breach::Dug }`; nothing comes of a broken wall |
| Foraging | `world/skills/forage.rs`, `data/forage.toml`: `[[ground]]` per landscape with `chance`, `count`, weighted `finds` (food: berries, mushrooms, hazelnuts, apples, angelica root); `forage_ground` is the one place that says where (overworld and local areas by landscape, never water); Survival's `forage` percent |
| Appraise | `world/skills/appraise.rs`: each game hour a chance (`Effect::Identify`, trained only) to tell one unidentified carried thing, from a stateless stream per hour; `World::learn_item` |

## 3. Material stock: the piece is the stock

**Decision: one object, by weight.** The stock salvage produces and recipes consume is the existing `piece`, carrying its material as its main instance part, exactly as `piece::make` builds it and `piece::break_up` yields it. No ingot, billet, or hide object is added. Reasons: the object exists, with names, colour, weight, worth, stacking by material (`Stack::same_goods`), saves, placements (`"piece:tin*3"` works today), and the breaking math already written and tested; a form per group would be seven objects that differ only in their noun; and the owner decided this once already (animated objects §16 D4).

What changes on it:

| Change | Where | Why |
|---|---|---|
| **The unit: 500 g of steel** (`weight = 500` in `piece.toml`; a piece of another material weighs 500 × its `weight` %: wood 200 g, hardwood 250 g, leather 175 g, gold 1230 g) | `objects/piece.toml`, `piece.rs` tests | 100 g makes a long sword cost 16 pieces and a quarterstaff 50; 500 g gives the counts in the table below. A kilogram makes a dagger (450 g) cost a whole ingot and return one: no loss on a round trip. Open question 1 |
| **The form word per group**: `data/stock.toml`, `[[group]]` rows: `metal`: ingot/ingots, `wood`: billet/billets, `hide`: hide/hides, `cloth`: bolt/bolts, `bone`: shard/shards, `stone`: block/blocks, `glass`: lump/lumps, `resin`: lump/lumps. Names "a steel ingot", "3 hardwood billets", "a leather hide", "a bolt of wool" (`of = true` on a row puts the material after: "a lump of amber", "a block of jade"). A group without a row keeps "piece of X" | `piece::names` reads the row; `content/stock.rs` loads the file (optional: without it every piece is "a piece of X") | The stock is seen on every list; "16 pieces of steel" reads as rubble, "3 steel ingots" as stock. The same file carries the group's profession (§4) |
| **Item class `materials`** ("Materials"), placed before `gems` in `item_classes.toml` | `piece.toml` `class = "materials"`; a new class row; the arrowhead (§10) takes it too | So lists, the `inventory` dump, and shops group it (`ItemClassId` is an index, not saved: no save change) |
| **Value**: unchanged, `value = 1` × the material's `value` % per piece (steel 1 coin, iron 1, gold 20, mithril as its data) | — | The smith's sword is worth far more than its ingots (`long_sword` 80 vs 3 coins of steel): crafting adds worth, as it should. Note the `value` of 1 coin for 500 g of steel is cheap; a first pass, tuned with the recipes in P5 |
| **Shops**: the smithy, the weaponsmith, the armourer, and the fletcher `buys` `materials`; the smithy and the general store `stock` it (a few ingots and billets among their wares, drawn by `frequency` as any stock is: `piece` gets `frequency = 40` and never a `kind`) | `shops.toml` | A smith without a source of steel but salvage has nothing to forge; a town sells some |

**How much stock is in an item** (the brief's question): **by weight**, one rule for salvage and recipes. `piece::count_for(grams, material)` = `max(1, round(grams / one piece of the material))` is the rounding `break_up` already uses; salvage applies it to the part's grams × its yield percent, a recipe to the part's grams whole. A fixed count per part (the alternative) would make the two disagree (salvage returns N, the recipe asks M) and need a number per object. Per-share alone is the same thing without the grams.

| Item (main part) | Grams of it | Recipe costs | Salvage returns (50 %) |
|---|---|---|---|
| knife (200 g steel) | 200 | 1 ingot | 1 ingot |
| dagger (450 g) | 450 | 1 | 1 |
| short sword, rapier (1100 g) | 1100 | 2 | 1 |
| scimitar (1500 g) | 1500 | 3 | 2 |
| long sword (1800 g, blade 90 %) | 1620 | 3 | 2 |
| great sword (3500 g) | 3500 | 7 | 4 |
| arrow shaft (85 % of 30 g; a billet of wood is 200 g, wood weighing 40 % of steel) | 25.5 | 1 billet per batch up to 11 | — |
| quarterstaff (2000 g wood) | 2000 | 10 billets | 5 |
| the owner's hardwood quarterstaff (a variant: hardwood 50 %, 250 g a billet) | 2000 | 8 | 4 |
| short bow (700 g), long bow (1300 g) | | 4, 7 | 2, 3 |

A round trip (forge, then salvage) loses about half: a dagger alone breaks even, which is no exploit (two checks and forty minutes for nothing). The leather grip of a long sword (180 g) is a part of a group with no profession (§4): it costs nothing and returns nothing ("the smith has a scrap of leather for the grip").

## 4. Materials, skills, and difficulty

**Decision: the group names the profession, in data.** `data/stock.toml`'s `[[group]]` rows (the ones that name the form word, §3) also name the skill that works the group and the stat its checks are made by:

| Group | Profession | Stat | Materials | Why |
|---|---|---|---|---|
| `metal` | Metalcraft | Strength | the 20 metals (lead has no group: never stock) | Temper's pair today |
| `wood` | Woodcraft | Dexterity | wood, hardwood, ironwood | Whittle's pair |
| `bone` | Woodcraft | Dexterity | bone, horn, ivory, chitin, dragonbone | carving is Woodcraft's word in `skills.toml`; bone, horn, and ivory are carved, not forged |
| gems (the `gem` table, no group) | Jewelcraft | Dexterity | the ten gems | the brief's |
| `stone` | Jewelcraft | Dexterity | stone, flint, obsidian, jade | the lapidary's: jade is cut and set, flint knapped; a stone block is Jewelcraft's stock |
| `resin` | Jewelcraft | Dexterity | amber | precious, set in amulets (`amber_amulet`) |
| `hide` | **none yet** | — | leather, hardened leather, fur, dragonscale | leatherworking is the epic's "later"; until then a hide part costs nothing in a recipe and gives nothing in salvage, and no hide stock exists |
| `cloth` | none yet | — | cloth, wool, silk | tailoring, later |
| `glass` | none yet | — | glass | nothing is made of it but flasks |

A row is `{ group = "metal", skill = "metalcraft", stat = "strength", name = "ingot", plural = "ingots" }`; `gems = { skill = "jewelcraft", stat = "dexterity" }` at the top level (gems have no group). The loader checks the skill ids and that no skill is named by two stats. Nothing in code names a group or a skill: `World::profession_of(material) -> Option<(SkillId, Stat)>` reads the file, and §5 and §7 read that.

**The difficulty per material: one number, `work`** (a new optional field on `MaterialDef`, default 0, a signed modifier on the check of every salvage and recipe that works the material; steel's reference). One number rather than the brief's two (`salvage` and `craft`) because the two would move together on every material (what is hard to forge is hard to break down well), and the recipe carries its own base difficulty (§6), so a material's one modifier lands on both. A first pass:

| `work` | Materials |
|---|---|
| −3 | tin, pewter, zinc, cadmium, bendalloy (soft metals) |
| −2 | copper, gold |
| −1 | iron, bronze, brass, aluminium, silver, electrum, wood, bone, horn |
| 0 | steel, duralumin, hardwood, ivory, chitin, stone, flint, smokestone |
| +1 | obsidian, amber, garnet, zircon, amethyst |
| +2 | chromium, jade, topaz, heliodor |
| +3 | nicrosil, ironwood, emerald |
| +4 | mithril, ruby, sapphire |
| +5 | dragonbone |
| +6 | diamond |
| +8 | adamantium |

**The profession's points: one effect, read per skill.** Each profession declares `benefit = [{ effect = "craft", curve = [[0, 0], [100, 8], [200, 12]] }]` (Temper's curve), and the checks read it through a new `Actor::skill_effect_of(content, skill, Effect::Craft)`: the one named skill's curve at its value, plus what affixes and talents add to **that** skill (Skill focus through `Skills::effect_with`, as `skill_effect` counts them), never summed across skills. Why a new read: `skill_effect` sums every skill that gives the effect, so Metalcraft's and Woodcraft's `craft` would add up on one check. `Effect::Temper`, `Whittle`, and `Fletch` stay as they are until P4 turns their uses into recipes, when each becomes that profession's `craft` (one curve each today: nothing changes in value).

## 5. Salvage

**Decision: one ability, for everyone, cross-profession.** `Ability::Salvage` in the Skills group, had by everyone (Fletch's and Whittle's precedent: "anyone can, after a fashion"; untrained, a check is the stat modifier alone against `12 + work`), `Aim::Item` on `ObjectList::Salvage` (`world::salvageable`: the stacks with at least one part a profession works, less the refused below). `Action::Salvage { item }`, replay text `salvage:LETTER`, `ActionKind::Salvage` (`salvage = 1000` in `actions.toml`), dispatched by one arm of `player_acts` to `World::start_salvage` in `world/crafting/salvage.rs`. Data `data/salvage.toml`: `seconds = 600`, `difficulty = 12`, `yield = 50`.

| Rule | Decision | Why |
|---|---|---|
| Where, with what | anywhere, no tool, no station | time is the penalty; a knife or an anvil would add two more "lacks" to explain for little; open question 8 |
| Time | ten minutes by `keep_at`, as Temper: interrupted, nothing happens and the time is spent | the one loop every long action uses |
| A stack of several | one at a time, set apart as Temper sets it (`set_apart`) | a stack of arrows salvaged at once would be a loop of checks; one is enough |
| The checks | for each **distinct material** among `Stack::parts(content)` (resolved: the base parts with the item's own in their place, gems included), in first-seen order, that `profession_of` maps: 2d12 + the profession's stat modifier + its `craft` points + the talents' checks of that stat (`TalentMods::check`), against `difficulty + work`; a natural 2 fails, a natural 24 succeeds (`check::makes`). Two gameplay draws a check | Temper's check, once per material; the brief's "each material present gets a roll on the matching skill" |
| Made | that material's stock: its grams × `yield` through `piece::count_for` (so `break_up` with `yield` as its percent, filtered to the materials made); a gem part gives its loose gem with its size and cut (an ornament's cut, a base item's the plain one) whole, never a percent | "a ruby and hardwood" |
| Failed | that material is lost: "The blade cracks and is good for nothing." / "The ruby shatters." | the simple reading of the brief; open question 4 asks whether a failed gem should lose a cut instead |
| The item | destroyed either way (`remove_stacks`); its affixes, its `Worked` improvements, and its rung or charges are gone; an unidentified item is never named ("You never learn what it was.") | §2: no magic survives `break_up`; Enchanting's extraction is a later hook on this very spot (§9) |
| A damaged item | salvaged as any other | its damage is in its state, not its metal |
| A material no profession works | gives nothing, says nothing (a leather grip) | §4 |
| Marks | `Use::Salvage` on each profession rolled, 100 %, whatever came of it (the training stream) | every check is practice |
| Refused, no time | worn or wielded ("Take it off first."; the list leaves them out), protected (`World::item_protected`: quests, uniques, of Proof; "It will not come apart."), unpaid shop goods (`may_loose`'s rule: "You have not paid for it."), a spirit weapon, a stack with nothing a profession works ("There is nothing in it worth saving.") | safety against salvaging the sword in one's hand; the protections every destroyer honours |
| Alignment, noise | none; a hammering's noise? **none**: salvage is quiet work (no anvil) | nothing to add |

The owner's example: a hardwood quarterstaff with a ruby tip. Parts: hardwood (wood → Woodcraft, Dexterity, `work` 0) and ruby (gem → Jewelcraft, Dexterity, +4). Two checks: Woodcraft's against 12, Jewelcraft's against 16. Both made: 4 hardwood billets (2000 × 50 % / 250) and the ruby, its size and cut. Wood made, ruby failed: 4 billets, the ruby shattered. Neither: nothing but ten minutes gone. Untrained in Jewelcraft (no skill row), the ruby's check is still made, at Dexterity alone.

## 6. Recipes as data

**Decision: `data/recipes/<skill>/<id>.toml`, one `[[recipe]]` a file, known by skill value.** The directory is the profession (`recipes/metalcraft/long_sword.toml`); a profession is a craft skill with a directory (so Jewelcraft is a profession the day its first recipe file lands). Read in path order into `Content::recipes` (`content/recipe.rs`, `RecipeDef`, `RecipeId` an index: not saved, actions name recipes by id).

| Field | Meaning | Checked at load |
|---|---|---|
| `id` | the file name; `craft:ID` names it | unique across professions |
| `name` | the row's name (default: the result object's name, or the action's for a `use` recipe) | |
| `result = { object = "long_sword", count = 1 }` | what is made; `count` dice for a stackable result (arrowheads `20`, arrows: Fletch's count rule, §10) | the object exists; `count` above 1 only on a stackable object |
| `[recipe.parts]` | which of the result's parts the maker chooses: `blade = "stock"` (a stack of stock whose material is on that part's `made_of` list and in the profession's groups), `head = { input = "arrowhead" }` (the part takes the material of the named input). A part not listed is the base item's material, **free** | every part named exists on the result; a `stock` part's list has a material of the profession's groups; an `input` part names an input |
| `inputs = [{ object = "arrowhead", count = 20 }]` | fixed objects used up (a stack of that object, any material; `count` per batch or, with `per_made = true`, per item made) | the object exists |
| `ornament = true` | the wizard offers **an optional gem** for the result's ornament part (`ObjectDef::ornament`, else the class's `[[ornament]]`; a cut gem, never rough), set on success, never lost on failure | the result has an ornament part |
| `difficulty = 13` | the check's base (§7) | |
| `seconds = 3600` | the work's time (`keep_at`) | ≥ 1 |
| `station = "anvil"` | a terrain id that must stand beside the player (`Dir::ALL`, as `anvil_beside`) | the terrain exists |
| `knife = true` | a knife carried (`Content::is_knife`), at the start and at the end | |
| `launcher = true` | a bow or crossbow ready, the batch for what it `launches` (Fletch's rule) | |
| `from = 25` | the trained value the recipe is listed from (default 0: anyone with the skill) | |
| `anyone = true` | listed for the untrained too (Temper, Whittle, Fletch) | |
| `use = "temper"` | **a built-in recipe**: the row runs an existing action in place of the generic craft (`temper`, `whittle`, `fletch`), its own list and rules unchanged | one of the three |

Why by skill value and not learned or found: a learned recipe is state (a list on the actor, saved), a source (scrolls? teachers?), and a reason the smith cannot make a dagger; none of it is asked for, and the skill value already grows by the level-up rolls, so "listed from Metalcraft 25" is a progression with no new machinery. Found or taught recipes stay a deferred idea (#196), not a field.

The weaponsmith's recipes (P5), each a file under `recipes/metalcraft/`, the result object's own weight pricing the stock:

| Recipe | Result | Chosen | `difficulty` | `seconds` | `from` | Ingots (steel) |
|---|---|---|---|---|---|---|
| knife | knife | main `stock`; `ornament` no | 10 | 1200 | 0 | 1 |
| dagger | dagger | main `stock`; ornament (pommel) | 11 | 1800 | 0 | 1 |
| short sword | short_sword | main; ornament | 12 | 3600 | 10 | 2 |
| scimitar | scimitar | main; ornament | 13 | 3600 | 20 | 3 |
| long sword | long_sword | `blade = "stock"`; ornament; the grip free | 13 | 3600 | 20 | 3 |
| rapier | rapier | main; ornament | 14 | 3600 | 30 | 2 |
| great sword | great_sword | main; ornament | 15 | 5400 | 40 | 7 |
| temper | — | `use = "temper"`, `anyone`, `station = "anvil"` | — | — | — | — |

All at `station = "anvil"`. The axes, maces, and polearms (iron, `made_of` metal) are the same shape and follow in the same phase if time allows, else as data later; `javelin` and `spear` (a wooden haft) wait for the input rule to take stock as well as objects (a haft from a billet: `haft = "stock"` of Woodcraft's group, a second profession's material in one recipe; the check stays the recipe's profession's). Armour is a later phase (§12).

## 7. The check and the result

**Decision: Temper's check, Temper's grades.** One check when the time is up (as `tempered`): 2d12 + the profession's stat modifier + `skill_effect_of(skill, Craft)` + `TalentMods::check(stat)`, against `difficulty + work` of the **main chosen part's** material (the blade's; a gem's `work` never counts: it is set, not forged); a natural 2 fails, a natural 24 succeeds. Two gameplay draws, then one for the `Aspect` if the margin earns one.

| Margin | Outcome | The item |
|---|---|---|
| ≥ 12 | **Marvellous** | two improvements (`Worked::improve` twice, each `Aspect` drawn among the item's as Temper draws), `attempts` 2 |
| 6 to 11 | **Fine** | one improvement, `attempts` 1 |
| 0 to 5 | made | plain, `attempts` 0 (a `Worked` with nothing on it is no state: the stack carries none) |
| below 0 | **failed** | nothing made; the stock spent comes back at `salvage.toml`'s `yield` through `piece::count_for` (so at least one piece of each material, as salvage), gems whole (set last, never spent), object inputs at the same percent (half the arrowheads); "The blade comes out brittle and you break it up for the metal." |

Why tie quality to `Worked`: the owner's hint, and it is the one quality state items have (names, worth ×2/×3/×5, `Mods::local`); a forge-made Fine sword is exactly a tempered one, so Temper's later attempts on it count the forge's (`per_attempt`) and the ladder stays one. Why not Masterwork at the forge: three improvements is the anvil's long work, not a lucky roll. Why half the stock back on a failure rather than all or none: "all lost" makes the first smith's failures a gold sink that stops them trying (against the no-anti-farming rule's spirit: time should be the penalty), "nothing lost" makes the check cosmetic; half is the same loss a salvage gives. Open question 5.

The made item is **identified** (the maker knows their work: `World::identify` on it, Temper's rule), its parts the chosen materials (`Stack::with_parts`, `item_parts::check`), so "a ruby-pommelled steel long sword" names, weighs, strikes, and sells by the system that exists. It goes into the pack (`add_to_inventory`); over the carrying limit as anything picked up. The stock is spent at the **end** (as Scribe spends its mana), so an interrupted work costs only time; the pack is re-read then (the stacks may have been dropped: "You have nothing left to work with."). Marks: `Use::Craft` on the profession, 100 % per work that ran its time, whatever came of it (Temper marks no stat today, and neither does this).

**Counted recipes** (`count` above 1: arrowheads, arrows) have no grades: the margin scales the count by Fletch's bands (`half_within`, `one_within` move from `fletch.toml` to `data/recipes.toml`, the recipes' settings file, with `yield` of a failure), the inputs `per_made` are used for what was made, and the stock as §10 prices it.

## 8. The menus: profession rows, the profession screen, the wizard

**Decision: one row per profession in the Skills group; one screen; the wizard is paged lists over it; the confirmation is the one dialog; every key fixed.**

| Piece | Design |
|---|---|
| The rows | `Ability::Profession(SkillId)`, `Source::Skill`, named "Metalcraft…" (the skill's name and an ellipsis: it opens a list), described "Your recipes as a smith." `every` lists one per skill with a recipe directory, in skill order, in the place of `Ability::Skill` for that skill (so `use_of` returns it). `had_by`: trained in the skill, **or** any recipe of it is `anyone` (so everyone sees "Metalcraft…" and "Woodcraft…" while Temper, Whittle, and Fletch are universal; the screen then lists only what they may use). Never unavailable: the screen explains each row. `Aim::Recipes`: the abilities screen opens `ScreenName::Profession(skill)`. The rows **Temper** (Here), **Whittle**, **Fletch**, and the "not usable yet" **Metalcraft**/**Woodcraft**/**Jewelcraft** rows go (`Offer::Temper` stays for the anvil's walk-in dialog, which opens the Metalcraft screen on Temper's row, and for the station check). **Salvage** stays a row of its own (§5). **Scribe** stays a Class active: it is the wizard's, not a profession's, and spells are no stock; a scribing profession is not designed |
| The profession screen | `ui/profession_screen.rs`, `ScreenName::Profession(SkillId)`, a `PagedList` (the scribe list's shape) grouped by the result's item class ("Melee weapons", "Missiles", "Materials"; built-in `use` rows under "Work"). A row: the recipe's name; a right column with what it takes, in words the player already sees elsewhere: "3 steel ingots, an anvil, about an hour" (stock in the player's own units, time in `Duration::spoken`); never the difficulty or the odds. **Greyed with its reason**, the first that applies: "beyond your skill" (`from`), "no anvil beside you", "no knife", "no bow or crossbow ready", "no metal to hand" (no stack of stock fits a `stock` part), "no arrowheads" (an input missing). A letter picks a row; Tab and the arrows page; Esc closes. `PerceptionView::recipes(skill) -> Vec<RecipeRow { id, name, takes, unavailable }>` is what it reads (and what the `recipes` dump layer prints, §11), computed in core from the pack, the cells beside the player, and the skills: presentation never decides availability |
| A `use` row | runs as today: Temper's `ObjectList::Temper` over the screen (`workable`), Whittle's, Fletch's `Action::Fletch` at once; their actions and refusals unchanged |
| The wizard | `ScreenName::CraftPick { recipe, step }`: one paged list per **choice** of the recipe in this order: each `stock` part (the stacks of stock in the pack whose material is on the part's list and in the profession's groups, as the inventory shows them, with the count the recipe would take: "steel ingots (3 of 7)"; a stack too small is greyed "too few"), each object `input` (the stacks of that object), then the `ornament` (the cut loose gems carried, plus a first row **None**, letter `-`; the gem's part named: "for the pommel"). The header carries the recipe's name and the running choice ("Long sword — blade: steel, pommel: —"). A letter picks and opens the next step; Esc goes back one step (the stack pops to the step before, its pick kept, `ScreenStack::picks`' pattern); a recipe with no choice skips to the dialog. A step with one fitting stack is **not** skipped (the player sees what is spent) |
| The confirmation | the one dialog: `Dialog::yes_no("Forge", "Forge a ruby-pommelled steel long sword? It uses 3 steel ingots and the ruby, at the anvil, and takes about an hour.", "Forge", Action::Craft { .. }, "Not now")`; the verb from the profession (`data/stock.toml` `verb = "Forge"`, Woodcraft "Make", Jewelcraft "Cut"); the result's name as `Stack::names` would write the made item. Declined: back to the last step |
| The action | `Action::Craft { recipe: String, picks: Vec<Option<usize>> }`, replay text `craft:ID[:POS,POS,…]`, the picks in the wizard's step order as inventory positions (`position_text`), `-` a skipped optional: `craft:long_sword:c,f`, `craft:arrows:b,d`, `craft:dagger:a,-`. The rules check every pick again (the stack exists, fits, suffices) and refuse with no time; the dev loop needs no screen |
| Keys | all fixed: letters and `-`, the list's page keys, Enter on the `>` row, Esc; no `KeyCommand`, no `Context` entry (CLAUDE.md's hard rule). `--screen profession` (over a run, the first profession's) and `--screen craft-pick` for captures; text snapshots in `screen_snapshots.rs` |

Why one screen and paged lists rather than a form: the stack already does "a list over a list" (Read → Identify), the lists know letters, paging, and greying, and each step is a plain choice from the pack; a form with fields would be a new input mode. Why a row per profession rather than one "Craft…" row: the brief's ("a profession menu per craft"), and a smith with no wood sees no empty Woodcraft group.

## 9. The five new skills and their first uses

All five are one `[[skill]]` each in `data/skills.toml` (P1), alphabetically placed, and reach the Skills screen, `dump --layers skills`, and the wiki's table at once. Marks only where the use lands (P1 for Mining and Herbalism: their hooks exist; the others when their phase comes).

| Skill | `kind` | Description (the one line) | First use (where in code) | Marks | Who trains it (first pass) |
|---|---|---|---|---|---|
| **Jewelcraft** | `action` | "Cutting and setting gems, and the working of stone and amber." | **P3**: the profession of gems, stone, and resin in salvage (§4); its row "Jewelcraft…" once a recipe exists. Later recipes: *Cut* (a loose gem one cut finer: uncut → crude → fair → …, `difficulty` by the cut, a failure a size smaller), *Set* (a gem into a ring or amulet's setting: the one ornament recipe that takes an existing item as input) | `salvage`, `craft` 100 % | gnome `1d6+4` (the race with Appraise); craftsperson optional 30 % `2d4`; merchant optional 20 %; dwarf optional 20 %; a background line (a jeweller's child) |
| **Mining** | `rules` | "Finding ore in the rock and breaking it out: digging comes easier." | **P1**: `Effect::Dig`, percent more work a dig turn (curve `[[0, 0], [100, 25], [200, 40]]`), read in `dig_turn` beside `TalentMods::dig_percent` (one hook, the sum capped as speeds are). **Later**: ore from a broken wall: on `Breach::Dug` by the player, trained at all, a chance (`data/mining.toml`: by the wall's `hardness` band, plus Mining's `find` percent) of **ore**, a second generic object (`objects/ore.toml`, `made_of = ["metal"]`, named by its metal: "a lump of iron ore"; the metal by a weighted table per branch depth, iron and copper common, mithril very rare and deep), smelted at an anvil by Metalcraft's *Smelt* recipe (input ore, `main = { input = "ore" }`: the ingot of the ore's metal) | `dug` 100 % (a wall the player breaks through) | dwarf and duergar `1d6+4` beside their Metalcraft; kobold `1d4+2` (they dig); craftsperson optional 25 %; commoner optional 10 %; a background line (a miner's child) |
| **Herbalism** | `passive` | "Knowing the plants: what heals, what harms, and where it grows." | **P1**: on a forage that finds something, a herbalist draws **one more find** from the ground's `herbs` list (`forage.toml` `[[ground]]` gains `herbs = [...]`: angelica root everywhere it grows, the rest as Alchemy's reagents come; `Effect::Herbs` percent is the chance, curve `[[0, 25], [100, 60], [200, 80]]`; one more gameplay draw after the find's, so a non-herbalist's forage draws as before) | `forage` 25 % | druid `2d6+10` (known for? no: Survival and Handle animal stay; Herbalism `2d4+4`); witch `2d4+4`; ranger optional 30 %; halfling `1d4+2`; two background lines (a wise-woman's child, the youth's gathering) |
| **Alchemy** | `action` | "Brewing potions, and telling one draught from another." | **P1** (small): **tells unknown potions and flasks** carried by the hour: `Effect::IdentifyPotion`, read in `appraise_hours` beside Appraise's `Identify` (the same stateless stream, the candidates filtered to the `potions` class; "You recognise the murky draught: a potion of healing."). **Later**: brewing recipes (`recipes/alchemy/`: a potion from herbs and a flask, no station, Intelligence), and the reagents | none until brewing (`craft`) | witch `2d6+10` (her known-for list unchanged); wizard optional 25 %; druid optional 25 %; a background line (an apothecary's child) |
| **Enchanting** | `action` | "The binding of magic into things, and the reading of what is bound." | **P1** (small): **tells unknown magic items** carried by the hour, `Effect::IdentifyMagic` in the same loop, filtered to stacks with magic. **Later**: Enchanting's craft on salvage's spot (§5): an affix extracted before the item breaks, and bound onto a made item; a design of its own | none until its craft | wizard optional 30 % `2d4+4`; witch optional 20 %; aristocrat optional 15 %; a background line (a scholar's child) |

Stats of their checks (`stock.toml`): Jewelcraft Dexterity; Mining has no check; Alchemy Intelligence; Enchanting Willpower; Herbalism none (a percent). The class and race tables are a first pass for the owner to strike through (open question 9); background lines follow `docs/dev/background.md` and are the P1 implementer's.

## 10. Fletching with a material cost: arrowheads and arrows

**Decision: arrows are a recipe with two inputs; a batch without heads is brittle; wood is free where trees are.**

| Piece | Design |
|---|---|
| **Arrowhead** | `objects/arrowhead.toml`: class `materials`, stackable, `materials = ["iron"]`, `made_of = ["metal", "bone", "stone"]`, 5 g, value 0 (its worth is the metal's by `value_with`), glyph `,`; a stack of another metal carries it as its main part ("20 steel arrowheads"); `bolt_head` the same for bolts (or one `arrowhead` for both: **one**, "a head is a head", the recipe decides what it tips) |
| **Arrowheads** (Metalcraft, `recipes/metalcraft/arrowheads.toml`) | `result = { object = "arrowhead", count = 20 }`, `parts.main = "stock"` (one ingot of any metal: the count is a batch, not the ingot's 500 g, which would make a hundred; the rest is slag), `station = "anvil"`, `difficulty = 10`, `seconds = 600`, `from = 0`; the margin scales the 20 by the bands |
| **Arrows** (Woodcraft, `recipes/woodcraft/arrows.toml`; **bolts** the twin) | `result = { object = "arrow" }` with Fletch's count (`1d4` + Woodcraft ÷ 25, Fletcher ×2: the recipe says `count = "fletch"`, the one special value, read from `fletch.toml`), `parts.head = { input = "arrowhead" }`, `parts.shaft = "stock"` (a billet of any wood), `inputs = [{ object = "arrowhead", count = 1, per_made = true }]`, `knife = true`, `launcher = true` (the batch is arrows or bolts by what the launcher `launches`, Fletch's `[[makes]]`), `difficulty = 12`, `seconds = 600`, `anyone`. One billet covers a batch (25.5 g a shaft against a 200 g billet of wood: up to eleven arrows round to one billet, `count_for`); the heads used are the arrows made. The result's parts are the head's metal and the shaft's wood: "steel-tipped ironwood arrows", their own goods |
| **Brittle arrows** | the same recipe **without arrowheads** (the input step offers **None** when the recipe says `inputs_optional = true`): the result is the `[[makes]]` row's `brittle` object (`brittle_arrow`, a sharpened shaft, `breaks = 100`) with the shaft's wood as its part. So `plain_from` goes: plain arrows need heads, whatever the skill, and **Fletcher's "never brittle" goes with it** (its +4, ×2, and half the time stay). Open question 10 |
| **Wood where trees are** | on ground `forage_ground` calls forest, dense forest, or swamp (the overworld and local areas by landscape; `data/recipes.toml` `free_wood = ["forest", "dense_forest", "swamp"]`), the shaft step is skipped and no billet is spent ("from what wood there is", today's rule kept where it makes sense); elsewhere a billet is needed, greyed "no wood to hand" without one. A ranger in the woods fletches as today, less the heads |
| **Fletch today** | P4 lists it as Woodcraft's `use = "fletch"` row unchanged; P6 replaces that row with the arrows and bolts recipes and retires `Action::Fletch` (`fletch` in action text stays readable as `craft:arrows` with no picks? **No**: old replays with `fletch` would need the heads they never had; `fletch` is removed, and the replays that use it (`tests/golden/manifest/`, `fletch_tests.rs`) are rewritten: saves are pinned, replays follow) |

`fletch.toml` keeps `count`, `value_per_count`, `knives`, `[[makes]]`; `seconds`, `difficulty`, `half_within`, `one_within`, `plain_from` move to the recipe and `data/recipes.toml`. `TalentMods::fletcher` is read in the arrows recipe's count and time (the recipe's `count = "fletch"` is where). `world::fletch_lack` becomes the arrows recipe's greying and refusal, in the same words.

## 11. What the player sees; determinism; saves; the wiki and the docs

**Hidden numbers.** The game shows what a recipe takes (stock in the player's units, the station, the time in words) and why a row is grey; never a difficulty, a `work` modifier, odds, or a margin. A made item says its name as Temper says it ("This is a Fine steel long sword!"). Salvage says what came out and what was lost, in words. The wiki states the tables: `wiki/src/crafting.md` (a new page under Skills in `SUMMARY.md`: how professions work, salvage, the forge, arrows), with generated tables from `xtask/src/wiki/crafting.rs`: `crafting-groups.md` (group, profession, stat, form word), `crafting-work.md` (every material's `work`), `recipes.md` (per profession: result, chosen parts, inputs, station, time, listed from, difficulty), `salvage.md` (the numbers, and what the smith-made weapons return). `altars-forges-springs.md`'s Temper section links to it; `skills.md`'s Metalcraft and Woodcraft rows and `skill-uses.md` point at the professions. `wiki/TERMS.md` gains the terms (profession, recipe, stock, salvage, ingot, billet, arrowhead).

**Determinism.** Every check draws from the gameplay stream in a fixed order (the materials in first-seen part order; the check, then the aspect); marks from the training stream; the recipe lists and `[[group]]` rows are `Vec`s read in path and file order, never a map iterated. The profession screen and the wizard read `PerceptionView` only (`recipes(skill)`, the inventory, `offer_at`); core recomputes availability when the action lands.

**Saves and replays.** New saved state: none beyond what exists (`Stack::parts`, `Worked`, the new objects by id). Content changes that move old hashes: the five skills on classes and races (the optional-skill streams draw in skill order: every class with an optional skill before a new one keeps its draws, those after move; the P1 PR notes which replays change), `piece`'s weight (an old save's pieces weigh more; pinned to the version, no shim), `Action::Salvage` and `Action::Craft` (new text; `fletch` removed in P6). `item_classes.toml` gains a class: `ItemClassId` is not saved.

**Docs.** A new topic `docs/dev/crafting.md` (professions, stock, salvage, recipes, the screen, the wizard; its tests), one line in CLAUDE.md's topic index, the action-syntax list (`salvage:LETTER`, `craft:ID[:POS,…]`), the `recipes` dump layer (dev: every profession's rows with availability and reason, `Long sword: 3 steel ingots, an anvil, about an hour (no anvil beside you)`), `docs/dev/objects.md` (the piece as stock), `docs/dev/skills.md` (the five skills, the professions in the menu), `docs/dev/screens.md` (the screens), `docs/dev/digging.md` (Mining), spec §7 "Objects and materials" (stock), §8 "Skills" (professions, salvage), §6 "Abilities" (the rows). Tests: `content/stock.rs`, `content/recipe.rs` (parsing, refusals), `world/crafting/salvage_tests.rs` (the owner's staff: both, one, none; a gem's cut kept; refusals; marks; a replay), `world/crafting/craft_tests.rs` (each recipe with each choice, the grades by margin, the failure's return, refusals, the stock spent at the end, a replay), `world/crafting/fletch_tests.rs` rewritten on the recipe, screen snapshots (the profession screen with a grey row, each wizard step, the dialog), `piece.rs` (the new unit, the form words), the abilities menu's rows.

## 12. Implementation plan: the phases

Each phase is its issue, refined from the section named; one PR each, in this order (P1 may run beside P2).

| Phase | Issue | Builds | Sections | Notes for the issue |
|---|---|---|---|---|
| **P1** | #1331 | the five `[[skill]]` rows; `Effect::Dig` in `dig_turn` and `Use::Dug` in `break_wall`; `Effect::Herbs` and `herbs` lists in `forage.toml` and `World::forage`; `Effect::IdentifyPotion` and `IdentifyMagic` in `appraise_hours`; the class, race, and background grants of the table; `docs/dev/skills.md`; the wiki's skills table (`cargo xtask wiki`) | §9 | the three `Effect`s are each one read where their neighbour is read (one hook each); no ability rows (Jewelcraft, Alchemy, and Enchanting are `action` skills and show "not usable yet" until each has a recipe directory, §8; Salvage in P3 is a row of its own, not Jewelcraft's; Mining and Herbalism are not `action`). Tests: the data loads; each skill trains and rolls; a herbalist's forage draws one more find and a plain one's draws as before; a miner digs faster; an alchemist's hour tells a potion |
| **P2** | #1332 | `piece.toml` to 500 g and class `materials`; the `materials` class row; `data/stock.toml` (`[[group]]` with form words, skill, stat, verb; `gems`); `content/stock.rs`; `piece::names` by form; `piece::count_for`; `MaterialDef::work` (default 0) and the table's values; `profession_of`; the shops' `buys`/`stock`; `docs/dev/objects.md`; the wiki's materials table gains `work` and the form | §3, §4 | Tests: names ("a steel ingot", "3 hardwood billets", "a lump of amber", "a piece of glass" without a row), stacking by material, weight and value, `count_for` against the table of §3, a save round-trip, a placement `"piece:steel*3"`, the shops stocking and buying it |
| **P3** | #1333 | `Ability::Salvage`, `Action::Salvage`, `world/crafting/salvage.rs`, `data/salvage.toml`, `ObjectList::Salvage` and `ScreenName::Salvage`, `Effect::Craft` on Metalcraft, Woodcraft, Jewelcraft with `Actor::skill_effect_of`, `Use::Salvage`; `docs/dev/crafting.md` (new); the wiki page's salvage section | §4 (the read), §5 | needs P1 (Jewelcraft) and P2. Tests as §11 lists; `ability` dump shows the row; a two-material item with both, one, or no checks made |
| **P4** | #1334 | `content/recipe.rs`, `data/recipes.toml`, the three `use` recipes (`recipes/metalcraft/temper.toml`, `recipes/woodcraft/whittle.toml`, `recipes/woodcraft/fletch.toml`); `Ability::Profession`, `Aim::Recipes`, the removal of the Temper, Whittle, Fletch, and `Ability::Skill` rows for professions; `PerceptionView::recipes`; `ui/profession_screen.rs`; the `recipes` dump layer; `Action::Craft` parsed and dispatched (refusing every generic recipe until P5: none exist); `--screen profession`; `docs/dev/screens.md` | §6 (the format, the `use` rows), §8 (the rows, the screen) | the wizard is P5's: with only `use` rows there is nothing to choose. Tests: the loader's refusals; the abilities menu lists "Metalcraft…" and "Woodcraft…" and no longer Temper, Whittle, or Fletch; `temper:`/`whittle:`/`fletch` unchanged; screen snapshots of the list and a grey row with its reason |
| **P5** | #1335 | the weaponsmith's recipe files of §6; `world/crafting/craft.rs` (`start_craft`, the `keep_at` loop, `crafted`: the check, the grades, the failure's return, identification, marks); `ui/craft_pick_screen.rs` (the wizard) and the dialog; `--screen craft-pick`; `Use::Craft`; the wiki's recipes table | §6, §7, §8 (the wizard, the dialog, the action) | needs P2 and P4. Tests as §11; the dev loop `craft:long_sword:c,f` end to end in a replay; the showcase's `forge` scenario gets ingots and a gem beside the anvil to try it |
| **P6** | #1336 | `objects/arrowhead.toml`; `recipes/metalcraft/arrowheads.toml`; `recipes/woodcraft/arrows.toml` and `bolts.toml` with `count = "fletch"`, `per_made`, `inputs_optional`, `free_wood`; `Action::Fletch` and the `fletch` row removed, `fletch.toml` trimmed; Fletcher's `plain` effect removed; `fletch_tests.rs` rewritten; the golden manifest's and replays' `fletch` steps rewritten; `docs/dev/skills.md` "Fletching" moves into `crafting.md` | §10 | needs P5. Tests: heads made and their metal; arrows from heads and wood with both parts; brittle without heads; the wood free in a forest and a billet elsewhere; Fletcher's ×2 and half time; refusal without a knife or a launcher |
| Later (unfiled) | — | Mining's ore and *Smelt* (§9), Herbalism's reagents and Alchemy's brewing (§9), Enchanting's extraction (§5, §9), armoursmithing (the armour objects as §6 recipes at an anvil; hide parts wait for leatherworking), Jewelcraft's *Cut* and *Set* (§9), javelins and spears (a second profession's stock in one recipe, §6), leatherworking and tailoring (hide and cloth get professions in `stock.toml`), materials in the smithy's goods beyond a few | | each a line on the epic once this document merges |

## 13. Open questions for the owner

1. **The unit.** 500 g of steel a piece (a dagger 1 ingot, a long sword 3, a great sword 7, a quarterstaff 8 billets): keep, or 100 g as today (×5 the counts), or 1 kg (a dagger costs and returns one ingot)?
2. **Hide, cloth, glass: no profession yet.** A leather grip is free in a recipe and lost in salvage, and no hide stock exists until leatherworking. Or should Woodcraft take hide for now, or should Leatherworking be a sixth skill in P1?
3. **Universal access.** Temper, Whittle, and Fletch stay anyone's (`anyone = true`), so "Metalcraft…" and "Woodcraft…" rows appear for every character, with only those entries for the untrained. Or should the rows be the trained's alone, and the three become theirs (the ranger keeps fletching; a warrior loses it)?
4. **A failed gem check** loses the gem (a diamond's check is against 18: a Jewelcraft-20 character fails it three times in five). Or should a failure take a cut (fine → fair) or a size, and only a natural 2 shatter it?
5. **A failed recipe** returns half the stock and the gems whole. Or all of it (the check only costs time), or none (the metal is slag)?
6. **Grades from the forge.** Fine at a margin of 6, Marvellous at 12, and the forge's attempts count toward Temper's rising difficulty. Or plain always, quality being the anvil's work alone?
7. **Recipes by skill value**, listed from a `from` value (scimitar and long sword from Metalcraft 20, great sword from 40), none learned or found. Agreed, or should some be found (a smith's pattern book) as a later mechanic on #196?
8. **Salvage anywhere with nothing**: or metal at an anvil only (as Temper), wood with a knife (as Whittle)?
9. **Who trains the five skills** (the table of §9): strike or add. In particular whether the witch takes Alchemy at `2d6+10` and the druid Herbalism, and whether dwarves get Mining beside Metalcraft.
10. **Brittle arrows by heads alone** (`plain_from` gone, Fletcher's "never brittle" gone; Fletcher keeps +4, ×2, half the time). Or should Fletcher's headless arrows stay plain?
11. **Mining's later shape**: ore smelted at an anvil (two steps, a second generic object), or ingots straight from the rock (one step, no new object)?
12. **Scribe** stays the wizard's class active, not a profession. Agreed?
13. **The anvil's walk-in dialog** keeps offering Temper (now opening the Metalcraft screen on its row). Or should walking into an anvil open the whole Metalcraft screen?
14. **Shops** buy and stock materials from P2 (the smithy, the weaponsmith, the armourer, the fletcher buy; the smithy and the general store stock a few). Or no trade in stock until the recipes exist (P5)?

## 14. Choices made along the way

| Choice | Taken | Rejected, and why |
|---|---|---|
| Stock objects | the one `piece`, named by group | one object per form (seven files that differ in a noun; the piece has the names, weight, worth, and stacking already); one object per material (56 files, and `made_of` lists would have to name objects) |
| Stock amount | by weight, `count_for` shared with `break_up` | a fixed count per recipe part (salvage and recipes disagree; a number per object); by share without grams (the same thing less exactly) |
| Difficulty per material | one `work` modifier | separate `salvage` and `craft` difficulties (they would move together; the recipe already has a base) |
| The profession's bonus | one `Effect::Craft` read per skill through a new `skill_effect_of` | an effect per profession (`craft_metal`, `craft_wood`, …: a variant per skill forever); reading the trained value through a shared curve (Skill focus and skill affixes would not count, unlike every other skill read) |
| Salvage's place | one row for everyone, cross-profession | a row per profession ("Salvage metal", "Salvage wood": the owner's staff needs two uses for one item); inside each profession screen (the same) |
| Salvage's station | none | an anvil for metal (Temper's reason is the item's improvement, which salvage does not do); open question 8 |
| Recipes known | by skill value | learned or found (new saved state and a source, unasked) |
| Quality | Temper's `Worked` grades by margin | a grade state of its own (two ladders on one item); none (the check only passes or fails) |
| Failure | half the stock back | all lost (a gold sink against the spirit of "time is the penalty"); nothing lost (a cosmetic check) |
| The wizard | paged lists on the screen stack, the one dialog | a form screen with fields (a new input mode); choosing in the abilities menu itself (no room for the running choice) |
| Rows | one per profession | one "Craft…" row with the professions as groups (the brief says per craft; a smith sees no empty Woodcraft) |
| Where Temper, Whittle, Fletch go | the profession screens' first rows, as `use` recipes | kept beside the rows (two places for one thing); made generic recipes at once (their rules are fine; the format carries them unchanged) |
| Arrowheads for bolts | one `arrowhead` object | a `bolt_head` (one more object for the same metal) |
| Fletching's wood | free where trees are, a billet elsewhere | always a billet (a ranger in the woods with no wood is absurd); always free (then wood stock has one use fewer) |
| Mining's first use | dig faster now, ore later | ore from the start (a second generic object and a recipe before the recipes exist) |
| Scribe | stays a class active | a scribing profession (spells are no stock; a design of its own) |
