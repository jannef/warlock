# Resting

Press `R` to rest. The message line asks:

```
Rest: (h)our, until (d)awn, until (n)ightfall? (Esc to cancel)
```

| Key | Rests |
|---|---|
| `h` | for one hour |
| `d` | until the next 06:00 (dawn) |
| `n` | until the next 18:00 (dusk) |

Any other command cancels the prompt. Resting until dawn or dusk always means the next one, even if
it is that time exactly.

## How resting works

Resting is a single action that waits turn after turn, each wait a normal action (6 seconds), on
the overworld too. Creatures keep acting and the world keeps being updated as usual, and the last
wait is cut short so the rest ends exactly at its goal. A rest never lasts more than a day.

A rest stops early when:

- a creature comes into view that you did not see the turn before ("A kobold comes into view; you
  stop resting after 2 hours."),
- a message is written to the message line, such as a new sound you hear ("You hear footsteps to
  the north-east.") or an attack you notice ("The kobold hits you."). A sound you have already
  been hearing does not stop you; see [Hearing](hearing.md), or
- under the open sky, rain starts or grows heavier ("It starts to rain.", "It starts to pour.",
  "The rain grows heavy."). Other news of the [weather](weather.md), such as the rain stopping,
  is written to the message line but lets you rest on.

So resting next to a creature walking back and forth behind a door is interrupted each time it
comes back into earshot. A creature already in view when you start does not stop the rest by
being there: only by leaving your sight and coming back, or by an attack you notice.

When it ends, the game tells you how long you rested ("You rest for 1 hour.", "You rest until
dawn."). Resting is how you wait out the night, wait for the light to change, or let your
wounds heal: hit points come back with time, faster the tougher you are (see
[Combat](combat.md#getting-better)).
