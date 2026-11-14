//! The craft talents (#1495): what a talent does to the holder's crafts: the trades, salvage,
//! tempering, craft time and grades, fletching, and the craftsperson's actives. [`CraftEffect`] is
//! the data (a talent's `[[effect]]` of one of [`CraftEffect::KINDS`]), [`CraftMods`] the sum every
//! rule reads (`TalentMods::craft`). A new effect of the area is a variant here, its field and its
//! arm in [`CraftMods::add`] here, its wiki text in `xtask/src/wiki/talents/craft.rs`, and its one
//! read.

use crate::content::Content;
use crate::dice::Dice;
use crate::skills::SkillId;
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CraftEffect {
    /// A batch of arrows or bolts (Fletcher, the ranger's level 1; spec §8 "Fletching"): `check`
    /// more on its check, `count_times` the count after the margin, `time_percent` of its time,
    /// and plain at any Woodcraft.
    Fletcher {
        check: i32,
        count_times: i32,
        time_percent: u32,
    },
    /// Handed out once, as it is taken (a Crafts trade: Smith, Wright, ...; `world/crafting/trade.rs`):
    /// `skill` trained at `dice` plus the holder's `trade_bonus`, rolled from the stream
    /// `character/trade/<race>,<gender>,<class>/<skill>`, and the tools of the trade, kit lines
    /// (`content/kit.rs`) made from the same stream and carried, worn, or wielded as they say.
    /// The skill is then a trade of the holder's: a level-up roll on it throws the die one step
    /// up, as a class's known-for skill does (`CraftMods::trades`, read in `skill_roll`).
    Trade {
        skill: String,
        dice: Dice,
        #[serde(default)]
        tools: Vec<crate::content::KitLineSpec>,
    },
    /// Added to the dice of every trade it takes, as the trade is handed down (Time served).
    TradeBonus { value: i32 },
    /// Item descriptions read an item's make: what salvage would give of it, the improvements
    /// left in it, how hard its stuff is to work (Eye for the make;
    /// [`crate::magic::describe::make_lines`]).
    EyeForMake,
    /// Nothing is wasted in its hands (Waste not): salvage gives back `salvage` % of each made
    /// material's grams, and a failed recipe `failed` % of its stock and object inputs, each
    /// where it is more than everyone's (`data/rules/salvage.toml`'s, `data/recipes.toml`'s
    /// `yield`; `world/crafting/salvage.rs::salvage_percent`, `world/crafting/craft_make.rs::failed_percent`).
    WasteNot { salvage: u8, failed: u32 },
    /// A failed Temper or Whittle damages nothing: the item is as it was, the attempt counted
    /// (Waste not; `world/crafting/temper.rs`).
    TemperSafe,
    /// Every craft it works takes `value` % of its time: recipes, Temper and Whittle, Cut and
    /// Set, Salvage, and the craftsperson's actives (Quick hands; the least held;
    /// `world/crafting/mastery.rs::craft_seconds`).
    CraftTimePercent { value: u32 },
    /// Added to a made check's margin as its grade or cut is read, never to whether it is made
    /// (Master's hand; summed; `world/crafting/mastery.rs::graded_margin`).
    GradeEase { value: i32 },
    /// Its graded work bears its mark (Maker's mark): an item made with a `kind` is built marked
    /// (`world/crafting/craft_make.rs::make_out`, the one read), and a marked item is worth
    /// `worth_percent` % more ([`TalentBook::mark_percent`], `Worked::value`), the patron pays
    /// more for it, and it holds one enchantment more (`magic/capacity.rs`).
    MakersMark { worth_percent: u32 },
    /// The class active Field forge: an ingot carried set on a heap of stones beside the player
    /// in `seconds`, a makeshift anvil for good (`world/crafting/field_forge.rs`; the craftsperson's).
    FieldForge { seconds: u32 },
    /// The class active Make do and mend: a damaged thing the player carries mended in the field,
    /// with no anvil, in `seconds`, by a check against `difficulty` plus its main material's
    /// `work` that destroys nothing (`world/crafting/make_do.rs`; the craftsperson design §7.5).
    MakeDo { seconds: u32, difficulty: i32 },
    /// The class active Trapwright: a trap rigged on a cell beside the player from what the rig
    /// list (`data/rigs.toml`) asks (`world/crafting/rig.rs`; spec §8 "Traps": "Rigging traps").
    Rigs,
}

impl CraftEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`CraftEffect`]).
    pub const KINDS: [&'static str; 12] = [
        "craft_time_percent",
        "eye_for_make",
        "field_forge",
        "fletcher",
        "grade_ease",
        "make_do",
        "makers_mark",
        "rigs",
        "temper_safe",
        "trade",
        "trade_bonus",
        "waste_not",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CraftMods {
    /// Its trades (the Crafts talents): a level-up roll on one throws the die one step up, as a
    /// class's known-for skill does (`world/character/skill_roll.rs`).
    pub trades: Vec<SkillId>,
    /// Added to a trade's dice as the trade is handed down (Time served; `world/crafting/trade.rs`).
    pub trade_bonus: i32,
    /// Item descriptions read an item's make (Eye for the make; `PerceptionView::describe_item`).
    pub eye_for_make: bool,
    /// Salvage's yield in percent, where more than `data/rules/salvage.toml`'s (Waste not; the most
    /// held; `world/crafting/salvage.rs::salvage_percent`).
    pub salvage_percent: u8,
    /// What a failed recipe gives back in percent, where more than `data/recipes.toml`'s `yield`
    /// (Waste not; the most held; `world/crafting/craft_make.rs::failed_percent`).
    pub failed_percent: u32,
    /// A failed Temper or Whittle damages nothing (Waste not; `world/crafting/temper.rs::work_out`).
    pub temper_safe: bool,
    /// The percent of its time every craft takes, where held (Quick hands; the least held;
    /// `world/crafting/mastery.rs::craft_seconds`).
    pub craft_time_percent: Option<u32>,
    /// Added to a made check's margin as it is graded, never to the check (Master's hand;
    /// `world/crafting/mastery.rs::graded_margin`).
    pub grade_ease: i32,
    /// Its graded work is built bearing its mark (Maker's mark; `world/crafting/craft_make.rs::make_out`).
    pub makers_mark: bool,
    /// It fletches far better (Fletcher, the ranger's; `world/crafting/fletch.rs`).
    pub fletcher: Option<Fletcher>,
    /// The active Field forge: the seconds it takes to set a makeshift anvil, if it has it.
    pub field_forge: Option<u32>,
    /// The active Make do and mend, if it has it (`world/crafting/make_do.rs`).
    pub make_do: Option<MakeDo>,
    /// The active Trapwright: traps rigged on a cell beside it (`world/crafting/rig.rs`).
    pub rigs: bool,
}

impl CraftMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: CraftEffect) {
        match effect {
            CraftEffect::Fletcher {
                check,
                count_times,
                time_percent,
            } => {
                self.fletcher = Some(Fletcher {
                    check,
                    count_times,
                    time_percent,
                });
            }
            // Its skill and tools are handed out as it is taken (`world/crafting/trade.rs`); the skill
            // stays a trade of the holder's.
            CraftEffect::Trade { skill, .. } => self.trades.extend(content.skill_id(&skill)),
            CraftEffect::TradeBonus { value } => self.trade_bonus += value,
            CraftEffect::EyeForMake => self.eye_for_make = true,
            CraftEffect::WasteNot { salvage, failed } => {
                self.salvage_percent = self.salvage_percent.max(salvage);
                self.failed_percent = self.failed_percent.max(failed);
            }
            CraftEffect::TemperSafe => self.temper_safe = true,
            CraftEffect::CraftTimePercent { value } => {
                let least = self.craft_time_percent.map_or(value, |had| had.min(value));
                self.craft_time_percent = Some(least);
            }
            CraftEffect::GradeEase { value } => self.grade_ease += value,
            CraftEffect::MakersMark { .. } => self.makers_mark = true,
            CraftEffect::FieldForge { seconds } => self.field_forge = Some(seconds),
            CraftEffect::MakeDo {
                seconds,
                difficulty,
            } => {
                self.make_do = Some(MakeDo {
                    seconds,
                    difficulty,
                })
            }
            CraftEffect::Rigs => self.rigs = true,
        }
    }
}

/// The active Make do and mend (spec §8 "Class abilities"; `world/crafting/make_do.rs`): how long a mend
/// takes, in seconds, and its check's difficulty before the material's `work`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MakeDo {
    pub seconds: u32,
    pub difficulty: i32,
}

/// What Fletcher does to a batch (spec §8 "Fletching"): plain arrows at any Woodcraft, and these
/// numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fletcher {
    /// Added to the batch's check.
    pub check: i32,
    /// What the count is multiplied by, after the check's margin.
    pub count_times: i32,
    /// Percent of the batch's time it takes.
    pub time_percent: u32,
}
