# Summoners

General conventions: `docs/dev/creatures.md`; the mechanism: `docs/dev/summoning.md`.

The summoners (#520): the tag `summoner` on every type with a `[creature.summon]`, each also of
another tribe. Three shared types, one file each in `assets/data/creatures/`, on #383's formulas
for a caster (XP `5L + 10`, St −3, base_hp `2.5 × L × 0.8`, In and Wp +4 over the living base):

| L | id | faction | tags | calls forth | count | mana (cost) | most |
|---|---|---|---|---|---|---|---|
| 8 | `goblin_wizard` | goblins | goblin, caster, summoner | goblin (twice as likely), goblin archer, hobgoblin, warg | 1d3 | 22 (8) | 6 |
| 18 | `necromancer` | cult | cultist, caster, summoner | tag `undead`, levels 3 to 12 | 1d4 | 30 (10) | 8 |
| 24 | `dark_wizard` | cult | cultist, caster, summoner | tag `devil`, levels 5 to 20 | 1d3 | 36 (12) | 5 |

All have a 30 s cooldown and the default range (8) and spread (2). The goblin wizard stays at
level 8, below the hobgoblins (`hobgoblin_roster_tests` holds every hobgoblin above every other
goblin); the wargs it calls join the goblins. The necromancer and the dark wizard are human
(`race = "human"`) and of the cult's faction and tag (`cultist`): living men dealing with the
same powers; the undead and devils they call take the cult's side, so they leave the goblins and
kobolds alone while the summoner lives in their ranks.

`content/summoner_roster_tests.rs` checks every `summoner` against its table, another tribe's
tag, and mana, and the three above against the XP guideline and the HP formula (±35 %);
`content::tag`'s test lists them. None is in a population yet: the showcase's summoning hall
(`summoners` scenario) places them. The three drow priestesses (#536) summon demons: their table is in
`docs/dev/creatures/drow.md` (the drow roster's checks cover them, not this file's `ROSTER`).
