//! Services: who sells a cure, a mending, an identification, or a skill's training, and for how
//! much.

use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{Content, DialogueId, ServiceKind};

/// The services (`[[dialogue.service]]` in the dialogue data): who sells each, what it does, and
/// its price before Pious, sorted by who sells it and then by the service's name.
pub(super) fn services(content: &Content) -> String {
    let mut rows: Vec<(String, String, String, String)> = Vec::new();
    for (i, d) in content.dialogues().iter().enumerate() {
        let who = capital(&content.speaker_name(DialogueId(i as u16)));
        for s in &d.services {
            let does = match (s.does, &s.train) {
                (ServiceKind::Cure, _) => "ends every disease, even one not yet broken out".into(),
                (ServiceKind::Identify, _) => "identifies everything carried that is not yet \
                                               known: magic items and scroll types"
                    .into(),
                (ServiceKind::Mend, _) => {
                    "mends every point of stat damage (not while a disease shows)".into()
                }
                (ServiceKind::TrainSkill, Some(t)) => {
                    let skill = (content.skill_id(&t.skill))
                        .map_or(t.skill.clone(), |id| content.skill(id).name.clone());
                    format!(
                        "trains you in [{skill}](skills.md#skill-{}) at {}, offered only while you \
                         are untrained in it",
                        t.skill, t.value
                    )
                }
                (ServiceKind::TrainSkill, None) => String::new(),
            };
            let price = match s.does {
                ServiceKind::Identify => format!("{} gold per unidentified stack", s.price),
                _ => format!("{} gold", s.price),
            };
            rows.push((who.clone(), s.name.clone(), does, price));
        }
    }
    if rows.is_empty() {
        return "No one offers a service yet.\n".to_string();
    }
    rows.sort();
    let mut out = String::from("| Who | Service | What it does | Price |\n|---|---|---|---:|\n");
    for (who, name, does, price) in rows {
        let _ = writeln!(out, "| {who} | {name} | {does} | {price} |");
    }
    out
}

/// The towns (a campaign's `data/towns/`): their people, the percent their services cost less
/// per quest of theirs completed, and how many quests make them free, sorted by name.
pub(super) fn towns(content: &Content) -> String {
    let mut rows: Vec<_> = content.towns().iter().collect();
    if rows.is_empty() {
        return "No town keeps goodwill yet.\n".to_string();
    }
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out =
        String::from("| Town | Its people | Off per quest | Free after |\n|---|---|---:|---:|\n");
    for town in rows {
        let mut people: Vec<String> = (0..content.dialogues().len())
            .map(|i| DialogueId(i as u16))
            .filter(|&d| content.town_of(d) == Some(town))
            .map(|d| content.speaker_name(d))
            .collect();
        people.sort();
        let free = match town.goodwill {
            0 => "never".to_string(),
            n => format!("{} quests", 100u32.div_ceil(n)),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {}% | {free} |",
            town.name,
            people.join(", "),
            town.goodwill
        );
    }
    out
}
