# Fiend roster (the fiends of no family)

General conventions: `docs/dev/creatures.md`.

The fiends of no family (#450): the hell hound 13, the nightmare 21, the succubus and the incubus
27, the night hag 33, the cambion 39, and the rakshasa 52, shared types on the undead roster's
formulas (#383) for their level and role (quick, standard, caster). Their faction is `fiends`
(hostile to the player only), deliberately **not** `animal`: the hell hound and the nightmare are
fiends in a beast's shape, so a Beast-friend calms neither. All carry the `fiend` tag (the demon,
devil, and yugoloth rosters share it; each family has its own faction and tag besides), the
casters `caster`, and the night hag `hag` (the hags' tag). The succubus and the incubus are one
fiend under two names: two types with the same numbers, told apart by name, colour, and weight
(`the_succubus_and_the_incubus_share_their_numbers`). `content/fiend_roster_tests.rs` checks each
type's level, faction, tags, alignment, XP, and the HP and dice formulas (within ±35 %, as the
undead's), and that the faction holds no type it does not list. Riders map onto what exists: fire
damage for the hound's bite and the steed's hooves, a Toughness drain (difficulty 17, checked by
Toughness) for the kiss, `held` (15 % at 18: the hag on the sleeper's chest) and `frightened` for
the night hag, `transfixed` and `frightened` for the rakshasa. The hell hound is in Fyrdelf's
population (its 13 inside danger 5–16, at home in the fires); the rest ship in none. Breath,
shapes, flight, charm, summoning, and dream-walking are #450's follow-ups.

Fixtures that write their own factions file list `fiends` (`equip_tests`, `magic_tests`,
`object_tests`, `read_tests`).
