//! Abilities (spec §6 "Abilities"): what a character can do beyond the commands every run has —
//! racial, class, skill, and style abilities. Unlike ADOM, they do not get a key each: they go
//! into one abilities menu (#156, to come), which lists [`PerceptionView::abilities`], the ones
//! the player can use now, asks for what each needs ([`Ability::aim`]), and applies the
//! [`Action`] it makes ([`Ability::action`]). Each is also a key command that is unbound by
//! default (rebindable in Controls).
//!
//! [`PerceptionView::abilities`]: crate::perception::PerceptionView::abilities

use crate::action::Action;
use crate::content::Content;
use crate::geom::Dir;
use crate::world::Actor;

/// An ability, as the abilities menu lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ability {
    /// Bash with the shield held (spec §8 "Weapon skills"): [`Action::Bash`].
    ShieldBash,
}

/// What an ability asks for before it is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aim {
    /// A direction, as opening a door does.
    Direction,
}

impl Ability {
    /// Every ability, in menu order.
    pub const ALL: &[Ability] = &[Ability::ShieldBash];

    /// Its name in the menu.
    pub const fn name(self) -> &'static str {
        match self {
            Ability::ShieldBash => "Bash with your shield",
        }
    }

    /// What it asks for before it is used.
    pub const fn aim(self) -> Aim {
        match self {
            Ability::ShieldBash => Aim::Direction,
        }
    }

    /// The action it is, aimed in `dir`.
    pub const fn action(self, dir: Dir) -> Action {
        match self {
            Ability::ShieldBash => Action::Bash(dir),
        }
    }

    /// Whether `actor` can use it now: a shield bash needs a shield to bash with.
    pub(crate) fn usable(self, actor: &Actor, content: &Content) -> bool {
        match self {
            Ability::ShieldBash => actor.shield(content).is_some(),
        }
    }
}
