# Guilds

Developer notes for guilds and their ranks (#425; the guild design `docs/design/adventurers-guild.md`
§4, §7.6, §10); the rules are in spec/08b-creatures.md ("Quests": "Guilds"). Keep it current with
the code, as CLAUDE.md says.

## Data

`content/guild.rs` loads the `guilds` table (`data/guilds/` and each campaign's `data/guilds/`, one
file per guild, `[[guild]]`) after the quests: `GuildDef { id, name, master, ranks, campaign }`,
`RankDef { title, material, by }`, `RankBy::{Given(QuestId), Done(QuestId)}`. Checks: the id, the
name (1 to 32 characters), the master (a speaker with a dialogue, `Content::speaker`), 1 to
`MAX_RANKS` ranks, each title 1 to `MAX_TITLE` characters and listed once, its `material` a
material, `given` or `done` (never both) a quest of the guild's own campaign; a rank with neither
(kept for later links) comes after every rank that has one. The field reference is the comment
at the top of `campaigns/ascmearc/data/guilds/weyfarers.toml`.

## Ranks

`world/guild.rs`. A rank is **derived** from the quest records, never saved, and nothing reads
it as a rule: `GuildDef::rank(given, done)` is the last rank whose quest is given (on the log,
active or done) or done (completed at least once); `Quests::guild_rank`/`guild_ranks` apply it
to the player's records, and `PerceptionView::own_guilds` gives the presentation `GuildRank`s
(the guild and the rank's index: `title()`, `GuildDef::color` for the material's colour).

The promotion message has one hook: `World::give` and `World::complete` (`world/quest.rs`) take
`guild_standing()` before they change the record and call `promote(&before)` at their end, which
says "<Master> names you a <Title> of <guild name>." for each guild whose rank rose. Asking again
(a reminder) changes nothing, so it is said once.

Shown: the character screen (`ui/character_screen.rs`, a `Guild` row under the alignment, the
title in its material's colour), the `stats` dump (`Guild Weyfarers' Gilde, Tin`), and the JSON
dump's `guilds` (`id`, `name`, `rank`; left out before joining one).

## The Weyfarers' Gilde

The Ascmearc's guild: `campaigns/ascmearc/data/guilds/weyfarers.toml` (seven ranks named by
allomantic metals of `data/materials/`, ordered by value: Tin, given `guild_quarry`; Pewter, done
`guild_hundred`; Iron, Steel, Duralumin, Electrum, Gold reserved, Iron for link 4, #426), its
master Wat One-Eye (`data/creatures/guildmaster.toml`, neutral, so he scorns no one; npc `wat` in
`data/dialogue/ceaptun.toml`, Cockney, topics `guild` and `board` after the one quest topic), and
his chain `data/quests/wat.toml`: `guild_quarry` (a creature pick of the three branches),
`guild_hundred` (a tag pick of seven group tags, count 100), `guild_scroll` (a pick of the
scrolls loot holds, a bring). The Gildhalle is in `levels/ceaptun.toml` (`docs/dev/campaign.md`).

## Tests

`world/guild_tests.rs` (Wat at his post and reached from the square, his topics one link at a
time and none gated for a Wicked character, Tin as link 1 is given and said once, Pewter as
link 2 is done, the scroll handed in and the unpaid one still billed, a replay, the load
checks), `ui/character_screen.rs` (the row and the metal's colour), the text snapshots
`character-guild` and `dialogue-guild`, and the `ceaptun_start` view snapshot. The wiki page is
`wiki/src/guild.md` with the tables of `xtask/src/wiki/guild.rs`.
