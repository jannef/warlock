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
//! Alignment never makes an ability unavailable (spec §8 "Alignment"): a holy gift whose band
//! the user is outside of is listed and used like any other, and does nothing
//! (`World::unanswered`).
//!
//! [`PerceptionView::abilities`]: crate::perception::PerceptionView::abilities

use crate::action::{Action, Command};
use crate::content::{
    ActionKind, ActiveAim, ClassActive, Content, Offer, ResourceCost, ResourceId, SignActive,
    SkillDef, SpellId, StatusId, TalentActive, TalentEffect, WeaponAbilityId, WeaponSkillId,
};
use crate::geom::{Dir, Pos};
use crate::skills::{Effect, SkillId};
use crate::talent::SongCost;
use crate::world::{Actor, BREATH_RANGE, LevelState, STARE_RANGE, VOICE_RANGE};
use std::borrow::Cow;

/// An ability, as the abilities menu lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ability {
    /// Throw your voice (spec §5 "Hearing"; #235): [`Action::ThrowVoice`] at a cell the player
    /// sees. The bard's (design §4.8, at level 16): had by whoever holds a talent with the
    /// `throw_voice` effect.
    ThrowVoice,
    /// A birth sign's active gift (spec §8 "Birth sign"): [`Action::Gift`], had by whoever's
    /// sign grants its talent; unavailable while its cooldown lasts.
    Sign(SignActive),
    /// A class's active (spec §8 "Class abilities"): [`Action::Use`], had by whoever's class
    /// gives it at their level; unavailable while its cooldown lasts (Rage: while raging and
    /// exhausted).
    Class(ClassActive),
    /// A caster's spell (spec §8 "Class abilities", "Spells"): [`Action::Cast`], had by
    /// whoever's class gives the ability that names it; unavailable without the mana it costs
    /// (that check is [`spell_unavailable`]'s).
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
    /// Listen at a closed door next to the actor (spec §5 "Hearing"; `world/listen.rs`):
    /// [`Action::Listen`], the use of Listen (the skill with the hearing effect, listed as this
    /// in its place). Had by whoever is trained in it; usable whenever they have it (the rules
    /// refuse, taking no time, where there is no closed door).
    Listen,
    /// Pick the lock of a door next to the actor (spec §8 "Doors"): [`Action::PickLock`], the
    /// use of Open lock (the skill with the pick effect, listed as this in its place). Had by
    /// whoever is trained in it; usable whenever they have it (the rules refuse, taking no time,
    /// where there is no lock).
    PickLock,
    /// Steal from someone next to you (spec §8 "Stealing"): [`Action::Steal`], the Steal
    /// skill's use. Whoever is trained in a skill with the `steal` benefit has it, in that
    /// skill's place among the skills.
    Steal,
    /// Wedge shut the closed door next to the actor (spec §8 "Doors"; `world/wedge.rs`):
    /// [`Action::Wedge`], Woodcraft's second use (the skill with the `wedge` effect). Had by
    /// whoever is trained in it; usable whenever they have it (the rules refuse, taking no time,
    /// where there is no closed door).
    Wedge,
    /// Feint at a foe next to you (spec §8 "Skills": Bluff; #1016): [`Action::Feint`], Bluff's
    /// use (the skill with the `feint` benefit, listed as this in its place while
    /// `data/feint.toml` exists). Had by whoever is trained in it; usable whenever they have it
    /// (the rules refuse, taking no time, where there is no foe with a mind to fool).
    Feint,
    /// Tame a beast next to you (spec §8 "Allies": Tame; `world/tame.rs`): [`Action::Tame`],
    /// Handle animal's use (the skill with the `tame` benefit, listed as this in its place while
    /// `data/companions.toml` and `data/allies.toml` exist). Had by whoever is trained in it, at
    /// any level; unavailable ([`Unavailable::HaveCompanion`]) while the player has a companion
    /// (the perception knows); otherwise the rules refuse, taking no time, without a beast there
    /// that may be tamed now.
    Tame,
    /// Threaten a foe you see (spec §8 "Skills": Intimidate; #868): [`Action::Intimidate`] at
    /// its cell, the Intimidate skill's use (the skill with the `intimidate` benefit, listed as
    /// this in its place while `data/intimidate.toml` exists). Had by whoever is trained in it;
    /// usable whenever they have it (the rules refuse, taking no time, without a foe there).
    Intimidate,
    /// Make arrows or bolts (spec §8 "Fletching"): [`Action::Fletch`], in Woodcraft's place,
    /// though anyone has it.
    Fletch,
    /// Forage for food in the wild (spec §8 "Foraging"): [`Action::Forage`], Survival's use (the
    /// skill with the `forage` benefit, listed as this in its place, while `data/forage.toml`
    /// exists). Had by whoever is trained in it; unavailable ([`Unavailable::NotHere`]) anywhere
    /// but on land of the overworld or a local area of it.
    Forage,
    /// Treat oneself or an ally beside one (spec §8 "Skills": Heal): [`Action::Heal`], the Heal
    /// skill's use (the skill with the `heal` effect, listed as this in its place). Had by
    /// whoever is trained in it; unavailable while there is nothing to treat on either
    /// ([`crate::heal::treatable`]). It asks which ailment when there are several
    /// ([`Aim::Ailment`]).
    Heal,
    /// Make out a scroll without reading it (spec §8 "Scrolls", "Skills": Decipher script;
    /// `world/decipher.rs`): [`Action::Decipher`] on a scroll of a type not yet identified, the
    /// skill's use (the skill with the `decipher` effect, listed as this in its place while
    /// `data/decipher.toml` exists). Had by whoever is trained in it; unavailable
    /// ([`Unavailable::NoUnknownScroll`], the perception knows) while the pack holds none. It
    /// asks which scroll ([`Aim::Item`]).
    Decipher,
    /// Catch a second wind (spec §8 "Talents"): [`Action::UseTalent`], the active of the
    /// Second wind talent (a talent with the `second_wind` effect). Had by whoever holds it;
    /// unavailable while its cooldown (Winded) lasts.
    SecondWind,
    /// Blood magic (spec §8 "Talents"): [`Action::UseTalent`], the active of the Blood magic
    /// talent (a talent with the `blood_magic` effect): hit points given for mana. Had by
    /// whoever holds it; usable whenever they have it (the rules refuse, taking no time, when
    /// it would cost their last hit point or their mana is full).
    BloodMagic,
    /// Bash with the shield held (spec §8 "Weapon skills"): [`Action::Bash`]. Everyone can bash
    /// with a shield (at shield bashing's rank 0 too), so everyone has it; it is available while
    /// a shield to bash with is held.
    ShieldBash,
    /// A weapon ability (spec §8 "Weapon skills", "Fighting styles"; `data/weapon_abilities/`),
    /// listed under the weapon skill or style named first whose rank teaches it ("Long blades:
    /// Deep cut"). Had by whoever's ranks teach it ([`Actor::weapon_abilities`]); a triggered one
    /// fires by itself and is never used ([`Unavailable::FiresByItself`]). An active one (#564)
    /// is used as [`Action::Act`] (aimed in a direction) or [`Action::Cast`] (aimed at nothing or
    /// a cell; the rules read a `use:ID` that names no spell as an active), for its stamina and
    /// time; unavailable while what the actor holds is not for it ([`Unavailable::NotSoHeld`])
    /// or its stamina is short ([`Unavailable::Short`]).
    Weapon(WeaponSkillId, WeaponAbilityId),
    /// Pray at a forgotten altar (spec §7 "Special rooms"): [`Action::Pray`]. A place offers
    /// it ([`Ability::here`]): listed only while the player stands next to such an altar (never
    /// a spent one), absent elsewhere.
    Pray,
    /// Work a carried metal weapon or armour at a forge's anvil (spec §7 "Special rooms"):
    /// [`Action::Temper`]. A place offers it ([`Ability::here`]): listed only while the player
    /// stands next to an anvil; it asks which item ([`Aim::Item`]).
    Temper,
    /// Drink at a healing spring (spec §7 "Special rooms"): [`Action::Drink`]. A place offers
    /// it, as Pray: listed only while the player stands next to a spring.
    Drink,
    /// Come to heel (spec §8 "Allies"): [`Action::Order`] with [`Command::Heel`], every ally of
    /// the player's on the level follows. The Allies group ([`Ability::orders`]): listed while
    /// the player has an ally anywhere, unavailable ([`Unavailable::NoAllies`]) with none on the
    /// level. No time, a voice's noise.
    Heel,
    /// Hold here (spec §8 "Allies"): [`Action::Order`] with [`Command::Hold`], every ally holds
    /// the cell it stands on. Listed and available as [`Ability::Heel`].
    Hold,
    /// Set on (spec §8 "Allies"): [`Action::Order`] with [`Command::SetOn`] a cell the player
    /// sees, every ally goes there and fights what is near it. Listed and available as
    /// [`Ability::Heel`]; it asks for the cell in the aiming mode.
    SetOn,
    /// Give (spec §8 "Allies"): [`Action::Give`], a stack the player carries for the neck of a
    /// kept beast of theirs (`world/collar.rs`). Not an order, it takes a `wear`'s time; the Allies
    /// group all the same, listed while the player keeps a beast ([`Ability::orders`]),
    /// unavailable ([`Unavailable::NotBeside`]) with none beside them.
    Give,
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
    /// Something worn or carried (none yet: a lantern is used as a tool, `u`).
    Item,
    /// The place the character stands in (spec §7 "Special rooms"): what a special room's
    /// feature offers next to it (Pray at a forgotten altar, Drink at a spring).
    Place,
    /// The character's allies (spec §8 "Allies"): the orders given to them.
    Ally,
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
        Source::Ally,
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
            Source::Ally => "Allies",
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
            Source::Ally => "ally",
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
    /// A cell in sight with a corpse on it, in the aiming mode (`world/corpse_spell.rs`).
    Corpse,
    /// What to treat, of what the player may treat now on themselves and their allies beside
    /// them ([`crate::heal::treatable`]): asked only when there is more than one (the Heal
    /// ability).
    Ailment,
    /// A stack the player carries, from a list of those it may work on (the Identify spell on
    /// the identify list; Temper: metal arms or armour, [`crate::world::workable`];
    /// [`Ability::action_on`]).
    Item,
    /// A weapon's shape, from the list of every one there is (the Hunter's Spirit weapon:
    /// [`crate::world::weapon_shapes`], `use:spirit_weapon:long_sword`).
    Shape,
    /// A spell the player knows, from the list of them with each one's price (the wizard's
    /// Scribe: [`crate::perception::PerceptionView::scribes`], `scribe:fireball`).
    Spell,
}

/// Why an ability cannot be used now: the abilities menu shows it beside the greyed-out row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unavailable {
    /// A shield bash without a shield to bash with.
    NoShield,
    /// Fletch without a knife carried (spec §8 "Fletching").
    NoKnife,
    /// Fletch without a bow or a crossbow ready (spec §8 "Fletching").
    NoLauncher,
    /// A skill whose use is not in the game yet.
    NotYet,
    /// Decipher with no scroll of an unidentified type in the pack.
    NoUnknownScroll,
    /// While a status lasts (spec §8 "Status effects"): one that blocks the kind of action the
    /// ability is (a raging berserker cannot read), or the ability's cooldown, which the class
    /// abilities bring (Winded: "you are winded"). Only time lifts it.
    While(StatusId),
    /// One that costs an alternative resource the character does not hold, or holds too little
    /// of (spec §8 "Resources"): "not enough focus".
    Short(ResourceId),
    /// One that costs more mana than the actor has (spec §8 "Mana"): time brings it back.
    NoMana,
    /// One the place does not allow: foraging anywhere but in the wild (spec §8 "Foraging").
    NotHere,
    /// The Heal ability with nothing to treat: no bleeding, poison, or disease that shows.
    NothingToTreat,
    /// A triggered weapon ability: it fires by itself, never chosen.
    FiresByItself,
    /// An active weapon ability while the actor does not hold what it is for: a weapon of its
    /// skill, the grip of its style, a shield for shield blocking's ("your hands are not so
    /// held").
    NotSoHeld,
    /// An order with no ally of the player's on the level to hear it (spec §8 "Allies").
    NoAllies,
    /// Give with the player's companion not beside them (spec §8 "Allies").
    NotBeside,
    /// Sic 'em with no companion of the player's on the level (`world/sic.rs`).
    NoCompanion,
    /// Stillness with nothing to end: none of the statuses it ends is suffered.
    Still,
    /// Call familiar with a familiar alive, wherever it is.
    FamiliarLives,
    /// Call familiar with a hostile in sight.
    Watched,
    /// Tame with a companion already (spec §8 "Allies": a companion is for good, and one at a
    /// time).
    HaveCompanion,
}

impl Unavailable {
    /// The reason, as the menu says it: "you are winded".
    pub fn reason(self, content: &Content) -> Cow<'static, str> {
        match self {
            Unavailable::NoShield => "you hold no shield".into(),
            Unavailable::NoKnife => "you have no knife".into(),
            Unavailable::NoLauncher => "you have no bow or crossbow ready".into(),
            Unavailable::NotYet => "not usable yet".into(),
            Unavailable::NoUnknownScroll => "no unknown scroll".into(),
            Unavailable::NoMana => "you have too little mana".into(),
            Unavailable::NotHere => "not usable here".into(),
            Unavailable::NothingToTreat => "nothing to treat".into(),
            Unavailable::FiresByItself => "fires by itself".into(),
            Unavailable::While(status) => {
                format!("you are {}", content.status(status).name.to_lowercase()).into()
            }
            Unavailable::Short(resource) => {
                let def = content.resource(resource);
                match &def.lacking {
                    Some(words) => words.clone().into(),
                    None => format!("not enough {}", def.name.to_lowercase()).into(),
                }
            }
            Unavailable::NotSoHeld => "your hands are not so held".into(),
            Unavailable::NoAllies => "no allies here".into(),
            Unavailable::NotBeside => "not beside you".into(),
            Unavailable::NoCompanion => "no companion here".into(),
            Unavailable::Still => "your mind is already still".into(),
            Unavailable::FamiliarLives => "your familiar lives".into(),
            Unavailable::Watched => "something is watching".into(),
            Unavailable::HaveCompanion => "you already have a companion".into(),
        }
    }

    /// Its id in dumps: `no_shield`, `while:winded`.
    pub fn id(self, content: &Content) -> Cow<'static, str> {
        match self {
            Unavailable::NoShield => "no_shield".into(),
            Unavailable::NoKnife => "no_knife".into(),
            Unavailable::NoLauncher => "no_launcher".into(),
            Unavailable::NotYet => "not_yet".into(),
            Unavailable::NoUnknownScroll => "no_unknown_scroll".into(),
            Unavailable::NoMana => "no_mana".into(),
            Unavailable::NotHere => "not_here".into(),
            Unavailable::NothingToTreat => "nothing_to_treat".into(),
            Unavailable::FiresByItself => "fires_by_itself".into(),
            Unavailable::NotSoHeld => "not_so_held".into(),
            Unavailable::NoAllies => "no_allies".into(),
            Unavailable::NotBeside => "not_beside".into(),
            Unavailable::NoCompanion => "no_companion".into(),
            Unavailable::Still => "still".into(),
            Unavailable::FamiliarLives => "familiar_lives".into(),
            Unavailable::Watched => "watched".into(),
            Unavailable::HaveCompanion => "have_companion".into(),
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
        // Woodcraft's second use: wedging a door, while some skill gives it.
        let wedge = (content.skills().iter())
            .any(|def| def.benefits.iter().any(|b| b.effect == Effect::Wedge))
            .then_some(Ability::Wedge);
        // Birth sign: the active gifts some sign grants, in their order.
        let signs = (SignActive::ALL.iter())
            .filter(|&&a| content.sign_active(a).is_some())
            .map(|&a| Ability::Sign(a));
        // Class: throwing one's voice.
        let class = [Ability::ThrowVoice];
        // Talents: the active ones the talent table has.
        let wind = (content.talents().iter())
            .flat_map(|(_, def)| &def.effects)
            .any(|e| matches!(e, TalentEffect::SecondWind { .. }));
        let blood = (content.talents().iter())
            .flat_map(|(_, def)| &def.effects)
            .any(|e| matches!(e, TalentEffect::BloodMagic { .. }));
        let talents = (wind.then_some(Ability::SecondWind))
            .into_iter()
            .chain(blood.then_some(Ability::BloodMagic));
        // Classes: the actives some class's table has, in their order.
        let classes = (ClassActive::ALL.iter())
            .filter(|&&a| content.class_active(a).is_some())
            .map(|&a| Ability::Class(a));
        // Every spell, by tier, then in path order: any may be learned.
        let spells = content.spells_by_level().into_iter().map(Ability::Spell);
        // Weapon skills and fighting styles: the shield bash, then what their ranks teach, by
        // skill in file order, then rank, then path order.
        let mut taught: Vec<(WeaponSkillId, u8, WeaponAbilityId)> = (content.weapon_abilities())
            .iter()
            .enumerate()
            .flat_map(|(i, a)| {
                let id = WeaponAbilityId(i as u16);
                a.skills.iter().map(move |&s| (s, a.rank, id))
            })
            .collect();
        taught.sort();
        let weapon_skills = std::iter::once(Ability::ShieldBash)
            .chain(taught.into_iter().map(|(s, _, id)| Ability::Weapon(s, id)));
        // Here: what some special room's feature offers.
        let offers = |offer| {
            (content.room_effects().defs().iter())
                .any(|e| e.offers.as_ref().is_some_and(|(o, _)| *o == offer))
        };
        let here = [Offer::Pray, Offer::Temper, Offer::Drink]
            .into_iter()
            .filter(|&offer| offers(offer))
            .map(Ability::offered);
        let mut all: Vec<Ability> = (signs.chain(class))
            .chain(classes)
            .chain(spells)
            .chain(talents)
            .chain(skills)
            .chain(wedge)
            .chain(weapon_skills)
            .chain(here)
            .chain(Ability::orders(
                content.allies().is_some(),
                content.allies().is_some(),
            ))
            .collect();
        all.sort_by_key(|a| a.source());
        all
    }

    /// The ability that is the use of skill `def`, once that use is in the game: Disable device's
    /// (the disarm effect's) is [`Ability::Disarm`], Open lock's (the pick effect's)
    /// [`Ability::PickLock`], the Steal skill's (the `steal` benefit, with `data/steal.toml`)
    /// [`Ability::Steal`], Bluff's (the `feint` benefit, with `data/feint.toml`)
    /// [`Ability::Feint`], Heal's (the heal effect's) [`Ability::Heal`], Handle animal's (the
    /// `tame` benefit, with `data/companions.toml` and `data/allies.toml`) [`Ability::Tame`],
    /// Intimidate's (the
    /// `intimidate` benefit, with `data/intimidate.toml`) [`Ability::Intimidate`], Survival's (the
    /// `forage` benefit, with `data/forage.toml`) [`Ability::Forage`], Woodcraft's (the `fletch`
    /// benefit, with `data/fletch.toml`) [`Ability::Fletch`], Decipher script's (the `decipher`
    /// benefit, with `data/decipher.toml`) [`Ability::Decipher`]. `None` for a skill whose
    /// use is to come, or that has none.
    fn use_of(def: &SkillDef, content: &Content) -> Option<Ability> {
        let gives = |effect| def.benefits.iter().any(|b| b.effect == effect);
        (gives(Effect::Disarm).then_some(Ability::Disarm))
            .or_else(|| gives(Effect::Hearing).then_some(Ability::Listen))
            .or_else(|| gives(Effect::Pick).then_some(Ability::PickLock))
            .or_else(|| gives(Effect::Heal).then_some(Ability::Heal))
            .or_else(|| {
                (gives(Effect::Tame)
                    && content.companions().is_some()
                    && content.allies().is_some())
                .then_some(Ability::Tame)
            })
            .or_else(|| {
                (gives(Effect::Feint) && content.feint().is_some()).then_some(Ability::Feint)
            })
            .or_else(|| {
                (gives(Effect::Intimidate) && content.intimidate().is_some())
                    .then_some(Ability::Intimidate)
            })
            .or_else(|| {
                (gives(Effect::Steal) && content.steal().is_some()).then_some(Ability::Steal)
            })
            .or_else(|| {
                (gives(Effect::Decipher) && content.decipher().is_some())
                    .then_some(Ability::Decipher)
            })
            .or_else(|| {
                (gives(Effect::Forage) && content.forage().is_some()).then_some(Ability::Forage)
            })
            .or_else(|| {
                (gives(Effect::Fletch) && content.fletch().is_some()).then_some(Ability::Fletch)
            })
    }

    /// The abilities the place offers to whoever stands at `at` on `level` (spec §7 "Special
    /// rooms"): what the feature on a cell next to it offers (Pray beside a forgotten altar, Drink beside a spring), in
    /// menu order. A spent altar is another terrain, which offers nothing.
    pub(crate) fn here(level: &LevelState, content: &Content, at: Pos) -> Vec<Ability> {
        let mut found: Vec<Ability> = (Dir::ALL.iter().map(|&d| at.step(d)))
            .filter_map(|p| level.terrain_at(p))
            .filter_map(|t| content.offer(t))
            .map(Ability::offered)
            .collect();
        found.sort();
        found.dedup();
        found
    }

    /// The ability a place's `offer` is used through.
    pub fn offered(offer: Offer) -> Ability {
        match offer {
            Offer::Pray => Ability::Pray,
            Offer::Temper => Ability::Temper,
            Offer::Drink => Ability::Drink,
        }
    }

    /// The Allies group (spec §8 "Allies"), in menu order: the orders while `any`, the player has
    /// an ally anywhere (the perception knows), and Give while `kept`, they have a companion; none
    /// otherwise, as a place's abilities are absent away from the place.
    pub(crate) fn orders(any: bool, kept: bool) -> Vec<Ability> {
        let orders = [Ability::Heel, Ability::Hold, Ability::SetOn];
        (orders.into_iter().filter(|_| any))
            .chain(kept.then_some(Ability::Give))
            .collect()
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
    /// trained in Open lock (the pick effect's) or a talent lets them (the `pick_lock` effect);
    /// stealing, if they are trained in a skill with the `steal` benefit or a talent lets them
    /// (the `steal` effect; it is listed only when stealing is in the game); treating
    /// oneself, if they are trained in a skill with the `heal` effect; foraging, if they are
    /// trained in Survival (the forage benefit's); second wind,
    /// if they hold the talent; throwing one's voice, if a talent gives it (the bard's); the
    /// shield bash and fletching, everyone.
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
            Ability::Listen => actor.skills.trains_effect(content, Effect::Hearing),
            Ability::Wedge => actor.skills.trains_effect(content, Effect::Wedge),
            // A talent with the `pick_lock` or `steal` effect gives them without the skills.
            Ability::PickLock => {
                actor.skills.trains_effect(content, Effect::Pick)
                    || actor.talent_mods(content).pick_lock
            }
            Ability::Steal => {
                actor.skills.trains_effect(content, Effect::Steal)
                    || actor.talent_mods(content).steal
            }
            Ability::Forage => actor.skills.trains_effect(content, Effect::Forage),
            Ability::Decipher => actor.skills.trains_effect(content, Effect::Decipher),
            Ability::Tame => actor.skills.trains_effect(content, Effect::Tame),
            Ability::Heal => actor.skills.trains_effect(content, Effect::Heal),
            Ability::Intimidate => actor.skills.trains_effect(content, Effect::Intimidate),
            Ability::Feint => actor.skills.trains_effect(content, Effect::Feint),
            Ability::SecondWind => actor.talent_mods(content).second_wind.is_some(),
            Ability::BloodMagic => actor.talent_mods(content).blood_magic.is_some(),
            Ability::ThrowVoice => actor.talent_mods(content).throw_voice,
            Ability::ShieldBash | Ability::Fletch => true,
            Ability::Weapon(skill, id) => actor.weapon_abilities(content).contains(&(skill, id)),
            // A place offers it, not who one is ([`Ability::here`]); an order needs allies
            // ([`Ability::orders`]).
            Ability::Pray | Ability::Temper | Ability::Drink => false,
            Ability::Heel | Ability::Hold | Ability::SetOn | Ability::Give => false,
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
            Ability::Listen => "listen",
            Ability::PickLock => "pick_lock",
            Ability::Steal => "steal",
            Ability::Wedge => "wedge",
            Ability::Feint => "feint",
            Ability::Tame => "tame",
            Ability::Intimidate => "intimidate",
            Ability::Forage => "forage",
            Ability::Fletch => "fletch",
            Ability::Decipher => "decipher",
            Ability::Heal => "heal",
            Ability::SecondWind => "second_wind",
            Ability::BloodMagic => "blood_magic",
            Ability::ShieldBash => "shield_bash",
            Ability::Weapon(_, id) => &content.weapon_ability(id).id,
            Ability::Pray => "pray",
            Ability::Temper => "temper",
            Ability::Drink => "drink",
            Ability::Heel => "heel",
            Ability::Hold => "hold",
            Ability::SetOn => "set_on",
            Ability::Give => "give",
        }
    }

    /// Its name in the menu (a weapon ability's after its skill's: "Long blades: Deep cut").
    pub fn name(self, content: &Content) -> Cow<'_, str> {
        let name: &str = match self {
            Ability::Weapon(skill, id) => {
                let skill = &content.weapon_skills().skill(skill).name;
                let name = &content.weapon_ability(id).name;
                return format!("{skill}: {name}").into();
            }
            Ability::Sign(active) => content.sign_active(active).map_or("", |a| &a.name),
            Ability::Class(active) => content.class_active(active).map_or("", |a| {
                // Stillness is named apart from Still mind, its talent.
                (a.effects.iter().find_map(TalentEffect::active_name)).unwrap_or(&a.name)
            }),
            Ability::Spell(spell) => content.spell_of(spell).map_or("", |s| &s.name),
            Ability::Skill(id) => &content.skill(id).name,
            Ability::ThrowVoice => "Throw your voice",
            Ability::Disarm => "Disarm a trap",
            Ability::Listen => "Listen at a door",
            Ability::PickLock => "Pick lock",
            Ability::Steal => "Steal",
            Ability::Wedge => "Wedge a door",
            Ability::Feint => "Feint",
            Ability::Tame => "Tame",
            Ability::Intimidate => "Intimidate",
            Ability::Forage => "Forage",
            Ability::Fletch => "Fletch",
            Ability::Decipher => "Decipher",
            Ability::Heal => "Heal",
            Ability::SecondWind => "Second wind",
            Ability::BloodMagic => "Blood magic",
            Ability::ShieldBash => "Shield bash",
            Ability::Pray => "Pray",
            Ability::Temper => "Temper",
            Ability::Drink => "Drink",
            Ability::Heel => "Come to heel",
            Ability::Hold => "Hold here",
            Ability::SetOn => "Set on",
            Ability::Give => "Give",
        };
        name.into()
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
            Ability::Listen => {
                "Press an ear to a closed door next to you for a turn, and hear what moves \
                 beyond it."
            }
            Ability::PickLock => "Open a locked door next to you without its key.",
            Ability::Steal => {
                "Lift a purse or a trinket from someone next to you who is not hostile; caught, \
                 they turn on you."
            }
            Ability::Wedge => {
                "Drive a wedge under a closed door next to you for a turn: those who would open it \
                 must kick it in. You open it as usual, pulling the wedge free."
            }
            Ability::Feint => {
                "Make a lying move at a foe next to you: fooled, it is off its guard for your \
                 next blow."
            }
            Ability::Tame => {
                "Spend a while beside a beast next to you that is not your foe, or has not seen \
                 you, to win it over, with food if you carry some: won, it is your companion for \
                 good."
            }
            Ability::Intimidate => {
                "Threaten a foe you see nearby: unless it keeps its nerve, it flees from you a \
                 moment, longer the better you intimidate."
            }
            Ability::Forage => {
                "Spend a while searching the wild around you for something to eat: berries, \
                 nuts, mushrooms, roots."
            }
            Ability::Fletch => {
                "Spend a while with a knife and a bow to hand, cutting arrows or bolts for it \
                 from what wood there is. Anyone can, after a fashion."
            }
            Ability::Decipher => {
                "Pore over a scroll you carry, of a kind you have not identified, for ten \
                 minutes: if you make out the writing, you learn what it is without reading it. \
                 Failed, the time is gone and you may try again."
            }
            Ability::Heal => {
                "Tend your own hurts or an ally's beside you: bind a bleeding wound, or try to \
                 ease a poison or a disease (once each)."
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
            Ability::Weapon(_, id) => &content.weapon_ability(id).description,
            Ability::Pray => {
                "Kneel at the forgotten altar beside you and pray for a minute. Something may \
                 answer, for good or ill."
            }
            Ability::Temper => {
                "Work a metal weapon or piece of armour you carry at the anvil beside you for \
                 ten minutes. It may come out finer, or damaged and stripped of what was done to \
                 it; damaged, the next try mends it or breaks it. Each try is harder than the \
                 last."
            }
            Ability::Drink => {
                "Drink from the spring beside you. Its water is mostly good, and now and then \
                 foul."
            }
            Ability::Heel => {
                "Call your allies to your side: they follow you and fight what threatens you. \
                 A shout, heard about you; it takes no time."
            }
            Ability::Hold => {
                "Tell your allies to hold where they stand, fighting only what comes beside \
                 them. A shout; it takes no time."
            }
            Ability::SetOn => {
                "Set your allies on a place you see: they go there and fight what is near it, \
                 then come back to heel. A shout; it takes no time."
            }
            Ability::Give => {
                "Fasten something for the neck about that of a beast you keep, beside you: its \
                 magic works on the beast, and what it wore before falls to the floor."
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
            | Ability::Listen
            | Ability::PickLock
            | Ability::Steal
            | Ability::Wedge
            | Ability::Feint
            | Ability::Tame
            | Ability::Intimidate
            | Ability::Forage
            | Ability::Fletch
            | Ability::Decipher
            | Ability::Heal => Source::Skill,
            Ability::ThrowVoice => Source::Class,
            Ability::SecondWind | Ability::BloodMagic => Source::Talent,
            Ability::ShieldBash | Ability::Weapon(..) => Source::WeaponSkill,
            Ability::Pray | Ability::Temper | Ability::Drink => Source::Place,
            Ability::Heel | Ability::Hold | Ability::SetOn | Ability::Give => Source::Ally,
        }
    }

    /// What it asks for before it is used (a spell: by its shape in `content`).
    pub fn aim(self, content: &Content) -> Aim {
        match self {
            Ability::Sign(active) if active.aimed() => Aim::Cell,
            Ability::Sign(_) => Aim::Nothing,
            Ability::Class(active) if active.aimed() => Aim::Direction,
            Ability::Spell(spell) if aims_item(content, spell) => Aim::Item,
            Ability::Spell(spell) if aims_corpse(content, spell) => Aim::Corpse,
            Ability::Spell(spell) if aimed(content, spell) => Aim::Cell,
            Ability::Spell(_) => Aim::Nothing,
            Ability::Class(active) if active.aims_cell() => Aim::Cell,
            Ability::Class(active) if active.aims_item() => Aim::Item,
            Ability::Class(active) if active.aims_shape() => Aim::Shape,
            Ability::Class(active) if active.aims_spell() => Aim::Spell,
            Ability::Weapon(_, id) => match content.weapon_ability(id).active.map(|a| a.aim) {
                Some(ActiveAim::Direction) => Aim::Direction,
                Some(ActiveAim::Cell) => Aim::Cell,
                Some(ActiveAim::Nothing) | None => Aim::Nothing,
            },
            Ability::Class(_)
            | Ability::Skill(_)
            | Ability::SecondWind
            | Ability::BloodMagic
            | Ability::Forage
            | Ability::Fletch
            | Ability::Pray
            | Ability::Heel
            | Ability::Hold
            | Ability::Drink => Aim::Nothing,
            Ability::Temper | Ability::Give | Ability::Decipher => Aim::Item,
            Ability::ThrowVoice | Ability::SetOn | Ability::Intimidate => Aim::Cell,
            Ability::Heal => Aim::Ailment,
            Ability::Disarm
            | Ability::Listen
            | Ability::PickLock
            | Ability::Steal
            | Ability::Wedge
            | Ability::Feint
            | Ability::Tame
            | Ability::ShieldBash => Aim::Direction,
        }
    }

    /// What it costs, as an action of this kind (`data/actions.toml`; the rules then price it as
    /// that action, spec §8 "Time"); `None` for one with no use yet.
    pub const fn cost(self) -> Option<ActionKind> {
        match self {
            Ability::Sign(active) => Some(active.cost()),
            Ability::Class(active) => Some(active.cost()),
            Ability::Spell(_) => Some(ActionKind::Cast),
            Ability::Skill(_) | Ability::Weapon(..) => None,
            Ability::ThrowVoice => Some(ActionKind::ThrowVoice),
            Ability::Disarm => Some(ActionKind::Disarm),
            // A listen is a turn of keeping still.
            Ability::Listen => Some(ActionKind::Wait),
            Ability::PickLock => Some(ActionKind::PickLock),
            Ability::Steal => Some(ActionKind::Steal),
            // A wedging is a turn of work.
            Ability::Wedge => Some(ActionKind::Wait),
            Ability::Feint => Some(ActionKind::Feint),
            Ability::Tame => Some(ActionKind::Tame),
            Ability::Intimidate => Some(ActionKind::Intimidate),
            Ability::Forage => Some(ActionKind::Forage),
            Ability::Fletch => Some(ActionKind::Fletch),
            Ability::Decipher => Some(ActionKind::Decipher),
            Ability::Heal => Some(ActionKind::Heal),
            Ability::SecondWind | Ability::BloodMagic => Some(ActionKind::Wait),
            Ability::ShieldBash => Some(ActionKind::Melee),
            Ability::Pray => Some(ActionKind::Pray),
            Ability::Temper => Some(ActionKind::Temper),
            Ability::Give => Some(ActionKind::Wear),
            Ability::Drink => Some(ActionKind::Drink),
            // An order takes no time (spec §8 "Allies"): it is priced as nothing.
            Ability::Heel | Ability::Hold | Ability::SetOn => None,
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
            | Ability::Listen
            | Ability::PickLock
            | Ability::Steal
            | Ability::Wedge
            | Ability::Feint
            | Ability::Tame
            | Ability::Intimidate
            | Ability::Forage
            | Ability::Fletch
            | Ability::Decipher
            | Ability::Heal
            | Ability::SecondWind
            | Ability::BloodMagic
            | Ability::ShieldBash
            | Ability::Weapon(..)
            | Ability::Pray
            | Ability::Temper
            | Ability::Drink
            | Ability::Heel
            | Ability::Hold
            | Ability::SetOn
            | Ability::Give => None,
        }
    }

    /// How far a cell it aims at may be, in cells, with what `content` says of it: a spell's
    /// is its data's base ([`crate::content::SpellData::base_range`]; a caster's own is
    /// [`Actor::spell_reach`]); every other ability's is [`Ability::range`].
    pub fn reach(self, content: &Content) -> Option<i32> {
        match self {
            Ability::Spell(spell) => content.spell_of(spell).and_then(|s| s.base_range()),
            Ability::Intimidate => content.intimidate().map(|r| r.range),
            Ability::Class(ClassActive::Sic) => content.sic_range(),
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
            (Ability::Weapon(_, id), dir) => {
                let ability = content.weapon_ability(id);
                match (ability.active.map(|a| a.aim), dir) {
                    (Some(ActiveAim::Direction), Some(dir)) => {
                        Some(Action::Act(ability.id.clone(), dir))
                    }
                    (Some(ActiveAim::Nothing), _) => Some(Action::Cast(ability.id.clone(), None)),
                    _ => None,
                }
            }
            (Ability::Skill(_) | Ability::ThrowVoice | Ability::Intimidate, _) => None,
            // The first thing there is to treat; the menu names one when there are several.
            (Ability::Heal, _) => Some(Action::Heal {
                what: None,
                ally: None,
            }),
            (Ability::SecondWind, _) => Some(Action::UseTalent(TalentActive::SecondWind)),
            (Ability::BloodMagic, _) => Some(Action::UseTalent(TalentActive::BloodMagic)),
            (Ability::Disarm, Some(dir)) => Some(Action::Disarm(dir)),
            (Ability::Listen, Some(dir)) => Some(Action::Listen(dir)),
            (Ability::PickLock, Some(dir)) => Some(Action::PickLock(dir)),
            (Ability::Steal, Some(dir)) => Some(Action::Steal(dir)),
            (Ability::Wedge, Some(dir)) => Some(Action::Wedge(dir)),
            (Ability::Feint, Some(dir)) => Some(Action::Feint(dir)),
            (Ability::Tame, Some(dir)) => Some(Action::Tame(dir)),
            (Ability::ShieldBash, Some(dir)) => Some(Action::Bash(dir)),
            (
                Ability::Disarm
                | Ability::Listen
                | Ability::PickLock
                | Ability::Steal
                | Ability::Wedge
                | Ability::Feint
                | Ability::Tame
                | Ability::ShieldBash,
                None,
            ) => None,
            (Ability::Pray, _) => Some(Action::Pray),
            (Ability::Drink, _) => Some(Action::Drink),
            (Ability::Forage, _) => Some(Action::Forage),
            (Ability::Fletch, _) => Some(Action::Fletch),
            // It asks for an item ([`Ability::action_on`]).
            (Ability::Temper | Ability::Decipher, _) => None,
            (Ability::Heel, _) => Some(Action::Order(Command::Heel)),
            (Ability::Hold, _) => Some(Action::Order(Command::Hold)),
            // It asks for a cell ([`Ability::action_at`]).
            (Ability::SetOn | Ability::Give, _) => None,
        }
    }

    /// The action it is, worked or cast on inventory stack `item`, for one that asks for a stack
    /// the player carries (Identify, Temper, Merchant's eye); `None` for any other.
    pub fn action_on(self, content: &Content, item: usize) -> Option<Action> {
        match self {
            Ability::Decipher => Some(Action::Decipher { item }),
            Ability::Temper => Some(Action::Temper { item }),
            Ability::Give => Some(Action::Give { item, ally: None }),
            Ability::Class(active) if active.aims_item() => Some(Action::UseOn(active, item)),
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
            Ability::Intimidate => Some(Action::Intimidate(at)),
            Ability::SetOn => Some(Action::Order(Command::SetOn(at))),
            Ability::Sign(active) if active.aimed() => Some(Action::Gift(active, Some(at))),
            Ability::Spell(spell) if aimed(content, spell) => {
                Some(Action::Cast(spell_name(content, spell)?, Some(at)))
            }
            Ability::Class(active) if active.aims_cell() => Some(Action::UseAt(active, at)),
            Ability::Weapon(_, id) => {
                let ability = content.weapon_ability(id);
                (ability.active.map(|a| a.aim) == Some(ActiveAim::Cell))
                    .then(|| Action::Cast(ability.id.clone(), Some(at)))
            }
            Ability::Sign(_)
            | Ability::Spell(_)
            | Ability::Class(_)
            | Ability::Skill(_)
            | Ability::Disarm
            | Ability::Listen
            | Ability::PickLock
            | Ability::Steal
            | Ability::Wedge
            | Ability::Feint
            | Ability::Tame
            | Ability::Forage
            | Ability::Fletch
            | Ability::Decipher
            | Ability::Heal
            | Ability::SecondWind
            | Ability::BloodMagic
            | Ability::ShieldBash
            | Ability::Pray
            | Ability::Temper
            | Ability::Drink
            | Ability::Heel
            | Ability::Hold
            | Ability::Give => None,
        }
    }

    /// Why `actor` cannot use it now; `None` when it can. Its own reasons come first, then a
    /// status of the actor's that blocks the kind of action it costs ([`Unavailable::While`]).
    pub(crate) fn unavailable(self, actor: &Actor, content: &Content) -> Option<Unavailable> {
        let own = match self {
            Ability::Class(active) => {
                (class_cooldown(actor, content, active).map(Unavailable::While))
                    .or_else(|| short_of(actor, class_cost(actor, content, active)?))
                    .or_else(|| still(actor, content, active))
                    .or_else(|| {
                        // A song costs one thing (a Virtuoso's nothing): its resource
                        // (Inspire's inspiration), or its mana, which a status that blocks
                        // casting (Raging) keeps from being spent.
                        let song = (active.sings())
                            .then(|| actor.talent_mods(content).song(active))
                            .flatten();
                        let (pool, mana) = match song.and_then(|s| s.cost) {
                            Some(SongCost::Resource(cost)) => (Some(cost), 0),
                            Some(SongCost::Mana(mana)) => (None, mana),
                            None => (None, 0),
                        };
                        if let Some(short) = pool.and_then(|cost| short_of(actor, cost)) {
                            return Some(short);
                        }
                        // Consecrate costs its mana.
                        let consecrate = (active == ClassActive::Consecrate)
                            .then(|| actor.talent_mods(content).consecrate)
                            .flatten();
                        let mana = mana + consecrate.map_or(0, |c| c.mana);
                        // Sanctuary costs its mana.
                        let sanctuary = (active == ClassActive::Sanctuary)
                            .then(|| actor.talent_mods(content).sanctuary)
                            .flatten();
                        let mana = mana + sanctuary.map_or(0, |s| s.mana);
                        // Knock costs its mana.
                        let knock = (active == ClassActive::Knock)
                            .then(|| actor.talent_mods(content).knock)
                            .flatten();
                        let mana = mana + knock.map_or(0, |k| k.mana);
                        let cost = actor.mana_cost(content, mana);
                        let blocked = (cost > 0)
                            .then(|| actor.effects.blocking(content, ActionKind::Cast))
                            .flatten();
                        (blocked.map(Unavailable::While))
                            .or((actor.mana < cost).then_some(Unavailable::NoMana))
                    })
                    // Scribe spends mana, which a status that blocks casting keeps (Raging);
                    // the same status keeps a witch from her calling.
                    .or_else(|| {
                        matches!(active, ClassActive::Scribe | ClassActive::CallFamiliar)
                            .then(|| actor.effects.blocking(content, ActionKind::Cast))
                            .flatten()
                            .map(Unavailable::While)
                    })
            }
            // Its mana is asked by [`spell_unavailable`].
            Ability::Spell(_) => None,
            Ability::Sign(active) => (actor.talent_mods(content).sign_cooldown(active))
                .filter(|&cooldown| actor.effects.has(cooldown))
                .map(Unavailable::While),
            Ability::Skill(_) => Some(Unavailable::NotYet),
            Ability::Fletch => crate::world::fletch_lack(actor, content),
            Ability::Weapon(_, id) => weapon_unavailable(actor, content, id),
            Ability::Heal => (crate::heal::treatable(actor, content).is_empty())
                .then_some(Unavailable::NothingToTreat),
            Ability::ThrowVoice
            | Ability::Disarm
            | Ability::Listen
            | Ability::PickLock
            | Ability::Steal
            | Ability::Wedge
            | Ability::Feint
            | Ability::Intimidate
            | Ability::Forage
            | Ability::BloodMagic
            // Whether the pack holds an unknown scroll is the perception's to say.
            | Ability::Decipher
            // Whether the player has a companion is the perception's to say.
            | Ability::Tame
            | Ability::Pray
            | Ability::Temper
            | Ability::Drink
            // Whether anyone is there to hear an order is the perception's to say.
            | Ability::Heel
            | Ability::Hold
            | Ability::SetOn
            | Ability::Give => None,
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
        // Volley costs Focus, Low blow and Master thief Openings, Stunning strike and Quivering
        // palm Ki, and the smites and Lay on hands Zeal instead
        // (their talents' `resource`, [`short_of`]), as War cry Rage; Lurk only its time.
        ClassActive::LayOnHands
        | ClassActive::Smite
        | ClassActive::BrandingSmite
        | ClassActive::WrathfulSmite
        | ClassActive::BanishingSmite
        | ClassActive::Whirlwind
        | ClassActive::WarCry
        | ClassActive::HolyRadiance
        | ClassActive::PatientDefence
        | ClassActive::StunningStrike
        | ClassActive::QuiveringPalm
        | ClassActive::StepOfTheWind
        | ClassActive::Stillness
        | ClassActive::Lurk
        | ClassActive::LowBlow
        | ClassActive::MasterThief
        | ClassActive::Inspire
        | ClassActive::Lullaby
        | ClassActive::Consecrate
        | ClassActive::Orison
        | ClassActive::Sanctuary
        | ClassActive::Cackle
        | ClassActive::CallFamiliar
        | ClassActive::Knock
        | ClassActive::Scribe
        | ClassActive::MerchantsEye
        | ClassActive::SpiritWeapon
        | ClassActive::Sic
        | ClassActive::Volley => Vec::new(),
    };
    cooldowns.into_iter().find(|&s| actor.effects.has(s))
}

/// Stillness with nothing to end (the ascetic's, #850): `actor` suffers none of its statuses.
fn still(actor: &Actor, content: &Content, active: ClassActive) -> Option<Unavailable> {
    if active != ClassActive::Stillness {
        return None;
    }
    let mods = actor.talent_mods(content);
    let ends = mods.stillness.as_ref()?;
    (!ends.iter().any(|&s| actor.effects.has(s))).then_some(Unavailable::Still)
}

/// Why `actor` cannot use weapon ability `id` now: a triggered one fires by itself; an active
/// one while what it holds is not for it, while its cooldown lasts (none in the catalogue),
/// while it holds less stamina than the ability costs it, and while a status blocks the action
/// it is.
fn weapon_unavailable(
    actor: &Actor,
    content: &Content,
    id: WeaponAbilityId,
) -> Option<Unavailable> {
    let Some(active) = content.weapon_ability(id).active else {
        return Some(Unavailable::FiresByItself);
    };
    if !actor.holds_for(content, id) {
        return Some(Unavailable::NotSoHeld);
    }
    let cooldown = active.cooldown.filter(|&s| actor.effects.has(s));
    let stamina = content.resource_id(crate::content::STAMINA);
    let cost = stamina.map(|resource| ResourceCost {
        resource,
        amount: actor.weapon_stamina(content, id),
    });
    (cooldown.map(Unavailable::While))
        .or_else(|| short_of(actor, cost?))
        .or_else(|| (actor.effects.blocking(content, active.action)).map(Unavailable::While))
}

/// Whether `spell` asks for a cell before it is cast: anything but a spell on oneself.
fn aimed(content: &Content, spell: SpellId) -> bool {
    content.spell_of(spell).is_some_and(|s| s.aimed())
}

/// Whether `spell` is aimed at a corpse the caster sees.
fn aims_corpse(content: &Content, spell: SpellId) -> bool {
    content.spell_of(spell).is_some_and(|s| s.aims_corpse())
}

/// Whether `spell` is cast on a stack the caster carries (Identify).
fn aims_item(content: &Content, spell: SpellId) -> bool {
    content.spell_of(spell).is_some_and(|s| s.aims_item())
}

/// The id `spell` is cast by (`use:ID`), if the content has it.
fn spell_name(content: &Content, spell: SpellId) -> Option<String> {
    content.spell_of(spell).map(|s| s.id.clone())
}

/// Why `actor` cannot cast `spell` now: what [`Ability::unavailable`] says, else
/// [`Unavailable::NoMana`] while it has less mana than the spell costs it ([`Actor::spell_cost`]).
pub fn spell_unavailable(actor: &Actor, content: &Content, spell: SpellId) -> Option<Unavailable> {
    let ability = Ability::Spell(spell);
    ability.unavailable(actor, content).or_else(|| {
        let def = content.spell_of(spell)?;
        (actor.mana < actor.spell_cost(content, def)).then_some(Unavailable::NoMana)
    })
}

/// What class active `active` costs `actor` in an alternative resource (spec §8 "Resources"):
/// its talent's `resource`, with Avatar's share of it for Lay on hands (`percent_of`, at least
/// 1); `None` when it costs none. The one reading of the cost: `use_class` spends it.
pub(crate) fn class_cost(
    actor: &Actor,
    content: &Content,
    active: ClassActive,
) -> Option<ResourceCost> {
    let mut cost = content.class_active(active)?.resource?;
    if active == ClassActive::LayOnHands
        && let Some(percent) = actor.talent_mods(content).avatar
    {
        let share = crate::content::percent_of(i64::from(cost.amount), u32::from(percent));
        cost.amount = u16::try_from(share).unwrap_or(cost.amount).max(1);
    }
    Some(cost)
}

/// Why `actor` cannot use an ability that costs `cost` (spec §8 "Resources"):
/// [`Unavailable::Short`] while they do not hold the resource or hold less than it costs.
fn short_of(actor: &Actor, cost: ResourceCost) -> Option<Unavailable> {
    let held = actor.resources.get(cost.resource).map(|p| p.value);
    (held.is_none_or(|v| v < cost.amount)).then_some(Unavailable::Short(cost.resource))
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
                "Here",
                "Allies"
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
                "feint",
                "decipher",
                "disarm",
                "tame",
                "heal",
                "intimidate",
                "knowledge",
                "listen",
                "metalcraft",
                "pick_lock",
                "steal",
                "forage",
                "fletch",
                "wedge",
            ]
        );
        let last: Vec<Ability> = every.iter().rev().take(7).rev().copied().collect();
        assert_eq!(
            last,
            [
                Ability::Pray,
                Ability::Temper,
                Ability::Drink,
                Ability::Heel,
                Ability::Hold,
                Ability::SetOn,
                Ability::Give
            ],
            "then what a place offers, then the Allies group"
        );
        // The weapon skills' group: the shield bash, then what the ranks teach, by skill in file
        // order, then rank.
        let weapon: Vec<Ability> = (every.iter())
            .filter(|a| a.source() == Source::WeaponSkill)
            .copied()
            .collect();
        assert_eq!(weapon.first(), Some(&Ability::ShieldBash));
        let keys: Vec<(WeaponSkillId, u8)> = (weapon[1..].iter())
            .map(|a| match a {
                Ability::Weapon(skill, id) => (*skill, content.weapon_ability(*id).rank),
                other => panic!("{other:?} among the weapon abilities"),
            })
            .collect();
        assert!(keys.windows(2).all(|w| w[0] <= w[1]), "{keys:?}");
        // Stealing is in the game: the Steal skill is listed as its use.
        assert!(every.contains(&Ability::Steal));
        let steal = content.skill_id("steal").expect("steal");
        assert!(!every.contains(&Ability::Skill(steal)));
    }

    #[test]
    fn no_two_abilities_share_an_id_or_a_name() {
        let content = Content::builtin().expect("content");
        let every = Ability::every(&content);
        for (i, a) in every.iter().enumerate() {
            for b in &every[i + 1..] {
                // One weapon ability may be taught by several skills: the same row.
                if matches!((a, b), (Ability::Weapon(_, x), Ability::Weapon(_, y)) if x == y) {
                    continue;
                }
                assert_ne!(a.id(&content), b.id(&content), "{a:?} and {b:?}");
                assert_ne!(a.name(&content), b.name(&content), "{a:?} and {b:?}");
            }
        }
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
    fn throwing_a_voice_aims_at_a_cell() {
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
