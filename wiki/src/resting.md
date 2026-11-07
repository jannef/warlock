# Resting

Press `b` to rest. The [message line](the-screen.md#messages) asks:

```
Rest: (h)our, until (d)awn, until (n)ightfall? (Esc to cancel)
```

| Key | Rests |
|---|---|
| `h` | for one hour |
| `d` | until the next 06:00 ([dawn](time.md#days)) |
| `n` | until the next 18:00 ([dusk](time.md#days)) |

Any other command cancels the prompt. Resting until [dawn](time.md#days) or [dusk](time.md#days)
always means the next one, even if it is that time right now.

## How resting works

Resting is a single action that waits turn after turn, each wait a normal action (6 seconds). It
works the same on the [overworld](overworld.md#travel). [Creatures](creatures.md) keep acting and
the world keeps changing. The last wait is cut short, so the rest ends exactly at its goal. A rest
never lasts more than a day.

While you rest, creatures away from you take their turns in larger steps, so a night's rest on a
level full of them goes quickly. A creature counts as away unless it is
[hostile](creatures.md#factions) to you and you (or the spot it is heading for) are within its
[sight](sight.md#line-of-sight). Away from you:

- A creature going about its routine walks up to **20 steps** of its round at once, then waits
  them out, so it keeps its own pace. "Its routine" means it sees no enemy, [hears](hearing.md)
  nothing worth a look, and has no [status effects](status-effects.md#how-they-work). It never
  takes such a stride onto a cell where you saw creatures when you last looked, nor where you could
  [hear](hearing.md) its step, nor to where it could see an enemy. Only its last step is heard. One
  standing still, or with someone in its way, waits 20 turns at once.
- A creature busy with something else (fighting another creature, looking into a noise, going
  back to its post) acts as usual, but **20 times slower**.

Creatures near you, in view, or in [earshot](hearing.md) take every turn as usual, and outside a
rest every creature does.

A rest stops early when:

- A creature comes into view that you did not see the turn before ("A kobold comes into view; you
  stop resting after 2 hours."). The exception is the people who talk and fight no one (see
  [People and talk](npcs.md)): you see who it is, and rest on.
- A message is written to the [message line](the-screen.md#messages), such as a new
  [sound](hearing.md#what-you-hear) ("You hear footsteps to the north-east.") or an attack you
  notice ("The kobold hits you."). A sound you have already been hearing does not stop you; see
  [Hearing](hearing.md). Nor does a [status effect](status-effects.md#how-they-work) wearing off
  ("The bleeding stops.", "The poison wears off.") or a disease leaving you ("Your fever
  breaks."); see [Status effects](status-effects.md) and [Diseases](diseases.md).
- Under the open sky, [rain](weather.md) starts or grows heavier ("It starts to rain.", "It
  starts to pour.", "The rain grows heavy."). Other news of the [weather](weather.md), such as the
  rain stopping, and someone [muttering](npcs.md#muttering) to themselves, is written to the
  message line but lets you rest on.
- You get [hungrier](hunger.md#how-hungry-you-are): [hungry](hunger.md#how-hungry-you-are),
  [weak](hunger.md#how-hungry-you-are), [fainting](hunger.md#how-hungry-you-are), or
  [starving](hunger.md#how-hungry-you-are) ("You are getting hungry."). Fainting from hunger stops
  it too, and so does losing a [hit point](hit-points.md#hit-points) to starving ("You are wasting
  away from hunger."). Resting burns [food](food.md#eating) like any other time (see
  [Hunger](hunger.md)). A full stomach settling ("You no longer feel satiated.") lets you rest on.
- On the [overworld](overworld.md#travel), someone finds your camp (see [Camping and
  foraging](encounters.md#camping-and-foraging)).

So resting next to a creature that walks back and forth behind a [door](doors.md) is interrupted
each time it comes back into [earshot](hearing.md). A [creature](creatures.md) already in view
when you start does not stop the rest just by being there. It stops the rest only by leaving your
[sight](sight.md#line-of-sight) and coming back, or by an attack you notice.

[Digging](digging.md) goes on turn after turn the same way, and stops early for the same reasons.

When the rest ends, the game tells you how long you rested ("You rest for 1 hour.", "You rest
until dawn."). Rest to wait out the night, to wait for the [light](light.md#what-light-is) to
change, or to let your wounds heal. [Hit points](hit-points.md#hit-points) come back with time,
faster the [tougher](stats.md#stat-toughness) you are, and **twice as fast while you rest** (see
[Healing](healing.md)). [Bleeding](status-effects.md#bleeding) and
[poison](status-effects.md#status-poisoned) still cost hit points while you rest, and bleeding
stops your healing until it stops.
