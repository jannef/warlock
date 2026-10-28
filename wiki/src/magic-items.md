# Magic items

Some of what you find is magic, after Path of Exile: a **base item** (a long sword, an iron helm,
a silver ring) with **prefixes** and **suffixes** that add to what it does. This is a first pass:
the lists and numbers below will change after playtesting.

## Rarity

| Rarity | Affixes | Name | Shown in |
|---|---|---|---|
| Normal | none | "long sword" | white |
| Magic | a prefix, a suffix, or one of each (most have one) | "Sharp long sword of the Bear" | blue |
| Rare | three to six affixes (most often three), at most three prefixes and three suffixes | "Grim Bite, long sword" | yellow |

The color is the name's, in your pack, on the equipment screen, and in an item's description; the
item's glyph keeps its own color. [Unique items](#unique-items) are rares too.

How often loot is normal, magic, or rare, and how many affixes magic and rare items have:

{{#include generated/magic-rarity.md}}

How many of a rare item's affixes are prefixes is drawn evenly among the splits that keep at most
three of each: three affixes are none to three prefixes, a quarter each; four are one, two, or
three prefixes, a third each; five are two or three, a half each; six are always three and three.

## Finding them

- **On the floor** of every level of a generated dungeon (see [Dungeons and danger](dungeons.md)) lie a few items of loot (now and
  then a [scroll](scrolls.md), which is never magic).
- **Vaults** hold a reward: the guard post's store room something magic, the shrine and the
  roofless chapel something.
- **Creatures** may drop something magic or rare where they die, besides what they carried: a
  kobold now and then, a kobold archer a little more often, a kobold shaman one time in two (see
  [Creatures](creatures.md)).
- Loot is sometimes set with a gem, and now and then a find is a loose gem instead (see
  [Gems](gems.md#finding-gems)).

The deeper you are, the better the loot: an item's **item level** is the depth it was found at
(a dungeon's first level is item level 1, the eighth 8). It
decides which base items can turn up and which **tiers** of each affix: low tiers are found from
the start, the strongest only deep down.

## Unidentified items

Magic and rare items are **unidentified** when you find them. You see their rarity ("a long sword
(magic)", "an iron helm (rare)"), not their affixes or a rare item's name. Their magic works all
the same while you wear or wield them, as in ADOM and Path of Exile: an unknown ring of Strength
makes you stronger, and one that glows lights your way. Wearing it for a while does not tell you
what it is. A [scroll of identify](scrolls.md#the-scroll-of-identify) will, and so does carrying
it for a day if you have the [talent](talents.md) Lore, and so does Factor Wulfrun in Cēaptūn,
for 100 gold per unidentified stack you carry ([Services](npcs.md#services)); once identified, an
item stays known
wherever it goes: "This is a Sharp long sword of the Bear!"

## What their magic does

Magic works while the item is **worn or wielded**, not while it is merely carried.

- **On a weapon, a missile weapon, or ammunition**, attack magic (to hit, damage, criticals,
  attack speed, range) changes the attacks made **with that item**: a Sharp sword hits harder, not
  your bow. A bow's magic and its arrows' add up for a shot.
- **On anything else** (gloves, rings, amulets, a helm), attack magic changes **every attack**
  you make, bare hands and thrown rocks included.
- Magic that changes the item itself (its Armour, Protection, Shield, or Parry, how much
  Dexterity it lets add to your Defence, its weight) is the item's own.
- Everything else is yours: stats, hit points, Dodge and Deflection and natural armour, how fast
  your wounds heal, how fast you move, how much you can carry, how quietly you walk, light, and
  [skills](skills.md): "+20 to Stealth" works as if your Stealth were 20 higher, even if you
  never trained it (the Skills screen still shows what you have trained).

A few rules from elsewhere carry over:

- **Stats** raised by an item count everywhere a stat does: Strength to hit and in your carrying
  limit, Toughness in your hit points, Dexterity in your Defence. The [character
  screen](character.md) shows your stats with what you wear. A [status effect](status-effects.md)
  that lowers a stat lowers it from there: poisoned while wearing a ring of Strength, you are
  still stronger than without the ring.
- **Defence**: within Armour, Shield, Parry, Deflection, and natural armour only the best counts;
  Dodge adds up (see [Combat](combat.md)). **Protection** always adds up, piece by piece, and a
  Hardened piece of armour adds its magic to its own (see
  [Combat](combat.md#protection)).
- **Hit points** and **mana**: putting on something that adds to them raises your maximum, not
  what you have; taking it off lowers both, if need be (see [Mana](mana.md)).
- **Healing**: magic that speeds healing adds on top of everything else that changes its pace
  (see [Healing](healing.md)); nothing heals you while you bleed, magic or not.
- **Throwing**: a magic weapon readied in the Missile weapon slot keeps its magic when you throw
  it, and still has it where it lands. Its Parry counts only while you hold it in a hand.
- **Speed** is never a number in the game: things "attack faster" or make you "move faster".
  Exactly, each takes its share off the time of every attack or step (the tables below), the
  shares of everything you wear and wield adding up, to at most half the time (twice as quick).
- **Light**: a glowing item makes its wearer carry a faint, steady light (the brightest one
  counts); it never flickers.
- **Categories**: every affix has a category (the tables below name it), and an item never has
  two affixes of one category. Most categories are one effect each, so an item has at most one
  affix of each kind of effect; a few hold a family: *physical damage* (Sharp, Vicious, and
  Heavy: a Sharp sword is never Heavy too) and *stealth* (of Stealth and of Shadows). A prefix
  and a suffix of one category do the same thing (Glinting and of Precision both add to hit), so
  an item has one or the other.

## Looking at an item

In your pack (`I`), press an item's letter to look at it; on the equipment screen (`i`), press a
slot's letter in upper case (`A` to `L`). The description names the item, says what it is
(rarity, base, every part it is made of, weight: "Magic long sword · steel blade, leather grip ·
1.8 kg") and whether you wear or wield it, then its base stats in words (how it is held or worn,
its damage and criticals, how far it shoots, what it needs, whether it pierces armour, its
Defence and Protection), what it gives in numbers ("To hit +1, damage 1d8+2", "Dv +2, Pv 3"; see
[Equipment](equipment.md#numbers)), and, if it is identified, its item level and each affix in plain
words: "Prefix Sharp: +2 damage", "Suffix of Celerity: attacks faster". `Esc` goes back.

## Base items

{{#include generated/magic-bases.md}}

## Prefixes

Prefixes are mostly offensive, or flat Defence and life. *Flaming*, *Freezing*, and *Venomous*
add a die of fire, cold, or poison to every hit, which no armour soaks (see [Damage types](damage-types.md)).

{{#include generated/magic-prefixes.md}}

## Suffixes

Suffixes are mostly stats, speed, criticals, utility, and skills; *of the Salamander* and *of
Warmth* resist fire and cold, in percent (see [Damage types](damage-types.md)).

{{#include generated/magic-suffixes.md}}

Each tier's value is rolled once, when the item is made, anywhere within its range; an item
remembers each affix's tier and value (so enchanting can work on them later).

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

A unique item is one of a kind: a rare item with a name, affixes, and an item level set by hand,
and a **power**, a rule of its own that no affix gives. It is found unidentified like any rare
item ("a stiletto (rare)"); identified, it shows its name, its affixes, its power in words, and a
line of lore. Each lies in one place in the world (most are carried by those who own them), so a
character finds each at most once; loot never makes one. Its power works while it is worn or
wielded, identified or not:

- **Inflict**: a hit with it that leaves its target alive causes a status, every time or by a
  chance (the status's resistance applies, as to a poisonous bite). A bow's power rides its
  arrows; a kick or the other hand's weapon has none of it.
- **Vision**: its wearer sees with a vision profile where it is better than their own eyes.
- **Ward**: its wearer never suffers a status.

{{#include generated/uniques.md}}
