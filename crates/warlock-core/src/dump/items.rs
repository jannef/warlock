//! The `inventory` and `equipment` layers: what the player carries, wears, and wields.

use super::*;

/// The player's inventory grouped by item class as the screens group it, a header line a class
/// (`Melee weapons:`), each stack by its position as action text names it (`a - a dagger (450
/// g)`, `b - a leather jerkin (being worn) (3.5 kg)`, `26 - a rock (1 kg)`), then what they
/// carry and weigh.
/// What `stack` gives in numbers, as the player knows it (`magic::numbers`), for JSON:
/// `{"to_hit": 0, "damage": "1d8+2", "range": null, "dv": 1, "pv": 1}` (ammunition's damage is
/// what it adds, `+1d4 fire`, or empty), each part null where it
/// has none; null for an item that gives nothing.
pub(super) fn numbers_json(stack: &Stack, content: &Content) -> Value {
    let numbers = crate::magic::Numbers::of(stack, content);
    if numbers.is_empty() {
        return Value::Null;
    }
    let attack = numbers.attack.as_ref();
    let defence = numbers.defence;
    json!({
        "to_hit": attack.map(|a| a.to_hit),
        "damage": attack.map(|a| a.damage_text(content)),
        "range": attack.and_then(|a| a.range),
        "dv": defence.map(|d| d.dv),
        "pv": defence.and_then(|d| d.pv),
    })
}

/// How far the player throws inventory stack `item`, a flask whose type they know.
pub(super) fn known_flask_range(view: &PerceptionView<'_>, item: usize) -> Option<i32> {
    let stack = view.inventory().get(item)?;
    let known = view.lore().is_identified(stack.object);
    let flask = view.content().object(stack.object).flask.is_some();
    (known && flask).then(|| view.throw_range(item)).flatten()
}

pub(super) fn inventory_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let unpaid = view.unpaid();
    let inventory = view.inventory();
    let mut rows: Vec<String> = Vec::new();
    let mut class = None;
    for i in view.by_class(inventory) {
        let s = &inventory[i];
        let here = view.item_class(s);
        if class != Some(here) {
            class = Some(here);
            rows.push(format!("{}:", content.item_class(here).name));
        }
        let mut name = view.lore().listed(s);
        // A wand's rung once its type is known (the spell scaling design §7.4).
        if let Some(rung) = s.known_wand_rung(view.lore()) {
            name += &format!(" (rung {rung})");
        }
        // A flask's throw once its type is known (spec §8 "Potions": "Flasks").
        if let Some(range) = known_flask_range(view, i) {
            name += &format!(" (throw {range})");
        }
        let weight = format_weight(s.weight(content));
        let mark = equipment::mark(content, view.equipment(), i)
            .map_or(String::new(), |m| format!(" ({m})"));
        let owed = (unpaid.get(i).copied().flatten())
            .map_or(String::new(), |p| format!(" (unpaid, {p} gold)"));
        // Last, the affixes it holds against its capacity, as they truly are
        // (docs/dev/enchanting.md).
        let holds = s.holds_text(content).unwrap_or_default();
        rows.push(format!(
            "{} - {name}{mark}{owed} ({weight}){holds}",
            position_text(i)
        ));
    }
    let weight = view.own_weight();
    let load = view.own_load();
    rows.push(format!(
        "carried {} ({}: {}), total weight {}",
        format_weight(weight.carried),
        load.burden().id(),
        load.over().join(", "),
        format_weight(weight.total())
    ));
    rows
}

/// The player's equipment, one slot a line by its letter: `c Body: a leather jerkin (b)` (the
/// inventory position in parentheses, as action text names it), `a Head: -` when empty, `a Head:
/// - (horns)` when a talent bars it.
pub(super) fn equipment_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let worn = view.equipment();
    (view.own_slots().into_iter())
        .enumerate()
        .map(|(i, (slot, def))| {
            let what = match worn.get(slot) {
                Some(item) => view.inventory().get(item).map_or("?".into(), |s| {
                    format!("{} ({})", view.lore().describe(s), position_text(item))
                }),
                // A slot the body bars: `a Head: - (horns)`.
                None => match view.barred(slot) {
                    Some(mark) => format!("- ({mark})"),
                    None => "-".into(),
                },
            };
            format!("{} {}: {what}", letter(i).unwrap_or('?'), def.name)
        })
        .collect()
}
