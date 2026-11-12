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
//!   own action (`temper:`, `whittle:`, `fletch`), so `craft:` refuses it, taking no time; a
//!   generic recipe is made by `world/craft_make.rs` (the forge's swords).
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
    /// No bow or crossbow ready that fletching makes ammunition for.
    Launcher,
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
            RecipeLack::Launcher => "no bow or crossbow ready".into(),
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
            RecipeLack::Launcher => "launcher".into(),
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
            takes: takes(content, player, recipe),
            unavailable: lack(content, level, player, recipe),
        })
        .collect()
}

/// The first reason `player` cannot make `recipe` now, in the screen's order: not built yet,
/// skill, station, knife, launcher, stock, inputs. Only `"stock"` parts and object inputs are
/// looked at: the other part sources and the stock inputs (docs/design/recipes.md §10) are
/// checked by the phases whose recipes first use them.
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
    if recipe.launcher && super::fletch::fletch_makes(player, content).is_none() {
        return Some(RecipeLack::Launcher);
    }
    let result = recipe.result.as_ref();
    for part in (recipe.parts.iter()).filter(|p| stock_only(&p.source)) {
        let Some(result) = result else { break };
        let worked = worked_materials(content, recipe, result.object, part.part);
        let fits = (player.inventory.iter())
            .any(|stack| stack_material(content, stack).is_some_and(|m| worked.contains(&m)));
        if !fits {
            return worked.first().map(|&m| RecipeLack::Stock(m));
        }
    }
    for (object, count) in (recipe.inputs.iter()).filter_map(|i| Some((i.object()?, i.count))) {
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

/// Whether a part's source is stock alone, the steps built so far: `"stock"`, or a table of
/// `stock` with no input that cannot be left off (a spear's haft, `{ stock = ["wood"] }`, any
/// profession's; docs/design/recipes.md §10).
pub(super) fn stock_only(source: &PartSource) -> bool {
    matches!(
        source,
        PartSource::Stock
            | PartSource::Sources {
                input: None,
                optional: false,
                ..
            }
    )
}

/// The materials part `part` of `object` may be made of as `recipe` takes it, in the part's list
/// order: what a stock stack must be of. A `"stock"` part takes what the recipe's profession
/// works; a table's `stock` list (any profession's) what it names.
pub(super) fn worked_materials(
    content: &Content,
    recipe: &RecipeDef,
    object: ObjectId,
    part: usize,
) -> Vec<MaterialId> {
    let source = (recipe.parts.iter()).find(|p| p.part == part);
    if let Some(PartSource::Sources { stock, .. }) = source.map(|p| &p.source) {
        return stock.clone();
    }
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

/// What `recipe` takes, in words, joined with commas: what a built-in one works on, the stock
/// and inputs, the station, a knife, a launcher, then the time.
fn takes(content: &Content, player: &Actor, recipe: &RecipeDef) -> String {
    let mut words: Vec<String> = Vec::new();
    match recipe.uses {
        Some(RecipeUse::Temper) => words.push("a metal weapon or armour".into()),
        Some(RecipeUse::Whittle) => words.push("a wooden weapon".into()),
        Some(RecipeUse::Cut) => words.push("a loose gem".into()),
        Some(RecipeUse::Set) => words.push("an item and a cut gem".into()),
        Some(RecipeUse::Fletch) | None => {}
    }
    if let Some(result) = &recipe.result {
        for part in (recipe.parts.iter()).filter(|p| stock_only(&p.source)) {
            if let Some(stock) = stock_words(content, recipe, result.object, part.part) {
                words.push(stock);
            }
        }
    }
    for input in &recipe.inputs {
        // Stock inputs are told by the phase whose recipe first takes one (P7).
        let Some(object) = input.object() else {
            continue;
        };
        let def = content.object(object);
        words.push(match input.per_made {
            true => format!("{} {} for each made", article(&def.name), def.name),
            false => counted(&def.name, &def.plural, input.count),
        });
    }
    if let Some(station) = recipe.station {
        let name = &content.terrain(station).name;
        words.push(format!("{} {name}", article(name)));
    }
    if recipe.knife {
        words.push("a knife".into());
    }
    if recipe.launcher {
        words.push("a bow or crossbow ready".into());
    }
    let seconds = match recipe.uses {
        Some(RecipeUse::Temper | RecipeUse::Whittle) => content.temper().seconds,
        Some(RecipeUse::Fletch) => {
            let percent = (player.talent_mods(content).fletcher).map_or(100, |f| f.time_percent);
            content
                .fletch()
                .map_or(0, |rules| rules.seconds * percent / 100)
        }
        // Their actions' times come with them (P10, P11).
        Some(RecipeUse::Cut | RecipeUse::Set) => 0,
        None => recipe.seconds,
    };
    if seconds > 0 {
        words.push(format!(
            "about {}",
            Duration::seconds(u64::from(seconds)).spoken()
        ));
    }
    words.join(", ")
}

/// The stock a chosen part takes, priced in the part's base material where the recipe takes it
/// (else the first it takes), once for each group it may be of: "3 steel ingots", "1 bone shard
/// or 1 block of flint" (docs/design/professions.md §3: the part's share of the result's weight,
/// in pieces; another material of the group takes as many of its own).
fn stock_words(
    content: &Content,
    recipe: &RecipeDef,
    object: ObjectId,
    part: usize,
) -> Option<String> {
    let base = content.object(object).parts.get(part)?;
    let worked = worked_materials(content, recipe, object, part);
    let piece = content.object(content.object_id(crate::piece::ID)?);
    let group = |m: MaterialId| content.material(m).group.clone();
    // The base material's group first: a club is "7 wood billets or 7 bone shards".
    let mut order = worked.clone();
    order.sort_by_key(|&m| group(m) != group(base.material));
    let mut groups: Vec<Option<String>> = Vec::new();
    let mut words: Vec<String> = Vec::new();
    for &first in &order {
        if groups.contains(&group(first)) {
            continue;
        }
        groups.push(group(first));
        // The base material when it is of this group, else the piece's own (steel: a wooden
        // small shield in metal is "13 steel ingots", never adamantium's), else the first.
        let named = |want: MaterialId| {
            (worked.iter().copied()).find(|&m| m == want && group(m) == group(first))
        };
        let material = (named(base.material))
            .or_else(|| piece.parts.first().and_then(|p| named(p.material)))
            .unwrap_or(first);
        let (count, _) = super::craft_make::stock_need(content, recipe, part, material)?;
        let (one, many) = crate::piece::names(content, piece, material);
        words.push(counted(&one, &many, count));
    }
    (!words.is_empty()).then(|| words.join(" or "))
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
