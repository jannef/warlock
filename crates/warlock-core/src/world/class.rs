//! Class abilities in the world (spec §8 "Class abilities"; `data/classes/`'s level tables). A
//! class gives the talents its table names as the player reaches their levels: at creation
//! (level 1's), as each level comes ([`World::level_up`] calls [`World::grant_class`] with its
//! news), and again when a run is restored, so a table that grew reaches saved characters (never
//! taking anything away). A given talent is held like a picked one, at rank 1, so every rule
//! reads it through `Actor::talent_mods`; what a rank hands out once (hit points by level, a stat
//! gift) is handed out as it is given ([`hand_out`]). A talent already held (a class ability the
//! player picked before their class gave it) is not given again.
//!
//! The one ability here that runs with time is Connoisseur's: a magic item carried long enough
//! is identified ([`World::appraise_carried`]).

use super::{Actor, World, focus};
use crate::content::{Content, TalentDef, TalentId};
use crate::talent::TalentMods;

/// Hands out what holding rank `rank` of `def` gives once, as it is taken or given at character
/// `level`: its hit points by level, into `actor`'s base hit points and hit points, its mana by
/// level likewise (Deep well), and its stat gifts, into its scores and potentials (a gift of
/// Toughness raises its hit points with its maximum).
pub(super) fn hand_out(content: &Content, actor: &mut Actor, def: &TalentDef, rank: u8, level: u8) {
    let mods = TalentMods::of(content, def.effects_at(rank));
    let before = actor.max_hp(content);
    for (stat, by) in mods.stats.iter().filter(|&(_, by)| by != 0) {
        actor.stats.gift(stat, by);
    }
    let hp = mods.hp_at(level) + actor.max_hp(content) - before;
    actor.hp_base += mods.hp_at(level);
    actor.hp += hp;
    actor.mana_base += mods.mana_at(level);
    actor.mana += mods.mana_at(level);
    focus::hold(content, actor, &mods.grants, level);
}

/// Gives `actor` the talents `class` (an id; none without a class) gives up to `level` that it
/// does not hold yet, each at rank 1 with what it hands out; returns them in table order.
pub(super) fn grant(
    content: &Content,
    class: Option<&str>,
    actor: &mut Actor,
    level: u8,
) -> Vec<TalentId> {
    let Some(class) = class.and_then(|c| content.class(c)) else {
        return Vec::new();
    };
    let book = content.talents();
    let mut given = Vec::new();
    for id in class.abilities_to(book, level) {
        let def = book.get(id);
        if actor.talents.rank_of(&def.id, None) > 0 {
            continue;
        }
        actor.talents.take(&def.id, None);
        hand_out(content, actor, def, 1, level);
        given.push(id);
    }
    given
}

impl World {
    /// The player's class's id, if the run has a character.
    fn class_id(&self) -> Option<String> {
        (self.state.character.as_ref()).map(|c| c.spec.class.clone())
    }

    /// Gives the player what their class gives up to `level` that they lack ([`grant`]); with
    /// `news`, each is news of the level ("You gain Fencing.").
    pub(super) fn grant_class(&mut self, level: u8, news: bool) {
        let content = self.content.clone();
        let class = self.class_id();
        let given = grant(&content, class.as_deref(), self.player_mut(), level);
        // A run restored keeps what its talents grant (spec §8 "Resources").
        let grants = self
            .state
            .player()
            .talent_mods(&content)
            .into_owned()
            .grants;
        focus::hold(&content, self.player_mut(), &grants, level);
        if news {
            for id in given {
                self.news
                    .push(format!("You gain {}.", content.talents().get(id).name));
            }
        }
    }

    /// The trees a bonus talent pick of the player's class may come from, in display order;
    /// `None`: any tree.
    pub(crate) fn bonus_trees(&self) -> Option<Vec<crate::content::TreeId>> {
        let class = self.class_id();
        (class.and_then(|c| self.content.class(&c)))
            .and_then(|c| c.bonus_trees(self.content.talents()))
    }

    /// `seconds` of carrying pass for the player (spec §8 "Class abilities": Connoisseur): with
    /// the ability, each unidentified magic item they carry counts them, and one carried long
    /// enough is identified ("This is ...!"). Nothing without it.
    pub(super) fn appraise_carried(&mut self, seconds: u64) {
        let Some(hours) = self
            .state
            .player()
            .talent_mods(&self.content)
            .identify_carried
        else {
            return;
        };
        let needed = u64::from(hours) * crate::time::HOUR;
        let mut known = Vec::new();
        let content = self.content.clone();
        for (i, stack) in self.player_mut().inventory.iter_mut().enumerate() {
            if stack.carry_for(&content, seconds, needed) {
                known.push(i);
            }
        }
        for i in known {
            self.identify(i);
        }
    }
}
