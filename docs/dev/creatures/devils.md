# Devils

The devil roster (#448): ten shared types, one file each in `assets/data/creatures/` (the lemure
5, imp 9, bearded devil 20, barbed devil 26, bone devil 32, erinyes 38, chain devil 42, horned
devil 48, ice devil 55, pit fiend 66), faction `devil` ("devils", hostile to the player and, by the
demons' `demon` faction's list (#449), to the demons), tags `devil` and
`fiend`. Since #531 Fyrdelf's population has the lemure and the imp, drawn up by the demons' fires; the
rest (the bearded devil, level 20, and the deeper) wait for deeper places. All come as strays too
(docs/dev/creatures.md), the deep ones only far out of depth.

They follow #383's formulas by level and role (shambler: the lemure; small: the imp; brute: the
horned devil, ice devil, and pit fiend; standard: the rest). `content/devil_roster_tests.rs`
(`ROLES`) checks every `devil` type's tags, XP, max HP, and dice within ±35 %, and that the
hierarchy's levels climb.

One natural attack each (creatures have no multi-attack): the riders are poisoned (imp, bearded
devil, bone devil, pit fiend), bleeding (barbed and horned devils), held (the chain devil, 25 % at
20), and sluggish (the ice devil's cold claws, `damage_type = "cold"`). Fire 100 and poison 50 for
all, cold 50 from the bearded devil up (the ice devil 100), poison 100 for the bone devil and the
pit fiend; no corpses. Gear: the bearded devil's glaive, the erinyes' long sword and chain mail,
loot on the erinyes and the pit fiend. Glyphs `u` (lemure), `i` (imp), `U` (the rest, each in its
own colour). The imp fights in melee.

Tests that list the factions by hand (equip, magic, object, read) carry `devil`; the mana tests'
fixture, which replaces the statuses, drops the three devils whose riders it does not define.

Waiting for mechanics (#448's follow-ups): several attacks a turn, flight, the imp's shapes and
invisibility, creature spells, animated chains, thorns, summoning, auras, infernal wounds, a
`large` size, a place in a population.

Presence (#1128, `world/aura.rs`): the barbed devil innate `barbed_hide` (status.toml: `retaliates` `1d6` piercing, "The barbed hide stabs you."); fear auras on the bone devil (radius 2, Willpower 17) and the pit fiend (radius 3, 22, and innate `fire_cloaked`); the ice devil's cold aura (`sluggish` 2, radius 2, Toughness 20, cold). `content::aura_roster_tests` lists every row.
Theft (docs/design/monster-dread.md §5.6, #1130; `docs/dev/creatures.md` "Theft and the
blink"): the imp `steals = { what = "small", chance = 30, cooldown = 60 }`, a coin purse at a
time; its poison kept, its invisibility D8's. It is not dread (a trait, design §6).

The imp (#1132, docs/dev/creatures.md "The unseen"): `[creature.invisible] cooldown = 300`,
it fades from sight every five minutes and stays in the room; its poison kept.

## Variance

The variance pass (#1134, `docs/dev/creatures.md` "Variance by role") sets the roles' to-hit (striker `L/4 + L/8`, brute and caster `L/4 − L/8`, the armed `L/4`), threat and crit (striker 19, brute 24 at ×3), and hit-point factors (quick and caster 0.5, brute 2.0). **Speed**: the hierarchy marches, 1000 to 1150 (the imp the quickest), the lemure crawls at 800; the dread barbed devil and pit fiend at 1400.

Gear condition (#1193, `data/gear_condition.toml`'s `devil` row): the lawful host keeps its arms, damaged 8 %, Fine 10 %, Marvellous 2.5 %, Masterwork 0.6 % (docs/dev/creatures.md, "Gear condition").
