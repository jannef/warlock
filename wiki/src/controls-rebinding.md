# Rebinding keys

You rebind the keys of play in the Controls screen: open the main menu or the pause menu, then
Controls. The keys of [menus and screens](controls-menus.md#menus-and-screens), [text fields](controls-menus.md#text-fields), the
question windows (`y` and `n`), the [info pages](the-screen.md#info-pages) (the digits and `Tab`),
the [rest prompt](resting.md#how-resting-works) (`h`, `d`, `n`), and [aiming](controls-prompts.md#aiming) (`Tab` and
`Enter`) are fixed. The hints on screen name them.

Controls lists the commands in groups, by where they are used: in play, while aiming, at
`--more--`, on menus and screens, in text fields, on the info pages, and (in builds that have
them) the developer keys. Outside play, each group holds only a "back" or "previous" command. It
has no key until you give it one, since `Esc` and `Shift` already do it, and it cannot take a key
that is fixed there. Last come the [ability](abilities.md#keys-of-their-own) keys, which have
no key until you give them one.

A command can have several keys: moving north is numpad `8` and Up. To change them, select the
command and press `Enter`. Its keys open in a short list under it, one per line, with **Add a
key** last. The first key is the main one, the key the hints on screen name.

- **Replace a key**: select it, press `Enter`, then press the new key.
- **Add a key**: select **Add a key**, press `Enter`, then press the new key.
- **Remove a key**: select it and press `Delete`. A command may be left with no key.
- `Esc` closes the list.

If another command in the same group already uses the key you press, the key moves to this
command and the screen says what moved. The old command keeps its other keys, or shows `(none)`
if that was its only one.

A key of play can be held with `Ctrl`, `Alt`, or `Super` (on a Mac `Ctrl`, `Option`, or `Cmd`; Controls calls `Option` `Alt`):
hold them while you press the new key, as in `Ctrl+r`. A modified key is its own key: `Ctrl+r`
never does what `r` does, and `r` does nothing while `Ctrl` is held. `Shift` counts with keys
that type no character, such as `Shift+Up`; a letter carries it already (`K` is not `k`). Keys
your system or browser keeps for itself, such as `Cmd+Q`, `Alt+F4`, or `Ctrl+W` in the browser,
cannot be bound, and the screen says why. The back commands of menus, screens, and prompts take
no modifiers.

The same key can mean different things in different places. `Tab` is the next target while
[aiming](controls-prompts.md#aiming) and the next page on an [info screen](the-screen.md#info-pages). `d`
[drops](objects.md#dropping) in play but [rests](resting.md#how-resting-works) until dawn in the
rest prompt.

You cannot lock yourself out. `Esc` always backs out of every screen and cancels every prompt, and
it cannot be taken. The keys of the menus, which you need to get back to Controls, and of text
fields, which you need to name a new character, are fixed. `Delete` on a command whose keys are closed
puts its default keys back, and **Reset all keys** resets every command. If your keys still get in the way, start the game with
`warlock --reset-keys`: every key goes back to its default, and the game starts as usual. Your
keys are kept with your [settings](settings.md).

## Sharing your keys

**Export keys…** in Controls writes all your keys to a file, `warlock-keys.toml`, in the folder
with your settings (Settings, then Files, shows where). In the browser it downloads the file.
**Import keys…** reads it back; in the browser, you choose the file. Use it to copy your keys to
another computer, to back them up, or to share them.

It is a plain text file you can edit. It lists each command by name with its keys, in the same
groups as the Controls screen, as in `descend = [">"]`. An import changes only the commands the
file lists and moves keys the way rebinding does. It skips anything it does not know, says so,
and is saved at once. From the command line, `warlock --export-keys FILE` and
`warlock --import-keys FILE` do the same. The desktop downloads come with the default keys in such
a file, `keymaps/default.toml`.

See also: [Controls](controls.md), [Settings](settings.md), and
[Menus and lists](controls-menus.md).
