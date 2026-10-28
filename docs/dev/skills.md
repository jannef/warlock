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
other roll, so they never move a score. A class's **optional skills** (#662,
`ClassDef::optional_skills`, `content::OptionalSkill { skill, chance, dice }`; the loader checks
the ids, chance 1 to 99, the dice within 0 to 100, and that none is one of the class's `skills`)
are rolled by `character::optional_skills` from their own stream,
`character/optional-skills/<spec>` (chance, then dice, in skill order; a class without any draws
nothing), so no other roll moves; the result is just the actor's `Skills`, saved and hashed as
before, and a replay needs no new field (the roll is a function of the seed and spec). Classes
that have them start with other skills on old seeds, so old replays' hashes change.
`ClassDef::trains` (the level-up roll's +1) stays the guaranteed list. Tests:
`content/class.rs` (parsing, refusals), `character.rs`
(`optional_skills_come_by_chance_and_never_move_another_roll`). Rules read a benefit through `Actor::skill_effect`
(the actor's skills plus what magic items' skill affixes add, `Skills::effect_with`), never
`Skills::effect` directly. Each effect is read in one place: Dodge in
`Actor::defence`, quiet in `noise::noise`, hearing in `Actor::hearing`, burden in
`World::burdened` (`Burden::slow_eased`), travel in `World::step_price`, bargain (Negotiation
since #666) in `world/shop.rs`, identify (Appraise; trained at all, below) in
`world/appraise.rs`, kick (athletics: Strength for kicking a door in) in `World::kick_door`, speed
(athletics, #548: percent quicker, added to the talents' `speed_percent`, the skills never
taking the sum past `MAX_SPEED`) in `Actor::speed`, pick
(Open lock: picking a lock, and trained at all, the Pick lock ability) in `World::pick_lock`,
search, search reach, and spot in `world/search.rs` (`search_modifier`, `search_turn`; Spot
trained at all lets the player notice in passing, `Skills::trains_effect`), disarm in
`world/disarm.rs` (`disarm_modifier`; trained at all, Disable device gives the Disarm ability,
`Ability::had_by`), steal in `world/steal.rs` (trained at all lets the player steal,
`docs/dev/stealing.md`), heal in `world/heal.rs` (trained at all gives the Heal ability;
`docs/dev/status-effects.md` "Treating oneself"), forage (Survival: percent added to a forage's chance; trained at all,
the Forage ability) in `world/forage.rs`, study (Spellcraft and Decipher script: points off the study check) in
`world/learn.rs::study_odds` (docs/dev/spells.md "Learning"; its use `Use::Study`, reported
when a study ends, is also `StatUse::Study`); each use
is reported where it happens (`make_noise`, `hear`, `fight.rs::resolve`, `apply`, `player_moves`,
`kick_door`, `search_turn` for `search`, `notice_hidden` for `spot`, `World::disarm` for
`disarm`, `pick_lock` for `pick`, `World::steal` for `steal`, `World::heal` for `heal`, `World::forage_done` for `forage` (a forage that ran its time), `World::pay` and
`World::trade` (a sale) in `world/shop.rs` for `deal`, `World::appraise` for `appraise`, `world/temper.rs::tempered` for `temper`, and `World::sidesteps` for a known trap avoided,
`dodge`, besides `fight.rs::resolve`). Metalcraft's `temper` benefit is read in
`world/temper.rs::tempered` (the forge's Strength check, docs/dev/special-rooms.md); Metalcraft
stays a skill "not usable yet" in the abilities menu, since Temper is offered by the anvil to
anyone. A new
effect or use is a variant there plus its one hook. The player sees values only
(`PerceptionView::own_skills`, the Skills screen `ui/skills_screen.rs`, `A`); never marks or
benefit numbers. Skills get no keys of their own: active uses are chosen from the one abilities
menu (#156, `docs/dev/screens.md`): each trained skill of kind `action` is an `Ability::Skill`
there (`warlock_core::ability`), unavailable ("not usable yet") until its use exists, when it
becomes an ability of its own in the skill's place (`Ability::use_of`): Disable device is
`Ability::Disarm` (`docs/dev/traps.md`), Open lock `Ability::PickLock` (`docs/dev/doors.md`),
Steal `Ability::Steal` (`docs/dev/stealing.md`), Heal `Ability::Heal` (`docs/dev/status-effects.md`).
A passive skill with a use of its own is listed as that use too (`Ability::every` asks
`Ability::use_of` of every skill): Survival is `Ability::Forage` (below).

**Foraging** (spec §8 "Foraging", #547): `Action::Forage` (replay text `forage`, cost `forage`
a turn, `ActionKind::Forage`), dispatched by one arm in `World::player_acts` to
`World::start_forage` in `world/forage.rs` (`Spent::Forage(seconds)`, then `World::forage`, a
`keep_at` loop like a prayer's; tests `world/forage_tests.rs`). Data `data/forage.toml`
(`content/forage.rs`, `ForageRules`, `Content::forage`; optional: without it nobody forages):
`seconds`, and a `[[ground]]` per `Landscape` (land only) with `chance`, `count` dice, and
weighted food `finds`. `world::forage_ground(content, level, pos)` is the one place that says
where: the overworld by the terrain underfoot's `landscape`, a `local:x,y` area by its edge
exit's overworld cell, never on water or ice (`LevelState::water_at`); the rule and the
abilities menu (`PerceptionView::abilities`: `Unavailable::NotHere`, "not usable here") both
ask it. Draws: the chance (one gameplay draw), the find (`magic::loot::pick_weighted`, one), and
the count's dice; the food is made then (`rot::made_at`) and joins the pack; then
`train(Use::Forage)`. Nothing is stored, so saves change only by content (the new
`ActionKind`, the new food objects). The wiki's table is `generated/foraging.md`
(`xtask/src/wiki/skills.rs::foraging`).
**Appraise** (spec §8 "Skills", #666; Negotiation took the shops' bargain): `world/appraise.rs`,
`World::appraise_hours(start, seconds)`, called for the player from `World::live` (the status
loop, so rests roll each hour they pass) and from `World::skip_time_dev` (`skip:`). It walks the
whole game hours in `(start, start + seconds]`; it returns at once unless the player is trained
in a skill with `Effect::Identify` (`Skills::trains_effect`: an affix alone never gives it) and,
per hour, unless `World::unidentified` (the appraisal service's list) is non-empty. Each hour
draws from its own stateless stream `Rng::stream(seed, "appraise/<hour>")`: `chance` of
`skill_effect(Identify)` percent, then `index` for the stack. No state is saved, so how the time
is cut never changes the outcome. `World::learn_item` (`world/loot.rs`) is `World::identify`
without its "This is ...!" line; Appraise says "You realise your X is Y." and reports
`Use::Appraise`. Tests: `world/appraise_tests.rs` (the split, prices, the line, untrained,
cutting time, rest and skip, replays), `world/shop_tests.rs::deals_train_negotiation`.

Pending hooks:
athletics (climbing out of pits, #128) and escape artist (pits, #128).
