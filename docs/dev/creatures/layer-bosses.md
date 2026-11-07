# The layer bosses of Grundlēas (#1118)

The unique bosses that keep every tenth floor of Grundlēas, the endless branch (spec §7 "The
endless branch"). The rules (the boss rows, the lair, the hidden stairs, made again until slain)
are in `docs/dev/mapgen.md` ("Layer bosses") and `crates/warlock-core/src/world/layer_boss.rs`;
this file holds the roster.

- **Faction** `layer_keepers` ("keepers of the layers", `assets/data/factions.toml`): hostile to
  the player only, not animals (a Beast-friend calms none). What a boss calls up joins it. No
  roster test binds the faction: each boss is hand-tuned, not on #383's formulas.
- **Campaign-owned**: `assets/campaigns/ascmearc/data/creatures/`, one file each, `unique = true`,
  in no population (uniques never stray): only the lair, a pinned vault in
  `assets/campaigns/ascmearc/vaults/` (`dungeons = ["grundleas"]`), places one.
- **Proper names** in the Ascmearc's Old English register (`content::is_proper`: said without
  an article). XP is the guideline for the level, `xp_percent = 200`; `loot = 100`.
- **Never on the stairs**: `Content::is_layer_boss` keeps a boss out of the stair followers
  (`world/stair_followers.rs`), so a boss is never on two floors at once.

| Depth | Id | Name | Level | What sets the fight apart | Lair |
|---|---|---|---|---|---|
| 10 | `gnaegan` | Gnægan the Rat King | 11 | summons 2d3 giant rats every 18 s (12 at most), three bites, one carrying filth fever | `gnaegans_midden`, rats about him, bones and loot |
| 20 | `attorcoppe` | Āttorcoppe the Spinner | 21 | lurks; a bite that poisons and webs; calls 1d4 barrow spiders every 24 s | `attorcoppes_web`, an unlit hall, lurking spiders |
| 30 | `hrimwyrm` | Hrīmwyrm the White | 31 | a dragon (#1099): 14d8 frost in a ten-cell cone every 36 s, claws and bite; immune to cold, fire −25 | `hrimwyrms_bed`, a long hall lit by blue crystals, still pools, a hoard |
| 40 | `wealaf` | Wēalāf the Unburied | 41 | calls up 1d3 undead of levels 20 to 35 every 30 s; a blade that drains Strength | `wealafs_hall`, braziers, a court of six grave knights |
| 50 | `grundwyrgen` | Grundwyrgen the Mere-Wife | 51 | a brute: two claws that may hold, a bite, 298 hit points | `grundwyrgens_mere`, deep water filling the hall |
| 60 | `ligetdry` | Līgetdrȳ the Storm-Wright | 61 | a caster (#1102): haste on himself, silence, chain lightning, lightning bolt; 420 mana; a weak staff; immune to electricity | `ligetdrys_stones`, staggered standing stones to break his line, storm-lit crystals |
| 70 | `staneoten` | Stānēoten the Hurler | 71 | a thrower: a 7d10 boulder (range 16) every action he has a line, staggering; a maul that stuns; 460 hit points | `staneotens_gallery`, a long low gallery, boulders for cover |
| 80 | `nihtscinna` | Nihtscinna the Hollow | 81 | incorporeal and lurking: waits in and walks through the walls (wall Defence 10); physical damage halved; a necrotic touch that drains Willpower and frightens | `nihtscinnas_warren`, a warren of short thin walls, unlit |
| 90 | `waelcyrge` | Wælcyrge the Chooser | 91 | speed 1900: nearly two actions to one; a spear struck twice, both bleeding | `waelcyrges_field`, an open hall of bones, no cover |
| 100 | `nithdraca` | Nīðdraca the Hate-Wyrm | 100 | a dragon and a summoner: 22d8 fire in a twelve-cell cone every 30 s, 1d2 demons of levels 40 to 70 every minute (4 at most), claws that set alight; 680 hit points; immune to fire, cold −25 | `nithdracas_hoard`, a long hall, lava in its corners, a hoard |

Depths 60 to 100 are #1122; past 100 a tenth floor is ordinary. **Levels past 70** (the owner: "can
be whatever level you deem suitable"): 81, 91, and 100, not 101, so the deepest keeper stands at
the player's own cap (`experience.toml`'s `max_level`, 100). Nothing had to be extended for them:
a creature's level is a `u8` with no cap at load, the XP guideline (5 × level + 10) and
`Experience::kill_xp` take any level, and the body rule is a formula by
level. Tests: `world/layer_boss_tests.rs` (the content checks
among them; the depths list runs to 100), `summoner_roster_tests`
(the four summoners carry `summoner` and another tribe's tag).

**The disarming blow** (#1166, `docs/dev/creatures.md`): Wælcyrge carries `disarms = { chance, difficulty }`, a melee hit that may knock the player's main-hand weapon onto an open cell beside them (20 % at 28).
