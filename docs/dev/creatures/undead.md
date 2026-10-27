# Undead roster

General conventions: `docs/dev/creatures.md`.

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
rest up to level 20: Fennmynster the stone haunt (in its stones), the pale drinker, and the
mummy (its crypts); Fyrdelf the draugr, barghest, hand of glory, vampire spawn, gallows dead,
bone wyvern, gibbering shade, fetch, revenant, and mummy priest (level 20, only out of depth on
its twelfth floor), the dead of its deep workings. All of batches 3 to 6 (levels 21 and up) ship
in no population (deep undead wait for deep places, #383), and the ghost is
incorporeal (wall Defence 2). The roster's holds (`held`, #431) are `{ status = "held", chance,
difficulty }`: the ghast's and fen ghoul's claws 10 % at 15, the gallows dead's tongue 20 % at 17, the lich's touch and the ghoul lord's claws 15 % at 18 (12 + level ÷ 5, as a drain's).

## Batch 4 (#387, levels 31–40)

All ten ship in no population. Each is one file in `assets/data/creatures/`:

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

All sixteen ship in no population. Each is one file in `assets/data/creatures/`:

| Level | Type | Notes |
|---|---|---|
| 41 | wraith lord | incorporeal, a touch that drains 2 Toughness |
| 42 | barrow king | a caster, drains 2 Toughness, an enchanted long sword, loot |
| 43 | sluagh | incorporeal, the fastest spirit (speed 1600), claws that frighten |
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
| 55 | hound of the hunt | speed 1700, frightens |

Drains are two points from level 40 (difficulty 12 + level ÷ 5, Toughness's checked by
toughness); the spirits' wall Defence is 2 + level ÷ 10 (6 to 7). The ghoul king's hold is 20 % at
20, the archlich's 20 % at 22. Plurals: `archlich` and `sluagh` in `plurals.toml`; the washer's
and the hound's are written in their files.

## Batch 6 (#389, levels 56–70)

All fifteen ship in no population. Each is one file in `assets/data/creatures/`; the table is
#389's, with its numbers unmoved:

| Level | Type | Notes |
|---|---|---|
| 56 | mummy king | caster, fist with mummy rot (35 %), wrappings with protection 5, fire hurts it more |
| 57 | wild huntsman | quick on foot (speed 1500), spear or glaive, sometimes magic |
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
three spirits. The nightworm's hold is 15 % at 24. Plurals: `huntsman`, `demilich`.
`ROSTER_TOP` in `undead_roster_tests.rs` is 70, the top of both batches.
