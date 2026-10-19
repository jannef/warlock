//! What actions cost (`assets/data/actions.toml`, spec §8 "Time").
//!
//! Every action that takes time has a cost in energy, [`NORMAL_COST`] (1000) for a normal action.
//! How long it takes an actor follows from the cost, the action's base time, and the actor's
//! speed: `cost / 1000 × 1000 / speed × base`, the base being [`BASE_TURN`] (6 s) for every
//! action but a step on the overworld, whose base is the terrain's `travel_minutes`. A normal
//! action takes a creature of normal speed 6 seconds, one of cost 1400 8.4 seconds, and a
//! normal action at speed 1111 about 5.4 seconds. The scheduler (`world/schedule.rs`) turns a
//! cost into an energy [`price`].
//!
//! The file maps action ids to costs (`wait = 1000`); an action it leaves out costs 1000, and so
//! does every action without the file. Creature types may give their attacks a cost of their own
//! (`attack_cost` in `data/creatures.toml`).

use super::ContentError;
use crate::time::Duration;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// The cost of a normal action, in energy.
pub const NORMAL_COST: u32 = 1000;
/// The costs an action may have: a tenth of a normal action to ten times one.
pub const COSTS: RangeInclusive<u32> = 100..=10_000;
/// How long a normal action takes at normal speed: the base time of every action but a step on
/// the overworld (spec §8 "Time").
pub const BASE_TURN: Duration = Duration::seconds(6);

// A cost of 1000 at speed 1000 takes exactly the base time, so an action's energy price is its
// cost times its base in seconds (an actor gains its speed in energy each second).
const _: () = assert!(NORMAL_COST == super::NORMAL_SPEED);

/// The actions that take time, as far as their cost goes. Their ids (`data/actions.toml`) are
/// [`ActionKind::id`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActionKind {
    /// A step onto terrain of `move_cost` 100. A step costs this scaled by the terrain's
    /// `move_cost` percent; on the overworld it costs this over the terrain's `travel_minutes`.
    Step,
    Wait,
    OpenDoor,
    CloseDoor,
    /// Unlocking a locked door with a key: part of opening it, so it costs this plus `open_door`.
    Unlock,
    /// A kick (`K`): at a door, to break it, or at a creature, an unarmed blow.
    Kick,
    /// Lighting or dousing the lantern.
    Lantern,
    /// Taking stairs: also entering a location or a local area from the overworld, and walking
    /// off the edge of a local area back to it.
    Stairs,
    PickUp,
    Drop,
    /// A melee attack: the player moving into a hostile creature, or a creature's.
    Melee,
    /// A shot from a missile weapon (the player's `f`, a creature's ranged attack).
    Shoot,
    /// Throwing a throwing weapon (`t`).
    Throw,
    /// Reloading a missile weapon that needs it after every shot (a crossbow): part of each of
    /// its shots.
    Reload,
    /// A creature's spell.
    Cast,
    /// Wielding a thing in a hand, or putting it away (`data/slots.toml` names each slot's
    /// action).
    Wield,
    /// Putting on or taking off a helmet, cloak, gloves, boots, a ring, or an amulet.
    Wear,
    /// Putting on or taking off body armor: it takes a while to strap on.
    BodyArmor,
    /// Readying a missile weapon, missiles, or a tool, or putting them away.
    Ready,
    /// A turn of eating (spec §8 "Hunger"): a meal takes as many as the food's `turns`.
    Eat,
    /// A turn of digging into a wall with a digging tool (spec §8 "Digging"): a dig takes many
    /// of them.
    Dig,
    /// Chatting with a creature (spec §8 "Dialogue"): a short action.
    Chat,
    /// Climbing out of the water onto ice (spec §5 "Ice"): on top of the step onto it.
    Climb,
}

impl ActionKind {
    pub const ALL: [ActionKind; 23] = [
        ActionKind::Step,
        ActionKind::Wait,
        ActionKind::OpenDoor,
        ActionKind::CloseDoor,
        ActionKind::Unlock,
        ActionKind::Kick,
        ActionKind::Lantern,
        ActionKind::Stairs,
        ActionKind::PickUp,
        ActionKind::Drop,
        ActionKind::Melee,
        ActionKind::Shoot,
        ActionKind::Throw,
        ActionKind::Reload,
        ActionKind::Cast,
        ActionKind::Wield,
        ActionKind::Wear,
        ActionKind::BodyArmor,
        ActionKind::Ready,
        ActionKind::Eat,
        ActionKind::Dig,
        ActionKind::Chat,
        ActionKind::Climb,
    ];

    /// Its id in `data/actions.toml`.
    pub const fn id(self) -> &'static str {
        match self {
            ActionKind::Step => "step",
            ActionKind::Wait => "wait",
            ActionKind::OpenDoor => "open_door",
            ActionKind::CloseDoor => "close_door",
            ActionKind::Unlock => "unlock",
            ActionKind::Kick => "kick",
            ActionKind::Lantern => "lantern",
            ActionKind::Stairs => "stairs",
            ActionKind::PickUp => "pick_up",
            ActionKind::Drop => "drop",
            ActionKind::Melee => "melee",
            ActionKind::Shoot => "shoot",
            ActionKind::Throw => "throw",
            ActionKind::Reload => "reload",
            ActionKind::Cast => "cast",
            ActionKind::Wield => "wield",
            ActionKind::Wear => "wear",
            ActionKind::BodyArmor => "body_armor",
            ActionKind::Ready => "ready",
            ActionKind::Eat => "eat",
            ActionKind::Dig => "dig",
            ActionKind::Chat => "chat",
            ActionKind::Climb => "climb",
        }
    }

    /// Whether it is an attack (a creature type may give its attacks their own cost).
    pub const fn is_attack(self) -> bool {
        matches!(
            self,
            ActionKind::Melee | ActionKind::Shoot | ActionKind::Throw | ActionKind::Cast
        )
    }

    /// Whether it puts equipment on or takes it off (a slot's `action` in `data/slots.toml`).
    /// Taking a thing off costs what putting it on does.
    pub const fn is_equipment(self) -> bool {
        matches!(
            self,
            ActionKind::Wield | ActionKind::Wear | ActionKind::BodyArmor | ActionKind::Ready
        )
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// The cost of every action (`data/actions.toml`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionCosts([u32; ActionKind::ALL.len()]);

impl Default for ActionCosts {
    /// Without the file every action costs [`NORMAL_COST`].
    fn default() -> Self {
        Self([NORMAL_COST; ActionKind::ALL.len()])
    }
}

impl ActionCosts {
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: BTreeMap<String, u32> = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let mut costs = Self::default();
        for (id, cost) in parsed {
            let kind = ActionKind::ALL
                .into_iter()
                .find(|k| k.id() == id)
                .ok_or_else(|| err(format!("unknown action `{id}`")))?;
            check_cost(cost).map_err(|m| err(format!("action `{id}`: {m}")))?;
            costs.0[kind.index()] = cost;
        }
        Ok(costs)
    }

    /// What `kind` costs, in energy (1000 for a normal action).
    pub fn cost(&self, kind: ActionKind) -> u32 {
        self.0[kind.index()]
    }
}

/// Checks that `cost` is within [`COSTS`].
pub(super) fn check_cost(cost: u32) -> Result<(), String> {
    if COSTS.contains(&cost) {
        Ok(())
    } else {
        Err(format!(
            "cost {cost} is out of range ({} to {})",
            COSTS.start(),
            COSTS.end()
        ))
    }
}

/// The energy price of an action of `cost` whose base time is `base`: the cost times the base in
/// seconds, at least 1. An actor gains its speed in energy each second, so the action takes it
/// `cost / 1000 × 1000 / speed × base`.
pub fn price(cost: u32, base: Duration) -> i64 {
    let seconds = i64::try_from(base.as_seconds()).unwrap_or(i64::MAX);
    i64::from(cost).saturating_mul(seconds).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prices_are_cost_times_base_seconds() {
        assert_eq!(price(NORMAL_COST, BASE_TURN), 6000);
        assert_eq!(price(1400, BASE_TURN), 8400);
        assert_eq!(price(NORMAL_COST, Duration::minutes(45)), 2_700_000);
        assert_eq!(price(NORMAL_COST, Duration::ZERO), 1);
    }

    #[test]
    fn costs_parse_default_and_are_validated() {
        let costs = ActionCosts::parse("a", "wait = 1000\nmelee = 1400\n").expect("valid");
        assert_eq!(costs.cost(ActionKind::Melee), 1400);
        assert_eq!(
            costs.cost(ActionKind::Cast),
            NORMAL_COST,
            "left out: normal"
        );
        assert!(
            ActionKind::ALL
                .iter()
                .all(|k| ActionCosts::default().cost(*k) == NORMAL_COST)
        );
        for (text, message) in [
            ("sleep = 1000", "unknown action `sleep`"),
            (
                "wait = 99",
                "action `wait`: cost 99 is out of range (100 to 10000)",
            ),
            ("wait = 10001", "cost 10001 is out of range"),
            ("wait = \"slow\"", "invalid type"),
        ] {
            let err = ActionCosts::parse("a", text).unwrap_err();
            assert!(err.message.contains(message), "{text}: {}", err.message);
        }
        let ids: Vec<_> = ActionKind::ALL.iter().map(|k| k.id()).collect();
        let mut unique = ids.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), ids.len(), "ids are unique");
        assert!(
            ActionKind::ALL
                .iter()
                .enumerate()
                .all(|(i, k)| k.index() == i)
        );
    }
}
