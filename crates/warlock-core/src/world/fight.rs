//! Fighting (spec §8 "Combat"): attacks between actors, hit points, regeneration, and death.
//!
//! An attack ([`World::attack`]) is resolved by the rules in [`crate::combat`] with the
//! attacker's [`Profile`] (its weapon, natural attack, or bare hands in melee; its missile weapon
//! for a shot) against the target's [`Defence`](crate::combat::Defence), drawing from the
//! gameplay random stream. Spells stay intents: nothing is rolled or damaged. Every attack is an
//! [`Event::Attacked`] with what came of it, and a noise.
//!
//! Protection (spec §8 "Combat"): what the target wears and its hide soak damage off every hit
//! ([`World::protection_of`], less the attack's penetration). A hit soaked to nothing
//! ([`Strike::Soaked`]) glances off ("Your sword glances off the cave beetle's shell.") and does
//! nothing else, no blood and no wound, but it landed: it trains as a hit does.
//!
//! Damage types (spec §8 "Damage types"): every blow is of a kind (a sword slashes or pierces,
//! whichever is better against its target; bare hands and bashes bludgeon), said with the
//! kind's verb ("The kobold stabs you.", "You cleave the giant rat."); Protection soaks only
//! the physical kinds, and the target's resistances ([`World::resistances_of`]) take their
//! share off each kind after it. A hit resisted to nothing ([`Strike::Resisted`]) does no harm
//! ("Your torch does the salamander no harm."): the only hint of a creature's resistances.
//! [`World::target_of`] gathers the three; [`World::resisted`] is for harm that is not a blow.
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
//!
//! Fighting styles (spec §8 "Fighting styles"): every attack is also fought in a style
//! ([`Actor::style`]) whose rank adds to the weapon skill's, and the player's attacks earn marks
//! in it too. A melee attack is a round of blows ([`World::second_blow`]): with a weapon in each
//! hand the off hand strikes after the main hand, both at a penalty; with a weapon and a shield
//! a blow that hits may be followed by a free shield bash, by the style's chance. Each blow is
//! resolved on its own: its weapon's skill and affixes, its blood, and the bleeding of a big hit.
//! A kick is made whatever the hands hold: it draws on unarmed fighting and no style.
//!
//! Free blows (spec §8 "Free blows", `free_blow.rs`): the end of a melee attack may bring a
//! cleave (it killed) or the target's riposte (its Parry made a blow miss), and a step away from
//! beside an opportunist its blow; [`World::second_blow`] decides each.

use super::free_blow::{self, Cue, Free, FreeBlow};
use super::{Actor, ActorId, AttackSkills, Spent, World, WorldState};
use crate::action::{AttackKind, Event};
use crate::combat::{self, Blow, Category, Profile, Strike, Target};
use crate::content::{
    ActionKind, Build, Content, CreatureId, DamageTypeId, Grip, ObjectDef, ObjectId, Role, Stance,
    StatusId, TrapId, WeaponSkillId, article,
};
use crate::damage::{self, Damage, Protection, Resistances};
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
    /// What killed them, if they saw it: a creature type. `None`: something unseen, a status,
    /// or no creature at all (see `cause`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<CreatureId>,
    /// The status effect that killed them, if one did (they bled to death).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub of: Option<StatusId>,
    /// What killed them when it was neither a creature's attack nor a status effect: starving,
    /// or choking on a meal (spec §8 "Hunger"), or a trap (spec §8 "Traps").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<Cause>,
}

/// A death that was neither a creature's doing nor a status effect's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    /// They starved to death.
    Starvation,
    /// They choked to death on this, overeating.
    Choked(ObjectId),
    /// A trap of this kind killed them.
    Trap(TrapId),
    /// The spell of a scroll of this kind they read caught them (spec §8 "Scrolls").
    Scroll(ObjectId),
    /// They fell through a collapsed floor (spec §7 "Special rooms").
    Fall,
    /// A spell of their own caught them (spec §8 "Class abilities").
    Spell(crate::content::SpellId),
    /// A forgotten altar they prayed at struck them down (spec §7 "Special rooms").
    Altar,
}

impl Cause {
    /// How the death screen says it: "Starved to death", "Choked to death on a ration",
    /// "Killed by a fireball trap", "Killed by their own scroll of fireball".
    pub fn epitaph(self, content: &Content) -> String {
        let what = |o: ObjectId| content.object(o).describe(1);
        match self {
            Cause::Starvation => "Starved to death".into(),
            Cause::Fall => "Fell through a collapsed floor".into(),
            Cause::Choked(o) => format!("Choked to death on {}", what(o)),
            Cause::Trap(t) => {
                let name = &content.trap(t).name;
                format!("Killed by {} {name}", article(name))
            }
            Cause::Scroll(o) => format!("Killed by their own {}", content.object(o).name),
            Cause::Spell(spell) => {
                let noun = (content.spell_of(spell))
                    .map_or_else(|| "spell".into(), |s| s.look.noun.clone());
                format!("Killed by their own {noun}")
            }
            Cause::Altar => "Struck down at a forgotten altar".into(),
        }
    }

    /// What the player is told as it happens: "You starve to death...". A trap's victim dies
    /// like any other ("You die...").
    pub fn message(self, content: &Content) -> String {
        let name = |o: ObjectId| content.object(o).name.clone();
        match self {
            Cause::Starvation => "You starve to death...".into(),
            Cause::Choked(o) => format!("You choke to death on the {}...", name(o)),
            Cause::Trap(_) | Cause::Scroll(_) | Cause::Spell(_) | Cause::Altar | Cause::Fall => {
                "You die...".into()
            }
        }
    }
}

/// What hurt an actor, and so what a death by it is put down to ([`World::hurt`], the one way
/// hit points go down).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Harm {
    /// Another actor's blow, shot, or kick: it killed, and a kill by the player earns them its
    /// experience (spec §8 "Experience").
    Actor(ActorId),
    /// A trap (spec §8 "Traps"): no one killed, and no one earns anything.
    Trap(TrapId),
    /// Barbed hide's barbs (spec §8 "Birth sign"): not a blow, and a death of them earns no one
    /// anything.
    Barbs,
    /// A status effect's ticks or a disease (spec §8 "Status effects", "Diseases"): a death is
    /// of status `of`, caused by `by` if anyone (a wound they dealt bled it to death), at the
    /// moment `when` of the time passing.
    Status {
        of: Option<StatusId>,
        by: Option<ActorId>,
        when: GameTime,
    },
    /// Starving at the moment `when` (spec §8 "Hunger").
    Starving(GameTime),
    /// A place (spec §7 "Special rooms": a forgotten altar's fire, its bolt from above): no one
    /// killed, and no one earns anything.
    Place,
    /// A fall through a collapsed floor (spec §7 "Special rooms"): no one killed, and no one
    /// earns anything.
    Fall,
    /// What never kills: it leaves the last hit point (the plunge into freezing water, spec §5
    /// "Ice"; a disease that does not kill).
    Spared,
}

/// What an attack is made with, in words: what it is called ("spear", "bite", "fist", "arrow",
/// "small shield", "kick"), what it does to its target ("hits", "bites", "shoots", "bashes",
/// "kicks"), and what the player does with it ("hit", "bash", "kick").
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Words {
    noun: String,
    verb: String,
    you: String,
}

impl Words {
    /// What hits ("arrow", "fireball") and what it does ("hits", "burns"); what the player does
    /// with it follows ("hit", "burn").
    pub(super) fn new(noun: &str, verb: &str) -> Self {
        Self {
            noun: noun.into(),
            verb: verb.into(),
            you: second_person(verb),
        }
    }

    #[cfg(test)]
    pub(super) fn test(noun: &str, verb: &str) -> Self {
        Self::new(noun, verb)
    }

    /// A kick (spec §8 "Doors"): "You kick the giant rat.", "The giant rat dodges your kick."
    pub(super) fn kick() -> Self {
        Self::new("kick", "kicks")
    }

    /// What a weapon's blow does (spec §8 "Damage types"): the weapon's own `verb` ("cleaves"),
    /// else the verb of `damage`, the kind it did ("slashes", "slash"), else "hits".
    fn blow(weapon: &ObjectDef, damage: Option<DamageTypeId>, content: &Content) -> Self {
        match (&weapon.verb, damage) {
            (Some(verb), _) => Self::new(&weapon.name, verb),
            (None, Some(kind)) => {
                let kind = content.damage_type(kind);
                Self {
                    noun: weapon.name.clone(),
                    verb: kind.verb.clone(),
                    you: kind.you.clone(),
                }
            }
            (None, None) => Self::new(&weapon.name, "hits"),
        }
    }

    /// What `actor` attacks with, by `kind`; a shot or a throw is made `with` its missile, a
    /// bash with a shield, an off-hand blow with the off hand's weapon; `damage` is the kind of
    /// damage the blow did, which a weapon's blow is said with.
    pub(super) fn of(
        actor: &Actor,
        kind: AttackKind,
        with: Option<ObjectId>,
        damage: Option<DamageTypeId>,
        content: &Content,
    ) -> Self {
        let words = |noun: &str, verb: &str| Self::new(noun, verb);
        match kind {
            AttackKind::Melee if let Some(with) = with => match content.object(with) {
                shield if shield.bash.is_some() => words(&shield.name, "bashes"),
                weapon => Self::blow(weapon, damage, content),
            },
            AttackKind::Melee => {
                let natural = content.creature(actor.creature).attack.as_ref();
                match (actor.weapon(content), natural) {
                    (Some((weapon, _)), _) => Self::blow(content.object(weapon), damage, content),
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
    /// A trap's missile, blades, or blast, by what hits ("arrow", "fireball"): it attacks by
    /// itself (spec §8 "Traps").
    Trap(&'a str),
}

impl End<'_> {
    /// As the subject: "You", "The kobold", "Something", "The arrow".
    fn subject(self) -> String {
        match self {
            End::You => "You".into(),
            End::Seen(name) | End::Trap(name) => crate::content::the_start(name),
            End::Unseen => "Something".into(),
        }
    }

    /// As the object: "you", "the kobold", "something".
    pub(super) fn object(self) -> String {
        match self {
            End::You => "you".into(),
            End::Seen(name) | End::Trap(name) => crate::content::the(name),
            End::Unseen => "something".into(),
        }
    }

    /// Whose: "your", "the kobold's"; a trap's own missile or blast is just "the" one.
    fn whose(self) -> String {
        match self {
            End::You => "your".into(),
            End::Seen(name) => format!("{}'s", crate::content::the(name)),
            End::Unseen => "something's".into(),
            End::Trap(_) => "the".into(),
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

/// A verb in the player's person: "slashes" → "slash", "bashes" → "bash", "pierces" →
/// "pierce", "hits" → "hit" (an `es` after a sibilant comes off, else the `s`); a verb that
/// does not end in `s` stays as it is.
pub(super) fn second_person(verb: &str) -> String {
    let (first, rest) = verb.split_once(' ').unwrap_or((verb, ""));
    let stem = match first.strip_suffix('s') {
        None => first.to_string(),
        Some(stem) => {
            let sibilant = ["sh", "ch", "ss", "x", "z"];
            match stem.strip_suffix('e') {
                Some(short) if sibilant.iter().any(|s| short.ends_with(s)) => short.to_string(),
                _ => stem.to_string(),
            }
        }
    };
    if rest.is_empty() {
        stem
    } else {
        format!("{stem} {rest}")
    }
}

/// What a target's armour is called in the messages of the blows it turns (spec §6 "Attacks").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Guard<'a> {
    /// Its natural armour, for a blow that missed because of it: "shell".
    pub(super) hide: Option<&'a str>,
    /// What took all of a blow its Protection soaked: the piece it wears with the most
    /// Protection, else its natural armour ([`Actor::soaked_by`]).
    pub(super) soak: Option<&'a str>,
}

impl<'a> Guard<'a> {
    /// The guard of `target` (alive, or fallen this action), if it is known, in `state` (the
    /// player's race may give them a hide).
    pub(super) fn of(target: Option<&Actor>, content: &'a Content, state: &WorldState) -> Self {
        let Some(target) = target else {
            return Self::default();
        };
        let race = state.race_of(content, target);
        Self {
            hide: target.hide(content, race).map(|h| h.name.as_str()),
            soak: target.soaked_by(content, race),
        }
    }
}

/// The message for an attack: by `attacker` on `target`, made with `words`, and what came of it
/// (`None`: an intent only, a spell or a shot with no missile weapon). `guard` names the
/// target's armour, for a blow it turned or soaked. Why an attack missed, and what a blow
/// soaked to nothing glanced off, is said only when the player saw both ends; `None` if they
/// saw neither and were not the target.
pub(super) fn attack_message(
    attacker: End<'_>,
    target: End<'_>,
    words: &Words,
    kind: AttackKind,
    strike: Option<Strike>,
    guard: Guard<'_>,
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
    let hit = match (attacker, kind) {
        (End::You, AttackKind::Ranged) => "shoot",
        (End::You, _) => words.you.as_str(),
        _ => words.verb.as_str(),
    };
    let text = match strike {
        Strike::Hit { critical } => {
            let end = if critical { " hard!" } else { "." };
            format!("{a} {hit} {t}{end}")
        }
        Strike::Soaked if both_seen => format!(
            "{} glances off {} {}.",
            capitalized(&weapon),
            target.whose(),
            guard.soak.unwrap_or("armour")
        ),
        Strike::Resisted if both_seen => format!("{} does {t} no harm.", capitalized(&weapon)),
        Strike::Soaked | Strike::Resisted => {
            format!("{a} {hit} {t} but {} no harm.", attacker.verb("do", "does"))
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
                guard.hide.unwrap_or("hide")
            ),
            Some(Category::Deflection) => format!(
                "{} turns aside before it reaches {t}.",
                capitalized(&weapon)
            ),
            Some(Category::Wall) => format!(
                "{} strikes stone where {} {} in the wall.",
                capitalized(&weapon),
                target.object(),
                target.verb("hide", "hides")
            ),
        },
    };
    Some(text)
}

/// The message for a death the player saw: "The giant rat dies." (a creature's by the verb of its
/// type, `dies`: "The animated chair splinters."), or the player's own.
pub(super) fn death_message(who: End<'_>, dies: &str) -> String {
    match who {
        End::You => "You die...".into(),
        End::Seen(name) => format!("{} {dies}.", crate::content::the_start(name)),
        End::Unseen | End::Trap(_) => "Something dies.".into(),
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
    /// natural attack, else its bare hands) and the blows that may follow it
    /// ([`World::second_blow`]), a shot with the missile weapon it has ready (with none, an
    /// intent only), or a spell (an intent only, until magic is designed). Returns the (first)
    /// blow, if one was struck.
    pub(super) fn attack(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let me = &self.state.actors[&attacker];
        // With a shield in each hand a blow is a bash (`two_shields.rs`).
        let opening = (kind == AttackKind::Melee)
            .then(|| me.opening_bash(&self.content))
            .flatten();
        let profile = match kind {
            AttackKind::Melee => Some(me.melee(&self.content)),
            AttackKind::Ranged => me.ranged(&self.content),
            AttackKind::Thrown | AttackKind::Spell => None,
        };
        let blow = match opening {
            Some((shield, profile)) => {
                let skills = self.skills_of(attacker, kind, Some(shield));
                let made = (kind, Some(shield), skills);
                self.resolve(attacker, target, made, Some(profile), true, events)
            }
            None => self.attack_with(attacker, target, kind, profile, events),
        };
        if kind == AttackKind::Melee {
            let mut parried = free_blow::parried(blow);
            if let Some(second) = self.second_blow(attacker, target, Cue::First(blow), events) {
                let with = second.with;
                let struck = self.strike_free(second, events);
                parried |= free_blow::parried(struck);
                self.both_hands(attacker, target, (blow, struck), with, events);
            }
            self.after_attack(attacker, target, parried, events);
        }
        blow
    }

    /// A blow more, by actor `striker` at `target`, on `cue` (spec §8 "Fighting styles", "Free
    /// blows"), if any: who strikes whom, and with what. The one place every blow beyond an
    /// attack's first is decided (`free_blow.rs`):
    /// - [`Cue::First`]: the second blow of `striker`'s melee attack on `target` (below);
    /// - [`Cue::Attacked`]: its end: a cleave if it killed `target`, else `target`'s riposte;
    /// - [`Cue::Cleaved`]: another cleave, with great cleave, after one that killed `target`;
    /// - [`Cue::Leaves`]: `target` steps away from beside `striker`: the opportunist's blow;
    /// - [`Cue::Approaches`]: `target` stepped up beside `striker`: the gatekeeper's blow.
    ///
    /// Only the second blow may draw while deciding, and only the flurry spends (its ki); the
    /// free blows do neither.
    pub(super) fn second_blow(
        &mut self,
        striker: ActorId,
        target: ActorId,
        cue: Cue,
        events: &mut Vec<Event>,
    ) -> Option<FreeBlow> {
        match cue {
            Cue::First(first) => (self.off_hand_or_bash(striker, target, first))
                .map(|(with, profile)| FreeBlow {
                    by: striker,
                    at: target,
                    kind: Free::Second,
                    with: Some(with),
                    profile,
                })
                .or_else(|| self.flurry(striker, target, events)),
            Cue::Attacked { parried } => {
                // A parry fires the target's weapon abilities (Staggering block) as the attack
                // ends, before the free blows it may bring.
                let stand =
                    |w: &Self, id| w.state.death.is_none() && w.state.actors.contains_key(&id);
                if parried && stand(self, striker) && stand(self, target) {
                    let fired = super::weapon_ability::Fired {
                        other: striker,
                        kind: AttackKind::Melee,
                        skills: self.held_skills(target),
                        blow: None,
                    };
                    self.weapon_trigger(target, crate::content::Trigger::Parried, fired, events);
                }
                (self.cleave(striker, target, false))
                    .or_else(|| self.riposte(striker, target, parried))
            }
            Cue::Cleaved => self.cleave(striker, target, true),
            Cue::Leaves => self.opportunist(striker, target),
            Cue::Approaches => self.gatekeeper(striker, target),
        }
    }

    /// The blow after the first of actor `attacker`'s melee attack on `target` (spec §8
    /// "Fighting styles"), if any, and what it is made with: with a weapon in each hand the off
    /// hand's ([`Actor::off_hand_blow`]); with a weapon and a shield, after a `first` that hit, a
    /// shield bash by the style's `bash` chance (one draw from the gameplay stream, only when
    /// the chance is above 0). None once the target is gone.
    fn off_hand_or_bash(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        first: Option<Blow>,
    ) -> Option<(ObjectId, Profile)> {
        if !self.state.actors.contains_key(&target) {
            return None;
        }
        let content = self.content.clone();
        let me = self.state.actors.get(&attacker)?;
        if let Some(off) = me.off_hand_blow(&content) {
            return Some(off);
        }
        let hit = first.is_some_and(|b| b.strike.is_hit());
        let grip = me.grip(&content)?.0;
        if grip == Grip::TwoShields {
            return self.other_shield(attacker, hit);
        }
        if grip != Grip::WeaponAndShield {
            return None;
        }
        let (shield, profile) = me.bash(&content)?;
        let chance = me.attack_benefit(&content, AttackKind::Melee, None).bash;
        (hit && chance > 0 && self.state.rng.chance(u32::from(chance))).then_some((shield, profile))
    }

    /// The Defence of actor `id`, its burden capping its Dexterity (spec §8 "Burden"), and the
    /// wall around an incorporeal creature inside one adding its Wall bonus
    /// (`incorporeal.rs`).
    pub(crate) fn defence_of(&self, id: ActorId) -> combat::Defence {
        let actor = &self.state.actors[&id];
        let defence = actor.defence(
            &self.content,
            self.state.load(&self.content, actor).penalty(),
            self.state.race_of(&self.content, actor),
        );
        self.wall_defence(id, defence)
    }

    /// The Protection of actor `id` (spec §8 "Combat"): what it wears and its hide soak off
    /// every physical hit, by kind; none for one that is gone.
    pub(crate) fn protection_of(&self, id: ActorId) -> Protection {
        (self.state.actors.get(&id)).map_or_else(
            || Protection::plain(0),
            |a| a.protection(&self.content, self.state.race_of(&self.content, a)),
        )
    }

    /// What actor `id` resists (spec §8 "Damage types"): its creature type's, the player's
    /// race's, what it wears and wields, and its talents' (Elemental ward); nothing for one that
    /// is gone.
    pub(crate) fn resistances_of(&self, id: ActorId) -> Resistances {
        (self.state.actors.get(&id)).map_or_else(Resistances::default, |a| {
            a.resistances(&self.content, self.state.race_of(&self.content, a))
        })
    }

    /// The side of actor `id` a blow strikes: its Defence, Protection, and resistances, and
    /// whether it is immune to criticals, having no vitals (`Content::has_vitals`; an attacker's
    /// Anatomist may find them, `ambush.rs::vitals`).
    pub(crate) fn target_of(&self, id: ActorId) -> Target {
        let creature = self.state.actors.get(&id).map(|a| a.creature);
        Target {
            defence: self.defence_of(id),
            protection: self.protection_of(id),
            resistances: self.resistances_of(id),
            no_criticals: creature.is_some_and(|c| !self.content.has_vitals(c, &[])),
        }
    }

    /// The side of actor `id` an attack strikes, a `missile` (a shot, a throw, a trap's) or
    /// not: Deflect missiles (spec §8 "Talents") adds its Dodge against missiles, and Parry
    /// counts against blows only, but for one with Measure (a weapon ability).
    pub(crate) fn target_against(&self, id: ActorId, missile: bool) -> Target {
        let mut target = self.target_of(id);
        let deflects =
            (self.state.actors.get(&id)).map_or(0, |a| a.talent_mods(&self.content).missile_dodge);
        if missile && deflects != 0 {
            target.defence = target.defence.plus(Category::Dodge, deflects);
        }
        // A blade parries blows, not missiles, but for one that has learned the distance
        // (Measure, a weapon ability: `weapon_ability.rs`).
        let parry = target.defence.bonus(Category::Parry);
        let measured = (self.state.actors.get(&id))
            .is_some_and(|a| parry > 0 && self.ability_switches(a).parry_missiles);
        if missile && parry > 0 && !measured {
            target.defence = target.defence.plus(Category::Parry, -parry);
        }
        target
    }

    /// What actor `id` takes of `damage` that is no blow (a blast, a bolt, a fall, the plunge
    /// into freezing water): the amount less what it resists of the kind
    /// ([`damage::resist`]); untyped harm in full.
    pub(crate) fn resisted(&self, id: ActorId, damage: Damage) -> i32 {
        let percent = self.resistances_of(id).get(damage.kind);
        damage::resist(damage.amount, percent)
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
        let skills = self.skills_of(attacker, kind, None);
        self.resolve(
            attacker,
            target,
            (kind, None, skills),
            profile,
            true,
            events,
        )
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
        let skills = self.skills_of(attacker, kind, Some(with));
        let kind = (kind, Some(with), skills);
        self.resolve(attacker, target, kind, Some(profile), false, events)
    }

    /// The weapon skill and fighting style actor `id`'s attack of `kind` made `with` a missile,
    /// a shield, or the off hand's weapon (or not) draws on ([`Actor::attack_skills`]).
    pub(super) fn skills_of(
        &self,
        id: ActorId,
        kind: AttackKind,
        with: Option<ObjectId>,
    ) -> AttackSkills {
        (self.state.actors.get(&id))
            .map(|a| a.attack_skills(&self.content, kind, with))
            .unwrap_or_default()
    }

    /// Rolls an attack of `kind` made `with` a missile, a shield, or the off hand's weapon (or
    /// not), drawing on weapon skill and fighting style `skills` (the player's earn their
    /// marks), and applies it, with its noise if `noisy`.
    pub(super) fn resolve(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        (kind, with, skills): (AttackKind, Option<ObjectId>, AttackSkills),
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
        let profile = profile.map(|p| self.glared(attacker, p));
        // Relentless (spec §8 "Birth sign"): a streak of blows at one creature.
        let profile = profile.map(|p| self.relentless(attacker, target, kind, p));
        // Favoured prey and a sneak attack (spec §8 "Class abilities").
        let profile = profile.map(|p| self.favoured(attacker, target, p));
        // Unbinder (spec §8 "Animated objects"): against a creature of a tag.
        let profile = profile.map(|p| self.bane(attacker, target, p));
        // Caught unawares (`surprise.rs`): the initiative check, before the strike's draws.
        let missile = matches!(kind, AttackKind::Ranged | AttackKind::Thrown);
        // A lurk's blow finds a creature not hunting the player unawares; a low blow is a sneak
        // attack whatever the target knows (`thief.rs`).
        let unaware =
            profile.is_some() && (self.unaware(target, attacker) || self.lurked(attacker, target));
        let low = profile.is_some() && self.low_blow_taken(attacker);
        let unseen = profile.is_some() && self.sneaks(attacker, target, unaware || low, missile);
        if unaware {
            self.startle(target, Some(attacker), events);
        }
        self.attacked_near(attacker, target);
        // A blow at a disguised mimic finds it out (`mimic.rs`), after the sneak attack is
        // decided: it was unaware.
        if self.disguised(target) {
            self.reveal_mimic(target, super::mimic::Reveal::Struck, events);
        }
        let profile = profile.map(|p| self.sneak_threat(attacker, unseen, p));
        let struck = self.target_against(target, missile);
        let struck = self.vitals(attacker, target, struck);
        let struck = self.flat_footed(target, unaware, struck);
        let struck = self.small_target(attacker, target, struck);
        // Feint (a weapon active, `weapon_active.rs`): rolled without the categories it ignores.
        let struck = self.feinted(attacker, struck);
        let levels = (self.level_of(attacker), self.level_of(target));
        // A talent may roll a fumble again (a birth sign's Winner's luck).
        let lucky = (self.state.actors.get(&attacker))
            .is_some_and(|a| !a.talents.is_empty() && a.talent_mods(&self.content).reroll_fumble);
        // What a hit causes besides its damage (a unique weapon's Inflict power).
        let made = profile.as_ref().map_or_else(Vec::new, |p| p.on_hit.clone());
        let rng = &mut self.state.rng;
        let blow = profile.map(|profile| combat::strike_with(&profile, &struck, rng, lucky));
        let blow = blow.map(|b| self.ambushed(attacker, unseen, b));
        let blow = blow.map(|b| self.lore_blow(attacker, target, b));
        events.push(Event::Attacked {
            attacker,
            target,
            kind,
            from,
            to,
            strike: blow.map(|b| b.strike),
            with,
            damage: blow.and_then(|b| b.kind),
        });
        if noisy {
            self.make_noise(attacker, Act::Attack(kind));
        }
        // A shopkeeper the player attacks turns on them (spec §7 "Shops").
        if attacker == self.state.player {
            self.provoke(target);
            self.provoke_court(target);
            self.break_empathy(target);
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
        if let Some(blow) = &blow {
            self.train_fight((attacker, target), kind, blow);
        }
        let player = self.state.player;
        if let Some(blow) = blow {
            if attacker == player {
                // The weapon skill first, then the style: their news is said in that order.
                for skill in skills.trained() {
                    self.practise(skill, blow.strike.is_hit(), levels.1);
                }
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
        // Resources built by blows (spec §8 "Resources"): a hit and a critical build the
        // attacker's, being hit the target's.
        if let Some(Strike::Hit { critical }) = blow.map(|b| b.strike) {
            self.gain_resource(attacker, Build::Hit);
            if critical {
                self.gain_resource(attacker, Build::Critical);
            }
            self.gain_resource(target, Build::Struck);
        }
        // Weapon abilities (`weapon_ability.rs`) fire on the same events, and on a melee
        // attack's miss by the target's Shield, Dexterity, or Dodge.
        self.resolve_triggers((attacker, target), (kind, skills), blow, events);
        self.sneak_trigger((attacker, target), (kind, skills), (unseen, blow), events);
        if let Some(blow) = blow
            && blow.damage > 0
        {
            self.bleed((attacker, target), from, kind, blow);
            let died = self.hurt(target, blow.damage, Harm::Actor(attacker), events);
            if died {
                self.train_kill((attacker, target), kind);
                // Of Siphoning and of the Hunt: a kill of one's own close by (`siphon.rs`).
                self.siphon(attacker, (target, to));
                let fired = super::weapon_ability::Fired {
                    other: target,
                    kind,
                    skills,
                    blow: Some(blow),
                };
                self.weapon_trigger(attacker, crate::content::Trigger::Kill, fired, events);
            }
            // A sneak attack that kills builds more (Patient blade's openings).
            if died && unseen {
                self.gain_resource(attacker, Build::SneakKill);
                self.vanishing_act(attacker, events);
            }
            if !died {
                self.big_hit_trigger((attacker, target), (kind, skills), blow, events);
                self.after_blow(attacker, target, kind, (&blow, &made), events);
                self.barbed(attacker, target, kind, events);
            }
        }
        // Drowning grip (spec §8 "Birth sign"): a hit holds, a miss or another target lets go.
        self.grip(attacker, target, kind, blow.as_ref(), events);
        blow
    }

    /// The level of actor `id` as weapon marks weigh it (spec §8 "Weapon skills"): a
    /// creature's type's (spec §8 "Experience"), the player's own.
    pub(super) fn level_of(&self, id: ActorId) -> u32 {
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
    pub(super) fn practise(&mut self, skill: WeaponSkillId, success: bool, level: u32) {
        let content = self.content.clone();
        let table = content.weapon_skills();
        let Some(rules) = table.marks() else {
            return;
        };
        let me = self.player_mut();
        let marks = rules.earned(success, level, me.weapon_skills.rank(skill));
        // Weapon drill (spec §8 "Class abilities"): more marks in a role's skills.
        let marks = (me.talent_mods(&content)).marks(table.skill(skill).role, marks);
        if marks > 0 {
            let old = me.weapon_skills.clone();
            let risen = me.weapon_skills.train(table, skill, marks);
            if !risen.is_empty() {
                // What the ranks taught before they rose, for the news of what they teach now.
                let new = std::mem::replace(&mut self.player_mut().weapon_skills, old);
                let before = self.state.player().weapon_abilities(&content);
                self.player_mut().weapon_skills = new;
                self.ranks_news(&risen, &before);
            }
        }
    }

    /// The news of the player's weapon skills `risen` (a rank gained, with the skills it raised
    /// by default), and of the weapon abilities they teach that the player did not know
    /// `before`: "You feel more skilled with long blades. You have learned the deep cut."
    pub(super) fn ranks_news(
        &mut self,
        risen: &[WeaponSkillId],
        before: &[(WeaponSkillId, crate::content::WeaponAbilityId)],
    ) {
        let content = self.content.clone();
        let table = content.weapon_skills();
        let mut lines: Vec<String> = (risen.iter())
            .map(|&id| format!("You feel more skilled {}.", table.skill(id).phrase))
            .collect();
        // The first active a rank teaches grants the stamina it is paid from to one without it
        // (the third place stamina is granted, beside the class tables and a restored run),
        // said after that active.
        let mut breath = self.grant_stamina(before);
        // Each with the rank's line of the skill it is listed under (else the last line).
        for (skill, mut line, active) in self.learned(before) {
            if active && std::mem::take(&mut breath) {
                line.push_str(" You learn to pace your breath.");
            }
            let at = (risen.iter().position(|&s| s == skill)).or(lines.len().checked_sub(1));
            if let Some(text) = at.and_then(|i| lines.get_mut(i)) {
                text.push(' ');
                text.push_str(&line);
            }
        }
        self.news.extend(lines);
    }

    /// The player, who did not know them `before`, has learned an active weapon ability: they
    /// hold stamina from now on, full, if they did not. Returns whether it was granted.
    fn grant_stamina(
        &mut self,
        before: &[(WeaponSkillId, crate::content::WeaponAbilityId)],
    ) -> bool {
        let content = self.content.clone();
        let Some(stamina) = content.resource_id(crate::content::STAMINA) else {
            return false;
        };
        let me = self.state.player();
        let active = (me.weapon_abilities(&content).into_iter())
            .filter(|(_, id)| !before.iter().any(|(_, b)| b == id))
            .any(|(_, id)| content.weapon_ability(id).active.is_some());
        if !active || me.resources.get(stamina).is_some() {
            return false;
        }
        let level = self.player_level();
        super::focus::hold(&content, self.player_mut(), &[stamina], level);
        true
    }

    /// Actor `id` loses `damage` hit points to `harm`, the one way hit points go down (blows,
    /// traps, status effects, diseases, starving, the plunge through the ice); at 0 it dies of
    /// it, unless the harm spares the last hit point. Returns whether it died.
    pub(super) fn hurt(
        &mut self,
        id: ActorId,
        damage: i32,
        harm: Harm,
        events: &mut Vec<Event>,
    ) -> bool {
        // Any harm finds a disguised mimic out (`mimic.rs`), and ends a slip past (`slip.rs`).
        if self.disguised(id) {
            self.reveal_mimic(id, super::mimic::Reveal::Struck, events);
        }
        self.end_slip(id);
        // Talents (spec §8 "Talents": Immune to pain): a hit does less after the soak, never
        // below 1; what is not a hit (a status, starving) is not eased.
        let damage = match harm {
            Harm::Actor(_) | Harm::Trap(_) | Harm::Place if damage > 1 => {
                let pain =
                    (self.state.actors.get(&id)).map_or(0, |a| a.talent_mods(&self.content).pain);
                (damage - pain).max(1)
            }
            _ => damage,
        };
        if matches!(harm, Harm::Actor(_)) && self.unstoppable(id, damage, events) {
            return false;
        }
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return false;
        };
        if harm == Harm::Spared {
            actor.hp = (actor.hp - damage).max(actor.hp.min(1));
            self.wake(id);
            return false;
        }
        actor.hp -= damage;
        if actor.hp > 0 {
            // A blow, or any harm, wakes a sleeper (`den.rs`).
            self.wake(id);
            return false;
        }
        // Rise from the ashes (spec §8 "Birth sign"): once in a life, but never from starving.
        if id == self.state.player
            && !matches!(harm, Harm::Starving(_))
            && (self.rise_from_ashes(events) || self.reprieve(events))
        {
            return false;
        }
        match harm {
            Harm::Spared => {}
            Harm::Actor(by) => self.dies(id, Some(by), None, self.state.time, events),
            Harm::Status { of, by, when } => self.dies(id, by, of, when, events),
            Harm::Starving(when) if id == self.state.player => {
                self.die_of(Cause::Starvation, when);
            }
            Harm::Starving(when) => self.dies(id, None, None, when, events),
            Harm::Barbs => self.dies(id, None, None, self.state.time, events),
            Harm::Place | Harm::Fall => {
                self.dies(id, None, None, self.state.time, events);
                if id == self.state.player
                    && let Some(death) = &mut self.state.death
                {
                    death.cause = Some(if matches!(harm, Harm::Fall) {
                        Cause::Fall
                    } else {
                        Cause::Altar
                    });
                }
            }
            Harm::Trap(trap) => {
                // No creature and no status: the trap is what killed them (spec §8 "Traps"),
                // and a trap's kill earns no one anything.
                self.dies(id, None, None, self.state.time, events);
                if id == self.state.player
                    && let Some(death) = &mut self.state.death
                {
                    death.cause = Some(Cause::Trap(trap));
                }
            }
        }
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
                cause: None,
            });
        } else {
            // The break lines are said with the death, before what the kill earned.
            let earned = self.news.len();
            // A kill builds the killer's resources (spec §8 "Resources").
            if let Some(by) = by {
                self.gain_resource(by, Build::Kill);
            }
            if by == Some(self.state.player) {
                self.earn_kill(creature);
                let level = self.state.current_level.clone();
                self.tally_kill(creature, &level);
                self.judge_kill(id);
            }
            let lines = self.creature_dies(id, by == Some(self.state.player));
            self.news.splice(earned..earned, lines);
        }
    }

    /// Creature `id` dies: it leaves the world, and what it carried (worn and wielded things
    /// too) falls on its cell, on top of what lies there, in the order it carried them, and its
    /// corpse on top of that (spec §8 "Hunger"); identified, when the player killed it
    /// (`by_player`) with Scavenger. Returns the lines for an animated thing's components that
    /// broke as they fell, if the player saw it ([`World::break_lines`]).
    fn creature_dies(&mut self, id: ActorId, by_player: bool) -> Vec<String> {
        let Some(actor) = self.state.actors.remove(&id) else {
            return Vec::new();
        };
        // A named monster's death meets every quest after it, whoever caused it (design §8.4).
        let def = self.content.creature(actor.creature);
        if def.unique {
            let slain = def.id.clone();
            self.state.quests.slay(&slain);
        }
        // An animated thing's body falls whole or not at all (spec §8 "Animated objects").
        let (mut dropped, broken) = self.remains(&actor);
        let mut lines = self.break_lines(&actor.level, actor.pos, &broken);
        dropped.extend(self.loot_of(&actor));
        // Scavenger (spec §8 "Creature gear"): what the player's kill drops is known at once.
        if by_player && self.state.player().talent_mods(&self.content).scavenger {
            dropped.iter_mut().for_each(|stack| _ = stack.identify());
        }
        // An incorporeal creature dies inside a wall: what it carried falls out of it.
        let mut actor = actor;
        actor.pos = self.landing(&actor.level, actor.pos);
        if !dropped.is_empty()
            && let Some(level) = self.state.levels.get_mut(&actor.level)
        {
            let mut pile = level.objects_at(actor.pos).to_vec();
            object::add_to_pile(&mut pile, &dropped, &self.content);
            level.set_pile(actor.pos, pile);
        }
        self.leave_corpse(&actor);
        // In a cursed room the dead rise (spec §7 "Special rooms"; `curse.rs`).
        lines.extend(self.curse_corpse(&actor, by_player));
        // A dead shopkeeper's goods are free to take (spec §7 "Shops").
        self.keeper_died(id, &actor.level);
        self.fallen.push(actor);
        lines
    }

    /// The player attacks `target`, which they `seen` or not, in melee: a blow, and the blows
    /// that may follow it ([`World::second_blow`]: the off hand's, a shield bash, then a cleave
    /// or the target's riposte). Each message is said at once, by what the player knows as they
    /// strike; events reach presentation only if they saw the target.
    pub(super) fn player_attacks(&mut self, target: ActorId, seen: bool, events: &mut Vec<Event>) {
        let player = self.state.player;
        let first = self.player_blow(target, seen, None, events);
        let mut parried = free_blow::parried(first);
        if let Some(second) = self.second_blow(player, target, Cue::First(first), events) {
            let with = second.with;
            let struck = self.player_blow(target, seen, Some(second), events);
            parried |= free_blow::parried(struck);
            self.both_hands(player, target, (first, struck), with, events);
        }
        self.after_attack(player, target, parried, events);
    }

    /// The player strikes `target` in melee: the first blow of an attack (`None`: with what
    /// they wield, its noise made), or a blow more ([`World::second_blow`]), made with what and
    /// how.
    pub(super) fn player_blow(
        &mut self,
        target: ActorId,
        seen: bool,
        more: Option<FreeBlow>,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let me = self.state.player();
        let noisy = more.as_ref().is_none_or(|b| b.kind.noisy());
        // With a shield in each hand the first blow is a bash (`two_shields.rs`).
        let (with, profile) = match more {
            Some(b) => (b.with, b.profile),
            None => match me.opening_bash(&self.content) {
                Some((shield, profile)) => (Some(shield), profile),
                None => (None, me.melee(&self.content)),
            },
        };
        let skills = me.attack_skills(&self.content, AttackKind::Melee, with);
        let words = |me: &Actor, content: &Content, damage| {
            Words::of(me, AttackKind::Melee, with, damage, content)
        };
        self.player_strikes(
            target,
            seen,
            (with, profile, skills),
            (&words, noisy),
            events,
        )
    }

    /// The player strikes `target` in melee with what they wield, as `profile` says (a class
    /// active's blow: Smite's dice, Quivering palm's bare hand), its noise made.
    pub(super) fn player_blow_with(
        &mut self,
        target: ActorId,
        seen: bool,
        profile: Profile,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let skills = (self.state.player()).attack_skills(&self.content, AttackKind::Melee, None);
        let words = |me: &Actor, content: &Content, damage| {
            Words::of(me, AttackKind::Melee, None, damage, content)
        };
        self.player_strikes(
            target,
            seen,
            (None, profile, skills),
            (&words, true),
            events,
        )
    }

    /// The player kicks `target` (spec §8 "Doors"): an unarmed blow in melee, whatever they
    /// wield ([`Actor::kick`]: it draws on and trains unarmed fighting, and no fighting style),
    /// said and shown like any blow, with the free blows the end of an attack brings
    /// ([`World::after_attack`]).
    pub(super) fn player_kicks(&mut self, target: ActorId, seen: bool, events: &mut Vec<Event>) {
        let profile = self.state.player().kick(&self.content);
        let skills = AttackSkills::only(self.content.weapon_skills().of_role(Role::Unarmed));
        let blow = (None, profile, skills);
        let words = |_: &Actor, _: &Content, _| Words::kick();
        let struck = self.player_strikes(target, seen, blow, (&words, true), events);
        self.after_attack(
            self.state.player,
            target,
            free_blow::parried(struck),
            events,
        );
    }

    /// The player bashes whoever stands in `dir` with the shield they hold (spec §8 "Weapon
    /// skills"): a blow like any other ([`World::player_attacks`]), with the shield's `bash`
    /// dice and shield bashing's benefits, for the price of a blow with the shield (`melee`, its
    /// own affixes and what the player wears, never the wielded weapon's cost or affixes:
    /// [`World::attack_cost`]), and the free blows the end of an attack brings
    /// ([`World::after_attack`]). Refused (no time) without a
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
                format!("You have no quarrel with {}.", crate::content::the(name))
            } else {
                "Something is in the way.".into()
            });
            return Spent::Nothing;
        }
        let me = self.state.player();
        let skills = me.attack_skills(&content, AttackKind::Melee, Some(shield));
        let with = me.shield_item(&content);
        let price = self.attack_price(player, ActionKind::Melee, with, skills);
        let blow = (Some(shield), profile, skills);
        let words = |me: &Actor, content: &Content, damage| {
            Words::of(me, AttackKind::Melee, Some(shield), damage, content)
        };
        let struck = self.player_strikes(target, seen, blow, (&words, true), events);
        self.after_attack(player, target, free_blow::parried(struck), events);
        Spent::Turn(price)
    }

    /// The player strikes `target` in melee with `profile`, made `with` a shield (a bash), the
    /// off hand's weapon, or not (a blow, a kick), drawing on (and training) `skills`, said in
    /// the `words` of the kind of damage it did, with its noise if `noisy` (the first blow of
    /// an attack): see [`World::player_attacks`]. Returns the blow.
    pub(super) fn player_strikes(
        &mut self,
        target: ActorId,
        seen: bool,
        (with, profile, skills): (Option<ObjectId>, Profile, AttackSkills),
        (words, noisy): (
            &dyn Fn(&Actor, &Content, Option<DamageTypeId>) -> Words,
            bool,
        ),
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let player = self.state.player;
        let mut made = Vec::new();
        let kind = (AttackKind::Melee, with, skills);
        let blow = self.resolve(player, target, kind, Some(profile), noisy, &mut made);
        let content = self.content.clone();
        let words = words(self.state.player(), &content, blow.and_then(|b| b.kind));
        let target_actor = self.actor_or_fallen(target);
        let name = target_actor.map(|a| content.creature(a.creature).name.as_str());
        let guard = Guard::of(target_actor, &content, &self.state);
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
            guard,
        ));
        if died && seen {
            let dies = target_actor.map_or("dies", |a| content.creature(a.creature).death_verb());
            said.push(death_message(end, dies));
        }
        for text in said {
            self.say(text);
        }
        self.say_news();
        if seen {
            events.extend(made);
        }
        blow
    }
}
