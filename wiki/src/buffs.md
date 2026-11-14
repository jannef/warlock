# Buffs, hexes, and cooldowns

Not every [status effect](status-effects.md) costs hit points. A status effect can help or hinder
in other ways while it lasts. The "What it does" column of [the tables](status-list.md) says
which of these each one does:

- **Stats.** It may raise a score as well as lower one. If a higher
  [Toughness](stats.md#stat-toughness) let you [heal](healing.md#how-fast-you-heal) to more hit points,
  those extra hit points go again when it ends.
- **Fighting.** It may add to your [to-hit](combat.md#hitting) and [damage](combat.md#damage), or
  to your [Dodge](combat.md#defence) and Parry, as a
  [combat stance](stances.md#stances) does, on top of your stance. Creatures can suffer these too.
- **Blows.** It may add damage of some [kind](damage-types.md) to your melee blows with either
  hand, as a [Flaming](affixes.md#prefixes) sword does. [Brand](magic.md#spell-brand) adds 1d4
  [fire](damage-types.md#damage-fire) ([burning blade](status-list.md#status-branded-blade)). A kick or a shield
  bash does not carry it. No [spell type's rule](spell-types.md) applies to a blow, so
  it never leaves anyone [burning](status-effects.md#burning).
- **Striking back.** It may hurt whoever lands a melee blow on you, as a spell of that kind would.
  [Fire cloak](magic.md#spell-fire-cloak) does 1d6 fire to the attacker
  ([fire-cloaked](status-list.md#status-fire-cloaked)), and one time in four leaves them burning. What it kills
  counts as your kill.
- **Water.** It may bear you up. [Water walking](magic.md#spell-water-walking)
  ([water-borne](status-list.md#status-water-borne)) makes [wading](terrain.md#water) cost no more than a step on
  dry land. [Ice](temperature.md#walking-on-ice) never gives way under you, however thin: a burst
  of heat or force leaves the ice you stand on, so you never [fall in](temperature.md#falling-in).
- **Slowness.** It may make everything you do take longer: at 150%, half again as long.
- **What you cannot do.** It may stop you doing something while it lasts. A
  [raging](status-list.md#status-raging) fighter cannot read ("You cannot read while raging."). Trying takes no
  time, and an [ability](abilities.md) that would do it is greyed out: `you are raging`.
- **What follows.** As some end, another begins: when a [rage](rage.md) subsides,
  [exhaustion](status-list.md#status-exhausted) follows.
- **Eyes.** It may lend you other eyes. A [potion of owl's eyes](potions.md#potion-of-owls-eyes)
  lets you see in the dark as [darkvision](vision.md#the-four-types) does, in each respect where it
  is better than your own sight.
- **Resistance.** It may make you [resist](damage-types.md#armour-and-resistance) a kind of
  damage. This counts like armour's resistance: together with what you wear it
  never goes past 75%, and never gives immunity.

## From potions and songs

The [potions](potions.md) that last a while give:

- boosts: [Fortified](status-list.md#status-fortified), [Nimble](status-list.md#status-nimble), [Hale](status-list.md#status-hale),
  [Keen](status-list.md#status-keen), [Steadfast](status-list.md#status-steadfast), and [Imposing](status-list.md#status-imposing), each
  a [stat](stats.md) +2 per degree;
- vision: [Cat-eyed](status-list.md#status-cat-eyed), [Owl-eyed](status-list.md#status-owl-eyed), and
  [Moth-eyed](status-list.md#status-moth-eyed);
- resistances: [Warmed](status-list.md#status-warmed), [Chill-proof](status-list.md#status-chill-proof),
  [Earthed](status-list.md#status-earthed), and [Proof against poison](status-list.md#status-poison-proof).

The bard's [songs](abilities.md#songs) give three more. [Inspired](status-list.md#status-inspired) (and a
[virtuoso](class-bard.md#talent-virtuoso)'s greater inspired) lifts the singer.
[Drowsy](status-list.md#status-drowsy), from the [lullaby](abilities.md#ability-lullaby), slows those who fail to
shake it off. Each goes up to three degrees, as many as the singer's [Music](skills.md#skill-music)
gives the song.

## Hexes

A **hex** is a status that a [hex spell](spellcasting.md#shapes) lays: [Hexed](status-list.md#status-ill-wished),
[Weakened](status-list.md#status-weakened), [Sluggish](status-list.md#status-sluggish), Drowsy, and
[Webbed](status-list.md#status-webbed). A [witch](class-witch.md) can feed her hexes:

- Her [cackle](abilities.md#ability-cackle) makes each hex she laid on a creature she sees grow a
  degree, up to its maximum, and starts its degree's time again.
- Her [familiar](familiars.md#the-touch)'s touch lays her Hexed on what it hits, and starts her
  other hexes on it afresh.
- Her [malison](class-witch.md#talent-malison) makes a creature take 3 more damage from her for each of
  her hexes on it, and a creature that dies under one of her hexes pays her mana.

## Cooldowns

A **cooldown** is a status effect that only waits to wear off. While you are, say,
[winded](status-list.md#status-winded), the ability that winded you cannot be used again
(`you are winded`). Time is the only price: nothing counts how often you use an ability.

**Raging** comes with the berserker's rage, one of the
[class abilities](class-berserker.md). While the rage lasts you are stronger and tougher but
less clever, and cannot read or spend [mana](mana.md#spending-it). When it runs out you are
**Exhausted** for two and a half minutes: a little weaker, and with no rage
until it passes.

See also: [Status effects](status-effects.md), [Spell circles](spell-circles.md), and
[Potions](potions.md).
