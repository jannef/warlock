//! Rumoured ways (spec §8 "Dialogue", "Rumoured ways"): a topic with a `route`
//! ([`crate::content::Topic::route`]) is a way the speaker describes, from her door to the place
//! she speaks of. Asking about it marks on the player's overworld map what they would have seen
//! walking that way by day: every cell in line of sight within [`daylight::SIGHT_DAY`] of a cell
//! of the route goes into the overworld's memory as the terrain it shows. Only the terrain grid
//! is read, which holds a hidden location's plain terrain (`world/reveal.rs`), so a rumour never
//! gives a hidden location away; a visible one (Cealdhlāw) is remembered as seeing it would.
//! Asking again marks nothing new. No time, no stream; the memory is the overworld's own, saved
//! as it always is. `world/talk.rs` has the one call.

use super::World;
use crate::content::DialogueId;
use crate::daylight::SIGHT_DAY;
use crate::fov::compute_fov;

/// What the player is told when a rumour marks something new on their map.
pub const MARKED: &str = "You mark the way on your map.";

impl World {
    /// Marks the way topic `topic` of dialogue `dialogue` describes on the player's overworld
    /// map (the module docs), making the overworld first if the run has not been there yet.
    /// Says [`MARKED`] if any cell was new to the map.
    pub(super) fn mark_rumoured_way(&mut self, dialogue: DialogueId, topic: &str) {
        let content = self.content.clone();
        let Some(route) = (content.dialogue(dialogue).topic(topic))
            .map(|t| t.route_cells())
            .filter(|r| !r.is_empty())
        else {
            return;
        };
        let Some(overworld) = content
            .overworld_of(&self.state.campaign)
            .map(|o| o.id.clone())
        else {
            return;
        };
        if self.visit(&overworld, None).is_err() {
            return;
        }
        let Some(level) = self.state.levels.get_mut(&overworld) else {
            return;
        };
        let mut seen = Vec::new();
        for &at in &route {
            compute_fov(
                at,
                SIGHT_DAY,
                |p| level.is_opaque(&content, p),
                |p| seen.push(p),
            );
        }
        let mut marked = false;
        for p in seen {
            let Some(&terrain) = level.terrain.get(p) else {
                continue;
            };
            if level.memory.get(p) != Some(&Some(terrain)) {
                level.memory.set(p, Some(terrain));
                marked = true;
            }
        }
        if marked {
            self.say(MARKED);
        }
    }
}
