//! The ground-truth layers (dev-only): `truth`'s map and lines, the header lines of `truth` and `actors`, and `rooms`.

use super::*;

/// Dev: the zones on the current level (`world/zone.rs`), one a line in the order laid:
/// `zone fire by #0 until 09:15:00, harm 4: (12, 5) (13, 5)`.
pub(super) fn zone_rows(world: &World) -> Vec<String> {
    (world.zones().iter())
        .map(|z| {
            let until = z.until();
            let cells: Vec<String> = z.cells().map(|p| p.to_string()).collect();
            let harm = match z.harm() {
                0 => String::new(),
                harm => format!(", harm {harm}"),
            };
            format!(
                "zone {} by #{} until {:02}:{:02}:{:02}{harm}: {}",
                z.kind().id(),
                z.by().0,
                until.hour(),
                until.minute(),
                until.second(),
                cells.join(" ")
            )
        })
        .collect()
}

/// Dev: the header of the `truth` and `actors` layers on a level with a danger rule (spec §7
/// "Danger levels"): `danger 3 population 2/6 spawn next 08:15:00`, its creatures hostile to
/// the player against its cap; none elsewhere.
pub(super) fn spawn_header(world: &World) -> Vec<String> {
    let Some((danger, hostiles, cap, next)) = world.spawn_status() else {
        return Vec::new();
    };
    let next = format!(
        "{:02}:{:02}:{:02}",
        next.hour(),
        next.minute(),
        next.second()
    );
    vec![format!(
        "danger {danger} population {hostiles}/{cap} spawn next {next}"
    )]
}

/// Dev: the header of the `truth` layer on the overworld where encounters are rolled (spec §7
/// "Overworld encounters"): `encounter rate here 300 bp/h, tables: roads, vale`, the rate with
/// the phase of the day and the matching tables' percents; none elsewhere.
pub(super) fn encounter_header(world: &World) -> Vec<String> {
    let Some(rate) = world.encounter_rate_here() else {
        return Vec::new();
    };
    let tables = world.encounter_tables_here();
    let tables = if tables.is_empty() {
        "none".to_string()
    } else {
        tables.join(", ")
    };
    vec![format!("encounter rate here {rate} bp/h, tables: {tables}")]
}

/// Dev: the risings due on the current level (spec §7 "Special rooms": a cursed room), one a
/// line in the order they died: `rising goblin at (12, 5) as zombie at 08:15:00`.
pub(super) fn rising_rows(world: &World) -> Vec<String> {
    let content = world.content();
    (world.risings().iter())
        .map(|r| {
            let at = r.at();
            format!(
                "rising {} at {} as {} at {:02}:{:02}:{:02}",
                content.creature(r.dead()).id,
                r.pos(),
                content.creature(r.risen()).id,
                at.hour(),
                at.minute(),
                at.second()
            )
        })
        .collect()
}

pub(super) fn truth_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (level, actors) = world.ground_truth();
    let actors: Vec<_> = actors.collect();
    rows(level.width(), level.height(), |p| {
        match (
            actors.iter().find(|a| a.pos == p),
            level.trap_at(p),
            level.objects_at(p).first(),
        ) {
            (Some(actor), _, _) => content.creature(actor.creature).glyph,
            (None, Some(trap), _) => content.trap(trap.kind).glyph,
            (None, None, Some(top)) => content.object(top.object).glyph,
            (None, None, None) if level.secret_at(p).is_some() => 'S',
            // A sealed room's breach while it stands (spec §7 "Special rooms").
            (None, None, None) if level.breach_at(content, p) => 'H',
            // A hidden location by its entrance's glyph (spec §7 "Hidden locations").
            (None, None, None) => (level.hidden_at(p).map(|h| h.terrain))
                .or(level.terrain_at(p))
                .map_or(' ', |t| content.terrain(t).glyph),
        }
    })
}

/// Dev: the hallowed ground on the current level (Sanctuary; `world/sanctuary.rs`), one a line
/// in the order laid: `hallow at (12, 5) r2 until 09:15:00` (`, dark` for dreaded ground).
pub(super) fn hallow_rows(world: &World) -> Vec<String> {
    (world.hallows().iter())
        .map(|h| {
            let until = h.until();
            format!(
                "hallow at {} r{} until {:02}:{:02}:{:02}{}",
                h.at(),
                h.radius(),
                until.hour(),
                until.minute(),
                until.second(),
                if h.dark() { ", dark" } else { "" }
            )
        })
        .collect()
}

/// Dev: the chests on the current level (spec §8 "Chests"; ground truth), one a line by
/// position: `chest 7 (hutch) at (12, 5): shut, locked, trapped (arrow), holds 3 stacks`.
pub(super) fn chest_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (level, _) = world.ground_truth();
    (level.chests().iter())
        .map(|(&p, chest)| {
            let band = (content.chests().band_of(chest.level)).map_or("?", |b| b.id.as_str());
            let state = (level.terrain_at(p))
                .and_then(|t| content.terrain(t).chest)
                .map_or("gone", |s| s.id());
            let locked = if level.lock_at(p).is_some() {
                ", locked"
            } else {
                ""
            };
            let trapped = match level.trap_at(p) {
                Some(trap) => format!(", trapped ({})", content.trap(trap.kind).id),
                None => String::new(),
            };
            let holds = match chest.holds.len() {
                0 => String::new(),
                1 => ", holds 1 stack".to_string(),
                n => format!(", holds {n} stacks"),
            };
            format!(
                "chest {} ({band}) at {p}: {state}{locked}{trapped}{holds}",
                chest.level
            )
        })
        .collect()
}

/// A line per statue of the stoned on the level (ground truth): what it was and how much it
/// holds, `statue (basilisk) at (12, 5): holds 2 stacks`.
pub(super) fn statue_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (level, _) = world.ground_truth();
    (level.statues().iter())
        .map(|(&p, statue)| {
            let holds = match statue.holds.len() {
                0 => "empty".to_string(),
                1 => "holds 1 stack".to_string(),
                n => format!("holds {n} stacks"),
            };
            // A name from the graveyard (`world/graveyard.rs`): `statue (kobold, Brakka the
            // dwarf) at (12, 5): empty`.
            let name = (statue.name.as_ref())
                .map_or_else(String::new, |f| format!(", {}", f.words(content)));
            format!(
                "statue ({}{name}) at {p}: {holds}",
                content.creature(statue.of).id
            )
        })
        .collect()
}

/// Dev: a line per Pit the run keeps a record of, wherever the player is (spec §7 "The Pits";
/// `world/pit.rs`): `pit 7 from depths:3: open` (`sealed` once its warden is dead).
pub(super) fn pit_rows(world: &World) -> Vec<String> {
    (world.state().pits().iter())
        .map(|(s, pit)| {
            let state = if pit.sealed() { "sealed" } else { "open" };
            format!("pit {s} from {}: {state}", pit.from())
        })
        .collect()
}

/// A line per gravestone on the level with what is cut in it (spec §7 "Graves"; ground truth
/// for every stone, read or not): `grave at (12, 5): Here lies Ælfric Wudu, 1381–1427.`
pub(super) fn grave_rows(world: &World) -> Vec<String> {
    let (level, _) = world.ground_truth();
    (level.graves().iter())
        .map(|(p, text)| format!("grave at {p}: {text}"))
        .collect()
}

/// A line per kind of leaving on the level (ground truth, `content/leaving.rs`) with its cells,
/// `leavings web: (12, 5), (13, 5)`: the statues, webs, and acid-pitted floor by terrain, torn
/// cloth by the piles that hold it.
pub(super) fn leaving_rows(world: &World) -> Vec<String> {
    use crate::content::Leaving;
    let content = world.content();
    let (level, _) = world.ground_truth();
    let mut by: std::collections::BTreeMap<Leaving, Vec<String>> = Default::default();
    for y in 0..level.height() {
        for x in 0..level.width() {
            let p = Pos::new(x, y);
            let terrain = level.terrain_at(p).and_then(|t| content.leaving_terrain(t));
            if let Some(leaving) = terrain {
                by.entry(leaving).or_default().push(p.to_string());
            }
            if (level.objects_at(p).iter()).any(|s| content.is_leaving_object(s.object)) {
                by.entry(Leaving::TornCloth)
                    .or_default()
                    .push(p.to_string());
            }
        }
    }
    (by.into_iter())
        .map(|(leaving, cells)| format!("leavings {}: {}", leaving.id(), cells.join(", ")))
        .collect()
}

/// The level's special rooms (ground truth): the terrain with each special room's plain ground
/// ([`crate::content::TerrainDef::is_plain_ground`]) drawn as its effect's mark, then one line
/// per room (glimpsed, entered, a hoard's stacks lifted and whether it sealed, a sealed room's
/// breach); headed by `pervading <id>` when an effect covers the level.
pub(super) fn room_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (level, _) = world.ground_truth();
    let mut out = rows(level.width(), level.height(), |p| {
        let Some(t) = level.terrain_at(p) else {
            return ' ';
        };
        let room = level.rooms().iter().find(|r| r.contains(p));
        match room {
            Some(room) if content.terrain(t).is_plain_ground() => {
                content.room_effect(room.effect).mark
            }
            _ => content.terrain(t).glyph,
        }
    });
    if let Some(id) = level.pervading() {
        out.insert(0, format!("pervading {}", content.room_effect(id).id));
    }
    for room in level.rooms() {
        let effect = content.room_effect(room.effect);
        let glimpsed = if room.glimpsed { ", glimpsed" } else { "" };
        let entered = if room.entered { ", entered" } else { "" };
        let side = room
            .side
            .map_or(String::new(), |s| format!(", side {}", s.id()));
        let lifted = match room.lifted {
            0 => String::new(),
            n => format!(", lifted {n}"),
        };
        let sealed = if room.sealed { ", sealed" } else { "" };
        let breach = (room.breach).map_or(String::new(), |b| format!(", breach {b}"));
        out.push(format!(
            "{} {} at {}-{}{side}{}{glimpsed}{entered}{lifted}{sealed}{breach}",
            effect.mark,
            effect.name,
            room.min,
            room.max,
            feature_state(level, content, room)
        ));
    }
    out
}

/// What became of a special room's feature, for the `rooms` layer: `, spent` once its cell is
/// the spent terrain of the table it offers, `, destroyed` once it is neither (an altar blown
/// apart); nothing while it stands, or for a room without one.
pub(super) fn feature_state(
    level: &LevelState,
    content: &Content,
    room: &SpecialRoom,
) -> &'static str {
    let effect = content.room_effect(room.effect);
    let (Some(feature), Some((_, table))) = (effect.feature, effect.offers.as_ref()) else {
        return "";
    };
    let spent = (table.as_deref())
        .and_then(|t| content.outcomes().get(t))
        .and_then(|t| t.spent);
    let cells = room.cells();
    let terrains: Vec<_> = cells.iter().filter_map(|&p| level.terrain_at(p)).collect();
    if terrains.contains(&feature) {
        ""
    } else if spent.is_some_and(|s| terrains.contains(&s)) {
        ", spent"
    } else {
        ", destroyed"
    }
}
