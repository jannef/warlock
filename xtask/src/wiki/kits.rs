//! Starting kits (spec §8 "Character creation"): what each class starts with and what each race
//! adds, items as they are named: the classes in the wiki's order, the races in the order New run
//! offers them.

use std::fmt::Write as _;
use warlock_core::content::{ClassDef, Content, Keep, Kit, KitItem};
use warlock_core::equipment::{self, Equipment};
use warlock_core::magic::LootGrade;
use warlock_core::object::Stack;
use warlock_core::text::or_list;

/// The classes' kits, one row each, in the wiki's order of the classes.
pub(super) fn class_kits(content: &Content) -> String {
    let mut out = String::from("| Class | Starts with |\n|---|---|\n");
    for c in super::class_guide::ordered(content) {
        let _ = writeln!(out, "| {} | {} |", c.name, class_kit(content, c));
    }
    out
}

/// One class's kit in words, its random spell scrolls too.
pub(super) fn class_kit(content: &Content, c: &ClassDef) -> String {
    let mut text = kit_text(content, &c.kit);
    if let Some(random) = &c.random_scrolls {
        let _ = write!(
            text,
            "; {} random [spell scrolls](scrolls.md) (each the scroll of one of the {} spells of \
             [spell level](magic.md#spell-levels) {} or below, as likely as any other)",
            random.count,
            random.scrolls.len(),
            highest_level(content, &random.scrolls),
        );
    }
    text
}

/// The highest spell level among the spells `scrolls` cast.
fn highest_level(content: &Content, scrolls: &[warlock_core::content::ObjectId]) -> u8 {
    (scrolls.iter())
        .filter_map(|&o| match &content.object(o).read {
            Some(warlock_core::content::Reading::Cast(cast)) => {
                Some(content.spell(cast.spell).level)
            }
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

/// What each race adds to its class's kit, one row each, and the slots it leaves bare.
pub(super) fn race_kits(content: &Content) -> String {
    let mut out = String::from("| Race | Adds | Bare |\n|---|---|---|\n");
    for r in &content.races().list {
        let bare: Vec<String> = (r.bare.iter())
            .map(|&s| content.slot(s).name.to_lowercase())
            .collect();
        let bare = match bare.is_empty() {
            true => "—".to_string(),
            false => bare.join(", "),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {bare} |",
            r.name,
            kit_text(content, &r.kit)
        );
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
        [] => String::new(),
        [one] => one.clone(),
        many => format!("one of {}", or_list(many)),
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
    if line.fill {
        notes.push("if the slot is free".into());
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
