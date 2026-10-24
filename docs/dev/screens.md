# Screens, input, and key commands

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/06-input-and-ui.md. Keep it current with the code, as CLAUDE.md says.

Modal screens (spec §6) — main menu, the saved characters (Characters), new run and character
creation (race, gender, class, name, summary), pause, settings, controls, the `?` help, the inventory (`i`), the equipment screen
(`e`) and its choice of what goes in a slot, an item's description (a letter in the inventory,
an upper-case slot letter on the equipment screen), the character screen (`@`), the weapon skills
screen (`W`), the quest log (`q`), the message history (`M`), choosing what to
pick up (`,`/`g` on several stacks), drop (`d`), eat (`E`), or read (`r`; then, for the scroll of
identify, what to read it on, `ScreenName::Identify`, which takes the read list's place), the
abilities menu (`a`), the
dialogue screen (opened by
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
  (`ScreenStack::picks`), else the scenario's character (the name step: what was typed, else an
  empty field), and the summary's Start is `MenuChoice::NewRun { scenario, seed, character, name
  }` with the seed `ScenarioPick::seed` resolved (#256): New run's seed field exists only where
  `input::dev_keys` is true (debug builds, `--dev`; `Launch::dev`), empty meaning the scenario's;
  elsewhere `Screen::new` hands `NewRunScreen::new` a `new_run_screen::fresh_seed()` (the
  platform clock and entropy, the app's, never core's) and no text mentions it
  (`ScenarioPick::shows_seed` gates the summary's `, seed N`); the main menu and the Characters screen read the saved characters when they open
  (`Launch::characters`, from `SaveStore::characters`; no other screen lists the saves);
  `EquipItem` takes the slot selected on the
  equipment screen beneath (`ScreenStack::selected_slot`), and `Describe` the stack looked at
  on the inventory or equipment screen beneath (`ScreenStack::looked_item`; the first stack
  without one, as `--screen describe` opens it over the inventory).
- **Input**: `input::route_keys` is the one reader of `KeyboardInput`. Stack not empty: the press
  goes to the top screen as a `Press` whose `nav` comes from the `Context::Menu` bindings (Esc is
  always `Nav::Back`; see `docs/dev/settings.md` for contexts: a screen never matches a key code or
  a character, except list letters and digits and the typed seed) and whose `edit` comes from the
  `Context::Text` bindings (Esc is always `Edit::Back`), for the text field, which reads only
  `edit` and `text` (the key event's text, literal; else the character its key stands for), so
  every key not bound there types; held keys type and erase but never accept
  (`ScreenStack::handle` →
  `ScreenAction`: stay, close, open, a
  `MenuChoice`, or `Act(Action)`: close and apply a gameplay action, e.g. `pickup:ac`,
  `equip:body:b`; the screens beneath stay, and `screens::refresh`, after `SimSet`, shows the
  equipment screen the result, as in ADOM); otherwise
  it is a gameplay command in the current `InputMode`. Opens and closes apply at once, so the next
  press in the same frame goes to the new top; menu choices (start, resume, save-and-quit to menu,
  quit) are messages applied by `screens::apply_choice`, and the rest of that frame's presses are
  dropped. Held-key repeats never open, close, choose, or act.
- **Draw**: `ui::draw` is the one system that writes the UI console: when anything it shows
  changed, it clears the console, draws the gameplay UI if a run exists, then the top screen.
  Footers name the menu keys as bound (`ui::menu::Hints`: `↑↓` and Enter with the defaults).
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
  `save_to_menu_dialog`), the step onto a known trap (`input::trap_dialog`, spec §8 "Traps"), a burst
  read so close it may catch the player (`input::burst_dialog`, spec §8 "Scrolls"), and the shop (`input.rs`: `leaving_dialog`, `sale_dialog`; spec §7 "Shops"). **A new
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
- **Item lists** (`ui/paged_list.rs`, spec §6 "Item lists", #188, #189): **every lettered list
  of stacks is the one `PagedList`** (the inventory, pick up, drop, throw, eat, read, identify, the
  equip choice;
  sell is a drop in a shop, identify a dev action): never a list or scrolling of its own. The
  abilities menu is one too, grouped by `Source` instead of item class
  (`PagedList::grouped(items, name)`; `PagedList::new` groups by item class). Items
  group by item class in class order (`PerceptionView::item_class`), a header row per class
  (`Row::Header`, no letter), each class in the items' own order. A page holds as many rows as
  the console has for it (the screen's `fit`/`layout`, which sets `PagedList::set_room`;
  where room, not the 26 letters, is what pages the list, the screen's margin and gaps give way
  first) and at most 26 items; letters run down a page from `a` and restart on every page
  (`at_letter`), a class split across pages repeats its header, and the frame shows `Page 2/3`
  (`draw_label`). The list's cursor picks the page shown; Up/Down move it across pages, and the
  `Context::Menu` commands `NextListPage` (`>`, PageDown) and `PrevListPage` (`<`, PageUp),
  `Nav::NextPage`/`PrevPage`, turn the page (cursor to its first item; no wrapping). Drawing knows
  the console, presses do not: `route_keys` keeps `ScreenStack::console` current and the stack
  calls the screen's `fit` before each press, so a letter names what is drawn even before the
  first draw (`--keys`). Screens hold stack positions (`Item::position`), never letters, and
  send actions by position.
- Prompts in the message line (open/close/bash/chat direction, rest, look mode, aiming,
  `--more--`) are `InputMode`s, not screens; a question (the step onto a known trap,
  `input::trap_dialog`) is the dialog (above).
- **The message line and `--more--`** (`ui/more.rs`, spec §6 "Messages", #207): `Game::read` is
  the message log as it was when the player last acted (`Game::apply` takes it before applying;
  a new run starts with nothing read, so its welcome is news; `Game::from_world`, a resumed run,
  has read everything; a replay's last step is news, `play_replay`; a loaded snapshot calls
  `Game::mark_read`), and `Game::unread` lines it up with the log (which only grows at its end,
  folds repeats into its last entry, and drops its oldest beyond `messages::CAPACITY`) to give
  this turn's messages. `more::pages` runs them on in rows of the console's width less a margin
  either side, split at word boundaries, in pages of `MESSAGE_ROWS`, every page but the last
  leaving room for ` --more--`. `more::wait` (after `SimSet`, before `draw`) enters
  `InputMode::More(0)` when a new action (`Game::applied`, or a new run) needs more than one
  page, from play only (a dev flag's aiming or look mode keeps its mode), and follows a resize.
  In `More(page)` `Gameplay::press` reads `Context::More` first: `MoreNext` (Space, Enter)
  turns to `more::advance(page, count)` (play once the last page shows), `MoreSkip` (unbound)
  and Esc go to play; every other command, and every held key's repeat, does nothing. Screens
  keep their keys while it waits beneath them. `more::draw` draws the page waited at with
  `--more--` after it, else the end of this turn's messages with earlier ones, one a row, in the
  rows left. Presentation only: dumps and replays never see pages. The rules never ask: a `move` steps onto whatever is there. `C` with
  exactly one creature in view next to the player chats with it at once
  (`input::chat_partner`), else asks for a direction.
- Screens are plain state machines tested without a window: `ui/menu.rs` (`Press`, `Hints`,
  `Panel`, main and pause menu), `ui/new_run_screen.rs`, `ui/settings_screen.rs`,
  `ui/controls_screen.rs` (commands grouped by `Context` under headings, notes on Esc and list
  letters, Reset all keys, Export keys…, Import keys…; also the `?` help: the same list,
  read-only, the developer keys only where they work), `ui/object_screen.rs` (inventory, pick up,
  drop, throw, eat; in the choices letters always name stacks, even `h j k l`; the eat choice
  lists food lying here first ("lying here"), then the pack's; in the inventory a letter opens
  the stack's description, and `i` closes it only when it names none; marks belong to stacks
  (positions), so they stay as pages turn),
  `ui/describe_screen.rs` (`DescribeScreen`: one stack as the player knows it, from
  `magic::describe`; names everywhere in their rarity's color, `ui::rarity_color`),
  `ui/equipment_screen.rs` (`EquipmentScreen`: slots by letter, which always name slots, so its
  own key `e` closes it only when it names none; `ItemScreen`: the stacks that fit one slot), `ui/character_screen.rs` (name, race, gender, class, age, birthday,
  and sign (`birth_parts`/`fit_parts`, shared with the summary), the player's scores, hit points
  and Defence total, the alignment band (`PerceptionView::own_alignment`), weight and burden
  thresholds; 72 wide like the summary),
  `ui/weapon_skills_screen.rs` (an info page: sections, ranks, and a faint ten-cell marks bar,
  the fighting styles in use marked `•`; two columns from 88 wide, else one that drops its gaps
  and margins and then scrolls with ↑↓; never a benefit), `ui/chargen_screen.rs` (character creation: `ChoiceScreen` per
  step with a Random row rolled from the seed, `SummaryScreen` (with the starting kit,
  `kit_words`, and the band `Start::alignment` falls in; 84 wide, dropping its gaps on a short
  console), `Draft`), `ui/name_screen.rs`
  (`NameScreen`, the Name step and the game's text field: typed text literal while the font draws
  it (`render::atlas::draws`) and there is room; the `Context::Text` keys accept, erase, put in the
  next of the race's random names (never the one shown; not on a held key), and back out; an empty
  field shows, dimmed, the name it gives), `ui/characters_screen.rs` (`CharactersScreen`: the saved
  characters, the one played last first; Enter is `MenuChoice::Resume(id)`, the Erase key asks the
  dialog whose Delete is `MenuChoice::DeleteCharacter(id)`, after which the main menu and the list
  open afresh; `--screen characters`, with `--capture` over a read-only `--save-dir`),
  `ui/quest_screen.rs`
  (the quest log from `PerceptionView::quests`: active then done, the selected one's objective,
  status, and the giver's offer; read-only), `ui/messages_screen.rs` (`MessagesScreen`,
  `ScreenName::Messages`, `--screen messages`: the message history, every message of
  `PerceptionView::messages` wrapped with further lines indented, a dim first line on what the log
  keeps, scrolled in lines from the newest at the bottom by `Nav::Up`/`Down` and a panel's height
  by `Nav::PrevPage`/`NextPage`; no info page, and no `PagedList`, whose rows are lettered single
  lines to choose from; the stack calls its `fit` before a press), `ui/dialogue_screen.rs`
  (the speaker, their last line, numbered topics; built from the last `Event::Talked`, so it never
  shows more than the player was told), `ui/abilities_screen.rs` (`AbilitiesScreen`, below),
  `ui/death_screen.rs`
  (who died, "Here lies Brakka,", who or what killed them, where, when, after how many turns).

Key commands `,`/`g` (pick up: one stack at once, else the `PickUp` screen), `d` (`Drop` screen),
`i` (`Inventory` screen), `e` (`Equipment` screen), `@` (`Character` screen), `W` (`WeaponSkills`
screen), `M` (`KeyCommand::Messages`: the `Messages` screen), `r` (`Read` screen, `KeyCommand::Read`: a basic command, not an ability), `p`
(`KeyCommand::Pay`), `a` (`KeyCommand::Abilities`: the `Abilities` screen), `A`
(`KeyCommand::RepeatAbility`), bash (`KeyCommand::Bash`, unbound), disarm
(`KeyCommand::Disarm`, unbound), and picking a lock (`KeyCommand::PickLock`, unbound) are in
`input.rs` and `settings/keys.rs`.

**Abilities** (spec §6 "Abilities"; racial, class, skill, weapon skill and style, item; the
owner's rule, #156) get no key of their own: they go into the one abilities menu.
- **Registry**: `warlock_core::ability` is the one place they are registered. A new ability is
  a variant of `Ability` (id, name, description, `Source`, `Aim`, cost `ActionKind`, the `Action`
  it makes from its aim), a line in `Ability::every` (menu order: by source, then as listed),
  who has it (`had_by`), and when it cannot be used (`unavailable` → `Unavailable` with its
  reason; `While(status)` for a status that blocks its cost's kind or is its cooldown,
  `docs/dev/status-effects.md`; one that asks for an alignment asks `ability::strayed` first,
  `Unavailable::Strayed`, `docs/dev/alignment.md`; `reason`/`id` take the content). `PerceptionView::abilities` is what presentation reads (`OwnAbility`: the ability and
  why it cannot be used now); the `abilities` dump layer and JSON key show the same, and the
  wiki's table (`xtask/src/wiki/abilities.rs`) lists `Ability::every`. Skills used as actions are
  `Ability::Skill(id)` until their use exists (never available); a use, when it comes, is a
  variant of its own, listed in its skill's place (`Ability::use_of`: the skill's benefit effect
  names it, so `had_by` asks `Skills::trains_effect`): `Ability::Disarm`, Disable device's
  (`docs/dev/traps.md`), and `Ability::PickLock`, Open lock's (`docs/dev/doors.md`). `Aim` has
  `Nothing` and `Direction`: a target (the aiming mode) and an item
  (a `PagedList` of the pack) join it with the first ability that needs one, as a variant and an
  arm in `Gameplay::use_ability`.
- **Menu**: `ui/abilities_screen.rs` (`AbilitiesScreen`, `ScreenName::Abilities`, `--screen
  abilities`): the `PagedList<Entry, Source>`, greyed rows with their reason, the selected
  ability's description below; the stack calls its `fit` before a press like the item lists. A
  letter or Enter on an available row answers `ScreenAction::UseAbility(ability)`; on an
  unavailable one it shows why and stays. Its own key closes it only when it names no row.
- **Dispatch**: `ScreenAction::UseAbility` is the one way an ability is used, from the menu, its
  own key (`KeyCommand::Bash`, `KeyCommand::Disarm`, and `KeyCommand::PickLock` answer it from
  `Gameplay::press`), or the quick repeat:
  `input::route_keys` closes the top screen if any, remembers it (`ScreenStack::last_ability`,
  cleared with the stack when a run ends, like the creation picks), and calls
  `Gameplay::use_ability`: `Aim::Direction` enters `InputMode::Ability(ability)` (the message
  line's "In which direction? (Esc to cancel)", disarming's "Disarm in which direction?",
  `ui/mod.rs`; a direction applies `ability.action(Some(dir))`),
  `Aim::Nothing` applies it at once, and one that cannot be used now is applied at once with a
  placeholder direction so the rules say why and take no time (as `U` does with no tool).
- **Quick repeat**: the menu opens on `ScreenStack::last_ability` (else the first available row);
  `A` (`RepeatAbility`, passed the last ability by `route_keys`) answers `UseAbility(last)`, or
  opens the menu when there is none.
- **Keys**: an ability with an action of its own may have a rebindable `KeyCommand`, left unbound
  by default (`settings/keys.rs::UNBOUND`); never a default key. Shield bash is the first; a
  style's rank 10 abilities (later) go there too.
