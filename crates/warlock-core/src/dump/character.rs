//! The `stats`, `skills`, and `talents` layers: the player's character.

use super::*;

/// The player's character, if any (`Name Brakka`, `Race Dwarf`, `Gender Female`, `Class
/// Warrior`, `Age 27`, `Born 3 Seedtide 1417`, `Sign Fire Manticore`, with a lucky day's title
/// `Sign Metal Manticore, the Iron Scourge`), then their own scores, one stat a line:
/// `Strength 12`.
pub(super) fn stat_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let mut rows = Vec::new();
    if let Some(c) = view.character() {
        let spec = &c.spec;
        rows.push(format!("Name {}", c.name));
        let race = spec
            .race(content)
            .map_or(spec.race.as_str(), |r| r.name.as_str());
        // The classes as the character screen's Class row says them (spec §8 "Prestige
        // classes"): `Bard`, `Bard 30` at the cap, `Bard 30, Hunter 4` once called.
        let class = c.class_text(content, view.own_experience().level);
        rows.push(format!("Race {race}"));
        rows.push(match spec.gender {
            Gender::Other => format!("Gender Other ({} +1)", c.raised.name()),
            gender => format!("Gender {}", gender.name()),
        });
        rows.push(format!("Class {class}"));
        rows.push(format!("Age {}", c.age_on(view.time())));
        rows.push(format!("Born {}", c.born.text(content.calendar())));
        if let Some(sign) = c.sign_title(content) {
            rows.push(format!("Sign {sign}"));
        }
    }
    rows.extend((view.own_scores().iter()).map(|(stat, score)| format!("{} {score}", stat.name())));
    if let Some(band) = view.own_alignment() {
        rows.push(format!("Alignment {}", band.name()));
    }
    // The combat stance by name, when not neutral (as the status bar shows it).
    let stance = view.own_stance();
    if !stance.is_neutral() {
        rows.push(format!("Stance {}", content.stance_def(stance).name));
    }
    // The guilds they belong to and their ranks (spec §8 "Quests": "Guilds"), a title only.
    for g in view.own_guilds() {
        rows.push(format!("Guild {}, {}", g.guild.bare_name(), g.title()));
    }
    let effects: Vec<&str> = (view.own_effects())
        .map(|s| content.status(s).name.as_str())
        .collect();
    if !effects.is_empty() {
        rows.push(format!("Effects {}", effects.join(", ")));
    }
    // Whose guard their openings are in (`world/classes/mark.rs`), their own choice.
    if let Some(mark) = view.own_mark() {
        rows.push(format!("Mark {mark}"));
    }
    let diseases: Vec<&str> = (view.own_diseases())
        .map(|d| content.disease(d).name.as_str())
        .collect();
    if !diseases.is_empty() {
        rows.push(format!("Diseases {}", diseases.join(", ")));
    }
    rows.extend(resistance_rows(view));
    rows
}

/// The player's resistances in words (spec §8 "Damage types"), a row per word: `Resistant to
/// fire, cold`, `Vulnerable to acid`, `Immune to poison`; never a percentage.
pub(super) fn resistance_rows(view: &PerceptionView<'_>) -> Vec<String> {
    use crate::damage::ResistanceWord;
    let content = view.content();
    let own = view.own_resistances();
    [
        ResistanceWord::Immune,
        ResistanceWord::Resistant,
        ResistanceWord::Vulnerable,
    ]
    .into_iter()
    .filter_map(|word| {
        let kinds: Vec<&str> = (own.iter())
            .filter(|(_, w)| *w == word)
            .map(|(k, _)| content.damage_type(*k).name.as_str())
            .collect();
        (!kinds.is_empty()).then(|| format!("{} {}", word.label(), kinds.join(", ")))
    })
    .collect()
}

/// The player's own trained skills, one a line in skill order: `Stealth 17`.
pub(super) fn skill_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    (view.own_skills().iter())
        .map(|s| format!("{} {}", content.skill(s.skill).name, s.value))
        .collect()
}

/// The player's talents in the order taken, one a line: its name for its choice; a race's and a
/// birth sign's gifts first, marked `(race)` and `(sign)`, the sign's flaw `(sign, flaw)`; a forgotten altar's
/// `(altar)` and `(altar, flaw)`; the background story's `(story)` and `(story, flaw)`; a profane
/// gift's `(profane)`, a profane bane's `(profane, bane)`, and a rising used up `(sign, spent)`
/// or `(profane, spent)` (Twice-born).
pub(super) fn talent_rows(view: &PerceptionView<'_>) -> Vec<String> {
    (view.own_talents().iter())
        .map(|t| {
            let name = &t.name;
            match (t.granted, t.flaw, t.class) {
                _ if t.profane == Some(GiftKind::Bane) => format!("{name} (profane, bane)"),
                _ if t.profane.is_some() && t.spent => format!("{name} (profane, spent)"),
                _ if t.profane.is_some() => format!("{name} (profane)"),
                _ if t.story && t.flaw => format!("{name} (story, flaw)"),
                _ if t.story => format!("{name} (story)"),
                _ if t.boon && t.flaw => format!("{name} (altar, flaw)"),
                _ if t.boon => format!("{name} (altar)"),
                (_, true, _) => format!("{name} (sign, flaw)"),
                (true, false, _) if t.race => format!("{name} (race)"),
                (true, false, _) if t.spent => format!("{name} (sign, spent)"),
                (true, false, _) => format!("{name} (sign)"),
                (false, false, true) => format!("{name} (class)"),
                _ => name.clone(),
            }
        })
        .collect()
}

/// The player's own trained skills by skill id, with their values: `{"stealth": 17}`.
pub(super) fn skills_json(view: &PerceptionView<'_>) -> Value {
    let content = view.content();
    let map: serde_json::Map<String, Value> = (view.own_skills().iter())
        .map(|s| (content.skill(s.skill).id.clone(), json!(s.value)))
        .collect();
    Value::Object(map)
}
