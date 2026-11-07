# Potions

A potion (`!`) is a draught in a glass flask. To drink one, press `e` to **consume**, the same
command that eats (see [Food](food.md)). The list titled "Consume what?" shows your
[food](food.md) and your potions together, grouped by kind: first what lies where you stand
("lying here"), then what you carry. Press a letter, or `Enter` on the marked row, to drink.

Drinking takes one [turn](time.md#what-takes-time), uses the potion up, and works at once: "You
drink the potion of healing. You feel your wounds close." You can drink a potion lying on the
floor without picking it up (in a [shop](shops.md#buying), that means you have bought it). You
can drink whether or not you are [hungry](hunger.md#how-hungry-you-are), and even while
[raging](status-effects.md#status-raging).

{{#include generated/potions.md}}

[Hit points](hit-points.md#hit-points) and [mana](mana.md#how-much) never go past your maximum.

### Potions that last

Some potions work **for a while** instead of at once:

- A **boost** raises a [stat](stats.md) by 2 for ten minutes. A greater boost raises it by 4,
  eases to 2 after ten minutes, and is gone ten minutes after that.
- A **[vision](vision.md#the-four-types)** potion lends you other eyes for half an hour. The
  potion of seeing lasts five minutes.
- The **potion of quickness** hastes you for a minute, as the [Haste](magic.md#spell-haste)
  spell does.
- A **resistance** potion makes you [resist](damage-types.md#armour-and-resistance) an element
  by 50 % for half an hour. It counts the way armour's resistance does, so together with what
  you wear it never reaches immunity: 75 % at most.
- A **helpful draught** lends you a [status effect](status-effects.md) a spell could also give:
  the potion of barkskin [Barkskin](status-effects.md#status-barkskin) and the potion of glamour
  [Glamoured](status-effects.md#status-glamoured) (both harder to hit, two minutes), the potion
  of renewal [Renewing](status-effects.md#status-renewing) (wounds close faster, four minutes),
  the potion of the quick [Quick-sensed](status-effects.md#status-quick-sensed) (you feel the
  living about you, three minutes), the potion of the unseen
  [Invisible](status-effects.md#status-invisible) (two minutes), and the potion of sea legs
  [Water-borne](status-effects.md#status-water-borne) (you walk on water, six minutes). A
  second one of the same kind starts its time over.

Each of these shows by name on the [status bar](the-screen.md#the-screen) and the [character
screen](the-screen.md#character): [Fortified](status-effects.md#status-fortified),
[Owl-eyed](status-effects.md#status-owl-eyed), [Warmed](status-effects.md#status-warmed), and so
on (see [Status effects](status-effects.md)). If you drink a second boost of the same stat while
one lasts, you keep the stronger of the two, and the time does not start over. A second vision
or resistance potion of the same kind starts its half hour over.

A potion's **marks** are [stat training marks](stats.md#training) that you always get when you
drink it (see [Stats](stats.md)): the body remembers the draught.

### Utility potions

Four potions do one useful thing at once:

- The **potion of sustenance** feeds you as a [food ration](food.md) does, 1500
  [nutrition](hunger.md#how-hungry-you-are), but never leaves you Bloated: what would go past
  that is lost, and you never choke or vomit on it.
- The **potion of restoration** does what the [scroll of
  restoration](scrolls.md#the-scroll-of-restoration) does: every point of [stat
  damage](diseases.md#stat-damage) is mended, every [drained](stats.md#drained-stats) point comes
  back, and [Stiffening](status-effects.md#status-stiffening) ends. Unlike the scroll, it is
  drunk even when there is nothing to restore ("You feel whole.").
- The **potion of blinking** moves you at once to a random free cell of the level, as a
  [teleport trap](traps.md#trap-teleport) does: anywhere, in sight or not, near danger or far
  from it. On the [overworld](overworld.md) it does nothing.
- The **potion of clear head** ends at once what clouds the mind or numbs the body:
  [Frightened](status-effects.md#status-frightened), [Drowsy](status-effects.md#status-drowsy),
  [Sluggish](status-effects.md#status-sluggish), [Stunned](status-effects.md#status-stunned),
  [Staggered](status-effects.md#status-staggered), and
  [Transfixed](status-effects.md#status-transfixed).

### Gambles

Three potions are a gamble even once you know them: each time you drink one, one outcome is
drawn by chance from its own table, as at a [healing spring](altars-forges-springs.md). It may
do good, nothing, or harm. The tables below give every outcome and its exact chance per
draught; the game only tells you what happens, in words.

- The **draught of chance** mostly heals you, clears your head, or lends you a stat
  [boost](#potions-that-last), but it may sicken you or make you drowsy.
- The **vial of troll's blood** always does two things at once: your wounds close faster
  ([Renewing](status-effects.md#status-renewing)) and you are
  [Poisoned](status-effects.md#status-poisoned).
- The **brewer's blunder** is a coin toss: a greater boost of one stat for a while, or a point
  of one stat [drained](stats.md#drained-stats) for good. A potion of restoration gives a
  drained point back.

{{#include generated/potion-gambles.md}}

## Unknown potions

A potion you have never drunk shows a **word and a colour** instead of its name: "a fizzy red
potion". Each kind of potion has its own word. The words are made up anew for each character
and stay the same for their whole life, so two fizzy red potions are always the same kind.

The colour belongs to the kind and never changes. It follows one rule: each family of effects
has a hue, and a potion with several effects mixes their hues.

| Family | Hue |
|---|---|
| [Healing](healing.md#how-fast) | red |
| Cures ([poison](damage-types.md#damage-poison), [disease](diseases.md#how-a-disease-goes), [a clouded mind](#utility-potions)) | green |
| Healing and cures together | russet, red mixed with green |
| [Mana](mana.md#how-much) | blue |
| Stat boosts | orange (greater ones deep orange) |
| [Vision](vision.md#the-four-types) | violet |
| [Speed](turns.md#speed) | yellow |
| [Resistance](damage-types.md#armour-and-resistance) | teal |
| Wards ([Barkskin](status-effects.md#status-barkskin), [Glamoured](status-effects.md#status-glamoured)) | bronze |
| [Renewal](status-effects.md#status-renewing) | rose |
| The unseen and the unplaced ([Quick-sensed](status-effects.md#status-quick-sensed), [Invisible](status-effects.md#status-invisible), [blinking](#utility-potions)) | opal |
| Utility ([Water-borne](status-effects.md#status-water-borne), [sustenance](#utility-potions)) | sky blue |
| [Restoration](#utility-potions) | mint |
| Smoke ([Fume](spellcasting.md#zones)) | ash grey |
| Light ([Lit](status-effects.md#status-lit), a [Flare](spellcasting.md#spells-with-a-rule-of-their-own)) | silver |
| [Acid](damage-types.md#damage-acid) | lime |
| [Gambles](#gambles) | iridescent |
| [Elixirs](#elixirs) (a [stat](stats.md) for good) | gold |
| Harm | brown |

So a red potion heals and a green one cures something. But two potions of one family may share
exactly the same colour: the colour tells you the family, never which potion it is.

You learn what a kind of potion is, for good, in any of these ways:

- by drinking one ("It is a potion of healing.");
- by reading the [scroll of identify](scrolls.md#the-scroll-of-identify) on it;
- by an appraisal (see [Services](npcs.md#services));
- because it was in your [starting kit](starting-kit.md).

Drinking an unknown potion is a gamble: the murky draught makes you sick. A [shop](shops.md#prices)
charges its true price for an unknown potion.

## Elixirs

An **elixir** is the rarest of potions. It raises one [stat](stats.md) by 1 for good, just as
[training](stats.md#training) a stat up does. It also raises that stat's
**[potential](stats.md#potentials)**, the ceiling training works toward (see [Stats](stats.md)),
by 1 or 2 more, with an even chance of each. If your score was already at its potential, the
potential first rises with the score. So after an elixir there is always room for at least one
more point of training, and half the time two.

The change takes effect at once. Your [hit points](hit-points.md#hit-points) follow a
[Toughness](stats.md#stat-toughness) elixir, your [mana](mana.md#how-much) an
[Intelligence](stats.md#stat-intelligence) or [Willpower](stats.md#stat-willpower) one, and your
[carrying limits](burden.md#how-much-you-can-carry) a [Strength](stats.md#stat-strength) one.
"You feel stronger." means the score rose. "You feel stronger, and that you could be stronger
yet." means the potential rose by 2. Each elixir also gives 60 [training marks](stats.md#training)
of its stat.

There are six elixirs, one for each stat, and all are gold:

| Elixir | Stat |
|---|---|
| might | [Strength](stats.md#stat-strength) |
| grace | [Dexterity](stats.md#stat-dexterity) |
| vigour | [Toughness](stats.md#stat-toughness) |
| wit | [Intelligence](stats.md#stat-intelligence) |
| will | [Willpower](stats.md#stat-willpower) |
| presence | [Presence](stats.md#stat-presence) |

They are found only deep ([item level](affixes.md#base-items) 8 and below), and seldom. A
[forgotten altar](altars-forges-springs.md#forgotten-altars)'s great boon may leave one, and an
alchemist very rarely has one for 2000 [gold](shops.md#money). A [general store](shops.md) or a
provisioner never does.

## Flasks

Any potion can be **[thrown](ranged.md#shooting-and-throwing)**: `T` lists your potions beside
your throwing weapons. A thrown potion never lands whole. It **shatters** where it comes down.

A **flask** is a potion made for throwing, such as the sleeping draught. Where it shatters, it
works as a [spell](magic.md#the-spells) does: the sleeping draught breathes the
[Sleep](magic.md#spell-sleep) spell over everyone around the cell. The table's **Thrown** column
gives each flask's spell, its numbers, and its range in cells.

- **Where it breaks**: a flask breaks on the first creature in the line from you to the cell you
  aim at, or before a wall, or on that cell. There is no roll to [hit](combat.md#hitting) and no
  miss: what matters is who stands where.
- **How far**: a flask flies its own range (6 cells for most), plus what
  [talents](talents.md) and [magic items](affixes.md) add to your throws. Any other potion flies
  only 3 cells.
- **What it does**: the spell works as if you had cast it, but your [stats](stats.md) and talents
  do not change it: a flask is always as strong as the table says. Its harm and its kills are
  yours, [experience](experience.md) too. A burst that harms can catch you if you throw it too
  close, and the game asks first. A flask that lays a [Drowsy](status-effects.md#status-drowsy)
  cloud or another [status](status-effects.md) spares you. Throwing a flask takes a throw's time
  and trains no [weapon skill](weapon-skills.md).
- **Learning it**: if you see the cell where an unknown flask breaks, you learn what it was ("It
  was a sleeping draught."). Thrown into the dark or round a corner, it is only heard, as "a crash
  of glass" ([Hearing](hearing.md#what-makes-noise)), and you learn nothing.
- **A potion that is no flask** spills uselessly: "The potion shatters; the draught soaks away."
  You learn nothing from it either. It is a way to be rid of a potion you know is bad.
- **Smoke**: a flask of smoke leaves a cloud of [smoke](spellcasting.md#zones) no eye sees into or
  through, yours and every creature's: a way past archers, or out of a fight.
- **Acid**: a vial of acid burns, keeps eating, and eats at the armour of anyone it harms, yours
  too if you stand in it ([Acid](spellcasting.md#spell-types)).
- **Drinking a flask** is a poor idea: the sleeping draught makes you drowsy, as the table says.
- **A flask of holy water** is a kind one: thrown, it casts [Rebuke](magic.md#spell-rebuke),
  which sears and frightens only the undead and harms nothing living; drunk, it washes a hex off
  ([Hexed](status-effects.md#status-ill-wished) and [Branded](status-effects.md#status-branded)
  end at once).
- **Thunder and blasting**: a flask of thunder goes off as [Thunderclap](magic.md#spell-thunderclap),
  heard over the whole level ([Hearing](hearing.md#what-makes-noise)); a blasting flask as
  [Shatter](magic.md#spell-shatter), which breaks shut doors and chests. A general store or a
  provisioner never sells either: only an alchemist does.
- **A flask of brilliance** is the kind one among the drinkable: drunk, it makes you shed light about you a while
  ([Lit](status-effects.md#status-lit)).

## Where to find them

Potions lie about as [loot](magic-items.md#finding-them). The common ones (healing, antidote)
turn up more often and from the first depths, the better ones deeper down. General stores and
provisioners sell a few among their wares. An **alchemist** sells only potions, always potions
of healing first, and buys them too (see [Shops](shops.md)). Some [quests](quests.md) pay in
potions.

## Talents

- [Thrifty drinker](talents.md#talent-thrifty-drinker) (Delver): one potion in four is not used
  up, elixirs included. Items that keep potions add their percent to the talent's chance, up to
  50 % in all.
- [Lasting draughts](talents.md#talent-lasting-draughts) (Vigour, needs
  [Toughness](stats.md#stat-toughness) 13): boost and vision potions last half again as long:
  15 minutes a degree for a boost, 45 minutes for cat's, owl's, or moth's eyes.
