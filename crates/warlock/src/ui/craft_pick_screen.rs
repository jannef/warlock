//! The forging wizard (spec §8 "Skills": professions; docs/design/professions.md §8; #1335):
//! what a generic recipe's row on a profession's screen opens. One paged list for each choice of
//! the recipe, in the order the rules give them (`PerceptionView::craft_steps`): the stock for
//! each chosen part ("blade"; the arrows' head: arrowheads, then shards and blocks, after a
//! **None** row that makes them brittle), each input, then the ornament (the cut gems carried,
//! after a **None** row on `-`). A row is a stack as the inventory lists it, with how many the
//! recipe takes of it ("uses 3", an arrowhead "uses 1 each"); a stack too small is greyed "too
//! few" and cannot be picked. The header carries the recipe's name and the running choice ("Long sword — blade: steel"). A
//! letter on the page, or Enter on the `>` row, picks and goes to the next step; `-` picks None
//! where a step allows it; Esc goes back a step, its pick kept (the list opens on
//! it), and closes the wizard from the first. The last pick opens the one dialog
//! (`ScreenName::CraftConfirm`), which asks with what the rules say of the picks; declined, it
//! closes back to the last step; accepted, it applies `Action::Craft`. After the work the
//! wizard opens again on its first step with the pack as it is now. Every key is fixed.
//!
//! An enchanting's later steps follow its earlier picks (`PerceptionView::craft_steps_given`,
//! read again after each pick, [`CraftPickScreen::reread`]): the seals list the families of the
//! crystals picked in words, the affix kept the item's affixes; a seal's None skips the seals
//! after it (and Esc skips back over them).

use super::dialog_screen::Dialog;
use super::menu::{HINTS, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::{ScreenAction, ScreenName};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use warlock_core::Action;
use warlock_core::content::{ItemClassId, RecipeId};
use warlock_core::perception::PerceptionView;
use warlock_core::world::{CraftOption, CraftStep, CraftStepKind};

/// Panel width: room for a stack's listed name with what the recipe takes of it.
const WIDTH: i32 = 64;

/// The dialog the wizard's last pick opens (`ScreenName::CraftConfirm`): "Forge a
/// ruby-pommelled long sword? It uses 3 steel ingots and the ruby, at the anvil, and takes about
/// 1 hour.", the profession's verb its title and its yes; the rules' reason, and only Back, when
/// the picks do not hold. Its no closes it, back to the last step.
pub fn craft_dialog(
    run: Option<&PerceptionView<'_>>,
    craft: Option<(RecipeId, Vec<Option<usize>>)>,
) -> Dialog {
    let (Some(view), Some((recipe, picks))) = (run, craft) else {
        return Dialog::new("Make", "There is nothing to make.")
            .cancel("Back", ScreenAction::Close);
    };
    let plan = match view.craft_plan(recipe, &picks) {
        Ok(plan) => plan,
        Err(why) => return Dialog::new("Make", why).cancel("Back", ScreenAction::Close),
    };
    let at = plan
        .station
        .map_or(String::new(), |s| format!(", at the {s}"));
    // A work that uses nothing up and needs no station (Enchanting's Unbind) says only its time.
    let text = match plan.uses.is_empty() && at.is_empty() {
        true => format!("{} {}? It takes {}.", plan.verb, plan.name, plan.time),
        false => format!(
            "{} {}? It uses {}{at}, and takes {}.",
            plan.verb, plan.name, plan.uses, plan.time
        ),
    };
    let action = Action::Craft {
        recipe: view.content().recipe(recipe).id.clone(),
        picks,
    };
    Dialog::yes_no(&plan.verb, text, &plan.verb, action, "Not now")
}

#[derive(Debug)]
pub struct CraftPickScreen {
    recipe: Option<RecipeId>,
    /// The recipe's name: "Long sword".
    name: String,
    steps: Vec<CraftStep>,
    /// The step shown.
    step: usize,
    /// A pick for every step, kept when going back (`None`: not made, or None chosen).
    picks: Vec<Option<usize>>,
    /// Each step's stacks, grouped by item class.
    lists: Vec<PagedList<CraftOption>>,
    /// The steps with no pick made, to tell a changed pack from picks that narrowed them.
    base: Vec<CraftStep>,
    /// A pick was made since the steps were read: they are read again ([`Self::reread`]).
    stale: bool,
}

impl CraftPickScreen {
    /// The wizard of `recipe`, read from the player's perception (`None` outside a run: empty);
    /// `recipe` `None` is the first recipe with a choice (`--screen craft-pick`).
    pub fn new(view: Option<&PerceptionView<'_>>, recipe: Option<RecipeId>) -> Self {
        let mut screen = Self {
            recipe,
            name: String::new(),
            steps: Vec::new(),
            step: 0,
            picks: Vec::new(),
            lists: Vec::new(),
            base: Vec::new(),
            stale: false,
        };
        if let Some(view) = view {
            screen.read(view);
        }
        screen
    }

    /// Reads the steps again from `view` (the first recipe with a choice when none is set): on
    /// the first step, no pick made.
    fn read(&mut self, view: &PerceptionView<'_>) {
        let content = view.content();
        let recipes = content.recipes();
        let recipe = self.recipe.or_else(|| {
            let generic = (recipes.list.iter().enumerate()).filter(|(_, r)| r.result.is_some());
            generic
                .map(|(i, _)| RecipeId(i as u16))
                .find(|&id| !view.craft_steps(id).is_empty())
        });
        let Some(recipe) = recipe else {
            return;
        };
        self.recipe = Some(recipe);
        self.name = content.recipe(recipe).name.clone();
        self.steps = view.craft_steps(recipe);
        self.base = self.steps.clone();
        self.stale = false;
        self.picks = vec![None; self.steps.len()];
        self.step = 0;
        self.lists = (0..self.steps.len())
            .map(|i| self.list_of(view, i))
            .collect();
    }

    /// The pack changed (the work done): when the steps differ, the wizard starts over on them.
    pub fn refresh(&mut self, view: &PerceptionView<'_>) {
        let Some(recipe) = self.recipe else {
            return;
        };
        if view.craft_steps(recipe) != self.base {
            self.read(view);
        }
    }

    /// Whether a pick was made since the steps were read.
    pub fn is_stale(&self) -> bool {
        self.stale
    }

    /// Reads the steps again as the picks made narrow them (the module docs): a step whose
    /// options changed gets its new list, and loses a pick no longer among them.
    pub fn reread(&mut self, view: &PerceptionView<'_>) {
        let Some(recipe) = self.recipe.filter(|_| self.stale) else {
            return;
        };
        self.stale = false;
        let steps = view.craft_steps_given(recipe, &self.picks);
        if steps.len() != self.steps.len() {
            return;
        }
        for (i, step) in steps.into_iter().enumerate() {
            if step == self.steps[i] {
                continue;
            }
            if (self.picks[i]).is_some_and(|at| !step.options.iter().any(|o| o.at == at)) {
                self.picks[i] = None;
            }
            self.steps[i] = step;
            self.lists[i] = self.list_of(view, i);
        }
        self.go_to(self.step);
    }

    /// The list of the step shown.
    fn list(&self) -> &PagedList<CraftOption> {
        &self.lists[self.step.min(self.lists.len().saturating_sub(1))]
    }

    /// The list of step `step`'s stacks, grouped by their item class.
    fn list_of(&self, view: &PerceptionView<'_>, step: usize) -> PagedList<CraftOption> {
        let Some(step) = self.steps.get(step) else {
            return PagedList::empty();
        };
        if matches!(step.kind, CraftStepKind::Family | CraftStepKind::Affix) {
            // Not stacks: one group, headed by what they are.
            let header = match step.kind {
                CraftStepKind::Family => "Families",
                _ => "Its magic",
            };
            let items = (step.options.iter().cloned())
                .map(|o| (ItemClassId(0), o))
                .collect();
            return PagedList::grouped(items, |_| header.to_string());
        }
        let content = view.content();
        let pack = view.inventory();
        let items = (step.options.iter())
            .filter_map(|o| Some((content.object(pack.get(o.at)?.object).class, o.clone())))
            .collect();
        PagedList::new(items, content)
    }

    /// The recipe the wizard makes, and the picks so far, in step order (the dialog's).
    pub fn chosen(&self) -> Option<(RecipeId, Vec<Option<usize>>)> {
        Some((self.recipe?, self.picks.clone()))
    }

    /// Lays the list out for a console of `cols` by `rows`: the frame, the margin, the header
    /// and None lines, the gaps, and the footer leave the rest to a page.
    pub fn fit(&self, _cols: u32, rows: u32) {
        for list in &self.lists {
            list.set_room((rows as i32 - 10).max(1) as usize);
        }
    }

    /// Applies a key press (the module docs).
    pub fn handle(&mut self, press: &Press, _bindings: &Bindings) -> ScreenAction {
        if press.nav == Nav::Back {
            if self.step == 0 {
                return ScreenAction::Close;
            }
            // Back over seals skipped by a None, to the first of them.
            let mut to = self.step - 1;
            while to > 0 && self.skipped_seal(to) && self.skipped_seal(to - 1) {
                to -= 1;
            }
            self.go_to(to);
            return ScreenAction::Stay;
        }
        let Some(step) = self.steps.get(self.step) else {
            return match press.nav {
                Nav::Enter => ScreenAction::Close,
                _ => ScreenAction::Stay,
            };
        };
        if press.text == Some('-') {
            return match step.optional {
                true => self.pick(None),
                false => ScreenAction::Stay,
            };
        }
        let step = self.step;
        if let Some(c) = press.text.filter(char::is_ascii_lowercase) {
            return match self.lists[step].at_letter(c) {
                Some(row) => {
                    self.lists[step].select(row);
                    self.choose(row)
                }
                None => ScreenAction::Stay,
            };
        }
        match press.nav {
            Nav::Enter => self.choose(self.lists[step].cursor()),
            nav => {
                self.lists[step].navigate(nav);
                ScreenAction::Stay
            }
        }
    }

    /// Picks list item `row`, if the stack holds enough.
    fn choose(&mut self, row: usize) -> ScreenAction {
        match self.list().get(row).filter(|o| o.enough) {
            Some(option) => self.pick(Some(option.at)),
            None => ScreenAction::Stay,
        }
    }

    /// Whether step `step` is a seal with None picked.
    fn skipped_seal(&self, step: usize) -> bool {
        self.steps[step].kind == CraftStepKind::Family && self.picks[step].is_none()
    }

    /// Makes the step's pick and goes on: to the next step, or after the last to the dialog. A
    /// seal's None is None for the seals after it too, which are passed over.
    fn pick(&mut self, at: Option<usize>) -> ScreenAction {
        self.picks[self.step] = at;
        self.stale = true;
        let mut next = self.step + 1;
        if at.is_none() && self.steps[self.step].kind == CraftStepKind::Family {
            while next < self.steps.len() && self.steps[next].kind == CraftStepKind::Family {
                self.picks[next] = None;
                next += 1;
            }
        }
        if next < self.steps.len() {
            self.go_to(next);
            return ScreenAction::Stay;
        }
        ScreenAction::Open(ScreenName::CraftConfirm)
    }

    /// Shows step `step`, the cursor on its pick if it has one.
    fn go_to(&mut self, step: usize) {
        self.step = step;
        let list = &mut self.lists[step];
        let row = (self.picks[step]).and_then(|at| list.iter().position(|o| o.at == at));
        if let Some(row) = row {
            list.select(row);
        }
    }

    pub fn draw(&self, console: &mut Console) {
        self.fit(console.cols(), console.rows());
        let count = self.list().tallest().max(1) as i32;
        let Some(step) = self.steps.get(self.step) else {
            let panel = Panel::new(console, " Forge ", WIDTH, 6);
            console.print(panel.inner_x(), panel.y + 2, "Nothing to choose.", TEXT_DIM);
            panel.footer(console, "Esc back");
            return;
        };
        let title = format!(" {}: {} ", self.name, step.label);
        let panel = Panel::new(console, &title, WIDTH, count + 9);
        self.list().draw_label(console, &panel);
        let x = panel.inner_x();
        let right = panel.x + panel.w - 2;
        let fits = |y: i32| y < panel.y + panel.h - 2;
        // The running choice: every step before the one shown.
        let chosen: Vec<String> = (self.steps.iter().zip(&self.picks))
            .take(self.step)
            .map(|(s, pick)| {
                let what = (pick.and_then(|at| s.options.iter().find(|o| o.at == at)))
                    .map_or("—".to_string(), |o| o.short.clone());
                format!("{}: {what}", s.label)
            })
            .collect();
        let header = match chosen.is_empty() {
            true => self.name.clone(),
            false => format!("{} — {}", self.name, chosen.join(", ")),
        };
        let header: String = header
            .chars()
            .take(panel.inner_w().max(0) as usize)
            .collect();
        console.print(x, panel.y + 2, &header, TEXT_DIM);
        let mut y = panel.y + 4;
        if step.optional && fits(y) {
            console.print(x + 2, y, "- - None", TEXT);
            y += 1;
        }
        if self.list().is_empty() && fits(y) {
            let none = match (step.optional, step.kind) {
                (true, CraftStepKind::Ornament) => {
                    "Nothing carried would do; None leaves it plain."
                }
                (true, CraftStepKind::Family) => "Nothing of the crystal's nature to seal.",
                (true, CraftStepKind::Affix) => "Choose the item first.",
                (true, _) => "Nothing carried would do; None makes them brittle.",
                (false, _) => "Nothing carried will do.",
            };
            console.print(x + 2, y, none, TEXT_DIM);
        }
        for (row, y) in self.list().rows().into_iter().zip(y..) {
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
            let Some(option) = self.list().get(index) else {
                continue;
            };
            let selected = index == self.list().cursor();
            let color = match (option.enough, selected) {
                (false, _) => TEXT_DIM,
                (true, true) => ACCENT,
                (true, false) => TEXT,
            };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            let state = match (step.kind, option.enough, option.each) {
                (CraftStepKind::Family, _, _) => "a crystal more".to_string(),
                (CraftStepKind::Affix, _, _) => "kept".to_string(),
                (_, true, true) => format!("uses {} each", option.takes),
                (_, true, false) => format!("uses {}", option.takes),
                (_, false, _) => "too few".to_string(),
            };
            let state_x = right - state.chars().count() as i32;
            let room = (state_x - x - 3).max(0) as usize;
            let name: String = format!("{letter} - {}", option.name)
                .chars()
                .take(room)
                .collect();
            console.print(x + 2, y, &name, color);
            console.print(state_x, y, &state, TEXT_DIM);
        }
        let keys = HINTS;
        let confirm = &keys.confirm;
        let mut hints = Vec::new();
        match self.list().last_letter() {
            Some('a') => hints.push(format!("a or {confirm} choose")),
            Some(last) => hints.push(format!("a-{last} or {confirm} choose")),
            None => {}
        }
        if step.optional {
            hints.push("- none".to_string());
        }
        if let Some(page) = self.list().hint(&keys) {
            hints.push(page);
        }
        hints.push("Esc back".to_string());
        panel.footer(console, &hints.join("   "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::content::Content;

    /// The forge with its stock picked up, beside the anvil.
    fn world() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut world = World::new(content, "forge", None).expect("world");
        for action in warlock_core::action::parse_actions("move:E pickup").expect("parses") {
            world.apply(&action);
        }
        world
    }

    #[test]
    fn letters_pick_esc_goes_back_keeping_the_pick_and_the_last_pick_asks() {
        let world = world();
        let view = world.perception();
        let sword = view
            .content()
            .recipes()
            .id("short_sword")
            .expect("short sword");
        let mut screen = CraftPickScreen::new(Some(&view), Some(sword));
        screen.fit(100, 60);
        let keys = Bindings::default();
        // The blade is required: `-` does nothing; the mithril (`c`) is picked.
        assert_eq!(screen.handle(&Press::typed('-'), &keys), ScreenAction::Stay);
        assert_eq!(screen.step, 0);
        assert_eq!(screen.handle(&Press::typed('c'), &keys), ScreenAction::Stay);
        assert_eq!(screen.step, 1);
        // Back to the blade, the cursor on the mithril; on again, then None for the pommel.
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &keys),
            ScreenAction::Stay
        );
        assert_eq!(screen.step, 0);
        let on = screen.list().selected().map(|o| o.short.clone());
        assert_eq!(on.as_deref(), Some("mithril"));
        screen.handle(&Press::nav(Nav::Enter), &keys);
        assert_eq!(
            screen.handle(&Press::typed('-'), &keys),
            ScreenAction::Open(ScreenName::CraftConfirm)
        );
        let (recipe, picks) = screen.chosen().expect("chosen");
        assert_eq!((recipe, picks.len()), (sword, 2));
        assert_eq!(picks[1], None);
        // The dialog names what will be made and how.
        let dialog = craft_dialog(Some(&view), screen.chosen());
        assert_eq!(dialog.title, "Forge");
        assert_eq!(
            dialog.text,
            "Forge a mithril short sword? It uses 2 mithril ingots, at the anvil, and takes \
             about 1 hour."
        );
        // Esc on the first step closes the wizard.
        let mut screen = CraftPickScreen::new(Some(&view), Some(sword));
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &keys),
            ScreenAction::Close
        );
    }

    /// A wizard trained in Enchanting with a long sword and four red crystals picked up.
    fn enchanter() -> World {
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.edit_table("classes", |text| {
            text.replace(
                "enchanting = { chance = 30,",
                "enchanting = { chance = 100,",
            )
        });
        files.insert(
            "levels/test_seal_ui.toml",
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n#####\n#@..#\n#####\n'''\n\
             [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
             [[item]]\nat = [2, 1]\nobject = \"long_sword\"\n\
             [[item]]\nat = [2, 1]\nobject = \"crystal_red\"\ncount = 4\n",
        );
        files.insert(
            "scenarios/test_seal_ui.toml",
            "name = \"T\"\nstart_level = \"test_seal_ui\"\n\
             character = { race = \"human\", gender = \"female\", class = \"wizard\" }\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut world = World::new(content, "test_seal_ui", Some(5)).expect("world");
        for action in warlock_core::action::parse_actions("move:E pickup").expect("parses") {
            world.apply(&action);
        }
        world
    }

    /// Picks the row of the step shown whose short name is `short`, then reads the steps again
    /// (as `route_keys` does after a press).
    fn choose_short(screen: &mut CraftPickScreen, view: &PerceptionView<'_>, short: &str) {
        let row = (screen.list().iter())
            .position(|o| o.short == short)
            .unwrap_or_else(|| panic!("{short} in step {}", screen.step));
        screen.lists[screen.step].select(row);
        screen.choose(row);
        screen.reread(view);
    }

    #[test]
    fn the_seals_follow_the_crystal_and_none_skips_the_rest() {
        let world = enchanter();
        let view = world.perception();
        let kindle = view.content().recipes().id("kindle").expect("kindle");
        let mut screen = CraftPickScreen::new(Some(&view), Some(kindle));
        screen.fit(100, 60);
        let labels: Vec<&str> = screen.steps.iter().map(|s| s.label.as_str()).collect();
        assert_eq!(
            labels,
            ["item", "crystal", "seal", "second seal", "third seal"]
        );
        choose_short(&mut screen, &view, "long sword");
        choose_short(&mut screen, &view, "red crystal");
        assert_eq!(screen.step, 2);
        assert_eq!(screen.steps[2].kind, CraftStepKind::Family);
        let words: Vec<String> = screen.list().iter().map(|o| o.name.clone()).collect();
        assert!(
            !words.is_empty() && !words.iter().any(|w| w == "mana"),
            "{words:?}"
        );
        // The first seal is not offered again; None then skips the third.
        let first = words[0].clone();
        choose_short(&mut screen, &view, &first);
        assert_eq!(screen.step, 3);
        assert!(!screen.list().iter().any(|o| o.name == first));
        let keys = Bindings::default();
        assert_eq!(
            screen.handle(&Press::typed('-'), &keys),
            ScreenAction::Open(ScreenName::CraftConfirm)
        );
        let (_, picks) = screen.chosen().expect("chosen");
        assert_eq!(picks.len(), 5);
        assert!(picks[2].is_some() && picks[3].is_none() && picks[4].is_none());
        let dialog = craft_dialog(Some(&view), screen.chosen());
        assert!(
            dialog
                .text
                .contains(&format!("one more red crystal, sealing out {first}")),
            "{}",
            dialog.text
        );
        // Esc from the skipped third seal goes back over it to the first one skipped.
        screen.go_to(4);
        screen.handle(&Press::nav(Nav::Back), &keys);
        assert_eq!(screen.step, 3);
    }
}
