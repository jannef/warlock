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

**Spell attacks** is the skill of spells that roll to hit: a ray, a bolt, or a touch that can
miss (see [Magic](magic.md#how-a-spell-finds-its-target)). Every such spell you cast with your
own mana that reaches a creature earns marks, more for a hit, as a blow does; its ranks add to
the spell's to-hit, widen its critical range, and at the top make a critical roll the damage
once more (still held to the spell's most). A spell read from a scroll or used from a wand aims
with Dexterity alone and trains nothing; spells that never miss train nothing either. The
[caster styles](fighting-styles.md#caster-styles) add to it.

## Seeing your skills

`W` opens the **weapon skills screen**: every skill, by kind (melee, missile and thrown,
shields, spells), then your [fighting styles](fighting-styles.md), each with your rank and a faint bar of
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
rises only a little more. Rank 10 is mastery.

{{#include generated/weapon-benefits.md}}

## What a rank teaches

Abilities come with ranks too. A rank may **teach an ability**: you are told so with the rank
("You feel more skilled with long blades. You have learned the deep cut."), and the
[abilities menu](abilities.md) (`a`) lists it under *Weapon skills*, after the skill that taught
it ("Long blades: Deep cut"). The ones below **fire by themselves**: they cost nothing, take no
time, and work whenever you fight with a weapon of the skill, so the menu shows them greyed with
"fires by itself". An ability names its skills: Deep cut is any blade's, at rank 5 in that
blade's skill. Creatures with the rank have them too, by the same rules: a spearman of rank 5
braces against you as you come.

**Active abilities** (marked *active* in the table) are used, not fired: you pick one from the
abilities menu, aim it as it asks (a direction for a blow, a place for a shot, nothing for a
sweep or Shield wall), and it costs two things, both only once it goes ahead (a refused use, with
nobody to strike or nothing to shoot, costs nothing):

- its **stamina** (see [Combat](combat.md#stamina); the menu shows it: "Spears: Lunge  3 stamina, in a direction"), and
- its **time**: the percent in the table of what the action it is takes you (a blow with what you
  hold, so your weapon's speed, your style, and your stance count; a shot; a wait). No active is
  instant. The game never shows this time.

The menu greys an active out while you are short of stamina ("you are short of breath") or while
what you hold is not for it ("your hands are not so held": Lunge needs a spear or a staff in
hand, Shield wall a shield, a style's actives its grip). Each active is its own action: three in
three turns cost the three together, and the empty pool is the price; with a pool of 12 that
comes back a point every 12 seconds, a 5-point active comes at best once every ten turns. A
character without stamina is given it, full, by the first rank that teaches an active ("You
learn to pace your breath."). Creatures use no actives.

- **Lunge** strikes the first foe within 2 cells along the direction; the cell between must be
  empty and passable (a closed door blocks), and nothing on it is sprung. A foe struck over the
  empty cell brings no free blows (no riposte, no cleave).
- **Piercing shot**: a hit flies on along its line and attacks the next one there, whoever it is,
  once.
- **Sunder** and **Gap shot** ignore all Protection; against a foe with none they cost the same
  and do nothing more (the game never tells you a foe's Protection).
- **Shield wall** leaves you Walled for 5 turns: your Shield +3 on top of the shield's own, and
  every attack you make takes a quarter longer. Walled and Hasted multiply.

{{#include generated/weapon-abilities.md}}

The [Early mastery](talents.md) talent teaches every one of them a rank sooner, and so do the
talents that make you fight as one more practised than you are (Stylist, Fencing). A status they
leave (Bleeding, Staggered) is a [status effect](status-effects.md) like any other: a staggered
foe's next action takes half again as long. A brace and a point-blank shot are
[free blows](combat.md#free-blows): they bring none of their own.

## Where your skills come from

You start with some skills from your [race](races.md) and your [class](classes.md): a warrior
knows long blades, a ranger bows, a thief daggers. Some are dice (`1d2` is one or two), rolled
when you start, and a barbarian learned either axes or great blades: which one, the dice decide.
Your starting ranks then raise their related skills, as above. Your race's and your class's
ranks add up.

Creatures have skills too: the kobolds of the war band know their spears and bows. They do not
get better (yet).
