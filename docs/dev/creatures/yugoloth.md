# Yugoloth roster

General conventions: `docs/dev/creatures.md`.

The yugoloths (#446): five shared types, one file each in `assets/data/creatures/`, faction
`yugoloth` (hostile to the player only; neutral to everyone else, the demons and devils included,
since relations are only symmetric hostile lists and there is no hire yet), tags `["yugoloth",
"fiend", ...]` (the arcanaloth also `caster`), on the undead roster's formulas by level and role:
the mezzoloth 22 (standard), canoloth 29 (quick), nycaloth 36 (brute), arcanaloth 46 (caster),
and ultroloth 58 (standard). Mercenary fiends of the lower planes: immune to acid and poison,
resisting fire, cold, and lightning 50, no corpse. `content/yugoloth_roster_tests.rs` holds every
`yugoloth` type to its level and role in `ROSTER`, the XP guideline, ±35 % of the max HP and dice
formulas, and the two tags (faction and `yugoloth` tag go together). Their riders map onto what
exists: the canoloth's barbed tongue holds (20 % at 18) and bleeds, the nycaloth's claws bleed,
the arcanaloth's poison, the ultroloth's blows carry its gaze (transfixed, frightened). They ship
in no population (no danger window reaches level 22 yet).
