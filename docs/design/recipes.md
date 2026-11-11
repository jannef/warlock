# Warlock — Recipes: the inventory for every item base, quality, gem cutting, herbs, and drops

Design document (2026-11-11; #1339, the P0b of the professions epic #1337), a follow-up to `docs/design/professions.md` (#1330, merged as #1342). Written against `origin/main` at 8e6fc152e (version 0.16.0). Nothing here is implemented; §11 maps it onto implementation issues, one PR each, against #1332–#1336, #1340, and #1341. It extends the professions design and takes its interim answers on #1337 as decided (the integrator's comment of 2026-11-11: the 500 g unit, hide to Woodcraft, universal Temper/Whittle/Fletch, a failed gem drops a cut or a size, half the stock back on a failure, Fine at 6 and Marvellous at 12, recipes by skill value, salvage anywhere, brittle arrows by heads alone, Mining later, Scribe a class active, the anvil's walk-in dialog, shops trading materials from P2). Where this document departs from `professions.md`, §2 says so and why; every departure follows the owner's comments on #1339 (primitive materials skip the smith) or the interim answers.

Every number below is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate). Numbers are a first pass; balance beyond it is out of scope. The aim throughout is the owner's: a solo game, short chains, few intermediates, no grinding, time as the only penalty.

## Contents

1. Summary
2. Departures from `professions.md`
3. Materials and stock: the full list, with its sources
4. Quality: the grades, and how crafting and tempering share them
5. Gem cutting: Cut and Set
6. The recipe inventory, by item class
7. Intermediates
8. Herbs (#1340): the list, the depth weighting, Gather
9. Material drops by creature tag (#1341)
10. The data format, the loader's checks, the wiki tables
11. Implementation issues
12. Open questions, each with the integrator's default
13. Choices made along the way

## 1. Summary

| Piece | Proposal |
|---|---|
| Two routes per base | **The recipe's profession is the recipe's, not the material's.** A base whose `made_of` allows metal has a Metalcraft recipe at an anvil; one that allows bone, stone, wood, horn, or hide has a Woodcraft recipe by hand, with a knife (the owner's comment: primitive materials skip the smith). A long sword is the smith's alone (metal and dragonbone: dragonbone blades stay loot); a knife, a dagger, a hand axe, a spear, a javelin, a club, a staff, arrows, bolts, shields, helmets, scale mail, and every piece of hide gear have a hand route. §6 |
| Stock | `professions.md`'s piece, by weight, unchanged in principle. Added: **`sinew`** (a new material and one-member group, a *length of sinew*: a bow's string), a **`unit` per group** (a hide is 8 pieces' worth, 1.4 kg of leather: a leather jerkin takes 3 hides, not 20), and **material overrides** in `stock.toml` (flint, stone, and obsidian are knapped: Woodcraft's; jade is cut: Jewelcraft's; fur's form word is *pelt*). **Herbs and essences are objects**, not materials (class `materials`, one file each): nine herbs, six elemental essences. §3 |
| Quality | Temper's ladder, Temper's words. A made item carries a `Worked` with as many improvements as its margin earns, **Fine at 6, Marvellous at 12, Masterwork at 20**, and `attempts` equal to its improvements, so a crafted Masterwork is `finished()` and the anvil refuses it, and a crafted Fine item's next Temper is at `13 + 2`. One ladder, one count. Material quality enters only through `work` (a mithril blade is less often Fine). Counted results, potions, rings, and amulets have no grade. §4 |
| Gem cutting | Jewelcraft's **Cut**: an uncut gem becomes a cut one in one check, its grade by the margin in the cut words gems already have (crude 0, fair 5, fine 10, exquisite 15, flawless 20); a crude result **loses a size** (a hurried cut wastes stone); a failure keeps it uncut and loses a size; a natural 2 shatters it. A cut gem may be **re-cut**, always a size smaller, its grade the better of old and new; a tiny gem is never re-cut. **Set** puts a cut gem into an item's ornament part. The gem's quality words are the cuts; Fine/Marvellous/Masterwork are never a gem's. §5 |
| Intermediates | **Two**: the arrowhead (metal heads for arrows and bolts; bone and flint heads need none: a shard or a block heads a batch directly) and the cut gem (Cut's product, Set's and the forge's input). Nothing else stands between stock and item. §7 |
| Herbs | nine herb objects gathered from glowing fungus by Herbalism's **Gather** (a *Here* ability aimed at a fungus cell beside the player, five minutes, `1d2` + Herbalism ÷ 50 sprigs; anyone may, untrained one sprig), drawn by a `find`/`deeper`/`level` table as gems are (glowcap and moonwort everywhere, mandrake from depth 6); the fungus never depletes; the same herbs join `forage.toml`'s `herbs` lists on the surface. Each herb is an input of Alchemy's brewing. §8 |
| Drops | `data/drops/<tag>.toml`, one file per creature tag: `[[drop]]` rows of a material (stock) or an object (an essence, a herb), a count as dice, a chance; a creature's own `[[creature.drops]]` adds or switches the tags' off. Dropped on death as loot is, from a stream of its own; a row applies to creatures that leave a corpse unless it says `bodiless`; small creatures roll at half the chance. Beasts give hide, bone, and sinew; kobolds flint; the goblin tribes and orcs scrap iron and leather; duergar and drow ingots; trolls and giants bone and hardened leather; skeletons bone; elementals their essence; dragons dragonbone and dragonscale. No butchering action. §9 |
| Format | `professions.md` §6 extended, not replaced: a part may name several sources (`stock` lists, an `input` object, `optional`), `inputs` may name a material or a group (stock as a fixed input: a spear's haft of any wood, a crossbow's iron lock, a bow's sinew), `failure` and `fumble` name what a failed brew leaves (a murky draught; a brewer's blunder on a natural 2), `use = "cut"` and `use = "set"` are Jewelcraft's built-ins, `data/recipes.toml` carries the grade bands. One file per recipe, herb, essence, drop table. Wiki tables per profession, a herbs table, a drops table. §10 |
| Issues | fourteen, one PR each, ordered in §11: P2b stock extensions, P4b the format, P5 refined, P15 drops (parallel), P6 refined, P5b primitive weapons, P7 bows, P8 hide gear, P9 armoursmithing, P10 Cut, P11 Set and jewellery, P12 Gather, P13 brewing, P14 essences and battle flasks. §11 |

## 2. Departures from `professions.md`

| `professions.md` said | This document says | Why |
|---|---|---|
| §4: stone and resin → Jewelcraft; hide, cloth, glass no profession | hide → Woodcraft (the interim answer 2); **flint, stone, obsidian → Woodcraft** (knapped by hand); jade and amber stay Jewelcraft's (cut and set). Cloth and glass still none | the owner's comment: flint and stone tools are hand-made with Woodcraft. Salvage follows the same map, so a flint knife comes apart by the skill that made it. Done as material rows over the group rows in `stock.toml`, not by moving the groups (jade is a lapidary's stone) |
| §3: one piece unit, 500 g of steel scaled by material weight | the same, times a **`unit` per group** (default 1): hide 8, cloth 4 | at 175 g a "leather hide" is a scrap, and a jerkin would take twenty; a hide of 1.4 kg takes three. The rule stays one (`count_for`: grams per piece = 500 × weight % × unit) |
| §7: no Masterwork at the forge | **Masterwork at a margin of 20** | the owner's brief: "a crafted item comes out at one of those grades … so a crafted Masterwork can't be tempered further" asks for the whole ladder. At 20 only a master (Metalcraft 200, a strong arm) sees it one time in four; a journeyman never |
| §6: `javelin` and `spear` wait for the input rule to take stock | the rule is here: `inputs` may name a material or a group, and a part may take `stock` of another profession's group | the owner's spears and javelins (a bone point on a wooden shaft) need it at once |
| §10: arrows take arrowheads or nothing (brittle) | arrows and bolts take **arrowheads, or a bone shard or flint block, or nothing** | the owner: "bone or flint heads with wood shafts, made straight from the raw materials. No arrowhead intermediate is needed for these" |
| §7 and interim 5: a failed recipe returns half the stock | a failed **brew** spends its herbs and leaves a **murky draught**; a natural 2 a **brewer's blunder** (both exist, both are what a bad brew is). Every other recipe returns half as decided | herbs are cheap and plentiful (§8); a failure that makes something bad is the alchemist's own kind of loss, and it gives the two gamble potions a source. Open question 6 |
| §9: Herbalism's first use is a second find on a forage | kept, and **Gather** added as its action (§8) | #1340 |

Everything else in `professions.md` stands: the piece, by weight, `work`, salvage, the recipe directory, the check, the menus, the wizard, the dialog, the phases P1–P6.

## 3. Materials and stock: the full list, with its sources

**Decision: the 56 materials plus `sinew`; herbs and essences are objects.** Every material of a group is stock (`piece` of it); the ten gems are loose gems; the groupless (paper, wax, plant, flesh, lead) are never stock. The table gives each material's profession (the skill whose checks salvage and work it; a recipe's own profession may differ, §6), its form word, its `work` (the professions design's values, unchanged), and **where it comes from**: salvage (any item with it), shops (the smithy and the general store stock a few, §3 of `professions.md`), drops (§9, by tag), or gathering (§8).

| Group (`unit`) | Form word | Profession | Materials (`work`) | Sources besides salvage |
|---|---|---|---|---|
| `metal` (1) | ingot | Metalcraft (Strength) | tin, pewter, zinc, cadmium, bendalloy (−3); copper, gold (−2); iron, bronze, brass, aluminium, silver, electrum (−1); steel, duralumin (0); chromium (+2); nicrosil (+3); mithril (+4); adamantium (+8) | shops: iron, steel, copper, bronze (a few ingots); drops: iron from goblins, hobgoblins, orcs, outlaws; iron and steel from duergar; steel and mithril from drow (§9); ore later (Mining) |
| `wood` (1) | billet | Woodcraft (Dexterity) | wood (−1), hardwood (0), ironwood (+3) | shops: wood, hardwood; drops: kobolds (wood, their clubs' stuff); free on wooded ground for arrows only (`professions.md` §10) |
| `bone` (1) | shard | Woodcraft | bone, horn (−1); ivory, chitin (0); dragonbone (+5) | drops: bone from beasts, vermin, undead, skeletons, trolls, giants, marsh-things; horn from boars, deer, devils; chitin from beetles, centipedes, spiders; dragonbone from dragons |
| `stone` (1) | block | **Woodcraft** for stone, flint, obsidian (knapped); **Jewelcraft** for jade (cut) | stone, flint (0); obsidian (+1); jade (+2) | drops: flint from kobolds and goblins; stone from the mud golem and earth elementals; jade never dropped (salvage of jade amulets, loot); a broken wall later (Mining) |
| `hide` (**8**) | hide; fur: **pelt** | **Woodcraft** (the interim answer 2) | leather, fur (−1); hardened leather (0); dragonscale (+5) | drops: leather from beasts, goblins, hobgoblins, orcs, outlaws; pelts from furred beasts (wolf, bear, lynx, panther, badger, polecat, hare); hardened leather from trolls, crocodiles, giants; dragonscale from dragons |
| `sinew` (1), **new** | length (of) | Woodcraft | sinew (−1) | drops: beasts, giants, trolls; never in shops (no one sells it) |
| `resin` (1) | lump (of) | Jewelcraft (Dexterity) | amber (+1) | salvage of amber amulets; loot |
| gems (no group) | a loose gem | Jewelcraft | smokestone (0); garnet, zircon, amethyst (+1); topaz, heliodor (+2); emerald (+3); ruby, sapphire (+4); diamond (+6) | loot and the gems' chance on drops, as today |
| `cloth` (4) | bolt (of) | none yet | cloth, wool, silk | not stock until tailoring: free in recipes, lost in salvage |
| `glass` (1) | lump (of) | none yet | glass | the same: a potion's flask is free (§6 Potions) |

`sinew` (`materials/sinew.toml`): group `sinew`, `weight = 30`, `value = 20`, no `rarity` or `gear` (loot never makes an item of it), `fit = { weapon = "odd", armour = "odd", jewellery = "odd" }`, `resists = { acid = 20, rust = 100, fire = 0 }`. It is an input (a bow's string, a lute's strings), never a part.

**Herbs and essences are objects** (class `materials`, stackable, `materials = ["plant"]` for a herb and a glyph of its own, 20 g; an essence 50 g, `fg` its element's), one file each in `objects/`, no `level` or `frequency` (never floor loot; the alchemist's `always` lists the two common herbs). Why objects and not materials of a `herb` group: a material is what items are made of, and `made_of = ["any"]` on jewellery would admit a moonwort ring; an object is a thing in a pack, and `inputs = [{ object = "moonwort", count = 2 }]` is the format's plain fixed input. The piece's names would also have needed a form word per herb. §8 lists the herbs, §9 the essences.

## 4. Quality: the grades, and how crafting and tempering share them

**Decision: one ladder, one count.** A made item's quality is a `Worked` (`magic/worked.rs`) exactly as Temper leaves it: its improvements (each an `Aspect` the base item has, drawn as Temper draws) and its `attempts`. The check is `professions.md` §7's; the margin bands are these (`data/recipes.toml`, §10):

| Margin | Grade | `Worked` | What the player hears |
|---|---|---|---|
| ≥ 20 | **Masterwork** | three improvements, `attempts` 3: `finished()` is true, so Temper refuses it ("It can be made no better.", the rule for a thrice-tempered item today) | "You have made a Masterwork steel long sword!" |
| 12 to 19 | **Marvellous** | two, `attempts` 2 | "This is a Marvellous steel long sword!" |
| 6 to 11 | **Fine** | one, `attempts` 1 | "This is a Fine steel long sword." |
| 0 to 5 | plain | no `Worked` (none is state) | "You have made a steel long sword." |
| below 0 | failed | nothing made; half the stock back, gems whole, object inputs at half (interim 5); a brew: §6 Potions | "The blade comes out brittle and you break it up for the metal." |
| a natural 2 | failed | as failed; a gem in the work shatters (§5); a brew leaves a brewer's blunder | |
| a natural 24 | made at least Fine | the margin's grade, or Fine if the margin says plain | |

**Why `attempts` equals the improvements.** Temper's difficulty is `13 + 2 × attempts`, counting every attempt ever made on the item. A forge that set `attempts` 0 on a Marvellous blade would let the anvil finish it at the base difficulty, as if the forge's work were free; setting it to the improvements makes a crafted Fine item exactly a once-tempered one, and the ladder stays one. A failed craft makes no item, so it leaves no `attempts` anywhere (a failed Temper damages an item; a failed forging has none to damage).

**What has a grade.** Anything whose base item has an `Aspect` (`worked::aspects`): weapons (Die, ToHit, Damage), armour, shields, and parrying weapons (Defence), anything with Protection. A **counted** recipe (arrows, arrowheads, sling stones, flaming arrows) has none: its margin scales the count by Fletch's bands (`professions.md` §7). Potions, rings, amulets, clothing, tools without an attack, and the lute have no aspect and come out plain whatever the margin; the check still passes or fails. Gems have the cut words (§5).

**Material quality.** Enters once, through `work` on the difficulty: a mithril blade (+4) is Fine 4 points less often than a steel one and Masterwork rarely; a tin one (−3) is easily Fine and still tin. No second channel (a material has no quality of its own; its worth and its blows are its parts').

**Odds, for the wiki** (2d12 plus the smith's whole bonus, the stat modifier and the `craft` points and the talents together, against `difficulty + work`; the long sword, 13, steel): at a bonus of +4 (an apprentice: a modest arm, Metalcraft 50) failed 19 %, plain 42 %, Fine 31 %, Marvellous 7 %, Masterwork never; at +8 (Metalcraft 100): 4 / 27 / 44 / 25 / 0; at +12 (Metalcraft 200, an average arm): 1 / 10 / 35 / 47 / 7; at +16 (Metalcraft 200 and a strong arm): 1 / 0 / 19 / 56 / 25. The wiki's crafting page generates such a table per profession from the data, at bonuses the player can reach.

**Gear condition and drops.** `data/gear_condition.toml` already rolls Fine/Marvellous/Masterwork on creature gear through the same `Worked`; nothing changes there. A crafted item dropped keeps its condition, as any does.

## 5. Gem cutting: Cut and Set

**Decision: one check sets the cut; stone is lost on the way down; the words are the cuts.** Both are Jewelcraft's built-in recipes (`use = "cut"`, `use = "set"`, as Temper is Metalcraft's), listed on its profession screen under *Work*; Dexterity's check plus Jewelcraft's `craft` points against `difficulty + work` of the gem.

| Piece | Design |
|---|---|
| **Cut** | `recipes/jewelcraft/cut.toml`: `use = "cut"`, `difficulty = 12`, `seconds = 1800`, `from = 0`. `Aim::Item` over `ObjectList::Cut`: the loose gems carried that may be cut (any uncut gem; any cut gem below flawless and above tiny). One check when the time is up. On an **uncut** gem: margin below 0, still uncut and **one size smaller** ("The stone splits badly; what is left is smaller."); 0 to 4 **crude**, one size smaller ("You rough it out, and lose more than you meant to."); 5 to 9 **fair**, 10 to 14 **fine**, 15 to 19 **exquisite**, 20 or more **flawless**, the size kept. A natural 2 shatters it whatever the size. The bands are a step of 5 (`data/recipes.toml` `cut_step = 5`), flawless where the forge's Masterwork is: a step of 6 would put flawless out of a master's reach on any precious stone (a ruby's `work` is +4) |
| **Re-cut** | the same recipe on a gem already cut (crude to exquisite): the check as above, the new grade the **better** of the old and the check's (a re-cut never worsens a cut: the cutter stops when it is no better), and the gem is **always one size smaller** (recutting takes stone). A **tiny** gem is never re-cut (refused: "There is nothing left to cut."); a flawless one is never cut again ("It could not be finer."). A natural 2 shatters it. Worth it on a large crude stone (a large crude ruby 2700 → a medium flawless 4800), seldom otherwise: the wiki says so with the multipliers |
| **Set** | `recipes/jewelcraft/set.toml`: `use = "set"`, `difficulty = 12`, `seconds = 1200`, `from = 10`. Two picks on the wizard: the item (an item carried whose ornament part is unset: `ObjectDef::ornament` else its class's `[[ornament]]` row, `gems.toml`), then the cut gem (never an uncut one: gems.toml's rule). Made: the gem becomes the item's ornament part with its size and cut (`Stack::with_parts`: "a ruby-pommelled steel long sword", "a gold and sapphire amulet"), the gem spent. Failed: the item untouched, the gem **one cut rougher** (a chipped stone; crude stays crude), a natural 2 shatters it (interim 4's shape). An item with its ornament set already is not listed ("It is set already.") |
| The forge's gem | `professions.md` §6's `ornament = true` stays: a gem chosen in the forging wizard is set with the work, without a check and never lost on a failed forging. Set is for items the player already has (a found sword, a bought ring) |
| What a set gem does | nothing but worth, as today (`gems.toml`: "does nothing else yet"). Enchanting's design may give a set gem a role later; not here |
| Salvage of a set gem | as decided (interim 4): a failed check drops a cut or a size (the first that can drop: a cut while above crude, else a size while above tiny), a natural 2 shatters |
| Words | the game says the cut and the size as names do ("a medium fine ruby"), and the lines above; never the margin. The gem's quality words are **uncut, crude, fair, fine, exquisite, flawless**; Fine/Marvellous/Masterwork are never said of a gem (the brief) |
| Marks | `Use::Craft` on Jewelcraft, 100 %, made or failed |
| Odds (wiki) | an uncut ruby (+4) against 16, by the cutter's whole bonus: at +4 (an apprentice) shattered 1 %, failed 38 %, crude 37 %, fair 21 %, fine 4 %; at +8: 1 / 14 / 31 / 35 / 17 / exquisite 2; at +12: 1 / 1 / 17 / 35 / 31 / 14 / flawless 1; at +16 (a master with a steady hand): shattered 1 %, crude 3 %, fair 21 %, fine 37 %, exquisite 28 %, flawless 10 %. The same master on a smokestone (`work` 0): fair 6 %, fine 24 %, exquisite 38 %, flawless 31 % |

Why no "quality" state on a gem beyond the cut: the brief asks for the gem's own words and the size, and `GemCut { size, cut }` already carries both on the part; a third state would be a second ladder on one stone.

## 6. The recipe inventory, by item class

Every base in `assets/data/objects/` (291 files, 293 objects with the arrows' three), by class, in `item_classes.toml`'s order. Columns: the recipe's profession (M Metalcraft at an anvil, W Woodcraft by hand with a knife, J Jewelcraft, A Alchemy; two rows where a base has two routes); what the maker chooses and what is fixed; the stock the base's own weight prices (`count_for`, by its own materials: a variant's count follows that material's weight: a bronze long sword's blade 1620 g × 112 % / 500 = 4 ingots); `difficulty` (before `work`); `seconds`; `from` (the trained value it is listed from; W recipes at `from` 0 are `anyone` where today's Fletch and Whittle are, else trained only). **Never crafted** bases say why. Times: a knife 20 min, a sword an hour, a great sword or a polearm an hour and a half, mail two hours, plate four; hand-work the same scale.

### Melee weapons

| Base | Route | Chosen / fixed | Stock | Diff. | Seconds | From |
|---|---|---|---|---|---|---|
| knife | M | main `stock` (metal) | 1 ingot | 10 | 1200 | 0 |
| knife | W | main `stock` (stone: flint, obsidian, stone; bone: bone, horn, chitin) | 1 block or shard | 9 | 1200 | 0 |
| dagger | M | main `stock` (metal); `ornament` pommel | 1 | 11 | 1800 | 0 |
| dagger | W | main `stock` (stone, bone) | 1 | 10 | 1800 | 0 |
| stiletto | M | main `stock` (metal) | 1 | 11 | 1800 | 0 |
| stiletto | — | no hand route: a slim steel point (its `made_of` bone and stone fall to the dagger's recipe) | | | | |
| short sword | M | main; ornament | 2 | 12 | 3600 | 10 |
| scimitar | M | main; ornament | 3 | 13 | 3600 | 20 |
| long sword | M | `blade` stock; ornament; the grip free (leather is free until a hide part costs: open question 13) | 3 | 13 | 3600 | 20 |
| rapier | M | main; ornament | 2 | 14 | 3600 | 30 |
| great sword | M | main; ornament | 7 | 15 | 5400 | 40 |
| the swords of dragonbone | — | loot only: `made_of` allows dragonbone, but no carving recipe for a blade; a later Woodcraft "carve" line if dragonbone stock ever piles up | | | | |
| hand axe | M | main `stock` (metal); throwable | 3 (iron) | 11 | 1800 | 0 |
| hand axe | W | main `stock` (stone) | 3 blocks | 10 | 1800 | 0 |
| battle axe | M | main | 5 | 13 | 3600 | 20 |
| great axe | M | main | 10 | 14 | 5400 | 30 |
| battle axe, great axe of stone | — | not by hand: a knapped head that size shatters; `made_of` keeps stone for loot's sake only | | | | |
| mace | M | main | 5 | 11 | 3600 | 0 |
| morning star | M | main | 6 | 12 | 3600 | 10 |
| war hammer | M | main | 5 | 12 | 3600 | 10 |
| maul | M | main | 12 | 13 | 5400 | 20 |
| club | W | main `stock` (wood, bone); `ornament` inlay | 7 billets or 7 shards | 8 | 1200 | 0 |
| quarterstaff | W | main `stock` (wood); ornament tip | 10 billets of any wood (a variant's grams scale with its material's weight, so the count holds) | 9 | 1800 | 0 |
| ironwood staff | W | main `stock` (ironwood only); ornament tip | 7 ironwood billets | 12 (+3 `work`) | 2700 | 30 |
| spear | M | `head` stock (metal); `haft` stock (wood, any profession's group); ornament inlay | 1 ingot, 7 billets | 11 | 1800 | 0 |
| spear | W | `head` stock (stone, bone); `haft` stock (wood) | 1 block or shard, 7 billets | 10 | 1800 | 0 |
| javelin | M | head stock (metal); haft stock (wood); throwable | 1, 4 | 10 | 1200 | 0 |
| javelin | W | head stock (stone, bone); haft stock (wood) | 1, 4 | 9 | 1200 | 0 |
| trident | M | main | 4 | 13 | 3600 | 20 |
| glaive | M | main; ornament inlay | 10 | 14 | 5400 | 30 |
| halberd | M | main; ornament inlay | 11 | 15 | 5400 | 40 |

### Missile weapons

| Base | Route | Chosen / fixed | Stock | Diff. | Seconds | From |
|---|---|---|---|---|---|---|
| short bow | W | main `stock` (wood, horn, dragonbone); `inputs` 1 sinew | 4 billets, 1 length | 11 | 3600 | 0 |
| long bow | W | the same | 7 billets, 1 length | 13 | 3600 | 20 |
| composite bow | W | main `stock` (horn, wood, dragonbone); `inputs` 2 sinew | 5 horn shards (or 5 billets), 2 lengths | 15 | 5400 | 40 |
| crossbow | W | main `stock` (wood); `inputs` 1 ingot of any metal (the lock and trigger: `{ group = "metal", count = 1 }`) | 16 billets, 1 ingot | 14 | 5400 | 30 |
| heavy crossbow | W | the same | 23 billets, 1 ingot | 16 | 7200 | 60 |
| sling | W | main `stock` (hide) | 1 hide | 8 | 600 | 0 |

A bow's `made_of` wood, horn, dragonbone makes the composite bow of horn the real one and a wooden "composite bow" a cheaper variant; the wizard offers what is carried. No Metalcraft route: nothing shoots that a smith makes.

### Missiles

| Base | Route | Chosen / fixed | Stock | Diff. | Seconds | From |
|---|---|---|---|---|---|---|
| arrow | W | `head`: arrowheads (`input`), **or** a shard or block (`stock` bone, stone), **or** none (`optional`: the brittle arrow); `shaft` stock (wood; free on wooded ground); `knife`, `launcher`; `count = "fletch"` | heads as made (1 arrowhead each; 1 shard or block a batch), 1 billet a batch up to 11 | 12 | 600 | 0 (`anyone`) |
| bolt | W | the twin, for a crossbow ready | 1 shard or block a batch; billets by the batch's weight (4 bolts a billet) | 12 | 600 | 0 (`anyone`) |
| brittle arrow, brittle bolt | W | the arrow or bolt recipe with no head chosen | 1 billet | 12 | 600 | 0 |
| sling stone | W | main `stock` (stone); `count = 10`; no knife (stones are chipped with a stone) | 1 block | 6 | 600 | 0 (`anyone`) |
| flaming arrow | W | `inputs` 10 arrows (any, `per_made`), 1 flask of oil; `count = 10`; no check to speak of | — | 8 | 600 | 0 |
| burning arrow | — | never: what a flaming arrow flies as |
| arrowhead | M | main `stock` (metal); `count = 20`; counted | 1 ingot a batch | 10 | 600 | 0 |

The arrow's parts are the head's material (an arrowhead's metal, or the shard's bone, or the block's flint) and the shaft's wood: "flint-tipped arrows", "bone-tipped hardwood arrows", their own goods, stacking only alike. Flint-tipped arrows strike as flint strikes (the material's `to_hit` and `damage`), which is the point of the metal route.

### Shields

| Base | Route | Chosen / fixed | Stock | Diff. | Seconds | From |
|---|---|---|---|---|---|---|
| buckler | M | main `stock` (metal); `ornament` inlay | 3 ingots | 11 | 3600 | 0 |
| buckler | W | main `stock` (wood, hide, chitin, dragonscale) | 3 billets, or 1 hide, or 3 shards | 10 | 3600 | 0 |
| small shield | W | main (wood, hide, chitin, dragonscale); ornament | 13 billets, or 2 hides, or 13 shards of chitin | 10 | 3600 | 0 |
| small shield | M | main (metal) | 13 ingots (a wooden shield's weight in iron) | 12 | 3600 | 10 |
| large shield | W | main; ornament | 30 billets, or 4 hides | 12 | 5400 | 20 |
| large shield | M | main | 30 ingots | 13 | 5400 | 20 |
| tower shield | M | main; ornament | 24 ingots | 14 | 7200 | 30 |
| tower shield | W | main (wood, hide, chitin, dragonscale) | 24 billets, or 3 hides | 14 | 7200 | 30 |

Shields are heavy in the data (a large shield 6 kg of wood), so their billet counts are large: by weight, as everything is. Wood is free nowhere but for arrows (open question 14).

### Body armour

| Base | Route | Chosen / fixed | Stock | Diff. | Seconds | From |
|---|---|---|---|---|---|---|
| padded coat | — | never yet: cloth has no profession (tailoring later) |
| leather jerkin | W | main `stock` (hide) | 3 hides | 11 | 3600 | 0 |
| studded leather | W | main `stock` (hide); `inputs` 1 ingot of any metal (the studs) | 5 hides, 1 ingot | 13 | 5400 | 20 |
| chain mail | M | main `stock` (metal) | 22 ingots | 14 | 7200 | 30 |
| scale mail | M | main (metal) | 28 ingots | 14 | 7200 | 30 |
| scale mail | W | main (bone, dragonscale: scales of bone or dragon) | 28 shards, or 4 hides of dragonscale | 14 | 7200 | 30 |
| banded mail | M | main | 34 ingots | 15 | 10800 | 40 |
| plate armour | M | main (metal); its `made_of` chitin and dragonbone are loot's | 44 ingots | 17 | 14400 | 60 |

A suit of chain weighs 11 kg and costs 22 ingots: the by-weight rule, honest about what mail is. Salvage returns 11. Open question 15.

### Helmets

| Base | Route | Chosen / fixed | Stock | Diff. | Seconds | From |
|---|---|---|---|---|---|---|
| leather cap | W | main `stock` (hide, bone) | 1 hide or 3 shards | 9 | 1800 | 0 |
| leather cap | M | main (metal: a skull cap) | 3 ingots | 11 | 1800 | 0 |
| iron helm | M | main (metal); `ornament` studs | 3 | 12 | 3600 | 10 |
| iron helm | W | main (hide, bone) | 1 hide or 3 shards | 11 | 3600 | 10 |
| great helm | M | main (metal); ornament studs | 6 | 14 | 5400 | 30 |
| great helm | W | main (hide, bone) | 1 hide or 6 shards | 13 | 5400 | 30 |

### Cloaks, gloves, boots

| Base | Route | Chosen / fixed | Stock | Diff. | Seconds | From |
|---|---|---|---|---|---|---|
| fur cloak | W | main `stock` (fur, leather) | 2 pelts | 10 | 3600 | 0 |
| travelling cloak | W | main (leather, fur) | 1 hide | 10 | 3600 | 0 |
| wool cloak | — | never yet: cloth; its `made_of` fur and leather are the two recipes above (a "wool cloak" of leather is the travelling cloak's cheaper twin: loot's business) |
| leather gloves | W | main (hide) | 1 hide | 9 | 1200 | 0 |
| gauntlets | M | main (metal) | 2 ingots | 12 | 3600 | 10 |
| leather boots | W | main (hide) | 1 hide | 9 | 1800 | 0 |
| iron-shod boots | M | main (metal); the leather free | 4 ingots | 12 | 3600 | 10 |

### Clothing

Never magic, never loot, worth a few coins; the hide and wood pieces are a hand's work for anyone, the cloth and wool pieces wait for tailoring.

| Base | Route | Stock | Diff. | Seconds | From |
|---|---|---|---|---|---|
| hide tunic, leather apron | W, main `stock` (hide) | 1 hide | 8 | 1800 | 0 |
| sandals, shoes, turnshoes, soft boots, hobnailed boots | W (hide); hobnailed: `inputs` 1 ingot | 1 hide | 7 | 1200 | 0 |
| clogs | W (wood) | 5 billets | 7 | 1200 | 0 |
| tunic, shift, smock, patched smock, frock, kirtle, gown, robe, habit, doublet, waistcoat, houppelande, long coat, silk shirt, footwraps, felt boots, slippers | — | never yet: cloth and wool have no profession (tailoring, the epic's "later"); the doublet's `made_of` hide gets a W recipe then too |
| reed kilt | — | never: plaited reeds, no one's craft; bought or found |

### Rings and amulets

Jewelcraft's, Dexterity; a ring's `made_of = ["any"]` is wide, but a recipe names the base object, so each is **its own metal** (a gold ring of iron would be worth gold by `value_with`; the three ring objects exist for their three prices).

| Base | Route | Chosen / fixed | Stock | Diff. | Seconds | From |
|---|---|---|---|---|---|---|
| iron ring | J | main `stock` (iron only); `ornament` setting (optional) | 1 ingot | 11 | 3600 | 0 |
| silver ring | J | main (silver); ornament | 1 | 13 | 3600 | 20 |
| gold ring | J | main (gold); ornament | 1 | 13 | 3600 | 20 |
| bone amulet | **W** | main `stock` (bone, horn, ivory); the cord free | 1 shard | 10 | 1800 | 0 |
| jade amulet | J | main (jade) | 1 block | 12 | 3600 | 10 |
| amber amulet | J | `pendant` stock (amber); the cord free | 1 lump | 12 | 3600 | 10 |
| gold amulet | J | `chain` stock (gold); `setting` a cut gem, **required** (`gem = true`: "a gold and sapphire amulet") | 1 ingot, a gem | 14 | 5400 | 30 |

A 10 g ring costs a 1.23 kg gold ingot: `count_for`'s minimum of one piece. The ingot is worth 20 coins and the ring 150, so the jeweller's margin is real; gold ingots come from salvaged gold (never from coins), a rare source. Open question 12.

### Light sources and tools

| Base | Route | Chosen / fixed | Stock | Diff. | Seconds | From |
|---|---|---|---|---|---|---|
| candle | — | never: wax has no profession; bought |
| lantern | M | main `stock` (iron, brass); the glass free | 1 ingot | 10 | 1800 | 0 |
| pick-axe | M | main (metal); the haft free | 6 ingots | 12 | 3600 | 10 |
| pick-axe | W | main (stone: a stone pick, a digger's first tool) | 6 blocks | 11 | 3600 | 10 |
| mattock | M | main (metal) | 9 ingots | 13 | 3600 | 20 |
| mattock | W | main (stone) | 9 blocks | 12 | 3600 | 20 |
| rope | — | never yet: cloth |
| the keys (brass, bronze, copper, iron, lead, rusty, skeleton) | — | never: a key opens a lock a level names; worth 0, never traded, never made |

### Household goods

| Base | Route | Why |
|---|---|---|
| candlestick, cooking pot, cutlery, kettle, ladle, teapot, warming pan, hanging lantern (metal); bellows, broom, bucket, chair, coat stand, stool (wood); book (paper); rug (wool) | — | **never crafted; salvage only.** Each is worth about what the stock in it is, or less (a chair: 25 billets of wood, worth 5 coins; the chair 4), nobody needs a teapot, and an animated thing already breaks into its pieces. Salvage takes them apart as any item |

### Food

Never crafted: cooking is no profession, and foraging and shops cover food. Angelica root stays a foraged remedy (and a herb list's member on the surface, §8).

### Potions (Alchemy)

Alchemy's recipes, Intelligence, no station, no knife, 1200 s each; `inputs` only (herbs, §8; essences, §9; a potion now and then), nothing chosen; the flask is free (glass has no profession: the alchemist has bottles as the smith has a scrap of leather). Failed: the herbs spent, a **murky draught** made (`failure = { object = "murky_draught" }`); a natural 2: a **brewer's blunder** (`fumble`). Made: the potion, identified (the maker knows their work). No grade.

| Potion | Inputs | Diff. | From |
|---|---|---|---|
| potion of healing | 2 moonwort | 10 | 0 |
| potion of greater healing | 3 moonwort, 1 comfrey | 13 | 25 |
| potion of full healing | 4 moonwort, 2 comfrey, 1 mandrake | 18 | 100 |
| potion of mana | 2 glowcap | 10 | 0 |
| potion of sustenance | 3 glowcap | 9 | 0 |
| potion of antidote | 1 feverfew, 1 glowcap | 11 | 10 |
| potion of cure disease | 2 feverfew, 1 comfrey | 13 | 25 |
| potion of clear head | 2 vervain | 11 | 10 |
| potion of barkskin | 2 comfrey, 1 bog myrtle | 12 | 20 |
| sleeping draught | 2 henbane | 11 | 10 |
| stinking flask | 1 henbane, 1 bog myrtle | 11 | 10 |
| potion of cat's eyes | 2 eyebright | 12 | 25 |
| potion of owl's eyes | 1 eyebright, 2 glowcap | 12 | 25 |
| potion of moth's eyes | 1 eyebright, 1 henbane | 12 | 25 |
| potion of warmth | 2 bog myrtle | 12 | 25 |
| potion of frost (chill-proof) | 1 bog myrtle, 1 essence of ice | 13 | 40 |
| potion of earthing | 1 bog myrtle, 1 essence of earth | 13 | 40 |
| potion of proofing | 1 bog myrtle, 1 feverfew | 12 | 25 |
| potion of might / grace / vigour / wit / will / presence | 1 mandrake + 1 comfrey / eyebright / moonwort / vervain / feverfew / bog myrtle | 14 | 50 |
| greater potion of might … presence | 2 mandrake + 2 of the same second herb | 17 | 100 |
| potion of renewal | 2 comfrey, 1 mandrake | 15 | 75 |
| flask of fire | 1 essence of fire, 1 flask of oil | 14 | 50 |
| flask of frost | 1 essence of ice, 1 glowcap | 14 | 50 |
| flask of thunder | 1 essence of lightning, 1 glowcap | 15 | 60 |
| flask of smoke | 1 essence of air, 1 henbane | 12 | 25 |
| pitch fire | 2 essence of fire, 1 flask of tar | 17 | 100 |
| blasting flask | 1 essence of fire, 1 essence of lightning | 18 | 100 |
| potion of sea legs | 1 essence of water, 1 glowcap | 13 | 40 |
| murky draught, brewer's blunder | — | never a recipe: a failed brew's results |
| draught of chance, troll's blood | — | never: gamble potions, loot only |
| the six elixirs | — | never: a permanent stat point is loot's and the deep dungeon's, not a brew's (open question 11) |
| potion of restoration, quickness, blinking, glamour, seeing, the quick, the unseen | — | never yet: the deeper draughts wait for a second alchemy pass (more herbs, a creature's part as an input) |
| holy water | — | never: a priest's blessing, not alchemy (Orison's, if anything, later) |
| flask of oil, flask of tar, flask of darkness, flask of enfeeblement, flask of sloth, vial of acid, vial of nightmares, vial of silence | — | never yet: oil and tar are bought (the smithy's and the general store's), the hexing flasks are a witch's draughts for a later pass |

Why these inputs: each herb has two or three uses and each common potion a cheap recipe, so the first hour of a herbalist's dungeon brews healing and mana; the boosts and the battle flasks need the deep herb or an elemental's death. Numbers are a first pass.

### Scrolls, wands, rods

| Base | Route | Why |
|---|---|---|
| every scroll of a spell, scroll of identify | — | **Scribe**, the wizard's class active (`professions.md` §8; interim 12), on a blank scroll: not a profession's recipe |
| blank scroll, sealed letter | — | never: paper has no profession; the letter is a quest's |
| the nine wands, the four rods | — | never here: Enchanting's craft is a design of its own (`professions.md` §9) |

### Gems, coins, miscellaneous

| Base | Route | Why |
|---|---|---|
| gem (the loose gem) | J | never *made*: Cut changes one (§5) |
| copper coin, gold coin | — | never: coins are money; a gold coin is not gold stock (no minting) |
| piece | — | the stock itself |
| lute | W | main `stock` (wood), `inputs` 1 sinew (the strings); 8 billets, 1 length; difficulty 14, 7200 s, from 40: a bard's own instrument |
| pipe | W | main `stock` (wood); 1 billet; difficulty 7, 600 s, from 0 |
| rock | — | never: picked up off the ground (a boulder's rubble) |
| boulder, bones, torn cloth | — | never: debris; bones are salvaged for bone (1500 g → 5 shards), torn cloth is nothing |
| silver chalice, minster reliquary | — | never: quest objects, protected (`World::item_protected`) |

## 7. Intermediates

| Intermediate | Made by | Used by | Why it exists |
|---|---|---|---|
| **arrowhead** (`objects/arrowhead.toml`, `professions.md` §10: class `materials`, 5 g, `made_of` metal, bone, stone, its material its part; 20 a batch from one ingot) | Metalcraft, at an anvil | arrows and bolts of the metal route | the owner's example; a smith makes heads by the score and a fletcher uses them by the dozen: a count that no by-weight rule gets right (one ingot would be a hundred heads). Bone and flint heads need **no** intermediate: a shard or block heads a batch straight (the owner's comment) |
| **the cut gem** (a loose gem, cut) | Cut (§5) | Set, the forge's `ornament`, the gold amulet's setting | it is an existing item, not a new object; listed as an intermediate because recipes ask for it |

Nothing else. Rejected: a bowstave (bow from billets straight), a haft (a part of the spear's recipe), a hilt, rivets and studs (an ingot's worth as an input), tanned versus raw hide (one hide), a bowstring (sinew is the stock), an empty flask (free), a sharpened stake (the brittle arrow), feathers for fletching (no object, no step: the knife and the wood are enough, as today), charcoal or fuel for the forge (the anvil is the station; time is the cost).

## 8. Herbs (#1340): the list, the depth weighting, Gather

**Decision: nine herb objects, one gather table shaped like the gems', one action, no depletion.**

| Herb (object) | Name as carried | Where | `find` / `deeper` / `level` | Alchemy uses |
|---|---|---|---|---|
| `glowcap` | glowcap, glowcaps (the fungus's own cap) | every depth | 40 / 0 / 1 | mana, sustenance, antidote, owl's eyes, frost, thunder, sea legs |
| `moonwort` | sprig of moonwort | shallow and down | 30 / −1 / 1 | healing, greater healing, full healing, vigour |
| `comfrey` | sprig of comfrey | shallow and down | 20 / 0 / 1 | greater and full healing, cure disease, barkskin, might, renewal |
| `feverfew` | sprig of feverfew | from depth 3 | 10 / 2 / 3 | antidote, cure disease, proofing, will |
| `henbane` | sprig of henbane | from depth 3 | 10 / 2 / 3 | sleeping draught, stinking flask, moth's eyes, smoke |
| `eyebright` | sprig of eyebright | from depth 4 | 6 / 2 / 4 | the three eyes, grace |
| `bog_myrtle` | sprig of bog myrtle | from depth 4 | 6 / 2 / 4 | barkskin, stinking flask, warmth, frost, earthing, proofing, presence |
| `vervain` | sprig of vervain | from depth 4 | 6 / 2 / 4 | clear head, wit |
| `mandrake` | mandrake root | from depth 6 | 2 / 2 / 6 | full healing, the six boosts and their greater kin, renewal |

| Piece | Design |
|---|---|
| **The objects** | `objects/<herb>.toml`, class `materials`, `materials = ["plant"]`, 20 g, stackable, glyph `"` (the fungus's own, so a sprig on the floor reads as a plant, not as food's `%`), `fg` each its own green or grey, `value` 5 (glowcap, moonwort), 8 (comfrey), 15 (feverfew, henbane), 25 (eyebright, bog myrtle, vervain), 60 (mandrake). No `food` (henbane eaten is a poisoning the game need not model), no `level`/`frequency` (never floor loot). The alchemist's `always` adds glowcap and moonwort; it `buys` `materials` |
| **The table** | `data/herbs.toml`: `seconds = 300`, `count = "1d2"`, `value_per_count = 50` (one more per 50 of Herbalism), `untrained = 1` (the untrained's count), and `[[herb]]` rows `{ object, find, deeper, level }` read as `gems.toml`'s sizes and cuts are (`find` the weight at item level 1, `deeper` added per level past the first, never below 0, none below `level`). The level's item level is the depth (what loot and gems use), so the cave floors of Fyrdelf 8 give mandrake one draw in ten and Cealdhlāw 1 none |
| **Gather** | `Ability::Gather`, the *Here* group (as Pray and Drink: the thing beside the player), had by everyone (anyone may pull a mushroom; Herbalism makes it worth doing), `Aim::Direction` when fungus stands on more than one neighbouring cell, else the one (#1327's adjacent-target rule). `Action::Gather { dir }`, text `gather:DIR`, `gather = 1000` in `actions.toml`, five minutes by `keep_at` (interrupted: nothing, the time spent). Done: `count` draws from the table, each one gameplay draw by weight at the level's item level (`1d2` + Herbalism ÷ 50; untrained 1), into the pack: "You gather two sprigs of moonwort and a glowcap." Marks: `Use::Gather` on Herbalism, 100 %. Refused, no time: no fungus beside ("There is nothing here to gather.") |
| **The fungus** | stays as it is: lit, glowing, never picked over. #1340's default, and the rule "time is the only cost". A picked state would be saved state, a light change under light stability, and a regrowth clock, for a lock-out the house rules reject. Open question 9 |
| **Where fungus is** | the caves generator's patches (`mapgen/caves.rs`) and the hand-made levels that place `fungus`; the rooms generator places none. The first pass leaves that: cave floors are the herbalist's ground, as forests are the forager's. Open question 10 asks whether rooms levels should get a patch |
| **The surface** | the same nine objects go into `forage.toml`'s `herbs` lists (`professions.md` §9, P1): moonwort and comfrey on grassland and in forests, feverfew and vervain in forests, henbane and bog myrtle in swamps, eyebright on hills, glowcap and mandrake never (the dungeon's); angelica root stays a find. So a druid forages herbs and a delver gathers them, by one herb list |
| **Herbalism's description** | "Knowing the plants: what heals, what harms, and where it grows." stands; `kind` stays `passive` with Gather its `use` (a passive skill with a use, as Survival has Forage) |
| Determinism | the draws from the gameplay stream in a fixed order (count, then each herb); marks from the training stream |

## 9. Material drops by creature tag (#1341)

**Decision: dropped on death, data by tag, one file per tag, a stream of its own; no butchering.** When a creature dies, after its carried gear falls and `loot_of` rolls (`world/loot.rs`), `World::drops_of(actor)` rolls its material drops from `Rng::stream(seed, "materials/{id}")` (a new stream: the loot and gem streams never move, so replays and maps stand) and lays them on its cell. Why drops and not a skinning action: an action is a menu row, a time, a check, and a corpse state for the same hide, and the brief's default is the drop; a Survival "butcher" line can come later on the same table (`harvest = true` rows).

| Rule | Decision |
|---|---|
| Data | `data/drops/<tag>.toml`, one file per tag (the churning-table rule): `bodiless = false` (default), `[[drop]]` rows `{ material = "bone", count = "1d2", chance = 50 }` (stock of that material, `count` pieces) or `{ object = "essence_of_fire", count = "1", chance = 60 }` (an object, `count` of it). A creature's own `[[creature.drops]]` rows (the same shape) add to its tags'; `drops_from_tags = false` on a creature switches its tags' rows off (a tame hound drops no hide? no: a dead one does; the switch is for oddities such as the training dummy, which never dies anyway) |
| Which rows apply | **every** tag's rows (a union, not the first tag: gear condition's "first listed" is for one roll, this is many), in `data/drops/` path order, then the creature's own; each row its own chance draw and dice |
| Bodies | a row applies only to a creature that leaves a corpse (`corpse` true) unless its file says `bodiless = true` (the skeleton's bone, an elemental's essence, the mud golem's stone). So a spirit (undead, no corpse) drops no bone from the undead row |
| Size | a `small` creature rolls each row at half its chance (rats and ravens are little hide); medium at the chance. One rule, from `sizes.toml` |
| Stacking | pieces of one material from several rows join in one stack (`Stack::same_goods`) |
| Cap | none: counts are small dice; a hill giant's `1d3` bone is three shards |
| Who | the player's kills and everyone else's alike (a drop is the body's, not a reward); the arena's dead litter bone, as they litter gear |
| Wiki | `creature-drops.md`: a generated table of every creature's resolved rows (tag rows plus its own, bodiless and size applied) with exact chances and dice, under Creatures; and the per-tag table |

The tags and their rows (a first pass; counts are dice, chances percent; `m` stock, `o` object):

| Tag file | `bodiless` | Rows |
|---|---|---|
| `beast` | no | m leather 1 (60); m bone 1d2 (50); m sinew 1 (40); the furred add their own (below) |
| `vermin` | no | m bone 1 (30) |
| `marsh-thing` | no | m bone 1 (40); m leather 1 (30) |
| `kobold` | no | m flint 1 (40); m wood 1 (25); m bone 1 (20) |
| `goblin` | no | m flint 1 (30); m iron 1 (20); m leather 1 (30) |
| `hobgoblin` | no | m iron 1 (40); m steel 1 (15); m leather 1 (30) (with `goblin`'s rows besides: a hobgoblin carries both tags) |
| `orc` | no | m iron 1 (40); m leather 1 (30); m bone 1 (20) |
| `outlaw` | no | m iron 1 (20); m leather 1 (30) |
| `duergar` | no | m iron 1d2 (50); m steel 1 (30) |
| `drow` | no | m steel 1 (30); m mithril 1 (3) |
| `troll` | no | m bone 1d2 (50); m hardened_leather 1 (40); m sinew 1 (30) |
| `giant` | no | m bone 1d3 (50); m leather 1d2 (50); m sinew 1 (40) |
| `ogre` | no | m bone 1d2 (40); m leather 1 (40) |
| `hag` | no | o henbane 1d2 (50); m bone 1 (30) |
| `undead` | no | m bone 1d2 (50) (the corpse-leaving dead: zombies, ghouls, draugr, vampires) |
| `skeleton` | **yes** | m bone 1d3 (80) |
| `spirit` | — | no file: nothing of a ghost |
| `construct` | yes | m stone 1d2 (60) (the mud golem; animated objects break into pieces already and carry the tag `animated`, which has no file) |
| `fire`, `ice`, `lightning`, `air`, `water`, `earth` (the elements) | yes | o essence of that element 1 (60); `earth` also m stone 1d2 (50) |
| `fire-thing` | no | o essence of fire 1 (25) (the salamander, the ember beetle, the magma crawler; the cinder elemental has `fire` too) |
| `dragon` | no | m dragonbone 1d2 (100); m dragonscale 1d2 (100); m sinew 1d2 (60) |
| `devil` | no | m horn 1 (30) |
| `demon`, `yugoloth`, `fiend`, `fey`, `gate`, `summoner`, `caster`, `archer`, `priest`, `cultist`, `eyeless` | — | no file: nothing bodily to take, or a role not a body |

Creatures' own rows (in their files): boar, great boar, deer: m horn 1 (50) (tusks and antlers); wolf, bear, lynx, panther, badger, polecat, hare: m fur 1 (50) in place of leather (their `drops_from_tags` stays on: a bear gives leather or fur or both); large spider, barrow spider, giant centipede, beetle, ember beetle: m chitin 1 (50); crocodile: m hardened_leather 1 (60); black pudding: nothing (eyeless, no file); the basilisk, cockatrice, gorgon, hydra, warg, dire warg, bat, deer are **untagged today** and get tags in the drops PR (`beast` for the warg, dire warg, deer, bat; the basilisk, cockatrice, gorgon, and hydra their own rows: m bone 1d2 (50), m hardened_leather 1 (40); the gorgon is `combat = "dragon"` and gets `dragon`'s rows as the README's rule says). Essences: `objects/essence_of_<element>.toml`, six, class `materials`, 50 g, value 30, stackable, `glyph = "*"` in the element's colour, named "essence of fire", "essences of fire".

**Fit to the solo game.** A beast's death gives a hide three times in five and a shard half the time: a day's hunting in Beorggeat's pass dresses a leather jerkin (3 hides) and heads a quiver. Flint comes with every kobold warren. Iron is scrappier (one ingot in five goblins): a dagger's worth from a warren, a sword's from a hobgoblin patrol, mail from salvage. Nothing needs farming; everything comes of what the player kills anyway.

## 10. The data format, the loader's checks, the wiki tables

`professions.md` §6's format stands; these fields are added. Everything stays one `[[recipe]]` a file under `data/recipes/<skill>/`, the profession the directory.

| Field | Meaning | Checked at load |
|---|---|---|
| `[recipe.parts]` entries may be a **table of sources**: `head = { input = "arrowhead", stock = ["bone", "stone"], optional = true }` | the wizard's one step for that part offers, in order, the stacks of the `input` object, then the stock whose material is in a listed group or is a listed material (groups and material ids both allowed: `["gold"]`, `["stone", "bone"]`), then **None** if `optional`. The part takes the material of what was chosen (an arrowhead's metal, a shard's bone); None leaves the part off and the result is the recipe's `brittle` object if it names one (`professions.md` §10's `[[makes]]` rule, moved onto the recipe: `brittle = "brittle_arrow"`) | each listed group or material exists; every listed material is on the result part's `made_of` (a recipe may narrow, never widen); `optional` needs `brittle` or a part the result can lack; `input` names an object the part's `made_of` admits by its own material |
| `parts.X = { stock = [...] }` of **another profession's group** | allowed: a spear's `haft = { stock = ["wood"] }` on a Metalcraft recipe. The check is the recipe's profession's; `work` is the **main chosen part's** material's (the head's), as §7 says | the groups exist |
| `parts.X = { gem = true }` | the step offers the cut loose gems carried, **required** (the gold amulet's setting); `ornament = true` at the recipe's level stays the optional gem for the result's ornament part | the part is of a gem material on the result |
| `inputs = [{ material = "sinew", count = 1 }]`, `[{ group = "metal", count = 1 }]` | fixed stock inputs: `count` pieces of that material, or of any one material of the group (the wizard asks which stack when more than one fits; a single fitting stack is still shown); used up as object inputs are; nothing on the result is made of them | the material or group exists; a `group` input never names a gem |
| `count = 10` on a stackable result, `count = "fletch"` (Fletch's rule), `per_made` on inputs | as `professions.md` §6 and §10 | `count` above 1 only on a stackable result |
| `failure = { object = "murky_draught" }`, `fumble = { object = "brewers_blunder" }` | what a failed check (and a natural 2) makes instead of nothing; the inputs are spent (no half back) when `failure` is set | the objects exist and are stackable |
| `use = "cut"`, `use = "set"` | Jewelcraft's built-ins, §5, beside `temper`, `whittle`, `fletch` | one of the five |
| `brittle = "brittle_arrow"` | the result when an `optional` part is None | the object exists; only with an `optional` part |
| `data/recipes.toml` | `fine = 6`, `marvellous = 12`, `masterwork = 20` (the forge's bands, §4), `cut_step = 5` (§5), `half_within`, `one_within` (the counted bands, from `fletch.toml`), `yield = 50` (a failure's return), `free_wood = [...]` (§10 of `professions.md`) | each band above the last |
| `data/stock.toml` | `[[group]]` rows gain `unit` (default 1); new `[[material]]` rows `{ material = "flint", skill = "woodcraft" }`, `{ material = "fur", name = "pelt", plural = "pelts" }` override a material's profession or form word | the material exists and has a group; the skill exists |
| `data/herbs.toml`, `data/drops/<tag>.toml` | §8, §9 | herbs: the objects exist and are stackable, weights above 0, `level` ≥ 1; drops: the material has a group (or is a gem? **no**: gems come by the gems' chance), the object exists, dice parse, chance 1 to 100, the tag is carried by at least one creature (a typo would silently drop nothing) |
| `[[creature.drops]]`, `drops_from_tags` | §9 | as drops |

**Wiki** (`xtask/src/wiki/crafting.rs`, one function per table, registered in `generate()` beside the skills' tables): `recipes-metalcraft.md`, `recipes-woodcraft.md`, `recipes-jewelcraft.md`, `recipes-alchemy.md` (per recipe: result, chosen parts with their groups, fixed inputs, stock counts for the base material, station, knife, time in words and seconds, listed from, difficulty, and the grade odds at three skill values, §4), `crafting-grades.md` (the bands and the words), `gem-cutting.md` (the cut bands, the size loss, the value multipliers: a table of what a large uncut ruby becomes at each outcome), `herbs.md` (every herb: where, the find weights per depth as percentages at depths 1, 3, 6, 10, its uses), `creature-drops.md` (§9). Pages: `wiki/src/crafting.md` (the professions design's page, extended: a section per profession linking its table), `wiki/src/herbs.md` under Objects beside Potions, `gems.md` gains "Cutting", `creatures.md` links the drops. `wiki/TERMS.md` gains: recipe, profession, stock, ingot, billet, shard, hide, pelt, sinew, arrowhead, Gather, herb, essence, Cut, Set, Masterwork (crafted), drop (material).

**Docs**: `docs/dev/crafting.md` (the professions design's new topic) gains the sections: routes and the recipe's profession, the grades and `attempts`, Cut and Set, herbs and Gather, drops; `docs/dev/creatures.md` one line on drops (the per-tag files; the roster files list their tribe's rows); `docs/dev/objects.md` (sinew, essences, herbs as `materials`-class objects); CLAUDE.md's action-syntax list (`gather:DIR`; `craft:` picks for a `None` as `-`); spec §7 "Objects and materials" (stock, herbs), §8 "Skills" (Gather, Cut, Set).

**Saves and replays**: new saved state none (a `Worked` with `attempts`, `Stack::parts`, new objects by id); content changes that move hashes: new objects (ids: nothing numbered is saved), `sinew` (a material index: materials are by id in saves), the new streams (none moves an old one), the tags added to untagged creatures (quests' `tag:` pools: the beast tag on the warg changes which kill quests count it; the drops PR notes which replays change).

## 11. Implementation issues

Each one PR, refined from the section named; the order below respects the dependencies. P1–P6 are `professions.md` §12's issues (#1331–#1336), some refined here; the new ones take the next free labels. "Needs" names the PRs that must have merged.

| # | Issue to file (or refine) | Builds | Sections | Needs | Replaces or extends |
|---|---|---|---|---|---|
| 1 | **P2b: stock extensions** — `sinew` material and group; `stock.toml` `unit` per group (hide 8, cloth 4) and `[[material]]` overrides (flint, stone, obsidian → Woodcraft; fur → pelt); `count_for` with `unit`; the nine herb objects, the six essence objects, class `materials`; the alchemist buys `materials` and `always` stocks glowcap and moonwort; `docs/dev/objects.md`; the wiki's materials table (profession per material, form word, unit) | §3, §8 (the objects), §9 (the essences) | §3 | #1332 (P2) | extends #1332: file as its own issue after P2 merges, so P2 stays the design's size |
| 2 | **P4b: the recipe format's second half** — part sources (`input` + `stock` + `optional`, `brittle`), `gem = true` parts, `inputs` of a material or group, `failure`/`fumble`, `use = "cut"`/`"set"` parsed (rows listed, "not usable yet" until P10), `data/recipes.toml` with the bands; loader checks and their tests; `docs/dev/crafting.md` | §10 | §10 | #1334 (P4) | extends #1334; with no recipe using the fields yet, the PR is the loader and its refusals |
| 3 | **P5 refined (#1335)** — `professions.md` §6's seven swords and §7's check, with §4's bands: Masterwork at 20, `attempts` = improvements, Temper's refusal of a crafted Masterwork, the words; the `forge` scenario's ingots | §4 | §4 | P4b | refines #1335 (the issue says "per the design"): add this document's §4 to its body |
| 4 | **P15: material drops by tag (#1341)** — `data/drops/<tag>.toml` for the tags of §9, `[[creature.drops]]` and `drops_from_tags`, `World::drops_of` on the `materials/{id}` stream after `loot_of`, the bodiless and size rules, the tags added to the untagged beasts, the creatures' own rows, `creature-drops.md`; `docs/dev/creatures.md` and the roster files | §9 | §9 | P2b | refines #1341; runs beside 2 and 3 |
| 5 | **P6 refined (#1336)** — arrowheads (as `professions.md` §10) and arrows/bolts with the head step `input` + `stock` (bone, stone) + None, sling stones (`count = 10`), flaming arrows (arrows + oil), the brittle rule, `fletch` retired | §6 Missiles | §6 | P5, P4b | refines #1336: the bone and flint heads, sling stones, flaming arrows added to its body |
| 6 | **P5b: the hand-made weapons** — Woodcraft's knife, dagger, hand axe, club, quarterstaff, ironwood staff, spear, javelin (bone, flint, stone, wood; a knife; the spear's haft a second group's stock); Metalcraft's spear and javelin (the haft rule), axes, maces, polearms, trident; the wiki's two tables | §6 Melee weapons | §6 | P5, P4b | new |
| 7 | **P7: bows, crossbows, the sling, the lute, the pipe** — Woodcraft with sinew and an iron input; the wiki | §6 Missile weapons, Miscellaneous | §6 | P5b, P2b | new |
| 8 | **P8: hide gear by hand** — leather jerkin, studded leather, leather cap, gloves, boots, cloaks, the hide and wood clothing, the wooden and hide shields and helmets (Woodcraft) | §6 Shields, Body armour, Helmets, Cloaks, Clothing | §6 | P5b, P2b | new |
| 9 | **P9: armoursmithing** — chain, scale, banded, plate, the metal shields and helmets, gauntlets, iron-shod boots, the lantern, pick-axe and mattock (Metalcraft; the stone pick by Woodcraft); the wiki | §6 | §6 | P5b | new |
| 10 | **P10: Cut** — `use = "cut"`, `ObjectList::Cut`, `world/cut.rs` (the bands, the size loss, re-cutting, the shatter), Jewelcraft's profession row appears; `gem-cutting.md` | §5 | §5 | P4b, P1 (#1331) | new (the professions design's "Cut later") |
| 11 | **P11: Set and jewellery** — `use = "set"` (two picks, the chipped gem), the rings and amulets (Jewelcraft; the bone amulet Woodcraft's), the gold amulet's required gem | §5, §6 Rings and amulets | §5, §6 | P10, P5 | new |
| 12 | **P12: Gather (#1340)** — `data/herbs.toml`, `Ability::Gather`, `Action::Gather`, `world/gather.rs`, `Use::Gather`; the herbs into `forage.toml`'s `herbs` lists; `herbs.md` | §8 | §8 | P2b, P1 | refines #1340 |
| 13 | **P13: brewing** — Alchemy's profession row and the herb recipes of §6 (Intelligence, no station, `failure`/`fumble`), a potion identified when made; `recipes-alchemy.md` | §6 Potions | §6 | P12, P4b, P5 | new (the professions design's "Alchemy's brewing later") |
| 14 | **P14: essences and the battle flasks** — the essence recipes of §6 (fire, frost, thunder, smoke, pitch fire, blasting flask, frost and earthing and sea legs) | §6 Potions | §6 | P13, P15 | new |

Later, unfiled, as `professions.md` §12 lists: Mining's ore and Smelt, Enchanting's extraction and binding (wands and rods), tailoring (cloth stock: the padded coat, the wool cloak, the clothing), a dragonbone carving line, the deeper draughts and the hexing flasks, a Survival butchering line on the drops table, a second herb list for the surface, materials in shops beyond a few.

## 12. Open questions, each with the integrator's default

1. **Masterwork from the forge** at a margin of 20 (a master one time in four, a journeyman never), `attempts` set to the improvements. Default: **yes** (the brief asks for the whole ladder).
2. **Flint, stone, and obsidian to Woodcraft** (knapped), jade to Jewelcraft, by `[[material]]` overrides in `stock.toml`. Default: **yes**.
3. **Sinew** as a new material and one-member group, an input of bows, the composite bow (two), the lute; never a part. Default: **yes**.
4. **Herbs and essences as objects** (class `materials`), not materials of a group. Default: **objects**.
5. **No flask input for brewing**: glass is free as a leather grip is; drinking leaves nothing. Default: **free**.
6. **A failed brew leaves a murky draught, a natural 2 a brewer's blunder**, the herbs spent (against interim 5's half back, for brews only). Default: **yes**.
7. **Cutting's size loss**: a crude result and a failure lose a size, a re-cut always does, tiny is never re-cut, flawless never cut, a natural 2 shatters; a re-cut never worsens the cut. Default: **yes**.
8. **Drops on death, no butchering**; every tag's rows apply; small creatures at half chance; `bodiless` rows for skeletons, elementals, the golem. Default: **drops**.
9. **The fungus never depletes**; the time per gather is the only limit. Default: **never depletes**.
10. **Fungus only where the caves generator and hand-made levels put it**; the rooms generator untouched (herbs on the surface come by forage). Default: **untouched** (a patch per rooms level is a one-line later issue if cave floors prove too few).
11. **Never brewed**: the elixirs, restoration, quickness, the unseen, blinking, glamour, seeing, the quick, holy water, the hexing flasks, oil and tar. Default: **never, for now**; a second alchemy pass may add the deeper draughts with new inputs.
12. **Jewellery's stock**: a 10 g ring costs a whole ingot (`count_for`'s minimum). Default: **accept** (gold is rare; the ring is worth more than the ingot).
13. **Hide parts on metal items** (the long sword's grip, the iron-shod boots' leather) stay free although hide now has a profession. Default: **free** (a scrap; charging a hide for a grip is a step for nothing).
14. **Wood is free only for arrows** on wooded ground; a shield in a forest still needs billets. Default: **arrows only**.
15. **Mail by weight**: chain 22 ingots and two hours, plate 44 and four; salvage returns half. Default: **accept** (the rule is one; a smith's mail is a campaign, as it should be; a `unit` for the metal group would cheapen every ingot).
16. **Untrained gathering** gives one sprig, trained `1d2` + Herbalism ÷ 50. Default: **yes**.
17. **Dragons drop dragonbone and dragonscale** at 100 %, `1d2` each, the only source besides salvage and deep loot. Default: **yes**.
18. **Tags for the untagged beasts** (warg, dire warg, deer, bat: `beast`; the basilisk, cockatrice, gorgon, hydra: own rows), changing which kill quests count them. Default: **tag them** in the drops PR, noting the replays that move.
19. **Household goods are never crafted** (salvage only). Default: **never**.
20. **Clothing of hide and wood is crafted**, worth little; cloth waits for tailoring. Default: **crafted** (the rows cost nothing but files, and a field-made pair of shoes fits the owner's "serviceable gear in the field").

## 13. Choices made along the way

| Choice | Taken | Rejected, and why |
|---|---|---|
| The route's profession | the recipe's directory, whatever the material's group | the material's profession (a flint knife by Jewelcraft; the owner says Woodcraft); a `hand = true` flag on a Metalcraft recipe (two professions in one file) |
| Flint and stone | material overrides to Woodcraft, jade kept | moving the whole stone group (jade is cut and set); a new `knapped` group (flint already has a group, and `made_of` lists say `stone`) |
| Hide's unit | `unit = 8` on the group | a hide object of its own (`professions.md` §3 rejected form objects once); leaving 175 g hides (twenty to a jerkin) |
| Herbs | objects, class `materials` | materials of a `herb` group (jewellery's `any`, form words per herb, a material that is never a part); food objects (henbane as a snack) |
| Essences | six objects | one `essence` object with the element as a part (elements are not materials); nothing (the battle flasks would have no source) |
| Masterwork | at 20 | never at the forge (`professions.md`'s first pass; the brief wants the ladder); at 18 (a master makes one in three) |
| The shared count | `attempts` = improvements | `attempts` 0 (the anvil finishes the forge's work at base difficulty); a separate "crafted" flag (two ladders) |
| Cut | one check sets the grade; a crude result loses a size | one grade per cut, five cuts to flawless (five half-hours and five checks for one stone: grinding); no size loss (the brief asks for it) |
| Re-cut | always a size smaller, never worse | refused (a poor cut is forever); a free retry (grinding) |
| Set's failure | the gem a cut rougher, a natural 2 shatters | the item damaged (the gem is the work); the gem lost (interim 4's spirit says a cut or a size) |
| Brittle heads | the head step offers heads, shards or blocks, or None | an intermediate bone arrowhead (the owner: none needed); flint heads as a separate recipe (one recipe, three sources) |
| A bow's string | sinew as a fixed input | a bowstring object (an intermediate for nothing); no string (then sinew has no use) |
| A crossbow's lock | one ingot of any metal as an input | a `lock` part (the object has none; a part for one recipe) |
| A failed brew | a murky draught; a brewer's blunder on a 2 | half the herbs back (the alchemist's failure should be a bad bottle; the two potions exist for it) |
| Drops | on death, by tag, every tag's rows | the first tag's rows (gear condition's rule is for one roll); a butchering action (a row, a time, a check, a corpse state for the same hide); drops only from the player's kills (a body is a body) |
| The drops' stream | `materials/{id}`, new | the `drop/{id}` loot stream (every existing drop would move) |
| Spirits' bone | the corpse rule (`bodiless` for skeletons and elementals) | `drops = "none"` on twenty-two spirit files; a `spirit` file that cancels undead's (a cancel rule) |
| Gather | a *Here* ability for everyone, Herbalism's count | trained only (the brief's default is "yes, poorly"); walking onto fungus (a wall-in dialog for every cave floor) |
| The fungus | untouched | a picked state (saved state, a light change, a regrowth clock, a lock-out) |
| Household goods | never crafted | recipes for teapots (worth less than their billets) |
| Scrolls | Scribe stays | a scribing recipe (interim 12) |
| Rings' metal | one recipe per ring object, its metal fixed | one ring recipe with the metal chosen (`value_with` prices an iron ring of gold as 33 iron rings) |
