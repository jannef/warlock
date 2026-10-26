//! Talents (spec §8 "Talents"): every tree with its talents, their prerequisites and their rules
//! in full (the game says them in words), and the coverage matrix by system.

use super::text::signed;
use std::fmt::Write as _;
use warlock_core::content::{
    Content, Need, OverTime, Role, SignActive, SpellData, TalentDef, TalentEffect, TreeId,
};

/// Every tree in display order: a heading, its description, and its talents (name, ranks, what
/// each rank needs, what it does as the rule, and the systems it touches).
pub(super) fn talents(content: &Content) -> String {
    let book = content.talents();
    let mut out = String::new();
    for (i, tree) in (0u8..).zip(&book.trees) {
        let _ = writeln!(out, "### {}\n\n{}\n", tree.name, tree.description);
        let rows: Vec<&TalentDef> = book.of_tree(TreeId(i)).map(|(_, t)| t).collect();
        if rows.is_empty() {
            out.push_str("No talents yet.\n\n");
            continue;
        }
        out.push_str("| Talent | Ranks | Needs | Effect | Systems |\n|---|---:|---|---|---|\n");
        for def in rows {
            let systems: Vec<&str> = (def.systems.iter())
                .map(|s| book.system(*s).name.as_str())
                .collect();
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} |",
                name(def),
                def.rank_count(),
                needs_text(content, def),
                effects_text(content, def),
                systems.join(", ")
            );
        }
        out.push('\n');
    }
    out
}

/// The talents each class picks at creation (spec §8 "Talents"): everyone's and the class's
/// more, by class name.
pub(super) fn creation(content: &Content) -> String {
    let base = content.experience().creation_talents();
    let mut out = String::from("| Class | Talents at creation |\n|---|---:|\n");
    let mut classes: Vec<_> = content.classes().iter().collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));
    for class in classes {
        let n = base.saturating_add(class.creation_talents);
        let _ = writeln!(out, "| {} | {n} |", class.name);
    }
    out
}

/// The racial talents (spec §8 "Talents", the Racial tree), race by race: those a race grants,
/// then those that ask for it, each with what else it needs and its rule. Races and talents
/// sort by name.
pub(super) fn racial(content: &Content) -> String {
    let book = content.talents();
    let Some(tree) = book.tree_id("racial") else {
        return "No racial talents yet.\n".to_string();
    };
    let mut races: Vec<_> = content.races().list.iter().collect();
    races.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from("| Race | Talent | Needs | Effect |\n|---|---|---|---|\n");
    for race in races {
        let mut rows: Vec<&TalentDef> = (book.of_tree(tree))
            .map(|(_, t)| t)
            .filter(|t| {
                race.talents.contains(&t.id)
                    || t.needs
                        .iter()
                        .any(|n| matches!(&n, Need::Race(id) if *id == race.id))
            })
            .collect();
        rows.sort_by_key(|t| (!t.granted, t.name.clone()));
        for def in rows {
            let others: Vec<String> = (def.needs.iter())
                .filter(|n| !matches!(n, Need::Race(_)))
                .map(|n| n.words(content))
                .collect();
            let needs = match (def.granted, others.is_empty()) {
                (true, _) => "given, never picked".to_string(),
                (false, true) => "—".to_string(),
                (false, false) => others.join(", "),
            };
            let _ = writeln!(
                out,
                "| {} | {} | {needs} | {} |",
                race.name,
                def.name,
                effects_text(content, def)
            );
        }
    }
    out
}

/// The classes' level tables (spec §8 "Class abilities"), in the order New run offers them:
/// each class with a table, its bonus talent picks, and every level's ability with its rule.
pub(super) fn class_levels(content: &Content) -> String {
    let book = content.talents();
    let mut out = String::new();
    for class in content.classes() {
        if class.abilities.is_empty() && class.bonus_talents.levels.is_empty() {
            continue;
        }
        let _ = writeln!(out, "### {}\n", class.name);
        let bonus = &class.bonus_talents;
        if !bonus.levels.is_empty() {
            let levels: Vec<String> = bonus.levels.iter().map(u8::to_string).collect();
            let trees = match class.bonus_trees(book) {
                None => "any tree".to_string(),
                Some(trees) => (trees.iter())
                    .map(|&t| book.tree(t).name.clone())
                    .collect::<Vec<_>>()
                    .join(" or "),
            };
            let _ = writeln!(
                out,
                "A bonus talent at levels {}, from {trees}.\n",
                levels.join(", ")
            );
        }
        out.push_str("| Level | Ability | Effect |\n|---:|---|---|\n");
        for (level, id) in &class.abilities {
            let Some(talent) = book.id(id) else { continue };
            let def = book.get(talent);
            let _ = writeln!(
                out,
                "| {level} | {} | {} |",
                def.name,
                effects_text(content, def)
            );
        }
        out.push('\n');
    }
    if out.is_empty() {
        out.push_str("No class has a level table yet.\n");
    }
    out
}

/// The coverage matrix: every system of the closed list with the talents that touch it, and
/// whether the bar (`[coverage]` in `data/talents.toml`) already holds it. Systems and talents
/// sort by name, never by file order.
pub(super) fn by_system(content: &Content) -> String {
    let book = content.talents();
    let bar = book.coverage;
    let mut systems: Vec<_> = (0u8..).zip(&book.systems).collect();
    systems.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    let mut out = String::from("| System | Talents | Held to the bar |\n|---|---|---|\n");
    for (i, system) in systems {
        let mut names: Vec<String> = (book.of_system(warlock_core::content::SystemId(i)))
            .map(|(_, t)| {
                // A class's, a race's, or a birth sign's gift is marked: given, never picked.
                let racial = book.tree_id("racial");
                if t.granted && book.tree(t.tree).class {
                    format!("{} (class)", t.name)
                } else if t.granted && Some(t.tree) == racial {
                    format!("{} (race)", t.name)
                } else if t.granted {
                    format!("{} (birth sign)", t.name)
                } else {
                    t.name.clone()
                }
            })
            .collect();
        names.sort();
        let held = if system.pending {
            "not yet (pending)".to_string()
        } else {
            match system.min_talents.unwrap_or(bar.min_talents) {
                1 => "yes (1 talent at least)".to_string(),
                least => format!("yes ({least} talents at least)"),
            }
        };
        let talents = if names.is_empty() {
            "—".to_string()
        } else {
            names.join(", ")
        };
        let _ = writeln!(out, "| {} | {talents} | {held} |", system.name);
    }
    out
}

/// "Weapon focus (a weapon skill)" for a talent with a choice, "Slight (flaw)" for a birth
/// sign's flaw, else the name.
fn name(def: &TalentDef) -> String {
    match def.choice {
        Some(kind) => format!("{} ({})", def.name, kind.words()),
        None if def.flaw => format!("{} (flaw)", def.name),
        None => def.name.clone(),
    }
}

/// The prerequisites: the talent's own, then each rank's where they differ ("Toughness 11 / 14
/// / 17"); "—" for none; a granted talent is never picked.
pub(super) fn needs_text(content: &Content, def: &TalentDef) -> String {
    if def.granted && content.talents().tree(def.tree).class {
        return "given by a class's table, never picked".into();
    }
    if def.granted {
        let mut races: Vec<String> = (content.races().list.iter())
            .filter(|r| r.talents.contains(&def.id))
            .map(|r| r.name.to_lowercase())
            .collect();
        races.sort();
        if !races.is_empty() {
            return format!("given to every {}, never picked", races.join(", "));
        }
        return "given by a birth sign, never picked".into();
    }
    let words = |needs: &[Need]| -> String {
        (needs.iter())
            .map(|n| n.words(content))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let common: Vec<Need> = def.needs.iter().collect();
    let per_rank: Vec<Vec<Need>> = (def.ranks.iter())
        .map(|r| r.needs.iter().collect())
        .collect();
    let mut parts = Vec::new();
    if !common.is_empty() {
        parts.push(words(&common));
    }
    if per_rank.iter().any(|r| !r.is_empty()) {
        let ranks: Vec<String> = (per_rank.iter().enumerate())
            .map(|(i, r)| {
                if r.is_empty() {
                    format!("{}: —", i + 1)
                } else {
                    format!("{}: {}", i + 1, words(r))
                }
            })
            .collect();
        parts.push(ranks.join("; "));
    }
    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join("; ")
    }
}

/// What the talent does, as the rule: its per-rank effects ("per rank" when it has several
/// ranks), then any rank's own.
pub(super) fn effects_text(content: &Content, def: &TalentDef) -> String {
    let ranked = def.rank_count() > 1;
    let mut parts: Vec<String> = (def.effects.iter())
        .map(|e| {
            let text = effect_text(content, e);
            if ranked {
                format!("{text} per rank")
            } else {
                text
            }
        })
        .collect();
    for (i, rank) in def.ranks.iter().enumerate() {
        for effect in &rank.effects {
            parts.push(format!("rank {}: {}", i + 1, effect_text(content, effect)));
        }
    }
    if let Some(cost) = def.resource {
        let name = content.resource(cost.resource).name.to_lowercase();
        parts.push(format!("costs {} {name}", cost.amount));
    }
    parts.join("; ")
}

/// One effect as the rule (talents' and class abilities').
/// A cooldown status by name and how long `degrees` of it last: "Graceless for 24 h".
fn cooldown_for(content: &Content, id: &str, degrees: u16) -> String {
    let Some(status) = content.status_id(id) else {
        return id.to_string();
    };
    let def = content.status(status);
    match def.eases.map(|s| s * u32::from(degrees)) {
        Some(s) if s % 3600 == 0 => format!("{} for {} h", def.name, s / 3600),
        Some(s) if s % 60 == 0 => format!("{} for {} min", def.name, s / 60),
        Some(s) => format!("{} for {s} s", def.name),
        None => def.name.clone(),
    }
}

/// The cooldown of the birth sign's active `active` at `degrees`, as [`cooldown_for`] says it.
fn sign_cooldown(content: &Content, active: SignActive, degrees: u16) -> String {
    let id = (content.sign_active(active).into_iter())
        .flat_map(|def| def.effects.iter())
        .find_map(|e| match e {
            TalentEffect::HornsGrace { cooldown, .. } | TalentEffect::Stare { cooldown, .. } => {
                Some(cooldown.clone())
            }
            _ => None,
        });
    id.map_or_else(
        || format!("{degrees} degree of its cooldown"),
        |id| cooldown_for(content, &id, degrees),
    )
}

/// An alternative resource's name ("Focus"), or its id when content has none by it.
fn resource_name(content: &Content, id: &str) -> String {
    (content.resource_id(id)).map_or_else(|| id.to_string(), |r| content.resource(r).name.clone())
}

/// A buff status in words: its name, what it adds to fighting, and how long it lasts
/// ("Inspired (to hit +1, Dodge +1, 5 min)").
fn buff(content: &Content, id: &str) -> String {
    let Some(s) = content.status_id(id).map(|s| content.status(s)) else {
        return id.to_string();
    };
    let c = s.combat;
    let parts = [
        ("to hit", c.to_hit),
        ("damage", c.damage),
        ("Dodge", c.dodge),
        ("Parry", c.parry),
    ];
    let mut said: Vec<String> = (parts.iter())
        .filter(|(_, v)| *v != 0)
        .map(|(what, v)| format!("{what} {}", signed(*v)))
        .collect();
    said.extend(s.eases.map(|e| format!("{} min", e / 60)));
    format!("{} ({})", s.name, said.join(", "))
}

pub(super) fn effect_text(content: &Content, effect: &TalentEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    let status = |id: &str| {
        (content.status_id(id)).map_or_else(|| id.to_string(), |s| content.status(s).name.clone())
    };
    match effect.clone() {
        TalentEffect::Dodge { value } => format!("Dodge {}", signed(value)),
        TalentEffect::Natural { value } => format!("natural armour {}", signed(value)),
        TalentEffect::NaturalUnarmoured { value } => {
            format!(
                "natural armour {} while you wear no body armour",
                signed(value)
            )
        }
        TalentEffect::MeleeDamage { value } => format!("melee damage {}", signed(value)),
        TalentEffect::Ferocity { value } => format!(
            "damage {} with every attack while below a third of your hit points",
            signed(value)
        ),
        TalentEffect::Threat { value } => {
            format!("threat range {} with every attack", signed(value))
        }
        TalentEffect::RerollFumble => "a natural 2 on the attack roll is rolled again, once".into(),
        TalentEffect::MaxHpPerLevels {
            per_levels,
            at_least,
        } => format!("+1 maximum hit point per {per_levels} levels, at least +{at_least}"),
        TalentEffect::MaxHpPercent { value } => {
            format!(
                "maximum hit points {} (rounded down, at least 1)",
                percent(value)
            )
        }
        TalentEffect::Stat { stat, value } => {
            format!("{} {} (score and potential)", stat.name(), signed(value))
        }
        TalentEffect::HealingPercent { value } => format!("healing pace {}", percent(value)),
        TalentEffect::ManaRegenPercent { value } => {
            format!("mana regeneration pace {}", percent(value))
        }
        TalentEffect::MaxManaPerLevels {
            per_levels,
            at_least,
        } => format!("+1 maximum mana per {per_levels} levels, at least +{at_least}"),
        TalentEffect::ManaRestingPercent { value } => {
            format!("mana regeneration pace while resting {}", percent(value))
        }
        TalentEffect::ManaCostPercent { value } => format!(
            "abilities' mana costs {} (rounded to the nearest, at least 1)",
            percent(value)
        ),
        TalentEffect::CasterLevel { value } => format!("caster level {}", signed(value)),
        TalentEffect::BloodMagic { hp, mana } => format!(
            "active (the time of a wait): give {hp} hit points (never your last) for {mana} \
             mana, up to your maximum; not while your mana is full"
        ),
        TalentEffect::FootstepsPercent { value } => format!("footsteps {}", percent(value)),
        TalentEffect::SilentDoors => "the doors you open and close are silent".to_string(),
        TalentEffect::Hearing { value } => format!("hears sounds {value} fainter"),
        TalentEffect::ExactFootsteps => {
            "footsteps you hear are placed at the cell they came from, not roughly".to_string()
        }
        TalentEffect::ThresholdPercent { value } => {
            format!("the light your eyes need {}", percent(value))
        }
        TalentEffect::Darkvision { value } => format!("darkvision {} cells", signed(value)),
        TalentEffect::SeesUv => "your eyes see ultraviolet light".to_string(),
        TalentEffect::Scent { status: id, radius } => format!(
            "sees creatures that are {} in line of sight within {radius} cells, however dark",
            status(&id).to_lowercase()
        ),
        TalentEffect::NightSight { value } => {
            format!("overworld sight at night {} cells", signed(value))
        }
        TalentEffect::DaySight { value } => {
            format!("overworld sight by day {} cells", signed(value))
        }
        TalentEffect::InnerLight { light } => {
            let reach = |id: &str| (content.light_id(id)).map_or(0, |l| content.light(l).radius);
            format!(
                "you always shed a faint light reaching {} cell (everyone sees it)",
                reach(&light)
            )
        }
        TalentEffect::LanternLight { light } => {
            let reach = |id: &str| (content.light_id(id)).map_or(0, |l| content.light(l).radius);
            format!(
                "your lantern burns brighter: its light reaches {} cells",
                reach(&light)
            )
        }
        TalentEffect::Ambush { dice } => format!(
            "+{dice} damage, after Protection, on a hit at a creature that has not seen you \
             (it hunts no one it has seen, and does not see you now)"
        ),
        TalentEffect::CarryPercent { value } => format!("carrying limit {}", percent(value)),
        TalentEffect::IgnoreBurdened => "being Burdened costs you nothing (as a dwarf)".to_string(),
        TalentEffect::StepSlowdownPercent { value } => format!(
            "the slowdown your burden and being Bloated put on your steps {}",
            percent(value)
        ),
        TalentEffect::CarryKg { value } => format!("carrying limit {} kg", signed(value)),
        TalentEffect::WeightlessCoins => "coins you carry weigh nothing".to_string(),
        TalentEffect::TravelPercent { value } => {
            format!("overworld journey time {}", percent(value))
        }
        TalentEffect::Swimmer => {
            "wading (water, marsh) costs a plain step and never gives a disease".into()
        }
        TalentEffect::TerrainAs { terrains, like } => {
            let name = |id: &str| {
                (content.terrain_id(id))
                    .map_or_else(|| id.to_string(), |t| content.terrain(t).name.clone())
            };
            let names: Vec<String> = terrains.iter().map(|t| name(t)).collect();
            format!(
                "a step onto {} costs what one onto {} does",
                names.join(", "),
                name(&like)
            )
        }
        TalentEffect::PlungeProof => "a plunge through the ice does you no harm".to_string(),
        TalentEffect::LightOnIce => {
            "ice holds you as if you carried nothing (your own weight only)".to_string()
        }
        TalentEffect::ClimbPercent { value } => {
            format!("climbing out onto ice costs {}", percent(value))
        }
        TalentEffect::WadingProof => "wading never gives you a disease".to_string(),
        TalentEffect::BeastFriend => "creatures of animal factions are neutral to you until you \
                                      attack one of that type"
            .to_string(),
        TalentEffect::Forecast { hours } => format!(
            "under the open sky you know the first change of weather within {hours} hours \
             (the status bar's weather)"
        ),
        TalentEffect::SunHealing { value } => format!(
            "healing pace {} under the open sun (sun up, no roof, whatever the clouds)",
            percent(value)
        ),
        TalentEffect::HungerPercent { value } => format!("hunger pace {}", percent(value)),
        TalentEffect::SmellRot => {
            "the eat list says whether food smells fresh, turning (old),              or rotten"
                .to_string()
        }
        TalentEffect::RotHarmPercent { value } => {
            format!(
                "sickness from rotten food {} (rounded down)",
                percent(value)
            )
        }
        TalentEffect::NoExertion => "blows, shots, and throws burn no nutrition".to_string(),
        TalentEffect::NoBloating => "a meal never leaves you Bloated: what does not fit below \
             Bloated is lost, and you never overeat"
            .to_string(),
        TalentEffect::CorpseStatsPercent { value } => format!(
            "a fresh corpse's chance to raise each stat {} (at most 100 %)",
            percent(value)
        ),
        TalentEffect::XpPercent { value } => format!("experience gained {}", percent(value)),
        TalentEffect::SkillRolls { value } => format!("skill rolls every level {}", signed(value)),
        TalentEffect::ExtraPick { level: 1 } => "one more talent at creation".to_string(),
        TalentEffect::ExtraPick { level } => format!("one more talent at level {level}"),
        TalentEffect::TrainSkill { dice } => format!(
            "the chosen skill trained at {dice}, or raised by {dice} if trained (rolled as the \
             rank is taken)"
        ),
        TalentEffect::LootMagicPercent { value } => format!(
            "the weights of the rarities better than a find's least {} (anything found: magic \
             and rare; what creatures drop, always magic: rare)",
            percent(value)
        ),
        TalentEffect::ReadPercent { value } => format!("reading time {}", percent(value)),
        TalentEffect::ScrollKept { chance } => {
            format!("a {chance} % chance a scroll you read is not used up")
        }
        TalentEffect::SparesSelf => "the bursts of scrolls you read never catch you".to_string(),
        TalentEffect::QuestXpPercent { value } => {
            format!("experience from quests {}", percent(value))
        }
        TalentEffect::QuestGoldPercent { value } => {
            format!("gold from quests {}", percent(value))
        }
        TalentEffect::FavourPercent { value } => {
            format!("shop favour {} (before the cap)", percent(value))
        }
        TalentEffect::FavourCap { value } => {
            format!("the cap on shop favour {} (50 % without)", percent(value))
        }
        TalentEffect::SellPercent { value } => format!(
            "shops pay {} for what they buy from you (never more than they would ask)",
            percent(value)
        ),
        TalentEffect::Credit { gold } => {
            format!("keepers let you leave owing up to {gold} gold (the debt stays on their bill)")
        }
        TalentEffect::IdentifyTrades => "what you buy or sell in a shop is identified".to_string(),
        TalentEffect::Pilgrim => {
            "temple services cost half (rounded up), and a cure also mends one point of each \
             damaged stat"
                .to_string()
        }
        TalentEffect::Scavenger => {
            "what a creature you kill drops is identified as it lands".to_string()
        }
        TalentEffect::ItemWorth => "item descriptions show what things are worth".to_string(),
        TalentEffect::GemWorth => "item descriptions show what each gem is worth".to_string(),
        TalentEffect::IdentifyCarried { hours } => format!(
            "a magic item you carry for {hours} hours (counted from when you gain this) is \
             identified"
        ),
        TalentEffect::GemFindPercent { value } => format!(
            "the chance a find is a loose gem {} (anything found, and what creatures drop)",
            percent(value)
        ),
        TalentEffect::GemCut { value } => format!(
            "loose gems you find are cut {value} grade finer, at most the finest (those on \
             levels made from then on, and those kills drop)"
        ),
        TalentEffect::Notice { value } => {
            format!(
                "notices hidden things in passing, {} to the check",
                signed(value)
            )
        }
        TalentEffect::DangerSense { value } => format!(
            "danger sense +{value} (at 1: how a level feels as you arrive: quiet at a danger 4 or \
             more below your level, uneasy up to your level, dangerous up to 3 above it, else \
             deadly; at 2 also a note when a creature comes to your level)"
        ),
        TalentEffect::SpawnEveryPercent { value } => format!(
            "the time between attempts to bring a creature to your level {}",
            percent(value)
        ),
        TalentEffect::SenseOfPlace => "knows a special room's nature from a glimpse of it, \
            and the effect covering a whole level by name on arrival"
            .into(),
        TalentEffect::RoomPacePercent { value } => format!(
            "where a special room quickens healing or mana, the quickening {} (a room's 300% \
             pace becomes {}%)",
            percent(value),
            100 + 200 * (100 + value) / 100
        ),
        TalentEffect::RoomStatusImmune { status: id } => format!(
            "a special room's (or a whole level's) {} never takes you",
            status(&id)
        ),
        TalentEffect::WakePercent { value } => format!(
            "the noise that wakes a sleeper, from what you do, {} (a den's 200% of its \
             hearing threshold becomes {}%)",
            percent(value),
            200 * (100 + value) / 100
        ),
        TalentEffect::StoneSense => "notices hidden things on stone in passing".into(),
        TalentEffect::StoneFind { value } => format!(
            "{} to every check to find what is hidden on stone, searching and in passing",
            signed(value)
        ),
        TalentEffect::Search { value } => {
            format!("{} to every check to find hidden things", signed(value))
        }
        TalentEffect::Searching { value } => format!("searching {}", signed(value)),
        TalentEffect::SearchReach { value } => format!(
            "searching reaches {value} cell{} farther",
            if value == 1 { "" } else { "s" }
        ),
        TalentEffect::TrapDamagePercent { value } => format!(
            "damage traps do you {} (rounded down, at least 1)",
            percent(value)
        ),
        TalentEffect::Skill { skill, value } => {
            let name =
                (content.skill_id(&skill)).map_or(skill.clone(), |s| content.skill(s).name.clone());
            format!("{name} counts {}", signed(value))
        }
        TalentEffect::MakeWay => {
            "anyone not hostile makes way for you, those keeping a post too".to_string()
        }
        TalentEffect::FreeBlow { trigger } => {
            use warlock_core::content::FreeBlowTrigger as T;
            match trigger {
                T::Cleave => {
                    "free blow: a melee blow that kills gives one at another hostile beside you \
                     (Cleave and Cleaving strike together: on and on, as Great cleave)"
                }
                T::GreatCleave => {
                    "free blow: cleaving goes on while the blows kill (once at each hostile beside \
                     you a turn)"
                }
                T::Riposte => "free blow: a blow that misses you by your Parry gives one back",
                T::Opportunist => "free blow: a hostile that steps away from beside you takes one",
                T::Gatekeeper => "free blow: a hostile that steps up beside you takes one",
                T::Flurry => {
                    "free blow: a melee attack with nothing in hand strikes twice, the second blow \
                     at \u{2212}2 to hit"
                }
            }
            .to_string()
        }
        TalentEffect::Weapon {
            role,
            skill,
            to_hit,
            damage,
            threat,
            ranks,
            speed,
            reload,
            ease,
            bash,
            shield,
            crit,
            at_rank,
        } => {
            let with = match (role, skill) {
                (_, Some(id)) => (content.weapon_skills().id(&id)).map_or(id.clone(), |w| {
                    content.weapon_skills().skill(w).name.clone()
                }),
                (None, None) => "the chosen skill".to_string(),
                (Some(Role::Melee), _) => "every melee weapon skill".to_string(),
                (Some(Role::Launcher), _) => "every missile weapon skill".to_string(),
                (Some(Role::Thrown), _) => "throws".to_string(),
                (Some(Role::Unarmed), _) => "unarmed fighting".to_string(),
                (Some(Role::ShieldBlock), _) => "shield blocking".to_string(),
                (Some(Role::ShieldBash), _) => "shield bashing".to_string(),
                (Some(Role::Style), _) => "every fighting style".to_string(),
            };
            let mut gains = Vec::new();
            for (value, what) in [(to_hit, "to hit"), (damage, "damage"), (shield, "Shield")] {
                if value != 0 {
                    gains.push(format!("{what} {}", signed(value)));
                }
            }
            if threat > 0 {
                gains.push(format!("threat range +{threat}"));
            }
            if crit > 0 {
                gains.push(format!("critical multiplier +{crit} (at most \u{d7}5)"));
            }
            if ranks > 0 {
                gains.push(format!("benefits as {ranks} rank higher (at most 10)"));
            }
            if speed > 0 {
                gains.push(format!("attacks {speed} % cheaper"));
            }
            if reload > 0 {
                gains.push(format!("reloading {reload} % cheaper"));
            }
            if ease > 0 {
                gains.push(format!("both two-weapon penalties eased by {ease}"));
            }
            if bash > 0 {
                gains.push(format!(
                    "a free shield bash's chance +{bash} % where there is one"
                ));
            }
            let rank = if at_rank > 0 {
                format!(" at rank {at_rank} or more")
            } else {
                String::new()
            };
            format!("{} with {with}{rank}", gains.join(", "))
        }
        TalentEffect::WeaponMarks { role, value } => {
            let role = match role {
                None => {
                    return format!(
                        "marks in every weapon skill and fighting style {}",
                        percent(value)
                    );
                }
                Some(Role::Launcher) => "missile".to_string(),
                Some(r) => r.id().replace('_', " "),
            };
            format!("marks in every {role} weapon skill {}", percent(value))
        }
        TalentEffect::TwoHandedCrit { value } => {
            format!("critical multiplier +{value} with a two-handed weapon (at most \u{d7}5)")
        }
        TalentEffect::UnarmedDie { sides } => {
            format!("blows with nothing in hand and kicks roll 1d{sides}")
        }
        TalentEffect::StanceDodge { stance, percent } => {
            format!("{} stance's Dodge penalty {percent} % smaller", stance.id())
        }
        TalentEffect::StanceCosts => {
            "changing your combat stance takes the time of a wait (it is free otherwise)".into()
        }
        TalentEffect::StepPercent {
            value,
            unarmoured,
            unburdened,
            no_shield,
        } => {
            let mut when = Vec::new();
            if unarmoured {
                when.push("in no body armour");
            }
            if unburdened {
                when.push("Unburdened");
            }
            if no_shield {
                when.push("with no shield");
            }
            let when = if when.is_empty() {
                String::new()
            } else {
                format!(" while {}", when.join(" and "))
            };
            format!("steps cost {}{when}", percent(value))
        }
        TalentEffect::Steadfast => "Burdened costs you nothing (as a dwarf)".to_string(),
        TalentEffect::Stout => {
            "Strained costs you what Burdened costs others (slowdown, Dexterity cap, noise)"
                .to_string()
        }
        TalentEffect::DiseaseImmune => "you never catch a disease".to_string(),
        TalentEffect::NoPenaltyTo { stat } => {
            format!("no status lowers your {}", stat.name())
        }
        TalentEffect::RaisePercent { status: id, value } => {
            format!(
                "what {} raises your scores by {}",
                status(&id),
                percent(value)
            )
        }
        TalentEffect::VeteransWind {
            per_level,
            at_least,
            cooldown,
        } => format!(
            "active (the time of a wait): {per_level} hit point per level back at once, at \
             least {at_least}; then {}",
            status(&cooldown)
        ),
        TalentEffect::LayOnHands {
            per_level,
            cooldown,
            degrees,
        } => format!(
            "active (the time of a cast): {per_level} hit points per level back at once; then \
             {degrees} degrees of {}",
            status(&cooldown)
        ),
        TalentEffect::Blaze {
            radius,
            damage,
            damage_type,
            fire,
            burns,
            cooldown,
            ..
        } => {
            let kind = (content.damage_type_id(&damage_type)).map_or(damage_type.clone(), |d| {
                content.damage_type(d).name.to_lowercase()
            });
            let light = (content.light_id(&fire)).map_or(0, |l| content.light(l).radius);
            let until = (content.status_id(&cooldown))
                .and_then(|s| content.status(s).eases)
                .map_or_else(String::new, |s| format!(" for {} h", s / 3600));
            format!(
                "active (the time of a cast; not on the overworld): a burst of radius {radius} \
                 around you, {damage} {kind} damage (one roll) to every creature in it but you, \
                 whole to each; it melts the ice it reaches and leaves a fire on every walkable \
                 cell of it but yours, its light reaching {light} cells, for {burns} s; then \
                 {}{until}",
                status(&cooldown)
            )
        }
        TalentEffect::Avatar => "Lay on hands heals every hit point and cures every status that \
             ticks (bleeding, poison) and every disease, and its cooldown is half as long"
            .to_string(),
        TalentEffect::Rage { resource: id } => {
            let rage = content.resource_id(&id).map(|r| content.resource(r));
            let gain = rage.map_or(id.clone(), |r| {
                format!("{} {}", r.start, r.name.to_lowercase())
            });
            let brings = rage.and_then(|r| r.status).map_or_else(String::new, |s| {
                format!(
                    ", and you are {} while you hold any",
                    content.status(s).name
                )
            });
            format!(
                "active (the time of a wait): you gain {gain} (see Resources){brings}; no rage \
                 while it or what follows it lasts"
            )
        }
        TalentEffect::ResourceBuild {
            resource: id,
            on,
            amount,
        } => format!(
            "{} builds {amount} more {} while you hold it",
            on.words(),
            resource_name(content, &id).to_lowercase()
        ),
        TalentEffect::ResourceMax {
            resource: id,
            value,
        } => {
            format!(
                "{} holds {value} more at most",
                resource_name(content, &id).to_lowercase()
            )
        }
        TalentEffect::ResourceStart {
            resource: id,
            value,
        } => format!(
            "{} starts {value} higher when you gain it",
            resource_name(content, &id).to_lowercase()
        ),
        TalentEffect::ResourceRate {
            resource: id,
            value,
        } => {
            let r = content.resource_id(&id).map(|r| content.resource(r));
            let way = match r.and_then(|r| r.over_time) {
                Some(OverTime::Decays { .. }) => "decay",
                _ => "regeneration",
            };
            format!(
                "{} {way} pace {}",
                resource_name(content, &id).to_lowercase(),
                percent(value)
            )
        }
        TalentEffect::Unstoppable { statuses, cost } => {
            let names: Vec<String> = statuses.iter().map(|s| status(s)).collect();
            format!(
                "while raging: {} tick nothing, and a blow that would take your last hit point \
                 leaves it for {cost} rage, when you have that much (spending the last of it \
                 ends the rage)",
                names.join(" and ")
            )
        }
        TalentEffect::Whirlwind => {
            "active (the time of a melee attack, one exertion): one blow at \
             every hostile beside you, clockwise from the north"
                .to_string()
        }
        TalentEffect::Smite {
            per_levels,
            sides,
            cooldown,
            degrees,
        } => format!(
            "active (the time of a melee attack), in a direction: a blow with 1d{sides} more \
             damage per {per_levels} levels (at least 1d{sides}); then {degrees} degree of {}",
            status(&cooldown)
        ),
        TalentEffect::Radiance { status: id, light } => {
            let reach = (content.light_id(&light)).map_or(0, |l| content.light(l).radius);
            format!(
                "active (the time of a cast), on and off: while you are {} you shed a light \
                 reaching {reach} cells",
                status(&id)
            )
        }
        TalentEffect::QuiveringPalm {
            status: id,
            severity,
        } => format!(
            "active (the time of a melee attack), in a direction, nothing in hand: a bare-handed \
             blow; a hit that does damage leaves {severity} degrees of {}",
            status(&id)
        ),
        TalentEffect::Hardened { bonus } => format!(
            "every level past the last that adds hit points adds them again, and {bonus} more"
        ),
        TalentEffect::LauncherRange { value, skill: None } => {
            format!("missile weapon range {} cells", signed(value))
        }
        TalentEffect::LauncherRange {
            value,
            skill: Some(skill),
        } => {
            let table = content.weapon_skills();
            let name = (table.id(&skill)).map_or(skill.clone(), |s| table.skill(s).name.clone());
            format!("range {} cells with launchers of {name}", signed(value))
        }
        TalentEffect::ThrowRange { value } => format!("throwing range {} cells", signed(value)),
        TalentEffect::MissileBreaks { value } => format!(
            "your missiles' chance to break when they hit {} (rounded down)",
            percent(value)
        ),
        TalentEffect::MaxDex { value } => {
            format!("worn armour's Dexterity cap {}", signed(value))
        }
        TalentEffect::Deflection { value } => format!("Deflection {}", signed(value)),
        TalentEffect::Protection {
            value,
            body_armour,
            shield,
            per_piece,
        } => {
            let mut text = format!("Protection {}", signed(value));
            if per_piece {
                text +=
                    " for every piece of armour worn (anything worn with Protection of its own)";
            }
            if let Some(at) = body_armour {
                text += &format!(
                    " while the armour piece worn with the most Protection has {at} or more"
                );
            }
            if shield {
                text += " while holding a shield";
            }
            text
        }
        TalentEffect::StanceToHit { stance, percent } => {
            format!(
                "{} stance's to-hit penalty {percent} % smaller",
                stance.id()
            )
        }
        TalentEffect::FleePercent { value } => format!("fleeing steps' cost {}", percent(value)),
        TalentEffect::Pain { value } => {
            format!(
                "every hit on you {} damage after the soak, never below 1",
                signed(-value)
            )
        }
        TalentEffect::Resist {
            value,
            damage: None,
        } => format!("resistance to the chosen element +{value} %"),
        TalentEffect::Resist {
            value,
            damage: Some(kind),
        } => {
            let name = (content.damage_type_id(&kind))
                .map_or(kind.clone(), |d| content.damage_type(d).name.to_lowercase());
            format!("resistance to {name} +{value} %")
        }
        TalentEffect::SlipPercent { value } => format!(
            "a step out of reach of every hostile beside you costs {}, whatever your stance",
            percent(value)
        ),
        TalentEffect::Streak { max, damage_every } => {
            let damage = if damage_every > 0 {
                format!(", and +1 damage for every {damage_every} blows before it")
            } else {
                String::new()
            };
            format!(
                "each melee blow at the creature your last blow was at (this turn or the one \
                 before) +1 to hit per blow before it, to +{max}{damage}; anything else starts \
                 it over"
            )
        }
        TalentEffect::OffHandDamage { value } => {
            format!("off-hand blows {} damage", signed(value))
        }
        TalentEffect::BestSkillThreat { value } => format!(
            "attacks with your best weapon skill (the highest rank you hold of the melee, \
             unarmed, launcher, and thrown skills; ties all count) threaten a critical {value} \
             lower"
        ),
        TalentEffect::Barbs { damage } => format!(
            "a creature that hits you in melee takes {damage} damage (no roll; a death of it \
             earns no experience)"
        ),
        TalentEffect::WoundMore { value } => {
            format!("a big hit that wounds you wounds {value} degree more")
        }
        TalentEffect::Distrusted => {
            "no one makes way for you, whatever else would have them (Noblesse too)".to_string()
        }
        TalentEffect::HornsGrace {
            cures,
            diseases,
            cooldown,
            degrees,
        } => {
            let names: Vec<String> = cures.iter().map(|c| status(c)).collect();
            format!(
                "active (the time of a wait): {} end and the first {diseases} of the diseases \
                 that show are shaken off (refused with nothing to mend); then {}",
                names.join(" and "),
                cooldown_for(content, &cooldown, degrees)
            )
        }
        TalentEffect::HornOfPlenty { per_level, degrees } => format!(
            "Horn's grace also gives back {per_level} hit point per level (never past your \
             maximum; it may then be used with only wounds to mend), and leaves {} instead",
            sign_cooldown(content, SignActive::HornsGrace, degrees)
        ),
        TalentEffect::Stare {
            status: id,
            severity,
            cooldown,
            degrees,
        } => {
            let (cost, eases) = (content.status_id(&id)).map_or((100, None), |s| {
                let def = content.status(s);
                (def.cost, def.eases)
            });
            let eases =
                eases.map_or_else(String::new, |e| format!(", a degree easing every {e} s"));
            format!(
                "active (the time of a wait, silent): a hostile you see within {} cells that \
                 sees you suffers {} at {severity} degrees (its actions cost {cost} %{eases}); \
                 one that does not see you is not held, and the cooldown waits; then {}",
                warlock_core::world::STARE_RANGE,
                status(&id),
                cooldown_for(content, &cooldown, degrees)
            )
        }
        TalentEffect::PetrifyingGaze { severity, degrees } => format!(
            "the Transfixing stare leaves {severity} degrees of its status, and leaves {} instead",
            sign_cooldown(content, SignActive::TransfixingStare, degrees)
        ),
        TalentEffect::Wildfire { radius, damage } => {
            format!("Blaze bursts out to radius {radius} and does {damage} instead")
        }
        TalentEffect::FireBreath {
            length,
            damage,
            damage_type,
            cooldown,
            degrees,
            ..
        } => {
            let kind = (content.damage_type_id(&damage_type)).map_or(damage_type.clone(), |d| {
                content.damage_type(d).name.to_lowercase()
            });
            format!(
                "active (the time of a cast; not on the overworld), aimed at a cell within {} \
                 cells: a bolt along the line toward it and on, {length} cells long or to the \
                 first wall, {damage} {kind} damage (one roll) to everyone on it; then {}",
                warlock_core::world::BREATH_RANGE,
                cooldown_for(content, &cooldown, degrees)
            )
        }
        TalentEffect::HoundsBay {
            status: id,
            severity,
            cooldown,
        } => {
            let eases = (content.status_id(&id))
                .and_then(|s| content.status(s).eases)
                .map_or_else(String::new, |e| format!(", a degree easing every {e} s"));
            format!(
                "active (the time of a wait; not on the overworld): a howl of loudness {}; \
                 every hostile that hears it and whose level is below yours suffers {} at \
                 {severity} degrees (it runs from you while a step takes it farther{eases}); \
                 no check; then {}",
                warlock_core::noise::HOWL_NOISE,
                status(&id),
                cooldown_for(content, &cooldown, 1)
            )
        }
        TalentEffect::RestingPercent { value } => {
            format!("healing pace while resting {}", percent(value))
        }
        TalentEffect::RiseFromAshes {
            hp_percent,
            status: id,
            radius,
            damage,
            damage_type,
            ..
        } => format!(
            "once in your life, a hit, a trap, or a status that would kill you does not (starving \
             and choking still do): you rise with {hp_percent} % of your maximum hit points \
             (Ashen ones), {damage} {damage_type} damage strikes every creature within {radius} \
             cell (one roll; no roll to hit) and the ice there melts, and you are {}",
            status(&id)
        ),
        TalentEffect::RebornWhole => "rising from the ashes brings you back whole: every hit \
             point, every status effect and disease gone (nothing follows them), and no Ashen"
            .to_string(),
        TalentEffect::SecondRising => {
            "you may rise from the ashes a second time in your life".to_string()
        }
        TalentEffect::StarvingHeals => {
            "hit points come back while you are Starving too (at your usual pace)".to_string()
        }
        TalentEffect::EatsRotten => "rotten food never harms you: no vomiting, no sickness, no \
             disease (as a race that eats rotten food)"
            .to_string(),
        TalentEffect::WadesFree => "wading (water, and ground a disease is caught wading in) \
             costs you a plain step; a disease may still be caught"
            .to_string(),
        TalentEffect::SecondWind {
            per_level,
            at_least,
            cooldown,
        } => format!(
            "active (the time of a wait): {per_level} hit point per level back at once, at \
             least {at_least}; then {} (no second wind while it lasts)",
            status(&cooldown)
        ),
        TalentEffect::IronStomach => "rotten food never makes you vomit and sickens you half \
             as much (degrees halved, rounded down); overeating never makes you vomit"
            .to_string(),
        TalentEffect::StatusEases { status: id, value } => {
            format!("{} eases {value} % faster", status(&id))
        }
        TalentEffect::NoPenalty { status: id } => {
            format!("{} never lowers your scores", status(&id))
        }
        TalentEffect::Clotting => {
            "a big hit wounds you no more; a confirmed critical one degree less".to_string()
        }
        TalentEffect::PenaltyPercent { value } => format!(
            "what statuses take off your scores {} (each, rounded toward 0)",
            percent(value)
        ),
        TalentEffect::DiseaseChecks { value } => {
            format!("disease checks (catching, progressions) {}", signed(value))
        }
        TalentEffect::DrainChecks { value } => {
            format!("checks against a drain {}", signed(value))
        }
        TalentEffect::Checks { stat, value } => {
            format!("every {} check {}", stat.name(), signed(value))
        }
        TalentEffect::DiseaseIntervalPercent { value } => format!(
            "a disease's progressions come {} further apart",
            percent(value)
        ),
        TalentEffect::DiseaseRecovery { value } => format!(
            "a disease goes after {value} fewer checks won in a row (at least 1; never \
             one that does not recover)"
        ),
        TalentEffect::KickDie { sides } => format!("kicks at creatures roll 1d{sides}"),
        TalentEffect::DoorKick { value } => {
            format!("door kicks as if Strength {}", signed(value))
        }
        TalentEffect::KickNoisePercent { value } => {
            format!("door kicks' noise {}", percent(value))
        }
        TalentEffect::DigPercent { value } => format!("digging work a turn {}", percent(value)),
        TalentEffect::TwoHandedPrice { value } => {
            format!("blows with a two-handed weapon cost {}", percent(value))
        }
        TalentEffect::CriticalStatus { status: id } => format!(
            "a critical you confirm in melee leaves the target {} (one degree)",
            status(&id)
        ),
        TalentEffect::Grip { status: id } => format!(
            "a creature you hit in melee is {} (it cannot step; beside you, it attacks you \
             instead) until you miss it, strike another, or move; no roll",
            status(&id)
        ),
        TalentEffect::CriticalConfirm { value } => {
            format!("the roll confirming your criticals {}", signed(value))
        }
        TalentEffect::Penetration { damage, value } => {
            let name = (content.damage_type_id(&damage)).map_or(damage.clone(), |d| {
                content.damage_type(d).name.to_lowercase()
            });
            format!(
                "blows that can do {name} damage: penetration {}",
                signed(value)
            )
        }
        TalentEffect::BurdenDex { value } => {
            format!("the burden's cap on Dexterity in Defence {}", signed(value))
        }
        TalentEffect::BurdenEase { value } => format!(
            "{value} % off the burden's slowdown, on top of Balance (Burdened ×1.10 for 60 %)"
        ),
        TalentEffect::SureStep => "a trap you step onto first gets a Dexterity check against \
             its difficulty: made, it does not go off, and you find it"
            .to_string(),
        TalentEffect::PitProof => "pits never catch you (you find them)".to_string(),
        TalentEffect::Disarm => {
            "the Disarm ability without Disable device (Dexterity alone)".to_string()
        }
        TalentEffect::SpeedPercent { value } => format!("speed {}", percent(value)),
        TalentEffect::MissileDodge { value } => {
            format!(
                "Dodge {} against shots, throws, and trap missiles",
                signed(value)
            )
        }
        TalentEffect::SmallTarget { value } => format!(
            "Dodge {} against the attacks of creatures larger than you",
            signed(value)
        ),
        TalentEffect::SpellDamagePercent { value } => format!(
            "damage from bursts and bolts (traps, scrolls) {} (rounded down)",
            percent(value)
        ),
        TalentEffect::Spell { spell } => content
            .class_spell(spell)
            .map_or_else(|| spell.id().to_string(), |def| spell_text(content, def)),
        TalentEffect::SpellPower { per_die, dice } => {
            let mut parts = Vec::new();
            if dice > 0 {
                parts.push(format!("{dice} more of their dice"));
            }
            if per_die != 0 {
                parts.push(format!("{} on every die", signed(per_die)));
            }
            format!(
                "your spells' and spell scrolls' damage: {}",
                parts.join(", and ")
            )
        }
        TalentEffect::SpellManaPercent { value } => {
            format!("your spells' mana {} (at least 1)", percent(value))
        }
        TalentEffect::DeflectionUnarmoured { value } => {
            format!("Deflection {} while you wear no body armour", signed(value))
        }
        TalentEffect::Purify => {
            "rotten food never sickens you and never exposes you to a disease (it may still make \
             you vomit)"
                .to_string()
        }
        TalentEffect::CorpseHarmless => {
            "a fresh corpse's own harm (its sickness and its vomiting) never touches you"
                .to_string()
        }
        TalentEffect::MealHeals { object, dice } => {
            let what = object.map_or_else(
                || "a fresh corpse".to_string(),
                |o| (content.object_id(&o)).map_or(o, |id| content.object(id).name.clone()),
            );
            format!("eating {what} heals {dice} hit points besides")
        }
        TalentEffect::DiseaseStill => "your diseases never progress: a failed check does no \
                                       harm (they still show until shaken off or cured)"
            .to_string(),
        TalentEffect::StatusDamage { status: id, value } => {
            format!(
                "{} damage on hits against a creature {}",
                signed(value),
                status(&id)
            )
        }
        TalentEffect::Reprieve {
            status: id,
            stat,
            by,
            ..
        } => {
            let stat = stat.map_or_else(String::new, |s| {
                format!(", and {} {} for good", s.name(), signed(by))
            });
            format!(
                "once in a life, what would kill you (not starving or choking) leaves you at 1 hit \
                 point, {}{stat}",
                status(&id)
            )
        }
        TalentEffect::FavouredPrey { to_hit, damage } => format!(
            "against the creature type you have killed most (ties: the first to get there), \
             to hit {} and damage {}",
            signed(to_hit),
            signed(damage)
        ),
        TalentEffect::ExactSounds => {
            "every sound you hear is placed at the cell it came from, not roughly".to_string()
        }
        TalentEffect::SkyFootstepsPercent { value } => format!(
            "footsteps under the open sky (no roof over the cell stepped to) {}",
            percent(value)
        ),
        TalentEffect::SilentUnburdened => "no footsteps at all while Unburdened".to_string(),
        TalentEffect::AttackNoisePercent { value } => {
            format!("the noise of your attacks {}", percent(value))
        }
        TalentEffect::TrapEase { value } => {
            format!("hidden traps {value} easier to find (not to disarm)")
        }
        TalentEffect::PickLock => "the Pick lock ability without Open lock".to_string(),
        TalentEffect::Steal => "the Steal ability without the Steal skill".to_string(),
        TalentEffect::ThrowVoice => "the Throw your voice ability".to_string(),
        TalentEffect::AmbushThreat { value } => format!(
            "attacks on a creature that has not seen you threaten a critical {value} lower \
             (never below 3)"
        ),
        TalentEffect::Finisher => "active (the time of a melee attack), in a direction: a blow \
                                   that is a critical whenever it hits, with no confirmation roll"
            .to_string(),
        TalentEffect::TrainedSkills { value } => {
            format!("every trained skill counts {}", signed(value))
        }
        TalentEffect::GrantsResource { resource } => {
            format!(
                "you hold {} from now on (see Resources)",
                resource_name(content, &resource)
            )
        }
        TalentEffect::RapidShot {
            resource,
            cost,
            saving,
        } => format!(
            "a shot you loose while you hold {cost} {} spends them and takes {saving} % less time",
            resource_name(content, &resource).to_lowercase()
        ),
        TalentEffect::FlurryCost { resource, cost } => format!(
            "each second blow of the flurry spends {cost} {}; with less, there is none",
            resource_name(content, &resource).to_lowercase()
        ),
        TalentEffect::Volley { shots } => format!(
            "active (the time of one shot, one exertion), at a cell: {shots} shots of the readied \
             missile weapon"
        ),
        TalentEffect::Inspire {
            status: id,
            mana,
            resource,
            cost,
        } => {
            let price = match resource {
                Some(r) => format!(
                    "{cost} {} while you hold that much, else {mana} mana",
                    resource_name(content, &r).to_lowercase()
                ),
                None => format!("{mana} mana"),
            };
            format!(
                "active (the time of a chat), {price}: {}",
                buff(content, &id)
            )
        }
        TalentEffect::Virtuoso { status: id } => {
            format!("Inspire gives {} instead, for nothing", buff(content, &id))
        }
    }
}

/// A caster's spell, as the class tables say it: what it does, its tier and mana, and its time.
fn spell_text(content: &Content, def: &SpellData) -> String {
    let casts = match def.casts {
        1 => "a cast".to_string(),
        n => format!("{n} casts"),
    };
    let mana = content.spell_rules().map_or(0, |r| {
        r.mana(def, i32::from(r.comfortable[usize::from(def.tier)]), 0)
    });
    format!(
        "spell `use:{}` (tier {}, {casts}, {mana} mana): {}: {}",
        def.id,
        def.tier,
        def.name,
        super::spells::does(content, def)
    )
}
