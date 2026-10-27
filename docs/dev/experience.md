# Experience

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08c-combat.md. Keep it current with the code, as CLAUDE.md says.

Experience (spec §8 "Experience", M21; to 60 in M30, #226, design `docs/design/progression.md`
§3): `content/experience.rs` holds the rules from `data/experience.toml`: the level curve in
kills (`[kills]`: `plateau`, `ramp`, `slope_per_100`, and the creature XP `guideline`), turned
into a threshold table once at load with exact integer math (`Experience::threshold`, rounded
once per level from the exact fraction; a test pins the table's hash, so a deliberate change to
the curve updates it), the early levels' discount (`[early] percent`, owner 2026-10-27: level 2
pays `percent[0]` % of its curve XP, level 3 `percent[1]`, and so on, levels past the list 100 %;
folded into `kills_fraction`, so `xp_for`, `threshold`, and `kills_for` all see it; refused when
longer than levels 2 to `max_level`, outside 1 to 100, or decreasing; the test
`early_levels_are_discounted` pins the owner's totals and checks that levels 12 and up cost what
they would undiscounted), `hp_level` (50, #557: levels 51 to 60 add no hit points but Hardened's; the thief's class `hp` is 4), `mana_level` (mana per level: `World::level_up_mana`,
`docs/dev/mana.md`), the WoW-style `kill_xp` scaling with its zero difference
bands, and `[discovery]` (`first_kill_xp`, `arrival_xp`). A character's thresholds are scaled
by their race's and their class's `experience` percents (`Factor`, `threshold_for`,
`level_for(xp, factor)`); `Factor::of(content, spec)` is the one way to get the player's, and
every reader of the level goes through it (`World::player_level`,
`PerceptionView::own_experience`). `pace_words` is how a factor reads on the character creation
steps (and in the wiki).

`world/experience.rs` applies them: `earn_kill` from `fight.rs::dies` when the player lands the
killing blow, or caused the status effect that killed (`Effect::by`), adds the first-kill bonus
the first time a creature type pays (remembered by id in `WorldState::progress`, `Progress::
discovered`, left out of the JSON while empty); `discover_level` from `World::travel` when the
arrival made the level's state (the start level, the overworld, and local areas give none);
`grant_xp`, level-up messages queued in `World::news` and said after the attack's own, or at
the end of `perceive` for a creature that bled to death and for an arrival. Discovery draws from
no random stream. Creatures carry `level` and `xp` in `creatures/`, on the guideline
(`content::experience` tests list the deliberate outliers, and the roster tests compare the base
`xp`), and may carry `xp_percent`, a hand-set modifier with no formula (any whole number from 0
up, default 100): `CreatureDef::kill_xp` applies it, and `earn_kill` starts both the level
scaling and the first-kill bonus from it (`Experience::kill_xp` and `first_kill_xp` count in
`u64` and saturate, as the modifier has no ceiling). The saved state is
`WorldState::experience` and `progress` (the level follows from the total and the character);
the UI reads `PerceptionView::own_experience()`. Other kills (traps, creatures) give nothing:
keep it that way by awarding only where the attacker is the player. `Progress` is where the
progression state of the design's later items (talents, due choices) goes.
