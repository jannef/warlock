# Skills

Skills are what you have trained in: moving quietly, tumbling out of harm's way, finding your way
through the wild. Your [race](races.md) and your [class](classes.md) train you in some of them when your character is made.
You are **untrained** in the rest. Some skills also let you do something new, such as stealing or
treating a wound; those uses are explained on [Using your skills](skill-uses.md).

- An untrained skill does not exist for you. It is not shown anywhere, and it gives you nothing.
  That is not the same as a skill at 0: a skill you are trained in at 0 is shown, and it can grow.
- A trained skill has a **value**, usually between 5 and 30 when you start. 100 is a master. Past
  100 a skill still grows, but slowly.
- `A` shows your **[Skills](the-screen.md#skills)** screen. It lists every skill you are trained in, with its value and a
  small meter (`━━──`) of how practised it is, which is how big its next roll will
  be (see [Getting better](#getting-better)). Use the arrow keys to read what each skill is about. Looking takes no
  time. `A` on the [character screen](character.md) (`@`) opens it too, and the summary at the end of character
  creation lists the skills you will start with.

The game never shows in numbers what a skill does for you. You see the value and feel the
difference. This page gives the rules, with the exact numbers.

## The skills

Skills come in three sorts:

- **Passive** skills work by themselves, all the time.
- Some are used as **actions**: [searching](searching.md#searching-1), [picking a lock](doors.md#picking-locks), [binding a wound](skill-uses.md#treating-yourself).
- Some are **checked by rules** when something calls for them, such as [athletics](skills.md#skill-athletics) when you [kick a door](doors.md#kicking).

Not every skill does something yet. These do: the passive skills, [Appraise](skills.md#skill-appraise), [Negotiation](skills.md#skill-negotiation), [Athletics](skills.md#skill-athletics)
(quicker in everything, and [kicking](doors.md#kicking) doors in; see [Doors](doors.md)), [Search](skills.md#skill-search) (see [Searching](searching.md)),
[Disable device](skills.md#skill-disable-device) (disarming traps; see [Traps](traps.md#disarming-traps)), [Open lock](skills.md#skill-open-lock) (picking locks; see [Doors](doors.md#picking-locks)), [Steal](skills.md#skill-steal) (see
[Stealing](skill-uses.md#stealing)), [Bluff](skills.md#skill-bluff) (see [Feinting](skill-uses.md#feinting)), [Heal](skills.md#skill-heal) (see [Treating yourself](skill-uses.md#treating-yourself)), and [Survival](skills.md#skill-survival) (see
[Foraging](skill-uses.md#foraging)), [Herbalism](skills.md#skill-herbalism) (herbs on a forage, more of them [gathered](herbs.md#gathering) from the glowing fungus), [Mining](skills.md#skill-mining) (quicker
[digging](digging.md)), [Alchemy](skills.md#skill-alchemy), and [Enchanting](skills.md#skill-enchanting) (telling potions and magic). The others are there to be trained, and get their uses as the game grows.

Skills used as actions appear in the [abilities menu](abilities.md) once you are trained in them, greyed out
until their uses come. Their names there:

- [Disable device](skills.md#skill-disable-device): **[Disarm a trap](abilities.md#ability-disarm)**
- [Open lock](skills.md#skill-open-lock): **[Pick lock](abilities.md#ability-pick-lock)**
- [Steal](skills.md#skill-steal): **[Steal](abilities.md#ability-steal)**
- [Bluff](skills.md#skill-bluff): **[Feint](abilities.md#ability-feint)**
- [Heal](skills.md#skill-heal): **[Heal](abilities.md#ability-heal)**
- [Decipher script](skills.md#skill-decipher-script): **[Decipher](abilities.md#ability-decipher)** (see [Deciphering a scroll unread](scrolls.md#deciphering-a-scroll-unread))
- [Survival](skills.md#skill-survival), though it is a passive skill: **[Forage](abilities.md#ability-forage)**
- [Jewelcraft](skills.md#skill-jewelcraft): **[Jewelcraft…](abilities.md#ability-jewelcraft)**, its [profession](skill-uses.md#professions)'s recipes (Cut and Set, see [Gem cutting](gem-cutting.md); rings and amulets, see [Making jewellery](crafting.md#making-jewellery))
- [Metalcraft](skills.md#skill-metalcraft): **[Metalcraft…](abilities.md#ability-metalcraft)**, its [profession](skill-uses.md#professions)'s recipes (Temper among them)
- [Woodcraft](skills.md#skill-woodcraft): **[Woodcraft…](abilities.md#ability-woodcraft)**, its [profession](skill-uses.md#professions)'s recipes (arrows, bolts, sling stones, and Whittle among them), and **[Wedge a door](abilities.md#ability-wedge)** for those
  trained in it (see [Wedging a door](doors.md#wedging-a-door))

{{#include generated/skills.md}}

Benefits marked **trained only** are given only to those trained in the skill. If you are
untrained you cannot try, and a magic item that raises the skill (see [Magic items](magic-items.md)) does not
change that. Everything else a skill gives, a [magic item](magic-items.md) gives even to the untrained, as if the skill
were trained at the item's bonus.

## What skills give

The passive skills help you more the higher they are:

- **[Tumble](skills.md#skill-tumble)** adds to the **[Dodge](combat.md#defence)** part of your [Defence](combat.md#defence) (see [Combat](combat.md#defence)), the one kind whose
  bonuses add up.
- **[Stealth](skills.md#skill-stealth)** makes your [footsteps](hearing.md#what-makes-noise), and the [doors](doors.md) you open and close, quieter (see [Hearing](hearing.md)).
  Your attacks are as loud as ever. It also helps you [slip past](skill-uses.md#slipping-past) a [creature](creatures.md) that sees in the dark:
  +8 at 100, +12 at 200.
- **[Listen](skills.md#skill-listen)** lets you hear fainter sounds. Your [hearing](hearing.md) threshold drops by the number shown,
  but never below 1.
- **[Balance](skills.md#skill-balance)** takes away some of the slowdown of a heavy load (see [Burden](burden.md)). A [burdened](burden.md#burden-1) master
  is slowed by a tenth, not a quarter. It also saves part of the time rough ground adds to a step
  ([bush](terrain.md#terrain-bush), [reeds](terrain.md#terrain-reeds), [rubble](terrain.md#terrain-rubble), ice): 60 % of the extra at 100, all of it at 200. Water and marsh
  are Swim's, not Balance's. And it saves part of the extra time a [staggered](status-effects.md#status-staggered) action costs: half at
  100, three quarters at 200.
- **[Music](skills.md#skill-music)** makes your [songs](abilities.md#songs) last longer: a degree more at 50, at 100, and every 100 after. It
  also makes them harder for those they are sung against to shake off: +4 to their difficulty at
  100, +8 at 200. Its two columns in the table below are its ladders. Every song trains it.
- **[Athletics](skills.md#skill-athletics)** makes you quicker in everything you do: a twentieth at 100 and a tenth at 200. It
  adds to the quickness of [talents](talents.md) such as [Fleet-footed](talents.md#talent-fleet-footed) and [Quick](talents.md#talent-quick). Your load slows you on top of it
  (see [Burden](burden.md)). It also helps you kick [doors](doors.md) in (see [Doors](doors.md)). Besides kicks, it trains now and
  then when you act under a load that slows you, on journeys on the [overworld](overworld.md#travel), and when you
  climb out of a [pit](traps.md#trap-pit).
- **[Escape artist](skills.md#skill-escape-artist)** slips bonds without being asked. When being [Held](status-effects.md#status-held), [Webbed](status-effects.md#status-webbed), or [Pinned](status-effects.md#status-pinned) stops
  what you try, the better of your [Strength](stats.md#stat-strength) and [Dexterity](stats.md#stat-dexterity) plus the skill is checked as part of the
  attempt (see [Slipping bonds](held.md#slipping-bonds)): +8 at 100, +12 at 200. Untrained, a web or a pin just stops
  the step. It also shortens the climb out of a [pit](traps.md#trap-pit): half from 50, none from 100. And it lets you
  [wriggle past](npcs.md#getting-past) someone who will not make way. Every such check and climb, and half the wriggles,
  train it.
- **[Survival](skills.md#skill-survival)** makes your journeys on the [overworld](overworld.md) quicker. It does nothing for your steps in a
  local area or a dungeon. It also lets you forage in the wild, finding food more often the higher
  it is (see [Foraging](skill-uses.md#foraging)).
- **[Swim](skills.md#skill-swim)** makes you quicker through [water](terrain.md#water) and [marsh](terrain.md#terrain-marsh): of what wading adds to a step, 60 % is
  saved at 100 and all of it at 200. The [Swimmer](talents.md#talent-swimmer) talent saves all of it at once. When you
  [plunge through the ice](temperature.md#falling-in), you make a check on your [Toughness](stats.md#stat-toughness) plus the table's points against 11; if you
  make it, you lose half the hit points. Swim is also what lets you into [deep water](terrain.md#deep-water) at all.
- **[Negotiation](skills.md#skill-negotiation)** gets you better prices in [shops](shops.md): you pay less and are offered more. A healer
  or a sage who sells a [service](npcs.md#services) asks less too, though a [temple](npcs.md#services) never haggles. A deal made (goods
  paid for, your own sold, or a service paid) trains it now and then.
- **[Appraise](skills.md#skill-appraise)** now and then tells you what something you carry or wear really is.
  - Every hour of game time ([resting](resting.md#how-resting-works) too), if you carry anything you do not fully know, you have
    a small chance to learn one of those things, picked at random. That covers an [unidentified](magic-items.md#unidentified-items)
    [magic item](magic-items.md), a [scroll](scrolls.md#unknown-scrolls) of a kind you have not identified, or a [wand](wands-and-rods.md#wands) whose charges you do not
    know. The chance is 2 % an hour at 0, 12 % at 100, 20 % at 200 (see the table below; items of
    Appraisal raise it). Only those trained in [Appraise](skills.md#skill-appraise) get it at all. Each thing learned this way
    trains it.
  - Appraise also has an eye for food. An hour at a time, a trained eye may tell how fresh a corpse
    or food is, in words (*fresh*, *old*, or *rotten*), in the eat list and when you look at it (see
    [Food](food.md#food-spoils)). [Survival](skills.md#skill-survival) gives the same eye, and with both the chances add up. Appraise gives 25 % at 0,
    60 % at 100, and 80 % at 200. Survival gives 15 %, 35 %, and 50 %. Rotten food is easier to
    tell, 25 points more, and no chance is above 95 %. Eating food whose freshness you could tell
    trains Appraise now and then.
- **[Alchemy](skills.md#skill-alchemy)** and **[Enchanting](skills.md#skill-enchanting)** look the same way, each hour, each at its own things.
  Alchemy tells a [potion or flask](potions.md) whose kind you do not know ("You recognise your fizzy
  scarlet potion: a flask of fire."). Enchanting tells a magic thing whose magic you do not know:
  an [unidentified](magic-items.md#unidentified-items) weapon, armour, ring or amulet, or a [wand or rod](wands-and-rods.md). Each gives 5 % an hour at 0,
  25 % at 100, and 40 % at 200, trained only, besides Appraise's own chance. Neither trains by it:
  their crafts are still to come.
- **[Mining](skills.md#skill-mining)** lets you [dig](digging.md) at all: untrained, you cannot. It
  makes digging quicker: each turn does 25 % more work at 100 and 40 % more at 200, on top of
  what talents and a Delving tool add. [Alberich](npcs.md#alberich) in Fyrdelf teaches it for
  gold.
- **[Herbalism](skills.md#skill-herbalism)** may find a herb besides on a forage that finds something (see
  [Foraging](skill-uses.md#foraging)), and finds one more herb for every 50 when you
  [gather](herbs.md#gathering) from the glowing fungus.
- **[Search](skills.md#skill-search)** helps you find [secret doors](doors.md#secret-doors) and other hidden things when you search (`s`, `Alt+s`). A
  master's search reaches one cell farther (see [Searching](searching.md)).
- **[Spot](skills.md#skill-spot)** lets you [notice hidden things in passing](searching.md#noticing-in-passing) at all, except traps. The higher it is, the more you
  notice.
- **[Detect traps](skills.md#skill-detect-traps)** does for hidden [traps](traps.md#finding-traps) what [Search](skills.md#skill-search) and [Spot](skills.md#skill-spot) do for everything else. It helps you find
  them when you search and lets you notice them in passing at all. The higher it is, the more of
  them you find. [Thieves](classes.md#thief) and [kobolds](races.md#race-kobold) start trained in it.
- **[Open lock](skills.md#skill-open-lock)** lets you pick locks at all (the [Pick lock](abilities.md#ability-pick-lock) ability). The higher it is, the more
  often a try works (see [Doors](doors.md#picking-locks)).
- **[Steal](skills.md#skill-steal)** lets you steal at all. The higher it is, the likelier your hand goes unnoticed (see
  [Stealing](skill-uses.md#stealing)).
- **[Bluff](skills.md#skill-bluff)** lets you feint at all ([Feint](abilities.md#ability-feint) in the abilities menu; see [Feinting](skill-uses.md#feinting)), and lets you
  [talk your way past](encounters.md#talking-past) some parties met on the overworld. The higher it is, the harder your lies
  are to see through: the [Willpower](stats.md#stat-willpower) check they ask is 8 harder at 100 and 12 at 200 (at most
  20).
- **[Heal](skills.md#skill-heal)** lets you bind your [bleeding](status-effects.md#bleeding) and tend your [poisons](status-effects.md#poisoned) and [diseases](diseases.md#how-a-disease-goes) at all. The higher it is,
  the more often a try works (see [Treating yourself](skill-uses.md#treating-yourself)).
- **[Knowledge](skills.md#skill-knowledge)** is knowing your enemy.
  - It adds to your check against a [creature](creatures.md)'s [drain](stats.md#drained-stats), a [disease](diseases.md#how-a-disease-goes) its attack carries, or its hold
    when you struggle. It helps only against creatures whose level your [Knowledge](skills.md#skill-knowledge) stands above,
    and more the farther above (see the table below; at most 6).
  - Standing above a creature's level by even 1 also tells you in [look mode](controls.md#look-mode) what its kind
    [resists, is immune to, and is vulnerable to](damage-types.md#armour-and-resistance), in words. That is lore about its kind, so it says
    nothing of what this one wears or has drunk, and it earns no marks.
  - It earns marks whenever you gain [discovery experience](experience.md#discovery) (a first kill of a kind, a new place
    that pays), and now and then whenever you roll to resist such a power, whatever comes of it.
  - A [creature](creatures.md)'s level is in its [table](creatures.md).
- **[Sneak attack](skills.md#skill-sneak-attack)** adds d6s to a hit on a foe that has not seen you, or is [surprised](status-effects.md#status-surprised).
  - You get 1d6 once you are trained at all, one more at 25, 50, 75, and 100, then one more every
    50 (5d6 at 100, 7d6 at 200). The dice come after [Protection](combat.md#protection), a [critical](combat.md#criticals) never doubles them,
    and they add to the dice of [talents](talents.md) such as [Ambusher](talents.md#talent-ambusher).
  - A blow counts at any reach. A shot, a throw, or a [spell](magic.md) that rolls to hit counts only within
    6 cells ([Long reach](talents.md#talent-long-reach): 9). Nothing without vitals takes the dice (see [Criticals](combat.md#criticals)).
  - It also makes those you catch unawares slower to gather their wits: their initiative check is
    8 harder at 100, 12 at 200.
  - Each sneak attack that hits trains it half the time.
  - Two weapon abilities trigger on a sneak attack that hits: [Knife in the dark](weapon-skills.md#weapon-ability-knife-in-the-dark) ([Daggers](weapon-skills.md#weapon-skill-daggers) 5) leaves
    the foe [bleeding](status-effects.md#bleeding), and [Hunter's shot](fighting-styles.md#style-ability-hunters-shot) ([Marksman](fighting-styles.md#style-marksman) 6) [pins](status-effects.md#status-pinned) it for one or two actions. Neither
    triggers without the dice.

  The whole rule is under [Surprise and sneak attacks](surprise.md).
- **[Initiative](skills.md#skill-initiative)** helps you keep your wits when something falls on you unseen: +8 to the initiative
  check at 100, +12 at 200. When you are [surprised](status-effects.md#status-surprised) all the same, your next action loses less time:
  half the extra at 100, three quarters at 200. Every such check trains it.
- **[Intimidate](skills.md#skill-intimidate)** lets you threaten one foe ([Intimidate](abilities.md#ability-intimidate) in the abilities menu; see
  [Intimidating](skill-uses.md#intimidating)). It makes your threats and your [War cry](abilities.md#ability-war-cry) (the [berserker](classes.md#berserker)'s) harder to stand
  against: the Willpower check they ask is 8 harder at 100 and 12 at 200 (at most 20). A threat
  also frightens longer the more the skill adds. Every threat and every cry trains it, whatever
  comes of it.
- **[Spellcraft](skills.md#skill-spellcraft)** steadies your casting. Every spell you cast with
  your own [mana](mana.md) makes a check after it goes off, and a failed one is a
  [miscast](miscasts.md). The skill adds to that check (+14 at 100, +22 at 200, +40 at 999) and makes a
  miscast milder (10 off the [miscast table](miscasts.md#the-miscast-table)'s total at 100, 20 at
  200). Untrained, you still cast, with nothing added.

[Skill focus](talents.md#talent-skill-focus) makes one skill you are trained in count 25 higher in the table below, the way a
[magic item](magic-items.md)'s bonus does. It works past 100 too, since 100 only slows a skill's [growth](skills.md#getting-better) and is no
ceiling. It stacks with [Master of none](talents.md#talent-master-of-none) and any magic item. Your Skills screen still shows the value
you trained. On most skills the +25 is worth 2 points of what the skill gives below 100, and 1
past it. On [Stealth](skills.md#skill-stealth)'s quiet, it is 15 % less noise below 100 and 7 % past it.

{{#include generated/skill-benefits.md}}

{{#include generated/skill-resist.md}}

## Who trains what

Each [race](races.md) trains one or two skills. Each [class](classes.md) trains a handful: a few of them well (`2d6+10`), the
rest a little (`2d4+4`). A skill both your [race](races.md) and your [class](classes.md) train adds the two. [Humans](races.md#race-human) train no skill of
their own, but they put extra work into one of their [class](classes.md)'s skills, picked at random. The dice are
rolled when your character is made, after your [stats](character.md#how-your-stats-add-up), so they never change a [stat](stats.md).

Each [class](classes.md) also has a few skills its members **may** start with. Each has the chance in the last
column, then its dice, rolled once when your character is made. So two characters of the same
[class](classes.md) can begin with different skills. The game shows only the skills you have, never the chances.

Your [race](races.md)'s skill and the [human](races.md#race-human)'s extra [class](classes.md) skill add to one of these as to any other. But a
level-up roll on one of them is an ordinary roll: the [class](classes.md)'s +1 is only for the skills in its Trains
column. A [race](races.md) may have such a skill too: a [drow](races.md#race-drow) may start with [Sneak attack](skills.md#skill-sneak-attack), whatever their class
(rolled after the class's own).

{{#include generated/skill-grants.md}}

Some [creatures](creatures.md) are trained in skills too: deer listen well, and the [kobold shaman](creatures.md#creature-kobold-shaman) knows something of
[spells](magic.md#the-spells). You never see a [creature](creatures.md)'s skills in the game. They are on
[Creatures in numbers](creature-numbers.md#stats-and-skills).

## Getting better

In short: **using a skill builds up practice, and each new level turns that practice into points.**

**Training marks.** Each time you use a skill in a way that matters, it may earn a **training
mark**. The chance is in the table of skills above. Skills you use all the time, such as
[balance](skills.md#skill-balance), earn marks rarely, so they do not race ahead.

- **[Stealth](skills.md#skill-stealth)**: a step that a [creature](creatures.md) nearby could have heard if you had not stepped softly.
- **[Listen](skills.md#skill-listen)**: [hearing](hearing.md) something new, and every turn spent [listening at a door](hearing.md#listening-at-a-door).
- **[Tumble](skills.md#skill-tumble)**: dodging an attack ("You dodge the kobold's spear."), or stepping past a [trap](traps.md#finding-traps) you know
  of ("You deftly avoid the arrow trap."). A blow that misses you for no reason your [Defence](combat.md#defence)
  explains (it would have missed anyone) counts as a dodge one time in four.
- **[Balance](skills.md#skill-balance)**: doing anything but waiting while your load slows you down (5 %), a step onto rough
  ground that slows you (10 %: bush, reeds, rubble, ice; not water), and an action taken while
  [staggered](status-effects.md#status-staggered) (20 %).
- **[Survival](skills.md#skill-survival)**: a journey on the [overworld](overworld.md#travel).
- **[Herbalism](skills.md#skill-herbalism)**: a forage that runs its time, one time in four, and every
  [gather](herbs.md#gathering) that runs its time.
- **[Mining](skills.md#skill-mining)**: every wall you dig through.
- **[Swim](skills.md#skill-swim)**: a step wading through [water](terrain.md#water) or [marsh](terrain.md#terrain-marsh) now and then (10 %), every [plunge through the ice](temperature.md#falling-in), and a
  quarter of the steps in [deep water](terrain.md#deep-water), made or not.
- **[Steal](skills.md#skill-steal)**: every attempt to steal, whatever comes of it.
- **[Bluff](skills.md#skill-bluff)**: half of your feints, and every try at [talking your way past](encounters.md#talking-past) a party, whatever comes of
  them.
- **[Open lock](skills.md#skill-open-lock)**: a try at picking a lock, whether it works or not.
- **[Heal](skills.md#skill-heal)**: a try at treating yourself, whether it works or not.

Only skills you are trained in earn marks. They gather until a roll on the skill spends them. The
Skills screen's meter shows how practised a skill is (the step its marks have reached on the ladder
below), never the marks.

**Rolls of their own.** When a skill's marks reach 100, it gets a roll at once, by itself, without
waiting for your next level. It is the same roll as a level-up's (below), and it is told the same
way: "Your stealth improves." It spends half the marks, so the skill drops back below 100 and must
earn its next one. It comes on top of your level's rolls, never instead of one, and it does not
wait for a level-up you have not finished.

**Skill rolls.** Every level gives you a number of **skill rolls** (see [Experience](experience.md)). You spend each
one on a trained skill of your choice, before you do anything else. Nothing stops you putting every
roll into one skill.

A roll throws the die the skill's marks have reached on the ladder, adds it to the skill, and
spends half of the skill's marks (rounded up). So a second roll on the same skill that level is a
step or two smaller (50 marks → 25 → 12 → 6 → 3: `3d3`, `2d4`, `1d5`, `1d3`, `+1`). A skill with no
marks still gains 1. Past 100, a roll adds half its result, rounded up. The game says "Your stealth
improves."

- A skill your **[class](classes.md) is known for** (its `2d6+10` skills) throws the die one step up the ladder.
- Any **other skill your [class](classes.md) trains**, or a skill **your [race](races.md) trains**, adds 1 to the roll.

Both stack: a [halfling](races.md#race-halfling) [thief](classes.md#thief)'s [stealth](skills.md#skill-stealth) is both a class specialty and racial, so it rolls one step up and +1.

The level-up screen shows these numbers before you choose. Beside each skill it shows the dice a
roll on it adds (`1d3`, `+2`, `1d5+1`; `(1d3)/2` past 100). For the selected skill it explains how
they came about (`Rolls 1d5+1: practice (20 marks) gives 1d5, +1 as your class trains it.`), with a
line saying where your rolls come from (`4 skill rolls a level: class 3, race 1`). The roll itself
is thrown only when you choose.

{{#include generated/skill-ladder.md}}

How many rolls a level gives: your [class](classes.md)'s, plus your [race](races.md)'s, plus your [Intelligence](stats.md#stat-intelligence)'s.

{{#include generated/skill-rolls.md}}

Only you train, for now. [Creatures](creatures.md) keep the skills they have. Your skills and their marks are saved
with your run.
