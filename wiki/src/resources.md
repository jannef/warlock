# Resources

Everyone has [hit points](hit-points.md#hit-points) and [mana](mana.md). Some characters also hold
**resources** of their own: pools of points that a class or a talent gives them, such as a
[berserker](class-berserker.md)'s [rage](rage.md) or a [ranger](class-ranger.md)'s
[focus](#resource-focus). You may hold several at once, and each has its own rules:

- Some **come back** by themselves over time, up to their maximum.
- Some **wear off** over time, down to nothing.
- Some **build up** from what you do: a kill, a blow that hits, a [critical](combat.md#criticals),
  being hit, a [sneak attack](class-thief.md#talent-sneak-attack).
- Abilities may **spend** them. An ability that costs more than you have is greyed out in the
  [abilities menu](abilities.md) ("not enough focus").
- Some **end** when they run out: the resource is gone, and its bar with it, until something gives
  it to you again. Others stay with you, empty, and fill up again.

While you hold a resource, the status bar shows it after your mana, as `Rage 7` or `Focus 7/10`
(what you have, and for some your maximum). The [character screen](character.md) (`@`) gives it a
row of its own.

A resource that wears off while you [rest](resting.md) may run out before you wake. One that does
not come back while resting waits for you to get up. Creatures never hold any resources.

Nothing converts between resources and mana: each is spent only by what asks for it.

Times below are given in base turns. A base turn is 6 seconds.

## The resources

Each class's own pool has a page of its own, beside its class:

- [Rage](rage.md): the [berserker](class-berserker.md)'s fury, from the Rage ability.
- [Focus](focus.md): the [ranger](class-ranger.md)'s, for quicker shots and volleys.
- [Openings](openings.md): the [thief](class-thief.md)'s gaps in a foe's guard, spent by Exploit.
- [Ki](ki.md): the [ascetic](class-ascetic.md)'s, for flurries and the open-hand arts.
- [Inspiration](inspiration.md): the [bard](class-bard.md)'s, for the Inspire song.
- [Zeal](zeal.md): the [paladin](class-paladin.md)'s, for smites and laying on hands.

Stamina is no one class's: it is the pool of anyone who fights with a weapon's active abilities.

## Stamina

Stamina is the fighter's breath: the pool that active weapon abilities are paid from (see
[Combat](stances.md#stamina)), and the [warrior's](class-warrior.md)
[Charge](class-warrior.md#talent-charge). A warrior holds it, full, from level 1 (Charge gives it).
Anyone else gets it, full, from the first [weapon rank](weapon-skills.md#raising-a-weapon-skill) that
teaches an active ability.

The pool holds 10, plus your [Toughness](stats.md#stat-toughness) [modifier](stats.md#modifiers),
plus a point per 10 [levels](experience.md#levels), and never less than 5 (Toughness 14 holds 12 at
level 1). It comes back a point every 2 base turns (12 seconds), resting too, and sits at 0 when
spent.

A [berserker](class-berserker.md) holds rage and stamina at once.
[Raging](status-effects.md#status-raging) blocks spells, but not active weapon abilities.

## Magic items on resources

A [magic item](magic-items.md) may name a pool and change it while you wear or wield it, whether
identified or not. It can give:

- a larger **maximum**, by so many points;
- a faster **pace**, by so many percent, added to what talents add.
  [Steady breathing](talents.md#talent-steady-breathing)'s 50 % and an item's 30 % make 180 %;
- for stamina, every active weapon ability **costing** so many points less. This adds to
  [Drillmaster](class-warrior.md#talent-drillmaster)'s discount, and a cost never goes below 1;
- so many more points on a **kill** (*of the Hunt*), never above the maximum. This follows the same
  rule as [mana](mana.md#how-much-mana-you-have) *of Siphoning*: only your own blow, shot, or
  [spell](magic.md#the-spells) counts, with the one you killed within 3 cells of you (see
  [Mana on a kill](mana.md#mana-on-a-kill)). It comes on top of what a pool builds on any kill by
  its own rule (inspiration's point).

Two items naming the same pool add up. Taking the item off lowers what you hold to the smaller
maximum at once.

Such an item never **grants** a pool: it does nothing for someone who does not hold it. Who holds
which pool:

- **Stamina:** every [warrior](class-warrior.md), [berserker](class-berserker.md),
  [paladin](class-paladin.md), [ranger](class-ranger.md), [ascetic](class-ascetic.md),
  [thief](class-thief.md), and [bard](class-bard.md), and anyone else once a
  [weapon rank](weapon-skills.md#raising-a-weapon-skill) teaches an active ability.
- **Rage:** a [raging](status-effects.md#status-raging) [berserker](class-berserker.md), only while
  the rage lasts.
- **Focus:** a [ranger](class-ranger.md) from level 12.
- **Ki:** an [ascetic](class-ascetic.md) from level 1.
- **Inspiration:** a [bard](class-bard.md) from level 9.
- **Openings:** a [thief](class-thief.md) from level 1.

A ring of rage on a [wizard](class-wizard.md) is dead weight. [Creatures](creatures.md) never hold
a pool, so such gear does nothing for them, only for whoever takes it from them.

{{#include generated/resources.md}}

## Talents on resources

Five [talents](talents.md) change a resource. Only the class that has the resource may take them,
as a pick from its trees. Each is on its resource's page:
[Battle trance](rage.md#the-talent-on-it), [Inner calm](ki.md#the-talent-on-it),
[Steady hand](focus.md#the-talent-on-it), [Patient blade](openings.md#the-talent-on-it), and
[Zealot](zeal.md#the-talent-on-it).
