//! Abilities (spec §6 "Abilities"): every ability the abilities menu can list, from the one
//! registry (`warlock_core::ability`).

use super::actions::at_normal_pace;
use std::fmt::Write as _;
use warlock_core::ability::{Ability, Aim};
use warlock_core::content::{BASE_TURN, ClassActive, Content, SignActive, TalentEffect, price};

/// Every ability, grouped as the menu groups them (by where they come from) and by name within a
/// group: who has it, what it asks for, what it does, and how long it takes at a normal pace. A
/// skill used as an action whose use is not in the game yet says so.
pub(super) fn abilities(content: &Content) -> String {
    let mut out = String::from(
        "| Ability | In the menu under | Who has it | Asks for | What it does | Takes at a normal \
         pace |\n|---|---|---|---|---|---:|\n",
    );
    let mut every = Ability::every(content);
    every.sort_by_key(|a| (a.source(), a.name(content).to_string()));
    for a in every {
        let who = match a {
            Ability::Sign(active) => sign_holders(content, active),
            Ability::Class(active) => class_holders(content, active),
            Ability::Spell(spell) => {
                let id = content.spell_of(spell).map_or("", |s| s.id.as_str());
                let who = table_holders(
                    content,
                    |e| matches!(e, TalentEffect::Spell { spell } if spell == id),
                );
                let who = if who.is_empty() {
                    "whoever learns it from a scroll".to_string()
                } else {
                    format!("{who}, or whoever learns it from a scroll")
                };
                let spell = content.spell_of(spell).zip(content.spell_rules());
                let (tier, cost) = spell.map_or((0, 0), |(s, r)| {
                    let at = i32::from(r.comfortable[usize::from(s.tier)]);
                    (s.tier, r.mana(s, at, 0))
                });
                format!("{who} (a tier-{tier} spell, {cost} mana a cast)")
            }
            Ability::Skill(_) => "everyone trained in the skill".into(),
            Ability::Disarm => "everyone trained in Disable device".into(),
            Ability::PickLock => "everyone trained in Open lock".into(),
            Ability::Steal => "everyone trained in Steal".into(),
            Ability::Forage => {
                "everyone trained in Survival (usable in the wild: the overworld's land and its \
                 local areas)"
                    .into()
            }
            Ability::SecondWind => "everyone with the Second wind talent (not while winded)".into(),
            Ability::BloodMagic => "everyone with the Blood magic talent".into(),
            Ability::ShieldBash => "everyone (usable with a shield in hand)".into(),
            Ability::ThrowVoice => {
                table_holders(content, |e| matches!(e, TalentEffect::ThrowVoice))
            }
            Ability::HoodLantern => "everyone (it needs a lantern with a hood)".into(),
            Ability::Pray => {
                "everyone beside a forgotten altar not yet spent (a prayer takes a minute)".into()
            }
        };
        let (asks, takes) = match a.cost() {
            // A long action: its prayer's time, whatever the pace.
            Some(_) if a == Ability::Class(ClassActive::Consecrate) => {
                let book = content.talents();
                let time = (book.id("consecrate").into_iter())
                    .flat_map(|t| book.get(t).effects.iter())
                    .find_map(|e| match e {
                        TalentEffect::Consecrate { time, .. } => Some(time.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                (aim(a.aim(content)), format!("{time} (a prayer)"))
            }
            // A long action too: the forage's time.
            Some(_) if a == Ability::Forage => {
                let minutes = content.forage().map_or(0, |r| r.seconds / 60);
                (aim(a.aim(content)), format!("{minutes} minutes (a forage)"))
            }
            Some(kind) => {
                let cost = content.action_costs().cost(kind);
                (aim(a.aim(content)), at_normal_pace(price(cost, BASE_TURN)))
            }
            None => ("—", "not usable yet".to_string()),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {who} | {asks} | {} | {takes} |",
            a.name(content),
            a.source().name(),
            a.description(content),
        );
    }
    out
}

/// Who has class active `active`: the classes whose table gives it, from which level, by name
/// ("paladins from level 12").
fn class_holders(content: &Content, active: ClassActive) -> String {
    table_holders(content, |e| e.active() == Some(active))
}

/// The classes whose table gives a talent with an effect `has` says yes to, from which level, by
/// name ("bards from level 32").
fn table_holders(content: &Content, has: impl Fn(&TalentEffect) -> bool) -> String {
    let book = content.talents();
    let mut who: Vec<(String, u8)> = (content.classes().iter())
        .filter_map(|c| {
            let (level, _) = c.abilities.iter().find(|(_, talent)| {
                (book.id(talent)).is_some_and(|t| (book.get(t).effects.iter()).any(&has))
            })?;
            Some((c.name.to_lowercase(), *level))
        })
        .collect();
    who.sort();
    let parts: Vec<String> = (who.iter())
        .map(|(class, level)| format!("{class}s from level {level}"))
        .collect();
    parts.join(", ")
}

/// Who has the birth sign's active `active`: those born under the animals and elements (and on
/// the lucky days) whose gifts name its talent ("those born in the year of the Unicorn").
fn sign_holders(content: &Content, active: SignActive) -> String {
    let Some(h) = content.horoscope() else {
        return String::new();
    };
    let gives = |grants: &[String]| {
        (grants.iter()).any(|id| content.sign_active(active).is_some_and(|t| &t.id == id))
    };
    let mut who: Vec<String> = (h.animals.iter())
        .filter(|p| gives(&p.grants))
        .map(|p| format!("those born in the year of the {}", p.name))
        .collect();
    who.extend(
        (h.elements.iter())
            .filter(|p| gives(&p.grants))
            .map(|p| format!("the {}-born", p.name)),
    );
    who.join(", ")
}

/// What an ability asks for before it is used, in words.
fn aim(aim: Aim) -> &'static str {
    match aim {
        Aim::Nothing => "nothing",
        Aim::Direction => "a direction",
        Aim::Cell => "a place you see",
    }
}
