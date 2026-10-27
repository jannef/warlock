# Quests

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08b-creatures.md. Keep it current with the code, as CLAUDE.md says.

Quests (spec §8 "Quests", #148): `content/quest.rs` loads the `quests` table (`data/quests.toml`, the field reference, and each
campaign's `data/quests/`; the showcase's four in `campaigns/showcase/data/quests/showcase.toml`) after levels and
dungeons (`QuestDef`: `giver` a `DialogueId`, `Goal::{Kill, Bring, Reach, Deliver}`, `Reward`,
`QuestTexts`, `requires` (a list: every quest named must be done; the file takes one id or a
list), `requires_alignment`, `alignment`, `repeatable`, `reveals` (a hidden
location of the overworld and the `RevealWhen` stage it is revealed at, `docs/dev/overworld.md`),
`where_` (`where`, appended to the log's objective); validation including everyone's topic budget of nine;
`Content::speaker_name` names a giver without an actor). `world/quest.rs` is the rules: quest
topics come first in `World::topics` (`quest_topics`: one `quest-ID` a quest) and
are answered in `World::ask` (`ask_quest`, `Spent::NoTime` when state changes). **Asking gives
the quest** (#422, `World::give`): the offer and the `accept` line in one reply, the record
made active at once (with its roll, below, and a kill quest's `counted` baseline when it is
rolled or a repeat), a delivery's item handed over, `reveals.when = "given"` fired. There is no
accepting, declining, or `withdraw` any more; the `accept-ID`/`decline-ID` topics of older
replays are no one's and refused, and an `offered` record of an older save loads as active
(`#[serde(alias)]` on `Stage::Active`). A quest is
offered only when `World::offers` it: the player's band meets its `requires_alignment`, and a
giver who scorns them offers none whose `alignment` is good; `complete` makes the quest's
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
wound counts too), `travel` marks reach quests (`reach`), and `World::apply` snapshots active
quests' `progress` before the player acts and says what rose after it, and again after the
turn (`quest_progress`/`quest_news`), so pick-ups, kills, and arrivals all report through one
place. `reveal_at` (beside the answer at the two places a stage is set: `give` and
`complete`) reveals a quest's location after the speaker's words. No abandoning and no cap on active quests (the owner's decision): don't add either.
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
`Rolled::candidates(content, campaign, open)` is the pool in content order (creature and
object ids, sorted tags, variants in file order); `quest_roll::check` fills every text for
every candidate at load (280 characters, no unknown or unclosed placeholder; a quest not rolled
may hold none) and checks a unique kill asks for 1. `Roll { target, variant, count }`
(`content::Roll`, slugs, saved in `QuestRecord::roll`; `roll_known` guards saves through
`Quests::unknown_to`). `world/quest_roll.rs` is the play: `branch_open` (no unrevealed hidden
location of the run's overworld leads into the branch), `quest_pool`, `roll_quest` (stream
`Rng::quest(seed, id)` = `quest/<id>`, used once at the giving, weights summed then one
`below`), `quest_roll_dev` (the dev `quest_roll:ID:TARGET`: active, no progress, target in the
full pool or a variant id; resets `counted`, fires a variant's reveal), `quest_rolls` (the
`progress` dump's `roll ...` rows), and Quarry sense. A rolled quest whose pool is empty now is
not offered. Test content: the showcase gardener's `slug_charm` (a pick of scrolls; text
snapshots `quest-offer-rolled`, `quest-log-rolled`), and `world/quest_roll_tests.rs`'s
fixtures in Cēaptūn (Godgifu gives a creature pick, a tag pick, and two variants revealing
Fennmynster and Fyrdelf) and the arena (a kobold made the demilich, a shared unique).
**The bounties system** (`[[system]] bounties` in `data/talents.toml`): Quarry sense (delver,
`quarry_sense`: `World::sense_quarry` on arrival, beside Sixth sense, "Your quarry is here."
when a living creature on the level counts for an active, unmet kill quest as given) and Sworn
foe (weaponcraft, `choice = "tag"`, a new `talent::Choice::Tag` over `Content::tags`;
`sworn_foe { to_hit, damage }`, read in `World::favoured` beside Favoured prey). Tests:
`world/quest_roll_tests.rs`.
**Creature tags** (#421, #441): a creature file's `tags = ["goblin", "archer"]` (plain
singular words, `content/tag.rs`: lower-case words joined by hyphens, 1 to `MAX_TAG` = 24
characters, each once; nothing declares a tag, it exists because a creature carries it; one
flat pool, no registry). A kill quest's `target = "tag:goblin"` makes `Goal::Kill`'s `target` a
`KillTarget::Tag` (`count` up to `MAX_TAG_COUNT`, 999, against a type's 99); the quest is
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
**Guilds** and their ranks, derived from these records and named as a quest is given or done,
are `docs/dev/guild.md`.
Presentation reads `PerceptionView::quests()` (`QuestView` strings, shared by the log and the
`quests` dump layer). Creature types have a `plural` for the messages ("3 / 3 giant rats").
