# Skills

Skills are what you have trained in: moving quietly, tumbling out of harm's way, finding your way
through the wild. Your [race](races.md) and your [class](classes.md) train you in some of them
when your character is made; the rest you are **untrained** in.

- An untrained skill does not exist for you. It is not shown anywhere, and it gives you nothing.
  That is not the same as a skill at 0: a skill you are trained in at 0 is shown, and it can
  grow.
- A trained skill has a **value**, usually between 5 and 30 when you start. 100 is a
  master. Past 100 a skill still grows, but slowly.
- `A` shows your **[Skills](controls.md#skills)** screen: every skill you are trained in, with its value, and beside it
  a small meter (`━━──`) of how practised it is: how big its next roll at a level-up will be
  (see [Getting better](#getting-better)). Use the arrow keys to read what each
  skill is about. It takes no time. `A` on the [character screen](character.md) (`@`) opens it
  too, and the summary at the end of character creation lists the skills you will start with.

What a skill does for you is never shown in numbers in the game: you see the value, and feel the
difference. This page gives the rules, with the exact numbers.

## The skills

Some skills work by themselves, all the time (**passive**); some will be used as **actions**
([searching](searching.md#searching-1), [picking a lock](doors.md#picking-locks), [binding a wound](#treating-yourself)); some are **read by rules** when something calls for
them ([athletics](skills.md#skill-athletics) when you [kick a door](doors.md#kicking)). For now only the passive skills, [Appraise](skills.md#skill-appraise), [Negotiation](skills.md#skill-negotiation), [Athletics](skills.md#skill-athletics)
(quicker in everything, and [kicking](doors.md#kicking) doors in, see [Doors](doors.md)), [Search](skills.md#skill-search) (see [Searching](searching.md)), [Disable
device](skills.md#skill-disable-device) (disarming traps, see [Traps](traps.md#disarming-traps)), [Open lock](skills.md#skill-open-lock) (picking locks,
see [Doors](doors.md#picking-locks)), [Steal](skills.md#skill-steal) (see [Stealing](#stealing) below), [Heal](skills.md#skill-heal) (see
[Treating yourself](#treating-yourself) below), and [Survival](skills.md#skill-survival) (foraging, see
[Foraging](#foraging) below) do something: the others are there to be trained, and get their uses
as the game grows. The skills used as actions are in the [abilities
menu](abilities.md) once you are trained in them, greyed out until their uses come; [Disable
device](skills.md#skill-disable-device)'s use is there as **[Disarm a trap](abilities.md#ability-disarm)**, [Open lock](skills.md#skill-open-lock)'s as **[Pick lock](abilities.md#ability-pick-lock)**, [Steal](skills.md#skill-steal)'s as **[Steal](abilities.md#ability-steal)**,
[Heal](skills.md#skill-heal)'s as **[Heal](abilities.md#ability-heal)**, and [Survival](skills.md#skill-survival)'s, though it is a passive skill, as **[Forage](abilities.md#ability-forage)**.

{{#include generated/skills.md}}

What is marked **trained only** is given only to those trained in the skill at all: untrained,
you cannot try, and a magic item that raises the skill (see [Magic items](magic-items.md)) does
not change that. Everything else a skill gives, a [magic item](magic-items.md) gives even to the untrained, as if
the skill were trained at the item's bonus.

## What skills give

The passive skills help you more the higher they are:

- **[Tumble](skills.md#skill-tumble)** adds to your [Defence](combat.md#defence) in the **[Dodge](combat.md#defence)** kind (see [Combat](combat.md#defence)), the one
  kind whose bonuses add up.
- **[Stealth](skills.md#skill-stealth)** makes your [footsteps](hearing.md#what-makes-noise) and the [doors](doors.md) you open and close quieter (see
  [Hearing](hearing.md)). Your attacks are as loud as ever. It also helps you
  [slip past](skills.md#slipping-past) a [creature](creatures.md) that sees in the dark: +8 at 100, +12 at 200.
- **[Listen](skills.md#skill-listen)** lets you hear fainter sounds: your [hearing](hearing.md) threshold drops by the number shown (it
  never drops below 1).
- **[Balance](skills.md#skill-balance)** takes some of the slowdown of a heavy load away (see [Burden](burden.md)): a
  [burdened](burden.md#burden-1) master is slowed by a tenth, not a quarter.
- **[Athletics](skills.md#skill-athletics)** makes you quicker in everything you do, a twentieth at 100 and a tenth at 200
  (it adds to the quickness of [talents](talents.md) such as [Fleet-footed](talents.md#talent-fleet-footed) and [Quick](talents.md#talent-quick); see
  [Burden](burden.md) for the load, which slows you on top of it). It also helps you kick [doors](doors.md)
  in (see [Doors](doors.md)). Besides kicks, it is trained now and then by acting under a load
  that slows you and by journeys on the [overworld](overworld.md#travel).
- **[Survival](skills.md#skill-survival)** makes your journeys on the [overworld](overworld.md) quicker (it does nothing for
  your steps in a local area or a dungeon), and lets you forage in the wild, finding food more
  often the higher it is (see [Foraging](#foraging)).
- **[Negotiation](skills.md#skill-negotiation)** gets you better prices in [shops](shops.md): you pay less and are offered
  more. A deal made (goods paid for, or your own sold) trains it now and then.
- **[Appraise](skills.md#skill-appraise)** now and then tells you what something you carry or wear really is. On every
  hour of game time that passes ([resting](resting.md#how-resting-works) too), if you carry anything you do not fully know
  (an [unidentified](magic-items.md#unidentified-items) [magic item](magic-items.md), a [scroll](scrolls.md#unknown-scrolls) of a kind you have not identified, a [wand](magic.md#wands) whose
  charges you do not know), you have a small chance to tell one of them, picked at random:
  2 % an hour at 0, 12 % at 100, 20 % at 200 (the table below; items of Appraisal raise it).
  Only those trained in [Appraise](skills.md#skill-appraise) get it at all. Each thing told this way trains it.
- **[Search](skills.md#skill-search)** helps you find [secret doors](doors.md#secret-doors) and other hidden things when you search (`s`, `S`),
  and a master's search reaches a cell farther (see [Searching](searching.md)).
- **[Spot](skills.md#skill-spot)** lets you [notice hidden things in passing](searching.md#noticing-in-passing) at all, [traps](traps.md#finding-traps) aside, and the higher it is,
  the more you notice.
- **[Detect traps](skills.md#skill-detect-traps)** is for hidden [traps](traps.md#finding-traps) what [Search](skills.md#skill-search) and [Spot](skills.md#skill-spot) are for the rest: it helps you find
  them when you search, lets you notice them in passing at all, and the higher it is, the more
  of them you find. [Thieves](classes.md#thief) start trained in it, and [kobolds](races.md#race-kobold).
- **[Open lock](skills.md#skill-open-lock)** lets you pick locks at all (the [Pick lock](abilities.md#ability-pick-lock) ability), and the higher it is, the
  more often a try works (see [Doors](doors.md#picking-locks)).
- **[Steal](skills.md#skill-steal)** lets you steal at all, and the higher it is, the likelier your hand goes unnoticed
  (see [Stealing](#stealing)).
- **[Heal](skills.md#skill-heal)** lets you bind your [bleeding](status-effects.md#bleeding) and tend your [poisons](status-effects.md#poisoned) and [diseases](diseases.md#how-a-disease-goes) at all, and the higher
  it is, the more often a try works (see [Treating yourself](#treating-yourself)).
- **[Knowledge](skills.md#skill-knowledge)** is knowing your enemy: it adds to your check against a [creature](creatures.md)'s [drain](stats.md#drained-stats), a
  [disease](diseases.md#how-a-disease-goes) its attack carries, or its hold when you struggle, but only against creatures whose
  level your [Knowledge](skills.md#skill-knowledge) stands above, and more the farther above (the table below; it never adds
  much). It earns marks whenever you gain [discovery experience](experience.md#discovery) (a first kill of a kind, a new
  place that pays) and, now and then, whenever you roll to resist such a power, whatever comes
  of it. A [creature](creatures.md)'s level is in its [table](creatures.md).
- **[Sneak attack](skills.md#skill-sneak-attack)** adds d6s to a hit on a foe that has not seen
  you, or is [surprised](status-effects.md#status-surprised): 1d6 once you are trained at all,
  one more at 25, 50, 75, and 100, then one more every 50 (5d6 at 100, 7d6 at 200). They come
  after [Protection](combat.md#protection), are never doubled by a
  [critical](combat.md#criticals), and add to the dice of [talents](talents.md) such as
  [Ambusher](talents.md#talent-ambusher). A blow counts at any reach; a shot, a throw, or a
  [spell](magic.md) that rolls to hit only within 6 cells ([Long reach](talents.md#talent-long-reach): 9).
  Nothing that has no vitals takes them (see [Criticals](combat.md#criticals)). It also makes
  those you catch unawares slower to gather their wits: their initiative check is 8 harder at
  100, 12 at 200. Each sneak attack that hits trains it half the time. Two weapon abilities
  fire on a sneak attack that hits: [Knife in the dark](weapon-skills.md#weapon-ability-knife-in-the-dark)
  ([Daggers](weapon-skills.md#weapon-skill-daggers) 5) leaves the foe [bleeding](status-effects.md#bleeding), and
  [Hunter's shot](fighting-styles.md#style-ability-hunters-shot) ([Marksman](fighting-styles.md#style-marksman) 6)
  [pins](status-effects.md#status-pinned) it for one or two actions; neither fires without the dice.
- **[Initiative](skills.md#skill-initiative)** helps you keep your wits when something falls on
  you unseen: +8 to the initiative check at 100, +12 at 200, and when you are
  [surprised](status-effects.md#status-surprised) all the same, your next action loses less time
  (half the extra at 100, three quarters at 200). Every such check trains it.

{{#include generated/skill-benefits.md}}

{{#include generated/skill-resist.md}}

## Who trains what

Each [race](races.md) trains one or two skills; each [class](classes.md) a handful, a few of them well (`2d6+10`) and the
rest a little (`2d4+4`). A skill your [race](races.md) and your [class](classes.md) both train adds the two. [Humans](races.md#race-human) train
none of their own, but they put extra work into one of their [class](classes.md)'s skills, picked at random.
The dice are rolled when your character is made, after your [stats](character.md#how-your-stats-add-up),
so they never change a [stat](stats.md).

As in ADOM, each [class](classes.md) also has a few skills its people **may** start with: each comes with the
chance in the last column, and then its dice, once, when your character is made. Two characters
of one [class](classes.md) can so begin with different skills. The game shows only the skills you have, never
the chances. Your [race](races.md)'s skill and the [human](races.md#race-human)'s extra [class](classes.md) skill add to one of these as to any
other, but a level-up roll on one of them is an ordinary roll: the [class](classes.md)'s +1 is for the skills in
its Trains column. A [race](races.md) may have such a skill too: a [drow](races.md#race-drow) may
start with [Sneak attack](skills.md#skill-sneak-attack), whatever their class (rolled after the
class's own).

{{#include generated/skill-grants.md}}

Some [creatures](creatures.md) are trained in skills too: deer listen well, and the [kobold shaman](creatures.md#creature-kobold-shaman) knows something
of [spells](magic.md#the-spells). You never see a [creature](creatures.md)'s skills in the game; they are on
[Creatures in numbers](creature-numbers.md#stats-and-skills).

## Getting better

Skills grow the way ADOM does it: **use charges a skill, and each new level cashes it in.**

**Training marks.** Each time you use a skill in a way that matters, it may earn a **training
mark**. The chance is shown in the table of skills above; skills you use all the time, such as
[balance](skills.md#skill-balance), earn marks rarely, so they do not race ahead.

- **[Stealth](skills.md#skill-stealth)**: a step that a [creature](creatures.md) nearby could have heard, if you had not stepped softly.
- **[Listen](skills.md#skill-listen)**: [hearing](hearing.md) something new.
- **[Tumble](skills.md#skill-tumble)**: dodging an attack ("You dodge the kobold's spear.") or stepping past a [trap](traps.md#finding-traps) you
  know of ("You deftly avoid the arrow trap.").
- **[Balance](skills.md#skill-balance)**: doing anything but waiting while your load slows you down.
- **[Survival](skills.md#skill-survival)**: a journey on the [overworld](overworld.md#travel).
- **[Steal](skills.md#skill-steal)**: every attempt to steal, whatever comes of it.
- **[Open lock](skills.md#skill-open-lock)**: a try at picking a lock, whether it works or not.
- **[Heal](skills.md#skill-heal)**: a try at treating yourself, whether it works or not.

Only skills you are trained in earn marks. Marks raise nothing by themselves: they gather until
your next level. The Skills screen's meter shows how practised a skill is (the step its marks
have reached on the ladder below), never the marks.

**Skill rolls.** Every level gives you a number of **skill rolls** (see
[Experience](experience.md)). You spend each one on a trained skill of your choice, before you
do anything else; nothing stops you putting every roll into one skill. A roll throws the die the
skill's marks have reached on the ladder and adds it to the skill, and spends half of the skill's
marks (rounded up), so a second roll on the same skill that level is a step or two smaller
(50 marks → 25 → 12 → 6 → 3: `3d3`, `2d4`, `1d5`, `1d3`, `+1`). A skill with no marks still
gains 1. Past 100 a roll adds half its result, rounded up. The game says "Your stealth improves."

- A skill your **[class](classes.md) is known for** (its `2d6+10` skills) throws the die one step up the
  ladder.
- Any **other skill your [class](classes.md) trains**, or a skill **your [race](races.md) trains**, adds 1 to the roll.
  Both stack: a [halfling](races.md#race-halfling) [thief](classes.md#thief)'s [stealth](skills.md#skill-stealth) is known for and racial, one step up and +1.

The level-up screen shows these numbers before you choose: beside each skill the dice a roll on
it adds (`1d3`, `+2`, `1d5+1`; `(1d3)/2` past 100), and for the selected skill how they came
about (`Rolls 1d5+1: practice (20 marks) gives 1d5, +1 as your class trains it.`), with a line
saying where your rolls come from (`4 skill rolls a level: class 3, race 1`). The roll itself
is thrown only when you choose.

{{#include generated/skill-ladder.md}}

How many rolls a level gives: your [class](classes.md)'s, plus your [race](races.md)'s, plus your [Intelligence](stats.md#stat-intelligence)'s.

{{#include generated/skill-rolls.md}}

Only you train, for now; [creatures](creatures.md) keep the skills they have. Your skills and their marks are
saved with your run.

## Slipping past

Most [creatures](creatures.md) see a few cells in any darkness ([darkvision](vision.md#the-four-types)), so standing
unlit beside one is no hiding place by itself. But the first time a creature that is not hunting
you would see you **only** by its darkvision (your cell too dark for its eyes: no
[lantern](lantern.md), no fire, no sky lighting you), it does not notice you at once: you try to
slip past it, once. It is a contest, the same dice on both sides: 2d12 plus your
[Dexterity](stats.md#stat-dexterity) [modifier](stats.md#modifiers) plus what your
[Stealth](skills.md#skill-stealth) gives (the benefits table above), against 2d12 plus the
creature's [Willpower](stats.md#stat-willpower) modifier plus its [Spot](skills.md#skill-spot), if it has any. A
tie is yours; a roll of 2 always fails and a roll of 24 always works. Your **edge** is the
difference:

{{#include generated/slipping-past.md}}

- **Unseen**, it does not see you while you stay in the dark: it goes about its business, walks
  past, or comes to look into the [footsteps](hearing.md#what-makes-noise) it heard and arrives beside you none the
  wiser. It has not seen you, so your first blow at it is a
  [sneak attack](skills.md#skill-sneak-attack). It notices you when you step into light it can
  see by (or light your [lantern](lantern.md)), when you attack it or anything else while it can
  see you, when it is hurt, or when the time above has passed.
- **Seen**, it hunts you as any creature would, and you get no second try until it gives up
  the hunt.

Nothing tells you which it was: a creature that did not notice you gives no sign.
Creatures never slip past you.

## Stealing

If you are trained in **[Steal](skills.md#skill-steal)** ([thieves](classes.md#thief) are), **[Steal](abilities.md#ability-steal)** is in your
[abilities menu](abilities.md): choose it, then a direction, and you try to lift something from
whoever stands there. You can steal only from someone you see who is not hostile to you: a
peaceful villager, a shopkeeper, a [creature](creatures.md) that leaves you alone. It takes a turn whatever comes
of it.

Whether you go unnoticed is a contest, the same dice on both sides: 2d12 plus your [Dexterity](stats.md#stat-dexterity)
[modifier](stats.md#modifiers) plus what your [Steal](skills.md#skill-steal) skill gives (the benefits table above), against 2d12 plus your
victim's [Willpower](stats.md#stat-willpower) modifier. A tie is yours; a roll of 2 always fails and a roll of 24 always
works. Only the difference between the two sides matters, your **edge**:

{{#include generated/stealing.md}}

- **Unnoticed**, you take one stack of what they carry in their pack, all of it: a whole purse.
  You never see what someone carries in their pack, so you do not choose: your fingers find
  something. What they wear or hold in their hands stays where it is. With nothing in their
  pack you come away empty-handed ("You find nothing to take from the gardener."), and they
  notice nothing all the same.
- **Caught**, they turn on you and come after you, for good ("The kobold catches you stealing and
  turns on you!"). A shopkeeper who catches you is outraged as a robbed one is: the [shop](shops.md) trades
  no more, and whatever you had not paid for is stolen (see [Shops](shops.md)).

Stealing from a shopkeeper takes from their purse, never from the [shop](shops.md) floor: what lies in a [shop](shops.md)
is for sale, and taking it out without paying is theft by the shop's own rules. A shopkeeper
whose purse you lifted unnoticed still trades with you, but cannot pay for what you sell.

You may try as often as you like; someone who caught you stays angry, and that is the price.

## Treating yourself

If you are trained in **[Heal](skills.md#skill-heal)** ([priests](classes.md#priest), [witches](classes.md#witch), [druids](classes.md#druid), and [paladins](classes.md#paladin) are), **[Heal](abilities.md#ability-heal)** is in your
[abilities menu](abilities.md), under Skills. It treats you, never anyone else (for now): your
**[bleeding](status-effects.md#bleeding)**, your **[poison](status-effects.md#poisoned)**, and a **[disease](diseases.md#how-a-disease-goes)** that shows (one you have caught but that has not
shown yet is unknown to you, so you cannot treat it). With nothing to treat it is greyed out,
"nothing to treat"; with more than one thing, it asks which ("Sickness" is a [disease](diseases.md#how-a-disease-goes) you cannot
name yet). Every try takes about a quarter of a minute, whatever comes of it, and makes no noise.

A try is a check (see [Diseases](diseases.md#checks)): 2d12 plus your [Intelligence](stats.md#stat-intelligence) [modifier](stats.md#modifiers) plus
what your [Heal](skills.md#skill-heal) skill gives (the benefits table above) against the difficulty below; a roll of 2
always fails and a roll of 24 always works.

- **[Bleeding](status-effects.md#bleeding)**: made, you bind the wound and the bleeding stops ("You bind the wound."); failed,
  "The blood still runs.", and you may try again at once, as often as you like. The worse the
  [bleeding](status-effects.md#bleeding), the harder it is: 6 plus 2 for each degree.
- **[Poison](status-effects.md#poisoned)**: made, you draw out some of it: it is a degree weaker ("You draw out some of the
  poison."). You get **one try** at a dose, whatever comes of it: afterwards "You can do no more
  for the poison." until it wears off; [poisoned](status-effects.md#poisoned) again later, you may try again. 8 plus 1 for each
  degree.
- **A [disease](diseases.md#how-a-disease-goes)**: made, you ease it ("You ease the filth fever somewhat."): it counts as one of
  the checks in a row that shake it off, though never the last one, so treating never cures it
  by itself. A [disease](diseases.md#how-a-disease-goes) that never goes away by itself, or one a single won check from going,
  instead takes a full interval from your try before it next gets worse. Again **one try** a
  bout, whatever comes of it ("You can do no more for the filth fever."); caught again later,
  you may try again. Its difficulty is the [disease](diseases.md#how-a-disease-goes)'s own.

{{#include generated/heal-targets.md}}

## Foraging

If you are trained in **[Survival](skills.md#skill-survival)** ([barbarians](classes.md#barbarian), [druids](classes.md#druid), [rangers](classes.md#ranger), and [lizardfolk](races.md#race-lizardfolk) are),
**[Forage](abilities.md#ability-forage)** is in your [abilities menu](abilities.md): choose it, and you spend a while searching
the land around you for something to eat. You can forage only in the wild: on the
[overworld](overworld.md), on open land, in woods, on the hills, or in the marshes, or in the
[local area](overworld.md#local-areas) of such a cell (an area forages as its cell on the [overworld](overworld.md#travel) does). Not on a road, not
in water or on ice, not in a town or a [dungeon](dungeons.md): there the menu greys Forage out ("not usable
here").

A forage takes ten minutes. Something coming into view, or anything else that would wake you
from a [rest](resting.md#how-resting-works), stops it, and a forage cut short finds nothing. One that runs its time finds food
with the land's chance below, plus what your [Survival](skills.md#skill-survival) gives (the benefits table above: up to 30
more); then one of the land's finds is picked by the shares shown, and you gather a few of it
("You gather 2 handfuls of berries."), fresh, into your pack. Otherwise "You find nothing worth
eating." Either way your [Survival](skills.md#skill-survival) may earn a training mark.

{{#include generated/foraging.md}}

What you find is ordinary [food](food.md) (see [Hunger](hunger.md)): berries and mushrooms spoil within days,
hazelnuts keep for months, and an angelica root cures [diseases](diseases.md#how-a-disease-goes). Forage as often as you like, in
the same spot too: the time is the price.
