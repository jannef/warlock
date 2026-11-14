//! Treating oneself or an ally (spec §8 "Skills": the Heal ability, #549; design
//! ranger-companion.md §9, "Tending"): the use of the Heal skill,
//! [`crate::action::Action::Heal`] (`heal`, `heal:ID`, `heal:ID:DIR`). What may be treated is
//! [`crate::heal`]; here is the try.
//!
//! - **Who**: an actor trained in a skill with the `heal` effect (Heal, `data/skills/`):
//!   the abilities menu lists the Heal ability for them ([`Ability::Heal`]). Anyone else is told
//!   "You know nothing of healing." and no time passes.
//! - **The patient**: the player, or with a direction the player's ally that way
//!   ([`allies_beside`]: on the level, beside them, whatever its source); none there is refused
//!   with no time ("You have no ally there.").
//! - **What**: the ailment the action names (a status id or a disease id), else the player's
//!   first there is to treat ([`crate::heal::treatable`]). Nothing to treat, or one that may not
//!   be treated now (not suffered, or a disease still incubating), is refused with no time and
//!   no draw.
//! - **A try** is a check ([`crate::check`]): 2d12 plus the healer's Intelligence modifier (of the
//!   effective score, spec §8 "Stats", and what talents add to every Intelligence check) plus
//!   what Heal gives ([`Effect::Heal`]) against a status's `treat` difficulty at the patient's
//!   degrees, or a disease's own `difficulty`. It takes the `heal` action's time whatever comes of it, may earn
//!   Heal a training mark ([`Use::Heal`]), and makes no noise.
//! - **Made**: a status that `stops` (bleeding) ends; any other (poison) eases a degree, ending
//!   at none; a disease is eased: it counts as a progression check won (toward its `recovery`),
//!   unless that would shake it off or it never recovers by itself, when its next progression is
//!   put off to a full interval from now instead. A try never cures a disease outright.
//! - **Failed**: any ailment may be tried again at once (time is the price, the owner's rule).

use crate::ability::Ability;
use crate::action::Event;
use crate::check::check;
use crate::content::{ActionKind, DiseaseId, StatusId};
use crate::geom::Dir;
use crate::heal::{self, Ailment};
use crate::rng::draw;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::world::{Actor, ActorId, Spent, World, WorldState};

/// The player's allies beside them on the current level, each by the way to it, in direction
/// order: the patients the Heal ability may tend besides the player.
pub(crate) fn allies_beside(state: &WorldState) -> Vec<(Dir, &Actor)> {
    let me = state.player();
    (Dir::ALL.into_iter())
        .filter_map(|d| {
            let ally = state.actor_at(&state.current_level, me.pos.step(d))?;
            (ally.master() == Some(state.player)).then_some((d, ally))
        })
        .collect()
}

impl World {
    /// The player treats `what` (an ailment's id) on themselves, or on their ally the way
    /// `ally`; or the first thing there is to treat on themselves with neither (module docs).
    /// Returns what it cost.
    pub(in crate::world) fn heal(
        &mut self,
        what: Option<&str>,
        ally: Option<Dir>,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let healer = self.state.player();
        if !Ability::Heal.had_by(healer, &content) {
            self.say("You know nothing of healing.");
            return Spent::Nothing;
        }
        let patient = match ally {
            None => self.state.player,
            Some(d) => match allies_beside(&self.state).iter().find(|(w, _)| *w == d) {
                Some((_, a)) => a.id,
                None => {
                    self.say("You have no ally there.");
                    return Spent::Nothing;
                }
            },
        };
        let me = &self.state.actors[&patient];
        let ailment = match what {
            None => heal::treatable(me, &content).first().copied(),
            Some(id) => Ailment::named(&content, id),
        };
        let Some(ailment) = ailment else {
            self.say("There is nothing to treat.");
            return Spent::Nothing;
        };
        if !heal::may_treat(me, &content, ailment) {
            self.say("There is nothing like that to treat.");
            return Spent::Nothing;
        }
        let healer = self.state.player();
        let modifier = healer.check_modifier(&content, Stat::Intelligence)
            + healer.skill_effect(&content, Effect::Heal);
        let difficulty = match ailment {
            Ailment::Status(s) => {
                let severity = me.effects.get(s).map_or(1, |e| e.severity);
                (content.status(s).treat.as_ref()).map_or(0, |t| t.difficulty_at(severity))
            }
            Ailment::Disease(d) => content.disease(d).difficulty,
        };
        let player = self.state.player;
        let price = self.price_of(player, ActionKind::Heal);
        let made = check(&mut draw!(self, "heal/player"), modifier, difficulty);
        if self.trains(Use::Heal) {
            self.train(Use::Heal);
        }
        match ailment {
            Ailment::Status(s) => self.treat_status(patient, s, made, events),
            Ailment::Disease(d) => self.treat_disease(patient, d, made),
        }
        Spent::Turn(price)
    }

    /// The disease `d` of `patient` in a sentence: "the sickness", "the filth fever"; an ally's
    /// "your wolf's sickness".
    fn disease_words(&self, patient: ActorId, d: DiseaseId) -> String {
        let Some(actor) = self.state.actors.get(&patient) else {
            return String::new();
        };
        let noun = heal::noun(&self.content, actor, d);
        if patient == self.state.player {
            format!("the {noun}")
        } else if let Some(data) = actor.kept() {
            format!("{}'s {noun}", data.name)
        } else {
            let name = &self.content.creature(actor.creature).name;
            format!("your {name}'s {noun}")
        }
    }

    /// The try at `patient`'s status `status` came out `made`: made, one that stops ends, any
    /// other eases a degree.
    fn treat_status(
        &mut self,
        patient: ActorId,
        status: StatusId,
        made: bool,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(treat) = content.status(status).treat.as_ref() else {
            return;
        };
        let said = if made {
            treat.made.clone()
        } else {
            treat.failed.clone()
        };
        self.say(said);
        let Some(actor) = self.state.actors.get_mut(&patient) else {
            return;
        };
        let Some(effect) = actor.effects.get_mut(status) else {
            return;
        };
        if !made {
            return;
        }
        if !treat.stops && effect.severity > 1 {
            effect.severity -= 1;
            return;
        }
        let ended = *effect;
        actor.effects.remove(status);
        self.ended(patient, &[ended], events);
    }

    /// The try at `patient`'s disease `disease` came out `made`: made, it is eased (module
    /// docs).
    fn treat_disease(&mut self, patient: ActorId, disease: DiseaseId, made: bool) {
        let content = self.content.clone();
        let def = content.disease(disease);
        let words = self.disease_words(patient, disease);
        let Some(actor) = self.state.actors.get_mut(&patient) else {
            return;
        };
        let recovery = super::disease::recovery_of(def, &actor.talent_mods(&content));
        let Some(infection) = actor.diseases.get_mut(disease) else {
            return;
        };
        if !made {
            self.say(format!("You can do no more for {words}."));
            return;
        }
        if def.recovers() && infection.streak.saturating_add(1) < recovery {
            infection.streak += 1;
        } else {
            let interval = u32::try_from(def.interval_seconds()).unwrap_or(u32::MAX);
            infection.left = infection.left.max(interval);
        }
        self.say(format!("You ease {words} somewhat."));
    }
}
