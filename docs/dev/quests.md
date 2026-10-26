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
Presentation reads `PerceptionView::quests()` (`QuestView` strings, shared by the log and the
`quests` dump layer). Creature types have a `plural` for the messages ("3 / 3 giant rats").
