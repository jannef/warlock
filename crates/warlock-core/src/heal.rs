//! What the Heal skill treats (spec §8 "Skills": the Heal ability, #549): the rules of which of
//! an actor's ailments may be treated now. The treating itself (the check, what comes of it, the
//! time) is `world/heal.rs`.
//!
//! An **ailment** is a status with a `treat` table (`data/status.toml`: bleeding, poisoned) or a
//! disease that shows (`data/diseases.toml`). One may be treated now when:
//!
//! - a status that `stops` (bleeding): always, while it lasts; a failed try may be made again;
//! - any other status (poison): until it has been tried once ([`crate::status::Effect::treated`]);
//! - a disease: once it shows, until it has been tried once ([`crate::disease::Infection::treated`]).
//!   One still incubating is not known to its sufferer, so it is not offered.
//!
//! In status order, then disease order: what a bare `heal` treats is the first of them.

use crate::content::{Content, DiseaseId, StatusId};
use crate::world::Actor;

/// Something the Heal skill treats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ailment {
    /// A status with a `treat` table.
    Status(StatusId),
    /// A disease that shows.
    Disease(DiseaseId),
}

impl Ailment {
    /// Its id in action text (`heal:bleeding`, `heal:filth_fever`): the status's or the
    /// disease's.
    pub fn id(self, content: &Content) -> &str {
        match self {
            Ailment::Status(s) => &content.status(s).id,
            Ailment::Disease(d) => &content.disease(d).id,
        }
    }

    /// The ailment `id` names (a status id, else a disease id), if any.
    pub fn named(content: &Content, id: &str) -> Option<Ailment> {
        (content.status_id(id).map(Ailment::Status))
            .or_else(|| content.disease_id(id).map(Ailment::Disease))
    }

    /// How `actor` knows it, for a choice of what to treat: the status's name ("Bleeding"), the
    /// disease's once they know which it is ("Filth fever"), else "Sickness".
    pub fn name(self, content: &Content, actor: &Actor) -> String {
        match self {
            Ailment::Status(s) => content.status(s).name.clone(),
            Ailment::Disease(d) => capitalized(&noun(content, actor, d)),
        }
    }
}

/// What `actor` calls disease `d` in a sentence: its name once they know which it is ("filth
/// fever"), else "sickness".
pub fn noun(content: &Content, actor: &Actor, d: DiseaseId) -> String {
    let known = actor.diseases.get(d).is_some_and(|i| i.is_known());
    if known {
        content.disease(d).name.clone()
    } else {
        "sickness".to_string()
    }
}

fn capitalized(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().map_or_else(String::new, |c| {
        c.to_uppercase().chain(chars).collect::<String>()
    })
}

/// Whether `actor` may treat `ailment` now: they suffer it, and it is not spent (see the
/// module's rules).
pub fn may_treat(actor: &Actor, content: &Content, ailment: Ailment) -> bool {
    match ailment {
        Ailment::Status(s) => {
            let Some(treat) = content.status(s).treat.as_ref() else {
                return false;
            };
            actor
                .effects
                .get(s)
                .is_some_and(|e| treat.stops || !e.treated)
        }
        Ailment::Disease(d) => (actor.diseases.get(d)).is_some_and(|i| i.shown && !i.treated),
    }
}

/// Everything `actor` may treat now, in status order, then disease order.
pub fn treatable(actor: &Actor, content: &Content) -> Vec<Ailment> {
    let statuses = (actor.effects.iter()).map(|e| Ailment::Status(e.status));
    let diseases = (actor.diseases.iter()).map(|i| Ailment::Disease(i.disease));
    (statuses.chain(diseases))
        .filter(|&a| may_treat(actor, content, a))
        .collect()
}

/// One thing the player may treat now, as the Heal ability's choice offers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnAilment {
    /// How they know it: "Bleeding", "Filth fever", "Sickness".
    pub name: String,
    /// The action that treats it: `heal:ID`.
    pub action: crate::action::Action,
}

/// What `actor` may treat now, each with its name and action, in [`treatable`]'s order.
pub fn own(actor: &Actor, content: &Content) -> Vec<OwnAilment> {
    (treatable(actor, content).into_iter())
        .map(|a| OwnAilment {
            name: a.name(content, actor),
            action: crate::action::Action::Heal(Some(a.id(content).to_string())),
        })
        .collect()
}
