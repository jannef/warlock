# Weapon skills

The more you fight with a kind of weapon, the better you get with it. Every kind of weapon is a
**weapon skill** with a **rank** from 0 (untrained) to 10. Each attack you make earns **marks** in
the skill it uses. When you have enough marks you gain a rank: "You feel more skilled with long
blades." A [shield](objects.md#class-shields) has a skill too, for [bashing](weapon-skills.md#weapon-skill-shield-bash) with it.

Each skill grows only by its own use. Someone who is good with a
[long sword](weapon-skills.md#weapon-skill-long-blades) is still a beginner with a short one until
they practise with it.

## The skills

{{#include generated/weapon-skills.md}}

Every weapon belongs to one skill (see the weapon tables in [Combat](combat.md#damage) and
[Ranged combat](ranged.md)). Every throw trains **[thrown weapons](weapon-skills.md#weapon-skill-thrown)**, whatever you throw: a dagger thrown is
not a dagger held. **[Unarmed](weapon-skills.md#weapon-skill-unarmed)** covers fighting with your bare hands, and every [kick](doors.md#kicking) at a creature,
whatever you wield (see [Doors](doors.md)).

No skill trains when your [shield](objects.md#class-shields) stops a blow. The Shield part of your [Defence](combat.md#defence) grows only with
the shield styles, [Sword and Board](fighting-styles.md#style-sword-and-board) and [Two Shields](fighting-styles.md#style-two-shields).

**[Shield bashing](weapon-skills.md#weapon-skill-shield-bash)** is an attack of its own. It bashes whoever stands in a direction with the
shield you hold. It does the shield's own [damage](combat.md#damage) plus your [Strength](stats.md#stat-strength), like every blow, and takes the
time of a blow. You need a shield to bash with, and you bash only what is [hostile](creatures.md#factions) to you. A shield
bash is an *ability*: choose it from the abilities menu (`a`, see [Abilities](abilities.md)), then pick a
direction. It has no key of its own by default. You can give it one in the
[Controls screen](controls.md#rebinding-keys) ("Bash with your shield").

Spells have their own page. Each [school of magic](schools.md) is a skill trained the same way, by
casting, and a spell that rolls to hit aims with its school's rank. The schools and the caster
styles are on the [Magic page](schools.md#the-magic-page), page 6.

## Seeing your skills

`W` opens the **weapon skills screen**. It lists every skill by kind (melee, missile and thrown,
shields), then your [fighting styles](fighting-styles.md). Each shows your rank and a faint bar of your marks
toward the next rank. A `•` marks the styles you fight in now. Opening it takes no time.

Skills you know stand out, and untrained ones are dimmer. In a wide window the list is in two
columns. In a small one it scrolls (↑↓). It is page 5 of the [info pages](the-screen.md#info-pages), right after your
[skills](skills.md) and before the [Magic page](schools.md#the-magic-page) (`6`, named on its bottom line), where the
[caster styles](schools.md#caster-styles) are listed.

The game never tells you in numbers what a rank gives you. You see your ranks, and you feel the
difference in a fight. The tables below are the exact rules.

## Marks and ranks

You earn marks for every attack, and more for a hit. How much you learn depends on whom you
fight. An opponent whose [level](experience.md#levels) is below your rank teaches you less, and eventually nothing at
all. One at your rank or above teaches you the full marks, but never more. Past rank 5, a
[giant rat](creatures.md#creature-rat) (level 1) teaches you nothing. To become a master you need masterful foes.

{{#include generated/weapon-marks.md}}

Marks you earn beyond what a rank needs count toward the next one. The first ranks come within a
few levels. From rank 7 on, each rank asks for far more, and the last ranks take most of a career.
Missile weapons need fewer marks than weapons up close. The schools of magic and the caster
styles, trained by casting, follow a gentler curve of their own. You start every skill with no
marks.

## Target dummies

The [Weyfarers' Gilde](guild.md#the-yard) keeps a yard of training dummies in Cēaptūn, straw men
on posts that never fight back. Walk into one to strike it, or [shoot](ranged.md#shooting-and-throwing)
or throw at it. A dummy never moves, is never hurt, and never dies, so it gives no
[experience](experience.md), loot, or [quest](quests.md) credit, and the townsfolk do not mind
you beating it. It is never [caught unawares](surprise.md#caught-unawares), and no attack on it is a [sneak attack](surprise.md#sneak-attacks).

A dummy teaches only the weapon skill and [fighting style](fighting-styles.md) of the attack,
never a [stat](stats.md#training), and slowly: half the marks a level-1 opponent would give, rounded
(2 for a hit and 1 for a miss at rank 0). It teaches blows, shield bashes, kicks, and throws up to
**rank 2**, and shots with a bow, crossbow, or sling (and their style) up to **rank 4**. When a
skill reaches that rank there, you are told "You have learned all the dummy can teach you.", and
it learns nothing more there. There is no daily limit: the time it takes is the price.

Missiles shot or thrown at a dummy stop there, hit or miss. One in four (25 %) is ruined; the
rest drop on a free cell beside the dummy, to be picked up again. A missile that never breaks is
never ruined.

{{#include generated/target-dummies.md}}

## What a rank gives

Each skill has its own benefits per rank, which apply while you fight with it:

- **[To hit](combat.md#hitting)** and **[Damage](combat.md#damage)** add to every attack with the skill. Damage is added to every roll
  of the dice, so a critical gets it twice or more.
- **[Critical range](combat.md#criticals)** widens the rolls that threaten a critical. A dagger that threatens on 20 to
  24 threatens on 19 to 24 at one wider (see [Criticals](combat.md#criticals)).
- **Time** takes that much off the time of each attack (a quick dagger). **Reload time** takes
  that much off reloading a [crossbow](weapon-skills.md#weapon-skill-crossbows).
- **[Parry](combat.md#defence)** adds to the Parry part of your [Defence](combat.md#defence) while you wield a weapon of the skill.
- **[Shield](combat.md#defence)** adds to the Shield part of your Defence while you hold a shield.

Rank 0 gives nothing. The first five ranks grow slowly. Ranks 6 to 10 grow much faster, above all
in damage, with a wider critical range, quicker attacks, and more Parry. To hit rises only a little
more. Rank 10 is mastery.

{{#include generated/weapon-benefits.md}}

## What a rank teaches

Ranks can also **teach an ability**. You are told so with the rank ("You feel more skilled with
long blades. You have learned the deep cut."). The [abilities menu](abilities.md) (`a`) then lists it
under *Weapon skills*, after the skill that taught it ("Long blades: Deep cut").

Most of these **fire by themselves**. They cost nothing, take no time, and work whenever you fight
with a weapon of the skill, so the menu shows them greyed with "fires by itself". An ability names
its skills: [Deep cut](weapon-skills.md#weapon-ability-deep-cut) belongs to any blade, at rank 5 in that blade's skill. Creatures with the
rank have the abilities too, by the same rules: a spearman of rank 5 braces against you as you
come.

**Active abilities** (marked *active* in the table) are used, not fired. You pick one from the
abilities menu and aim it as it asks: a direction for a blow, a place for a shot, nothing for a
sweep or Shield wall. It costs two things, and only once it goes ahead. A refused use, with nobody
to strike or nothing to shoot, costs nothing.

- **Stamina** (see [Combat](stances.md#stamina)). The menu shows it: "Spears: Lunge  3 stamina, in a direction".
- **Time**: the percentage in the table of the time the action normally takes you. For a blow
  with what you hold, that means your weapon's speed, your [style](fighting-styles.md), and your [stance](stances.md#stances) all count.
  The action may also be a shot or a wait. No active is instant. The game never shows this time.

The menu greys an active out while you are short of stamina ("you are short of breath"), or while
what you hold does not suit it ("your hands are not so held"). [Lunge](weapon-skills.md#weapon-ability-lunge) needs a spear or a staff in hand.
A style's actives need its grip: [Shield wall](fighting-styles.md#style-ability-shield-wall) needs a shield and a weapon, or two shields.

Each active is its own action. Three in three turns cost all three together, and the empty pool is
the price. With a pool of 12 that comes back a point every 12 seconds, a 5-point active comes at
best once every ten turns. A character without stamina gets a full pool from the first rank that
teaches an active ("You learn to pace your breath."). Creatures use no actives.

- **[Lunge](weapon-skills.md#weapon-ability-lunge)** strikes the first foe within 2 cells in the direction. The cell between must be empty
  and passable (a closed [door](doors.md) blocks), and nothing on it is sprung. A foe struck across the empty
  cell brings no free blows (no [riposte](talents.md#talent-riposte), no [cleave](talents.md#talent-cleave)).
- **[Piercing shot](weapon-skills.md#weapon-ability-piercing-shot)**: a hit flies on along its line and attacks the next one there, whoever it
  is, once.
- **[Sunder](weapon-skills.md#weapon-ability-sunder)** and **[Gap shot](fighting-styles.md#style-ability-gap-shot)** ignore all [Protection](combat.md#protection). Against a foe with none, they cost the same
  and do nothing more (the game never tells you a foe's Protection).
- **[Shield wall](fighting-styles.md#style-ability-shield-wall)** ([Sword and Board](fighting-styles.md#style-sword-and-board) or [Two Shields](fighting-styles.md#style-two-shields)) leaves you [Walled](status-effects.md#status-walled) for 5 turns. Your Shield gets +3 on
  top of the shield's own, and every attack you make takes a quarter longer. Walled and [Hasted](status-effects.md#status-hasted)
  multiply.

{{#include generated/weapon-abilities.md}}

The [Early mastery](talents.md#talent-early-mastery) talent teaches every one of them a rank sooner. So do the talents that
make you fight as someone more practised ([Stylist](talents.md#talent-stylist), [Fencing](talents.md#talent-fencing)). A status these abilities leave,
such as [Bleeding](status-effects.md#bleeding) or [Staggered](status-effects.md#status-staggered), is a [status effect](status-effects.md) like any other: a [staggered](status-effects.md#status-staggered) foe's next
action takes half again as long. A brace and a point-blank shot are [free blows](stances.md#free-blows), so they bring
no free blows of their own.

## Where your skills come from

You start with some skills from your [race](races.md) and your [class](classes.md). A [warrior](classes.md#warrior) knows [long blades](weapon-skills.md#weapon-skill-long-blades),
a [ranger](classes.md#ranger) knows [bows](weapon-skills.md#weapon-skill-bows), and a [thief](classes.md#thief) knows [daggers](weapon-skills.md#weapon-skill-daggers). Some starting ranks are dice (`1d2` is one or two),
rolled when you start. A [berserker](classes.md#berserker) has learned either [axes](weapon-skills.md#weapon-skill-axes) or great blades, and the dice decide
which. Your race's and your class's ranks add up.

Creatures have skills too: the [kobolds](creatures.md#creature-kobold) of the war band know their [spears](weapon-skills.md#weapon-skill-spears) and [bows](weapon-skills.md#weapon-skill-bows). They do not
get better (yet).
