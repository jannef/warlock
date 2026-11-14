//! Content's accessors for creatures: their types, factions, sizes and rarities, the draw,
//! dialogue, corpses, and what actions cost them.

use super::{
    ActionCosts, ActionKind, Animated, Body, Content, CreatureDef, CreatureId, CreatureRarity,
    Dialogue, DialogueId, FactionId, Factions, ObjectId, SizeDef, Speaker, Stance, danger, faction,
    required,
};
use std::collections::BTreeSet;

impl Content {
    /// Whether creature type `c` may come as a stray to an area whose population does not name
    /// it (spec §7 "Danger levels"): hostile to the player, no unique, no animated object, and
    /// not marked `stray = false`.
    pub fn may_stray(&self, c: CreatureId) -> bool {
        let def = self.creature(c);
        let player = self.creature_id(required::PLAYER);
        def.stray
            && !def.unique
            && def.animated.is_none()
            && player.is_some_and(|p| p != c && self.stance(c, p) == faction::Stance::Hostile)
    }

    pub fn creature(&self, id: CreatureId) -> &CreatureDef {
        &self.creatures[usize::from(id.0)]
    }

    pub fn creature_id(&self, name: &str) -> Option<CreatureId> {
        self.creature_ids.get(name).copied()
    }

    pub fn creature_defs(&self) -> &[CreatureDef] {
        &self.creatures
    }

    /// Animated objects (spec §8 "Animated objects"): the tiers, the drop rule, and the bodies
    /// their household types are generated from; empty without `data/animated.toml`.
    pub fn animated(&self) -> &Animated {
        &self.animated
    }

    /// The household body creature type `creature` is made of, if it is an animated one.
    pub fn animated_body(&self, creature: CreatureId) -> Option<&Body> {
        (self.animated).body_of(self.creature(creature).animated.as_ref())
    }

    /// The factions of `data/factions.toml` (empty without the file).
    pub fn factions(&self) -> &Factions {
        &self.factions
    }

    /// How creatures of types `a` and `b` stand toward each other: their factions' stance
    /// (spec §7 "Factions"); neutral without factions. The player is the creature `player`.
    pub fn stance(&self, a: CreatureId, b: CreatureId) -> Stance {
        match (self.creature(a).faction, self.creature(b).faction) {
            (Some(a), Some(b)) => self.factions.stance(a, b),
            _ => Stance::Neutral,
        }
    }

    /// How two actors stand toward each other, given the factions they belong to (their type's,
    /// or the one their level placed them in); neutral if either has none.
    pub fn stance_of(&self, a: Option<FactionId>, b: Option<FactionId>) -> Stance {
        match (a, b) {
            (Some(a), Some(b)) => self.factions.stance(a, b),
            _ => Stance::Neutral,
        }
    }

    /// The sizes of `data/sizes.toml`, in file order (only medium without the file).
    pub fn sizes(&self) -> &[SizeDef] {
        &self.sizes
    }

    /// The size at `index` in [`Content::sizes`] (a race's or creature type's `size`).
    pub fn size(&self, index: usize) -> &SizeDef {
        &self.sizes[index]
    }

    /// The creature rarities of `data/creature_rarities.toml`, in file order (only common
    /// without the file).
    pub fn creature_rarities(&self) -> &[CreatureRarity] {
        &self.creature_rarities
    }

    /// The rarity at `index` in [`Content::creature_rarities`] (a creature type's `rarity`).
    pub fn creature_rarity(&self, index: usize) -> &CreatureRarity {
        &self.creature_rarities[index]
    }

    /// The default odds of a draw (`data/dungeons.toml`'s `[defaults]`; spec §7 "Danger
    /// levels"), if given.
    pub fn draw_odds(&self) -> Option<danger::Odds> {
        self.draw_odds
    }

    /// The candidates of a draw ([`danger::draw`]) among `kinds`: each type with its levels (a
    /// family's range, `animated_blade`'s 5 to 20; else its own level), its rarity's factor, and
    /// its `dread` flag, those `met` (uniques the run has had, by id) left out.
    pub fn candidates(
        &self,
        kinds: &[CreatureId],
        met: &BTreeSet<String>,
    ) -> Vec<danger::Candidate> {
        (kinds.iter())
            .map(|&c| (c, self.creature(c)))
            .filter(|(_, def)| !(def.unique && met.contains(&def.id)))
            .map(|(c, def)| {
                let factor = self.creature_rarity(def.rarity).factor;
                match self.family_levels(c) {
                    // A member names its whole family.
                    Some(levels) => (self.family(c), levels, factor, def.dread),
                    None => (c, (def.level, def.level), factor, def.dread),
                }
            })
            .collect()
    }

    /// The candidates of a floor's population or a timed spawn ([`Content::candidates`]), each
    /// type not hostile to the player by default (the neutral animals; their stance in the data,
    /// whatever empathy a player has) weighed at `odds.neutral` % of its rarity's factor (#1195:
    /// a dungeon is for fighting).
    pub fn draw_candidates(
        &self,
        kinds: &[CreatureId],
        met: &BTreeSet<String>,
        odds: &danger::Odds,
    ) -> Vec<danger::Candidate> {
        let mut candidates = self.candidates(kinds, met);
        if odds.neutral < 100 {
            let player = self.creature_id(required::PLAYER);
            for (c, _, factor, _) in &mut candidates {
                if player.is_some_and(|p| self.stance(*c, p) != faction::Stance::Hostile) {
                    // At least 1 unless 0 % (a rare type at a small share still comes).
                    let scaled = u32::from(*factor) * u32::from(odds.neutral) / 100;
                    let scaled = if odds.neutral > 0 { scaled.max(1) } else { 0 };
                    *factor = u16::try_from(scaled).unwrap_or(u16::MAX);
                }
            }
        }
        candidates
    }

    /// The index of size `id` in [`Content::sizes`].
    pub fn size_index(&self, id: &str) -> Option<usize> {
        self.sizes.iter().position(|s| s.id == id)
    }

    /// The corpse a creature of type `creature` leaves when it dies, if any.
    pub fn corpse_of(&self, creature: CreatureId) -> Option<ObjectId> {
        self.corpses.get(usize::from(creature.0)).copied().flatten()
    }

    /// What creatures say (`data/dialogue.toml`; empty without it), in file order.
    pub fn dialogues(&self) -> &[Dialogue] {
        &self.dialogues
    }

    pub fn dialogue(&self, id: DialogueId) -> &Dialogue {
        &self.dialogues[usize::from(id.0)]
    }

    /// The dialogue of a creature of type `creature` that a level named `npc`: the npc's own if
    /// it has one, else its type's; `None` if neither talks.
    pub fn dialogue_of(&self, creature: CreatureId, npc: Option<&str>) -> Option<DialogueId> {
        let find = |speaker: &Speaker| self.dialogues.iter().position(|d| d.speaker == *speaker);
        let npc = npc.and_then(|n| find(&Speaker::Npc(n.to_string())));
        npc.or_else(|| find(&Speaker::Creature(creature)))
            .map(|i| DialogueId(i as u16))
    }

    /// The npc dialogue of unique character `npc`, if it has one.
    pub fn dialogue_of_npc(&self, npc: &str) -> Option<DialogueId> {
        let speaker = Speaker::Npc(npc.to_string());
        (self.dialogues.iter().position(|d| d.speaker == speaker)).map(|i| DialogueId(i as u16))
    }

    /// What every action costs (`data/actions.toml`; 1000 each without it).
    pub fn action_costs(&self) -> &ActionCosts {
        &self.actions
    }

    /// What action `kind` costs a creature of type `creature`: its type's own attack cost for an
    /// attack, if it has one, else the action's cost (spec §8 "Time").
    pub fn action_cost(&self, creature: CreatureId, kind: ActionKind) -> u32 {
        // A throw of its own (spec §8 "Ranged combat": "Creatures' throws") costs its own.
        let throw = self.creature(creature).throw.as_ref().and_then(|t| t.cost);
        if let Some(cost) = throw.filter(|_| kind == ActionKind::Throw) {
            return cost;
        }
        match self.creature(creature).attack_cost {
            Some(cost) if kind.is_attack() => cost,
            _ => self.actions.cost(kind),
        }
    }
}
