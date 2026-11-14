//! The merchant's investment (spec §8 "Class abilities": Invest; #1407).
//!
//! A character with the Invest class ability (`FortuneMods::invest`, the merchant's from level 1)
//! has a business partner in town: a town's class helper for `invest` (`content/town.rs`),
//! placed with the aristocrat's servant's code (`stipend.rs::place_servant`), of the player's
//! race and speaking its register, whose dialogue has `[dialogue.partner]` words.
//!
//! - **Chatting** ([`World::partner_chat`], one line in `talk.rs::chat`): a one-liner (the
//!   rotation), then what the stake is worth in whole gold (`worth`), or that there is none
//!   (`idle`); it takes `chat`'s time and opens the dialogue screen on the partner's topics.
//! - **The topics** ([`World::invest_topics`], one line in `talk.rs::topics`): `invest_<n>` for
//!   each of `data/rules/invest.toml`'s `steps` the player carries the gold for, and `invest_all`;
//!   `withdraw_<n>` for each step the stake is worth, and `withdraw_all`. Asking one
//!   ([`World::ask_invest`], one line in `talk.rs::ask`) takes `chat`'s time: the gold goes in
//!   less the fee (`fees.invest` percent, rounded up), or comes out less its fee (`fees.withdraw`)
//!   into the pack, set down at the player's feet what would overload them. Any amount is taken
//!   by its id (`ask:E:invest_250`); more than the player has, or than the stake is worth, is
//!   refused with no time.
//! - **The week** ([`World::tick_fund`], one line in `World::apply` after the action's time has
//!   passed, so a long rest or a journey catches up every week gone): each game week (seven days
//!   from the epoch) begun since the last one the stake took changes by `week.base` plus the
//!   bonus plus `week.dice` basis points, drawn from the week's own stream `invest/<week>`, so
//!   nothing else moves and the result does not depend on when it is worked out. The stake is
//!   kept in hundredths of a coin, so small stakes grow too.
//! - **The bonus** ([`World::invest_bonus`]): `bonus.quest` per quest of the partner's town's
//!   people completed (each quest once), plus the partner's own `bonus` (its `[[town.servant]]`
//!   entry, a campaign's) for each quest it names completed (Fire in the
//!   deep). Derived from the quest records, never saved.
//! - **The partner's death** ([`World::partner_dies`], one line in `fight.rs::dies`): the stake
//!   is lost with them.
//!
//! Saved and hashed: [`WorldState::fund`](crate::world::WorldState), left out while nothing was ever
//! put in, so other runs hash as before.

use crate::action::Event;
use crate::content::{
    ActionKind, DialogueId, InvestRules, StatUse, TalentEffect, TownDef, TownServant,
};
use crate::object::Stack;
use crate::rng::Rng;
use crate::talent::FortuneEffect;
use crate::time::DAY;
use crate::world::{ActorId, Spent, TopicView, World};
use serde::{Deserialize, Serialize};

/// A game week in seconds.
pub const WEEK: u64 = 7 * DAY;

/// Hundredths of a coin in a coin.
const CENTS: u64 = 100;

/// A basis point's divisor: 10 000 basis points are the whole.
const WHOLE: i128 = 10_000;

/// The merchant's stake with their partner.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Fund {
    /// What it is worth, in hundredths of a gold coin.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub cents: u64,
    /// The last week (since the epoch) whose return it took.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub week: u64,
}

fn is_zero(n: &u64) -> bool {
    *n == 0
}

impl Fund {
    pub fn is_empty(&self) -> bool {
        self.cents == 0 && self.week == 0
    }

    /// What it is worth in whole coins.
    pub fn gold(&self) -> u32 {
        u32::try_from(self.cents / CENTS).unwrap_or(u32::MAX)
    }
}

/// A topic's amount: a number of coins, or everything.
enum Amount {
    All,
    Gold(u32),
}

/// Topic `topic` as a deal: put in (`true`) or taken out, and how much. `None` for any other.
fn deal(topic: &str) -> Option<(bool, Amount)> {
    let (put, amount) = match (
        topic.strip_prefix("invest_"),
        topic.strip_prefix("withdraw_"),
    ) {
        (Some(amount), _) => (true, amount),
        (_, Some(amount)) => (false, amount),
        _ => return None,
    };
    let amount = match amount {
        "all" => Amount::All,
        n => Amount::Gold(n.parse().ok().filter(|&n| n > 0)?),
    };
    Some((put, amount))
}

/// The stake `cents` after a week's return of `bp` basis points (never below nothing).
pub(crate) fn grown(cents: u64, bp: i32) -> u64 {
    let factor = (WHOLE + i128::from(bp)).max(0);
    u64::try_from(i128::from(cents) * factor / WHOLE).unwrap_or(u64::MAX)
}

/// The return of week `week` with the bonus `bonus`, in basis points: the base, the bonus, and
/// the week's roll from its own stream.
pub(crate) fn week_return(rules: &InvestRules, seed: u64, week: u64, bonus: i32) -> i32 {
    let mut rng = Rng::stream(seed, &format!("invest/{week}"));
    rules.base + bonus + rules.dice.roll(&mut rng)
}

impl World {
    /// The words of a business partner, if dialogue `d` is one's.
    fn partner_words(&self, d: DialogueId) -> Option<&crate::content::PartnerWords> {
        self.content.dialogue(d).partner.as_ref()
    }

    /// Whether the player deals with business partners: the Invest ability, and the rules.
    fn invests(&self) -> bool {
        self.content.invest().is_some()
            && self
                .state
                .player()
                .talent_mods(&self.content)
                .fortune
                .invest
    }

    /// The town that keeps the business partner, and its entry: the first class helper for a
    /// talent that invests.
    fn partner_town(&self) -> Option<(&TownDef, &TownServant)> {
        let book = self.content.talents();
        let invests = |talent: &str| {
            (book.id(talent)).is_some_and(|t| {
                (book.get(t).effects.iter())
                    .any(|e| matches!(e, TalentEffect::Fortune(FortuneEffect::Invest)))
            })
        };
        (self.content.towns().iter())
            .flat_map(|t| t.servants.iter().map(move |s| (t, s)))
            .find(|(_, s)| invests(&s.talent))
    }

    /// The weekly bonus, in basis points, the player's quests earn (the module docs).
    pub fn invest_bonus(&self) -> i32 {
        let Some(rules) = self.content.invest() else {
            return 0;
        };
        let Some((town, partner)) = self.partner_town() else {
            return 0;
        };
        let done = (self.state.quests.records.iter()).filter(|(_, r)| r.completed > 0);
        let mut bonus = 0i32;
        for (id, _) in done {
            let of_town = (self.content.quest_id(id))
                .map(|q| self.content.quest(q).giver)
                .and_then(|giver| self.content.town_of(giver))
                .is_some_and(|t| t == town);
            if of_town {
                bonus = bonus.saturating_add(rules.per_quest);
            }
            bonus = bonus.saturating_add(partner.bonus.get(id).copied().unwrap_or(0));
        }
        bonus
    }

    /// Every week begun since the stake last took its return takes it now (the module docs).
    pub(in crate::world) fn tick_fund(&mut self) {
        let now = self.state.time.seconds() / WEEK;
        if self.state.fund.cents == 0 || self.state.fund.week >= now {
            return;
        }
        let Some(rules) = self.content.invest().cloned() else {
            return;
        };
        let bonus = self.invest_bonus();
        let seed = self.state.seed;
        let fund = &mut self.state.fund;
        while fund.week < now && fund.cents > 0 {
            fund.week += 1;
            fund.cents = grown(fund.cents, week_return(&rules, seed, fund.week, bonus));
        }
        fund.week = now;
    }

    /// The partner's topics (the module docs); none for anyone else.
    pub(in crate::world) fn invest_topics(&self, dialogue: DialogueId) -> Vec<TopicView> {
        let (Some(_), Some(rules)) = (self.partner_words(dialogue), self.content.invest()) else {
            return Vec::new();
        };
        if !self.invests() {
            return Vec::new();
        }
        let topic = |id: String, name: String| TopicView {
            id,
            name,
            price: None,
            items: None,
            ware: None,
        };
        let gold = self.gold_carried();
        let worth = self.state.fund.gold();
        let mut topics = Vec::new();
        for &step in rules.steps.iter().filter(|&&s| s <= gold) {
            topics.push(topic(
                format!("invest_{step}"),
                format!("Invest {step} gold"),
            ));
        }
        if gold > 0 {
            let name = format!("Invest all your gold ({gold})");
            topics.push(topic("invest_all".into(), name));
        }
        for &step in rules.steps.iter().filter(|&&s| s <= worth) {
            topics.push(topic(
                format!("withdraw_{step}"),
                format!("Withdraw {step} gold"),
            ));
        }
        if worth > 0 {
            let name = format!("Withdraw everything ({worth} gold)");
            topics.push(topic("withdraw_all".into(), name));
        }
        topics
    }

    /// The gold coins the player carries.
    fn gold_carried(&self) -> u32 {
        crate::chest::currency(&self.content).map_or(0, |c| {
            crate::world::people::shop::count_of(&self.state.player().inventory, c)
        })
    }

    /// Chats with creature `id`, who speaks `dialogue`, if it is a business partner: a
    /// one-liner and what the stake is worth (the module docs). `None` for anyone else.
    pub(in crate::world) fn partner_chat(
        &mut self,
        id: ActorId,
        dialogue: DialogueId,
        events: &mut Vec<Event>,
    ) -> Option<Spent> {
        let content = self.content.clone();
        let d = content.dialogue(dialogue);
        let words = d.partner.as_ref()?;
        let said = self.state.actors[&id].talk.said;
        let line = self.one_liner(id, d, said);
        let worth = match self.state.fund.gold() {
            0 => words.idle.clone(),
            gold => words.worth.replace("{gold}", &gold.to_string()),
        };
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.talk.said += 1;
            actor.talk.met = true;
        }
        self.answer(id, format!("{line} {worth}"), None, events);
        self.train_stat(StatUse::Chat);
        Some(Spent::Turn(
            self.price_of(self.state.player, ActionKind::Chat),
        ))
    }

    /// Asks creature `id`, whose dialogue is `dialogue`, for deal `topic`: puts gold in or takes
    /// it out (the module docs). `None` when it is no partner's deal.
    pub(in crate::world) fn ask_invest(
        &mut self,
        id: ActorId,
        dialogue: DialogueId,
        topic: &str,
        events: &mut Vec<Event>,
    ) -> Option<Spent> {
        let words = self.partner_words(dialogue)?.clone();
        let (put, amount) = deal(topic)?;
        let rules = self.content.invest()?.clone();
        if !self.invests() {
            return None;
        }
        let (gold, fee) = if put {
            let carried = self.gold_carried();
            let gold = match amount {
                Amount::All => carried,
                Amount::Gold(n) => n,
            };
            if gold == 0 || gold > carried {
                self.say("You do not carry that much gold.");
                return Some(Spent::Nothing);
            }
            let fee = InvestRules::fee(gold, rules.fee_in);
            self.put_in(gold, fee);
            self.say(format!("You hand over {gold} gold."));
            (gold, fee)
        } else {
            let worth = self.state.fund.gold();
            let gold = match amount {
                Amount::All => worth,
                Amount::Gold(n) => n,
            };
            if gold == 0 || gold > worth {
                self.say("Your stake is not worth that much.");
                return Some(Spent::Nothing);
            }
            let fee = InvestRules::fee(gold, rules.fee_out);
            self.take_out(gold, fee);
            (gold, fee)
        };
        let said = if put {
            &words.invested
        } else {
            &words.withdrawn
        };
        let line = (said.replace("{gold}", &gold.to_string())).replace("{fee}", &fee.to_string());
        self.answer(id, line, Some(topic.to_string()), events);
        self.train_stat(StatUse::Chat);
        Some(Spent::Turn(
            self.price_of(self.state.player, ActionKind::Chat),
        ))
    }

    /// `gold` coins from the pack into the stake, `fee` of them kept by the partner. A stake
    /// started afresh takes its first return when the next week begins.
    fn put_in(&mut self, gold: u32, fee: u32) {
        let Some(coin) = crate::chest::currency(&self.content) else {
            return;
        };
        let player = self.player_mut();
        let gone = crate::world::people::shop::take(&mut player.inventory, coin, gold);
        player.equipment.after_removal(&gone);
        let now = self.state.time.seconds() / WEEK;
        let fund = &mut self.state.fund;
        if fund.cents == 0 {
            fund.week = now;
        }
        fund.cents = (fund.cents).saturating_add(u64::from(gold - fee) * CENTS);
    }

    /// `gold` coins out of the stake, `fee` of them kept by the partner and the rest received.
    fn take_out(&mut self, gold: u32, fee: u32) {
        let fund = &mut self.state.fund;
        fund.cents = fund.cents.saturating_sub(u64::from(gold) * CENTS);
        let paid = gold - fee;
        if let Some(coin) = crate::chest::currency(&self.content).filter(|_| paid > 0) {
            self.receive(vec![Stack::new(coin, paid)]);
        }
    }

    /// Creature `id` dies: a business partner takes the stake with them (the module docs).
    pub(in crate::world) fn partner_dies(&mut self, id: ActorId) {
        let partner = (self.state.actors.get(&id))
            .and_then(|a| self.dialogue_of(a))
            .is_some_and(|d| self.partner_words(d).is_some());
        if partner && self.state.fund.cents > 0 {
            self.state.fund.cents = 0;
            self.say("With your business partner dead, your stake is lost.");
        }
    }

    /// The `progress` dump's row (dev-only): the stake in hundredths, the last week it took,
    /// and the weekly bonus, `invest 98.00 gold, week 7550, bonus 3 bp`; `None` while nothing
    /// was ever put in.
    pub fn invest_row(&self) -> Option<String> {
        let fund = &self.state.fund;
        (!fund.is_empty()).then(|| {
            format!(
                "invest {}.{:02} gold, week {}, bonus {} bp",
                fund.cents / CENTS,
                fund.cents % CENTS,
                fund.week,
                self.invest_bonus()
            )
        })
    }
}
