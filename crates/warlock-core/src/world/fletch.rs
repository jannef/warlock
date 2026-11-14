//! Fletching (spec §8 "Fletching"; #1021, #1336): what the `count = "fletch"` recipes (the arrows
//! and bolts of `recipes/woodcraft/`) and a recipe's `launcher` read beyond the generic making of
//! `world/craft_make.rs`, which makes them.
//!
//! - **A launcher ready** ([`shoots`]): a recipe with `launcher` wants a missile weapon ready
//!   whose `launches` is the result's `ammo` (a bow for arrows, a crossbow for bolts); the row
//!   names it by `data/fletch.toml`'s `[[makes]]` word ([`launcher_word`]: "a bow ready").
//! - **The count** ([`fletch_count`], [`fletch_most`]): `data/fletch.toml`'s `count` dice (at
//!   least 1) plus one for every full `value_per_count` of the recipe's skill (Woodcraft), then
//!   the margin's bands, then the ranger's Fletcher's `count_times`. Fletcher also adds its
//!   `check` and cuts the time to its `time_percent` ([`fletcher_seconds`]).
//! - **Wood where trees are** ([`wood_free`]): on ground whose landscape `data/recipes.toml`'s
//!   `free_wood` lists (the overworld's or a local area's forest, dense forest, or swamp), a
//!   `count = "fletch"` recipe's wood part has no step and takes no billet: the shaft is cut from
//!   what wood there is.

use super::{Actor, LevelState};
use crate::content::{Content, ObjectId, RecipeCount, RecipeDef};
use crate::geom::Pos;
use crate::rng::Rng;

/// Whether `actor` has a missile weapon ready that shoots `object` (its `ammo`).
pub(super) fn shoots(actor: &Actor, content: &Content, object: ObjectId) -> bool {
    let Some(ammo) = content.object(object).ammo.as_deref() else {
        return false;
    };
    let launches = (actor.launcher(content)).and_then(|(l, _)| content.object(l).launches.clone());
    launches.as_deref() == Some(ammo)
}

/// The launcher that shoots `object`, in a word (`data/fletch.toml`'s `[[makes]]`): "bow".
pub(super) fn launcher_word(content: &Content, object: ObjectId) -> &str {
    let ammo = content.object(object).ammo.as_deref();
    (content.fletch().zip(ammo))
        .and_then(|(rules, ammo)| rules.makes_for(ammo))
        .map_or("launcher", |m| m.launcher.as_str())
}

/// Whether `recipe` counts by Fletch's rule (`count = "fletch"`).
pub(super) fn is_fletch(recipe: &RecipeDef) -> bool {
    recipe
        .result
        .as_ref()
        .is_some_and(|r| r.count == RecipeCount::Fletch)
}

/// What `actor`'s skill in `recipe` (Woodcraft) adds to its count: one for every full
/// `value_per_count` of its trained value.
pub(super) fn share(content: &Content, actor: &Actor, recipe: &RecipeDef) -> i32 {
    let Some(rules) = content.fletch() else {
        return 0;
    };
    i32::from(actor.skills.value(recipe.skill).unwrap_or(0)) / i32::from(rules.value_per_count)
}

/// The most a batch of `recipe` by `actor` can make: the dice's most and the skill's share,
/// times Fletcher's.
pub(super) fn fletch_most(content: &Content, actor: &Actor, recipe: &RecipeDef) -> u32 {
    let dice = content.fletch().map_or(1, |rules| rules.count.max().max(1));
    let most = (dice + share(content, actor, recipe)) * count_times(content, actor, recipe);
    most.max(1).unsigned_abs()
}

/// The count a batch rolls before the margin (one gameplay draw a die): `data/fletch.toml`'s
/// dice, at least 1, and the skill's `share`.
pub(super) fn fletch_count(rng: &mut Rng, content: &Content, share: i32) -> i32 {
    content
        .fletch()
        .map_or(1, |rules| rules.count.roll(rng).max(1))
        + share
}

/// What Fletcher multiplies a `count = "fletch"` batch's count by (1 without it, or for another
/// recipe).
pub(super) fn count_times(content: &Content, actor: &Actor, recipe: &RecipeDef) -> i32 {
    match is_fletch(recipe) {
        true => (actor.talent_mods(content).craft.fletcher).map_or(1, |f| f.count_times),
        false => 1,
    }
}

/// What Fletcher adds to a `count = "fletch"` batch's check (0 without it, or for another
/// recipe).
pub(super) fn fletcher_check(content: &Content, actor: &Actor, recipe: &RecipeDef) -> i32 {
    match is_fletch(recipe) {
        true => (actor.talent_mods(content).craft.fletcher).map_or(0, |f| f.check),
        false => 0,
    }
}

/// How long `actor`'s work on `recipe` takes, in seconds: its `seconds`, cut by Fletcher for a
/// `count = "fletch"` recipe, then by Quick hands (every recipe's time, read here alone:
/// [`super::mastery::craft_seconds`]).
pub(super) fn fletcher_seconds(content: &Content, actor: &Actor, recipe: &RecipeDef) -> u32 {
    let percent = match is_fletch(recipe) {
        true => (actor.talent_mods(content).craft.fletcher).map_or(100, |f| f.time_percent),
        false => 100,
    };
    let seconds = (recipe.seconds * percent / 100).max(1);
    super::mastery::craft_seconds(content, actor, seconds)
}

/// Whether wood is free at `at` on `level` (module docs): its landscape on `data/recipes.toml`'s
/// `free_wood`.
pub(super) fn wood_free(content: &Content, level: &LevelState, at: Pos) -> bool {
    let Some(settings) = content.recipes().settings.as_ref() else {
        return false;
    };
    super::forage::landscape_at(content, level, at).is_some_and(|l| settings.free_wood.contains(&l))
}

/// Whether part `part` of `recipe`'s result is free here for `actor` (module docs): a
/// `count = "fletch"` recipe's part whose every worked material is wood, on wooded ground.
pub(super) fn part_free(
    content: &Content,
    level: &LevelState,
    actor: &Actor,
    recipe: &RecipeDef,
    part: usize,
) -> bool {
    let Some(result) = recipe.result.as_ref() else {
        return false;
    };
    if !is_fletch(recipe) {
        return false;
    }
    let worked = super::craft::worked_materials(content, recipe, result.object, part);
    let wood =
        |m: &crate::content::MaterialId| content.material(*m).group.as_deref() == Some("wood");
    !worked.is_empty() && worked.iter().all(wood) && wood_free(content, level, actor.pos)
}
