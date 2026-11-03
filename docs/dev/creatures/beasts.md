# Beast roster

General conventions: `docs/dev/creatures.md`.

The beasts (#839, the druid design `docs/design/druid-and-summons.md` §7.3; #916, the ranger's
`docs/design/ranger-companion.md` §12.2): the wild animals the druid's calls bring to the
player's side and the ranger's companions (`docs/dev/allies.md`, `docs/dev/companion.md`),
shared types, one file each in `assets/data/creatures/`, on the conventions of that directory's
`README.toml`: faction `wildlife` (neutral to everyone; an animal faction, so Beast-friend reads
it), tag `beast` on every one, `tameable = true`, a `nose` where the beast hunts by scent (the
cats, the spider, the lizard, the bat, the beetle, and the deer have none). Being neutral to the
player, they never come as strays (`Content::may_stray` reads the stance) and no population
names them: today they are met only as allies (a ranger's from the first turn; the wild ones in
the woods are C7's). Each has a `depart` line, the words of its going when its call runs out
("The wolf slips back into the dark."; `World::ally_departs`, default "fades away").

`content/beast_roster_tests.rs` holds every `beast` to its row (`BEASTS`: level, max hit points
as Toughness + `base_hp`, attack dice), the XP guideline, wildlife, no stray, no unique,
tameable, and a `depart` line. The vermin and the orcs' wolves that are tameable too (`rat`,
`bat`, `beetle`, `deer`, `warg`, `dire_warg`) keep their own rosters.

## Roster

| Type | Level | XP | Speed | Max HP | Attack | Nose | Notes |
|---|---:|---:|---:|---:|---|---:|---|
| `hound` | 1 | 15 | 1200 | 12 | bite 1d4 piercing | 6 | low-light, hearing 20, footsteps 40; the commonest start (human, elf, half-elf, dwarf, halfling, half-orc, ratfolk) |
| `polecat` | 1 | 15 | 1300 | 10 | bite 1d4 piercing | 6 | low-light, hearing 15, footsteps 20, small; halfling, gnome, half-elf, ratfolk |
| `lynx` | 2 | 20 | 1300 | 13 | claws 1d6 slashing | — | darkvision, hearing 25, footsteps 20; elf, half-elf |
| `badger` | 2 | 20 | 900 | 19 | bite 1d6 piercing | 6 | low-light, a thick hide (Defence 1), footsteps 40, small; dwarf, gnome, duergar |
| `boar` | 2 | 20 | 1100 | 20 | tusks 1d8 piercing | 6 | low-light, footsteps 60 (loud); human, half-orc, lizardfolk |
| `large_spider` | 2 | 20 | 1100 | 13 | bite 1d4 piercing, poison 25 % as the barrow spider's | — | darkvision, footsteps 20, small, green ichor; goblin, drow, kobold, duergar |
| `cave_lizard` | 2 | 20 | 1000 | 16 | bite 1d6 piercing | — | darkvision, scales (Defence 1), footsteps 30, small; kobold, lizardfolk |
| `wolf` | 4 | 30 | 1250 | 16 | bite 1d6 piercing | 8 | darkvision, hearing 20, footsteps 60 (a pack is heard); Call of the wild's; a rare start |
| `black_hound` | 5 | 35 | 1250 | 24 | bite 1d8 piercing | 10 | darkvision, hearing 22, footsteps 40; the best nose; a rare start |
| `great_boar` | 6 | 40 | 1100 | 30 | tusks 2d6 piercing | 6 | low-light, a thick hide (Defence 2), footsteps 70; a rare start |
| `panther` | 6 | 40 | 1400 | 24 | claws 1d8 slashing | — | darkvision, hearing 20, footsteps 10 (the quietest feet); a rare start |
| `bear` | 12 | 70 | 1000 | 60 | claws 2d6 slashing | 6 | low-light eyes, a thick hide (Defence 1; the body rule's Protection 0 at 12), footsteps 70; Call of the forest's |

Call the elements draws from the elemental roster (`elementals.md`): the greater earth (24),
water (26), air (31), and fire (38) elementals. Follow-ups: a wolf-pack row in the Ascmearc's
woods encounters as wildlife ("Hunt them" / "Let them be"); the boar, the lynx, and the badger in
the forest and hill local areas and the Ascmearc's wildlife encounters (the companion design's
C7).
