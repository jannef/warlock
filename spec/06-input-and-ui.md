# Warlock specification — 6. Input and UI

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

## 6. Input and UI

- Keyboard-first. Default sets: vi-keys (`hjklyubn`), numpad, arrows (4 directions).
- Symbol commands (`<`, `>`, `?`) are bound by **logical key**, so they work on non-US layouts.
- Bindings live in the settings file. The `?` help (the controls list, read-only) is generated from
  the binding table, so it is always accurate. Every key is rebindable (below).
- **Screens**: the main menu, New run and its character creation steps, pause menu, Settings,
  Controls, help, the inventory, the equipment screen and its choice of what goes in a slot, an
  item's description, the character screen, the Skills screen, the weapon skills screen, the quest
  log, choosing what to pick up, drop, throw, or eat, the dialogue screen, and the death screen
  are modal
  screens on one stack. Opening
  a screen puts it on top; Esc or Back closes it and returns to the screen below as it was left.
  While any screen is open it receives every key press, the run (if any) stays visible beneath
  it, and no game time passes but for what is chosen on a screen (picking up, dropping, wearing,
  wielding, eating), which is done as it is chosen and takes its time. Prompts in the message
  line (a direction to open, close, kick, or bash, to dig, or to chat, how long to rest, look
  mode, aiming a shot or a throw) are not screens.
- **The dialog**: every question the game asks first (quitting, a step onto a known trap, a
  shop's way out and its offers) is one dialog screen given its content: a title (on its
  frame), the question, and its options, each a label (`Pay 47 gold and leave`, `Stay`), a role
  (accept, decline, cancel, or plain), and what choosing it does (a menu choice, a game action,
  or a further dialog in its place). It sizes itself to its content and lists the options in a
  column, each with its shortcut keys. ↑/↓ (and ←/→) move the selection and Enter takes it;
  the Yes and No keys (`y` and `n` by default; rebindable, in a context of their own, as `y` and
  `n` move the player in play) pick the accept option and the decline option (else the cancel
  one); Esc picks the cancel option (else the decline one). The safe option (cancel, else
  decline) is selected at first, so Enter alone never does what the player might regret. A held
  key's repeats never open or answer it.
- **Known traps** (§8 "Traps"): a step onto a trap the player knows (where no creature they see
  stands) asks first, in the dialog, ADOM-like: `Really step onto the fireball trap?` with
  `Step onto it` (accept: the move as usual, replay text `move:DIR`: the rules never ask) and
  `Stay back` (cancel, selected first). Asking is the interface's: the rules step onto whatever
  they are told to.
- **Reading too close** (§8 "Scrolls"): a burst read from a scroll (fireball, frost) aimed where,
  as far as the player knows, it may catch them too (they stand within its radius of where it
  would go off: the first creature they see on the line of fire, the last cell before one they
  know is solid, or the cursor) asks first, in the dialog: `The fireball may catch you too. Read
  the scroll of fireball anyway?` with `Read it` (accept: the read as usual, replay text
  `read:LETTER:X,Y`), `Aim again` (decline: back to aiming, on the same target), and `Don't read
  it` (cancel, selected first). The rules never ask.
- **Main menu** (drawn in glyphs; the game opens on it unless `--scenario`, `--new`, `--replay`,
  `--record`, or `--capture` name what to run): Continue (only if a save exists), New run
  (scenario list from content, optional seed, then character creation), Settings, Controls,
  Quit.
- **Character creation** (§8 "Character creation"): after the scenario and seed, New run asks for
  a **race**, a **gender**, and a **class**, one screen each and in that order (race first: it
  is the biggest choice and sets the player's eyes), then shows a **summary**. Each step lists
  its options in data order with a `>` marker, the selected option's one-line description, and
  for races and classes their stat adjustments as written in data, dice included
  (`St 1d3+1  Dx −1`), and for races their vision; the gender step names the stat "other"
  raises with the run's seed. Every step ends with a
  **Random** row, rolled from the run seed and how often Random was pressed there. Up/Down (or
  `k`/`j`) choose, Enter goes on, Esc goes back a step as it was left. A step opens on the
  choice made there before going back, else on the scenario's character's, else on the first
  option. The summary lists Start the run, then Race, Gender, and Class with their choices (Enter
  on one returns to that step as it was left; the steps after it close), the character's age,
  birthday, and birth sign on one line (`Age 27 · born 3 Seedtide 1417 · year of the Fire
  Manticore`; the sign on a line of its own when the panel is too narrow), the six starting
  scores (the final scores only: the dice already rolled, no breakdown, never a potential or a
  modifier) with the starting trained skills and their values beside them (§8 "Skills"; in two
  columns when there are more than six, down the first, then the second; below the scores, as
  `Appraise 11 · Disable device 16 · …`, when the panel is too narrow), the vision, and the
  scenario and seed; "other" names the stat it raised. The
  player does not choose the age, the birthday, or the sign. Starting replaces any saved run. Content without races or classes skips
  character creation: New run starts the scenario as it is.
- **Pause menu**: Esc during a run (when no prompt is open) opens it, drawn in glyphs:
  Resume, Settings, Controls, Save and quit to menu, Quit (which saves). No game time passes while
  it is open; Esc or Back in Settings/Controls returns to it.
- **Settings** (`settings.toml` in the config dir, §7 "File locations"; on the web, browser `localStorage`): font size (logical
  px, 12 to 36), reduced effects and the per-effect switches, blood (§4.8), color palette, the
  temperature unit (`temperature = "celsius"` (the default) or `"fahrenheit"`: how the status bar
  shows the temperature, whole degrees, applied only when it is shown), key bindings, and a Files
  screen (where the files are and why; Open folder). A missing or damaged file means
  defaults and a logged warning. Changes apply live (a font size change rebuilds the glyph atlas)
  and are written immediately.
- **Every key is a command in its context** (#159): play (the map, the direction prompts, look
  mode), aiming (next and previous target, shoot or throw: `Tab`, none, Enter), the rest prompt
  (`h`, `d`, `n`), menus and screens (up, down, left, right: the arrows and `k` `j` `h` `l`;
  choose: Enter; back: no key of its own, as Esc always backs out; erase a character or reset a
  key: Delete, Backspace; on the pick-up and drop lists mark: Space, mark everything: `,`; on
  every item list the next and previous page: `>` and PageDown, `<` and PageUp), info pages (a
  digit, `Tab`; Shift+`Tab` the previous), dialogs (yes, no), and the developer keys (§9). A key is unique within its context only,
  so `Tab` is next target while aiming and next page on an info screen; a mode or screen with a
  context of its own reads it first (the rest prompt and aiming before play's keys). Two kinds of
  keys are fixed: Esc, which always cancels a prompt, backs out of every screen, and opens the
  pause menu, and the letters and digits that name the rows of a list (stacks, slots, topics).
  Hints (screen footers, the prompts, the status bar's `? controls`, the welcome line's
  `Press ? for controls.`) name the keys as bound.
- **Rebinding**: in Controls (the commands by context under headings, below notes on Esc and on
  list letters), select a command and press a key; it becomes the command's primary
  key (alternates such as the numpad stay). A key another command of the same context uses is taken
  from it and bound to the new command, and the screen says so ("K moved to Wait; Kick has no
  key." or "... (was Kick)."), so the table never holds a conflict; a command left without a key
  shows as none and is saved as an empty list. Fixed commands never lose their key: Esc (cancel,
  which opens the pause menu) is refused with a reason, and so is the last key of the commands
  that reach Controls from anywhere (up, down, and choose on the menus), so no binding, settings
  file, or keymap can lock the player out. Delete resets a command, "Reset all keys" resets
  everything, and so does `--reset-keys` on the command line (saved at once; the game then starts
  as usual).
- **Keymaps**: "Export keys…" in Controls writes every command's keys, by context, with a format
  version (`version = 1`), to `warlock-keys.toml` in the config folder (on the web, a download)
  and says where; "Import keys…" reads it back (on the web, from a file picker).
  `--export-keys FILE` and `--import-keys FILE` do the same from the command line, then exit. The
  file is TOML, to read and edit by hand. An import changes only the commands it lists, as
  rebinding does: a key another command of the context has moves to the listed one (said), a key
  two listed commands claim goes to the first in Controls' order, unknown commands and keys are
  skipped with a warning, the lockout rules hold, and the change applies and is saved at once. A
  file without a version (command ids at the top level, or a settings file's `[bindings]`) is
  read too, and a newer version as far as this one knows. The defaults ship as
  `keymaps/default.toml` beside the game.
- First-phase commands: move, wait, search (`s`, a turn; `z`, for a while: up to 10 turns,
  stopping when something is found or interrupts, like a rest; §8 "Searching"; basic commands,
  not abilities), open/close door, kick (`K`, then a direction; §8 "Doors"),
  ascend/descend, light/douse lantern (`L`),
  pick up (`,` or `g`), drop (`d`), inventory (`i`), equipment (`e`), character (`@`), skills
  (`S`), weapon skills (`W`), quest log (`q`), rest (`R`), eat (`E`), read (`r`; a basic
  command, not an ability), look, shoot or throw what
  is in the missile weapon slot
  (`t`), throw from the pack (`T`), fire (`f`, unbound by default), more aggressive / more
  defensive combat stance (`+` / `-`; §8 "Stances"), bash with a shield (an ability, below: no
  key by default; bound in Controls, it asks for a direction, like opening a door), next target
  (`Tab`, while aiming), use a tool (`U`, then a direction: dig), chat (`C`), pay (`p`, in a
  shop; §7 "Shops"), help, save and quit
  (rebindable, unbound by default: quitting is done from the pause menu). Every way to quit (the
  pause menu's Save and quit to menu and Quit, the main menu's Quit, a bound quick key) asks
  first, in the dialog (`Save the run and quit Warlock?`: `Save and quit`, `Stay`; closing the
  window or the tab does not ask). On the web Quit saves and reloads the page to a fresh main
  menu (a page cannot close itself), which has no Quit item. Free actions (look, help, the
  inventory, looking at the equipment screen, the character screen, the Skills screen, the
  weapon skills screen, the quest log, asking about a topic, menus, vision switch, a change of
  stance, paying in a shop) do not
  advance time.
- **Objects** (M11) lie in piles on cells, the top first: the map shows the top object's glyph
  (§4.2), and dropped objects go on top. Stepping onto a pile says what lies there: `You see here
  a dagger.` for one stack, `There are several objects here.` for more. Look mode names what lies
  on a cell, seen or remembered: `a dagger`, `a dagger and 12 arrows`, up to three stacks, then a
  count (`17 objects`). A stack is some number of one object: stackable objects (coins, arrows,
  potions) join a stack of the same object wherever they go; any other object is a stack of one.
  A door does not close on objects in its doorway.
- **Item lists** (#188, #189): every list of stacks the player chooses from or reads (the
  inventory, picking up, dropping, what to throw, eat, or read, what to read the scroll of
  identify on, what goes in an equipment slot; selling is dropping in a shop) is one paged list. Its stacks are grouped by **item class** (§7 "Objects
  and materials": melee weapons, missile weapons, missiles, shields, body armour, ... coins,
  miscellaneous), in class order, each class under a header row with its name and no letter;
  within a class they keep their order (the order they were picked up, or the pile's). A page
  shows as many rows as fit the screen (where the screen is what makes more pages, the panel's
  margin and gaps give way first) and at most 26 stacks, lettered `a` down the page: letters
  restart at `a` on every page, so on page 2 `a` names that page's first stack, and a class
  split across pages repeats its header there. The frame says `Page 2/3`, and `>` or PageDown
  turns to the next page, `<` or PageUp to the previous (rebindable list commands; not Tab, which
  switches info pages); the `>` cursor moves across pages with the arrows, and the page follows
  it. There is no scrolling. Letters always name stacks on a list, never commands. Marks belong
  to stacks, not letters, so they stay as the pages turn. Actions name stacks by where they are
  stored (their position), never by a page's letter, and no message names a letter.
- **Pick up** (`,` or `g`) takes a lone stack at once. Where several lie, the list shows them
  (above) with glyph, name, and weight: a letter marks or unmarks its stack (`+` marked, `-`
  not), `,` marks everything (on every page), the arrows move the `>` cursor and Space marks its
  row, Enter picks up the marked stacks (the cursor's row if none is marked), Esc cancels. **Drop**
  (`d`) chooses from the inventory the same way. Picking up and dropping are one action each
  (replay text `pickup:ac`, `drop:b`, by position: `a` to `z` are the first 26, and any position
  may be written as a number, `drop:0,2,30`; `pickup` and `drop` alone take everything) and take their
  action's time (§8 "Time"; at the burden the player has as they begin, §8 "Burden"); with
  nothing there or nothing carried, they are refused and take none. Picking up is allowed
  however much the player carries, even into Overloaded. Dropping something worn or wielded
  takes it off first ("You take off a leather jerkin."), which takes its time too, and the stacks
  after a dropped one move up a place (worn things keep being worn). In a shop, picking up goods
  adds them to what the player owes, and dropping puts back what is unpaid and offers to buy the
  player's own things (§7 "Shops"); the inventory and drop screens mark what is unpaid
  ("(unpaid, 45 gold)"), the pick-up screen what is for sale.
- **Inventory** (`i`): what the player carries, ADOM-like: as many stacks as they can carry
  (weight is the only limit, §8 "Burden"), in the item list (above), each with its glyph, name,
  and weight, then what the player carries (with their burden when they have one,
  §8 "Burden": `Carrying 28.4 kg (burdened)`) and their weight (their own plus what they carry),
  and, dimmer, the weights at which their burden changes: `Burdened over 15 kg, strained over 30
  kg, overloaded over 45 kg` (from scores the player knows, never a modifier; lines wrap at part
  boundaries in a narrow window). What is worn or wielded is marked with its slot's mark: "(being
  worn)", "(wielded)", "(in off hand)", "(on left hand)", "(readied)", "(quivered)", "(in use)"
  (a tool), and a two-handed weapon "(wielded in both hands)"; the drop screen marks it the same
  way. Names are in their rarity's color (§8 "Magic items": magic a soft blue, rare a pale
  yellow; normal as ever; palettes apply as to all UI text); the glyph keeps its own. It takes no
  action: a stack's letter opens its **description** (below); `i` (when it is no stack's letter)
  or Esc closes it.
- **Item description** (#149): one carried stack as the player knows it, opened by its letter in
  the inventory or by a slot's upper-case letter (`A` to `M`) on the equipment screen, over the
  screen it came from: its glyph and name (in its rarity's color, with its mark if worn or
  wielded), what it is (`Magic long sword · steel · 1.8 kg`), its base stats in words
  (`magic::describe`: "Long blade, for the main hand", "Damage 1d8; criticals on 20–24, ×2
  damage", "Shoots up to 15 cells", "Needs Strength 14 to draw", "Quick to strike", "Armour +4",
  "Dexterity adds at most +4 to Defence while worn", "Breaks on a hit 30% of the time"), and for
  a magic or rare item either "Unidentified: what its magic does is unknown, though it works all
  the same." or its item level and each affix, "Prefix  Sharp: +2 damage", "Suffix  of Celerity:
  attacks faster" (affix magnitudes are item properties and shown; speed only in words; global
  attack effects say "with every attack"). Never the player's own to-hit or a modifier.
  Read-only; Esc, Back, or Enter closes it.
- **Equipment** (`e`; M16), as in ADOM: the slots, lettered `a` to `m` in this order — Head, Neck,
  Body, Cloak, Main hand, Off hand, Hands, Left ring, Right ring, Feet, Missile weapon, Missiles,
  Tool (data, `data/slots.toml`, §7, so more can be added; the Tool slot came with M19) — each with
  what is in it (glyph, name, weight) or `—`, then what the player carries and weighs. A two-handed
  weapon shows in both hands, "(both hands)". A slot's letter (or Enter on the `>` row; the arrows
  move it) opens the choice of what goes there: the inventory stacks that fit the slot (and what it
  holds, such as a two-handed weapon in the off hand), in the item list (above), marked as in the
  inventory; the stack already there is the `>` row. Choosing a stack (its letter, or Enter) puts it
  on in the slot, taking off first what is there (a swap) and moving it if it is in another slot;
  choosing what is already there takes it off. Rings fit either ring slot, a light weapon (`light_weapon`, §7: the
  dagger, the stiletto, the short sword) either hand (the off hand too: two weapons, §8 "Fighting styles"), a
  two-handed weapon takes both hands (and puts away what they hold), launchers and their ammunition go in the missile weapon
  and missiles slots, digging tools in the Tool slot. That is one action (replay text
  `equip:body:b`, `remove:body`): it is done at once and takes its time, the choice closes, and the
  equipment screen beneath shows the result, as in ADOM; Esc goes back to it, and Esc there returns
  to play (its own key too, when that is not a slot's letter). A slot's upper-case letter opens the
  description of what is in it (above). Names show in their rarity's color, as in the inventory.
  Each thing put on or taken off takes the time of its slot's action (§8 "Time": wielding,
  wearing, body armor, readying; taking off costs what putting on does), so a swap takes both, and
  a refused one (the wrong slot, nothing there) takes none. Messages say it: "You put on a leather
  jerkin.", "You wield a quarterstaff in both hands.", "You ready 24 arrows.", "You take off an
  iron helm.", "You put away a short bow." Equipped things stay in the inventory and still count
  toward the player's weight and burden (§8 "Burden"). What is worn and wielded fights and protects
  (§8 "Combat": the wielded weapon's blows, the readied missile weapon's shots, armour, shield, and
  parry in Defence; a weapon's `attack_cost` how long a blow takes); being worn changes neither
  sight, noise, nor speed (its weight does, as part of the load), but for what a magic item's
  affixes do while it is worn or wielded (§8 "Magic items").
- **Character** (`@`): who the player is, as far as they know it: their name (the player
  creature's, "You"), their race, gender, and class (`Race    Dwarf`; "other" with the stat it
  raised), their age today, birthday, and birth sign as on the summary (when the run has a
  character), their score in each stat (§8 "Stats") in display order with its abbreviation and
  full name (`St  Strength  12`; as it is now, with what magic items they wear add; a score a
  status effect or stat damage lowers shows lowered, in a muted red; §8 "Status effects",
  "Diseases"), their level and experience (`Lv  Level  3`, `XP  Experience
  85 / 120`, the total alone at the highest level; §8 "Experience"), their hit points (`HP  Hit
  points  18/24`) and Defence
  (`Dv  Defence  15`; the total only, as its Dexterity part would give away a modifier; §8
  "Combat"), their combat stance by name (`Stance  Neutral`; §8 "Stances"), the status effects
  they suffer, by name, when there are any (`Status  Bleeding, Poisoned`; never how severe or for
  how long), the diseases they know they have, by name, on a row of their own below (`Diseased
  Filth fever, red ache`; one that has not progressed yet is only "Diseased"; never a timer or a
  check; §8 "Diseases"), their weight (their own plus what they carry, and their burden when they have one:
  `Weight 98.4 kg (you 70 kg, carrying 28.4 kg, burdened)`), and, dimmer, the weights at which
  their burden changes, as in the inventory (§8 "Burden"). The panel is 72 wide, so the birth and
  the thresholds take a line each. Never a modifier, a potential, or a to-hit, which the player
  does not know. Read-only; `@`, Enter, or Esc closes it, and the Skills key (`S`) opens the
  Skills screen over it. The status bar does not show stats: it is full at common window sizes.
- **Skills** (`S`; §8 "Skills"): the player's trained skills, in skill order (data order), each
  with its name, its value, and, dimmer, a four-cell meter of how near its next point is in
  quarters (`━` for each quarter reached, `─` for the rest: `━━──`; shape, not color, tells them
  apart); untrained skills are not there ("You are not trained in any skill." when none is).
  ↑↓ choose a skill, whose one-line description shows below the list (a list taller than the
  panel scrolls). Never a benefit's number, a mark, or a chance. Read-only; `S`, Enter, or Esc
  closes it (back to the character screen when opened from there).
- **Abilities** (the owner's rule; #156): racial, class, skill, and style abilities go into one
  abilities menu (to come), not a key each, unlike ADOM. Each is a core `Action` with replay
  text and a key command that is unbound by default (rebindable in Controls). The list the menu
  will read is `warlock_core::ability` (`Ability`: name, what it asks for (`Aim`), the `Action`
  it makes) and `PerceptionView::abilities` (those the player can use now). The first is the
  shield bash (§8 "Weapon skills"), listed while the player holds a shield to bash with.
- **Weapon skills** (`W`; M22, §8 "Weapon skills", "Fighting styles"), ADOM-like: the player's
  weapon skills and fighting styles in four sections in data order — Melee (the `melee` and
  `unarmed` skills), Missile and thrown (`launcher`, `thrown`), Shields (`shield_block`,
  `shield_bash`), Fighting styles (`style`) — each with its name, its rank (0 to 10; untrained
  skills dimmer, so what the player knows stands out), and a faint bar of ten cells (`━` earned,
  `─` still to earn, muted colors) of their marks toward the next rank (none at rank 10). The
  styles the player fights in now (`PerceptionView::own_styles_in_use`: their hands', their
  readied missile weapon's, throwing's while something is ready to throw) are marked with an
  accent `•` before the name, and the Fighting styles heading then has the legend `• in use`.
  Where the console is wide enough (88 columns) the sections are in two columns, Melee and
  Missile and thrown on the left, Shields and Fighting styles on the right; else one. Never what
  a rank gives, a mark count, or a creature's ranks. Read-only; `W`, Enter, or Esc closes it; in
  a short window the gaps between sections, the margin, and the gap above the footer give way
  first (one column fits the default 1280×720 window), and a list still too long scrolls with
  ↑↓ (`↑`/`↓` on the frame where rows are hidden, `↑↓ scroll` in the footer). Without weapon
  skills in content it says "You know no weapon skills."
- **Quest log** (`q`; §8 "Quests"): the quests the player has taken up ("Active"), then those
  they have done ("Done"), by name, each in the order of `data/quests.toml`; offers not yet
  answered are not there ("You have taken up no quests." when there are none). ↑↓ choose one;
  below the list it shows what it asks ("Kill 3 giant rats in the Arena."), how far it has got
  ("2 / 3 killed.", "You carry 1 / 3.", "Not reached yet.", "You no longer carry the sealed
  letter.", or once its goal is met "Return to the old hermit." / "Take the sealed letter to the
  old hermit."; "Done." for a done one), and who gave it, in their words (the offer, wrapped, at
  most six lines). Read-only: quests are not abandoned (the owner's decision). `q`, Enter, or Esc
  closes it.
  The character screen, the inventory, the equipment screen, the Skills screen, the weapon
  skills screen, and the quest log are the numbered info pages 1 to 6: on one of them, a digit,
  Tab, or Shift+Tab replaces it with another, and the tab line on its bottom
  border names them, each after its key: by their names where the panel is wide enough
  (`1 Character · 2 Inventory · …`), else the current page by its name and the others by short
  names (`1 Char · 2 Inventory · 3 Equip · 4 Skills · 5 Weapons · 6 Quests`), else all by their
  short names, else the current page by its name and the others by their keys alone.
- **Death screen** (§8 "Combat"): when the player dies, the death screen opens over the run as it
  ended, in place of any other screen, and the run's save is deleted (permadeath): `Y O U   D I E
  D`, then who killed them ("Killed by a kobold", or "Killed by something you did not see" when
  they did not see it), the status effect that did (its `death` line: "Bled to death"; a
  disease kills by Diseased: "Died of disease"), or what, when neither did (§8 "Hunger":
  "Starved to death", "Choked to death on a food ration"; §8 "Traps": "Killed by a fireball
  trap"), where
  (the level's name), when (the date and time), and after how many turns, and the level they
  had reached ("You had reached level 3."). It takes every key press; Enter or Esc returns to
  the main menu (no Continue: nothing is
  left to continue). A run that has ended is never saved again, by any trigger.
- **Weight**: every object and every creature (the player included) has a weight in grams. An
  actor weighs its creature's base weight plus everything it carries; the rules know every
  actor's weight (what it carries decides its burden, §8 "Burden"), but the player is shown only
  their own.
  Creatures carry objects from their spawn (`carry`, §7), and wield and wear some of them
  (`wield`, `wear`); they drop everything when they die (§8 "Combat"), and their corpse on top
  (§8 "Hunger"), and nothing else makes them drop or change anything yet. Worn and wielded things weigh what they weigh.
  Weights read `450 g` below a kilogram and `3.5 kg` above.
- **Chat** (`C`; §8 "Dialogue"): with exactly one creature in view next to the player, it talks
  to that one at once; otherwise it asks for a direction in the message line ("Talk in which
  direction? (Esc to cancel)"), like opening a door. The answer is a message: `The gardener says:
  "Mind the beds. Flowers bruise if you so much as look at them."` A creature with topics also
  opens the **dialogue screen**: its name as the title, what it said (wrapped), and its topics
  numbered 1 to 9 with a `>` marker on the selected one; a topic's number, or Enter on the
  selected one (the arrows move it), asks about it, which takes no time: the screen closes, the
  answer is said, and the screen opens again with it, on the topic asked (also when nothing is
  left to ask: a quest handed in to someone without other topics). Esc (or `C`) leaves. Quest
  topics (§8 "Quests") come first.
  Chatting takes the time of `chat` (§7 "Action costs"); with no creature the player sees there,
  it is refused ("You see no one there to talk to.") and takes none.
- **Rest** (`R`) asks in the message line: `Rest: (h)our, until (d)awn, until (n)ightfall? (Esc to
  cancel)`. `h` rests an hour, `d` until 06:00, `n` until 18:00 (replay text `rest:1h`,
  `rest:dawn`, `rest:dusk`); any other command cancels. Resting is one action that passes turn
  after turn and stops early when a creature comes into view (not one who talks and fights no
  one, such as the showcase's people: the player sees who it is) or a message is logged (§8
  "Time"),
  such as an attack the player perceives (§8 "Combat"), a status effect beginning ("You feel
  sick."), a disease showing or worsening ("You feel feverish.", "Fever wracks you."; §8
  "Diseases"), a sound that is news (§5 "Hearing"), or rain starting or growing heavier (§5
  "Weather"), the player getting hungrier (into Hungry or worse) or fainting from hunger (§8
  "Hunger"), and when the player dies;
  a sound already known goes on without stopping it, and so does other news of the weather (the
  rain easing or stopping, clouds gathering or parting), a status effect ending ("The bleeding
  stops."), a disease leaving ("Your fever breaks."), a full stomach settling ("You no longer
  feel satiated."), and someone muttering (§8 "Dialogue"), which are logged all the same. Hit points come back twice as fast while resting
  (§8 "Healing").
  It is a gameplay input mode like the open/close/kick direction prompt, not a menu screen.
- **Eat** (`E`; §8 "Hunger") opens the choice of what to eat: the food lying on the player's
  cell first (marked "lying here"; the `>` cursor starts on it, as ADOM asks about the floor
  first), then the food in the inventory, in the item list (above); with nothing, "You have
  nothing to eat." A letter, or Enter on the `>` row, eats it (replay text by position: `eat:b`
  for inventory stack `b`, `eat:floor:a` for the top stack of the pile underfoot); Esc
  cancels. A
  meal is one action of several turns, stopped early like a rest; "You eat the food ration. You
  feel satiated." when it is done, "You stop eating." (or "A kobold comes into view; you stop
  eating.") when not, the food left as it was. What is no food is refused and takes no time
  ("You cannot eat a dagger.").
- **Read** (`r`; §8 "Scrolls") opens the choice of what to read: the scrolls in the inventory
  (what has something to read on it), in the item list; with nothing, "You have nothing to
  read." A letter, or Enter on the `>` row, chooses: a spell scroll is then aimed (Targeting,
  below), and the scroll of identify replaces the list with the unidentified stacks the player
  carries ("You carry nothing unidentified."), where a letter or Enter reads it on that stack
  (replay text `read:LETTER:LETTER`, the scroll then the stack). Esc cancels either list.
- **Dig** (`U`, "use a tool", ADOM's use-item key, an item's use rather than an ability; §8
  "Digging") asks in the message line `Dig in which direction? (Esc to cancel)`; a direction
  digs (replay text `dig:E`), any other command cancels. With no digging tool ready nothing is
  asked: the rules say so at once ("You have no digging tool ready."). Digging goes on turn
  after turn like a rest and stops for the same reasons.
- **Messages**: the last two show in the message line; the same text logged again in a row folds
  into one entry with a count (`You open the door. (×2)`). Heard sounds are logged only when they
  are news (§5 "Hearing"), so a patrol walking back and forth out of sight gives one message per
  pass, not one a turn.
- **Attacks** (§8 "Combat") are reported when the player perceives them: they see the attacker
  or the target, or are the target. Messages say what came of an attack and never carry a number:
  a hit ("The kobold hits you.", "The giant rat bites the kobold."; with a natural attack, its
  verb), a confirmed critical ("The kobold hits you hard!"), a miss, and why it missed when the
  player saw both ends ("The kobold misses you." for a plain miss; "You dodge the kobold's
  spear.", "The kobold's spear glances off your armour.", "You block the kobold's spear with your
  shield.", "You parry the kobold's spear.", "Your sword glances off the cave beetle's shell."),
  a shot ("The kobold archer shoots the giant rat."), and a death ("The giant rat dies.", "You
  die..."). Spells are announced ("The kobold shaman casts a spell at you."). An end the player
  does not see is
  "something" ("Something hits you.", "The kobold misses something."), and a miss is then just
  a miss. A creature's attack is reported when the turn is perceived, the player's own as it is
  made (by what they see as they strike). Attacks the player perceives nothing of leave no
  message (and no event). A weapon skill rank gained is said after the attack that earned it
  ("You feel more skilled with long blades.", "You feel more skilled at blocking with a
  shield."; §8 "Weapon skills"), without a number. A shield bash is a blow ("You bash the
  kobold.", "You miss the kobold.", "The kobold dodges your small shield."). Status effects (§8)
  are said as they begin and end for the player ("You are bleeding.", "The bleeding stops.", "You
  feel sick.", "The poison wears off."), right after the attack that caused them; a creature the
  player sees die of one dies in its words ("The giant rat bleeds to death."). The player's
  diseases speak for themselves (§8 "Diseases"): nothing is said as one is caught or incubates;
  its `onset` as it shows ("You feel feverish."), a symptom at each progression that goes badly
  ("Fever wracks you."), and its `recovered` as it leaves ("Your fever breaks."); Diseased itself
  says nothing. Moving into a
  hostile creature attacks it in melee ("You hit the giant rat.", "You miss something."; it
  takes a turn); into any other one swaps places with it ("You swap places with the deer.", "You
  swap places with something."; the player's step), or, if it will not make way, stops ("The
  sentry doesn't budge.", "The giant is too big to squeeze past.", "Something is in the way.";
  no time passes; §8 "Displacement").
- **Targeting** (ADOM-like; `ui/target.rs`): `t` aims what is in the Missile weapon slot: a launcher
  shoots its readied ammunition, a throwing weapon or a stack (rocks) is thrown one at a time,
  any other weapon is thrown whole (improvised: -4 to hit, `IMPROVISED_TO_HIT`, and
  `IMPROVISED_RANGE` cells, in `combat.rs`); with nothing there, or no ammunition, it is refused
  ("You have nothing ready to throw or shoot.", "You have no arrows ready."). `T` opens the choice
  of what to throw: the inventory's throwing weapons, in the item list (a letter or Enter
  chooses; "You have nothing to throw."). `f` (no default key) aims the readied launcher only.
  A spell scroll chosen to read (`r`, above) is aimed the same way, as far as its spell reaches
  (its `range`; a bolt's length); a bolt's line is blocked only by what is solid, as it runs
  through creatures. Aiming is an input mode like look mode: no time passes.
  The cursor starts on the last target if the player still sees it (a creature) or it is on this
  level (a cell), else on the nearest hostile creature in view, else on the player; `Tab` and
  `Shift`+`Tab` cycle through the creatures in view, nearest first (ties: the lower id); the
  movement keys move it anywhere on the level. The line of fire (the integer line the rules fly
  the missile along, §8) is drawn as faint, unlit dots from the player toward the cursor, over
  what the player perceives: it ends before the first cell they see or remember as solid (cells
  never seen let it pass) and at the edge of the range; a cursor beyond something solid or a
  creature in view in the way is blocked, one beyond the range out of range, and both show on a
  muted red cursor and in the message line ("Out of range.", "The way is blocked.", after the
  target's description, as look mode gives it). No chance to hit is ever shown. `Enter` (or the
  key that started it, or `t` for a shot or a throw) looses the missile or reads the scroll
  (replay text `fire:X,Y`, `throw:LETTER:X,Y`, `read:LETTER:X,Y`; a burst that may catch the
  player asks first, above) and remembers the target, so `t` then `Enter` shoots it again; `Esc`
  cancels. Out of range or at the player's own cell, the rules refuse it and no time passes.
- **Look mode** describes what the player perceives at the cursor: a creature and its terrain,
  with how it stands toward the player and how hurt it looks ("A giant rat (hostile, unhurt), on
  stone floor."; its type tells the player its faction's stance toward them, the only faction
  information presentation gets; health in words, never hit points: unhurt, lightly wounded
  (above two thirds), wounded (above a third), badly wounded (above a sixth), or nearly dead),
  the status effects that show on it ("A giant rat (hostile, wounded, bleeding), on stone
  floor."; a status's `seen` word: bleeding shows, poison does not; §8 "Status effects"),
  what the creature wields, wears, and has ready, which is in plain sight ("A kobold (hostile,
  wounded), wielding a spear, on stone floor.", "A kobold shaman (hostile, unhurt), wearing a
  bone amulet, ...", "A kobold archer (hostile, unhurt), with a short bow and 20 arrows ready,
  ..."; never what else it carries), terrain (a wall in sight the player has dug into: "a stone
  wall, partly dug"), remembered terrain, blood seen or remembered there ("on blood-stained stone
  floor", "A blood-smeared stone wall."; §8 "Blood"; not with the blood switch off), where
  stairs lead, a known trap as part of the ground ("A fireball trap, on stone floor.", "A kobold
  (hostile, unhurt), on a pit on stone floor.", "You remember a pit here, on stone floor."; §8
  "Traps"), the special room walkable ground lies in once the player has been inside it ("A
  stone floor in an echoing hall.", "You remember a stone floor here, in a sanctuary."; §7
  "Special rooms"), the price of goods for sale in a shop on a cell in sight ("A dagger (for
  sale, 45 gold), on plank floor."; §7 "Shops"), and a sound heard there ("You hear footsteps
  here.").
- Screen layout: map viewport, message log, status panel (level, the player's hit points as
  `HP 18/24` (never below 0), the status effects they suffer by name, the most important first
  (`Bleeding`, `Poisoned`, `Diseased`; data order; §8 "Status effects"; never which disease, §8
  "Diseases"), their hunger unless they are Not hungry (`Hungry`, `Weak`, `Fainting`, `Starving`,
  `Satiated`, `Bloated`; §8 "Hunger"), their burden when they have one (`Burdened`, `Strained`,
  `Overloaded`; §8 "Burden"), their level as `Lv 3` (§8 "Experience"; it goes after the burden
  and before the time and the date), region (on the overworld and in
  its local areas), time of day and date as `14:32 · 12 Rainmoon, Year 1444`, the moon's phase as a
  glyph and its name (`◑ First quarter`; with several moons each moon's name and glyph, `Moon ●
  Ilse ☽`), the weather under the open sky (`Clear`, `Overcast`, `Rain`, `Heavy rain`; §5
  "Weather"), the temperature where the player is, always (`4 °C`, or `39 °F` with the setting:
  whole degrees, rounded half away from zero; tinted a subtle cool blue below freezing and warm
  from 30 °C, the number saying it all; §5 "Temperature"), turn, position, light at the player,
  vision profile, lantern, seed). The phase glyph
  shows the lit part of the disc, lit on the right while waxing: `○` new, `☽` waxing crescent,
  `◑` first quarter and waxing gibbous, `●` full, `◐` waning gibbous and last quarter, `☾` waning
  crescent. When the window is narrow, segments give way in priority order — seed, position,
  lantern, vision, light, turn, then the moon's phase (first cut down to its glyph), the
  temperature, region, date (first cut down to day and month, `12 Rainmoon`), weather, time,
  burden, hunger, the status effects (the least important first), and hit points, the level name
  last — and
  segments that fit again in the room left take it back, most important first. Outdoors the
  weather outranks the date: it changes the light and how far one sees.
