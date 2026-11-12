//! A profession's screen (spec §8 "Skills"; docs/design/professions.md §8; #1334): what its row
//! in the abilities menu ("Metalcraft…") opens. The recipes the player sees
//! (`PerceptionView::recipes`), in the one paged list grouped by the result's item class, the
//! built-in recipes (Temper, Whittle, Fletch) first under "Work". A row is the recipe's name and
//! what it takes in words ("a metal weapon or armour, an anvil, about 10 minutes"), never a
//! difficulty or the odds; one that cannot be made now is greyed with its reason in place of
//! what it takes ("no anvil beside you"), which the line under the list then gives in full. A
//! letter on the page, or Enter on the `>` row, makes a recipe that can be made: a built-in one
//! runs as from its old row (Temper's and Whittle's lists over this screen, Fletch at once);
//! a greyed one does nothing. The list keys turn the page; Esc closes it. Every key is the
//! list's own, fixed.

use super::menu::{Hints, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::{ScreenAction, ScreenName};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use warlock_core::Action;
use warlock_core::content::{ItemClassId, RecipeId, RecipeUse};
use warlock_core::perception::PerceptionView;
use warlock_core::skills::SkillId;

/// Panel width: room for a recipe's name with what it takes.
const WIDTH: i32 = 76;
/// The name column's least width, the letter included (wider for a longer name).
const NAME_W: i32 = 18;

/// One recipe as the list shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    id: RecipeId,
    /// The recipe's id: what the action names.
    key: String,
    name: String,
    /// What it takes, in words.
    takes: String,
    /// Why it cannot be made now, in words; `None` when it can.
    why: Option<String>,
    /// The built-in action it runs, if it is one.
    uses: Option<RecipeUse>,
    /// A generic recipe with choices to make: the forging wizard asks them.
    wizard: bool,
    /// A generic recipe that chooses nothing (a brew) and can be made: the dialog confirms it.
    confirm: bool,
}

#[derive(Debug)]
pub struct ProfessionScreen {
    skill: Option<SkillId>,
    /// The panel's title: "Metalcraft".
    title: String,
    /// By the result's item class; `None` (the built-in recipes, "Work") first.
    recipes: PagedList<Entry, Option<ItemClassId>>,
}

impl ProfessionScreen {
    /// The list of profession `skill`, read from the player's perception (`None` outside a
    /// run: empty); `skill` `None` is the first profession there is (`--screen profession`).
    pub fn new(view: Option<&PerceptionView<'_>>, skill: Option<SkillId>) -> Self {
        let Some(view) = view else {
            return Self {
                skill,
                title: String::new(),
                recipes: PagedList::empty(),
            };
        };
        let content = view.content();
        // `--screen profession`: the first profession the player sees recipes of (Jewelcraft's
        // are the trained's alone), else the first.
        let professions = content.recipes().professions();
        let skill = skill.or_else(|| {
            (professions.iter().copied())
                .find(|&s| !view.recipes(s).is_empty())
                .or_else(|| professions.first().copied())
        });
        let Some(skill) = skill else {
            return Self {
                skill,
                title: String::new(),
                recipes: PagedList::empty(),
            };
        };
        let rows = (view.recipes(skill).into_iter())
            .map(|row| {
                let def = content.recipe(row.id);
                let class = (def.result).map(|r| content.object(r.object).class);
                let wizard = !view.craft_steps(row.id).is_empty();
                let entry = Entry {
                    id: row.id,
                    key: def.id.clone(),
                    name: row.name,
                    takes: row.takes,
                    why: row.unavailable.map(|why| why.reason(content)),
                    uses: def.uses,
                    wizard,
                    confirm: !wizard && def.uses.is_none() && view.craft_plan(row.id, &[]).is_ok(),
                };
                (class, entry)
            })
            .collect();
        let recipes = PagedList::grouped(rows, |class| match class {
            None => "Work".to_string(),
            Some(class) => content.item_class(class).name.clone(),
        });
        Self {
            skill: Some(skill),
            title: content.skill(skill).name.clone(),
            recipes,
        }
    }

    /// The profession shown.
    pub fn skill(&self) -> Option<SkillId> {
        self.skill
    }

    /// The recipe on the `>` row when it chooses nothing and can be made (a brew), with its
    /// (empty) picks: what the confirming dialog asks about.
    pub fn chosen(&self) -> Option<(RecipeId, Vec<Option<usize>>)> {
        let entry = self.recipes.get(self.recipes.cursor())?;
        entry.confirm.then(|| (entry.id, Vec::new()))
    }

    /// The run changed (a recipe made): the rows read again, the cursor where it was.
    pub fn refresh(&mut self, view: &PerceptionView<'_>) {
        let cursor = self.recipes.cursor();
        *self = Self::new(Some(view), self.skill);
        if cursor < self.recipes.iter().count() {
            self.recipes.select(cursor);
        }
    }

    /// Lays the list out for a console of `cols` by `rows`: a page takes the rows the frame, the
    /// margin, the line under the list, the gaps, and the footer leave.
    pub fn fit(&self, _cols: u32, rows: u32) {
        self.recipes.set_room((rows as i32 - 8).max(1) as usize);
    }

    /// Applies a key press: a letter on the page shown, or Enter on the `>` row, makes that
    /// recipe; the list keys move the cursor and turn the page; Esc goes back.
    pub fn handle(&mut self, press: &Press, _bindings: &Bindings) -> ScreenAction {
        if press.nav == Nav::Back || (self.recipes.is_empty() && press.nav == Nav::Enter) {
            return ScreenAction::Close;
        }
        if let Some(c) = press.text.filter(char::is_ascii_lowercase) {
            return match self.recipes.at_letter(c) {
                Some(row) => {
                    self.recipes.select(row);
                    self.choose(row)
                }
                None => ScreenAction::Stay,
            };
        }
        match press.nav {
            Nav::Enter => return self.choose(self.recipes.cursor()),
            nav => {
                self.recipes.navigate(nav);
            }
        }
        ScreenAction::Stay
    }

    /// Makes the recipe of list item `row`, if it can be made now: a built-in one runs its own
    /// action (Temper's and Whittle's lists ask which item first); a greyed one does nothing.
    fn choose(&self, row: usize) -> ScreenAction {
        let Some(entry) = self.recipes.get(row).filter(|e| e.why.is_none()) else {
            return ScreenAction::Stay;
        };
        match entry.uses {
            Some(RecipeUse::Temper) => ScreenAction::Open(ScreenName::Temper),
            Some(RecipeUse::Whittle) => ScreenAction::Open(ScreenName::Whittle),
            Some(RecipeUse::Fletch) => ScreenAction::Act(Action::Fletch),
            Some(RecipeUse::Cut) => ScreenAction::Open(ScreenName::Cut),
            // Greyed "not usable yet" until P11 builds its action.
            Some(RecipeUse::Set) => ScreenAction::Stay,
            // The forging wizard asks for the picks, then the dialog; a recipe with none to make
            // is applied at once, the rules saying what comes of it.
            None if entry.wizard => ScreenAction::Open(ScreenName::CraftPick(entry.id)),
            // A brew chooses nothing: the dialog confirms what it uses.
            None if entry.confirm => ScreenAction::Open(ScreenName::CraftConfirm),
            None => ScreenAction::Act(Action::Craft {
                recipe: entry.key.clone(),
                picks: Vec::new(),
            }),
        }
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: margin, the list, a gap, the line under it, a gap, the footer, the frame.
        self.fit(console.cols(), console.rows());
        let count = self.recipes.tallest().max(1) as i32;
        let title = format!(" {}: recipes ", self.title);
        let panel = Panel::new(console, &title, WIDTH, count + 7);
        self.recipes.draw_label(console, &panel);
        let x = panel.inner_x();
        let right = panel.x + panel.w - 2;
        let fits = |y: i32| y < panel.y + panel.h - 2;
        let top = panel.y + 2;
        // The name column fits the longest name ("x - Greater potion of presence") and a gap.
        let longest = (self.recipes.iter())
            .map(|e| e.name.chars().count() as i32)
            .max()
            .unwrap_or(0);
        let name_w = NAME_W.max(longest + 5);
        if self.recipes.is_empty() && fits(top) {
            console.print(x + 2, top, "You know no recipe of it.", TEXT_DIM);
        }
        for (row, y) in self.recipes.rows().into_iter().zip(top..) {
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
            let Some(entry) = self.recipes.get(index) else {
                continue;
            };
            let selected = index == self.recipes.cursor();
            // Greyed out when it cannot be made now; the reason in place of what it takes says
            // why, so it reads without color too.
            let color = match (entry.why.is_some(), selected) {
                (true, _) => TEXT_DIM,
                (false, true) => ACCENT,
                (false, false) => TEXT,
            };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            console.print(x + 2, y, &format!("{letter} - {}", entry.name), color);
            let state_x = x + 2 + name_w;
            let room = (right - state_x).max(0) as usize;
            let state = entry.why.as_ref().unwrap_or(&entry.takes);
            let state: String = state.chars().take(room).collect();
            console.print(state_x, y, &state, TEXT_DIM);
        }
        // Under the list: what the selected recipe takes, in full.
        let under = panel.y + panel.h - 4;
        if let Some(entry) = self.recipes.selected().filter(|_| fits(under)) {
            let text = format!("Takes {}.", entry.takes);
            let text: String = text.chars().take(panel.inner_w().max(0) as usize).collect();
            console.print(x, under, &text, TEXT_DIM);
        }
        let keys = Hints::of(bindings);
        let confirm = &keys.confirm;
        let choose = match self.recipes.last_letter() {
            Some('a') => format!("a or {confirm}"),
            Some(last) => format!("a-{last} or {confirm}"),
            None => confirm.clone(),
        };
        let page = (self.recipes.hint(&keys)).map_or(String::new(), |hint| format!("{hint}   "));
        if self.recipes.is_empty() {
            panel.footer(console, "Esc back");
        } else {
            panel.footer(console, &format!("{choose} make   {page}Esc back"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::character::CharacterSpec;
    use warlock_core::content::Content;

    fn world(scenario: &str) -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        let spec: CharacterSpec = "human,female,ranger".parse().expect("a character");
        World::with_character(content, scenario, None, Some(&spec)).expect("world")
    }

    #[test]
    fn a_built_in_recipe_runs_its_own_action_and_a_greyed_one_nothing() {
        let world = world("forge");
        let content = world.content().clone();
        let metal = content.skill_id("metalcraft").expect("metalcraft");
        let wood = content.skill_id("woodcraft").expect("woodcraft");
        // Beside the forge's anvil: Temper opens its list over the screen.
        let mut screen = ProfessionScreen::new(Some(&world.perception()), Some(metal));
        screen.fit(100, 60);
        let press = |c| Press::typed(c);
        assert_eq!(
            screen.handle(&press('a'), &Bindings::default()),
            ScreenAction::Open(ScreenName::Temper)
        );
        // Woodcraft's rows: the built-ins first, Fletch then Whittle (path order), then the
        // hand-made weapons; all greyed without a knife.
        let mut screen = ProfessionScreen::new(Some(&world.perception()), Some(wood));
        screen.fit(100, 60);
        let names: Vec<&str> = screen.recipes.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names[..3], ["Fletch", "Whittle", "Club"]);
        assert_eq!(
            screen.recipes.get(0).and_then(|e| e.why.as_deref()),
            Some("no knife")
        );
        assert_eq!(
            screen.handle(&press('a'), &Bindings::default()),
            ScreenAction::Stay
        );
        // The first profession with no skill named, and nothing outside a run.
        assert_eq!(
            ProfessionScreen::new(Some(&world.perception()), None).skill(),
            Some(metal)
        );
        assert!(ProfessionScreen::new(None, None).recipes.is_empty());
    }
}
