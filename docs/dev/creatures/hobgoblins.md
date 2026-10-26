# Hobgoblin roster

General conventions: `docs/dev/creatures.md`.

The hobgoblins (#454), the goblin tribe's second tier: seven shared types from level 9 to 24
(`hobgoblin`, `_archer`, `_legionnaire`, `_warcaster`, `_sergeant`, `_captain`, `_warlord`),
faction `goblins`, tags `goblin` and `hobgoblin` (plus `archer`, `caster`), glyph `g` in
colours apart from the goblins'. Stats from #383's base for aware creatures (In 9 + 0.2L, Wp 10
+ 0.3L, Pr 8 + 0.25L for the living), `base_hp = 2.5 × L × role` (soldier 1.0, archer 0.9,
caster 0.8, officer 1.1); they fight with their gear (racial long sword, long bow, spear,
quarterstaff, great sword), so no natural attack and no `on_hit` rider.
`content/hobgoblin_roster_tests.rs` holds them to the XP guideline, the HP formula (±20 %),
the faction and tags, levels above every other goblin's, and a racial weapon. Fyrdelf's
population has all but the warlord, which no place is deep enough for yet; they and the duergar
fight on sight (`bestiary_tests`' stances).
