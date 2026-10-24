//! Status effects on actors (spec §8 "Status effects"): the rules of one actor's effects. What
//! the statuses are is data ([`crate::content::StatusDef`], `data/status.toml`); applying them
//! to the world (causes, ticks as the clock moves, deaths, messages) is `world/status.rs`.
//!
//! An actor carries its **effects** ([`Effects`]): at most one of each status, each with a
//! **severity** in degrees (1 up to the status's `max`), its progress toward its next **tick**
//! and toward **easing** by a degree, and who caused it. All of it is game time, in whole seconds:
//!
//! - **Ticks**: a status with a `tick` ticks once every `tick` seconds at one degree, `n` times
//!   as often at `n` degrees: the effect banks `seconds × severity` "degree-seconds" ([`Effect::
//!   tick`]) and ticks for every `tick` of them, so a change of severity changes the pace at
//!   once and nothing is lost or gained by splitting time. A tick costs the status's `damage` in
//!   hit points.
//! - **Easing**: a status with `eases` loses a degree every `eases` seconds ([`Effect::wear`]);
//!   at none it is gone. Without, it lasts until cured.
//! - **Stacking** ([`Stacking`]): a new dose of a status the actor has adds its degrees (to the
//!   `max`), or takes the stronger of the two (`refresh` also starts its easing over).
//! - **Stats** ([`Effects::penalty`]): a status's `penalty` lowers (or raises) scores from a
//!   severity on; several add up. Rules read the stats through it
//!   ([`crate::world::Actor::effective_stats`]).
//! - **Healing** ([`Effects::regen`]): the slowest `regen` of its statuses.
//! - **Fighting** ([`Effects::sharpen`], [`Effects::defence`]): while it lasts, a status adds its
//!   `combat` to-hit and damage to every attack and its Dodge and Parry to Defence, as a combat
//!   stance does.
//! - **Slowness** ([`Effects::slow`]): each status multiplies the price of every action by its
//!   `cost` percent.
//! - **Blocks** ([`Effects::blocking`]): the actions a status refuses while it lasts, by kind.
//! - **Follows**: as a status ends, the one it `follows` begins (`world/buff.rs`).
//!
//! Every effect is in the world state (saved, covered by the state hash). The player knows their
//! own statuses by name, and sees on creatures those a status says show (`seen`); never a
//! severity or a timer.

use crate::combat::{Category, Profile};
use crate::content::{ActionKind, Content, Stacking, StatusDef, StatusId};
use crate::stats::{PerStat, Stat};
use crate::talent::TalentMods;
use crate::world::ActorId;
use serde::{Deserialize, Serialize};

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// One status on an actor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effect {
    pub status: StatusId,
    /// How strong it is, in degrees: 1 up to the status's `max`.
    pub severity: u16,
    /// Degree-seconds toward its next tick: each second adds its severity, and every `tick` of
    /// them is a tick. Always below the status's `tick`.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub tick: u32,
    /// Seconds toward easing by a degree; always below the status's `eases`.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub wear: u32,
    /// Who caused it (the last to add to it), if anyone did: a creature that dies of it counts
    /// as killed by them (experience, #133, credits the player's wounds to the player).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<ActorId>,
}

/// What came of a new dose of a status ([`Effects::add`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Added {
    /// The actor did not have it: it begins.
    Began,
    /// It grew stronger.
    Worse,
    /// Nothing changed but perhaps its timers (it was as strong already).
    Same,
}

/// Seconds until `bank` reaches `need` while it grows by `rate` a second: at least 1.
fn until(bank: u64, need: u64, rate: u64) -> u64 {
    need.saturating_sub(bank).div_ceil(rate.max(1)).max(1)
}

impl Effect {
    /// Seconds until its next tick (`None`: it does not tick).
    pub fn until_tick(&self, def: &StatusDef) -> Option<u64> {
        let every = u64::from(def.tick?);
        Some(until(u64::from(self.tick), every, u64::from(self.severity)))
    }

    /// Seconds until it eases by a degree (`None`: it lasts until cured).
    pub fn until_eases(&self, def: &StatusDef) -> Option<u64> {
        self.until_eases_every(def.eases)
    }

    /// Seconds until it eases by a degree when it eases every `every` seconds (its sufferer's
    /// own pace: a talent may ease it faster; `None`: it lasts until cured).
    pub fn until_eases_every(&self, every: Option<u32>) -> Option<u64> {
        Some(until(u64::from(self.wear), u64::from(every?), 1))
    }

    /// `seconds` pass at its present severity: returns how many times it ticked and how many
    /// degrees it eased. Callers keep `seconds` within [`Effect::until_eases`], so its severity
    /// held all along; the severity itself is left to them.
    pub fn advance(&mut self, def: &StatusDef, seconds: u64) -> (u64, u64) {
        self.advance_every(def, def.eases, seconds)
    }

    /// [`Effect::advance`] at its sufferer's own pace of easing, a degree every `eases` seconds.
    pub fn advance_every(
        &mut self,
        def: &StatusDef,
        eases: Option<u32>,
        seconds: u64,
    ) -> (u64, u64) {
        let ticks = def.tick.map_or(0, |every| {
            let banked = u64::from(self.tick) + seconds * u64::from(self.severity);
            self.tick = u32::try_from(banked % u64::from(every)).unwrap_or(0);
            banked / u64::from(every)
        });
        let eased = eases.map_or(0, |every| {
            let worn = u64::from(self.wear) + seconds;
            self.wear = u32::try_from(worn % u64::from(every)).unwrap_or(0);
            worn / u64::from(every)
        });
        (ticks, eased)
    }
}

/// An actor's effects: at most one per status, in status order (the most important first).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Effects(Vec<Effect>);

impl Effects {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Every effect, the most important first.
    pub fn iter(&self) -> impl Iterator<Item = &Effect> {
        self.0.iter()
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut Effect> {
        self.0.iter_mut()
    }

    /// Its effect of `status`, if it suffers it.
    pub fn get(&self, status: StatusId) -> Option<&Effect> {
        self.0.iter().find(|e| e.status == status)
    }

    pub fn has(&self, status: StatusId) -> bool {
        self.get(status).is_some()
    }

    /// A new dose of `severity` degrees of status `status` (`def`), caused by `by`: added by the
    /// status's stacking rule, never above its `max`.
    pub fn add(
        &mut self,
        status: StatusId,
        def: &StatusDef,
        severity: u16,
        by: Option<ActorId>,
    ) -> Added {
        let severity = severity.clamp(1, def.max);
        let Some(effect) = self.0.iter_mut().find(|e| e.status == status) else {
            let at = self.0.partition_point(|e| e.status < status);
            self.0.insert(
                at,
                Effect {
                    status,
                    severity,
                    tick: 0,
                    wear: 0,
                    by,
                },
            );
            return Added::Began;
        };
        let before = effect.severity;
        effect.severity = match def.stacking {
            Stacking::Add => before.saturating_add(severity).min(def.max),
            Stacking::Refresh | Stacking::Highest => before.max(severity),
        };
        if def.stacking == Stacking::Refresh {
            effect.wear = 0;
        }
        if by.is_some() {
            effect.by = by;
        }
        if effect.severity > before {
            Added::Worse
        } else {
            Added::Same
        }
    }

    /// Whether these effects make sense with `content` (a save from other content, or damaged,
    /// does not): known statuses, in order, each once, their severities and timers in range.
    pub fn is_valid_for(&self, content: &Content) -> bool {
        let known = content.status_defs().len();
        let in_order = self.0.windows(2).all(|w| w[0].status < w[1].status);
        in_order
            && self.0.iter().all(|e| {
                usize::from(e.status.0) < known && {
                    let def = content.status(e.status);
                    (1..=def.max).contains(&e.severity)
                        && def.tick.map_or(e.tick == 0, |every| e.tick < every)
                        && def.eases.map_or(e.wear == 0, |every| e.wear < every)
                }
            })
    }

    /// Removes its effect of `status`; returns whether it had one.
    pub fn remove(&mut self, status: StatusId) -> bool {
        let before = self.0.len();
        self.0.retain(|e| e.status != status);
        self.0.len() < before
    }

    /// Removes the effects that have eased away (severity 0), returning them as they were.
    pub(crate) fn drop_spent(&mut self) -> Vec<Effect> {
        let spent = self.0.iter().filter(|e| e.severity == 0).copied().collect();
        self.0.retain(|e| e.severity > 0);
        spent
    }

    /// What its statuses do to each score now (their penalties at their severities, added up);
    /// `None` when they do nothing to any.
    pub fn penalty(&self, content: &Content) -> Option<PerStat<i32>> {
        self.penalty_for(content, &TalentMods::default())
    }

    /// [`Effects::penalty`] for a sufferer with `talents`: a status that never lowers its scores
    /// takes nothing, and the lowerings shrink by what its talents say ([`TalentMods::penalty`]:
    /// poison resistance, Unbowed).
    pub fn penalty_for(&self, content: &Content, talents: &TalentMods) -> Option<PerStat<i32>> {
        let mut total: Option<PerStat<i32>> = None;
        for e in &self.0 {
            if let Some(by) = content.status(e.status).penalty_at(e.severity) {
                let sum = total.get_or_insert(PerStat::splat(0));
                for stat in Stat::ALL {
                    sum[stat] += talents.penalty(e.status, stat, i32::from(by[stat]));
                }
            }
        }
        total
    }

    /// `profile` with what its statuses add to the to-hit and to every roll of the damage of its
    /// attacks (their `combat`, added up).
    pub fn sharpen(&self, content: &Content, profile: Profile) -> Profile {
        let (to_hit, damage) = (self.0.iter())
            .map(|e| content.status(e.status).combat)
            .fold((0, 0), |(h, d), c| (h + c.to_hit, d + c.damage));
        Profile {
            to_hit: profile.to_hit + to_hit,
            bonus: profile.bonus + damage,
            ..profile
        }
    }

    /// The Defence bonuses its statuses give, one per status and category (none that are zero):
    /// Dodge bonuses stack, and of the Parry bonuses the highest counts, as always.
    pub fn defence<'a>(
        &'a self,
        content: &'a Content,
    ) -> impl Iterator<Item = (Category, i32)> + 'a {
        (self.0.iter())
            .map(|e| content.status(e.status).combat)
            .flat_map(|c| [(Category::Dodge, c.dodge), (Category::Parry, c.parry)])
            .filter(|&(_, bonus)| bonus != 0)
    }

    /// `price` as its statuses make it: each multiplies it by its `cost` percent, rounded up.
    pub fn slow(&self, content: &Content, price: i64) -> i64 {
        (self.0.iter())
            .map(|e| i64::from(content.status(e.status).cost))
            .filter(|&cost| cost != 100)
            .fold(price, |price, cost| {
                price.saturating_mul(cost).saturating_add(99) / 100
            })
    }

    /// The first (the most important) of its statuses that refuses actions of `kind` while it
    /// lasts (`blocks`), if any.
    pub fn blocking(&self, content: &Content, kind: ActionKind) -> Option<StatusId> {
        (self.0.iter())
            .map(|e| e.status)
            .find(|&s| content.status(s).blocks.contains(&kind))
    }

    /// How fast hit points come back while these last, in percent of normal: the slowest of
    /// their statuses' `regen`, 100 without any.
    pub fn regen(&self, content: &Content) -> u16 {
        (self.0.iter())
            .map(|e| content.status(e.status).regen)
            .min()
            .unwrap_or(100)
    }

    /// How fast mana comes back while these last (spec §8 "Mana"): the slowest of their
    /// statuses' `mana_regen`, 100 without any.
    pub fn mana_regen(&self, content: &Content) -> u16 {
        (self.0.iter())
            .map(|e| content.status(e.status).mana_regen)
            .min()
            .unwrap_or(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;

    /// The built-in statuses.
    fn content() -> Content {
        Content::load(&ContentFiles::builtin()).expect("builtin content")
    }

    fn def(content: &Content, id: &str) -> (StatusId, StatusDef) {
        let status = content.status_id(id).expect(id);
        (status, content.status(status).clone())
    }

    #[test]
    fn ticks_come_faster_with_severity_and_split_time_changes_nothing() {
        let content = content();
        let (status, bleeding) = def(&content, "bleeding");
        let every = u64::from(bleeding.tick.expect("bleeding ticks"));
        let mut one = Effect {
            status,
            severity: 1,
            tick: 0,
            wear: 0,
            by: None,
        };
        let mut three = Effect { severity: 3, ..one };
        assert_eq!(one.until_tick(&bleeding), Some(every));
        assert_eq!(three.until_tick(&bleeding), Some(every.div_ceil(3)));
        // Within an easing, three degrees tick three times as often.
        let span = u64::from(bleeding.eases.expect("bleeding eases")) - 1;
        let (a, _) = one.advance(&bleeding, span);
        let (b, _) = three.advance(&bleeding, span);
        assert_eq!((a, b), (span / every, 3 * span / every));
        // A span passed in pieces ticks exactly as often as in one go.
        let mut whole = Effect {
            severity: 2,
            tick: 0,
            wear: 0,
            ..one
        };
        let mut pieces = whole;
        let (t, _) = whole.advance(&bleeding, 50);
        let split: u64 = [7, 13, 1, 29]
            .iter()
            .map(|&s| pieces.advance(&bleeding, s).0)
            .sum();
        assert_eq!((t, whole), (split, pieces));
    }

    #[test]
    fn doses_stack_by_the_status_rule() {
        let content = content();
        let (status, bleeding) = def(&content, "bleeding");
        let mut effects = Effects::default();
        assert_eq!(
            effects.add(status, &bleeding, 1, Some(ActorId(3))),
            Added::Began
        );
        assert_eq!(effects.add(status, &bleeding, 2, None), Added::Worse);
        let e = *effects.get(status).expect("bleeding");
        assert_eq!(
            (e.severity, e.by),
            (3, Some(ActorId(3))),
            "added; the cause kept"
        );
        // Never past the max.
        effects.add(status, &bleeding, bleeding.max, Some(ActorId(4)));
        let e = *effects.get(status).expect("bleeding");
        assert_eq!((e.severity, e.by), (bleeding.max, Some(ActorId(4))));
        assert_eq!(effects.add(status, &bleeding, 1, None), Added::Same);
        // The others keep the stronger, and refresh starts its easing over.
        for (stacking, wear) in [(Stacking::Highest, 7), (Stacking::Refresh, 0)] {
            let def = StatusDef {
                stacking,
                ..bleeding.clone()
            };
            let mut effects = Effects::default();
            effects.add(status, &def, 3, None);
            effects.iter_mut().for_each(|e| e.wear = 7);
            assert_eq!(effects.add(status, &def, 2, None), Added::Same);
            let e = *effects.get(status).expect("it");
            assert_eq!((e.severity, e.wear), (3, wear), "{stacking:?}");
            assert_eq!(effects.add(status, &def, 4, None), Added::Worse);
        }
        assert!(effects.remove(status) && !effects.remove(status));
        assert!(effects.is_empty());
    }

    #[test]
    fn slows_multiply_rounding_up_and_the_first_block_names_why() {
        let mut files = ContentFiles::builtin();
        let (_, text) = files.table("status").next().expect("data/status.toml");
        let text = format!(
            "{text}[[status]]\nid = \"slowed\"\nname = \"Slowed\"\nmax = 1\ncost = 150\n\
             blocks = [\"read\"]\n\
             [[status]]\nid = \"mired\"\nname = \"Mired\"\nmax = 1\ncost = 200\n\
             blocks = [\"read\", \"melee\"]\n\
             combat = {{ to_hit = 2, damage = -1, dodge = -2, parry = 1 }}\n"
        );
        files.insert("data/status.toml", &text);
        let content = Content::load(&files).expect("content");
        let (slowed, slow) = def(&content, "slowed");
        let (mired, mire) = def(&content, "mired");
        let mut effects = Effects::default();
        assert_eq!(effects.slow(&content, 6000), 6000, "nothing: as it was");
        effects.add(slowed, &slow, 1, None);
        assert_eq!(effects.slow(&content, 6000), 9000);
        assert_eq!(effects.slow(&content, 7), 11, "10.5, rounded up");
        effects.add(mired, &mire, 1, None);
        assert_eq!(effects.slow(&content, 6000), 18_000, "×1.5, then ×2");
        // The most important first: Slowed comes before Mired in the file.
        assert_eq!(effects.blocking(&content, ActionKind::Read), Some(slowed));
        assert_eq!(effects.blocking(&content, ActionKind::Melee), Some(mired));
        assert_eq!(effects.blocking(&content, ActionKind::Wait), None);
        // Fighting: to-hit and damage add to a profile; Dodge and Parry are bonuses of their own.
        let profile = Profile {
            to_hit: 3,
            damage: "1d3".parse().expect("dice"),
            bonus: 1,
            threat: 24,
            crit: 2,
            confirm: 0,
            extra: None,
            penetration: 0,
            kinds: Default::default(),
            riders: Vec::new(),
        };
        let sharp = effects.sharpen(&content, profile);
        assert_eq!((sharp.to_hit, sharp.bonus), (5, 0));
        let defence: Vec<(Category, i32)> = effects.defence(&content).collect();
        assert_eq!(defence, [(Category::Dodge, -2), (Category::Parry, 1)]);
    }

    #[test]
    fn effects_keep_status_order_and_combine_penalties_and_healing() {
        let content = content();
        let (bleed, bleeding) = def(&content, "bleeding");
        let (poison, poisoned) = def(&content, "poisoned");
        let mut effects = Effects::default();
        effects.add(poison, &poisoned, 1, None);
        effects.add(bleed, &bleeding, 1, None);
        let order: Vec<StatusId> = effects.iter().map(|e| e.status).collect();
        assert_eq!(order, [bleed, poison], "the most important first");
        assert_eq!(effects.regen(&content), bleeding.regen.min(poisoned.regen));
        assert_eq!(Effects::default().regen(&content), 100);
        // Poison does nothing to stats until it is strong.
        assert_eq!(effects.penalty(&content), None);
        let from = poisoned.penalty.expect("poison has a penalty").from;
        effects.add(poison, &poisoned, from, None);
        let penalty = effects.penalty(&content).expect("strong poison");
        assert_eq!(
            (
                penalty[Stat::Strength],
                penalty[Stat::Dexterity],
                penalty[Stat::Toughness]
            ),
            (-1, -1, 0)
        );
    }
}
