# Animated objects

Things a will set moving and left behind: a candlestick that rocks after you, a teapot that
scalds, a broom that scuttles, a rug that wraps around your legs. They set on the living who
come near and leave everything else alone. The household ones are almost harmless, and worth
killing for what they are made of: a silver candlestick is a prize at any level. Deeper, a
sword, an axe, or a staff dances alone: an **animated blade**, quick and fragile, and its weapon
the prize. Deeper still walk the composites: **animated armour**, a suit with nobody inside that
fights with its gauntlets, and the **animated panoply**, the full kit of a knight with no knight,
wielding and wearing real magic items whose powers are its own. A panoply is among the most
dangerous things in the game, and among the most rewarding.

## What they are made of

An animated household thing *is* a real household object: an animated candlestick is a
candlestick, of brass, iron, pewter, or (rarely) silver. Its colour on the map is its
material's, and [look mode](controls.md#look-mode) names it: "an animated candlestick of
silver (hostile, unhurt), rocking". A clatter of cutlery is three to six pieces of cutlery at
once.

An animated blade is one real melee weapon, of a kind found at its level or below, which it
**wields** as anyone would: the weapon's damage, threat, and any magic on it (a flaming blade
burns), with the blade's Strength and a rank in the weapon's skill that grows with its level.
The deeper it is met, the likelier its weapon is magic, and from level 15 now and then rare. It
shows on the map as its weapon does, `)` in the colour of the weapon's material, and look mode
names what it wields as it would anyone's: "an animated blade (hostile, unhurt), wielding a
steel long sword". Like any weapon a monster carries, it is unidentified until you hold it. A
blade is hard to hit (its thin edge), shrugs off thrusts (piercing), and breaks under heavy
blows (bludgeoning); it floats without a sound.

Animated armour and a panoply are made of several real things at once: body armour always, a
helm, a shield, gauntlets, and for a panoply a melee weapon and, the deeper it is, boots, a cloak,
an amulet, and a ring on each hand (from level 50). Each piece is drawn as a blade's weapon is
and worn or wielded as anyone's: the armour's Protection, the shield's Defence, the weapon's
blows. A panoply that wields a two-handed weapon carries no shield. Both show on the map as `A`,
in the colour of their body armour's material; look mode names what they wear and wield: "an
animated armour (hostile, unhurt), wearing a bronze cap and a padded coat". Animated armour has no
weapon: it slams with its gauntlets. Both are slow and loud, open doors, and shrug off some of a
thrust (the armour, whose blows pass through the hollow). A panoply holds a rank in its weapon's
skill as a blade does, and beside a shield ranks in Sword and Board.

All blades are one kind of creature in every way that counts: you learn of animated blades once,
a quest to kill them counts every one, and they are drawn onto a floor as one kind, at the level
of the floor's danger (a blade of level 12 on a floor of danger 12, never below level 5 or above
20). So are all animated armours and all panoplies, each among themselves.

When one is destroyed, what it was made of falls **whole** half the time, as the object it was
(a silver candlestick you can sell). Otherwise it **breaks into the parts it was made of**: a
gem set in it falls loose, every other part falls as pieces of its material (a silver
candlestick as pieces of silver, still worth something), and paper and wax are lost. Whatever
magic it had is gone with it. The **Salvager** talent raises the share that falls whole. They
leave no corpse and never bleed.

## What they are

They are constructs: nothing lives in them to sicken, they have no blood or breath for poison,
and nothing in them bleeds, so they are never **diseased**, **poisoned**, or **bleeding**. They
can still be frightened (rarely: their will is not their own), held, staggered, or transfixed.
What they are made of decides what hurts them most: wood and cloth burn, metal takes lightning
badly, and glass, stone, and bone shatter under heavy blows. Nothing can be stolen from them:
what they carry is what they are.

The **Unbinder** talent strikes at the binding, not the thing: your blows against animated
things are surer and harder. **Appraiser's eye** tells a bound thing's worth at a glance: look
mode adds "It hums with power." when anything it wears or wields is magic or rare.

## What their magic does for them

Whatever an animated thing wears and wields works for it exactly as it would for you, rings and
amulets too: a Flaming sword burns, a ring of the Bear makes its blows harder, an amulet of the Ox
gives it more life, armour of the Salamander takes the edge off fire, a ring of Light makes it
glow (you see it coming). The gaps are few:

| Affix effect | Works for an animated thing? |
|---|---|
| Stat raises (of the Bear, of the Lynx, of the Ox, ...) | Yes: hit points, to hit, damage, Defence |
| To hit, damage, added damage, threat, criticals, parrying | Yes |
| Elemental damage (Flaming, Freezing, ...) | Yes |
| Attack speed, moving speed | Yes |
| Armour, shields, Dodge, Deflection, natural armour | Yes |
| Protection | Yes |
| Resistances | Yes |
| More hit points | Yes |
| Regeneration | Yes |
| Stealth (quieter steps) | Yes |
| Light | Yes: it glows |
| Skills | Yes, where a rule reads a creature's skill |
| Weight | Its things weigh less, which changes nothing: it carries itself |
| More mana, mana regeneration, caster level | Yes, as for any creature (see [Mana](mana.md)), though it casts no spells (yet) |
| Carrying | No, and it needs none |
| Range, draw, durability (missile weapons) | No: it wields melee weapons only |

## Where they are found

The barrow of Cealdhlāw keeps candlesticks, cutlery, teapots, and cooking pots among its grave
goods; the drowned minster of Fennmynster its candlesticks, books, lanterns, and rugs. Blades
dance in the barrow's lower floors and all through Fyrdelf, where the duergar smiths' work
wakes, and empty suits of armour walk Fyrdelf's deeper floors (they are of level 12 at least).
Now and then one strays elsewhere. No panoply walks anywhere in the Ascmearc: it waits for deeper
places.

## In numbers

Every animated type is generated from its tier and its body by the formulas below (rounded down
part by part): Strength 11 + 0.4 × level, Toughness 12 + 0.4 × level, Dexterity 10 + 0.25 ×
level, Willpower 16 + 0.35 × level, Intelligence and Presence 2 (they are mindless), then the
tier's adjustments; hit points beyond Toughness 2.5 × level × the tier's factor; damage dice the
nearest of 1d2, 1d3, … 5d12 to 2 + 0.55 × level, moved by the tier's steps. A blade has no dice
of its own (it strikes with its weapon); its speed is 1200 + 5 × level (at most 1600) and its
natural Defence 2 + level / 10.

{{#include generated/animated-tiers.md}}

### By level

Speed 1000 is a normal pace; a household thing moves at its body's own.

{{#include generated/animated-numbers.md}}

### Blades

A blade's weapon is drawn among the melee weapons found at its level or below, by how common
each is, then made **at least magic** by the chance below (and a magic one may come out rare, as
any find may), or **rare** outright by the second chance, at an item level equal to the blade's;
otherwise it is plain, of a material drawn as a monster's gear is. Its rank in the weapon's skill
is 1 + level / 10, at most 8.

Animated armour and a panoply draw each of their pieces the same way, slot by slot in the order
below: each line is tried at its chance, from the level it names, as many times as it says (a
ring for each hand); a slot already taken (the off hand beside a two-handed weapon) is passed
over. Animated armour's gauntlets strike with the dice of its level; its Strength is the role's.
A panoply's rank in Sword and Board beside a shield is level / 15.

{{#include generated/animated-families.md}}

### What they leave

What a creature of each family is made of, expected (a household thing is its one body), what of
it falls whole with the drop rule, and the chances that each piece is at least magic and rare.
The rest falls as its parts.

{{#include generated/animated-loot.md}}

### The household things

Speed 1000 is a normal pace. A thing's Protection, resistances, and death line come from the
material its object is first made of; one drawn of another material (a silver candlestick of a
brass kind) changes its colour and worth, never its numbers.

{{#include generated/animated-bodies.md}}

### By material

{{#include generated/animated-groups.md}}

### What they break into

What each household thing falls to when it breaks, by the material it is made of. The pieces
are half a part's weight, counted in pieces of that material (see
[pieces](objects.md#pieces)), one at least.

{{#include generated/animated-parts.md}}
