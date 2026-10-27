# Troll roster

General conventions: `docs/dev/creatures.md`.

The troll tribe (#456, levels 13 to 42; not every tribe spans 1 to 70): shared types, one file
each in `assets/data/creatures/`, on the conventions in that directory's `README.toml`: faction
`troll` (hostile to the player, the vermin, the kobolds, the undead, and the duergar, the stances
the troll had as one of the goblins, so moving it there changed no fight; neutral to the goblins
and the ogre), tag `troll` on every troll (the marsh troll stays `marsh`, the rotting troll
`undead`; both carry the tag, so a `tag:troll` kill quest counts them). Every troll that was one
of "the goblins' ogres and trolls" is now of the `troll` faction, so the spec's duergar stance
(hostile to the goblins) reads as hostile to the trolls too.

A troll's regeneration is its Toughness: there is no per-creature healing rate, so the tribe's
formulas are the undead roster's (#383) with Toughness 4 higher, and fire hurts all but the fire
troll (`resist fire` negative). `content/troll_roster_tests.rs` checks every living troll
(`ROLES`: brute, standard, quick, caster) against the formulas (XP guideline, ±35 % of max HP and
dice average) and the span's ends.

## Roster

The troll 13 (above), the hill troll 15 (slow, claws that stagger, a maul half the time;
Fyrdelf's), and in no population yet, waiting for deeper places: the forest troll
18 (quick, opens doors, claws that leave bleeding wounds, a spear half the time), the troll
shaman 21 (a caster with a staff half the time, sometimes something worth taking), the ice troll
24 (resists cold, claws that leave one Sluggish), the stone troll 27 (slow, a stony hide of
protection 5 that resists slashing and piercing but cracks under blunt blows, fists that
stagger), the two-headed troll 30 (sees and hears more, and its bites, which bleed, come quicker:
an attack of its own cost), the fire troll 33 (lit by its own embers, a searing touch, resists
fire, hurt more by cold), the troll chieftain 37 (a great axe, scale mail, and a helm, often
magic; claws that frighten), and the elder troll 42 (very rare, huge, claws that bleed and
stagger). The marsh troll stays a marsh thing and the rotting troll undead; both carry the
`troll` tag.

The two-headed troll is the first type with an `attack_cost` (700; `world/cost_tests.rs` names
it). Only the hill troll joined a population (Fyrdelf, beside the troll). Follow-ups: fire or
acid stopping a troll's healing, the stone troll and sunlight, a `large` size.
