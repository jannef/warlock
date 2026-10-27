# Devils

The devil roster (#448): ten shared types, one file each in `assets/data/creatures/` (the lemure
5, imp 9, bearded devil 20, barbed devil 26, bone devil 32, erinyes 38, chain devil 42, horned
devil 48, ice devil 55, pit fiend 66), faction `devil` ("devils", hostile to the player and, by the
demons' `demon` faction's list (#449), to the demons), tags `devil` and
`fiend`. Since #531 Fyrdelf's population has the lemure, the imp, and the bearded devil (level 20,
met only out of depth on its twelfth floor), drawn up by the demons' fires; the rest lie deeper
than any dungeon.

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
own colour). The imp fights in melee (`combat = "caster"` only announces spells today).

Tests that list the factions by hand (equip, magic, object, read) carry `devil`; the mana tests'
fixture, which replaces the statuses, drops the three devils whose riders it does not define.

Waiting for mechanics (#448's follow-ups): several attacks a turn, flight, the imp's shapes and
invisibility, creature spells, animated chains, thorns, summoning, auras, infernal wounds, a
`large` size, a place in a population.
