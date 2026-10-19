# Burden

Everything you carry weighs something, and the more you carry, the harder it is to move, fight,
and keep quiet. As in ADOM, a heavy load slows **everything** you do.

## How much you can carry

Your carrying **limit** comes from your [Strength](stats.md), which counts double, and your
Toughness, which keeps you going under a load all day:

**Limit = 1.5 kg × (2 × Strength + Toughness) × your size**

Small folk (halflings and gnomes) carry three quarters of what anyone else does:

{{#include generated/burden-races.md}}

It follows your scores, not the bonuses they give, so it grows evenly with them: each point of
Strength is worth 3 kg, each point of Toughness 1.5 kg (three quarters of that for the small).
Some characters:

{{#include generated/burden-examples.md}}

**Everything you carry counts**: worn armour and the weapon in your hand weigh what they weigh
(see [Equipment](equipment.md)). Your own body does not. For scale, a leather jerkin weighs 3.5
kg, a chain mail shirt 11 kg, an iron helm 1.5 kg, a short sword 1.1 kg, and a small shield 2.5
kg (see [Objects](objects.md#the-objects)). An average human in mail and a helm, with a sword and
a shield, is burdened, as a fighter in medium armour usually is.

## Burden

Your **burden** is what you carry against your limit, by thirds:

{{#include generated/burden.md}}

- **Slower**: burden makes every action take longer, not only steps: waiting, opening doors,
  fighting, picking things up, putting armour on, climbing stairs, and journeys on the
  [overworld](overworld.md) alike (a 20-minute road takes 25 minutes burdened). The creatures
  around you get that much more time to act (see [Turns and movement](turns.md)).
- **Less nimble**: your [Dexterity](stats.md) adds less to your [Defence](combat.md#defence).
  Heavy armour limits it too; the lower limit counts.
- **Louder**: your footsteps carry farther (see [Hearing](hearing.md)).
- **Overloaded**, you cannot move at all ("You carry too much to move."): not a step, not up or
  down the stairs. Everything else still works, as slowly as when strained: drop something, and
  you can go on. Picking things up is always allowed, even when it overloads you.

**Dwarves** have carried loads all their lives: being burdened costs them nothing, no time, no
Dexterity, no noise. From strained on, they suffer like everyone else.

The [balance](skills.md#what-skills-give) skill takes some of the slowdown away: at 100, a burden
slows you by a tenth instead of a quarter, and strained by a fifth instead of a half.

Creatures carry by the same rules, from their own Strength, Toughness, and size.

## What you see

- The **status bar** shows your burden when you have one (`Burdened`, `Strained`, `Overloaded`),
  right after your hit points; it is one of the last things to give way on a narrow screen.
- A **message** says when it changes: "You are burdened.", "You strain under your load.", "You
  carry too much to move.", "You are no longer burdened."
- The **inventory** (`i`) and the **character screen** (`@`) show what you carry, your burden
  if you have one, and the weights at which it changes:
  `Burdened over 15 kg, strained over 30 kg, overloaded over 45 kg`.

You never see a creature's burden or what it carries.
