# Resting

Press `b` to rest. The [message line](controls.md#messages) asks:

```
Rest: (h)our, until (d)awn, until (n)ightfall? (Esc to cancel)
```

| Key | Rests |
|---|---|
| `h` | for one hour |
| `d` | until the next 06:00 ([dawn](time.md#days)) |
| `n` | until the next 18:00 ([dusk](time.md#days)) |

Any other command cancels the prompt. Resting until [dawn](time.md#days) or [dusk](time.md#days) always means the next one, even if
it is that time exactly.

## How resting works

Resting is a single action that waits turn after turn, each wait a normal action (6 seconds), on
the [overworld](overworld.md#travel) too. [Creatures](creatures.md) keep acting and the world keeps being updated, and the last wait is
cut short so the rest ends exactly at its goal. A rest never lasts more than a day.

While you rest, creatures away from you take their turns coarsely, so a night's rest on a level
full of them is quick. A creature is away unless it is [hostile](creatures.md#factions) to you and you (or the spot it
is heading for) are within its [sight](sight.md#line-of-sight). Away from you:

- a creature going about its routine (seeing no enemy within its sight, [hearing](hearing.md) nothing worth a
  look, with no [status effects](status-effects.md#how-they-work)) walks up to **20 steps** of its round at once and then waits them
  out, so it keeps its own pace. It never takes such a stride onto a cell where you saw creatures
  when you last looked, nor where you could [hear](hearing.md) its step, nor to where it could see an enemy,
  and only its last step is heard. One standing still, or with someone in its way, waits 20
  turns at once.
- a creature busy with something else (fighting another creature, looking into a noise, going
  back to its post) acts as usual but **20 times slower**.

Near you, in view, or in [earshot](hearing.md), creatures take every turn as usual, and outside a rest they
always do.

A rest stops early when:

- a creature comes into view that you did not see the turn before ("A kobold comes into view; you
  stop resting after 2 hours."), unless it is one of the people who talk and fight no one (see
  [People and talk](npcs.md)): you see who it is, and rest on,
- a message is written to the [message line](controls.md#messages), such as a new [sound](hearing.md#what-you-hear) you hear ("You hear footsteps to
  the north-east.") or an attack you notice ("The kobold hits you."). A sound you have already
  been hearing does not stop you; see [Hearing](hearing.md). A [status effect](status-effects.md#how-they-work) wearing off ("The
  bleeding stops.", "The poison wears off.") or a disease leaving you ("Your fever breaks.") does
  not stop you either; see [Status effects](status-effects.md) and [Diseases](diseases.md),
- under the open sky, [rain](weather.md) starts or grows heavier ("It starts to rain.", "It starts to pour.",
  "The rain grows heavy."). Other news of the [weather](weather.md), such as the rain stopping,
  and someone [muttering](npcs.md#muttering) to themselves, is written to the message line but lets you rest on, or
- you get [hungrier](hunger.md#how-hungry-you-are): [hungry](hunger.md#how-hungry-you-are), [weak](hunger.md#how-hungry-you-are), [fainting](hunger.md#how-hungry-you-are), or [starving](hunger.md#how-hungry-you-are) ("You are getting hungry."), or you faint
  from hunger. Resting burns [food](food.md#eating) like any other time (see [Hunger](hunger.md)); a full stomach
  settling ("You no longer feel satiated.") lets you rest on.

So resting next to a creature walking back and forth behind a [door](doors.md) is interrupted each time it
comes back into [earshot](hearing.md). A [creature](creatures.md) already in view when you start does not stop the rest by
being there: only by leaving your [sight](sight.md#line-of-sight) and coming back, or by an attack you notice.

[Digging](digging.md) goes on turn after turn the same way, and stops early for the same
reasons.

When it ends, the game tells you how long you rested ("You rest for 1 hour.", "You rest until
dawn."). Resting is how you wait out the night, wait for the [light](light.md#what-light-is) to change, or let your
wounds heal: [hit points](combat.md#hit-points) come back with time, faster the [tougher](stats.md#stat-toughness) you are, and **twice as fast
while you rest** (see [Healing](healing.md)). [Bleeding](status-effects.md#bleeding) and [poison](status-effects.md#status-poisoned) still cost [hit points](combat.md#hit-points) while you
rest, and bleeding stops your healing until it stops.
