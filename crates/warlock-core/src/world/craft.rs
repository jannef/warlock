//! Crafting by recipe (spec §8 "Skills": professions; docs/design/professions.md §6, §8;
//! docs/dev/crafting.md): what a profession's screen lists, and [`Action::Craft`].
//!
//! - **The rows** ([`recipe_rows`], what [`PerceptionView::recipes`] reads): a profession's
//!   recipes in path order that the player may see (all of them for one trained in the skill,
//!   else the `anyone` ones), each with what it takes in words the player sees elsewhere ("an
//!   anvil, about 10 minutes"; never a difficulty or odds) and the first reason it cannot be made
//!   now, if any ([`RecipeLack`]): computed here, from the pack, the cells beside the player, and
//!   the skills, so presentation never decides availability.
//! - **The action** ([`Action::Craft`], `craft:ID[:POS,...]`): a built-in recipe's row runs its
//!   own action (`temper:`, `whittle:`), so `craft:` refuses it, taking no time; a generic recipe
//!   is made by `world/craft_make.rs` (the forge's swords, the arrows).
//!
//! [`Action::Craft`]: crate::action::Action::Craft
//! [`PerceptionView::recipes`]: crate::perception::PerceptionView::recipes

use super::{Actor, LevelState, Spent, World};
use crate::content::{
    Content, MaterialId, ObjectId, PartSource, RecipeDef, RecipeId, RecipeUse, TerrainId, article,
    counted,
};
use crate::geom::Dir;
use crate::skills::SkillId;
use crate::time::Duration;

/// One recipe as the profession's screen lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeRow {
    pub id: RecipeId,
    pub name: String,
    /// What it takes, in words: "a metal weapon or armour, an anvil, about 10 minutes".
    pub takes: String,
    /// Why it cannot be made now, the first reason that applies; `None` when it can.
    pub unavailable: Option<RecipeLack>,
}

/// Why a recipe cannot be made now (the screen greys its row with [`RecipeLack::reason`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecipeLack {
    /// A built-in recipe whose action is not built yet (Jewelcraft's Cut and Set).
    NotYet,
    /// The player's trained value is below the recipe's `from`.
    Skill,
    /// No station of this terrain beside the player.
    Station(TerrainId),
    /// No knife carried.
    Knife,
    /// No launcher ready that shoots this result (a bow for arrows).
    Launcher(ObjectId),
    /// No stack of stock fits a chosen part: this material's group is what is missing.
    Stock(MaterialId),
    /// Too few of an input carried.
    Input(ObjectId),
}

impl RecipeLack {
    /// The reason, as the row says it: "no anvil beside you".
    pub fn reason(self, content: &Content) -> String {
        match self {
            RecipeLack::NotYet => "not usable yet".into(),
            RecipeLack::Skill => "beyond your skill".into(),
            RecipeLack::Station(t) => format!("no {} beside you", content.terrain(t).name),
            RecipeLack::Knife => "no knife".into(),
            RecipeLack::Launcher(o) => {
                format!("no {} ready", super::fletch::launcher_word(content, o))
            }
            RecipeLack::Stock(m) => {
                let def = content.material(m);
                format!("no {} to hand", def.group.as_deref().unwrap_or(&def.name))
            }
            RecipeLack::Input(o) => format!("no {}", content.object(o).plural),
        }
    }

    /// Its id in dumps: `not_yet`, `skill`, `station:anvil`, `knife`, `launcher`, `stock:metal`,
    /// `input:arrowhead`.
    pub fn id(self, content: &Content) -> String {
        match self {
            RecipeLack::NotYet => "not_yet".into(),
            RecipeLack::Skill => "skill".into(),
            RecipeLack::Station(t) => format!("station:{}", content.terrain(t).id),
            RecipeLack::Knife => "knife".into(),
            RecipeLack::Launcher(_) => "launcher".into(),
            RecipeLack::Stock(m) => {
                let def = content.material(m);
                format!("stock:{}", def.group.as_deref().unwrap_or(&def.id))
            }
            RecipeLack::Input(o) => format!("input:{}", content.object(o).id),
        }
    }
}

/// The rows of profession `skill` for `player` on `level` (the module docs).
pub(crate) fn recipe_rows(
    content: &Content,
    level: &LevelState,
    player: &Actor,
    skill: SkillId,
) -> Vec<RecipeRow> {
    let trained = player.skills.get(skill).is_some();
    (content.recipes().of(skill))
        .filter(|(_, recipe)| trained || recipe.anyone)
        .map(|(id, recipe)| RecipeRow {
            id,
            name: recipe.name.clone(),
            takes: takes(content, level, player, recipe),
            unavailable: lack(content, level, player, recipe),
        })
        .collect()
}

/// The first reason `player` cannot make `recipe` now, in the screen's order: not built yet,
/// skill, station, knife, launcher, stock, inputs. `"stock"` parts (but a `count = "fletch"`
/// recipe's wood on wooded ground, `world/fletch.rs`), a required part's sources, and object
/// inputs are looked at (those a part's sources name only there); stock inputs and gem parts
/// (docs/design/recipes.md §10) are checked by the phases whose recipes first use them.
pub(super) fn lack(
    content: &Content,
    level: &LevelState,
    player: &Actor,
    recipe: &RecipeDef,
) -> Option<RecipeLack> {
    if recipe.uses.is_some_and(|u| !u.usable()) {
        return Some(RecipeLack::NotYet);
    }
    let value = player.skills.value(recipe.skill).unwrap_or(0);
    if value < recipe.from {
        return Some(RecipeLack::Skill);
    }
    if let Some(station) = recipe.station {
        let beside =
            (Dir::ALL.iter()).any(|&d| level.terrain_at(player.pos.step(d)) == Some(station));
        if !beside {
            return Some(RecipeLack::Station(station));
        }
    }
    if recipe.knife && !(player.inventory.iter()).any(|s| content.is_knife(s.object)) {
        return Some(RecipeLack::Knife);
    }
    let result = recipe.result.as_ref();
    if let Some(result) = result.filter(|_| recipe.launcher)
        && !super::fletch::shoots(player, content, result.object)
    {
        return Some(RecipeLack::Launcher(result.object));
    }
    let carries = |materials: &[MaterialId]| {
        (player.inventory.iter())
            .any(|stack| stack_material(content, stack).is_some_and(|m| materials.contains(&m)))
    };
    let mut sourced = Vec::new();
    for part in &recipe.parts {
        let Some(result) = result else { break };
        match &part.source {
            PartSource::Stock => {
                let worked = worked_materials(content, recipe, result.object, part.part);
                let free = super::fletch::part_free(content, level, player, recipe, part.part);
                if !free && !carries(&worked) {
                    return worked.first().map(|&m| RecipeLack::Stock(m));
                }
            }
            PartSource::Sources {
                input,
                stock,
                optional,
            } => {
                sourced.extend(*input);
                let held = input.is_some_and(|o| player.inventory.iter().any(|s| s.object == o));
                if !optional && !held && !carries(stock) {
                    return match (input, stock.first()) {
                        (Some(o), _) => Some(RecipeLack::Input(*o)),
                        (None, m) => m.map(|&m| RecipeLack::Stock(m)),
                    };
                }
            }
            PartSource::Gem => {}
        }
    }
    let inputs = (recipe.inputs.iter()).filter_map(|i| Some((i.object()?, i.count)));
    for (object, count) in inputs.filter(|(o, _)| !sourced.contains(o)) {
        let carried: u32 = (player.inventory.iter())
            .filter(|s| s.object == object)
            .map(|s| s.count)
            .sum();
        if carried < count {
            return Some(RecipeLack::Input(object));
        }
    }
    None
}

/// The materials part `part` of `object` may be made of that `recipe`'s profession works, in the
/// part's list order: what a stock stack must be of.
pub(super) fn worked_materials(
    content: &Content,
    recipe: &RecipeDef,
    object: ObjectId,
    part: usize,
) -> Vec<MaterialId> {
    (content.object(object).made_of(part).iter().copied())
        .filter(|&m| {
            (content.stock().profession_of(content, m)).is_some_and(|p| p.skill == recipe.skill)
        })
        .collect()
}

/// The material of `stack` when it is stock (a piece): its main part's.
pub(super) fn stack_material(
    content: &Content,
    stack: &crate::object::Stack,
) -> Option<MaterialId> {
    if Some(stack.object) != content.object_id(crate::piece::ID) {
        return None;
    }
    stack.parts(content).first().map(|p| p.material)
}

/// What `recipe` takes, in words, joined with commas: what a built-in one works on, the parts'
/// stock and sources (a stock part priced for the batch at its most; none for wood free here),
/// the other inputs, the station, a knife, a launcher, then the time.
fn takes(content: &Content, level: &LevelState, player: &Actor, recipe: &RecipeDef) -> String {
    let mut words: Vec<String> = Vec::new();
    match recipe.uses {
        Some(RecipeUse::Temper) => words.push("a metal weapon or armour".into()),
        Some(RecipeUse::Whittle) => words.push("a wooden weapon".into()),
        Some(RecipeUse::Cut) => words.push("a loose gem".into()),
        Some(RecipeUse::Set) => words.push("an item and a cut gem".into()),
        None => {}
    }
    let input_words = |input: &crate::content::RecipeInput| {
        let def = content.object(input.object()?);
        Some(match input.per_made {
            true => format!(
                "{} for each made",
                counted(&def.name, &def.plural, input.count)
            ),
            false => counted(&def.name, &def.plural, input.count),
        })
    };
    let mut sourced = Vec::new();
    if let Some(result) = &recipe.result {
        let batch = super::craft_make::batch_most(content, player, recipe);
        for part in &recipe.parts {
            match &part.source {
                PartSource::Stock => {
                    if super::fletch::part_free(content, level, player, recipe, part.part) {
                        continue;
                    }
                    let worked = worked_materials(content, recipe, result.object, part.part);
                    words.extend(stock_words(
                        content,
                        result.object,
                        part.part,
                        &worked,
                        batch,
                    ));
                }
                PartSource::Sources { input, stock, .. } => {
                    let mut either: Vec<String> = Vec::new();
                    let input = (recipe.inputs.iter())
                        .find(|i| i.object().is_some() && i.object() == *input);
                    if let Some(input) = input {
                        sourced.extend(input.object());
                        either.extend(input_words(input));
                    }
                    // A piece of each group listed, priced as the batch takes it.
                    let mut groups: Vec<Option<&str>> = Vec::new();
                    for &m in stock {
                        let group = content.material(m).group.as_deref();
                        if groups.contains(&group) {
                            continue;
                        }
                        groups.push(group);
                        let same: Vec<MaterialId> = (stock.iter().copied())
                            .filter(|&x| content.material(x).group.as_deref() == group)
                            .collect();
                        either.extend(stock_words(content, result.object, part.part, &same, batch));
                    }
                    if !either.is_empty() {
                        words.push(either.join(" or "));
                    }
                }
                PartSource::Gem => {}
            }
        }
    }
    for input in &recipe.inputs {
        // Stock inputs are told by the phase whose recipe first takes one (P7).
        if input.object().is_some_and(|o| !sourced.contains(&o)) {
            words.extend(input_words(input));
        }
    }
    if let Some(station) = recipe.station {
        let name = &content.terrain(station).name;
        words.push(format!("{} {name}", article(name)));
    }
    if recipe.knife {
        words.push("a knife".into());
    }
    if let Some(result) = recipe.result.as_ref().filter(|_| recipe.launcher) {
        let word = super::fletch::launcher_word(content, result.object);
        words.push(format!("{} {word} ready", article(word)));
    }
    let seconds = match recipe.uses {
        Some(RecipeUse::Temper | RecipeUse::Whittle) => content.temper().seconds,
        // Their actions' times come with them (P10, P11).
        Some(RecipeUse::Cut | RecipeUse::Set) => 0,
        None => super::fletch::fletcher_seconds(content, player, recipe),
    };
    if seconds > 0 {
        words.push(format!(
            "about {}",
            Duration::seconds(u64::from(seconds)).spoken()
        ));
    }
    words.join(", ")
}

/// The stock part `part` of `batch` of `object` takes, priced in the part's base material where
/// it is among `materials` (else the first of them): "3 steel ingots" (docs/design/professions.md
/// §3: the part's share of the result's weight, scaled by the material's, in pieces).
fn stock_words(
    content: &Content,
    object: ObjectId,
    part: usize,
    materials: &[MaterialId],
    batch: u32,
) -> Option<String> {
    let def = content.object(object);
    let base = def.parts.get(part)?;
    let material = (materials.iter().copied())
        .find(|&m| m == base.material)
        .or_else(|| materials.first().copied())?;
    let weight = |m: MaterialId| u64::from(content.material(m).weight).max(1);
    let grams = u64::from(def.weight) * u64::from(base.share) * weight(material)
        / weight(base.material)
        / 100;
    let count = crate::piece::count_for(content, grams * u64::from(batch), material)?;
    let piece = content.object(content.object_id(crate::piece::ID)?);
    let (one, many) = crate::piece::names(content, piece, material);
    Some(counted(&one, &many, count))
}

impl World {
    /// The player makes a recipe (the module docs): a generic one is begun by
    /// [`World::start_making`]; a built-in or unknown one is refused, taking no time.
    pub(super) fn start_craft(&mut self, recipe: &str, picks: &[Option<usize>]) -> Spent {
        let content = self.content.clone();
        let Some(id) = content.recipes().id(recipe) else {
            self.say("You know no such recipe.");
            return Spent::Nothing;
        };
        let def = content.recipe(id);
        let text = match def.uses {
            Some(uses) if !uses.usable() => "You cannot do that yet.".to_string(),
            Some(_) => format!("{} is done from its own list.", def.name),
            None => return self.start_making(id, picks),
        };
        self.say(text);
        Spent::Nothing
    }
}
