//! Talents (spec §8 "Talents"): every tree with its talents, their prerequisites and their rules
//! in full (the game says them in words), and the coverage matrix by system.

use super::text::signed;
use std::fmt::Write as _;
use warlock_core::content::{Content, Need, Role, TalentDef, TalentEffect, TreeId};

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
                // A birth sign's gift is marked: it is given, never picked.
                if t.granted {
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
            format!("yes ({} talents at least)", bar.min_talents)
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
fn needs_text(content: &Content, def: &TalentDef) -> String {
    if def.granted {
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
    parts.join("; ")
}

/// One effect as the rule (talents' and class abilities').
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
        TalentEffect::Threat { value } => {
            format!("threat range {} with every attack", signed(value))
        }
        TalentEffect::RerollFumble => "a natural 2 on the attack roll is rolled again, once".into(),
        TalentEffect::MaxHpPerLevels {
            per_levels,
            at_least,
        } => format!("+1 maximum hit point per {per_levels} levels, at least +{at_least}"),
        TalentEffect::Stat { stat, value } => {
            format!("{} {} (score and potential)", stat.name(), signed(value))
        }
        TalentEffect::HealingPercent { value } => format!("healing pace {}", percent(value)),
        TalentEffect::FootstepsPercent { value } => format!("footsteps {}", percent(value)),
        TalentEffect::SilentDoors => "the doors you open and close are silent".to_string(),
        TalentEffect::Hearing { value } => format!("hears sounds {value} fainter"),
        TalentEffect::CarryPercent { value } => format!("carrying limit {}", percent(value)),
        TalentEffect::CarryKg { value } => format!("carrying limit {} kg", signed(value)),
        TalentEffect::WeightlessCoins => "coins you carry weigh nothing".to_string(),
        TalentEffect::TravelPercent { value } => {
            format!("overworld journey time {}", percent(value))
        }
        TalentEffect::Swimmer => {
            "wading (water, marsh) costs a plain step and never gives a disease".into()
        }
        TalentEffect::HungerPercent { value } => format!("hunger pace {}", percent(value)),
        TalentEffect::RotHarmPercent { value } => {
            format!(
                "sickness from rotten food {} (rounded down)",
                percent(value)
            )
        }
        TalentEffect::NoExertion => "blows, shots, and throws burn no nutrition".to_string(),
        TalentEffect::XpPercent { value } => format!("experience gained {}", percent(value)),
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
        TalentEffect::ItemWorth => "item descriptions show what things are worth".to_string(),
        TalentEffect::GemWorth => "item descriptions show what each gem is worth".to_string(),
        TalentEffect::IdentifyCarried { hours } => format!(
            "a magic item you carry for {hours} hours (counted from when you gain this) is \
             identified"
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
        TalentEffect::StoneSense => "notices hidden things on stone in passing".into(),
        TalentEffect::Search { value } => {
            format!("{} to every check to find hidden things", signed(value))
        }
        TalentEffect::SearchReach { value } => format!(
            "searching reaches {value} cell{} farther",
            if value == 1 { "" } else { "s" }
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
                Role::Launcher => "missile".to_string(),
                r => r.id().replace('_', " "),
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
        TalentEffect::Avatar => "Lay on hands heals every hit point and cures every status that \
             ticks (bleeding, poison) and every disease, and its cooldown is half as long"
            .to_string(),
        TalentEffect::Rage { status: id } => format!(
            "active (the time of a wait): you suffer {}; no rage while it or what follows it \
             lasts",
            status(&id)
        ),
        TalentEffect::BloodFrenzy { seconds } => {
            format!(
                "a kill of yours while raging keeps the rage {seconds} s longer (never past its full length)"
            )
        }
        TalentEffect::Unstoppable { statuses } => {
            let names: Vec<String> = statuses.iter().map(|s| status(s)).collect();
            format!(
                "while raging: {} tick nothing, and a blow that would take your last hit point \
                 leaves it and ends the rage",
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
            cooldown,
            degrees,
        } => format!(
            "active (the time of a melee attack), in a direction, nothing in hand: a bare-handed \
             blow; a hit that does damage leaves {severity} degrees of {}; then {degrees} degrees \
             of {}",
            status(&id),
            status(&cooldown)
        ),
        TalentEffect::Hardened { bonus } => format!(
            "every level past the last that adds hit points adds them again, and {bonus} more"
        ),
        TalentEffect::LauncherRange { value } => {
            format!("missile weapon range {} cells", signed(value))
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
        TalentEffect::Resist { value } => format!("resistance to the chosen element +{value} %"),
        TalentEffect::RestingPercent { value } => {
            format!("healing pace while resting {}", percent(value))
        }
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
        TalentEffect::Checks { stat, value } => {
            format!("every {} check {}", stat.name(), signed(value))
        }
        TalentEffect::DiseaseRecovery { value } => format!(
            "a disease goes after {value} fewer checks won in a row (at least 1; never \
             one that does not recover)"
        ),
        TalentEffect::KickDie { sides } => format!("kicks at creatures roll 1d{sides}"),
        TalentEffect::DoorKick { value } => {
            format!("door kicks as if Strength {}", signed(value))
        }
        TalentEffect::DigPercent { value } => format!("digging work a turn {}", percent(value)),
        TalentEffect::TwoHandedPrice { value } => {
            format!("blows with a two-handed weapon cost {}", percent(value))
        }
        TalentEffect::CriticalStatus { status: id } => format!(
            "a critical you confirm in melee leaves the target {} (one degree)",
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
        TalentEffect::SpellDamagePercent { value } => format!(
            "damage from bursts and bolts (traps, scrolls) {} (rounded down)",
            percent(value)
        ),
    }
}
