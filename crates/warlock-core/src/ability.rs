//! Abilities (spec §6 "Abilities", #156): what a character can do beyond the commands every run
//! has, because of who they are — racial, class, skill, weapon skill and fighting style, and
//! item abilities. Unlike ADOM, they do not get a key each: they all go into one abilities menu,
//! which lists [`PerceptionView::abilities`] grouped by [`Source`], asks for what each needs
//! ([`Ability::aim`]), and applies the [`Action`] it makes ([`Ability::action`]), so replays
//! record an ability as the action it is. An ability that has an action of its own may also be a
//! key command, unbound by default (rebindable in Controls: shield bash).
//!
//! **This is the one registry**: a system that gives abilities adds them here, as a variant of
//! [`Ability`] with its name, source, aim, cost, and action, a line in [`Ability::every`] that
//! lists it, and who has it ([`Ability::had_by`]); [`Ability::unavailable`] says when it cannot
//! be used now and why (the menu greys it out with the reason). Nothing else gets a key of its
//! own. A limit on how often an ability may be used is a **cooldown status** (spec §8 "Status
//! effects": Winded, with `eases` and no tick) given as it is used: while it lasts the ability is
//! [`Unavailable::While`] it, and so is one whose kind of action a status `blocks`; nothing
//! counts uses.
//!
//! [`PerceptionView::abilities`]: crate::perception::PerceptionView::abilities

use crate::action::Action;
use crate::content::{ActionKind, Content, SkillDef, StatusId};
use crate::geom::Dir;
use crate::skills::{Effect, SkillId};
use crate::world::Actor;
use std::borrow::Cow;

/// An ability, as the abilities menu lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ability {
    /// A skill used as an action (`kind = "action"` in `data/skills.toml`) that the actor is
    /// trained in, whose use is not in the game yet: listed, never available (spec §8
    /// "Skills"). A use, when it comes, is a variant of its own with its action, and its skill
    /// is no longer listed as this.
    Skill(SkillId),
    /// Pick the lock of a door next to the actor (spec §8 "Doors"): [`Action::PickLock`], the
    /// Open lock skill's use. Had by those trained in a skill with the `pick` effect
    /// ([`Effect::Pick`]: Open lock), which is then listed as this, not as [`Ability::Skill`].
    PickLock,
    /// Bash with the shield held (spec §8 "Weapon skills"): [`Action::Bash`]. Everyone can bash
    /// with a shield (at shield bashing's rank 0 too), so everyone has it; it is available while
    /// a shield to bash with is held.
    ShieldBash,
}

/// Where an ability comes from: the abilities menu's groups, in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Source {
    /// The character's race (none yet).
    Race,
    /// The character's class (none yet).
    Class,
    /// A trained skill used as an action (spec §8 "Skills").
    Skill,
    /// A weapon skill or a fighting style (spec §8 "Weapon skills", "Fighting styles").
    WeaponSkill,
    /// Something worn or carried (none yet).
    Item,
}

impl Source {
    /// Every source, in menu order.
    pub const ALL: &[Source] = &[
        Source::Race,
        Source::Class,
        Source::Skill,
        Source::WeaponSkill,
        Source::Item,
    ];

    /// Its group's heading in the abilities menu.
    pub const fn name(self) -> &'static str {
        match self {
            Source::Race => "Racial",
            Source::Class => "Class",
            Source::Skill => "Skills",
            Source::WeaponSkill => "Weapon skills",
            Source::Item => "Items",
        }
    }

    /// Its id in dumps.
    pub const fn id(self) -> &'static str {
        match self {
            Source::Race => "race",
            Source::Class => "class",
            Source::Skill => "skill",
            Source::WeaponSkill => "weapon_skill",
            Source::Item => "item",
        }
    }
}

/// What an ability asks for before it is used. A target (the aiming mode) and an item (a list of
/// the pack) join these with the first ability that needs one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aim {
    /// Nothing: choosing it uses it.
    Nothing,
    /// A direction, as opening a door does.
    Direction,
}

/// Why an ability cannot be used now: the abilities menu shows it beside the greyed-out row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unavailable {
    /// A shield bash without a shield to bash with.
    NoShield,
    /// A skill whose use is not in the game yet.
    NotYet,
    /// While a status lasts (spec §8 "Status effects"): one that blocks the kind of action the
    /// ability is (a raging barbarian cannot read), or the ability's cooldown, which the class
    /// abilities bring (Winded: "you are winded"). Only time lifts it.
    While(StatusId),
}

impl Unavailable {
    /// The reason, as the menu says it: "you are winded".
    pub fn reason(self, content: &Content) -> Cow<'static, str> {
        match self {
            Unavailable::NoShield => "you hold no shield".into(),
            Unavailable::NotYet => "not usable yet".into(),
            Unavailable::While(status) => {
                format!("you are {}", content.status(status).name.to_lowercase()).into()
            }
        }
    }

    /// Its id in dumps: `no_shield`, `while:winded`.
    pub fn id(self, content: &Content) -> Cow<'static, str> {
        match self {
            Unavailable::NoShield => "no_shield".into(),
            Unavailable::NotYet => "not_yet".into(),
            Unavailable::While(status) => format!("while:{}", content.status(status).id).into(),
        }
    }
}

impl Ability {
    /// Every ability there is, in menu order: by [`Source`], then as each source lists its own.
    /// The one place abilities are registered (the wiki's table lists these).
    pub fn every(content: &Content) -> Vec<Ability> {
        // Skills: those used as actions, in skill order, each as its use where it has one.
        let skills = (content.skills().iter().enumerate())
            .filter(|(_, def)| def.is_active())
            .filter_map(|(i, def)| {
                let skill = Ability::Skill(SkillId(u8::try_from(i).ok()?));
                Some(Ability::use_of(def).unwrap_or(skill))
            });
        // Weapon skills and fighting styles: the shield bash.
        let weapon_skills = [Ability::ShieldBash];
        let mut all: Vec<Ability> = Vec::new();
        for ability in skills.chain(weapon_skills) {
            if !all.contains(&ability) {
                all.push(ability);
            }
        }
        all.sort_by_key(|a| a.source());
        all
    }

    /// The ability that is the use of skill `def` (spec §8 "Skills"), if its use is in the game:
    /// by the effect that the use reads (Open lock's `pick`: Pick lock).
    fn use_of(def: &SkillDef) -> Option<Ability> {
        (def.benefits.iter()).find_map(|b| match b.effect {
            Effect::Pick => Some(Ability::PickLock),
            _ => None,
        })
    }

    /// Every ability `actor` has, in menu order.
    pub(crate) fn of(actor: &Actor, content: &Content) -> Vec<Ability> {
        (Ability::every(content).into_iter())
            .filter(|a| a.had_by(actor, content))
            .collect()
    }

    /// Whether `actor` has it (whether or not they can use it now): a skill, if they are trained
    /// in it; picking locks, if they are trained in Open lock; the shield bash, everyone.
    fn had_by(self, actor: &Actor, content: &Content) -> bool {
        match self {
            Ability::Skill(id) => actor.skills.get(id).is_some(),
            Ability::PickLock => actor.skills.trains_effect(content, Effect::Pick),
            Ability::ShieldBash => true,
        }
    }

    /// Its id in dumps: `shield_bash`, `pick_lock`, or the skill's id.
    pub fn id(self, content: &Content) -> &str {
        match self {
            Ability::Skill(id) => &content.skill(id).id,
            Ability::PickLock => "pick_lock",
            Ability::ShieldBash => "shield_bash",
        }
    }

    /// Its name in the menu.
    pub fn name(self, content: &Content) -> &str {
        match self {
            Ability::Skill(id) => &content.skill(id).name,
            Ability::PickLock => "Pick lock",
            Ability::ShieldBash => "Shield bash",
        }
    }

    /// What it does, in a line.
    pub fn description(self, content: &Content) -> &str {
        match self {
            Ability::Skill(id) => &content.skill(id).description,
            Ability::PickLock => "Open a locked door next to you without its key.",
            Ability::ShieldBash => "Strike someone next to you with the shield you hold.",
        }
    }

    /// Where it comes from: its group in the menu.
    pub const fn source(self) -> Source {
        match self {
            Ability::Skill(_) | Ability::PickLock => Source::Skill,
            Ability::ShieldBash => Source::WeaponSkill,
        }
    }

    /// What it asks for before it is used.
    pub const fn aim(self) -> Aim {
        match self {
            Ability::Skill(_) => Aim::Nothing,
            Ability::PickLock | Ability::ShieldBash => Aim::Direction,
        }
    }

    /// What it costs, as an action of this kind (`data/actions.toml`; the rules then price it as
    /// that action, spec §8 "Time"); `None` for one with no use yet.
    pub const fn cost(self) -> Option<ActionKind> {
        match self {
            Ability::Skill(_) => None,
            Ability::PickLock => Some(ActionKind::PickLock),
            Ability::ShieldBash => Some(ActionKind::Melee),
        }
    }

    /// The action it is, aimed `dir` when it asks for a direction; `None` for one with no use
    /// yet, or one that asks for a direction without it.
    pub const fn action(self, dir: Option<Dir>) -> Option<Action> {
        match (self, dir) {
            (Ability::Skill(_), _) => None,
            (Ability::PickLock, Some(dir)) => Some(Action::PickLock(dir)),
            (Ability::ShieldBash, Some(dir)) => Some(Action::Bash(dir)),
            (Ability::PickLock | Ability::ShieldBash, None) => None,
        }
    }

    /// Why `actor` cannot use it now; `None` when it can. Its own reasons come first, then a
    /// status of the actor's that blocks the kind of action it costs ([`Unavailable::While`]).
    pub(crate) fn unavailable(self, actor: &Actor, content: &Content) -> Option<Unavailable> {
        let own = match self {
            Ability::Skill(_) => Some(Unavailable::NotYet),
            Ability::PickLock => None,
            Ability::ShieldBash => actor
                .shield(content)
                .is_none()
                .then_some(Unavailable::NoShield),
        };
        own.or_else(|| {
            let blocked = actor.effects.blocking(content, self.cost()?);
            blocked.map(Unavailable::While)
        })
    }
}

/// One of the player's abilities, as they know it: for the abilities menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnAbility {
    pub ability: Ability,
    /// Why it cannot be used now; `None` when it can.
    pub unavailable: Option<Unavailable>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sources_are_in_menu_order_and_named() {
        let mut sorted = Source::ALL.to_vec();
        sorted.sort();
        assert_eq!(sorted, Source::ALL, "the menu's order is the enum's");
        let names: Vec<&str> = Source::ALL.iter().map(|s| s.name()).collect();
        assert_eq!(
            names,
            ["Racial", "Class", "Skills", "Weapon skills", "Items"]
        );
    }

    #[test]
    fn every_ability_is_registered_in_menu_order() {
        let content = Content::builtin().expect("content");
        let every = Ability::every(&content);
        assert!(every.windows(2).all(|w| w[0].source() <= w[1].source()));
        // The skills used as actions, in skill order (Open lock as its use, Pick lock), then the
        // shield bash.
        let skills: Vec<&str> = (every.iter())
            .filter(|a| a.source() == Source::Skill)
            .map(|a| a.id(&content))
            .collect();
        assert_eq!(
            skills,
            [
                "appraise",
                "bluff",
                "decipher_script",
                "disable_device",
                "handle_animal",
                "heal",
                "intimidate",
                "knowledge",
                "metalcraft",
                "music",
                "pick_lock",
                "steal",
                "use_magic_device",
                "woodcraft",
            ]
        );
        assert_eq!(every.last(), Some(&Ability::ShieldBash));
    }

    #[test]
    fn a_shield_bash_asks_for_a_direction_and_is_a_bash() {
        let bash = Ability::ShieldBash;
        assert_eq!(bash.aim(), Aim::Direction);
        assert_eq!(bash.source(), Source::WeaponSkill);
        assert_eq!(bash.cost(), Some(ActionKind::Melee));
        assert_eq!(bash.action(Some(Dir::E)), Some(Action::Bash(Dir::E)));
        assert_eq!(bash.action(None), None, "not without a direction");
        assert_eq!(Ability::Skill(SkillId(0)).action(Some(Dir::E)), None);
    }

    #[test]
    fn picking_a_lock_asks_for_a_direction_and_is_its_own_action() {
        let pick = Ability::PickLock;
        assert_eq!(pick.aim(), Aim::Direction);
        assert_eq!(pick.source(), Source::Skill);
        assert_eq!(pick.cost(), Some(ActionKind::PickLock));
        assert_eq!(pick.action(Some(Dir::W)), Some(Action::PickLock(Dir::W)));
        assert_eq!(pick.action(None), None, "not without a direction");
    }
}
