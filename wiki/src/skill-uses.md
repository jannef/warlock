# Using your skills

Some [skills](skills.md) do more than add to a roll: they let you do something. This page explains those
uses: slipping past a creature in the dark, stealing, feinting, threatening, treating wounds and
poison, making arrows, whittling a better bow, and foraging for food. Most are used from the
[abilities menu](abilities.md) (`a`). Where a section says "the benefits table", it means the table under
[What skills give](skills.md#what-skills-give).

## Slipping past

Most [creatures](creatures.md) see a few cells into any darkness ([darkvision](vision.md#the-four-types)), so standing unlit beside one is
no hiding place by itself. But the first time a creature that is not hunting you would see you
**only** by its darkvision, it does not notice you at once. That means your cell is too dark for
its eyes: no [lantern](lantern.md), no fire, no sky lighting you. You get one try to slip past it.

It is a contest with the same dice on both sides. You roll 2d12 plus your [Dexterity](stats.md#stat-dexterity) [modifier](stats.md#modifiers) plus
what your [Stealth](skills.md#skill-stealth) gives (the benefits table). The creature rolls 2d12 plus its [Willpower](stats.md#stat-willpower) modifier
plus its [Spot](skills.md#skill-spot), if it has any. A tie is yours. A roll of 2 always fails and a roll of 24 always
succeeds. Your **edge** is the difference between the two sides:

{{#include generated/slipping-past.md}}

- **Unseen**: it does not see you while you stay in the dark. It goes about its business, walks
  past, or comes to look into the [footsteps](hearing.md#what-makes-noise) it heard and arrives beside you none the wiser. It has
  not seen you, so your first blow at it is a [sneak attack](skills.md#skill-sneak-attack). It notices you when you step into
  light it can see by (or light your [lantern](lantern.md)), when you attack it or anything else while it can see
  you, when it is hurt, or when the time in the table has passed.
- **Seen**: it hunts you as any creature would. You get no second try until it gives up the hunt.

Nothing tells you which it was: a creature that did not notice you gives no sign. Creatures never
slip past you.

## Stealing

If you are trained in **[Steal](skills.md#skill-steal)** ([thieves](classes.md#thief) are), **[Steal](abilities.md#ability-steal)** is in your [abilities menu](abilities.md). Choose it, then a
direction, and you try to lift something from whoever stands there. You can steal only from someone
you see who is not hostile to you: a peaceful villager, a shopkeeper, a [creature](creatures.md) that leaves you
alone. It takes a turn whatever happens.

Whether you go unnoticed is a contest with the same dice on both sides. You roll 2d12 plus your
[Dexterity](stats.md#stat-dexterity) [modifier](stats.md#modifiers) plus what your [Steal](skills.md#skill-steal) skill gives (the benefits table). Your victim rolls 2d12 plus
their [Willpower](stats.md#stat-willpower) modifier. A tie is yours. A roll of 2 always fails and a roll of 24 always succeeds.
Only the difference between the two sides matters, your **edge**:

{{#include generated/stealing.md}}

- **Unnoticed**: you take one stack of what they carry in their pack, all of it, such as a whole
  purse. You never see what someone carries in their pack, so you do not choose: your fingers find
  something. What they wear or hold in their hands stays where it is. If their pack is empty, you
  come away empty-handed ("You find nothing to take from the gardener."), and they notice nothing
  all the same.
- **Caught**: they turn on you and come after you, for good ("The kobold catches you stealing and
  turns on you!"). A shopkeeper who catches you is as outraged as a robbed one. The [shop](shops.md) trades no
  more, and whatever you had not paid for counts as stolen (see [Shops](shops.md)).

Stealing from a shopkeeper takes from their purse, never from the [shop](shops.md) floor. What lies in a [shop](shops.md) is for
sale, and taking it out without paying is theft by the shop's own rules. A shopkeeper whose purse
you lifted unnoticed still trades with you, but cannot pay for what you sell.

You may try as often as you like. Someone who caught you stays angry, and that is the price.

## Feinting

If you are trained in **[Bluff](skills.md#skill-bluff)**, **[Feint](abilities.md#ability-feint)** is in your [abilities menu](abilities.md). Choose it, then a direction, and
you make a false move at whoever stands there. It must be a [creature](creatures.md) you see, hostile to you, with a
mind to fool. Skeletons, spirits, and animated things (those without [vitals](combat.md#criticals)) do not count.
Otherwise the feint is refused and takes no time. A feint takes a turn.

The foe rolls 2d12 plus its [Willpower](stats.md#stat-willpower) [modifier](stats.md#modifiers) against the difficulty below, plus your [Presence](stats.md#stat-presence)
modifier, plus what your Bluff adds (the benefits table).

- If it falls short: "The kobold falls for it." It is [surprised](status-effects.md#status-surprised), and its next action does not end
  that. So your next blow finds it [flat-footed](surprise.md), with [sneak dice](surprise.md) if you have them. Its second
  action, like its first, costs it extra time, and ends the surprise. A foe quick enough to act
  twice before you do has recovered by the time you strike.
- Otherwise: "The kobold is not fooled." The turn is spent.

{{#include generated/feinting.md}}

You may feint as often as you like. Half of your feints train Bluff, whatever comes of them. There
is no talking your way out of being [caught stealing](#stealing). A lie is for the fight, or for getting past a
party on the road before it comes to a fight ([Talk your way past](encounters.md#talking-past)).

## Intimidating

If you are trained in **[Intimidate](skills.md#skill-intimidate)**, **[Intimidate](abilities.md#ability-intimidate)** is in your [abilities menu](abilities.md). Choose it, then a place
you see (with the [aiming](controls.md#aiming) cursor), and you threaten whoever stands there. It must be a [creature](creatures.md)
hostile to you, within reach, with a heart that can quail. Skeletons, spirits, and animated things
(those without [vitals](combat.md#criticals)) do not count. Otherwise the threat is refused and takes no time. A threat
takes a turn and is spoken aloud: those around you [hear](hearing.md) a voice.

It is the [War cry](abilities.md#ability-war-cry)'s check, at one foe. If the foe hears you and sees you, it rolls 2d12 plus its
[Willpower](stats.md#stat-willpower) [modifier](stats.md#modifiers) against the difficulty below, plus your [Strength](stats.md#stat-strength) modifier, plus what your
Intimidate adds (the benefits table). If it falls short, it is [frightened](status-effects.md#status-frightened) and runs from you, a few
moments per degree. How many degrees depends on your skill: a little Intimidate frightens a foe for
a moment, a great deal frightens it as long as a war cry does. A foe that does not hear or see you
pays you no heed, and the turn is spent all the same.

{{#include generated/intimidating.md}}

You may threaten as often as you like. Every threat trains Intimidate, whatever comes of it.

## Treating yourself

If you are trained in **[Heal](skills.md#skill-heal)** ([priests](classes.md#priest), [witches](classes.md#witch), [druids](classes.md#druid), and [paladins](classes.md#paladin) are), **[Heal](abilities.md#ability-heal)** is in your
[abilities menu](abilities.md), under Skills. It treats your **[bleeding](status-effects.md#bleeding)**, your **[poison](status-effects.md#poisoned)**, and a **[disease](diseases.md#how-a-disease-goes)** that shows. A
disease you have caught but that has not shown yet is unknown to you, so you cannot treat it.

It treats an [ally](allies.md) of yours standing beside you the same way, from the start: you know your own
allies' hurts, and they are listed after yours ("Your wolf's bleeding"). With nothing to treat, it
is greyed out: "nothing to treat". With more than one thing, it asks which. "Sickness" is a
[disease](diseases.md#how-a-disease-goes) you cannot name yet.

The try uses your [Intelligence](stats.md#stat-intelligence) and your skill. The difficulty depends on the patient's ailment.
Every try takes about a quarter of a minute, whatever comes of it, and makes no noise.

A try is a check (see [Diseases](diseases.md#checks)): 2d12 plus your [Intelligence](stats.md#stat-intelligence) [modifier](stats.md#modifiers) plus what your [Heal](skills.md#skill-heal) skill
gives (the benefits table), against the difficulty below. A roll of 2 always fails and a roll of 24
always succeeds.

- **[Bleeding](status-effects.md#bleeding)**: the difficulty is 6 plus 2 for each degree of [bleeding](status-effects.md#bleeding). If you make it, you bind the
  wound and the bleeding stops ("You bind the wound."). If you fail: "The blood still runs." You
  may try again at once, as often as you like.
- **[Poison](status-effects.md#poisoned)**: the difficulty is 8 plus 1 for each degree. If you make it, you draw out some of the
  poison and it is a degree weaker ("You draw out some of the poison."). If you fail: "The poison
  will not come out." You may try again at once, as often as you like.
- **A [disease](diseases.md#how-a-disease-goes)**: the difficulty is the [disease](diseases.md#how-a-disease-goes)'s own. If you make it, you ease it ("You ease the filth
  fever somewhat."). It counts as one of the checks in a row that shake the disease off, but never
  the last one, so treating never cures it by itself. A [disease](diseases.md#how-a-disease-goes) that never goes away by itself, or
  one a single won check from going, instead takes a full interval from your try before it next
  gets worse. If you fail: "You can do no more for the filth fever." You may try again at once.

{{#include generated/heal-targets.md}}

## Professions

[Metalcraft](skills.md#skill-metalcraft) and [Woodcraft](skills.md#skill-woodcraft) are
**professions**: crafts with a list of **recipes**, the things you can make or do with them. Each
has one row in the [abilities menu](abilities.md#the-abilities-menu), in the skill's place among
the Skills: **[Metalcraft…](abilities.md#ability-metalcraft)** and
**[Woodcraft…](abilities.md#ability-woodcraft)**. The row opens the profession's list.

You know a recipe by your skill, not by learning it. Some recipes are open to everyone, trained or
not:

- Metalcraft: **Temper**, a metal weapon or armour made finer at an anvil (see
  [Forges](altars-forges-springs.md#forges)).

Those trained in Metalcraft also forge swords at an anvil from metal stock (see
[Crafting](crafting.md)).
- Woodcraft: **Arrows** and **Crossbow bolts** for your bow or crossbow (see
  [Fletching](#fletching)), **Sling stones** (see
  [Crafting](crafting.md#batches-arrowheads-arrows-and-stones)), and **Whittle**, a wooden weapon
  made finer with a knife (see [Whittling](#whittling)).

Each row says what the recipe takes, such as "a metal weapon or armour, an anvil, about 10
minutes". A recipe you cannot make now is greyed out with the reason: "no anvil beside you", "no
knife", "no bow ready". Choose a recipe by its letter, or move to it and press Enter.

## Fletching

Anyone can make arrows and bolts. **Arrows** and **Crossbow bolts** are recipes on everyone's
[Woodcraft…](abilities.md#ability-woodcraft) list (see [Professions](#professions)). You need:

- a knife, in your [pack](objects.md#your-pack) or wielded. A dagger or a stiletto will do, or
  anything else of the [Daggers and knives](weapon-skills.md#weapon-skill-daggers) skill; a short
  sword will not.
- a bow [ready](ranged.md#getting-ready) for arrows, a crossbow for bolts.
- a billet of wood for the shafts, unless you are in a forest, a dense forest, or a swamp, where
  you cut them from the trees.
- a head for each arrow, if you want them to last (below).

Without them the list greys the recipe out: "no knife", "no bow ready", "no wood to hand".

You choose the head first:

- **[Arrowheads](crafting.md#batches-arrowheads-arrows-and-stones)** a smith has forged: one for
  each arrow, and the arrow takes the arrowhead's metal.
- **A bone shard or a block of flint** (or obsidian, horn, ivory, and the like): one heads a whole
  batch, and the arrows take its material. Flint-headed arrows strike as flint does.
- **None**: a sharpened shaft. These [brittle arrows](ammunition.md) fly, but break on any hit.

Arrows of different heads and woods are different goods: they do not stack together.

A batch takes ten minutes. Something coming into view, or anything else that would wake you from
a [rest](resting.md#how-resting-works), stops the work: the time is spent, and nothing else. A
batch that runs its full time rolls a check on your [Dexterity](stats.md), helped by your
Woodcraft, and the result decides how much of the count you get. If you are trained in
Woodcraft, it earns a training mark for every finished batch. Fletch as often as you like: the
time is the price.

A [ranger](classes.md#ranger)'s [Fletcher](talents.md#talent-fletcher) makes this far quicker and
surer, and rangers start with a knife. But even a ranger's arrows need heads not to be brittle.

{{#include generated/fletching.md}}

## Whittling

With a knife carried, anyone can improve a wooden weapon the way an [anvil](altars-forges-springs.md#forges) improves a metal one.
**Whittle** is a recipe on everyone's [Woodcraft…](abilities.md#ability-woodcraft) list, beside
the arrows (see [Professions](#professions)). It is greyed out with "no knife" if you have none.

Choose a weapon, a missile weapon, or a shield whose main part is wood: bows, crossbows, staves,
clubs. An attempt takes ten minutes. Something coming into view stops it; then the item is
untouched and the attempt does not count.

The rules are those of [tempering](altars-forges-springs.md#forges), item for item:

- A successful attempt gives one improvement (Fine, Marvellous, Masterwork) in something the item
  has.
- A failed one damages it and takes all its improvements away.
- A damaged item is mended or destroyed.
- Every attempt is harder than the last, for the item's whole life. So one item has one state,
  however it was worked.

The check is 2d12 plus your [Dexterity](stats.md) modifier, what Woodcraft gives, and what talents add to
Dexterity checks, against 13 plus 2 for every attempt ever made on the item. The odds are those of
the [forge table](altars-forges-springs.md#forges). Woodcraft's second benefit is +8 at 100 and +12 at 200. If you are trained in
Woodcraft, it earns a training mark for every finished attempt.

## Foraging

If you are trained in **[Survival](skills.md#skill-survival)** ([berserkers](classes.md#berserker), [druids](classes.md#druid), [rangers](classes.md#ranger), and [lizardfolk](races.md#race-lizardfolk) are), **[Forage](abilities.md#ability-forage)** is in your
[abilities menu](abilities.md). Choose it, and you spend a while searching the land around you for something to
eat.

You can forage only in the wild: on the [overworld](overworld.md), on open land, in woods, on the hills, or in the
marshes, or in the [local area](overworld.md#local-areas) of such a cell (an area forages like its cell on the [overworld](overworld.md#travel)). You
cannot forage on a road, in water or on ice, in a town, or in a [dungeon](dungeons.md). There the menu greys
Forage out ("not usable here").

A forage takes ten minutes. Something coming into view, or anything else that would wake you from
a [rest](resting.md#how-resting-works), stops it, and a forage cut short finds nothing. One that runs its full time finds food
with the land's chance below, plus what your [Survival](skills.md#skill-survival) gives (the benefits table: up to 30 more). Then
one of the land's finds is picked by the shares shown, and you gather a few of it, fresh, into your
pack ("You gather 2 handfuls of berries."). Otherwise: "You find nothing worth eating." Either way,
your [Survival](skills.md#skill-survival) may earn a training mark.

If you are also trained in **[Herbalism](skills.md#skill-herbalism)**, a forage that finds something may find a herb
besides: one of the land's herbs below, picked by the shares shown ("Your eye for herbs finds an
angelica root besides."). The chance is 25 % at 0, 60 % at 100, and 80 % at 200. Land with no herbs
gives none. A forage trains Herbalism one time in four, whatever it finds.

{{#include generated/foraging.md}}

What you find is ordinary [food](food.md) (see [Hunger](hunger.md)). Berries and mushrooms spoil within days, hazelnuts
keep for months, and an angelica root cures [diseases](diseases.md#how-a-disease-goes). Forage as often as you like, in the same
spot too: the time is the price.
