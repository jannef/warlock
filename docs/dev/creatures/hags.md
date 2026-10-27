# Hag roster

General conventions: `docs/dev/creatures.md`.

The hags (#451): the green hag 18 (caster), sea hag 26 (standard), annis hag 34 (brute), and
bheur hag 42 (caster), shared types in `assets/data/creatures/` on the undead roster's formulas;
`content/hag_roster_tests.rs` holds every type of the `hags` faction to them (its own `ROLES`,
±35 %) and checks the `hag` tag, which the bog hag (faction `marsh`, the fen's) carries too.
Faction `hags` (hostile to the player only), so the fixtures that write their own factions file
list it. Their riders are existing mechanics only: the green hag's `transfixed` and `ill_wished`
(illusions and misery), the sea hag's `frightened` and a Toughness drain (her death gaze), the
annis hag's `held` (her embrace, grip 18), the bheur hag's cold damage and `sluggish` (no hunger
rider exists); `world/mana_tests.rs` drops the green and bheur hags with its own status file.
Illusions, death gazes, blizzards, shapeshifting, covens, and water-dwelling (the sea hag walks
the land only, #196) are follow-ups. They ship in no population (checked again in #531): only Fyrdelf's window (8–19)
reaches the green hag's level, and a volcano is no place for her.
