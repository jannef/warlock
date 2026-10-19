//! Fighting (spec §8 "Combat"): attacks between actors, hit points, regeneration, and death.
//!
//! An attack ([`World::attack`]) is resolved by the rules in [`crate::combat`] with the
//! attacker's [`Profile`] (its weapon, natural attack, or bare hands in melee; its missile weapon
//! for a shot) against the target's [`Defence`](crate::combat::Defence), drawing from the
//! gameplay random stream. Spells stay intents: nothing is rolled or damaged. Every attack is an
//! [`Event::Attacked`] with what came of it, and a noise.
//!
//! Damage takes hit points; at 0 the target dies ([`Event::Died`]). A creature is removed from
//! the world and drops what it carried on its cell; the player's run ends ([`Death`]): every
//! action after that is refused, and the app shows the death screen and deletes the save.
//!
//! Hit points come back with game time, one per [`combat::regen_seconds`] of it, for every actor
//! on the current level ([`World::regenerate`], as the scheduler moves the clock).
//!
//! Messages (spec §6 "Attacks") never carry numbers: "The kobold hits you.", "…hits you hard!" for
//! a critical, "You block the kobold's spear with your shield.", "The giant rat dies." The
//! player's own attacks are said as they are made, a creature's when the turn is perceived (it is
//! said if the player saw either end of it, or was its target).

use super::{Actor, ActorId, World};
use crate::action::{AttackKind, Event};
use crate::combat::{self, Blow, Category, Profile, Strike};
use crate::content::{Content, CreatureId, ObjectId};
use crate::noise::Act;
use crate::object;
use crate::skills::Use;
use crate::stats::Stat;
use crate::time::GameTime;
use serde::{Deserialize, Serialize};

/// How the player's run ended (spec §6 "Death").
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Death {
    /// The turn it happened in.
    pub turn: u64,
    pub time: GameTime,
    /// Where: the name of the level.
    pub place: String,
    /// What killed them, if they saw it: a creature type. `None`: something unseen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<CreatureId>,
}

/// What an attack is made with, in words: what it is called ("spear", "bite", "fist", "arrow")
/// and what it does to its target ("hits", "bites", "shoots").
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Words {
    noun: String,
    verb: String,
}

impl Words {
    #[cfg(test)]
    pub(super) fn test(noun: &str, verb: &str) -> Self {
        Self {
            noun: noun.into(),
            verb: verb.into(),
        }
    }

    /// What `actor` attacks with, by `kind`; a shot or a throw is made `with` its missile.
    pub(super) fn of(
        actor: &Actor,
        kind: AttackKind,
        with: Option<ObjectId>,
        content: &Content,
    ) -> Self {
        let words = |noun: &str, verb: &str| Self {
            noun: noun.into(),
            verb: verb.into(),
        };
        match kind {
            AttackKind::Melee => {
                let natural = content.creature(actor.creature).attack.as_ref();
                match (actor.weapon(content), natural) {
                    (Some((weapon, _)), _) => words(&content.object(weapon).name, "hits"),
                    (None, Some(attack)) => words(&attack.name, &attack.verb),
                    (None, None) => words("fist", "hits"),
                }
            }
            AttackKind::Ranged => match with.or_else(|| actor.ammunition(content)) {
                Some(ammo) => words(&content.object(ammo).name, "shoots"),
                None => words("shot", "shoots"),
            },
            AttackKind::Thrown => match with {
                Some(thrown) => words(&content.object(thrown).name, "hits"),
                None => words("throw", "hits"),
            },
            AttackKind::Spell => words("spell", "casts a spell at"),
        }
    }
}

/// One end of an attack as the player perceives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum End<'a> {
    You,
    /// A creature the player sees, by its type's name.
    Seen(&'a str),
    /// Something the player does not see.
    Unseen,
}

impl End<'_> {
    /// As the subject: "You", "The kobold", "Something".
    fn subject(self) -> String {
        match self {
            End::You => "You".into(),
            End::Seen(name) => format!("The {name}"),
            End::Unseen => "Something".into(),
        }
    }

    /// As the object: "you", "the kobold", "something".
    fn object(self) -> String {
        match self {
            End::You => "you".into(),
            End::Seen(name) => format!("the {name}"),
            End::Unseen => "something".into(),
        }
    }

    /// Whose: "your", "the kobold's".
    fn whose(self) -> String {
        match self {
            End::You => "your".into(),
            End::Seen(name) => format!("the {name}'s"),
            End::Unseen => "something's".into(),
        }
    }

    /// Its own: "your", "its".
    fn own(self) -> &'static str {
        match self {
            End::You => "your",
            _ => "its",
        }
    }

    /// A verb in its person: `you` for the player ("dodge"), `other` for anyone else ("dodges").
    fn verb<'v>(self, you: &'v str, other: &'v str) -> &'v str {
        match self {
            End::You => you,
            _ => other,
        }
    }
}

/// The message for an attack: by `attacker` on `target`, made with `words`, and what came of it
/// (`None`: an intent only, a spell or a shot with no missile weapon). `hide` is the target's
/// natural armour, for a blow it turned. Why an attack missed is said only when the player saw
/// both ends; `None` if they saw neither and were not the target.
pub(super) fn attack_message(
    attacker: End<'_>,
    target: End<'_>,
    words: &Words,
    kind: AttackKind,
    strike: Option<Strike>,
    hide: Option<&str>,
) -> Option<String> {
    if attacker == End::Unseen && target == End::Unseen {
        return None;
    }
    let (a, t) = (attacker.subject(), target.object());
    let Some(strike) = strike else {
        let verb = match kind {
            AttackKind::Melee => attacker.verb("attack", "attacks"),
            AttackKind::Ranged => attacker.verb("shoot at", "shoots at"),
            AttackKind::Thrown => attacker.verb("throw at", "throws at"),
            AttackKind::Spell => attacker.verb("cast a spell at", "casts a spell at"),
        };
        return Some(format!("{a} {verb} {t}."));
    };
    let weapon = format!("{} {}", attacker.whose(), words.noun);
    let both_seen = attacker != End::Unseen && target != End::Unseen;
    let text = match strike {
        Strike::Hit { critical } => {
            let verb = match (attacker, kind) {
                (End::You, AttackKind::Ranged) => "shoot",
                (End::You, _) => "hit",
                _ => words.verb.as_str(),
            };
            let end = if critical { " hard!" } else { "." };
            format!("{a} {verb} {t}{end}")
        }
        Strike::Missed { by } => match by.filter(|_| both_seen) {
            None => format!("{a} {} {t}.", attacker.verb("miss", "misses")),
            Some(Category::Dexterity | Category::Dodge) => format!(
                "{} {} {weapon}.",
                target.subject(),
                target.verb("dodge", "dodges")
            ),
            Some(Category::Armour) => format!(
                "{} glances off {} armour.",
                capitalized(&weapon),
                target.whose()
            ),
            Some(Category::Shield) => format!(
                "{} {} {weapon} with {} shield.",
                target.subject(),
                target.verb("block", "blocks"),
                target.own()
            ),
            Some(Category::Parry) => format!(
                "{} {} {weapon}.",
                target.subject(),
                target.verb("parry", "parries")
            ),
            Some(Category::Natural) => format!(
                "{} glances off {} {}.",
                capitalized(&weapon),
                target.whose(),
                hide.unwrap_or("hide")
            ),
            Some(Category::Deflection) => format!(
                "{} turns aside before it reaches {t}.",
                capitalized(&weapon)
            ),
        },
    };
    Some(text)
}

/// The message for a death the player saw: "The giant rat dies.", or the player's own.
pub(super) fn death_message(who: End<'_>) -> String {
    match who {
        End::You => "You die...".into(),
        End::Seen(name) => format!("The {name} dies."),
        End::Unseen => "Something dies.".into(),
    }
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

impl World {
    /// How the player's run ended, once it has: no action is taken after that.
    pub fn death(&self) -> Option<&Death> {
        self.state.death.as_ref()
    }

    /// An actor alive, or one that died during the current action (for the words of what it
    /// did before it died).
    pub(super) fn actor_or_fallen(&self, id: ActorId) -> Option<&Actor> {
        (self.state.actors.get(&id)).or_else(|| self.fallen.iter().find(|a| a.id == id))
    }

    /// Actor `attacker` attacks actor `target`: a melee blow with what it wields (else its
    /// natural attack, else its bare hands), a shot with the missile weapon it has ready (with
    /// none, an intent only), or a spell (an intent only, until magic is designed). Returns the
    /// blow, if one was struck.
    pub(super) fn attack(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let me = &self.state.actors[&attacker];
        let profile = match kind {
            AttackKind::Melee => Some(me.melee(&self.content)),
            AttackKind::Ranged => me.ranged(&self.content),
            AttackKind::Thrown | AttackKind::Spell => None,
        };
        self.attack_with(attacker, target, kind, profile, events)
    }

    /// The Defence of actor `id`, its burden capping its Dexterity (spec §8 "Burden").
    pub(crate) fn defence_of(&self, id: ActorId) -> combat::Defence {
        let actor = &self.state.actors[&id];
        actor.defence(
            &self.content,
            self.state.load(&self.content, actor).penalty(),
        )
    }

    /// Actor `attacker` attacks actor `target` with `profile` (`None`: an intent only, nothing is
    /// rolled), against the target's Defence: the event, the noise, and the damage, which may
    /// kill. A missile's attacks ([`World::missile_hits`], `shoot.rs`) make their noise once, as
    /// it is loosed.
    pub(crate) fn attack_with(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        profile: Option<Profile>,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        self.resolve(attacker, target, (kind, None), profile, true, events)
    }

    /// A missile `with` loosed by `attacker` reaches actor `target`, attacking it with `profile`
    /// (a shot or a throw, `kind`): the event and the damage, which may kill; no noise (the
    /// missile made it as it was loosed).
    pub(super) fn missile_hits(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        (kind, with): (AttackKind, ObjectId),
        profile: Profile,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        self.resolve(
            attacker,
            target,
            (kind, Some(with)),
            Some(profile),
            false,
            events,
        )
    }

    /// Rolls an attack of `kind` made `with` a missile (or not) and applies it, with its noise
    /// if `noisy`.
    fn resolve(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        (kind, with): (AttackKind, Option<ObjectId>),
        profile: Option<Profile>,
        noisy: bool,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let (Some(from), Some(to)) = (
            self.state.actors.get(&attacker).map(|a| a.pos),
            self.state.actors.get(&target).map(|a| a.pos),
        ) else {
            return None;
        };
        let defence = self.defence_of(target);
        let blow = profile.map(|profile| combat::strike(&profile, &defence, &mut self.state.rng));
        events.push(Event::Attacked {
            attacker,
            target,
            kind,
            from,
            to,
            strike: blow.map(|b| b.strike),
            with,
        });
        if noisy {
            self.make_noise(attacker, Act::Attack(kind));
        }
        // An attack the player dodged (Dexterity or Dodge made it miss) trains tumbling.
        let dodged = blow.is_some_and(|b| {
            matches!(
                b.strike,
                Strike::Missed {
                    by: Some(Category::Dexterity | Category::Dodge)
                }
            )
        });
        if dodged && target == self.state.player && self.trains(Use::Dodge) {
            self.train(Use::Dodge);
        }
        if let Some(blow) = blow
            && blow.damage > 0
        {
            self.hurt(target, blow.damage, attacker, events);
        }
        blow
    }

    /// Actor `id` loses `damage` hit points to `by`; at 0 it dies.
    fn hurt(&mut self, id: ActorId, damage: i32, by: ActorId, events: &mut Vec<Event>) {
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        actor.hp -= damage;
        if actor.hp > 0 {
            return;
        }
        let (at, creature) = (actor.pos, actor.creature);
        events.push(Event::Died { actor: id, at });
        if id == self.state.player {
            let by = self.actor_or_fallen(by).map(|a| a.creature);
            self.state.death = Some(Death {
                turn: self.state.turn,
                time: self.state.time,
                place: self.state.current().name.clone(),
                by,
            });
        } else {
            if by == self.state.player {
                self.earn_kill(creature);
                let level = self.state.current_level.clone();
                self.tally_kill(creature, &level);
            }
            self.creature_dies(id);
        }
    }

    /// Creature `id` dies: it leaves the world, and what it carried (worn and wielded things
    /// too) falls on its cell, on top of what lies there, in the order it carried them.
    fn creature_dies(&mut self, id: ActorId) {
        let Some(actor) = self.state.actors.remove(&id) else {
            return;
        };
        if !actor.inventory.is_empty()
            && let Some(level) = self.state.levels.get_mut(&actor.level)
        {
            let mut pile = level.objects_at(actor.pos).to_vec();
            object::add_to_pile(&mut pile, &actor.inventory, &self.content);
            level.set_pile(actor.pos, pile);
        }
        self.fallen.push(actor);
    }

    /// `seconds` of game time pass for actors `ids` (on the current level, alive): each hurt one
    /// gets a hit point back for every [`combat::regen_seconds`] of its Toughness, the seconds
    /// left over counting toward the next.
    pub(super) fn regenerate(&mut self, ids: impl Iterator<Item = ActorId>, seconds: u64) {
        for id in ids {
            let Some(actor) = self.state.actors.get_mut(&id) else {
                continue;
            };
            let max = actor.max_hp();
            if actor.hp <= 0 || actor.hp >= max {
                actor.regen = 0;
                continue;
            }
            let every = u64::from(combat::regen_seconds(actor.stats.score(Stat::Toughness)));
            let banked = u64::from(actor.regen).saturating_add(seconds);
            let gained = i32::try_from(banked / every).unwrap_or(i32::MAX);
            actor.hp = actor.hp.saturating_add(gained).min(max);
            actor.regen = if actor.hp >= max {
                0
            } else {
                u32::try_from(banked % every).unwrap_or(0)
            };
        }
    }

    /// The player attacks `target`, which they `seen` or not, in melee: the message is said at
    /// once, by what the player knows as they strike; events reach presentation only if they
    /// saw the target.
    pub(super) fn player_attacks(&mut self, target: ActorId, seen: bool, events: &mut Vec<Event>) {
        let player = self.state.player;
        let mut made = Vec::new();
        self.attack(player, target, AttackKind::Melee, &mut made);
        let content = self.content.clone();
        let words = Words::of(self.state.player(), AttackKind::Melee, None, &content);
        let target_actor = self.actor_or_fallen(target);
        let name = target_actor.map(|a| content.creature(a.creature).name.as_str());
        let hide = (target_actor)
            .and_then(|a| content.creature(a.creature).natural.as_ref())
            .map(|h| h.name.as_str());
        let end = match name {
            Some(name) if seen => End::Seen(name),
            _ => End::Unseen,
        };
        let strike = made.iter().find_map(|e| match e {
            Event::Attacked { strike, .. } => *strike,
            _ => None,
        });
        let died = made.iter().any(|e| matches!(e, Event::Died { .. }));
        let mut said = Vec::new();
        said.extend(attack_message(
            End::You,
            end,
            &words,
            AttackKind::Melee,
            strike,
            hide,
        ));
        if died && seen {
            said.push(death_message(end));
        }
        for text in said {
            self.say(text);
        }
        self.say_news();
        if seen {
            events.extend(made);
        }
    }
}
