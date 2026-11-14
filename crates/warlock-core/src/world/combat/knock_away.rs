//! The disarming blow (#1166; a creature gimmick in the style of docs/design/monster-dread.md):
//! a creature type with `disarms = { chance, difficulty }` (`content/creature.rs`: the drow
//! blademaster and captain, the death knight, the ultroloth, Wælcyrge the Chooser) may knock
//! the weapon from the player's main hand as it fights them. Enemy-only: no creature disarms
//! another, and the player's own Disarm is trap work (`disarm.rs`), which shares nothing with it.
//!
//! **When** ([`World::knock_away`], hooked in `fight.rs` after a blow's riders): a melee hit of
//! the creature's lands on the player and leaves them alive, and the player's main hand (the
//! first hand slot) holds a weapon (a thing with an attack: a shield alone is not knocked away).
//! Then, in order, each from the gameplay stream:
//! - the chance, one draw; missed, nothing more;
//! - the open cells beside the player, in row order: walkable, no deep water (`swim`), no closed
//!   door, no one standing there. With none, nothing happens and nothing more is drawn: the
//!   weapon stays in hand;
//! - the player's Dexterity check against the type's `difficulty`, two draws ([`check::check`]),
//!   with [`hold`]'s modifier; made, the weapon stays and nothing is said (the player only
//!   feels a blow);
//! - failed, the cell, one draw with two or more to pick from.
//!
//! **What**: the whole stack leaves the pack and the hands (every slot it filled: a two-handed
//! weapon leaves both) and lies on top of the pile on that cell, where the player picks it up
//! as anything else (a turn, and the step to it). A spirit weapon (`spirit_weapon.rs`) is not
//! let go: it fades, as one leaving the hand always does. No noise of its own: the fight's is
//! heard already.
//!
//! **Words**: "The drow blademaster knocks your scimitar from your hand!" (by the type's name,
//! as a seen creature is said; "Something" for one the player does not see), then where it went:
//! "It lands beside you." The equipment screen shows the hand empty.

use crate::action::{AttackKind, Positions};
use crate::check;
use crate::content::{Content, Grip};
use crate::geom::Pos;
use crate::object;
use crate::stats::Stat;
use crate::world::{ActorId, World};

/// The modifier `who` holds on to its main-hand weapon `weapon` with (module docs): its
/// Dexterity modifier and its talents' Dexterity checks, its rank in the weapon's skill, and
/// its grip: the `[knock_away]` numbers of `data/rules/tuning.toml` for a two-handed weapon and
/// with a shield, its
/// rank in the style of one weapon alone (Duelist) when the other hand is empty.
pub fn hold(content: &Content, who: &crate::world::Actor, weapon: crate::content::ObjectId) -> i32 {
    let dexterity = who.check_modifier(content, Stat::Dexterity);
    let skill = (content.object(weapon).skill).map_or(0, |s| i32::from(who.weapon_skills.rank(s)));
    let grip = match who.grip(content) {
        Some((Grip::TwoHanded, _)) => content.tuning().knock_away.two_handed,
        Some((Grip::WeaponAndShield, _)) => content.tuning().knock_away.shield,
        Some((Grip::OneWeapon, _)) => (who.style(content, AttackKind::Melee, None))
            .map_or(0, |style| i32::from(who.weapon_skills.rank(style))),
        _ => 0,
    };
    dexterity + skill + grip
}

impl World {
    /// The cells beside the player a knocked-away weapon may land on, in row order (module
    /// docs).
    fn knock_landings(&self) -> Vec<Pos> {
        let me = self.player_pos();
        let level = self.state.current();
        let content = &self.content;
        (-1..=1)
            .flat_map(|dy| (-1..=1).map(move |dx| Pos::new(me.x + dx, me.y + dy)))
            .filter(|&p| p != me && level.is_walkable(content, p))
            .filter(|&p| !level.is_closed_door(content, p))
            .filter(|&p| (level.terrain_at(p)).is_some_and(|t| !content.terrain(t).swim))
            .filter(|&p| (self.state.actor_at(&self.state.current_level, p)).is_none())
            .collect()
    }

    /// Creature `foe`'s melee blow hit `victim` and left them alive: it may knock their weapon
    /// away (module docs).
    pub(super) fn knock_away(&mut self, foe: ActorId, victim: ActorId, kind: AttackKind) {
        let player = self.state.player;
        if kind != AttackKind::Melee || victim != player || foe == player {
            return;
        }
        let content = self.content.clone();
        let Some(me) = self.state.actors.get(&foe) else {
            return;
        };
        let Some(blow) = content.creature(me.creature).disarms else {
            return;
        };
        let (pos, name) = (me.pos, content.creature(me.creature).name.clone());
        let Some(hand) = content.hand_slots().next() else {
            return;
        };
        let held = self.state.player().equipment.get(hand);
        let Some(item) = held.filter(|&i| {
            (self.state.player().inventory.get(i))
                .is_some_and(|s| content.object(s.object).attack.is_some())
        }) else {
            return;
        };
        if !self.state.rng.chance(u32::from(blow.chance)) {
            return;
        }
        let landings = self.knock_landings();
        if landings.is_empty() {
            return;
        }
        let weapon = self.state.player().inventory[item].object;
        let modifier = hold(&content, self.state.player(), weapon);
        if check::check(&mut self.state.rng, modifier, blow.difficulty) {
            return;
        }
        let at = match landings.len() {
            1 => landings[0],
            n => landings[self.state.rng.index(n)],
        };
        let who = if self.perception.sees_actors_at(pos) {
            crate::content::the_start(&name)
        } else {
            "Something".to_string()
        };
        let what = &content.object(weapon).name;
        self.say(format!("{who} knocks your {what} from your hand!"));
        if self.state.player().inventory[item].is_spirit() {
            // Spirit-stuff out of the hand fades (`spirit_weapon.rs`), and says so.
            self.player_mut().equipment.take_off(item);
            self.fade_spirit();
            return;
        }
        let me = self.player_mut();
        let stack = me.inventory.remove(item);
        me.equipment.after_removal(&Positions::of([item]));
        let level = self.state.current_mut();
        let mut pile = level.objects_at(at).to_vec();
        object::add_to_pile(&mut pile, &[stack], &content);
        level.set_pile(at, pile);
        self.say("It lands beside you.");
    }
}
