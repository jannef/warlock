# Healing

This page answers "how do I get my health back?". Lost [hit points](hit-points.md#hit-points)
come back by themselves over time, and there are quicker ways too. What stops bleeding, poison,
and disease is in the second section.

## Getting your health back

- **Wait or rest.** Hit points come back by themselves, faster the tougher you are
  ([below](#how-fast-you-heal)). [Resting](resting.md) (`b`) is the quickest way to let that time pass,
  and you heal twice as fast while you rest.
- **Drink a potion.** A [potion of healing](potions.md#potion-of-healing), a
  [greater](potions.md#potion-of-greater-healing) one, or a
  [potion of full healing](potions.md#potion-of-full-healing) brings hit points back at once.
- **Cast a spell.** [Mend](magic.md#spell-mend), [Cure wounds](magic.md#spell-cure-wounds), and
  [Benison](magic.md#spell-benison) bring some back, and [Renewal](magic.md#spell-renewal) makes
  you heal four times as fast. On the undead they
  [harm instead](spell-rules.md#healing-harms-the-undead).
- **Pray.** A [priest](class-priest.md)'s prayer mends [wounds](prayer.md#prayer-wounds) (see
  [Prayer](prayer.md)), and a [paladin](class-paladin.md)'s
  [Lay on hands](abilities.md#ability-lay-on-hands) heals with zeal.
- **Catch your breath.** The [Second wind](abilities.md#ability-second-wind) talent gives hit
  points back at once.
- **Find a place that heals.** A [healing spring](altars-forges-springs.md#healing-springs) may
  heal you when you drink at it, and in a [sanctuary](special-rooms.md#room-sanctuary) wounds heal
  three times as fast.

## Curing bleeding, poison, and disease

Each of these slows or stops your healing until it ends, and each ends by itself in time
(see [Status effects](status-effects.md) and [Diseases](diseases.md)). To end one sooner:

- **[Bleeding](status-effects.md#bleeding)**: bind the wound with the
  [Heal](skill-uses.md#treating-yourself) skill, drink a greater healing potion or one of full
  healing, or a priest's prayer for [wounds](prayer.md#prayer-wounds).
- **[Poison](status-effects.md#poisoned)**: a [potion of antidote](potions.md#potion-of-antidote)
  or of full healing, the spell [Remove disease](magic.md#spell-remove-disease), the Heal skill (it
  draws out some of a dose), or a priest's prayer against [poison](prayer.md#prayer-poison).
- **[Disease](diseases.md#curing-a-disease)**: an angelica root, a
  [potion of cure disease](potions.md#potion-of-cure-disease), Remove disease, a temple's cure (see
  [Services](npcs.md#services)), or a priest's prayer against [disease](prayer.md#prayer-disease).
  The Heal skill eases one but never cures it.
- **Stunned, frightened, or drowsy**: a [potion of clear head](potions.md#potion-of-clear-head).
- **Everything else**: the "How it ends" column of the
  [status tables](status-list.md) lists every way to end each one.

Those born in the year of the Unicorn have [Horn's grace](abilities.md#ability-horns-grace), which
ends bleeding and poison and shakes off a disease.

## How fast you heal

At the usual pace you get a [hit point](hit-points.md#hit-points) back every so often. The tougher
you are, the faster it comes:

{{#include generated/regeneration.md}}

[Resting](resting.md#how-resting-works), [status effects](status-effects.md#how-they-work), and
starving (see [Hunger](hunger.md)) change the pace:

{{#include generated/healing.md}}

When several things change the pace at once:

- If several status effects slow your healing, only the slowest counts.
- An effect that speeds healing up ([Renewal](magic.md#spell-renewal)'s, four times the pace) counts
  only while nothing slows it.
- A [disease](diseases.md#how-a-disease-goes) that slows healing more than being
  [Diseased](status-list.md#status-diseased) does counts too (see [Diseases](diseases.md)).
- Resting makes up for some of it. Resting while
  [poisoned](status-effects.md#poisoned) heals at the usual pace.
- While you [bleed](status-effects.md#bleeding) or starve, nothing heals you, resting or not.
- In a [sanctuary](special-rooms.md#room-sanctuary) (a [special room](special-rooms.md)), wounds heal
  three times as fast on top of all that. This holds for anyone there, not just you.
- [Magic items](magic-items.md) that speed healing ("hit points come back 50% faster") add on top
  again.

## The fine print

- Healing runs on game time, whatever you are doing: waiting, walking, fighting, or making a long
  journey on the [overworld](overworld.md). Only the total time counts, not how it is split up.
- Progress toward the next [hit point](hit-points.md#hit-points) carries over from turn to turn. It is
  lost once you are unhurt.
- [Creatures](creatures.md) heal by the same rules, at their own [Toughness](stats.md#stat-toughness),
  while you are on their level. They never [rest](resting.md#how-resting-works).
- Some creatures **regenerate**. Most [trolls](creatures.md#creature-troll) heal four times as fast
  as their Toughness alone would give (the [creature numbers](creature-numbers.md) say which ones do).
  This counts as a status effect that speeds healing, so anything that slows healing slows theirs
  too. When you see one get a tenth of its hit points back at once, you are told: *The troll's
  wounds close.*
- **Fire and acid stop all healing** while they last. Nothing heals while
  [Burning](status-list.md#status-burning) or [Corroded](status-list.md#status-corroded): not a
  troll's regeneration, and not yours.

Your [mana](mana.md) comes back in the same way, paced by your [Willpower](stats.md#stat-willpower).

See also [Status effects](status-effects.md) and Hit points.
