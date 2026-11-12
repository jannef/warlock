//! Recipes (`assets/data/recipes/<skill>/<id>.toml`, `data/recipes.toml`; the design:
//! docs/design/professions.md §6, docs/dev/crafting.md): what a profession makes, as data.
//!
//! A **profession** is a skill with a directory of recipes (`recipes/metalcraft/`): the abilities
//! menu lists one row for it ([`crate::ability::Ability::Profession`]), which opens its screen.
//! Each file holds one `[[recipe]]`, its file name its id (unique across professions); they are
//! read in path order into [`Content::recipes`], a [`RecipeId`] being an index (never saved:
//! actions name recipes by id). A recipe is either **built in** (`use`: Temper, Whittle, Fletch,
//! which run their own actions with their own lists and rules) or **generic** (a `result` made
//! from stock and inputs in `seconds` against `difficulty`; parsed and checked now, made from P5).
//! `data/recipes.toml` holds what the professions share: each one's line in the abilities menu.
//! Recipes are known by skill value (`from`), never learned: there is no state.

use super::{Content, ContentError, ObjectId, TerrainId, duplicate_id};
use crate::dice::Dice;
use crate::skills::SkillId;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Index of a recipe in [`Content::recipes`]: in path order. Not saved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecipeId(pub u16);

/// A built-in recipe: the existing action its row runs, in place of the generic craft.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecipeUse {
    /// Temper a metal weapon or armour at an anvil (`world/temper.rs`).
    Temper,
    /// Whittle a wooden weapon with a knife (`world/temper.rs`).
    Whittle,
    /// Make arrows or bolts for the launcher ready (`world/fletch.rs`).
    Fletch,
}

impl RecipeUse {
    pub const ALL: [RecipeUse; 3] = [RecipeUse::Temper, RecipeUse::Whittle, RecipeUse::Fletch];

    /// Its id in data (`use = "temper"`).
    pub const fn id(self) -> &'static str {
        match self {
            RecipeUse::Temper => "temper",
            RecipeUse::Whittle => "whittle",
            RecipeUse::Fletch => "fletch",
        }
    }

    /// The row's name when the recipe gives none.
    pub const fn name(self) -> &'static str {
        match self {
            RecipeUse::Temper => "Temper",
            RecipeUse::Whittle => "Whittle",
            RecipeUse::Fletch => "Fletch",
        }
    }

    fn parse(id: &str) -> Result<Self, String> {
        (Self::ALL.into_iter().find(|u| u.id() == id)).ok_or_else(|| {
            let ids: Vec<&str> = Self::ALL.iter().map(|u| u.id()).collect();
            format!("unknown `use` `{id}` ({})", ids.join(", "))
        })
    }
}

/// Where a chosen part's material comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartSource {
    /// A stack of stock whose material is on the part's `made_of` list and of the profession
    /// (`blade = "stock"`).
    Stock,
    /// The material of the input of this object (`head = { input = "arrowhead" }`).
    Input(ObjectId),
}

/// A part of the result the maker chooses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecipePart {
    /// Its index among the result's parts.
    pub part: usize,
    pub source: PartSource,
}

/// Fixed objects a recipe uses up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecipeInput {
    pub object: ObjectId,
    /// How many, per batch (or per item made, with `per_made`).
    pub count: u32,
    pub per_made: bool,
}

/// What a generic recipe makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecipeResult {
    pub object: ObjectId,
    /// How many: dice above 1 only for a stackable object.
    pub count: Dice,
}

/// One recipe (the module docs; the fields as docs/design/professions.md §6 names them).
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
    /// The check's base (a generic recipe's).
    pub difficulty: i32,
    /// The work's time, in seconds (a generic recipe's; a built-in one takes its action's).
    pub seconds: u32,
    /// The terrain that must stand beside the maker.
    pub station: Option<TerrainId>,
    /// A knife carried.
    pub knife: bool,
    /// A bow or crossbow ready that fletching makes ammunition for.
    pub launcher: bool,
    /// The trained value the recipe is listed from (0: anyone with the skill).
    pub from: u16,
    /// Listed for the untrained too.
    pub anyone: bool,
    /// The built-in action the row runs, if it is one.
    pub uses: Option<RecipeUse>,
}

/// What `data/recipes.toml` says of a profession.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfessionText {
    pub skill: SkillId,
    /// Its abilities-menu row's line ("Your recipes as a smith.").
    pub description: String,
}

/// Every recipe in path order, and the professions' texts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Recipes {
    pub list: Vec<RecipeDef>,
    pub professions: Vec<ProfessionText>,
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
}

/// `data/recipes.toml`: the settings the recipes share. The forge's bands, the counted bands, and
/// a failure's yield join it with the phases that read them (docs/design/recipes.md §10).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipesFile {
    #[serde(default)]
    profession: Vec<ProfessionSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfessionSpec {
    skill: String,
    description: String,
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
    #[serde(default)]
    anyone: bool,
    #[serde(rename = "use")]
    uses: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultSpec {
    object: String,
    #[serde(default = "one")]
    count: Dice,
}

fn one() -> Dice {
    Dice::fixed(1)
}

/// A chosen part's source: `"stock"`, or `{ input = "arrowhead" }`. Part sources of more shapes
/// (stock lists, `optional`, `gem`) join it in P4b (docs/design/recipes.md §10).
#[derive(Deserialize)]
#[serde(untagged)]
enum PartSpec {
    Word(String),
    Input(InputPartSpec),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputPartSpec {
    input: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputSpec {
    object: String,
    count: u32,
    #[serde(default)]
    per_made: bool,
}

/// The settings file's path.
pub(super) const SETTINGS: &str = "data/recipes.toml";

/// Parses the settings file `settings` and every recipe file of `files` (path order; the settings
/// file among them is skipped), checking each as docs/design/professions.md §6 lists: a file in a
/// skill's directory, one recipe a file, its id its file name and unique, the result an object
/// and a count above 1 only on a stackable one, a built-in recipe with no result, parts, inputs,
/// ornament, difficulty, or seconds, a generic one with a result, a difficulty, and seconds of at
/// least 1, every part named on the result, a stock part's list holding a material of the
/// profession, an input part naming an input, inputs that are objects (at least one), an
/// ornament only on a result with an ornament part, a station that is terrain, and a known `use`.
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
        for row in file.profession {
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
            recipes.professions.push(ProfessionText {
                skill,
                description: row.description,
            });
        }
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
        recipes.list.push(recipe);
    }
    Ok(recipes)
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
        anyone: spec.anyone,
        uses: None,
    };
    if let Some(uses) = &spec.uses {
        let uses = RecipeUse::parse(uses)?;
        let generic = spec.result.is_some()
            || !spec.parts.is_empty()
            || !spec.inputs.is_empty()
            || spec.ornament
            || spec.difficulty.is_some()
            || spec.seconds.is_some();
        if generic {
            return Err(format!(
                "a `use` recipe runs its action: no result, parts, inputs, ornament, \
                 difficulty, or seconds (`use = \"{}\"`)",
                uses.id()
            ));
        }
        let name = spec.name.clone().unwrap_or_else(|| uses.name().into());
        return Ok(RecipeDef {
            uses: Some(uses),
            ..common(name)
        });
    }
    let result = spec.result.as_ref().ok_or("needs a `result` or a `use`")?;
    let object = (content.object_id(&result.object))
        .ok_or_else(|| format!("unknown result object `{}`", result.object))?;
    let def = content.object(object);
    if result.count.min() < 1 {
        return Err(format!("result count `{}` may make none", result.count));
    }
    if result.count.max() > 1 && !def.stackable {
        return Err(format!("`{}` does not stack: count 1", result.object));
    }
    let difficulty = spec.difficulty.ok_or("needs a `difficulty`")?;
    let seconds = spec
        .seconds
        .filter(|&s| s >= 1)
        .ok_or("needs `seconds` of 1 or more")?;
    let mut inputs = Vec::new();
    for input in &spec.inputs {
        let id = (content.object_id(&input.object))
            .ok_or_else(|| format!("unknown input object `{}`", input.object))?;
        if input.count == 0 {
            return Err(format!("input `{}`: count 0", input.object));
        }
        inputs.push(RecipeInput {
            object: id,
            count: input.count,
            per_made: input.per_made,
        });
    }
    let mut parts = Vec::new();
    for (name, source) in &spec.parts {
        let part = part_index(content, object, name)
            .ok_or_else(|| format!("`{}` has no part `{name}`", result.object))?;
        let source = match source {
            PartSpec::Word(word) if word == "stock" => {
                let of_skill = (def.made_of(part).iter()).any(|&m| {
                    (content.stock().profession_of(content, m)).is_some_and(|p| p.skill == skill)
                });
                if !of_skill {
                    return Err(format!(
                        "part `{name}`: none of what it may be made of is the profession's stock"
                    ));
                }
                PartSource::Stock
            }
            PartSpec::Word(word) => {
                return Err(format!(
                    "part `{name}`: `{word}` is no source (`\"stock\"` or `{{ input = ... }}`)"
                ));
            }
            PartSpec::Input(InputPartSpec { input }) => {
                let id = (content.object_id(input))
                    .filter(|id| inputs.iter().any(|i| i.object == *id))
                    .ok_or_else(|| format!("part `{name}`: `{input}` is none of the inputs"))?;
                PartSource::Input(id)
            }
        };
        parts.push(RecipePart { part, source });
    }
    parts.sort_by_key(|p| p.part);
    if spec.ornament {
        let placed = (content.gems()).and_then(|g| g.ornament(def.class));
        if def.ornament.is_none() && placed.is_none() {
            return Err(format!("`{}` has no ornament part", result.object));
        }
    }
    let name = spec.name.clone().unwrap_or_else(|| capitalized(&def.name));
    Ok(RecipeDef {
        result: Some(RecipeResult {
            object,
            count: result.count,
        }),
        parts,
        inputs,
        ornament: spec.ornament,
        difficulty,
        seconds,
        ..common(name)
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

/// `text` with its first letter in upper case ("Long sword").
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
