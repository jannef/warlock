//! Special rooms (spec §7 "Special rooms").

use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{Content, Hoard, StatusId};
use warlock_core::world::rise_candidates;

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
            let name = content.terrain(floor).name.to_lowercase();
            does.push(if e.collapse.is_some() {
                format!(
                    "Its middle is {name}; the floor beside its walls and doors is sound, so \
                     there is always a way round."
                )
            } else {
                format!("Its floor is {name}.")
            });
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
        if let Some(d) = e.dressing {
            does.push(format!(
                "{} stand along its walls, one per {} floor cells (1 to {}).",
                capital(&content.plural(&content.terrain(d.terrain).name)),
                d.per_cells,
                d.most
            ));
        }
        if let Some(feature) = e.feature {
            let offers = match &e.offers {
                Some((_, _)) => ": pray beside it (see Forgotten altars)",
                None => "",
            };
            does.push(format!(
                "A {} stands in it{offers}.",
                content.terrain(feature).name.to_lowercase()
            ));
        }
        if let Some(shrine) = &e.shrine {
            does.push(shrine_does(content, shrine));
        }
        if let Some(c) = e.collapse {
            does.push(format!(
                "Every step of yours onto its cracked floor has a {}% chance ({}% while you are \
                 Burdened or worse) to give way: you fall to the level below, landing somewhere \
                 open on it (never behind a locked door, never in a shop), take {} damage (it can \
                 kill you), and are Staggered. Creatures never fall (Light tread: \
                 you never do).",
                c.chance,
                (u32::from(c.chance) * 2).min(100),
                c.damage
            ));
        }
        if e.warp > 0 {
            does.push(format!(
                "Every step inside, yours or a creature's, has a {}% chance to move the stepper \
                 to a free spot elsewhere on the level, as a teleport trap does.",
                e.warp
            ));
        }
        if let Some(h) = e.hoard {
            let [least, most] = h.guardians;
            does.push(format!(
                "{} items lie on its floor, {} item levels above the level's. The first stack \
                 you lift inside seals it: every door of it shuts (unless something is in the \
                 doorway) and locks with a lock of the level's depth that only a skeleton key \
                 opens, and {least} to {most} creatures hostile to you of the level's danger + \
                 {} come on its edge, out of your sight where they can (Hoard-wise: the second \
                 stack). It seals once.",
                h.items, h.loot_level, h.danger
            ));
        }
        if let Some(rise) = &e.rise {
            does.push(format!(
                "A creature that dies in it (not one of the {}) rises from its corpse {} later \
                 as one of them, unless the corpse is gone by then (see Cursed rooms).",
                faction_name(content, &rise.faction),
                rise.after.spoken()
            ));
        }
        if e.unholy {
            does.push("Consecrate lifts it for good.".into());
        }
        let mut said = match &e.shrine {
            Some(s) => format!(
                "Of its side: \"{}\" Of the other: \"{}\" Neutral: \"{}\"",
                s.welcome, s.unwelcome, e.enter
            ),
            None => format!("\"{}\"", e.enter),
        };
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
        if e.swarm.as_ref().is_some_and(|s| !s.kinds.is_empty()) {
            only.push("where one of its own creatures can be met at the level's danger".into());
        }
        if let Some(s) = e.swarm.as_ref().filter(|s| !s.factions.is_empty()) {
            let names: Vec<_> = (s.factions.iter())
                .filter_map(|f| content.factions().id(f))
                .map(|f| content.factions().def(f).name.clone())
                .collect();
            only.push(format!(
                "in a dungeon whose own creatures include {}",
                names.join(" or ")
            ));
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

/// What a shrine of a side does, exactly: "It is good or evil, one to one, ..."
fn shrine_does(content: &Content, shrine: &warlock_core::content::Shrine) -> String {
    let def = content.status(shrine.status);
    let c = def.combat;
    let mut hinders = vec![format!("wounds heal at {}% of the pace", def.regen)];
    if c.to_hit != 0 {
        hinders.push(format!("{:+} to hit", c.to_hit));
    }
    let eases = def
        .eases
        .map_or(String::new(), |s| format!("; it eases {s} s after leaving"));
    format!(
        "It is good or evil, one to one, and only its stone's light tells which (gold good, red \
         evil). Everyone inside, creatures too, is treated by their own side: one of its side \
         heals {} as fast; one of the other side is {}: {}{eases}; a neutral one, nothing.",
        times(shrine.regen),
        def.name,
        hinders.join(", ")
    )
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
/// how many sleepers and of what, what wakes them, their cry, the hoard, and the themes.
pub(super) fn teeming_dens(content: &Content) -> String {
    let mut out = String::from(
        "| Den | Sleepers | Of | Wakes at | Cry | Hoard | Warren | Tribe |\n\
         |---|---|---|:---:|:---:|---|:---:|:---:|\n",
    );
    let mut dens: Vec<_> = (content.room_effects().defs().iter())
        .filter_map(|e| e.swarm.as_ref().map(|s| (e, s)))
        .collect();
    dens.sort_by(|a, b| a.0.name.cmp(&b.0.name));
    for (e, s) in dens {
        let of = if !s.kinds.is_empty() {
            let mut names: Vec<String> = (s.kinds.iter())
                .map(|&c| content.creature(c).name.clone())
                .collect();
            names.sort();
            format!("its own: {}", names.join(", "))
        } else if s.factions.is_empty() {
            "any type hostile to you".to_string()
        } else {
            let names: Vec<_> = (s.factions.iter())
                .filter_map(|f| content.factions().id(f))
                .map(|f| content.factions().def(f).name.clone())
                .collect();
            format!("{}, hostile or not", names.join(", "))
        };
        let of = if s.danger > 0 {
            format!("{of}; drawn {} dangers deeper than the level", s.danger)
        } else {
            of
        };
        let hoard = match s.hoard {
            Hoard::Items => format!(
                "one item per 4 sleepers (at least 1), item level +{}",
                s.loot_level
            ),
            Hoard::Corpses => "per 4 sleepers (at least 1), a pile of bones and a fresh corpse \
                               of a creature of the branch"
                .to_string(),
        };
        let _ = writeln!(
            out,
            "| {} | one per {} floor cells, {} to {} | {of} | {}% of the hearing threshold | {} \
             | {hoard} | {}% | {}% |",
            capital(&e.name),
            s.per_cells,
            s.min,
            s.max,
            s.wake,
            s.cry,
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

/// A faction's name by its id, lower case ("undead").
fn faction_name(content: &Content, id: &str) -> String {
    let factions = content.factions();
    (factions.id(id)).map_or_else(|| id.to_string(), |f| factions.def(f).name.to_lowercase())
}

/// What the dead rise as in a cursed room (spec §7 "Special rooms": an effect with a `rise`), by
/// the dead's level: the shared types of the rise's faction of that level, else of the nearest
/// level (the lower on a tie), each alike; levels that rise alike share a row.
pub(super) fn risings(content: &Content) -> String {
    let Some(rise) = (content.room_effects().defs().iter()).find_map(|e| e.rise.as_ref()) else {
        return String::new();
    };
    let Some(faction) = content.factions().id(&rise.faction) else {
        return String::new();
    };
    let top = (content.creature_defs().iter())
        .filter(|d| d.corpse && !d.unique)
        .map(|d| d.level)
        .max()
        .unwrap_or(1);
    let mut rows: Vec<(u8, u8, Vec<String>)> = Vec::new();
    for level in 1..=top {
        let mut names: Vec<String> = (rise_candidates(content, faction, "", level).into_iter())
            .map(|c| content.creature(c).name.clone())
            .collect();
        names.sort();
        match rows.last_mut() {
            Some((_, to, last)) if *last == names => *to = level,
            _ => rows.push((level, level, names)),
        }
    }
    let mut out = String::from("| The dead's level | Rise as (each alike) |\n|:---:|---|\n");
    for (from, to, names) in rows {
        let levels = if from == to {
            from.to_string()
        } else {
            format!("{from}–{to}")
        };
        let _ = writeln!(out, "| {levels} | {} |", names.join(", "));
    }
    out
}
