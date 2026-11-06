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
| Water | standard | water mote 4, lesser water elemental 7, water elemental 14, greater 26, elder 42, brimwolf 55 | walks on land; resists fire and piercing, electricity hurts it more; `sluggish` blows, `engulfed` (drowned, docs/dev/status-effects.md) from the greater up |
| Ice | brute | frost mote 5, lesser ice elemental 11, ice elemental 21, greater 33, elder 48, rimethurse 66 | slow, a cold blow that leaves one `sluggish`, immune to cold, fire hurts it more |
| Lightning | quick | spark mote 8, lesser lightning elemental 15, lightning elemental 23, greater 36, elder 52, thunderhead 70 | fast, a shock that `staggered`, immune to electricity; the `storm` light (`data/lights.toml`, added for it; the spark mote only a glow) |

**Bursts** (the dread design §5.3, #1127; `world/creature_burst.rs`): the six motes are
**vortices** (`[creature.burst]` with `when = "touches"`, radius 1, no `attack`, speed +200):
stone mote `2d4` bludgeoning, ember mote `2d4` fire, dust devil `2d4` slashing, water mote `2d4`
bludgeoning, frost mote `2d6` cold, spark mote `2d6` electricity. Every other rung but the earth
line's has **death throes** (`when = "dies"`), `(2 + L/8)d6` (half the doubled damage spells'
`(4 + L/4)d6` of its level), by line and rung (lesser, with the cinder elemental on the fire
line's; plain; greater; elder; eldest):

| Line | Type | Radius by rung | Rider | Also |
|---|---|---|---|---|
| fire | fire, effect `fireball` | 1 / 2 / 2 / 3 / 3 | 50 % `burning` from the plain rung | — |
| lightning | electricity, `shatter` | 1 / 2 / 2 / 3 / 3 | 50 % `staggered` | — |
| ice | cold, `frost_burst` | 1 / 2 / 2 / 3 / 3 | `sluggish` | — |
| air | bludgeoning, `shatter` | 1 / 1 / 2 / 2 / 3 | 50 % `staggered` | `gust`: piles blown a cell outward |
| water | bludgeoning, `shatter` | 1 / 1 / 2 / 2 / 3 | — | `douses`: Burning and fires put out |

The fire line's blows also set alight (`on_hit` 25 % `burning`). The **earth line** does not
burst: it throws stones (`[creature.throw]`, the `boulder` object named "stone", range
5 + L/10, the fist's dice a size smaller, `cost = 1500`, bludgeoning, 15 % `staggered`), and one
inside a wall comes out before it throws (`hurl.rs`). `content/elemental_roster_tests.rs`'s
`RUNGS` table pins each line's burst and the earth line's throw, and every elemental is on a
rung or a vortex.

The greater, elder, and eldest rungs are size medium with great weights (no larger size yet) and
drop loot now and then. Riders are existing statuses only.

`content/elemental_roster_tests.rs` holds every `elemental`-tagged type to the XP guideline and to
±35 % of the max HP and dice formulas for its role (`ELEMENTS`: the element's role, motes `Small`),
checks one element tag, poison and the element's own immunity, no corpse, and that each element has
at least six rungs from level 10 or lower to 50 or higher. Test content that lists its own factions
(`world/{equip,magic,object,read}_tests.rs`) adds `elemental`, and `world/mana_tests.rs`'s own
STATUS keeps `staggered` and `sluggish` for the elementals' riders.

Population lines: Fyrdelf the lesser fire elemental and the earth elemental, Fennmynster the
lesser water elemental. #531 placed the rest up to level 17: Fennmynster the water mote, frost
mote, and lesser ice elemental (its cold water); Fyrdelf the stone mote, ember mote, and dust
devil (the shallow galleries' filler, as the bats are), the spark mote, the lesser earth, air,
and lightning elementals, and the water elemental. The air elemental (19) and those of level 21
and up (the ice elemental and every greater and elder one) ship in none: no danger window reaches
them (Fyrdelf's runs 5 to 16, #534). Deferred (#453 "Follow-ups"): large size,
water-bound behaviour, knockback, whirlwind, engulf, breath, elements at war, gem drops.

Presence (#1128, `world/aura.rs`): the fire salamander is made with `fire_cloaked` and keeps it (innate: whoever strikes it burns; never shown in the look line).

The invisible stalker (#1132, docs/design/monster-dread.md §5.11): the air line's kin of level 22
(uncommon, dread), `invisible = true` and `hunts = true` (docs/dev/creatures.md "The unseen"):
never seen, it comes to the player wherever they are on its level; a quick role (1500, `3d8`
slashing), no corpse, no blood, and, as every elemental now, no heartbeat for Sense the quick. No
population names the air elementals of its depth yet, so it comes as a stray. It keeps the roster's
formulas (`elemental_roster_tests`). It bursts where it dies as the air elemental does (`4d6`, radius 1, a
gust; #1127's death throes, `elemental_roster_tests::KIN`).

## Variance

The variance pass (#1134, `docs/dev/creatures.md` "Variance by role") sets the roles' to-hit (striker `L/4 + L/8`, brute and caster `L/4 − L/8`, the armed `L/4`), threat and crit (striker 19, brute 24 at ×3), and hit-point factors (quick and caster 0.5, brute 2.0). **Speed**, by element: earth grinds at 800, ice at 850 to 950, fire and water at 1000 to 1100, air and lightning at 1250, the top of the bulk; the vortices kept D3's quicker speeds (dust devil 1650, spark mote 1550, ember and water motes 1300; design §5.3); the invisible stalker at 1500.
