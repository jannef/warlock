//! The profane gifts in the world (docs/design/profane-gifts.md §7.3, §7.4;
//! `docs/dev/profane-gifts.md`): giving one ([`World::bestow_gift`], its relations applied) and
//! taking one away ([`World::rescind`]), with the dev actions `gift:ID` and `rescind:ID`; and
//! the meal that gives them ([`World::profane_meal`], §4): a profane corpse's signature, its
//! draws on the table (§7.2, [`World::draw_gift`]), and the pull toward evil.
//!
//! A gift held is a talent in [`Actor::talents`](super::Actor) and [`Actor::boons`](super::Actor),
//! so a restored run puts it back (`stars::grant`) and the state hash covers it. Its `stat`
//! effects are handed out as it takes and taken back as it goes; every other effect is read at its
//! hook and is simply there or not.

use super::alignment::Deed;
use super::{Spent, World};
use crate::content::{GiftDef, GiftTier, OnEat, TalentEffect, TalentId};
use crate::stats::DRAIN_FLOOR;
use crate::talent::{Taken, VigourEffect};

impl World {
    /// Whether a draw may give the player `gift` (§7.2's pool): not held, not excluded by a gift
    /// held, not barred to their race.
    pub(crate) fn may_take_gift(&self, gift: &GiftDef) -> bool {
        let me = self.state.player();
        let book = self.content.talents();
        let holds = |t: TalentId| me.talents.has(&book.get(t).id);
        let race = self.state.character.as_ref().map(|c| c.spec.race.as_str());
        !holds(gift.talent)
            && !gift.excludes.iter().any(|&t| holds(t))
            && !race.is_some_and(|r| gift.not_for.iter().any(|n| n == r))
    }

    /// The gifts of `tier` a draw may give the player now, and how many the tier holds (its
    /// drawn gifts: a signature is never drawn).
    pub(crate) fn gift_pool(&self, tier: GiftTier) -> (usize, usize) {
        let gifts = (self.content.profane().iter()).filter(|g| g.tier == tier && g.drawn());
        let (all, open) = gifts.fold((0, 0), |(all, open), g| {
            (all + 1, open + usize::from(self.may_take_gift(g)))
        });
        (open, all)
    }

    /// A profane corpse's own steps (§4, after its message and the gifts every corpse gives,
    /// before it feeds): its signature (`gives`), sure; then each of its `profane` rolls, a gift
    /// drawn ([`World::draw_gift`]) and given, each complete before the next draw; then the
    /// deed, [`Deed::EatProfane`], its notes said. Nothing for a corpse that is not profane.
    pub(super) fn profane_meal(&mut self, on_eat: &OnEat) {
        let (Some(gives), Some(rolls)) = (&on_eat.gives, &on_eat.profane) else {
            return;
        };
        let content = self.content.clone();
        if let Some(signature) = content.talents().id(gives) {
            self.bestow_gift(signature);
        }
        for _ in 0..rolls.rolls {
            match self.draw_gift(rolls) {
                Some(gift) => {
                    self.bestow_gift(gift);
                }
                None => self.say("Nothing more of it will take in you."),
            }
        }
        self.player_did(Deed::EatProfane);
        // The band crossed (and a paladin's zeal), said now, before the hunger news: the meal
        // ends after the last turn's perception.
        self.say_news();
    }

    /// One roll (§7.2): a tier by `rolls`' odds (one gameplay draw), then one of that tier's
    /// gifts the player may take by weight (another); an empty tier gives way to the next up,
    /// then down (lesser → greater → dire; dire → greater → lesser). `None` when all three are
    /// empty.
    pub(super) fn draw_gift(&mut self, rolls: &crate::content::ProfaneRolls) -> Option<TalentId> {
        let tier = *(self.state.rng).weighted(&GiftTier::ALL, |&t| u32::from(rolls.weight(t)))?;
        let order = match tier {
            GiftTier::Lesser => [GiftTier::Lesser, GiftTier::Greater, GiftTier::Dire],
            GiftTier::Greater => [GiftTier::Greater, GiftTier::Dire, GiftTier::Lesser],
            GiftTier::Dire => [GiftTier::Dire, GiftTier::Greater, GiftTier::Lesser],
        };
        let content = self.content.clone();
        for tier in order {
            let pool: Vec<&GiftDef> = (content.profane().iter())
                .filter(|g| g.tier == tier && g.drawn() && self.may_take_gift(g))
                .collect();
            if let Some(gift) = self.state.rng.weighted(&pool, |g| u32::from(g.weight)) {
                return Some(gift.talent);
            }
        }
        None
    }

    /// Gives the player `gift` (§7.3): an `undoes` with a gift it names held takes that away and
    /// is not taken (that gift's `undone` line, then this one's `instead`); else every gift its
    /// `replaces` names is taken away (each one's `undone` line), and it is taken with its `stat`
    /// handed out (its `line`, then "You are <name>."). Whether it was taken. Excludes and races
    /// are the pool's business ([`World::may_take_gift`]), not this.
    pub(super) fn bestow_gift(&mut self, gift: TalentId) -> bool {
        let content = self.content.clone();
        let Some(def) = content.profane().gift(gift) else {
            return false;
        };
        let book = content.talents();
        let holds = |w: &Self, t: TalentId| w.state.player().talents.has(&book.get(t).id);
        if holds(self, gift) {
            return false;
        }
        let undone: Vec<TalentId> = (def.undoes.iter().copied())
            .filter(|&t| holds(self, t))
            .collect();
        if !undone.is_empty() {
            for t in undone {
                self.rescind(t);
            }
            self.say(def.instead.clone().unwrap_or_default());
            return false;
        }
        let replaced: Vec<TalentId> = (def.replaces.iter().copied())
            .filter(|&t| holds(self, t))
            .collect();
        for t in replaced {
            self.rescind(t);
        }
        let id = book.get(gift).id.clone();
        self.player_mut().boons.push(Taken {
            id: id.clone(),
            choice: None,
        });
        self.regrant();
        self.shift_stats(&book.get(gift).effects, 1);
        self.say(def.line.clone());
        self.say(format!("You are {}.", book.get(gift).name));
        true
    }

    /// Takes gift `gift` away from the player (§7.4), saying its `undone` line: out of their
    /// talents and boons, its `stat` effects given back in the negative (never below
    /// [`DRAIN_FLOOR`]), hit points and mana kept within their maxima. Whether it was held.
    pub(super) fn rescind(&mut self, gift: TalentId) -> bool {
        let content = self.content.clone();
        let Some(def) = content.profane().gift(gift) else {
            return false;
        };
        let talent = content.talents().get(gift);
        let me = self.player_mut();
        let before = me.boons.len();
        me.boons.retain(|b| b.id != talent.id);
        if me.boons.len() == before {
            return false;
        }
        self.regrant();
        self.shift_stats(&talent.effects, -1);
        if let Some(line) = &def.undone {
            self.say(line.clone());
        }
        true
    }

    /// The player's granted talents put back from their character and boons (a gift added or
    /// gone), as a restored run puts them.
    fn regrant(&mut self) {
        let content = self.content.clone();
        let character = self.state.character.clone();
        let me = self.player_mut();
        let boons = std::mem::take(&mut me.boons);
        super::stars::grant(&content, character.as_ref(), &boons, &mut me.talents);
        me.boons = boons;
    }

    /// Hands out `effects`' `stat` effects times `sign`: a rise raises score and potential (a
    /// Toughness rise brings its hit points), a fall lowers both, never below [`DRAIN_FLOOR`],
    /// with stat damage, hit points, and mana kept within what is left.
    fn shift_stats(&mut self, effects: &[TalentEffect], sign: i32) {
        let content = self.content.clone();
        let id = self.state.player;
        for effect in effects {
            let &TalentEffect::Vigour(VigourEffect::Stat { stat, value }) = effect else {
                continue;
            };
            let by = value * sign;
            let me = self.player_mut();
            if by > 0 {
                let before = me.max_hp(&content);
                me.stats.gift(stat, by);
                me.hp += (me.max_hp(&content) - before).max(0);
            } else if by < 0 {
                let points = u8::try_from(-by).unwrap_or(u8::MAX);
                me.stats.lessen(stat, points, DRAIN_FLOOR);
                let score = me.stats.score(stat);
                me.damage.take(stat, 0, score);
            }
        }
        self.keep_hp_within_max(id);
        let me = self.player_mut();
        let max_mana = me.max_mana(&content);
        me.hp = me.hp.max(1);
        me.mana = me.mana.min(max_mana).max(0);
    }

    /// The dev action `gift:ID`: the player is given gift `id` as a draw would give it, refused
    /// (no time) for no gift, one held, or one a draw would not offer them. Takes no time.
    pub(super) fn gift_dev(&mut self, id: &str) -> Spent {
        let content = self.content.clone();
        let gift = (content.talents().id(id)).and_then(|t| content.profane().gift(t));
        let Some(gift) = gift else {
            self.say(format!("There is no profane gift `{id}`."));
            return Spent::Nothing;
        };
        if !self.may_take_gift(gift) {
            self.say(format!("`{id}` would not take in you."));
            return Spent::Nothing;
        }
        self.bestow_gift(gift.talent);
        Spent::NoTime
    }

    /// The dev action `rescind:ID`: gift `id` is taken from the player, refused (no time) for one
    /// not held. Takes no time.
    pub(super) fn rescind_dev(&mut self, id: &str) -> Spent {
        let content = self.content.clone();
        let gift = (content.talents().id(id)).filter(|&t| content.profane().gift(t).is_some());
        match gift {
            Some(t) if self.rescind(t) => Spent::NoTime,
            _ => {
                self.say(format!("You hold no profane gift `{id}`."));
                Spent::Nothing
            }
        }
    }
}

#[cfg(test)]
#[path = "profane_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "signature_tests.rs"]
mod signature_tests;
