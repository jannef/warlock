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
//! An ability that asks for an alignment (`requires_alignment` in its data, spec §8
//! "Alignment": the paladin's gifts are Good or better) stays the character's when they stray
//! from it, but cannot be used ([`Unavailable::Strayed`], "you have strayed") until they climb
//! back: its arm of [`Ability::unavailable`] asks [`strayed`] first. The class tables (#243) and
//! talents (#220) carry the requirement as an `Option<Requirement>` in their data.
//!
//! [`PerceptionView::abilities`]: crate::perception::PerceptionView::abilities

use crate::action::Action;
use crate::alignment::Requirement;
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
    /// Disarm a trap the actor knows of, next to them (spec §8 "Disarming traps"):
    /// [`Action::Disarm`], the use of Disable device (the skill with the disarm effect, listed as
    /// this in its place). Had by whoever is trained in it; usable whenever they have it (the
    /// rules refuse, taking no time, where they know of no trap).
    Disarm,
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
    /// One that asks for an alignment the character has strayed from (spec §8 "Alignment"):
    /// they keep it, and may use it again once they climb back.
    Strayed,
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
            Unavailable::Strayed => "you have strayed".into(),
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
            Unavailable::Strayed => "strayed".into(),
            Unavailable::While(status) => format!("while:{}", content.status(status).id).into(),
        }
    }
}

impl Ability {
    /// Every ability there is, in menu order: by [`Source`], then as each source lists its own.
    /// The one place abilities are registered (the wiki's table lists these).
    pub fn every(content: &Content) -> Vec<Ability> {
        // Skills: those used as actions, in skill order, each as its use once that is in the game.
        let skills = (content.skills().iter().enumerate())
            .filter(|(_, def)| def.is_active())
            .filter_map(|(i, def)| {
                let id = SkillId(u8::try_from(i).ok()?);
                Some(Ability::use_of(def).unwrap_or(Ability::Skill(id)))
            });
        // Weapon skills and fighting styles: the shield bash.
        let weapon_skills = [Ability::ShieldBash];
        let mut all: Vec<Ability> = skills.chain(weapon_skills).collect();
        all.sort_by_key(|a| a.source());
        all
    }

    /// The ability that is the use of skill `def`, once that use is in the game: Disable device's
    /// (the disarm effect's) is [`Ability::Disarm`]. `None` for a skill whose use is to come.
    fn use_of(def: &SkillDef) -> Option<Ability> {
        let gives = |effect| def.benefits.iter().any(|b| b.effect == effect);
        gives(Effect::Disarm).then_some(Ability::Disarm)
    }

    /// Every ability `actor` has, in menu order.
    pub(crate) fn of(actor: &Actor, content: &Content) -> Vec<Ability> {
        (Ability::every(content).into_iter())
            .filter(|a| a.had_by(actor, content))
            .collect()
    }

    /// Whether `actor` has it (whether or not they can use it now): a skill, if they are trained
    /// in it; disarming, if they are trained in Disable device (the skill with the disarm
    /// effect); the shield bash, everyone.
    pub(crate) fn had_by(self, actor: &Actor, content: &Content) -> bool {
        match self {
            Ability::Skill(id) => actor.skills.get(id).is_some(),
            Ability::Disarm => actor.skills.trains_effect(content, Effect::Disarm),
            Ability::ShieldBash => true,
        }
    }

    /// Its id in dumps: `shield_bash`, or the skill's id.
    pub fn id(self, content: &Content) -> &str {
        match self {
            Ability::Skill(id) => &content.skill(id).id,
            Ability::Disarm => "disarm",
            Ability::ShieldBash => "shield_bash",
        }
    }

    /// Its name in the menu.
    pub fn name(self, content: &Content) -> &str {
        match self {
            Ability::Skill(id) => &content.skill(id).name,
            Ability::Disarm => "Disarm a trap",
            Ability::ShieldBash => "Shield bash",
        }
    }

    /// What it does, in a line.
    pub fn description(self, content: &Content) -> &str {
        match self {
            Ability::Skill(id) => &content.skill(id).description,
            Ability::Disarm => {
                "Take apart a trap you know of next to you; a clumsy try sets it off."
            }
            Ability::ShieldBash => "Strike someone next to you with the shield you hold.",
        }
    }

    /// Where it comes from: its group in the menu.
    pub const fn source(self) -> Source {
        match self {
            Ability::Skill(_) | Ability::Disarm => Source::Skill,
            Ability::ShieldBash => Source::WeaponSkill,
        }
    }

    /// What it asks for before it is used.
    pub const fn aim(self) -> Aim {
        match self {
            Ability::Skill(_) => Aim::Nothing,
            Ability::Disarm | Ability::ShieldBash => Aim::Direction,
        }
    }

    /// What it costs, as an action of this kind (`data/actions.toml`; the rules then price it as
    /// that action, spec §8 "Time"); `None` for one with no use yet.
    pub const fn cost(self) -> Option<ActionKind> {
        match self {
            Ability::Skill(_) => None,
            Ability::Disarm => Some(ActionKind::Disarm),
            Ability::ShieldBash => Some(ActionKind::Melee),
        }
    }

    /// The action it is, aimed `dir` when it asks for a direction; `None` for one with no use
    /// yet, or one that asks for a direction without it.
    pub const fn action(self, dir: Option<Dir>) -> Option<Action> {
        match (self, dir) {
            (Ability::Skill(_), _) => None,
            (Ability::Disarm, Some(dir)) => Some(Action::Disarm(dir)),
            (Ability::ShieldBash, Some(dir)) => Some(Action::Bash(dir)),
            (Ability::Disarm | Ability::ShieldBash, None) => None,
        }
    }

    /// Why `actor` cannot use it now; `None` when it can. Its own reasons come first, then a
    /// status of the actor's that blocks the kind of action it costs ([`Unavailable::While`]).
    pub(crate) fn unavailable(self, actor: &Actor, content: &Content) -> Option<Unavailable> {
        let own = match self {
            Ability::Skill(_) => Some(Unavailable::NotYet),
            Ability::Disarm => None,
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

/// Why `actor` cannot use an ability that asks for alignment `required` (its data's
/// `requires_alignment`; `None`: any): [`Unavailable::Strayed`] while their band falls short of
/// it (spec §8 "Alignment"). Every ability that has a requirement asks this first in
/// [`Ability::unavailable`].
pub fn strayed(
    actor: &Actor,
    content: &Content,
    required: Option<Requirement>,
) -> Option<Unavailable> {
    let band = actor.band(content);
    (!required?.allows(band)).then_some(Unavailable::Strayed)
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
        // The skills used as actions, in skill order (Disable device as its use, disarming),
        // then the shield bash.
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
                "disarm",
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
        assert_eq!(every.last(), Some(&Ability::ShieldBash));
    }

    #[test]
    fn an_ability_that_asks_for_an_alignment_is_unavailable_to_one_who_strayed() {
        use crate::alignment::{Band, Score};
        use crate::character::CharacterSpec;
        let content = std::sync::Arc::new(Content::builtin().expect("content"));
        let paladin: CharacterSpec = "human,male,paladin".parse().unwrap();
        let world = crate::World::with_character(content.clone(), "showcase", None, Some(&paladin))
            .expect("world");
        // A test ability of a paladin's: Good or better.
        let gift = Some(Requirement {
            min: Some(Band::Good),
            max: None,
        });
        let mut me = world.state().player().clone();
        assert_eq!(me.band(&content), Band::Good, "a paladin starts Good");
        assert_eq!(strayed(&me, &content, gift), None);
        assert_eq!(strayed(&me, &content, None), None, "no requirement");
        // Fallen to Kind: kept, but greyed out.
        me.alignment = Some(Score::new(200));
        assert_eq!(strayed(&me, &content, gift), Some(Unavailable::Strayed));
        assert_eq!(Unavailable::Strayed.reason(&content), "you have strayed");
        assert_eq!(Unavailable::Strayed.id(&content), "strayed");
        // Climbed back: usable again.
        me.alignment = Some(Score::new(800));
        assert_eq!(strayed(&me, &content, gift), None);
        // A dark gift, Callous or worse, the other way round.
        let dark = Some(Requirement {
            min: None,
            max: Some(Band::Callous),
        });
        assert_eq!(strayed(&me, &content, dark), Some(Unavailable::Strayed));
        me.alignment = Some(Score::new(-500));
        assert_eq!(strayed(&me, &content, dark), None);
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
    fn disarming_is_disable_devices_use_and_asks_for_a_direction() {
        let content = Content::builtin().expect("content");
        let disable = (content.skills().iter())
            .find(|s| s.id == "disable_device")
            .expect("Disable device");
        assert_eq!(Ability::use_of(disable), Some(Ability::Disarm));
        let disarm = Ability::Disarm;
        assert_eq!(disarm.aim(), Aim::Direction);
        assert_eq!(disarm.source(), Source::Skill);
        assert_eq!(disarm.cost(), Some(ActionKind::Disarm));
        assert_eq!(disarm.action(Some(Dir::W)), Some(Action::Disarm(Dir::W)));
        assert_eq!(disarm.action(None), None, "not without a direction");
        // Its skill is listed as it, not as a skill with no use yet.
        let every = Ability::every(&content);
        assert!(every.contains(&Ability::Disarm));
        let listed_as_skill = every
            .iter()
            .any(|a| matches!(a, Ability::Skill(id) if content.skill(*id).id == "disable_device"));
        assert!(!listed_as_skill);
    }
}
