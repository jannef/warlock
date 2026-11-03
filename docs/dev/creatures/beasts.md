# Beast roster

General conventions: `docs/dev/creatures.md`.

The beasts (#839; the druid design `docs/design/druid-and-summons.md` §7.3): the wild animals the
druid's calls bring to the player's side (`docs/dev/allies.md`), shared types, one file each in
`assets/data/creatures/`, on the conventions of that directory's `README.toml`: faction
`wildlife` (neutral to everyone; an animal faction, so Beast-friend reads it), tag `beast` on
every one. Being neutral to the player, they never come as strays (`Content::may_stray` reads
the stance) and no population names them: today they are met only as allies. Each has a
`depart` line, the words of its going when its call runs out ("The wolf slips back into the
dark."; `World::ally_departs`, default "fades away").

`content/beast_roster_tests.rs` holds every `beast` to its row (`BEASTS`: level, max hit points
as Toughness + `base_hp`, attack dice), the XP guideline, wildlife, no stray, no unique, and a
`depart` line.

## Roster

| Type | Level | XP | Speed | Max HP | Attack | Notes |
|---|---:|---:|---:|---:|---|---|
| `wolf` | 4 | 30 | 1250 | 16 | bite 1d6 piercing | darkvision, hearing 20, footsteps 60 (a pack is heard); Call of the wild's |
| `bear` | 12 | 70 | 1000 | 60 | claws 2d6 slashing | low-light eyes, a thick hide (Defence 1; the body rule's Protection 0 at 12), footsteps 70; Call of the forest's |

Call the elements draws from the elemental roster (`elementals.md`): the greater earth (24),
water (26), air (31), and fire (38) elementals. Follow-ups: a wolf-pack row in the Ascmearc's
woods encounters as wildlife ("Hunt them" / "Let them be"), the ranger's companion (#825).
