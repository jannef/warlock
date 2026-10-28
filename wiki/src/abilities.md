# Abilities

Some of what you can do comes from who you are: your [race](races.md), your [class](classes.md), the [skills](skills.md) you are
trained in, your [weapon skills](weapon-skills.md) and [fighting styles](fighting-styles.md), and, later, things you carry. These are your
**abilities**. Unlike in ADOM, they do not each get a key of their own: they are all in one
menu. The basic commands (moving, opening [doors](doors.md), [kicking](doors.md#kicking), [searching](searching.md#searching-1), [picking up](objects.md#picking-up), [eating](food.md#eating), [shooting](ranged.md#shooting-and-throwing),
[resting](resting.md#how-resting-works), ...) keep their keys (see [Controls](controls.md)).

## The abilities menu

`a` opens the abilities menu. It lists your abilities in groups, by where they come from
(Racial, Class, Talents, Skills, Weapon skills, Items; a group you have nothing in is left out), each with
a letter:

- beside each, what it asks for once you choose it, such as `in a direction`;
- an ability you cannot use right now is greyed out, with why: `you hold no shield`, `not
  usable yet` for a [skill](skills.md) whose use is not in the game yet, or `you are winded` while a
  [status effect](status-effects.md#buffs-and-cooldowns) keeps you from it;
- the arrow keys move the `>` cursor, and the ability under it is described at the bottom;
- a letter, or `Enter` on the `>` row, uses it: the menu closes, the ability asks for what it
  needs (a direction, on the message line, or a place, with the aiming cursor, see
  [Aiming](controls.md#aiming)), and then it takes its time like any other action.
  Choosing one you cannot use only tells you why;
- a long list shows a page at a time: `>` and `<` turn the pages, and the letters start again at
  `a` on every page;
- `Esc` closes the menu.

Looking at the menu takes no time. It remembers the ability you used last and opens on it, so
`a` `Enter` uses it again, and `X` does the same in one key (with nothing used yet, `X` opens the
menu).

## Every ability

{{#include generated/abilities.md}}

What the ranks of [weapon skills](weapon-skills.md) and [fighting styles](fighting-styles.md) teach is listed in the menu too, under
*Weapon skills* after the skill's name ("Long blades: Deep cut"), greyed with `fires by itself`:
those abilities are never chosen, they work by themselves as you fight. They have tables of their
own: [Weapon skills](weapon-skills.md#what-a-rank-teaches) and
[Fighting styles](fighting-styles.md#what-a-rank-teaches).

- **Throw your voice**: a voice sounds at a place you choose that you see, not solid, within 6
  cells (the [aiming](controls.md#aiming) cursor, as for a shot). It is as loud there as someone talking, and it is
  made there, not where you stand, so [creatures](creatures.md) that hear it and cannot see that spot go to have
  a look (see [Hearing](hearing.md#throwing-your-voice)). It takes a normal action. [Bards](classes.md#bard) learn it at level 32
  (see [Classes](classes.md)).
- **Shield bash**: anyone can bash with a shield (see [Weapon skills](weapon-skills.md)), so it is
  always in the menu, and you can use it while you hold a shield. It strikes whoever stands in
  the direction you choose, in the time of a blow.
- **Disarm a trap**: everyone trained in [Disable device](skills.md#skill-disable-device) (the [thief](classes.md#thief)) has it, in place of the
  skill. Choose a direction, toward a [trap](traps.md#finding-traps) you know of next to you, on the floor or on a [door](doors.md):
  you try to take it apart (see [Traps](traps.md#disarming-traps)).
- **Pick lock**: everyone trained in [Open lock](skills.md#skill-open-lock) (the [thief](classes.md#thief)) has it, in place of the skill. Choose a
  direction, toward a closed [door](doors.md) next to you: you try its lock (see
  [Doors](doors.md#picking-locks)).
- **Steal**: anyone trained in the [Steal](skills.md#skill-steal) skill has it, in the skill's place among the Skills.
  It asks for a direction and tries to lift something from whoever stands there, if they are
  not hostile to you (see [Stealing](skills.md#stealing)).
- **Heal**: anyone trained in the [Heal](skills.md#skill-heal) skill has it, in the skill's place among the Skills. It
  treats you: your [bleeding](status-effects.md#bleeding), a [poison](status-effects.md#poisoned), or a [disease](diseases.md#how-a-disease-goes) that shows, asking which when there is more
  than one; greyed out "nothing to treat" otherwise (see
  [Treating yourself](skills.md#treating-yourself)).
- **Forage**: anyone trained in [Survival](skills.md#skill-survival) has it, in the skill's place among the Skills. It asks
  for nothing: you spend ten minutes searching the wild around you for food. Only in the wild,
  on the [overworld](overworld.md#travel) and in its [local areas](overworld.md#local-areas); elsewhere it is greyed out, "not usable here" (see
  [Foraging](skills.md#foraging)).
- **Other skills used as actions** ([Handle animal](skills.md#skill-handle-animal), ...): each is in the menu once you are trained
  in it (see [Skills](skills.md)), greyed out until its use comes to the game.
- **[Class abilities](talents.md#class-abilities)** ([Rage](abilities.md#ability-rage), [Veteran's wind](abilities.md#ability-veterans-wind), [Whirlwind](abilities.md#ability-whirlwind), [Lay on hands](abilities.md#ability-lay-on-hands), [Smite](abilities.md#ability-smite), [Holy radiance](abilities.md#ability-holy-radiance),
  [Quivering palm](abilities.md#ability-quivering-palm), [Consecrate](abilities.md#ability-consecrate), [Knock](abilities.md#ability-knock)): your [class](classes.md)'s level table gives them (see [Classes](classes.md#levels)),
  each under Class from the level that brings it. Most leave you [spent](status-effects.md#status-spent), [winded](status-effects.md#status-winded), or [drained](status-effects.md#status-drained) for a
  while afterwards, greyed out until it passes; a [paladin](classes.md#paladin)'s holy gifts are never greyed out by
  [alignment](alignment.md), but do nothing unless you are Kind or better (see [Alignment](alignment.md)). They have no key of their own
  yet: `X` uses the one you used last again.
- No [race](races.md) gives an ability yet.

## Keys of their own

If you use an ability often, you can give it a key in the [Controls](controls.md#rebinding-keys) screen (the [shield bash](abilities.md#ability-shield-bash) is
"Bash with your shield", [disarming](traps.md#disarming-traps) "Disarm a trap", [picking a lock](doors.md#picking-locks) "Pick a lock", [stealing](skills.md#stealing)
"Steal", and [throwing your voice](hearing.md#throwing-your-voice) "Throw your voice"; none has a
key by default): the key uses the ability just as the menu would, asking for a direction or a
place if it needs one. Without a shield, it says so at once and takes no time. `a` ("Abilities")
and `X` ("Use the last ability again") can be rebound like every key.
