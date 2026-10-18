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
| Character (your stats and weight) | `@` |
| Rest | `R` |
| Look around | `x` |
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

- what you see there: a creature, marked "(hostile)" if it is hostile to you (see
  [Creatures](creatures.md)), the objects lying there, and the terrain, as in "A giant rat
  (hostile), on stone floor.";
- what you remember there, if you cannot see it now: the terrain and the objects you saw on it;
- for stairs and the places on the overworld, where they lead;
- a sound you heard there this turn ([Hearing](hearing.md));
- or "You don't know what is there."

Looking takes no time. Press `x` or `Esc` to leave.

## Objects

`,` or `g` picks up what lies where you stand: a single stack at once, or, where several lie, from
a screen listing them by letter. `d` opens the same kind of screen for what you carry, to drop.
On these screens a letter marks or unmarks a stack (even `h`, `j`, `k`, `l`), `,` marks
everything, the arrows and Space work too, `Enter` confirms, and `Esc` cancels. `i` shows your
pack and what it weighs; it takes no time. See [Objects and your pack](objects.md).

## The screen

The map fills most of the window. Above it, the **message line** shows the last two messages;
those from earlier turns are dimmed, and a message repeated in a row is shown once with a count,
as in `You open the door. (×2)`. Prompts and look mode use the same line.

Below the map, the **status bar** shows, from left to right:

- the name of the level, and on the overworld and in its local areas the region you are in;
- the time, the date, and the phase of the moon (see [Time and the calendar](time.md));
- the weather, while you are under the open sky (see [Weather](weather.md));
- the turn count and your position (column, row);
- `Light`: how brightly your own cell is lit, where 255 is full light (see [Light](light.md));
- your vision type, whether your lantern is lit, and the run's seed.

At its right end, `? controls` reminds you where the keys are.

When the window is too narrow for everything, the status bar leaves out the least important parts
first: the seed, your position, your lantern, your vision, the light at your feet, and the turn
count; then it shortens the moon's phase to its picture, and only then drops it; then the region,
then the year of the date, then the date. The weather, the time, and the name of the level stay
longest. Parts come back as soon as there is room.

## Character

`@` shows your name, your [stats](stats.md), and your weight. It takes no time; press `@` or `Esc`
to close it.

## Menus

`Esc` during play (when no prompt is open) opens the pause menu: Resume, Settings, Controls, Save
and quit to menu, and Quit, which saves too. The main menu, where the game opens, has Continue
(when there is a saved run), New run, Settings, Controls, and Quit; see [Saving](saving.md) and
[Settings](settings.md). While any menu or screen is open (including the inventory, the character
screen, and the pick-up and drop screens), no game time passes.

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
