//! Content's accessors for the rules files: the numbers of one system each, most of them optional.

use super::{
    AllyRules, CompanionRules, Content, EncounterRules, FamiliarRules, FeintRules, FletchRules,
    ForageRules, GearConditions, HerbRules, HungerDef, IntimidateRules, InvestRules, MiningRules,
    PatronRules, PrayerRules, RangedRules, RequiredIds, RigRules, RuinRules, SalvageRules,
    SearchRules, SmiteRules, StatTraining, StealRules, StipendRules, SurpriseRules, SwimRules,
    TagDrops, TemperRules,
};

impl Content {
    /// The plural of `word` by the plural dictionary ([`Plurals::plural`]): "wolves", "goblins".
    pub fn plural(&self, word: &str) -> String {
        self.plurals.plural(word)
    }

    /// The ids the engine names in code (`content/required.rs`), resolved at load.
    pub fn ids(&self) -> &RequiredIds {
        &self.ids
    }

    /// The numbers of encounters (spec §7 "Leaving an outdoor area"; `data/encounters.toml`,
    /// else [`EncounterRules::default`]).
    pub fn encounters(&self) -> &EncounterRules {
        &self.encounters
    }

    /// The numbers of searching for hidden features (spec §8 "Searching"; `data/search.toml`,
    /// else [`SearchRules::default`]).
    pub fn search(&self) -> &SearchRules {
        &self.search
    }

    /// The rules of stealing that live in data (spec §8 "Stealing"; `data/steal.toml`); `None`
    /// without the file, when nobody steals.
    pub fn steal(&self) -> Option<&StealRules> {
        self.steal.as_ref()
    }

    /// The rules of surprise that live in data (spec §8 "Surprise and sneak attacks";
    /// `data/surprise.toml`); `None` without the file, when nobody is surprised.
    pub fn surprise(&self) -> Option<&SurpriseRules> {
        self.surprise.as_ref()
    }

    /// The rules of smiting that live in data (spec §8 "Class abilities": "Smite";
    /// `data/smite.toml`); `None` without the file, when nothing is unholy.
    pub fn smite(&self) -> Option<&SmiteRules> {
        self.smite.as_ref()
    }

    /// How arrows and bolts are made (spec §8 "Fletching"; `data/fletch.toml`); `None` without the
    /// file, when nobody fletches.
    pub fn fletch(&self) -> Option<&FletchRules> {
        self.fletch.as_ref()
    }

    /// The numbers of salvaging an item for its materials (`data/salvage.toml`); `None` without
    /// the file, when nobody salvages.
    pub fn salvage(&self) -> Option<&SalvageRules> {
        self.salvage.as_ref()
    }

    /// The traps the craftsperson's Trapwright rigs (`data/rigs.toml`); `None` without the file,
    /// when Trapwright is not usable yet.
    pub fn rigs(&self) -> Option<&RigRules> {
        self.rigs.as_ref()
    }

    /// What foraging in the wild gives (spec §8 "Foraging"; `data/forage.toml`); `None` without
    /// the file, when nobody forages.
    pub fn forage(&self) -> Option<&ForageRules> {
        self.forage.as_ref()
    }

    /// What the glowing fungus gives a gatherer (spec §8 "Skills": Herbalism; `data/herbs.toml`);
    /// `None` without the file, when nobody gathers.
    pub fn herbs(&self) -> Option<&HerbRules> {
        self.herbs.as_ref()
    }

    /// What a wall a miner digs through gives (spec §8 "Digging"; `data/mining.toml`); `None`
    /// without the file, when nothing is mined.
    pub fn mining(&self) -> Option<&MiningRules> {
        self.mining.as_ref()
    }

    /// The penalties of shots by distance (spec §8 "Ranged combat"; `data/ranged.toml`); none
    /// without the file.
    pub fn ranged(&self) -> &RangedRules {
        &self.ranged
    }

    /// The numbers of feinting (spec §8 "Skills": Bluff; `data/feint.toml`); `None` without the
    /// file, when nobody feints.
    pub fn feint(&self) -> Option<&FeintRules> {
        self.feint.as_ref()
    }

    /// What an aristocrat's stipend grows by (spec §8 "Class abilities": Stipend;
    /// `data/stipend.toml`); `None` without the file, when it never grows.
    pub fn stipend(&self) -> Option<&StipendRules> {
        self.stipend.as_ref()
    }

    /// What a merchant's investment pays and costs (spec §8 "Class abilities": Invest;
    /// `data/invest.toml`); `None` without the file, when no one invests.
    pub fn invest(&self) -> Option<&InvestRules> {
        self.invest.as_ref()
    }

    /// What a craftsperson's patron pays (spec §8 "Class abilities": Patron;
    /// `data/patron.toml`); `None` without the file, when no one commissions work.
    pub fn patron(&self) -> Option<&PatronRules> {
        self.patron.as_ref()
    }

    /// The numbers of intimidating (spec §8 "Skills": Intimidate; `data/intimidate.toml`); `None`
    /// without the file, when nobody intimidates.
    pub fn intimidate(&self) -> Option<&IntimidateRules> {
        self.intimidate.as_ref()
    }

    /// What area spells do to the items lying in them (spec §8 "Spells": "Ruin";
    /// `data/ruin.toml`); no rows without the file.
    pub fn ruin(&self) -> &RuinRules {
        &self.ruin
    }

    /// The numbers of swimming in deep water (spec §8 "Skills": Swim; `data/swim.toml`); `None`
    /// without the file, when nobody swims.
    pub fn swim(&self) -> Option<&SwimRules> {
        self.swim.as_ref()
    }

    /// The priest's prayer (spec §8 "Class abilities"; `data/prayer.toml`); `None` without the
    /// file, when every prayer is refused.
    pub fn prayer(&self) -> Option<&PrayerRules> {
        self.prayer.as_ref()
    }

    /// The constants of the ally core (spec §8 "Allies"; `data/allies.toml`); `None` without the
    /// file, when no ally can be made.
    pub fn allies(&self) -> Option<&AllyRules> {
        self.allies.as_ref()
    }

    /// The animal companion's data (spec §8 "Allies"; `data/companions.toml`); `None` without
    /// the file, when no companion can be made.
    pub fn companions(&self) -> Option<&CompanionRules> {
        self.companions.as_ref()
    }

    /// The witch's familiar's data (spec §8 "Allies"; `data/familiars.toml`); `None` without
    /// the file, when no familiar can be made.
    pub fn familiars(&self) -> Option<&FamiliarRules> {
        self.familiars.as_ref()
    }

    /// The rules of stat training (spec §8 "Stats"; `data/stats.toml`, else
    /// [`StatTraining::default`]: nothing trains).
    pub fn stat_training(&self) -> &StatTraining {
        &self.stat_training
    }

    /// The numbers of working an item at an anvil (spec §7 "Special rooms"; `data/temper.toml`,
    /// else [`TemperRules::default`]).
    pub fn temper(&self) -> &TemperRules {
        &self.temper
    }

    /// Material drops by creature tag (`data/drops/`, spec §7 "Material drops"), in path
    /// order; a creature type's own resolved rows are [`CreatureDef::drops`].
    pub fn tag_drops(&self) -> &[TagDrops] {
        &self.drops
    }

    /// The condition creature gear spawns in (spec §8 "Creature gear";
    /// `data/gear_condition.toml`, else none: as made).
    pub fn gear_condition(&self) -> &GearConditions {
        &self.gear_condition
    }

    /// How the player hungers, `data/hunger.toml` (spec §8 "Hunger"); `None` without the file:
    /// nobody hungers, nothing is eaten, and food does not rot.
    pub fn hunger(&self) -> Option<&HungerDef> {
        self.hunger.as_ref()
    }
}
