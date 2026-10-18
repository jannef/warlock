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

Resting is a single action that waits turn after turn. Creatures keep acting and the world keeps
being updated as usual, and the last turn is cut short so the rest ends exactly at its goal. A rest
never lasts more than a day.

A rest stops early when:

- a creature comes into view that you did not see the turn before, or
- a message is written to the message line, such as a new sound you hear ("You hear footsteps to
  the north-east."). A sound you have already been hearing does not stop you; see
  [Hearing](hearing.md).

So resting next to a creature walking back and forth behind a door is interrupted each time it
comes back into earshot.

When it ends, the game tells you how long you rested. Resting is how you wait out the night, or
wait for the light to change. There are no hit points or other statistics yet, so resting does not
heal anything.
