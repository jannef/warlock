# Skills

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08i-character.md. Keep it current with the code, as CLAUDE.md says.

Skills (spec §8 "Skills", #132) live in `warlock-core`: `skills.rs` (`Skills` per actor: trained
skills only, a missing one is untrained and gives nothing; `Effect` and `Use`, the typed benefit
and training-mark hooks; `Benefit` growth by `steps` or a `curve`; `Ladder` and `RollRules`, the
skill rolls' data; `Skills::mark` adds a mark, `Skills::roll` applies a roll and spends the
marks), `content/skill.rs` (`data/skills.toml`: the skills, `[ladder]`, `[rolls]`; adding a skill
is one entry), `world/train.rs` (marks for the player only, each through `World::mark_skill` in `world/skill_roll.rs`, the one hook (a quest's `skills` reward marks there too, docs/dev/quests.md), drawn from the `training` stream in
`WorldState`, never the gameplay stream; nothing said), and `world/skill_roll.rs` (#230: the
level-up rolls, below).

**Growth is skill rolls only** (design `docs/design/progression.md` §5): a level's, and the
automatic one (#1328). Marks are a reservoir: `World::level_up` (`world/talent.rs`) puts `World::level_rolls()` rolls (class
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
(`advance_auto`). **Automatic rolls** (#1328): every mark is added through
`World::mark_skill(id, by)` (`world/skill_roll.rs`; `World::train` calls it a mark at a time, and
`World::teach_skills` with a quest reward's `marks` at once, after its words). Once the skill's marks reach `RollRules::auto_at` (`[rolls]
auto_at`, 100; 0 turns it off) it rolls at once through `World::roll_skill`, the one path
`advance_skill` uses too: the same dice, `levels_rng`, words, and spend, again while the spent
marks are still at the threshold (a big gift). No `Due` is touched, and a pending level-up does
not hold it back. The Skills screen's meter (`skills::progress`) is the ladder step in quarters.
Tests: `skills.rs` (ladder, rolls, meter), `content/skill.rs`, `content/class.rs`
(`known_for`, `rolls`), `world/skill_roll_tests.rs` (counts, refusals, dice and affinities,
streams, saves and replays), `world/skills_tests.rs` (marks by use). Race and class grants are dice rolled in `character.rs` after every
other roll, so they never move a score. A class's **optional skills** (#662,
`ClassDef::optional_skills`, `content::OptionalSkill { skill, chance, dice }`; the loader checks
the ids)
are rolled by `character::optional_skills` from their own stream,
`character/optional-skills/<spec>` (chance, then dice, in skill order; a class without any draws
nothing), so no other roll moves; the result is just the actor's `Skills`, saved and hashed as
before, and a replay needs no new field (the roll is a function of the seed and spec). Classes
that have them start with other skills on old seeds, so old replays' hashes change.
`ClassDef::trains` (the level-up roll's +1) stays the guaranteed list. A background story's
skill starts go through `Skills::add` too, after every roll (docs/dev/background.md), and so
do a fresh caster corpse's (#1103, `on_eat.skills`, `World::corpse_skills` in `world/eat.rs`:
Decipher script learned at `gives` or raised by it up to `up_to`; docs/dev/hunger.md). A race's optional skills
(#682, `RaceDef::optional_skills`: the drow's Sneak attack) share the type and the parser
(`content/skill.rs::optional_skills`) and the stream: the
class's are rolled first, then the race's, so no class's draws move and a race without any
draws nothing more. Tests: `content/class.rs`, `content/race.rs` (parsing, refusals),
`character.rs` (`optional_skills_come_by_chance_and_never_move_another_roll`,
`a_drow_may_start_with_sneak_attack_and_no_other_characters_skills_move`).
Sneak attack's `sneak_dice` is read in `World::sneak_dice` and `surprise` (the craft) in
`World::startle`'s `craft`; Initiative's `initiative` in `World::startle` and `surprise_ease` in
`World::status_cost` (`world/ambush.rs`, `world/surprise.rs`; `docs/dev/surprise.md`). Their uses:
`sneak_attack` (in `ambushed`, where the dice land; shared with the stats as
`StatUse::SneakAttack`, a Dexterity mark) and `surprise` (in `startle`, the player's check made,
whatever came of it). Intimidate's `intimidate` (#844) is read in `World::war_cry`
(`world/war_cry.rs`), added to the difficulty of the Willpower check the cry asks of every hostile
that hears and sees the player, and its use `intimidate` is marked there, once a cry, whatever
came of it (`world/war_cry_tests.rs`). The skill's own use, the **Intimidate** ability (#868),
is `Action::Intimidate(Pos)` (replay text `intimidate:X,Y`, cost `intimidate`,
`ActionKind::Intimidate`), dispatched by one arm in `World::player_acts` to `World::intimidate`
in `world/intimidate.rs`: a single-target threat on War cry's check, sharing
`World::threat_difficulty` (the one read of `intimidate` for the difficulty), `World::heard_by`
(who would hear a noise, asked before it is made), and `World::cow` (the nerve check, the
status, the message) with `World::war_cry`. Its numbers are data, `data/intimidate.toml`
(`content/intimidate.rs`, `IntimidateRules`: `range`, `difficulty`, `status` (Frightened, one
that `flees`), and `degrees`, the ladder of `[points, degrees]` by what Intimidate adds, its
points rising (the one load check beside resolution), read by `IntimidateRules::degrees_at`); without the file the skill stays "not usable yet". It is
marked once a threat, refused or not reached (no hearing, no sight) aside. Tests:
`world/intimidate_tests.rs` (the menu row, refusals with no time and no draw, the turn and the
voice and the mark, the check, the ladder), `content/intimidate.rs` (parsing, refusals). Bluff's
`feint` (#1016; `docs/design/skill-uses.md` §6.2 B1) is read in `World::feint` (`world/feint.rs`),
Bluff's use: `Action::Feint(Dir)` (replay text `feint:DIR`, cost `feint`, `ActionKind::Feint`),
`Ability::Feint`, data `data/feint.toml` (`content/feint.rs`, `FeintRules`: `difficulty`, `status`,
Surprised, `flat_footed` and `per_action` with `max = 1`; the loader only resolves it). The foe's nerve (its Willpower modifier
and talents' checks, two gameplay draws, `check::check`) against `difficulty` + the player's Presence
modifier + the points; failed, Surprised, caused by the player. No noise, no need of being heard
or seen. Marked by `Use::Feint` at 50 %, on the training stream, whatever came of it. Surprised
would end with the foe's next action, which falls between the feint and the blow, so `World::feint`
puts it on `World::fresh` (`weapon_ability.rs::spend_actions`): the foe's first action in the
feint's turn does not ease it. `fresh` is emptied as a turn ends, so it is never saved across
one. The foe's second action ends it; both cost it dear. The ability's id is `feint`, as the
Duelist's weapon active, False lead, is (`Ability::Weapon`, `use:false_lead:E`): the two rows are told apart by
`Source`, and nothing looks an ability up by id. No caught-out check (the owner): `World::steal`
and the shops read nothing of Bluff. Tests: `world/feint_tests.rs`, `content/feint.rs`. Music's
`song_degrees` (#870) is read in `World::sing` (`world/song.rs`): the degrees a song adds to its
status, capped by the status's `max`; `song_craft`, the difficulty a song on its hearers adds to
their check, is read there too (the Lullaby's, #873). Its use `song` is marked once a song,
whatever it does; Music is `kind = "rules"` (no abilities-menu row). Tests:
`world/song_tests.rs`, `world/lullaby_tests.rs`. Rules read a benefit through `Actor::skill_effect`
(the actor's skills plus what magic items' skill affixes and talents add, Skill focus's chosen
skill and Master of none's every one, `Skills::effect_with`), never
`Skills::effect` directly. Each effect is read in one place: Dodge in
`Actor::defence`, quiet in `noise::noise`, hearing in `Actor::hearing`, burden in
`World::burdened` (`Burden::slow_eased`), travel in `World::step_price`, bargain (Negotiation
since #666) in `world/shop.rs`, identify (Appraise; trained at all, below) in
`world/appraise.rs`, kick (athletics: Strength for kicking a door in) in `World::kick_door`, speed
(athletics, #548: percent quicker, added to the talents' `speed_percent`, the skills never
taking the sum past `MAX_SPEED`) in `Actor::speed`, pick
(Open lock: picking a lock, and trained at all, the Pick lock ability) in `World::pick_lock`,
search, search reach, spot, and detect traps in `world/search.rs` (`search_modifier`,
`search_turn`; Spot trained at all lets the player notice in passing, Detect traps the same for
hidden traps, whose checks read it in place of Search and Spot, `Skills::trains_effect`), disarm in
`world/disarm.rs` (`disarm_modifier`; trained at all, Disable device gives the Disarm ability,
`Ability::had_by`), slip (Stealth: points added to slipping past a creature's darkvision) in
`world/slip.rs` (docs/dev/surprise.md), steal in `world/steal.rs` (trained at all lets the player steal,
`docs/dev/stealing.md`), heal in `world/heal.rs` (trained at all gives the Heal ability, on oneself or an ally beside one;
`docs/dev/status-effects.md` "Treating oneself"), forage (Survival: percent added to a forage's chance; trained at all,
the Forage ability) in `world/forage.rs`, study (Decipher script since #1270: points off the study check) in
`world/learn.rs::study_odds` and learn (Decipher script: the spell level a scroll teaches for certain, at least 1, and reads with no check; trained at all lets
the player learn from a scroll) in `Actor::learning` (docs/dev/spells.md "Learning"; their use `Use::Study`, reported
when a study ends, is also `StatUse::Study`), decipher (Decipher script: points on the check of reading a scroll, read by anyone, but untrained it is 0: the skill does nothing without the skill, whatever raises it) in `World::deciphers`, `world/decipher.rs` (docs/dev/scrolls.md "Making out a scroll"; its use `Use::Decipher`, reported per check, trains no stat, as a made read's Intelligence mark comes of `use_up`). All reading trains Decipher script (#1270): `Use::Decipher` 100 %, `Use::Read` 50 % (a scroll read and used up, reported in `read.rs::use_up` beside its Intelligence mark; no stat of its own), and `Use::Study` 100 % (a grimoire read to its end). Spellcraft keeps only its `cast` mark (25 %) and has no benefit: nothing reads it until a rule does; each use
is reported where it happens (`make_noise`, `hear`, `fight.rs::resolve`, `apply`, `player_moves`,
`kick_door`, `search_turn` for `search`, `notice_hidden` for `spot`, both for `find_trap` (a
trap found), `World::disarm` for
`disarm`, `pick_lock` for `pick`, `World::steal` for `steal`, `World::heal` for `heal`, `World::forage_done` for `forage` (a forage that ran its time), `World::order` and `fight.rs::dies` for `handle` (an order given to one's allies, an ally's kill: Handle animal's marks, docs/dev/allies.md), `world/tame.rs::tame_check` for `handle` too (a try at taming), and the `tame` benefit read in `tame_odds` (trained at all, the Tame ability; docs/dev/companion.md "Taming"), `World::pay` and
`World::trade` (a sale) in `world/shop.rs` for `deal`, `World::appraise` for `appraise`, `world/temper.rs::tempered` for `temper`, and `World::sidesteps` for a known trap avoided,
`dodge`, besides `fight.rs::resolve`). Listen at a door (#1026) is `Use::Listen`, 100 % per `listen:DIR` (`world/listen.rs`, `hearing.md`). Marks from plain play (#1014, `docs/design/skill-uses.md` 6.8, 6.9, 6.11): `Use::Sight` (Spot) in `surprise.rs::mark_noticed`, the first sighting of a creature that has not noticed the player; `Use::Sprung` (Detect traps) in `trap.rs::train_from_sprung`, a hidden trap going off under the player, or (half the time, a training-stream draw) under a creature they see; `Use::Overhear` (Listen) in `perceive.rs::hear`, a sound that is no news heard while resting or waiting (`World::waiting` is set around the player's wait); Detect traps also marks on `disarm` and, with Spot, on `search`, as data. Concentration's `mana_regen` and `max_mana` (trained only, `Actor::mind`) are read in
`World::mana_rate` and `Actor::max_mana` (`world/concentration.rs`, docs/dev/mana.md), its `cast`
use reported by `World::spend_mana` (Spellcraft's `cast` mark, 25 %, rides the same use, #989; since #1270 its only mark). Spellcraft's `cast` (points on the Spellcraft check every cast with one's own mana makes) and `ward` (points off a miscast's total) are read in `world/miscast.rs` (`Actor::cast_bonus`, `Actor::miscast_ward`; #1271, docs/dev/spells.md "Miscasts"), by anyone: an untrained caster has none of them and checks all the same. Knowledge's `resist` (#667) is read
in `world/knowledge.rs` (`World::resisting`, `resist_bonus`) through `Actor::skill_effect_above`
(`Skills::effect_above`: each benefit read at the value less the creature's level, nothing at or
below it), called for the modifier of the player's check against a creature's drain
(`World::drain`, which takes the drainer), exposure to a disease its attack carries
(`World::expose` with a `by`), and a struggle against its hold (`World::struggle`, the effect's
`by`); the same call marks `Use::Resist` (training stream only), and `World::discovered` marks
`Use::Discover` beside the first-kill and arrival XP (`world/experience.rs`). The player only, and
only against a creature: no bonus, no draw otherwise. The same skills standing above a creature's
level at all (`Actor::skill_stands_above`, `Skills::stands_above`: one above is enough, though the
curve gives nothing below 3) tell look mode what its kind resists (#1030): `PerceptionView::known_resistances`
reads `Actor::known_resistances` (`world/knowledge.rs`), the type's `resist` in `ResistanceWord`s,
never gear or statuses, no marks; JSON dumps carry it as a seen actor's `known`; `ui/look.rs::known`
writes the line. Tests `world/knowledge_tests.rs`.
Metalcraft's `temper` benefit is read in
`world/temper.rs::tempered` (the Strength check of an attempt at an anvil, whose difficulty rises
with every attempt on the item, docs/dev/special-rooms.md); Metalcraft
stays a skill "not usable yet" in the abilities menu, since Temper is offered by the anvil to
anyone. The professions' skills (#1331, epic #1337) **Alchemy, Enchanting, Herbalism, Jewelcraft,
and Mining** are data only for now, **no effect yet**: no marks, no benefit, and no class, race,
or story trains them. Kinds as `docs/design/professions.md` §9: Mining `rules`, Herbalism
`passive`, the others `action` (shown "not usable yet"). Skill ids are file order (alphabetical),
and `Skills` is saved and hashed by that index, so inserting them shifted the later skills'
indices: state hashes and saves of runs with skills change (saves are pinned to the version).
Effects and training grants are deferred to a follow-up issue.
A new
effect or use is a variant there plus its one hook. The player sees values only
(`PerceptionView::own_skills`, the Skills screen `ui/skills_screen.rs`, `A`); never marks or
benefit numbers. Skills get no keys of their own: active uses are chosen from the one abilities
menu (#156, `docs/dev/screens.md`): each trained skill of kind `action` is an `Ability::Skill`
there (`warlock_core::ability`), unavailable ("not usable yet") until its use exists, when it
becomes an ability of its own in the skill's place (`Ability::use_of`): Disable device is
`Ability::Disarm` (`docs/dev/traps.md`), Open lock `Ability::PickLock` (`docs/dev/doors.md`),
Steal `Ability::Steal` (`docs/dev/stealing.md`), Heal `Ability::Heal` (`docs/dev/status-effects.md`),
Intimidate `Ability::Intimidate` (above; aimed at a cell, `Ability::reach` from the data),
Bluff `Ability::Feint` (above; in a direction).
A passive skill with a use of its own is listed as that use too (`Ability::every` asks
`Ability::use_of` of every skill): Survival is `Ability::Forage` (below). Woodcraft is
`Ability::Fletch`, which anyone has, trained or not (below), and `Ability::Wedge` for those trained
(`Ability::every` adds it right after the skills while some skill gives `Effect::Wedge`; see
`docs/dev/doors.md` "Wedging").

**Fletching** (spec §8 "Fletching", #1021, design `docs/design/skill-uses.md` §6.5 W1):
`Action::Fletch` (replay text `fletch`, cost `fletch` a turn, `ActionKind::Fletch`), dispatched by
one arm in `World::player_acts` to `World::start_fletch` in `world/fletch.rs` (`Spent::Fletch(seconds)`,
then `World::fletch`, a `keep_at` loop with the forage's `Watch::Forage`; tests
`world/fletch_tests.rs`). Data `data/fletch.toml` (`content/fletch.rs`, `FletchRules`,
`Content::fletch`; optional: without it Woodcraft is a skill with no use): `seconds`, `difficulty`,
`count`, `value_per_count`, `half_within`, `one_within` (`FletchRules::made`, the margin's bands),
`plain_from`, `knives` (a weapon skill: `Content::is_knife`), and `[[makes]]` by what a launcher's
`launches` is. `world::fletch_lack` is the one place that says what is missing: the abilities row's
`Unavailable::NoKnife` / `NoLauncher` (`Ability::unavailable`) and the rules' refusal, in the same
words. The check reads Dexterity's modifier, `TalentMods::check`, `skill_effect(Effect::Fletch)`
(Woodcraft's curve, Skill focus through it), and `TalentMods::fletcher`; the count and the brittle
line read the trained value, `Skills::value_of_effect`, so no item or talent bonus moves them.
Marks: `Use::Fletch` for every batch that ran its time, whatever it made. Fletcher's numbers are
its effect's (`talents/fletcher.toml`).

A damaged item (Temper's failure, or acid, rust, or heat: `world/item_damage.rs`, #1131, #1168) is a point worse until
mended (`Mods::add_worked`: its Protection if it gives some, else its Defence, else its damage).
Mithril and adamantium are metals, so Temper works them as steel.

**Whittling** (#1028, design W3): `Action::Whittle { item }` (replay text `whittle:LETTER`, cost `whittle` a turn, `ActionKind::Whittle`) is Temper's twin for wood: `world/temper.rs` keeps both, a `Craft` (`Temper` or `Whittle`) in `Tempering` choosing the material group (`metal` or `wood`), the stat and effect (Strength and `Effect::Temper`, or Dexterity and `Effect::Whittle`), the mark (`Use::Temper`, `Use::Whittle`), the turn price, and the message lines. Both share `data/temper.toml` and `magic/worked.rs`, so one item has one state. `Ability::Whittle` stands beside Fletch in `Ability::every` (the skills iterator adds it after Fletch), is had by everyone, asks for an item (`ObjectList::Whittle`, `ScreenName::Whittle`, the list `world::workable_wood`), and is unavailable without a knife (`world::whittle_lack`, `Unavailable::NoKnife`). Tests: `world/whittle_tests.rs`.

**Balance** (spec §8 "Skills", #1029, `docs/design/skill-uses.md` §6.13): besides `burden`,
`Effect::Footing` is read in `World::step_price` (`world/schedule.rs`) in the `else` of the wading
branch, on a step `World::rough_step` (`world/footing.rs`) calls rough: off the overworld,
`move_cost` above 100, not `wading_step`; the same `surcharge_spared`. `Use::RoughStep` is marked by
`World::train_rough_step` after `wade` in the step (`world/travel.rs`). `Effect::SteadyEase` is read
in `World::status_cost` (`world/surprise.rs`) for the status `footing::STAGGERED` (`"staggered"`),
beside Initiative's ease of Surprised; `Use::Staggered` is marked in `spend_actions`
(`world/weapon_ability.rs`) for the player's action that eases Staggered (not the one that gave it).
Tests `world/footing_tests.rs`.

**Swim** (spec §8 "Skills", #1018, `docs/design/skill-uses.md` §6.4; `kind = "rules"`, no menu
row): `Effect::Wade` is read in `World::step_price` (`world/schedule.rs`), on a step
`World::wading_step` calls wading (off the overworld, `move_cost` above 100, `stars::wading`): a
percent of what the terrain adds over a plain step is spared (`surcharge_spared`, rounded up), 100
for `swimmer` and `Actor::wades_free`, so Swim never goes past them. Its use `wade` is reported at the
head of `World::wade` (`world/disease.rs`), before the talents' early return: a wader whom a
talent spares still trains. `Effect::Swim` is read in `World::keeps_afloat` (`world/ice.rs`,
`docs/dev/temperature.md`), which marks `swim` once a check; it is `Skills::trains_effect`-gated, so
an item's Swim bonus alone asks no check. Tests `world/swim_tests.rs`.

**Deep water** (spec §8 "Skills": Swim, #1019; design §6.4 S2) is `world/swim.rs`, one line each
in `player_moves`, the refused actions (`pick_up_objects`, `drop_objects`, `start_rest`, the
search dispatch, `start_dig`, lighting in `use_tool`), and `equip`. The terrain's `swim = true`
with `walkable = false` (`content/mod.rs` checks the pair) is why nothing else needs to know of it:
paths, travel, spawning, teleports, blinks, and landings treat it as impassable. `at_the_shore`
stops a step from anything else: water not in the level's memory is written there, with no time, so
the step cannot go in unasked (presentation asks only of what the player knows,
`PerceptionView::dives_at`); then `shore_refusal`, shared with `dives_at`. `stroke_fails` is the
check of a step that starts in deep water, wherever it ends (**the dive itself asks none**: a
"Dive in" that left you on the shore, cold, would make no sense; the issue left this open): two
gameplay draws (the 2d12), `Use::Stroke` whatever came of it (a use of its own, as `swim` is 100 %
for a plunge), then `Harm::Spared`. `data/swim.toml` has the numbers (`content/swim.rs`, optional:
without it nobody swims); burden goes by `Load::penalty`, as every burden hook does. `dive_in` and
`equip` put out the ready light (`douse_afloat`), so a second lantern, lit in the pack, cannot be
made ready to cheat it. Prices: `wading_step` is true of `swim` terrain, so Wade, Swimmer
(`terrain_as`), Strong swimmer, and Undertow apply; `World::wade` marks no `wade` for it.
`route_to_shore` (close-in in `ai.rs`, Follow in `ally.rs`) widens a ring at a time up to
`SHORE_REACH` when the target is afloat with no ground in reach, so creatures wait on the nearest
shore; only then does it search more than once. A swimmer swaps with no one (`Refusal::Water`). An
Overloaded swimmer cannot step (`can_step_to`, as anywhere) and cannot drop: they can still throw
things away, eat, or teleport, so it is no trap. Tests `world/swim_deep_tests.rs`; the showcase's
quarry pit holds a pool to try it in (the test world only; #1020 places it in the world).

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
`Use::Appraise`. Appraise also carries `Effect::Freshness` (with Survival's, #1031: the tell of how fresh
food is, `world/freshness.rs`, docs/dev/hunger.md) and the mark `judged`. Tests: `world/appraise_tests.rs` (the split, prices, the line, untrained,
cutting time, rest and skip, replays), `world/shop_tests.rs::deals_train_negotiation`.

Overworld encounters (#804; `docs/dev/encounters.md`): Spot's and Survival's `lookout`,
Stealth's `stalk`, and Survival's `evade` are read in `world/encounter.rs` only (the player's
through `Actor::skill_effect`, a party's lead type's through `Skills::effect`); `Use::Lookout`
(Spot and Survival) is reported on every lookout there. A resting player's lookout (#807) reads
Listen's `hearing` (with Keen ears, capped at `MAX_ENCOUNTER_BONUS`) instead and reports
`Use::Hear`.

Escape artist (#1015) has no action and no ability row. `Effect::Escape` is read in
`World::slipping` (`world/hold.rs`, docs/dev/status-effects.md "Holds"), `Effect::Climb` (steps at
50 and 100: halved, then skipped) in `World::climb_price` (`world/trap.rs`, the pit's one arm);
both only for those trained (`trains_effect`). `Use::Escape` is marked per check whatever came of
it, `Use::Climb` per climb out (Escape artist 100 %, Athletics 50 % and no effect). Tests:
`world/escape_tests.rs`. `Effect::Escape` is also read by `World::wriggle` (`world/wriggle.rs`,
#1032: a step into a neutral creature that keeps its post, an automatic Dexterity check against
11 plus its Willpower modifier, one more `Use::Wriggle` at 50 %; tests `wriggle_tests.rs`).

## Tumble: plain misses

A blow at the player that comes back `Strike::Missed { by: None }` becomes a dodge one time in
four (`World::plain_miss_dodged`, `world/train.rs`, called in `fight.rs` after `caught_shot`):
one `chance(25)` on the training stream, so the gameplay stream does not move. Misses by a
category stay as they are; blows between creatures or from the player draw nothing
(`docs/design/skill-uses.md` Tu1). `balance_probe/training.rs` uses the same function.
