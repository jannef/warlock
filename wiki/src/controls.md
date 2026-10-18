# Controls

The game is played with the keyboard. These are the default keys. The in-game `?` screen is
authoritative: it is generated from your current key bindings, and you can rebind every command in
the Controls screen (main menu or pause menu, then Controls): select a command and press the new
key. A key another command already uses is refused, and Delete resets a command.

| Command | Keys |
|---|---|
| Move north / northeast / east / southeast | `k` / `u` / `l` / `n` |
| Move south / southwest / west / northwest | `j` / `b` / `h` / `y` |
| Move (numpad) | `8` `9` `6` `3` `2` `1` `4` `7` |
| Move (arrows) | Up, Right, Down, Left (four directions) |
| Wait a turn | `.` or numpad `5` |
| Open a door | `o`, then a direction |
| Close a door | `c`, then a direction |
| Go down stairs | `>` |
| Go up stairs | `<` |
| Light or douse the lantern | `L` |
| Pick up | `,` or `g` |
| Drop | `d` |
| Inventory | `i` |
| Rest | `R` |
| Look around | `x` |
| Show or hide the controls | `?` |
| Pause menu | `Esc` |
| Save and quit | `Q` |

The movement keys are the vi keys (`hjklyubn`), the numpad, and the arrows. Symbol keys such as
`<`, `>`, and `?` are matched by the character they produce, so they work on other keyboard
layouts.

## Prompts

Open, close, and rest ask a question on the message line. Answer with the key shown, or press
`Esc` to cancel. Pressing any other command cancels the prompt.

## Look mode

`x` puts a cursor on the map. Move it with the movement keys to read what is there: what you see
(a creature, marked "(hostile)" if it is hostile to you, see [Creatures](creatures.md); the
objects lying there; the terrain), what you remember (terrain and objects), a
sound you heard there ([Hearing](hearing.md)), or that you do not know. Looking takes no time. Press `x` or `Esc` to leave.

## Objects

`,` or `g` picks up what lies where you stand: a single stack at once, or, where several lie, from
a screen listing them by letter. `d` opens the same kind of screen for what you carry, to drop.
On these screens a letter marks or unmarks a stack (even `h`, `j`, `k`, `l`), `,` marks
everything, the arrows and Space work too, `Enter` confirms, and `Esc` cancels. `i` shows your
pack and what it weighs; it takes no time. See [Objects and your pack](objects.md).

## Menus

`Esc` during play opens the pause menu: Resume, Settings, Controls, Save and quit to menu, and
Quit. While any menu or screen is open (including the inventory and the pick-up and drop
screens), no game time passes. Settings include font size, color palettes
(default, high contrast, and a red-green colorblind variant), reduced effects, and the key
bindings.
