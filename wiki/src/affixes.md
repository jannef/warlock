# Prefixes and suffixes

A [magic item](magic-items.md) is a base item with a prefix, a suffix, or both. This page lists the base items and every prefix and suffix, and says how an item's name is put together.

## Base items

{{#include generated/magic-bases.md}}

## Prefixes

Prefixes are mostly offensive, or give flat [Defence](combat.md#defence) and life.

- *Flaming*, *Freezing*, *Venomous*, *Shocking*, *Corrosive*, and *Withering* add a die of
  [fire](damage-types.md#damage-fire), [cold](damage-types.md#damage-cold),
  [poison](damage-types.md#damage-poison), [electricity](damage-types.md#damage-electricity),
  [acid](damage-types.md#damage-acid), or [necrotic](damage-types.md#damage-necrotic) damage to
  every hit, which no armour soaks (see [Damage types](damage-types.md)).
- *Mage-wrought*, *Baleful*, and *Potent* serve a caster (see [Magic](magic.md)). *Potent* adds 1
  to 3 more to every [spell](spellcasting.md#how-a-spell-grows)'s roll, harm or healing.
- *Tireless* raises your [stamina](resources.md#resource-stamina) (see [Resources](resources.md)).
- *Hobnailed* [boots](objects.md#class-boots) [kick doors](doors.md#kicking) in more easily.
- A *Delving* tool or pair of gloves [digs](digging.md#digging) faster. Magic pick-axes do turn up
  in loot.

{{#include generated/magic-prefixes.md}}

## Suffixes

Suffixes are mostly [stats](stats.md), [speed](turns.md#speed), [criticals](combat.md#criticals),
utility, and [skills](skills.md).

- **Resistance**: *of the Salamander*, *of Warmth*, *of the Adder*, *of the Storm*, *of Glass*,
  *of the Anvil*, and *of the Living* [resist](damage-types.md#armour-and-resistance)
  [fire](damage-types.md#damage-fire), [cold](damage-types.md#damage-cold),
  [poison](damage-types.md#damage-poison), [electricity](damage-types.md#damage-electricity),
  [acid](damage-types.md#damage-acid), [force](damage-types.md#damage-force), and
  [necrotic](damage-types.md#damage-necrotic) damage, in percent (see [Damage
  types](damage-types.md)).
- **Casters**: *of Thrift*, *of the Magus*, *of Farcasting*, *of Lingering*, *of Siphoning*, and
  *of Wands* are explained on the [Magic](magic.md) and [Mana](mana.md) pages. *Of Wands* makes the
  [wands](wands-and-rods.md#wands) you use cast a rung or two higher.
- **Stamina**: *of Breath*, *of Ease*, and *of the Hunt* help
  [stamina](resources.md#resource-stamina).
- **Class resources**: *of Fury*, *of Steadiness*, *of Stillness*, *of Guile*, and *of the Muse*
  raise the most [rage](resources.md#rage), [focus](resources.md#resource-focus),
  [ki](resources.md#resource-ki), [openings](resources.md#resource-openings), or
  [inspiration](resources.md#resource-inspiration) you can hold. Only
  [berserkers](classes.md#berserker), [rangers](classes.md#ranger),
  [ascetics](classes.md#ascetic), [thieves](classes.md#thief), and [bards](classes.md#bard) hold
  those (see [Resources](resources.md)), so to anyone else they are no use.
- *Of Siphoning* and *of the Hunt* give their [mana](mana.md#mana-on-a-kill) or
  [stamina](resources.md#resource-stamina) only for a kill made by your own blow, shot, or spell,
  with the slain within three cells.
- **Other uses**: *of Sustenance* slows [hunger](hunger.md#how-hungry-you-are), *of Wariness*
  softens [traps](traps.md#finding-traps), *of Health* helps against
  [disease](diseases.md#how-a-disease-goes), and *of the Alchemist* sometimes leaves a
  [potion](potions.md) undrunk.

The skill suffixes raise one [skill](skills.md) each:

| Suffix | Skill |
|---|---|
| of Seeking | [Search](skills.md#skill-search) |
| of the Eagle | [Spot](skills.md#skill-spot) |
| of the Stag | [Athletics](skills.md#skill-athletics) |
| of the Leech | [Heal](skills.md#skill-heal) |
| of the Picklock | [Open lock](skills.md#skill-open-lock) |
| of Disarming | [Disable device](skills.md#skill-disable-device) |
| of Appraisal | [Appraise](skills.md#skill-appraise) |
| of Haggling | [Negotiation](skills.md#skill-negotiation) |
| of the Scholar | [Spellcraft](skills.md#skill-spellcraft) |
| of Letters | [Decipher script](skills.md#skill-decipher-script) |
| of the Cutpurse | [Steal](skills.md#skill-steal) |

*Of Shadows*, *of the Hare*, *of the Fox*, *of the Wanderer*, and *of Balance* raise skills too.

[Spot](skills.md#skill-spot), [Open lock](skills.md#skill-open-lock), [Disable
device](skills.md#skill-disable-device), [Heal](skills.md#skill-heal),
[Steal](skills.md#skill-steal), and [Appraise](skills.md#skill-appraise) give nothing at all to
those not trained in them (they are marked **trained only** on the Skills page). Their suffixes
help only those trained in them: an untrained wearer still cannot notice, pick, disarm, treat,
or steal.

{{#include generated/magic-suffixes.md}}

Each tier's value is rolled once, when the item is made, anywhere within its range. An item
remembers each affix's tier and value, so that enchanting can work on them later.

## What systems have affixes

Every affix by the part of the game it works in, with the [talents](talents.md) that work in the
same place. An affix's value adds to a talent's, so a worn item and a talent stack within the
same limits.

{{#include generated/magic-systems.md}}

## Names

A magic item is called by its prefix and suffix around its base: "Sharp long sword", "long sword
of the Bear", "Sharp long sword of the Bear". A rare item gets a name of two words, the second
fitting what it is, followed by its base: "Grim Bite, long sword".

A gem the item is set with (see [Gems](gems.md#items-set-with-a-gem)) goes between the prefix and
the base, "Sharp emerald-pommelled long sword of the Bear", and shows even when the item is
unidentified. A rare item keeps its two words and the gem goes with its base: "Grim Bite,
emerald-pommelled long sword". An item of a material other than its usual one (see
[Materials](materials.md#items-of-other-materials)) names that material after the gem: "Sharp
emerald-pommelled bronze long sword of the Bear".

{{#include generated/magic-names.md}}
