//! What a command or an ability that acts on a cell beside the player may act on (#1327): the
//! ways to its legal targets, as the player knows them (the perception snapshot only: hidden
//! traps, secret doors, and unseen creatures never count). The input code and the abilities menu
//! both read it, so the keys and the menu agree: with no target beside the player the action is
//! refused at once (toward [`PerceptionView::blank_way`], so the rules say why, taking no time),
//! with one it is done at once, and with several the direction is asked.
//!
//! Kick is not here: any direction may be kicked. The rules keep their own refusals for action
//! text that names a direction (`open:E` at a wall), so replays and the CLI are unchanged.

use crate::ability::Ability;
use crate::action::Action;
use crate::content::{ActiveAim, ClassActive, Content, LevelKind, Stance, WeaponDoes};
use crate::geom::{Dir, Pos};
use crate::perception::{CellView, PerceptionView, SeenActor};
use crate::world::ActorKind;

/// A command or an ability that asks for a direction toward something beside the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Beside {
    /// Open: a closed door or a shut chest.
    Open,
    /// Close: an open door with nothing in its way.
    Close,
    /// Chat: a creature in view.
    Chat,
    /// Use tool with a digging tool ready: a wall that breaks.
    Dig,
    /// An ability aimed in a direction ([`crate::ability::Aim::Direction`]).
    Use(Ability),
}

/// What a [`Beside`] acts on: the abilities menu names it when there is none
/// ([`crate::ability::Unavailable::NoTarget`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// A closed door or a shut chest (Open).
    Openable,
    /// An open door with nothing in its way (Close).
    OpenDoor,
    /// A closed door (Listen).
    ClosedDoor,
    /// A closed door not wedged already (Wedge).
    Unwedged,
    /// A closed door or a shut chest not known to be unlocked (Pick lock, Knock).
    Lock,
    /// A trap the player knows of (Disarm).
    Trap,
    /// A creature in view (Chat).
    Someone,
    /// A creature in view not hostile, that has pockets (Steal).
    Mark,
    /// A hostile creature in view within reach (a shield bash, the aimed class blows, the weapon
    /// actives aimed in a direction).
    Foe,
    /// A hostile creature in view with a mind to fool (Feint).
    Fool,
    /// A beast in view that may be tamed, no ally already (Tame).
    Beast,
    /// A wall that breaks (Dig).
    Wall,
    /// The glowing fungus (Gather).
    Fungus,
}

impl Target {
    /// Why an ability that acts on it cannot be used now, as the menu says it.
    pub const fn reason(self) -> &'static str {
        match self {
            Target::Openable => "nothing beside you to open",
            Target::OpenDoor => "no open door beside you",
            Target::ClosedDoor | Target::Unwedged => "no closed door beside you",
            Target::Lock => "no lock beside you",
            Target::Trap => "no known trap beside you",
            Target::Someone => "no one beside you",
            Target::Mark => "no one beside you to rob",
            Target::Foe => "no foe within reach",
            Target::Fool => "no foe beside you to fool",
            Target::Beast => "no beast beside you to tame",
            Target::Wall => "nothing beside you to dig",
            Target::Fungus => "no fungus beside you",
        }
    }

    /// Its id in dumps (`no_target:foe`).
    pub const fn id(self) -> &'static str {
        match self {
            Target::Openable => "openable",
            Target::OpenDoor => "open_door",
            Target::ClosedDoor => "closed_door",
            Target::Unwedged => "unwedged",
            Target::Lock => "lock",
            Target::Trap => "trap",
            Target::Someone => "someone",
            Target::Mark => "mark",
            Target::Foe => "foe",
            Target::Fool => "fool",
            Target::Beast => "beast",
            Target::Wall => "wall",
            Target::Fungus => "fungus",
        }
    }
}

/// What the input does with a [`Beside`] now ([`PerceptionView::way`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    /// Nothing to act on beside the player: the action toward this way, which the rules refuse
    /// with their own words, taking no time.
    Refuse(Dir),
    /// One thing to act on: the action toward it, at once.
    Act(Dir),
    /// Several: ask which way.
    Ask,
}

impl Beside {
    /// What it acts on; `None` for an ability not aimed in a direction.
    pub fn target(self, content: &Content) -> Option<Target> {
        Some(match self {
            Beside::Open => Target::Openable,
            Beside::Close => Target::OpenDoor,
            Beside::Chat => Target::Someone,
            Beside::Dig => Target::Wall,
            Beside::Use(ability) => match ability {
                Ability::Disarm => Target::Trap,
                Ability::Listen => Target::ClosedDoor,
                Ability::Wedge => Target::Unwedged,
                Ability::PickLock | Ability::Class(ClassActive::Knock) => Target::Lock,
                Ability::Steal => Target::Mark,
                Ability::Feint => Target::Fool,
                Ability::Tame => Target::Beast,
                Ability::Gather => Target::Fungus,
                Ability::ShieldBash => Target::Foe,
                Ability::Class(active) if active.aimed() => Target::Foe,
                Ability::Weapon(_, id) => {
                    let aim = content.weapon_ability(id).active.map(|a| a.aim);
                    (aim == Some(ActiveAim::Direction)).then_some(Target::Foe)?
                }
                _ => return None,
            },
        })
    }

    /// The action it is toward `dir`; `None` for an ability not aimed in a direction.
    pub fn action(self, content: &Content, dir: Dir) -> Option<Action> {
        match self {
            Beside::Open => Some(Action::Open(dir)),
            Beside::Close => Some(Action::Close(dir)),
            Beside::Chat => Some(Action::Chat(dir)),
            Beside::Dig => Some(Action::Dig(dir)),
            Beside::Use(ability) => ability.action(content, Some(dir)),
        }
    }

    /// How many cells it reaches toward a foe: a lunge's reach, else 1.
    fn reach(self, content: &Content) -> u8 {
        match self {
            Beside::Use(Ability::Weapon(_, id)) => match content.weapon_ability(id).effect {
                WeaponDoes::ReachBlow { reach } => reach.max(1),
                _ => 1,
            },
            _ => 1,
        }
    }
}

impl PerceptionView<'_> {
    /// The ways to what `what` may act on beside the player, in compass order, as they know it;
    /// `None` for an ability not aimed in a direction.
    pub fn ways(&self, what: Beside) -> Option<Vec<Dir>> {
        let target = what.target(self.content)?;
        let reach = what.reach(self.content);
        if target == Target::Fungus {
            return Some(self.gather_ways());
        }
        if target == Target::Wall && self.level_kind() == LevelKind::Overworld {
            return Some(Vec::new());
        }
        let me = self.viewer();
        Some(
            (Dir::ALL.into_iter())
                .filter(|&dir| match target {
                    Target::Foe => self.foe_toward(dir, reach),
                    _ => self.fits(target, me.step(dir)),
                })
                .collect(),
        )
    }

    /// What the input does with `what` now: refuse it, do it toward the one target, or ask
    /// (module docs); `None` for an ability not aimed in a direction.
    pub fn way(&self, what: Beside) -> Option<Way> {
        let ways = self.ways(what)?;
        Some(match ways.as_slice() {
            [] => Way::Refuse(self.blank_way()),
            [dir] => Way::Act(*dir),
            _ => Way::Ask,
        })
    }

    /// The way an action with nothing to act on is aimed, so the rules refuse it with their own
    /// words: the first, in compass order, toward a wall as the player knows it, no door or
    /// chest (where nobody unseen stands to be struck by mistake), else north.
    pub fn blank_way(&self) -> Dir {
        let me = self.viewer();
        (Dir::ALL.into_iter())
            .find(|&dir| {
                self.known(me.step(dir)).is_some_and(|t| {
                    let def = self.content.terrain(t);
                    !def.walkable && !def.is_door() && def.chest.is_none()
                })
            })
            .unwrap_or(Dir::N)
    }

    /// The terrain the player sees or remembers at `p`.
    fn known(&self, p: Pos) -> Option<crate::content::TerrainId> {
        match self.cell(p) {
            CellView::Visible(t) | CellView::Remembered(t) => Some(t),
            CellView::Unknown => None,
        }
    }

    /// The creature the player sees at `p`, the player never.
    fn seen_creature(&self, p: Pos) -> Option<&SeenActor> {
        self.actor_at(p).filter(|a| a.kind == ActorKind::Creature)
    }

    /// Whether a hostile the player sees stands within `reach` cells `dir` of them, with nobody
    /// and no known wall before it (a lunge reaches on over floor).
    fn foe_toward(&self, dir: Dir, reach: u8) -> bool {
        let mut cell = self.viewer();
        for _ in 0..reach {
            cell = cell.step(dir);
            if let Some(actor) = self.actor_at(cell) {
                return actor.kind == ActorKind::Creature && self.stance(actor) == Stance::Hostile;
            }
            let open = self
                .known(cell)
                .is_some_and(|t| self.content.terrain(t).walkable);
            if !open {
                return false;
            }
        }
        false
    }

    /// Whether the cell at `p` holds a `target` as the player knows it.
    fn fits(&self, target: Target, p: Pos) -> bool {
        let def = self.known(p).map(|t| self.content.terrain(t));
        let closed = def.is_some_and(|d| d.is_closed_door());
        let chest = def.is_some_and(|d| d.is_shut_chest());
        match target {
            Target::Openable => closed || chest,
            Target::OpenDoor => {
                def.is_some_and(|d| d.is_open_door())
                    && self.actor_at(p).is_none()
                    && self.objects(p).is_empty()
            }
            Target::ClosedDoor => closed,
            Target::Unwedged => closed && !self.level.wedged.contains_key(&p),
            // A chest Chest-wise shows unlocked is no lock to pick; a door's lock is unknown
            // until tried.
            Target::Lock => {
                closed || (chest && self.chest_at(p).is_none_or(|c| c.locked != Some(false)))
            }
            Target::Trap => self.trap(p).is_some(),
            Target::Someone => self.seen_creature(p).is_some(),
            Target::Mark => self.seen_creature(p).is_some_and(|a| {
                let def = self.content.creature(a.creature);
                self.stance(a) != Stance::Hostile
                    && def.animated.is_none()
                    && !self.content.is_mimic(a.creature)
            }),
            Target::Fool => self.seen_creature(p).is_some_and(|a| {
                self.stance(a) == Stance::Hostile && self.content.has_vitals(a.creature)
            }),
            Target::Beast => self.seen_creature(p).is_some_and(|a| {
                !a.ally && !a.companion && self.content.creature(a.creature).tameable
            }),
            Target::Wall => {
                def.is_some_and(|d| !d.walkable && d.breaks.is_some()) && !self.level.on_edge(p)
            }
            Target::Foe => {
                (self.seen_creature(p)).is_some_and(|a| self.stance(a) == Stance::Hostile)
            }
            Target::Fungus => false,
        }
    }
}
