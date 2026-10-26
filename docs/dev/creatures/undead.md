# Undead roster

General conventions: `docs/dev/creatures.md`.

The undead roster (#383; batch 1 #384, levels 1–10; batch 2 #385, 11–20; batch 3 #386, 21–30): shared types, one file each in
`assets/data/creatures/`, on the conventions noted in that directory's `README.toml`. Each batch
adds its types to `content/undead_roster_tests.rs` (`ROLES`, the type's role) and raises
`ROSTER_TOP`: every `undead` type must be on the XP guideline (`OFF_GUIDELINE` lists the pale
drinker), lie within ±35 % of #383's max HP and dice formulas for its level and role, and every
level from 1 to `ROSTER_TOP` must have one. Batch 1's population lines: Cealdhlāw the crawling
hand, skeletal rat, kobold zombie, skeleton archer, skeletal wolf, and skeleton warrior;
Fennmynster the corpse candle, poltergeist, ghast, shadow, and bog body; Fyrdelf the ogre zombie
and grave hound. Batch 2's: Fennmynster the wraith and fen ghoul; Fyrdelf the burning skull,
knocker, spectre, charred dead, duergar skeleton, and rotting troll. The mummy, barghest, draugr,
vampire spawn, hand of glory, gallows dead, gibbering shade, bone wyvern, revenant, fetch, and
mummy priest ship in no population, as does all of batch 3 (levels 21 and up) (deep undead wait for deep places, #383), and the ghost is
incorporeal (wall Defence 2). The roster's holds (`held`, #431) are `{ status = "held", chance,
difficulty }`: the ghast's and fen ghoul's claws 10 % at 15, the gallows dead's tongue 20 % at 17, the lich's touch 15 % at 18 (12 + level ÷ 5, as a drain's).
