//! Treating oneself (spec §8 "Skills": the Heal ability, #549): the use of the Heal skill,
//! [`crate::action::Action::Heal`] (`heal`, `heal:ID`). What may be treated is
//! [`crate::heal`]; here is the try.
//!
//! - **Who**: an actor trained in a skill with the `heal` effect (Heal, `data/skills.toml`):
//!   the abilities menu lists the Heal ability for them ([`Ability::Heal`]). Anyone else is told
//!   "You know nothing of healing." and no time passes. For now the patient is the player
//!   themselves.
//! - **What**: the ailment the action names (a status id or a disease id), else the first there
//!   is to treat ([`crate::heal::treatable`]). Nothing to treat, or one that may not be treated
//!   now (one try spent), is refused with no time and no draw.
//! - **A try** is a check ([`crate::check`]): 2d12 plus the Intelligence modifier (of the
//!   effective score, spec §8 "Stats", and what talents add to every Intelligence check) plus
//!   what Heal gives ([`Effect::Heal`]) against a status's `treat` difficulty at its degrees, or a
//!   disease's own `difficulty`. It takes the `heal` action's time whatever comes of it, may earn
//!   Heal a training mark ([`Use::Heal`]), and makes no noise.
//! - **Made**: a status that `stops` (bleeding) ends; any other (poison) eases a degree, ending
//!   at none; a disease is eased: it counts as a progression check won (toward its `recovery`),
//!   unless that would shake it off or it never recovers by itself, when its next progression is
//!   put off to a full interval from now instead. A try never cures a disease outright.
//! - **Failed**: a status that `stops` may be tried again at once (time is the price, the
//!   owner's rule); a poison or a disease may not be tried again (its effect's or infection's
//!   `treated`, saved state, cleared as it ends). A poison or a disease is marked tried whatever
//!   comes of the try: one try each.

use super::{Spent, World};
use crate::ability::Ability;
use crate::action::Event;
use crate::check::check;
use crate::content::{ActionKind, DiseaseId, StatusId};
use crate::heal::{self, Ailment};
use crate::skills::{Effect, Use};
use crate::stats::Stat;

impl World {
    /// The player treats `what` (an ailment's id), or the first thing there is to treat with
    /// none (module docs). Returns what it cost.
    pub(super) fn heal(&mut self, what: Option<&str>, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let me = self.state.player();
        if !Ability::Heal.had_by(me, &content) {
            self.say("You know nothing of healing.");
            return Spent::Nothing;
        }
        let ailment = match what {
            None => heal::treatable(me, &content).first().copied(),
            Some(id) => Ailment::named(&content, id),
        };
        let Some(ailment) = ailment else {
            self.say("There is nothing to treat.");
            return Spent::Nothing;
        };
        if !heal::may_treat(me, &content, ailment) {
            let suffers = match ailment {
                Ailment::Status(s) => me.effects.has(s),
                Ailment::Disease(d) => me.diseases.get(d).is_some_and(|i| i.shown),
            };
            if !suffers {
                self.say("There is nothing like that to treat.");
            } else {
                let said = self.failed_words(ailment);
                self.say(said);
            }
            return Spent::Nothing;
        }
        let modifier = me.effective_stats(&content).modifier(Stat::Intelligence)
            + me.skill_effect(&content, Effect::Heal)
            + me.talent_mods(&content).check(Stat::Intelligence);
        let difficulty = match ailment {
            Ailment::Status(s) => {
                let severity = me.effects.get(s).map_or(1, |e| e.severity);
                (content.status(s).treat.as_ref()).map_or(0, |t| t.difficulty_at(severity))
            }
            Ailment::Disease(d) => content.disease(d).difficulty,
        };
        let player = self.state.player;
        let price = self.price_of(player, ActionKind::Heal);
        let made = check(&mut self.state.rng, modifier, difficulty);
        if self.trains(Use::Heal) {
            self.train(Use::Heal);
        }
        match ailment {
            Ailment::Status(s) => self.treat_status(s, made, events),
            Ailment::Disease(d) => self.treat_disease(d, made),
        }
        Spent::Turn(price)
    }

    /// What the player is told when a try at `ailment` fails, or when it may not be tried again.
    fn failed_words(&self, ailment: Ailment) -> String {
        match ailment {
            Ailment::Status(s) => (self.content.status(s).treat.as_ref())
                .map_or_else(String::new, |t| t.failed.clone()),
            Ailment::Disease(d) => {
                let noun = heal::noun(&self.content, self.state.player(), d);
                format!("You can do no more for the {noun}.")
            }
        }
    }

    /// The try at the player's status `status` came out `made`: made, one that stops ends, any
    /// other eases a degree; one that does not stop is spent either way.
    fn treat_status(&mut self, status: StatusId, made: bool, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(treat) = content.status(status).treat.as_ref() else {
            return;
        };
        let player = self.state.player;
        let said = if made {
            treat.made.clone()
        } else {
            treat.failed.clone()
        };
        self.say(said);
        let Some(actor) = self.state.actors.get_mut(&player) else {
            return;
        };
        let Some(effect) = actor.effects.get_mut(status) else {
            return;
        };
        if !treat.stops {
            effect.treated = true;
        }
        if !made {
            return;
        }
        if !treat.stops && effect.severity > 1 {
            effect.severity -= 1;
            return;
        }
        let ended = *effect;
        actor.effects.remove(status);
        self.ended(player, &[ended], events);
    }

    /// The try at the player's disease `disease` came out `made`: made, it is eased (module
    /// docs); either way it may not be tried again.
    fn treat_disease(&mut self, disease: DiseaseId, made: bool) {
        let content = self.content.clone();
        let def = content.disease(disease);
        let player = self.state.player;
        let noun = heal::noun(&content, self.state.player(), disease);
        let Some(actor) = self.state.actors.get_mut(&player) else {
            return;
        };
        let recovery = super::disease::recovery_of(def, &actor.talent_mods(&content));
        let Some(infection) = actor.diseases.get_mut(disease) else {
            return;
        };
        infection.treated = true;
        if !made {
            self.say(format!("You can do no more for the {noun}."));
            return;
        }
        if def.recovers() && infection.streak.saturating_add(1) < recovery {
            infection.streak += 1;
        } else {
            let interval = u32::try_from(def.interval_seconds()).unwrap_or(u32::MAX);
            infection.left = infection.left.max(interval);
        }
        self.say(format!("You ease the {noun} somewhat."));
    }
}
