# Magic items

Some of what you find is magic. A magic item is a **base item** (a long sword, an iron helm, a
silver ring) with **prefixes** and **suffixes** that add to what it does, much as in Path of
Exile. The lists and numbers below are not final and will change after playtesting. Every base item, prefix, and suffix is listed on
[Prefixes and suffixes](affixes.md).

## Rarity

| Rarity | Affixes | Name | Shown in |
|---|---|---|---|
| Normal | none | "long sword" | white |
| Magic | a [prefix](affixes.md#prefixes), a [suffix](affixes.md#suffixes), or one of each (most have one) | "Sharp long sword of the Bear" | blue |
| Rare | three to six affixes (most often three), at most three prefixes and three suffixes, fewer if the item cannot bear them ([Capacity](#capacity)) | "Grim Bite, long sword" | yellow |

The color is the color of the item's name, in your pack, on the equipment screen, and in its
description. The item's glyph keeps its own color. [Unique items](#unique-items) are rares too.

How often loot is normal, magic, or rare, and how many affixes magic and rare items have:

{{#include generated/magic-rarity.md}}

How many of a rare item's affixes are [prefixes](affixes.md#prefixes) is picked evenly among the splits
that keep at most three of each:

- three affixes: none to three prefixes, a quarter each;
- four affixes: one, two, or three prefixes, a third each;
- five affixes: two or three prefixes, a half each;
- six affixes: always three and three.

## Capacity

Every item can bear only so many **enchantments** (affixes, [prefixes](affixes.md#prefixes) and
[suffixes](affixes.md#suffixes) together): one to six. It is the sum of:

- **2** for any item;
- **+1** for a ring or an amulet (they are made to carry a charm);
- **+1** for a Fine item, **+2** for a Marvellous one, **+3** for a Masterwork one (see
  [Forges](altars-forges-springs.md#forges) and [Crafting](crafting.md#how-well-it-comes-out));
  a damaged item counts as plain, and stays so once mended until it is made finer again;
- its main material's **binding**, −2 to +2 (the [materials table](materials.md) lists it:
  silver −2, tin and flint −1, steel and iron 0, gold +1, mithril and dragonbone +2);
- **+1** for a [gem](gems.md) set in it, **+2** for a flawless one.

It is never less than one or more than six. Some examples:

| Item | Can bear |
|---|---:|
| a steel long sword | 2 |
| a Fine steel long sword, or one with a ruby pommel | 3 |
| a Marvellous bronze scimitar | 4 |
| an iron ring | 3 |
| a gold ring | 4 |
| a gold ring set with an emerald | 5 |
| a silver ring | 1 |
| a Fine mithril long sword with a sapphire pommel | 6 |
| a Masterwork great sword with a flawless diamond pommel | 6 |

**Every item is made to fit.** Loot, what creatures carry, and the game's own named items never
hold more than they can bear. A find that rolls more loses the extra, one at a time: one from
whichever of its prefixes or suffixes it has more of, picked at random, a suffix when they are
even. A rare that loses some stays rare and keeps its name, so a rare on a plain steel sword has
just two affixes. The rares with more are rings and amulets, gem-set and finer items, and the
precious materials.

How it works out for what you find: nearly nine finds in ten can bear two enchantments, so about
87 % of rares keep two affixes, 10 % to 12 % keep three or more, and only 7 % to 8 % keep all they
rolled. A magic item (one or two affixes) almost always fits: about 87 % lose nothing.

Only a damaged Fine, Marvellous, or Masterwork item can bear less than it holds. It keeps all its
magic, which works as before.

The game tells you what an item can bear only if you are trained in
[Appraise](skills.md#skill-appraise) or have the craftsperson's talent
[Eye for the make](classes.md#eye-for-the-make): its description then says "It can bear two
enchantments.", or how many more it could bear.

## Finding them

- **On the floor** of every level of a generated dungeon (see [Dungeons and danger](dungeons.md))
  lie a few items of loot. Now and then one is a [scroll](scrolls.md), which is never magic.
- **Vaults** hold a reward of the floor's depth. The guard post's store room holds something
  magic, the shrine and the roofless chapel something, and the flooded cell coins, a
  [gem](gems.md), a ring, or an amulet.
- **Creatures** may drop something magic or rare where they die, besides what they carried. A
  [kobold](creatures.md#creature-kobold) does now and then, a [kobold
  archer](creatures.md#creature-kobold-archer) a little more often, and a [kobold
  shaman](creatures.md#creature-kobold-shaman) one time in two (see [Creatures](creatures.md)).
- Loot is sometimes set with a gem, and now and then a find is a loose gem instead (see
  [Gems](gems.md#finding-gems)).

The deeper you are, the better the loot. An item's **item level** is the depth it was found at:
a dungeon's first level is item level 1, the eighth is 8. What a creature drops has the
creature's level as its item level when that is higher, so a creature [out of its
depth](dungeons.md#who-lives-there-and-how-many) drops a deeper item. The game never shows an
item's level.

The item level decides which base items can turn up and which **tiers** of each affix. Low tiers
are found from the start, and the strongest only deep down. An item rolls among the tiers close
below its own level, so a deep find is seldom a weak one. The exact rule is under the rarity
tables above.

## Unidentified items

Magic and rare items are **unidentified** when you find them. You see their rarity ("a long
sword (magic)", "an iron helm (rare)"), but not their affixes or a rare item's name.

Their magic works all the same while you wear or wield them. An unknown
[ring](objects.md#class-rings) of [Strength](stats.md#stat-strength) makes you stronger, and one
that glows lights your way. Wearing it for a while does not tell you what it is. These do:

- a [scroll of identify](scrolls.md#the-scroll-of-identify);
- a [merchant](classes.md#merchant)'s [Merchant's eye](talents.md#talent-merchants-eye), from
  level 12: a careful look that takes as long as reading a scroll;
- carrying the item for a day, if you were born to [Old lore](talents.md#talent-old-lore);
- Factor Wulfrun in Cēaptūn, for 100 [gold](shops.md#money) per unidentified
  [stack](objects.md#piles) you carry (see [Services](npcs.md#services)).

Once identified, an item stays known wherever it goes: "This is a Sharp long sword of the
Bear!"

## What their magic does

Magic works while the item is **[worn or wielded](equipment.md#what-fits-where)**, not while it is
merely carried.

- **On a weapon, a missile weapon, or [ammunition](ammunition.md)**, attack magic ([to
  hit](combat.md#hitting), [damage](combat.md#damage), [criticals](combat.md#criticals), attack
  speed, [range](ranged.md#shooting-and-throwing)) changes only the attacks made **with that
  item**. A Sharp sword hits harder, but your bow does not. A bow's magic and its arrows' add up
  for a shot.
- **On anything else** ([gloves](objects.md#class-gloves), [rings](objects.md#class-rings),
  [amulets](objects.md#class-amulets), a [helm](objects.md#class-helmets)), attack magic changes
  **every attack** you make, bare hands and thrown rocks included.
- Magic that changes the item itself belongs to the item: its [Armour](combat.md#defence),
  [Protection](combat.md#protection), [Shield](combat.md#defence), or [Parry](combat.md#defence),
  how much [Dexterity](stats.md#stat-dexterity) it lets add to your [Defence](combat.md#defence),
  and its weight.
- Everything else is yours: [stats](stats.md), [hit points](hit-points.md#hit-points),
  [Dodge](combat.md#defence), [Deflection](combat.md#defence), and natural armour, how fast your
  wounds [heal](healing.md#how-fast), how fast you [move](turns.md#speed), how much you can
  [carry](burden.md#how-much-you-can-carry), how quietly you [walk](hearing.md#what-makes-noise),
  [light](light.md#what-light-is), and [skills](skills.md). "+20 to Stealth" works as if your
  [Stealth](skills.md#skill-stealth) were 20 higher, even if you never trained it. The [Skills
  screen](the-screen.md#skills) still shows only what you have trained.

Rules from elsewhere in the game carry over:

- **Stats** raised by an item count everywhere a stat does:
  [Strength](stats.md#stat-strength) [to hit](combat.md#hitting) and in your [carrying
  limit](burden.md#how-much-you-can-carry), [Toughness](stats.md#stat-toughness) in your [hit
  points](hit-points.md#hit-points), [Dexterity](stats.md#stat-dexterity) in your
  [Defence](combat.md#defence). The [character screen](character.md) shows your stats with what
  you wear. A [status effect](status-effects.md) that lowers a stat lowers it from there: if you
  are [poisoned](status-effects.md#status-poisoned) while wearing a ring of Strength, you are still
  stronger than you would be without the ring.
- **[Defence](combat.md#defence)**: among Armour, Shield, Parry, Deflection, and natural armour,
  only the best counts. Dodge adds up (see [Combat](combat.md)).
  **[Protection](combat.md#protection)** always adds up, piece by piece, and a
  [Hardened](talents.md#talent-hardened) piece of armour adds its magic to its own (see
  [Combat](combat.md#protection)).
- **[Hit points](hit-points.md#hit-points)** and **[mana](mana.md#how-much)**: putting on
  something that adds to them raises your maximum, not what you have now. Taking it off lowers
  both, if need be (see [Mana](mana.md)).
- **Healing**: magic that speeds healing adds on top of everything else that changes its pace
  (see [Healing](healing.md)). Nothing heals you while you [bleed](status-effects.md#bleeding),
  magic or not.
- **Throwing**: a magic weapon [readied](ranged.md#getting-ready) in the Missile weapon slot keeps
  its magic when you [throw](ranged.md#shooting-and-throwing) it, and still has it where it lands.
  Its [Parry](combat.md#defence) counts only while you hold it in a hand.
- **[Speed](turns.md#speed)** is never a number in the game. Things "attack faster" or make you
  "move faster". In exact terms, each takes its share off the time of every attack or step (see
  the tables below). The shares of everything you wear and wield add up, to at most half the
  time, which is twice as quick.
- **[Light](light.md#light-sources)**: a glowing item gives its wearer a faint, steady light that
  never flickers. Only the brightest one counts.

### Categories

Every affix has a category, named in the tables below, and an item never has two affixes of the
same category. Most categories hold one effect, so an item has at most one affix of each kind of
effect. The *stealth* category holds a family: of Stealth and of Shadows.

[Damage](combat.md#damage) is split into several categories, so a weapon may carry more than one
damage affix:

- *physical damage*: Sharp and Vicious, a flat bonus;
- *extra damage*: Heavy and of Wounding, an extra die;
- one category per element: Flaming, Freezing, Venomous, Shocking, Corrosive, Withering.

So a Sharp sword may also be Heavy, or Flaming, but never both Heavy and of Wounding. A prefix
and a suffix of the same category do the same thing (Glinting and of Precision both add to hit),
so an item has one or the other.

## Looking at an item

To look at an item, press its letter in your [pack](objects.md#your-pack) (`I`), or press a
slot's letter in upper case (`A` to `M`) on the [equipment
screen](equipment.md#the-equipment-screen) (`i`). The description shows, in this order:

- the item's name;
- what it is: rarity, base, every part it is made of, and weight ("Magic long sword · steel
  blade, leather grip · 1.8 kg"), and whether you wear or wield it;
- its base stats in words: how it is held or worn, its [damage](combat.md#damage) and
  [criticals](combat.md#criticals), how far it shoots, what it needs, whether it pierces armour,
  and its [Defence](combat.md#defence) and [Protection](combat.md#protection);
- what it gives in numbers ("To hit +1, damage 1d8+2", "Dv +2, Pv 3"; see
  [Equipment](equipment.md#numbers));
- if it is identified, each affix in plain words: "Prefix Sharp: +2 damage", "Suffix of
  Celerity: attacks faster". The item level is never shown in the game.

`Esc` goes back.

## Unique items

A unique item is one of a kind: a [rare](#rarity) item whose name, affixes, and [item
level](affixes.md#base-items) are set by hand. It also has a **power**, a rule of its own that no affix
gives.

It is found unidentified like any rare item ("a stiletto (rare)"). Once identified, it shows its
name, its affixes, its power in words, and a line of lore. Each one lies in one place in the
world, so a character finds each at most once, and loot never makes one. Most are carried by
those who own them. Three are only ever offered, very rarely, by a
[pedlar](npcs.md#the-pedlars-offer).

No [item damage](materials.md#protected-items) ever harms it.

Its power works while it is worn or wielded, identified or not:

- **Inflict**: a hit with it that leaves its target alive causes a
  [status](status-effects.md#how-they-work), either every time or by a chance. The status's
  resistance applies, as it does to a poisonous bite. A bow's power rides its arrows. A kick, or
  the weapon in your other hand, has none of it.
- **Vision**: its wearer sees with a [vision](vision.md#the-four-types) profile wherever that is
  better than their own eyes.
- **Ward**: its wearer never suffers a [status](status-effects.md#how-they-work).

{{#include generated/uniques.md}}
