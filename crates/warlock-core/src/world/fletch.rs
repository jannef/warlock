//! Fletching (spec §8 "Fletching"; #1021): [`Action::Fletch`], the Fletch ability, Woodcraft's use
//! though anyone may.
//!
//! - **What it needs** ([`fletch_lack`], the abilities menu's reason and the rules' refusal, in the
//!   same words, with no time): a knife carried ([`Content::is_knife`]; in the pack, worn, wielded,
//!   or neither), and a launcher ready whose `launches` has a row in `data/fletch.toml` (a bow, a
//!   crossbow; never a sling). Anywhere, and no material is used up.
//! - **Time**: the file's `seconds`, in turns of the `fletch` action ([`World::keep_at`], the
//!   rest's machinery, the last turn cut short to end on time; less with Fletcher, its effect's
//!   `time_percent`): a creature coming into view, a message, or a blow stops it, and a batch cut
//!   short makes nothing (the time is spent). On the overworld it is watched as a forage is
//!   ([`Watch::Forage`]: awake, in the open, for as long).
//! - **The check**, once the time is up: two gameplay draws (the natural roll), then the count's
//!   dice (a draw per die): the natural roll plus the Dexterity modifier, the talents' Dexterity
//!   checks, what Woodcraft gives ([`Effect::Fletch`]), and Fletcher's `check`, less the file's
//!   `difficulty`, is the margin; the count is the dice plus Woodcraft's trained value over
//!   `value_per_count`; [`FletchRules::made`] is what the margin leaves of it; Fletcher then
//!   multiplies that by `count_times`.
//! - **What**: `plain` from `plain_from` of Woodcraft, or with Fletcher; else `brittle`. Into the
//!   pack (joining a stack like it, the readied one too). Then Woodcraft earns its mark
//!   ([`Use::Fletch`], the training stream), whatever came of it: no anti-farming limit, time is
//!   the price.
//!
//! [`Action::Fletch`]: crate::action::Action::Fletch
//! [`FletchRules::made`]: crate::content::FletchRules::made

use super::encounter::Watch;
use super::rest::Ended;
use super::{Actor, Spent, World};
use crate::ability::Unavailable;
use crate::action::Event;
use crate::combat;
use crate::content::{ActionKind, Content, FletchMakes};
use crate::object::{self, Stack};
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::time::Duration;

/// What `actor` lacks to fletch: a knife, else a launcher of a kind `data/fletch.toml` lists.
pub(crate) fn fletch_lack(actor: &Actor, content: &Content) -> Option<Unavailable> {
    let knife = (actor.inventory.iter()).any(|stack| content.is_knife(stack.object));
    if !knife {
        return Some(Unavailable::NoKnife);
    }
    fletch_makes(actor, content)
        .is_none()
        .then_some(Unavailable::NoLauncher)
}

/// What `actor`'s ready launcher's ammunition is made as, if it has one fletching makes for.
fn fletch_makes<'c>(actor: &Actor, content: &'c Content) -> Option<&'c FletchMakes> {
    let (launcher, _) = actor.launcher(content)?;
    let kind = content.object(launcher).launches.as_deref()?;
    content.fletch()?.makes_for(kind)
}

impl World {
    /// The player begins a batch; refused, taking no time, lacking a knife or a launcher.
    pub(super) fn start_fletch(&mut self) -> Spent {
        let content = self.content.clone();
        let Some(rules) = content.fletch() else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        if let Some(lack) = fletch_lack(me, &content) {
            let reason = lack.reason(&content);
            let mut letters = reason.chars();
            let first = letters.next().map_or(' ', |c| c.to_ascii_uppercase());
            self.say(format!("{first}{}.", letters.as_str()));
            return Spent::Nothing;
        }
        let fletcher = me.talent_mods(&content).fletcher;
        let percent = fletcher.map_or(100, |f| f.time_percent);
        self.say("You sit down with your knife and some straight wood.");
        Spent::Fletch(rules.seconds * percent / 100)
    }

    /// Works turn after turn until the batch's time is up or something stops it (module docs):
    /// its last turn cut short to end exactly when its time is up.
    pub(super) fn fletch(&mut self, seconds: u32, events: &mut Vec<Event>) {
        let until = self.state.time + Duration::seconds(u64::from(seconds));
        let turn = self.price_of(self.state.player, ActionKind::Fletch);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| {
                let price = turn.min(w.price_until(until));
                w.watch_roll(Watch::Forage, price);
                price
            },
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.fletched(),
            Ended::Came(came) => {
                let text = format!("{}; you put down the knife.", self.came_line(came));
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You put down the knife."),
        }
    }

    /// A batch ran its time: the check, what is made, the mark (module docs).
    fn fletched(&mut self) {
        let content = self.content.clone();
        let Some(rules) = content.fletch() else {
            return;
        };
        let me = self.state.player();
        // Something may have taken the bow or the knife while the time passed.
        let (None, Some(makes)) = (fletch_lack(me, &content), fletch_makes(me, &content)) else {
            self.say("You have nothing left to work with.");
            return;
        };
        let mods = me.talent_mods(&content);
        let fletcher = mods.fletcher;
        let modifier = me.effective_stats(&content).modifier(Stat::Dexterity)
            + mods.check(Stat::Dexterity)
            + me.skill_effect(&content, Effect::Fletch)
            + fletcher.map_or(0, |f| f.check);
        let woodcraft = me.skills.value_of_effect(&content, Effect::Fletch);
        let natural = combat::roll_natural(&mut self.state.rng);
        let margin = natural + modifier - rules.difficulty;
        let dice = rules.count.roll(&mut self.state.rng).max(1);
        let count = dice + i32::from(woodcraft / rules.value_per_count);
        let made = rules.made(margin, count) * fletcher.map_or(1, |f| f.count_times);
        if made > 0 {
            let plain = fletcher.is_some() || woodcraft >= rules.plain_from;
            let object = if plain { makes.plain } else { makes.brittle };
            let stack = Stack::new(object, made.unsigned_abs());
            let text = format!(
                "You make {}.",
                object::phrase(std::slice::from_ref(&stack), self.lore())
            );
            self.say(text);
            object::add_to_inventory(self.state.inventory_mut(), stack, &content);
        } else {
            self.say("You spoil every shaft you cut.");
        }
        if self.trains(Use::Fletch) {
            self.train(Use::Fletch);
        }
    }
}
