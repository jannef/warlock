# Skills

Skills are what you have trained in: moving quietly, tumbling out of harm's way, finding your way
through the wild. Your [race](races.md) and your [class](classes.md) train you in some of them
when your character is made; the rest you are **untrained** in.

- An untrained skill does not exist for you. It is not shown anywhere, and it gives you nothing.
  That is not the same as a skill at 0: a skill you are trained in at 0 is shown, and it can
  grow.
- A trained skill has a **value**, usually between 5 and 30 when you start. 100 is a
  master. Past 100 a skill still grows, but slowly.
- `A` shows your **[Skills](the-screen.md#skills)** screen: every skill you are trained in, with its value, and beside it
  a small meter (`━━──`) of how practised it is: how big its next roll at a level-up will be
  (see [Getting better](#getting-better)). Use the arrow keys to read what each
  skill is about. It takes no time. `A` on the [character screen](character.md) (`@`) opens it
  too, and the summary at the end of character creation lists the skills you will start with.

What a skill does for you is never shown in numbers in the game: you see the value, and feel the
difference. This page gives the rules, with the exact numbers.

## The skills

Some skills work by themselves, all the time (**passive**); some will be used as **actions**
([searching](searching.md#searching-1), [picking a lock](doors.md#picking-locks), [binding a wound](skill-uses.md#treating-yourself)); some are **read by rules** when something calls for
them ([athletics](skills.md#skill-athletics) when you [kick a door](doors.md#kicking)). For now only the passive skills, [Appraise](skills.md#skill-appraise), [Negotiation](skills.md#skill-negotiation), [Athletics](skills.md#skill-athletics)
(quicker in everything, and [kicking](doors.md#kicking) doors in, see [Doors](doors.md)), [Search](skills.md#skill-search) (see [Searching](searching.md)), [Disable
device](skills.md#skill-disable-device) (disarming traps, see [Traps](traps.md#disarming-traps)), [Open lock](skills.md#skill-open-lock) (picking locks,
see [Doors](doors.md#picking-locks)), [Steal](skills.md#skill-steal) (see [Stealing](skill-uses.md#stealing) below), [Bluff](skills.md#skill-bluff) (see
[Feinting](skill-uses.md#feinting) below), [Heal](skills.md#skill-heal) (see
[Treating yourself](skill-uses.md#treating-yourself) below), and [Survival](skills.md#skill-survival) (foraging, see
[Foraging](skill-uses.md#foraging) below) do something: the others are there to be trained, and get their uses
as the game grows. The skills used as actions are in the [abilities
menu](abilities.md) once you are trained in them, greyed out until their uses come; [Disable
device](skills.md#skill-disable-device)'s use is there as **[Disarm a trap](abilities.md#ability-disarm)**, [Open lock](skills.md#skill-open-lock)'s as **[Pick lock](abilities.md#ability-pick-lock)**, [Steal](skills.md#skill-steal)'s as **[Steal](abilities.md#ability-steal)**, [Bluff](skills.md#skill-bluff)'s as **[Feint](abilities.md#ability-feint)**,
[Heal](skills.md#skill-heal)'s as **[Heal](abilities.md#ability-heal)**, [Decipher script](skills.md#skill-decipher-script)'s as **[Decipher](abilities.md#ability-decipher)** (see [Deciphering a scroll unread](scrolls.md#deciphering-a-scroll-unread)), [Survival](skills.md#skill-survival)'s, though it is a passive skill, as **[Forage](abilities.md#ability-forage)**, and [Woodcraft](skills.md#skill-woodcraft)'s, which everyone has trained or not, as **[Fletch](abilities.md#ability-fletch)** (see [Fletching](skill-uses.md#fletching) below), and, for those trained in it, **[Wedge a door](abilities.md#ability-wedge)** (see [Wedging a door](doors.md#wedging-a-door)).

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
  [slip past](skill-uses.md#slipping-past) a [creature](creatures.md) that sees in the dark: +8 at 100, +12 at 200.
- **[Listen](skills.md#skill-listen)** lets you hear fainter sounds: your [hearing](hearing.md) threshold drops by the number shown (it
  never drops below 1).
- **[Balance](skills.md#skill-balance)** takes some of the slowdown of a heavy load away (see [Burden](burden.md)): a
  [burdened](burden.md#burden-1) master is slowed by a tenth, not a quarter. It also spares a part of what rough ground
  adds to a step ([bush](terrain.md#terrain-bush), [reeds](terrain.md#terrain-reeds), [rubble](terrain.md#terrain-rubble), ice; not water or
  marsh, which are Swim's: 60 % of the extra at 100, all of it at 200), and a part of the extra time a
  [staggered](status-effects.md#status-staggered) action costs (half at 100, three quarters at 200).
- **[Music](skills.md#skill-music)** makes your [songs](abilities.md#songs) last longer (a
  degree more at 50, at 100, and every 100 after) and harder for those they are sung against to
  shake off (+4 to their difficulty at 100, +8 at 200); its two columns in the table below are
  its ladders. Every song trains it.
- **[Athletics](skills.md#skill-athletics)** makes you quicker in everything you do, a twentieth at 100 and a tenth at 200
  (it adds to the quickness of [talents](talents.md) such as [Fleet-footed](talents.md#talent-fleet-footed) and [Quick](talents.md#talent-quick); see
  [Burden](burden.md) for the load, which slows you on top of it). It also helps you kick [doors](doors.md)
  in (see [Doors](doors.md)). Besides kicks, it is trained now and then by acting under a load
  that slows you, by journeys on the [overworld](overworld.md#travel), and by climbing out of a
  [pit](traps.md#trap-pit).
- **[Escape artist](skills.md#skill-escape-artist)** slips bonds without being asked: when
  [Held](status-effects.md#status-held), [Webbed](status-effects.md#status-webbed), or
  [Pinned](status-effects.md#status-pinned) stops what you try, the better of your
  [Strength](stats.md#stat-strength) and [Dexterity](stats.md#stat-dexterity) plus the skill is
  checked as part of the attempt (see [Slipping bonds](status-effects.md#slipping-bonds); untrained,
  a web or a pin just refuses the step). +8 at 100, +12 at 200. It also shortens the climb out of
  a [pit](traps.md#trap-pit): half from 50, none from 100. It also lets you [wriggle past](npcs.md#getting-past) someone who will not make way. Every such
  check, climb, and half the wriggles train it.
- **[Survival](skills.md#skill-survival)** makes your journeys on the [overworld](overworld.md) quicker (it does nothing for
  your steps in a local area or a dungeon), and lets you forage in the wild, finding food more
  often the higher it is (see [Foraging](skill-uses.md#foraging)).
- **[Swim](skills.md#skill-swim)** makes you quicker through [water](terrain.md#water) and [marsh](terrain.md#terrain-marsh) (of what wading adds to a step, 60%
  is spared at 100 and all of it at 200; the [Swimmer](talents.md#talent-swimmer) talent spares it all at once). And
  when you [plunge through the ice](temperature.md#falling-in), you make a check on your
  [Toughness](stats.md#stat-toughness) plus the table's points against 11: made, half the hit points. And it
  is what lets you into [deep water](terrain.md#deep-water) at all.
- **[Negotiation](skills.md#skill-negotiation)** gets you better prices in [shops](shops.md): you pay less and are offered
  more, and a healer or a sage who sells a [service](npcs.md#services) asks less (a [temple](npcs.md#services) never haggles).
  A deal made (goods paid for, your own sold, or a service paid) trains it now and then.
- **[Appraise](skills.md#skill-appraise)** now and then tells you what something you carry or wear really is. On every
  hour of game time that passes ([resting](resting.md#how-resting-works) too), if you carry anything you do not fully know
  (an [unidentified](magic-items.md#unidentified-items) [magic item](magic-items.md), a [scroll](scrolls.md#unknown-scrolls) of a kind you have not identified, a [wand](wands-and-rods.md#wands) whose
  charges you do not know), you have a small chance to tell one of them, picked at random:
  2 % an hour at 0, 12 % at 100, 20 % at 200 (the table below; items of Appraisal raise it).
  Only those trained in [Appraise](skills.md#skill-appraise) get it at all. Each thing told this way trains it.
  Appraise also has an eye for food: an hour at a time, a trained eye may tell in words how fresh a corpse or food is
  (*fresh*, *old*, or *rotten*), in the eat list and when you look at it (see [Food](food.md#food-spoils)). [Survival](skills.md#skill-survival)
  gives the same eye, and where you have both the chances add up (25 % at 0, 60 % at 100, 80 % at 200 for Appraise; 15 %,
  35 %, 50 % for Survival; rotten food is plainer to tell, 25 points more, and no chance is above 95 %). Eating food whose
  freshness you could tell trains Appraise now and then.
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
  (see [Stealing](skill-uses.md#stealing)).
- **[Bluff](skills.md#skill-bluff)** lets you feint at all ([Feint](abilities.md#ability-feint) in the abilities menu;
  see [Feinting](skill-uses.md#feinting)) and to [talk your way past](encounters.md#talking-past) some parties met on the
  overworld, and the higher it is, the harder your lies are to see through: the
  [Willpower](stats.md#stat-willpower) check they ask is 8 harder at 100, 12 at 200 (at most 20).
- **[Heal](skills.md#skill-heal)** lets you bind your [bleeding](status-effects.md#bleeding) and tend your [poisons](status-effects.md#poisoned) and [diseases](diseases.md#how-a-disease-goes) at all, and the higher
  it is, the more often a try works (see [Treating yourself](skill-uses.md#treating-yourself)).
- **[Knowledge](skills.md#skill-knowledge)** is knowing your enemy: it adds to your check against a [creature](creatures.md)'s [drain](stats.md#drained-stats), a
  [disease](diseases.md#how-a-disease-goes) its attack carries, or its hold when you struggle, but only against creatures whose
  level your [Knowledge](skills.md#skill-knowledge) stands above, and more the farther above (the table below; at most 6).
  Standing above a creature's level by even 1 also tells you, in [look mode](controls.md#look-mode), what its kind
  [resists, is immune to, and is vulnerable to](damage-types.md#armour-and-resistance), in words; that much is lore of its kind, so it
  says nothing of what this one wears or has drunk, and earns no marks. It earns marks whenever you gain [discovery experience](experience.md#discovery) (a first kill of a kind, a new
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
  The whole rule is under [Surprise and sneak attacks](surprise.md).
- **[Initiative](skills.md#skill-initiative)** helps you keep your wits when something falls on
  you unseen: +8 to the initiative check at 100, +12 at 200, and when you are
  [surprised](status-effects.md#status-surprised) all the same, your next action loses less time
  (half the extra at 100, three quarters at 200). Every such check trains it.
- **[Intimidate](skills.md#skill-intimidate)** lets you threaten one foe
  ([Intimidate](abilities.md#ability-intimidate) in the abilities menu; see
  [Intimidating](skill-uses.md#intimidating)) and makes your threats and your
  [War cry](abilities.md#ability-war-cry) (the [berserker](classes.md#berserker)'s) harder to stand
  against: the Willpower check they ask is 8 harder at 100, 12 at 200 (at most 20). A threat
  also frightens longer the more it adds. Every threat and every cry trains it, whatever comes of
  it.

[Skill focus](talents.md#talent-skill-focus) makes one skill you are trained in count 25 higher
in the table below, as a [magic item](magic-items.md)'s bonus does: past 100 too (100 slows a
skill's [growth](skills.md#getting-better), and is no ceiling), on top of
[Master of none](talents.md#talent-master-of-none) and any magic item. Your Skills screen still
shows the value you trained. On most skills the +25 is worth 2 points of what the skill gives below 100 and
1 past it; on [Stealth](skills.md#skill-stealth)'s quiet, 15 % less noise below 100 and 7 % past it.

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
- **[Listen](skills.md#skill-listen)**: [hearing](hearing.md) something new, and every turn spent [listening at a door](hearing.md#listening-at-a-door).
- **[Tumble](skills.md#skill-tumble)**: dodging an attack ("You dodge the kobold's spear.") or stepping past a [trap](traps.md#finding-traps) you
  know of ("You deftly avoid the arrow trap."). A blow that misses you for no reason your
  [Defence](combat.md#defence) explains (it would have missed anyone) counts as a dodge one time in four.
- **[Balance](skills.md#skill-balance)**: doing anything but waiting while your load slows you down (5 %), a step onto rough ground that slows (10 %: bush, reeds, rubble, ice; not water), and an action taken [staggered](status-effects.md#status-staggered) (20 %).
- **[Survival](skills.md#skill-survival)**: a journey on the [overworld](overworld.md#travel).
- **[Swim](skills.md#skill-swim)**: a step wading through [water](terrain.md#water) or [marsh](terrain.md#terrain-marsh) now and then (10 %), every [plunge through the ice](temperature.md#falling-in), and a quarter of the steps in [deep water](terrain.md#deep-water), made or not.
- **[Steal](skills.md#skill-steal)**: every attempt to steal, whatever comes of it.
- **[Bluff](skills.md#skill-bluff)**: half of your feints, and every try at [talking your way past](encounters.md#talking-past) a party, whatever comes of them.
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
