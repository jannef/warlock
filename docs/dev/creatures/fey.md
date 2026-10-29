# Fey roster

General conventions: `docs/dev/creatures.md`.

**Courts (#452).** A type may give `factions = { seelie = 50, unseelie = 50 }` (weights, 1 to 1000;
never with `faction`) instead of `faction` (`CreatureDef::factions`, checked in
`Content::creature_factions`; `CreatureDef::faction` is then the heaviest, ties the first by id,
for type-level answers: the wiki, `Content::stance`). Each creature of it joins one when it is
made: `world/court.rs::join`, one call in `travel::make_creature` (so spawns, populations, level
files, and vaults alike), unless the spawn names a `faction` of its own (a level legend's wins).
Stream `court/<actor id>` (`Rng::court`), drawn only for such a type, so no other stream moves.
The pick is `Actor::faction_override` (saved and hashed when set; no new save field).

**The court rule.** `seelie` ("Seelie court") is hostile to the Unseelie only, so neutral to the
player; `unseelie` is hostile to the player and the Seelie; `wrathful_fey` is hostile to the
player and the Unseelie. Every other faction is neutral to all three (none is `animal`). A
faction's `provoked` (`FactionDef::provoked`) is the faction one of its creatures joins when the
player attacks it in melee, with a missile, or with a spell: `World::provoke_court`, one line
beside the shopkeeper's `provoke` in `fight.rs` and `spell.rs`. The Seelie court's is
`wrathful_fey`: a Seelie fey the player strikes turns on them for good; only the one struck.
Fixtures that write their own factions file list `seelie` and `unseelie`.

**The roster.** Thirteen shared types, one file each in `assets/data/creatures/`, tagged `fey`,
each of the Seelie or the Unseelie court by its weights (the creatures README's notes): sprite 2
(small, a short bow, a poisoned stab), pixie 4 (small caster, a touch that transfixes), quickling
6 (speed 2000), boggle 7 (small, hard to hit, opens doors), darkling 9 (small, darkvision), satyr
10, redcap 12 (small, quick, a heavy stamp), blink dog 14, meenlock 16 (claws that frighten and
may hold), dryad 19 and nymph 22 (casters whose touch transfixes), korred 25 (small, very tough,
fists that may hold), yeth hound 31 (quick, a bite that frightens). Lean Unseelie: redcap and
meenlock 10/90, quickling and yeth hound 25/75; lean Seelie: dryad and nymph 90/10, blink dog
80/20; the rest even. `content/fey_roster_tests.rs` holds levels, roles, and court weights against
#383's formulas (XP `5L + 10`; max HP and dice within ±35 %). Riders are existing statuses only
(poisoned, transfixed, frightened, held); the redcap has no bleeding rider (`mana_tests`' status
fixture lacks Bleeding). Tests of the mechanism: `world/court_tests.rs` (weights over many draws,
per-seed determinism, save and load, a level's faction winning, look mode's stance, a struck
Seelie fey).

Fennmynster's population takes the pixie, the boggle, and the darkling, and since #531 the
sprite, the quickling, and the satyr (the redcap, level 12, waits for deeper places); Fyrdelf's the meenlock (the unseelie dread of its dark galleries).
The blink dog and the dryad, woodland fey, are in no population (the Ascmearc has no woodland
branch), nor are the nymph, the korred, and the yeth hound (levels 22 to 31: no window reaches
them). Teleport, illusions, flight, charm, tree-binding, a death burst, poisoned
missiles, oil, stone shaping, captive transformation, and a woodland place for the rest are
follow-ups.

Surprise (#686, spec §8 "Surprise and sneak attacks"): four of the Unseelie's kin sneak-attack,
by their `skills`: the quickling `sneak_attack = 40` (2d6) with `initiative = 150` (it cuts
before it is seen, and is nearly never caught off guard), the darkling 30 (2d6) and 50, the
redcap 40 (2d6) and 40, the meenlock 30 (2d6) and 30. The meenlock **lurks** (`lurks = true`,
`world/lurk.rs`): a hostile it sees, it waits for where it stands until it comes beside it or
the meenlock is hurt. `bestiary_tests::the_sneak_attackers_carry_their_craft_and_two_lurk` checks the numbers.
