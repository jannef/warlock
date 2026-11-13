//! Crafting by recipe (spec §8 "Skills": professions; docs/design/professions.md §6, §8;
//! docs/dev/crafting.md): what a profession's screen lists, and [`Action::Craft`].
//!
//! - **The rows** ([`recipe_rows`], what [`PerceptionView::recipes`] reads): a profession's
//!   recipes in path order, for one trained in the skill (none for the untrained: every recipe
//!   is a skill's, #1387), each with what it takes in words the player sees elsewhere ("an
//!   anvil, about 10 minutes"; never a difficulty or odds) and the first reason it cannot be made
//!   now, if any ([`RecipeLack`]): computed here, from the pack, the cells beside the player, and
//!   the skills, so presentation never decides availability.
//! - **Known inputs** ([`known_object`]): an object input counts only once the player knows its
//!   type ([`Lore::is_identified`]: a labelled potion, flask, scroll, wand, or rod does not), in
//!   the rows, the wizard's steps, and the action alike, so no row tells what an unknown stack is
//!   (#1383).
//! - **The action** ([`Action::Craft`], `craft:ID[:POS,...]`): refused, taking no time, for one
//!   untrained in the recipe's skill ("You are not trained in Woodcraft."; `World::untrained`,
//!   which `temper:` and `whittle:` ask too); a built-in recipe's row runs its own action
//!   (`temper:`, `whittle:`), so `craft:` refuses it, taking no time, but for Cut and Set, which are
//!   `craft:cut:LETTER` and `craft:set:ITEM,GEM` themselves (`world/cut.rs`, `world/set.rs`); a
//!   generic recipe is made by `world/craft_make.rs` (the forge's weapons, the hand-made ones,
//!   the arrows, the brews).
//!
//! [`Action::Craft`]: crate::action::Action::Craft
//! [`PerceptionView::recipes`]: crate::perception::PerceptionView::recipes

use super::{Actor, LevelState, Spent, World};
use crate::content::{
    Content, InputKind, MaterialId, ObjectId, PartSource, RecipeDef, RecipeId, RecipeInput,
    RecipeUse, TerrainId, article, counted,
};
use crate::geom::Dir;
use crate::scroll_label::Lore;
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
    /// The player's trained value is below the recipe's `from`.
    Skill,
    /// No station of this terrain beside the player.
    Station(TerrainId),
    /// No knife carried.
    Knife,
    /// No launcher ready that shoots this result (a bow for arrows).
    Launcher(ObjectId),
    /// No stack of stock fits a chosen part, or holds a stock input: this material's group is
    /// what is missing.
    Stock(MaterialId),
    /// Too few of an input carried.
    Input(ObjectId),
    /// No cut loose gem carried for a part that must be one (a gold amulet's setting, Set's).
    Gem,
    /// Nothing carried with an unset ornament part (Set's).
    Setting,
    /// Nothing carried an enchanting recipe may work (`world/enchant.rs`).
    Nothing,
    /// Only items with no room for another affix (Quicken's).
    Room,
    /// No crystal carried, for an enchanting recipe that takes one.
    Crystal,
}

impl RecipeLack {
    /// The reason, as the row says it: "no anvil beside you".
    pub fn reason(self, content: &Content) -> String {
        match self {
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
            RecipeLack::Input(o) => format!("not enough {}", content.object(o).plural),
            RecipeLack::Gem => "no gem to hand".into(),
            RecipeLack::Setting => "nothing to set a gem in".into(),
            RecipeLack::Nothing => "nothing to enchant".into(),
            RecipeLack::Room => "it holds all it can".into(),
            RecipeLack::Crystal => "no crystal to hand".into(),
        }
    }

    /// Its id in dumps: `skill`, `station:anvil`, `knife`, `launcher`, `stock:metal`,
    /// `input:arrowhead`, `gem`, `setting`, `nothing`, `room`, `crystal`.
    pub fn id(self, content: &Content) -> String {
        match self {
            RecipeLack::Skill => "skill".into(),
            RecipeLack::Station(t) => format!("station:{}", content.terrain(t).id),
            RecipeLack::Knife => "knife".into(),
            RecipeLack::Launcher(_) => "launcher".into(),
            RecipeLack::Stock(m) => {
                let def = content.material(m);
                format!("stock:{}", def.group.as_deref().unwrap_or(&def.id))
            }
            RecipeLack::Input(o) => format!("input:{}", content.object(o).id),
            RecipeLack::Gem => "gem".into(),
            RecipeLack::Setting => "setting".into(),
            RecipeLack::Nothing => "nothing".into(),
            RecipeLack::Room => "room".into(),
            RecipeLack::Crystal => "crystal".into(),
        }
    }
}

/// The rows of profession `skill` for `player` on `level` (the module docs).
pub(crate) fn recipe_rows(
    content: &Content,
    lore: Lore<'_>,
    level: &LevelState,
    player: &Actor,
    skill: SkillId,
) -> Vec<RecipeRow> {
    if player.skills.get(skill).is_none() {
        return Vec::new();
    }
    (content.recipes().of(skill))
        .map(|(id, recipe)| RecipeRow {
            id,
            name: recipe.name.clone(),
            takes: takes(content, level, player, recipe),
            unavailable: lack(content, lore, level, player, recipe),
        })
        .collect()
}

/// The first reason `player` cannot make `recipe` now, in the screen's order: skill, station,
/// knife, Set's item and gem, launcher, stock, inputs, a gem part. `"stock"` parts (but a
/// `count = "fletch"` recipe's wood on wooded ground, `world/fletch.rs`), a required part's
/// sources, object inputs (those a part's sources name only there), stock inputs, and gem parts
/// are looked at, an object input only of a type the player knows. A stock input (a bow's sinew) wants one stack holding all it takes, as the
/// wizard picks one.
pub(super) fn lack(
    content: &Content,
    lore: Lore<'_>,
    level: &LevelState,
    player: &Actor,
    recipe: &RecipeDef,
) -> Option<RecipeLack> {
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
    if recipe.uses == Some(RecipeUse::Set) {
        return super::set::lack(content, player);
    }
    if let Some(RecipeUse::Enchant(_)) = recipe.uses {
        return super::enchant::lack(content, player, recipe);
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
                let pack = &player.inventory;
                let held = input.is_some_and(|o| pack.iter().any(|s| known_object(lore, s, o)));
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
    for input in &recipe.inputs {
        let fitting = (player.inventory.iter()).filter(|s| input_fits(lore, input, s));
        let Some(object) = input.object() else {
            if !fitting.into_iter().any(|s| s.count >= input.count) {
                return input_material(content, input).map(RecipeLack::Stock);
            }
            continue;
        };
        if sourced.contains(&object) {
            continue;
        }
        if fitting.map(|s| s.count).sum::<u32>() < input.count {
            return Some(RecipeLack::Input(object));
        }
    }
    let gem_part = (recipe.parts.iter()).any(|p| p.source == PartSource::Gem);
    let gem = (player.inventory.iter()).any(|s| super::craft_make::cut_gem(content, s).is_some());
    (gem_part && !gem).then_some(RecipeLack::Gem)
}

/// Whether `stack` is what `input` takes: a stack of its object, of a type the player knows
/// ([`known_object`]), or a piece of its material or of a material of its group (a crossbow's
/// lock: an ingot of any metal).
pub(super) fn input_fits(
    lore: Lore<'_>,
    input: &RecipeInput,
    stack: &crate::object::Stack,
) -> bool {
    let content = lore.content;
    match &input.kind {
        InputKind::Object(object) => known_object(lore, stack, *object),
        InputKind::Material(material) => stack_material(content, stack) == Some(*material),
        InputKind::Group(group) => stack_material(content, stack)
            .is_some_and(|m| content.material(m).group.as_deref() == Some(group.as_str())),
    }
}

/// Whether `stack` is of `object` as the player knows it: of that object, and of a type they
/// have identified (module docs). An unknown flask of oil is no flask of oil to a recipe, so its
/// row cannot tell what it is (#1383). The one test every object input is matched by.
pub(super) fn known_object(lore: Lore<'_>, stack: &crate::object::Stack, object: ObjectId) -> bool {
    stack.object == object && lore.is_identified(object)
}

/// The material a stock input names, or the first of its group: what its lack and its words
/// are said by.
fn input_material(content: &Content, input: &RecipeInput) -> Option<MaterialId> {
    match &input.kind {
        InputKind::Object(_) => None,
        InputKind::Material(material) => Some(*material),
        InputKind::Group(group) => (content.materials().iter())
            .position(|m| m.group.as_deref() == Some(group.as_str()))
            .and_then(|i| u16::try_from(i).ok())
            .map(MaterialId),
    }
}

/// A stock input in words: "a length of sinew", "an ingot of any metal".
fn stock_input_words(content: &Content, input: &RecipeInput) -> Option<String> {
    let material = input_material(content, input)?;
    let piece = content.object(content.object_id(crate::piece::ID)?);
    let (one, many) = match &input.kind {
        InputKind::Group(group) => {
            let (name, plural, _) = content.stock().form_of(content, material)?;
            (
                format!("{name} of any {group}"),
                format!("{plural} of any {group}"),
            )
        }
        _ => crate::piece::names(content, piece, material),
    };
    Some(counted(&one, &many, input.count))
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
        Some(RecipeUse::Enchant(kind)) => {
            words.extend(super::enchant::takes(kind, recipe.crystals))
        }
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
                    let object = result.object;
                    words.extend(stock_words(
                        content, recipe, object, part.part, &worked, batch,
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
                    let object = result.object;
                    either.extend(stock_words(
                        content, recipe, object, part.part, stock, batch,
                    ));
                    if !either.is_empty() {
                        words.push(either.join(" or "));
                    }
                }
                PartSource::Gem => {}
            }
        }
    }
    if (recipe.parts.iter()).any(|p| p.source == PartSource::Gem) {
        words.push("a cut gem".into());
    }
    for input in &recipe.inputs {
        let Some(object) = input.object() else {
            words.extend(stock_input_words(content, input));
            continue;
        };
        if !sourced.contains(&object) {
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
        Some(RecipeUse::Cut | RecipeUse::Set | RecipeUse::Enchant(_)) => recipe.seconds,
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

/// The stock part `part` of `batch` of `object` takes, from `materials`, priced in the part's
/// base material where it is among them (else the cheapest of its group), once for each group
/// they hold, the base material's group first: "3 steel ingots", "7 wood billets or 7 bone shards"
/// (docs/design/professions.md §3: the part's share of the result's weight, in pieces; another
/// material of the group takes as many of its own).
fn stock_words(
    content: &Content,
    recipe: &RecipeDef,
    object: ObjectId,
    part: usize,
    materials: &[MaterialId],
    batch: u32,
) -> Option<String> {
    let base = content.object(object).parts.get(part)?;
    let piece = content.object(content.object_id(crate::piece::ID)?);
    let group = |m: MaterialId| content.material(m).group.clone();
    // The base material's group first, each other group by its cheapest: a club is "7 wood
    // billets or 7 bone shards", a bow "4 wood billets or 4 horn shards".
    let mut order = materials.to_vec();
    order.sort_by_key(|&m| group(m) != group(base.material));
    let mut groups: Vec<Option<String>> = Vec::new();
    let mut words: Vec<String> = Vec::new();
    for &first in &order {
        if groups.contains(&group(first)) {
            continue;
        }
        groups.push(group(first));
        // The base material when it is of this group, else the piece's own (steel: a wooden
        // small shield in metal is "13 steel ingots", never adamantium's), else the group's
        // cheapest.
        let named = |want: MaterialId| {
            (materials.iter().copied()).find(|&m| m == want && group(m) == group(first))
        };
        let material = (named(base.material))
            .or_else(|| piece.parts.first().and_then(|p| named(p.material)))
            .or_else(|| {
                (materials.iter().copied())
                    .filter(|&m| group(m) == group(first))
                    .min_by_key(|&m| content.material(m).value)
            })
            .unwrap_or(first);
        let count = super::craft_make::stock_need(content, recipe, part, material, batch)?;
        let (one, many) = crate::piece::names(content, piece, material);
        words.push(counted(&one, &many, count));
    }
    (!words.is_empty()).then(|| words.join(" or "))
}

impl World {
    /// Whether the player is untrained in profession `skill`, saying so ("You are not trained
    /// in Metalcraft."): every recipe is its skill's (#1387), so its action is then refused,
    /// taking no time.
    pub(super) fn untrained(&mut self, skill: SkillId) -> bool {
        if self.state.player().skills.get(skill).is_some() {
            return false;
        }
        let name = &self.content.skill(skill).name;
        self.say(format!("You are not trained in {name}."));
        true
    }

    /// The skill of built-in recipe `uses` (Temper's Metalcraft, Whittle's Woodcraft), if the
    /// content has it.
    pub(super) fn skill_of_use(&self, uses: RecipeUse) -> Option<SkillId> {
        self.content
            .recipes()
            .of_use(uses)
            .map(|recipe| recipe.skill)
    }

    /// The player makes a recipe (the module docs): a generic one is begun by
    /// [`World::start_making`]; a built-in or unknown one is refused, taking no time.
    pub(super) fn start_craft(&mut self, recipe: &str, picks: &[Option<usize>]) -> Spent {
        let content = self.content.clone();
        let Some(id) = content.recipes().id(recipe) else {
            self.say("You know no such recipe.");
            return Spent::Nothing;
        };
        let def = content.recipe(id);
        if self.untrained(def.skill) {
            return Spent::Nothing;
        }
        let text = match def.uses {
            Some(RecipeUse::Cut) => return self.start_cut(id, picks),
            Some(RecipeUse::Set) => return self.start_set(id, picks),
            Some(RecipeUse::Enchant(_)) => return self.start_enchant(id, picks),
            Some(_) => format!("{} is done from its own list.", def.name),
            None => return self.start_making(id, picks),
        };
        self.say(text);
        Spent::Nothing
    }
}
