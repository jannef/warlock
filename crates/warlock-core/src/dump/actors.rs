//! The `actors` layer (dev-only): every actor on the level as it truly is.

use super::*;

/// An actor's weapon skills (ground truth): `daggers 2 (13/30), thrown 1 (0/10)`, the trained
/// ones only; `None` without any.
pub(super) fn actor_weapon_skills(
    content: &crate::content::Content,
    a: &crate::world::Actor,
) -> Option<String> {
    let table = content.weapon_skills();
    let skills: Vec<String> = (a.weapon_skills.trained())
        .map(|(id, t)| match table.need(id, t.rank) {
            Some(need) => format!("{} {} ({}/{need})", table.skill(id).id, t.rank, t.marks),
            None => format!("{} {}", table.skill(id).id, t.rank),
        })
        .collect();
    (!skills.is_empty()).then(|| skills.join(", "))
}

/// Every actor on the level (ground truth): id, creature, position, hit points, Defence and
/// Protection (`dv 11 pv 3`), its alignment (`alignment evil`; the player's score and band,
/// `alignment 450 (good)`), the court it joined for a type of several factions (`court
/// unseelie`), weight, what it carries, its burden and carrying limit, its stats
/// as score/potential
/// (`St 12/15`), its trained skills with their marks and the die those set on the ladder
/// (`stealth 17 (7 1d3)`), its status effects with their severities (`effects bleeding 2`), the
/// player's nutrition and hunger (`nutrition 3540 (not_hungry)`), and, if it has any, the unique
/// character it is and how far its talk has got; then all food on the level and in the packs of
/// those on it, with when it was made (a corpse: when its creature died) and how it has rotted
/// (`corpse kobold_corpse at (12, 5): died 1444-11-11 08:12:00, 2 hours ago, fresh`, `food
/// bread*2 carried by 0: made ..., 70 hours ago, old`): the stage food never shows the player.
pub(super) fn actor_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (level, actors) = world.ground_truth();
    let now = world.time();
    let food = |where_: String, s: &crate::object::Stack| {
        let made = s.died?;
        let stage = crate::rot::stage(content, s, now)?;
        let (what, verb) = match content.object(s.object).corpse {
            Some(_) => ("corpse", "died"),
            None => ("food", "made"),
        };
        let count = match s.count {
            1 => String::new(),
            n => format!("*{n}"),
        };
        Some(format!(
            "{what} {}{count} {where_}: {verb} {made}, {} ago, {}",
            content.object(s.object).id,
            now.since(made).spoken(),
            stage.id()
        ))
    };
    let on_level = (level.objects.iter())
        .flat_map(|(p, pile)| pile.iter().map(move |s| (*p, s)))
        .filter_map(|(p, s)| food(format!("at {p}"), s));
    let carried = (world.ground_truth().1)
        .flat_map(|a| a.inventory.iter().map(move |s| (a.id, s)))
        .filter_map(|(id, s)| food(format!("carried by {}", id.0), s))
        .collect::<Vec<_>>();
    let foods: Vec<String> = on_level.chain(carried).collect();
    let images = world.images();
    // Asking a creature's eyes warms the light cache: a copy of the world asks.
    let mut probe = world.clone();
    let player = world.state().player;
    let rows: Vec<String> = actors
        .map(|a| {
            let w = a.weight(content);
            let load = world.actor_load(a.id).expect("the actor exists");
            let carried: Vec<String> = (a.inventory.iter())
                .map(|s| s.describe_truth(content) + &s.holds_text(content).unwrap_or_default())
                .collect();
            let carried = if carried.is_empty() {
                "nothing".to_string()
            } else {
                carried.join(", ")
            };
            let equipped: Vec<String> = (a.equipment.items().into_iter())
                .filter_map(|(slot, item)| {
                    let stack = a.inventory.get(item)?;
                    Some(format!(
                        "{} {}",
                        content.slot(slot).id,
                        stack.describe_truth(content)
                    ))
                })
                .collect();
            let equipped = if equipped.is_empty() {
                String::new()
            } else {
                format!("; equipped {}", equipped.join(", "))
            };
            // What its magic items do to it (spec §8 "Magic items"), by effect id.
            let mods = a.mods(content);
            let mut magic: Vec<String> = (mods.iter())
                .map(|(effect, value)| format!("{} {value:+}", effect.id()))
                .collect();
            if let Some(light) = mods.light {
                magic.push(format!("light {}", content.light(light).id));
            }
            for &(skill, value) in mods.skills() {
                magic.push(format!("skill {} {value:+}", content.skill(skill).id));
            }
            for &(effect, pool, value) in mods.pools() {
                let pool = &content.resource(pool).id;
                magic.push(format!("{} {pool} {value:+}", effect.id()));
            }
            let equipped = if magic.is_empty() {
                equipped
            } else {
                format!("{equipped}; magic {}", magic.join(" "))
            };
            let stats: Vec<String> = Stat::ALL
                .iter()
                .map(|&s| {
                    let (score, potential) = (a.stats.score(s), a.stats.potential(s));
                    format!("{} {score}/{potential}", s.abbrev())
                })
                .collect();
            let skills: Vec<String> = (a.skills.iter())
                .map(|(id, t)| {
                    let ladder = content.ladder();
                    let die = ladder.die(ladder.step(t.marks));
                    format!("{} {} ({} {die})", content.skill(id).id, t.value, t.marks)
                })
                .collect();
            let skills = if skills.is_empty() {
                String::new()
            } else {
                format!("; skills {}", skills.join(", "))
            };
            let hunger = match (a.satiation, a.hunger(content)) {
                (Some(s), Some(h)) => format!("; nutrition {} ({})", s.nutrition, h.id()),
                _ => String::new(),
            };
            let effects: Vec<String> = (a.effects.iter())
                .map(|e| format!("{} {}", content.status(e.status).id, e.severity))
                .collect();
            let effects = if effects.is_empty() {
                String::new()
            } else {
                format!("; effects {}", effects.join(", "))
            };
            let diseases = disease_text(a, content);
            // Its alternative resources (spec §8 "Resources"): value/maximum, and the progress
            // toward the next point over time in hundredths of a second.
            let resources: Vec<String> = (a.resources.iter())
                .map(|p| {
                    let id = &content.resource(p.resource).id;
                    let max = a.resource_max(content, p.resource, world.holder_level(a.id));
                    format!("{id} {}/{max} ({})", p.value, p.progress)
                })
                .collect();
            let resources = if resources.is_empty() {
                String::new()
            } else {
                format!(" resources {}", resources.join(", "))
            };
            let weapons = actor_weapon_skills(content, a)
                .map_or(String::new(), |s| format!("; weapon skills {s}"));
            // Who it is and how far its talk has got (spec §8 "Dialogue"), if anything.
            let npc = (a.npc.as_ref()).map_or(String::new(), |n| format!("; npc {n}"));
            // Where it stands between good and evil (spec §8 "Alignment"): its score and band
            // (the player's), or its nature.
            let alignment = match a.alignment {
                Some(score) => format!("; alignment {} ({})", score.value(), a.band(content).id()),
                None => format!("; alignment {}", a.nature(content).id()),
            };
            // The court it joined, for a type of several factions (spec §7 "Factions": the fey).
            let court = match a.faction(content) {
                Some(f) if !content.creature(a.creature).factions.is_empty() => {
                    format!("; court {}", content.factions().def(f).id)
                }
                _ => String::new(),
            };
            // Its grade (spec §8 "Grades"): `; grade big`.
            let def = content.creature(a.creature);
            let court = match crate::grade::word(def, a.graded()) {
                Some(word) => format!("{court}; grade {word}"),
                None => court,
            };
            // Unseen (`world/unseen.rs`): `; invisible`, by its type or a status now; `; shadowy`.
            let court = match (a.invisible(content), def.shadowy) {
                (true, _) => format!("{court}; invisible"),
                (false, true) => format!("{court}; shadowy"),
                (false, false) => court,
            };
            // Rooted (`world/rooted.rs`): `; rooted`, a creature that never moves from its cell.
            let court = if def.rooted {
                format!("{court}; rooted")
            } else {
                court
            };
            // Who summoned it (spec §8 "Summoning"), and so whose side it took.
            let court = match a.summoner {
                Some(by) => format!("{court}; summoned by {by}"),
                None => court,
            };
            // Whose guard its openings are in (`world/mark.rs`).
            let court = match a.mark {
                Some(mark) => format!("{court}; mark {mark}"),
                None => court,
            };
            // Whose ally it is, what made it one, how long it stays, and its order (spec §8
            // "Allies"): `; ally of #0 (summoned, 4 min; follow)`, `(companion; stay at (12, 5))`,
            // `(summoned; follow; retinue)`.
            let court = match &a.ally {
                Some(ally) => {
                    let lasts = match ally.until {
                        Some(until) => {
                            let seconds = until.since(world.time()).as_seconds();
                            format!(", {} min", seconds.div_ceil(60))
                        }
                        None => String::new(),
                    };
                    let order = match ally.order.at() {
                        Some(at) => format!("{} at ({}, {})", ally.order.id(), at.x, at.y),
                        None => ally.order.id().to_string(),
                    };
                    let retinue = if ally.retinue { "; retinue" } else { "" };
                    let source = ally.source.id();
                    // A kept beast's name and grade (`world/companion.rs`, `familiar.rs`).
                    let named = match ally.source.data() {
                        Some(c) => format!("; named {}, grade {}", c.name, c.grade),
                        None => String::new(),
                    };
                    format!(
                        "{court}; ally of {} ({source}{lasts}; {order}{retinue}{named})",
                        ally.master
                    )
                }
                None => court,
            };
            // Risen berserk, serving no one (`world/corpse_spell.rs`): its faction and its clock.
            let court = match a.risen {
                Some(until) => {
                    let faction = (a.faction(content))
                        .map_or("none", |f| content.factions().def(f).id.as_str());
                    let seconds = until.since(world.time()).as_seconds();
                    format!(
                        "{court}; risen, faction {faction}, {} min",
                        seconds.div_ceil(60)
                    )
                }
                None => court,
            };
            // The overworld encounter that made it (spec §7 "Overworld encounters").
            let court = match world.party_of(a.id) {
                Some(entry) => format!("{court}; of encounter {entry}"),
                None => court,
            };
            // A trader's offer, once made (`world/wares.rs`): the item and its price, or sold.
            let court = match world.state().wares.get(&a.id) {
                Some(w) => match &w.item {
                    Some(item) => {
                        let item = item.describe_truth(content);
                        format!("{court}; wares {item} for {} gold", w.price)
                    }
                    None => format!("{court}; wares sold ({} gold)", w.price),
                },
                None => court,
            };
            // The traders' uniques the run has offered (each once).
            let court = match a.id == player && !world.state().uniques_offered.is_empty() {
                true => {
                    let offered: Vec<&str> =
                        world.state().uniques_offered.iter().map(String::as_str).collect();
                    format!("{court}; uniques offered {}", offered.join(", "))
                }
                false => court,
            };
            let talk = if a.talk.is_new() {
                String::new()
            } else {
                let met = if a.talk.met { ", met" } else { "" };
                let (said, muttered) = (a.talk.said, a.talk.muttered);
                format!("; talk said {said}, muttered {muttered}{met}")
            };
            // Whether it has seen the player, and when the player last saw it (`surprise.rs`).
            let talk = if a.id == player {
                talk
            } else {
                // Slipped past in the dark (`slip.rs`): when, while it lasts.
                let aware = if probe.slipped_past(a.id, player) {
                    "slipped past"
                } else if probe.unaware(a.id, player) {
                    "unaware of player"
                } else {
                    "aware"
                };
                let aware = match a.mind.noticed {
                    Some(turn) => format!("{aware}, noticed turn {turn}"),
                    None => aware.to_string(),
                };
                // A lurker waits for its prey (`lurk.rs`).
                let aware = if content.creature(a.creature).lurks {
                    format!("{aware}, lurks")
                } else {
                    aware
                };
                match a.mind.missed {
                    Some(turn) => format!("{talk}; {aware}, missed turn {turn}"),
                    None => format!("{talk}; {aware}"),
                }
            };
            // The player's background story by ids (spec §8 "Background story").
            let talk = match world.state().character.as_ref().filter(|_| a.id == player) {
                Some(c) if !c.story.is_empty() => {
                    let ids: Vec<String> = c.story.iter().map(|t| t.id()).collect();
                    format!("{talk}; story {}", ids.join(" "))
                }
                _ => talk,
            };
            // What it resists, in percent by kind, as it counts (ground truth: never shown in
            // play), and the total behind it where the cap cuts it (#1620): `fire 75 (of 200)`.
            let resistances = world.resistances_of(a.id);
            let resists: Vec<String> = (resistances.iter())
                .map(|(k, p)| {
                    let total = resistances.total(Some(k));
                    let id = &content.damage_type(k).id;
                    if p == crate::damage::IMMUNE || total == p {
                        format!("{id} {p}")
                    } else {
                        format!("{id} {p} (of {total})")
                    }
                })
                .collect();
            let resists = if resists.is_empty() {
                String::new()
            } else {
                format!("; resists {}", resists.join(" "))
            };
            // Its reflections in a hall of mirrors (`world/mirrors.rs`), wherever the player is.
            let mirrored: Vec<String> = (images.iter())
                .filter(|i| i.of == a.id)
                .map(|i| i.pos.to_string())
                .collect();
            let resists = if mirrored.is_empty() {
                resists
            } else {
                format!("{resists}; images {}", mirrored.join(" "))
            };
            format!(
                "{} {} at {}: hp {}/{} mp {}/{}{resources} dv {} pv {}{alignment}{court}; weight {} (base {} + carried {}: \
                 {carried}){equipped}; {} (limit {}){weapons}; stats {}{skills}{effects}{diseases}\
                 {hunger}{npc}{talk}{resists}",
                a.id,
                content.creature(a.creature).id,
                a.pos,
                a.hp,
                a.max_hp(content),
                a.mana,
                a.max_mana(content),
                world.defence_of(a.id).total(),
                world.protection_of(a.id).base,
                format_weight(w.total()),
                format_weight(w.base),
                format_weight(w.carried),
                load.burden().id(),
                format_weight(load.limit),
                stats.join(" "),
            )
        })
        .collect();
    rows.into_iter().chain(foods).collect()
}

/// An actor's diseases, stat damage, and drained points, as the `actors` dump shows them
/// (ground truth; `; drained St 2` after the rest):
/// `; diseases filth_fever incubating 2d, red_ache 3 (1 won) 1d; damage St 2`, each infection
/// with its progressions so far (and checks won in a row) and the time to its next turn, in
/// days and hours; empty without any.
pub(super) fn disease_text(a: &crate::world::Actor, content: &crate::content::Content) -> String {
    let mut out = String::new();
    let infections: Vec<String> = (a.diseases.iter())
        .map(|i| {
            let id = &content.disease(i.disease).id;
            let left = u64::from(i.left);
            let (days, hours) = (left / crate::time::DAY, left % crate::time::DAY / 3600);
            let stage = if i.shown {
                let won = match i.streak {
                    0 => String::new(),
                    n => format!(" ({n} won)"),
                };
                format!("{}{won}", i.progressions)
            } else {
                "incubating".to_string()
            };
            format!("{id} {stage} {days}d{hours}h")
        })
        .collect();
    if !infections.is_empty() {
        let _ = write!(out, "; diseases {}", infections.join(", "));
    }
    let damaged: Vec<String> = (Stat::ALL.iter())
        .filter(|&&s| a.damage.get(s) > 0)
        .map(|&s| format!("{} {}", s.abbrev(), a.damage.get(s)))
        .collect();
    if !damaged.is_empty() {
        let _ = write!(out, "; damage {}", damaged.join(" "));
    }
    let drained: Vec<String> = (Stat::ALL.iter())
        .filter(|&&s| a.drained[s] > 0)
        .map(|&s| format!("{} {}", s.abbrev(), a.drained[s]))
        .collect();
    if !drained.is_empty() {
        let _ = write!(out, "; drained {}", drained.join(" "));
    }
    out
}
