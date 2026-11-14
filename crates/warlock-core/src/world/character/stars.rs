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
use crate::talent::{Taken, Talents};

/// Puts the talents `character`'s race grants (Versatile, `racial.rs`), the gifts of their
/// sign, and the flaws a forgotten altar marked them with (`boons`, spec §7 "Special rooms":
/// Forsaken) into `talents` in grant order, in place of whatever granted talents it held (none
/// for a run without a character); an altar's other boons (Chosen) are held too.
pub(super) fn grant(
    content: &Content,
    character: Option<&Character>,
    boons: &[Taken],
    talents: &mut Talents,
) {
    let mut grants = super::racial::grants(content, character);
    grants.extend(character.map_or(Vec::new(), |c| c.grants(content)));
    grants.extend(boons.iter().map(|b| b.id.as_str()));
    talents.regrant(content, &grants);
    for boon in boons {
        talents.take(&boon.id, boon.choice.as_deref());
    }
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
/// What it records once it has risen the second time Second rising allows.
pub(crate) const RISEN_TWICE: &str = "risen_twice";
/// What it records once it has risen by a profane gift (Twice-born, docs/dev/profane-gifts.md):
/// a once of its own, so a Phoenix who is Twice-born rises once by each.
pub(crate) const PROFANE_RISEN: &str = "profane_risen";

/// Who gives `actor` the rising from the ashes: a talent of theirs that is no profane gift (the
/// Phoenix's sign), and a profane gift (Twice-born).
fn rising_sources(actor: &super::Actor, content: &Content) -> (bool, bool) {
    let book = content.talents();
    let (mut sign, mut profane) = (false, false);
    for taken in actor.talents.iter() {
        let Some(id) = book.id(&taken.id) else {
            continue;
        };
        let rises = (book.get(id).effects.iter()).any(|e| {
            matches!(
                e,
                crate::content::TalentEffect::Vigour(
                    crate::talent::VigourEffect::RiseFromAshes { .. }
                )
            )
        });
        if rises {
            if content.profane().gift(id).is_some() {
                profane = true;
            } else {
                sign = true;
            }
        }
    }
    (sign, profane)
}

impl World {
    /// The player, about to die of a hit, a trap, or a status, rises from the ashes instead if
    /// their talents let them and they have not yet (spec §8 "Birth sign": once in a life): the
    /// once is spent, they are Ashen and back at a share of their (Ashen) hit points, and flames
    /// burst from them, one roll of their damage for every creature they reach, melting the ice
    /// there. Returns whether they rose. A gameplay draw (the flames' damage) only then.
    pub(super) fn rise_from_ashes(&mut self, events: &mut Vec<Event>) -> bool {
        let content = self.content.clone();
        let player = self.state.player;
        let mods = self.state.player().talent_mods(&content).into_owned();
        let Some(rise) = mods.vigour.rise else {
            return false;
        };
        // Once in a life by the sign; Second rising (a Phoenix's pick) allows one more; a
        // profane gift (Twice-born) one of its own, after the sign's.
        let (sign, profane) = rising_sources(self.state.player(), &content);
        let happened = |flag: &str| self.state.progress.happened(flag);
        let flag = if sign && !happened(RISEN) {
            RISEN
        } else if sign && mods.vigour.second_rising && !happened(RISEN_TWICE) {
            RISEN_TWICE
        } else if profane && !happened(PROFANE_RISEN) {
            PROFANE_RISEN
        } else {
            return false;
        };
        self.state.progress.once.insert(flag.to_string());
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
        let roll = burst.damage.roll(&mut self.state.rng, burst.reroll);
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
