# Undead roster

General conventions: `docs/dev/creatures.md`.

Grades (spec §8 "Grades", #1135): an undead type naming no `grade_words` of its own is "withered"
when lesser and "ancient" when greater, by its `undead` tag (`grade::TAG_WORDS`); its uniques
(the layer bosses, the named dead) have none.

The undead roster (#383; batch 1 #384, levels 1–10; batch 2 #385, 11–20; batch 3 #386, 21–30; batch 4 #387, 31–40; batch 5 #388, 41–55): shared types, one file each in
`assets/data/creatures/`, on the conventions noted in that directory's `README.toml`. Each batch
adds its types to `content/undead_roster_tests.rs` (`ROLES`, the type's role) and raises
`ROSTER_TOP`: every `undead` type must be on the XP guideline (`OFF_GUIDELINE` lists the pale
drinker), lie within ±35 % of #383's max HP and dice formulas for its level and role, and every
level from 1 to `ROSTER_TOP` must have one. Batch 1's population lines: Cealdhlāw the crawling
hand, skeletal rat, kobold zombie, skeleton archer, skeletal wolf, and skeleton warrior;
Fennmynster the corpse candle, poltergeist, ghast, shadow, and bog body; Fyrdelf the ogre zombie
and grave hound. Batch 2's: Fennmynster the wraith and fen ghoul; Fyrdelf the burning skull,
knocker, spectre, charred dead, duergar skeleton, and rotting troll. #531 placed the
rest up to level 17: Fennmynster the stone haunt (in its stones), the pale drinker, and the
mummy (its crypts); Fyrdelf the draugr, barghest, hand of glory, vampire spawn, and gallows dead,
the dead of its deep workings. The bone wyvern, gibbering shade, fetch, revenant, and mummy
priest (levels 18 to 20) and all of batches 3 to 6 (levels 21 and up) ship
in no population (deep undead wait for deep places, #383; until then they come only as strays, docs/dev/creatures.md), and the ghost is
incorporeal (wall Defence 2). The roster's holds (`held`, #431) are `{ status = "held", chance,
difficulty }`: the ghast's and fen ghoul's claws 10 % at 15, the gallows dead's tongue 20 % at 17, the lich's touch and the ghoul lord's claws 15 % at 18 (12 + level ÷ 5, as a drain's).

## Batch 4 (#387, levels 31–40)

All ten ship in no population (strays only). Each is one file in `assets/data/creatures/`:

| Level | Type | Notes |
|---|---|---|
| 31 | greater shadow | incorporeal, a cold touch that drains Strength |
| 32 | ghoul lord | quick, opens doors, claws that hold and carry filth fever |
| 33 | black shuck | a quick black dog that frightens |
| 34 | draugr lord | huge, all but untouched by cold, a great axe, sometimes magic |
| 35 | nosferatu | quick, opens doors, a bite that drains Toughness and carries filth fever, fire hurts it more |
| 36 | mummy lord | a caster whose fist carries mummy rot, wrappings with protection 3, something worth taking |
| 37 | screaming skull | small, its bite frightens |
| 38 | church grim | a quick grey dog that frightens, hardened to cold |
| 39 | white lady | incorporeal, a touch that transfixes and drains Presence |
| 40 | bone colossus | bone with protection 5 |

The greater shadow and white lady are spirits (`incorporeal`, wall Defence 5); the nosferatu's
plural is itself (`plurals.toml` has `colossus` and `lady`).

## Batch 5 (#388, levels 41–55)

All sixteen ship in no population (strays only). Each is one file in `assets/data/creatures/`:

| Level | Type | Notes |
|---|---|---|
| 41 | wraith lord | incorporeal, a touch that drains 2 Toughness |
| 42 | barrow king | a caster, drains 2 Toughness, an enchanted long sword, loot |
| 43 | sluagh | incorporeal, the fastest of the bulk spirits (speed 1250), claws that frighten |
| 44 | ghoul king | quick, opens doors, claws that hold and carry grave rot |
| 45 | death knight | black plate and an enchanted great sword, hardened to cold, loot |
| 45 | vampire lord | quick, a bite that drains 2 Toughness and a stare that transfixes, loot |
| 46 | nightganger | a huge brute whose blow drains 2 Toughness and may frighten |
| 47 | wormlord | a caster whose touch carries burrowing worms |
| 48 | washer at the ford | incorporeal, frightens and drains 2 Presence |
| 49 | night mare | incorporeal, transfixes and drains 2 Willpower |
| 50 | bone dragon | bone with protection 7 |
| 51 | drowned king | slow brute, hardened to cold, marsh ague |
| 52 | rotting dragon | slow brute, grave rot |
| 53 | ghost lord | an incorporeal caster, transfixes and drains 2 Willpower |
| 54 | archlich | a caster (range 10), holds and drains 2 Intelligence, loot, very rare |
| 55 | hound of the hunt | speed 1250 (the bulk's top), frightens |

Drains are two points from level 40 (difficulty 12 + level ÷ 5, Toughness's checked by
toughness); the spirits' wall Defence is 2 + level ÷ 10 (6 to 7). The ghoul king's hold is 20 % at
20, the archlich's 20 % at 22. Plurals: `archlich` and `sluagh` in `plurals.toml`; the washer's
and the hound's are written in their files.

The sluagh sneak-attacks (#686, spec §8 "Surprise and sneak attacks"): `sneak_attack = 120`
(5d6), `initiative = 60`, the host falling out of the night on the unwary. It has no vitals
(incorporeal), so nothing sneak-attacks it back. `bestiary_tests::the_sneak_attackers_carry_their_craft_and_two_lurk` checks the numbers.

## Batch 6 (#389, levels 56–70)

All fifteen ship in no population (strays only). Each is one file in `assets/data/creatures/`; the table is
#389's, with its numbers unmoved:

| Level | Type | Notes |
|---|---|---|
| 56 | mummy king | caster, fist with mummy rot (35 %), wrappings with protection 5, fire hurts it more |
| 57 | wild huntsman | quick on foot (speed 1250), spear or glaive, sometimes magic |
| 58 | elder vampire | drains Toughness 2, transfixes |
| 59 | dread lord | plate and a great sword (magic 75), frightens |
| 60 | deathshade | incorporeal, drains Strength 2, frightens |
| 61 | nightwing | very quick, drains Toughness 2 |
| 62 | dread wyrm | bone, protection 9 |
| 63 | nightworm | brute, drains Toughness 2, holds |
| 64 | lich-drake | bone with protection 8, cold 75, drains Intelligence 2 |
| 65 | ankou | incorporeal, scythe, drains Toughness 2, frightens |
| 66 | deathless king | caster, drains Willpower 2, magic regalia |
| 67 | charnel titan | bone with protection 10, grave rot |
| 68 | ghost dragon | incorporeal, drains Toughness 2, frightens |
| 69 | eldest vampire | drains Toughness 3, transfixes |
| 70 | demilich | the roster's only `unique`, caster (range 12), drains Intelligence 3, physical 50 |

Drains are 2 points (3 for the two apexes) at difficulty 12 + level ÷ 5; wall Defence is 8 for the
three spirits. The nightworm swallows (`swallowed`, docs/dev/status-effects.md) at 15 % and 24. Plurals: `huntsman`, `demilich`.
`ROSTER_TOP` in `undead_roster_tests.rs` is 70, the top of both batches.

Spells (#1102, docs/dev/creatures.md "Casters"; in the order each prefers them): the barrow wight `hex_of_weakness`, `grave_breath` (mana 6); the ash wraith `scorching_ray`, `grave_breath` (mana 24); the mummy priest `fear`, `cause_wounds`, `blight` (mana 60); the lich `haste`, `chain_lightning`, `fireball`, `grave_spear` (mana 120); the mummy lord `fear`, `blight`, `grave_spear` (mana 80); the barrow king `fear`, `grave_spear`, `rime` (mana 110); the wormlord `fear`, `leech`, `blight` (mana 90); the ghost lord `fear`, `slumber`, `grave_spear` (mana 120); the archlich `haste`, `silence`, `sunfire`, `chain_lightning`, `grave_spear` (mana 300); the mummy king `fear`, `flame`, `grave_spear` (mana 130); the deathless king `haste`, `fear`, `sunfire`, `grave_spear`, `chain_lightning` (mana 300); the demilich `silence`, `sunfire`, `chain_lightning`, `leech`, `grave_spear` (mana 350).

Gazes (#1125, docs/dev/creatures.md "Gazes"): the vampires' transfixing stare left the bite for
a `[creature.gaze]` (the spawn range 4 at 25 %, the vampire 5 at 30, the nosferatu 5 at 35, the
lord 5 at 40, the elder and the eldest 6 at 45 and 50; the bite keeps its Toughness drain), as
did the spectre's Willpower drain (range 4, 40 %), the white lady's, the night mare's, and the
ghost lord's transfixion (range 5, 35 %; their touch keeps its drain), and the hand of glory's
candle (range 3, 30 %). The washer at the ford keeps her fear on her touch, beside #1128's fear aura.
`world/bestiary_tests.rs` reads the spectre's drain on its gaze.
The burning skull bursts when it dies (the dread design §5.3, #1127: `[creature.burst]`
`when = "dies"`, `2d6` fire, radius 1; `world/creature_burst.rs`).
Presence (#1128, `world/aura.rs`; design `docs/design/monster-dread.md` §5.4, §9): cold auras (`sluggish`, radius 2, Toughness, `damage_type = "cold"`) on the wraith, dread wraith, and wraith lord (1 to 3 degrees, difficulty 13, 16, 18); fear auras (`frightened` 2, radius 3, Willpower) on the washer at the ford (19), the deathshade (21), the archlich (20), and the demilich (22); the gibbering shade's silence (radius 3, no check, every 6 s); heard fear auras on the banshee (radius 6, loudness 100, 15, every 24 s, with a wail cry) and the screaming skull (radius 4, loudness 120, 17, with a scream cry). Cries: howls on the barghest, bodach, black shuck, church grim, and hound of the hunt (150, every 120 s); banging on the knocker and the poltergeist; a rattle on the charnel heap and the bone dragon. `content::aura_roster_tests` lists every row.
## Dread gimmicks (docs/design/monster-dread.md §6)

D5 (#1129): the **charnel heap** (22) splits under slashing and piercing (`splits = { on =
["slashing", "piercing"], least = 8 }`) and engulfs (`engulfed` at 15 %, grip 18; filth fever
kept; slow); the **wormhost** (26) dies into grave worms (`dies_into`, `2d4` within 1; its
burrowing worms kept); the **wormlord** (47) dies into `3d6` within 2 and breeds them as it
fights (`[creature.summon]` of grave worms, `1d2` every 30 s, `most = 6`, no mana, verb "sheds";
tagged `summoner`); the **nightworm** (63) swallows (`swallowed`) in place of its hold. All four
are dread at their tiers (D0). Tests: `world/split_tests.rs`.

The shadowy (#1132, docs/dev/creatures.md "The unseen"): the shadow, the greater shadow,
the deathshade, and the sluagh have `shadowy = true`: on a cell dim to the player's eyes they are
unseen unless beside them, and a cold draught tells of one within 2. Their drains are kept.

## Variance

The variance pass (#1134, `docs/dev/creatures.md` "Variance by role") sets the roles' to-hit (striker `L/4 + L/8`, brute and caster `L/4 − L/8`, the armed `L/4`), threat and crit (striker 19, brute 24 at ×3), and hit-point factors (quick and caster 0.5, brute 2.0). **Speed**, by temperament: the dead are slow and patient. The shamblers walk at 800 (the zombies, the drowned, the bog bodies), the standards at 1000, the ghouls and hounds lope up to 1250; the dread are the quick spirits and vampires at 1400 and up (the shadows, wraiths, banshee, white lady, washer, night mare, the vampires) and the slow and awful at 700 (the charnel heap, the wormhost and wormlord, the nightworm, the lich and archlich).
