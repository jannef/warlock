//! The `spells` layer: the spells the player knows and what each does now.

use super::*;

/// The spells the player knows, one a line with its spell level, rank, and cost, then what it
/// does now as the abilities menu will show it (`Fireball (level 11, rank 1, 26 mana): 4d6
/// fire, range 11, radius 2`).
pub(super) fn spell_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let known = (view.own_spells().into_iter()).filter_map(|spell| {
        let def = content.spell_of(spell)?;
        let cost = view.own_spell_cost(spell)?;
        let rank = view.own_spell_rank(spell)?;
        let does = spell_does(view, spell);
        let does = if does.is_empty() {
            String::new()
        } else {
            format!(": {}", does.join(", "))
        };
        Some(format!(
            "{} (level {}, rank {rank}, {cost} mana){does}",
            def.name, def.level
        ))
    });
    let rows: Vec<String> = known.collect();
    if rows.is_empty() {
        vec!["no spells".into()]
    } else {
        rows
    }
}

/// What the player's spell `spell` does now, in parts: its roll with its damage type or
/// "healed", then its reach (`range 11`, `length 9`, `radius 2`).
pub(super) fn spell_does(view: &PerceptionView<'_>, spell: crate::content::SpellId) -> Vec<String> {
    let content = view.content();
    let Some(def) = content.spell_of(spell) else {
        return Vec::new();
    };
    let mut parts: Vec<String> = view
        .own_spell_roll(spell)
        .map(|r| r.text)
        .into_iter()
        .collect();
    if let Some(reach) = view.own_spell_reach(spell) {
        parts.extend(crate::world::reach_words(def.shape, reach));
    }
    parts
}

/// The spells the player knows (`[{"id", "level", "rank", "cost", "roll", "damage_type",
/// "reach"}]`: the roll's text as the abilities menu shows it, `"healed"` for a heal's type,
/// `"called"` for a summon's count, `reach` with `range`, `length`, `radius`, `jumps`; each null
/// where none).
pub(super) fn spells_json(view: &PerceptionView<'_>) -> Value {
    use crate::content::Shape;
    let content = view.content();
    let known: Vec<Value> = (view.own_spells().into_iter())
        .filter_map(|spell| {
            let def = content.spell_of(spell)?;
            let roll = view.own_spell_roll(spell);
            let damage_type = roll.as_ref().map(|r| match r.damage_type {
                Some(kind) => content.damage_type(kind).id.clone(),
                None if def.kind == crate::content::SpellPurpose::Summon => "called".into(),
                None => "healed".into(),
            });
            let reach = view.own_spell_reach(spell).unwrap_or_default();
            let (range, length) = match def.shape {
                Shape::Cone | Shape::Line => (None, reach.range),
                _ => (reach.range, None),
            };
            Some(json!({
                "id": def.id,
                "level": def.level,
                "rank": view.own_spell_rank(spell),
                "cost": view.own_spell_cost(spell),
                "roll": roll.map(|r| r.text),
                "damage_type": damage_type,
                "reach": {
                    "range": range,
                    "length": length,
                    "radius": reach.radius,
                    "jumps": reach.jumps,
                },
            }))
        })
        .collect();
    Value::Array(known)
}
