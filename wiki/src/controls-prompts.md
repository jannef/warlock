# Prompts, looking, and aiming

## Prompts

Some commands ask a question on the [message line](the-screen.md#messages) before they act:

- [Kicking](doors.md#kicking-a-door-open) asks `In which direction? (Esc to cancel)`, and so do opening or
  closing a [door](doors.md), [bashing](abilities.md#ability-shield-bash) with your shield, and
  [stealing](abilities.md#ability-steal) when more than one thing beside you could be meant (see
  [Something beside you](#something-beside-you)).
- [Digging](digging.md), with more than one wall beside you to dig, asks `Dig in which direction?
  (Esc to cancel)` (see [Digging and walls](digging.md)).
- [Chatting](npcs.md#chatting) when several creatures are near asks `Talk in which direction? (Esc
  to cancel)` (see [People and talk](npcs.md)).
- [Disarming a trap](traps.md#disarming-traps), with more than one known trap beside you, asks
  `Disarm in which direction? (Esc to cancel)`
  (see [Traps](traps.md#disarming-traps)).
- [Resting](resting.md#how-resting-works) asks how long (see [Resting](resting.md)).

Answer with a direction or the key shown, or press `Esc` to cancel. Pressing any other command also
cancels the prompt. The [rest prompt's](resting.md#how-resting-works) keys (`h`, `d`, `n`) are its
own and fixed: in play `d` still drops.

A few things ask first, in a small window over the map: quitting, stepping onto a trap you know
(see [Traps](traps.md)), leaving a [shop](shops.md) owing money, and dropping something a shop
would buy (see [Shops](shops.md)). The window lists its answers, each with the keys that choose it.
`y` picks the yes answer, `n` the no answer, and `Esc` the one that backs out. You can also move
with the arrow keys and press `Enter`. The safe answer is selected at first, so pressing `Enter`
alone never does something you might regret. The yes and no keys are their own: you can rebind
them without touching any key of play.

## Something beside you

A command or [ability](abilities.md) that works on something next to you looks around you first:

- With **nothing** beside you to use it on, it does nothing and tells you so at once, taking no
  time. No prompt opens.
- With **one** thing, it acts on that at once, without asking.
- With **two or more**, it asks which way, as above.

Only what you know counts: a door or chest you see or remember, a trap you know of, a creature you
can see. What each one looks for:

| Command or ability | What it works on beside you |
|---|---|
| Open | a closed [door](doors.md) or a shut [chest](chests.md) |
| Close | an open door with nothing standing in it |
| [Chat](npcs.md#chatting) | any creature |
| Use tool, with a [digging](digging.md) tool ready | a wall you can dig through |
| [Disarm a trap](abilities.md#ability-disarm) | a trap you know of |
| [Pick lock](abilities.md#ability-pick-lock), [Knock](abilities.md#ability-knock) | a closed door or a shut chest (unless you can see the chest is not locked) |
| [Listen at a door](abilities.md#ability-listen) | a closed door |
| [Wedge a door](abilities.md#ability-wedge) | a closed door not wedged already |
| [Steal](abilities.md#ability-steal) | a creature that is not hostile |
| [Feint](abilities.md#ability-feint) | a hostile creature with a mind to fool |
| [Tame](abilities.md#ability-tame) | a beast that can be tamed and is no ally |
| [Gather](abilities.md#ability-gather) | the glowing fungus |
| [Shield bash](abilities.md#ability-shield-bash), the smites and other class blows, weapon blows aimed in a direction | a hostile creature (a lunge reaches one two paces off) |

[Kicking](doors.md#kicking-a-door-open) always asks: you may kick any way you like.

## Look mode

`l` puts a cursor on the map, on you. Move it with the movement keys, and the message line
describes what you know about the cell under it:

- What you see there. For a [creature](creatures.md): "hostile" if it is
  [hostile](creatures.md#factions) to you, how hurt it looks (unhurt, lightly wounded, wounded,
  badly wounded, or nearly dead; see [Combat](combat.md)), whether it
  [bleeds](status-effects.md#bleeding) (see [Status effects](status-effects.md)), and what it
  [wields and wears](equipment.md#what-fits-where). Then the [objects](objects.md) lying there and
  the [terrain](terrain.md): "A giant rat (hostile, unhurt), on stone floor." or "A kobold
  (hostile, badly wounded, bleeding), wielding a spear, on stone floor." A wall you have started
  digging into is "partly dug".
- A [trap](traps.md#finding-traps) you know there, as part of the ground: "A pit, on stone
  floor." (see [Traps](traps.md)).
- What you remember there, if you cannot see it now: the terrain and the objects you saw on it.
- Where [stairs](terrain.md#stairs) and places on the [overworld](overworld.md#locations) lead.
- The [special room](special-rooms.md) the cell lies in, once you have been inside it: "A stone
  floor in an echoing hall."
- A [sound](hearing.md) you heard there this turn ([Hearing](hearing.md)).
- Otherwise, "You don't know what is there."

If your [Knowledge](skills.md#skill-knowledge) is above a creature's level, a second line says
what you know of its kind: "You know the skeleton is immune to poison, resistant to piercing, and
vulnerable to bludgeoning." (see [Skills](skills.md#skill-knowledge)).

Looking takes no time. Press `l` or `Esc` to leave.

## Aiming

`t` shoots or throws whatever is in your Missile weapon slot. A [bow, a crossbow, or a
sling](ranged.md#getting-ready) shoots its [ammunition](ammunition.md) from the Missiles slot; you
ready both with `i`. A [throwing weapon](ranged.md#shooting-and-throwing) (a dagger, a spear,
rocks) or any other weapon is thrown. `T` instead lists what you carry that can be thrown, so you
can choose one. There is no separate key for firing a launcher: `t` does it.

Either key puts a cursor on the map:

- It starts on your last target if you still see it and it is within range (an empty spot is never
  remembered), else on the nearest [hostile](creatures.md#factions) creature you see within range,
  else on you.
- `Tab` moves it to the next creature you see, nearest first, and `Shift`+`Tab` moves it back. The
  movement keys move it anywhere, into the dark too.
- A faint dotted line shows the way the missile would fly, as far as you know the ground. It ends
  before anything solid you know of, and at the edge of the range. A target past the line's end is
  marked on the cursor, and the message line says "Out of range." or "The way is blocked." (a
  wall, or a creature in the way that the missile would reach first).
- The [message line](the-screen.md#messages) describes the target as [look mode](#look-mode) does.
  It never shows a chance to hit.
- `Enter`, or the key that started the aim (`t` again, or `T` for a throw), shoots or throws, and
  `Esc` cancels. The next time, the cursor starts on the same target, so `t` `Enter` shoots it
  again.

`Tab` and `Enter` are fixed. Only the previous target is in Controls, with no key until you bind
one. An [ability](abilities.md) that asks for a place is aimed the same way: `Enter`, the
abilities key `a`, or the key that repeats the last ability, `X`, uses it.

A spell scroll you read (`r`, see [Scrolls](scrolls.md)) is aimed the same way, and `Enter` or `r`
reads it at the cursor. A [lightning bolt](magic.md#spell-lightning-bolt) runs through creatures,
so only walls block its line. If a [fireball](magic.md#spell-fireball) would land close enough to
catch you, the game asks first.

[Throwing your voice](hearing.md#throwing-your-voice) (an [ability](abilities.md)) is aimed the
same way too. Nothing flies, so there is no line. On a cell you do not see, or a wall, the cursor
is marked and the message line says "You do not see there."; beyond 6 cells it says "Out of
range." `Enter` throws your voice there.

See [Ranged combat](ranged.md).

See also: [Controls](controls.md) and [Menus and lists](controls-menus.md).
