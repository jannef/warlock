# Controls

The game is played with the keyboard. These are the default keys; every one of them can be
rebound (see [Rebinding keys](#rebinding-keys)), and the hints on screen always name your keys.
The in-game `?` screen is authoritative: it is generated from your current key bindings.

| Command | Keys |
|---|---|
| Move (numpad) | `8` `9` `6` `3` `2` `1` `4` `7` |
| Move (arrows) | Up, Right, Down, Left (four directions) |
| Attack | move into a [hostile](creatures.md#factions) [creature](creatures.md) |
| [Swap places](npcs.md#getting-past) | move into any other creature |
| [Chat](npcs.md#chatting) with someone next to you | `C` (then a direction, if several are near) |
| Wait a [turn](time.md#what-takes-time) | `.` or numpad `5` |
| [Search](searching.md#searching-1) the cells around you (a [turn](time.md#what-takes-time)) | `s` (see [Searching](searching.md)) |
| [Search](searching.md#searching-1) for a while (up to 10 [turns](time.md#what-takes-time), stops when something turns up) | `S` |
| Open a [door](doors.md) | `o`, then a direction (or walk into the door) |
| Close a [door](doors.md) | `c`, then a direction |
| [Kick](doors.md#kicking) (a [door](doors.md), to break it in, or a [creature](creatures.md)) | `k`, then a direction (see [Doors](doors.md)) |
| Go down [stairs](terrain.md#stairs), or enter a place on the [overworld](overworld.md#locations) | `>` |
| Go up [stairs](terrain.md#stairs), or [leave an outdoor area](overworld.md#leaving-an-area) when no enemy is about | `<` |
| [Pick up](objects.md#picking-up) | `,` or `;` |
| [Drop](objects.md#dropping) | `d` |
| [Pay](shops.md#buying) the [shopkeeper](creatures.md#creature-shopkeeper) | `p` |
| [Equipment](equipment.md) ([wear and wield](equipment.md#what-fits-where)) | `i` |
| [Inventory](objects.md#your-pack) (your pack) | `I` |
| [Character](the-screen.md#character) (your [stats](stats.md) and weight) | `@` |
| [Skills](the-screen.md#skills) | `A` |
| [Weapon skills](weapon-skills.md) | `W` |
| [Quest log](quests.md#the-quest-log) | `q` |
| [Talents](talents.md) (yours, and every other with what it needs) | `G` |
| [Message history](the-screen.md#messages) (see [Messages](the-screen.md#messages)) | `v` |
| [Info page](the-screen.md#info-pages) 1 to 9 (while an info screen is open) | `1` to `9` |
| Next / previous [info page](the-screen.md#info-pages) (wraps) | `Tab` / `Shift`+`Tab` |
| [Rest](resting.md#how-resting-works) | `b` |
| Consume ([eat](food.md#eating)) | `e` |
| Read a [scroll](scrolls.md) | `r`, choose it, then aim a [spell](magic.md) (see [Scrolls](scrolls.md)) |
| [Look around](#look-mode) | `l` |
| [Shoot or throw](ranged.md#shooting-and-throwing) what is in your [Missile weapon slot](ranged.md#getting-ready) | `t`, then [aim](#aiming) (see below) |
| [Throw](ranged.md#shooting-and-throwing) something from your [pack](objects.md#your-pack) | `T`, choose it, then [aim](#aiming) |
| More aggressive combat [stance](combat.md#stances), more defensive (no time passes) | `+`, `-` |
| [Abilities](abilities.md#the-abilities-menu): everything you can do because of who you are, in one menu | `a` (see [Abilities](abilities.md)) |
| Use the [ability](abilities.md) you used last again | `X` |
| [Bash](abilities.md#ability-shield-bash) with your shield (an [ability](abilities.md): also in the [abilities menu](abilities.md#the-abilities-menu)) | no key until you bind one, then a direction |
| [Throw your voice](hearing.md#throwing-your-voice) (an [ability](abilities.md): also in the [abilities menu](abilities.md#the-abilities-menu)) | no key until you bind one, then aim |
| [Disarm a trap](traps.md#disarming-traps) (an [ability](abilities.md): also in the [abilities menu](abilities.md#the-abilities-menu)) | no key until you bind one, then a direction |
| [Steal](abilities.md#ability-steal) (an [ability](abilities.md): also in the [abilities menu](abilities.md#the-abilities-menu)) | no key until you bind one, then a direction |
| Next target while [aiming](#aiming) (with Shift: the one before) | `Tab` |
| Use a tool: light, hood, or douse a [lantern](lantern.md) ready in the Tool [slot](equipment.md#what-fits-where); else [dig](digging.md) into a wall with a pick-axe | `u` (to dig, then a direction) |
| Show or hide the controls | `?` |
| Show the next [messages](the-screen.md#messages) at `--more--` / skip the rest | `Space` or `Enter` / `Esc` |
| Yes / no, when a question asks | `y` / `n` |
| [Pause menu](the-screen.md#menus) | `Esc` |

The default keys are [ADOM](https://www.adom.de/)'s wherever ADOM has the command, so ADOM
players find them where their fingers expect them. Movement is the numpad and the arrows, as in
ADOM (the arrows move in four directions only); if you prefer the vi keys (`hjklyubn`), bind them
in Controls (`k`, `l`, and `u` are Kick, Look around, and Use a tool, as in ADOM: give those
other keys first). Commands ADOM does not have, or has only behind a prefix or `Ctrl` (which the
game does not bind), have keys ADOM leaves free:
the [message history](the-screen.md#messages) `v`, [rest](resting.md#how-resting-works) `b`, the [talents](talents.md) `G`, and the quick repeat of an [ability](abilities.md) `X`. Search for a while is
search's key in capitals, `S` (ADOM's save and quit): save and quit from the pause menu (`Esc`),
or bind a quick key for it in Controls. Symbol keys such as
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
| Reset a key (Controls), delete a saved [character](saving.md#when-the-game-saves) (Characters) | `Delete` or `Backspace` |
| Mark a [stack](objects.md#piles), mark everything ([picking up](objects.md#picking-up), [dropping](objects.md#dropping)) | `Space`, `,` |
| Next / previous page of a list of items | `>` or `PageDown` / `<` or `PageUp` |

## Text fields

In a text field (your character's [name](character.md#name)), every key types what it types, but these:

| Command | Keys |
|---|---|
| Accept the text | `Enter` |
| Erase the last character | `Backspace` or `Delete` |
| A random name | `Tab` |
| Back | `Esc`, always |

On a list that names its rows by letter (your [pack](objects.md#your-pack), a [pile](objects.md#piles), the [equipment](equipment.md#what-fits-where) slots) or by number
(the [topics](npcs.md#chatting) someone offers), those letters and digits always pick rows, whatever
else they are bound to. A list of items is grouped by kind of item ([weapons](objects.md#class-melee-weapons), [armour](objects.md#class-body-armour), [food](objects.md#class-food), ...)
and shows a page at a time: the letters name the rows of the page you see, starting again at
`a` on every page.

## Rebinding keys

Keys of menus, lists, screens, and prompts are fixed; new ones are never rebindable. Where the Controls screen lists such keys, it is the current state, and a later update may fix them. You can rebind the keys of play in the Controls screen (main menu or pause menu, then
Controls). It lists the commands in groups, by where they are used: in play, while aiming, in the
[rest prompt](resting.md#how-resting-works), at `--more--`, on menus and screens, in text fields, on the [info pages](the-screen.md#info-pages), in the question windows, and (in builds
that have them) the developer keys; last come the [ability](abilities.md#keys-of-their-own) keys, unbound until you give one a key. Select a command, press `Enter`, then press the new key. It
becomes the command's main key, and its other keys (such as the numpad) stay. A key another
command of the same group already uses moves to the new command, and the screen says what moved;
the old command keeps its other keys, or shows `(none)` if that was its only one. The same key
can mean different things in different groups: `Tab` is the next target while [aiming](#aiming) and the next
page on an [info screen](the-screen.md#info-pages), and `d` [drops](objects.md#dropping) in play but [rests](resting.md#how-resting-works) until dawn in the rest prompt.

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

Opening, closing, and [kicking](doors.md#kicking) a [door](doors.md), [bashing](abilities.md#ability-shield-bash) with your shield, and [stealing](abilities.md#ability-steal) ask `In which
direction? (Esc to cancel)` on the [message line](the-screen.md#messages), [digging](digging.md) asks `Dig in which direction? (Esc to cancel)` (see
[Digging and walls](digging.md)), [chatting](npcs.md#chatting) with several creatures near asks `Talk in which
direction? (Esc to cancel)` (see [People and talk](npcs.md)), [disarming a trap](traps.md#disarming-traps) asks `Disarm in
which direction? (Esc to cancel)` (see [Traps](traps.md#disarming-traps)), and
[resting](resting.md#how-resting-works) asks how long (see [Resting](resting.md)). Answer with a direction or the key shown, or
press `Esc` to cancel. Pressing any other command cancels the prompt. The [rest prompt's](resting.md#how-resting-works) keys (`h`,
`d`, `n`) are their own: they can be rebound without touching the keys of play (where `d`
drops), and the prompt names them as bound.

A few things ask first, in a small window over the map: quitting, stepping onto a trap you know
(see [Traps](traps.md)), leaving a [shop](shops.md) owing money, and dropping something a shop would buy
(see [Shops](shops.md)). The window lists its answers,
each with the keys that choose it: `y` picks the yes answer, `n` the no answer, and `Esc` the
one that backs out; or move with the arrow keys and press `Enter`. The safe answer is chosen at
first, so `Enter` alone never does what you might regret. The yes and no keys are their own: you
can rebind them without touching any key of play.

## Look mode

`l` puts a cursor on the map, on you. Move it with the movement keys, and the message line
describes what you know about the cell under it:

- what you see there: a [creature](creatures.md), marked "hostile" if it is [hostile](creatures.md#factions) to you (see
  [Creatures](creatures.md)), with how hurt it looks (unhurt, lightly wounded, wounded, badly
  wounded, or nearly dead; see [Combat](combat.md)) and whether it [bleeds](status-effects.md#bleeding) (see
  [Status effects](status-effects.md)), what it [wields and wears](equipment.md#what-fits-where), the [objects](objects.md) lying there, and
  the [terrain](terrain.md), as in "A giant rat (hostile, unhurt), on stone floor." or "A kobold (hostile,
  badly wounded, bleeding), wielding a spear, on stone floor."; a wall you have started digging
  into is "partly dug";
- a [trap](traps.md#finding-traps) you know there, as part of the ground: "A pit, on stone floor." (see [Traps](traps.md));
- what you remember there, if you cannot see it now: the terrain and the objects you saw on it;
- for [stairs](terrain.md#stairs) and the places on the [overworld](overworld.md#locations), where they lead;
- the [special room](special-rooms.md) the ground lies in, once you have been inside it, as in
  "A stone floor in an echoing hall.";
- a [sound](hearing.md) you heard there this turn ([Hearing](hearing.md));
- or "You don't know what is there."

Below it, on a [creature](creatures.md) whose level your [Knowledge](skills.md#skill-knowledge) stands above, is what you
know of its kind: "You know the skeleton is immune to poison, resistant to piercing, and
vulnerable to bludgeoning." (see [Skills](skills.md#skill-knowledge)).

Looking takes no time. Press `l` or `Esc` to leave.

## Aiming

`t` shoots or throws whatever is in your Missile weapon slot: a [bow, a crossbow, or a sling](ranged.md#getting-ready)
shoots its [ammunition](ammunition.md) from the Missiles slot (ready both with `i`); a [throwing weapon](ranged.md#shooting-and-throwing) (a dagger,
a spear, rocks) or any other weapon is thrown. `T` instead lists what you carry that can be
thrown, to choose one. (There is no separate key for firing a launcher: `t` does it.) Either puts a cursor on the map:

- it starts on your last target, if you still see it (or on the same spot), else on the nearest
  [hostile](creatures.md#factions) creature you see, else on you;
- `Tab` moves it to the next creature you see, nearest first, `Shift`+`Tab` back; the movement
  keys move it anywhere, also into the dark;
- a faint dotted line shows the way the missile would fly, as far as you know the ground: it
  ends before anything solid you know of and at the edge of the range. A target beyond it is
  marked on the cursor and in the message line: "Out of range." or "The way is blocked." (a
  wall, or a creature in the way, which the missile would reach first);
- the [message line](the-screen.md#messages) describes the target as [look mode](#look-mode) does. It never shows a chance to hit;
- `Enter` (or `t` again) shoots or throws, `Esc` cancels. Doing it again later starts on the same
  target, so `t` `Enter` shoots it again. The aiming keys (next target, previous target, which has
  no key until you bind one, and shoot or throw) are their own group in Controls.

A spell scroll you read (`r`, see [Scrolls](scrolls.md)) is aimed the same way: `Enter` or `r`
reads it at the cursor. A [lightning bolt](magic.md#spell-lightning-bolt) runs through creatures, so only walls block its line;
a [fireball](magic.md#spell-fireball) read so close that it may catch you asks first.

[Throwing your voice](hearing.md#throwing-your-voice) (an [ability](abilities.md)) is aimed the same way too, but nothing flies, so
there is no line: the cursor is marked, and the message line says "You do not see there.", on a
cell you do not see (or a wall), and "Out of range." beyond 6 cells. `Enter` throws it there.

See [Ranged combat](ranged.md).

## Objects

`,` or `;` picks up what lies where you stand: a single stack at once, or, where several lie, from
a list of them. `d` opens the same kind of list for what you carry, to drop. On these lists a
letter marks or unmarks a [stack](objects.md#piles) (even `h`, `j`, `k`, `l`), `,` marks everything (on every
page), the arrows and Space work too, `>` and `<` turn the pages of a long list (what you marked
stays marked), `Enter` confirms, and `Esc` cancels. `I` shows your
[pack](objects.md#your-pack), what it weighs, and how much more you can carry before your [burden](burden.md) grows; it
takes no time. See [Objects and your pack](objects.md).

`T` opens a list of what you carry that can be [thrown](ranged.md#shooting-and-throwing): choose one by its letter and aim it.

`e` (Consume) opens a list of what you can [eat](food.md#eating), titled "Consume what?": [food](food.md) lying where you stand first, then the food you carry.
A letter, or `Enter` on the marked row, eats it. See
[Food and corpses](food.md).

## Abilities

`a` opens the [abilities menu](abilities.md#the-abilities-menu): everything you can do because of who you are ([throwing your voice](hearing.md#throwing-your-voice),
a [shield bash](abilities.md#ability-shield-bash), [disarming a trap](traps.md#disarming-traps), the other [skills](skills.md) you use as actions, and
later racial and class abilities), grouped by where each comes from and lettered like a list of
items. Choose one by its letter, or `Enter` on the `>` row; one you cannot use now is greyed out
with why. It then asks for what it needs, such as a direction or a place to aim at. The menu opens
on the ability you used last, and `X` uses that one again at once.
Abilities have no keys of their own unless you give them one in Controls, where those few keys
are listed last, under "Ability keys". See [Abilities](abilities.md).

`i` shows what you [wear and wield](equipment.md#what-fits-where), slot by slot, as ADOM's `i` does (`I` is your pack). A slot's
letter (on this screen every letter names a slot) opens a list of what you carry that fits there: its letter puts it on, swapping
out what was there, and choosing what is there takes it off. Each change takes its time; you
stay on the equipment screen until you press `Esc`. See [Equipment](equipment.md).
