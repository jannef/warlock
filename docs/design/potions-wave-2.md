# Warlock — Potions, wave 2: flasks to throw, draughts to gamble on

**Built; this file is history.** Current behaviour is in spec §8 "Potions" and `docs/dev/potions.md`.

Design document for the owner's review (2026-11-07). Written against `origin/main` at 855c7579. Nothing here is implemented; §11 cuts it into issues. It builds on what is there: the potions and their `Draught` kinds (`docs/dev/potions.md`, `content/potion.rs`, `world/health/potion.rs`, spec §8 "Potions"), throwing (`docs/dev/ranged-combat.md`, `world/combat/shoot.rs`), the one cast path and what pays for a cast (`world/magic/cast.rs`: `Pay::{Mana, Scroll, Wand, Creature}`), zones (`docs/dev/spells.md` "Zones": `fire`, `fouled`, `slick`, `dark`), the spell types' rules (`data/spells.toml` `[types]`), the statuses (`data/status.toml`), item damage (`docs/dev/objects.md`), hearing (`docs/dev/hearing.md`), the outcome tables (`data/outcomes/`), and the spell effects (`data/effects.toml`). Every number is data and hidden in the game; the wiki states it.

**The owner's request** (2026-11-07, verbatim): *"design perhaps 20–40 new potions with various effects. Some potions should be dangerous to drink; their main purpose could be to use as a throwing weapon (like: releases a cloud of smoke to hide from archers etc, or an explosion of fire). Gives some small risk of chugging unidentified potions."*

## Contents

1. Summary
2. What exists today
3. The one idea: a flask is a scroll you throw
4. The potions (31)
5. Throwing a potion
6. Drinking the dangerous ones
7. Where they come from
8. What the player sees, and the wiki
9. Rules and code impact
10. Rejected ideas
11. Slices
12. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **The count** | **31 new potions** (§4): 6 helpful draughts that lend a status, 4 utility draughts, 3 gambles, and 18 **flasks**, whose point is the throw and whose drink is a small harm |
| **Throwing** | a potion with an `[object.throw]` table is a **flask**: thrown at a cell within its `range`, it flies as a missile, shatters where it comes down (on the first creature in its way, or the cell aimed at), and its **splash is a spell cast at that cell by nobody**, through the one cast path with a new `Pay::Flask` (as a scroll is `Pay::Scroll`): bursts, zones, riders, the spell types' rules, the effect shown, and what perception reports all come as they are. No to-hit roll: a flask is aimed as a burst spell is, not swung. §3, §5 |
| **What the splashes are** | existing spells where one fits, at a fixed rung: Fireball, Frost, Reek (stench), Mirk (darkness), Slick (oil), Sleep, Web (tar), Fear, Silence, Shatter (a blasting flask), Thunderclap (a self-shaped spell cast where the flask lands: a thunderstone), Rebuke (holy water), Hex of weakness and Curse of sloth (ray spells at radius 0). **Three new spells** for what no spell does: **Fume** (a smoke zone no eye sees through: a new `ZoneKind::Smoke`), **Flare** (a flash that stuns those it reaches), **Acid splash** (an acid burst that also eats one worn thing of each it harms). §4.4, §9 |
| **Drinking a flask** | every flask has an `[object.drink]` of kind `harm` as the murky draught has: a few degrees of one status (Burning, Corroded, Poisoned, Drowsy, Stunned, Frightened, Weakened, Sluggish, Silenced), never hit-point dice; the worst costs about 9 hit points over a minute and lies at item level 9. Unpleasant, rarely deadly, never a run-ender at full health; the type is learned as today. §6 |
| **New draught kinds** | `status` (lends any status that ends: Barkskin, Glamoured, Renewing, Quick-sensed, Invisible, Water-borne, Lit), `nourish` (a meal in a bottle), `restore` (the scroll of restoration's cleanse), `blink` (the teleport trap's jump), `gamble` (one draw from a `data/outcomes/` table, the spring's machinery); `cure` gains a `clear_head` means. §4, §9 |
| **Colours** | the hue rule stands: each family a hue, the colour hints at the family, never the type. New hues: scarlet (fire), white (cold), lime (acid), ash grey (smoke), black (darkness), pink (sleep), olive (stench), colourless (oil), amber (tar), indigo (dread), slate (force and thunder), silver (light), bronze (wards), rose (renewal), opal (the unseen and the unplaced), sky blue (utility), mint (restoration), and **iridescent** for the gambles. `potion_labels.toml` needs at least 66 words (35 today, one per type). §4.5 |
| **Sources** | loot by `level` and `frequency` as today (the wave's frequencies sum to about 330 against today's 505: a potion find becomes about two thirds more likely, flasks the common half of it); the alchemist stocks them all; general stores and provisioners stock the cheap flasks too; three creature **bombers** (a kobold alchemist, a goblin firebomber, a cultist) throw flasks through `[creature.throw]`. §7 |
| **Determinism, saves** | a throw draws nothing of its own; the splash draws as its spell's cast does; a gamble draws one kind and one outcome. No new saved state but `ZoneKind::Smoke` (a zone as the four are) and the new statuses' absence (none: every status used exists). Saves pinned to the version as ever |
| **Slices** | eight PRs in three waves: S1 the flask rules (one PR, no new content), S2 the `status` kind and the helpful draughts; then S3 the new spells and the smoke zone, S4 the flasks on existing spells, S5 the fire, cold, and acid flasks; then S6 the utility kinds, S7 the gambles, S8 the bombers. §11 |

**Key choices, and why**

1. *A flask casts a spell where it breaks.* Every thrown effect the owner names (smoke, fire, frost, acid, sleep gas, oil, stench, flash, noise) is a burst, a zone, or a status on an area, which is exactly what the cast path does, with its riders, its types' rules (fire leaves a fire on the centre, cold freezes water and douses, force breaks doors), its effects, and its perception already written and tested. A second area-effect engine for potions would copy it. A scroll is a spell paid with paper; a flask is a spell paid with glass and a throw.
2. *No to-hit roll.* A flask is not a weapon: its burst forgives, and a roll against the creature aimed at would only make the splash land one cell nearer or farther. A burst spell's aim rule already answers "what if someone is in the way" (the first creature on the line catches it) and "how far" (the range). The owner's "how a miss lands" becomes: there is no miss, only who stands where.
3. *A drunk flask hurts a little, through a status.* The owner asked for a small risk. A status eases, can be treated (Heal), cured (an antidote), or quenched (water), and reads as "unpleasant"; hit-point dice would read as a trap. The harm scales with the flask's item level, so the depth you find it at is the risk you take.
4. *No new talent, no new key.* The wave extends the potions system and the throw command: potions appear in the throw chooser (`T`), the Consume list is unchanged, and the one abilities menu gains nothing. Thrifty drinker and Lasting draughts apply as they do (a flask thrown is never kept: it broke).
5. *Three new spells, not three special cases.* Fume, Flare, and Acid splash are classic low spells (Obscuring mist, Flare, Acid splash) a wizard would want anyway; as spells they get scrolls, wands, and the wiki's spell tables for free, and the flask code stays one rule. Q4 asks whether they are learnable or flask-only.

## 2. What exists today

| Thing | Where | What it gives this design |
|---|---|---|
| 35 potion types, 9 `Draught` kinds (`heal`, `cure`, `mana`, `elixir`, `harm`, `boost`, `vision`, `haste`, `resistance`) | `content/potion.rs`, `assets/data/objects/potion_of_*.toml` | the object shape (`!`, `potions`, glass, 250, stackable, `level`/`frequency` loot), colours by hue, labels, `known_scrolls`, marks, Thrifty drinker |
| the murky draught | `murky_draught.toml` | the one bad potion: `harm`, Poisoned 2, brown, no marks; the model for every flask's drink |
| the drink path | `world/health/potion.rs::drink_potion` via `World::eat` | priced, one taken, "You drink the …", the type learned, the kind's effect, marks |
| throwing | `world/combat/shoot.rs::throw`, `loose`, `fly`, `come_down`; `ObjectDef::can_throw` (a throwing weapon or an improvised one: `attack` and no `range`, `launches`, `ammo`); `IMPROVISED_RANGE` 3, `IMPROVISED_TO_HIT` -4 | a potion has no `attack`, so today "You cannot throw a fizzy red potion."; `Event::Missile` flies any object; `breaks` is one draw on a hit |
| the cast path | `world/magic/cast.rs`: `Cast::settle(caster, content, (spell, def), Pay, aim)`, `perform`, `spell_lands`, `cast_single`; `Pay::{Mana(i32), Scroll(item, rungs), Wand(item, rungs), Creature}` | a scroll's read is `read_cast`: `cast_aim` (the burst's centre stops at the first creature in the way, `read.rs::burst_center`), a fixed `rungs`, `perform` |
| zones | `world/magic/zone.rs`, `ZoneKind::{Fire, Fouled, Slick, Dark}`, `LevelState::zones`, `zone_memory`, `zone_turn_start`, `zone_tread`, `darkened`/`shade` | Reek, Mirk, Slick, Balefire lay them; the dev `zone:`; words in `ui/look.rs::zone_words`, tint `ui/zone_tint.rs` |
| spell types | `data/spells.toml` `[types]`: fire's Burning rider and the fire it leaves, cold's Sluggish and `douses`/`frost`, force's `shatter`/`doors`, acid's Corroded, poison's Poisoned | what a flask of each type does beyond its dice, for free |
| statuses | `data/status.toml`: Burning 3/30 s, Corroded 3/12 s, Poisoned 10/120 s, Drowsy 3/12 s (cost 300), Stunned (per action, flat-footed), Frightened 3/6 s (flees), Weakened 3/120 s, Sluggish 3/60 s, Silenced (10 s), Webbed 6/6 s, Barkskin, Glamoured, Renewing, Quick-sensed, Invisible, Water-borne, Lit, Seeing | every status this wave lends or inflicts exists; none is added |
| item damage | `World::harm_item(at, ItemHarm::Acid)`, `withstands`, protection | the acid flask's bite on gear |
| hearing | `noise::make`, `SoundKind` (Explosion, Thunder, Banging, …), a spell's `cast.noise = { sound, loudness }` | what a shattering flask is heard as; Thunderclap's 150 |
| outcomes | `data/outcomes/{spring,forgotten_altar}.toml`, `content/outcome.rs`: `kinds` by weight, `[[outcome]]` with `kind`, `tier`, `weight`, `effect`, `message` | the gamble draughts' tables |
| the teleport trap | `world/fixtures/trap.rs::teleport(victim, at, events)` | the blink draught |
| creature throws | `[creature.throw]` (`content/throw.rs`: `object`, `damage`, `range`, `verb`, …), `World::hurls_at`, `creature_hurls` | the bombers: a throw whose `object` is a flask |
| the throw chooser | `T` (`ui/object_screen.rs`, the list of what can be thrown) | where flasks appear |
| the wiki | `xtask/src/wiki/potions.rs` → `wiki/src/potions.md` | the table that gains a Thrown column |

## 3. The one idea: a flask is a scroll you throw

A potion whose file has an `[object.throw]` table is a **flask**. The table:

| Field | Meaning | Default |
|---|---|---|
| `spell` | the spell its splash casts, an id of `data/spells/` | required |
| `rungs` | the rung of the spell's ladder it casts at (as a scroll's `rungs`) | 0 |
| `range` | how far it is thrown, in cells (`THROW_RANGES`, 2 to 20) | 6 |
| `noise` | what its shattering is heard as, `{ sound, loudness }`, in place of the spell's `cast.noise` (a flask does not chant) | `{ sound = "shatter", loudness = 30 }` |
| `words` | the splash's line for the thrower, in place of the spell's `cast.message` ("Smoke billows from the shards.") | the spell's `cast.message` |

`content/potion.rs` resolves it to `Flask { spell, rungs, range, noise, words }` on `ObjectDef::flask`, checked as `drink` is: the spell must exist and be of a shape a cell can hold (`burst`, `self`, `ray`, `line` cast from the landing cell toward the thrower's aim; never `cone`, Q7), a potion with a `throw` table has a `drink` table too (a flask is still a potion: §6), `range` in `THROW_RANGES`.

The throw: `World::throw` accepts a flask stack where it accepts a weapon (`can_throw` is `is_thrown() || is_improvised() || flask.is_some()`; `throw_range` gives the flask's `range` plus the thrower's `throw_range` talents and item effects as any throw gets). It is aimed as a burst spell is (`cast_aim` with the flask's spell: within range, in sight; the burst's centre is the first creature on the line or the cell aimed at), priced as a throw (`attack_price(Throw)`, so stance and burden count), the flask leaves the stack, flies as an `Event::Missile` of its object (the `!` glyph on the free-glyph layer, as any thrown thing), and **shatters where it comes down**: nothing lands (no `come_down`, no `breaks` draw), and `Cast::settle` with `Pay::Flask(object)` casts its spell at that cell at its `rungs` through `perform`. The splash's `by` is the thrower (a kill is theirs; a zone's `by` too), its roll is the spell's own at that rung with no caster hook (no Empower, no reroll, no school marks: glass, not skill), and the spell types' rules run as for any cast. A flask thrown by the player marks nothing (no weapon skill: it is not an attack).

Why this and not `[object.splash]` with its own `damage`, `radius`, `status`, `zone` fields: those are a `[[spell]]`'s fields, and the code that applies them is the cast path. The three effects no spell has (smoke, flash, acid splash) are written as spells (§4.4), so there is one rule.

## 4. The potions

Columns: *Drink* is the `[object.drink]`; *Throw* the `[object.throw]` (`—`: thrown, it only breaks, §5); *L/f* item level and frequency; *Value* gold; *Colour* the type's (§4.5). Severities and dice are data, hidden in the game; the wiki states them. Marks follow today's rule (15 for a common drink, on the stat of its family; none for a harm).

### 4.1 Helpful draughts (kind `status`, new: lends any status that ends)

| Id, name | Drink | Throw | L/f | Value | Colour |
|---|---|---|---|---|---|
| `potion_of_barkskin`, potion of barkskin | Barkskin 2 (Dodge +2, two minutes); marks Toughness | — | 3/15 | 100 | bronze |
| `potion_of_glamour`, potion of glamour | Glamoured 2 (Dodge +3, two minutes); marks Presence | — | 5/12 | 150 | bronze |
| `potion_of_renewal`, potion of renewal | Renewing 2 (hit points back quicker, four minutes); marks Toughness | — | 4/15 | 120 | rose |
| `potion_of_the_quick`, potion of the quick | Quick-sensed 3 (every living thing within 8 you do not see felt as a heartbeat, three minutes); marks Willpower | — | 5/10 | 150 | opal |
| `potion_of_the_unseen`, potion of the unseen | Invisible 1 (two minutes: unseen by what needs eyes); marks Dexterity | — | 7/6 | 300 | opal |
| `potion_of_sea_legs`, potion of sea legs | Water-borne 3 (walk on water, six minutes); marks Dexterity | — | 4/10 | 100 | sky blue |

### 4.2 Utility draughts (one new kind each)

| Id, name | Drink (kind) | Throw | L/f | Value | Colour |
|---|---|---|---|---|---|
| `potion_of_sustenance`, potion of sustenance | `nourish`: a meal's satiation at once (`satiation = 1500`, a ration's; `World::satiate`, the eat path's add, never past Full; no corpse rules); marks Toughness | — | 2/20 | 50 | sky blue |
| `potion_of_restoration`, potion of restoration | `restore`: the scroll of restoration's cleanse (drained points and stat damage back, `world/magic/read.rs::read_cleanse`'s core lifted into `World::cleanse`, two callers); marks Willpower | — | 6/8 | 250 | mint |
| `potion_of_blinking`, potion of blinking | `blink`: you are elsewhere on the level at once (`World::teleport`, the teleport trap's: a random walkable cell out of sight); no marks (it is a gamble you choose) | — | 4/12 | 80 | opal |
| `potion_of_clear_head`, potion of clear head | `cure` with `cures = ["clear_head"]`: a new means in the `cured_by` of Frightened, Drowsy, Sluggish, Stunned, Staggered, Transfixed (the clouded mind and the numbed body); marks Willpower | — | 3/15 | 80 | pale green |

### 4.3 Gambles (kind `gamble`, new: one draw from a `data/outcomes/` table)

| Id, name | Drink: the table | Throw | L/f | Value | Colour |
|---|---|---|---|---|---|
| `draught_of_chance`, draught of chance | `outcomes = "draught_of_chance"`: `kinds = { boon = 50, nothing = 20, bane = 30 }`; boons: `heal = "4d8"` (30), `mana = "half"` (15), a boost status at 1 (`status = { id = "fortified", severity = 1 }`, one entry per boost, 5 each, 30); nothing: "It tastes of nothing much."; banes: Poisoned 2 (15), Drowsy 2 (10), Weakened 1 (5) | — | 2/20 | 30 | iridescent |
| `trolls_blood`, troll's blood | `outcomes = "trolls_blood"`: one kind, one outcome: Renewing 3 **and** Poisoned 2 at once (an outcome with two `effect`s: the regeneration is real, so is the sickness); marks Toughness 15 | — | 5/10 | 120 | iridescent |
| `brewers_blunder`, brewer's blunder | `outcomes = "brewers_blunder"`: `kinds = { boon = 45, nothing = 10, bane = 45 }`; boons: a random greater boost (severity 2, 6 entries); banes: Drained 1 (a drained point, the scroll of restoration's or the potion's to undo), 6 entries, one per stat's drain | — | 6/8 | 100 | iridescent |

A gamble's draws: one for the kind, one for the outcome by weight, as the spring's drink draws (`content/outcome.rs`; `exhaust = 0`, nothing leans). The outcome `effect` kinds the tables need that the spring's and altar's lack: `status = { id, severity }` and `drain = N` (one `OutcomeEffect` variant each, applied through `World::afflict` and the drain path the scroll of restoration undoes). No anti-farming: drink as many as you find.

### 4.4 Flasks (kind `harm` to drink; the throw is the point)

The *Drink* column is the harm (§6 has what each costs). *Splash* is the spell cast where it shatters, at the rung named; *Area* and *Lasts* are that spell's at that rung.

| Id, name | Splash (spell, rung) | Area, lasts | Drink (harm) | L/f | Value | Colour |
|---|---|---|---|---|---|---|
| `flask_of_smoke`, flask of smoke | **Fume** (new, §9.3), 0 | burst, radius 2; a **smoke** zone 15 turns: no eye sees into or through it, light passes; those inside see their own cell's neighbours | Stunned 1 ("You choke on it.") | 2/20 | 40 | ash grey |
| `flask_of_oil`, flask of oil | Slick, 0 | burst, radius 1; a slick zone 20 turns | Poisoned 1 ("Lamp oil. You retch.") | 2/20 | 25 | colourless |
| `flask_of_brilliance`, flask of brilliance | **Flare** (new), 0 | burst, radius 1; Stunned 1 on all it reaches with eyes (`only_sighted`, a new rider filter beside `only_tags`: not the eyeless, Q8); a bright flash shown (`effect = "flare"`, a white `shatter`-like burst with a `light`) | **not a harm**: `status` Lit 1 (half an hour of light about you, the Light cantrip's); marks Willpower | 2/20 | 40 | silver |
| `sleeping_draught`, sleeping draught | Sleep, 0 | burst, radius 1; Drowsy 2 | Drowsy 3 ("Your eyelids droop.") | 3/15 | 60 | pink |
| `stinking_flask`, stinking flask | Reek, 0 | burst, radius 2, 2d4 poison; a fouled zone 10 turns | Poisoned 3 ("Foul beyond words.") | 4/12 | 60 | olive |
| `flask_of_tar`, flask of tar | Web, 0 | burst, radius 1; Webbed 3 | Poisoned 1 ("It glues your teeth, and your stomach turns.") | 4/10 | 60 | amber |
| `vial_of_acid`, vial of acid | **Acid splash** (new), 0 | burst, radius 1, 2d4 acid; Corroded 1 on those harmed (the type's rider); one worn thing of each harmed tested against acid (`harm_item`, §9.3) | Corroded 2 ("It sears your throat.") | 4/12 | 80 | lime |
| `flask_of_enfeeblement`, flask of enfeeblement | Hex of weakness, 0 | ray at radius 0: whoever stands on the cell; Weakened 1 (two minutes) | Weakened 2 ("Your arms go slack.") | 4/10 | 70 | indigo |
| `flask_of_sloth`, flask of sloth | Curse of sloth, 0 | the cell's creature; Sluggish 1 | Sluggish 2 ("Everything slows.") | 4/10 | 70 | indigo |
| `flask_of_darkness`, flask of darkness | Mirk, 0 | burst, radius 2; a dark zone 30 turns | Hexed 2 (Ill-wished: to-hit -2, two minutes; "Your sight swims.") | 5/10 | 90 | black |
| `vial_of_nightmares`, vial of nightmares | Fear, 0 | burst, radius 1; Frightened 2 | Frightened 2 ("Dread floods you.") | 5/8 | 90 | indigo |
| `vial_of_silence`, vial of silence | Silence, 0 | burst, radius 1; Silenced 1 (no cast, read, or song for 10 s) | Silenced 1 ("Your voice is gone a moment.") | 5/8 | 90 | black |
| `flask_of_thunder`, flask of thunder | Thunderclap, 0 | `self`-shaped, cast at the landing cell: radius 1, 2d6 force, Stunned 1; **loudness 150, heard over the level** (a decoy anyone can throw: creatures that hear it and see no enemy go to it, as to a thrown voice) | Stunned 1 ("Your ears ring.") | 5/8 | 100 | slate |
| `flask_of_fire`, flask of fire | Fireball, 0 | burst, radius 2, 4d6 fire; Burning 1 on one in four; a fire left on the centre (the type's rule) | Burning 1 ("It burns all the way down!") | 6/10 | 120 | scarlet |
| `flask_of_frost`, flask of frost | Frost, 0 | burst, radius 2, 4d6 cold; Sluggish 1 on those harmed; water frozen, fires doused | Sluggish 2 ("Cold spreads through you.") | 6/10 | 120 | white |
| `blasting_flask`, blasting flask | Shatter, 0 | burst, radius 1, 4d6 force; shut doors and chests it reaches broken, thin ice shattered; loudness 100 | Stunned 1 ("It goes off in your gut.") | 7/6 | 200 | slate |
| `holy_water`, holy water | Rebuke, 0 | burst, radius 2, 4d6 fire to the undead only, Frightened 3 on them; nothing to the living (the spell's `only_tags`) | **not a harm**: `cure` with `cures = ["holy_water"]`, a means added to Ill-wished's and Branded's `cured_by` (hexes washed off); marks Willpower | 5/8 | 100 | silver |
| `pitch_fire`, pitch fire | **Pitch fire** (new, §9.3) | burst, radius 1, 2d6 fire; a fire zone 6 turns (Balefire's zone in a burst) | Burning 2 ("Liquid fire.") | 9/6 | 250 | scarlet |

Why these fourteen splashes on existing spells: each is a tactic the owner named or its neighbour. Smoke, darkness, and thunder answer archers (hide, blind them, pull them off their line). Oil, tar, sleep, and sloth hold a corridor. Fire, frost, acid, and blasting are the bombs. Enfeeblement and nightmares soften a big foe. Silence is a flask for the summoning hall. Holy water is the one every priest's player asks for. Brilliance is the two-way one: light when drunk, a flash when thrown.

### 4.5 Colours

The hue rule (spec §8 "Potions") stands: one hue per effect family, the colour hinting at the family, never the type; potions of one colour share one `fg`. New families and their hues (`fg` first guesses, kept in data):

| Family | Hue | `fg` |
|---|---|---|
| fire (flask of fire, pitch fire) | scarlet | `#e04020` (healing is `#c83c3c`: the two are kin, and that is the gamble the owner asked for) |
| cold (frost) | white | `#e8ecf0` |
| acid | lime | `#a0d020` |
| smoke | ash grey | `#8a8a86` |
| darkness, silence | black | `#303038` |
| sleep | pink | `#e890b0` |
| stench | olive | `#787830` |
| oil | colourless | `#c0c8cc` ("a fizzy colourless potion") |
| tar | amber | `#d0a020` (orange, the boosts', is `#d89a30`: close, by design; a tar flask is the boosts' bad cousin) |
| dread, hexes (nightmares, enfeeblement, sloth) | indigo | `#4830a0` |
| force (blasting, thunder) | slate | `#607080` |
| light, the holy (brilliance, holy water) | silver | `#d8dce8` |
| wards (barkskin, glamour) | bronze | `#b07838` |
| renewal | rose | `#e07890` |
| the unseen and the unplaced (quick, unseen, blinking) | opal | `#b8d0e0` |
| utility (sustenance, sea legs) | sky blue | `#70b0e0` |
| restoration | mint | `#90e0b0` |
| gambles | iridescent | `#c0a0e0` (Q3) |

`potion_labels.toml` has 35 words for 35 types; the wave needs 66, so 31 more words go in (the check `check_potion_labels` fails until they do): brackish, briny, chalky, clotted, curdled, dusty, effervescent, frothy, gelid, glassy, gritty, inky, lumpy, lustrous, mottled, muddy, murky (the murky draught is a name, not a label: fine), opaque, powdery, ropy, sandy, scummy, silty, slimy, sludgy, speckled, steaming, sticky, streaked, turbid, watery.

## 5. Throwing a potion

| Question | Answer |
|---|---|
| **How** | `T`, the throw chooser, lists flasks beside the throwing weapons (a potion without a `throw` table is listed too, marked as any improvised thing is: Q1); the aiming mode opens at the flask's range with the burst shape of its spell shown as a spell's is (`AimKind::Throw` already draws the line; it gains the spell's area as `AimKind::Read` has). Action text: `throw:c:X,Y`, as any throw |
| **Range** | the flask's `range` (6 by default; 4 for the heavy pitch fire: data) plus the thrower's `throw_range` (talents, item effects), as every throw |
| **Where it bursts** | the burst's centre by the cast path's aim rule (`burst_center`): the first creature on the line from the thrower to the cell aimed at catches it and it bursts on that creature's cell; else the cell aimed at (sight must reach it, as for a spell). A `self`-shaped splash (Thunderclap) is centred there; a `ray` splash takes the cell's creature, if any, else nothing happens but the shattering |
| **A miss** | there is none: no to-hit roll, nothing to dodge. What there is: the burst's area, and whoever stands in the way. Q2 asks whether a flask thrown farther than half its range should scatter one cell on a failed Dexterity check; the default is no |
| **What lands** | nothing: the flask shatters (no `come_down`, no `breaks` draw). A flask thrown into water shatters on the water, and the splash's spell does what it does there (fire's fire is not laid on water; frost freezes it) |
| **The noise** | the flask's `noise`: a crash of glass at 30 by default, a new `SoundKind::Shatter` ("a crash of glass"; `j` in the sound layer, after the creature sounds), replacing the spell's `cast.noise` (a flask does not chant). Thunder and blasting flasks set theirs (150 thunder, 100 explosion) |
| **The effect shown** | the spell's `cast.effect` at the landing (fireball's fire, frost's burst, blight's cloud); Fume's and Flare's new entries in `effects.toml` are brief and dim (the subtle rule). The flight is the `!` on the free-glyph layer, as a thrown dagger's |
| **What the player is told** | "You throw the fizzy grey potion." (by the name they know), then the splash's `words` ("Smoke billows from the shards.") when they see the landing cell, and for an unknown type **"It was a flask of smoke."** and the type learned (`learn_scroll`), **only when the player sees the burst's centre** as it lands (the splash's event is in their perception). Thrown blind into the dark or round a corner, nothing is learned: "You hear glass break." |
| **Messages for those hit** | the spell's own: "The kobold is burnt.", "The goblin archer is caught in the smoke." (a zone's words), as a scroll's cast says them |
| **Thrown by a creature** | a creature type with `[creature.throw] object = "flask_of_fire"` (and no `damage`: the flask's splash is the attack) hurls flasks: `creature_hurls` sees a flask object and, instead of `fly`'s attack, flies the missile and casts the splash at where it comes down with `Pay::Flask`, the creature as `by` (its kill). The AI's `hurls_at` is unchanged (range, clear shot, not adjacent); a bomber keeps its distance as an archer does. A creature's flask is the boulder's kind of throw: not from its pack, endless (Q9). The player sees "The kobold alchemist hurls a flask." and the splash, and learns the type on seeing it, as for their own. A flask in a creature's **pack** (its gear line, a drop) is never thrown and never drunk in this wave: the pack-reading AI is a later wave (Q10) |
| **What is refused, with no time** | on the overworld ("You cannot do that on the overworld.", `can_loose_here`), out of range, at oneself, a cell out of sight (as a spell), a flask held while Engulfed or Swallowed (`blocks = ["throw"]`, as today) |

## 6. Drinking the dangerous ones

Every flask is a potion: it has a `[object.drink]` and the Consume list offers it. Drinking one (`drink_potion`) is today's `harm` kind: "You drink the fizzy scarlet potion. It is a flask of fire. It burns all the way down!", the type learned, `dice` degrees of its `status` afflicted, no marks. Nothing about the drink path changes but the data.

The risk, in numbers (the owner wants it small; the wiki states these, the game says only the words):

| Flask | Drink | What it costs, roughly | Ended by |
|---|---|---|---|
| smoke, brilliance's cousins thunder, blasting | Stunned 1 | the next action half again as slow and flat-footed until it is taken; no hit points | the next action |
| oil, tar | Poisoned 1 | 1 hit point over two minutes | antidote, Heal, time |
| sleeping draught | Drowsy 3 | 36 seconds at a third of your speed; no hit points | clear head, time |
| stinking flask | Poisoned 3 | about 6 hit points over six minutes; never the stat penalty (from 4) | antidote, Heal, time |
| acid | Corroded 2 | about 6 hit points over 24 seconds | time (it eases fast) |
| enfeeblement, sloth | Weakened 2, Sluggish 2 | Strength and Dexterity -2 for four minutes; or two minutes at two thirds speed | clear head (sloth), time |
| darkness | Hexed 2 | to-hit -2 for two minutes | holy water, time |
| nightmares | Frightened 2 | to-hit -3 for 12 seconds (the player does not flee: `flees` moves creatures) | clear head, time |
| silence | Silenced 1 | no spell, scroll, or song for 10 seconds | time |
| fire | Burning 1 | 3 hit points over 30 seconds; a step into water ends it | water, time |
| frost | Sluggish 2 | two minutes at two thirds speed | time |
| pitch fire | Burning 2 | about 9 hit points over a minute | water, time |

Three guards, by data and one rule:

1. **No hit-point dice on a drink.** `harm` has none today and gets none: every cost comes through a status that ticks, so it is slow, visible on the status bar, and can be ended. The content check holds it (a `harm` with `damage` is refused).
2. **The harm scales with the depth.** The flask's item level is the least level its loot appears at: a level-1 character meets the oil and smoke flasks (Stunned, Poisoned 1); Burning 2 lies at level 9, where a character has fifty hit points and more. The table's worst case at level 1 is about 3 hit points (an early flask of fire found shallow by luck: Burning 1, and water is near).
3. **A status can kill, and this design does not change that** (Poisoned at 1 hit point does what it does): a chugger at 2 hit points takes the risk the owner asked for. Q5 asks whether a drink's status should never take the last hit point; the default is no guard, ADOM's honesty.

Identification by drinking is as today: the type is learned whatever happens, and the item description then says both halves: "Thrown, it bursts into choking smoke. Drunk, it is a bad idea." The flask of brilliance and holy water have **good** drinks (Lit; a hex washed off) and are the safe ones in the silver family: the hue rule says the silver family is kind.

## 7. Where they come from

| Source | Rule |
|---|---|
| **Loot** | by `level` and `frequency` as every potion (`magic::loot::roll`, never magic). Today's 35 types sum to about 505 frequency; the wave's 31 sum to about 330, flasks about 190 of it. So a potion is about two thirds more likely to be the item rolled, and a found potion is a flask about one time in four. Q6 asks whether to trim today's frequencies a little instead (the boosts' 20 each, the visions' 15) to keep potions as common as now; the default is to accept more potions: the wave adds 31 reasons to find one |
| **Shops** | the **alchemist** stocks and buys them all (the `potions` class; healing first as today). **General stores and provisioners** stock the class by frequency as today, so they carry the cheap flasks (smoke, oil, brilliance, sleeping draught) and the common draughts; their `never` list gains the gambles and the costly flasks (fire, frost, blasting, pitch fire, thunder), which only an alchemist lays out, beside the elixirs already named there. Hand-made shops (Cēaptūn) unchanged; its alchemist, if it has one, may name a flask of smoke in its `always` |
| **Creatures: bombers** | three roster entries, one per tribe file, each a `[creature.throw]` whose `object` is a flask, `range` 6, `verb = "hurls"`: a **kobold alchemist** (kobolds, danger 3: `flask_of_oil`; a slick corridor is the kobold trick), a **goblin firebomber** (goblins, danger 6: `flask_of_fire`), a **cultist** of the summoning hall (danger 8: `stinking_flask`). Each drops a flask or two of its kind (a gear line), so the player gets the weapon that was used on them. The tribe-roster rule (owner): the tribe PRs decide the details, not this design |
| **Creature packs** | a hedge witch, a cultist, an apothecary carry a draught or two (gear lines), as the first wave's design said and the rosters have not yet done; never thrown or drunk by them (§5) |
| **Quest rewards, vaults, piles** | by id as today: the still room vault gains a flask of smoke and a potion of barkskin; the hub's pile a flask of brilliance (the safe one to learn throwing with); the trap hall's pile beside the wands a blasting flask (a locked chest stands there to break) |
| **The forgotten altar** | the great boon `elixir` is unchanged; a new minor boon names `potion_of_restoration`, a minor bane leaves a `brewers_blunder` at your feet |

## 8. What the player sees, and the wiki

- **The Consume list** is unchanged: every potion is there, flasks too, by the name known. **The throw chooser (`T`)** lists flasks (and, Q1, other potions as improvised throws) under their class heading, as it groups weapons. The aiming mode shows the flask's range and its spell's area.
- **The item description** (`magic/describe.rs::potion_lines`): a known flask gets a second line for the throw, in words: "Thrown, it bursts into choking smoke that no eye sees through." / "Drunk, it is a bad idea." An unknown one keeps "You do not know what this potion does." Never dice, radii, or turns.
- **Look mode** names a smoke zone "thick smoke" (`zone_words`), tinted grey (`zone_tint`), remembered as the four zones are; a cell in smoke that the player does not see shows by the smoke itself as a dark cell shows by its dark (`PerceptionView::zone_at`).
- **The status bar and character screen** show the lent statuses as they show Blessed; nothing new there.
- **Messages**: §5's and §6's lines; a creature's hurl through `hurl::throw_messages`; a gamble's outcome `message`.
- **Dumps**: the `inventory` layer marks a flask's throw as it marks a wand's rung: `a flask of smoke (throw 6)` once known; `truth`'s zone lines gain `smoke`; JSON stacks gain `throw: { range }` when known. The dev `zone:KIND` gains `smoke`.
- **Text snapshots** (`ui/screen_snapshots.rs`): `throw-list-flasks`, `potion-description-flask`, `aim-flask` (the area shown), `look-smoke`; no golden (the flight and the effects are the existing ones; Fume's and Flare's effects are checked by eye in the PR, `--capture`, and not kept).
- **The wiki** (`cargo xtask wiki`): `potions.md`'s generated table gains **Thrown** (the splash in a sentence, the spell linked, the radius and turns exact) and **Drink** says the harm exactly ("Burning 1: 3 hit points over 30 seconds"); a new section "Flasks" explains throwing in prose (every term linked, `TERMS.md`); `status-effects.md`'s zone table gains smoke; the three new spells appear in the magic tables as any spell; `shops.md` names what the alchemist alone sells. The game stays vague, the wiki exact.

## 9. Rules and code impact

### 9.1 Content

| Change | Where |
|---|---|
| `[object.throw]` → `ThrowSpec` → `Flask { spell, rungs, range, noise, words }` on `ObjectDef::flask`; checks (§3); a flask must also `drink` | `content/potion.rs` (the one potion module), `content/object.rs` (`can_throw`, `throw_range`) |
| `Draught::Status { status, severity }` (any status; the loader checks only that it resolves, #1205), `Draught::Nourish { satiation }`, `Draught::Restore`, `Draught::Blink`, `Draught::Gamble { outcomes }` (the file must exist and every outcome's effect be one a drink may do) | `content/potion.rs` |
| `cured_by` means `clear_head` (Frightened, Drowsy, Sluggish, Stunned, Staggered, Transfixed) and `holy_water` (Ill-wished, Branded) | `data/status.toml` |
| `OutcomeEffect::Status { id, severity }`, `::Drain(n)` | `content/outcome.rs`, `world/fixtures/outcome.rs` |
| three spells (§9.3), their scrolls (`scroll_of_fume`, `_flare`, `_acid_splash`, by the every-spell-has-a-scroll rule, Q4), two effects (`fume`, `flare`) | `data/spells/`, `data/objects/`, `data/effects.toml` |
| 31 potion files; 31 label words; the shops' `never` lists; three outcome tables | `data/objects/`, `data/potion_labels.toml`, `data/shops.toml`, `data/outcomes/` |

### 9.2 Rules

| Hook | One line each |
|---|---|
| `World::throw` | a flask stack passes `can_throw`; aimed with `cast_aim(flask.spell)`; `player_looses` flies it; at the landing `shatter_flask(by, object, cell)` instead of `come_down` |
| `World::shatter_flask(by, object, at, events)` (new, `world/health/flask.rs`) | the flask's noise, the `Event::Missile` cut as any, then `Cast::settle(by, …, Pay::Flask(object), aim at the cell)` and `perform`; the words for the thrower |
| `Pay::Flask(ObjectId)` | `pay_cast`: nothing to pay (the flask is gone); `spell_landed`: no marks, no `read` cell; the roll hooks read nothing of the thrower (`Cast::settle` takes a `caster` for the stats: a flask's cast uses the spell's rung alone, so `settle` gains a `by_flask` constructor beside `by_creature`) |
| `creature_hurls` | a throw whose object is a flask flies and shatters (the one branch), the creature as `by` |
| `ZoneKind::Smoke` | laid as the four are; `LevelState::is_opaque` (the one rule the player's `perception.rs` and a creature's `sight.rs` ask through their `is_opaque` closures) answers true on a smoke cell for everyone, so no sight reaches into or across it, and a viewer inside sees only the cells beside them (the FOV's first ring, as a viewer standing in a wall would); light passes (not `darkened`); `zone_tread` and `zone_turn_start` do nothing for it; Gale clears it (`[Fire, Fouled, Dark, Smoke]`), rain does not |
| `Draught::Status` | `World::lend`, the four lending kinds' path, generalised to any status that ends (the check is content's) |
| `Draught::Nourish` | `World::satiate(player, n)`, the eat path's add |
| `Draught::Restore` | `World::cleanse(player)`, lifted from `read_cleanse` |
| `Draught::Blink` | `World::teleport(player, at, events)`, the trap's |
| `Draught::Gamble` | `World::draw_outcome(table)` → `apply_outcome`, the spring's, with the two new effects |
| acid's `corrodes` (§9.3) | `spell_type_lands`: a creature harmed by acid has one worn stack tested with `harm_item(ItemHarm::Acid)` (one draw by `withstands`; the player's own gear when a creature's acid harms them: the vial thrown at you) |
| `only_sighted` rider filter | `spell_lands`: the rider skipped for a creature type with no eyes (`CreatureDef::sightless`, the ones with `vision = "none"` or the `eyeless` tag, whichever exists: checked in the slice) |

Determinism: a throw draws nothing (no to-hit); the splash draws as its spell's cast (fire's rider chance per creature, Slick's step checks); a gamble two draws; a drink's `harm` its `dice`. Saves: no new fields but the `Smoke` variant of a zone's `kind` (pinned to the version, as every save change). Replays: `throw:c:X,Y` is the existing text; `eat:c` the drink. Hashes of runs that never throw a flask are unchanged.

### 9.3 The three new spells and the acid rule

| Spell | Data | Why a spell |
|---|---|---|
| **Fume** | level 4, conjuration (or transmutation, whichever school the smoke fits: the slice decides with the schools' list), `kind = "hex"`, burst, range 8, radius 2, `effect = { zone = { kind = "smoke", turns = 15 } }`; grows: radius 3 at Willpower 18; `cast.effect = "fume"` (a grey, dim, slow-fading burst, no light, no shake) | Obscuring mist: every wizard's escape; the smoke flask is it in glass |
| **Flare** | level 2, evocation, `kind = "hex"`, burst, range 8, radius 1, `effect = { status = "stunned", severity = 1, only_sighted = true }`; `cast.effect = "flare"` (white, a `light` of radius 4 for its 0.2 s, no darken) | the cantrip Flare; the brilliance flask's throw |
| **Acid splash** | level 3, evocation, `kind = "damage"`, burst, range 8, radius 1, acid, 2d4; grows: Intelligence 13 2d6, 19 4d4; `cast.effect = "acid_arrow"`'s burst cousin (`acid_splash`, green, brief) | the cantrip; the vial's throw |
| **Pitch fire** | level 8, evocation, `kind = "damage"`, burst, range 6, radius 1, fire, 2d6, `effect = { zone = { kind = "fire", turns = 6 } }`; `cast.effect = "fireball"` | Balefire lays a line; a thrown pot of pitch lays a pool. Q11 asks whether to use Balefire's line instead and skip this spell |

**Acid eats gear**: `[types].acid` gains `corrodes = true`: a creature harmed by an acid spell has one worn or wielded stack (the first by slot order) tested with `World::withstands(ItemHarm::Acid)` and, failing, `harm_item` (damaged, then destroyed; protected items never). One draw per creature harmed, after the rider's. It makes Acid arrow and Acid splash the spells that ruin a knight's plate, which is what acid is for; the wiki's spell types table says so.

## 10. Rejected ideas

| Idea | Why not |
|---|---|
| **A to-hit roll for a thrown flask** (the Thrown weapons skill, misses that land a cell off) | a flask is not a weapon; the burst's area forgives already; a miss mechanic would need scatter rules, a flask that lands intact, and a reason the splash is weaker; the aim rule of a burst spell answers "who is in the way" |
| **`[object.splash]` with its own fields** (`damage`, `radius`, `status`, `zone`) | a `[[spell]]`'s fields, applied by the cast path; a second engine |
| **A healing potion thrown heals who it splashes** | charming, but it needs a `heal` cast at a cell that `cure_wounds` (a ray) is not, and a thrown potion of healing is a wasted one in every roguelike the owner leans on; a potion without a `throw` table breaks and is gone (Q1) |
| **Oils poured on a weapon or armour** (venom, sharpness, mending) | an `on_item` use as the scroll of identify's, a temporary affix, and a new item state: a wave of its own (`#196`, deferred ideas) |
| **A potion of fire breath, of flight, of polymorph, of forgetting the map** | each a system (a granted active, levitation, a changed creature type, memory erasure) this wave does not add |
| **Creatures drinking potions from their packs** | the pack-reading AI for creatures (and their drinking) is the wave after this (Q10) |
| **A `blind` status for the flash and the darkness drink** | no status worsens sight today (`vision` lends, never takes); a `blind` field on `StatusDef` read in `Actor::eyes` and `sight.rs` is a real sub-feature with its own hooks; Stunned (flat-footed, slow) is the flash's effect for now and reads as "dazzled" |
| **New talents** (Alchemist's arm: flasks fly farther; Iron stomach: half the harm) | the owner's house rule: none proposed here; the system tag `potions` exists with its two talents, and `throw_range` talents already reach flasks |
| **Flasks in the abilities menu, or a Throw potion key** | `T` is the throw command; one list, no new key, nothing in the abilities menu |
| **A guard that a drunk harm never takes the last hit point** | Q5; the default is honesty: it ticks as any status |
| **Brewing** (a Craft that makes potions) | a system of its own |

## 11. Slices

| Slice | What | Adds | Depends on |
|---|---|---|---|
| **S1** | **The flask rules**: `[object.throw]`, `Flask`, `Pay::Flask`, `World::shatter_flask`, the throw chooser and aiming area, learning on seeing the burst, the noise, the description line, the dump marks, the wiki column; tests with a fixture flask on an existing spell (`world/health/flask_tests.rs`: the throw, who is in the way, the splash's kills, unknown learned only when seen, refusals, a replay); **one content potion to prove it: the sleeping draught** (Sleep) | one action path (the existing `throw`); no saved state | — |
| **S2** | **`Draught::Status`** and the six helpful draughts (§4.1), the labels' words for them; `potion_status_tests.rs` grows | no actions, no saved state | — |
| **S3** | **Fume, Flare, Acid splash**, their scrolls and effects, **`ZoneKind::Smoke`** (the opacity hook, Gale, the look words, the tint, the dev `zone:smoke`), `only_sighted`, acid's `corrodes`; the flasks of smoke, brilliance, and the vial of acid | saved state (`Smoke`), three spells | S1 |
| **S4** | **The flasks on existing spells**: oil, stinking, tar, enfeeblement, sloth, darkness, nightmares, silence, thunder, blasting, holy water (and `holy_water`'s means); the `never` lists | no new rules; content and tests (each flask's throw and drink once) | S1 |
| **S5** | **Fire and cold**: flask of fire, flask of frost, **Pitch fire** (or Balefire's line, Q11) and its flask | one spell | S1 |
| **S6** | **The utility kinds**: `nourish`, `restore` (`World::cleanse` lifted), `blink`, `clear_head`; their four draughts | no actions | — |
| **S7** | **`Draught::Gamble`**, the two outcome effects, the three tables and draughts | no actions | — |
| **S8** | **The bombers**: `creature_hurls`'s flask branch, three roster entries in their tribe files (`docs/dev/creatures/<tribe>.md` each), their drops; `creature_flask_tests.rs` | no actions | S1, S4 or S5 (their flasks) |

Three waves: S1 and S2 first (S2 is independent and small); then S3, S4, S5, S6, S7 by overlap (S4 and S5 are content on S1; S6 and S7 touch `Draught` and should not run beside each other); S8 last. Each slice updates `docs/dev/potions.md` (a "Flasks" section in S1), the spec's "Potions" (and "Zones", "Spell types" where touched), `docs/dev/spells.md`'s zone list, and the wiki, as CLAUDE.md asks. No golden: Fume's and Flare's looks are checked by `--capture` in their PR and not kept.

## 12. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| 1 | A potion with no `throw` table: thrown as an improvised thing (it flies, shatters, "The potion shatters; the draught soaks into the floor.", gone) or refused ("You cannot throw it.") | **thrown and wasted**: one rule for the `!` class in the throw chooser, and a way to be rid of a known bad one at a distance; nothing is learned from a splash with no effect |
| 2 | A flask thrown past half its range: scatter one cell on a failed Dexterity check, or fly true | **fly true**: no draw, no scatter rule; the aim rule's "first creature in the way" is where throws go wrong |
| 3 | The gambles' colour: one hue of their own (iridescent), or the hue of the family each mimics (the draught of chance red, troll's blood rose, brewer's blunder orange), bending the hue rule on purpose | **iridescent**: the hue rule says a colour never lies; a gamble that looks like a healing potion would make every red potion a gamble, and red is the one colour a new player must trust |
| 4 | Fume, Flare, Acid splash (and Pitch fire): plain spells with scrolls that classes may learn, or flask-only (`flask_only = true`: no scroll, no learning, a new flag) | **plain spells**: one rule, four low spells a wizard wants, scrolls and wands for free; nothing in the system today is flask-only or scroll-only but by comment (Slick) |
| 5 | A drunk flask's status never takes the last hit point (a guard in the tick for statuses a drink gave) | **no guard**: it would need a mark on the status's dose; the risk is the owner's "small risk", and a character at 2 hit points who drinks an unknown potion is taking it on purpose |
| 6 | Potions more common (the wave's frequencies on top of today's), or today's trimmed to keep the class as common as now | **more common**: 31 new reasons to find one; flasks are consumables meant to be thrown, and the alchemist's shelves want filling; revisit after the balance probe |
| 7 | `cone`-shaped spells as splashes (Burning hands from the landing cell toward the aim) | **no**: a cone from a cell toward the aim's far side would reach where the thrower did not look; bursts, rays, and self-shapes are enough |
| 8 | Flare's stun on the eyeless (oozes, animated things): skipped by a rider filter, or everyone stunned | **skipped** (`only_sighted`): a flash is light, and a flask thrown at a mimic doing nothing is the right lesson; the filter is a sibling of `only_tags` |
| 9 | A bomber's flasks: endless (as a giant's boulders) or a supply (a count on the throw, saved) | **endless**, as the boulder: no new saved state, and the AI's throw cadence and range already limit it; a supply is a one-line field later if a bomber proves tiresome |
| 10 | Creatures drinking and throwing from their packs (a pack-reading AI, the dropped flask the player sees them pick up) | **not in this wave**: `#196`'s list; the bombers give the player the experience of being bombed without the AI |
| 11 | Pitch fire as a new spell, or the flask of pitch fire casting Balefire (a line of fire from the landing cell along the aim, length 5) | **a new spell**: a thrown pot of pitch lays a pool, not a line; and a burst-shaped fire zone is a spell the list lacks |
| 12 | The names: flask, vial, draught, potion mixed by feel (`flask_of_fire`, `vial_of_acid`, `sleeping_draught`), or every id `potion_of_*` | **mixed**: the names are the world's register; the class is `potions` and the glyph `!` either way, and the labels never show the name until the type is known |
| 13 | Holy water: a flask of Rebuke (fire to the undead, nothing to the living), or a cure drink only | **both**, as written: the undead-only burst is what every priest's player reaches for; the drink that washes a hex off is its kind half |
