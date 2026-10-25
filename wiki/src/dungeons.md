# Dungeons and danger

A dungeon is a stack of floors made the first time you set foot on each, then kept as you left
them. Every place has a **danger level**, a number the game never shows you. On a dungeon floor
it is the dungeon's base danger plus a step for every floor down; a hand-made place has one of its
own; the overworld and the land you explore from it have none. A place could also grow more
dangerous as you do (a puppy cave of the old games), but no place does yet.

## Who lives there, and how many

Each dungeon names the creatures that live in it. Every creature's level is its danger: on a
floor of danger 6, the creatures you meet are mostly of level 5 and 6, some a little weaker, a
few weaker still, and now and then one from deeper down. Each time a creature is chosen, every
kind that lives there gets a weight by how far its level is below the floor's danger, and the
choice goes by weight (kinds of the same weight are equally likely):

{{#include generated/danger-weights.md}}

One draw in twenty (unless a place says otherwise) is made one to three dangers deeper, so a
careless step on a quiet floor can still meet something it should not. A creature marked "once a
run" comes at most once in your life, however many floors it could come to.

## A floor starts full

A dungeon floor with a danger is full when you first come down to it: its population (the
number in the tables below; by default 35 creatures for every 1000 cells you could walk on, at
least 25 and at most 80) is chosen as above and spread over its plain floor, never within 8
steps of a staircase, never in a shop or a sanctuary. Most walk a short beat, back and forth;
where there is no room for one, they wait where they stand. They come armed as their kind is
(see the creatures' gear), on top of the few the floor's own rooms and vaults hold. A creature
marked "once a run" that you have met already is not among them. A hand-made place with a
danger starts as it was made; only the arrivals below fill it up.

## They keep coming

While you are on a floor with a danger, the creatures that live there keep coming: every so
often (a quarter of an hour of game time, unless a place says otherwise) one may come, with a
chance, as long as the floor holds fewer creatures hostile to you than its population. It comes
somewhere you cannot see, never within 8 steps of you, never on stairs, in a doorway, on a shop
floor, or in a sanctuary. A cleared floor fills up again while you stay; a long rest below lets
it refill. A floor you are not on is frozen: nothing comes while you are away, and the clock
starts again when you come back.

The talents of the Delver tree help: **Quiet delver** puts each next arrival off twice as long,
and **Sixth sense** tells you how a place feels against your own level as you arrive (quiet,
uneasy, dangerous, or deadly), and at its second rank notes it when something stirs on your
floor.

## The dungeons

{{#include generated/dungeons.md}}

### Floor by floor

{{#include generated/dungeon-floors.md}}
