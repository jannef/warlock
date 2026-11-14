//! Every actor of the run, by id ([`Actors`], `WorldState::actors`), with an index of who is on
//! which level (#1507): a run keeps the actors of every level it visited, and the rules that ask
//! who is on one level (each creature, each turn) read the index instead of all of them.
//!
//! Reading is the map's own (`Deref`); what changes the map goes through the methods here,
//! which keep the index. An actor's level changes only in [`Actors::set_level`]. The index is
//! derived: never saved or hashed (the map serializes as before), rebuilt on load.

use super::{Actor, ActorId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Actors {
    map: BTreeMap<ActorId, Actor>,
    /// The ids on each level, by the level's id (an actor in the retinue too: it keeps the
    /// level it left).
    by_level: BTreeMap<String, BTreeSet<ActorId>>,
}

impl Deref for Actors {
    type Target = BTreeMap<ActorId, Actor>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<'a> IntoIterator for &'a Actors {
    type Item = (&'a ActorId, &'a Actor);
    type IntoIter = std::collections::btree_map::Iter<'a, ActorId, Actor>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl Serialize for Actors {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.map.serialize(s)
    }
}

impl<'de> Deserialize<'de> for Actors {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        BTreeMap::deserialize(d).map(Self::from_map)
    }
}

impl Actors {
    fn from_map(map: BTreeMap<ActorId, Actor>) -> Self {
        let by_level = index(&map);
        Self { map, by_level }
    }

    /// The actors on level `level`, in id order (the order of the whole map, filtered), an
    /// actor in the retinue among them.
    pub fn on_level<'a>(&'a self, level: &str) -> impl Iterator<Item = &'a Actor> + 'a {
        (self.by_level.get(level).into_iter().flatten()).map(|id| &self.map[id])
    }

    /// Actor `id`, to change. Never its level: [`Actors::set_level`] moves it.
    pub fn get_mut(&mut self, id: &ActorId) -> Option<&mut Actor> {
        self.map.get_mut(id)
    }

    /// Every actor, to change. Never their levels: [`Actors::set_level`] moves one.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Actor> {
        self.map.values_mut()
    }

    /// Adds actor `actor` under `id`, replacing any there; returns the one replaced.
    pub fn insert(&mut self, id: ActorId, actor: Actor) -> Option<Actor> {
        let level = actor.level.clone();
        let old = self.map.insert(id, actor);
        if let Some(old) = &old {
            self.unindex(id, &old.level);
        }
        self.by_level.entry(level).or_default().insert(id);
        old
    }

    /// Takes actor `id` out of the run.
    pub fn remove(&mut self, id: &ActorId) -> Option<Actor> {
        let actor = self.map.remove(id)?;
        self.unindex(*id, &actor.level);
        Some(actor)
    }

    /// Keeps only the actors `keep` says to.
    pub fn retain(&mut self, mut keep: impl FnMut(&ActorId, &mut Actor) -> bool) {
        self.map.retain(|id, a| keep(id, a));
        self.by_level = index(&self.map);
    }

    /// Moves actor `id` to level `to`; returns the level it was on.
    pub fn set_level(&mut self, id: ActorId, to: &str) -> Option<String> {
        let actor = self.map.get_mut(&id)?;
        let from = std::mem::replace(&mut actor.level, to.to_string());
        self.unindex(id, &from);
        self.by_level.entry(to.to_string()).or_default().insert(id);
        Some(from)
    }

    fn unindex(&mut self, id: ActorId, level: &str) {
        if let Some(ids) = self.by_level.get_mut(level) {
            ids.remove(&id);
            if ids.is_empty() {
                self.by_level.remove(level);
            }
        }
    }

    /// Whether the index says what the actors' own levels say (`WorldState::check`, and after
    /// every action in debug builds).
    pub(super) fn indexed(&self) -> bool {
        self.by_level == index(&self.map)
    }
}

/// The ids on each level.
fn index(map: &BTreeMap<ActorId, Actor>) -> BTreeMap<String, BTreeSet<ActorId>> {
    let mut by_level: BTreeMap<String, BTreeSet<ActorId>> = BTreeMap::new();
    for (id, actor) in map {
        by_level.entry(actor.level.clone()).or_default().insert(*id);
    }
    by_level
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Content;
    use crate::world::World;
    use std::sync::Arc;

    fn actors() -> Actors {
        let content = Arc::new(Content::builtin().expect("content"));
        World::new(content, "showcase", None)
            .expect("world")
            .state
            .actors
    }

    /// A level's actors are the whole map's on it, in the same order.
    fn agrees(actors: &Actors) {
        assert!(actors.indexed());
        for level in (actors.values())
            .map(|a| a.level.as_str())
            .chain(["nowhere"])
        {
            let indexed: Vec<ActorId> = actors.on_level(level).map(|a| a.id).collect();
            let scanned: Vec<ActorId> = (actors.values())
                .filter(|a| a.level == level)
                .map(|a| a.id)
                .collect();
            assert_eq!(indexed, scanned, "{level}");
        }
    }

    #[test]
    fn the_index_follows_every_change() {
        let mut actors = actors();
        assert!(actors.len() > 2, "the showcase has creatures");
        agrees(&actors);
        let ids: Vec<ActorId> = actors.keys().copied().collect();
        let from = actors.set_level(ids[1], "elsewhere").expect("an actor");
        assert_ne!(from, "elsewhere");
        agrees(&actors);
        assert_eq!(actors.on_level("elsewhere").count(), 1);
        let moved = actors.remove(&ids[1]).expect("there");
        agrees(&actors);
        assert_eq!(actors.on_level("elsewhere").count(), 0);
        let mut back = moved.clone();
        back.level = from;
        assert!(actors.insert(ids[1], back).is_none());
        agrees(&actors);
        // Replacing one with an actor of another level moves it in the index.
        assert!(actors.insert(ids[1], moved).is_some());
        agrees(&actors);
        assert_eq!(actors.on_level("elsewhere").count(), 1);
        actors.retain(|id, _| *id != ids[2]);
        agrees(&actors);
    }

    /// Saved (and hashed) as the map alone, and loaded with its index.
    #[test]
    fn saves_the_map_alone() {
        let actors = actors();
        let json = serde_json::to_string(&actors).expect("serializes");
        assert_eq!(json, serde_json::to_string(&*actors).expect("serializes"));
        let loaded: Actors = serde_json::from_str(&json).expect("loads");
        assert_eq!(loaded, actors);
        agrees(&loaded);
    }
}
