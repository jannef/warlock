//! Services (spec §8 "Services"; the campaign design §6.1, #329, #432, #1391): a cure, a
//! mending, an identification, or a skill taught, sold for gold by someone whose dialogue has a
//! `[[dialogue.service]]`.
//!
//! A service is asked for like a topic ([`World::topics`] lists it after the topics, with its
//! price; `ask:E:cure` in action text), so it needs no screen, action, or key of its own.
//! [`World::ask_service`] does it:
//! - **Refusals** (no time, nothing taken): with nothing to do the speaker says the service's
//!   `nothing` ("Nothing ails you, child."); a mending while a disease shows gets "Be cured
//!   first." (the damage would come back, as the daily mending is held then); without the gold
//!   the player is told "You cannot pay."
//! - **Paying**: the price in the shops' currency moves from the player's pack into the
//!   speaker's, through the purse code `pay` uses (`shop.rs`'s `hand_over`).
//! - **`cure`**: [`World::cure`] with [`PRAYER`], which every disease lists in its `cured_by`
//!   (`data/diseases.toml`), incubating ones too: the speaker sees what the player does not
//!   yet. The cure events say what ended.
//! - **`mend`**: [`World::mend`]: every point of stat damage comes back at once, and every point
//!   drains took (spec §8 "Stats": "Drain"; asked for with only drained points, it mends too).
//! - **`identify`** (#432): every inventory stack the player does not fully know
//!   ([`World::unidentified`]: unknown magic, or an unidentified scroll type) is identified in
//!   one go through [`World::identify`], each saying what it turned out to be. Its price is per
//!   such stack (not per piece), so the total is the data's price times their count.
//! - **`train_skill`** (#1391): the player, untrained in the skill its `train` names, is trained
//!   in it at `train.value` ([`crate::skills::Skills::add`]; a fixed number, no roll), and told
//!   "Alberich trains you in mining." Offered only while they are untrained in it
//!   ([`World::service_offered`]: [`World::topics`] leaves it out once trained; asked for anyway,
//!   the speaker says its `nothing`, no time). Moved by favour like a healer's (never Pious).
//!
//! Then the speaker says the service's `reply`, and it takes the time of `chat`. The price is
//! the data's, moved by the player's favour (their Presence, skills, and talents; a keeper's
//! alignment is a shop's, not a service's) for a healer's or a sage's, and never for a
//! `temple` service (a temple does not haggle, #1031): the price less the favour in percent,
//! rounded up, at least 1. Then the goodwill of the speaker's town (`goodwill.rs`, #1247) takes
//! its percent off that, rounded up: only 100% off makes it free, and a free service changes no
//! hands and trains nothing. Every service paid trains Negotiation ([`Use::Deal`]). Pious halves
//! a cure's or a mending's price (rounded up) first, and with Pious a cure also mends a point of each
//! damaged stat ([`FaithMods::services`], the one hook). Lore and Connoisseur do not move an
//! identification's price. No stream is drawn, and nothing new is saved: a service changes only what is
//! saved already (the pack, the diseases, the stat damage, the scores and drained points).
//!
//! [`FaithMods::services`]: crate::talent::FaithMods::services

use super::goodwill::discounted;
use super::shop::{count_of, haggling};
use crate::action::Event;
use crate::content::{ActionKind, DialogueId, MAX_FAVOUR, Service, ServiceKind};
use crate::skills::SkillId;
use crate::skills::Use;
use crate::text::capitalized;
use crate::world::turn::actor::undrained;
use crate::world::{ActorId, Spent, World};

/// The means a temple's cure names in `cured_by`.
pub const PRAYER: &str = "prayer";

/// What it says to a mending asked for while a disease shows.
const CURE_FIRST: &str = "Be cured first.";

impl World {
    /// What `service`, sold by whoever speaks `dialogue`, costs the player now, per stack for an
    /// identification: a cure's or a mending's half, rounded up, with Pious; then, unless a
    /// temple sells it, less the player's favour; then less their town's goodwill (see the
    /// module docs).
    pub fn service_price(&self, dialogue: DialogueId, service: &Service) -> u32 {
        discounted(self.haggled_price(service), self.goodwill_off(dialogue))
    }

    /// [`World::service_price`] before the goodwill.
    fn haggled_price(&self, service: &Service) -> u32 {
        let healing = matches!(service.does, ServiceKind::Cure | ServiceKind::Mend);
        let price = if healing
            && self
                .state
                .player()
                .talent_mods(&self.content)
                .faith
                .services
        {
            service.price.div_ceil(2)
        } else {
            service.price
        };
        let Some(shops) = self.content.shops().filter(|_| !service.temple) else {
            return price;
        };
        let favour =
            haggling(&self.content, shops, self.state.player()).clamp(-MAX_FAVOUR, MAX_FAVOUR);
        let percent = u64::from((100 - favour).unsigned_abs());
        let haggled = (u64::from(price) * percent).div_ceil(100).max(1);
        u32::try_from(haggled).unwrap_or(u32::MAX)
    }

    /// How many things `service` would be paid for now: the player's unidentified stacks for an
    /// identification, `None` for a service paid once.
    pub fn service_items(&self, service: &Service) -> Option<u32> {
        (service.does == ServiceKind::Identify)
            .then(|| u32::try_from(self.unidentified().len()).unwrap_or(u32::MAX))
    }

    /// Whether `service` is offered to the player now: a training only while they are untrained
    /// in its skill (never without a skills table to know it by); every other service always.
    pub fn service_offered(&self, service: &Service) -> bool {
        if service.train.is_none() {
            return true;
        }
        (self.service_skill(service)).is_some_and(|skill| {
            let player = self.state.player();
            player.skills.get(skill).is_none()
        })
    }

    /// The skill a `train_skill` service teaches, and `None` for any other (or without a skills
    /// table: the loader refuses an unknown skill otherwise).
    fn service_skill(&self, service: &Service) -> Option<SkillId> {
        let train = service.train.as_ref()?;
        self.content.skill_id(&train.skill)
    }

    /// The positions of the player's inventory stacks they do not fully know: unidentified magic,
    /// or a scroll of a type not yet identified.
    pub fn unidentified(&self) -> Vec<usize> {
        let lore = self.lore();
        (self.state.player().inventory.iter().enumerate())
            .filter(|(_, stack)| !lore.knows(stack))
            .map(|(i, _)| i)
            .collect()
    }

    /// Asks creature `id`, whose dialogue is `dialogue`, for its service `topic` (see the module
    /// docs). `None` when it sells no such service.
    pub(super) fn ask_service(
        &mut self,
        id: ActorId,
        dialogue: DialogueId,
        topic: &str,
        events: &mut Vec<Event>,
    ) -> Option<Spent> {
        let service = self.content.dialogue(dialogue).service(topic)?.clone();
        let player = self.state.player;
        let actor = self.state.player();
        let refusal = match service.does {
            ServiceKind::Cure if !self.prayer_cures(player) => Some(service.nothing.clone()),
            ServiceKind::Mend if !actor.damage.is_damaged() && undrained(&actor.drained) => {
                Some(service.nothing.clone())
            }
            ServiceKind::Mend if actor.diseases.shows() => Some(CURE_FIRST.to_string()),
            ServiceKind::Identify if self.unidentified().is_empty() => {
                Some(service.nothing.clone())
            }
            ServiceKind::TrainSkill if !self.service_offered(&service) => {
                Some(service.nothing.clone())
            }
            _ => None,
        };
        if let Some(line) = refusal {
            self.answer(id, line, Some(topic.to_string()), events);
            return Some(Spent::Nothing);
        }
        let items = self.service_items(&service).unwrap_or(1);
        let price = self.service_price(dialogue, &service).saturating_mul(items);
        let currency = self.content.shops().map(|s| s.currency);
        let gold = currency.map_or(0, |c| count_of(&self.state.player().inventory, c));
        if gold < price {
            self.say("You cannot pay.");
            return Some(Spent::Nothing);
        }
        // Free for the town's goodwill (`goodwill.rs`): nothing changes hands.
        if price > 0 {
            self.hand_over(player, id, price);
            let who = self.called(&self.state.actors[&id]);
            self.say(format!("You pay {who} {price} gold."));
            self.train(Use::Deal);
        }
        match service.does {
            ServiceKind::Cure => {
                self.cure(player, PRAYER, events);
                let pilgrim = self
                    .state
                    .player()
                    .talent_mods(&self.content)
                    .faith
                    .services;
                if let Some(actor) = self.state.actors.get_mut(&player)
                    && pilgrim
                    && !actor.diseases.shows()
                {
                    actor.damage.mend_point();
                }
            }
            ServiceKind::Mend => self.mend(player),
            ServiceKind::Identify => {
                for item in self.unidentified() {
                    self.identify(item);
                }
            }
            ServiceKind::TrainSkill => self.teach_skill(id, &service),
        }
        self.answer(id, service.reply, Some(topic.to_string()), events);
        Some(Spent::Turn(self.price_of(player, ActionKind::Chat)))
    }

    /// A `train_skill` service done by `teacher`: the player trained in its skill at its value,
    /// and told so in words.
    fn teach_skill(&mut self, teacher: ActorId, service: &Service) {
        let (Some(skill), Some(train)) = (self.service_skill(service), &service.train) else {
            return;
        };
        self.player_mut().skills.add(skill, i32::from(train.value));
        let name = self.content.skill(skill).name.to_lowercase();
        let who = capitalized(&self.called(&self.state.actors[&teacher]));
        self.say(format!("{who} trains you in {name}."));
    }

    /// Whether a prayer would end anything of actor `id`'s: a status or a disease (showing or
    /// not) whose `cured_by` lists [`PRAYER`].
    fn prayer_cures(&self, id: ActorId) -> bool {
        let Some(actor) = self.state.actors.get(&id) else {
            return false;
        };
        let prays = |by: &[String]| by.iter().any(|c| c == PRAYER);
        (actor.effects.iter()).any(|e| prays(&self.content.status(e.status).cured_by))
            || (actor.diseases.iter()).any(|i| prays(&self.content.disease(i.disease).cured_by))
    }

    /// Every point of actor `id`'s stat damage mends at once, and every point drains took comes
    /// back (a temple's mending).
    pub(crate) fn mend(&mut self, id: ActorId) {
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.damage.mend_all();
        }
        self.restore_drained(id);
    }
}
