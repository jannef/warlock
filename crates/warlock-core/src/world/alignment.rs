//! Alignment's rules (spec §8 "Alignment", #254): what moves the player's score and what reads
//! it. [`World::shift_alignment`] is the one place a score changes (`crate::alignment` holds the
//! score, the bands, and the diminishing; `content/alignment.rs` the numbers).
//!
//! **Deeds** ([`Deed`]), each hooked in once where it happens: a kill (`fight.rs::dies`, beside
//! the experience it earns: [`World::judge_kill`]), attacking a shopkeeper, theft, and paying
//! what is owed (`shop.rs`: [`World::player_did`]), eating a corpse of one's own race or of a
//! peaceful creature who talks (`eat.rs`: [`World::ate_corpse`]), and a quest's own shift on
//! completion (`quest.rs`), and consecrating an unholy place (`consecrate.rs`). Stealing (`steal.rs`: `World::got_away_with`, a theft that took something)
//! is a [`Deed::Theft`] too.
//!
//! **Readers**: good folk turn cold toward an Evil or Wicked character, evil folk toward a Good
//! or Saintly one ([`World::scorns`]): a cold greeting in place of their words
//! ([`World::cold_greeting`]) and none of their quests that would do the player good
//! ([`World::offers`], which also checks a quest's `requires_alignment`). A good shopkeeper's
//! prices follow the band ([`keeper_favour`]); a holy gift works only for the bands it names
//! ([`World::answered`]), and is never barred.

use super::{Actor, ActorId, World};
use crate::alignment::Alignment;
use crate::content::{Content, CreatureId, Dialogue, QuestDef, Shifts, Shops, Stance};

/// What moved an alignment score. Talents that change what a deed costs (one that spares an
/// evil character's conscience the killing of peaceful folk) will read it in
/// [`World::shift_alignment`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deed {
    /// Killed someone not hostile to them.
    KilledPeaceful,
    /// Killed a beast (a neutral creature that does not talk) not hostile to them: hunting.
    Hunted,
    /// Killed someone hostile to them.
    KilledHostile,
    /// Attacked a shopkeeper who traded with them.
    AttackedShopkeeper,
    /// Left a shop with goods they had not paid for.
    Theft,
    /// Paid a shopkeeper what they owed.
    PaidDebt,
    /// Ate the corpse of one of their own race, or of a peaceful creature who talks.
    AteKin,
    /// Completed a quest that moves the score.
    Quest,
    /// Consecrated an unholy place (Consecrate).
    Consecrated,
    /// Prayed at a forgotten altar of this side (spec §7 "Special rooms"): a good altar's
    /// prayer is a little good, an evil one's evil, a neutral one's nothing.
    Prayed(Alignment),
}

impl Deed {
    /// What it is worth by the rules, for the deeds they fix; `None` for those whose worth
    /// depends on whom or what (a kill, a quest).
    fn worth(self, shift: &Shifts) -> Option<i32> {
        match self {
            Deed::AttackedShopkeeper => Some(shift.attack_shopkeeper),
            Deed::Theft => Some(shift.theft),
            Deed::PaidDebt => Some(shift.pay_debt),
            Deed::AteKin => Some(shift.eat_kin),
            Deed::Consecrated => Some(shift.consecrate),
            Deed::Prayed(Alignment::Good) => Some(shift.prayed_good),
            Deed::Prayed(Alignment::Evil) => Some(shift.prayed_evil),
            Deed::Prayed(Alignment::Neutral) => Some(0),
            Deed::KilledPeaceful | Deed::Hunted | Deed::KilledHostile | Deed::Quest => None,
        }
    }
}

/// How far a shop's keeper moves its prices in `actor`'s favour for their alignment, in percent
/// (spec §7 "Shops"): a good keeper by the actor's band (`[shops] favour` in
/// `data/alignment.toml`); others not at all. Every keeper is of the `[keeper]` type.
pub(super) fn keeper_favour(content: &Content, shops: &Shops, actor: &Actor) -> i32 {
    let keeper = content.creature(shops.keeper.creature).alignment;
    match content.alignment() {
        Some(rules) if keeper == Alignment::Good => rules.favour(actor.band(content)),
        _ => 0,
    }
}

impl World {
    /// The one place an alignment score changes: `actor`'s moves by a deed worth `amount` (good
    /// if positive), diminishing as [`crate::alignment::Score::shifted`] says. The player is
    /// told when it crosses into another band, after the deed's own messages, as a note that
    /// does not stop a rest; anything smaller is silent. Nothing happens to an actor without a score (creatures stand where
    /// their type does) or without alignment rules.
    pub(super) fn shift_alignment(&mut self, actor: ActorId, amount: i32, cause: Deed) {
        // The cause is the hook for the talents that will change a deed's cost.
        let _ = cause;
        let Some(rules) = self.content.alignment() else {
            return;
        };
        let Some(score) = self.state.actors.get(&actor).and_then(|a| a.alignment) else {
            return;
        };
        let shifted = score.shifted(amount);
        let news = (rules.news(rules.band(score), rules.band(shifted))).map(str::to_string);
        if let Some(a) = self.state.actors.get_mut(&actor) {
            a.alignment = Some(shifted);
        }
        if let Some(text) = news.filter(|_| actor == self.state.player) {
            self.notes.push(text);
        }
    }

    /// The player did `deed`, whose worth the alignment rules fix (attacking a shopkeeper,
    /// theft, paying a debt, eating kin).
    pub(super) fn player_did(&mut self, deed: Deed) {
        let worth = (self.content.alignment()).and_then(|rules| deed.worth(&rules.shift));
        if let Some(amount) = worth {
            self.shift_alignment(self.state.player, amount, deed);
        }
    }

    /// The player killed `victim`, still in the world: killing someone hostile is worth what
    /// their alignment says (an evil one: a little good); killing someone not hostile is evil,
    /// the more so if they are good, but a beast (a neutral creature that does not talk) may be
    /// hunted.
    pub(super) fn judge_kill(&mut self, victim: ActorId) {
        let (Some(rules), Some(actor)) = (self.content.alignment(), self.state.actors.get(&victim))
        else {
            return;
        };
        let nature = actor.nature(&self.content);
        let talks = (self.content)
            .dialogue_of(actor.creature, actor.npc.as_deref())
            .is_some();
        let shift = rules.shift;
        let (amount, deed) = if self.stance(self.state.player, victim) == Stance::Hostile {
            (shift.kill_hostile.of(nature), Deed::KilledHostile)
        } else if nature == Alignment::Neutral && !talks {
            (shift.kill_beast, Deed::Hunted)
        } else {
            (shift.kill_peaceful.of(nature), Deed::KilledPeaceful)
        };
        self.shift_alignment(self.state.player, amount, deed);
    }

    /// The player ate a corpse of `creature`: eating one of their own race (the creature's
    /// `race`), or a creature that talks and is not hostile to them by its type, is eating kin.
    pub(super) fn ate_corpse(&mut self, creature: CreatureId) {
        let def = self.content.creature(creature);
        let race = (self.state.character.as_ref()).map(|c| c.spec.race.as_str());
        let kin = def.race.is_some() && def.race.as_deref() == race;
        let player = self.state.player().creature;
        let peaceful = self.content.stance(player, creature) != Stance::Hostile;
        let talks = self.content.dialogue_of(creature, None).is_some();
        if kin || (peaceful && talks) {
            self.player_did(Deed::AteKin);
        }
    }

    /// Whether `actor` turns cold toward the player for their alignment: a good creature
    /// toward an Evil or Wicked character, an evil one toward a Good or Saintly character
    /// ([`crate::alignment::Band::scorned_by`]).
    pub(super) fn scorns(&self, actor: &Actor) -> bool {
        let band = self.state.player().band(&self.content);
        actor.kind != super::ActorKind::Player && band.scorned_by(actor.nature(&self.content))
    }

    /// What creature `id`, speaking `d`, says in place of its greeting and its lines while it
    /// scorns the player: one of its `greet_evil` lines (a good speaker) or `greet_good` lines
    /// (an evil one), in turn like the one-liners (`said` of them so far); `None` while it does
    /// not scorn them, or has no such lines.
    pub(super) fn cold_greeting(&self, id: ActorId, d: &Dialogue, said: u32) -> Option<String> {
        let actor = self.state.actors.get(&id).filter(|a| self.scorns(a))?;
        let lines = match actor.nature(&self.content) {
            Alignment::Good => &d.greet_evil,
            Alignment::Evil => &d.greet_good,
            Alignment::Neutral => return None,
        };
        let key = self.rotation_key(id, b"cold");
        let k = super::talk::rotation(key, said, lines.len());
        lines.get(k).cloned()
    }

    /// Whether quest `def` may be offered to the player by someone who scorns them or not
    /// (`scorned`): the player's band meets its `requires_alignment`, and someone who scorns
    /// them offers no quest that would do them good (one whose `alignment` shift is good).
    pub(super) fn offers(&self, def: &QuestDef, scorned: bool) -> bool {
        let band = self.state.player().band(&self.content);
        let required = (def.requires_alignment).is_none_or(|r| r.allows(band));
        required && !(scorned && def.alignment > 0)
    }
}
