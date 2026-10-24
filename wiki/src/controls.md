# Controls

The game is played with the keyboard. These are the default keys; every one of them can be
rebound (see [Rebinding keys](#rebinding-keys)), and the hints on screen always name your keys.
The in-game `?` screen is authoritative: it is generated from your current key bindings.

| Command | Keys |
|---|---|
| Move north / northeast / east / southeast | `k` / `u` / `l` / `n` |
| Move south / southwest / west / northwest | `j` / `b` / `h` / `y` |
| Move (numpad) | `8` `9` `6` `3` `2` `1` `4` `7` |
| Move (arrows) | Up, Right, Down, Left (four directions) |
| Attack | move into a hostile creature |
| Swap places | move into any other creature |
| Chat with someone next to you | `C` (then a direction, if several are near) |
| Wait a turn | `.` or numpad `5` |
| Search the cells around you (a turn) | `s` (see [Searching](searching.md)) |
| Search for a while (up to 10 turns, stops when something turns up) | `z` |
| Open a door | `o`, then a direction (or walk into the door) |
| Close a door | `c`, then a direction |
| Kick (a door, to break it in, or a creature) | `K`, then a direction (see [Doors](doors.md)) |
| Go down stairs, or enter a place on the overworld | `>` |
| Go up stairs | `<` |
| Light or douse the lantern | `L` |
| Pick up | `,` or `g` |
| Drop | `d` |
| Pay the shopkeeper | `p` |
| Inventory | `i` |
| Equipment (wear and wield) | `e` |
| Character (your stats and weight) | `@` |
| Skills | `S` |
| Weapon skills | `W` |
| Quest log | `q` |
| Message history (see [Messages](#messages)) | `M` |
| Info page 1 to 9 (while an info screen is open) | `1` to `9` |
| Next / previous info page (wraps) | `Tab` / `Shift`+`Tab` |
| Rest | `R` |
| Eat | `E` |
| Read a scroll | `r`, choose it, then aim a spell (see [Scrolls](scrolls.md)) |
| Look around | `x` |
| Shoot or throw what is in your Missile weapon slot | `t`, then aim (see below) |
| Throw something from your pack | `T`, choose it, then aim |
| More aggressive combat stance, more defensive (no time passes) | `+`, `-` |
| Abilities: everything you can do because of who you are, in one menu | `a` (see [Abilities](abilities.md)) |
| Use the ability you used last again | `A` |
| Bash with your shield (an ability: also in the abilities menu) | no key until you bind one, then a direction |
| Throw your voice (an ability: also in the abilities menu) | no key until you bind one, then aim |
| Disarm a trap (an ability: also in the abilities menu) | no key until you bind one, then a direction |
| Steal (an ability: also in the abilities menu) | no key until you bind one, then a direction |
| Hood your lantern (an ability: also in the abilities menu) | no key until you bind one |
| Next target while aiming (with Shift: the one before) | `Tab` |
| Use a tool: dig into a wall with a pick-axe | `U`, then a direction |
| Show or hide the controls | `?` |
| Show the next messages at `--more--` / skip the rest | `Space` or `Enter` / `Esc` |
| Yes / no, when a question asks | `y` / `n` |
| Pause menu | `Esc` |
| Save and quit (unbound by default; use the pause menu) | none |

The movement keys are the vi keys (`hjklyubn`), the numpad, and the arrows. Symbol keys such as
`<`, `>`, and `?` are matched by the character they produce, so they work on other keyboard
layouts.

## Menus and screens

On every menu and screen:

| Command | Keys |
|---|---|
| Up / down | Up / Down, or `k` / `j` |
| Left / right (change a setting) | Left / Right, or `h` / `l` |
| Choose | `Enter` |
| Back | `Esc`, always |
| Reset a key (Controls), delete a saved character (Characters) | `Delete` or `Backspace` |
| Mark a stack, mark everything (picking up, dropping) | `Space`, `,` |
| Next / previous page of a list of items | `>` or `PageDown` / `<` or `PageUp` |

## Text fields

In a text field (your character's name), every key types what it types, but these:

| Command | Keys |
|---|---|
| Accept the text | `Enter` |
| Erase the last character | `Backspace` or `Delete` |
| A random name | `Tab` |
| Back | `Esc`, always |

On a list that names its rows by letter (your pack, a pile, the equipment slots) or by number
(the topics someone offers), those letters and digits always pick rows, whatever
else they are bound to. A list of items is grouped by kind of item (weapons, armour, food, ...)
and shows a page at a time: the letters name the rows of the page you see, starting again at
`a` on every page.

## Rebinding keys

You can rebind every key but `Esc` in the Controls screen (main menu or pause menu, then
Controls). It lists the commands in groups, by where they are used: in play, while aiming, in the
rest prompt, at `--more--`, on menus and screens, in text fields, on the info pages, in the question windows, and (in builds
that have them) the developer keys. Select a command, press `Enter`, then press the new key. It
becomes the command's main key, and its other keys (such as the numpad) stay. A key another
command of the same group already uses moves to the new command, and the screen says what moved;
the old command keeps its other keys, or shows `(none)` if that was its only one. The same key
can mean different things in different groups: `Tab` is the next target while aiming and the next
page on an info screen, and `y` and `n` move you in play but answer yes and no in a question.

You cannot lock yourself out: `Esc` always backs out of every screen and cancels every prompt,
and it cannot be taken; neither can the last key of up, down, or choose on the menus, which you
need to come back to Controls, nor the last key that accepts a text field, which you need to name
a new character. `Delete` resets the selected command to its default keys, and
**Reset all keys** resets every command. If your keys still get in the way, start the game with
`warlock --reset-keys`: every key goes back to its default, and the game starts as usual. Your keys
are kept with your [settings](settings.md).

### Sharing your keys

**Export keys…** in Controls writes all your keys to a file, `warlock-keys.toml`, in the folder
with your settings (Settings, then Files, shows where); in the browser version it downloads the
file. **Import keys…** reads it back (in the browser, choose the file). Copy it to another
computer or install, back it up, or share it. It is a plain text file you can edit: each command
by name with its keys, in groups like the Controls screen's, as in `descend = [">"]`. An import
changes only the commands the file lists, moves keys the way rebinding does, skips anything it does
not know (and says so), and is saved at once. From the command line, `warlock --export-keys FILE`
and `warlock --import-keys FILE` do the same. The desktop downloads come with the default keys as
such a file, `keymaps/default.toml`.

## Prompts

Opening, closing, and kicking a door, bashing with your shield, and stealing ask `In which
direction? (Esc to cancel)` on the message line, digging asks `Dig in which direction? (Esc to cancel)` (see
[Digging and walls](digging.md)), chatting with several creatures near asks `Talk in which
direction? (Esc to cancel)` (see [People and talk](npcs.md)), disarming a trap asks `Disarm in
which direction? (Esc to cancel)` (see [Traps](traps.md#disarming-traps)), and
resting asks how long (see [Resting](resting.md)). Answer with a direction or the key shown, or
press `Esc` to cancel. Pressing any other command cancels the prompt. The rest prompt's keys (`h`,
`d`, `n`) are their own: they can be rebound without touching movement, and the prompt names
them as bound.

A few things ask first, in a small window over the map: quitting, stepping onto a trap you know
(see [Traps](traps.md)), leaving a shop owing money, and dropping something a shop would buy
(see [Shops](shops.md)). The window lists its answers,
each with the keys that choose it: `y` picks the yes answer, `n` the no answer, and `Esc` the
one that backs out; or move with the arrow keys and press `Enter`. The safe answer is chosen at
first, so `Enter` alone never does what you might regret. The yes and no keys are their own: in
a question `y` and `n` do not move you, and you can rebind them without touching movement.

## Look mode

`x` puts a cursor on the map, on you. Move it with the movement keys, and the message line
describes what you know about the cell under it:

- what you see there: a creature, marked "hostile" if it is hostile to you (see
  [Creatures](creatures.md)), with how hurt it looks (unhurt, lightly wounded, wounded, badly
  wounded, or nearly dead; see [Combat](combat.md)) and whether it bleeds (see
  [Status effects](status-effects.md)), what it wields and wears, the objects lying there, and
  the terrain, as in "A giant rat (hostile, unhurt), on stone floor." or "A kobold (hostile,
  badly wounded, bleeding), wielding a spear, on stone floor."; a wall you have started digging
  into is "partly dug";
- a trap you know there, as part of the ground: "A pit, on stone floor." (see [Traps](traps.md));
- what you remember there, if you cannot see it now: the terrain and the objects you saw on it;
- for stairs and the places on the overworld, where they lead;
- the [special room](special-rooms.md) the ground lies in, once you have been inside it, as in
  "A stone floor in an echoing hall.";
- a sound you heard there this turn ([Hearing](hearing.md));
- or "You don't know what is there."

Looking takes no time. Press `x` or `Esc` to leave.

## Aiming

`t` shoots or throws whatever is in your Missile weapon slot: a bow, a crossbow, or a sling
shoots its ammunition from the Missiles slot (ready both with `e`); a throwing weapon (a dagger,
a spear, rocks) or any other weapon is thrown. `T` instead lists what you carry that can be
thrown, to choose one. (`f`, firing the missile weapon, has no key by default; bind it in the
controls screen.) Either puts a cursor on the map:

- it starts on your last target, if you still see it (or on the same spot), else on the nearest
  hostile creature you see, else on you;
- `Tab` moves it to the next creature you see, nearest first, `Shift`+`Tab` back; the movement
  keys move it anywhere, also into the dark;
- a faint dotted line shows the way the missile would fly, as far as you know the ground: it
  ends before anything solid you know of and at the edge of the range. A target beyond it is
  marked on the cursor and in the message line: "Out of range." or "The way is blocked." (a
  wall, or a creature in the way, which the missile would reach first);
- the message line describes the target as look mode does. It never shows a chance to hit;
- `Enter` (or `t` again) shoots or throws, `Esc` cancels. Doing it again later starts on the same
  target, so `t` `Enter` shoots it again. The aiming keys (next target, previous target, which has
  no key until you bind one, and shoot or throw) are their own group in Controls.

A spell scroll you read (`r`, see [Scrolls](scrolls.md)) is aimed the same way: `Enter` or `r`
reads it at the cursor. A lightning bolt runs through creatures, so only walls block its line;
a fireball read so close that it may catch you asks first.

Throwing your voice (an [ability](abilities.md)) is aimed the same way too, but nothing flies, so
there is no line: the cursor is marked, and the message line says "You do not see there.", on a
cell you do not see (or a wall), and "Out of range." beyond 6 cells. `Enter` throws it there.

See [Ranged combat](ranged.md).

## Objects

`,` or `g` picks up what lies where you stand: a single stack at once, or, where several lie, from
a list of them. `d` opens the same kind of list for what you carry, to drop. On these lists a
letter marks or unmarks a stack (even `h`, `j`, `k`, `l`), `,` marks everything (on every
page), the arrows and Space work too, `>` and `<` turn the pages of a long list (what you marked
stays marked), `Enter` confirms, and `Esc` cancels. `i` shows your
pack, what it weighs, and how much more you can carry before your [burden](burden.md) grows; it
takes no time. See [Objects and your pack](objects.md).

`T` opens a list of what you carry that can be thrown: choose one by its letter and aim it.

`E` opens a list of what you can eat: food lying where you stand first, then the food you carry.
A letter, or `Enter` on the marked row, eats it. See
[Food and corpses](food.md).

## Abilities

`a` opens the abilities menu: everything you can do because of who you are (throwing your voice,
a shield bash, disarming a trap, the other skills you use as actions, hooding your lantern, and
later racial and class abilities), grouped by where each comes from and lettered like a list of
items. Choose one by its letter, or `Enter` on the `>` row; one you cannot use now is greyed out
with why. It then asks for what it needs, such as a direction or a place to aim at. The menu opens
on the ability you used last, and `A` uses that one again at once.
Abilities have no keys of their own unless you give them one in Controls. See
[Abilities](abilities.md).

`e` shows what you wear and wield, slot by slot. A slot's letter (on this screen every letter
names a slot) opens a list of what you carry that fits there: its letter puts it on, swapping
out what was there, and choosing what is there takes it off. Each change takes its time; you
stay on the equipment screen until you press `Esc`. See [Equipment](equipment.md).

## The screen

The map fills most of the window. Above it, the **message line** (two rows) shows what this
turn brought (see [Messages](#messages)). Prompts and look mode use the same line.

Below the map, the **status bar** shows, from left to right:

- the name of the level, and on the overworld and in its local areas the region you are in;
- your hit points, as `HP 18/24`: what you have left of your most (see [Combat](combat.md));
- what ails you, if anything, the most urgent first: `Bleeding`, `Poisoned`, `Diseased` (see
  [Status effects](status-effects.md) and [Diseases](diseases.md));
- how hungry you are, unless you are not hungry: `Hungry`, `Weak`, `Fainting`, `Starving`,
  `Satiated`, or `Bloated` (see [Hunger](hunger.md));
- your burden, when you have one: `Burdened`, `Strained`, or `Overloaded` (see
  [Burden](burden.md));
- the time, the date, and the phase of the moon (see [Time and the calendar](time.md));
- the weather, while you are under the open sky (see [Weather](weather.md));
- the temperature where you are, as `4 °C` (or °F), cool blue below freezing and warm orange
  from 30 °C (see [Temperature and ice](temperature.md));
- the turn count and your position (column, row);
- `Light`: how brightly your own cell is lit, where 255 is full light (see [Light](light.md));
- your vision type, your lantern (`Lantern out`, `Lantern lit`, or `Lantern hooded`), and the
  run's seed.

At its right end, `? controls` reminds you where the keys are. It names the key as you bound it
(`Esc menu` when the controls have no key), and so does the welcome line a new run starts with.

When the window is too narrow for everything, the status bar leaves out the least important parts
first: the seed, your position, your lantern, your vision, the light at your feet, and the turn
count; then it shortens the moon's phase to its picture, and only then drops it; then the
temperature, the region, then the year of the date, then the date. The weather, the time, your
burden, your hunger, your status effects (the least urgent first), your hit points, and the name
of the level stay longest. Parts come back as soon as there is room.

## Messages

The message line shows the messages of this turn, everything said since you last acted, one
after another, wrapped at the ends of words. Rows they leave free show earlier messages, one a
row; those from before the last turn are dimmed. A message repeated in a row is shown once with
a count, as in `You open the door. (×2)`.

When a turn says more than the two rows hold, the line shows what fits followed by `--more--`
and waits: `Space` or `Enter` shows the rest, a page at a time, and `Esc` skips what is left of
this turn's messages. Until then no other key does anything (a key you hold down never turns
the page), so nothing happens that you have not read. A rest or a long dig counts as one turn
here: what was said during it, and what stopped it, shows the same way. Both keys can be rebound in their own group in Controls
(skipping has no key until you bind one; `Esc` always skips). A screen that opens by itself,
such as someone answering you, comes first: `--more--` waits beneath it until it closes.

`M` opens the **message history**: every message the game keeps (the last 200), the newest at
the bottom, where it opens. The up and down keys scroll it a line at a time, `<` and `>` (or
`PageUp` and `PageDown`) a screenful; `Esc`, `Enter`, or `M` closes it. It takes no time.

## Character

`@` shows your name, your race, gender, and class, your age, birthday, and
[birth sign](horoscope.md), your [stats](stats.md) (one that a status effect or a disease lowers
is shown lowered, in red), your hit points and Defence (`Dv`; see [Combat](combat.md)), what ails
you (`Status  Bleeding, Poisoned`; see [Status effects](status-effects.md)), the diseases you know
you have (`Diseased  Filth fever`; see [Diseases](diseases.md)), your weight and what you
carry, and the weights at which your [burden](burden.md) changes. It takes no time; press `@` or
`Esc` to close it, or `S` for your skills.

## Skills

`S` lists the [skills](skills.md) you are trained in, each with its value and a small meter of how
practised it is (how big its next roll at a level-up will be). The arrow keys pick a skill to read what it is about. It takes no
time; press `S` or `Esc` to close it.

## Info pages

The character screen, the inventory, the equipment screen, the skills screen, the weapon skills
screen, and the quest log are numbered pages of one set, shown on the bottom edge of each as
`1 Character · 2 Inventory · 3 Equipment · 4 Skills · 5 Weapon skills · 6 Quests`, the one you
are on highlighted. Where that does not fit, the others go by short names
(`1 Char · 2 Inventory · 3 Equip · 4 Skills · 5 Weapons · 6 Quests`), and on the narrowest
screens by their numbers alone. While one of them is open, `1` to `9` switch to the page with that number (the row
of number keys above the letters; the numpad still moves), and `Tab` / `Shift`+`Tab` go to the
next or previous page, wrapping around. The page replaces the one you were on, so `Esc` closes it
and takes you back to where you opened the first one, as if you had only ever opened that one.
The number keys do nothing during play, and they keep their own meaning on a screen that is a
step of a page, such as choosing what goes in a slot. Each page also keeps its own key (`@`,
`i`, `e`, `S`) to open it and to close it. All of these keys can be rebound in the Controls screen, as
"Info page 1" to "Info page 9", "Next info page", and "Previous info page"; they are separate
from the play keys, so `Tab` can be both "next target" while aiming and "next info page" on an
info screen.

## Death

When you die, a screen says what killed you (if you saw it), where, when, and after how many
turns. Your save is already gone (see [Saving](saving.md)); `Enter` or `Esc` returns to the main
menu.

## Menus

`Esc` during play (when no prompt is open) opens the pause menu: Resume, Settings, Controls, Save
and quit to menu, and Quit, which saves too. The main menu, where the game opens, has Continue
(when there is a saved run), New run, Settings, Controls, and Quit; see [Saving](saving.md) and
[Settings](settings.md). New run goes on to creating your character, a step at a time: the arrow
keys (or `j` and `k`) choose, `Enter` goes on, and `Esc` goes back a step (see
[Your character](character.md)). While any menu or screen is open (including the inventory, the
equipment screen, the character screen, the pick-up and drop screens, the abilities menu, and the
screen of topics someone you chat with offers), no game time passes, except for what you choose
there: picking up, dropping, putting on or taking off, and an ability take their time. Asking
about a topic takes none.

## Developer keys

Builds made for testing, and desktop builds started with `--dev`, add a few keys for trying
things out. They are not part of normal play, and they are not in the browser version. `?` lists
them, marked "(dev)", and Controls can rebind them there:

| Key | Does |
|---|---|
| `F2` | Reveal the map: draws everything on the level, even what you cannot see, in violet |
| `F3` | Switch to the next [vision type](vision.md) |
| `F4` | Show how brightly each cell you see is lit, as a digit from `0` (dark) to `9` (fully lit) |
| `F5` / `F9` | Save / load a snapshot of the run |
| `F6` | Skip an hour: the clock jumps ahead, and nobody acts meanwhile |
| `F7` | Set the next [weather](weather.md) for three hours, after which the usual weather returns |
| `F8` | Identify everything you carry: [magic items](magic-items.md) show what they are |
