# Potions

A potion (`!`) is a draught in a glass flask. Press `e` to **consume**, the same command that
eats: the list titled "Consume what?" shows your food and your potions together, grouped by
kind, first what lies where you stand ("lying here"), then what you carry. A letter, or `Enter`
on the marked row, drinks it. Drinking takes one turn, uses the potion up, and works at once:
"You drink the potion of healing. You feel your wounds close." You can drink a potion lying on
the floor without picking it up (in a shop, you have bought it), and you can drink whether or
not you are hungry, and while raging.

{{#include generated/potions.md}}

Hit points and mana never go past your maximum. A potion's **marks** are stat training marks,
given for sure as you drink it (see [Stats](stats.md)): the body remembers the draught.

## Unknown potions

A potion you have never drunk shows by a **word and a colour** instead of its name: "a fizzy
red potion". Each kind of potion has its own word, made up anew for each character and kept for
their whole life (two fizzy red potions are always the same kind). The colour is the kind's own
and never changes, and it follows one rule: each family of effects has a hue, and a potion of
several effects mixes their hues.

| Family | Hue |
|---|---|
| Healing | red |
| Cures (poison, disease) | green |
| Healing and cures together | russet, red mixed with green |
| Mana | blue |
| Harm | brown |

So a red potion heals and a green one cures something, but two potions of one family may share
a colour exactly: the colour tells you the family, never which potion it is.

You learn what a kind of potion is, for good, by drinking one ("It is a potion of healing."),
by reading the scroll of identify on it, by an appraisal (see [Services](npcs.md#services)), or
because it was in your starting kit. Drinking an unknown potion is a gamble: the murky draught
makes you sick. A shop's price for an unknown potion is its true price.

## Where to find them

Potions lie about as loot, the commoner ones (healing, antidote) more often and from the first
depths, the better ones deeper. General stores and provisioners sell a few among their wares;
an **alchemist** sells potions only, always potions of healing first, and buys them (see
[Shops](shops.md)). Some quests pay in potions.

## Talents

- **Thrifty drinker** (Delver): one potion in four is not used up.
- **Nose for draughts** (Scholar): an unknown potion you pick up tells you its family by its
  smell, never its kind: "The fizzy red potion smells of honey and iron: healing of some kind."
