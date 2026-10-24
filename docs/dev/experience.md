# Experience

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08c-combat.md. Keep it current with the code, as CLAUDE.md says.

Experience (spec §8 "Experience", M21; to 60 in M30, #226, design `docs/design/progression.md`
§3): `content/experience.rs` holds the rules from `data/experience.toml`: the level curve in
kills (`[kills]`: `plateau`, `ramp`, `slope_per_100`, and the creature XP `guideline`), turned
into a threshold table once at load with exact integer math (`Experience::threshold`, rounded
once per level from the exact fraction; a test pins the table's hash, so a deliberate change to
the curve updates it), `hp_level`, `mana_level` (mana per level: `World::level_up_mana`,
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
(`content::experience` tests list the deliberate outliers). The saved state is
`WorldState::experience` and `progress` (the level follows from the total and the character);
the UI reads `PerceptionView::own_experience()`. Other kills (traps, creatures) give nothing:
keep it that way by awarding only where the attacker is the player. `Progress` is where the
progression state of the design's later items (talents, due choices) goes.
