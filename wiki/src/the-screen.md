# The screen

The map fills most of the window. Above it, the **[message line](#messages)** (two rows) tells you
what this turn brought. Prompts and [look mode](controls-prompts.md#look-mode) use the same line. This page
also covers the info pages, the menus, and the death screen; the keys are on
[Controls](controls.md).

Below the map, the **status bar** shows, from left to right:

- The name of the level. On the overworld and in its local areas, also the region you are in.
- Your [hit points](hit-points.md#hit-points), as `HP 18/24`: what you have left, then your most
  (see [Combat](combat.md)).
- Your [mana](mana.md#how-much-mana-you-have), as `MP 5/12`, the same way (see [Mana](mana.md)).
- Each [resource](resources.md) you hold, as `Rage 7` or `Focus 7/10` (see
  [Resources](resources.md)).
- What ails you, if anything, the most urgent first: `Bleeding`, `Poisoned`, `Diseased` (see
  [Status effects](status-effects.md) and [Diseases](diseases.md)).
- How [hungry](hunger.md#how-hungry-you-are) you are, unless you are not hungry: `Hungry`, `Weak`,
  `Fainting`, `Starving`, `Satiated`, or `Bloated`. From `Hungry` down, the colour grows more
  alarming as it gets worse (see [Hunger](hunger.md#what-you-see)).
- Your [burden](burden.md#burden-levels), when you have one: `Burdened`, `Strained`, or `Overloaded`
  (see [Burden](burden.md)).
- The time, the date, and the [phase of the moon](sun.md#phases) (see [Time and the
  calendar](time.md)).
- The [weather](weather.md#how-the-weather-changes), while you are under the open sky (see
  [Weather](weather.md)).
- The [temperature](temperature.md) where you are, as `4 °C` (or °F). It is cool blue below
  freezing and warm orange from 30 °C (see [Temperature and ice](temperature.md)).
- The [turn](time.md#what-takes-time) count and your position (column, row).
- `Light`: how brightly your own cell is [lit](light.md#what-light-is), where 255 is full light (see
  [Light](light.md)).
- Your [vision type](vision.md#the-four-types).

At its right end, `? controls` reminds you where the keys are. It names the key as you bound it
(`Esc menu` when the controls have no key), and so does the welcome line at the start of a new
run.

When the window is too narrow for everything, the status bar leaves out the least important parts
first:

1. your position, your vision, the light at your feet, and the turn count;
2. then the moon's [phase](sun.md#phases), first shortened to its picture, then dropped;
3. then the temperature, the region, the year of the date, and then the date.

The [weather](weather.md#how-the-weather-changes), the time, your [burden](burden.md#burden-levels),
your [hunger](hunger.md#how-hungry-you-are), your [status effects](status-effects.md#how-they-work)
(the least urgent go first), your [resources](resources.md) (the last one goes first), your
[mana](mana.md#how-much-mana-you-have), your [hit points](hit-points.md#hit-points), and the name of the level
stay longest. From `Hungry` down, your hunger outlasts them all. Parts come back as soon as there
is room.

## Messages

The message line shows this turn's messages: everything said since you last acted, one after
another, wrapped at the ends of words. Rows they leave free show earlier messages, one per row;
those from before the last turn are dimmed. A message repeated in a row is shown once with a count,
as in `You open the door. (×2)`.

When a turn says more than the two rows hold, the line shows what fits, then `--more--`, and
waits. `Space` or `Enter` shows the rest, a page at a time, and `Esc` skips the rest of this turn's
messages. Until then no other key does anything, and a key you hold down never turns the page, so
nothing happens that you have not read.

A [rest](resting.md#how-resting-works) or a long [dig](digging.md) counts as one turn here: what was
said during it, and what stopped it, shows the same way. Both keys can be rebound in their own
group in Controls. Skipping has no key until you bind one, but `Esc` always skips. A screen that
opens by itself, such as someone answering you, comes first: `--more--` waits beneath it until it
closes.

`v` opens the **message history**: the last 200 messages, the newest at the bottom, where it
opens. The up and down keys scroll it a line at a time, and `<` and `>` (or `PageUp` and
`PageDown`) a screenful. `Esc`, `Enter`, or `v` closes it. It takes no time.

## Character

`@` shows who you are:

- your name, [race](races.md), [gender](character.md#gender), and [class](classes.md);
- your [age, birthday](character.md#age-birthday-and-sign), and [birth sign](horoscope.md);
- your [stats](stats.md); one lowered by a status effect or a disease is shown lowered, in red;
- your [hit points](hit-points.md#hit-points) and [Defence](combat.md#defence) (`Dv`; see
  [Combat](combat.md));
- what ails you (`Status  Bleeding, Poisoned`; see [Status effects](status-effects.md)), and the
  diseases you know you have (`Diseased  Filth fever`; see [Diseases](diseases.md));
- your weight, what you carry, and the weights at which your [burden](burden.md) changes.

It takes no time. Press `@` or `Esc` to close it, or `A` for your skills.

## Skills

`A` lists the [skills](skills.md) you are trained in. Each shows its value and a small meter of how
practised it is, which is how big its next [roll at a level-up](skills.md#improving-a-skill) will be.
Pick a skill with the arrow keys to read what it does. It takes no time. Press `A` or `Esc` to
close it.

## Info pages

Nine screens form one set of numbered pages: the [character screen](#character), the
[inventory](objects.md#your-pack), the [equipment screen](equipment.md#the-equipment-screen), the
[skills screen](#skills), the [weapon skills](weapon-skills.md) screen, the [Magic
page](schools.md#the-magic-page), the [quest log](quests.md#the-quest-log), the
[talents](talents.md) page, and [your story](background.md#when-it-is-told). The bottom edge of
each shows

`1 Character · 2 Inventory · 3 Equipment · 4 Skills · 5 Weapons · 6 Magic · 7 Quests · 8 Talents · 9 Story`

with the page you are on highlighted. All pages are the same width, so the frame stays put as you
turn them. In a small window the pages go by short names (`1 Char · 2 Inv · 3 Eqp · 4 Skl · 5 Weap
· 6 Mag · 7 Qst · 8 Tal · 9 Story`), and in the narrowest by their numbers alone.

While one of them is open, `1` to `9` switch to the page with that number. These are the number
keys above the letters; on a page the numpad moves the selection, as the arrows do. `Tab` and `Shift`+`Tab` go to the next or previous
page, wrapping around. The new page replaces the one you were on, so `Esc` closes it and takes you
back to where you opened the first one.

The number keys do nothing during play. On a screen that is a step of a page, such as choosing
what goes in a slot, they keep their own meaning. Every page except the Magic page also has its
own key (`@`, `I`, `i`, `A`) that opens and closes it; you reach the Magic page by its number
from another page. The digits and `Tab` are fixed; only "Previous info page" can be given a key
in the Controls screen. They are separate from the play keys, so `Tab` can be both "next target"
while aiming and "next info page" on an info screen.

## Death

When you die, a screen says what killed you (if you saw it), where, when, and after how many
turns. Your save is already gone (see [Saving](saving.md)). `Enter` or `Esc` returns to the main
menu.

## Menus

`Esc` during play, when no prompt is open, opens the pause menu: Resume, Settings, Controls, Save
and quit to menu, and Quit, which saves too.

The main menu, where the game opens, has Continue (when there is a saved run), Characters (when
any character is saved), Graveyard (once any character has died), New run, Settings, Controls, and
Quit; see [Saving](saving.md) and [Settings](settings.md). New run goes on to
creating your character, one step at a time. The arrow keys (or `j` and `k`) choose, `Enter` goes
on, and `Esc` goes back a step (see [Your character](character.md)).

No game time passes while a menu or screen is open. That includes the
[inventory](objects.md#your-pack), the [equipment screen](equipment.md#the-equipment-screen), the
[character screen](#character), the pick-up screen and the pack's drop mode, the [abilities
menu](abilities.md#the-abilities-menu), and the screen of [topics](npcs.md#chatting) someone you
[chat](npcs.md#chatting) with offers. Only what you choose to do there takes time:
[picking up](objects.md#picking-up), [dropping](objects.md#dropping), putting on or taking off,
and using an [ability](abilities.md). Asking about a topic takes none.

## Developer keys

Test builds, and desktop builds started with `--dev`, add a few keys for trying things out. They
are not part of normal play, and the browser version does not have them. `?` lists them, marked
"(dev)", and Controls can rebind them:

| Key | Does |
|---|---|
| `F2` | Reveal the map: draws everything on the level, even what you cannot see, in violet |
| `F3` | Switch to the next [vision type](vision.md) |
| `F4` | Show how brightly each cell you see is lit, as a digit from `0` (dark) to `9` (fully lit) |
| `F5` / `F9` | Save / load a snapshot of the run |
| `F6` | Skip an hour: the [clock](time.md#the-calendar) jumps ahead, and nobody acts meanwhile |
| `F7` | Set the next [weather](weather.md) for three hours, after which the usual weather returns |
| `F8` | Identify everything you carry: [magic items](magic-items.md) show what they are |

See also: [Controls](controls.md), [Menus and lists](controls-menus.md), and
[Your character](character.md).
