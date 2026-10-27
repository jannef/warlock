# Drow roster

General conventions: `docs/dev/creatures.md`.

The drow (#519, levels 14 to 50): the dark elves of the deep, twelve shared types, one file each
in `assets/data/creatures/` (`drow_*`). Faction `drow` (after `duergar` in
`assets/data/factions.toml`): hostile to the player, the kobolds, the goblins (the hobgoblins
with them), the orcs, and the duergar, their rivals of the deep roads; neutral to the demons (the
priestesses would call on them), the vermin (the spiders sacred to their goddess), the undead,
the fire things, the giants, the trolls, and everyone else. Not an animal faction. Tags: `drow`
on every type, `archer` on the crossbowman and the arbalester, `caster` on the mage, the
archmage, and the three priestesses, `priest` (new) on the priestess, the high priestess, and the
matron. All `alignment = "evil"`, `race = "drow"` (the playable race: eating one is eating kin to
a drow character), `vision = "darkvision"`, medium, open doors, quiet footsteps, keen hearing,
and `resist = { poison = 25 }` (the race's; 50 for the high priestess and the matron). Glyph `e`
(the deep elves' letter in other roguelikes; before them only the elemental motes used it, in
bright elemental colours), each drow a violet, indigo, steel, teal, or rose of its own.

## Formulas

`content/drow_roster_tests.rs` keeps them to #519's numbers, the undead roster's (#383) level
scaling on the drow's own base: xp `5L + 10`; St `11 + 0.4L`, Dx `12 + 0.25L`, To `11 + 0.4L`,
In, Wp, and Pr `10 + 0.4L` (casters +4 each), within 1, capped at 40; by role (standard ±0,
archer Dx +3, quick Dx +5 To −1, caster St −3, officer St +2 To +2) and `base_hp = 2.5L × role`
(standard 1.0, archer and quick 0.9, caster 0.8, officer 1.1), max HP within ±35 %; dice the
ladder's nearest to `2 + 0.55L` (then `13 + 0.25(L − 20)`), casters one step lower, within
±35 %; `to_hit = L / 10` from L20. Speeds are #383's (quick `1200 + 5L`, else 1000 then +4 a
level past 20).

## Poisoned weapons

A rider (`on_hit`) fires only on a natural attack, never on a wielded weapon
(`World::on_hit_of`; only a unique weapon's Inflict power rides on gear). So every drow fights
with a natural attack named for its weapon ("envenomed short sword", "scourge", "twin
scimitars"), carrying `poisoned` (and `weakened` on the assassin and the two eldest priestesses,
`frightened` on the captain), and its gear is armour, cloak, boots, amulets, and coins, never a
weapon (the roster test rolls the gear to check it). The archers shoot with a natural throw
(`[creature.throw]`, object `bolt`, named "poisoned bolt", verb "shoots"): the drow's sleep poison
as `drowsy`, and `poisoned`. The blademaster's two blades are `attack_cost = 600` with its dice a
step below the formula's (`world/cost_tests.rs` lists it). The assassin's dagger threatens from
19 and multiplies by 3.

## Roster

The warrior 14 (common), crossbowman 16 (common), stalker 19 (quick, nearly soundless), mage 22
(rare caster), priestess 24 (rare caster, scourge), blademaster 28 (quick, two strikes a turn),
assassin 32 (rare, quick, the strongest venom), arbalester 34 (sees and shoots farthest),
captain 37 (rare officer), archmage 40 (rare caster), high priestess 44 (very rare), and the
matron 50 (very rare, `unique = true`: drawn at most once a run; no proper name, so shared).
Loot from 10 to 100.

## Populations

None: their levels overlap only Fyrdelf's (danger 5–16) at the bottom, and a duergar delf is no
drow place. They wait for a deep place of their own.

## Follow-ups

The priestesses' demon summoning (the summoning system's summon table), poison on wielded
weapons, creature light sensitivity (the race's `light_blind`), damaging creature spells, spider
mounts, and a drow place to give them a population.
