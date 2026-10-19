//! The info-screen group (spec §6): the screens that show what the player knows about themselves
//! are numbered pages of one set. While one is the top screen, the top-row digits `1`..`9`
//! replace it with the numbered page ([`ScreenAction::Replace`], not a push), so Esc still
//! returns to wherever the first one was opened from. Each page keeps its own opening key in play.
//!
//! A new info screen joins the group by adding one entry to [`PAGES`] (in the order its number
//! should have) and calling [`draw_tabs`] after its panel is drawn. Sub-steps of a page (choosing
//! what goes in a slot) are not pages and keep their digits.

use super::menu::{Panel, Press};
use super::screens::{ScreenAction, ScreenName};
use super::{ACCENT, TEXT_DIM};
use crate::input::{Bindings, Context, KeyCommand};
use crate::render::console::Console;

/// The pages, in number order, with their tab labels: the screen's name, and a short one for a
/// panel too narrow for every name. Only screens that exist are listed.
const PAGES: &[(ScreenName, &str, &str)] = &[
    (ScreenName::Character, "Character", "Char"),
    (ScreenName::Inventory, "Inventory", "Inv"),
    (ScreenName::Equipment, "Equipment", "Equip"),
    (ScreenName::Skills, "Skills", "Skills"),
    (ScreenName::WeaponSkills, "Weapon skills", "Weapons"),
    (ScreenName::Quests, "Quests", "Quests"),
];

/// How a page is named on the tab line.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Label {
    /// By its name ("Inventory").
    Name,
    /// By its short name ("Inv").
    Short,
    /// By its key alone.
    Key,
}

/// Whether `name` is one of the pages.
pub fn is_page(name: ScreenName) -> bool {
    PAGES.iter().any(|(n, _, _)| *n == name)
}

/// What a press does on page `current`: replace it with the page the press names, if that is
/// another page. The keys are the `Pages` context of the bindings: a page's number, and next and
/// previous (wrapping; Shift reverses a next or previous). Numpad digits are movement unless the
/// player binds them.
pub fn switch(current: ScreenName, press: &Press, bindings: &Bindings) -> Option<ScreenAction> {
    let command = bindings.owner_in(Context::Pages, press.binding.as_ref()?)?;
    let here = PAGES.iter().position(|(n, _, _)| *n == current)?;
    let step = |forward: bool| {
        let (len, forward) = (PAGES.len(), forward != press.shift);
        if forward {
            (here + 1) % len
        } else {
            (here + len - 1) % len
        }
    };
    let to = match command {
        KeyCommand::Page(n) => usize::from(n).checked_sub(1)?,
        KeyCommand::NextPage => step(true),
        KeyCommand::PrevPage => step(false),
        _ => return None,
    };
    let (name, _, _) = PAGES.get(to)?;
    (*name != current).then_some(ScreenAction::Replace(*name))
}

/// The tab line that fits in `width` cells, the first of: every page by its name; the current
/// page by its name and the others by their short names; every page by its short name; the
/// current page by its name and the others by their keys alone; bare keys; nothing. Each page
/// named has its key before its name. Pairs of (text, is the current page).
fn tabs(current: ScreenName, bindings: &Bindings, width: usize) -> Vec<(String, bool)> {
    let make = |label: &dyn Fn(bool) -> Label| -> Vec<(String, bool)> {
        let mut parts = Vec::new();
        for (i, (name, full, short)) in PAGES.iter().enumerate() {
            let key = u8::try_from(i + 1)
                .ok()
                .and_then(|n| (bindings.keys_of(KeyCommand::Page(n)).first()).map(|k| k.display()))
                .unwrap_or_default();
            let here = *name == current;
            let named = match label(here) {
                Label::Name => *full,
                Label::Short => *short,
                Label::Key => "",
            };
            let text = match (named.is_empty(), key.is_empty()) {
                (true, _) => key,
                (false, true) => named.to_string(),
                (false, false) => format!("{key} {named}"),
            };
            parts.push((text, here));
        }
        parts
    };
    let len = |parts: &[(String, bool)]| {
        parts.iter().map(|(t, _)| t.chars().count()).sum::<usize>() + 3 * (parts.len() - 1)
    };
    let others = |other: Label| move |here: bool| if here { Label::Name } else { other };
    [
        make(&|_| Label::Name),
        make(&others(Label::Short)),
        make(&|_| Label::Short),
        make(&others(Label::Key)),
        make(&|_| Label::Key),
    ]
    .into_iter()
    .find(|parts| len(parts) <= width)
    .unwrap_or_default()
}

/// Draws the tab line on the panel's bottom border, as the title sits on the top one:
/// ` 1 Character · 2 Inventory · … `, the current page in the accent color.
pub fn draw_tabs(console: &mut Console, panel: &Panel, bindings: &Bindings, current: ScreenName) {
    let tabs = tabs(current, bindings, (panel.inner_w() - 2).max(0) as usize);
    if tabs.is_empty() {
        return;
    }
    let y = panel.y + panel.h - 1;
    let mut x = panel.inner_x();
    x += console.print(x, y, " ", TEXT_DIM);
    for (i, (text, is_current)) in tabs.iter().enumerate() {
        if i > 0 {
            x += console.print(x, y, " · ", TEXT_DIM);
        }
        x += console.print(x, y, text, if *is_current { ACCENT } else { TEXT_DIM });
    }
    console.print(x, y, " ", TEXT_DIM);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use bevy::prelude::KeyCode;

    fn press(binding: KeyBinding, shift: bool) -> Press {
        Press {
            binding: Some(binding),
            shift,
            ..Press::nav(super::super::menu::Nav::Other)
        }
    }

    fn top_row(ch: &str) -> Press {
        press(KeyBinding::Char(ch.into()), false)
    }

    #[test]
    fn page_keys_name_pages_and_numpad_none() {
        use ScreenName::*;
        let b = Bindings::default();
        let replace = |name| Some(ScreenAction::Replace(name));
        assert_eq!(switch(Character, &top_row("2"), &b), replace(Inventory));
        assert_eq!(switch(Inventory, &top_row("2"), &b), None, "already there");
        assert_eq!(switch(Skills, &top_row("5"), &b), replace(WeaponSkills));
        assert_eq!(switch(WeaponSkills, &top_row("6"), &b), replace(Quests));
        assert_eq!(
            switch(Character, &top_row("9"), &b),
            None,
            "no such page yet"
        );
        assert_eq!(switch(Character, &top_row("0"), &b), None);
        let numpad = press(KeyBinding::Code(KeyCode::Numpad2), false);
        assert_eq!(switch(Character, &numpad, &b), None);
    }

    #[test]
    fn tab_and_shift_tab_wrap_around() {
        use ScreenName::*;
        let b = Bindings::default();
        let tab = |shift| press(KeyBinding::Code(KeyCode::Tab), shift);
        let replace = |name| Some(ScreenAction::Replace(name));
        assert_eq!(switch(Character, &tab(false), &b), replace(Inventory));
        assert_eq!(switch(Skills, &tab(false), &b), replace(WeaponSkills));
        assert_eq!(switch(WeaponSkills, &tab(false), &b), replace(Quests));
        assert_eq!(switch(Quests, &tab(false), &b), replace(Character));
        assert_eq!(switch(Character, &tab(true), &b), replace(Quests));
        let up = press(KeyBinding::Code(KeyCode::PageUp), false);
        assert_eq!(switch(Inventory, &up, &b), replace(Character));
    }

    #[test]
    fn page_keys_are_rebindable_and_independent_of_play() {
        use ScreenName::*;
        let mut b = Bindings::default();
        // Tab means next target while aiming and next page here, at once.
        assert_eq!(
            b.owner_in(Context::Aim, &KeyBinding::Code(KeyCode::Tab)),
            Some(KeyCommand::NextTarget)
        );
        assert!(b.conflicts().is_empty());
        // Page 3 on `w`.
        b.bind_primary(KeyCommand::Page(3), KeyBinding::Char("w".into()))
            .unwrap();
        assert_eq!(
            switch(Character, &top_row("w"), &b),
            Some(ScreenAction::Replace(Equipment))
        );
        // Page keys are unique among themselves (a used one moves), but may be play keys too.
        let two = KeyBinding::Char("2".into());
        b.bind_primary(KeyCommand::NextTarget, two.clone()).unwrap();
        assert_eq!(b.owner_in(Context::Pages, &two), Some(KeyCommand::Page(2)));
        b.bind_primary(KeyCommand::Page(3), two.clone()).unwrap();
        assert_eq!(b.owner_in(Context::Pages, &two), Some(KeyCommand::Page(3)));
        assert_eq!(b.owner_in(Context::Aim, &two), Some(KeyCommand::NextTarget));
    }

    #[test]
    fn the_tab_line_shortens_to_fit() {
        let b = Bindings::default();
        let line = |current, w| {
            let parts = tabs(current, &b, w);
            (parts.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>()).join(" · ")
        };
        let inventory = |w| line(ScreenName::Inventory, w);
        assert_eq!(
            inventory(79),
            "1 Character · 2 Inventory · 3 Equipment · 4 Skills · 5 Weapon skills · 6 Quests"
        );
        // The 72-wide panels (68 cells for tabs): the current page by its name, the others by
        // their short names.
        let short = "1 Char · 2 Inventory · 3 Equip · 4 Skills · 5 Weapons · 6 Quests";
        assert_eq!((inventory(78), inventory(64)), (short.into(), short.into()));
        assert_eq!(
            line(ScreenName::WeaponSkills, 68),
            "1 Char · 2 Inv · 3 Equip · 4 Skills · 5 Weapon skills · 6 Quests"
        );
        let shortest = "1 Char · 2 Inv · 3 Equip · 4 Skills · 5 Weapons · 6 Quests";
        assert_eq!(
            (inventory(63), inventory(58)),
            (shortest.into(), shortest.into())
        );
        assert_eq!(inventory(57), "1 · 2 Inventory · 3 · 4 · 5 · 6");
        assert_eq!(inventory(31), "1 · 2 Inventory · 3 · 4 · 5 · 6");
        assert_eq!(inventory(30), "1 · 2 · 3 · 4 · 5 · 6");
        assert_eq!(inventory(20), "");
    }
}
