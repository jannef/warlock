//! The `progress` layer (dev-only): level-up choices, callings, rolled quests, and marks.

use super::*;

/// The player's progression (dev): the choices due, the talents they may take now with the
/// choices each may be taken for, and the creature types whose first kill has paid.
/// Each calling a run of the campaign may walk through and where the player stands with it
/// (`progress`, dev-only; spec §8 "Prestige classes").
pub(super) fn calling_rows(world: &World) -> Vec<String> {
    let content = world.content();
    (content.callings_of(world.campaign()))
        .map(|c| match world.calling_status(c) {
            CallingStatus::Open => format!("calling {}: open", c.class),
            CallingStatus::LevelNotMet => format!("calling {}: level {} not met", c.class, c.level),
            CallingStatus::NotFrom => {
                let class = (world.character())
                    .and_then(|k| k.spec.class(content).ok())
                    .map_or("nobody".to_string(), |k| {
                        format!(
                            "{} {}",
                            crate::text::article(&k.name),
                            k.name.to_lowercase()
                        )
                    });
                format!("calling {}: not for {class}", c.class)
            }
            CallingStatus::Called => format!("called {} at {}", c.class, c.level),
        })
        .collect()
}

pub(super) fn progress_rows(world: &World) -> Vec<String> {
    let view = world.perception();
    let content = view.content();
    let progress = &world.state().progress;
    let mut rows: Vec<String> = (progress.due.iter())
        .map(|due| {
            let count = |n: u8, what: &str| match n {
                0 => None,
                1 => Some(format!("1 {what}")),
                n => Some(format!("{n} {what}s")),
            };
            let parts: Vec<String> = [
                count(due.rolls, "skill roll"),
                count(due.talents, "talent"),
                count(due.bonus, "bonus talent"),
            ]
            .into_iter()
            .flatten()
            .collect();
            format!("due level {}: {}", due.level, parts.join(", "))
        })
        .collect();
    // What the level-up screen shows of skill rolls (#543): where a level's come from, and
    // the dice a roll on each trained skill adds (`/2`: halved past the soft cap).
    let count = view.roll_count();
    if count.total() > 0 {
        rows.push(format!(
            "rolls a level {} (class {}, race {}, intelligence {}, talents {})",
            count.total(),
            count.class,
            count.race,
            count.intelligence,
            count.talents
        ));
    }
    let dice: Vec<String> = (view.own_skills().iter())
        .filter_map(|s| {
            let roll = view.skill_roll(s.skill)?;
            let half = if roll.halved { "/2" } else { "" };
            let id = &content.skill(s.skill).id;
            Some(format!("{id} {}{half}", roll.dice()))
        })
        .collect();
    if !dice.is_empty() {
        rows.push(format!("roll dice {}", dice.join(", ")));
    }
    let takeable: Vec<String> = (view.talents().iter())
        .filter(|t| !t.openings.is_empty())
        .map(|t| {
            let id = &content.talents().get(t.id).id;
            let ways: Vec<&str> = t.openings.iter().flatten().map(String::as_str).collect();
            if ways.is_empty() {
                id.clone()
            } else {
                format!("{id}:{{{}}}", ways.join(","))
            }
        })
        .collect();
    if !takeable.is_empty() {
        rows.push(format!("takeable {}", takeable.join(", ")));
    }
    // The callings of the run's campaign and where the player stands with each (spec §8
    // "Prestige classes"): `calling hunter: open`, `calling hunter: level 30 not met`,
    // `calling hunter: not for a thief`, and once called `called hunter at 30`.
    rows.extend(calling_rows(world));
    let kills: Vec<&str> = progress
        .discovered
        .kills
        .iter()
        .map(String::as_str)
        .collect();
    if !kills.is_empty() {
        rows.push(format!("discovered {}", kills.join(", ")));
    }
    // Beast-friend: the animal types the player attacked, theirs to fight again.
    let broken: Vec<&str> = (progress.empathy_broken.iter())
        .map(String::as_str)
        .collect();
    if !broken.is_empty() {
        rows.push(format!("empathy broken {}", broken.join(", ")));
    }
    // Rolled quests: what each drew, of how many it could draw now; the named monsters dead.
    for (id, roll, pool) in world.quest_rolls() {
        let variant = roll.variant.map_or_else(String::new, |v| format!(" ({v})"));
        rows.push(format!(
            "roll {id} {}{variant} x{} of {pool}",
            roll.target, roll.count
        ));
    }
    // What an aristocrat's family has put by, waiting with their servant (`world/stipend.rs`).
    rows.extend(world.stipend_row());
    // A merchant's stake with their partner (`world/invest.rs`).
    rows.extend(world.invest_row());
    // A craftsperson's commissions from their patron (`world/patron.rs`).
    rows.extend(world.patron_row());
    let slain: Vec<&str> = world.quests().slain().collect();
    if !slain.is_empty() {
        rows.push(format!("slain {}", slain.join(", ")));
    }
    // The prayer's strain (the priest's, hidden in the game), its face, and what it answers.
    if let Some(prayer) = world.prayer_state() {
        let locked = if prayer.locked { " (locked)" } else { "" };
        let uses: Vec<&str> = prayer.uses.iter().map(|u| u.id()).collect();
        let answers = prayer.answers.map_or("all".to_string(), |n| n.to_string());
        rows.push(format!(
            "prayer strain {}{locked}, face {}, answers {answers}, uses {}",
            prayer.strain,
            prayer.face.id(),
            uses.join(", ")
        ));
    }
    // Each stat's pile with the price in force: the rise's, or the drop's for a negative pile.
    let player = world.state().player();
    let rules = world.content().stat_training();
    let marked: Vec<String> = (player.stat_marks.iter())
        .filter_map(|(stat, n)| {
            let stats = &player.stats;
            let price = rules.price_for(n, stats.score(stat), stats.potential(stat))?;
            Some(format!("{} {n}/{price}", stat.abbrev()))
        })
        .collect();
    if !marked.is_empty() {
        rows.push(format!("stat marks {}", marked.join(", ")));
    }
    // The profane gifts held and each tier's pool: the gifts a draw may give now of all it holds.
    if !world.content().profane().is_empty() {
        rows.push(profane_row(world));
    }
    // The rung each known spell's roll is cast at (the spell scaling design §11): of its
    // ladder's length, with the score that reached it and the rungs from outside the stats.
    let content = world.content();
    for spell in view.own_spells() {
        let Some(def) = content.spell_of(spell) else {
            continue;
        };
        let Some(ladder) = def.grows.roll() else {
            continue;
        };
        let mods = player.spell_mods(content, def);
        let rung = player.spell_rung(content, def, crate::world::CastBy::Own, &mods);
        let score = player.effective_stats(content).score(ladder.stat);
        let mut why = vec![format!("{} {score}", ladder.stat.abbrev())];
        for (name, rungs) in player.spell_rung_sources(content, def) {
            why.push(format!("+{rungs} {name}"));
        }
        rows.push(format!(
            "{} rung {rung}/{} ({})",
            def.id,
            ladder.len(),
            why.join("; ")
        ));
    }
    // What a grimoire teaches for certain (spec §8 "Spells": "Learning").
    let trained = (player.skills.iter()).find(|&(id, _)| {
        let benefits = &world.content().skill(id).benefits;
        benefits
            .iter()
            .any(|b| b.effect == crate::skills::Effect::Learn)
    });
    rows.push(match (player.learning(world.content()), trained) {
        (Some(learning), Some((id, skill))) => format!(
            "learning {learning} ({} {})",
            world.content().skill(id).name,
            skill.value
        ),
        _ => "cannot learn: not trained in Decipher script".to_string(),
    });
    rows
}

/// `profane 3 held; pool lesser 9/14, greater 12/12, dire 14/14` (`progress`, dev-only;
/// docs/design/profane-gifts.md §11).
fn profane_row(world: &World) -> String {
    let content = world.content();
    let player = world.state().player();
    let held = (player.talents.iter())
        .filter_map(|t| content.talents().id(&t.id))
        .filter(|&t| content.profane().gift(t).is_some())
        .count();
    let pools: Vec<String> = (GiftTier::ALL.iter())
        .map(|&tier| {
            let (open, all) = world.gift_pool(tier);
            format!("{} {open}/{all}", tier.id())
        })
        .collect();
    format!("profane {held} held; pool {}", pools.join(", "))
}
