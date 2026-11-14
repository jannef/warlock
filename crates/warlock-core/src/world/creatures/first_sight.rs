//! First-sight lines (docs/design/monster-dread.md §9 channel 4; #1133): a creature type with a
//! `first_sight` line ("A basilisk. Do not meet its eyes.") has it said once a run, the first
//! time the player sees one, after the turn's other perception lines. The types whose line was
//! said are `Progress::sighted`, saved with the run.

use crate::world::{ActorKind, World};

impl World {
    /// Says the first-sight line of every type the player sees now for the first time (module
    /// docs), in the order perception lists them.
    pub(in crate::world) fn first_sight(&mut self) {
        let content = self.content.clone();
        let seen: Vec<_> = (self.perception().actors().iter())
            .filter(|a| a.kind == ActorKind::Creature && self.state.actors.contains_key(&a.id))
            .map(|a| a.creature)
            .collect();
        for creature in seen {
            let def = content.creature(creature);
            let Some(line) = def.first_sight.as_ref() else {
                continue;
            };
            if self.state.progress.sighted.insert(def.id.clone()) {
                self.say(line.clone());
            }
        }
    }
}
