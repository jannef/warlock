//! The `abilities` and `recipes` layers: what the player can do and why not.

use super::*;

/// The player's abilities, one a line in menu order: its name, its source, and what it asks for
/// or why it cannot be used now.
pub(super) fn ability_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    (view.abilities().iter())
        .map(|own| {
            let a = own.ability;
            let state = match own.unavailable {
                Some(why) => why.reason(content),
                None => aim_word(a.aim(content)).into(),
            };
            let source = a.source().id().replace('_', " ");
            format!("{} ({source}): {state}", view.ability_name(a))
        })
        .collect()
}

/// Every profession's recipes as its screen lists them for the player: a `Metalcraft:` line,
/// then one a line, `Temper: a metal weapon or armour, an anvil, about 10 minutes (no anvil
/// beside you)`; `none` under a profession with no recipe they see.
pub(super) fn recipe_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let mut rows = Vec::new();
    for skill in content.recipes().professions() {
        rows.push(format!("{}:", content.skill(skill).name));
        let recipes = view.recipes(skill);
        if recipes.is_empty() {
            rows.push("  none".into());
        }
        for row in recipes {
            let why = (row.unavailable)
                .map_or(String::new(), |why| format!(" ({})", why.reason(content)));
            rows.push(format!("  {}: {}{why}", row.name, row.takes));
        }
    }
    rows
}

/// What an ability asks for, in a word or two.
pub(super) fn aim_word(aim: Aim) -> &'static str {
    match aim {
        Aim::Nothing => "nothing",
        Aim::Direction => "a direction",
        Aim::Cell => "a cell",
        Aim::Corpse => "a corpse you see",
        Aim::Ailment => "what to treat",
        Aim::Item(_) => "an item you carry",
        Aim::Throw => "something to throw, then a target",
        Aim::Shape => "a weapon's shape",
        Aim::Rig => "a rig and a direction",
        Aim::Spell => "a spell you know",
        Aim::Recipes => "a recipe",
    }
}

/// The player's abilities in menu order: `[{"id", "source", "aim", "unavailable"}]`
/// (`unavailable` the reason's id, `null` when it can be used now).
pub(super) fn abilities_json(view: &PerceptionView<'_>) -> Value {
    let content = view.content();
    (view.abilities().iter())
        .map(|own| {
            let a = own.ability;
            json!({
                "id": a.id(content),
                "source": a.source().id(),
                "aim": aim_word(a.aim(content)),
                "unavailable": own.unavailable.map(|why| why.id(content)),
            })
        })
        .collect()
}
