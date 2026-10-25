//! Character creation's Talents step (spec §6 "New run", §8 "Talents"; #236): after the class
//! and before the name, the character picks its creation talents, two for most (the bard, the
//! lucky class, three). It lists the talents the drafted character may take (the run as it
//! would start, `World::draft`: its scores, skills, class abilities, and sign), grouped by tree,
//! each with the start of its description, as the one paged list. A letter, or Enter on the `>`
//! row, marks a talent (`*`), or unmarks it; a talent with several choices lists them in its
//! place first. Marking one may open another (Fast healer once Healthy is marked); unmarking one
//! drops the picks that needed it. With every pick marked the step moves on to the name. The
//! **Random** row takes the character's Random picks, the ones a run started without picks gets
//! (`World::random_creation`), and moves on. Esc steps back to the class; the picks are
//! remembered (`Picks::talents`) and kept where they still hold.
//!
//! A plain state machine like every screen, testable without a window.

use super::character_screen::fit_joined;
use super::chargen_screen::{Draft, WIDTH};
use super::menu::{Hints, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::{ScreenAction, ScreenName};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use std::sync::Arc;
use warlock_core::World;
use warlock_core::content::{Content, TalentId, TreeId};
use warlock_core::world::pick_text;

/// Rows besides the list: the frame, a margin, a gap, the description (three lines), a gap,
/// the footer, the frame.
const EXTRA: i32 = 9;

/// How the list groups its rows: by tree, after the Random (and Done) rows under a heading of their own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Group {
    Chance,
    Tree(TreeId),
}

/// One row: a talent, one of its choices, or Random.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    /// The talent (`None`: the Random row).
    talent: Option<TalentId>,
    name: String,
    description: String,
    /// The picks it may be taken as (`id`, or `id:choice` for each choice).
    ways: Vec<String>,
    /// The Done row (with every pick marked): go on with them.
    done: bool,
}

#[derive(Debug)]
pub struct ChargenTalentsScreen {
    /// The run as it would start, its picks due; `None` when there is nothing to draft (content
    /// without races or classes, or a draft without a scenario).
    draft: Option<World>,
    content: Arc<Content>,
    /// How many talents the character picks.
    count: u8,
    /// The picks marked, in the order marked.
    picks: Vec<String>,
    entries: PagedList<Entry, Group>,
    /// A talent's choices, listed in its place.
    choosing: Option<PagedList<Entry, Group>>,
    /// Why a press did nothing, until the next.
    notice: Option<String>,
}

impl ChargenTalentsScreen {
    /// The step for the character drafted so far, with `remembered` picks (from before going
    /// back) marked where they still hold.
    pub fn new(content: &Arc<Content>, draft: &Draft, remembered: Option<&[String]>) -> Self {
        let world = (draft.scenario.as_ref())
            .zip(draft.spec(content))
            .and_then(|(s, spec)| World::draft(content.clone(), &s.id, Some(s.seed), &spec).ok());
        let count = world
            .as_ref()
            .and_then(|w| w.creation_offers(&[]).ok())
            .map_or(0, |(left, _)| left);
        let mut screen = Self {
            draft: world,
            content: content.clone(),
            count,
            picks: Vec::new(),
            entries: PagedList::empty(),
            choosing: None,
            notice: None,
        };
        for pick in remembered.unwrap_or_default() {
            screen.picks.push(pick.clone());
            if !screen.holds() || screen.picks.len() > usize::from(count) {
                screen.picks.pop();
            }
        }
        screen.rebuild();
        // Nothing marked yet: the cursor on Random, so Enter takes what a run gets by default;
        // every pick marked (coming back): on Done.
        let at = match screen.left() {
            0 => screen.entries.iter().position(|e| e.done),
            _ if screen.picks.is_empty() => screen.entries.iter().position(|e| e.talent.is_none()),
            _ => None,
        };
        if let Some(at) = at {
            screen.entries.select(at);
        }
        screen
    }

    /// The picks marked, in order.
    pub fn picks(&self) -> &[String] {
        &self.picks
    }

    /// Whether the picks marked may all be taken, in order.
    fn holds(&self) -> bool {
        (self.draft.as_ref()).is_some_and(|w| w.creation_offers(&self.picks).is_ok())
    }

    /// The picks still to mark.
    fn left(&self) -> usize {
        usize::from(self.count).saturating_sub(self.picks.len())
    }

    /// Lists the talents: those the character may take with no pick marked, those the picks
    /// open, and those marked; then the Random row. The cursor stays on the talent it was on.
    fn rebuild(&mut self) {
        let content = self.content.clone();
        let book = content.talents();
        let selected = self.entries.selected().and_then(|e| e.talent);
        let mut found: Vec<(TalentId, Vec<Option<String>>)> = Vec::new();
        let mut add = |offers: Vec<(TalentId, Vec<Option<String>>)>| {
            for (id, ways) in offers {
                match found.iter_mut().find(|(t, _)| *t == id) {
                    Some((_, known)) => {
                        for way in ways {
                            if !known.contains(&way) {
                                known.push(way);
                            }
                        }
                    }
                    None => found.push((id, ways)),
                }
            }
        };
        if let Some(world) = &self.draft {
            add(world.creation_offers(&[]).map_or(Vec::new(), |(_, o)| o));
            for n in 1..=self.picks.len() {
                let marked = &self.picks[..n];
                add(world.creation_offers(marked).map_or(Vec::new(), |(_, o)| o));
            }
        }
        // The marked ones are listed even when their last rank is taken.
        for pick in &self.picks {
            let (id, choice) = split(pick);
            if let Some(talent) = book.id(id) {
                add(vec![(talent, vec![choice.map(str::to_string)])]);
            }
        }
        found.sort_by_key(|(id, _)| (book.get(*id).tree, *id));
        let mut items: Vec<(Group, Entry)> = (found.into_iter())
            .map(|(id, ways)| {
                let def = book.get(id);
                let ways = (ways.iter())
                    .map(|w| pick_text(&def.id, w.as_deref()))
                    .collect();
                let entry = Entry {
                    talent: Some(id),
                    name: def.name.clone(),
                    description: def.description.clone(),
                    ways,
                    done: false,
                };
                (Group::Tree(def.tree), entry)
            })
            .collect();
        if self.draft.is_some() {
            let row = |name: &str, description: &str, done| Entry {
                talent: None,
                name: name.into(),
                description: description.into(),
                ways: Vec::new(),
                done,
            };
            if self.left() == 0 {
                items.push((
                    Group::Chance,
                    row("Done", "Go on with these talents.", true),
                ));
            }
            items.push((
                Group::Chance,
                row("Random", "Let chance pick your talents.", false),
            ));
        }
        self.entries = PagedList::grouped(items, |group| match group {
            Group::Tree(tree) => book.tree(tree).name.clone(),
            Group::Chance => "Chance".into(),
        });
        let at = (self.entries.iter()).position(|e| e.talent.is_some() && e.talent == selected);
        if let Some(at) = at {
            self.entries.select(at);
        }
    }

    /// The list shown: a talent's choices while they are listed, else the talents.
    fn list(&self) -> &PagedList<Entry, Group> {
        self.choosing.as_ref().unwrap_or(&self.entries)
    }

    fn list_mut(&mut self) -> &mut PagedList<Entry, Group> {
        self.choosing.as_mut().unwrap_or(&mut self.entries)
    }

    /// Whether the talent of `entry` is marked.
    fn marked(&self, entry: &Entry) -> bool {
        entry.ways.iter().any(|w| self.picks.contains(w))
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        self.notice = None;
        if press.nav == Nav::Back {
            if self.choosing.take().is_none() {
                return ScreenAction::Close;
            }
            return ScreenAction::Stay;
        }
        if let Some(c) = press.text.filter(char::is_ascii_lowercase) {
            return match self.list().at_letter(c) {
                Some(index) => self.choose(index),
                None => ScreenAction::Stay,
            };
        }
        match press.nav {
            // Nothing to pick (no draft): move on.
            Nav::Enter if self.draft.is_none() => ScreenAction::Open(ScreenName::Name),
            Nav::Enter => self.choose(self.list().cursor()),
            nav => {
                self.list_mut().navigate(nav);
                ScreenAction::Stay
            }
        }
    }

    /// Chooses row `index`: marks or unmarks its talent (or its choice), takes the Random
    /// picks, or lists the talent's choices.
    fn choose(&mut self, index: usize) -> ScreenAction {
        self.list_mut().select(index);
        let Some(entry) = self.list().get(index).cloned() else {
            return ScreenAction::Stay;
        };
        if entry.done {
            return ScreenAction::Open(ScreenName::Name);
        }
        if entry.talent.is_none() {
            self.picks = self
                .draft
                .as_ref()
                .map_or(Vec::new(), World::random_creation);
            self.rebuild();
            return ScreenAction::Open(ScreenName::Name);
        }
        let in_choices = self.choosing.is_some();
        if self.marked(&entry) && !in_choices {
            // Unmark it, and drop the picks after it that no longer hold.
            self.picks.retain(|p| !entry.ways.contains(p));
            let mut kept = Vec::new();
            for pick in std::mem::take(&mut self.picks) {
                self.picks = kept.clone();
                self.picks.push(pick.clone());
                if self.holds() {
                    kept.push(pick);
                }
            }
            self.picks = kept;
            self.rebuild();
            return ScreenAction::Stay;
        }
        if entry.ways.len() > 1 && !in_choices {
            let items = (entry.ways.iter())
                .map(|way| {
                    let (id, choice) = split(way);
                    let name = (self.content.talents().id(id))
                        .map(|t| {
                            self.content
                                .talents()
                                .get(t)
                                .name_for(&self.content, choice)
                        })
                        .unwrap_or_else(|| way.clone());
                    let choice = Entry {
                        talent: entry.talent,
                        name,
                        description: entry.description.clone(),
                        ways: vec![way.clone()],
                        done: false,
                    };
                    (Group::Chance, choice)
                })
                .collect();
            let title = entry.name.clone();
            self.choosing = Some(PagedList::grouped(items, move |_| title.clone()));
            return ScreenAction::Stay;
        }
        if self.left() == 0 {
            self.notice = Some(format!(
                "You have chosen {} talents: unmark one first.",
                self.count
            ));
            return ScreenAction::Stay;
        }
        let Some(way) = entry.ways.first().cloned() else {
            return ScreenAction::Stay;
        };
        self.picks.push(way);
        if !self.holds() {
            self.picks.pop();
            self.notice = Some(format!("You cannot take {} yet.", entry.name));
            return ScreenAction::Stay;
        }
        self.choosing = None;
        self.rebuild();
        match self.left() {
            0 => ScreenAction::Open(ScreenName::Name),
            _ => ScreenAction::Stay,
        }
    }

    /// Lays the list out for a console of `rows` (drawing calls it, and the stack before a
    /// press, so a letter names what is drawn).
    pub fn fit(&self, _cols: u32, rows: u32) {
        let room = (rows as i32 - EXTRA).max(1) as usize;
        self.entries.set_room(room);
        if let Some(list) = &self.choosing {
            list.set_room(room);
        }
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        self.fit(console.cols(), console.rows());
        let list = self.list();
        let count = list.tallest().max(1) as i32;
        let title = format!(" Choose your talents (4 of 5): {} left ", self.left());
        let panel = Panel::new(console, &title, WIDTH, count + EXTRA);
        list.draw_label(console, &panel);
        let (x, top) = (panel.inner_x(), panel.y + 2);
        let fits = |y: i32| y < panel.y + panel.h - 2;
        if list.is_empty() && fits(top) {
            console.print(x + 2, top, "There are no talents to choose.", TEXT_DIM);
        }
        let name_w = (list.iter().map(|e| e.name.chars().count()).max()).unwrap_or(0) as i32;
        let cursor = list.cursor();
        let visible = (panel.h - EXTRA).max(1) as usize;
        for (row, y) in list.rows().into_iter().take(visible).zip(top..) {
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
            let Some(entry) = list.get(index) else {
                continue;
            };
            let selected = index == cursor;
            let marked = self.marked(entry);
            let color = if selected { ACCENT } else { TEXT };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            let mark = if marked { '*' } else { ' ' };
            console.print(x + 2, y, &format!("{letter} {mark}{}", entry.name), color);
            // The start of what it does, as room allows.
            let at = x + 5 + name_w + 2;
            let room = (panel.x + panel.w - 2 - at).max(0) as usize;
            if self.choosing.is_none() && entry.talent.is_some() && room > 8 {
                let mut text: String = entry.description.chars().take(room).collect();
                if entry.description.chars().count() > room {
                    text = text.chars().take(room - 1).collect::<String>() + "…";
                }
                console.print(at, y, &text, TEXT_DIM);
            }
        }
        let (text, color) = match (&self.notice, list.selected()) {
            (Some(notice), _) => (notice.clone(), ACCENT),
            (None, Some(entry)) => (entry.description.clone(), TEXT_DIM),
            (None, None) => (String::new(), TEXT_DIM),
        };
        let words: Vec<String> = text.split(' ').map(String::from).collect();
        let lines = fit_joined(&words, " ", panel.inner_w());
        for (line, y) in lines.iter().take(3).zip(panel.y + panel.h - 6..) {
            if fits(y) && y >= top + count {
                console.print(x, y, line, color);
            }
        }
        let keys = Hints::of(bindings);
        let mut footer = format!("{} or a letter mark", keys.confirm);
        if let Some(hint) = list.hint(&keys) {
            footer = format!("{footer}   {hint}");
        }
        footer = format!("{footer}   Esc back");
        panel.footer(console, &footer);
    }
}

/// A pick as written: `id` or `id:choice`.
fn split(pick: &str) -> (&str, Option<&str>) {
    match pick.split_once(':') {
        Some((id, choice)) => (id, Some(choice)),
        None => (pick, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::chargen_screen::ScenarioPick;
    use warlock_core::character::CharacterSpec;

    fn content() -> Arc<Content> {
        Arc::new(Content::builtin().expect("content"))
    }

    fn draft(content: &Content, character: &str) -> Draft {
        let showcase = content.scenario("showcase").expect("showcase");
        let spec: CharacterSpec = character.parse().expect("a character");
        let mut draft = Draft {
            scenario: Some(ScenarioPick {
                id: "showcase".into(),
                name: showcase.name.clone(),
                seed: 1,
                shows_seed: false,
                character: Some(spec.clone()),
                start: showcase.start,
            }),
            ..Draft::default()
        };
        draft.chosen.race = Some(spec.race.clone());
        draft.chosen.gender = Some(spec.gender.id().to_string());
        draft.chosen.class = Some(spec.class.clone());
        draft
    }

    fn at(screen: &ChargenTalentsScreen, id: &str) -> usize {
        let talent = screen.content.talents().id(id).expect("a talent");
        (screen.entries.iter())
            .position(|e| e.talent == Some(talent))
            .expect("listed")
    }

    #[test]
    fn marking_picks_moves_on_and_a_chain_opens() {
        let content = content();
        let mut screen =
            ChargenTalentsScreen::new(&content, &draft(&content, "dwarf,male,warrior"), None);
        assert_eq!(screen.count, 2);
        let book = content.talents();
        let fast = book.id("fast_healer").expect("a talent");
        assert!(!screen.entries.iter().any(|e| e.talent == Some(fast)));
        assert_eq!(screen.choose(at(&screen, "healthy")), ScreenAction::Stay);
        assert_eq!(screen.picks(), ["healthy"]);
        // Healthy marked opens Fast healer (a dwarf warrior is tough enough).
        assert!(screen.entries.iter().any(|e| e.talent == Some(fast)));
        // Unmarking Healthy unmarks nothing else, and closes it again.
        screen.choose(at(&screen, "healthy"));
        assert!(screen.picks().is_empty());
        screen.choose(at(&screen, "healthy"));
        assert_eq!(
            screen.choose(at(&screen, "fast_healer")),
            ScreenAction::Open(ScreenName::Name)
        );
        assert_eq!(screen.picks(), ["healthy", "fast_healer"]);
        // Unmarking Healthy now drops Fast healer, which needed it.
        screen.choose(at(&screen, "healthy"));
        assert!(screen.picks().is_empty(), "{:?}", screen.picks());
    }

    #[test]
    fn random_takes_the_characters_random_picks() {
        let content = content();
        let draft = draft(&content, "human,female,bard");
        let mut screen = ChargenTalentsScreen::new(&content, &draft, None);
        assert_eq!(screen.count, 4, "the bard's and Versatile's");
        let random = (screen.entries.iter())
            .position(|e| e.name == "Random")
            .expect("a Random row");
        assert_eq!(screen.choose(random), ScreenAction::Open(ScreenName::Name));
        let spec = draft.spec(&content).expect("a character");
        let world = World::draft(content.clone(), "showcase", Some(1), &spec).expect("draft");
        assert_eq!(screen.picks(), world.random_creation());
        // Remembered picks are marked again where they hold.
        let again = ChargenTalentsScreen::new(&content, &draft, Some(screen.picks()));
        assert_eq!(again.picks(), screen.picks());
        assert_eq!(screen.handle(&Press::nav(Nav::Back)), ScreenAction::Close);
    }
}
