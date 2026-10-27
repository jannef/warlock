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
use crate::content::{
    ActionKind, ClassActive, Content, Offer, ResourceCost, ResourceId, SignActive, SkillDef,
    SpellId, StatusId, TalentEffect,
};
use crate::geom::{Dir, Pos};
use crate::skills::{Effect, SkillId};
use crate::world::{Actor, BREATH_RANGE, LevelState, STARE_RANGE, VOICE_RANGE};
use std::borrow::Cow;

/// An ability, as the abilities menu lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ability {
    /// Throw your voice (spec §5 "Hearing"; #235): [`Action::ThrowVoice`] at a cell the player
    /// sees. The bard's (design §4.8, at level 32): had by whoever holds a talent with the
    /// `throw_voice` effect.
    ThrowVoice,
    /// A birth sign's active gift (spec §8 "Birth sign"): [`Action::Gift`], had by whoever's
    /// sign grants its talent; unavailable while its cooldown lasts.
    Sign(SignActive),
    /// A class's active (spec §8 "Class abilities"): [`Action::Use`], had by whoever's class
    /// gives it at their level; unavailable while its cooldown lasts (Rage: while raging and
    /// exhausted), and, for the paladin's gifts, while they have strayed from Good.
    Class(ClassActive),
    /// A caster's spell (spec §8 "Class abilities", "Spells"): [`Action::Cast`], had by
    /// whoever's class gives the ability that names it; unavailable without the mana it costs
    /// (that check needs the caster level: [`spell_unavailable`]).
    Spell(SpellId),
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
    /// Pick the lock of a door next to the actor (spec §8 "Doors"): [`Action::PickLock`], the
    /// use of Open lock (the skill with the pick effect, listed as this in its place). Had by
    /// whoever is trained in it; usable whenever they have it (the rules refuse, taking no time,
    /// where there is no lock).
    PickLock,
    /// Steal from someone next to you (spec §8 "Stealing"): [`Action::Steal`], the Steal
    /// skill's use. Whoever is trained in a skill with the `steal` benefit has it, in that
    /// skill's place among the skills.
    Steal,
    /// Forage for food in the wild (spec §8 "Foraging"): [`Action::Forage`], Survival's use (the
    /// skill with the `forage` benefit, listed as this in its place, while `data/forage.toml`
    /// exists). Had by whoever is trained in it; unavailable ([`Unavailable::NotHere`]) anywhere
    /// but on land of the overworld or a local area of it.
    Forage,
    /// Treat oneself (spec §8 "Skills": Heal): [`Action::Heal`], the Heal skill's use (the skill
    /// with the `heal` effect, listed as this in its place). Had by whoever is trained in it;
    /// unavailable while there is nothing to treat ([`crate::heal::treatable`]). It asks which
    /// ailment when there are several ([`Aim::Ailment`]).
    Heal,
    /// Catch a second wind (spec §8 "Talents"): [`Action::SecondWind`], the active of the
    /// Second wind talent (a talent with the `second_wind` effect). Had by whoever holds it;
    /// unavailable while its cooldown (Winded) lasts.
    SecondWind,
    /// Blood magic (spec §8 "Talents"): [`Action::BloodMagic`], the active of the Blood magic
    /// talent (a talent with the `blood_magic` effect): hit points given for mana. Had by
    /// whoever holds it; usable whenever they have it (the rules refuse, taking no time, when
    /// it would cost their last hit point or their mana is full).
    BloodMagic,
    /// Bash with the shield held (spec §8 "Weapon skills"): [`Action::Bash`]. Everyone can bash
    /// with a shield (at shield bashing's rank 0 too), so everyone has it; it is available while
    /// a shield to bash with is held.
    ShieldBash,
    /// Lower or raise the lantern's hood (spec §5 "Light sources"; #235): [`Action::HoodLantern`].
    /// Listed with what the player carries, the lantern; the Hooded light talent (design §7.5)
    /// will give it, and until then everyone has it (without a lantern the rules say so).
    HoodLantern,
    /// Pray at a forgotten altar (spec §7 "Special rooms"): [`Action::Pray`]. A place offers
    /// it ([`Ability::here`]): listed only while the player stands next to such an altar (never
    /// a spent one), absent elsewhere.
    Pray,
}

/// Where an ability comes from: the abilities menu's groups, in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Source {
    /// The character's race (none yet).
    Race,
    /// The character's birth sign (spec §8 "Birth sign"): its active gifts.
    Sign,
    /// The character's class (spec §8 "Class abilities"): its actives, and throwing one's voice
    /// (everyone's until the class tables give it to the bard alone).
    Class,
    /// The spells the character casts (spec §8 "Spells"), paid in mana.
    Spell,
    /// A talent the character took (spec §8 "Talents"): its active ones.
    Talent,
    /// A trained skill used as an action (spec §8 "Skills").
    Skill,
    /// A weapon skill or a fighting style (spec §8 "Weapon skills", "Fighting styles").
    WeaponSkill,
    /// Something worn or carried (the lantern's hood).
    Item,
    /// The place the character stands in (spec §7 "Special rooms"): what a special room's
    /// feature offers next to it (Pray at a forgotten altar).
    Place,
}

impl Source {
    /// Every source, in menu order.
    pub const ALL: &[Source] = &[
        Source::Race,
        Source::Sign,
        Source::Class,
        Source::Spell,
        Source::Talent,
        Source::Skill,
        Source::WeaponSkill,
        Source::Item,
        Source::Place,
    ];

    /// Its group's heading in the abilities menu.
    pub const fn name(self) -> &'static str {
        match self {
            Source::Race => "Racial",
            Source::Sign => "Birth sign",
            Source::Class => "Class",
            Source::Spell => "Spells",
            Source::Talent => "Talents",
            Source::Skill => "Skills",
            Source::WeaponSkill => "Weapon skills",
            Source::Item => "Items",
            Source::Place => "Here",
        }
    }

    /// Its id in dumps.
    pub const fn id(self) -> &'static str {
        match self {
            Source::Race => "race",
            Source::Sign => "sign",
            Source::Class => "class",
            Source::Spell => "spell",
            Source::Talent => "talent",
            Source::Skill => "skill",
            Source::WeaponSkill => "weapon_skill",
            Source::Item => "item",
            Source::Place => "place",
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
    /// What to treat, of what the player may treat now ([`crate::heal::treatable`]): asked
    /// only when there is more than one (the Heal ability).
    Ailment,
    /// A stack the player carries, on the identify list (the Identify spell,
    /// [`Ability::action_on`]).
    Item,
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
    /// One that costs an alternative resource the character does not hold, or holds too little
    /// of (spec §8 "Resources"): "not enough focus".
    Short(ResourceId),
    /// One that costs more mana than the actor has (spec §8 "Mana"): time brings it back.
    NoMana,
    /// One the place does not allow: foraging anywhere but in the wild (spec §8 "Foraging").
    NotHere,
    /// The Heal ability with nothing to treat: no bleeding, and no poison or disease that may
    /// still be tried.
    NothingToTreat,
}

impl Unavailable {
    /// The reason, as the menu says it: "you are winded".
    pub fn reason(self, content: &Content) -> Cow<'static, str> {
        match self {
            Unavailable::NoShield => "you hold no shield".into(),
            Unavailable::NotYet => "not usable yet".into(),
            Unavailable::Strayed => "you have strayed".into(),
            Unavailable::NoMana => "you have too little mana".into(),
            Unavailable::NotHere => "not usable here".into(),
            Unavailable::NothingToTreat => "nothing to treat".into(),
            Unavailable::While(status) => {
                format!("you are {}", content.status(status).name.to_lowercase()).into()
            }
            Unavailable::Short(resource) => {
                let name = content.resource(resource).name.to_lowercase();
                format!("not enough {name}").into()
            }
        }
    }

    /// Its id in dumps: `no_shield`, `while:winded`.
    pub fn id(self, content: &Content) -> Cow<'static, str> {
        match self {
            Unavailable::NoShield => "no_shield".into(),
            Unavailable::NotYet => "not_yet".into(),
            Unavailable::Strayed => "strayed".into(),
            Unavailable::NoMana => "no_mana".into(),
            Unavailable::NotHere => "not_here".into(),
            Unavailable::NothingToTreat => "nothing_to_treat".into(),
            Unavailable::While(status) => format!("while:{}", content.status(status).id).into(),
            Unavailable::Short(resource) => {
                format!("short:{}", content.resource(resource).id).into()
            }
        }
    }
}

impl Ability {
    /// Every ability there is, in menu order: by [`Source`], then as each source lists its own.
    /// The one place abilities are registered (the wiki's table lists these).
    pub fn every(content: &Content) -> Vec<Ability> {
        // Skills: those used as actions, in skill order, each as its use once that is in the game,
        // and a passive skill with a use of its own (Survival's Forage) as that use.
        let skills = (content.skills().iter().enumerate()).filter_map(|(i, def)| {
            let id = SkillId(u8::try_from(i).ok()?);
            match Ability::use_of(def, content) {
                Some(ability) => Some(ability),
                None => def.is_active().then_some(Ability::Skill(id)),
            }
        });
        // Birth sign: the active gifts some sign grants, in their order.
        let signs = (SignActive::ALL.iter())
            .filter(|&&a| content.sign_active(a).is_some())
            .map(|&a| Ability::Sign(a));
        // Class: throwing one's voice.
        let class = [Ability::ThrowVoice];
        // Talents: the active ones the talent table has.
        let wind = (content.talents().iter())
            .flat_map(|(_, def)| def.effects_at(def.rank_count()))
            .any(|e| matches!(e, TalentEffect::SecondWind { .. }));
        let blood = (content.talents().iter())
            .flat_map(|(_, def)| def.effects_at(def.rank_count()))
            .any(|e| matches!(e, TalentEffect::BloodMagic { .. }));
        let talents = (wind.then_some(Ability::SecondWind))
            .into_iter()
            .chain(blood.then_some(Ability::BloodMagic));
        // Classes: the actives some class's table has, in their order.
        let classes = (ClassActive::ALL.iter())
            .filter(|&&a| content.class_active(a).is_some())
            .map(|&a| Ability::Class(a));
        // Every spell, by tier, then in path order: any may be learned.
        let spells = content.spells_by_tier().into_iter().map(Ability::Spell);
        // Weapon skills and fighting styles: the shield bash.
        let weapon_skills = [Ability::ShieldBash];
        // Items: the lantern's hood.
        let items = [Ability::HoodLantern];
        // Here: what some special room's feature offers.
        let offers = |offer| {
            (content.room_effects().defs().iter())
                .any(|e| e.offers.as_ref().is_some_and(|(o, _)| *o == offer))
        };
        let here = offers(Offer::Pray).then_some(Ability::Pray);
        let mut all: Vec<Ability> = (signs.chain(class))
            .chain(classes)
            .chain(spells)
            .chain(talents)
            .chain(skills)
            .chain(weapon_skills)
            .chain(items)
            .chain(here)
            .collect();
        all.sort_by_key(|a| a.source());
        all
    }

    /// The ability that is the use of skill `def`, once that use is in the game: Disable device's
    /// (the disarm effect's) is [`Ability::Disarm`], Open lock's (the pick effect's)
    /// [`Ability::PickLock`], the Steal skill's (the `steal` benefit, with `data/steal.toml`)
    /// [`Ability::Steal`], Heal's (the heal effect's) [`Ability::Heal`], Survival's (the
    /// `forage` benefit, with `data/forage.toml`) [`Ability::Forage`]. `None` for a skill whose
    /// use is to come, or that has none.
    fn use_of(def: &SkillDef, content: &Content) -> Option<Ability> {
        let gives = |effect| def.benefits.iter().any(|b| b.effect == effect);
        (gives(Effect::Disarm).then_some(Ability::Disarm))
            .or_else(|| gives(Effect::Pick).then_some(Ability::PickLock))
            .or_else(|| gives(Effect::Heal).then_some(Ability::Heal))
            .or_else(|| {
                (gives(Effect::Steal) && content.steal().is_some()).then_some(Ability::Steal)
            })
            .or_else(|| {
                (gives(Effect::Forage) && content.forage().is_some()).then_some(Ability::Forage)
            })
    }

    /// The abilities the place offers to whoever stands at `at` on `level` (spec §7 "Special
    /// rooms"): what the feature on a cell next to it offers (Pray beside a forgotten altar), in
    /// menu order. A spent altar is another terrain, which offers nothing.
    pub(crate) fn here(level: &LevelState, content: &Content, at: Pos) -> Vec<Ability> {
        let mut found: Vec<Ability> = (Dir::ALL.iter().map(|&d| at.step(d)))
            .filter_map(|p| level.terrain_at(p))
            .filter_map(|t| content.offer_at(t))
            .map(|(offer, _)| match offer {
                Offer::Pray => Ability::Pray,
            })
            .collect();
        found.sort();
        found.dedup();
        found
    }

    /// Every ability `actor` has, in menu order.
    pub(crate) fn of(actor: &Actor, content: &Content) -> Vec<Ability> {
        (Ability::every(content).into_iter())
            .filter(|a| a.had_by(actor, content))
            .collect()
    }

    /// Whether `actor` has it (whether or not they can use it now): a skill, if they are trained
    /// in it; disarming, if they are trained in Disable device (the skill with the disarm
    /// effect) or a talent lets them (the craftsperson's Tinker); picking locks, if they are
    /// trained in Open lock (the pick effect's) or a talent lets them (the thief's Pick lock);
    /// stealing, if they are trained in a skill with the `steal` benefit or a talent lets them
    /// (the thief's Light fingers; it is listed only when stealing is in the game); treating
    /// oneself, if they are trained in a skill with the `heal` effect; foraging, if they are
    /// trained in Survival (the forage benefit's); second wind,
    /// if they hold the talent; throwing one's voice, if a talent gives it (the bard's); the
    /// shield bash and the lantern's hood, everyone.
    pub(crate) fn had_by(self, actor: &Actor, content: &Content) -> bool {
        match self {
            Ability::Sign(active) => actor.talent_mods(content).has_sign_active(active),
            Ability::Class(active) => actor.talent_mods(content).has_active(active),
            Ability::Spell(spell) => actor.knows_spell(content, spell),
            Ability::Skill(id) => actor.skills.get(id).is_some(),
            Ability::Disarm => {
                actor.skills.trains_effect(content, Effect::Disarm)
                    || actor.talent_mods(content).disarm
            }
            // The thief's Pick lock and Light fingers give them without the skills.
            Ability::PickLock => {
                actor.skills.trains_effect(content, Effect::Pick)
                    || actor.talent_mods(content).pick_lock
            }
            Ability::Steal => {
                actor.skills.trains_effect(content, Effect::Steal)
                    || actor.talent_mods(content).steal
            }
            Ability::Forage => actor.skills.trains_effect(content, Effect::Forage),
            Ability::Heal => actor.skills.trains_effect(content, Effect::Heal),
            Ability::SecondWind => actor.talent_mods(content).second_wind.is_some(),
            Ability::BloodMagic => actor.talent_mods(content).blood_magic.is_some(),
            Ability::ThrowVoice => actor.talent_mods(content).throw_voice,
            Ability::ShieldBash | Ability::HoodLantern => true,
            // A place offers it, not who one is ([`Ability::here`]).
            Ability::Pray => false,
        }
    }

    /// Its id in dumps: `shield_bash`, `steal`, or the skill's id.
    pub fn id(self, content: &Content) -> &str {
        match self {
            Ability::Sign(active) => active.id(),
            Ability::Class(active) => active.id(),
            Ability::Spell(spell) => content.spell_of(spell).map_or("", |s| &s.id),
            Ability::Skill(id) => &content.skill(id).id,
            Ability::ThrowVoice => "throw_voice",
            Ability::Disarm => "disarm",
            Ability::PickLock => "pick_lock",
            Ability::Steal => "steal",
            Ability::Forage => "forage",
            Ability::Heal => "heal",
            Ability::SecondWind => "second_wind",
            Ability::BloodMagic => "blood_magic",
            Ability::ShieldBash => "shield_bash",
            Ability::HoodLantern => "hood_lantern",
            Ability::Pray => "pray",
        }
    }

    /// Its name in the menu.
    pub fn name(self, content: &Content) -> &str {
        match self {
            Ability::Sign(active) => content.sign_active(active).map_or("", |a| &a.name),
            Ability::Class(active) => content.class_active(active).map_or("", |a| &a.name),
            Ability::Spell(spell) => content.spell_of(spell).map_or("", |s| &s.name),
            Ability::Skill(id) => &content.skill(id).name,
            Ability::ThrowVoice => "Throw your voice",
            Ability::Disarm => "Disarm a trap",
            Ability::PickLock => "Pick lock",
            Ability::Steal => "Steal",
            Ability::Forage => "Forage",
            Ability::Heal => "Heal",
            Ability::SecondWind => "Second wind",
            Ability::BloodMagic => "Blood magic",
            Ability::ShieldBash => "Shield bash",
            Ability::HoodLantern => "Hood your lantern",
            Ability::Pray => "Pray",
        }
    }

    /// What it does, in a line.
    pub fn description(self, content: &Content) -> &str {
        match self {
            Ability::Sign(active) => content.sign_active(active).map_or("", |a| &a.description),
            Ability::Class(active) => content.class_active(active).map_or("", |a| &a.description),
            Ability::Spell(spell) => content.spell_of(spell).map_or("", |s| &s.description),
            Ability::Skill(id) => &content.skill(id).description,
            Ability::ThrowVoice => {
                "Make a voice sound at a place you see nearby: those who hear it go and look there."
            }
            Ability::Disarm => {
                "Take apart a trap you know of next to you; a clumsy try sets it off."
            }
            Ability::PickLock => "Open a locked door next to you without its key.",
            Ability::Steal => {
                "Lift a purse or a trinket from someone next to you who is not hostile; caught, \
                 they turn on you."
            }
            Ability::Forage => {
                "Spend a while searching the wild around you for something to eat: berries, \
                 nuts, mushrooms, roots."
            }
            Ability::Heal => {
                "Tend your own hurts: bind a bleeding wound, or try to ease a poison or a \
                 disease (once each)."
            }
            Ability::SecondWind => {
                "Catch your breath in the thick of it: some of your hit points come back at \
                 once, and then you are winded for a while."
            }
            Ability::BloodMagic => {
                "Give some of your hit points, never the last, to have some of your mana back at \
                 once."
            }
            Ability::ShieldBash => "Strike someone next to you with the shield you hold.",
            Ability::HoodLantern => {
                "Lower your lantern's hood, or raise it: hooded, it dimly lights the ground around \
                 you but not you."
            }
            Ability::Pray => {
                "Kneel at the forgotten altar beside you and pray for a minute. Something may \
                 answer, for good or ill."
            }
        }
    }

    /// Where it comes from: its group in the menu.
    pub const fn source(self) -> Source {
        match self {
            Ability::Sign(_) => Source::Sign,
            Ability::Class(_) => Source::Class,
            Ability::Spell(_) => Source::Spell,
            Ability::Skill(_)
            | Ability::Disarm
            | Ability::PickLock
            | Ability::Steal
            | Ability::Forage
            | Ability::Heal => Source::Skill,
            Ability::ThrowVoice => Source::Class,
            Ability::SecondWind | Ability::BloodMagic => Source::Talent,
            Ability::ShieldBash => Source::WeaponSkill,
            Ability::HoodLantern => Source::Item,
            Ability::Pray => Source::Place,
        }
    }

    /// What it asks for before it is used (a spell: by its shape in `content`).
    pub fn aim(self, content: &Content) -> Aim {
        match self {
            Ability::Sign(active) if active.aimed() => Aim::Cell,
            Ability::Sign(_) => Aim::Nothing,
            Ability::Class(active) if active.aimed() => Aim::Direction,
            Ability::Spell(spell) if aims_item(content, spell) => Aim::Item,
            Ability::Spell(spell) if aimed(content, spell) => Aim::Cell,
            Ability::Spell(_) => Aim::Nothing,
            Ability::Class(active) if active.aims_cell() => Aim::Cell,
            Ability::Class(_)
            | Ability::Skill(_)
            | Ability::SecondWind
            | Ability::BloodMagic
            | Ability::HoodLantern
            | Ability::Forage
            | Ability::Pray => Aim::Nothing,
            Ability::ThrowVoice => Aim::Cell,
            Ability::Heal => Aim::Ailment,
            Ability::Disarm | Ability::PickLock | Ability::Steal | Ability::ShieldBash => {
                Aim::Direction
            }
        }
    }

    /// What it costs, as an action of this kind (`data/actions.toml`; the rules then price it as
    /// that action, spec §8 "Time"); `None` for one with no use yet.
    pub const fn cost(self) -> Option<ActionKind> {
        match self {
            Ability::Sign(active) => Some(active.cost()),
            Ability::Class(active) => Some(active.cost()),
            Ability::Spell(_) => Some(ActionKind::Cast),
            Ability::Skill(_) => None,
            Ability::ThrowVoice => Some(ActionKind::ThrowVoice),
            Ability::Disarm => Some(ActionKind::Disarm),
            Ability::PickLock => Some(ActionKind::PickLock),
            Ability::Steal => Some(ActionKind::Steal),
            Ability::Forage => Some(ActionKind::Forage),
            Ability::Heal => Some(ActionKind::Heal),
            Ability::SecondWind | Ability::BloodMagic => Some(ActionKind::Wait),
            Ability::ShieldBash => Some(ActionKind::Melee),
            Ability::HoodLantern => Some(ActionKind::Hood),
            Ability::Pray => Some(ActionKind::Pray),
        }
    }

    /// How far a cell it aims at may be, in cells (a disc, as ranges are); `None` for one that
    /// aims at no cell.
    pub const fn range(self) -> Option<i32> {
        match self {
            Ability::ThrowVoice => Some(VOICE_RANGE),
            Ability::Sign(SignActive::TransfixingStare) => Some(STARE_RANGE),
            Ability::Sign(SignActive::FireBreath) => Some(BREATH_RANGE),
            Ability::Sign(SignActive::HornsGrace)
            | Ability::Sign(SignActive::Blaze)
            | Ability::Sign(SignActive::HoundsBay)
            | Ability::Class(_)
            | Ability::Spell(_)
            | Ability::Skill(_)
            | Ability::Disarm
            | Ability::PickLock
            | Ability::Steal
            | Ability::Forage
            | Ability::Heal
            | Ability::SecondWind
            | Ability::BloodMagic
            | Ability::ShieldBash
            | Ability::HoodLantern
            | Ability::Pray => None,
        }
    }

    /// How far a cell it aims at may be, in cells, with what `content` says of it: a spell's
    /// is its data's before Willpower ([`crate::content::SpellData::base_reach`]; a caster's
    /// own is [`Actor::spell_reach`]); every other ability's is [`Ability::range`].
    pub fn reach(self, content: &Content) -> Option<i32> {
        match self {
            Ability::Spell(spell) => content.spell_of(spell).and_then(|s| s.base_reach()),
            other => other.range(),
        }
    }

    /// [`Ability::reach`] at `time`: a full moon lengthens the Transfixing stare's (spec §8
    /// "Birth sign", the flavour; [`Content::full_moon_reach`]).
    pub fn reach_at(self, content: &Content, time: crate::time::GameTime) -> Option<i32> {
        let reach = self.reach(content)?;
        Some(match self {
            Ability::Sign(SignActive::TransfixingStare) => reach + content.full_moon_reach(time),
            _ => reach,
        })
    }

    /// The action it is, aimed `dir` when it asks for a direction; `None` for one with no use
    /// yet, one that asks for a direction without it, or one that asks for a cell
    /// ([`Ability::action_at`]).
    pub fn action(self, content: &Content, dir: Option<Dir>) -> Option<Action> {
        match (self, dir) {
            (Ability::Class(active), dir) if active.aimed() => {
                dir.map(|dir| Action::Use(active, Some(dir)))
            }
            (Ability::Class(active), _) if active.aims_cell() => None,
            (Ability::Class(active), _) => Some(Action::Use(active, None)),
            (Ability::Sign(active), _) if active.aimed() => None,
            (Ability::Sign(active), _) => Some(Action::Gift(active, None)),
            (Ability::Spell(spell), _) if aimed(content, spell) || aims_item(content, spell) => {
                None
            }
            (Ability::Spell(spell), _) => Some(Action::Cast(spell_name(content, spell)?, None)),
            (Ability::Skill(_) | Ability::ThrowVoice, _) => None,
            // The first thing there is to treat; the menu names one when there are several.
            (Ability::Heal, _) => Some(Action::Heal(None)),
            (Ability::SecondWind, _) => Some(Action::SecondWind),
            (Ability::BloodMagic, _) => Some(Action::BloodMagic),
            (Ability::Disarm, Some(dir)) => Some(Action::Disarm(dir)),
            (Ability::PickLock, Some(dir)) => Some(Action::PickLock(dir)),
            (Ability::Steal, Some(dir)) => Some(Action::Steal(dir)),
            (Ability::ShieldBash, Some(dir)) => Some(Action::Bash(dir)),
            (Ability::Disarm | Ability::PickLock | Ability::Steal | Ability::ShieldBash, None) => {
                None
            }
            (Ability::HoodLantern, _) => Some(Action::HoodLantern),
            (Ability::Pray, _) => Some(Action::Pray),
            (Ability::Forage, _) => Some(Action::Forage),
        }
    }

    /// The action it is, cast on inventory stack `item`, for one that asks for a stack the
    /// player carries (Identify); `None` for any other.
    pub fn action_on(self, content: &Content, item: usize) -> Option<Action> {
        match self {
            Ability::Spell(spell) if aims_item(content, spell) => {
                Some(Action::CastOn(spell_name(content, spell)?, item))
            }
            _ => None,
        }
    }

    /// The action it is, aimed at level cell `at`, for one that asks for a cell; `None` for any
    /// other.
    pub fn action_at(self, content: &Content, at: Pos) -> Option<Action> {
        match self {
            Ability::ThrowVoice => Some(Action::ThrowVoice(at)),
            Ability::Sign(active) if active.aimed() => Some(Action::Gift(active, Some(at))),
            Ability::Spell(spell) if aimed(content, spell) => {
                Some(Action::Cast(spell_name(content, spell)?, Some(at)))
            }
            Ability::Class(active) if active.aims_cell() => Some(Action::UseAt(active, at)),
            Ability::Sign(_)
            | Ability::Spell(_)
            | Ability::Class(_)
            | Ability::Skill(_)
            | Ability::Disarm
            | Ability::PickLock
            | Ability::Steal
            | Ability::Forage
            | Ability::Heal
            | Ability::SecondWind
            | Ability::BloodMagic
            | Ability::ShieldBash
            | Ability::HoodLantern
            | Ability::Pray => None,
        }
    }

    /// Why `actor` cannot use it now; `None` when it can. Its own reasons come first, then a
    /// status of the actor's that blocks the kind of action it costs ([`Unavailable::While`]).
    pub(crate) fn unavailable(self, actor: &Actor, content: &Content) -> Option<Unavailable> {
        let own = match self {
            Ability::Class(active) => {
                let def = content.class_active(active);
                let required = def.and_then(|a| a.needs.alignment);
                (strayed(actor, content, required))
                    .or_else(|| class_cooldown(actor, content, active).map(Unavailable::While))
                    .or_else(|| short_of(actor, def?.resource?))
                    .or_else(|| {
                        // Inspire costs its inspiration while that is held, else mana
                        // (Virtuoso's none), which a status that blocks casting (Raging) keeps
                        // from being spent.
                        let inspire = (active == ClassActive::Inspire)
                            .then(|| actor.talent_mods(content).inspiring())
                            .flatten();
                        let inspired = (inspire.and_then(|i| i.instead))
                            .is_some_and(|c| short_of(actor, c).is_none());
                        let mana = inspire.filter(|_| !inspired).map_or(0, |i| i.mana);
                        // Consecrate costs its mana.
                        let consecrate = (active == ClassActive::Consecrate)
                            .then(|| actor.talent_mods(content).consecrate)
                            .flatten();
                        let mana = mana + consecrate.map_or(0, |c| c.mana);
                        let cost = actor.mana_cost(content, mana);
                        let blocked = (cost > 0)
                            .then(|| actor.effects.blocking(content, ActionKind::Cast))
                            .flatten();
                        (blocked.map(Unavailable::While))
                            .or((actor.mana < cost).then_some(Unavailable::NoMana))
                    })
            }
            // Its mana is asked with the caster level ([`spell_unavailable`]).
            Ability::Spell(_) => None,
            Ability::Sign(active) => (actor.talent_mods(content).sign_cooldown(active))
                .filter(|&cooldown| actor.effects.has(cooldown))
                .map(Unavailable::While),
            Ability::Skill(_) => Some(Unavailable::NotYet),
            Ability::Heal => (crate::heal::treatable(actor, content).is_empty())
                .then_some(Unavailable::NothingToTreat),
            Ability::ThrowVoice
            | Ability::HoodLantern
            | Ability::Disarm
            | Ability::PickLock
            | Ability::Steal
            | Ability::Forage
            | Ability::BloodMagic
            | Ability::Pray => None,
            Ability::SecondWind => (actor.talent_mods(content).second_wind)
                .map(|w| w.cooldown)
                .filter(|&cooldown| actor.effects.has(cooldown))
                .map(Unavailable::While),
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

/// The status of `actor`'s that keeps it from using class active `active` now: its cooldown,
/// or for Rage the rage itself and what follows it.
fn class_cooldown(actor: &Actor, content: &Content, active: ClassActive) -> Option<StatusId> {
    let mods = actor.talent_mods(content);
    let cooldowns: Vec<StatusId> = match active {
        ClassActive::Rage => (mods.rage_status(content).into_iter())
            .flat_map(|s| [Some(s), content.status(s).follows])
            .flatten()
            .collect(),
        ClassActive::VeteransWind => mods.veterans_wind.map(|w| w.cooldown).into_iter().collect(),
        ClassActive::LayOnHands => mods.lay_on_hands.map(|h| h.cooldown).into_iter().collect(),
        ClassActive::Smite => mods.smite.map(|s| s.cooldown).into_iter().collect(),
        // Volley costs Focus, Master thief Openings, and Quivering palm Ki instead (their
        // talents' `resource`, [`short_of`]).
        ClassActive::Whirlwind
        | ClassActive::HolyRadiance
        | ClassActive::QuiveringPalm
        | ClassActive::MasterThief
        | ClassActive::Inspire
        | ClassActive::Consecrate
        | ClassActive::Volley => Vec::new(),
    };
    cooldowns.into_iter().find(|&s| actor.effects.has(s))
}

/// Whether `spell` asks for a cell before it is cast: anything but a spell on oneself.
fn aimed(content: &Content, spell: SpellId) -> bool {
    content.spell_of(spell).is_some_and(|s| s.aimed())
}

/// Whether `spell` is cast on a stack the caster carries (Identify).
fn aims_item(content: &Content, spell: SpellId) -> bool {
    content.spell_of(spell).is_some_and(|s| s.aims_item())
}

/// The id `spell` is cast by (`use:ID`), if the content has it.
fn spell_name(content: &Content, spell: SpellId) -> Option<String> {
    content.spell_of(spell).map(|s| s.id.clone())
}

/// Why `actor`, of caster level `caster_level`, cannot cast `spell` now: what
/// [`Ability::unavailable`] says, else [`Unavailable::NoMana`] while it has less mana than the
/// spell costs it ([`Actor::spell_cost`]).
pub fn spell_unavailable(
    actor: &Actor,
    content: &Content,
    spell: SpellId,
    caster_level: i32,
) -> Option<Unavailable> {
    let ability = Ability::Spell(spell);
    ability.unavailable(actor, content).or_else(|| {
        let def = content.spell_of(spell)?;
        (actor.mana < actor.spell_cost(content, def, caster_level)).then_some(Unavailable::NoMana)
    })
}

/// Why `actor` cannot use an ability that costs `cost` (spec §8 "Resources"):
/// [`Unavailable::Short`] while they do not hold the resource or hold less than it costs.
fn short_of(actor: &Actor, cost: ResourceCost) -> Option<Unavailable> {
    let held = actor.resources.get(cost.resource).map(|p| p.value);
    (held.is_none_or(|v| v < cost.amount)).then_some(Unavailable::Short(cost.resource))
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
            [
                "Racial",
                "Birth sign",
                "Class",
                "Spells",
                "Talents",
                "Skills",
                "Weapon skills",
                "Items",
                "Here"
            ]
        );
    }

    #[test]
    fn every_ability_is_registered_in_menu_order() {
        let content = Content::builtin().expect("content");
        let every = Ability::every(&content);
        assert!(every.windows(2).all(|w| w[0].source() <= w[1].source()));
        assert_eq!(
            every.first(),
            Some(&Ability::Sign(SignActive::HornsGrace)),
            "the birth sign's first"
        );
        let class = every.iter().find(|a| a.source() == Source::Class);
        assert_eq!(class, Some(&Ability::ThrowVoice), "the class's first");
        // The skills used as actions, in skill order (Disable device and Open lock as their uses,
        // disarming and picking locks; Survival, a passive skill, as its use, foraging), then the
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
                "disarm",
                "handle_animal",
                "heal",
                "intimidate",
                "knowledge",
                "metalcraft",
                "music",
                "pick_lock",
                "steal",
                "forage",
                "woodcraft",
            ]
        );
        let last: Vec<Ability> = every.iter().rev().take(3).rev().copied().collect();
        assert_eq!(
            last,
            [Ability::ShieldBash, Ability::HoodLantern, Ability::Pray],
            "then what a place offers"
        );
        // Stealing is in the game: the Steal skill is listed as its use.
        assert!(every.contains(&Ability::Steal));
        let steal = content.skill_id("steal").expect("steal");
        assert!(!every.contains(&Ability::Skill(steal)));
    }

    #[test]
    fn stealing_asks_for_a_direction_and_is_had_by_those_trained_in_it() {
        let content = std::sync::Arc::new(Content::builtin().expect("content"));
        let steal = Ability::Steal;
        assert_eq!(steal.aim(&content), Aim::Direction);
        assert_eq!(steal.source(), Source::Skill);
        assert_eq!(steal.cost(), Some(ActionKind::Steal));
        assert_eq!(
            steal.action(&content, Some(Dir::N)),
            Some(Action::Steal(Dir::N))
        );
        assert_eq!(
            steal.action(&content, None),
            None,
            "not without a direction"
        );
        assert_eq!(steal.id(&content), "steal");
        // The showcase's thief is trained in Steal; a warrior is not.
        let thief = crate::World::new(content.clone(), "showcase", None).expect("thief");
        let mine = thief.perception().abilities();
        assert!(
            mine.iter()
                .any(|a| a.ability == steal && a.unavailable.is_none())
        );
        let spec = "human,male,warrior".parse().expect("spec");
        let warrior =
            crate::World::with_character(content, "showcase", None, Some(&spec)).expect("warrior");
        let theirs = warrior.perception().abilities();
        assert!(!theirs.iter().any(|a| a.ability == steal));
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
        // A paladin's own start, before the stars (the showcase's is Basilisk-born).
        me.alignment = Some(Score::new(450));
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
        let content = Content::builtin().expect("content");
        let bash = Ability::ShieldBash;
        assert_eq!(bash.aim(&content), Aim::Direction);
        assert_eq!(bash.source(), Source::WeaponSkill);
        assert_eq!(bash.cost(), Some(ActionKind::Melee));
        assert_eq!(
            bash.action(&content, Some(Dir::E)),
            Some(Action::Bash(Dir::E))
        );
        assert_eq!(bash.action(&content, None), None, "not without a direction");
        assert_eq!(
            Ability::Skill(SkillId(0)).action(&content, Some(Dir::E)),
            None
        );
        assert_eq!(bash.range(), None);
        assert_eq!(bash.action_at(&content, Pos::new(3, 4)), None);
    }

    #[test]
    fn throwing_a_voice_aims_at_a_cell_and_the_hood_at_nothing() {
        let content = Content::builtin().expect("content");
        let voice = Ability::ThrowVoice;
        assert_eq!(
            (voice.aim(&content), voice.source()),
            (Aim::Cell, Source::Class)
        );
        assert_eq!(voice.cost(), Some(ActionKind::ThrowVoice));
        assert_eq!(voice.range(), Some(VOICE_RANGE));
        assert_eq!(voice.action(&content, None), None, "not without a cell");
        let at = Pos::new(3, 4);
        assert_eq!(voice.action_at(&content, at), Some(Action::ThrowVoice(at)));
        let hood = Ability::HoodLantern;
        assert_eq!(
            (hood.aim(&content), hood.source()),
            (Aim::Nothing, Source::Item)
        );
        assert_eq!(hood.cost(), Some(ActionKind::Hood));
        assert_eq!(hood.action(&content, None), Some(Action::HoodLantern));
        assert_eq!((hood.range(), hood.action_at(&content, at)), (None, None));
    }

    #[test]
    fn disarming_is_disable_devices_use_and_asks_for_a_direction() {
        let content = Content::builtin().expect("content");
        let disable = (content.skills().iter())
            .find(|s| s.id == "disable_device")
            .expect("Disable device");
        assert_eq!(Ability::use_of(disable, &content), Some(Ability::Disarm));
        let disarm = Ability::Disarm;
        assert_eq!(disarm.aim(&content), Aim::Direction);
        assert_eq!(disarm.source(), Source::Skill);
        assert_eq!(disarm.cost(), Some(ActionKind::Disarm));
        assert_eq!(
            disarm.action(&content, Some(Dir::W)),
            Some(Action::Disarm(Dir::W))
        );
        assert_eq!(
            disarm.action(&content, None),
            None,
            "not without a direction"
        );
        // Its skill is listed as it, not as a skill with no use yet.
        let every = Ability::every(&content);
        assert!(every.contains(&Ability::Disarm));
        let listed_as_skill = every
            .iter()
            .any(|a| matches!(a, Ability::Skill(id) if content.skill(*id).id == "disable_device"));
        assert!(!listed_as_skill);
    }

    #[test]
    fn picking_locks_is_open_locks_use_and_asks_for_a_direction() {
        let content = Content::builtin().expect("content");
        let open_lock = (content.skills().iter())
            .find(|s| s.id == "open_lock")
            .expect("Open lock");
        assert_eq!(
            Ability::use_of(open_lock, &content),
            Some(Ability::PickLock)
        );
        let pick = Ability::PickLock;
        assert_eq!(pick.aim(&content), Aim::Direction);
        assert_eq!(pick.source(), Source::Skill);
        assert_eq!(pick.cost(), Some(ActionKind::PickLock));
        assert_eq!(
            pick.action(&content, Some(Dir::W)),
            Some(Action::PickLock(Dir::W))
        );
        assert_eq!(pick.action(&content, None), None, "not without a direction");
    }
}
