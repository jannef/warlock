# Creatures

Creatures are the other glyphs that move about the world. Today they are simple: they cannot
fight, notice you, or speak. This page will grow as the game does.

## What they do

- Each creature walks a fixed, looping **patrol route** and takes one step each turn, however long
  your own action took.
- If something blocks its way (a closed door, another creature, you), it waits and tries again the
  next turn.
- Creatures only act on the level you are on.
- You only see a creature while you see its cell, so creatures in the dark are hidden unless they
  carry a light. See [Sight and memory](sight.md). You never remember where one was.

Combat is not in the game yet: nothing can hurt you, and you cannot hurt anything.

## The creatures

{{#include generated/creatures.md}}
