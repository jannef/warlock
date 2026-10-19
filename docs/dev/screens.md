# Screens, input, and key commands

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/06-input-and-ui.md. Keep it current with the code, as CLAUDE.md says.

Modal screens (spec §6) — main menu, new run and character creation (race, gender, class,
summary), pause, settings, controls, the `?` help, the inventory (`i`), the equipment screen
(`e`) and its choice of what goes in a slot, an item's description (a letter in the inventory,
an upper-case slot letter on the equipment screen), the character screen (`@`), the weapon skills
screen (`W`), the quest log (`q`), choosing what to
pick up (`,`/`g` on several stacks), drop (`d`), or eat (`E`), the dialogue screen (opened by
`screens::converse` after an action this frame whose outcome is talk with topics,
`Event::Talked`; asking is `ScreenAction::Act(Action::Ask)`, which closes it, and the answer
reopens it on the topic asked, also when no topics are left), and the death screen (opened by
`screens::on_death` when `World::death` is set; Enter/Esc is `MenuChoice::LeaveRun`, back to the
main menu without saving; no `--screen`: capture a dead replay instead) — are one
screen stack (`ui/screens.rs`, the `ScreenStack` resource); add a screen there, never a second way
to open, read keys for, or draw one:
- **Open**: `Screens::open(ScreenName)` (a `SystemParam` over the stack, content, save store, and
  the run, if any) is the one way a screen opens: `?`, `i`, `e`, `@`, `W`, `,`, `d`, and Esc in play,
  sub-screens, the startup main menu and `--screen`, and the return to the main menu after a run.
  Screens that show the run read the perception snapshot when they open (no time passes while
  they are open but for an action chosen on one). Back/Esc pops the top screen, so a sub-screen
  returns to the screen it was opened from as it was left; `ScreenAction::Return(name)` pops
  until `name` is on top (the summary going back to a step). Sub-screens read what was chosen
  below them off the stack: character creation's steps (`ScreenStack::draft`: New run's scenario
  and seed, each step's selection) preselect what was chosen before going back
  (`ScreenStack::picks`), else the scenario's character, and the summary's Start is
  `MenuChoice::NewRun { scenario, seed, character }`; `EquipItem` takes the slot selected on the
  equipment screen beneath (`ScreenStack::selected_slot`), and `Describe` the stack looked at
  on the inventory or equipment screen beneath (`ScreenStack::looked_item`; the first stack
  without one, as `--screen describe` opens it over the inventory).
- **Input**: `input::route_keys` is the one reader of `KeyboardInput`. Stack not empty: the press
  goes to the top screen (`ScreenStack::handle` → `ScreenAction`: stay, close, open, a
  `MenuChoice`, or `Act(Action)`: close and apply a gameplay action, e.g. `pickup:ac`,
  `equip:body:b`; the screens beneath stay, and `screens::refresh`, after `SimSet`, shows the
  equipment screen the result, as in ADOM); otherwise
  it is a gameplay command in the current `InputMode`. Opens and closes apply at once, so the next
  press in the same frame goes to the new top; menu choices (start, resume, save-and-quit to menu,
  quit) are messages applied by `screens::apply_choice`, and the rest of that frame's presses are
  dropped. Held-key repeats never open, close, choose, or act.
- **Draw**: `ui::draw` is the one system that writes the UI console: when anything it shows
  changed, it clears the console, draws the gameplay UI if a run exists, then the top screen.
- **Dialog** (`ui/dialog_screen.rs`): the one question screen; **every** confirmation is this
  screen with its own parameters, never a screen or prompt of its own. A screen or gameplay
  answers `ScreenAction::Dialog(Box<Dialog>)` (`Dialog` converts `into()` it; `Screens::ask`
  opens it). A `Dialog` is a `title` (on the frame), the question `text` (wrapped), and
  `options`: each a `Choice` with a `label`, a `Role` (`Accept`, `Decline`, `Cancel`, `Plain`),
  and an `outcome`, any `ScreenAction` (`From<MenuChoice>` is `Choose`, `From<Action>` is `Act`,
  a `Dialog` is a follow-up that takes its place, `Close`). Build it with `Dialog::new(title,
  text).accept(..).decline(..).cancel(..).plain(..)`, or `Dialog::yes_no(title, text, yes,
  outcome, no)` for a plain question. It sizes itself to its content and draws the options in
  a column, each with its shortcut keys. Keys: the menu keys move the selection, Enter takes it;
  the `Context::Dialog` commands `KeyCommand::Yes`/`No` (`y`/`n` by default, rebindable; their
  own context because `y`/`n` move the player in play) pick the Accept option and the Decline
  option (else Cancel); Esc picks Cancel (else Decline; else closes); the safe option (Cancel,
  else Decline) is selected first. A menu choice pops the dialog and is applied as usual, an
  action closes it and is applied (`play.act`, which may answer with a dialog of its own: the
  sell offer), a follow-up dialog replaces it (`ScreenStack::handle`). Users: every quit path
  (`menu::quit_dialog(saving)` for the pause and main menu Quit and a bound `KeyCommand::Quit`,
  `save_to_menu_dialog`), the step onto a known trap (`input::trap_dialog`, spec §8 "Traps"),
  and the shop (`input.rs`: `leaving_dialog`, `sale_dialog`; spec §7 "Shops"). **A new
  question is a `Dialog` value at its call site**, e.g. `Dialog::yes_no("Quest", text,
  "Accept", Action::.., "Not now")` returned as `ScreenAction::Dialog`, with no screen, prompt,
  `InputMode`, or key handling of its own (quests are offered and accepted through dialogue
  topics, which are no question).
  `--screen dialog` shows the quit dialog. On wasm, quit saves then reloads the page
  (`storage::web::reload`; `AppExit` also reloads), and the main menu has no Quit.
- **Info pages** (`ui/info_tabs.rs`): the screens about the player (Character, Inventory,
  Equipment, Skills, Weapon skills, Quests) are an ordered group, `PAGES`, each with its name and
  a short name for the tab line ("Inv"). The tab line takes the first of these that fits the
  panel: every name; the current page's name and the others' short names; every short name; the
  current page's name and the others' keys; bare keys. While one is the top screen, the page keys replace
  it (`ScreenAction::Replace` -> `Screens::replace`: the stack depth is unchanged, so Esc still
  returns to where the first was opened) and each draws its tab line on the bottom border with
  `info_tabs::draw_tabs`. **A new info screen joins the group**: one entry in `PAGES` (its
  position is its number) and one `draw_tabs` call in its `draw`; never a separate way to switch.
  Sub-steps of a page (`EquipItem`, pick-up/drop/throw lists) are not pages and keep their
  number keys. The keys are the `Context::Pages` commands `Page(1..9)`, `NextPage` (Shift
  reverses), `PrevPage`: rebindable like all commands (`KeyCommand::context`: keys are unique
  per context, so Tab is `NextTarget` in play and `NextPage` here; `Bindings::owner_in` and
  `lookup` read one context). `Press::shift` carries Shift from the router. The dialogue screen
  is no page: its digits ask about topics.
- Prompts in the message line (open/close/bash/chat direction, rest, look mode, aiming) are
  `InputMode`s, not screens; a question (the step onto a known trap, `input::trap_dialog`) is
  the dialog (above). The rules never ask: a `move` steps onto whatever is there. `C` with
  exactly one creature in view next to the player chats with it at once
  (`input::chat_partner`), else asks for a direction.
- Screens are plain state machines tested without a window: `ui/menu.rs` (`Press`, `Panel`, main
  and pause menu), `ui/new_run_screen.rs`, `ui/settings_screen.rs`, `ui/controls_screen.rs` (also
  the `?` help: the same binding list, read-only), `ui/object_screen.rs` (inventory, pick up,
  drop, throw, eat: stacks by letter; in the choices letters always name stacks, even `h j k l`;
  the eat choice lists food lying here first, without letters, then the pack's by letter; in the
  inventory a letter opens the stack's description, and `i` closes it only when it names none),
  `ui/describe_screen.rs` (`DescribeScreen`: one stack as the player knows it, from
  `magic::describe`; names everywhere in their rarity's color, `ui::rarity_color`),
  `ui/equipment_screen.rs` (`EquipmentScreen`: slots by letter, which always name slots, so its
  own key `e` closes it only when it names none; `ItemScreen`: the stacks that fit one slot, by
  their inventory letters), `ui/character_screen.rs` (name, race, gender, class, age, birthday,
  and sign (`birth_parts`/`fit_parts`, shared with the summary), the player's scores, hit points
  and Defence total, weight and burden thresholds; 72 wide like the summary),
  `ui/weapon_skills_screen.rs` (an info page: sections, ranks, and a faint ten-cell marks bar,
  the fighting styles in use marked `•`; two columns from 88 wide, else one that drops its gaps
  and margins and then scrolls with ↑↓; never a benefit), `ui/chargen_screen.rs` (character creation: `ChoiceScreen` per
  step with a Random row rolled from the seed, `SummaryScreen`, `Draft`), `ui/quest_screen.rs`
  (the quest log from `PerceptionView::quests`: active then done, the selected one's objective,
  status, and the giver's offer; read-only), `ui/dialogue_screen.rs`
  (the speaker, their last line, numbered topics; built from the last `Event::Talked`, so it never
  shows more than the player was told), `ui/death_screen.rs`
  (who or what killed the player, where, when, after how many turns).

Key commands `,`/`g` (pick up: one stack at once, else the `PickUp` screen), `d` (`Drop` screen),
`i` (`Inventory` screen), `e` (`Equipment` screen), `@` (`Character` screen), `W` (`WeaponSkills`
screen), `p` (`KeyCommand::Pay`), and bash (`KeyCommand::Bash`: a direction prompt,
`InputMode::Direction`) are in
`input.rs` and `settings/keys.rs`. **Abilities** (racial, class, skill, style; the owner's rule,
#156) get no key of their own: they go into one abilities menu (to come). Each is a core
`Action` with replay text, an entry in `warlock_core::ability` (`Ability`, listed by
`PerceptionView::abilities` when usable), and a rebindable `KeyCommand` left unbound by default
(`settings/keys.rs::UNBOUND`). Shield bash is the first. A style's rank 10 abilities (later)
go there too.
