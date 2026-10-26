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
topics come first in `World::topics` (`quest_topics`: `quest-ID`, `accept-ID`, `decline-ID`) and
are answered in `World::ask` (`ask_quest`, `Spent::NoTime` when state changes; a quest is
offered only when `World::offers` it: the player's band meets its `requires_alignment`, and a
giver who scorns them offers none whose `alignment` is good; `complete` makes the quest's
`alignment` shift, `docs/dev/alignment.md`); `Quests` in
`WorldState` (records by quest id: stage, completed, counted, reached; the kill tally by level
and creature id, for every type (#244: the ranger's Favoured prey reads it too), and `prey`, the
type killed most, the first to get there of those tied), left out of the JSON while empty so runs
without quests hash as before. Hooks: `fight.rs::dies` tallies the
player's kills (`tally_kill`, beside `earn_kill`, so a creature bled to death by the player's
wound counts too), `travel` marks reach quests (`reach`), and `World::apply` snapshots active
quests' `progress` before the player acts and says what rose after it, and again after the
turn (`quest_progress`/`quest_news`), so pick-ups, kills, and arrivals all report through one
place. `reveal_at` (beside the answer at the three places a stage is set: `ask_quest`'s offer,
`accept`, `complete`) reveals a quest's location after the speaker's words. No abandoning and no cap on active quests (the owner's decision): don't add either.
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
Presentation reads `PerceptionView::quests()` (`QuestView` strings, shared by the log and the
`quests` dump layer). Creature types have a `plural` for the messages ("3 / 3 giant rats").
