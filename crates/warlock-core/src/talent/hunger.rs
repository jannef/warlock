//! The hunger talents (#1495): what a talent does to the holder's hunger and meals: nutrition,
//! rotten food, and corpses. [`HungerEffect`] is the data (a talent's `[[effect]]` of one of
//! [`HungerEffect::KINDS`]), [`HungerMods`] the sum every rule reads (`TalentMods::hunger`). A new
//! effect of the area is a variant here, its field and its arm in [`HungerMods::add`] here, its
//! wiki text in `xtask/src/wiki/talents/hunger.rs`, and its one read.

use crate::content::Content;
use crate::dice::Dice;
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HungerEffect {
    /// Hunger burns this percent faster (negative: slower).
    HungerPercent { value: i32 },
    /// The sickness rotten food gives, this percent more (negative: less), rounded down.
    RotHarmPercent { value: i32 },
    /// It smells how fresh food is: the eat list says fresh, turning, or rotten (Hound's nose).
    SmellRot,
    /// Exerting oneself (a blow, a shot, a throw) burns no extra nutrition.
    NoExertion,
    /// A meal never leaves it Bloated: what does not fit below Bloated is lost (Gourmand).
    NoBloating,
    /// Hunger never weakens it: Weak, Fainting, and Starving take nothing off its Strength and
    /// Dexterity (`Actor::penalty`), and it never faints (`World::faint`; Long fast). Starving
    /// still costs hit points.
    NoHungerWeakness,
    /// A fresh corpse's stat training marks and chances to widen a potential, this percent more
    /// (Gourmand).
    CorpseStatsPercent { value: i32 },
    /// Rotten food never makes it vomit and sickens it half as much; overeating never makes it
    /// vomit (Iron stomach).
    IronStomach,
    /// Rotten food never harms it: no vomiting, no sickness, no disease, as a race that eats
    /// rotten food (Deep roots).
    EatsRotten,
    /// Rotten food neither sickens it nor exposes it to a disease (Purify).
    Purify,
    /// A corpse's own harm (`on_eat`: sickness, vomiting) never touches it (Herb lore,
    /// Cauldron).
    CorpseHarmless,
    /// Eating `object` (an id in `data/objects/`; none: a fresh corpse) also heals `dice` hit
    /// points (Herb lore's angelica root, Cauldron's corpses).
    MealHeals {
        #[serde(default)]
        object: Option<String>,
        dice: Dice,
    },
}

impl HungerEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`HungerEffect`]).
    pub const KINDS: [&'static str; 12] = [
        "corpse_harmless",
        "corpse_stats_percent",
        "eats_rotten",
        "hunger_percent",
        "iron_stomach",
        "meal_heals",
        "no_bloating",
        "no_exertion",
        "no_hunger_weakness",
        "purify",
        "rot_harm_percent",
        "smell_rot",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HungerMods {
    /// Percent faster hunger burns (negative: slower).
    pub hunger_percent: i32,
    /// Percent more sickness from rotten food (negative: less).
    pub rot_harm_percent: i32,
    /// It smells how fresh food is.
    pub smell_rot: bool,
    /// Exertion burns nothing.
    pub no_exertion: bool,
    /// A meal never leaves it Bloated.
    pub no_bloating: bool,
    /// Hunger never weakens it: Weak or worse takes nothing off its stats, and it never faints
    /// (Long fast).
    pub no_hunger_weakness: bool,
    /// Percent more of a fresh corpse's stat training marks and its chances to widen a
    /// potential (`on_eat` `train` and `widen`).
    pub corpse_stats_percent: i32,
    /// Rotten food and overeating never make it vomit, and rotten food sickens it half as much.
    pub iron_stomach: bool,
    /// Rotten food neither sickens it nor exposes it to a disease.
    pub purify: bool,
    /// A corpse's own harm never touches it.
    pub corpse_harmless: bool,
    /// Hit points a meal heals besides: `(object, dice)`, `None` for a fresh corpse.
    pub meal_heals: Vec<(Option<crate::content::ObjectId>, Dice)>,
    /// Rotten food never harms it (Deep roots).
    pub eats_rotten: bool,
}

impl HungerMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: HungerEffect) {
        match effect {
            HungerEffect::HungerPercent { value } => self.hunger_percent += value,
            HungerEffect::RotHarmPercent { value } => self.rot_harm_percent += value,
            HungerEffect::SmellRot => self.smell_rot = true,
            HungerEffect::NoExertion => self.no_exertion = true,
            HungerEffect::NoBloating => self.no_bloating = true,
            HungerEffect::NoHungerWeakness => self.no_hunger_weakness = true,
            HungerEffect::CorpseStatsPercent { value } => self.corpse_stats_percent += value,
            HungerEffect::IronStomach => self.iron_stomach = true,
            HungerEffect::Purify => self.purify = true,
            HungerEffect::CorpseHarmless => self.corpse_harmless = true,
            HungerEffect::MealHeals { object, dice } => match object {
                None => self.meal_heals.push((None, dice)),
                Some(name) => {
                    if let Some(id) = content.object_id(&name) {
                        self.meal_heals.push((Some(id), dice));
                    }
                }
            },
            HungerEffect::EatsRotten => self.eats_rotten = true,
        }
    }
}
