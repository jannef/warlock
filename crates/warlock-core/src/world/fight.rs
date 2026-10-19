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
//! A blow that hits and leaves its target alive may cause status effects (bleeding from a big
//! hit or a critical, poison from a rat's bite; `status.rs`, [`World::after_blow`]). Hit points
//! come back with game time, one per [`combat::regen_seconds`] of it at the normal pace, for
//! every actor on the current level (`status.rs`, as the scheduler moves the clock), at the rate
//! [`World::healing_rate`] decides (three times as fast in a sanctuary).
//!
//! Messages (spec §6 "Attacks") never carry numbers: "The kobold hits you.", "…hits you hard!" for
//! a critical, "You block the kobold's spear with your shield.", "The giant rat dies." The
//! player's own attacks are said as they are made, a creature's when the turn is perceived (it is
//! said if the player saw either end of it, or was its target).
//!
//! Weapon skills (spec §8 "Weapon skills"): every attack draws on a skill ([`Actor::attack_skill`];
//! its profile and price carry what the rank gives), and each attack the player makes earns
//! marks in it, more for a hit, more against a stronger opponent (by level, spec §8
//! "Experience"); a blow the player's shield blocks earns marks in shield blocking
//! ([`World::practise`]). A rank gained is said after the attack that earned it: "You feel more
//! skilled with daggers." A shield bash ([`World::bash`]) is a blow with a held shield, its own
//! damage, and its own skill.

use super::{Actor, ActorId, Spent, World};
use crate::action::{AttackKind, Event};
use crate::combat::{self, Blow, Category, Profile, Strike};
use crate::content::{
    ActionKind, Content, CreatureId, ObjectId, Role, Stance, StatusId, WeaponSkillId,
};
use crate::geom::Dir;
use crate::noise::Act;
use crate::object;
use crate::skills::Use;
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
    /// What killed them, if they saw it: a creature type. `None`: something unseen, or a status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<CreatureId>,
    /// The status effect that killed them, if one did (they bled to death).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub of: Option<StatusId>,
}

/// What an attack is made with, in words: what it is called ("spear", "bite", "fist", "arrow",
/// "small shield"), what it does to its target ("hits", "bites", "shoots", "bashes"), and what
/// the player does with it ("hit", "bash").
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Words {
    noun: String,
    verb: String,
    you: &'static str,
}

impl Words {
    #[cfg(test)]
    pub(super) fn test(noun: &str, verb: &str) -> Self {
        Self {
            noun: noun.into(),
            verb: verb.into(),
            you: if verb == "bashes" { "bash" } else { "hit" },
        }
    }

    /// What `actor` attacks with, by `kind`; a shot or a throw is made `with` its missile, a
    /// bash with a shield.
    pub(super) fn of(
        actor: &Actor,
        kind: AttackKind,
        with: Option<ObjectId>,
        content: &Content,
    ) -> Self {
        let words = |noun: &str, verb: &str| Self {
            noun: noun.into(),
            verb: verb.into(),
            you: "hit",
        };
        match kind {
            AttackKind::Melee if let Some(shield) = with => Self {
                you: "bash",
                ..words(&content.object(shield).name, "bashes")
            },
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
                (End::You, _) => words.you,
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
        let skill = (self.state.actors.get(&attacker))
            .and_then(|a| a.attack_skill(&self.content, kind, with));
        let levels = (self.level_of(attacker), self.level_of(target));
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
        // A shopkeeper the player attacks turns on them (spec §7 "Shops").
        if attacker == self.state.player {
            self.provoke(target);
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
        let player = self.state.player;
        if let Some(blow) = blow {
            if attacker == player
                && let Some(skill) = skill
            {
                self.practise(skill, blow.strike.is_hit(), levels.1);
            }
            let blocked = Strike::Missed {
                by: Some(Category::Shield),
            };
            let block = self.content.weapon_skills().of_role(Role::ShieldBlock);
            if target == player
                && blow.strike == blocked
                && let Some(block) = block
            {
                self.practise(block, true, levels.0);
            }
        }
        if let Some(blow) = blow
            && blow.damage > 0
        {
            self.bleed((attacker, target), from, kind, blow);
            let died = self.hurt(target, blow.damage, attacker, events);
            if !died {
                self.after_blow(attacker, target, kind, &blow, events);
            }
        }
        blow
    }

    /// The level of actor `id` as weapon marks weigh it (spec §8 "Weapon skills"): a
    /// creature's type's (spec §8 "Experience"), the player's own.
    fn level_of(&self, id: ActorId) -> u32 {
        if id == self.state.player {
            return u32::from(self.player_level());
        }
        let creature = self.actor_or_fallen(id).map(|a| a.creature);
        creature.map_or(1, |c| u32::from(self.content.creature(c).level))
    }

    /// The player used weapon skill `skill` against an opponent of `level`: a `success` (a hit,
    /// a blow blocked) or not. It earns its marks ([`crate::content::MarkRules`]), and each skill
    /// a rank gained raised (with the skills it defaults to) waits in `news` to be said after the
    /// attack's own messages: "You feel more skilled with long blades." No numbers.
    fn practise(&mut self, skill: WeaponSkillId, success: bool, level: u32) {
        let content = self.content.clone();
        let table = content.weapon_skills();
        let Some(rules) = table.marks() else {
            return;
        };
        let me = self.player_mut();
        let marks = rules.earned(success, level, me.weapon_skills.rank(skill));
        if marks > 0 {
            for id in me.weapon_skills.train(table, skill, marks) {
                let phrase = &table.skill(id).phrase;
                self.news.push(format!("You feel more skilled {phrase}."));
            }
        }
    }

    /// Actor `id` loses `damage` hit points to `by`; at 0 it dies. Returns whether it died.
    fn hurt(&mut self, id: ActorId, damage: i32, by: ActorId, events: &mut Vec<Event>) -> bool {
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return false;
        };
        actor.hp -= damage;
        if actor.hp > 0 {
            return false;
        }
        self.dies(id, Some(by), None, self.state.time, events);
        true
    }

    /// Actor `id` dies at moment `when`: killed by actor `by` (if anyone), of status `of` (if
    /// one did it). A creature leaves the world; the player's run ends. A creature killed by the
    /// player earns them its experience (spec §8 "Experience"), and so does one that dies of a
    /// status effect the player caused (`by`: a wound they dealt bleeds it to death).
    pub(super) fn dies(
        &mut self,
        id: ActorId,
        by: Option<ActorId>,
        of: Option<StatusId>,
        when: GameTime,
        events: &mut Vec<Event>,
    ) {
        let Some((at, creature)) = self.state.actors.get(&id).map(|a| (a.pos, a.creature)) else {
            return;
        };
        events.push(Event::Died { actor: id, at, of });
        if id == self.state.player {
            // A blow's dealer, if the player saw it (settled as the turn is perceived); a status
            // names itself instead.
            let by = (by.filter(|_| of.is_none()))
                .and_then(|b| self.actor_or_fallen(b))
                .map(|a| a.creature);
            self.state.death = Some(Death {
                turn: self.state.turn,
                time: when,
                place: self.state.current().name.clone(),
                by,
                of,
            });
        } else {
            if by == Some(self.state.player) {
                self.earn_kill(creature);
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
        // A dead shopkeeper's goods are free to take (spec §7 "Shops").
        self.keeper_died(id, &actor.level);
        self.fallen.push(actor);
    }

    /// The player attacks `target`, which they `seen` or not, in melee: the message is said at
    /// once, by what the player knows as they strike; events reach presentation only if they
    /// saw the target.
    pub(super) fn player_attacks(&mut self, target: ActorId, seen: bool, events: &mut Vec<Event>) {
        let profile = self.state.player().melee(&self.content);
        self.player_strikes(target, seen, (None, profile), events);
    }

    /// The player bashes whoever stands in `dir` with the shield they hold (spec §8 "Weapon
    /// skills"): a blow like any other ([`World::player_attacks`]), with the shield's `bash`
    /// dice and shield bashing's benefits, for the price of `melee`. Refused (no time) without a
    /// shield to bash with, with nobody there, and against someone not hostile.
    pub(super) fn bash(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some((shield, profile)) = self.state.player().bash(&content) else {
            self.say("You have no shield to bash with.");
            return Spent::Nothing;
        };
        let to = self.player_pos().step(dir);
        let level = &self.state.current().id;
        let Some((target, creature)) = (self.state.actor_at(level, to)).map(|a| (a.id, a.creature))
        else {
            self.say("There is nothing there to bash.");
            return Spent::Nothing;
        };
        let seen = self.perception.sees_actors_at(to);
        if self.stance(player, target) != Stance::Hostile {
            self.say(if seen {
                let name = &content.creature(creature).name;
                format!("You have no quarrel with the {name}.")
            } else {
                "Something is in the way.".into()
            });
            return Spent::Nothing;
        }
        self.player_strikes(target, seen, (Some(shield), profile), events);
        let skill = content.weapon_skills().of_role(Role::ShieldBash);
        Spent::Turn(self.attack_price(player, ActionKind::Melee, skill))
    }

    /// The player strikes `target` in melee with `profile`, made `with` a shield (a bash) or
    /// not (a blow): see [`World::player_attacks`].
    fn player_strikes(
        &mut self,
        target: ActorId,
        seen: bool,
        (with, profile): (Option<ObjectId>, Profile),
        events: &mut Vec<Event>,
    ) {
        let player = self.state.player;
        let mut made = Vec::new();
        let kind = (AttackKind::Melee, with);
        self.resolve(player, target, kind, Some(profile), true, &mut made);
        let content = self.content.clone();
        let words = Words::of(self.state.player(), AttackKind::Melee, with, &content);
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
