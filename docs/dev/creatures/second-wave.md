# The second wave

General conventions: `docs/dev/creatures.md`.

The dread design's second-wave types (#1138; `docs/design/monster-dread.md` §15 and §12 D14):
five dread types, content on the mechanisms the dread slices built, and the hydra's heads. Shared
types, one file each in `assets/data/creatures/`, on that directory's `README.toml`. All come as
strays (the dread draw's, `dread_one_in`, and the out-of-depth draws); no population names them
yet. Each dread type has a `first_sight` line (D9); the leavings are the ones that suit.

They follow #383's formulas by level and role, the natural attacks' dice summed (the hydra's five
heads share a brute's budget). XP, max HP and dice follow the guideline and the ±35 % band;
`content/second_wave_roster_tests.rs` checks each type's gimmick; `content/dread_roster_tests.rs` lists the five at their tiers and their leavings.

## Roster

| Type | Level | Role | Tier | Faction | Max HP | Attack | Gimmick | Leavings |
|---|---:|---|---|---|---:|---|---|---|
| `hydra_head` | 4 | small | common (not dread; `stray = false`) | marsh | 12 | bite 1d6 | what a hydra dies into | — |
| `cockatrice` | 7 | small | rare | vermin | 20 | beak 1d8, Stiffening 25 % behind Toughness 13 | stiffening by touch (D2); a hiss cry | statue |
| `rust_monster` | 12 | standard | uncommon | vermin | 47 | feelers 2d6 bludgeoning | `corrodes = { chance = 50, what = "both", kind = "rust" }` (D7, #1168): its hits rust the armour worn, the weapon that strikes it rusts; quick (1200) | — |
| `black_pudding` | 18 | shambler | rare | vermin | 92 | pseudopod 3d6 acid | `splits` under slashing and piercing (D5); `corrodes` both, 25 (D7); no vitals, no corpse, bloodless; slow (700) | acid pit |
| `hydra` | 24 | brute | rare | marsh | 123 | five bites 1d6 | five natural attacks (#1099), `regen = 400` (D7: a brute), `dies_into` 1d4 hydra heads (D5); fire hurts it more; a hiss cry | — |
| `gorgon` | 28 | brute | very rare | vermin | 140 | horns 3d10 | `combat = "dragon"`: a breath of grey vapour (4d6 poison, a 5-cell cone, a minute's cooldown) whose `riders` give a degree of Stiffening behind Toughness 17; immune to poison | statue |

## Mechanism touches

Two small extensions of what D1–D9 built, each for one of these types:

- **A breath's riders** (`content/breath.rs`, `world/breath.rs`): `[creature.breath]` takes an
  optional `riders` list (`on_hit` entries, statuses and drains as a gaze's), each falling by its
  chance on everyone the cone harmed and still alive (`World::rider`). The gorgon's. The wiki's
  breaths table has a *Then* column. Test: `world/statue_tests.rs`
  (`the_gorgons_breath_stiffens_...`).
- **`corrodes` both ways** (`CorrodesWhat::Both`, `CorrodesWhat::eats`): the armour and the
  weapon forms at the one chance. The rust monster's and the black pudding's. Test:
  `world/corrode_tests.rs::the_rust_monster_rusts_both_ways_and_never_cloth`.

The hydra's heads: `world/split_tests.rs::heads_crawl_out_of_a_dead_hydra`.

**Material drops** (#1341; `data/drops/`, docs/dev/objects.md "Material drops"): own rows on the cockatrice, gorgon, and hydra (and the basilisk): bone 1d2 (50 %), hardened leather 1 (40 %). The gorgon (`combat = "dragon"`) carries `dragon` since, as every dragon type does, and rolls `dragon.toml` before its own. The black pudding (`eyeless`, no corpse) and the hydra's heads (untagged) leave nothing; the rust monster stays untagged.
