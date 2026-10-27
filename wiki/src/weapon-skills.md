# Weapon skills

The more you fight with a kind of weapon, the better you get with it, as in ADOM. Every kind of
weapon is a **weapon skill** with a **rank** from 0 (untrained) to 10. Each attack you make
earns **marks** in the skill it uses, and when you have enough marks you gain a rank: "You feel
more skilled with long blades." Shields are skills too: one for blocking with a shield, one for
bashing with it.

Skill with one weapon carries over to related ones, as in the tabletop game *The Riddle of
Steel*: a swordsman who is good with a long sword is not a beginner with a short one.

## The skills

{{#include generated/weapon-skills.md}}

Every weapon belongs to one skill (see the weapon tables in [Combat](combat.md#damage) and
[Ranged combat](ranged.md)). Every throw trains **thrown weapons**, whatever you throw: a dagger
thrown is not a dagger held. **Unarmed** is fighting with your bare hands, and every kick at a
creature, whatever you wield (see [Doors](doors.md)).

**Shield blocking** trains when your shield stops a blow: when the message says "You block the
kobold's spear with your shield." It adds to the Shield part of your [Defence](combat.md#defence)
while you hold a shield. **Shield bashing** is an attack of its own: it bashes whoever stands
in a direction with the shield you hold, for the shield's own damage (and your Strength, as
every blow), in the time of a blow. You need a shield to bash with, and you bash only what is
hostile to you. A shield bash is an *ability*: choose it from the abilities menu (`a`, see
[Abilities](abilities.md)), then a direction. It has no key of its own by default; give it one in
the Controls screen ("Bash with your shield") if you like.

## Seeing your skills

`W` opens the **weapon skills screen**: every skill, by kind (melee, missile and thrown,
shields), then your [fighting styles](fighting-styles.md), each with your rank and a faint bar of
your marks toward the next one; a `•` marks the styles you fight in now. It takes no time.
Skills you know stand out; untrained ones are dimmer. In a wide window the list is in two
columns; in a small one it scrolls (↑↓). It is page 5 of the
[info pages](controls.md#info-pages), right after your skills.

The game never tells you what a rank gives you, in numbers: you see your ranks, and you feel the
difference in a fight. The tables below are the rules, exactly.

## Marks and ranks

You earn marks for every attack, more for a hit. How much you learn depends on whom you fight:
an opponent whose [level](experience.md) is above your rank teaches you more, one below it
less, and at last nothing at all: past rank 5 a giant rat (level 1) teaches you nothing more,
and to become a master you need masterful foes.

{{#include generated/weapon-marks.md}}

The marks you earn past what a rank needs count toward the next one. Missile weapons need fewer
marks than weapons up close. You start every skill with no marks.

## Related skills

When you gain a rank in a skill, every related skill is raised to your new rank less the
penalty between the two, if that is more than you have in it: long blades 8 gives short blades
6. A skill raised this way starts its marks again from 0. The relations go both ways, and only
one step: a rank in long blades raises short blades, but not, through them, thrown weapons.
Unrelated skills (bows and axes) never move each other.

{{#include generated/weapon-defaults.md}}

## What a rank gives

Each skill has its own benefits per rank, applied while you fight with it:

- **To hit** and **Damage** add to every attack with the skill (damage to every roll of the
  dice, so a critical gets it twice or more).
- **Critical range** widens the rolls that threaten a critical: a dagger that threatens on 20
  to 24 threatens on 19 to 24 one wider (see [Criticals](combat.md#criticals)).
- **Time** takes that much off the time of each attack (a quick dagger), **Reload time** off
  reloading a crossbow.
- **Parry** adds to the Parry part of your Defence while you wield a weapon of the skill.
- **Shield** adds to the Shield part of your Defence while you hold a shield.

Rank 0 gives nothing. The first five ranks grow slowly; ranks 6 to 10 grow much faster, in
damage above all, with a wider critical range, quicker attacks, and more Parry, while to hit
rises only a little more. Rank 10 is mastery; later, the masters of a weapon will learn feats of
their own.

{{#include generated/weapon-benefits.md}}

## Where your skills come from

You start with some skills from your [race](races.md) and your [class](classes.md): a warrior
knows long blades, a ranger bows, a thief daggers. Some are dice (`1d2` is one or two), rolled
when you start, and a barbarian learned either axes or great blades: which one, the dice decide.
Your starting ranks then raise their related skills, as above. Your race's and your class's
ranks add up.

Creatures have skills too: the kobolds of the war band know their spears and bows. They do not
get better (yet).
