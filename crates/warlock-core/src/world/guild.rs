//! Guild ranks (spec §8 "Quests": "Guilds"; the guild design §7.6): the player's rank in each
//! guild is derived from their quest records, never saved, and is a title only. The master names
//! the player to a new rank as the quest that reaches it is given or done, once.

use super::{Quests, Stage, World};
use crate::content::{Content, GuildDef};
use crate::perception::PerceptionView;

/// The player's standing in a guild they have joined.
#[derive(Clone, Debug)]
pub struct GuildRank<'a> {
    pub guild: &'a GuildDef,
    /// The rank, an index into the guild's `ranks`.
    pub rank: usize,
}

impl GuildRank<'_> {
    /// The rank's title ("Tin").
    pub fn title(&self) -> &str {
        &self.guild.ranks[self.rank].title
    }
}

impl Quests {
    /// The rank the player holds in `guild`, by index; `None` before joining.
    pub(crate) fn guild_rank(&self, content: &Content, guild: &GuildDef) -> Option<usize> {
        let record = |q| self.records.get(&content.quest(q).id);
        guild.rank(
            |q| record(q).is_some_and(|r| r.stage == Stage::Active || r.completed > 0),
            |q| record(q).is_some_and(|r| r.completed > 0),
        )
    }

    /// The guilds the player belongs to, in table order, with their ranks.
    pub(crate) fn guild_ranks<'a>(&self, content: &'a Content) -> Vec<GuildRank<'a>> {
        (content.guilds().iter())
            .filter_map(|guild| {
                let rank = self.guild_rank(content, guild)?;
                Some(GuildRank { guild, rank })
            })
            .collect()
    }
}

impl<'a> PerceptionView<'a> {
    /// The guilds the player belongs to and their ranks, which they know.
    pub fn own_guilds(&self) -> Vec<GuildRank<'a>> {
        self.quests.guild_ranks(self.content)
    }
}

impl World {
    /// Each guild's rank before a quest was given or done, to [`World::promote`] after.
    pub(super) fn guild_standing(&self) -> Vec<Option<usize>> {
        let content = &self.content;
        (content.guilds().iter())
            .map(|g| self.state.quests.guild_rank(content, g))
            .collect()
    }

    /// Names the player to each guild rank risen since `before` ([`World::guild_standing`]):
    /// "Wat One-Eye names you a Pewter of the Weyfarers' Gilde."
    pub(super) fn promote(&mut self, before: &[Option<usize>]) {
        let content = self.content.clone();
        for (guild, before) in content.guilds().iter().zip(before) {
            let now = self.state.quests.guild_rank(&content, guild);
            if let Some(rank) = now.filter(|&r| Some(r) > *before) {
                let title = &guild.ranks[rank].title;
                self.say(format!(
                    "{} names you {} {title} of {}.",
                    super::talk::capitalized(&content.speaker_called(guild.master)),
                    crate::content::article(title),
                    guild.name,
                ));
            }
        }
    }
}
