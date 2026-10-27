# Skills

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08i-character.md. Keep it current with the code, as CLAUDE.md says.

Skills (spec §8 "Skills", #132) live in `warlock-core`: `skills.rs` (`Skills` per actor: trained
skills only, a missing one is untrained and gives nothing; `Effect` and `Use`, the typed benefit
and training-mark hooks; `Benefit` growth by `steps` or a `curve`; `Ladder` and `RollRules`, the
level-up rolls' data; `Skills::mark` adds a mark, `Skills::roll` applies a roll and spends the
marks), `content/skill.rs` (`data/skills.toml`: the skills, `[ladder]`, `[rolls]`; adding a skill
is one entry), `world/train.rs` (marks for the player only, drawn from the `training` stream in
`WorldState`, never the gameplay stream; nothing said), and `world/skill_roll.rs` (#230: the
level-up rolls, below).

**Growth is level-up rolls only** (design `docs/design/progression.md` §5). Marks are a
reservoir: `World::level_up` (`world/talent.rs`) puts `World::level_rolls()` rolls (class
`rolls` + race `rolls` + `RollRules::for_intelligence` of the raised Intelligence; 0 when the
player is trained in nothing) into each level's `Due::rolls`; `refuse_due` refuses everything
else ("You have skill rolls to spend."); `advance:SKILL` (`World::advance_skill`) throws
`Ladder::die(step(marks) + known_for)` from `levels_rng` (made on the first draw), adds +1 for a
class's other or a race's skill (`ClassDef::is_known_for`/`trains`, `RaceDef::skills`), and `Skills::roll` adds it (halved from `soft_cap`) and spends
`RollRules::spent` of the marks. The dice and the count are each computed in one public
function the rule and the level-up screen share (#543): `world::skill_roll` (a `SkillRoll`: the
marks, the practice die, the known-for step, the die, the `RollBonus` and why, `halved`) and
`world::roll_count` (a `RollCount`: class, race, Intelligence, talents), read by the screen
through `PerceptionView::skill_roll`/`roll_count` and by the dev `progress` dump (`rolls a
level ...`, `roll dice athletics 1d3, ...`). `levelup:auto` spends rolls on the first trained skill
(`advance_auto`). The Skills screen's meter (`skills::progress`) is the ladder step in quarters.
Tests: `skills.rs` (ladder, rolls, meter), `content/skill.rs`, `content/class.rs`
(`known_for`, `rolls`), `world/skill_roll_tests.rs` (counts, refusals, dice and affinities,
streams, saves and replays), `world/skills_tests.rs` (marks by use). Race and class grants are dice rolled in `character.rs` after every
other roll, so they never move a score. Rules read a benefit through `Actor::skill_effect`
(the actor's skills plus what magic items' skill affixes add, `Skills::effect_with`), never
`Skills::effect` directly. Each effect is read in one place: Dodge in
`Actor::defence`, quiet in `noise::noise`, hearing in `Actor::hearing`, burden in
`World::burdened` (`Burden::slow_eased`), travel in `World::step_price`, bargain in
`world/shop.rs`, kick (athletics: Strength for kicking a door in) in `World::kick_door`, pick
(Open lock: picking a lock, and trained at all, the Pick lock ability) in `World::pick_lock`,
search, search reach, and spot in `world/search.rs` (`search_modifier`, `search_turn`; Spot
trained at all lets the player notice in passing, `Skills::trains_effect`), disarm in
`world/disarm.rs` (`disarm_modifier`; trained at all, Disable device gives the Disarm ability,
`Ability::had_by`), steal in `world/steal.rs` (trained at all lets the player steal,
`docs/dev/stealing.md`), heal in `world/heal.rs` (trained at all gives the Heal ability;
`docs/dev/status-effects.md` "Treating oneself"), study (Spellcraft and Decipher script: points off the study check) in
`world/learn.rs::study_odds` (docs/dev/spells.md "Learning"; its use `Use::Study`, reported
when a study ends, is also `StatUse::Study`); each use
is reported where it happens (`make_noise`, `hear`, `fight.rs::resolve`, `apply`, `player_moves`,
`kick_door`, `search_turn` for `search`, `notice_hidden` for `spot`, `World::disarm` for
`disarm`, `pick_lock` for `pick`, `World::steal` for `steal`, `World::heal` for `heal`, and `World::sidesteps` for a
known trap avoided, `dodge`, besides `fight.rs::resolve`). A new
effect or use is a variant there plus its one hook. The player sees values only
(`PerceptionView::own_skills`, the Skills screen `ui/skills_screen.rs`, `A`); never marks or
benefit numbers. Skills get no keys of their own: active uses are chosen from the one abilities
menu (#156, `docs/dev/screens.md`): each trained skill of kind `action` is an `Ability::Skill`
there (`warlock_core::ability`), unavailable ("not usable yet") until its use exists, when it
becomes an ability of its own in the skill's place (`Ability::use_of`): Disable device is
`Ability::Disarm` (`docs/dev/traps.md`), Open lock `Ability::PickLock` (`docs/dev/doors.md`),
Steal `Ability::Steal` (`docs/dev/stealing.md`), Heal `Ability::Heal` (`docs/dev/status-effects.md`).
Pending hooks:
athletics (climbing out of pits, #128) and escape artist (pits, #128).
