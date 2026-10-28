# Burden

Everything you carry weighs something, and the more you carry, the harder it is to move, fight,
and keep [quiet](hearing.md#how-loud). As in ADOM, a heavy load slows **everything** you do.

## How much you can carry

Your carrying **limit** comes from your [Strength](stats.md#stat-strength), which counts double, and your
[Toughness](stats.md#stat-toughness), which keeps you going under a load all day:

**Limit = 1.5 kg × (2 × Strength + Toughness) × your size**

Small folk ([halflings](races.md#race-halfling) and [gnomes](races.md#race-gnome)) carry three quarters of what anyone else does:

{{#include generated/burden-races.md}}

It follows your scores, not the bonuses they give, so it grows evenly with them: each point of
[Strength](stats.md#stat-strength) is worth 3 kg, each point of [Toughness](stats.md#stat-toughness) 1.5 kg (three quarters of that for the small).
Some characters:

{{#include generated/burden-examples.md}}

**Everything you carry counts**: worn armour and the weapon in your hand weigh what they weigh
(see [Equipment](equipment.md)). Your own body does not. For scale, a leather jerkin weighs 3.5
kg, a chain mail shirt 11 kg, an iron helm 1.5 kg, a short sword 1.1 kg, and a small shield 2.5
kg (see [Objects](objects.md#the-objects)). An average [human](races.md#race-human) in mail and a helm, with a sword and
a shield, is [burdened](burden.md#burden-1), as a fighter in medium armour usually is.

## Burden

Your **burden** is what you carry against your limit, by thirds:

{{#include generated/burden.md}}

- **Slower**: burden makes every action take longer, not only steps: waiting, opening doors,
  [fighting](combat.md#hitting), [picking things up](objects.md#picking-up), [putting armour on](equipment.md#how-long-it-takes), climbing [stairs](terrain.md#stairs), and journeys on the
  [overworld](overworld.md) alike (a 20-minute road takes 25 minutes burdened). The [creatures](creatures.md)
  around you get that much more time to act (see [Turns and movement](turns.md)).
- **Less nimble**: your [Dexterity](stats.md#stat-dexterity) adds less to your [Defence](combat.md#defence).
  Heavy armour limits it too; the lower limit counts.
- **Louder**: your [footsteps](hearing.md#what-makes-noise) carry farther (see [Hearing](hearing.md)).
- **Overloaded**, you cannot move at all ("You carry too much to move."): not a step, not up or
  down the [stairs](terrain.md#stairs). Everything else still works, as slowly as when strained: drop something, and
  you can go on. Picking things up is always allowed, even when it overloads you.

**[Dwarves](races.md#race-dwarf)** have carried loads all their lives: being burdened costs them nothing, no time, no
[Dexterity](stats.md#stat-dexterity), no [noise](hearing.md#how-loud). From strained on, they suffer like everyone else.

The [balance](skills.md#skill-balance) skill takes some of the slowdown away: at 100, a burden
slows you by a tenth instead of a quarter, and strained by a fifth instead of a half.

[Creatures](creatures.md) carry by the same rules, from their own [Strength](stats.md#stat-strength), [Toughness](stats.md#stat-toughness), and size.

In a **[heavy hall](special-rooms.md#room-heavy)** (a [special room](special-rooms.md)) everyone's limit is halved while they
are inside: the same load is a heavier burden there. You cannot step into one if you would be
overloaded inside, but you can always step out.

## What you see

- The **[status bar](controls.md#the-screen)** shows your burden when you have one (`Burdened`, `Strained`, `Overloaded`),
  right after your [hit points](combat.md#hit-points); it is one of the last things to give way on a narrow screen.
- A **[message](controls.md#messages)** says when it changes: "You are burdened.", "You strain under your load.", "You
  carry too much to move.", "You are no longer burdened."
- The **[inventory](objects.md#your-pack)** (`I`) and the **[character screen](controls.md#character)** (`@`) show what you carry, your burden
  if you have one, and the weights at which it changes:
  `Burdened over 15 kg, strained over 30 kg, overloaded over 45 kg`.

You never see a [creature](creatures.md)'s burden or what it carries; creatures' carrying limits are on
[Creatures in numbers](creature-numbers.md#pace-senses-and-size).
