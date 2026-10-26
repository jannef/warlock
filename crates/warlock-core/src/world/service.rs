//! Services (spec §8 "Services"; the campaign design §6.1, #329): a cure or a mending sold for
//! gold by someone whose dialogue has a `[[dialogue.service]]`.
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
//!
//! Then the speaker says the service's `reply`, and it takes the time of `chat`. The price is
//! the data's, never moved by favour (a temple does not haggle): Pilgrim halves it (rounded up),
//! and with Pilgrim a cure also mends a point of each damaged stat ([`TalentMods::services`],
//! the one hook). No stream is drawn, and nothing new is saved: a service changes only what is
//! saved already (the pack, the diseases, the stat damage, the scores and drained points).
//!
//! [`TalentMods::services`]: crate::talent::TalentMods::services

use super::actor::undrained;
use super::shop::count_of;
use super::{ActorId, Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, DialogueId, ServiceKind};

/// The means a temple's cure names in `cured_by`.
pub const PRAYER: &str = "prayer";

/// What it says to a mending asked for while a disease shows.
const CURE_FIRST: &str = "Be cured first.";

impl World {
    /// What a service listed at `price` costs the player now: half, rounded up, with Pilgrim.
    pub fn service_price(&self, price: u32) -> u32 {
        if self.state.player().talent_mods(&self.content).services {
            price.div_ceil(2)
        } else {
            price
        }
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
            _ => None,
        };
        if let Some(line) = refusal {
            self.answer(id, line, Some(topic.to_string()), events);
            return Some(Spent::Nothing);
        }
        let price = self.service_price(service.price);
        let currency = self.content.shops().map(|s| s.currency);
        let gold = currency.map_or(0, |c| count_of(&self.state.player().inventory, c));
        if gold < price {
            self.say("You cannot pay.");
            return Some(Spent::Nothing);
        }
        self.hand_over(player, id, price);
        let who = self.called(&self.state.actors[&id]);
        self.say(format!("You pay {who} {price} gold."));
        match service.does {
            ServiceKind::Cure => {
                self.cure(player, PRAYER, events);
                let pilgrim = self.state.player().talent_mods(&self.content).services;
                if let Some(actor) = self.state.actors.get_mut(&player)
                    && pilgrim
                    && !actor.diseases.shows()
                {
                    actor.damage.mend_point();
                }
            }
            ServiceKind::Mend => self.mend(player),
        }
        self.answer(id, service.reply, Some(topic.to_string()), events);
        Some(Spent::Turn(self.price_of(player, ActionKind::Chat)))
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
