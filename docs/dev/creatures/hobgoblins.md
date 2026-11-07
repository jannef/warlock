# Hobgoblin roster

General conventions: `docs/dev/creatures.md`.

The hobgoblins (#454), the goblin tribe's second tier: seven shared types from level 9 to 24
(`hobgoblin`, `_archer`, `_legionnaire`, `_warcaster`, `_sergeant`, `_captain`, `_warlord`),
faction `goblins`, tags `goblin` and `hobgoblin` (plus `archer`, `caster`), glyph `g` in
colours apart from the goblins'. Stats from #383's base for aware creatures (In 9 + 0.2L, Wp 10
+ 0.3L, Pr 8 + 0.25L for the living), `base_hp = 2.5 × L × role` (soldier 1.0, archer 0.9,
caster 0.5, officer 1.1; #1134); they fight with their gear (racial long sword, long bow, spear,
quarterstaff, great sword), so no natural attack and no `on_hit` rider.
`content/hobgoblin_roster_tests.rs` holds them to the XP guideline, the HP formula (±20 %),
the faction and tags, levels above every other goblin's, and a racial weapon. Fyrdelf's
population has all but the captain (19; it left when Fyrdelf's danger was squished to 5–16,
#534) and the warlord, which no place is deep enough for yet; they and the duergar
fight on sight (`bestiary_tests`' stances).

Spells (#1102, docs/dev/creatures.md "Casters"; in the order each prefers them): the hobgoblin warcaster `frost`, `arcane_bolt`, `magic_missile` (mana 40).

## Variance

The variance pass (#1134, `docs/dev/creatures.md` "Variance by role") sets the roles' to-hit (striker `L/4 + L/8`, brute and caster `L/4 − L/8`, the armed `L/4`), threat and crit (striker 19, brute 24 at ×3), and hit-point factors (quick and caster 0.5, brute 2.0). **Speed**: drilled soldiers marching at 1000 (the rank-and-file within 950 to 1050), the warlord's 1050.

Gear materials (#1191): the faction's `gear` in `data/factions.toml`: the goblins' faction (goblins and hobgoblins) favours bronze (30 %), copper (10 %), and bone (10 %), each its share of the draws for an item whose construction list holds it, the usual draw the rest (`docs/dev/objects.md` "Materials and variants").
