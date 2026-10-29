//! Starting kits (spec §8 "Character creation"): what each class starts with and what each race
//! adds, items as they are named, in the order New run offers them.

use std::fmt::Write as _;
use warlock_core::content::{Content, Keep, Kit, KitItem};
use warlock_core::equipment::{self, Equipment};
use warlock_core::magic::LootGrade;
use warlock_core::object::Stack;

/// The classes' kits, one row each.
pub(super) fn class_kits(content: &Content) -> String {
    let mut out = String::from("| Class | Starts with |\n|---|---|\n");
    for c in content.classes() {
        let _ = writeln!(out, "| {} | {} |", c.name, kit_text(content, &c.kit));
    }
    out
}

/// What each race adds to its class's kit, one row each.
pub(super) fn race_kits(content: &Content) -> String {
    let mut out = String::from("| Race | Adds |\n|---|---|\n");
    for r in &content.races().list {
        let _ = writeln!(out, "| {} | {} |", r.name, kit_text(content, &r.kit));
    }
    out
}

/// A kit in words: its lines, separated by semicolons (a choice lists its alternatives with
/// commas); "nothing" for none.
fn kit_text(content: &Content, kit: &Kit) -> String {
    if kit.is_empty() {
        return "nothing".into();
    }
    let lines: Vec<String> = kit.items().iter().map(|l| line_text(content, l)).collect();
    lines.join("; ")
}

/// One line of a kit: its item ("a dagger", "15 gold coins") or "one of" its alternatives, then
/// in parentheses where it is kept, how magic it may be, and when it comes: "a dagger
/// (wielded, with Twin Blades)", "a gold ring (on left hand, magic or rare)".
pub(super) fn line_text(content: &Content, line: &KitItem) -> String {
    let names: Vec<String> = (line.one_of.iter())
        .map(|stacks| named(content, stacks))
        .collect();
    let what = match names.as_slice() {
        [one] => one.clone(),
        [init @ .., last] => format!("one of {} or {last}", init.join(", ")),
        [] => String::new(),
    };
    let mut notes = Vec::new();
    if line.keep != Keep::Carry {
        let object = line.one_of[0][0].object;
        let slot = equipment::free_slot(content, &Equipment::default(), object);
        notes.extend(slot.map(|s| content.slot(s).mark.clone()));
    }
    notes.extend(line.magic.map(|grade| {
        match grade {
            LootGrade::Any => "perhaps magic",
            LootGrade::Magic => "magic or rare",
            LootGrade::Rare => "rare",
        }
        .to_string()
    }));
    if let Some(skill) = line.trained {
        notes.push(format!(
            "with {}",
            content.weapon_skills().skill(skill).name
        ));
    }
    if let Some(gender) = line.gender {
        notes.push(format!("if {}", gender.name().to_lowercase()));
    }
    match notes.is_empty() {
        true => what,
        false => format!("{what} ({})", notes.join(", ")),
    }
}

/// The stacks a placement makes, named as one: "a ruby", "2 daggers".
fn named(content: &Content, stacks: &[Stack]) -> String {
    let mut all = stacks[0].clone();
    all.count = stacks.iter().map(|s| s.count).sum();
    all.describe(content)
}
