# Magic items

Some of what you find is magic, after Path of Exile: a **base item** (a long sword, an iron helm,
a silver ring) with **prefixes** and **suffixes** that add to what it does. This is a first pass:
the lists and numbers below will change after playtesting.

## Rarity

| Rarity | Affixes | Name | Shown in |
|---|---|---|---|
| Normal | none | "long sword" | white |
| Magic | a [prefix](#prefixes), a [suffix](#suffixes), or one of each (most have one) | "Sharp long sword of the Bear" | blue |
| Rare | three to six affixes (most often three), at most three prefixes and three suffixes | "Grim Bite, long sword" | yellow |

The color is the name's, in your pack, on the equipment screen, and in an item's description; the
item's glyph keeps its own color. [Unique items](#unique-items) are rares too.

How often loot is normal, magic, or rare, and how many affixes magic and rare items have:

{{#include generated/magic-rarity.md}}

How many of a rare item's affixes are [prefixes](#prefixes) is drawn evenly among the splits that keep at most
three of each: three affixes are none to three prefixes, a quarter each; four are one, two, or
three prefixes, a third each; five are two or three, a half each; six are always three and three.

## Finding them

- **On the floor** of every level of a generated dungeon (see [Dungeons and danger](dungeons.md)) lie a few items of loot (now and
  then a [scroll](scrolls.md), which is never magic).
- **Vaults** hold a reward: the guard post's store room something magic, the shrine and the
  roofless chapel something.
- **Creatures** may drop something magic or rare where they die, besides what they carried: a
  [kobold](creatures.md#creature-kobold) now and then, a [kobold archer](creatures.md#creature-kobold-archer) a little more often, a [kobold shaman](creatures.md#creature-kobold-shaman) one time in two (see
  [Creatures](creatures.md)).
- Loot is sometimes set with a gem, and now and then a find is a loose gem instead (see
  [Gems](gems.md#finding-gems)).

The deeper you are, the better the loot: an item's **item level** is the depth it was found at
(a dungeon's first level is item level 1, the eighth 8). What a creature drops has the
creature's level as its item level where that is higher, so a creature [out of its depth](dungeons.md#who-lives-there-and-how-many) drops
a deeper item. The game never shows an item's level. The item level
decides which base items can turn up and which **tiers** of each affix: low tiers are found from
the start, the strongest only deep down, and an item rolls among the tiers close below its own
level, so a deep find is seldom a weak one (the exact rule is under the rarity tables above).

## Unidentified items

Magic and rare items are **unidentified** when you find them. You see their rarity ("a long sword
(magic)", "an iron helm (rare)"), not their affixes or a rare item's name. Their magic works all
the same while you wear or wield them, as in ADOM and Path of Exile: an unknown [ring](objects.md#class-rings) of [Strength](stats.md#stat-strength)
makes you stronger, and one that glows lights your way. Wearing it for a while does not tell you
what it is. A [scroll of identify](scrolls.md#the-scroll-of-identify) will, and so does carrying
it for a day if you have the [talent](talents.md) [Lore](talents.md#talent-lore), and so does Factor Wulfrun in Cēaptūn,
for 100 [gold](shops.md#money) per unidentified [stack](objects.md#piles) you carry ([Services](npcs.md#services)); once identified, an
item stays known
wherever it goes: "This is a Sharp long sword of the Bear!"

## What their magic does

Magic works while the item is **[worn or wielded](equipment.md#what-fits-where)**, not while it is merely carried.

- **On a weapon, a missile weapon, or [ammunition](ammunition.md)**, attack magic ([to hit](combat.md#hitting), [damage](combat.md#damage), [criticals](combat.md#criticals),
  attack speed, [range](ranged.md#shooting-and-throwing)) changes the attacks made **with that item**: a Sharp sword hits harder, not
  your bow. A bow's magic and its arrows' add up for a shot.
- **On anything else** ([gloves](objects.md#class-gloves), [rings](objects.md#class-rings), [amulets](objects.md#class-amulets), a [helm](objects.md#class-helmets)), attack magic changes **every attack**
  you make, bare hands and thrown rocks included.
- Magic that changes the item itself (its [Armour](combat.md#defence), [Protection](combat.md#protection), [Shield](combat.md#defence), or [Parry](combat.md#defence), how much
  [Dexterity](stats.md#stat-dexterity) it lets add to your [Defence](combat.md#defence), its weight) is the item's own.
- Everything else is yours: [stats](stats.md), [hit points](combat.md#hit-points), [Dodge](combat.md#defence) and [Deflection](combat.md#defence) and natural armour, how fast
  your wounds [heal](healing.md#how-fast), how fast you [move](turns.md#speed), how much you can [carry](burden.md#how-much-you-can-carry), how quietly you [walk](hearing.md#what-makes-noise), [light](light.md#what-light-is), and
  [skills](skills.md): "+20 to Stealth" works as if your [Stealth](skills.md#skill-stealth) were 20 higher, even if you
  never trained it (the [Skills screen](controls.md#skills) still shows what you have trained).

A few rules from elsewhere carry over:

- **Stats** raised by an item count everywhere a stat does: [Strength](stats.md#stat-strength) [to hit](combat.md#hitting) and in your [carrying
  limit](burden.md#how-much-you-can-carry), [Toughness](stats.md#stat-toughness) in your [hit points](combat.md#hit-points), [Dexterity](stats.md#stat-dexterity) in your [Defence](combat.md#defence). The [character
  screen](character.md) shows your stats with what you wear. A [status effect](status-effects.md)
  that lowers a stat lowers it from there: [poisoned](status-effects.md#status-poisoned) while wearing a ring of Strength, you are
  still stronger than without the ring.
- **[Defence](combat.md#defence)**: within Armour, Shield, Parry, Deflection, and natural armour only the best counts;
  Dodge adds up (see [Combat](combat.md)). **[Protection](combat.md#protection)** always adds up, piece by piece, and a
  [Hardened](talents.md#talent-hardened) piece of armour adds its magic to its own (see
  [Combat](combat.md#protection)).
- **[Hit points](combat.md#hit-points)** and **[mana](mana.md#how-much)**: putting on something that adds to them raises your maximum, not
  what you have; taking it off lowers both, if need be (see [Mana](mana.md)).
- **Healing**: magic that speeds healing adds on top of everything else that changes its pace
  (see [Healing](healing.md)); nothing heals you while you [bleed](status-effects.md#bleeding), magic or not.
- **Throwing**: a magic weapon [readied](ranged.md#getting-ready) in the Missile weapon slot keeps its magic when you [throw](ranged.md#shooting-and-throwing)
  it, and still has it where it lands. Its [Parry](combat.md#defence) counts only while you hold it in a hand.
- **[Speed](turns.md#speed)** is never a number in the game: things "attack faster" or make you "move faster".
  Exactly, each takes its share off the time of every attack or step (the tables below), the
  shares of everything you wear and wield adding up, to at most half the time (twice as quick).
- **[Light](light.md#light-sources)**: a glowing item makes its wearer carry a faint, steady light (the brightest one
  counts); it never flickers.
- **Categories**: every affix has a category (the tables below name it), and an item never has
  two affixes of one category. Most categories are one effect each, so an item has at most one
  affix of each kind of effect; *stealth* holds a family (of Stealth and of Shadows). [Damage](combat.md#damage) is
  several categories, so a weapon may carry more than one damage affix: *physical damage* (Sharp
  and Vicious, a flat bonus), *extra damage* (Heavy and of Wounding, an extra die), and one per
  element (Flaming, Freezing, Venomous, Shocking, Corrosive, Withering): a Sharp sword may be Heavy too, or Flaming, but never
  both Heavy and of Wounding. A prefix and a suffix of one category do the same thing (Glinting
  and of Precision both add to hit), so an item has one or the other.

## Looking at an item

In your [pack](objects.md#your-pack) (`I`), press an item's letter to look at it; on the [equipment screen](equipment.md#the-equipment-screen) (`i`), press a
slot's letter in upper case (`A` to `M`). The description names the item, says what it is
(rarity, base, every part it is made of, weight: "Magic long sword · steel blade, leather grip ·
1.8 kg") and whether you wear or wield it, then its base stats in words (how it is held or worn,
its [damage](combat.md#damage) and [criticals](combat.md#criticals), how far it shoots, what it needs, whether it pierces armour, its
[Defence](combat.md#defence) and [Protection](combat.md#protection)), what it gives in numbers ("To hit +1, damage 1d8+2", "Dv +2, Pv 3"; see
[Equipment](equipment.md#numbers)), and, if it is identified, each affix in plain words (the item
level is never shown in the game): "Prefix Sharp: +2 damage", "Suffix of Celerity: attacks faster". `Esc` goes back.

## Base items

{{#include generated/magic-bases.md}}

## Prefixes

Prefixes are mostly offensive, or flat [Defence](combat.md#defence) and life. *Flaming*, *Freezing*, *Venomous*,
*Shocking*, *Corrosive*, and *Withering* add a die of [fire](damage-types.md#damage-fire), [cold](damage-types.md#damage-cold), [poison](damage-types.md#damage-poison), [electricity](damage-types.md#damage-electricity), [acid](damage-types.md#damage-acid), or
[necrotic](damage-types.md#damage-necrotic) damage to every hit, which no armour soaks (see [Damage types](damage-types.md)).
*Mage-wrought*, *Baleful*, and *Potent* serve a caster (see [Magic](magic.md)), *Tireless* raises
your [stamina](resources.md#resource-stamina) (see [Resources](resources.md)), *Hobnailed* [boots](objects.md#class-boots) [kick doors](doors.md#kicking) in more easily, and a
*Delving* tool or pair of gloves [digs](digging.md#digging) faster (a magic pick-axe turns up in loot).

{{#include generated/magic-prefixes.md}}

## Suffixes

Suffixes are mostly [stats](stats.md), [speed](turns.md#speed), [criticals](combat.md#criticals), utility, and [skills](skills.md); *of the Salamander*, *of
Warmth*, *of the Adder*, *of the Storm*, *of Glass*, *of the Anvil*, and *of the Living* [resist](damage-types.md#armour-and-resistance)
[fire](damage-types.md#damage-fire), [cold](damage-types.md#damage-cold), [poison](damage-types.md#damage-poison), [electricity](damage-types.md#damage-electricity), [acid](damage-types.md#damage-acid), [force](damage-types.md#damage-force), and [necrotic](damage-types.md#damage-necrotic) damage, in percent (see [Damage
types](damage-types.md)). The casters'
suffixes (of the Adept, of Thrift, of the Magus, of Farcasting, of Lingering, of Siphoning, of
Wands) are told on the [Magic](magic.md) and [Mana](mana.md) pages; of Breath, of Ease, and of the
Hunt help [stamina](resources.md#resource-stamina). *Of Fury*, *of Steadiness*, *of Stillness*, *of Guile*, and *of the Muse* raise
the most [rage](resources.md#rage), [focus](resources.md#resource-focus), [ki](resources.md#resource-ki), [openings](resources.md#resource-openings), or [inspiration](resources.md#resource-inspiration) you can hold: only [barbarians](classes.md#barbarian), [rangers](classes.md#ranger), [monks](classes.md#monk),
[thieves](classes.md#thief), and [bards](classes.md#bard) hold those (see [Resources](resources.md)), so to anyone else they are of no
use. *Of Siphoning* and *of the Hunt* give their [mana](mana.md#mana-on-a-kill) or [stamina](resources.md#resource-stamina) only for a kill of your own blow,
shot, or spell, the slain within three cells. *Of Sustenance* slows [hunger](hunger.md#how-hungry-you-are), *of Wariness* softens
[traps](traps.md#finding-traps), *of Health* helps against [disease](diseases.md#how-a-disease-goes), and *of the Alchemist* sometimes leaves a [potion](potions.md)
undrunk.

The skill suffixes raise one [skill](skills.md) each: *of Seeking* ([Search](skills.md#skill-search)), *of the Eagle*
([Spot](skills.md#skill-spot)), *of the Stag* ([Athletics](skills.md#skill-athletics)), *of the Leech* ([Heal](skills.md#skill-heal)), *of the Picklock* ([Open lock](skills.md#skill-open-lock)), *of
Disarming* ([Disable device](skills.md#skill-disable-device)), *of Appraisal* ([Appraise](skills.md#skill-appraise)), *of Haggling* ([Negotiation](skills.md#skill-negotiation)), *of the Scholar* ([Spellcraft](skills.md#skill-spellcraft)), *of
Letters* ([Decipher script](skills.md#skill-decipher-script)), *of the Cutpurse* ([Steal](skills.md#skill-steal)), besides *of Shadows*, *of the Hare*, *of
the Fox*, *of the Wanderer*, and *of Balance*. [Spot](skills.md#skill-spot), [Open lock](skills.md#skill-open-lock), [Disable device](skills.md#skill-disable-device), [Heal](skills.md#skill-heal), [Steal](skills.md#skill-steal), and [Appraise](skills.md#skill-appraise)
give nothing to those not trained in them at all (marked **trained only** on the Skills page),
so their suffixes help only those trained in them: an untrained wearer still cannot notice,
pick, disarm, treat, or steal.

{{#include generated/magic-suffixes.md}}

Each tier's value is rolled once, when the item is made, anywhere within its range; an item
remembers each affix's tier and value (so enchanting can work on them later).

## What systems have affixes

Every affix by the part of the game it works in, and the [talents](talents.md) that work at the same place:
an affix's value adds to theirs (or, for [Attuned](talents.md#talent-attuned) and [Bellows](talents.md#talent-bellows), is raised by them), so a worn item
and a talent stack within the same limits.

{{#include generated/magic-systems.md}}

## Names

A magic item is called by its prefix and suffix around its base: "Sharp long sword", "long sword
of the Bear", "Sharp long sword of the Bear". A rare item gets a name of two words, the second
fitting what it is, and is called by its name and its base: "Grim Bite, long sword". A gem the
item is set with (see [Gems](gems.md#items-set-with-a-gem)) goes between the prefix and the base,
"Sharp emerald-pommelled long sword of the Bear", and shows unidentified too; a rare item keeps
its two words and the gem follows in its base, "Grim Bite, emerald-pommelled long sword". An
item of another material than its usual one (see [Objects](objects.md#items-of-other-materials))
names it after the gem: "Sharp emerald-pommelled bronze long sword of the Bear".

{{#include generated/magic-names.md}}

## Unique items

A unique item is one of a kind: a [rare](#rarity) item with a name, affixes, and an [item level](#base-items) set by hand,
and a **power**, a rule of its own that no affix gives. It is found unidentified like any rare
item ("a stiletto (rare)"); identified, it shows its name, its affixes, its power in words, and a
line of lore. Each lies in one place in the world (most are carried by those who own them), so a
character finds each at most once; loot never makes one. Its power works while it is worn or
wielded, identified or not:

- **Inflict**: a hit with it that leaves its target alive causes a [status](status-effects.md#how-they-work), every time or by a
  chance (the status's resistance applies, as to a poisonous bite). A bow's power rides its
  arrows; a kick or the other hand's weapon has none of it.
- **Vision**: its wearer sees with a [vision](vision.md#the-four-types) profile where it is better than their own eyes.
- **Ward**: its wearer never suffers a [status](status-effects.md#how-they-work).

{{#include generated/uniques.md}}
