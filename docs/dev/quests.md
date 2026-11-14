# Quests

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08b-creatures.md. Keep it current with the code, as CLAUDE.md says.

Quests (spec §8 "Quests", #148): `content/quest.rs` loads the `quests` table (`data/quests.toml`, the field reference, and each
campaign's `data/quests/`; the showcase's four in `campaigns/showcase/data/quests/showcase.toml`) after levels and
dungeons (`QuestDef`: `giver` a `DialogueId`, `Goal::{Kill, Bring, Reach, Deliver}`, `Reward`,
`QuestTexts`, `requires` (a list: every quest named must be done; the file takes one id or a
list), `requires_alignment`, `min_level` (with its `early` refusal: `ask_quest` says it and
gives nothing while `World::player_level` is below it; the topic is still listed), `alignment`, `repeatable`, `reveals` (a hidden
location of the overworld and the `RevealWhen` stage it is revealed at, `docs/dev/overworld.md`),
`where_` (`where`, appended to the log's objective); validation including everyone's topic budget of nine;
`Content::speaker_name` names a giver without an actor). `world/quest.rs` is the rules: quest
topics come first in `World::topics` (`quest_topics`: one `quest-ID` a quest) and
are answered in `World::ask` (`ask_quest`, `Spent::NoTime` when state changes). **Asking gives
the quest** (#422, `World::give`): the offer and the `accept` line in one reply, the record
made active at once (with its roll, below, and a kill quest's `counted` baseline when it is
a `pick` or a repeat), a delivery's item handed over, `reveals.when = "given"` fired. There is no
accepting, declining, or `withdraw` any more; the `accept-ID`/`decline-ID` topics of older
replays are no one's and refused, and an `offered` record of an older save loads as active
(`#[serde(alias)]` on `Stage::Active`). A quest is
offered only when `World::offers` it: the player's band meets its `requires_alignment` (a
giver who scorns them still offers it, #692); `complete` makes the quest's
`alignment` shift, `docs/dev/alignment.md`); `Quests` in
`WorldState` (records by quest id: stage, completed, counted, reached, roll; the kill tally by
level and creature id, for every type (#244: the ranger's Favoured prey reads it too), `prey`,
the type killed most, the first to get there of those tied, and `slain`, the unique types that
died), each left out of the JSON while empty so runs without quests hash as before. Hooks:
`fight.rs::creature_dies` (every creature death, at any hand) adds a dying unique type to
`Quests::slain`, and a kill goal on a unique type (a named monster; `count` 1, checked at load)
is met by `slain` alone, whenever and however it died (the guild design §8.4: no softlock);
`fight.rs::dies` tallies the
player's kills (`tally_kill`, beside `earn_kill`, so a creature bled to death by the player's
wound counts too; under the level it died on, or `Actor::home` for a stair follower,
`stair_followers.rs`), `travel` marks reach quests (`reach`), and `World::apply` snapshots active
quests' `progress` before the player acts and says what rose after it, and again after the
turn (`quest_progress`/`quest_news`), so pick-ups, kills, and arrivals all report through one
place. `reveal_at` (beside the answer at the two places a stage is set: `give` and
`complete`) reveals a quest's location after the speaker's words. No abandoning and no cap on active quests (the owner's decision): don't add either. What a quest asks to be
brought or delivered (`World::quest_objects`) is protected from all item damage, ruin, acid, rust,
heat, and a failed temper alike (`world/item_damage.rs`, #1170; docs/dev/objects.md
"Protection"); a quest's reward is an ordinary item unless another quest's `hand_in` names it
(the owner: no special quest items for rewards).
A reward's `skills` (#1329, `Reward::skills`, a `SkillReward` each: Grave goods teaches Heal)
are applied in `complete_saying` after the items, by `World::teach_skills`: a skill the player is
untrained in is trained at `start` (`Skills::add`, rolled on `levels_rng`, the stream background
and level rolls use, never the gameplay one), a trained one gets `marks` training marks at once,
after the reward's words, through `World::mark_skill(id, by)` (`world/skill_roll.rs`, the one hook
every mark of the player's goes through), so marks reaching `[rolls] auto_at` roll the skill at
once (#1328; Grave goods' 100 marks roll a trained Heal once); never both. The words are the
reward's `taught` and `more`, else defaults; never numbers. Load refuses an unknown skill and an
entry with neither `start` nor `marks`.
A quest done for one of a town's people (`data/towns/`) also raises that town's goodwill, its
services cheaper by its percent: `World::tell_goodwill` after "Quest complete", the count derived
from `QuestRecord::completed` (#1247; docs/dev/npcs.md "Goodwill").
The log's objective is the **first breadcrumb**, `world::quest_wants`: "<Giver> wants you to
<asks, lower-cased>." (a bring to the giver names no recipient); `quest_asks` stays the plain
goal, and for a rolled quest describes its pool (the wiki's goal column).
**Rolled quests** (#422, the guild design §5): `content/quest_roll.rs` holds the data:
`QuestDef::roll: Option<Rolled>` (`Rolled::Pick(Pick::{Creatures(branches), Tags(tags),
Objects{class, loot}})` or `Rolled::Variants`), parsed from `pick` or `[[quest.variant]]`
(exactly one of `target`, `pick`, variants; a pick of creatures or tags is a kill quest's, of
objects a bring's). A rolled quest's `goal` is a template whose target is the full pool's
first; only `QuestDef::rolled(content, roll)`, the quest as given (goal after the roll, a
variant's texts, `reveals`, `where`, every placeholder filled, `roll` cleared), is read for a
given quest: `Quests::given` makes it (a `Cow`, borrowed for every quest not rolled), and
`progress`, the log, `ask_quest`'s reminder, `complete`, and `quest_news` go through it.
`Rolled::candidates(content, campaign, open)` is the pool in content order, animated objects left out (creature and
object ids, sorted tags, variants in file order); `quest_roll::check` fills every text for
every candidate at load (no unknown or unclosed placeholder; nothing on the texts' length, #1205).
`Roll { target, variant, count }`
(`content::Roll`, slugs, saved in `QuestRecord::roll`; `roll_known` guards saves through
`Quests::unknown_to`). `world/quest_roll.rs` is the play: `branch_open` (no unrevealed hidden
location of the run's overworld leads into the branch), `quest_pool`, `roll_quest` (stream
`Rng::quest(seed, id)` = `quest/<id>`, used once at the giving, weights summed then one
`below`), `quest_roll_dev` (the dev `quest_roll:ID:TARGET`: active, no progress, target in the
full pool or a variant id; resets `counted`, fires a variant's reveal), `quest_rolls` (the
`progress` dump's `roll ...` rows). A rolled quest whose pool is empty now is
not offered. **Beast-friend** (#1189): `quest_pool` leaves out a kill target the player's talent
calms (`beast_friend::calms`, the creature type's own faction `animal`; a tag when any of its
types of the quest's campaign is; a variant by its target), then the `fallback` variants
(`Variant::fallback`, checked at load: not every variant) unless nothing else is left. The talent
is read at the giving only; the saved `Roll` keeps it. A variant counts every kill, as an
authored quest does; only a `pick` (or a repeat) sets the `counted` baseline at the giving
(`give`). The example: Leofric's `barrow_rats` (`rats` → `rat`, fallback `skeletal` →
`skeletal_rat` with his own words; `barrow_heart`'s "The rats were the least of it" reads right
for both). Tests: `world/druid_quest_tests.rs` (every rolled kill quest of every campaign over
200 draws for a druid). Test content: the showcase gardener's `slug_charm` (a pick of scrolls; text
snapshots `quest-offer-rolled`, `quest-log-rolled`), and `world/quest_roll_tests.rs`'s
fixtures in Cēaptūn (Godgifu gives a creature pick, a tag pick, and two variants revealing
Fennmynster and Fyrdelf) and the arena (a kobold made the demilich, a shared unique).
**Bounties' talent** (the `bounties` tag folded into `talk` and `combat`, #773; Quarry sense was
cut in #774): Sworn foe (weaponcraft, `choice = "tag"`, a new `talent::Choice::Tag` over `Content::tags`;
`sworn_foe { to_hit, damage }`, read in `World::favoured` beside Favoured prey). Tests:
`world/quest_roll_tests.rs`.
**Creature tags** (#421, #441): a creature file's `tags = ["goblin", "archer"]` (plain
singular words, by custom lower-case words joined by hyphens; `content/tag.rs` asks only for
no `:` or whitespace, which action text splits on; nothing declares a tag, it exists because a creature carries it; one
flat pool, no registry). A kill quest's `target = "tag:goblin"` makes `Goal::Kill`'s `target` a
`KillTarget::Tag`; the quest is
refused when no type a run of its campaign can meet carries the tag (`quest::check_tag`: a typo,
or a tag only another campaign's types carry). `KillTarget::has` (`Content::has_tag`) is the one
membership read, by creature type only, so an actor's `faction_override` changes nothing;
`Quests::kills_for` sums the tally of every tagged type over every level the quest counts on.
The noun is the tag's plural by the plural dictionary (`Content::plural`, hyphens said as
spaces: `marsh-thing` is "marsh things"), or the quest's `what`; the objective then lists the
tagged types a run of the quest's campaign can meet (`world::quest::members`,
`Content::tagged_in`: "Kill 100 goblins: goblins, goblin archers, and goblin chiefs."), and
progress says the same noun. **Tag new creatures**: a new creature of a group carries the
group's tag (every creature of the `undead` faction carries `undead`; `content::tag`'s test
checks it), and a role tag where one fits (the list is in `assets/data/creatures/README.toml`).
Tests: `content/tag.rs` (the word rule, the built-in members, the undead check),
`world/tag_tests.rs` (levels summed, a bled-out one counts, a trap's or another's kill does not,
the override, `what`, load checks of tag targets and of creature tags); the wiki table is
`creatures::creature_tags` (`creature-tags.md`).
**A patron's commission** (#1421, `world/patron.rs`; docs/dev/npcs.md "The craftsperson's
patron"): not a `QuestDef`; while one is open `PerceptionView::quests` lists it after the active
quests (`PerceptionView::commission`: id `commission`, `QuestKind::Commission`, which a quest file
cannot name, `#[serde(skip_deserializing)]`). It has no `QuestRecord`, so nothing that tallies
quests (experience, guild rank, the Invest bonus, goodwill, `completed`) sees it.
**Escort quests** (#1139, `world/escort.rs`): `Goal::Escort { escort, level, joins }`
(`type = "escort"`, `target` a level as `reach` names one, `escort` a speaker that must be
`Speaker::Npc`, `joins` a reply; never rolled; `escort` and
`joins` refused on other kinds). `QuestDef::escort()`; `speakers()` includes it (the campaign
check). Hooks, one line each: `talk.rs::chat` → `World::escort_joins` (an active escort quest
whose escort is the dialogue, the actor no one's ally, `allies.toml` present: the `joins` line
through `answer` instead of the greeting, then the ally, `docs/dev/allies.md` "Escorts");
`travel.rs::travel` → `World::escorts_home` after `follow_along` (the escort, an acting ally of
the player's with `AllySource::Escort { quest }`, on the target level: released, then
the escort's `thanks` (optional) said as a message, then `World::complete_saying(speaker, q,
None)`, `complete` without a conversation, the speaker the giver if an actor speaking the
giver's dialogue stands on that level, else the escort: the `done` line said as a message, the
reward as a hand-in's); `fight.rs::creature_dies` → `World::escort_dies`
(the dead actor's dialogue is an active escort quest's escort, joined or not: `Stage::Failed`,
"Quest failed: NAME."). `progress` is always (0, 1) for an escort quest, so it is never ready
to hand in and its giver keeps the reminder. `Stage::Failed` (saved; never offered again,
`available`); `QuestView::failed`, the log lists it as "Eadgyth (failed)" (`ui/quest_screen.rs`), its status "Failed: Eadgyth is dead.", among the
finished ones, the `quests` dump's `(failed, ...)` with the status, JSON `failed`. The active
status reads `PerceptionView::escorting(quest)` from the snapshot's allies: "Eadgyth follows
you." / "Eadgyth is not with you.". Tests: `world/escort_tests.rs` (a fixture hamlet and cave on
the built-in content).
**A quest's escort** (`world/escort_waits.rs`, #1141): `waits = { creature, level, at }`
(`content::Waits` in `Goal::Escort`; checked at load: a creature type, a level of the campaign,
and exactly one place named `at` on it, a hand-made level's `places` or those of the vaults
`DungeonDef::vaults(depth)` pins on a branch level) makes the escort for a level that cannot
name a unique character (a vault holds no npc). `place_waiting(state, content, level)` runs at
the end of `travel.rs::instantiate_level` and in `quest.rs::give` when the level exists already:
each active escort quest with a `waits` on it whose npc no actor of the run is yet gets a
`make_creature` of `creature` with `npc` the escort's, on the place's cell or the nearest free
walkable one (ring search, row order). No record is kept: the actor existing is the guard (dead,
the quest has failed; home, it is done). The example, the Ascmearc's first escort:
`campaigns/ascmearc/data/quests/wynflaed.toml`, Gammer Wynflæd of Holtwīc sends the player for
Ælfrun (a `cottar`, dialogue in `data/dialogue/holtwic.toml`), `waits` on Ælfhām's moss bed
(`aelfham:3`, the pinned `moss_bower` vault), `reveals` the mound as given, done home in
Holtwīc with her thanks and the wise-woman's `done`. Tests: `world/wynflaed_quest_tests.rs`.
**Lure quests** (#1446, `world/lure.rs`, whose module docs are the rules): `Goal::Lure {
creature, level, waits, binding }` (`type = "lure"`, `target` a unique creature type, `level` a
hand-made level of the campaign, `waits = { level, at }` with no `creature`, `[quest.binding]`:
`content::Binding { status, faction, post, words, shatters, reforms, boon }`, `Boon { skill,
bonus }`; `QuestDef::lure()`; never rolled; load checks: the target unique, the status without
`eases`, the faction known, the post walkable on `level`, at least one word). Hooks, one line
each: `travel.rs::travel` → `World::lure_places(to)` (after `escorts_home`: the creature made on
its place if the quest is active and none of its type is anywhere, the `reforms` line when
`Quests::slain` holds it), `quest.rs::give` → the same for the `waits` level; `travel.rs::
take_stairs` → `World::lure_home` after `followers_arrive` (the creature, unbound, on `level`
with the player and the giver: `words`, `bind` (faction, stance, mind, `post`, the status through
`afflict_one`), `complete_saying(giver, q, None)`); `fight.rs::creature_dies` →
`World::lured_dies` (unbound while active: the `shatters` line; the quest stays active, nothing
fails, the next arrival on the `waits` level makes it anew); `cut.rs`, `set.rs`, and
`craft_make.rs::craft_modifier` (every made recipe of the skill: Jewelcraft's rings and amulets,
#1490) add `World::binding_boon(skill)` to the check (a bound one on the player's level). It travels only by
the stairs' own rule (`stair_followers.rs`: hostile and beside the player), so it is never lost to
a level change it cannot make: left behind, it stays on its floor until fetched. `progress` is
(0, 1); the log's status "The crystal guardian is not yet bound in Gimmwīc."; the objective
"Lure the crystal guardian from Meyn Bew 4 to Gimmwīc." The dev action `lure:ID`
(`World::lure_dev`, no time) brings the active quest's creature beside the player, aware of them,
made if it is nowhere. The example: Ysella the Binder's `crystal_guardian`
(`campaigns/ascmearc/data/quests/ysella.toml`), revealing Gimmwīc's hidden `meyn_bew` (the
crystal portal, `opens` said instead of the map line) as given; reward 1200 gold, 4000
experience, an uncut sapphire; the bound guardian keeps Kerensa's Stones' corner (67, 4), its
`bound` circle (enchantment) on it, and gives Jewelcraft's Cut and Set +2 in Gimmwīc. Tests:
`world/crystal_guardian_tests.rs`.
**Guilds** and their ranks, derived from these records and named as a quest is given or done,
are `docs/dev/guild.md`.
Presentation reads `PerceptionView::quests()` (`QuestView` strings, shared by the log and the
`quests` dump layer). Creature types have a `plural` for the messages ("3 / 3 giant rats").
