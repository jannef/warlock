//! Recipes (`assets/data/recipes/<skill>/<id>.toml`, `data/recipes.toml`; the design:
//! docs/design/professions.md §6, docs/design/recipes.md §10, docs/dev/crafting.md): what a
//! profession makes, as data.
//!
//! A **profession** is a skill with a directory of recipes (`recipes/metalcraft/`): the abilities
//! menu lists one row for it ([`crate::ability::Ability::Profession`]), which opens its screen.
//! Each file holds one `[[recipe]]`, its file name its id (unique across professions); they are
//! read in path order into [`Content::recipes`], a [`RecipeId`] being an index (never saved:
//! actions name recipes by id). A recipe is either **built in** (`use`: Temper and Whittle,
//! which run their own actions with their own lists and rules, and Jewelcraft's Cut and Set,
//! whose files give their `difficulty` and `seconds`) or **generic** (a `result` made from stock and inputs in `seconds` against
//! `difficulty`; parsed and checked now, made from P5). The format's second half (P4b) is parsed
//! and checked here, and used by the phases that make the recipes needing it: part sources
//! (`input`, a `stock` list, `optional` with `brittle`), `gem` parts, inputs of a material or a
//! group, `count = "fletch"`, and `failure` and `fumble`. `data/recipes.toml` holds what the
//! professions share: each one's line in the abilities menu, and the bands ([`RecipeSettings`]).
//! Recipes are known by skill value (`from`), never learned: there is no state.

use super::{Content, ContentError, Landscape, MaterialId, ObjectId, TerrainId, duplicate_id};
use crate::dice::Dice;
use crate::skills::SkillId;
use crate::stats::Stat;
use crate::text::capitalized;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Index of a recipe in [`Content::recipes`]: in path order. Not saved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecipeId(pub u16);

/// A built-in recipe: the existing action its row runs, in place of the generic craft.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecipeUse {
    /// Temper a metal weapon or armour at an anvil (`world/crafting/temper.rs`).
    Temper,
    /// Whittle a wooden weapon with a knife (`world/crafting/temper.rs`).
    Whittle,
    /// Cut or re-cut a loose gem (Jewelcraft's, docs/design/recipes.md §5; `world/crafting/cut.rs`).
    Cut,
    /// Set a cut gem in an item (Jewelcraft's, §5; `world/crafting/set.rs`).
    Set,
    /// One of Enchanting's works (`use = "enchant"` with its `enchant` kind;
    /// docs/design/enchanting.md §8; `world/crafting/enchant.rs`).
    Enchant(EnchantKind),
}

/// What an Enchanting recipe does to the item it works (docs/design/enchanting.md §8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnchantKind {
    /// A plain item made magic with one affix of its crystal's families.
    Kindle,
    /// One affix more on a magic item with room, of the kind it lacks.
    Quicken,
    /// A magic item's affixes rolled anew, as many and of the same kinds.
    Recast,
    /// A plain item made rare: three to six affixes within its capacity, one crystal's families
    /// for each of the first three.
    Imbue,
    /// A magic item made rare: one affix more and a name.
    Ennoble,
    /// Every affix off: the item plain again, nothing back.
    Unbind,
}

impl EnchantKind {
    pub const ALL: [EnchantKind; 6] = [
        EnchantKind::Kindle,
        EnchantKind::Quicken,
        EnchantKind::Recast,
        EnchantKind::Imbue,
        EnchantKind::Ennoble,
        EnchantKind::Unbind,
    ];

    /// Its id in data (`enchant = "kindle"`).
    pub const fn id(self) -> &'static str {
        match self {
            EnchantKind::Kindle => "kindle",
            EnchantKind::Quicken => "quicken",
            EnchantKind::Recast => "recast",
            EnchantKind::Imbue => "imbue",
            EnchantKind::Ennoble => "ennoble",
            EnchantKind::Unbind => "unbind",
        }
    }

    /// The row's name when the recipe gives none.
    pub const fn name(self) -> &'static str {
        match self {
            EnchantKind::Kindle => "Kindle",
            EnchantKind::Quicken => "Quicken",
            EnchantKind::Recast => "Recast",
            EnchantKind::Imbue => "Imbue",
            EnchantKind::Ennoble => "Ennoble",
            EnchantKind::Unbind => "Unbind",
        }
    }

    /// Whether it rolls affixes, and so takes a crystal at least (Unbind takes none).
    pub const fn rolls(self) -> bool {
        !matches!(self, EnchantKind::Unbind)
    }

    fn parse(id: &str) -> Result<Self, String> {
        (Self::ALL.into_iter().find(|k| k.id() == id)).ok_or_else(|| {
            let ids: Vec<&str> = Self::ALL.iter().map(|k| k.id()).collect();
            format!("unknown `enchant` `{id}` ({})", ids.join(", "))
        })
    }
}

/// Enchanting's `use` id.
const ENCHANT: &str = "enchant";

/// The most crystals an Enchanting recipe takes (docs/design/enchanting.md §8: Imbue's three).
pub const MOST_CRYSTALS: u8 = 3;

impl RecipeUse {
    /// The uses but Enchanting's, which `use = "enchant"` names with its `enchant` kind.
    pub const ALL: [RecipeUse; 4] = [
        RecipeUse::Temper,
        RecipeUse::Whittle,
        RecipeUse::Cut,
        RecipeUse::Set,
    ];

    /// Its id in data (`use = "temper"`).
    pub const fn id(self) -> &'static str {
        match self {
            RecipeUse::Temper => "temper",
            RecipeUse::Whittle => "whittle",
            RecipeUse::Cut => "cut",
            RecipeUse::Set => "set",
            RecipeUse::Enchant(_) => ENCHANT,
        }
    }

    /// The row's name when the recipe gives none.
    pub const fn name(self) -> &'static str {
        match self {
            RecipeUse::Temper => "Temper",
            RecipeUse::Whittle => "Whittle",
            RecipeUse::Cut => "Cut",
            RecipeUse::Set => "Set",
            RecipeUse::Enchant(kind) => kind.name(),
        }
    }

    /// Whether its file gives the check's `difficulty` and the work's `seconds` (Jewelcraft's
    /// Cut and Set and Enchanting's works, whose numbers live in their recipes; Temper and
    /// Whittle keep theirs in their own data files).
    pub const fn checked(self) -> bool {
        matches!(
            self,
            RecipeUse::Cut | RecipeUse::Set | RecipeUse::Enchant(_)
        )
    }

    /// The use `id` names, an `enchant` one's kind from `enchant`.
    fn parse(id: &str, enchant: Option<&str>) -> Result<Self, String> {
        if id == ENCHANT {
            let kind = enchant.ok_or("`use = \"enchant\"` needs an `enchant` kind")?;
            return EnchantKind::parse(kind).map(RecipeUse::Enchant);
        }
        if enchant.is_some() {
            return Err("`enchant` is only for `use = \"enchant\"`".into());
        }
        (Self::ALL.into_iter().find(|u| u.id() == id)).ok_or_else(|| {
            let mut ids: Vec<&str> = Self::ALL.iter().map(|u| u.id()).collect();
            ids.push(ENCHANT);
            format!("unknown `use` `{id}` ({})", ids.join(", "))
        })
    }
}

/// Where a chosen part's material comes from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartSource {
    /// A stack of stock whose material is on the part's `made_of` list and of the profession
    /// (`blade = "stock"`).
    Stock,
    /// A table of sources (`head = { input = "arrowhead", stock = ["bone", "stone"], optional =
    /// true }`): the wizard's one step for the part offers, in order, the stacks of the `input`
    /// object, then stock of the listed materials, then None if `optional`. The part takes the
    /// material of what was chosen; None leaves it off and the result is the recipe's `brittle`.
    Sources {
        /// The object input whose material the part takes (one of the recipe's inputs).
        input: Option<ObjectId>,
        /// The stock it may be of: the listed groups' and materials' that the part's `made_of`
        /// holds, in `made_of` order. Any profession's stock (a spear's wooden haft).
        stock: Vec<MaterialId>,
        /// None may be chosen.
        optional: bool,
    },
    /// A cut loose gem carried, required (`setting = { gem = true }`).
    Gem,
}

/// A part of the result the maker chooses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipePart {
    /// Its index among the result's parts.
    pub part: usize,
    pub source: PartSource,
}

/// What an input is: an object, or fixed stock nothing on the result is made of.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputKind {
    /// `object = "arrowhead"`: stacks of that object.
    Object(ObjectId),
    /// `material = "sinew"`: pieces of stock of that material.
    Material(MaterialId),
    /// `group = "metal"`: pieces of stock of any one material of the group (never a gem).
    Group(String),
}

/// What a recipe uses up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeInput {
    pub kind: InputKind,
    /// How many, per batch (or per item made, with `per_made`).
    pub count: u32,
    pub per_made: bool,
}

impl RecipeInput {
    /// The object, for an object input.
    pub fn object(&self) -> Option<ObjectId> {
        match self.kind {
            InputKind::Object(object) => Some(object),
            _ => None,
        }
    }
}

/// How many a generic recipe makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecipeCount {
    /// Dice (`count = 10`, `count = "2d6"`; 1 by default).
    Dice(Dice),
    /// Fletch's rule (`count = "fletch"`): `fletch.toml`'s count and Woodcraft's share, the
    /// margin scaling it by the counted bands; the ranger's Fletcher read with it (the arrows
    /// and bolts, `world/crafting/craft_make.rs`).
    Fletch,
}

/// What a generic recipe makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecipeResult {
    pub object: ObjectId,
    /// How many: above 1 only for a stackable object.
    pub count: RecipeCount,
}

/// One recipe (the module docs; the fields as docs/design/professions.md §6 and
/// docs/design/recipes.md §10 name them).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeDef {
    /// The file name: what `craft:ID` names.
    pub id: String,
    /// The profession: the skill whose directory it is in.
    pub skill: SkillId,
    /// The row's name: the given one, else the result's (capitalized) or the use's.
    pub name: String,
    /// What a generic recipe makes; `None` for a built-in one.
    pub result: Option<RecipeResult>,
    /// The parts the maker chooses, in the result's part order.
    pub parts: Vec<RecipePart>,
    pub inputs: Vec<RecipeInput>,
    /// The wizard offers an optional cut gem for the result's ornament part.
    pub ornament: bool,
    /// The check's base (a generic recipe's, and Cut's and Set's).
    pub difficulty: i32,
    /// The work's time, in seconds (a generic recipe's, and Cut's and Set's; another built-in
    /// one takes its action's).
    pub seconds: u32,
    /// The terrain that must stand beside the maker.
    pub station: Option<TerrainId>,
    /// A knife carried.
    pub knife: bool,
    /// A launcher ready that shoots the result (its `ammo`; `data/fletch.toml`'s `[[makes]]`).
    pub launcher: bool,
    /// The trained value the recipe is listed from (0: anyone with the skill).
    pub from: u16,
    /// The built-in action the row runs, if it is one.
    pub uses: Option<RecipeUse>,
    /// The crystals an Enchanting recipe takes (0 to [`MOST_CRYSTALS`]; 0 for any other).
    pub crystals: u8,
    /// The dialog's verb, where it is not the profession's (`stock.toml`): a ring is made, not
    /// cut ("Make").
    pub verb: Option<String>,
    /// What a failed check makes instead of nothing (stackable; the inputs are spent).
    pub failure: Option<ObjectId>,
    /// What a natural 2 makes instead of nothing (stackable).
    pub fumble: Option<ObjectId>,
    /// The result when an `optional` part is None.
    pub brittle: Option<ObjectId>,
}

/// What `data/recipes.toml` says of a profession.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfessionText {
    pub skill: SkillId,
    /// Its abilities-menu row's line ("Your recipes as a smith.").
    pub description: String,
    /// The check's stat, over the stock's profession's (Alchemy's Intelligence: herbs are no
    /// stock); without one the stock's decides (`world/crafting/craft_make.rs::profession_stat`).
    pub stat: Option<Stat>,
    /// The dialog's verb, the same way ("Brew").
    pub verb: Option<String>,
    /// What is said when a recipe with a `failure` object fails ("The brew clouds over.").
    pub failed: Option<String>,
    /// What is said when a recipe with a `fumble` object meets a natural 2.
    pub fumbled: Option<String>,
}

/// The settings of `data/recipes.toml` the recipes share (docs/design/recipes.md §4, §5, §10).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeSettings {
    /// The check's margin from which a made item is Fine.
    pub fine: i32,
    /// The margin from which it is Marvellous.
    pub marvellous: i32,
    /// The margin from which it is a Masterwork.
    pub masterwork: i32,
    /// The margins a gem's cut rises by, a cut a step (crude below one step, flawless at four).
    pub cut_step: i32,
    /// How far below the difficulty a counted recipe still makes half its count.
    pub half_within: i32,
    /// How far below the difficulty a counted recipe still makes one.
    pub one_within: i32,
    /// The percent of the stock and object inputs a failure gives back.
    pub yield_percent: u32,
    /// The landscapes where wood is free (an arrow's shaft takes no billet).
    pub free_wood: Vec<Landscape>,
}

/// Every recipe in path order, the professions' texts, and the shared settings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Recipes {
    pub list: Vec<RecipeDef>,
    pub professions: Vec<ProfessionText>,
    /// `data/recipes.toml`'s settings; `None` without the file (fixture content).
    pub settings: Option<RecipeSettings>,
}

impl Recipes {
    /// The skills with a recipe directory, in skill order: the professions.
    pub fn professions(&self) -> Vec<SkillId> {
        let mut skills: Vec<SkillId> = self.list.iter().map(|r| r.skill).collect();
        skills.sort();
        skills.dedup();
        skills
    }

    /// The recipes of profession `skill`, in path order, with their ids.
    pub fn of(&self, skill: SkillId) -> impl Iterator<Item = (RecipeId, &RecipeDef)> {
        (self.list.iter().enumerate())
            .filter(move |(_, r)| r.skill == skill)
            .map(|(i, r)| (RecipeId(i as u16), r))
    }

    /// The first recipe running built-in `uses` (the Cut list's: what `craft:` names).
    pub fn of_use(&self, uses: RecipeUse) -> Option<&RecipeDef> {
        self.list.iter().find(|r| r.uses == Some(uses))
    }

    /// The recipe with id `name`.
    pub fn id(&self, name: &str) -> Option<RecipeId> {
        let i = self.list.iter().position(|r| r.id == name)?;
        Some(RecipeId(u16::try_from(i).ok()?))
    }

    /// The abilities-menu line of profession `skill`, if `data/recipes.toml` gives one.
    pub fn description(&self, skill: SkillId) -> Option<&str> {
        (self.professions.iter())
            .find(|p| p.skill == skill)
            .map(|p| p.description.as_str())
    }

    /// What `data/recipes.toml` says of profession `skill`, if it lists it.
    pub fn profession(&self, skill: SkillId) -> Option<&ProfessionText> {
        self.professions.iter().find(|p| p.skill == skill)
    }
}

/// `data/recipes.toml`: the settings the recipes share (docs/design/recipes.md §10).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipesFile {
    #[serde(default)]
    profession: Vec<ProfessionSpec>,
    fine: i32,
    marvellous: i32,
    masterwork: i32,
    cut_step: i32,
    half_within: i32,
    one_within: i32,
    #[serde(rename = "yield")]
    yield_percent: u32,
    free_wood: Vec<Landscape>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfessionSpec {
    skill: String,
    description: String,
    #[serde(default)]
    stat: Option<Stat>,
    #[serde(default)]
    verb: Option<String>,
    #[serde(default)]
    failed: Option<String>,
    #[serde(default)]
    fumbled: Option<String>,
}

/// One file of `data/recipes/<skill>/`: one recipe (none in a file of comments only).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipeFile {
    #[serde(default)]
    recipe: Vec<RecipeSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipeSpec {
    id: String,
    name: Option<String>,
    result: Option<ResultSpec>,
    #[serde(default)]
    parts: BTreeMap<String, PartSpec>,
    #[serde(default)]
    inputs: Vec<InputSpec>,
    #[serde(default)]
    ornament: bool,
    difficulty: Option<i32>,
    seconds: Option<u32>,
    station: Option<String>,
    #[serde(default)]
    knife: bool,
    #[serde(default)]
    launcher: bool,
    #[serde(default)]
    from: u16,
    #[serde(rename = "use")]
    uses: Option<String>,
    enchant: Option<String>,
    crystals: Option<u8>,
    verb: Option<String>,
    failure: Option<ObjectSpec>,
    fumble: Option<ObjectSpec>,
    brittle: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultSpec {
    object: String,
    #[serde(default = "one")]
    count: CountSpec,
}

/// `count`: dice, or the word `fletch`.
#[derive(Deserialize)]
#[serde(untagged)]
enum CountSpec {
    Dice(Dice),
    Word(String),
}

fn one() -> CountSpec {
    CountSpec::Dice(Dice::fixed(1))
}

/// `failure = { object = "murky_draught" }`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectSpec {
    object: String,
}

/// A chosen part's source: `"stock"`, or a table (`{ input = "arrowhead", stock = ["bone"],
/// optional = true }`, `{ gem = true }`).
#[derive(Deserialize)]
#[serde(untagged)]
enum PartSpec {
    Word(String),
    Table(SourcesSpec),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourcesSpec {
    input: Option<String>,
    stock: Option<Vec<String>>,
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    gem: bool,
}

/// An input: exactly one of `object`, `material`, and `group`, and a count.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputSpec {
    object: Option<String>,
    material: Option<String>,
    group: Option<String>,
    count: u32,
    #[serde(default)]
    per_made: bool,
}

/// The settings file's path.
pub(super) const SETTINGS: &str = "data/recipes.toml";

/// Parses the settings file `settings` and every recipe file of `files` (path order; the settings
/// file among them is skipped), checking each as docs/design/professions.md §6 and
/// docs/design/recipes.md §10 list (docs/dev/crafting.md has the table): a file in a skill's
/// directory, one recipe a file, its id its file name and unique, then [`resolve`]'s checks. The
/// settings file: known skills, each once, non-empty descriptions, each band above the last.
pub(super) fn parse<'a>(
    settings: Option<(&str, &str)>,
    files: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Recipes, ContentError> {
    let mut recipes = Recipes::default();
    // Content without skills (a fixture) has no professions.
    if content.skills().is_empty() {
        return Ok(recipes);
    }
    if let Some((path, text)) = settings {
        let err = |m: String| ContentError::new(path, m);
        let file: RecipesFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for row in &file.profession {
            let skill = (content.skill_id(&row.skill))
                .ok_or_else(|| err(format!("profession: unknown skill `{}`", row.skill)))?;
            if recipes.description(skill).is_some() {
                return Err(err(format!("profession `{}` is listed twice", row.skill)));
            }
            if row.description.is_empty() {
                return Err(err(format!(
                    "profession `{}`: empty description",
                    row.skill
                )));
            }
            let empty = |line: &Option<String>| line.as_ref().is_some_and(String::is_empty);
            if empty(&row.verb) || empty(&row.failed) || empty(&row.fumbled) {
                return Err(err(format!("profession `{}`: an empty line", row.skill)));
            }
            recipes.professions.push(ProfessionText {
                skill,
                description: row.description.clone(),
                stat: row.stat,
                verb: row.verb.clone(),
                failed: row.failed.clone(),
                fumbled: row.fumbled.clone(),
            });
        }
        recipes.settings = Some(settings_of(file).map_err(err)?);
    }
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    for (path, text) in files {
        if path == SETTINGS {
            continue;
        }
        let err = |m: String| ContentError::new(path, m);
        let file: RecipeFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if file.recipe.len() > 1 {
            return Err(err("one `[[recipe]]` a file".into()));
        }
        let Some(spec) = file.recipe.into_iter().next() else {
            continue;
        };
        let (dir, stem) = place(path).ok_or_else(|| {
            err("a recipe goes in its skill's directory: data/recipes/<skill>/<id>.toml".into())
        })?;
        if spec.id != stem {
            return Err(err(format!(
                "recipe `{}`: its id is its file name",
                spec.id
            )));
        }
        if let Some(first) = seen.get(&spec.id) {
            return Err(duplicate_id("recipe", &spec.id, path, first));
        }
        seen.insert(spec.id.clone(), path.to_string());
        let skill = (content.skill_id(dir))
            .ok_or_else(|| err(format!("recipe `{stem}`: no skill `{dir}` (its directory)")))?;
        let recipe =
            resolve(spec, skill, content).map_err(|m| err(format!("recipe `{stem}`: {m}")))?;
        if let Some(RecipeUse::Enchant(kind)) = recipe.uses
            && recipes.list.iter().any(|r| r.uses == recipe.uses)
        {
            return Err(err(format!("recipe `{stem}`: a second `{}`", kind.id())));
        }
        recipes.list.push(recipe);
    }
    Ok(recipes)
}

/// The settings of the file, checked: each band above the last (Fine above 0, Marvellous above
/// Fine, Masterwork above Marvellous; a cut's step above 0; `half_within` 0 or more and
/// `one_within` above it), a yield of at most 100, and each free-wood landscape once.
fn settings_of(file: RecipesFile) -> Result<RecipeSettings, String> {
    // Each band and the one below it (`None`: a floor, the value it must be above).
    let bands = [
        ("fine", file.fine, None, 0),
        ("marvellous", file.marvellous, Some("fine"), file.fine),
        (
            "masterwork",
            file.masterwork,
            Some("marvellous"),
            file.marvellous,
        ),
        ("cut_step", file.cut_step, None, 0),
        ("half_within", file.half_within, None, -1),
        (
            "one_within",
            file.one_within,
            Some("half_within"),
            file.half_within,
        ),
    ];
    for (name, value, below, floor) in bands {
        if value <= floor {
            return Err(match below {
                Some(below) => format!("`{name}` ({value}) must be above `{below}` ({floor})"),
                None => format!("`{name}` ({value}) must be above {floor}"),
            });
        }
    }
    if file.yield_percent > 100 {
        return Err(format!("`yield` {} is above 100", file.yield_percent));
    }
    let mut free_wood = file.free_wood.clone();
    free_wood.sort();
    free_wood.dedup();
    if free_wood.len() != file.free_wood.len() {
        return Err("`free_wood` lists a landscape twice".into());
    }
    Ok(RecipeSettings {
        fine: file.fine,
        marvellous: file.marvellous,
        masterwork: file.masterwork,
        cut_step: file.cut_step,
        half_within: file.half_within,
        one_within: file.one_within,
        yield_percent: file.yield_percent,
        free_wood: file.free_wood,
    })
}

/// The skill directory and the file stem of a recipe file's `path`
/// (`data/recipes/metalcraft/temper.toml` → `metalcraft`, `temper`); `None` for a file not
/// directly in a directory of `recipes/`.
fn place(path: &str) -> Option<(&str, &str)> {
    let mut segments = path.rsplit('/');
    let stem = segments.next()?.strip_suffix(".toml")?;
    let dir = segments.next()?;
    (segments.next()? == "recipes").then_some((dir, stem))
}

/// One recipe, checked: a known `use` with no result, parts, inputs, ornament, failure, fumble,
/// or brittle, and a `difficulty` and `seconds` of 1 or more exactly when it is
/// [`RecipeUse::checked`] (Cut and Set); a generic recipe with a result object, a count that
/// makes at least one and above 1 (or `fletch`) only on a stackable object, a `difficulty`, and
/// `seconds` of 1 or more; inputs ([`input`]); parts ([`part_source`]); an ornament only on a
/// result with an ornament part; `failure` and `fumble` stackable objects; `brittle` an object,
/// only with an `optional` part (and an `optional` part only with it); a station that is terrain.
fn resolve(spec: RecipeSpec, skill: SkillId, content: &Content) -> Result<RecipeDef, String> {
    let station = match &spec.station {
        Some(id) => Some(
            content
                .terrain_id(id)
                .ok_or(format!("unknown station `{id}`"))?,
        ),
        None => None,
    };
    let common = |name: String| RecipeDef {
        id: spec.id.clone(),
        skill,
        name,
        result: None,
        parts: Vec::new(),
        inputs: Vec::new(),
        ornament: false,
        difficulty: 0,
        seconds: 0,
        station,
        knife: spec.knife,
        launcher: spec.launcher,
        from: spec.from,
        uses: None,
        crystals: 0,
        verb: spec.verb.clone(),
        failure: None,
        fumble: None,
        brittle: None,
    };
    if let Some(uses) = &spec.uses {
        let uses = RecipeUse::parse(uses, spec.enchant.as_deref())?;
        let generic = spec.result.is_some()
            || !spec.parts.is_empty()
            || !spec.inputs.is_empty()
            || spec.ornament
            || (!uses.checked() && (spec.difficulty.is_some() || spec.seconds.is_some()))
            || spec.failure.is_some()
            || spec.fumble.is_some()
            || spec.brittle.is_some();
        if generic {
            return Err(format!(
                "a `use` recipe runs its action: no result, parts, inputs, ornament, \
                 difficulty, seconds, failure, fumble, or brittle (`use = \"{}\"`)",
                uses.id()
            ));
        }
        let (difficulty, seconds) = match uses.checked() {
            true => (
                spec.difficulty.ok_or("needs a `difficulty`")?,
                (spec.seconds.filter(|&s| s > 0)).ok_or("needs `seconds` of 1 or more")?,
            ),
            false => (0, 0),
        };
        let crystals = match (uses, spec.crystals) {
            (RecipeUse::Enchant(kind), n) => {
                let n = n.unwrap_or(0);
                if n > MOST_CRYSTALS {
                    return Err(format!("`crystals` {n} is above {MOST_CRYSTALS}"));
                }
                if kind.rolls() != (n > 0) {
                    return Err(match kind.rolls() {
                        true => format!("`{}` takes a crystal at least", kind.id()),
                        false => format!("`{}` takes no crystal", kind.id()),
                    });
                }
                n
            }
            (_, Some(_)) => return Err("`crystals` is only for `use = \"enchant\"`".into()),
            (_, None) => 0,
        };
        let name = spec.name.clone().unwrap_or_else(|| uses.name().into());
        return Ok(RecipeDef {
            uses: Some(uses),
            difficulty,
            seconds,
            crystals,
            ..common(name)
        });
    }
    if spec.enchant.is_some() || spec.crystals.is_some() {
        return Err("`enchant` and `crystals` are only for `use = \"enchant\"`".into());
    }
    let result = spec.result.as_ref().ok_or("needs a `result` or a `use`")?;
    let object = (content.object_id(&result.object))
        .ok_or_else(|| format!("unknown result object `{}`", result.object))?;
    let def = content.object(object);
    let count = match &result.count {
        CountSpec::Dice(dice) if dice.min() < 1 => {
            return Err(format!("result count `{dice}` may make none"));
        }
        CountSpec::Dice(dice) => RecipeCount::Dice(*dice),
        CountSpec::Word(word) if word == "fletch" => RecipeCount::Fletch,
        CountSpec::Word(word) => {
            return Err(format!(
                "result count `{word}` is neither dice nor `\"fletch\"`"
            ));
        }
    };
    let many = match count {
        RecipeCount::Dice(dice) => dice.max() > 1,
        RecipeCount::Fletch => true,
    };
    if many && !def.stackable {
        return Err(format!("`{}` does not stack: count 1", result.object));
    }
    let difficulty = spec.difficulty.ok_or("needs a `difficulty`")?;
    let seconds = spec
        .seconds
        .filter(|&s| s >= 1)
        .ok_or("needs `seconds` of 1 or more")?;
    let inputs = (spec.inputs.iter())
        .map(|i| input(content, i))
        .collect::<Result<Vec<_>, _>>()?;
    let mut parts = Vec::new();
    for (name, source) in &spec.parts {
        let part = part_index(content, object, name)
            .ok_or_else(|| format!("`{}` has no part `{name}`", result.object))?;
        let source = part_source(content, skill, object, part, source, &inputs)
            .map_err(|m| format!("part `{name}`: {m}"))?;
        parts.push(RecipePart { part, source });
    }
    parts.sort_by_key(|p| p.part);
    if spec.ornament {
        let placed = (content.gems()).and_then(|g| g.ornament(def.class));
        if def.ornament.is_none() && placed.is_none() {
            return Err(format!("`{}` has no ornament part", result.object));
        }
    }
    let stackable = |field: &str, spec: &Option<ObjectSpec>| -> Result<Option<ObjectId>, String> {
        let Some(ObjectSpec { object }) = spec else {
            return Ok(None);
        };
        let id = (content.object_id(object))
            .ok_or_else(|| format!("unknown {field} object `{object}`"))?;
        if !content.object(id).stackable {
            return Err(format!("{field} object `{object}` does not stack"));
        }
        Ok(Some(id))
    };
    let failure = stackable("failure", &spec.failure)?;
    let fumble = stackable("fumble", &spec.fumble)?;
    let optional =
        (parts.iter()).any(|p| matches!(p.source, PartSource::Sources { optional: true, .. }));
    let brittle = match &spec.brittle {
        Some(id) => {
            let id = content
                .object_id(id)
                .ok_or_else(|| format!("unknown brittle object `{id}`"))?;
            if !optional {
                return Err("`brittle` is only for a recipe with an `optional` part".into());
            }
            Some(id)
        }
        None if optional => {
            return Err("an `optional` part needs the recipe's `brittle` object".into());
        }
        None => None,
    };
    let name = spec.name.clone().unwrap_or_else(|| capitalized(&def.name));
    Ok(RecipeDef {
        result: Some(RecipeResult { object, count }),
        parts,
        inputs,
        ornament: spec.ornament,
        difficulty,
        seconds,
        failure,
        fumble,
        brittle,
        ..common(name)
    })
}

/// An input, checked: exactly one of `object`, `material`, and `group`; an object or material
/// that exists, a group some material has (a gem names no group); a count above 0.
fn input(content: &Content, spec: &InputSpec) -> Result<RecipeInput, String> {
    let kind = match (&spec.object, &spec.material, &spec.group) {
        (Some(object), None, None) => InputKind::Object(
            (content.object_id(object))
                .ok_or_else(|| format!("unknown input object `{object}`"))?,
        ),
        (None, Some(material), None) => InputKind::Material(
            (content.material_id(material))
                .ok_or_else(|| format!("unknown input material `{material}`"))?,
        ),
        (None, None, Some(group)) => {
            if (content.material_id(group)).is_some_and(|m| content.material(m).gem.is_some()) {
                return Err(format!(
                    "input group `{group}`: a gem is no group (gems are chosen as parts)"
                ));
            }
            if !group_exists(content, group) {
                return Err(format!("unknown input group `{group}`"));
            }
            InputKind::Group(group.clone())
        }
        _ => return Err("an input names one of `object`, `material`, and `group`".into()),
    };
    let what = match &kind {
        InputKind::Object(o) => &content.object(*o).id,
        InputKind::Material(m) => &content.material(*m).id,
        InputKind::Group(g) => g,
    };
    if spec.count == 0 {
        return Err(format!("input `{what}`: count 0"));
    }
    Ok(RecipeInput {
        kind,
        count: spec.count,
        per_made: spec.per_made,
    })
}

/// Whether some material is of group `group`.
fn group_exists(content: &Content, group: &str) -> bool {
    (content.materials().iter()).any(|m| m.group.as_deref() == Some(group))
}

/// Part `part`'s source, checked. `"stock"`: some material of the part's `made_of` is the
/// profession's stock. A table: `gem` alone, the part able to be of a gem; else an `input` or a
/// `stock` list or both: the input one of the recipe's object inputs whose own material the
/// part's `made_of` admits, each listed name a group some material has or a material (a listed
/// material on `made_of`, a listed group with one there: a recipe narrows, never widens).
fn part_source(
    content: &Content,
    skill: SkillId,
    object: ObjectId,
    part: usize,
    spec: &PartSpec,
    inputs: &[RecipeInput],
) -> Result<PartSource, String> {
    let made_of = content.object(object).made_of(part);
    let table = match spec {
        PartSpec::Word(word) if word == "stock" => {
            let of_skill = made_of.iter().any(|&m| {
                (content.stock().profession_of(content, m)).is_some_and(|p| p.skill == skill)
            });
            if !of_skill {
                return Err("none of what it may be made of is the profession's stock".into());
            }
            return Ok(PartSource::Stock);
        }
        PartSpec::Word(word) => {
            return Err(format!(
                "`{word}` is no source (`\"stock\"` or a table: `input`, `stock`, `gem`)"
            ));
        }
        PartSpec::Table(table) => table,
    };
    if table.gem {
        if table.input.is_some() || table.stock.is_some() || table.optional {
            return Err("a `gem` part has no other source and is never optional".into());
        }
        if !made_of.iter().any(|&m| content.material(m).gem.is_some()) {
            return Err("`gem`, but it may be made of no gem".into());
        }
        return Ok(PartSource::Gem);
    }
    if table.input.is_none() && table.stock.is_none() {
        return Err("names no source (`input`, `stock`, or `gem`)".into());
    }
    let input = match &table.input {
        Some(name) => {
            let id = (content.object_id(name))
                .filter(|id| inputs.iter().any(|i| i.object() == Some(*id)))
                .ok_or_else(|| format!("`{name}` is none of the inputs"))?;
            if !made_of.contains(&content.object(id).material()) {
                return Err(format!(
                    "input `{name}`: its material may not make the part"
                ));
            }
            Some(id)
        }
        None => None,
    };
    let listed = table.stock.as_deref().unwrap_or_default();
    if table.stock.is_some() && listed.is_empty() {
        return Err("an empty `stock` list".into());
    }
    // A name that is a group and a material too (`stone`) is the group.
    for name in listed {
        if group_exists(content, name) {
            if !(made_of.iter()).any(|&m| content.material(m).group.as_deref() == Some(name)) {
                return Err(format!(
                    "stock `{name}`: none of the group is what it may be made of"
                ));
            }
        } else if let Some(m) = content.material_id(name) {
            if !made_of.contains(&m) {
                return Err(format!("stock `{name}` is not what it may be made of"));
            }
        } else {
            return Err(format!("unknown stock group or material `{name}`"));
        }
    }
    let stock = (made_of.iter().copied())
        .filter(|&m| {
            let def = content.material(m);
            (listed.iter()).any(|n| *n == def.id || def.group.as_deref() == Some(n.as_str()))
        })
        .collect();
    Ok(PartSource::Sources {
        input,
        stock,
        optional: table.optional,
    })
}

/// The index of part `name` among `object`'s parts: `main` is the first.
fn part_index(content: &Content, object: ObjectId, name: &str) -> Option<usize> {
    let parts = &content.object(object).parts;
    if name == "main" {
        return (!parts.is_empty()).then_some(0);
    }
    let id = content.part_id(name)?;
    parts.iter().position(|p| p.part == Some(id))
}
