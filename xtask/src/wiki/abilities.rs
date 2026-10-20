//! Abilities (spec §6 "Abilities"): every ability the abilities menu can list, from the one
//! registry (`warlock_core::ability`).

use super::actions::at_normal_pace;
use std::fmt::Write as _;
use warlock_core::ability::{Ability, Aim};
use warlock_core::content::{BASE_TURN, Content, price};

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
            Ability::Skill(_) => "everyone trained in the skill",
            Ability::ShieldBash => "everyone (usable with a shield in hand)",
            Ability::ThrowVoice => "everyone",
            Ability::HoodLantern => "everyone (it needs a lantern with a hood)",
        };
        let (asks, takes) = match a.cost() {
            Some(kind) => {
                let cost = content.action_costs().cost(kind);
                (aim(a.aim()), at_normal_pace(price(cost, BASE_TURN)))
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

/// What an ability asks for before it is used, in words.
fn aim(aim: Aim) -> &'static str {
    match aim {
        Aim::Nothing => "nothing",
        Aim::Direction => "a direction",
        Aim::Cell => "a place you see",
    }
}
