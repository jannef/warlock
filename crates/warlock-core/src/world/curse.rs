//! Cursed rooms (spec §7 "Special rooms", #408): the dead rise. A creature that dies on a cell
//! whose effects raise the dead ([`crate::content::Here::rise`]: a cursed room, or a whole cursed
//! level), is not of the rise's faction, and leaves a corpse is marked to rise
//! ([`World::curse_corpse`], one line in `fight.rs::creature_dies`): what it rises as is drawn
//! at once from the gameplay stream ([`rises_as`]: a type of the faction of the dead's level,
//! else of the nearest level, the lower on a tie), and a [`Rising`] is kept in the level's state
//! until it is due. At the end of every turn ([`World::rise_tick`], beside the spawns) the
//! risings due on the **current** level rise: if a corpse of the dead's type still lies where it
//! died, one is taken and the new creature stands there (or on the nearest free cell). A corpse
//! picked up, eaten, or destroyed meanwhile does not rise; one carried out and dropped elsewhere
//! is not cursed again. Levels the player is not on stay frozen, so what was due while they
//! were away rises on their first turn back. Grave warden: what the player kills never rises.

use super::{World, travel};
use crate::content::{Content, CreatureId, CreatureSpawn, FactionId};
use crate::geom::Pos;
use crate::noise::Act;
use crate::rng::Rng;
use crate::time::GameTime;
use serde::{Deserialize, Serialize};

/// One of the dead due to rise (kept in the level's state, saved and hashed).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rising {
    /// When it rises.
    pub(crate) at: GameTime,
    /// Where it died, where its corpse lies.
    pub(crate) pos: Pos,
    /// What died (whose corpse rises).
    pub(crate) from: CreatureId,
    /// What it rises as.
    pub(crate) into: CreatureId,
}

impl Rising {
    /// Whether its creature types are in `content` (a guard for loaded saves).
    pub(crate) fn is_valid_for(&self, content: &Content) -> bool {
        let types = content.creature_defs().len();
        usize::from(self.from.0) < types && usize::from(self.into.0) < types
    }

    /// When it rises.
    pub fn at(&self) -> GameTime {
        self.at
    }

    /// Where its corpse lies.
    pub fn pos(&self) -> Pos {
        self.pos
    }

    /// What died.
    pub fn dead(&self) -> CreatureId {
        self.from
    }

    /// What it rises as.
    pub fn risen(&self) -> CreatureId {
        self.into
    }
}

/// The types the dead of `level` may rise as in `campaign`: the types of `faction`, never
/// unique, shared or the campaign's own, of `level` if any is, else of the nearest level that has
/// one (the lower on a tie), in id order. Empty when the faction has none at all.
pub fn rise_candidates(
    content: &Content,
    faction: FactionId,
    campaign: &str,
    level: u8,
) -> Vec<CreatureId> {
    let of_faction: Vec<(CreatureId, u8)> = (content.creature_defs().iter().enumerate())
        .filter(|(_, d)| {
            d.faction == Some(faction)
                && !d.unique
                && d.campaign.as_deref().is_none_or(|c| c == campaign)
        })
        .filter_map(|(i, d)| Some((CreatureId(u16::try_from(i).ok()?), d.level)))
        .collect();
    let distance = |l: u8| (l.abs_diff(level), l);
    let Some(nearest) = of_faction.iter().map(|&(_, l)| distance(l)).min() else {
        return Vec::new();
    };
    (of_faction.into_iter())
        .filter(|&(_, l)| distance(l) == nearest)
        .map(|(id, _)| id)
        .collect()
}

/// What a creature of `level` rises as: one of [`rise_candidates`], drawn alike from `rng`
/// (one draw, none when there is a single candidate or none).
pub fn rises_as(
    content: &Content,
    rng: &mut Rng,
    faction: FactionId,
    campaign: &str,
    level: u8,
) -> Option<CreatureId> {
    let candidates = rise_candidates(content, faction, campaign, level);
    match candidates.len() {
        0 => None,
        1 => Some(candidates[0]),
        n => {
            let i = rng.below(u32::try_from(n).unwrap_or(u32::MAX));
            candidates.get(usize::try_from(i).ok()?).copied()
        }
    }
}

impl World {
    /// Creature `dead` has just died (its corpse, if any, left at its cell): if the effects there
    /// raise the dead and it may rise, its rising is kept; returns the line the player is told,
    /// if they saw it. Grave warden: never what the player (`by_player`) killed.
    pub(super) fn curse_corpse(&mut self, dead: &super::Actor, by_player: bool) -> Option<String> {
        let content = self.content.clone();
        let rise = self.effects_at(&dead.level, dead.pos).rise()?.clone();
        let faction = content.factions().id(&rise.faction)?;
        let def = content.creature(dead.creature);
        if def.faction == Some(faction) || !def.corpse || content.corpse_of(dead.creature).is_none()
        {
            return None;
        }
        if by_player && self.state.player().talent_mods(&content).grave_warden {
            return None;
        }
        let campaign = self.state.campaign.clone();
        let into = rises_as(&content, &mut self.state.rng, faction, &campaign, def.level)?;
        let rising = Rising {
            at: self.state.time + rise.after,
            pos: dead.pos,
            from: dead.creature,
            into,
        };
        self.state.levels.get_mut(&dead.level)?.risings.push(rising);
        let seen =
            dead.level == self.state.current_level && self.perception.sees_actors_at(dead.pos);
        seen.then(|| {
            format!(
                "{}'s corpse twitches.",
                crate::content::the_start(&def.name)
            )
        })
    }

    /// The end of a turn: every rising due on the current level rises, in the order they died.
    pub(super) fn rise_tick(&mut self) {
        let now = self.state.time;
        let level = self.state.current_mut();
        if level.risings.is_empty() {
            return;
        }
        let (due, waiting): (Vec<Rising>, Vec<Rising>) =
            level.risings.drain(..).partition(|r| r.at <= now);
        level.risings = waiting;
        for rising in due {
            if self.state.death.is_some() {
                return;
            }
            self.rise(rising);
        }
    }

    /// `rising` is due: if a corpse of its dead still lies where it died, one is taken and the
    /// risen creature stands there (or on the nearest free cell). The player is told if they see
    /// it, else they may hear it groan.
    fn rise(&mut self, rising: Rising) {
        let content = self.content.clone();
        let Some(corpse) = content.corpse_of(rising.from) else {
            return;
        };
        let id = self.state.current_level.clone();
        let level = self.state.current_mut();
        let mut pile = level.objects_at(rising.pos).to_vec();
        let Some(at) = pile.iter().position(|s| s.object == corpse) else {
            return;
        };
        pile[at].count -= 1;
        if pile[at].count == 0 {
            pile.remove(at);
        }
        level.set_pile(rising.pos, pile);
        let spawn = CreatureSpawn {
            creature: rising.into,
            faction: None,
            alignment: None,
            npc: None,
            pos: self.free_cell_near(&id, rising.pos),
            route: Vec::new(),
            carry: Vec::new(),
            equipment: Default::default(),
            rolls_gear: false,
            asleep: false,
            status: Vec::new(),
        };
        let risen = travel::make_creature(&mut self.state, &content, &id, &spawn);
        // A newcomer: a rest looks up at once.
        self.spawned += 1;
        let seen =
            self.perception.sees_actors_at(rising.pos) || self.perception.sees_actors_at(spawn.pos);
        if seen {
            let (dead, risen) = (
                &content.creature(rising.from).name,
                &content.creature(rising.into).name,
            );
            let article = crate::content::article(risen);
            self.say(format!(
                "{}'s corpse rises as {article} {risen}!",
                crate::content::the_start(dead)
            ));
        } else {
            self.make_noise_at(risen, Act::Mutter, 100);
        }
    }

    /// The risings due on the current level, in the order they died (dev dumps).
    pub fn risings(&self) -> &[Rising] {
        &self.state.current().risings
    }
}
