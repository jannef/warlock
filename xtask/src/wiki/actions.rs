//! Action costs (spec §8 "Time"), how long an action takes at a normal pace, and everyone's
//! speed (spec §8 "Speed").

use super::order;
use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{ActionKind, BASE_TURN, Content, NORMAL_COST, NORMAL_SPEED, price};
use warlock_core::time::Duration;

/// How long an action of energy price `price` takes at normal speed (the player's, 1000):
/// `6 seconds`, `5.4 seconds`, `45 minutes`.
pub(super) fn at_normal_pace(price: i64) -> String {
    at_speed(price, NORMAL_SPEED)
}

/// How long an action of energy price `price` takes an actor of `speed` (spec §8 "Speed": it
/// gains its speed in energy a second), to a tenth of a second under a minute: `6 seconds`,
/// `7.5 seconds`, `45 minutes`.
pub(super) fn at_speed(price: i64, speed: u32) -> String {
    let speed = i64::from(speed.max(1));
    let tenths = (price * 10 + speed / 2) / speed;
    if tenths % 10 != 0 && tenths < 600 {
        return format!("{}.{} seconds", tenths / 10, tenths % 10);
    }
    let seconds = u64::try_from(price.div_euclid(speed)).unwrap_or(0);
    Duration::seconds(seconds).spoken()
}

/// What each action costs and how long it takes at a normal pace (spec §8 "Time"), and the
/// creature types whose attacks cost something else.
pub(super) fn action_costs(content: &Content) -> String {
    let mut out = String::from("| Action | Cost | Takes at a normal pace |\n|---|---:|---:|\n");
    for &kind in ActionKind::ALL {
        let what = match kind {
            ActionKind::Step => "A step onto normal ground (other ground: see Terrain)",
            ActionKind::Wait => "Waiting",
            ActionKind::OpenDoor => "Opening a door",
            ActionKind::CloseDoor => "Closing a door",
            ActionKind::Unlock => "Unlocking a door with a key (on top of opening it)",
            ActionKind::PickLock => {
                "A try at picking a lock (once it gives, opening the door comes on top)"
            }
            ActionKind::Kick => "Kicking (a door, or a creature)",
            ActionKind::Lantern => "Lighting or dousing the lantern",
            ActionKind::Stairs => {
                "Taking stairs, entering a place from the overworld, or walking off an area's edge"
            }
            ActionKind::PickUp => "Picking up (however many stacks)",
            ActionKind::Drop => "Dropping (however many stacks)",
            ActionKind::Melee => "Attacking up close",
            ActionKind::Shoot => "Shooting with a missile weapon",
            ActionKind::Throw => "Throwing",
            ActionKind::Reload => "Reloading a crossbow (after every shot, on top of shooting)",
            ActionKind::Cast => "Casting a spell (creatures)",
            ActionKind::Wield => "Wielding something, or putting it away",
            ActionKind::Wear | ActionKind::BodyArmor => "Putting something on, or taking it off",
            ActionKind::Ready => "Readying something, or putting it away",
            ActionKind::Eat => "A turn of eating (a meal takes several: see Food and corpses)",
            ActionKind::Read => "Reading a scroll",
            ActionKind::Dig => "A turn of digging (digging through a wall takes many)",
            ActionKind::Search => "A turn of searching (searching for a while takes several)",
            ActionKind::Disarm => "Trying to disarm a trap (whatever comes of it)",
            ActionKind::Chat => "Chatting with someone (asking about a topic takes no time)",
            ActionKind::Climb => {
                "Climbing out of the water onto ice (on top of the step onto the ice)"
            }
        };
        // Equipment actions say which slots take them (data/slots.toml); a kind no slot uses
        // is left out.
        let what = if kind.is_equipment() {
            let slots = slots_of(content, kind);
            if slots.is_empty() {
                continue;
            }
            format!("{what}: {slots}")
        } else {
            what.to_string()
        };
        let cost = content.action_costs().cost(kind);
        let _ = writeln!(
            out,
            "| {what} | {cost} | {} |",
            at_normal_pace(price(cost, BASE_TURN))
        );
    }
    for c in content.creature_defs() {
        if let Some(cost) = c.attack_cost {
            let _ = writeln!(out, "\nThe {}'s attacks cost {cost} each.", c.name);
        }
    }
    out
}

/// Everyone's speed (spec §8 "Speed"): yours, whatever your race, then every creature's, the
/// quickest first (then by name), with how long a normal action takes each.
pub(super) fn speeds(content: &Content) -> String {
    let mut out = String::from("| Who | Speed | A normal action takes |\n|---|---:|---:|\n");
    let mut all = order::creatures(content);
    all.sort_by_key(|(_, c)| (c.id != "player", std::cmp::Reverse(c.speed)));
    for (_, c) in all {
        let who = if c.id == "player" {
            "You, whatever your race".to_string()
        } else {
            capital(&c.name)
        };
        let _ = writeln!(
            out,
            "| {who} | {} | {} |",
            c.speed,
            at_speed(price(NORMAL_COST, BASE_TURN), c.speed)
        );
    }
    out
}

/// The names of the slots whose action is `kind`, in screen order, lower case after the first:
/// "Head, neck, cloak".
fn slots_of(content: &Content, kind: ActionKind) -> String {
    let names: Vec<String> = (content.slots().iter())
        .filter(|s| s.action == kind)
        .map(|s| s.name.to_lowercase())
        .collect();
    capital(&names.join(", "))
}
