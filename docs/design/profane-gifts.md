# Warlock — Profane gifts: the layer bosses' corpses and the gift table

Design document (2026-11-13) for #1471, **approved by the owner on 2026-11-14 with the recommendations**: every question of the review took the recommended answer, recorded in §14 and folded into the rules below. Written against `origin/main` at 1559218c6 (version 0.16.0), finalised against 7095e8bdb (0.17.0). Nothing here is implemented; §13 maps it onto implementation issues, one PR each. Resistance follows #1620's model (§6.6), and the phases that grant resistance come after it. It builds on what is there: eating and corpses (`docs/dev/hunger.md`: `OnEat`, `eat.rs::eat_fresh`, `corpse_gifts`, the rot stages), alignment (`docs/dev/alignment.md`, `data/alignment.toml`), stat training and gifts (`docs/dev/stats.md`: `Stats::gift`, the marks), the talent machinery (`docs/dev/progression.md`: `TalentEffect`, `TalentMods`, `granted` talents, `Actor::boons`, `World::bestow`), the layer bosses (`world/layer_boss.rs`, spec §7 "The endless branch"), and the ten boss files in `assets/campaigns/ascmearc/data/creatures/`.

Every number below is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate). The aim: **eating a layer boss is the biggest gamble in the game**: a sure, large, fitting change from the boss itself, one to three draws on a table of permanent changes to the body, good, bad, and mixed, and a heavy pull toward evil, all for life. The corpses never rot, so the meal can wait; nothing else about them waits for anything.

## Contents

1. Summary
2. The owner's brief, and what exists today
3. The corpses: every boss leaves one, and it never rots
4. The meal: what eating a boss corpse does, in order
5. The signature gifts: each corpse's own effect, old → new
6. The profane gift table
7. The draw: rolls, tiers, weights, duplicates, relations
8. Losing a gift, curing, and the room left for the good use
9. Alignment: the pull toward evil
10. Races, classes, equipment slots, statuses
11. What the player sees; the dumps; saves; determinism
12. Balance, by the depth each boss sits at
13. Implementation plan: the phases
14. Decisions: the owner's review
15. Choices made along the way

## 1. Summary

| Piece | Rule |
|---|---|
| The corpses | All ten layer bosses leave a corpse (`corpse = true`; Wēalāf's and Nihtscinna's `corpse = false` go), and every boss corpse is **everlasting**: a new creature field `everlasting = true` (uniques only) makes its corpse object's `Food` keep and last forever (`keeps`/`lasts` become `Option<Duration>`, `None` = never; `rot::stage` says Fresh for ever, `rot_away` leaves it, a trained eye judges it fresh). An everlasting creature always leaves its corpse: a crystal spell does not turn it to glass, and a petrifying death does not make it a statue. Where it fell is where it is eaten: a corpse weighs what the boss weighed (Nīðdraca 6,000 kg), and that stays so. §3 |
| The meal | Eating a boss corpse, in order: its words; its **signature gift**, sure (`on_eat.gives = "<id>"`), with its stat gifts and skills; then **N draws** on the profane table (`on_eat.profane = { rolls, tiers }`), each a tier drawn by the boss's depth, then a gift by weight among those the eater may still take; then the alignment deed `eat_profane` (−300, diminishing as every deed does); then it feeds. No vomiting: the flesh always takes. §4, §5, §7, §9 |
| Gifts are talents | A profane gift is a `granted` talent of a new tree, `profane`, one file each in `data/profane/<id>.toml`, read into the talent book (so `TalentEffect`, `TalentMods`, the Talents page, the `talents` dump, and the restore all work unchanged) and into a **gift table** beside it with the draw data: `kind` (boon, mixed, bane), `tier` (lesser, greater, dire), `weight`, `only` (a signature's boss), the relations `excludes`, `replaces`, `undoes`, `not_for` (races), and the line said when it takes. Given through `World::bestow` into `Actor::boons`, as an altar's Chosen is; marked `(profane)` / `(profane, bane)`. §6, §11 |
| The table | **50 gifts**: 10 signatures (one per boss, never drawn) and **40 drawn**: 14 lesser, 12 greater, 14 dire; by weight 45 % boons, 27 % mixed, 27 % banes in the lesser tier, 45/28/26 in the greater, 37/25/38 in the dire. Effects are existing `TalentEffect`s wherever one fits (stats, resistances, natural armour, speed, hunger, healing, darkvision, scent, clotting, free actives) plus six new kinds: `kin_to`, `bars_slot`, `opens_slot`, `unarmed_on_hit`, `vision`, `max_mana_percent`. A gift's `resist` adds to the player's resistance total, which has no cap; only 75 % of it is effective (#1620). No gift makes the player immune. §6 |
| The draw | One gameplay-stream draw of the tier (the boss's weights), one weighted draw among the tier's **eligible** gifts (not held, not excluded by a held gift, not barred to the race); an empty tier falls to the next (up, then down); an empty table says so and gives nothing. **No cap**: the table's size is the cap (20 draws at most in a run that eats all ten, against 40 drawn gifts). §7 |
| Relations | `excludes` (symmetric; never drawn beside the other), `replaces` (the new takes the old away and takes: the greater form of a lesser), `undoes` (the new takes the old away and is **not** taken itself: a bane that only cancels the boon it opposes, "the withering takes your oak heart, and nothing more"). Taking a gift away is `World::rescind`: its `stat` handouts reversed by `Stats::gift` of the negative, hit points and mana kept within their maxima, everything else read at hooks and so simply gone. §7, §8 |
| Cures | None in this design. A gift leaves only by `replaces` or `undoes`. `World::rescind` is the one hook the deferred **good-aligned use** of these corpses (#196) and any later cleansing would call. §8 |
| Alignment | A new `[shift]` deed, `eat_profane = -300`: the largest single sin in the game (`eat_kin` is −200). Three corpses take a saint to Neutral, five to Evil, ten to Wicked. A paladin's zeal empties (`sin_empties_zeal`), a priest's holy actives shut as the bands fall: that is the cost for the good. §9 |
| Seen | Each gift's line, then "You are Horned." in the gift's name; the Talents page lists them under **Profane** with their descriptions; the equipment screen shows a barred slot `— (horns)`; the character screen's resistances in words as now. Dev: `talents` `(profane, bane)`, `progress` `profane 3 held; pool lesser 9/14, greater 12/12, dire 14/14`. The wiki states every gift's exact effect, odds, and relations (`profane-gifts.md`, generated). §11 |
| Phases | P1 everlasting corpses for all ten; P2 the gift data, the tree, `bestow`/`rescind`, the dev `gift:`/`rescind:` actions, the dumps, five gifts on existing effects; P3 the draw on eating, the alignment deed, Gnægan's meal, the wiki page and tables (after #1620); P4 the other nine signatures and the gift actives' source (after #1620); P5 the body effects (`bars_slot`, `opens_slot`, `unarmed_on_hit`, `vision`, `kin_to`, `max_mana_percent`) with the gifts that use them; P6 the rest of the table, Twice-born, the full-table checks (after #1620). §13 |

## 2. The owner's brief, and what exists today

The brief (#1471, 2026-11-13): (1) **every** layer boss of Grundlēas leaves a corpse; (2) eating one has **very large upsides and is very much a gamble**: each corpse keeps or gets its own effect, redesigned to be big and fitting its fantasy, and in addition **rolls on a profane gift table**; (3) the table is a set of **permanent changes to the eater's body and nature**, good, bad, and mixed, in the spirit of roguelike mutation systems but **original to Warlock**: what each does, how they are drawn (weights, tiers by depth, duplicates, conflicts and replacements, a cap or none), how they show, whether any can be lost or cured, and how they interact with races, classes, slots, and statuses; (4) eating any of them is a **big pull toward evil**; (5) **they never rot**; (6) deferred: a good-aligned alternative use, left room for and not designed (#196).

| Part | Today |
|---|---|
| The bosses | Ten `unique` creatures of faction `layer_keepers`, placed by their lairs at depths 10 to 100 of Grundlēas (`[[dungeon.endless.boss]]`, `grundleas.toml`); their death opens the hidden `>`; the floor and everything on it is forgotten when left (spec §7 "The endless branch") |
| Their corpses | Gnægan: `on_eat = { message, vomit = 90, harm = "1d6" }`. Āttorcoppe: `vomit = 80, harm = "2d4"`. Hrīmwyrm: `vomit = 20, train = { strength = 150, toughness = 150 }`. **Wēalāf: `corpse = false`.** Grundwyrgen: `vomit = 70, harm = "2d6"`. Līgetdrȳ: `vomit = 40`, nothing else. Stānēoten: `vomit = 30, train = { strength = 200 }`. **Nihtscinna: `corpse = false`.** Wælcyrge: `vomit = 50, train = { dexterity = 200 }`. Nīðdraca: `vomit = 20, train = { strength = 200, toughness = 200 }`. Four harm or do nothing, two leave nothing, four give marks a dozen ordinary giants would |
| Eating | `world/eat.rs`: `Action::Eat` → `dine` (turns by the corpse's size: medium 6; interrupted like a rest, the food untouched until the last turn) → `finish`: rotten → `eat_rotten`; fresh corpse → `eat_fresh`: `message`, `vomit`, `corpse_gifts` (`train` marks into `Actor::stat_marks`, `widen` draws), `corpse_skills`, `heal`, `harm`, then `feed`. `OnEat` fields: `message`, `vomit`, `train`, `widen`, `heal`, `harm`, `satiation`, `skills` |
| Rot | `Food { nutrition, turns, keeps, lasts }`, corpses by size (`[corpses.size]` medium: keeps 8h, lasts 36h); `rot::stage(age)` Fresh/Old/Rotten/Gone from the age alone; `World::rot_pack` and `rot_away` drop gone food; a ratfolk eats rotten food for the feed alone |
| Alignment | `data/alignment.toml` `[shift]`: `eat_kin = -200` the largest sin, `kill_peaceful` −150/−100, `theft` −100; the diminishing rule in `Score::shifted`; `World::shift_alignment` the one place a score moves; `sin_empties_zeal` on any fall; the bands gate only holy actives and band-gated passives |
| Talents given for life | `Actor::boons: Vec<Taken>` (saved, hashed): an altar's Chosen and Forsaken (`World::bestow_talent`, `bestow_flaw`, `world/boons.rs`) and a story's `talent`/`flaw`; put back on restore by `stars::grant` through `Talents::regrant`; `OwnTalent::boon`/`story` mark them `(altar)`, `(story, flaw)` in the `talents` dump and "altar"/"story" on the Talents page. The birth signs' gifts are `granted` talents of the `stars` tree; a flaw is a talent with `flaw = true` |
| Effects | `TalentEffect` (`content/talent.rs`), about 180 kinds, each read at one hook (`TalentMods`), including everything a body might change: `stat`, `natural`, `dodge`, `resist`, `speed_percent`, `hunger_percent`, `healing_percent`, `darkvision`, `threshold_percent`, `scent`, `clotting`, `wound_more`, `eats_rotten`, `footsteps_percent`, `carry_percent`, `unarmed_die`, `kick_die`, `swimmer`, `plunge_proof`, `inner_light`, `initiative_bonus`, the sign actives `fire_breath`, `stare`, `hounds_bay`, `rise_from_ashes` |
| Slots | `data/slots.toml`: 16 slots; the aristocrat's three extra ring slots are gated by `class = "aristocrat"` on the slot; a race's `bare` list only shapes its starting kit (halflings' feet) |
| Precedents in Warlock | The forgotten altar's great outcomes (random for-life boons and banes), the birth sign's gifts and flaws, the stat gifts of elixirs and corpses, the miscast scars (Ashen, Worn thin: statuses that ease in a day). None is a permanent random change to the body with a table of its own |

## 3. The corpses: every boss leaves one, and it never rots

**Decision: all ten leave a corpse, and a new creature field `everlasting = true` makes it keep forever.**

| Rule | What it is |
|---|---|
| Who leaves one | Wēalāf's and Nihtscinna's `corpse = false` are removed. The corpse object is made at load as every corpse is (`<creature>_corpse`, `Content::load_hunger`), feeding by size: all ten are medium (no `size` set), so 1,000 nutrition over 6 turns of eating. Nihtscinna's weighs what it weighed, 100 g, a husk; the words in its file say so |
| `everlasting` | A creature field, default false, refused with `corpse = false` (a content error) and refused on any creature that is not `unique`: only the once-a-run leave flesh that keeps. The corpse object's `Food` gets `keeps: None, lasts: None` |
| Always a corpse | An everlasting creature's death always lays its corpse. `World::crystallise` (`world/crystal.rs`, the crystal spells of `docs/dev/enchanting.md`) skips it: no glass, no crystals, the corpse as usual. A death by a petrifying status lays the corpse too, not a statue (`fight.rs::creature_dies`: `stoned` is false for it). The brief says every layer boss leaves a corpse, and a spell chosen for loot must not silently cost the meal |
| `Food::keeps`/`lasts` | `Option<Duration>`. `rot::stage` returns `Fresh` when `keeps` is `None`; `gone_at` returns `None` when `lasts` is; `World::rot_pack`, `rot_away`, and `rot::until_rots` skip it. An object file may not set `keeps` or `lasts` to "never": the option is the loader's, from `everlasting` alone |
| What is told | Nothing names the rule. The dev `actors` layer prints the stage `fresh`; `--state` the raw `died`. A trained eye (`freshness::judged`) says `fresh`, truthfully. Shops: not spoilt |
| Where it is eaten | A corpse weighs what the boss weighed (the rule for every corpse): Gnægan 60 kg, Āttorcoppe 300 kg, Hrīmwyrm 2,500 kg, Nīðdraca 6,000 kg. Nine of ten cannot be carried by anyone, so the meal is eaten in the lair, after the fight, whenever the player chooses: the floor stays while they stay on it, and they may rest for days first (that is what never rotting is for). Leaving the floor forgets it with the corpse (spec §7). The deferred good use must reckon with the same: it will be a rite at the corpse, not an item carried out |
| The wiki | `food.md` "Corpses": one paragraph, "The corpses of the keepers of Grundlēas never rot."; the generated corpses table marks them *everlasting* in its shelf-life column |

Why a field and not `keeps = "never"` on food: the ten corpses are made from their creatures, not from object files, and the rule belongs to the creature; an object with a shelf life of never would also need a loader rule for every other food system (shops, foraging) that this design has no reason to open.

## 4. The meal: what eating a boss corpse does, in order

`eat.rs::eat_fresh`, for a corpse whose creature has `on_eat.gives` or `on_eat.profane` (a **profane corpse**; the loader requires both, and `everlasting`, on the same creature):

| Step | What happens | Said |
|---|---|---|
| 1 | The corpse's `message` | "It is rats, many rats, and none of them fresh." |
| 2 | No `vomit` (the loader refuses `vomit` on a profane corpse: the flesh always takes). `train`, `widen`, `skills`, `heal`, `harm` as any corpse's (Līgetdrȳ keeps a caster's Decipher script) | As today |
| 3 | **The signature gift** (`gives`): `World::bestow_gift(id)`: taken into `Actor::talents` and `Actor::boons` with what it hands out (`class::hand_out` for `stat`), its relations applied (§7) | The gift's line, then "You are Rime-blooded." |
| 4 | **The draws** (`profane.rolls` times, §7): tier, then gift, then `bestow_gift` | Each gift's line, then "You are Horned." / for an `undoes`: the undone gift's `undone` line |
| 5 | **Alignment**: `player_did(Deed::EatProfane)` → `shift_alignment(−300)` | The band's `fall` line as a note, if a band is crossed |
| 6 | `feed` (1,000 nutrition, overeating as ever) | Hunger and burden news |

The whole is one `finish`: an interrupted meal (the 6 turns stop like a rest) has done nothing yet and the corpse lies untouched, as today. There is no confirmation before the meal (§14, D2): the eat command eats it as it eats any corpse.

## 5. The signature gifts: each corpse's own effect, old → new

Each boss's `on_eat` names one gift of its own, **sure**, `only = "<boss>"` on the gift (weight 0 in every draw; the loader checks a gift with `only` is named by exactly that creature's `gives`, and a `gives` names a gift with its `only`). Where the old effect was stat marks, the signature gives a **stat gift** instead (`stat`: score and potential, for good, as an altar's Gift of the forgotten does), bigger and sure: 200 marks were worth less than one point at these levels. Every signature is `kind = "mixed"` or `"boon"`; its tier is its boss's depth band (for the wiki's grouping only).

| Boss (depth) | Old | New: the signature gift | Kind | Effects (`TalentEffect`s; `kin_to`, `max_mana_percent` new, §6.4) | Line |
|---|---|---|---|---|---|
| Gnægan the Rat King (10) | vomit 90, harm 1d6 | **Rat-king's court** — the rats know their king | mixed | `kin_to { tag = "vermin" }` (every vermin creature, rats and spiders among them, is neutral to the player until the player strikes one of its type: Beast-friend's rule by tag); `disease_checks +4`; `resist poison 25`; `hunger_percent +20` (many mouths to feed) | "The squealing in your head dies down to a murmur, and you find you understand it." |
| Āttorcoppe the Spinner (20) | vomit 80, harm 2d4 | **Spinner's blood** — her venom runs in you now | boon | `resist poison 100`; `no_penalty webbed`, `status_eases webbed 200` (silk falls off you); `ambush 1d6` (a sneak attack's dice, as Ambusher) | "Your blood runs cold and bitter, and the dark of the hall is suddenly a comfort." |
| Hrīmwyrm the White (30) | vomit 20, train St 150 To 150 | **Rime-blooded** — the cold under the world is in your veins | mixed | `resist cold 100`, `resist fire −25`; `plunge_proof` (freezing water does not chill you); `stat strength +1`, `stat toughness +1` | "The wyrm's flesh is cold as the deep, and the cold stays." |
| Wēalāf the Unburied (40) | no corpse | **Barrow-king's will** — a will that outlasted a kingdom | mixed | `stat willpower +2`; `resist necrotic 50`; `drain_checks +4`; `healing_percent −25` (dead flesh knits slowly) | "Your heart slows, and does not mind." |
| Grundwyrgen the Mere-Wife (50) | vomit 70, harm 2d6 | **Mere-wife's hide** — a hide that turns blades | boon | `natural +2`; `stat strength +2`; `swimmer`; `status_eases held 100` (no grip holds you long) | "Your skin thickens and greys, and the water no longer feels cold." |
| Līgetdrȳ the Storm-Wright (60) | vomit 40 | **Storm-wright's marrow** — the thunder under the hills found a new house | boon | `resist electricity 100`; `stat intelligence +2`; `mana_regen_percent +25`; the corpse keeps a caster's `skills = { decipher_script = { gives = 10, up_to = 60 } }` beside it | "Your teeth ache, your hair stands on end, and it does not stop." |
| Stānēoten the Hurler (70) | vomit 30, train St 200 | **Hurler's arm** — the last giant's strength, and his slowness | mixed | `stat strength +3`; `carry_percent +50`; `throw_range +3`; `speed_percent −10` | "It sits in you like a stone. Your arms thicken; your feet grow heavy." |
| Nihtscinna the Hollow (80) | no corpse | **Hollow of the night** — something of you is no longer quite there | mixed | `darkvision +4`; `initiative_bonus +4`; `resist slashing 25`, `resist piercing 25`, `resist bludgeoning 25`; `no_penalty frightened`; `stat presence −2`; `max_hp_percent −10` | "The husk is nothing in the mouth. Afterwards the dark is a little less dark, and you are a little less here." |
| Wælcyrge the Chooser (90) | vomit 50, train Dx 200 | **Chooser's quickness** — she acted twice to anyone's once; now so do you, nearly | mixed | `speed_percent +15`; `stat dexterity +3`; `initiative_bonus +6`; `clotting`; `hunger_percent +25` | "Lean and bitter. Your heart beats faster, and keeps the pace." |
| Nīðdraca the Hate-Wyrm (100) | vomit 20, train St 200 To 200 | **Hate-wyrm's heart** — the oldest fire under the hills, banked in you | mixed | `resist fire 100`, `resist cold −25`; `stat strength +2`, `stat toughness +2`; `fire_breath { length = 10, damage = "12d8", damage_type = "fire", spell = "dragon_fire", cooldown = "1h", degrees = 1 }` (the Dragon sign's active, larger, in the abilities menu under its source) | "The wyrm's flesh is hot as a forge, and the heat finds a place behind your breastbone and stays." |

The signatures' 100s are resistance, not immunity (§6.6): 100 in the total, 75 of it effective, and 25 to spare against a curse or a vulnerability. Rime-blooded and Hate-wyrm's heart meet in a player who eats both: resistances sum (`TalentMods::resist`), so both wyrms in the blood give a total of 75 to fire and to cold, all of it effective. The vulnerabilities stay on both signatures (§14, D8). No relation is needed; the wiki says so.

## 6. The profane gift table

Forty drawn gifts in three tiers, each a file `data/profane/<id>.toml`. A gift's **description** (the Talents page, the wiki) says what it does in words; its **line** is said as it takes; the **effects** are exact and hidden. `w` is the weight within its tier. Relations in §7.

### 6.1 Lesser (14; by weight 45 % boons, 27 % mixed, 27 % banes)

| Id | Name | Kind | w | Line | Effects | Relations |
|---|---|---|---|---|---|---|
| `midden_gut` | Midden-gut | boon | 12 | "Your stomach turns over once, and is never troubled again." | `eats_rotten`; `rot_harm_percent −100` | `not_for = ["ratfolk"]` |
| `soft_footed` | Soft-footed | boon | 12 | "The soles of your feet grow thick and silent." | `footsteps_percent −50` | excludes `hoofed` |
| `barrow_eyed` | Barrow-eyed | boon | 12 | "The dark thins." | `darkvision +3` (from none: 3 cells) | excludes `dim_eyed`; replaced by `deep_eyed` |
| `thick_blooded` | Thick-blooded | boon | 10 | "Your blood runs slow and dark." | `clotting`; `disease_checks +2` | excludes `thin_skinned`, `glass_boned` |
| `devourer` | Devourer's belly | boon | 10 | "You find you want more." | `corpse_stats_percent +100` (a corpse's marks and widen chances doubled); `no_bloating` | — |
| `horned` | Horned | mixed | 8 | "Two knots of bone push through your brow." | `bars_slot head`; `stat presence +1`; `natural +1` | — |
| `hoofed` | Hoofed | mixed | 8 | "Your feet harden, and ring on the stone." | `bars_slot feet`; `speed_percent +5`; `footsteps_percent +50`; `kick_die 8` | excludes `soft_footed` |
| `mere_eyed` | Mere-eyed | mixed | 6 | "Your eyes go pale and wide." | `threshold_percent −25` (your eyes need a quarter less light); `day_sight −2` (the overworld by day, nearer) | excludes `dim_eyed`; replaced by `deep_eyed` |
| `tailed` | Tailed | mixed | 6 | "Something new keeps your balance." | `dodge +1`; `swimmer`; `footsteps_percent +15` | — |
| `thick_skulled` | Thick-skulled | mixed | 6 | "Your skull thickens. Thoughts take the long way round." | `natural +1`; `stat intelligence −1` | — |
| `hungering` | Hungering | bane | 12 | "You are hungry again already." | `hunger_percent +50` | replaced by `wyrms_hunger`; undone by nothing |
| `thin_skinned` | Thin-skinned | bane | 8 | "Your skin goes thin as paper." | `wound_more 1` (a big hit wounds a degree more); `trap_damage_percent +25` | excludes `thick_blooded`, `scaled` |
| `reeking` | Reeking | bane | 6 | "A smell of the midden follows you, and you cannot wash it off." | `skill { skill = "stealth", value = −15 }`; `distrusted` (no one makes way for you) | — |
| `craven` | Craven-hearted | bane | 8 | "Something in you flinches, and goes on flinching." | `status_lasts frightened +100`; `checks willpower −1` | — |

### 6.2 Greater (12; 45 % boons, 28 % mixed, 26 % banes)

| Id | Name | Kind | w | Line | Effects | Relations |
|---|---|---|---|---|---|---|
| `hill_thewed` | Hill-thewed | boon | 10 | "Your shoulders broaden." | `stat strength +2` | — (no undoer: see below the table) |
| `adder_quick` | Adder-quick | boon | 10 | "Your hands are quicker than your thoughts." | `stat dexterity +2` | undone by `palsied` |
| `oak_hearted` | Oak-hearted | boon | 10 | "Your heart beats slow and strong." | `stat toughness +2` (hit points with it) | undone by `withered` |
| `scaled` | Scaled | boon | 10 | "Your skin hardens into small grey scales." | `natural +2`; `resist slashing 10` | excludes `thin_skinned`, `glass_boned`, `hollowed` |
| `blood_nosed` | Blood-nosed | boon | 8 | "You smell blood from across the hall." | `scent { status = "bleeding", radius = 8 }`; `hearing +2` | — |
| `clawed` | Clawed | mixed | 8 | "Your nails thicken into claws." | `bars_slot hands`; `unarmed_die 8`; `unarmed_on_hit { status = "bleeding", chance = 25, severity = 1 }` | excludes `many_fingered` |
| `venom_toothed` | Venom-toothed | mixed | 8 | "Your eye-teeth lengthen, and your mouth tastes bitter." | `unarmed_on_hit { status = "poisoned", chance = 30, severity = 2 }`; `resist poison 50`; `stat presence −1` | — |
| `many_fingered` | Many-fingered | mixed | 6 | "A sixth finger uncurls on each hand." | `opens_slot extra_ring_1`; `stat dexterity −1` | excludes `clawed` |
| `ember_veined` | Ember-veined | mixed | 8 | "Your veins glow faintly under the skin." | `resist fire 50`; `inner_light { light = "ember_glow" }` (a dim red light you shed always: you see by it, and so does everything else; a new preset in `data/lights.toml`) | replaced by `wyrm_blooded` |
| `slow_blooded` | Slow-blooded | bane | 10 | "Your wounds close slowly, and your feet drag." | `healing_percent −33`; `speed_percent −5` | — |
| `dim_eyed` | Dim-eyed | bane | 8 | "A film settles over your eyes." | `threshold_percent +40` (your eyes need two fifths more light) | excludes `barrow_eyed`, `mere_eyed`, `deep_eyed` |
| `marked_by_the_deep` | Marked by the deep | bane | 10 | "Something far below has your scent." | `spawn_every_percent −40` (creatures come to your floor 40 % more often) | — |

### 6.3 Dire (14; 37 % boons, 25 % mixed, 38 % banes)

| Id | Name | Kind | w | Line | Effects | Relations |
|---|---|---|---|---|---|---|
| `twice_born` | Twice-born | boon | 6 | "Something in you will not stay dead." | `rise_from_ashes` with the Phoenix's numbers (once, from a death: up with half the hit points, Ashen; the burst as the sign's), under its own progress flag `profane_risen`: a Phoenix who is Twice-born rises once by each (§14, D7) | — |
| `stone_sinewed` | Stone-sinewed | boon | 10 | "Your sinews turn to cord and your bones to stone." | `stat strength +3`; `carry_percent +30` | — |
| `deep_eyed` | Deep-eyed | boon | 8 | "The dark is gone. There is only what is in it." | `vision { profile = "darkvision" }` (your eyes see as a dwarf's do, whatever your race's); `darkvision +6`; `sees_unaware` | replaces `barrow_eyed`, `mere_eyed`; excludes `dim_eyed` |
| `wyrm_blooded` | Wyrm-blooded | boon | 10 | "Your blood runs thick and slow, and nothing much hurts it." | `resist fire 50`, `resist cold 50`, `resist electricity 50`, `resist poison 50` | replaces `ember_veined` |
| `fell_minded` | Fell-minded | boon | 10 | "Your thoughts come faster and colder." | `stat intelligence +3`; `max_mana_percent +25`; `mana_regen_percent +25` | — |
| `wing_shouldered` | Wing-shouldered | mixed | 8 | "Two leathery stubs push out of your shoulder-blades, too small to fly on." | `bars_slot cloak`; `speed_percent +10`; `travel_percent +25`; `stat toughness −1` | — |
| `hollowed` | Hollowed | mixed | 8 | "Blades pass half through you now. So does warmth." | `resist slashing 25`, `resist piercing 25`, `resist bludgeoning 25`; `healing_percent −50`; `stat presence −2` | excludes `scaled` |
| `storm_marrowed` | Storm-marrowed | mixed | 8 | "Your bones hum." | `resist electricity 100`; `mana_cost_percent −20`; `max_hp_percent −10`; `hunger_percent +30` | — |
| `stone_eyed` | Stone-eyed | mixed | 6 | "Your eyes go grey as slate, and people look away." | `stare { status = "transfixed", severity = 2, cooldown = "1h" }` (the Basilisk sign's Transfixing stare, an active in the abilities menu); `distrusted`; `skill { skill = "negotiation", value = −20 }` | — |
| `glass_boned` | Glass-boned | bane | 10 | "Your bones go light and brittle." | `natural −2` (a negative value, allowed for gifts); `trap_damage_percent +50`; `wound_more 1` | excludes `scaled`, `thick_blooded` |
| `withered` | Withered | bane | 10 | "The flesh wastes on your frame." | `stat toughness −3` (hit points with it; never below `DRAIN_FLOOR`, as an altar's wrath) | undoes `oak_hearted` |
| `palsied` | Palsied | bane | 10 | "Your hands shake, and do not stop." | `stat dexterity −3`; `initiative_bonus −4` | undoes `adder_quick` |
| `unquiet` | Unquiet | bane | 8 | "Sleep comes, but rest does not." | `resting_percent −100`, `mana_resting_percent −100` (a rest heals and recovers no faster than waking; the hourly pace is untouched) | — |
| `wyrms_hunger` | Wyrm's hunger | bane | 8 | "You could eat the hoard, and the wyrm after it." | `hunger_percent +150`; `no_bloating` | replaces `hungering` |

`hill_thewed` has no undoer: Strength is what the signatures pile on, and a bane that cancelled it would cancel too much; its cell in the table above reads "—".

### 6.4 The new effect kinds

Six `TalentEffect`s join, each read at one hook, usable by any talent later:

| Kind | Fields | What it does, and where it is read |
|---|---|---|
| `kin_to` | `tag` | Creatures with the tag are neutral to the holder until the holder attacks one of that type (Beast-friend's rule, `TalentMods::beast_friend`, generalised: `TalentMods::kin_tags: Vec<String>`, read where the stance toward the player is derived). A struck type stays hostile as Beast-friend's does |
| `bars_slot` | `slot` | The slot cannot be used by the holder: `TalentMods::barred_slots: Vec<SlotId>`. Read by `Pack::equip`/the equip screen's choice (refused: "Your horns are in the way." — the gift's `bars_message`), by the equipment screen (the row shows `— (horns)`, the gift's `bars_mark`), and once, as the gift takes: what is worn there is taken off and falls to the player's cell ("Your iron helm no longer fits, and falls to the floor."). Never `main_hand`, `off_hand`, `missile_weapon`, `missiles`, or `tool` (the loader refuses: a body changes, a grip does not) |
| `opens_slot` | `slot` | A slot gated to a class (`slot.class`) is open to the holder too: the gate reads `class == holder's class || TalentMods::opens_slots contains it`. Only the aristocrat's extra ring slots are gated today |
| `unarmed_on_hit` | `status`, `chance`, `severity` | The holder's unarmed melee blows (`UnarmedDie`'s path: no weapon in the main hand) carry an `on_hit` entry, applied in `after_blow`'s loop as a creature's natural attack's is. Several sum as several entries |
| `vision` | `profile` | The holder's vision profile is `profile` (an id in `data/vision.toml`) instead of the race's: read where `WorldState::eyes` takes the race's. A creature cannot hold it (talents are the player's); the dev F3 cycle still overrides it |
| `max_mana_percent` | `value` | Maximum mana this percent more (negative: less, never below 0): one hook in `World::max_mana` beside the other modifiers (`docs/dev/mana.md`), as `max_hp_percent` is for hit points |

Everything else in the tables is an existing kind: `stat`, `natural`, `dodge`, `resist` (negative allowed: a vulnerability, as creatures' `resist` is), `speed_percent`, `hunger_percent`, `rot_harm_percent`, `eats_rotten`, `no_bloating`, `corpse_stats_percent`, `healing_percent`, `resting_percent`, `mana_resting_percent`, `mana_regen_percent`, `mana_cost_percent`, `max_hp_percent`, `darkvision`, `threshold_percent`, `day_sight`, `sees_unaware`, `scent`, `hearing`, `footsteps_percent`, `clotting`, `wound_more`, `disease_checks`, `drain_checks`, `checks`, `status_eases`, `status_lasts`, `no_penalty`, `trap_damage_percent`, `spawn_every_percent`, `carry_percent`, `throw_range`, `travel_percent`, `swimmer`, `plunge_proof`, `initiative_bonus`, `ambush`, `unarmed_die`, `kick_die`, `inner_light`, `skill`, `distrusted`, and the actives `fire_breath`, `stare`, `rise_from_ashes`.

**Forbidden in a gift** (the loader refuses them: they are handed out once and cannot be rescinded cleanly, or they are a class's or a pick's business): `max_hp_per_levels`, `max_mana_per_levels`, `weapon_ranks`, `weapon_marks`, `extra_pick`, `skill_rolls`, `take_up_skills`, `starting_gold`, `starting_gems`, `heirloom`, `grants_resource`, `companion`, `familiar`, `xp_percent`. `stat` is the one handed-out kind allowed, because `Stats::gift` of the negative undoes it exactly (the potential falls with the score, never below the score; a Toughness fall cuts hit points to the new maximum, never below 1).

### 6.5 The data shape

```toml
# data/profane/horned.toml
[[gift]]
id = "horned"
name = "Horned"
kind = "mixed"                 # boon | mixed | bane
tier = "lesser"                # lesser | greater | dire
weight = 8                     # within the tier; 0 and `only` for a signature
description = "Horns: no helm fits, but you carry yourself like something to be reckoned with."
line = "Two knots of bone push through your brow."
undone = "Your horns crumble and fall away."        # said when `undoes` or `replaces` takes it
effects = [
  { kind = "bars_slot", slot = "head", mark = "horns", refused = "Your horns are in the way." },
  { kind = "stat", stat = "presence", value = 1 },
  { kind = "natural", value = 1 },
]
# excludes = ["..."], replaces = ["..."], undoes = ["..."], not_for = ["ratfolk"], only = "gnaegan"
```

The loader (`content/profane.rs`, `ProfaneTable`) reads every file into a `GiftDef` and registers each as a `TalentDef` of tree `profane` with `granted = true`, `flaw = (kind == bane)`, no `needs`, `systems` from its effects, so `TalentBook` sees them and `may_take` refuses them as it refuses every granted talent. Checks at load: ids unique against every talent; `kind`, `tier`, `weight` (0 only with `only`, else 1 to 100); every relation names a gift; `excludes` made symmetric (written once, on either gift: the phases write it on the gift that lands later, so a partial table loads); `replaces`/`undoes` acyclic and never naming a signature; `not_for` races known; `only` a creature with `gives` of it; no forbidden effect; `bars_slot` not a grip slot; each tier holds at least one drawn gift of each kind; and the campaign check: every `[[dungeon.endless.boss]]` creature has `corpse = true`, `everlasting`, `gives`, and `profane`.

Two checks need the whole design in place and switch on with it: the tier coverage (each tier holds a drawn gift of each kind) in P6, and the campaign check (every endless boss has `gives` and `profane`) in P4. Until then a partial table loads.

The tree: `data/talents.toml` gains `[[tree]] id = "profane"`, `name = "Profane"`, `no_free = true` (never offered). The Talents page shows it last, after Stars.

### 6.6 Resistance and immunity (#1620)

Every `resist` in this design is written against #1620's model, and the phases that add one come after it:

- A gift's `resist` adds to the player's resistance total for that damage type, beside the race's, the talents', and the gear's affixes and materials. The total has no cap: Spinner's blood on a drow is 125 to poison.
- Only 75 of a total is effective. The clamp is made where resistance is read (`damage::resist`, afflict doses, and the other readers #1620 routes through its one function), never in the stored total. So the surplus over 75 is a buffer: Rime-blooded's cold 100 still leaves 75 effective under a −25 curse.
- A negative `resist` is a vulnerability, floored at −100 as now.
- Immunity is a creature type's own feature under #1620 (its damage-type immunity list), not a resistance of 100. No gift grants it; nothing in this design makes the player immune to a damage type. The words the player sees come from the effective value, as #1620 sets them.
- Gear with resistance affixes (`docs/dev/magic-items.md`, `docs/dev/enchanting.md`) adds to the same total. Gifts do not touch items, their affixes, or their capacity; `bars_slot` and `opens_slot` only decide where an item can be worn.

## 7. The draw: rolls, tiers, weights, duplicates, relations

### 7.1 Each boss's rolls

`on_eat.profane = { rolls = N, tiers = { lesser = a, greater = b, dire = c } }`, the weights summing to 100 (checked):

| Boss | Depth | Rolls | Lesser | Greater | Dire |
|---|---|---|---|---|---|
| Gnægan | 10 | 1 | 100 | 0 | 0 |
| Āttorcoppe | 20 | 1 | 80 | 20 | 0 |
| Hrīmwyrm | 30 | 1 | 60 | 40 | 0 |
| Wēalāf | 40 | 2 | 40 | 50 | 10 |
| Grundwyrgen | 50 | 2 | 30 | 55 | 15 |
| Līgetdrȳ | 60 | 2 | 20 | 55 | 25 |
| Stānēoten | 70 | 2 | 10 | 50 | 40 |
| Nihtscinna | 80 | 3 | 0 | 50 | 50 |
| Wælcyrge | 90 | 3 | 0 | 35 | 65 |
| Nīðdraca | 100 | 3 | 0 | 20 | 80 |

A run that eats all ten makes 20 draws against 40 drawn gifts; the first three corpses (one roll each) are the cheap taste of the gamble, the last three (three rolls each, mostly dire) the real one.

### 7.2 One roll

| Step | Rule |
|---|---|
| The tier | One gameplay-stream draw (`state.rng`, as `bestow_talent`'s) by the boss's tier weights |
| The pool | The tier's drawn gifts (`weight > 0`) the eater **may take**: not held; not `excludes`-related to any held gift; not `not_for` the eater's race. An undoer with nothing to undo stays in the pool and is taken as a bane in its own right (Withered is Toughness −3 whether or not you were Oak-hearted; §7.3) |
| The gift | One weighted draw among the pool (`weight`); the odds shift as the pool empties, which the wiki says |
| An empty tier | The next tier up, then down (lesser → greater → dire; dire → greater → lesser); all three empty: "Nothing more of it will take in you." and the roll gives nothing (the table is the only cap) |
| Order | Signature first, then the rolls in order, each `bestow_gift` complete (relations applied) before the next draw, so a draw never offers what the one before excluded |

### 7.3 Relations

| Relation | Written on | When the new gift G is drawn or given and the holder has H | What is said |
|---|---|---|---|
| `excludes` | either (made symmetric at load) | G is **not in the pool** while H is held (a signature's excludes bar drawn gifts; a drawn gift can never bar a signature: signatures are sure) | — |
| `replaces` | G, naming H | H is **rescinded**, then G is taken: the greater form of a lesser (Deep-eyed over Barrow-eyed, Wyrm-blooded over Ember-veined, Wyrm's hunger over Hungering) | H's `undone` line, then G's line and "You are Deep-eyed." |
| `undoes` | G, naming H | H is **rescinded** and G is **not taken**: a bane that cancels the boon it opposes and no more (Withered over Oak-hearted). Without H held, G is taken as any bane | H's `undone` line, then G's `instead` line ("The withering takes your oak heart, and nothing more.") |

Why three and not one: a mutation system is only a gamble worth taking when a bad draw can be dug out of and a good one can be lost; `replaces` is the ladder up, `undoes` the slide down, `excludes` what cannot be at once (two kinds of feet). Each is one `if` in `bestow_gift`.

### 7.4 Rescinding

`World::rescind(gift)`: removes it from `Actor::talents` and `Actor::boons`; for each `stat` effect, `Stats::gift(stat, −value)` (the floor `DRAIN_FLOOR`, as an altar's wrath), hit points and mana cut to their maxima, never below 1 (`keep_hp_within_max`); a `bars_slot` gone frees the slot (nothing is put back); an `opens_slot` gone **drops what is worn in the slot to the floor** ("Your ring slips from a finger that is no longer there."); an active's cooldown status is left to ease. Everything else is read at its hook and is simply no longer there. `rescind` is also the hook §8 leaves for the deferred good use.

## 8. Losing a gift, curing, and the room left for the good use

**Decision: no cure in this design** (§14, D3). A profane gift leaves the body only by `replaces` or `undoes` (§7.3). Not the temple's mend (that restores drained points, a different thing), not holy water, not a scroll of restoration, not a prayer. The pact is: what you ate is part of you. Reasons: (1) the brief asks for a gamble, and a gamble with a refund is a purchase; (2) the owner's house rule against anti-farming limits cuts the other way here — nothing stops a player from eating the next corpse hoping for `undoes` or `replaces`, and that is the only remedy; (3) the deferred good-aligned use (#196) is the natural home of any cleansing, and its design should decide whether the good alternative *removes* profane gifts or merely *forgoes* them.

What is left for it: `World::rescind(gift)` (§7.4), which takes any gift away cleanly; `GiftDef::kind` to pick banes from boons; and the everlasting corpse itself, in its lair, that a rite could be performed over. Nothing in this design depends on the corpse being eaten: a corpse used another way would simply never reach `eat_fresh`.

## 9. Alignment: the pull toward evil

**Decision: a new `[shift]` deed, `eat_profane = -300`**, one call in `eat_fresh` (step 5), diminishing as every deed does (`Score::shifted`: against the lean it counts in full; with it, by the share of the way left).

| From | After 1 | 2 | 3 | 4 | 5 | 10 |
|---|---|---|---|---|---|---|
| Saintly 1000 | 700 Good | 400 Good | 100 Neutral | −200 Callous | −440 Evil | about −900 Wicked |
| Neutral 0 | −300 Callous | −510 Evil | −657 Evil | −759 Wicked | −831 Wicked | about −970 Wicked |
| Evil −500 | −650 Evil | −755 Wicked | −828 Wicked | — | — | — |

(From 1000: four eaten in full, against the lean, to −200; then 300 × 800⁄1000 = 240 → −440, 300 × 560⁄1000 = 168 → −608, …, rounded toward 0. The bands: Good from 400, Kind from 150, Neutral from −149, Callous from −399, Evil from −749.) So a saint who eats three is Neutral, five is Evil, all ten Wicked; nobody who eats more than two stays Good. What that costs: a paladin's zeal empties at each meal (`sin_empties_zeal`); a priest's holy actives (Consecrate, Holy radiance, Sanctuary: `alignment = { min = "kind" }`) stop working once Neutral; good shopkeepers' prices worsen at Evil; good quest-givers scorn. Nothing is refused (alignment bars nothing, #642): the choice stays the player's, and the wiki states the number.

Why −300 and not −200 (as `eat_kin`) or more: it must be the biggest single sin in the game (the brief: "a big pull"), big enough that a good character cannot eat casually, and small enough that one meal does not undo a good run outright (a saint stays Good after one). §14, D1.

The dev `actors` layer's `alignment` score shows the fall; `alignment_tests.rs` gains the deed's case (full against the lean, diminished with it, the paladin's zeal emptied, the bands' notes after the gifts' lines).

## 10. Races, classes, equipment slots, statuses

| With | Rule |
|---|---|
| Races | `not_for` bars a gift that would do nothing (Midden-gut for ratfolk). Everything else stacks on the race: a dwarf's darkvision grows by Barrow-eyed's 3 cells; a drow's `resist poison 25` sums with Spinner's blood to 125, 75 of it effective (§6.6); Deep-eyed's `vision` on a darkvision race changes nothing but adds its `darkvision +6` and `sees_unaware`; halflings (`bare = ["feet"]`, wearing no shoes) lose nothing to Hoofed's barred feet. A race's `hunger` percent and the gifts' `hunger_percent` sum as talents' do (`World::hunger_percent`, at least 1). Sizes are untouched: no gift changes size (§15) |
| Classes | No class is barred from eating; the cost falls on the good (§9). Casters gain most from Fell-minded, Storm-marrowed, Storm-wright's marrow; fighters from the Strength gifts; everyone from speed. Nīðdraca's Fire breath and Stone-eyed's stare are the sign actives' own `TalentEffect`s (`fire_breath`, `stare`). `Ability::Sign(active)` is already offered to any holder of the effect (`TalentMods::has_sign_active`), and used the same way (`use:fire_breath:12,5`, `use:transfixing_stare:12,5`). What changes: the menu row's source is a new `Source::Profane` ("profane") when no sign talent of the holder gives the active, and "sign" when one does. A Dragon-sign player who eats the Hate-wyrm has one Fire breath, the larger: `TalentMods::fire_breath` keeps the effect with the larger `damage`, one cooldown status (`breath`); two stares likewise keep the larger `severity` |
| Slots | `bars_slot` and `opens_slot` (§6.4). A spirit weapon, a two-handed grip, missiles: never touched. The equipment screen's row for a barred slot: `Head        — (horns)`; `equip:head:b` refused with the gift's `refused` line, no time. An `extra_ring_1` opened by Many-fingered shows as the aristocrat's does, "on a finger" |
| Statuses | Gifts are not statuses and never ease. They read statuses through `status_eases`, `status_lasts`, `no_penalty`, `scent`, `unarmed_on_hit`, and the actives' cooldown statuses (`breath`, `gaze`, shared with the signs'). Twice-born's rising leaves the player Ashen as the Phoenix's does. Nothing a gift does is a disease, and `disease_immune` is not on the table (a cure by gift would be a cure, §8) |
| Companions, familiars, allies | Only the player eats. An ally's kill opens the stairs and leaves the corpse as any kill does |
| Stat training | Signatures give points (`stat`), not marks, so `corpse_stats_percent` (Gourmand, Devourer's belly) does not touch them; Līgetdrȳ's Decipher script `skills` gift is a corpse skill as today |
| Hunger | A boss corpse feeds 1,000 nutrition (medium) with overeating as ever; `no_bloating` gifts spare the Bloated slow |

## 11. What the player sees; the dumps; saves; determinism

| Surface | What |
|---|---|
| The meal | §4's lines in order. A gift: its `line`, then "You are Horned." (the name; a signature the same: "You are Rime-blooded."). An `undoes`: the undone gift's `undone`, then the undoer's `instead`. An empty table: "Nothing more of it will take in you." Never a number, never "boon" or "bane" |
| The Talents page | A **Profane** group after Stars: each gift's name and `description`, banes marked as flaws are ("flaw" after the name, as a sign's); the source word "profane" where "altar" and "story" go |
| The character screen | Resistances in words as now (`Resistant to cold`, `Vulnerable to fire`: the words of the effective value, §6.6); a stat gift shows in the score as any gift does; a barred slot on the equipment screen |
| The eat list, look mode | A boss corpse is named and judged as any corpse (a trained eye: fresh). Nothing marks it profane: the boss's name is the tell, and the wiki tells the rest (§14, D2) |
| `talents` dump | `Horned (profane)`, `Hungering (profane, bane)`; JSON `talents[].profane: "boon" \| "mixed" \| "bane"` (left out otherwise) through a new `OwnTalent::profane(kind)` |
| `stats` dump | Unchanged: resistances in words, the alignment band |
| `abilities` dump | `Fire breath (profane): a cell` |
| `progress` dump (dev) | `profane 3 held; pool lesser 9/14, greater 12/12, dire 14/14` (the eligible count over the tier's drawn gifts) |
| `actors` dump (dev) | A boss corpse among the food with stage `fresh` (it always is) |
| Saves | `Actor::boons` already holds for-life talents and is restored by `stars::grant` → `Talents::regrant`; profane gifts ride in it with no new field. Twice-born's flag is a progress flag (`Progress::happened`), `profane_risen`, beside `risen` and `risen_twice` (`world/stars.rs`): the rising takes `risen` first, then `risen_twice` with Second rising, then `profane_risen` with Twice-born. A save naming a gift the content lacks fails to load, as any unknown talent does (saves are pinned to the version). The hash covers `Actor::boons` and `talents` already |
| Determinism | Every draw on `state.rng` in `eat_fresh`, in a fixed order; `everlasting` and the tier weights are content. Replays record `eat:b`, nothing new; `replay --create` then `--verify` on a run that eats Gnægan is a test |
| Dev actions | `gift:ID` gives a gift, `rescind:ID` takes one away (both no time, dev-only, for captures and tests), listed in `docs/dev/cli.md`'s action text with the other dev actions |
| The wiki | A new page `wiki/src/profane-gifts.md` (under Food and corpses in `SUMMARY.md`): what the keepers' corpses do, the pull toward evil with its number, that they never rot, then the generated tables: `profane-corpses.md` (each boss: its signature gift's exact effects, rolls, tier odds), `profane-gifts.md` (every drawn gift: tier, kind, odds within its tier at a full pool, exact effects, relations, races barred). `food.md` gains the everlasting paragraph; `alignment.md`'s deeds table gains `eat_profane` (its generator lists the `[shift]` fields); `dungeon-guide.md`'s Grundlēas section links the page; `TERMS.md` gains "profane gift", "signature gift", "everlasting". New generators in `xtask/src/wiki/profane.rs`, two lines in `generate()` beside hunger's |

## 12. Balance, by the depth each boss sits at

The player at depth d of Grundlēas is about level d (the branch's danger is its depth; `docs/balance.md` "Players" has the level-10/30/50 characters). What a gift is worth against that:

| Depth | Signature's weight | The draws | Judgement |
|---|---|---|---|
| 10 (Gnægan) | `kin_to vermin` ends the rat and spider fights of the first twenty floors; +4 disease checks against filth fever; poison 25 %. Worth about one good talent | 1 lesser: 45 % a small boon, 27 % mixed, 27 % a bane that hurts (Hungering +50 % food at level 10 is felt; Reeking −15 Stealth hurts a thief) | A taste: large for its level, rarely ruinous |
| 20–30 | Spinner's blood: poison resistance at the effective cap for the rest of the run, with 25 to spare, plus 1d6 sneak dice (Ambusher's whole worth) at level 20. Rime-blooded: cold resistance at the cap (the frost creatures of the thirties, Hrīmwyrm's kin) at the price of fire +25 % and two points | 1 roll, up to 40 % greater: a stat point pair is a talent (Stubborn's +2 Toughness) | Strong; the fire vulnerability bites in the sixties and at the hundredth floor |
| 40–70 | Barrow-king's will: Willpower +2, necrotic 50 %, drain checks +4 against the undead courts, for −25 % healing (a real cost: `healing_rate` is the one pace). Mere-wife's hide: +2 natural armour is a suit's upgrade at 50 (`docs/balance.md` "Protection by body": 11–16 at level 50), plus Strength +2. Storm-wright's marrow: a caster's dream. Hurler's arm: +3 Strength and half again the load for −10 % speed, which is about a tenth fewer actions for life | 2 rolls each, 10–40 % dire: the first run-changing banes appear (Slow-blooded −33 % healing, Marked by the deep +40 % spawns; dire: Unquiet, Withered −3) | The gamble proper. A fighter who eats all four is a good deal stronger and a good deal stranger |
| 80–100 | Hollow of the night: 25 % to every weapon damage type is worth more than Protection at level 80 (monsters hit for 30–50), for −10 % hit points and −2 Presence. Chooser's quickness: +15 % speed and +3 Dexterity is the largest single buff in the game outside Haste. Hate-wyrm's heart: fire resistance at the cap, +2/+2, and a 12d8 (54) fire cone every hour: modest against level-100 hit points (300–600), free, and it sets the burning | 3 rolls each, 50–80 % dire: 38 % of the dire tier is banes. Expect about one dire bane in each of the last three meals: Unquiet (resting heals nothing: potions, spells, and the hourly pace remain), Palsied −3 Dexterity, Glass-boned | Very large upside, real risk, as the brief asks. A level-90 character can carry a bane; a level-10 one could not, which is why the lesser tier has none of them |

Sums: a character who eats all ten and draws averagely holds about Strength +10, Toughness +2, Dexterity +3, Willpower +2, Intelligence +2, Presence −2, speed +5 % (Hurler's −10, Chooser's +15), poison, cold, electricity, and fire resisted at the effective 75 % (§6.6), 25 % to blades, and seven to nine drawn gifts of which two or three are banes, at Wicked. That is a different creature, which is the point; unbalanced choices are fine (the owner), and the wiki states every number so the player chooses knowing.

Nothing here touches the balance probe (`docs/balance.md`): gifts are optional and late; a "with every signature" column is not worth the probe's time.

## 13. Implementation plan: the phases

One PR each, in this order; each starts from `main` after the one before it has merged. Every phase keeps the content loading: a gift lands only with the effects it uses, and each `excludes` is written on the gift that lands later (§6.5). Every phase updates the docs it touches: a new topic file `docs/dev/profane-gifts.md` (made in P2, indexed with the other topics), `docs/dev/hunger.md`, `docs/dev/progression.md`, and the others named below; the spec where the player meets the change; the wiki in the same PR (CLAUDE.md "Wiki"). Three phases come after #1620 (§6.6), because they add resistance: P3, P4, and P6.

| Phase | After | Scope (code, data, docs) | Tests |
|---|---|---|---|
| **P1 Everlasting corpses** | — | `content/creature.rs`: `everlasting` (refused without `corpse` or `unique`). The hunger content (`Food`, `Content::load_hunger`): `keeps`/`lasts` as `Option<Duration>`, `None` from `everlasting` alone. The rot code: `rot::stage`, `gone_at`, `until_rots`, `World::rot_pack`, `rot_away`, and `freshness::judged` for `None`. `world/crystal.rs::crystallise` skips an everlasting creature; `fight.rs::creature_dies` lays its corpse, not a statue (§3). Data: `corpse = true` and new words for Wēalāf and Nihtscinna; `everlasting = true` on all ten in `assets/campaigns/ascmearc/data/creatures/`. The campaign check: every `[[dungeon.endless.boss]]` creature has `corpse` and `everlasting`. Docs: `hunger.md`, `enchanting.md` (the glass skip). Spec `08h-hunger.md`. Wiki: `food.md`'s paragraph; the corpses table (`xtask/src/wiki/hunger.rs`) marks them everlasting | A boss corpse 30 days old is Fresh, not dropped, judged fresh; the loader refuses `everlasting` without `corpse` and on a non-unique; a boss killed by a crystal spell and one petrified both leave the corpse; the campaign check; `release_content` |
| **P2 The gift machinery** | P1 | `content/profane.rs` (`GiftDef`, `ProfaneTable`, the per-gift checks of §6.5, the forbidden effects of §6.4); the `profane` tree in `data/talents.toml`; `data/profane/` with **seven** gifts on existing effects, enough to prove every relation: `midden_gut` (`not_for`), `hungering` and `wyrms_hunger` (`replaces`), `oak_hearted` and `withered` (`undoes`), `thick_blooded` and `thin_skinned` (`excludes`). `world/profane.rs`: `World::bestow_gift`, `rescind` (§7.3, §7.4). `OwnTalent::profane`; the `talents` and `progress` dumps (§11); the Talents page's Profane group (`crates/warlock/src/ui/`). The dev actions `gift:ID` and `rescind:ID` (action text; `docs/dev/cli.md`). Docs: the new `profane-gifts.md`, `progression.md`. No spec or wiki change: nothing reaches play yet | `world/profane_tests.rs`: bestow; rescind reverses `stat` exactly (Withered's floor); replaces; undoes with and without the boon; excludes both ways; `not_for`; restore puts gifts back (`from_state`, the hash equal). A screen snapshot of the Talents page with a boon and a bane |
| **P3 Gnægan's meal, the draw, the deed** | P2, #1620 | `OnEat::gives` and `profane` with the loader's rules (§4: no `vomit`, `gives`, `profane`, and `everlasting` together; §7.1: the tier weights sum to 100). `eat.rs`: the profane steps of §4 and the draw of §7.2 (`state.rng`), with the lines. `Deed::EatProfane`; `eat_profane = -300` in `data/alignment.toml`. Gnægan's signature `rat_kings_court` with every effect but `kin_to` (that lands in P5), and Gnægan's `on_eat` rewritten (`profane = { rolls = 1, tiers = { lesser = 100 } }`). Docs: `hunger.md` ("Profane corpses"), `alignment.md`, `profane-gifts.md`. Spec: `08h-hunger.md` (profane corpses), `07c-generation.md` (the endless branch's keepers leave them). Wiki: the page `wiki/src/profane-gifts.md` under Food and corpses in `SUMMARY.md`; `xtask/src/wiki/profane.rs` with the tables `profane-corpses.md` and `profane-gifts.md`, one line each in `generate()` beside hunger's; the deeds table gains `eat_profane`; `TERMS.md` gains "profane gift", "signature gift", "everlasting" | `eat_tests`: §4's order and lines; an interrupted meal does nothing; the tier fallbacks; the empty table's line; the deed in full and diminished, the paladin's zeal emptied; `replay --create` then `--verify` through Gnægan's meal; `cargo xtask wiki --check-links` |
| **P4 The other nine signatures** | P3, #1620 | Nine signature files with `only` (§5); the nine `on_eat` rewrites (no `vomit`, `train` → `stat`, Līgetdrȳ's `skills` kept) with their `profane` rolls and tiers (§7.1). The campaign check gains `gives` and `profane` for every endless boss. `Source::Profane` for `Ability::Sign` rows a gift gives (§10); `TalentMods::fire_breath` and `stare` keep the stronger. Docs: `hunger.md`, `profane-gifts.md`, `docs/dev/screens.md` if the menu's sources change. Wiki: the generated tables pick the signatures up; `dungeon-guide.md`'s Grundlēas section links the page | Per corpse: its gift held, its stats moved, its resistance totals (Rime-blooded with a −25 curse stays at 75 effective); the Dragon sign plus Hate-wyrm's heart: one breath, the larger; the `abilities` dump's `Fire breath (profane)`; the campaign check |
| **P5 The body** | P4 | The six new effects of §6.4 in `content/talent.rs` and `TalentMods`, each at its hook: `bars_slot` (`Pack::equip` refused, the equipment screen's row, the drop as it takes), `opens_slot` (the slot's class gate in `data/slots.toml`'s reader), `unarmed_on_hit` (`after_blow`), `vision` (where the race's profile is read), `kin_to` (Beast-friend's rule by tag), `max_mana_percent` (`World::max_mana`). The gifts that use them and their partners: `horned`, `hoofed`, `soft_footed`, `clawed`, `many_fingered`, `barrow_eyed`, `mere_eyed`, `dim_eyed`, `deep_eyed`, `fell_minded`. Rat-king's court gains `kin_to { tag = "vermin" }`. Docs: `progression.md`, `equipment.md`, `mana.md`, `profane-gifts.md`. Wiki: the generated tables; `equipment.md`'s note on barred and opened slots | A helm on the floor as Horned takes, and the equip refused with no time; a ring in the opened slot, and its fall at rescind; a vermin neutral until struck, then its type hostile; Clawed's unarmed blow makes bleeding; a human with Deep-eyed sees as a dwarf; Fell-minded's mana; a screen snapshot of the equipment screen with `— (horns)` |
| **P6 The rest of the table** | P5, #1620 | The other 23 drawn gifts of §6: lesser `devourer`, `tailed`, `thick_skulled`, `reeking`, `craven`; greater `hill_thewed`, `adder_quick`, `scaled`, `blood_nosed`, `venom_toothed`, `ember_veined`, `slow_blooded`, `marked_by_the_deep`; dire `twice_born`, `stone_sinewed`, `wyrm_blooded`, `wing_shouldered`, `hollowed`, `storm_marrowed`, `stone_eyed`, `glass_boned`, `palsied`, `unquiet`. The `ember_glow` light in `data/lights.toml`. Twice-born's `profane_risen` in `world/stars.rs` (§11). The tier coverage check on (§6.5). Docs: `profane-gifts.md`, `docs/balance.md`'s note that gifts are outside the probe. Wiki: the page's prose finished (every relation, the odds as the pool empties) | The loader over the full table (every tier and kind present, relations symmetric and acyclic); the `progress` dump's pool counts after each of ten meals in a fixture run; a Phoenix who is Twice-born rises twice; `cargo xtask wiki --check-links` |

## 14. Decisions: the owner's review

The owner approved the design on 2026-11-14 with the recommendations: every question took the recommended answer. Each is folded into the sections named.

| | Question | Decision |
|---|---|---|
| D1 | The size of the pull toward evil | `eat_profane = -300`, the largest single sin (§9) |
| D2 | A confirmation before the meal | None. A boss corpse is eaten as any corpse is, and nothing marks it profane (§4, §11) |
| D3 | A cure | None. A gift leaves only by `replaces` or `undoes`; `World::rescind` waits for the good-aligned use (#196) (§8) |
| D4 | The `undoes` relation | Kept: a bane that only cancels the boon it opposes (§7.3) |
| D5 | Signatures sure, draws random | As designed: each boss's signature is sure, the draws random (§5, §7) |
| D6 | The rolls per boss | 1/1/1/2/2/2/2/3/3/3 from Gnægan to Nīðdraca (§7.1) |
| D7 | Twice-born's rising and the Phoenix's | Separate flags: `profane_risen` beside `risen`; a Phoenix who is Twice-born rises once by each (§6.3, §11) |
| D8 | Vulnerabilities on the wyrms' signatures | Kept: Rime-blooded fire −25, Hate-wyrm's heart cold −25 (§5) |
| D9 | `everlasting` for uniques only | Yes; the loader refuses it on any other creature (§3) |
| D10 | The corpses' weight | Unchanged: a corpse weighs what the boss weighed and is eaten where it fell (§3) |

Two rules were added when the design was finalised, from code that landed after it was written: an everlasting creature always leaves its corpse, never glass or a statue (§3; the crystal spells of `docs/dev/enchanting.md`); and resistance follows #1620, so no gift makes the player immune (§6.6).

## 15. Choices made along the way

- **Gifts are talents, not a system of their own.** Every effect a body could change already exists as a `TalentEffect` read at one hook; `Actor::boons` already holds for-life talents and survives restore; the Talents page and the `talents` dump already mark sources. A parallel `Actor::gifts` with its own effect enum would be the speculative abstraction the house rules forbid. The gift table is a thin layer (kind, tier, weight, relations) over `TalentDef`.
- **Their own directory, `data/profane/`, not `data/talents/`.** They are a churning content table of their own kind (one file per entity, as the rule says), the talents' wiki table should not list them, and a gift file carries fields no talent has.
- **Six new effect kinds, no more.** Each is asked for by at least two gifts or a signature and is a general talent effect any later talent could use (`kin_to` is Beast-friend's rule by tag; `opens_slot` is what the aristocrat's class gate should have been). What was left out: a size change (`sizes.toml` has two sizes and a dozen readers: the burden, the corpses, Small target; too large a change for one gift), a faction change, a new natural attack with its own dice and verb (Clawed uses `unarmed_die` and an `on_hit` instead), flight, and anything that changes the race (a gift never makes a human a dwarf; Deep-eyed gives dwarf's eyes and says so).
- **Stat gifts for signatures, marks for nothing.** The old `train` 150–200 marks were a corpse's gift scaled for ordinary giants; at the levels these corpses are eaten a point is worth more than a pile, and a sure point is a gift the player can feel.
- **No vomiting.** A meal of a layer boss that comes back up is a joke at the player's expense after the hardest fight on the floor; the gamble is in the draws, not the stomach.
- **The alignment deed is one number.** No scaling by depth (a deeper corpse is no more evil than a shallower one: it is the eating), no reduction for an already-evil eater beyond the diminishing every deed has.
- **No cap.** The owner's rule against anti-farming limits; the bosses are once a run; the table is finite; 20 draws against 40 gifts with exclusions leaves the table never quite exhausted.
- **The deferred good use** is designed around, not for: everlasting corpses in their lairs, `rescind`, and `GiftDef::kind` are all it needs from here; what it does is its own document (#196).
