//! Class abilities in the world (spec §8 "Class abilities"; `data/classes/`'s level tables). A
//! class gives the talents its table names as the player reaches their levels: at creation
//! (level 1's), as each level comes ([`World::level_up`] calls [`World::grant_class`] with its
//! news), and again when a run is restored, so a table that grew reaches saved characters (never
//! taking anything away). A given talent is held like a picked one, at the rank the table names
//! (`{ level = 25, talent = "jack_of_all_trades", rank = 2 }`; 1 by default), so every rule
//! reads it through `Actor::talent_mods`; what a rank hands out once (hit points by level, a stat
//! gift, weapon ranks, skills taken up) is handed out as it is given ([`hand_out`]). A talent already held (a pickable the
//! table names, which the player picked before their class gave it) is not given again: the
//! level brings a talent pick in its place instead ([`held_already`], #790).
//!
//! The one ability here that runs with time is Connoisseur's: a magic item carried long enough
//! is identified ([`World::appraise_carried`]).

use super::{Actor, World, focus};
use crate::character::Character;
use crate::content::{Content, Role, TalentDef, TalentEffect, TalentId};
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
    actor.grow_mana_base(content, mods.mana_at(level));
    focus::hold(content, actor, &mods.grants, level);
    for effect in def.effects_at(rank) {
        if let TalentEffect::WeaponRanks { to, skills, .. } = effect {
            raise_ranks(content, actor, to, &skills);
        }
    }
}

/// A `weapon_ranks` hand-out (Travelled hands): every weapon skill and style `skills` names
/// (none: every row but the schools of magic and hidden styles, whatever the content holds)
/// held below rank `to` rises to it, in file order, its defaults with it.
fn raise_ranks(content: &Content, actor: &mut Actor, to: u8, skills: &[String]) {
    let table = content.weapon_skills();
    let rows: Vec<_> = match skills.is_empty() {
        true => (table.ids())
            .filter(|(_, s)| s.role != Role::School && !s.hidden)
            .map(|(id, _)| id)
            .collect(),
        false => skills.iter().filter_map(|s| table.id(s)).collect(),
    };
    for id in rows {
        actor.weapon_skills.raise_to(table, id, to);
    }
}

/// What the hand-outs of rank `rank` of `def` say as a class table gives it (Travelled hands:
/// one line, never one per skill).
fn handed_news(def: &TalentDef, rank: u8) -> Vec<String> {
    (def.effects_at(rank))
        .filter_map(|e| match e {
            TalentEffect::WeaponRanks { news, .. } | TalentEffect::TakeUpSkills { news, .. } => {
                news
            }
            _ => None,
        })
        .collect()
}

/// Gives `actor` (of `character`; none without one) the talents their classes give up to total
/// level `level` at the ranks they name that it does not hold yet, table by table (the class's
/// to its cap, a prestige class's to its own level; spec §8 "Prestige classes"), each rank with
/// what it hands out: a rank a table names comes to one holding the rank below, and every rank
/// missing below it comes with it (a restore); returns each rank given, `(talent, rank)`, in
/// table order.
pub(super) fn grant(
    content: &Content,
    character: Option<&Character>,
    actor: &mut Actor,
    level: u8,
) -> Vec<(TalentId, u8)> {
    let Some(character) = character else {
        return Vec::new();
    };
    let book = content.talents();
    let mut given = Vec::new();
    for (table, own) in character.class_levels(content, level) {
        for (id, rank) in table.ranks_to(book, own) {
            let def = book.get(id);
            for r in actor.talents.rank_of(&def.id, None) + 1..=rank {
                actor.talents.take(&def.id, None);
                hand_out(content, actor, def, r, level);
                given.push((id, r));
            }
        }
    }
    given
}

/// The talents the class of total level `level` gives at exactly that level that `actor`
/// already holds: a pickable the table names (Fast healer, Beast-friend, Evasion, Riposte) that
/// the player took as a pick before the class gave it. Each brings a talent pick in its place
/// (#790), so a pick made early is never lost to the table.
pub(super) fn held_already(
    content: &Content,
    character: Option<&Character>,
    actor: &Actor,
    level: u8,
) -> Vec<TalentId> {
    let Some((table, own)) = character.and_then(|c| c.class_of(content, level)) else {
        return Vec::new();
    };
    let book = content.talents();
    if !book.has_class_tree() {
        return Vec::new();
    }
    (table.abilities().iter())
        .filter(|(l, _, _)| *l == own)
        .filter_map(|(_, id, rank)| book.id(id).map(|t| (t, *rank)))
        .filter(|&(id, rank)| actor.talents.rank_of(&book.get(id).id, None) >= rank)
        .map(|(id, _)| id)
        .collect()
}

impl World {
    /// Gives the player what their classes give up to `level` that they lack ([`grant`]); with
    /// `news`, each is news of the level ("You gain Fencing.").
    pub(super) fn grant_class(&mut self, level: u8, news: bool) {
        let content = self.content.clone();
        let character = self.state.character.clone();
        let given = grant(&content, character.as_ref(), self.player_mut(), level);
        // A run restored keeps what its talents grant (spec §8 "Resources").
        let grants = self
            .state
            .player()
            .talent_mods(&content)
            .into_owned()
            .grants;
        focus::hold(&content, self.player_mut(), &grants, level);
        for &(id, rank) in &given {
            // Jack of all trades: the skills it takes up, from the levels stream.
            self.take_up_skills(content.talents().get(id).effects_at(rank));
        }
        if news {
            for (id, rank) in given {
                let def = content.talents().get(id);
                self.news.push(match rank {
                    1 => format!("You gain {}.", def.name),
                    _ => format!("Your {} grows.", def.name),
                });
                self.news.extend(handed_news(def, rank));
            }
        }
    }

    /// The trees a bonus talent pick brought by total level `level` may come from (the class of
    /// that level's `bonus_talents`), in display order; `None`: any tree.
    pub(crate) fn bonus_trees_at(&self, level: u8) -> Option<Vec<crate::content::TreeId>> {
        let character = self.state.character.as_ref()?;
        let (table, _) = character.class_of(&self.content, level)?;
        table.bonus_trees(self.content.talents())
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
