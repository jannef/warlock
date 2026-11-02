# Surprise and sneak attacks

A blow from the dark lands harder than one in plain sight, and whoever takes it is caught off
guard. The rule is the same both ways round: a creature can catch you unawares exactly as you
can catch it.

The game never shows the check, the dice, or the odds: it tells you what came of them. The
numbers are below.

## Unaware

- A **creature is unaware of you** while it hunts no one it has seen and its eyes do not see
  you now. A sleeper always is, and so is a [mimic](mimics.md) still in disguise. A creature
  that is still searching for someone it lost sight of is not.
- **You are unaware of a creature** you do not see now and have not seen for a while (the table
  below: how long you stay aware of one). An attack that catches you unawares counts as seeing
  its attacker, so a creature you cannot see surprises you with its first blow only, until you
  have forgotten it again. A creature struck from the dark stays unaware of whoever struck it, so
  each shot from the dark catches it unawares again.
- **Slipping past**: in the dark, a creature that would see you only by its darkvision may miss
  you altogether: your [Dexterity](stats.md#stat-dexterity) and [Stealth](skills.md#skill-stealth)
  against its [Willpower](stats.md#stat-willpower) and [Spot](skills.md#skill-spot). While you
  have slipped past it, it is unaware of you. The odds and what ends it are under
  [Slipping past](skills.md#slipping-past).

With [Stalker's eye](talents.md#talent-stalkers-eye), the [ranger](classes.md#ranger)'s from level 2, you can tell which creatures in
view have not noticed you: look mode says "unaware of you", and they are drawn with a slow,
faint shimmer. Without it nothing tells you.

## Caught unawares

An attack that finds its target unaware **surprises** it: a blow, a shot, a throw, or a
[spell](magic.md) that rolls to hit. So does a hidden [trap](traps.md) springing under you (one
you had not found) or under a creature (they never know where a trap is); never one springing
beside you.

- The attack itself is struck **flat-footed**: the target's [Dexterity](stats.md#stat-dexterity)
  and its Dodge count nothing toward its [Defence](combat.md#defence).
- Then comes the **initiative check**: 2d12 plus the target's Dexterity modifier, plus what
  [Wary](talents.md#talent-wary) and the [Initiative](skills.md#skill-initiative) skill add,
  against 13 plus the attacker's **craft**: what its [Sneak attack](skills.md#skill-sneak-attack)
  skill adds, and the thief's [Sneak attack](talents.md#talent-sneak-attack) talent (+2). A trap
  has no craft. A sleeper or a mimic in disguise makes no check: it is always surprised.
- **Made**: "You keep your wits about you." (or "The kobold whirls to face you."). Nothing
  more. With [Hair trigger](talents.md#talent-hair-trigger), when the one who tried it is a
  hostile beside you, you strike it at once, before its attack is rolled.
- **Failed**: "You are taken by surprise!" (or "You catch the kobold unawares."), and the target
  is [Surprised](status-effects.md#status-surprised) until it next acts:
  - it stays flat-footed: no Dexterity and no Dodge in its Defence;
  - it makes no free blow: no riposte, no blow at a foe stepping up or away, no
    [Point-blank shot](weapon-skills.md#weapon-ability-point-blank);
  - its next action costs half as much time again. The [Initiative](skills.md#skill-initiative)
    skill spares part of that extra time and [Cool head](talents.md#talent-cool-head) all of it.
    That action ends it: "You recover your wits."

{{#include generated/surprise-check.md}}

Your **edge** is your Dexterity modifier plus what Wary and Initiative add. The columns are the
attacker's craft and the difficulty it makes; a roll of 2 always fails and a roll of 24 always
works.

## Sneak attacks

Whoever has **sneak dice** adds them to a hit on a target unaware of them or Surprised. The dice
come from the [Sneak attack](skills.md#skill-sneak-attack) skill (1d6 once you are trained at
all) and from talents: [Ambusher](talents.md#talent-ambusher) and the thief's
[Improved sneak attack](talents.md#talent-improved-sneak-attack). They are added after
[Protection](combat.md#protection) and are never doubled by a [critical](combat.md#criticals).
[Master thief](talents.md#talent-master-thief) makes such a hit a critical threat 2 sooner.

- A blow counts at any reach. A shot, a throw, or an attack spell counts only within a few cells
  (the table below; [Long reach](talents.md#talent-long-reach) adds to it).
- Only the first blow of a fight usually finds the foe unaware. A foe that failed its check is
  still Surprised for the next one; one that made it is not.
- The thief's [Lurk](talents.md#talent-lurk) waits in ambush: a hostile that steps up beside you
  takes your blow at once, caught unawares unless it was already hunting you.
  [Low blow](talents.md#talent-low-blow) is a sneak attack whatever the foe knows, but does not
  surprise it. [Vanishing act](talents.md#talent-vanishing-act) leaves you light on your feet
  after a sneak attack that kills. Every sneak attack that hits gives the thief an
  [opening](resources.md#resource-openings).
- Two weapon abilities fire on a sneak attack that hits:
  [Knife in the dark](weapon-skills.md#weapon-ability-knife-in-the-dark) and
  [Hunter's shot](fighting-styles.md#style-ability-hunters-shot).

{{#include generated/surprise-rules.md}}

## The skills

What [Sneak attack](skills.md#skill-sneak-attack), [Initiative](skills.md#skill-initiative), and
[Stealth](skills.md#skill-stealth)'s slipping past give by value. A thief starts with Sneak
attack (`2d6+10`), and a [drow](races.md#race-drow) may, whatever their class; Initiative is
trained by warriors, rangers, monks, barbarians, bards, and thieves (see
[Who trains what](skills.md#who-trains-what)).

{{#include generated/surprise-ladders.md}}

## No vitals

Some things have no vitals to find: no sneak attack dice land on them, and no
[critical](combat.md#criticals) is ever confirmed on them. [Vital strike](weapon-skills.md#weapon-ability-vital-strike)
and the master thief's finisher are a plain blow on them ("It has no vital spot."). They are
still surprised like anyone, and still struck flat-footed. The fleshy undead (draugr, ghouls,
the drowned, vampires, mummies) have vitals.

{{#include generated/surprise-vitals.md}}

## Creatures

Some creatures are good at it too: they have sneak dice, a craft that makes your check harder,
an Initiative of their own, or a sharp eye against your slipping past. A few **lurk**: one that sees
you waits where it stands rather than come at you, and attacks once you come beside it (or
shoots or casts from where it stands, if it can); once it is hurt, it hunts like any other.

{{#include generated/creature-surprise.md}}
