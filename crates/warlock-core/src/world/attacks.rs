//! The player's attacks as the character screen shows them (spec §6 "Character screen", §8
//! "Combat"): for each attack they have ready — the main hand's blow (or bare hands'), the off
//! hand's while they fight with two weapons, and a shot or throw with what the Missile weapon
//! slot holds — its to-hit, damage, and reach, taken from the very [`Profile`] a strike would
//! roll ([`Actor::melee`], [`Actor::off_hand_blow`], [`Actor::ranged`], [`Actor::thrown`], with
//! the fighting style in use and everything else those read), so the sheet can never drift
//! from the rules.
//!
//! What the player does not know stays out: the profiles are built for the actor as it would
//! be if its unidentified items had no magic ([`Stack::as_known`]), so an unknown affix moves
//! no number. What depends on the moment (an ambush, a dazzled eye, a class ability's blow) is
//! not part of an attack's standing profile, so it is not shown either.

use super::Actor;
use crate::action::AttackKind;
use crate::combat::Profile;
use crate::content::Content;
use crate::damage::{Kinds, Rider};
use crate::dice::Dice;

/// Where an attack comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackSource {
    /// A blow with the weapon in the main hand, or with bare hands (or a natural attack).
    MainHand,
    /// The second blow of two weapons.
    OffHand,
    /// A shot with the readied launcher, or a throw of what the Missile weapon slot holds.
    Missile,
}

impl AttackSource {
    /// Its name on the character screen and in dumps.
    pub fn label(self) -> &'static str {
        match self {
            AttackSource::MainHand => "Main hand",
            AttackSource::OffHand => "Off hand",
            AttackSource::Missile => "Missile",
        }
    }

    /// Its id in JSON dumps.
    pub fn id(self) -> &'static str {
        match self {
            AttackSource::MainHand => "main_hand",
            AttackSource::OffHand => "off_hand",
            AttackSource::Missile => "missile",
        }
    }
}

/// One attack the player has ready, as the rules would strike it now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnAttack {
    pub source: AttackSource,
    /// Melee, a shot (ranged), or a throw.
    pub kind: AttackKind,
    /// What it is made with, as the player knows it: "long sword", "bare hands".
    pub name: String,
    /// Added to the 2d12 attack roll.
    pub to_hit: i32,
    /// The damage dice with every flat bonus folded in.
    pub damage: Dice,
    /// More dice rolled with them (a magic weapon's "adds 1d4 damage").
    pub extra: Option<Dice>,
    /// The kinds its own dice do (a blow does the better of two).
    pub kinds: Kinds,
    /// Damage of other kinds added to every hit.
    pub riders: Vec<Rider>,
    /// How far it reaches, in cells: a shot's or a throw's; `None` for a blow.
    pub range: Option<i32>,
}

impl OwnAttack {
    fn of(
        source: AttackSource,
        kind: AttackKind,
        name: String,
        profile: Profile,
        range: Option<i32>,
    ) -> Self {
        Self {
            source,
            kind,
            name,
            to_hit: profile.to_hit,
            damage: profile.damage.plus(profile.bonus),
            extra: profile.extra,
            kinds: profile.kinds,
            riders: profile.riders,
            range,
        }
    }

    /// The to-hit as written: `+4`, `-1`, `+0`.
    pub fn to_hit_text(&self) -> String {
        format!("{:+}", self.to_hit)
    }

    /// The damage as written: the dice (and any extra dice) with the flat bonus last, then the
    /// kinds and every rider: `1d8+2 slashing`, `1d6+1d4+1 slashing or piercing`, `1d8 slashing
    /// +1d6 fire`.
    pub fn damage_text(&self, content: &Content) -> String {
        let bonus = self.damage.bonus() + self.extra.map_or(0, |e| e.bonus());
        let dice = |d: Dice| d.plus(-d.bonus());
        let mut text = match (self.damage.is_fixed(), self.extra) {
            (true, None) => self.damage.bonus().max(1).to_string(),
            (_, None) => self.damage.to_string(),
            (_, Some(extra)) => {
                let mut text = format!("{}+{}", dice(self.damage), dice(extra));
                if bonus != 0 {
                    text.push_str(&format!("{bonus:+}"));
                }
                text
            }
        };
        let kinds: Vec<&str> = (self.kinds.iter())
            .map(|k| content.damage_type(k).name.as_str())
            .collect();
        if !kinds.is_empty() {
            text.push(' ');
            text.push_str(&kinds.join(" or "));
        }
        for rider in &self.riders {
            text.push_str(&format!(
                " +{} {}",
                rider.dice,
                content.damage_type(rider.kind).name
            ));
        }
        text
    }

    /// The whole line as the character screen and the dump write it: `+4 to hit, 1d8+2 slashing`,
    /// `+3 to hit, 1d6 piercing, range 10`.
    pub fn summary(&self, content: &Content) -> String {
        let mut text = format!(
            "{} to hit, {}",
            self.to_hit_text(),
            self.damage_text(content)
        );
        if let Some(range) = self.range {
            text.push_str(&format!(", range {range}"));
        }
        text
    }
}

impl Actor {
    /// The attacks it has ready, as it knows them (module docs): the main hand's blow, the off
    /// hand's with two weapons, and the missile weapon's shot or throw, in that order.
    pub fn known_attacks(&self, content: &Content) -> Vec<OwnAttack> {
        let me = self.as_known();
        let mut attacks = Vec::new();
        let main = match me.weapon_item(content) {
            Some(item) => self.inventory[item].title(content),
            None => match &content.creature(me.creature).attack {
                Some(natural) => natural.name.clone(),
                None => "bare hands".into(),
            },
        };
        attacks.push(OwnAttack::of(
            AttackSource::MainHand,
            AttackKind::Melee,
            main,
            me.melee(content),
            None,
        ));
        if let (Some((item, _)), Some((_, profile))) =
            (me.off_hand(content), me.off_hand_blow(content))
        {
            let name = self.inventory[item].title(content);
            attacks.push(OwnAttack::of(
                AttackSource::OffHand,
                AttackKind::Melee,
                name,
                profile,
                None,
            ));
        }
        if let (Some(item), Some(profile)) = (me.launcher_item(content), me.ranged(content)) {
            let name = self.inventory[item].title(content);
            let range = me.launcher_range(content);
            attacks.push(OwnAttack::of(
                AttackSource::Missile,
                AttackKind::Ranged,
                name,
                profile,
                range,
            ));
        } else if let Some(item) = me.ready_to_throw(content)
            && let Some(profile) = me.thrown(item, content)
        {
            let name = self.inventory[item].title(content);
            let range = me.throw_range(item, content);
            attacks.push(OwnAttack::of(
                AttackSource::Missile,
                AttackKind::Thrown,
                name,
                profile,
                range,
            ));
        }
        attacks
    }

    /// It as the player knows it: every unidentified item it carries without its magic.
    fn as_known(&self) -> Actor {
        let mut me = self.clone();
        for stack in &mut me.inventory {
            *stack = stack.as_known();
        }
        me
    }
}
