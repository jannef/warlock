//! Special rooms (spec §7 "Special rooms").

use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::Content;

/// How many times as much `percent` is, in words: "twice", "two fifths", "30% of".
fn times(percent: u16) -> String {
    match percent {
        200 => "twice".into(),
        300 => "three times".into(),
        50 => "half".into(),
        40 => "two fifths".into(),
        25 => "a quarter".into(),
        p => format!("{p}% of"),
    }
}

/// The special rooms (spec §7 "Special rooms"): what each is called, what it says the first time
/// you enter it, and what it does inside, in words.
pub(super) fn special_rooms(content: &Content) -> String {
    let mut out = String::from("| Room | The first time you enter | Inside |\n|---|---|---|\n");
    for e in content.room_effects().defs() {
        let mut does: Vec<String> = Vec::new();
        if e.light < 100 {
            does.push(format!(
                "All light is drained to {}% of what it would be.",
                e.light
            ));
        }
        if !e.glow.is_dark() {
            let faint = if e.glow.luma() < 80 { "faint " } else { "" };
            let warm = if e.glow.rgb[0] > e.glow.rgb[2] {
                "warm "
            } else {
                ""
            };
            does.push(format!("A {faint}{warm}glow lights it."));
        }
        if e.noise != 100 {
            let farther = if e.noise > 100 {
                ", and so carries farther"
            } else {
                ""
            };
            does.push(format!(
                "Every noise made in it is {} as loud{farther}.",
                times(e.noise)
            ));
        }
        if e.regen != 100 {
            does.push(format!("Wounds heal {} as fast.", times(e.regen)));
        }
        if e.mana_regen != 100 {
            does.push(format!("Mana comes back {} as fast.", times(e.mana_regen)));
        }
        if e.shunned {
            does.push("Creatures hostile to you do not step in.".into());
        }
        if let Some(floor) = e.floor {
            does.push(format!(
                "Its floor is {}.",
                content.terrain(floor).name.to_lowercase()
            ));
        }
        let _ = writeln!(
            out,
            "| {} | \"{}\" | {} |",
            capital(&e.name),
            e.enter,
            does.join(" ")
        );
    }
    out
}

/// Each branch's special-room and shop odds (spec §7 "Special rooms", "Shops"), its depths with
/// the same odds in one row, branches by name: the chance a room is special, how many a level
/// may get, which rooms, the chance a rooms-and-corridors level has a shop (if it has a room
/// that fits), its kinds, and who keeps it.
pub(super) fn branch_odds(content: &Content) -> String {
    let mut out = String::from(
        "| Levels | A room is special | At most a level | Special rooms | A shop | Shops | Kept by |\n\
         |---|:---:|:---:|---|:---:|---|---|\n",
    );
    let none = || ("never".to_string(), "—".to_string(), "—".to_string());
    let mut branches: Vec<_> = content.dungeons().collect();
    branches.sort_by(|a, b| a.name.cmp(&b.name));
    for def in branches {
        let odds = |depth| (def.room_odds(depth), def.shop_odds(depth));
        let mut from = 1;
        for depth in 1..=def.depth {
            if depth < def.depth && odds(depth + 1) == odds(depth) {
                continue;
            }
            let levels = if from == depth {
                def.level_name(depth)
            } else {
                format!("{} {from}–{depth}", def.name)
            };
            from = depth + 1;
            let (Some(rooms), shops) = odds(depth) else {
                continue;
            };
            let (chance, most, effects) = if rooms.generated() {
                let effects: Vec<String> = (rooms.effects.iter())
                    .map(|e| capital(&content.room_effect(*e).name))
                    .collect();
                (
                    format!("{}%", rooms.chance),
                    rooms.per_level.to_string(),
                    effects.join(", "),
                )
            } else {
                none()
            };
            let (shop, kinds, keeper) = match (shops, content.shops()) {
                (Some(s), Some(all)) if s.generated() => {
                    let kinds = if s.kinds.len() == all.defs().len() {
                        "Any".to_string()
                    } else {
                        (s.kinds.iter())
                            .map(|k| all.get(*k).name.clone())
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    let keeper = capital(&content.creature(s.keeper).name);
                    (format!("{}%", s.chance), kinds, keeper)
                }
                _ => none(),
            };
            let _ = writeln!(
                out,
                "| {levels} | {chance} | {most} | {effects} | {shop} | {kinds} | {keeper} |"
            );
        }
    }
    out
}
