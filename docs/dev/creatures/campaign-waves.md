# Campaign creature waves

Notes per wave (barrow #333, fen #334, delf #335); the general conventions are in `docs/dev/creatures.md` "Creature waves". A roster PR edits only its own tribe file in this directory.

The barrow wave (danger 1–6): grave worm, barrow spider, goblin, goblin archer, skeleton, zombie,
giant centipede, barrow ghoul, goblin chief, barrow wight; factions `goblins` and `undead` (the
undead are immune to poison by `resist`, and leave the vermin alone); the barrow wight is the
Ascmearc's. The fen wave (danger 3–10; 4–11 until #534): giant leech, giant frog, drowned, crocodile, cultist,
will-o'-wisp, ghost, mud golem, cult priest, marsh troll, bog hag; factions `marsh` (the fen's
living things, not an animal faction) and `cult` (hostile to the marsh); the drowned and the
ghost are `undead`. The cultist, the cult priest, and the bog hag (`unique`) are the Ascmearc's.
The design's marsh eel (`habitat = "water"`) is left out: placing and keeping a creature in water
is a rule of its own, not content. The wisp carries the `wisp` light (`data/lights.toml`). The wiki's creature tables
(`xtask/src/wiki/creatures.rs`) pick new types up by themselves; the status-effect and disease
tables name the bites that cause them, so `xtask/src/wiki/mod.rs`'s test rows move when a wave's
`on_hit` names a status or disease. A drain in `on_hit` (`docs/dev/stats.md` "Drain", #382) is shown in the creature attack table (`on_hit` text) and in `drain-attacks.md`; the pale drinker, the first drainer, is Fennmynster's since #531.
The delf wave (danger 5–16; 8–19 until #534): ember beetle, duergar miner, duergar warrior, ogre, fire
salamander, troll, duergar smith, magma crawler, ash wraith, basilisk, cinder elemental, and the
fire drake (the Ascmearc's own, `campaigns/ascmearc/data/creatures/`, `unique`); factions
`duergar` (hostile to the goblins, whose kin the ogre and the troll are, and to the undead,
which the ash wraith is) and `fire` (the burning things; hostile to the player only). No
`great_club` object exists, so the ogre's racial weapon is the maul; the pick-axe is a tool
(`racial_weapon` must have damage), so the duergar miner carries it as a Tool-slot gear line and
has the hand axe, the duergar's racial weapon, as a second, rarer line. The basilisk's slowing
bite is the existing `sluggish` status.
Fyrdelf (#339) is where the wave lives: its branch's `population` (with the bats, kobolds, and
giant centipedes the shallow floors draw on), leaving out the drake, which only the drake's
hoard places (a vault its twelfth level names, so a run meets it once, there). Its stores are
kept by the `duergar_trader` (the Ascmearc's own type, `merchants`, the shopkeeper's level and
XP, an outlier of the XP guideline like the shopkeeper).
Fennmynster (#338) is where the fen wave lives: all eleven are its branch's `population`, the bog
hag too (unique, so the population and spawns bring her once a run); the sunken crypt at its
bottom adds two drowned standing guard in its water.

The barrow spider sneak-attacks and lurks (#686, spec §8 "Surprise and sneak attacks"):
`sneak_attack = 10` (1d6), `initiative = 20`, and `lurks = true` (`world/lurk.rs`): one that
sees the player waits in its passage until they come beside it or it is hurt, so Cealdhlāw's
dark holds a thing that waits rather than one that comes.
`bestiary_tests::the_sneak_attackers_carry_their_craft_and_two_lurk` checks the numbers.

The dread design's D7 (#1131): the ember beetle's acid blood pits a weapon that strikes it
(`corrodes = { chance = 25, what = "weapon" }`, `world/corrode.rs`) and spatters whoever strikes it in melee (`innate = ["acid_blood"]`, a
status that `retaliates` acid `1d3`, #1128's innate statuses), and the marsh troll is a brute that regenerates (`regen = 400`,
`2d8`, `base_hp` 36; `docs/dev/creatures/trolls.md`).

The will-o'-wisp (#1132, docs/dev/creatures.md "The unseen"): `[creature.invisible] cooldown =
300`, it winks out with its light (an invisible creature sheds none of its own) and comes again
unseen; its spark sets the struck one's mana ebbing (`ebbing`, 30 % of hits).
