//! Talking past a party on the overworld (spec §7 "Overworld encounters"; Bluff, #1017; the
//! design `docs/design/skill-uses.md` §6.2 B3): `encounter:bluff`, [`Choice::Bluff`].
//!
//! Offered ([`offered`]) only for an entry flagged `talk_past` (a hostile entry with a `group`,
//! `content/encounter.rs`), only when the party met the player face to face
//! ([`Outcome::Met`]), and only to a player trained in a skill with the `feint` benefit (Bluff:
//! [`crate::ability::Ability::Feint`]'s test). It sits between Evade and Fight.
//!
//! The party's lead type ([`World::lead_of`]) checks its Willpower modifier, two gameplay draws,
//! against `data/rules/feint.toml`'s `difficulty` + the player's Presence modifier + their talents'
//! Presence checks + Bluff's `feint` points: a feint's contest, at the party's lead. The lead
//! failed, "You talk your way past.": the party lets the player go, as [`Choice::Pass`] does, and
//! the try took a `chat` action's time. Made, "They are not fooled.": the fight opens as
//! [`Choice::Fight`] opens it, at its price. Marks [`Use::Bluffed`] whatever comes of it. This is
//! the one lie that gets the player out of anything: no caught player talks their way out (the
//! owner, B2).

use super::encounter::{Choice, How, Opening, Outcome, Pending};
use crate::check;
use crate::content::{ActionKind, Content, EncounterEntry};
use crate::rng::draw;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::world::{Actor, Spent, World};

/// The choices `pending` offers `player`: its outcome's, and Bluff's for a flagged entry met face
/// to face by one trained in it, between Evade and Fight.
pub(crate) fn offered(
    pending: &Pending,
    entry: &EncounterEntry,
    player: &Actor,
    content: &Content,
) -> Vec<Choice> {
    let mut choices = pending.outcome.choices().to_vec();
    let talks = entry.talk_past
        && pending.outcome == Outcome::Met
        && content.feint().is_some()
        && crate::ability::Ability::Feint.had_by(player, content);
    if talks {
        let at = (choices.iter())
            .position(|&c| c == Choice::Fight)
            .unwrap_or(choices.len());
        choices.insert(at, Choice::Bluff);
    }
    choices
}

impl World {
    /// `encounter:bluff` (the module docs): `pending`, of `entry`, answered by a lie.
    pub(super) fn talk_past(&mut self, entry: &EncounterEntry, pending: Pending) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let difficulty = content.feint().map_or(0, |f| f.difficulty);
        let me = self.state.player();
        let difficulty = difficulty
            + me.check_modifier(&content, Stat::Presence)
            + me.skill_effect(&content, Effect::Feint);
        let lead = self.lead_of(entry, &pending.members, pending.cell);
        let nerve = lead.map_or(0, |c| content.creature(c).stats.modifier(Stat::Willpower));
        let fooled = !check::check(&mut draw!(self, "talk_past/player"), nerve, difficulty);
        if self.trains(Use::Bluffed) {
            self.train(Use::Bluffed);
        }
        if fooled {
            self.say("You talk your way past.");
            return Spent::Turn(self.price_of(player, ActionKind::Chat));
        }
        self.say("They are not fooled.");
        self.encounter.opening = Some(Opening {
            pending,
            how: How::Fight,
        });
        Spent::Turn(self.price_of(player, ActionKind::Stairs))
    }
}
