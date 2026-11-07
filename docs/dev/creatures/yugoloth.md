# Yugoloth roster

General conventions: `docs/dev/creatures.md`.

The yugoloths (#446): five shared types, one file each in `assets/data/creatures/`, faction
`yugoloth` (hostile to the player only; neutral to everyone else, the demons and devils included,
since relations are only symmetric hostile lists and there is no hire yet), tags `["yugoloth",
"fiend", ...]` (the arcanaloth also `caster`), on the undead roster's formulas by level and role:
the mezzoloth 22 (standard), canoloth 29 (quick), nycaloth 36 (brute), arcanaloth 46 (caster),
and ultroloth 58 (standard). Mercenary fiends of the lower planes: immune to acid and poison,
resisting fire, cold, and lightning 50, no corpse. Every
`yugoloth` type is written to its level and role, the XP guideline, ±35 % of the max HP and dice
formulas, and the two tags (faction and `yugoloth` tag go together). Their riders map onto what
exists: the canoloth's barbed tongue latches (`latched`, 20 % at 18) and bleeds, the nycaloth's claws bleed,
the arcanaloth's poison, the ultroloth's gaze (transfixed, frightened; #1125, range 5, 40 %; docs/dev/creatures.md
"Gazes"). They ship
in no population (no danger window reaches level 22 yet), so they come only as strays drawn far
out of depth (docs/dev/creatures.md).

Spells (#1102, docs/dev/creatures.md "Casters"; in the order each prefers them): the arcanaloth `fire_cloak`, `chain_lightning`, `fireball`, `magic_missile` (mana 130).

## Variance

The variance pass (#1134, `docs/dev/creatures.md` "Variance by role") sets the roles' to-hit (striker `L/4 + L/8`, brute and caster `L/4 − L/8`, the armed `L/4`), threat and crit (striker 19, brute 24 at ×3), and hit-point factors (quick and caster 0.5, brute 2.0). **Speed**: mercenaries at a soldier's pace, 1000 to 1100, the canoloth's hunt at 1250; the ultroloth, dread, at 1400.

**The disarming blow** (#1166, `docs/dev/creatures.md`): the ultroloth carries `disarms = { chance, difficulty }`, a melee hit that may knock the player's main-hand weapon onto an open cell beside them (20 % at 24).
