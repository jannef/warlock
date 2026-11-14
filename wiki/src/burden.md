# Burden

Everything you carry has weight. The more you carry, the harder it is to move, fight, and keep
[quiet](hearing.md#how-loud). A heavy load slows down **everything** you do, not only walking.

## How much you can carry

Your carrying **limit** comes from your [Strength](stats.md#stat-strength), which counts
double, and your [Toughness](stats.md#stat-toughness), which keeps you going under a load all
day:

**Limit = 1.5 kg × (2 × Strength + Toughness) × your size**

Small folk ([halflings](races.md#race-halfling) and [gnomes](races.md#race-gnome)) carry
three quarters of what anyone else does:

{{#include generated/burden-races.md}}

The limit uses your scores, not the bonuses they give, so it grows evenly with them. Each point
of [Strength](stats.md#stat-strength) is worth 3 kg and each point of
[Toughness](stats.md#stat-toughness) 1.5 kg, or three quarters of that for the small. Some
examples:

{{#include generated/burden-examples.md}}

**Everything you carry counts**, including the armour you wear and the weapon in your hand
(see [Equipment](equipment.md)). Your own body does not. For scale, a leather jerkin weighs 3.5
kg, a chain mail shirt 11 kg, an iron helm 1.5 kg, a short sword 1.1 kg, and a small shield 2.5
kg (see [Objects](objects.md#the-objects)). An average [human](races.md#race-human) in mail and
a helm, with a sword and a shield, is [burdened](burden.md#burden-levels). A fighter in medium armour
usually is.

## Burden levels

Your **burden** compares what you carry with your limit. It rises by thirds of the limit:

{{#include generated/burden.md}}

- **Slower**: every action takes longer, not only steps. That includes waiting, opening doors,
  [fighting](combat.md#hitting), [picking things up](objects.md#picking-up), [putting armour
  on](equipment.md#how-long-it-takes), climbing [stairs](terrain.md#stairs), and journeys on the
  [overworld](overworld.md). A 20-minute road takes 25 minutes burdened. The
  [creatures](creatures.md) around you get that much more time to act (see [Turns and
  movement](turns.md)).
- **Less nimble**: your [Dexterity](stats.md#stat-dexterity) adds less to your
  [Defence](combat.md#defence). Heavy armour limits it too, and the lower of the two limits
  counts.
- **Louder**: your [footsteps](hearing.md#what-makes-noise) carry farther (see
  [Hearing](hearing.md)).
- **Overloaded**: you cannot move at all ("You carry too much to move."), not even a step or up
  or down the [stairs](terrain.md#stairs). Everything else still works, as slowly as when
  strained. Drop something and you can go on. You can always pick things up, even when that
  overloads you.

[Dwarves](races.md#race-dwarf) and [duergar](races.md#race-duergar) have carried loads all
their lives. Being burdened costs them nothing: no time, no
[Dexterity](stats.md#stat-dexterity), and no [noise](hearing.md#how-loud). From strained on,
they suffer like everyone else.

The [Balance](skills.md#skill-balance) skill takes away some of the slowdown. At 100, being
burdened slows you by a tenth instead of a quarter, and being strained by a fifth instead of a
half.

The **burden line**, three [talents](talents.md) of the Grit tree taken in order, eases all of
it a step at a time:

- [Pack mule](talents.md#talent-pack-mule) (Strength 11): you carry a tenth more, 60 % of the
  slowdown is gone (added to what [Balance](skills.md#skill-balance) takes away; together they
  can take it all), and each burden's limit on your [Dexterity](stats.md#stat-dexterity) in
  [Defence](combat.md#defence) is 1 higher.
- [Porter](talents.md#talent-porter) (Pack mule, Toughness 12): a tenth more again, the
  Dexterity limits 1 higher again, and being burdened costs you nothing, as it costs a dwarf
  nothing.
- [Beast of burden](talents.md#talent-beast-of-burden) (Porter, level 12): a tenth more again,
  and being strained costs you only what being burdened costs others.

Some classes are given the first steps: [commoners](classes.md#commoner) Pack mule at
level 1, [merchants](classes.md#merchant) at 6, and [thieves](classes.md#thief)
Pack mule and Porter at 25.

[Creatures](creatures.md) carry by the same rules, using their own
[Strength](stats.md#stat-strength), [Toughness](stats.md#stat-toughness), and size.

In a [heavy hall](special-rooms.md#room-heavy), a [special room](special-rooms.md), everyone's
limit is halved while they are inside, so the same load is a heavier burden there. You cannot
step into one if you would be overloaded inside, but you can always step out.

## What you see

- The [status bar](the-screen.md#the-screen) shows your burden when you have one
  (`Burdened`, `Strained`, `Overloaded`), right after your [hit
  points](hit-points.md#hit-points). On a narrow screen it is one of the last things to be
  dropped.
- A [message](the-screen.md#messages) tells you when it changes: "You are burdened.", "You
  strain under your load.", "You carry too much to move.", "You are no longer burdened."
- The [inventory](objects.md#your-pack) (`I`) and the [character screen](the-screen.md#character)
  (`@`) show what you carry, your burden if you have one, and the weights at which it changes:
  `Burdened over 15 kg, strained over 30 kg, overloaded over 45 kg`.

The game never shows you a [creature](creatures.md)'s burden or what it carries. Creatures'
carrying limits are listed on [Creatures in numbers](creature-numbers.md#pace-senses-and-size).
