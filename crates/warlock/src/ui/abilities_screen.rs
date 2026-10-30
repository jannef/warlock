//! The abilities menu (spec §6 "Abilities"; #156; `a` in play): everything the player can do
//! because of who they are — racial, class, skill, weapon skill and fighting style, and item
//! abilities — in one menu, unlike ADOM's key for each. It lists
//! `PerceptionView::abilities` (the one registry, `warlock_core::ability`) as the one paged list
//! (`paged_list`), grouped by where each comes from under a header per source, lettered down each
//! page from `a`. Each row has the ability's name and what it asks for ("in a direction"); a
//! spell's says its spell level and what it costs the player now first ("level 6, 11 mana"),
//! an active weapon ability its stamina ("3 stamina, in a direction"; never its time). One that
//! cannot be used now is greyed out with why ("you hold no shield"), and choosing it says so
//! instead. The selected ability's one-line description shows below the list.
//!
//! A letter, or Enter on the `>` row, uses an ability: the menu closes and the ability asks for
//! what it needs, as its action would (a direction in the message line), then is applied as that
//! action ([`ScreenAction::UseAbility`]). The menu opens on the ability used last, so Enter
//! repeats it (and so does the quick-repeat key, `A`, in play). Esc closes it, and so does its
//! own key when it names no row. Read from the perception snapshot when it opens; no game time
//! passes while it is open.

use super::character_screen::fit_joined;
use super::menu::{Hints, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use warlock_core::ability::{Ability, Aim, Source, Unavailable};
use warlock_core::perception::PerceptionView;

/// Width of the panel, at least: wider while a row needs it (a weapon ability's name after its
/// skill's, "Sword and Board: Staggering block"), as the console allows.
const WIDTH: i32 = 60;
/// Columns of a row besides its name and its state: the frame, the margins, the cursor, the
/// letter, and the gaps.
const ROW_EXTRA: i32 = 11;
/// Rows besides the list: the frame, a margin, a gap, the description (two lines), a gap, the
/// footer, the frame.
const EXTRA: i32 = 8;

/// One ability as the menu shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    ability: Ability,
    name: String,
    description: String,
    /// What it asks for if it can be used now ("in a direction"), else why not.
    state: String,
    available: bool,
}

#[derive(Debug)]
pub struct AbilitiesScreen {
    /// The abilities, grouped by source and paged; its cursor is the `>` row.
    entries: PagedList<Entry, Source>,
    /// Why the ability just chosen cannot be used now, shown in place of the description until
    /// the next press.
    notice: Option<String>,
}

impl AbilitiesScreen {
    /// The menu of the player's abilities (`None` outside a run: none), the cursor on `last`
    /// (the ability used last) if it is listed, else on the first that can be used now.
    pub fn new(view: Option<&PerceptionView<'_>>, last: Option<Ability>) -> Self {
        let entries = view.map_or(Vec::new(), |view| {
            let content = view.content();
            (view.abilities().into_iter())
                .map(|own| {
                    let a = own.ability;
                    let state = match own.unavailable {
                        Some(why) => why.reason(content),
                        None => needs(a.aim(content)).into(),
                    };
                    // A spell says its spell level and what it costs the player now, first;
                    // short of mana, that it is too little; one cast at a place, that it is
                    // aimed (the row has no room for more).
                    let state = match a {
                        Ability::Spell(spell) => {
                            let level = content.spell_of(spell).map_or(0, |s| s.level);
                            let cost = view.own_spell_cost(spell).unwrap_or(0);
                            let head = format!("level {level}, {cost} mana");
                            match own.unavailable {
                                Some(Unavailable::NoMana) => format!("{head}: too little").into(),
                                Some(_) => format!("{head}: {state}").into(),
                                None if state.is_empty() => head.into(),
                                None => format!("{head}, aimed").into(),
                            }
                        }
                        // An active weapon ability says what it costs the player in
                        // stamina, first (never its time).
                        Ability::Weapon(_, id) if content.weapon_ability(id).active.is_some() => {
                            let head = format!("{} stamina", view.own_weapon_stamina(id));
                            match own.unavailable {
                                Some(_) => format!("{head}: {state}").into(),
                                None if state.is_empty() => head.into(),
                                None => format!("{head}, {state}").into(),
                            }
                        }
                        _ => state,
                    };
                    let entry = Entry {
                        ability: a,
                        name: a.name(content).to_string(),
                        description: a.description(content).to_string(),
                        state: state.to_string(),
                        available: own.unavailable.is_none(),
                    };
                    (a.source(), entry)
                })
                .collect()
        });
        let mut entries = PagedList::grouped(entries, |source| source.name().to_string());
        let start = (entries.iter().position(|e| Some(e.ability) == last))
            .or_else(|| entries.iter().position(|e| e.available));
        if let Some(start) = start {
            entries.select(start);
        }
        Self {
            entries,
            notice: None,
        }
    }

    /// The ability on the `>` row, if any.
    #[cfg(test)]
    fn selected(&self) -> Option<Ability> {
        self.entries.selected().map(|e| e.ability)
    }

    /// Applies a key press: a letter on the page shown, or Enter on the `>` row, uses its
    /// ability (or says why it cannot be used now); ↑↓ move the cursor and the list keys turn
    /// the page; Esc closes the menu, and so does its own key when that names no row.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        self.notice = None;
        let own_key = (press.binding.as_ref())
            .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::Abilities));
        if press.nav == Nav::Back {
            return ScreenAction::Close;
        }
        // Letters name the page's abilities (even `j` and `k`, which move selections elsewhere).
        if let Some(c) = press.text.filter(char::is_ascii_lowercase) {
            return match self.entries.at_letter(c) {
                Some(index) => self.choose(index),
                None if own_key => ScreenAction::Close,
                None => ScreenAction::Stay,
            };
        }
        match press.nav {
            _ if own_key => return ScreenAction::Close,
            Nav::Enter if self.entries.is_empty() => return ScreenAction::Close,
            Nav::Enter => return self.choose(self.entries.cursor()),
            nav => {
                self.entries.navigate(nav);
            }
        }
        ScreenAction::Stay
    }

    /// Chooses entry `index`: uses its ability, or says why it cannot be used now.
    fn choose(&mut self, index: usize) -> ScreenAction {
        self.entries.select(index);
        match self.entries.get(index) {
            Some(entry) if entry.available => ScreenAction::UseAbility(entry.ability),
            Some(entry) => {
                self.notice = Some(format!("{}: {}.", entry.name, entry.state));
                ScreenAction::Stay
            }
            None => ScreenAction::Stay,
        }
    }

    /// Lays the list out for a console of `cols` by `rows`: how many rows a page may take.
    /// Drawing calls it, and so does the screen stack before a press, so a letter names what is
    /// drawn.
    pub fn fit(&self, _cols: u32, rows: u32) {
        self.entries.set_room((rows as i32 - EXTRA).max(1) as usize);
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        self.fit(console.cols(), console.rows());
        let count = self.entries.tallest().max(1) as i32;
        let longest = |f: fn(&Entry) -> &str| {
            (self.entries.iter())
                .map(|e| f(e).chars().count())
                .max()
                .unwrap_or(0) as i32
        };
        let wide = longest(|e| &e.name) + longest(|e| &e.state) + ROW_EXTRA;
        let width = wide.clamp(WIDTH, (console.cols() as i32).max(WIDTH));
        let panel = Panel::new(console, " Abilities ", width, count + EXTRA);
        self.entries.draw_label(console, &panel);
        let (x, top) = (panel.inner_x(), panel.y + 2);
        let fits = |y: i32| y < panel.y + panel.h - 2;
        if self.entries.is_empty() && fits(top) {
            console.print(x + 2, top, "You have no abilities.", TEXT_DIM);
        }
        // The name column: the longest name on any page, so turning the page keeps it.
        let name_w = (self.entries.iter())
            .map(|e| e.name.chars().count())
            .max()
            .unwrap_or(0) as i32;
        let cursor = self.entries.cursor();
        let visible = (panel.h - EXTRA).max(1) as usize;
        for (row, y) in self.entries.rows().into_iter().take(visible).zip(top..) {
            if !fits(y) {
                break;
            }
            let (index, letter) = match row {
                Row::Header(name) => {
                    draw_header(console, x, y, name);
                    continue;
                }
                Row::Item { index, letter } => (index, letter),
            };
            let Some(entry) = self.entries.get(index) else {
                continue;
            };
            let selected = index == cursor;
            // Greyed out when it cannot be used now; the reason beside it says why, so it reads
            // without color too.
            let color = match (entry.available, selected) {
                (false, _) => TEXT_DIM,
                (true, true) => ACCENT,
                (true, false) => TEXT,
            };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            console.print(x + 2, y, &format!("{letter}  {}", entry.name), color);
            let state_x = x + 5 + name_w + 2;
            let room = (panel.x + panel.w - 2 - state_x).max(0) as usize;
            let state: String = entry.state.chars().take(room).collect();
            console.print(state_x, y, &state, TEXT_DIM);
        }
        // Below the list: why the ability just chosen cannot be used, else what the selected
        // one does.
        let (text, color) = match (&self.notice, self.entries.selected()) {
            (Some(notice), _) => (notice.as_str(), ACCENT),
            (None, Some(entry)) => (entry.description.as_str(), TEXT_DIM),
            (None, None) => ("", TEXT_DIM),
        };
        let words: Vec<String> = text.split(' ').map(String::from).collect();
        let lines = fit_joined(&words, " ", panel.inner_w());
        // In a panel cut short, never over the list.
        for (line, y) in lines.iter().take(2).zip(panel.y + panel.h - 5..) {
            if fits(y) && y >= top + count {
                console.print(x, y, line, color);
            }
        }
        panel.footer(console, &self.footer(bindings));
    }

    /// The footer: the letters of the page shown and Enter to use, the page keys while there are
    /// pages, and how to leave, keys as bound.
    fn footer(&self, bindings: &Bindings) -> String {
        let keys = Hints::of(bindings);
        let page = (self.entries.hint(&keys)).map_or(String::new(), |hint| format!("{hint}   "));
        let confirm = &keys.confirm;
        let close = match bindings.keys_of(KeyCommand::Abilities).first() {
            Some(key) if self.entries.is_empty() => format!("{} or Esc close", key.display()),
            _ => "Esc close".to_string(),
        };
        match self.entries.last_letter() {
            Some('a') => format!("a or {confirm} use   {page}{close}"),
            Some(last) => format!("a-{last} or {confirm} use   {page}{close}"),
            None => close,
        }
    }
}

/// What an ability that can be used now asks for, as its row says it.
fn needs(aim: Aim) -> &'static str {
    match aim {
        Aim::Nothing => "",
        Aim::Direction => "in a direction",
        Aim::Cell => "at a place you see",
        Aim::Ailment => "on yourself",
        Aim::Item => "on something you carry",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::action::parse_actions;
    use warlock_core::content::Content;

    /// The showcase's thief, with the storeroom's small shield in hand if `shield`.
    fn thief(shield: bool) -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut world = World::new(content, "showcase", None).expect("world");
        if shield {
            let walk = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 \
                        move:N pickup:a equip:off_hand:b";
            for action in parse_actions(walk).expect("actions") {
                world.apply(&action);
            }
        }
        world
    }

    fn drawn(screen: &AbilitiesScreen, cols: u32, rows: u32) -> Vec<String> {
        let mut console = Console::default();
        console.resize(cols, rows);
        screen.draw(&mut console, &Bindings::default());
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect()
            })
            .collect()
    }

    #[test]
    fn lists_the_abilities_by_source_greying_out_what_cannot_be_used() {
        let world = thief(false);
        let screen = AbilitiesScreen::new(Some(&world.perception()), None);
        let text = drawn(&screen, 80, 30).join("\n");
        for expected in [
            " Abilities ",
            "Birth sign",
            "> a  Transfixing stare  at a place you see",
            "Skills",
            "  b  Disarm a trap      in a direction",
            "  c  Pick lock          in a direction",
            "  d  Steal              in a direction",
            "Weapon skills",
            "  e  Shield bash        you hold no shield",
            "a-e or Enter use   Esc close",
        ] {
            assert!(text.contains(expected), "{expected:?} in\n{text}");
        }
        // The cursor starts on the first that can be used: the thief's birth sign's stare (a
        // Basilisk's).
        let stare = Ability::Sign(warlock_core::content::SignActive::TransfixingStare);
        assert_eq!(screen.selected(), Some(stare));
        assert_eq!(screen.selected().map(|a| a.source()), Some(Source::Sign));
    }

    #[test]
    fn a_letter_uses_an_ability_that_can_be_used_and_says_why_not_otherwise() {
        let b = Bindings::default();
        let world = thief(false);
        let mut screen = AbilitiesScreen::new(Some(&world.perception()), None);
        assert_eq!(screen.handle(&Press::typed('e'), &b), ScreenAction::Stay);
        assert_eq!(screen.selected(), Some(Ability::ShieldBash));
        let text = drawn(&screen, 80, 30).join("\n");
        assert!(text.contains("Shield bash: you hold no shield."), "{text}");
        // The next press clears the notice; the description shows again.
        screen.handle(&Press::nav(Nav::Down), &b);
        let text = drawn(&screen, 80, 30).join("\n");
        assert!(!text.contains("you hold no shield."), "{text}");
        // With a shield in hand it can be used; the cursor starts on the first that can be used
        // (the birth sign's stare) unless another was used last.
        let world = thief(true);
        let mut screen = AbilitiesScreen::new(Some(&world.perception()), None);
        let stare = Ability::Sign(warlock_core::content::SignActive::TransfixingStare);
        assert_eq!(screen.selected(), Some(stare));
        assert_eq!(
            screen.handle(&Press::typed('c'), &b),
            ScreenAction::UseAbility(Ability::PickLock)
        );
        let last = Some(Ability::ShieldBash);
        let mut screen = AbilitiesScreen::new(Some(&world.perception()), last);
        assert_eq!(screen.selected(), Some(Ability::ShieldBash));
        assert_eq!(
            screen.handle(&Press::typed('e'), &b),
            ScreenAction::UseAbility(Ability::ShieldBash)
        );
        let text = drawn(&screen, 80, 30).join("\n");
        assert!(
            text.contains("> e  Shield bash        in a direction"),
            "{text}"
        );
        assert!(text.contains("Strike someone next to you with the shield you hold."));
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::UseAbility(Ability::ShieldBash)
        );
        assert_eq!(
            screen.handle(&Press::typed('d'), &b),
            ScreenAction::UseAbility(Ability::Steal)
        );
        // `a` names the first row here (the stare), so it does not close the menu; Esc does.
        assert_eq!(
            screen.handle(&Press::typed('a'), &b),
            ScreenAction::UseAbility(stare)
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
    }

    #[test]
    fn it_opens_on_the_ability_used_last() {
        let world = thief(false);
        let view = world.perception();
        let last = view.abilities()[2].ability;
        let screen = AbilitiesScreen::new(Some(&view), Some(last));
        assert_eq!(screen.selected(), Some(last));
    }

    #[test]
    fn empty_and_tiny_consoles() {
        let b = Bindings::default();
        let mut none = AbilitiesScreen::new(None, None);
        let text = drawn(&none, 80, 30).join("\n");
        assert!(text.contains("You have no abilities."), "{text}");
        assert!(text.contains("a or Esc close"), "{text}");
        let a = Press {
            binding: KeyBinding::from_text("a"),
            ..Press::typed('a')
        };
        assert_eq!(none.handle(&a, &b), ScreenAction::Close);
        let world = thief(true);
        let last = Some(Ability::ShieldBash);
        let screen = AbilitiesScreen::new(Some(&world.perception()), last);
        for (cols, rows) in [(48, 12), (30, 8), (5, 3), (0, 0)] {
            assert_eq!(drawn(&screen, cols, rows).len(), rows as usize);
        }
        // A short console pages the list, showing the cursor's page: letters restart on it.
        let last = Some(Ability::ShieldBash);
        let screen = AbilitiesScreen::new(Some(&world.perception()), last);
        let text = drawn(&screen, 80, 12).join("\n");
        assert!(text.contains("Page 3/3"), "{text}");
        assert!(text.contains("> a  Shield bash"), "{text}");
    }
}
