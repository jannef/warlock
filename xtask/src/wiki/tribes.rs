//! Tribes (spec §8 "Quests", `data/tribes/`): the groups of creature types a kill quest may ask
//! for as a whole, with every member named exactly.

use super::order;
use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{Content, CreatureId};

/// Every tribe the wiki describes, by name: its members (the types of its factions, less its
/// exceptions, and any named besides), the factions it is drawn from, and the types of those
/// factions that are not members.
pub(super) fn tribes(content: &Content) -> String {
    let mut out = String::from("| Tribe | Members | Factions | Not members |\n|---|---|---|---|\n");
    let mut tribes: Vec<_> = (content.tribes().iter())
        .filter(|t| order::described(content, t.campaign.as_deref()))
        .collect();
    tribes.sort_by_cached_key(|t| t.plural.to_lowercase());
    for tribe in tribes {
        let names = |ids: &mut dyn Iterator<Item = CreatureId>| {
            let mut names: Vec<String> = ids
                .map(|c| content.creature(c))
                .filter(|c| order::described(content, c.campaign.as_deref()))
                .map(|c| c.name.clone())
                .collect();
            names.sort_by_cached_key(|n| n.to_lowercase());
            if names.is_empty() {
                "-".to_string()
            } else {
                names.join(", ")
            }
        };
        let mut factions: Vec<String> = (tribe.factions.iter())
            .map(|&f| content.factions().def(f).name.clone())
            .collect();
        factions.sort();
        let factions = if factions.is_empty() {
            "-".to_string()
        } else {
            factions.join(", ")
        };
        let _ = writeln!(
            out,
            "| {} | {} | {factions} | {} |",
            capital(&tribe.plural),
            names(&mut tribe.members.iter().copied()),
            names(&mut tribe.except.iter().copied()),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tribes_name_their_members_and_who_is_left_out() {
        let content = Content::builtin().expect("content");
        let table = tribes(&content);
        assert!(
            table.contains(
                "| Goblins | goblin, goblin archer, goblin chief | goblins | ogre, troll |"
            ),
            "{table}"
        );
        assert!(
            table.contains("| Kobolds | kobold, kobold archer, kobold shaman | kobolds | - |"),
            "{table}"
        );
        assert_eq!(table.lines().count(), 2 + 7, "{table}");
    }
}
