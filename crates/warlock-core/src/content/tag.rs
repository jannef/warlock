//! Creature tags (spec §8 "Quests", #441): plain words on creature types, one flat pool, as a
//! card game has creature types. A creature file says `tags = ["goblin", "archer"]`; nothing
//! declares a tag: it exists because a creature carries it. A kill quest asks for one as
//! `target = "tag:goblin"`, and any type that carries it counts.
//!
//! A tag is the base word, singular (`goblin`, not `goblins`): lower-case letters in words
//! joined by hyphens (`marsh-thing`), 1 to [`MAX_TAG`] characters. Said to the player, its
//! hyphens are spaces and its plural comes from the plural dictionary ([`super::Plurals`]).
//! Membership is by creature **type**: an actor's `faction_override` changes nothing.

use super::Content;
use super::creature::CreatureId;

/// The longest tag, in characters.
pub const MAX_TAG: usize = 24;
/// The prefix of a kill quest's `target` that names a tag (`tag:goblin`).
pub const TAG_TARGET: &str = "tag:";

/// Whether `tag` is written as a tag: lower-case words joined by hyphens, 1 to [`MAX_TAG`]
/// characters. The error says what is wrong.
pub fn check(tag: &str) -> Result<(), String> {
    let words_ok = tag
        .split('-')
        .all(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_lowercase()));
    if tag.is_empty() || tag.len() > MAX_TAG || !words_ok {
        return Err(format!(
            "tag `{tag}`: tags are lower-case words joined by hyphens, 1 to {MAX_TAG} characters"
        ));
    }
    Ok(())
}

/// A tag as the player reads it, one of them: `marsh-thing` is "marsh thing".
pub fn spoken(tag: &str) -> String {
    tag.replace('-', " ")
}

impl Content {
    /// Whether creature type `creature` carries tag `tag`.
    pub fn has_tag(&self, creature: CreatureId, tag: &str) -> bool {
        self.creature(creature).tags.iter().any(|t| t == tag)
    }

    /// The creature types that carry tag `tag`, in table order.
    pub fn tagged<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = CreatureId> + 'a {
        (0..self.creature_defs().len())
            .map(|i| CreatureId(i as u16))
            .filter(move |&c| self.has_tag(c, tag))
    }

    /// The types carrying tag `tag` that a run of campaign `campaign` can meet: the shared ones
    /// and that campaign's (every one when `campaign` is `None`, outside every campaign).
    pub fn tagged_in<'a>(
        &'a self,
        tag: &'a str,
        campaign: Option<&'a str>,
    ) -> impl Iterator<Item = CreatureId> + 'a {
        self.tagged(tag).filter(move |&c| {
            let theirs = self.creature(c).campaign.as_deref();
            campaign.is_none() || theirs.is_none() || theirs == campaign
        })
    }

    /// Every tag some creature carries, sorted, each once.
    pub fn tags(&self) -> Vec<&str> {
        let all: std::collections::BTreeSet<&str> = (self.creature_defs().iter())
            .flat_map(|c| c.tags.iter().map(String::as_str))
            .collect();
        all.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_are_lower_case_words_and_hyphens() {
        for good in ["goblin", "marsh-thing", "a", "abcdefghijklmnopqrstuvwx"] {
            assert_eq!(check(good), Ok(()), "{good}");
        }
        for bad in [
            "",
            "Goblin",
            "goblin_chief",
            "goblin chief",
            "-goblin",
            "goblin-",
            "marsh--thing",
            "orc2",
            "abcdefghijklmnopqrstuvwxy",
        ] {
            assert!(check(bad).is_err(), "{bad:?}");
        }
        assert_eq!(spoken("marsh-thing"), "marsh thing");
    }

    #[test]
    fn the_builtin_tags_and_their_members() {
        let content = Content::builtin().expect("content");
        let members = |tag: &str| -> Vec<String> {
            (content.tagged(tag))
                .map(|c| content.creature(c).id.clone())
                .collect()
        };
        assert_eq!(
            members("goblin"),
            ["goblin", "goblin_archer", "goblin_chief"],
            "the ogre and the troll left out"
        );
        assert_eq!(
            members("kobold"),
            ["kobold", "kobold_archer", "kobold_shaman"]
        );
        assert!(!members("vermin").contains(&"basilisk".to_string()));
        assert!(members("vermin").contains(&"rat".to_string()));
        assert!(!members("duergar").contains(&"duergar_trader".to_string()));
        assert!(members("marsh-thing").contains(&"bog_hag".to_string()));
        assert!(members("fire-thing").contains(&"fire_drake".to_string()));
        assert!(members("caster").contains(&"kobold_shaman".to_string()));
        assert!(members("archer").contains(&"goblin_archer".to_string()));
        // Every troll: the tribe's, the fen's, and the risen one.
        for troll in ["troll", "marsh_troll", "rotting_troll", "elder_troll"] {
            assert!(members("troll").contains(&troll.to_string()), "{troll}");
        }
        assert!(content.tags().iter().all(|t| check(t).is_ok()));
    }

    /// Later undead batches cannot forget it: every creature of the `undead` faction carries
    /// the `undead` tag.
    #[test]
    fn every_undead_carries_the_undead_tag() {
        let content = Content::builtin().expect("content");
        let undead = content.factions().id("undead").expect("undead faction");
        for (i, def) in content.creature_defs().iter().enumerate() {
            if def.faction == Some(undead) {
                assert!(
                    content.has_tag(CreatureId(i as u16), "undead"),
                    "`{}` is of the undead faction: give it `tags = [\"undead\"]`",
                    def.id
                );
            }
        }
    }
}
