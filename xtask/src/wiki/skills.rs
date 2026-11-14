//! Skills (spec §8 "Skills"): what each does and trains on, what it gives by value, who trains
//! it, the provisional increase curve, the odds of stealing (spec §8 "Stealing"), how long an
//! intimidated foe flees (spec §8 "Skills": Intimidate), and what foraging finds (spec §8
//! "Foraging").

use super::util::{Term, anchored, capital, factor, link, percent_of};
use std::fmt::Write as _;
use warlock_core::burden::Burden;
use warlock_core::check::{CONTEST_WAYS, contest_ways, ways};
use warlock_core::content::{Content, OptionalSkill, RecipeCount, TalentEffect};
use warlock_core::dice::Dice;
use warlock_core::skills::{Effect, SkillId};

/// Every skill in the order the Skills screen lists them: what kind it is, what it is about,
/// what it gives (the benefits table has the numbers), and when it earns a training mark (with
/// the chance).
pub(super) fn skills(content: &Content) -> String {
    let mut out = String::from(
        "| Skill | Kind | What it is about | Gives | Trains on |\n|---|---|---|---|---|\n",
    );
    for skill in content.skills() {
        let gives: Vec<String> = (skill.benefits.iter())
            .map(|b| match gated_effect(b.effect) {
                true => format!("{} (**trained only**)", effect_name(b.effect)),
                false => effect_name(b.effect).to_string(),
            })
            .collect();
        let gives = if gives.is_empty() {
            "nothing yet".to_string()
        } else {
            gives.join("; ")
        };
        let trains: Vec<String> = (skill.marks.iter())
            .map(|m| format!("{} ({}%)", m.on.describe(), m.chance))
            .collect();
        let trains = if trains.is_empty() {
            "nothing yet".to_string()
        } else {
            trains.join("; ")
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {gives} | {trains} |",
            anchored(Term::Skill, &skill.id, &skill.name),
            skill.kind.name(),
            skill.description
        );
    }
    out
}

/// Whether `effect` is given only to those trained in its skill at all
/// (`Skills::trains_effect`: noticing in passing, identifying, disarming, picking, stealing, treating,
/// foraging): to the untrained, a magic item's bonus to the skill gives none of it.
pub(super) fn gated_effect(effect: Effect) -> bool {
    matches!(
        effect,
        Effect::Spot
            | Effect::Learn
            | Effect::Freshness
            | Effect::Identify
            | Effect::IdentifyPotion
            | Effect::IdentifyMagic
            | Effect::Herbs
            | Effect::Disarm
            | Effect::Escape
            | Effect::Climb
            | Effect::Pick
            | Effect::Wedge
            | Effect::Steal
            | Effect::Feint
            | Effect::Heal
            | Effect::Tame
            | Effect::Forage
            | Effect::ManaRegen
            | Effect::MaxMana
            | Effect::Swim
    )
}

/// Whether everything `skill` gives is for those trained in it only (a skill suffix of it helps
/// them only).
pub(super) fn gated(skill: &warlock_core::content::SkillDef) -> bool {
    !skill.benefits.is_empty() && skill.benefits.iter().all(|b| gated_effect(b.effect))
}

/// What an effect does, in words.
fn effect_name(effect: Effect) -> &'static str {
    match effect {
        Effect::Dodge => "a Dodge bonus to Defence",
        Effect::Quiet => "quieter steps and doors",
        Effect::Hearing => "keener hearing",
        Effect::Search => "finding hidden things when you search, not traps",
        Effect::SearchReach => "searching farther around you",
        Effect::Spot => "noticing hidden things in passing, not traps",
        Effect::DetectTraps => "finding hidden traps, searching and in passing",
        Effect::Study => "learning spells from grimoires",
        Effect::Cast => "the Spellcraft check every cast makes: fewer miscasts",
        Effect::Ward => "milder miscasts",
        Effect::Learn => "what a scroll teaches for certain, and reads for certain",
        Effect::Decipher => "reading a scroll you cannot simply read",
        Effect::Disarm => "disarming traps",
        Effect::Burden => "less slowdown from your load",
        Effect::Travel => "quicker journeys on the overworld",
        Effect::Forage => "foraging at all, and finding food more often",
        Effect::Bargain => "better prices in shops, and at a healer's or a sage's",
        Effect::Freshness => "telling how fresh food and corpses are",
        Effect::Identify => "now and then telling something you carry for what it is",
        Effect::IdentifyPotion => "now and then telling a potion or flask you carry for what it is",
        Effect::IdentifyMagic => "now and then telling a magic thing you carry for what it is",
        Effect::Herbs => "now and then a herb besides on a forage that finds something",
        Effect::Dig => "digging: more work a turn",
        Effect::Kick => "more Strength for kicking doors in",
        Effect::Slip => "slipping past eyes that see in the dark",
        Effect::Lookout => "seeing those you meet on the overworld before they see you",
        Effect::Stalk => "going unseen by those you meet on the overworld",
        Effect::Evade => "slipping away from those you meet on the overworld",
        Effect::Steal => "stealing at all, and better",
        Effect::Feint => "feinting at all, and your feints are harder to see through",
        Effect::SneakDice => "more damage on a foe that has not seen you or is surprised",
        Effect::Surprise => "those you catch unawares recover less often",
        Effect::Initiative => "keeping your wits when caught unawares",
        Effect::SurpriseEase => "less time lost when you are surprised",
        Effect::Heal => "treating your bleeding, poison, and diseases",
        Effect::Tame => "taming beasts at all, and better",
        Effect::Escape => "slipping bonds",
        Effect::Climb => "out of a pit quicker",
        Effect::Pick => "picking locks",
        Effect::Intimidate => {
            "your threats and your war cry are harder to stand against, and a threat frightens longer"
        }
        Effect::SongDegrees => "your songs last longer",
        Effect::SongCraft => "your songs are harder to shake off",
        Effect::Wedge => "a door you wedge shut is harder to kick in",
        Effect::Temper => "tempering at a forge's anvil",
        Effect::Whittle => "whittling wood with a knife",
        Effect::Craft => "its craft: making its recipes and salvaging the materials it works",
        Effect::EnchantLevel => "the item level your enchanting rolls its magic at",
        Effect::Wade => "wading through water and marsh is quicker",
        Effect::Footing => "walking over rough ground (bush, reeds, rubble, ice) is quicker",
        Effect::SteadyEase => "less time lost when you are staggered",
        Effect::Swim => {
            "a plunge through the ice may hurt half as much, and a step in deep water goes as you meant it"
        }
        Effect::ManaRegen => "mana back faster",
        Effect::MaxMana => "a slightly larger mana pool (at most a fifth more)",
        Effect::Speed => "quicker in everything",
        Effect::Resist => "resisting the powers of creatures it stands above",
    }
}

/// The values the benefits table shows (#1545: the skill's value as a column, few enough to read).
const BENEFIT_VALUES: [u16; 6] = [0, 10, 20, 50, 100, 200];

/// What each skill's benefits give at a range of values, from the data: one row per benefit, the
/// skill named on its first row, the values as columns (#1545).
pub(super) fn skill_benefits(content: &Content) -> String {
    let mut out = String::from("| Skill | Benefit |");
    for value in BENEFIT_VALUES {
        let _ = write!(out, " {value} |");
    }
    out.push_str("\n|---|---|");
    out.push_str(&"---:|".repeat(BENEFIT_VALUES.len()));
    out.push('\n');
    for skill in content.skills() {
        // Knowledge's resist is read above a creature's level, not at the value: its own table.
        let benefits = skill.benefits.iter().filter(|b| b.effect != Effect::Resist);
        for (i, b) in benefits.enumerate() {
            let name = if i == 0 {
                link(Term::Skill, &skill.id, &skill.name)
            } else {
                String::new()
            };
            let _ = write!(out, "| {name} | {} |", benefit_words(b.effect));
            for value in BENEFIT_VALUES {
                let _ = write!(out, " {} |", benefit_cell(b, value));
            }
            out.push('\n');
        }
    }
    out
}

/// A benefit's name in the benefits table.
fn benefit_words(effect: Effect) -> &'static str {
    match effect {
        Effect::Dodge => "Dodge",
        Effect::Quiet => "noise",
        Effect::Hearing => "hears fainter by",
        Effect::Search => "searching",
        Effect::SearchReach => "search reaches",
        Effect::Spot => "noticing",
        Effect::DetectTraps => "finding traps",
        Effect::Study => "study check",
        Effect::Cast => "casting check",
        Effect::Ward => "off a miscast's total",
        Effect::Learn => "learning",
        Effect::Decipher => "reading scrolls",
        Effect::Disarm => "disarming traps",
        Effect::Burden => "Burdened / Strained",
        Effect::Travel => "journeys",
        Effect::Forage => "foraging",
        Effect::Bargain => "prices",
        Effect::Freshness => "telling freshness, each hour",
        Effect::Identify => "identifying, each hour",
        Effect::IdentifyPotion => "telling potions, each hour",
        Effect::IdentifyMagic => "telling magic, each hour",
        Effect::Herbs => "a herb besides",
        Effect::Dig => "digging",
        Effect::Kick => "kicking doors in",
        Effect::Slip => "slipping past",
        Effect::Lookout => "lookout",
        Effect::Stalk => "their lookout",
        Effect::Evade => "slipping away",
        Effect::Steal => "stealing",
        Effect::Feint => "feint's difficulty",
        Effect::SneakDice => "sneak attack dice",
        Effect::Surprise => "their check's difficulty",
        Effect::Initiative => "initiative check",
        Effect::SurpriseEase => "surprise's extra cost",
        Effect::Heal => "treating",
        Effect::Tame => "taming",
        Effect::Escape => "slipping bonds",
        Effect::Climb => "a pit's climb",
        Effect::Pick => "picking locks",
        Effect::Intimidate => "threat's difficulty",
        Effect::SongDegrees => "song degrees",
        Effect::SongCraft => "song's difficulty",
        Effect::Wedge => "wedged door's sturdiness",
        Effect::Temper => "tempering",
        Effect::Whittle => "whittling",
        Effect::Craft => "craft",
        Effect::EnchantLevel => "enchanting's item level",
        Effect::Wade => "wading's extra cost",
        Effect::Footing => "rough ground's extra cost",
        Effect::SteadyEase => "staggering's extra cost",
        Effect::Swim => "Swim check",
        Effect::ManaRegen => "mana back",
        Effect::MaxMana => "mana pool",
        Effect::Speed => "speed",
        Effect::Resist => "resisting",
    }
}

/// What a benefit gives at a value, as the benefits table says it.
fn benefit_cell(b: &warlock_core::skills::Benefit, value: u16) -> String {
    let n = b.at(value);
    match b.effect {
        _ if n == 0 => "—".to_string(),
        Effect::Dodge
        | Effect::Search
        | Effect::Spot
        | Effect::DetectTraps
        | Effect::Study
        | Effect::Cast
        | Effect::Decipher
        | Effect::Slip
        | Effect::Lookout
        | Effect::Stalk
        | Effect::Evade
        | Effect::Steal
        | Effect::Feint
        | Effect::Heal
        | Effect::Tame
        | Effect::Disarm
        | Effect::Escape
        | Effect::Pick
        | Effect::Intimidate
        | Effect::SongDegrees
        | Effect::SongCraft
        | Effect::Wedge
        | Effect::Temper
        | Effect::Whittle
        | Effect::Craft
        | Effect::Swim
        | Effect::Surprise
        | Effect::Initiative
        | Effect::Resist => format!("+{n}"),
        Effect::Climb if n == 1 => "halved".to_string(),
        Effect::Climb => "skipped".to_string(),
        Effect::SneakDice => format!("{n}d6"),
        Effect::SurpriseEase | Effect::Wade | Effect::Footing | Effect::SteadyEase => {
            format!("\u{2212}{n}%")
        }
        Effect::SearchReach => format!("+{n} cell{}", if n == 1 { "" } else { "s" }),
        Effect::Kick => format!("+{n} Strength"),
        Effect::Quiet | Effect::Travel => format!("\u{2212}{n}%"),
        Effect::Ward => format!("\u{2212}{n}"),
        Effect::Bargain => format!("{n}% better"),
        Effect::Forage => format!("+{n}%"),
        Effect::Freshness
        | Effect::Identify
        | Effect::IdentifyPotion
        | Effect::IdentifyMagic
        | Effect::Herbs => format!("{n}%"),
        Effect::Dig => format!("+{n}%"),
        Effect::Speed | Effect::ManaRegen => format!("+{n}%"),
        Effect::MaxMana => format!("+{n} mana"),
        Effect::Hearing | Effect::Learn => n.to_string(),
        Effect::EnchantLevel => format!("item level {n}"),
        Effect::Burden => {
            let slowed = |b: Burden| factor(b.slow_eased(100, n));
            format!(
                "×{} / ×{}",
                slowed(Burden::Burdened),
                slowed(Burden::Strained)
            )
        }
    }
}

/// The margins the resist table shows: how far a skill's value stands above a creature's level.
const RESIST_MARGINS: [u16; 12] = [0, 1, 4, 5, 10, 15, 20, 25, 30, 35, 40, 60];

/// What the skills with the resist benefit (Knowledge, spec §8 "Skills") add to your check
/// against a creature's drain, a disease its attack carries, or its hold, by how far the skill's
/// value stands above the creature's level.
pub(super) fn skill_resist(content: &Content) -> String {
    let columns: Vec<(&str, &warlock_core::skills::Benefit)> = (content.skills().iter())
        .flat_map(|s| s.benefits.iter().map(move |b| (s.name.as_str(), b)))
        .filter(|(_, b)| b.effect == Effect::Resist)
        .collect();
    let mut out = String::from("| Above the creature's level |");
    for (name, _) in &columns {
        let _ = write!(out, " {name}: resisting |");
    }
    out.push_str("\n|---:|");
    out.push_str(&":---:|".repeat(columns.len()));
    out.push('\n');
    for margin in RESIST_MARGINS {
        let _ = write!(out, "| {margin} |");
        for (_, b) in &columns {
            let n = b.at(margin);
            let cell = if n == 0 {
                "—".to_string()
            } else {
                format!("+{n}")
            };
            let _ = write!(out, " {cell} |");
        }
        out.push('\n');
    }
    out
}

/// A skill's name, for the grants tables.
fn skill_name(content: &Content, id: SkillId) -> &str {
    &content.skill(id).name
}

/// What each race and each class trains, as dice (`1d6+4`), in the order New run offers them.
pub(super) fn skill_grants(content: &Content) -> String {
    let list = |grants: &[(SkillId, Dice)]| -> String {
        let parts: Vec<String> = (grants.iter())
            .map(|&(id, dice)| format!("{} `{dice}`", skill_name(content, id)))
            .collect();
        parts.join(", ")
    };
    // The optional skills, each by its dice and chance (the class's are rolled first, then the
    // race's).
    let optional = |skills: &[OptionalSkill]| -> String {
        let parts: Vec<String> = (skills.iter())
            .map(|o| {
                let name = skill_name(content, o.skill);
                format!("{name} `{}` ({} %)", o.dice, o.chance)
            })
            .collect();
        match parts.is_empty() {
            true => "-".into(),
            false => parts.join(", "),
        }
    };
    let mut out = String::from("| Race | Trains | May start with |\n|---|---|---|\n");
    for r in &content.races().list {
        let mut trains = list(&r.skills);
        if !r.class_skill.is_zero() {
            if !trains.is_empty() {
                trains.push_str("; ");
            }
            let _ = write!(
                trains,
                "one of the class's skills `+{}`, picked at random",
                r.class_skill
            );
        }
        if trains.is_empty() {
            trains = "-".into();
        }
        let may = optional(&r.optional_skills);
        let _ = writeln!(out, "| {} | {trains} | {may} |", r.name);
    }
    out.push_str("\n| Class | Trains | May start with |\n|---|---|---|\n");
    for c in content.classes() {
        let may = optional(&c.optional_skills);
        let mut trains = list(&c.skills);
        if let Some(choice) = &c.skill_choice {
            let names: Vec<&str> = (choice.skills.iter())
                .map(|&id| skill_name(content, id))
                .collect();
            let _ = write!(
                trains,
                "; one of {} `{}`, picked at random",
                names.join(", "),
                choice.dice
            );
        }
        let _ = writeln!(out, "| {} | {trains} | {may} |", c.name);
    }
    out
}

/// How often a theft goes unnoticed (spec §8 "Stealing"), by the thief's edge: their Dexterity
/// modifier plus what their Steal skill gives, less the victim's Willpower modifier. The
/// contest's dice are the same on both sides, so only the difference counts.
pub(super) fn stealing(_content: &Content) -> String {
    let mut out = String::from("| Your edge | Unnoticed | Caught |\n|:---:|:---:|:---:|\n");
    for edge in -6..=10 {
        let won = contest_ways(edge, 0);
        let sign = if edge < 0 { "\u{2212}" } else { "+" };
        let _ = writeln!(
            out,
            "| {sign}{} | {} | {} |",
            edge.abs(),
            percent_of(won, CONTEST_WAYS),
            percent_of(CONTEST_WAYS - won, CONTEST_WAYS)
        );
    }
    out
}

/// How often a foe is fooled by the player's feint (spec §8 "Skills": Bluff; `data/feint.toml`),
/// by the edge of their Presence modifier and what Bluff adds over its Willpower modifier.
pub(super) fn feinting(content: &Content) -> String {
    let Some(rules) = content.feint() else {
        return "Nobody feints.\n".into();
    };
    let status = &content.status(rules.status).name;
    let mut out = format!(
        "The foe rolls 2d12 + its Willpower modifier against {} + your Presence modifier + what \
         Bluff adds; short of it, it is {status}.\n\n\
         | Your Presence and Bluff, less its Willpower | Fooled | Not fooled |\n|---:|---:|---:|\n",
        rules.difficulty
    );
    for edge in [-6, -3, 0, 3, 6, 9, 12] {
        let made = ways(0, rules.difficulty + edge);
        let sign = if edge < 0 { "\u{2212}" } else { "+" };
        let _ = writeln!(
            out,
            "| {sign}{} | {} | {} |",
            edge.abs(),
            percent_of(144 - made, 144),
            percent_of(made, 144)
        );
    }
    out
}

/// How long a foe that fails to keep its nerve against the player's Intimidate flees (spec §8
/// "Skills": Intimidate; `data/intimidate.toml`): each rung of the ladder, by the points
/// Intimidate adds and the lowest value of the skill that adds them.
pub(super) fn intimidating(content: &Content) -> String {
    let Some(rules) = content.intimidate() else {
        return "Nobody intimidates anyone.\n".into();
    };
    let benefit = (content.skills().iter())
        .flat_map(|s| s.benefits.iter())
        .find(|b| b.effect == Effect::Intimidate);
    let status = &content.status(rules.status).name;
    let mut out = format!(
        "Within {} cells; the check is {} + your Strength modifier + what Intimidate adds.\n\n\
         | Intimidate | Adds | {status} for |\n|---:|---:|:---:|\n",
        rules.range, rules.difficulty
    );
    for &(points, degrees) in &rules.degrees {
        let value = (0..=warlock_core::skills::MAX_VALUE)
            .find(|&v| benefit.is_some_and(|b| b.at(v) >= points))
            .map_or("—".to_string(), |v| format!("{v}+"));
        let plural = if degrees == 1 { "" } else { "s" };
        let _ = writeln!(out, "| {value} | +{points} | {degrees} degree{plural} |");
    }
    out
}

/// How often the player slips past a creature that would see them by darkvision alone (spec §8
/// "Surprise and sneak attacks"; `data/surprise.toml` `[slip]`), by their edge: their Dexterity
/// modifier plus Stealth's `slip`, less the creature's Willpower modifier and Spot. The same
/// contest as a theft's: only the difference counts.
pub(super) fn slipping_past(content: &Content) -> String {
    let Some(turns) = content.surprise().and_then(|r| r.slip_turns) else {
        return "Nobody slips past anyone.\n".into();
    };
    let mut out = format!(
        "A creature you slipped past stays blind to you for {turns} turns at most.\n\n\
         | Your edge | Unseen | Seen |\n|:---:|:---:|:---:|\n"
    );
    for edge in -6..=10 {
        let won = contest_ways(edge, 0);
        let sign = if edge < 0 { "\u{2212}" } else { "+" };
        let _ = writeln!(
            out,
            "| {sign}{} | {} | {} |",
            edge.abs(),
            percent_of(won, CONTEST_WAYS),
            percent_of(CONTEST_WAYS - won, CONTEST_WAYS)
        );
    }
    out
}

/// What a forage finds on each land (spec §8 "Foraging"; `data/forage.toml`), by landscape:
/// the chance before Survival, how many of a find, and each find's share, by name.
pub(super) fn foraging(content: &Content) -> String {
    let Some(rules) = content.forage() else {
        return "Nobody forages.\n".into();
    };
    let mut out = format!(
        "A forage takes {} minutes.\n\n| Land | Chance | How many | Finds | Herbs (Herbalism) |\n|---|:---:|:---:|---|---|\n",
        rules.seconds / 60
    );
    let mut grounds: Vec<_> = rules.grounds.iter().collect();
    grounds.sort_by_key(|g| g.landscape);
    for g in grounds {
        let total = g.total_weight();
        let mut finds: Vec<(String, u32)> = (g.finds.iter())
            .map(|&(o, w)| (content.object(o).name.clone(), w))
            .collect();
        finds.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let finds: Vec<String> = (finds.iter())
            .map(|(name, w)| format!("{name} {}", percent_of(*w, total)))
            .collect();
        let herb_total: u32 = g.herbs.iter().map(|(_, w)| *w).sum();
        let mut herbs: Vec<(String, u32)> = (g.herbs.iter())
            .map(|&(o, w)| (content.object(o).name.clone(), w))
            .collect();
        herbs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let herbs: Vec<String> = (herbs.iter())
            .map(|(name, w)| format!("{name} {}", percent_of(*w, herb_total)))
            .collect();
        let herbs = if herbs.is_empty() {
            "—".to_string()
        } else {
            herbs.join(", ")
        };
        let _ = writeln!(
            out,
            "| {} | {}% | `{}` | {} | {herbs} |",
            g.landscape.id().replace('_', " "),
            g.chance,
            g.count,
            finds.join(", ")
        );
    }
    out
}

/// Fletching's numbers (spec §8 "Fletching"; the `count = "fletch"` recipes, `data/fletch.toml`,
/// `data/recipes.toml`'s bands, and Fletcher's effect): the time, the check, the count, the
/// margin's bands, and what each head makes.
pub(super) fn fletching(content: &Content) -> String {
    let Some(rules) = content.fletch() else {
        return "Nobody fletches.\n".into();
    };
    let recipes = content.recipes();
    let fletch =
        (recipes.list.iter()).find(|r| r.result.is_some_and(|x| x.count == RecipeCount::Fletch));
    let (Some(arrows), Some(settings)) = (fletch, recipes.settings.as_ref()) else {
        return "Nobody fletches.\n".into();
    };
    let fletcher = (content.talents().iter())
        .flat_map(|(_, def)| &def.effects)
        .find_map(|e| match e {
            TalentEffect::Fletcher {
                check,
                count_times,
                time_percent,
            } => Some((*check, *count_times, *time_percent)),
            _ => None,
        });
    let (check, times, time) = fletcher.unwrap_or((0, 1, 100));
    let seconds = arrows.seconds;
    let per = rules.value_per_count;
    let skill = &content.skill(arrows.skill).name;
    let mut out = String::from("| | Anyone | With Fletcher |\n|---|:---:|:---:|\n");
    let _ = writeln!(
        out,
        "| Time | {} minutes | {} minutes |",
        seconds / 60,
        seconds * time / 100 / 60
    );
    let _ = writeln!(
        out,
        "| Check | 2d12 + Dexterity modifier + talents + {skill}'s craft points, against {} \
         and the head's work | the same, +{check} |",
        arrows.difficulty
    );
    let _ = writeln!(
        out,
        "| Count | `{}` + {skill}'s value ÷ {per} | the same |",
        rules.count
    );
    let _ = writeln!(
        out,
        "| Made, check {0} or more | the whole count | {times} times the count |",
        arrows.difficulty
    );
    let _ = writeln!(
        out,
        "| Made, up to {} short | half the count, rounded up | {times} times that |",
        settings.half_within
    );
    let _ = writeln!(
        out,
        "| Made, up to {} short | one | {times} |",
        settings.one_within
    );
    let _ = writeln!(out, "| Made, shorter still | none | none |");
    out
}

/// What the Heal skill treats (spec §8 "Treating oneself"): each status with a `treat` table at
/// its least and most degrees, then each disease, with the check's difficulty, the chance of
/// making it at a modifier of +0 and of +8 (Intelligence modifier plus Heal), and what comes of
/// a made and a failed try. Statuses in file order, diseases by name.
pub(super) fn heal_targets(content: &Content) -> String {
    let mut out = String::from(
        "| Ailment | Difficulty | Chance at +0 | Chance at +8 | Made | Failed |\n\
         |---|---:|---:|---:|---|---|\n",
    );
    let odds = |difficulty: i32| {
        let at = |m| percent_of(warlock_core::check::ways(m, difficulty), 144);
        (at(0), at(8))
    };
    for (_, def) in content.status_ids() {
        let Some(treat) = def.treat.as_ref() else {
            continue;
        };
        let (made, failed) = if treat.stops {
            ("it stops", "try again as often as you like")
        } else {
            ("it eases a degree", "no more tries at this dose")
        };
        let degrees = if def.max > 1 {
            vec![1, def.max]
        } else {
            vec![1]
        };
        for severity in degrees {
            let difficulty = treat.difficulty_at(severity);
            let (zero, eight) = odds(difficulty);
            let s = if severity == 1 { "" } else { "s" };
            let _ = writeln!(
                out,
                "| {} ({severity} degree{s}) | {difficulty} | {zero} | {eight} | {made} | {failed} |",
                def.name
            );
        }
    }
    let mut diseases: Vec<_> = content.disease_defs().iter().collect();
    diseases.sort_by(|a, b| a.name.cmp(&b.name));
    for def in diseases {
        let (zero, eight) = odds(def.difficulty);
        let made = if def.recovers() {
            "counts as a check won toward recovery (never the last one needed)"
        } else {
            "its next turn for the worse comes a full interval later"
        };
        let _ = writeln!(
            out,
            "| {} | {} | {zero} | {eight} | {made} | no more tries at this bout |",
            capital(&def.name),
            def.difficulty
        );
    }
    out
}

/// The skill ladder: the die a level-up roll throws by the skill's marks, and the die a skill the
/// class is known for throws (one step up).
pub(super) fn skill_ladder(content: &Content) -> String {
    let ladder = content.ladder();
    let mut out = String::from(
        "| Marks | Die | Average | Known for: die | Known for: average |\n|---:|---|---:|---|---:|\n",
    );
    let average = |d: Dice| format!("{:.1}", f64::from(d.min() + d.max()) / 2.0);
    for (step, (&marks, &die)) in ladder.marks.iter().zip(&ladder.dice).enumerate() {
        let up = ladder.die(step + 1);
        let _ = writeln!(
            out,
            "| {marks} | `{die}` | {} | `{up}` | {} |",
            average(die),
            average(up)
        );
    }
    out
}

/// The skill rolls each class gives a level and the skills it is known for, then the races'
/// extra rolls, each by name; and what Intelligence adds.
pub(super) fn skill_rolls(content: &Content) -> String {
    let names = |ids: &[SkillId]| -> String {
        let parts: Vec<&str> = ids.iter().map(|&id| skill_name(content, id)).collect();
        if parts.is_empty() {
            "-".into()
        } else {
            parts.join(", ")
        }
    };
    let mut out = String::from("| Class | Rolls a level | Known for |\n|---|---:|---|\n");
    let mut classes: Vec<_> = content.classes().iter().collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));
    for c in classes {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            c.name,
            c.rolls,
            names(&c.known_for)
        );
    }
    out.push_str("\n| Race | Extra rolls a level |\n|---|---:|\n");
    let mut races: Vec<_> = content.races().list.iter().collect();
    races.sort_by(|a, b| a.name.cmp(&b.name));
    for r in races {
        let _ = writeln!(out, "| {} | {} |", r.name, r.rolls);
    }
    let rules = content.skill_rolls();
    let _ = writeln!(
        out,
        "\nIntelligence adds one roll at {} and one more for every {} points above it \
         ({}: +1, {}: +2, {}: +3).",
        rules.intelligence_from,
        rules.intelligence_every,
        rules.intelligence_from,
        u16::from(rules.intelligence_from) + u16::from(rules.intelligence_every),
        u16::from(rules.intelligence_from) + 2 * u16::from(rules.intelligence_every),
    );
    if rules.auto_at > 0 {
        let _ = writeln!(
            out,
            "\nA skill whose marks reach {} rolls at once by itself, besides these.",
            rules.auto_at
        );
    }
    out
}
