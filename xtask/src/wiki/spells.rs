//! Spells (spec §8 "Spells"): the shapes' and resolutions' cost multipliers, the classes'
//! casting percents, the spell list by spell level, the classes' guaranteed spells, the armour
//! that makes casting dearer, and the wands.

use super::text::{Term, anchored, damage_link, link};
use std::fmt::Write as _;
use warlock_core::content::{
    BurstIce, Content, Resolution, RodDoes, RodShape, Shape, SpellData, SpellPurpose, TalentEffect,
};

/// What each shape multiplies the base cost by; a burst by radius.
pub(super) fn shapes(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut out = String::from("| Shape | Area | Cost |\n|---|:---:|:---:|\n");
    for &shape in Shape::ALL {
        let area = if shape.is_area() { "yes" } else { "no" };
        if shape == Shape::Burst {
            for radius in 1..=rules.max_radius() {
                let cost = rules.shape_cost(shape, radius);
                let _ = writeln!(
                    out,
                    "| burst, radius {radius} | {area} | ×{} |",
                    times(cost)
                );
            }
        } else {
            let cost = rules.shape_cost(shape, 0);
            let _ = writeln!(out, "| {} | {area} | ×{} |", shape.id(), times(cost));
        }
    }
    out
}

/// What every spell of a damage type does beyond its damage (`[types]` in `data/spells.toml`), by
/// the type's name.
pub(super) fn types(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut rows: Vec<(String, String, String, String)> = (rules.types.iter())
        .map(|(&kind, rule)| {
            let status = |s| {
                let def = content.status(s);
                link(Term::Status, &def.id, &def.name)
            };
            let mut harmed = Vec::new();
            if let Some(rider) = rule.rider {
                let mut text = format!("{} {}", status(rider.status), rider.severity);
                if rider.chance < 100 {
                    let _ = write!(text, ", {} % of the time", rider.chance);
                }
                harmed.push(text);
            }
            harmed.extend(rule.ends.iter().map(|&s| format!("ends {}", status(s))));
            if rule.shove {
                harmed.push(
                    "a [big hit](combat.md#damage) or a critical knocks it a cell away".to_string(),
                );
            }
            if let Some(bite) = rule.bite {
                harmed.push(format!(
                    "{} % of the harm against one below {} % of its hit points",
                    bite.percent, bite.below_percent
                ));
            }
            for tag in &rule.immune_tags {
                harmed.push(format!("{tag} creatures are immune"));
            }
            let mut cells = Vec::new();
            match rule.ice {
                Some(BurstIce::Melts) => cells.push("melts ice".to_string()),
                Some(BurstIce::Freezes(um)) => {
                    cells.push(format!("freezes water, {} cm thick", um / 10_000));
                }
                None => {}
            }
            if rule.douses {
                cells.push("puts out fires".into());
            }
            if rule.shatter {
                cells.push("a blast shatters ice thinner than 1 cm per 100 of the blast".into());
            }
            if rule.doors {
                cells.push("a blast breaks shut doors, locked and trapped ones too".into());
            }
            let left = rule.fire.map_or_else(String::new, |(_, burns)| {
                format!("a fire on its centre, {burns} s")
            });
            (
                damage_link(content, kind),
                harmed.join("; "),
                cells.join("; "),
                left,
            )
        })
        .collect();
    rows.sort();
    let mut out = String::from(
        "| Type | On each creature harmed | On the cells it reaches | A burst leaves |\n\
         |---|---|---|---|\n",
    );
    for (name, harmed, cells, left) in rows {
        let _ = writeln!(out, "| {name} | {harmed} | {cells} | {left} |");
    }
    out
}

/// Each resolution's roll and cost.
pub(super) fn resolutions(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut out = String::from("| Resolution | Roll | Cost |\n|---|---|:---:|\n");
    for &resolution in Resolution::ALL {
        let ratio = rules.ratio(resolution);
        let cost = if resolution == Resolution::Area {
            "the shape's".to_string()
        } else {
            format!("×{}", times(ratio.cost))
        };
        let roll = match resolution {
            Resolution::Attack => {
                "2d12 plus the aim stat's modifier against Defence; may critical, the damage \
                 rolled again"
            }
            Resolution::Touch => {
                "as attack, against one creature next to you; may critical, the damage rolled \
                 again"
            }
            Resolution::Auto => "none: it always hits, never a critical",
            Resolution::Area => {
                "none: one roll for everyone in it, half beyond its radius − 1; Evasion halves it"
            }
        };
        let _ = writeln!(out, "| {} | {roll} | {cost} |", resolution.id());
    }
    out
}

/// Each class's casting percent, most first, then by name.
pub(super) fn casting(content: &Content) -> String {
    let mut classes: Vec<(&str, u8)> = (content.classes().iter())
        .map(|c| (c.name.as_str(), c.casting))
        .collect();
    classes.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let mut out = String::from("| Class | Casting |\n|---|:---:|\n");
    for (name, casting) in classes {
        let _ = writeln!(out, "| {name} | {casting} % |");
    }
    out
}

/// Every spell, by spell level and then name: its school, shape and resolution, its base reach,
/// what its base does, its ladders in words (what it grows with), its mana (no armour, no
/// talents; whoever casts it), and its time in casts.
pub(super) fn list(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut spells: Vec<&SpellData> = content.spells().iter().collect();
    spells.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
    let mut out = String::from(
        "| Spell | Level | School | Shape and resolution | Reach | What it does | Grows with | Mana \
         | Casts |\n|---|:---:|---|---|---|---|---|:---:|:---:|\n",
    );
    for spell in spells {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {}, {} | {} | {} | {} | {} | {} |",
            anchored(Term::Spell, &spell.id, &spell.name),
            spell.level,
            spell.school.id(),
            spell.shape.id(),
            spell.resolution.id(),
            reach(spell),
            does(content, spell),
            grows_with(spell),
            rules.mana(spell, 0),
            spell.casts,
        );
    }
    out
}

/// The spell circles' statuses (`data/circles.toml`): each buff's and hex's spell, the status it
/// marks, and its school's colour; buffs first, then hexes, each by spell name.
pub(super) fn circles(content: &Content) -> String {
    let mut rows: Vec<(bool, &SpellData, &warlock_core::content::CircledStatus)> = (content
        .spells())
    .iter()
    .filter(|s| matches!(s.kind, SpellPurpose::Buff | SpellPurpose::Hex))
    .filter_map(|s| {
        let (status, _) = s.effect.status?;
        Some((
            s.kind == SpellPurpose::Hex,
            s,
            content.circles().of(status)?,
        ))
    })
    .collect();
    rows.sort_by(|a, b| (a.0, &a.1.name).cmp(&(b.0, &b.1.name)));
    let mut out =
        String::from("| Spell | Kind | Status | School | Colour |\n|---|---|---|---|---|\n");
    for (hex, spell, row) in rows {
        let status = content.status(row.status);
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | `{}` |",
            link(Term::Spell, &spell.id, &spell.name),
            if hex { "hex" } else { "buff" },
            link(Term::Status, &status.id, &status.name),
            link(Term::School, row.school.id(), row.school.id()),
            content.circles().color(row.school),
        );
    }
    out
}

/// Each class's guaranteed spells (its level table's spell abilities), by class name: each
/// spell with its spell level and the level it comes at, lowest first; every other spell is learned
/// from scrolls.
pub(super) fn class_spells(content: &Content) -> String {
    let book = content.talents();
    let mut classes: Vec<_> = content.classes().iter().collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from(
        "| Class | Casting | Guaranteed spells (spell level, from level) |\n|---|:---:|---|\n",
    );
    for class in classes {
        let mut given: Vec<(u8, &str, u8)> = Vec::new();
        for (level, talent) in &class.abilities {
            let Some(id) = book.id(talent) else {
                continue;
            };
            for effect in &book.get(id).effects {
                let TalentEffect::Spell { spell } = effect else {
                    continue;
                };
                if let Some(s) = content.spell_id(spell).map(|s| content.spell(s)) {
                    given.push((*level, s.name.as_str(), s.level));
                }
            }
        }
        let spells = if given.is_empty() {
            "none".to_string()
        } else {
            (given.iter())
                .map(|(from, name, level)| format!("{name} (level {level}, from {from})"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(out, "| {} | {} % | {spells} |", class.name, class.casting);
    }
    out
}

/// What `spell` grows with: each of its ladders as written, in words ("Intelligence 13: 3d6,
/// 16: 4d6; range Willpower 14: 11, 18: 12"), the roll's first and unnamed, the others by their
/// dimension; "—" without any.
pub(super) fn grows_with(spell: &SpellData) -> String {
    let text: Vec<String> = (spell.grows.named().into_iter())
        .map(|(dim, ladder)| match dim {
            "damage" | "heal" => ladder.words(),
            other => format!("{other} {}", ladder.words()),
        })
        .collect();
    if text.is_empty() {
        "—".into()
    } else {
        text.join("; ")
    }
}

/// How far `spell` reaches at its base: "6", a burst's "range 10, radius 2", "next to you" for
/// a touch, "—" on oneself.
fn reach(spell: &SpellData) -> String {
    let n = |v: Option<u8>| v.map_or_else(String::new, |v| v.to_string());
    match spell.shape {
        Shape::Myself => "—".into(),
        Shape::Touch => "next to you".into(),
        Shape::Burst => format!("range {}, radius {}", n(spell.range), n(spell.radius)),
        Shape::Cone | Shape::Line => n(spell.length),
        Shape::Ray | Shape::Bolt => n(spell.range),
    }
}

/// What `spell` does at its base, as its data says: its roll, the status it leaves (at its base
/// degrees), what it does to ice and walls.
pub(super) fn does(content: &Content, spell: &SpellData) -> String {
    let status = |s| content.status(s).name.clone();
    let mut text = match spell.kind {
        SpellPurpose::Damage => {
            let rolled = spell.damage.map_or_else(String::new, |r| r.to_string());
            let kind = (spell.damage_type)
                .map_or_else(String::new, |k| format!(" {}", content.damage_type(k).name));
            format!("{rolled}{kind} damage")
        }
        SpellPurpose::Heal => {
            let healed = spell
                .effect
                .heal
                .map_or_else(String::new, |r| r.to_string());
            format!("{healed} hit points back")
        }
        SpellPurpose::Buff | SpellPurpose::Hex => match spell.effect.status {
            Some((s, n)) => {
                let who = if spell.kind == SpellPurpose::Buff {
                    "you are"
                } else if spell.shape == Shape::Burst {
                    "everyone it reaches but you, friend or foe, suffers"
                } else {
                    "a hostile you see suffers"
                };
                format!("{who} {} ({n})", status(s))
            }
            None => String::new(),
        },
        SpellPurpose::Cleanse => {
            let mut ends: Vec<String> = spell.effect.cures.iter().map(|&s| status(s)).collect();
            if spell.effect.diseases {
                ends.push("every disease".into());
            }
            format!("ends {}", ends.join(" and "))
        }
        SpellPurpose::Utility => spell.description.clone(),
        SpellPurpose::Summon => match &spell.effect.summon {
            Some(call) => {
                let pool: Vec<&str> = (call.pool.iter())
                    .map(|&c| content.creature(c).name.as_str())
                    .collect();
                format!(
                    "calls {} of {} to your side for {} minutes, {} at most at once",
                    call.count,
                    pool.join(" or "),
                    call.minutes,
                    call.most
                )
            }
            None => String::new(),
        },
    };
    if spell.kind == SpellPurpose::Damage
        && let Some((s, n)) = spell.effect.status
    {
        let _ = write!(text, "; everyone it harms suffers {} ({n})", status(s));
    }
    // A spell's own `heat` or `frost` stands in for its type's; a burst's reaches its cells (the
    // cell struck of a ray or a bolt is in the "Spell types" table).
    let rule = content.spell_type(spell.damage_type);
    let ice = spell.look.ice.or(rule.and_then(|r| r.ice));
    if spell.shape == Shape::Burst {
        text.push_str(&super::traps::ice_text(ice));
    }
    if spell.look.blast > 0 {
        text.push_str("; it breaks walls");
    }
    if spell.open_sky {
        text.push_str("; only under the open sky");
    }
    if spell.study_difficulty != 0 {
        let _ = write!(
            text,
            "; {:+} to the difficulty of learning it",
            spell.study_difficulty
        );
    }
    text
}

/// The armour and shields that make spells dearer while worn, most first, then by name.
pub(super) fn armour(content: &Content) -> String {
    let mut worn: Vec<(&str, u16)> = (content.object_defs().iter())
        .filter(|o| o.spell_cost_percent > 0)
        .map(|o| (o.name.as_str(), o.spell_cost_percent))
        .collect();
    worn.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let mut out = String::from("| Worn | Spells cost more |\n|---|:---:|\n");
    for (name, percent) in worn {
        let _ = writeln!(out, "| {name} | {percent} % |");
    }
    out
}

/// The wands (spec §8 "Magic items": "Wands"): each one's spell, its spell level, charges, the
/// roll it casts at (its lowest rung, before the user's wand rungs), and where loot finds it; by
/// spell level, then name.
pub(super) fn wands(content: &Content) -> String {
    let mut wands: Vec<_> = (content.object_defs().iter())
        .filter_map(|o| Some((o, o.wand?)))
        .collect();
    wands.sort_by(|(a, x), (b, y)| {
        let level = |w: &warlock_core::content::Wand| content.spell(w.spell).level;
        level(x).cmp(&level(y)).then(a.name.cmp(&b.name))
    });
    let mut out = String::from(
        "| Wand | Spell | Spell level | Charges | Strength | Item level | Frequency | Value |\n\
         |---|---|:---:|:---:|:---:|:---:|:---:|:---:|\n",
    );
    for (object, wand) in wands {
        let spell = content.spell(wand.spell);
        // Its strength: the rolls at its kind's lowest and highest rung, one of which (or one
        // between) each wand draws as it is made.
        let at = |rung: u8| spell.roll_at(usize::from(rung)).map(|r| r.to_string());
        let casts = match (at(wand.min_rung), at(wand.max_rung)) {
            (Some(lo), Some(hi)) if lo != hi => format!("{lo} to {hi}"),
            (Some(lo), _) => lo,
            _ => "—".to_string(),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} gold |",
            object.name,
            spell.name,
            spell.level,
            wand.charges,
            casts,
            object.level,
            object.frequency,
            object.value
        );
    }
    out
}

/// The rods (spec §8 "Magic items": "Rods"): each one's effect, its shape and reach (with its
/// ladder, as a spell's), what it does exactly, its mana cost, and where loot finds it; by item
/// level, then name.
pub(super) fn rods(content: &Content) -> String {
    let mut rods: Vec<_> = (content.object_defs().iter())
        .filter_map(|o| Some((o, o.rod?)))
        .collect();
    rods.sort_by(|(a, _), (b, _)| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
    let mut out = String::from(
        "| Rod | Effect | Shape | Reach | What it does | Mana | Item level | Frequency | Value |\n\
         |---|---|---|---|---|:---:|:---:|:---:|:---:|\n",
    );
    for (object, rod) in rods {
        let effect = content.rod_effect(rod.effect);
        let shape = match effect.shape {
            RodShape::Burst => format!("burst, radius {}", effect.radius),
            other => other.id().to_string(),
        };
        let reach = match &effect.grows {
            None => effect.range.to_string(),
            Some(ladder) => format!("{}; {}", effect.range, ladder.words()),
        };
        let does = match effect.does {
            RodDoes::Dig { power } => {
                format!("breaks every wall on its line of hardness below {power}")
            }
            RodDoes::Teleport => {
                "moves the first creature it hits to a random free cell of the level".to_string()
            }
            RodDoes::Status {
                status,
                severity,
                up_to_level,
            } => {
                let name = &content.status(status).name;
                let takes = up_to_level.map_or_else(String::new, |l| {
                    format!(". Takes creatures to level {l}; one above it shrugs it off")
                });
                format!("{name} {severity} on everyone in it{takes}")
            }
        };
        let _ = writeln!(
            out,
            "| {} | {} | {shape} | {reach} | {does} | {} | {} | {} | {} gold |",
            object.name, effect.name, rod.mana, object.level, object.frequency, object.value
        );
    }
    out
}

/// A percent as a multiplier: `160` is `1.6`, `100` is `1`.
fn times(percent: u16) -> String {
    let whole = percent / 100;
    match percent % 100 {
        0 => whole.to_string(),
        n if n % 10 == 0 => format!("{whole}.{}", n / 10),
        n => format!("{whole}.{n:02}"),
    }
}
