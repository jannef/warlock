# Hag roster

General conventions: `docs/dev/creatures.md`.

The hags (#451): the green hag 18 (caster), sea hag 26 (standard), annis hag 34 (brute), and
bheur hag 42 (caster), shared types in `assets/data/creatures/` on the undead roster's formulas;
`content/hag_roster_tests.rs` holds every type of the `hags` faction to them (its own `ROLES`,
±35 %) and checks the `hag` tag, which the bog hag (faction `marsh`, the fen's) carries too.
Faction `hags` (hostile to the player only), so the fixtures that write their own factions file
list it. Their riders are existing mechanics only: the green hag's `transfixed` and `ill_wished`
(illusions and misery), the sea hag's `frightened` and a Toughness drain (her death gaze, `[creature.gaze]` since #1125:
range 4, 35 %; docs/dev/creatures.md "Gazes"), the
annis hag's `held` (her embrace, grip 18), the bheur hag's cold damage and `sluggish` (no hunger
rider exists); `world/mana_tests.rs` drops the green and bheur hags with its own status file.
Illusions, death gazes, blizzards, shapeshifting, covens, and water-dwelling (the sea hag walks
the land only, #196) are follow-ups. They ship in no population (checked again in #531): only Fyrdelf's window (5–16)
reaches the green hag's level, and a volcano is no place for her. They come only as strays (docs/dev/creatures.md).

Spells (#1102, docs/dev/creatures.md "Casters"; in the order each prefers them): the green hag `curse_of_sloth`, `mesmerise`, `blight` (mana 45); the bheur hag `curse_of_sloth`, `rime`, `frost` (mana 70).

Presence (#1128, `world/aura.rs`): the bheur hag's cold aura (`sluggish`, radius 2, Toughness 18, cold). Her hunger rider (design §5.12) is not built yet.

## Variance

The variance pass (#1134, `docs/dev/creatures.md` "Variance by role") sets the roles' to-hit (striker `L/4 + L/8`, brute and caster `L/4 − L/8`, the armed `L/4`), threat and crit (striker 19, brute 24 at ×3), and hit-point factors (quick and caster 0.5, brute 2.0). **Speed**: the hags hobble at 1000 (the bheur hag's winter at 1100); the sea hag, dread, at 700, slow and awful out of the water.
