# Demon roster

General conventions: `docs/dev/creatures.md`.

The demon roster (#449): ten shared types, one file each in `assets/data/creatures/` (the
dretch 8, quasit 12, balgura 24, chasme 28, hezrou 30, vrock 34, marilith 40, glabrezu 44,
nalfeshnee 50, balor 62), faction `demon` (hostile to the player, kobolds, goblins, duergar, and
marsh things, and the devils (hostile both ways, from the demons' side); inserted after `fire`, and in the four test fixtures' factions files), tags
`["demon", "fiend"]`. They use the undead roster's formulas and roles:
Max HP and dice are written to #383's conventions, with both tags and no corpse;
`content/demon_roster_tests.rs` pins the faction's stances. Riders are
existing statuses only: the chasme's drone is `transfixed` (no sleep status exists), the
hezrou's stench `poisoned`, the vrock's screech and the nalfeshnee's horror `frightened`, the
glabrezu's pincers and the balor's whip `held` (difficulty 12 + level ÷ 5), the quasit's venom
`poisoned` and `weakened`. Several attacks a turn do not exist: the marilith's six blades are
`attack_cost = 500` and the balor's sword and whip `attack_cost = 700` (it wields a great sword
from its gear, so its natural whip is a fallback); `world/cost_tests.rs` lists them as the only
types with an attack cost. Only the dretch and the quasit are in a population (Fyrdelf's); the
rest wait for deep places.

Creatures (what each one is): in Fyrdelf's population, the dretch 8 (small, slow, filthy claws
that give filth fever and eat at armour, `corrodes` 10, #1131) and the quasit 12 (small, quick, claws that poison and weaken). In no
population yet, waiting for deep places: the balgura 24 (a savage ape, a brute on quick feet),
the chasme 28 (a giant fly whose drone transfixes), the hezrou 30 (a hulking toad whose stench
sickens: claws that poison, and whose filth eats at armour, `corrodes` 25, #1131), the vrock 34 (a vulture whose screech frightens), the marilith 40
(six blades, striking twice as often), the glabrezu 44 (pincers that hold fast), the nalfeshnee 50
(a winged boar whose bite fills with horror), and the balor 62 (wreathed in fire and lit by it,
immune to fire, a great sword, often magic, struck faster than one blow a turn, and a flaming whip
that holds when it has no sword). Demons are evil, have darkvision, are immune to poison, resist
fire (the balor is immune), and from level 24 resist cold and electricity a little; no corpse; dark
ichor for blood.

The balor bursts when it dies (the dread design §5.3, #1127: `[creature.burst]` `when =
"dies"`, `8d12` fire, radius 2, 50 % `burning`; `world/creature_burst.rs`), and is made with its Fire cloak
(innate, #1128, below).
Presence (#1128, `world/aura.rs`): the hezrou's stench (`poisoned`, radius 2, Toughness 16, every 30 s); the chasme's drone (`drowsy`, heard, radius 3, loudness 50, Willpower 16, every 18 s, a buzzing cry); fear auras on the vrock (radius 2, 17, a scream cry for its screech) and the nalfeshnee (radius 3, 19); the balor innate `fire_cloaked` (its burst is D3's). `content::aura_roster_tests` lists every row.

The quasit (#1132, docs/dev/creatures.md "The unseen"): `[creature.invisible] cooldown =
300`, it stings and is gone from sight, not from the room; its poison kept.

## Variance

The variance pass (#1134, `docs/dev/creatures.md` "Variance by role") sets the roles' to-hit (striker `L/4 + L/8`, brute and caster `L/4 − L/8`, the armed `L/4`), threat and crit (striker 19, brute 24 at ×3), and hit-point factors (quick and caster 0.5, brute 2.0). **Speed**: chaos runs at every pace, 900 (the dretch) to 1200 (the quasit, the balgura); the dread at the edges, the hezrou at 700, the chasme and the balor at 1400.
