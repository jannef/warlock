# Controls

The game is played with the keyboard. These are the default keys. The in-game `?` screen is
authoritative: it is generated from your current key bindings.

| Command | Keys |
|---|---|
| Move north / northeast / east / southeast | `k` / `u` / `l` / `n` |
| Move south / southwest / west / northwest | `j` / `b` / `h` / `y` |
| Move (numpad) | `8` `9` `6` `3` `2` `1` `4` `7` |
| Move (arrows) | Up, Right, Down, Left (four directions) |
| Attack | move into a hostile creature |
| Wait a turn | `.` or numpad `5` |
| Open a door | `o`, then a direction (or walk into the door) |
| Close a door | `c`, then a direction |
| Go down stairs, or enter a place on the overworld | `>` |
| Go up stairs | `<` |
| Light or douse the lantern | `L` |
| Pick up | `,` or `g` |
| Drop | `d` |
| Inventory | `i` |
| Equipment (wear and wield) | `e` |
| Character (your stats and weight) | `@` |
| Skills | `S` |
| Info page 1 to 9 (while an info screen is open) | `1` to `9` |
| Next / previous info page (wraps) | `Tab` or `PageDown` / `Shift`+`Tab` or `PageUp` |
| Rest | `R` |
| Look around | `x` |
| Fire your missile weapon | `f`, then aim (see below) |
| Throw something | `t`, then aim |
| Next target while aiming (with Shift: the one before) | `Tab` |
| Show or hide the controls | `?` |
| Pause menu | `Esc` |
| Save and quit | `Q` |

The movement keys are the vi keys (`hjklyubn`), the numpad, and the arrows. Symbol keys such as
`<`, `>`, and `?` are matched by the character they produce, so they work on other keyboard
layouts.

## Rebinding keys

You can rebind every command in the table above but `Esc` in the Controls screen (main menu or
pause menu, then Controls): select a command, press `Enter`, then press the new key. It becomes
the command's main key, and its other keys (such as the numpad) stay. A key another command
already uses is refused, and the screen tells you which command has it. `Delete` resets the
selected command to its default keys, and **Reset all keys** resets every command. Your keys are
kept with your [settings](settings.md).

## Prompts

Opening and closing a door ask `In which direction? (Esc to cancel)` on the message line, and
resting asks how long (see [Resting](resting.md)). Answer with a direction or the key shown, or
press `Esc` to cancel. Pressing any other command cancels the prompt.

## Look mode

`x` puts a cursor on the map, on you. Move it with the movement keys, and the message line
describes what you know about the cell under it:

- what you see there: a creature, marked "hostile" if it is hostile to you (see
  [Creatures](creatures.md)), with how hurt it looks (unhurt, lightly wounded, wounded, badly
  wounded, or nearly dead; see [Combat](combat.md)), what it wields and wears, the objects lying
  there, and the terrain, as in "A giant rat (hostile, unhurt), on stone floor." or "A kobold
  (hostile, badly wounded), wielding a spear, on stone floor.";
- what you remember there, if you cannot see it now: the terrain and the objects you saw on it;
- for stairs and the places on the overworld, where they lead;
- the [special room](special-rooms.md) the ground lies in, once you have been inside it, as in
  "A stone floor in an echoing hall.";
- a sound you heard there this turn ([Hearing](hearing.md));
- or "You don't know what is there."

Looking takes no time. Press `x` or `Esc` to leave.

## Aiming

`f` fires the missile weapon you have ready (a bow, a crossbow, a sling: ready it and its
ammunition with `e`), `t` throws something (a throwing weapon you have ready in the missiles slot,
or one you choose from a list of what you can throw). Both put a cursor on the map:

- it starts on your last target, if you still see it (or on the same spot), else on the nearest
  hostile creature you see, else on you;
- `Tab` moves it to the next creature you see, nearest first, `Shift`+`Tab` back; the movement
  keys move it anywhere, also into the dark;
- a faint dotted line shows the way the missile would fly, as far as you know the ground: it
  ends before anything solid you know of and at the edge of the range. A target beyond it is
  marked on the cursor and in the message line: "Out of range." or "The way is blocked." (a
  wall, or a creature in the way, which the missile would reach first);
- the message line describes the target as look mode does. It never shows a chance to hit;
- `Enter` (or `f` again, or `t` again when throwing) shoots, `Esc` cancels. Shooting again later
  starts on the same target, so `f` `Enter` shoots it again.

See [Ranged combat](ranged.md).

## Objects

`,` or `g` picks up what lies where you stand: a single stack at once, or, where several lie, from
a screen listing them by letter. `d` opens the same kind of screen for what you carry, to drop.
On these screens a letter marks or unmarks a stack (even `h`, `j`, `k`, `l`), `,` marks
everything, the arrows and Space work too, `Enter` confirms, and `Esc` cancels. `i` shows your
pack, what it weighs, and how much more you can carry before your [burden](burden.md) grows; it
takes no time. See [Objects and your pack](objects.md).

`t` with nothing throwable ready opens a list of what you carry that can be thrown, by its pack
letter: choose one and aim it.

`e` shows what you wear and wield, slot by slot. A slot's letter (on this screen every letter
names a slot) opens a list of what you carry that fits there: its letter puts it on, swapping
out what was there, and choosing what is there takes it off. Each change takes its time; you
stay on the equipment screen until you press `Esc`. See [Equipment](equipment.md).

## The screen

The map fills most of the window. Above it, the **message line** shows the last two messages;
those from earlier turns are dimmed, and a message repeated in a row is shown once with a count,
as in `You open the door. (×2)`. Prompts and look mode use the same line.

Below the map, the **status bar** shows, from left to right:

- the name of the level, and on the overworld and in its local areas the region you are in;
- your hit points, as `HP 18/24`: what you have left of your most (see [Combat](combat.md));
- your burden, when you have one: `Burdened`, `Strained`, or `Overloaded` (see
  [Burden](burden.md));
- the time, the date, and the phase of the moon (see [Time and the calendar](time.md));
- the weather, while you are under the open sky (see [Weather](weather.md));
- the turn count and your position (column, row);
- `Light`: how brightly your own cell is lit, where 255 is full light (see [Light](light.md));
- your vision type, whether your lantern is lit, and the run's seed.

At its right end, `? controls` reminds you where the keys are.

When the window is too narrow for everything, the status bar leaves out the least important parts
first: the seed, your position, your lantern, your vision, the light at your feet, and the turn
count; then it shortens the moon's phase to its picture, and only then drops it; then the region,
then the year of the date, then the date. The weather, the time, your burden, your hit points,
and the name of the level stay longest. Parts come back as soon as there is room.

## Character

`@` shows your name, your race, gender, and class, your age, birthday, and
[birth sign](horoscope.md), your [stats](stats.md), your hit points and Defence (`Dv`; see
[Combat](combat.md)), your weight and what you carry, and the weights at which your
[burden](burden.md) changes. It takes no time; press `@` or `Esc` to close it, or `S` for your
skills.

## Skills

`S` lists the [skills](skills.md) you are trained in, each with its value and a small meter of how
near it is to its next point. The arrow keys pick a skill to read what it is about. It takes no
time; press `S` or `Esc` to close it.

## Info pages

The character screen, the inventory, the equipment screen, and the skills screen are numbered pages of one set,
shown on the bottom edge of each as `1 Character · 2 Inventory · 3 Equipment · 4 Skills`, the one you are on
highlighted. While one of them is open, `1` to `9` switch to the page with that number (the row
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
equipment screen, the character screen, and the pick-up and drop screens), no game time passes,
except for what you choose there: picking up, dropping, and putting on or taking off take their
time.

## Developer keys

Builds made for testing, and desktop builds started with `--dev`, add a few keys for trying
things out. They are not part of normal play, and they are not in the browser version. `?` lists
them, marked "(dev)":

| Key | Does |
|---|---|
| `F2` | Reveal the map: draws everything on the level, even what you cannot see, in violet |
| `F3` | Switch to the next [vision type](vision.md) |
| `F4` | Show how brightly each cell you see is lit, as a digit from `0` (dark) to `9` (fully lit) |
| `F5` / `F9` | Save / load a snapshot of the run |
| `F6` | Skip an hour: the clock jumps ahead, and nobody acts meanwhile |
| `F7` | Set the next [weather](weather.md) for three hours, after which the usual weather returns |
| `F8` | Identify everything you carry: [magic items](magic-items.md) show what they are |
