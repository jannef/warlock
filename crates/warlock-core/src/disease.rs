//! Diseases on actors (spec §8 "Diseases"): the rules of one actor's infections. What the
//! diseases are is data ([`crate::content::DiseaseDef`], `data/diseases.toml`); applying them to
//! the world (exposure, their turns as the clock moves, stat damage, deaths, messages) is
//! `world/disease.rs`.
//!
//! An actor carries its **diseases** ([`Diseases`]): at most one infection of each, in disease
//! order, each in one of two stages, all of it game time in whole seconds:
//!
//! - **Incubating**: caught but not yet showing, for the disease's `incubation` (rolled when
//!   caught, in days). Nothing about it can be perceived.
//! - **Showing** ([`Infection::shown`]): its onset came; the actor is Diseased (the one status
//!   all diseases show as) and it **progresses** every `interval` days: a check, which a
//!   failure costs stat damage or hit points, and which won enough times in a row (`recovery`)
//!   shakes it off.
//!
//! [`Infection::left`] counts the seconds to its next turn (its onset, then each progression),
//! so it makes no difference how time is cut up. The sufferer knows which disease it is after
//! its first progression ([`Infection::is_known`]); until then it is only "Diseased". Every
//! infection is in the world state (saved, covered by the state hash).

use crate::content::{Content, DiseaseId};
use crate::world::ActorId;
use serde::{Deserialize, Serialize};

fn is_zero_u16(n: &u16) -> bool {
    *n == 0
}

fn is_zero_u8(n: &u8) -> bool {
    *n == 0
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// One disease an actor carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Infection {
    pub disease: DiseaseId,
    /// Seconds until its next turn: its onset while it incubates, then its next progression.
    /// At least 1.
    pub left: u32,
    /// Whether its incubation is over: it shows, and the actor is Diseased.
    #[serde(default, skip_serializing_if = "is_false")]
    pub shown: bool,
    /// How many times it has progressed (a check each).
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub progressions: u16,
    /// Checks won in a row: enough of them shake it off.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub streak: u8,
    /// Who gave it, if anyone did: a creature it kills counts as killed by them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<ActorId>,
    /// Whether the Heal skill has been tried on it (`world/heal.rs`): one try per disease,
    /// whatever comes of it. It goes with the infection: caught again, it may be tried again.
    #[serde(default, skip_serializing_if = "is_false")]
    pub treated: bool,
}

impl Infection {
    /// Whether the sufferer knows which disease it is: after its first progression. (Heal or
    /// Knowledge, #132, could tell it sooner later.)
    pub fn is_known(&self) -> bool {
        self.progressions > 0
    }
}

/// An actor's diseases: at most one infection of each, in disease order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Diseases(Vec<Infection>);

impl Diseases {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Every infection, in disease order.
    pub fn iter(&self) -> impl Iterator<Item = &Infection> {
        self.0.iter()
    }

    /// Its infection with `disease`, if it carries it.
    pub fn get(&self, disease: DiseaseId) -> Option<&Infection> {
        self.0.iter().find(|i| i.disease == disease)
    }

    pub(crate) fn get_mut(&mut self, disease: DiseaseId) -> Option<&mut Infection> {
        self.0.iter_mut().find(|i| i.disease == disease)
    }

    pub fn has(&self, disease: DiseaseId) -> bool {
        self.get(disease).is_some()
    }

    /// Whether any of them shows: the actor is Diseased.
    pub fn shows(&self) -> bool {
        self.0.iter().any(|i| i.shown)
    }

    /// The diseases the sufferer knows by name, in disease order.
    pub fn known(&self) -> impl Iterator<Item = DiseaseId> + '_ {
        self.0.iter().filter(|i| i.is_known()).map(|i| i.disease)
    }

    /// It catches `disease`, caused by `by`, to show in `incubation` seconds (at least 1).
    /// Returns whether it did: not when it carries it already.
    pub(crate) fn catch(
        &mut self,
        disease: DiseaseId,
        incubation: u64,
        by: Option<ActorId>,
    ) -> bool {
        if self.has(disease) {
            return false;
        }
        let at = self.0.partition_point(|i| i.disease < disease);
        self.0.insert(
            at,
            Infection {
                disease,
                left: u32::try_from(incubation).unwrap_or(u32::MAX).max(1),
                shown: false,
                progressions: 0,
                streak: 0,
                by,
                treated: false,
            },
        );
        true
    }

    /// Removes its infection with `disease`, returning it.
    pub(crate) fn remove(&mut self, disease: DiseaseId) -> Option<Infection> {
        let at = self.0.iter().position(|i| i.disease == disease)?;
        Some(self.0.remove(at))
    }

    /// Seconds until the next turn of any of them (`None`: it carries none).
    pub fn until_next(&self) -> Option<u64> {
        self.0.iter().map(|i| u64::from(i.left)).min()
    }

    /// `seconds` pass (callers keep them within [`Diseases::until_next`]): returns the diseases
    /// whose turn came, in disease order. Their timers are left at 0 for the turn to set.
    pub(crate) fn pass(&mut self, seconds: u64) -> Vec<DiseaseId> {
        let seconds = u32::try_from(seconds).unwrap_or(u32::MAX);
        let mut due = Vec::new();
        for i in &mut self.0 {
            i.left = i.left.saturating_sub(seconds);
            if i.left == 0 {
                due.push(i.disease);
            }
        }
        due
    }

    /// How fast hit points come back while these show, in percent of normal: the slowest
    /// `regen` of the showing diseases that have one, 100 otherwise. (Their status, Diseased,
    /// has its own.)
    pub fn regen(&self, content: &Content) -> u16 {
        (self.0.iter())
            .filter(|i| i.shown)
            .filter_map(|i| content.disease(i.disease).regen)
            .min()
            .unwrap_or(100)
    }

    /// How fast mana comes back while these show (spec §8 "Mana"): the slowest `mana_regen` of
    /// the showing diseases that have one, 100 otherwise. (Diseased has its own.)
    pub fn mana_regen(&self, content: &Content) -> u16 {
        (self.0.iter())
            .filter(|i| i.shown)
            .filter_map(|i| content.disease(i.disease).mana_regen)
            .min()
            .unwrap_or(100)
    }

    /// Whether these make sense with `content` (a save from other content, or damaged, does
    /// not): known diseases, in order, each once, their timers in range, and nothing counted of
    /// one still incubating.
    pub fn is_valid_for(&self, content: &Content) -> bool {
        let known = content.disease_defs().len();
        let in_order = self.0.windows(2).all(|w| w[0].disease < w[1].disease);
        in_order
            && self.0.iter().all(|i| {
                usize::from(i.disease.0) < known && {
                    let def = content.disease(i.disease);
                    i.left >= 1
                        && u64::from(i.left) <= def.longest_wait()
                        && (i.shown || (i.progressions == 0 && i.streak == 0))
                        && (!def.recovers() || i.streak < def.recovery)
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::DAY;

    #[test]
    fn infections_keep_disease_order_and_are_caught_once() {
        let mut diseases = Diseases::default();
        assert!(diseases.catch(DiseaseId(3), DAY, None));
        assert!(diseases.catch(DiseaseId(1), 2 * DAY, Some(ActorId(4))));
        assert!(!diseases.catch(DiseaseId(3), 5, None), "once");
        let order: Vec<DiseaseId> = diseases.iter().map(|i| i.disease).collect();
        assert_eq!(order, [DiseaseId(1), DiseaseId(3)]);
        assert_eq!(diseases.until_next(), Some(DAY));
        assert!(!diseases.shows() && diseases.known().count() == 0);
        // A day passes: the third's turn comes.
        assert!(diseases.pass(DAY - 1).is_empty());
        assert_eq!(diseases.pass(1), [DiseaseId(3)]);
        assert_eq!(diseases.get(DiseaseId(1)).map(|i| i.left), Some(DAY as u32));
        let third = diseases.get_mut(DiseaseId(3)).expect("caught");
        third.shown = true;
        third.left = DAY as u32;
        assert!(diseases.shows() && diseases.known().count() == 0);
        diseases.get_mut(DiseaseId(3)).expect("it").progressions = 1;
        assert_eq!(diseases.known().collect::<Vec<_>>(), [DiseaseId(3)]);
        assert_eq!(
            diseases.remove(DiseaseId(1)).map(|i| i.by),
            Some(Some(ActorId(4)))
        );
        assert!(diseases.remove(DiseaseId(1)).is_none());
        assert!(!diseases.is_empty());
    }
}
