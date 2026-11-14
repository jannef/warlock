//! The beast talents (spec §8 "Allies"): what a talent does with the holder's companion, its
//! familiar, and beasts. [`BeastEffect`] is the data (a talent's `[[effect]]` of one of
//! [`BeastEffect::KINDS`]), [`BeastMods`] the sum every rule reads (`TalentMods::beast`). One
//! of the areas of [`crate::content::TalentEffect`] and [`crate::talent::TalentMods`]
//! (#1495): a new beast effect is a variant here, its field and its arm in [`BeastMods::add`]
//! here, its wiki text in `xtask/src/wiki/talents/beast.rs`, and its one read.

use serde::Deserialize;

/// What a beast talent does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BeastEffect {
    /// An animal companion from the first turn (Animal companion, the ranger's level 1; spec §8
    /// "Allies": `World::bring_companion`): a beast of its race's list, or rarely a rare one.
    Companion,
    /// A familiar from the first turn (Familiar, the witch's level 1; spec §8 "Allies":
    /// `World::bring_familiar`): a small beast of its race's list, or rarely a rare one, whose
    /// touch is her evil eye.
    Familiar,
    /// `bonus` on its taming checks (Animal companion's edge; the Tame skill ability reads it).
    Tame { bonus: i32 },
    /// It sees what its companion sees, by its companion's eyes and nose (Hunter's bond; the
    /// second viewpoint in `Perception::compute`).
    SharedSight,
    /// It sees what its familiar sees, by its familiar's eyes and nose (Familiar's sight; the same
    /// second viewpoint, `familiars.toml`'s `nose` at the least); no buff crosses.
    FamiliarSight,
    /// A buff status that lands on it or its companion lands on the other too (Hunter's bond;
    /// `world/allies/bond.rs`); never a hex.
    SharedBuffs,
    /// A creature beside both it and its companion is flanked: its melee blows and the
    /// companion's at it are `to_hit` likelier to hit (Pack tactics; `world/allies/pack.rs`).
    PackTactics { to_hit: i32 },
    /// Its companion's attacks carry its Favoured prey bonus against its favoured prey (Pack
    /// tactics; `world/allies/pack.rs`).
    SharesFavouredPrey,
    /// Creatures of animal factions are neutral to it until it attacks one of that type
    /// (Beast-friend).
    BeastFriend,
    /// Creatures whose type carries tag `tag` are neutral to it until it attacks one of that
    /// type: Beast-friend's rule by tag (Rat-king's court's vermin; `world/allies/beast_friend.rs`).
    KinTo { tag: String },
}

impl BeastEffect {
    /// The `kind`s of the data that are beast effects, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`BeastEffect`]).
    pub const KINDS: [&'static str; 10] = [
        "beast_friend",
        "companion",
        "familiar",
        "familiar_sight",
        "kin_to",
        "pack_tactics",
        "shared_buffs",
        "shared_sight",
        "shares_favoured_prey",
        "tame",
    ];
}

/// What an actor's beast talents do, summed: one field per effect kind, read by one rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BeastMods {
    /// An animal companion from the first turn (Animal companion; `world/allies/companion.rs`).
    pub companion: bool,
    /// A familiar from the first turn (Familiar; `world/allies/familiar.rs`).
    pub familiar: bool,
    /// Added to its taming checks (Animal companion's edge; the Tame skill ability reads it).
    pub tame: i32,
    /// It sees what its companion sees (Hunter's bond; `Perception::compute`).
    pub shared_sight: bool,
    /// It sees what its familiar sees (Familiar's sight; `Perception::compute`).
    pub familiar_sight: bool,
    /// Its buffs and its companion's are shared (Hunter's bond; `world/allies/bond.rs`).
    pub shared_buffs: bool,
    /// To hit on its melee blows and its companion's at a creature beside them both (Pack
    /// tactics; `world/allies/pack.rs`).
    pub pack_tactics: i32,
    /// Its companion's attacks carry its Favoured prey (Pack tactics; `world/allies/pack.rs`).
    pub shares_favoured_prey: bool,
    /// Creatures of animal factions are neutral to it until it attacks one of their type.
    pub beast_friend: bool,
    /// The tags whose creatures are neutral to it until it attacks one of their type (`kin_to`).
    pub kin_tags: Vec<String>,
}

impl BeastMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, effect: &BeastEffect) {
        match effect {
            BeastEffect::Companion => self.companion = true,
            BeastEffect::Familiar => self.familiar = true,
            BeastEffect::Tame { bonus } => self.tame += *bonus,
            BeastEffect::SharedSight => self.shared_sight = true,
            BeastEffect::FamiliarSight => self.familiar_sight = true,
            BeastEffect::SharedBuffs => self.shared_buffs = true,
            BeastEffect::PackTactics { to_hit } => self.pack_tactics += *to_hit,
            BeastEffect::SharesFavouredPrey => self.shares_favoured_prey = true,
            BeastEffect::BeastFriend => self.beast_friend = true,
            BeastEffect::KinTo { tag } => {
                if !self.kin_tags.contains(tag) {
                    self.kin_tags.push(tag.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every kind in [`BeastEffect::KINDS`] is a variant of its own, and every variant is in it
    /// (the match below stops compiling when a variant is added: add its kind too).
    #[test]
    fn kinds_are_the_variants() {
        let mut seen = Vec::new();
        for kind in BeastEffect::KINDS {
            let fields = match kind {
                "tame" => "bonus = 1",
                "pack_tactics" => "to_hit = 1",
                "kin_to" => "tag = \"vermin\"",
                _ => "",
            };
            let effect: BeastEffect = toml::from_str(&format!("kind = \"{kind}\"\n{fields}"))
                .unwrap_or_else(|e| panic!("{kind}: {e}"));
            let variant = std::mem::discriminant(&effect);
            assert!(!seen.contains(&variant), "{kind} twice");
            seen.push(variant);
            match effect {
                BeastEffect::Companion
                | BeastEffect::Familiar
                | BeastEffect::Tame { .. }
                | BeastEffect::SharedSight
                | BeastEffect::FamiliarSight
                | BeastEffect::SharedBuffs
                | BeastEffect::PackTactics { .. }
                | BeastEffect::SharesFavouredPrey
                | BeastEffect::BeastFriend
                | BeastEffect::KinTo { .. } => {}
            }
        }
        assert_eq!(seen.len(), 10, "one kind a variant");
    }
}
