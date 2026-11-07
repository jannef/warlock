# Duergar roster

General conventions: `docs/dev/creatures.md`.

The duergar (#518), the grey dwarves of the deep: twelve shared types from level 8 to 40, a notch
below the drow (about 14 to 50). Three came with the delf wave (#335, `campaign-waves.md`: the
miner 9, the warrior 11, the smith 13) and keep their numbers; nine are the roster's. One file
each in `assets/data/creatures/` (`duergar_*`), faction `duergar` (unchanged: hostile to the
player, the goblins and their kin, the undead, and the giants; the orcs, demons, and trolls list
it back; neutral to the kobolds, the cult, the fire things, the merchants, and peaceful folk). The
drow roster makes `drow` hostile to `duergar`: stances are symmetric, so the duergar's faction
does not name the drow. The `duergar_trader` stays a merchant, and the `duergar_skeleton` undead.

Every duergar: tag `duergar` (plus `archer` on the slinger and the crossbowman, `caster` on the
priest and the mindbreaker, and `priest` on the priest), `alignment = "evil"`, `race =
"duergar"`, `vision = "darkvision"`, opens doors, medium, `resist = { poison = 25, cold = 10 }`
(the smith, thane, and king fire 25 too). Glyph `h` in colours apart from every other `h` (the
hags, the trader; at least 45 apart in RGB from all of them).

## Formulas

#383's scaling on living stats, as the orcs' (#455): XP `5 × L + 10`; St and To `12 + 0.4L`, Dx
`10 + 0.25L`, by role (archer Dx +3; standard and officer ±0; quick Dx +5 To −1; brute St +5 Dx
−2 To +5; caster St −3); In `9 + 0.2L`, Wp `12 + 0.3L`, Pr `6 + 0.25L` (dour and stubborn, joyless;
casters In and Wp +4); `base_hp = 2.5 × L × role` (archer and standard 1.0, officer 1.1, quick
0.5, brute 2.0, caster 0.5; #1134). The armed ones hold their racial weapon at rank `1 + L/7` at least (raised to aim at `L/4`, #1169, `docs/dev/creatures.md`); the two
who fight bare (the spider rider, whose spider bites, and the mindbreaker's mind-lash) have a
natural attack on #383's dice ladder (caster one step down), to-hit by role (#1134: the spider rider a striker's, the
mindbreaker a caster's), and a rider.
`content/duergar_roster_tests.rs` checks all of it (the three older types only for XP and hit
points), the stances, and the gear rolls (armed nine times in ten, the shooters with ammunition).

## Roster

The slinger 8 (a sling and stones, a hand axe half the time), the crossbowman 15 (a crossbow and
bolts, studded leather), the veteran 17 (battle axe, scale mail, helm, shield), the priest 19 (a
caster, mace and mail, an amulet sometimes magic), the hulk 22 (a duergar swollen to twice its
height: a maul, a grey hide of Defence 1 and Protection 2, slow), the spider rider 25 (quick,
speed 1250; its spider's bite poisons), the mindbreaker 29 (a caster; its mind-lash, force damage, and
a stare that frightens and transfixes: its gaze, #1125, range 4, 30 %), the thane 34 (Old English *þegn*: great axe, banded mail, great helm,
gold), and the duergar king 40 (very rare: war hammer, plate, great helm, large shield, gold).

## Populations

Fyrdelf, the duergar's own mine (danger 5 to 16): the slinger, crossbowman, and veteran (17: only
out of depth, at the bottom) join the miner, warrior, and smith. The priest (19) left it when its
danger was squished (#534), so no bounty names what next to never comes there; it, the hulk, and
everything above are in no population yet: met only as strays
(docs/dev/creatures.md).

Follow-ups: enlarging as an ability used in a fight, a mount as a creature of its own.

Invisibility (#1132, docs/dev/creatures.md "The unseen"): every duergar but the skeleton has
`[creature.invisible] cooldown = 300`: it fades from sight (two minutes of Invisible) at the start
of its turn whenever the five minutes are up, and goes on with its turn; the priest and the
mindbreaker as well as their spells. Their hobnails are heard as ever. `world/unseen_tests.rs`
checks the rows.

Spells (#1102, docs/dev/creatures.md "Casters"; in the order each prefers them): the duergar priest `cure_wounds`, `blessing`, `flay`, `cause_wounds` (mana 40); the duergar mindbreaker `fear`, `bind`, `flay` (mana 60).

## Variance

The variance pass (#1134, `docs/dev/creatures.md` "Variance by role") sets the roles' to-hit (striker `L/4 + L/8`, brute and caster `L/4 − L/8`, the armed `L/4`), threat and crit (striker 19, brute 24 at ×3), and hit-point factors (quick and caster 0.5, brute 2.0). **Speed**: grey dwarves are steady, 950 to 1000 (the rank-and-file at 1000), the spider rider's mount 1250.
