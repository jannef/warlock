# Experience

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08c-combat.md. Keep it current with the code, as CLAUDE.md says.

Experience (spec §8 "Experience", M21; to 60 in M30, #226, design `docs/design/progression.md`
§3; to 100 in #877, `docs/design/prestige-classes.md` §5, `max_level` 100 = `MAX_LEVEL`, the
classes' default cap `class_max` 60): `content/experience.rs` holds the rules from `data/experience.toml`: the level curve in
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
no gameplay stream (a Knowledge mark, `Use::Discover`, draws from the training stream:
`World::discovered`, `world/knowledge.rs`). Creatures carry `level` and `xp` in `creatures/`, on the guideline
(`content::experience` tests list the deliberate outliers, and the roster tests compare the base
`xp`), and may carry `xp_percent`, a hand-set modifier with no formula (any whole number from 0
up, default 100): `CreatureDef::kill_xp` applies it, and `earn_kill` starts both the level
scaling and the first-kill bonus from it (`Experience::kill_xp` and `first_kill_xp` count in
`u64` and saturate, as the modifier has no ceiling). The saved state is
`WorldState::experience` and `progress` (the level follows from the total and the character);
the UI reads `PerceptionView::own_experience()`. Other kills (traps, creatures) give nothing, with
one exception: a kill by an ally of the player's pays the player through `earn_kill_at` at
`World::ally_share()` (their class's `ally_share`, else the allies' `ally_percent`;
`docs/dev/allies.md`; `fight.rs::dies`), as the owner decided. A creature made of another
(`Actor::split_from`, `world/split.rs::offspring_share`; docs/design/monster-dread.md §5.5) pays
less, after any ally's share: a half cut off a splitting creature `content::HALF_SHARE` (25 %),
one that crawled out of a corpse `CORPSE_SHARE` (10 %); the first-kill bonus is scaled with it,
and neither counts toward a quest's kills (`tally_kill` is skipped). Otherwise:
keep it that way by awarding only where the attacker is the player. `Progress` is where the
progression state of the design's later items (talents, due choices) goes.

## Prestige classes (#876; `docs/design/prestige-classes.md` §4)

One level, two classes. `Experience::level_for(xp, factor, cap)` stops at `cap`,
`World::level_cap` = `Character::level_cap`: the class's `ClassDef::max_level` (the class file's
`max_level`, else `[levels] class_max` of `data/experience.toml`, `Experience::class_max`, 60;
the bard's is 30, #874, the one class that names one;
a run without a character stops at `class_max` too;
filled in by `ClassDef::settle_cap` right after the experience file loads, which also checks the
table's levels within it) plus a prestige class's `levels` once called. XP past the cap banks as
it does at the curve's top (`PerceptionView::own_experience` shows no next level at a cap).
`PrestigeDef` (`content/prestige.rs`, `data/prestige/`) is a class in what a level reads;
`content::Table` is the one view of either kind, and `Character::class_of(content, level)` says
which table total `level` falls in and at what level of its own (past the cap: the prestige
class's, from 1); `Character::class_levels` gives every table with its level reached.

Every per-level read goes through it: `World::level_hp(content, level)` and `level_mana(content,
level)` (summed per level in `World::rise_from` and `level_up_mana`, and read for the lost level
in `boons.rs::lose_level`), `roll_count(.., level)` (`World::level_rolls(level)`; the view's
`roll_count()` and `bonus_trees()` read the first due's level, `PerceptionView::due_level`),
`skill_roll` (known for: the union of both classes'), `Table::bonus_at` in `level_up` and
`World::bonus_trees_at(level)` for each `Due` (a bonus pick is spent on, offered, and settled
against its own level's trees: `bonus_trees_due`, `bonus_offers_at`), and `class::grant` over
`class_levels` (creation, each level, restore; `held_already` by `class_of`).

The calling (`content/calling.rs`, `CallingDef`, a campaign's `data/callings/`; `world/calling.rs`):
`World::calling_status(calling)` (`CallingStatus`: Open, NotFrom, LevelNotMet, Called) and
`World::calling_open(class)` (the run's campaign's calling, or one outside every campaign, open
to this character); `World::call(calling)` is the one place a prestige class is given: sets
`Character::prestige`, says the `message`, raises `WorldState::experience` to the first prestige
level's threshold when it is short (`threshold_for(cap + 1, factor)`, exactly; a fuller bank is
left alone), then `rise_from(before)` brings every level the XP buys. `add_xp` is now
`add_xp` + `rise_from(before)`, so a cap that rises and XP that grows take the same path. The
dev action `call:ID` (`Action::Call`, `World::call_dev`) ignores the level. Draws: none of its
own. Dumps: `stats` `Class Bard 30, Hunter 4` (`Character::class_text`), `progress` the calling
rows (`dump::calling_rows`), JSON `character.prestige`, `character.classes`, and `callings` with
`--state`. Tests: `world/calling_tests.rs` on a fixture (the class tables stripped, the bard
capped at 12, the Warden prestige class of 5 levels and its calling in the showcase):
the cap and the bank, open and not, the exact XP and the first level's hit points and rolls,
the bonus trees per due, a full bank spent, the dev call, Unmade's lost level, a restore, a
replay; `world/strange_spirit_tests.rs` on the shipped cap and callings (both 30, kept together
by `the_callings_ask_for_the_bards_full_class`), a new bard to Hunter 1 at 31 by replay;
`content::prestige`, `content::calling`, and `content::class` tests the loading and every
refusal; `content::experience` the capped `level_for` and `class_max`.
