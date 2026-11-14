//! Wriggling past (spec §8 "Displacement", Escape artist; #1032): a step into a cell held by a
//! creature that will not make way ([`Refusal::Budge`]: it keeps its post or never gives way)
//! is, for a player trained in Escape artist, an automatic check instead of a refusal, as the
//! check against bonds is part of the attempt (`hold.rs`): no action, no ability row, no key.
//!
//! The player's Dexterity modifier, what talents add to Dexterity's checks, and Escape artist's
//! `escape` points, against the `[wriggle]` difficulty of `data/tuning.toml` plus the creature's
//! Willpower modifier (and its talents'). Two draws of the gameplay stream. Made, the step goes
//! ahead as the swap of a talked-to NPC does (`talk.rs::swap_places`); failed, the turn is the
//! step's price and the player stays. Every try marks Escape artist ([`Use::Wriggle`]) at the
//! data's chance.
//!
//! Never for a hostile creature (the step is a blow then), a creature too big to squeeze past, one
//! that cannot move at all (overloaded), or while swimming (those refusals stand), and never for a
//! shop's keeper (the owner, #1032): it keeps its door, the refusal stands, and no draw, mark, or
//! time is spent.

use super::talk::Refusal;
use super::*;
use crate::check::check;
use crate::skills::Effect as SkillEffect;
use crate::stats::Stat;

/// How a step into a creature that will not make way came out.
pub(super) enum Wriggled {
    /// Not a wriggle: the refusal stands.
    No,
    /// Made: the step goes ahead and swaps places.
    Past,
    /// Failed: this is the turn spent.
    Stuck(Spent),
}

impl World {
    /// Whether the player, stepping onto `to` where creature `id` stands refusing for `why`,
    /// wriggles past it (module docs). `seen`: whether the player sees it.
    pub(super) fn wriggle(&mut self, id: ActorId, why: Refusal, to: Pos, seen: bool) -> Wriggled {
        let content = self.content.clone();
        let me = self.state.player();
        if why != Refusal::Budge
            || !me.skills.trains_effect(&content, SkillEffect::Escape)
            || !self.load_of(id).penalty().can_move()
            || self.state.current().shops.iter().any(|s| s.keeper == id)
        {
            return Wriggled::No;
        }
        let modifier = me
            .check_modifier(&content, Stat::Dexterity)
            .saturating_add(me.skill_effect(&content, SkillEffect::Escape));
        let Some(them) = self.state.actors.get(&id) else {
            return Wriggled::No;
        };
        let nerve = them.check_modifier(&content, Stat::Willpower);
        let who = match seen {
            true => self.called(them),
            false => "something".to_string(),
        };
        let past = check(
            &mut self.state.rng,
            modifier,
            content.tuning().wriggle.difficulty.saturating_add(nerve),
        );
        if self.trains(Use::Wriggle) {
            self.train(Use::Wriggle);
        }
        if past {
            self.say(format!("You wriggle past {who}."));
            return Wriggled::Past;
        }
        self.say(format!(
            "You try to wriggle past {who}, but it is in the way."
        ));
        let player = self.state.player;
        let onto = self.state.current().terrain[to];
        Wriggled::Stuck(Spent::Turn(self.step_price(
            player,
            self.content.terrain(onto),
            false,
        )))
    }
}
