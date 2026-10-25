//! The birth sign's gifts in the world (spec §8 "Birth sign"): the talents a character's sign
//! grants ([`crate::character::Character::grants`]) are put into the player's
//! [`Actor::talents`](super::Actor) when a run starts and again whenever one is restored, so they
//! are never stored as a choice: a gift added to the data reaches a saved character, one taken
//! out leaves it, and no save breaks. What each gift does is read through
//! [`crate::talent::TalentMods`] where its rule lives, as every talent is.

use super::spell::Caster;
use super::{ActorId, World};
use crate::action::{Event, SpellShape};
use crate::character::Character;
use crate::content::{Content, TerrainDef};
use crate::damage::Damage;
use crate::geom::Pos;
use crate::ice::Shock;
use crate::talent::Talents;

/// Puts the talents `character`'s race grants (Versatile, `racial.rs`) and the gifts of their
/// sign into `talents` in grant order, in place of whatever granted talents it held (none for a
/// run without a character).
pub(super) fn grant(content: &Content, character: Option<&Character>, talents: &mut Talents) {
    let mut grants = super::racial::grants(content, character);
    grants.extend(character.map_or(Vec::new(), |c| c.grants(content)));
    talents.regrant(content, &grants);
}

/// Whether stepping onto `terrain` is wading (spec §8 "Birth sign": Strong swimmer): water
/// (terrain that freezes) or ground a disease is caught wading in (a marsh).
pub(super) fn wading(content: &Content, terrain: &TerrainDef) -> bool {
    if terrain.freeze.is_some() {
        return true;
    }
    let Some(id) = content.terrain_id(&terrain.id) else {
        return false;
    };
    (content.disease_ids()).any(|(_, d)| d.wading.as_ref().is_some_and(|w| w.terrain.contains(&id)))
}

/// What [`Progress::once`](super::Progress) records once the Phoenix has risen.
pub(crate) const RISEN: &str = "risen";

impl World {
    /// The player, about to die of a hit, a trap, or a status, rises from the ashes instead if
    /// their talents let them and they have not yet (spec §8 "Birth sign": once in a life): the
    /// once is spent, they are Ashen and back at a share of their (Ashen) hit points, and flames
    /// burst from them, one roll of their damage for every creature they reach, melting the ice
    /// there. Returns whether they rose. A gameplay draw (the flames' damage) only then.
    pub(super) fn rise_from_ashes(&mut self, events: &mut Vec<Event>) -> bool {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(rise) = self.state.player().talent_mods(&content).rise else {
            return false;
        };
        if self.state.progress.happened(RISEN) {
            return false;
        }
        self.state.progress.once.insert(RISEN.to_string());
        self.say("You burst into flame and rise from your ashes!");
        // Reborn whole (a lucky day's gift): whole again, and no Ashen (`lucky.rs`).
        if !self.reborn_whole(events) {
            self.afflict(player, rise.status, 1, None, events);
            let me = self.player_mut();
            let max = i64::from(me.max_hp(&content));
            me.hp = i32::try_from(max * i64::from(rise.hp_percent) / 100)
                .unwrap_or(i32::MAX)
                .max(1);
        }
        let center = self.player_pos();
        let burst = &rise.burst;
        let level = self.state.current();
        let opaque = |p: Pos| p != center && level.is_opaque(&content, p);
        let cells = super::spell::burst_cells(center, burst.radius, opaque);
        events.push(Event::Spell {
            effect: burst.spell.clone(),
            origin: center,
            shape: SpellShape::Burst {
                center,
                radius: burst.radius,
            },
            cells: cells.clone(),
        });
        let roll = burst.damage.roll(&mut self.state.rng);
        let here = self.state.current_level.clone();
        let caught: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.id != player && a.level == here && cells.contains(&a.pos))
            .map(|a| a.id)
            .collect();
        for id in caught {
            let damage = Damage::new(roll, burst.kind);
            self.spell_hurts(Caster::Actor(player), id, (center, false), damage, events);
        }
        self.shock(center, burst.radius, Shock::Heat, events);
        true
    }
}
