# Potions

A potion (`!`) is a draught in a glass flask. Press `e` to **consume**, the same command that
eats (see [Food](food.md)): the list titled "Consume what?" shows your [food](food.md) and your potions together, grouped by
kind, first what lies where you stand ("lying here"), then what you carry. A letter, or `Enter`
on the marked row, drinks it. Drinking takes one [turn](time.md#what-takes-time), uses the potion up, and works at once:
"You drink the potion of healing. You feel your wounds close." You can drink a potion lying on
the floor without picking it up (in a [shop](shops.md#buying), you have bought it), and you can drink whether or
not you are [hungry](hunger.md#how-hungry-you-are), and while [raging](status-effects.md#status-raging).

{{#include generated/potions.md}}

[Hit points](combat.md#hit-points) and [mana](mana.md#how-much) never go past your maximum. Some potions work **for a while** instead: a
boost raises a [stat](stats.md) by 2 for ten minutes (a greater one by 4, easing to 2 after ten minutes,
then gone ten minutes later), a [vision](vision.md#the-four-types) potion lends you other eyes for half an hour, the potion
of quickness hastes you for a minute as the [Haste](magic.md#spell-haste) spell does, and a resistance potion makes you
[resist](damage-types.md#armour-and-resistance) an element by 50 % for half an hour, counted as armour's resistance is, so together with
what you wear it never reaches immunity (at most 75 %). Each shows on the [status bar](controls.md#the-screen) and the
[character screen](controls.md#character) by name ([Fortified](status-effects.md#status-fortified), [Owl-eyed](status-effects.md#status-owl-eyed), [Warmed](status-effects.md#status-warmed), ...); see
[Status effects](status-effects.md). A second boost of the same stat while one lasts keeps the
stronger of the two and does not start the time over; a second vision or resistance potion of
the same kind starts its half hour over. A potion's **marks** are [stat training marks](stats.md#training),
given for sure as you drink it (see [Stats](stats.md)): the body remembers the draught.

## Unknown potions

A potion you have never drunk shows by a **word and a colour** instead of its name: "a fizzy
red potion". Each kind of potion has its own word, made up anew for each character and kept for
their whole life (two fizzy red potions are always the same kind). The colour is the kind's own
and never changes, and it follows one rule: each family of effects has a hue, and a potion of
several effects mixes their hues.

| Family | Hue |
|---|---|
| [Healing](healing.md#how-fast) | red |
| Cures ([poison](damage-types.md#damage-poison), [disease](diseases.md#how-a-disease-goes)) | green |
| Healing and cures together | russet, red mixed with green |
| [Mana](mana.md#how-much) | blue |
| Stat boosts | orange (greater ones deep orange) |
| [Vision](vision.md#the-four-types) | violet |
| [Speed](turns.md#speed) | yellow |
| [Resistance](damage-types.md#armour-and-resistance) | teal |
| [Elixirs](#elixirs) (a [stat](stats.md) for good) | gold |
| Harm | brown |

So a red potion heals and a green one cures something, but two potions of one family may share
a colour exactly: the colour tells you the family, never which potion it is.

You learn what a kind of potion is, for good, by drinking one ("It is a potion of healing."),
by reading the [scroll of identify](scrolls.md#the-scroll-of-identify) on it, by an appraisal (see [Services](npcs.md#services)), or
because it was in your [starting kit](starting-kit.md). Drinking an unknown potion is a gamble: the murky draught
makes you sick. A [shop](shops.md#prices)'s price for an unknown potion is its true price.

## Elixirs

An **elixir** is the rarest of potions: it raises one [stat](stats.md) by 1 for good, as a stat [trained](stats.md#training) up
does, and its **[potential](stats.md#potentials)** (the ceiling training works toward, see [Stats](stats.md)) by 1 or
2 more at random, an even chance of each. Where your score was already at its potential, the
potential rises with the score first, so after an elixir there is always room for at least one
more point of training, and half the time two. Your [hit points](combat.md#hit-points) follow a [Toughness](stats.md#stat-toughness) elixir at
once, your [mana](mana.md#how-much) an [Intelligence](stats.md#stat-intelligence) or [Willpower](stats.md#stat-willpower) one, your [carrying limits](burden.md#how-much-you-can-carry) a [Strength](stats.md#stat-strength) one. "You feel
stronger." says the score rose; "You feel stronger, and that you could be stronger yet." says
the potential rose by 2. Each elixir also gives 20 [training marks](stats.md#training) of its stat.

There are six, one a stat: the elixirs of **might** ([Strength](stats.md#stat-strength)), **grace** ([Dexterity](stats.md#stat-dexterity)),
**vigour** ([Toughness](stats.md#stat-toughness)), **wit** ([Intelligence](stats.md#stat-intelligence)), **will** ([Willpower](stats.md#stat-willpower)), and **presence**
([Presence](stats.md#stat-presence)), all gold. They lie only deep ([item level](magic-items.md#base-items) 8 and below) and seldom, a [forgotten
altar](special-rooms.md#forgotten-altars)'s great boon may leave one, and an alchemist very rarely has one at 2000 [gold](shops.md#money); a [general
store](shops.md) or a provisioner never does.

## Where to find them

Potions lie about as [loot](magic-items.md#finding-them), the commoner ones (healing, antidote) more often and from the first
depths, the better ones deeper. General stores and provisioners sell a few among their wares;
an **alchemist** sells potions only, always potions of healing first, and buys them (see
[Shops](shops.md)). Some [quests](quests.md) pay in potions.

## Talents

- **[Thrifty drinker](talents.md#talent-thrifty-drinker)** (Delver): one potion in four is not used up, an elixir too. Items that
  keep potions add their percent to the talent's chance, up to 50 % in all.
- **[Quick study of the body](talents.md#talent-quick-study-of-the-body)** (Scholar): a potion's or an elixir's training marks are doubled.
- **[Lasting draughts](talents.md#talent-lasting-draughts)** (Vigour, needs [Toughness](stats.md#stat-toughness) 13): boost and vision potions last half again
  as long (15 minutes a degree for a boost, 45 minutes for a vision potion).
- **[Cauldron-wise](talents.md#talent-cauldron-wise)** (the [witch](classes.md#witch)'s, at [level](experience.md#levels) 12): healing potions heal a fifth more (rounded
  down), and boosts last a fifth longer (12 minutes a degree); with Lasting draughts, 17
  minutes a degree.
- **[Nose for draughts](talents.md#talent-nose-for-draughts)** (Scholar): an unknown potion you pick up tells you its family by its
  smell, never its kind: "The fizzy red potion smells of honey and iron: healing of some kind."
