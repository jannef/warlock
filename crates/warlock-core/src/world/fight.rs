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
use super::{Actor, ActorId, AttackSkills, Spent, World};
use crate::action::{AttackKind, Event};
use crate::combat::{self, Blow, Category, Profile, Strike, Target};
use crate::content::{
    ActionKind, Content, CreatureId, DamageTypeId, Grip, ObjectDef, ObjectId, Role, Stance,
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
}

impl Cause {
    /// How the death screen says it: "Starved to death", "Choked to death on a ration",
    /// "Killed by a fireball trap", "Killed by their own scroll of fireball".
    pub fn epitaph(self, content: &Content) -> String {
        let what = |o: ObjectId| content.object(o).describe(1);
        match self {
            Cause::Starvation => "Starved to death".into(),
            Cause::Choked(o) => format!("Choked to death on {}", what(o)),
            Cause::Trap(t) => {
                let name = &content.trap(t).name;
                format!("Killed by {} {name}", article(name))
            }
            Cause::Scroll(o) => format!("Killed by their own {}", content.object(o).name),
        }
    }

    /// What the player is told as it happens: "You starve to death...". A trap's victim dies
    /// like any other ("You die...").
    pub fn message(self, content: &Content) -> String {
        let name = |o: ObjectId| content.object(o).name.clone();
        match self {
            Cause::Starvation => "You starve to death...".into(),
            Cause::Choked(o) => format!("You choke to death on the {}...", name(o)),
            Cause::Trap(_) | Cause::Scroll(_) => "You die...".into(),
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
            End::Seen(name) | End::Trap(name) => format!("The {name}"),
            End::Unseen => "Something".into(),
        }
    }

    /// As the object: "you", "the kobold", "something".
    fn object(self) -> String {
        match self {
            End::You => "you".into(),
            End::Seen(name) | End::Trap(name) => format!("the {name}"),
            End::Unseen => "something".into(),
        }
    }

    /// Whose: "your", "the kobold's"; a trap's own missile or blast is just "the" one.
    fn whose(self) -> String {
        match self {
            End::You => "your".into(),
            End::Seen(name) => format!("the {name}'s"),
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
    /// The guard of `target` (alive, or fallen this action), if it is known.
    pub(super) fn of(target: Option<&Actor>, content: &'a Content) -> Self {
        let Some(target) = target else {
            return Self::default();
        };
        Self {
            hide: (content.creature(target.creature).natural.as_ref()).map(|h| h.name.as_str()),
            soak: target.soaked_by(content),
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
        },
    };
    Some(text)
}

/// The message for a death the player saw: "The giant rat dies.", or the player's own.
pub(super) fn death_message(who: End<'_>) -> String {
    match who {
        End::You => "You die...".into(),
        End::Seen(name) => format!("The {name} dies."),
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
        let profile = match kind {
            AttackKind::Melee => Some(me.melee(&self.content)),
            AttackKind::Ranged => me.ranged(&self.content),
            AttackKind::Thrown | AttackKind::Spell => None,
        };
        let blow = self.attack_with(attacker, target, kind, profile, events);
        if kind == AttackKind::Melee {
            let mut parried = free_blow::parried(blow);
            if let Some(second) = self.second_blow(attacker, target, Cue::First(blow)) {
                parried |= free_blow::parried(self.strike_free(second, events));
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
    /// - [`Cue::Leaves`]: `target` steps away from beside `striker`: the opportunist's blow.
    ///
    /// Only the second blow may draw while deciding; the free blows never do.
    pub(super) fn second_blow(
        &mut self,
        striker: ActorId,
        target: ActorId,
        cue: Cue,
    ) -> Option<FreeBlow> {
        match cue {
            Cue::First(first) => {
                (self.off_hand_or_bash(striker, target, first)).map(|(with, profile)| FreeBlow {
                    by: striker,
                    at: target,
                    kind: Free::Second,
                    with: Some(with),
                    profile,
                })
            }
            Cue::Attacked { parried } => (self.cleave(striker, target, false))
                .or_else(|| self.riposte(striker, target, parried)),
            Cue::Cleaved => self.cleave(striker, target, true),
            Cue::Leaves => self.opportunist(striker, target),
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
        let (Grip::WeaponAndShield, _) = me.grip(&content)? else {
            return None;
        };
        let (shield, profile) = me.bash(&content)?;
        let chance = me.attack_benefit(&content, AttackKind::Melee, None).bash;
        let hit = first.is_some_and(|b| b.strike.is_hit());
        (hit && chance > 0 && self.state.rng.chance(u32::from(chance))).then_some((shield, profile))
    }

    /// The Defence of actor `id`, its burden capping its Dexterity (spec §8 "Burden").
    pub(crate) fn defence_of(&self, id: ActorId) -> combat::Defence {
        let actor = &self.state.actors[&id];
        actor.defence(
            &self.content,
            self.state.load(&self.content, actor).penalty(),
        )
    }

    /// The Protection of actor `id` (spec §8 "Combat"): what it wears and its hide soak off
    /// every physical hit, by kind; none for one that is gone.
    pub(crate) fn protection_of(&self, id: ActorId) -> Protection {
        (self.state.actors.get(&id))
            .map_or_else(|| Protection::plain(0), |a| a.protection(&self.content))
    }

    /// What actor `id` resists (spec §8 "Damage types"): its creature type's, and what it wears
    /// and wields; nothing for one that is gone.
    pub(crate) fn resistances_of(&self, id: ActorId) -> Resistances {
        (self.state.actors.get(&id))
            .map_or_else(Resistances::default, |a| a.resistances(&self.content))
    }

    /// The side of actor `id` a blow strikes: its Defence, Protection, and resistances.
    pub(crate) fn target_of(&self, id: ActorId) -> Target {
        Target {
            defence: self.defence_of(id),
            protection: self.protection_of(id),
            resistances: self.resistances_of(id),
        }
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
        let struck = self.target_of(target);
        let levels = (self.level_of(attacker), self.level_of(target));
        let rng = &mut self.state.rng;
        let blow = profile.map(|profile| combat::strike(&profile, &struck, rng));
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
        if let Some(blow) = blow
            && blow.damage > 0
        {
            self.bleed((attacker, target), from, kind, blow);
            let died = self.hurt(target, blow.damage, Harm::Actor(attacker), events);
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
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return false;
        };
        if harm == Harm::Spared {
            actor.hp = (actor.hp - damage).max(actor.hp.min(1));
            return false;
        }
        actor.hp -= damage;
        if actor.hp > 0 {
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
            if by == Some(self.state.player) {
                self.earn_kill(creature);
                let level = self.state.current_level.clone();
                self.tally_kill(creature, &level);
            }
            self.creature_dies(id);
        }
    }

    /// Creature `id` dies: it leaves the world, and what it carried (worn and wielded things
    /// too) falls on its cell, on top of what lies there, in the order it carried them, and its
    /// corpse on top of that (spec §8 "Hunger").
    fn creature_dies(&mut self, id: ActorId) {
        let Some(actor) = self.state.actors.remove(&id) else {
            return;
        };
        let mut dropped = actor.inventory.clone();
        dropped.extend(self.loot_of(&actor));
        if !dropped.is_empty()
            && let Some(level) = self.state.levels.get_mut(&actor.level)
        {
            let mut pile = level.objects_at(actor.pos).to_vec();
            object::add_to_pile(&mut pile, &dropped, &self.content);
            level.set_pile(actor.pos, pile);
        }
        self.leave_corpse(&actor);
        // A dead shopkeeper's goods are free to take (spec §7 "Shops").
        self.keeper_died(id, &actor.level);
        self.fallen.push(actor);
    }

    /// The player attacks `target`, which they `seen` or not, in melee: a blow, and the blows
    /// that may follow it ([`World::second_blow`]: the off hand's, a shield bash, then a cleave
    /// or the target's riposte). Each message is said at once, by what the player knows as they
    /// strike; events reach presentation only if they saw the target.
    pub(super) fn player_attacks(&mut self, target: ActorId, seen: bool, events: &mut Vec<Event>) {
        let player = self.state.player;
        let first = self.player_blow(target, seen, None, events);
        let mut parried = free_blow::parried(first);
        if let Some(second) = self.second_blow(player, target, Cue::First(first)) {
            parried |= free_blow::parried(self.player_blow(target, seen, Some(second), events));
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
        let with = more.as_ref().and_then(|b| b.with);
        let noisy = more.as_ref().is_none_or(|b| b.kind.noisy());
        let profile = more.map_or_else(|| me.melee(&self.content), |b| b.profile);
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
                format!("You have no quarrel with the {name}.")
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
    fn player_strikes(
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
        let guard = Guard::of(target_actor, &content);
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
            said.push(death_message(end));
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
