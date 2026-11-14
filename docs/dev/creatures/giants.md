# Giant roster

General conventions: `docs/dev/creatures.md`.

The giant tribe (#457, levels 12 to 50): faction `giant` (hostile to the player, the vermin, and
the kobolds, as the ogre was among the goblins, so Fyrdelf plays as before; the duergar and the
undead list it; the goblins and their kin stay neutral), tags `giant` on every type and `ogre` on
the ogre, ogre mauler, and ogre mage, `caster` on the ogre mage and giant shaman. The ogre moved
there from `goblins`, and its fist went from 1d8 to 2d8 to sit on the formula. Shared types, one
file each; they are written to #457's formulas (#383's for a living
brute: each has a role, giant, ogre, or caster; max HP and dice within ±35 %, the XP
guideline, the tags, and every giant but the shaman throws a boulder at the level's dice from
5 + L/10 cells, the stone giant two farther).

The roster: the ogre 12 (slow, racial maul), ogre mauler 15 (slow, racial maul, hide), ogre mage
18 (a caster, a bone charm sometimes magic); the `H` giants: hill giant 20, stone giant 24 (the
surest, farthest, quickest thrower), frost giant 28 (blocks of ice, crushing or freezing; resists
cold, fire hurts it more), fire giant 32 (burning boulders; immune to fire, cold hurts it more),
giant shaman 36 (a caster, throws nothing), cloud giant 40 (sees far), storm giant 45 (lightning
in its sword and its boulders; immune to electricity), giant jarl 50 (very rare, a hall's gold on
its belt).

The giants fight with a natural attack named for their giant-sized weapon (no loot to carry off)
and throw (`[creature.throw]`, `world/combat/hurl.rs`, `docs/dev/ranged-combat.md`); the ogres carry
mauls and throw nothing. A creature's throw is `World::hurls_at` in the AI's fight step, before
the melee/ranged/caster choice, `World::creature_hurls`, said by `hurl::throw_messages`.
Fyrdelf's population adds the ogre mauler (the ogre mage, 18, left it when its danger was
squished to 5–16, #534); the giants ship in no population until a
deep place takes them (until then they come only as strays, docs/dev/creatures.md). Only small and medium sizes exist, so they are medium with huge weights (a
large size is a follow-up). The `boulder` object (miscellaneous, no kind) is what flies; it is
never loot, never in a shop, never placed.

Spells (#1102, docs/dev/creatures.md "Casters"; in the order each prefers them): the ogre mage `rime`, `frost`, `magic_missile` (mana 45); the giant shaman `cure_wounds`, `rime`, `lightning_bolt` (mana 80).

## Variance

The variance pass (#1134, `docs/dev/creatures.md` "Variance by role") sets the roles' to-hit (striker `L/4 + L/8`, brute and caster `L/4 − L/8`, the armed `L/4`), threat and crit (striker 19, brute 24 at ×3), and hit-point factors (quick and caster 0.5, brute 2.0). **Speed**: the ogres lumber at 800 and 850, the giants stride at 950 to 1050.

Gear materials (#1191): the faction's `gear` in `data/factions/`: giants favour bone (25 %), fur (20 %), and iron (20 %), each its share of the draws for an item whose construction list holds it, the usual draw the rest (`docs/dev/objects.md` "Materials and variants").

**Material drops** (#1341; `data/drops/`, docs/dev/objects.md "Material drops"): `giant.toml`: bone 1d3 (50 %), leather 1d2 (50 %), sinew 1 (40 %); the ogres add `ogre.toml`: bone 1d2 (40 %), leather 1 (40 %).
