# The screen

INTRO

## The screen

The map fills most of the window. Above it, the **[message line](#messages)** (two rows) shows what this
turn brought (see [Messages](#messages)). Prompts and [look mode](controls.md#look-mode) use the same line.

Below the map, the **status bar** shows, from left to right:

- the name of the level, and on the overworld and in its local areas the region you are in;
- your [hit points](hit-points.md#hit-points), as `HP 18/24`: what you have left of your most (see [Combat](combat.md));
- your [mana](mana.md#how-much), as `MP 5/12`, the same way (see [Mana](mana.md));
- each [resource](resources.md) you hold, as `Rage 7` or `Focus 7/10` (see [Resources](resources.md));
- what ails you, if anything, the most urgent first: `Bleeding`, `Poisoned`, `Diseased` (see
  [Status effects](status-effects.md) and [Diseases](diseases.md));
- how [hungry](hunger.md#how-hungry-you-are) you are, unless you are not hungry: `Hungry`, `Weak`, `Fainting`, `Starving`,
  `Satiated`, or `Bloated`, from `Hungry` down in a colour that grows more alarming as it worsens (see [Hunger](hunger.md#what-you-see));
- your [burden](burden.md#burden-1), when you have one: `Burdened`, `Strained`, or `Overloaded` (see
  [Burden](burden.md));
- the time, the date, and the [phase of the moon](sun.md#phases) (see [Time and the calendar](time.md));
- the [weather](weather.md#how-the-weather-changes), while you are under the open sky (see [Weather](weather.md));
- the [temperature](temperature.md) where you are, as `4 °C` (or °F), cool blue below freezing and warm orange
  from 30 °C (see [Temperature and ice](temperature.md));
- the [turn](time.md#what-takes-time) count and your position (column, row);
- `Light`: how brightly your own cell is [lit](light.md#what-light-is), where 255 is full light (see [Light](light.md));
- your [vision type](vision.md#the-four-types) and the run's seed.

At its right end, `? controls` reminds you where the keys are. It names the key as you bound it
(`Esc menu` when the controls have no key), and so does the welcome line a new run starts with.

When the window is too narrow for everything, the status bar leaves out the least important parts
first: the seed, your position, your vision, the light at your feet, and the turn
count; then it shortens the moon's [phase](sun.md#phases) to its picture, and only then drops it; then the
temperature, the region, then the year of the date, then the date. The [weather](weather.md#how-the-weather-changes), the time, your
[burden](burden.md#burden-1), your [hunger](hunger.md#how-hungry-you-are), your [status effects](status-effects.md#how-they-work) (the least urgent first), your [resources](resources.md) (the last
first), your [mana](mana.md#how-much), your [hit points](hit-points.md#hit-points),
and the name of the level stay longest, and from `Hungry` down your hunger outlasts them all. Parts come back as soon as there is room.

## Messages

The message line shows the messages of this turn, everything said since you last acted, one
after another, wrapped at the ends of words. Rows they leave free show earlier messages, one a
row; those from before the last turn are dimmed. A message repeated in a row is shown once with
a count, as in `You open the door. (×2)`.

When a turn says more than the two rows hold, the line shows what fits followed by `--more--`
and waits: `Space` or `Enter` shows the rest, a page at a time, and `Esc` skips what is left of
this turn's messages. Until then no other key does anything (a key you hold down never turns
the page), so nothing happens that you have not read. A [rest](resting.md#how-resting-works) or a long [dig](digging.md) counts as one turn
here: what was said during it, and what stopped it, shows the same way. Both keys can be rebound in their own group in Controls
(skipping has no key until you bind one; `Esc` always skips). A screen that opens by itself,
such as someone answering you, comes first: `--more--` waits beneath it until it closes.

`v` opens the **message history**: every message the game keeps (the last 200), the newest at
the bottom, where it opens. The up and down keys scroll it a line at a time, `<` and `>` (or
`PageUp` and `PageDown`) a screenful; `Esc`, `Enter`, or `v` closes it. It takes no time.

## Character

`@` shows your name, your [race](races.md), [gender](character.md#gender), and [class](classes.md), your [age, birthday](character.md#age-birthday-and-sign), and
[birth sign](horoscope.md), your [stats](stats.md) (one that a status effect or a disease lowers
is shown lowered, in red), your [hit points](hit-points.md#hit-points) and [Defence](combat.md#defence) (`Dv`; see [Combat](combat.md)), what ails
you (`Status  Bleeding, Poisoned`; see [Status effects](status-effects.md)), the diseases you know
you have (`Diseased  Filth fever`; see [Diseases](diseases.md)), your weight and what you
carry, and the weights at which your [burden](burden.md) changes. It takes no time; press `@` or
`Esc` to close it, or `A` for your skills.

## Skills

`A` lists the [skills](skills.md) you are trained in, each with its value and a small meter of how
practised it is (how big its next [roll at a level-up](skills.md#getting-better) will be). The arrow keys pick a skill to read what it is about. It takes no
time; press `A` or `Esc` to close it.

## Info pages

The [character screen](#character), the [inventory](objects.md#your-pack), the [equipment screen](equipment.md#the-equipment-screen), the [skills screen](#skills), the [weapon skills](weapon-skills.md)
screen, the [Magic page](schools.md#the-magic-page), the [quest log](quests.md#the-quest-log), the [talents](talents.md) page, and [your story](background.md#when-it-is-told) are numbered pages of one set, shown on the bottom
edge of each as
`1 Character · 2 Inventory · 3 Equipment · 4 Skills · 5 Weapons · 6 Magic · 7 Quests · 8 Talents · 9 Story`, the one
you are on highlighted. All pages are the same width, so the frame stays put as you turn them.
On a window too small for that, every page goes by short names
(`1 Char · 2 Inv · 3 Eqp · 4 Skl · 5 Weap · 6 Mag · 7 Qst · 8 Tal · 9 Story`), and on the narrowest
by their numbers alone. While one of them is open, `1` to `9` switch to the page with that number (the row
of number keys above the letters; the numpad still moves), and `Tab` / `Shift`+`Tab` go to the
next or previous page, wrapping around. The page replaces the one you were on, so `Esc` closes it
and takes you back to where you opened the first one, as if you had only ever opened that one.
The number keys do nothing during play, and they keep their own meaning on a screen that is a
step of a page, such as choosing what goes in a slot. Each page but the Magic page also keeps
its own key (`@`, `I`, `i`, `A`) to open it and to close it; the Magic page is reached by its
number from another page. All of these keys can be rebound in the Controls screen, as
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
[Your character](character.md)). While any menu or screen is open (including the [inventory](objects.md#your-pack), the
[equipment screen](equipment.md#the-equipment-screen), the [character screen](#character), the pick-up and drop screens, the [abilities menu](abilities.md#the-abilities-menu), and the
screen of [topics](npcs.md#chatting) someone you [chat](npcs.md#chatting) with offers), no game time passes, except for what you choose
there: [picking up](objects.md#picking-up), [dropping](objects.md#dropping), putting on or taking off, and an [ability](abilities.md) take their time. Asking
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
| `F6` | Skip an hour: the [clock](time.md#the-calendar) jumps ahead, and nobody acts meanwhile |
| `F7` | Set the next [weather](weather.md) for three hours, after which the usual weather returns |
| `F8` | Identify everything you carry: [magic items](magic-items.md) show what they are |
