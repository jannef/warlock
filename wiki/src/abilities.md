# Abilities

Some of what you can do comes from who you are: your race, your class, the skills you are
trained in, your weapon skills and fighting styles, and, later, things you carry. These are your
**abilities**. Unlike in ADOM, they do not each get a key of their own: they are all in one
menu. The basic commands (moving, opening doors, kicking, searching, picking up, eating, shooting,
resting, ...) keep their keys (see [Controls](controls.md)).

## The abilities menu

`a` opens the abilities menu. It lists your abilities in groups, by where they come from
(Racial, Class, Talents, Skills, Weapon skills, Items; a group you have nothing in is left out), each with
a letter:

- beside each, what it asks for once you choose it, such as `in a direction`;
- an ability you cannot use right now is greyed out, with why: `you hold no shield`, `not
  usable yet` for a skill whose use is not in the game yet, or `you are winded` while a
  [status effect](status-effects.md#buffs-and-cooldowns) keeps you from it;
- the arrow keys move the `>` cursor, and the ability under it is described at the bottom;
- a letter, or `Enter` on the `>` row, uses it: the menu closes, the ability asks for what it
  needs (a direction, on the message line), and then it takes its time like any other action.
  Choosing one you cannot use only tells you why;
- a long list shows a page at a time: `>` and `<` turn the pages, and the letters start again at
  `a` on every page;
- `Esc` closes the menu.

Looking at the menu takes no time. It remembers the ability you used last and opens on it, so
`a` `Enter` uses it again, and `A` does the same in one key (with nothing used yet, `A` opens the
menu).

## Every ability

{{#include generated/abilities.md}}

- **Shield bash**: anyone can bash with a shield (see [Weapon skills](weapon-skills.md)), so it is
  always in the menu, and you can use it while you hold a shield. It strikes whoever stands in
  the direction you choose, in the time of a blow.
- **Disarm a trap**: everyone trained in Disable device (the thief) has it, in place of the
  skill. Choose a direction, toward a trap you know of next to you, on the floor or on a door:
  you try to take it apart (see [Traps](traps.md#disarming-traps)).
- **Pick lock**: everyone trained in Open lock (the thief) has it, in place of the skill. Choose a
  direction, toward a closed door next to you: you try its lock (see
  [Doors](doors.md#picking-locks)).
- **Other skills used as actions** (Steal, Heal, ...): each is in the menu once you are trained
  in it (see [Skills](skills.md)), greyed out until its use comes to the game.
- **Class abilities** (Rage, Veteran's wind, Whirlwind, Lay on hands, Smite, Holy radiance,
  Quivering palm): your class's level table gives them (see [Classes](classes.md#what-each-level-gives)),
  each under Class from the level that brings it. Most leave you spent, winded, or drained for a
  while afterwards, greyed out until it passes; a paladin's gifts answer only while the paladin
  is Good or better (greyed out with `you have strayed` otherwise). They have no key of their own
  yet: `A` uses the one you used last again.
- No race or item gives an ability yet.

## Keys of their own

If you use an ability often, you can give it a key in the Controls screen (the shield bash is
"Bash with your shield", disarming "Disarm a trap", and picking a lock "Pick a lock", none with a
key by default): the key uses the ability just as the menu would, asking for a direction if it
needs one. Without a shield, it says so at once and takes no time. `a` ("Abilities") and `A`
("Use the last ability again") can be rebound like every key.
