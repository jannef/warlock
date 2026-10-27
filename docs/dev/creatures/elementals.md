# Elemental roster

General conventions: `docs/dev/creatures.md`.

The elemental roster (#453): six elements (fire, earth, air, water, ice, lightning), each a ladder
of six rungs (a mote, lesser, the plain name, greater, elder, and an eldest with an Old English
name: balefire, hoarstone, welkin wind, brimwolf, rimethurse, thunderhead) from level 1 to 70,
one file each in `assets/data/creatures/`, on the README's conventions (the undead roster's
formulas, a role by element). Faction `elemental` (hostile to the player only); tags `elemental`
and the element's word; no corpse, no blood, immune to poison; motes `e` and small, the rest `E`.
The existing cinder elemental (L17) is the fire line's middle-low rung, unchanged but for its
tags, in the delf's `fire` faction.

| Element | Role | Rungs (level) | Notes |
|---|---|---|---|
| Fire | standard | ember mote 2, lesser fire elemental 9, cinder elemental 17, fire elemental 27, greater 38, elder 50, balefire 64 | immune to fire, cold hurts it more; carries `ember_full`, `lava`, or `brazier` light |
| Earth | brute | stone mote 1, lesser earth elemental 6, earth elemental 13, greater 24, elder 40, hoarstone 60 | slow, protection, resists piercing and slashing, immune to electricity; `staggered` blows, `held` from the elder; incorporeal from the earth elemental up (`wall_defence = 1 + level / 20`) |
| Air | quick | dust devil 3, lesser air elemental 10, air elemental 19, greater 31, elder 45, welkin wind 58 | very fast, hard to hit, resists piercing and electricity; `staggered` blows |
| Water | standard | water mote 4, lesser water elemental 7, water elemental 14, greater 26, elder 42, brimwolf 55 | walks on land; resists fire and piercing, electricity hurts it more; `sluggish` blows, `held` from the greater up |
| Ice | brute | frost mote 5, lesser ice elemental 11, ice elemental 21, greater 33, elder 48, rimethurse 66 | slow, a cold blow that leaves one `sluggish`, immune to cold, fire hurts it more |
| Lightning | quick | spark mote 8, lesser lightning elemental 15, lightning elemental 23, greater 36, elder 52, thunderhead 70 | fast, a shock that `staggered`, immune to electricity; the `storm` light (`data/lights.toml`, added for it; the spark mote only a glow) |

The greater, elder, and eldest rungs are size medium with great weights (no larger size yet) and
drop loot now and then. Riders are existing statuses only.

`content/elemental_roster_tests.rs` holds every `elemental`-tagged type to the XP guideline and to
±35 % of the max HP and dice formulas for its role (`ELEMENTS`: the element's role, motes `Small`),
checks one element tag, poison and the element's own immunity, no corpse, and that each element has
at least six rungs from level 10 or lower to 50 or higher. Test content that lists its own factions
(`world/{equip,magic,object,read}_tests.rs`) adds `elemental`, and `world/mana_tests.rs`'s own
STATUS keeps `staggered` and `sluggish` for the elementals' riders.

Population lines: Fyrdelf the lesser fire elemental and the earth elemental, Fennmynster the
lesser water elemental. #531 placed the rest up to level 19: Fennmynster the water mote, frost
mote, and lesser ice elemental (its cold water); Fyrdelf the stone mote, ember mote, and dust
devil (the shallow galleries' filler, as the bats are), the spark mote, the lesser earth, air,
and lightning elementals, the water elemental, and the air elemental. Those of level 21 and up
(the ice elemental and every greater and elder one) ship in none: no danger window reaches them. Deferred (#453 "Follow-ups"): large size,
water-bound behaviour, knockback, whirlwind, engulf, burning, breath, elements at war, gem drops.
