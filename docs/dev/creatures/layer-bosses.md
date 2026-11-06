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

Depths 60 and 70 are #1122. Tests: `world/layer_boss_tests.rs` (the content checks
among them), `creature_body_tests` (each `natural` on its body's line), `summoner_roster_tests`
(the three summoners carry `summoner` and another tribe's tag).
