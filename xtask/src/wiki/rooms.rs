//! Special rooms (spec §7 "Special rooms").

use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{Content, StatusId};

/// How many times as much `percent` is, in words: "twice", "two fifths", "30% of".
fn times(percent: u16) -> String {
    match percent {
        200 => "twice".into(),
        300 => "three times".into(),
        50 => "half".into(),
        40 => "two fifths".into(),
        25 => "a quarter".into(),
        1 => "a hundredth".into(),
        p => format!("{p}% of"),
    }
}

/// A share of the draws in percent, to a tenth.
fn share(part: u32, of: u32) -> String {
    let tenths = (part * 1000 + of / 2) / of.max(1);
    format!("{}.{}%", tenths / 10, tenths % 10)
}

/// The special rooms (spec §7 "Special rooms"): what each is called, what it says the first time
/// you enter it (and, for some, from outside, and on a level it covers whole), what it does
/// inside, in words, and how often it is drawn: its weight, its share of the draws where no
/// dungeon says otherwise, the chance it covers the whole level instead, and what it needs.
pub(super) fn special_rooms(content: &Content) -> String {
    let mut out = String::from(
        "| Room | The first time you enter | Inside | Weight | Share | Whole level | Only |\n\
         |---|---|---|:---:|:---:|:---:|---|\n",
    );
    let total: u32 = (content.room_effects().defs().iter())
        .map(|e| u32::from(e.weight))
        .sum();
    for e in content.room_effects().defs() {
        let mut does: Vec<String> = Vec::new();
        if e.light == 0 {
            does.push("No light at all is kept in it, carried light too.".into());
        } else if e.light < 100 {
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
        if e.swarm.is_some() {
            does.push("Full of sleeping creatures and their hoard (see Teeming dens).".into());
        }
        if let Some(status) = e.status {
            does.push(held(content, status));
        }
        if e.temperature != 0 {
            let way = if e.temperature > 0 {
                "warmer"
            } else {
                "colder"
            };
            does.push(format!(
                "It is {} °C {way} than the level around it, and nothing else.",
                e.temperature.unsigned_abs()
            ));
        }
        if e.burden != 100 {
            does.push(format!(
                "Everyone's carrying limit is {} what it is outside, so a load weighs on one {} \
                 as much.",
                times(e.burden),
                times(
                    (10_000 / u32::from(e.burden))
                        .try_into()
                        .unwrap_or(u16::MAX)
                ),
            ));
        }
        if let Some(m) = e.mirrors {
            does.push(format!(
                "Everyone inside you see is shown with {} reflections of them on free floor of \
                 the room, moving every turn and looking just like them (Mirror-wise: none \
                 within 2 cells of you). A blow or a step into one meets only glass and takes a \
                 blow's time; a missile aimed at one flies on through it. A creature outside \
                 shooting or throwing at you inside aims at you or one of your reflections, \
                 each alike; creatures inside are not fooled.",
                m.images
            ));
        }
        if e.warp > 0 {
            does.push(format!(
                "Every step inside, yours or a creature's, has a {}% chance to move the stepper \
                 to a free spot elsewhere on the level, as a teleport trap does.",
                e.warp
            ));
        }
        let mut said = format!("\"{}\"", e.enter);
        if let Some(near) = &e.near {
            let _ = write!(said, " From outside: \"{near}\"");
        }
        if let Some(pervade) = &e.pervade {
            let _ = write!(said, " Over a whole level: \"{pervade}\"");
        }
        let whole = if e.level_chance > 0 {
            format!("{}%", e.level_chance)
        } else {
            "never".into()
        };
        let mut only: Vec<String> = Vec::new();
        if e.needs_danger {
            only.push("on a level with a danger".into());
        }
        if e.needs_below {
            only.push("with a level below".into());
        }
        if e.min_cells > 0 {
            only.push(format!("in a room of {} floor cells or more", e.min_cells));
        }
        if let Some(most) = e.max_cells {
            only.push(format!("in a room of {most} floor cells or fewer"));
        }
        let only = if only.is_empty() {
            "—".to_string()
        } else {
            capital(&only.join(", "))
        };
        let _ = writeln!(
            out,
            "| {} | {said} | {} | {} | {} | {whole} | {only} |",
            capital(&e.name),
            does.join(" "),
            e.weight,
            share(u32::from(e.weight), total),
        );
    }
    out
}

/// What a room's status does to everyone inside, exactly: "Everyone inside with mana is Ebbing:
/// 1 mana drained every 20 s, none coming back; it eases 30 s after leaving."
fn held(content: &Content, status: StatusId) -> String {
    let def = content.status(status);
    let mut what: Vec<String> = Vec::new();
    if let Some(tick) = def.tick {
        if def.damage > 0 {
            what.push(format!("{} hit points lost every {tick} s", def.damage));
        }
        if def.mana_damage > 0 {
            what.push(format!("{} mana drained every {tick} s", def.mana_damage));
        }
    }
    match def.mana_regen {
        100 => {}
        0 => what.push("none coming back".into()),
        p => what.push(format!("mana back at {p}% of the pace")),
    }
    if !def.blocks.is_empty() {
        let doing: Vec<&str> = def.blocks.iter().map(|k| k.doing()).collect();
        let list = match doing.split_last() {
            Some((last, rest)) if !rest.is_empty() => format!("{}, or {last}", rest.join(", ")),
            _ => doing.join(""),
        };
        what.push(format!("unable to {}", list.replace("your ", "their ")));
    }
    let who = if def.touches_only_mana() {
        "Everyone inside with mana"
    } else {
        "Everyone inside"
    };
    let eases = def
        .eases
        .map_or(String::new(), |s| format!("; it eases {s} s after leaving"));
    format!(
        "{who}, creatures too, is {}: {}{eases}.",
        def.name,
        what.join(", ")
    )
}

/// The numbers of each teeming den (spec §7 "Special rooms": an effect with a `swarm`), by name:
/// how many sleepers, what wakes them, their cry, the hoard, and the themes.
pub(super) fn teeming_dens(content: &Content) -> String {
    let mut out = String::from(
        "| Den | Sleepers | Wakes at | Cry | Hoard | Warren | Tribe |\n\
         |---|---|:---:|:---:|---|:---:|:---:|\n",
    );
    let mut dens: Vec<_> = (content.room_effects().defs().iter())
        .filter_map(|e| e.swarm.map(|s| (e, s)))
        .collect();
    dens.sort_by(|a, b| a.0.name.cmp(&b.0.name));
    for (e, s) in dens {
        let _ = writeln!(
            out,
            "| {} | one per {} floor cells, {} to {} | {}% of the hearing threshold | {} | one \
             item per 4 sleepers (at least 1), item level +{} | {}% | {}% |",
            capital(&e.name),
            s.per_cells,
            s.min,
            s.max,
            s.wake,
            s.cry,
            s.loot_level,
            s.themes.kind,
            s.themes.faction,
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
                let mut effects: Vec<String> = (rooms.favoured.iter())
                    .map(|&(e, share)| {
                        format!("{} {share}%", capital(&content.room_effect(e).name))
                    })
                    .collect();
                effects.push(if rooms.favoured.is_empty() {
                    "Any, by weight".into()
                } else {
                    "the rest any, by weight".into()
                });
                if !rooms.banned.is_empty() {
                    let banned: Vec<String> = (rooms.banned.iter())
                        .map(|e| content.room_effect(*e).name.clone())
                        .collect();
                    effects.push(format!("never {}", banned.join(", ")));
                }
                (
                    format!("{}%", rooms.chance),
                    rooms.per_level.to_string(),
                    effects.join("; "),
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
