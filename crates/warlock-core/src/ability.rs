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
use crate::content::{ActionKind, Content, StatusId};
use crate::geom::{Dir, Pos};
use crate::skills::SkillId;
use crate::world::{Actor, VOICE_RANGE};
use std::borrow::Cow;

/// An ability, as the abilities menu lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ability {
    /// Throw your voice (spec §5 "Hearing"; #235): [`Action::ThrowVoice`] at a cell the player
    /// sees. The bard's (design §4.8, at level 32); until the class tables give it to them alone,
    /// everyone has it.
    ThrowVoice,
    /// A skill used as an action (`kind = "action"` in `data/skills.toml`) that the actor is
    /// trained in, whose use is not in the game yet: listed, never available (spec §8
    /// "Skills"). A use, when it comes, is a variant of its own with its action, and its skill
    /// is no longer listed as this.
    Skill(SkillId),
    /// Bash with the shield held (spec §8 "Weapon skills"): [`Action::Bash`]. Everyone can bash
    /// with a shield (at shield bashing's rank 0 too), so everyone has it; it is available while
    /// a shield to bash with is held.
    ShieldBash,
    /// Lower or raise the lantern's hood (spec §5 "Light sources"; #235): [`Action::HoodLantern`].
    /// Listed with what the player carries, the lantern; the Hooded light talent (design §7.5)
    /// will give it, and until then everyone has it (without a lantern the rules say so).
    HoodLantern,
}

/// Where an ability comes from: the abilities menu's groups, in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Source {
    /// The character's race (none yet).
    Race,
    /// The character's class (throwing one's voice, everyone's until the class tables).
    Class,
    /// A trained skill used as an action (spec §8 "Skills").
    Skill,
    /// A weapon skill or a fighting style (spec §8 "Weapon skills", "Fighting styles").
    WeaponSkill,
    /// Something worn or carried (the lantern's hood).
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

/// What an ability asks for before it is used. An item (a list of the pack) joins these with the
/// first ability that needs one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aim {
    /// Nothing: choosing it uses it.
    Nothing,
    /// A direction, as opening a door does.
    Direction,
    /// A cell, in the aiming mode, as a shot does ([`Ability::action_at`]).
    Cell,
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
        // Skills: those used as actions, in skill order.
        let skills = (content.skills().iter().enumerate())
            .filter(|(_, def)| def.is_active())
            .filter_map(|(i, _)| u8::try_from(i).ok().map(|i| Ability::Skill(SkillId(i))));
        // Class: throwing one's voice.
        let class = [Ability::ThrowVoice];
        // Weapon skills and fighting styles: the shield bash.
        let weapon_skills = [Ability::ShieldBash];
        // Items: the lantern's hood.
        let items = [Ability::HoodLantern];
        let mut all: Vec<Ability> = (class.into_iter())
            .chain(skills)
            .chain(weapon_skills)
            .chain(items)
            .collect();
        all.sort_by_key(|a| a.source());
        all
    }

    /// Every ability `actor` has, in menu order.
    pub(crate) fn of(actor: &Actor, content: &Content) -> Vec<Ability> {
        (Ability::every(content).into_iter())
            .filter(|a| a.had_by(actor))
            .collect()
    }

    /// Whether `actor` has it (whether or not they can use it now): a skill, if they are trained
    /// in it; the shield bash, throwing one's voice, and the lantern's hood, everyone.
    fn had_by(self, actor: &Actor) -> bool {
        match self {
            Ability::Skill(id) => actor.skills.get(id).is_some(),
            Ability::ThrowVoice | Ability::ShieldBash | Ability::HoodLantern => true,
        }
    }

    /// Its id in dumps: `shield_bash`, or the skill's id.
    pub fn id(self, content: &Content) -> &str {
        match self {
            Ability::Skill(id) => &content.skill(id).id,
            Ability::ThrowVoice => "throw_voice",
            Ability::ShieldBash => "shield_bash",
            Ability::HoodLantern => "hood_lantern",
        }
    }

    /// Its name in the menu.
    pub fn name(self, content: &Content) -> &str {
        match self {
            Ability::Skill(id) => &content.skill(id).name,
            Ability::ThrowVoice => "Throw your voice",
            Ability::ShieldBash => "Shield bash",
            Ability::HoodLantern => "Hood your lantern",
        }
    }

    /// What it does, in a line.
    pub fn description(self, content: &Content) -> &str {
        match self {
            Ability::Skill(id) => &content.skill(id).description,
            Ability::ThrowVoice => {
                "Make a voice sound at a place you see nearby: those who hear it go and look there."
            }
            Ability::ShieldBash => "Strike someone next to you with the shield you hold.",
            Ability::HoodLantern => {
                "Lower your lantern's hood, or raise it: hooded, it dimly lights the ground around \
                 you but not you."
            }
        }
    }

    /// Where it comes from: its group in the menu.
    pub const fn source(self) -> Source {
        match self {
            Ability::Skill(_) => Source::Skill,
            Ability::ThrowVoice => Source::Class,
            Ability::ShieldBash => Source::WeaponSkill,
            Ability::HoodLantern => Source::Item,
        }
    }

    /// What it asks for before it is used.
    pub const fn aim(self) -> Aim {
        match self {
            Ability::Skill(_) | Ability::HoodLantern => Aim::Nothing,
            Ability::ThrowVoice => Aim::Cell,
            Ability::ShieldBash => Aim::Direction,
        }
    }

    /// What it costs, as an action of this kind (`data/actions.toml`; the rules then price it as
    /// that action, spec §8 "Time"); `None` for one with no use yet.
    pub const fn cost(self) -> Option<ActionKind> {
        match self {
            Ability::Skill(_) => None,
            Ability::ThrowVoice => Some(ActionKind::ThrowVoice),
            Ability::ShieldBash => Some(ActionKind::Melee),
            Ability::HoodLantern => Some(ActionKind::Hood),
        }
    }

    /// How far a cell it aims at may be, in cells (a disc, as ranges are); `None` for one that
    /// aims at no cell.
    pub const fn range(self) -> Option<i32> {
        match self {
            Ability::ThrowVoice => Some(VOICE_RANGE),
            Ability::Skill(_) | Ability::ShieldBash | Ability::HoodLantern => None,
        }
    }

    /// The action it is, aimed `dir` when it asks for a direction; `None` for one with no use
    /// yet, one that asks for a direction without it, or one that asks for a cell
    /// ([`Ability::action_at`]).
    pub const fn action(self, dir: Option<Dir>) -> Option<Action> {
        match (self, dir) {
            (Ability::Skill(_) | Ability::ThrowVoice, _) => None,
            (Ability::ShieldBash, Some(dir)) => Some(Action::Bash(dir)),
            (Ability::ShieldBash, None) => None,
            (Ability::HoodLantern, _) => Some(Action::HoodLantern),
        }
    }

    /// The action it is, aimed at level cell `at`, for one that asks for a cell; `None` for any
    /// other.
    pub const fn action_at(self, at: Pos) -> Option<Action> {
        match self {
            Ability::ThrowVoice => Some(Action::ThrowVoice(at)),
            Ability::Skill(_) | Ability::ShieldBash | Ability::HoodLantern => None,
        }
    }

    /// Why `actor` cannot use it now; `None` when it can. Its own reasons come first, then a
    /// status of the actor's that blocks the kind of action it costs ([`Unavailable::While`]).
    pub(crate) fn unavailable(self, actor: &Actor, content: &Content) -> Option<Unavailable> {
        let own = match self {
            Ability::Skill(_) => Some(Unavailable::NotYet),
            Ability::ThrowVoice | Ability::HoodLantern => None,
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
        assert_eq!(
            every.first(),
            Some(&Ability::ThrowVoice),
            "the class's first"
        );
        // The skills used as actions, in skill order, then the shield bash.
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
                "open_lock",
                "steal",
                "use_magic_device",
                "woodcraft",
            ]
        );
        let last: Vec<Ability> = every.iter().rev().take(2).rev().copied().collect();
        assert_eq!(last, [Ability::ShieldBash, Ability::HoodLantern]);
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
        assert_eq!(bash.range(), None);
        assert_eq!(bash.action_at(Pos::new(3, 4)), None);
    }

    #[test]
    fn throwing_a_voice_aims_at_a_cell_and_the_hood_at_nothing() {
        let voice = Ability::ThrowVoice;
        assert_eq!((voice.aim(), voice.source()), (Aim::Cell, Source::Class));
        assert_eq!(voice.cost(), Some(ActionKind::ThrowVoice));
        assert_eq!(voice.range(), Some(VOICE_RANGE));
        assert_eq!(voice.action(None), None, "not without a cell");
        let at = Pos::new(3, 4);
        assert_eq!(voice.action_at(at), Some(Action::ThrowVoice(at)));
        let hood = Ability::HoodLantern;
        assert_eq!((hood.aim(), hood.source()), (Aim::Nothing, Source::Item));
        assert_eq!(hood.cost(), Some(ActionKind::Hood));
        assert_eq!(hood.action(None), Some(Action::HoodLantern));
        assert_eq!((hood.range(), hood.action_at(at)), (None, None));
    }
}
