//! The info-screen group (spec §6): the screens that show what the player knows about themselves
//! are numbered pages of one set. While one is the top screen, the top-row digits `1`..`9`
//! replace it with the numbered page ([`ScreenAction::Replace`], not a push), so Esc still
//! returns to wherever the first one was opened from. Each page keeps its own opening key in play.
//!
//! A new info screen joins the group by adding one entry to [`PAGES`] (in the order its number
//! should have), sizing its panel with [`panel_width`] (one width for every page, so the frame
//! and the tab line stay put as the pages turn), and calling [`draw_tabs`] after its panel is
//! drawn. Sub-steps of a page (choosing
//! what goes in a slot) are not pages and keep their digits.

use super::menu::{Panel, Press};
use super::screens::{ScreenAction, ScreenName};
use super::{ACCENT, TEXT_DIM};
use crate::input::{Bindings, Context, KeyCommand};
use crate::render::console::Console;

/// The pages, in number order, with their tab labels: a name, and a short one for a console too
/// narrow for every name (short enough that the line fits an 80-column console). Only screens
/// that exist are listed; the ninth fills the last digit.
const PAGES: &[(ScreenName, &str, &str)] = &[
    (ScreenName::Character, "Character", "Char"),
    (ScreenName::Inventory, "Inventory", "Inv"),
    (ScreenName::Equipment, "Equipment", "Eqp"),
    (ScreenName::Skills, "Skills", "Skl"),
    (ScreenName::WeaponSkills, "Weapons", "Weap"),
    (ScreenName::Magic, "Magic", "Mag"),
    (ScreenName::Quests, "Quests", "Qst"),
    (ScreenName::Talents, "Talents", "Tal"),
    (ScreenName::Story, "Story", "Story"),
];

/// The width every page's panel has at least, so that the pages share one frame and one tab
/// line: the whole line by name, ` 1 Character · … · 9 Story `, in the frame and its margins.
/// A console narrower than this shortens every page's tab line the same way ([`tabs`]).
const WIDTH: i32 = 111;

/// The width of the panel of a page that would be `own` wide alone: at least [`WIDTH`].
pub const fn panel_width(own: i32) -> i32 {
    if own > WIDTH { own } else { WIDTH }
}

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

/// The key that turns to page `name` with its name, as a hint line names it ("6 Magic"); `None`
/// for a screen that is no page, or a page with no key.
pub fn hint(name: ScreenName, bindings: &Bindings) -> Option<String> {
    let i = PAGES.iter().position(|(n, _, _)| *n == name)?;
    let key = bindings.keys_of(KeyCommand::Page(u8::try_from(i + 1).ok()?));
    Some(format!("{} {}", key.first()?.display(), PAGES[i].1))
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

/// The tab line that fits in `width` cells, the first of: every page by its name; every page by
/// its short name; bare keys; nothing. Every page is labelled the same way whichever is the
/// current one (and every page's panel is as wide, [`panel_width`]), so the line stays put as
/// the pages turn and only its highlight moves. Each page named has its key before its name.
/// Pairs of (text, is the current page).
fn tabs(current: ScreenName, bindings: &Bindings, width: usize) -> Vec<(String, bool)> {
    let make = |label: Label| -> Vec<(String, bool)> {
        let mut parts = Vec::new();
        for (i, (name, full, short)) in PAGES.iter().enumerate() {
            let key = u8::try_from(i + 1)
                .ok()
                .and_then(|n| (bindings.keys_of(KeyCommand::Page(n)).first()).map(|k| k.display()))
                .unwrap_or_default();
            let here = *name == current;
            let named = match label {
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
    [Label::Name, Label::Short, Label::Key]
        .into_iter()
        .map(make)
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
        assert_eq!(switch(WeaponSkills, &top_row("6"), &b), replace(Magic));
        assert_eq!(switch(Magic, &top_row("7"), &b), replace(Quests));
        assert_eq!(switch(Quests, &top_row("8"), &b), replace(Talents));
        assert_eq!(switch(Talents, &top_row("9"), &b), replace(Story));
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
        assert_eq!(switch(WeaponSkills, &tab(false), &b), replace(Magic));
        assert_eq!(switch(Magic, &tab(false), &b), replace(Quests));
        assert_eq!(switch(Quests, &tab(false), &b), replace(Talents));
        assert_eq!(switch(Talents, &tab(false), &b), replace(Story));
        assert_eq!(switch(Story, &tab(false), &b), replace(Character));
        assert_eq!(switch(Character, &tab(true), &b), replace(Story));
        // PageUp and PageDown turn the pages of an item list instead (the inventory is one).
        let up = press(KeyBinding::Code(KeyCode::PageUp), false);
        assert_eq!(switch(Inventory, &up, &b), None);
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
    fn the_tab_line_is_the_same_on_every_page_and_shortens_to_fit() {
        let b = Bindings::default();
        let line = |current, w| {
            let parts = tabs(current, &b, w);
            (parts.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>()).join(" · ")
        };
        let full = "1 Character · 2 Inventory · 3 Equipment · 4 Skills · 5 Weapons · 6 Magic · \
                    7 Quests · 8 Talents · 9 Story";
        // The common width's line (the frame, a margin, and a space each side) names them all,
        // on every page, and the current page is the one highlighted.
        let room = (WIDTH - 6) as usize;
        assert_eq!(
            full.chars().count(),
            room,
            "the common width fits the names exactly"
        );
        for (page, _, _) in PAGES {
            assert_eq!(line(*page, room), full);
            let tabs = tabs(*page, &b, room);
            let here: Vec<usize> = (0..tabs.len()).filter(|i| tabs[*i].1).collect();
            assert_eq!(here.len(), 1);
            assert_eq!(PAGES[here[0]].0, *page);
        }
        // A narrower console shortens every page's line alike.
        // Short enough for an 80-column console (74 cells inside the frame and its margins).
        let short = "1 Char · 2 Inv · 3 Eqp · 4 Skl · 5 Weap · 6 Mag · 7 Qst · 8 Tal · 9 Story";
        assert!(short.chars().count() <= 74);
        let n = short.chars().count();
        assert_eq!(line(ScreenName::Inventory, room - 1), short);
        assert_eq!(line(ScreenName::Talents, n), short);
        assert_eq!(
            line(ScreenName::Talents, n - 1),
            "1 · 2 · 3 · 4 · 5 · 6 · 7 · 8 · 9"
        );
        assert_eq!(line(ScreenName::Talents, 32), "");
    }

    #[test]
    fn a_hint_names_a_page_by_its_key_and_name() {
        let b = Bindings::default();
        assert_eq!(hint(ScreenName::Magic, &b).as_deref(), Some("6 Magic"));
        assert_eq!(hint(ScreenName::Messages, &b), None);
    }

    #[test]
    fn every_page_is_at_least_the_common_width() {
        assert_eq!(panel_width(52), WIDTH);
        assert_eq!(panel_width(72), WIDTH);
        assert_eq!(panel_width(120), 120);
    }
}
