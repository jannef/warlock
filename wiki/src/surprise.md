# Surprise and sneak attacks

A blow from the dark lands harder than one in plain sight, and whoever takes it is caught off
guard. The rule works both ways: a creature can catch you unawares exactly as you can catch it.

The game never shows the check, the dice, or the odds. It only tells you what came of them. The
numbers are below.

## Unaware

- A **creature is unaware of you** while its eyes do not see you and it is not hunting anyone it
  has seen. A sleeper is always unaware, and so is a [mimic](mimics.md) still in disguise. A
  creature still searching for someone it lost sight of is not unaware.
- **You are unaware of a creature** you do not see now and have not seen for a while. The table
  below says how long you stay aware of one. An attack that catches you unawares counts as seeing
  its attacker. So a creature you cannot see surprises you with its first blow only, until you
  have forgotten it again. A creature struck from the dark, on the other hand, stays unaware of
  whoever struck it, so each shot from the dark catches it unawares again.
- **Slipping past**: in the dark, a creature that would see you only by its darkvision may miss
  you altogether. Your [Dexterity](stats.md#stat-dexterity) and [Stealth](skills.md#skill-stealth)
  are tested against its [Willpower](stats.md#stat-willpower) and [Spot](skills.md#skill-spot).
  While you have slipped past it, it is unaware of you. The odds, and what ends it, are under
  [Slipping past](skill-uses.md#slipping-past).

Nothing tells you which creatures have not noticed you, unless you have
[Stalker's eye](class-ranger.md#talent-stalkers-eye) (the [ranger](class-ranger.md)'s, from level 2). With it, look mode says
"unaware of you", and such creatures are drawn with a slow, faint shimmer.

After you attack, though, the attack's own message tells you how it found a target you see (see
[What the attack says](#what-the-attack-says)).

## Caught unawares

An attack that finds its target unaware **surprises** it. That covers a blow, a shot, a throw, or
a [spell](magic.md) that rolls to hit. A hidden [trap](traps.md) also surprises whoever it springs
under: you, if you had not found it, or a creature, since creatures never know where traps are. A
trap springing beside you does not.

- The attack itself is struck **flat-footed**: the target's [Dexterity](stats.md#stat-dexterity)
  and its Dodge count for nothing in its [Defence](combat.md#defence).
- Then comes the **initiative check**. The target rolls 2d12 plus its Dexterity modifier, plus
  what [Wary](talents.md#talent-wary) and the [Initiative](skills.md#skill-initiative) skill add.
  It must reach 13 plus the attacker's **craft**. Craft is what the attacker's
  [Sneak attack](skills.md#skill-sneak-attack) skill adds, plus 2 for the thief's
  [Sneak attack](class-thief.md#talent-sneak-attack) talent. A trap has no craft. A sleeper or a mimic
  in disguise makes no check: it is always surprised.
- **Made**: "You keep your wits about you." (or "The kobold whirls to face you."). Nothing more
  happens. With [Hair trigger](talents.md#talent-hair-trigger), if the attacker is a hostile beside
  you, you strike it at once, before its attack is rolled.
- **Failed**: "You are taken by surprise!" (or, for your attack, "You hit the unsuspecting
  kobold."). The target is
  [Surprised](status-effects.md#status-surprised) until it next acts (after a
  [feint](skill-uses.md#feinting), for one action more). While Surprised:
  - it stays flat-footed, with no Dexterity and no Dodge in its Defence;
  - it makes no free blow: no riposte, no blow at a foe stepping up or away, no
    [Point-blank shot](weapon-skills.md#weapon-ability-point-blank);
  - its next action takes half as long again. The [Initiative](skills.md#skill-initiative)
    skill saves part of that extra time, and [Cool head](talents.md#talent-cool-head) saves all of
    it. That action ends the surprise: "You recover your wits."

{{#include generated/surprise-check.md}}

Your **edge** is your Dexterity modifier plus what Wary and Initiative add. The columns are the
attacker's craft and the difficulty it makes. A roll of 2 always fails, and a roll of 24 always
succeeds.

### Stunned

The ascetic's [Stunning strike](class-ascetic.md#talent-stunning-strike) leaves a foe it wounds
[Stunned](status-effects.md#status-stunned) until it next acts. Stunned is flat-footed just like
Surprised: no Dexterity and no Dodge in Defence, no free blow, and
[sneak dice](#sneak-attacks) land on it. Its next action also takes half as long again, but
neither [Initiative](skills.md#skill-initiative) nor [Cool head](talents.md#talent-cool-head)
saves any of that time. They help against surprise, not against a blow to the head.

### What the attack says

Your own attack that rolls to hit (a blow, a shot, a throw, or a [spell](magic.md)) names its
target by how it caught it, in the attack's own line, hit or miss:

| The target was | The message |
|---|---|
| asleep | "You hit the sleeping kobold." |
| unaware of you, and failed its [initiative check](#caught-unawares) (or a mimic in disguise) | "You hit the unsuspecting kobold." |
| already [Surprised](status-effects.md#status-surprised) | "You hit the surprised kobold." |
| [Stunned](status-effects.md#status-stunned) | "You miss the stunned kobold." |

These are exactly the times your [sneak dice](#sneak-attacks) can land, except one: an unaware
target that makes its check whirls to face you ("The kobold whirls to face you."), and that line
is all you are told, though the attack was still struck flat-footed. Nothing is said of a target
you do not see, and never a number. A creature's attack on you says nothing of the kind: "You
are taken by surprise!" already tells it.

## Sneak attacks

If you have **sneak dice**, you add them to a hit on a target that is unaware of you, Surprised, or
[Stunned](#stunned). The dice come from the [Sneak attack](skills.md#skill-sneak-attack) skill (1d6
once you are trained at all) and from talents: [Ambusher](talents.md#talent-ambusher) and the
thief's [Improved sneak attack](class-thief.md#talent-improved-sneak-attack). They are added after
[Protection](combat.md#protection), and a [critical](combat.md#criticals) never doubles them.
[Master thief](class-thief.md#talent-master-thief) makes such a hit a critical threat 2 sooner.

- A blow counts at any reach. A shot, a throw, or an attack spell counts only within a few cells
  (see the table below; [Long reach](talents.md#talent-long-reach) adds to it).
- Usually only the first blow of a fight finds the foe unaware. A foe that failed its check is
  still Surprised for your next blow. One that made it is not.
- The thief's [Lurk](class-thief.md#talent-lurk) waits in ambush: a hostile that steps up beside you
  takes your blow at once, caught unawares unless it was already hunting you.
- [Exploit](openings.md#exploit) is a sneak attack whatever the foe knows, but it does not
  surprise the foe.
- [Vanishing act](class-thief.md#talent-vanishing-act) leaves you light on your feet after a sneak
  attack that kills.
- Every sneak attack that hits gives the thief an [opening](resources.md#resource-openings),
  if it is on their [mark](openings.md#the-mark) or they have none.
- Two weapon abilities trigger on a sneak attack that hits:
  [Knife in the dark](weapon-skills.md#weapon-ability-knife-in-the-dark) and
  [Hunter's shot](fighting-styles.md#style-ability-hunters-shot).

{{#include generated/surprise-rules.md}}

## The skills

This table shows what [Sneak attack](skills.md#skill-sneak-attack), [Initiative](skills.md#skill-initiative), and
[Stealth](skills.md#skill-stealth)'s slipping past give at each value. A thief starts with Sneak
attack (`2d6+10`), and a [drow](races.md#race-drow) may too, whatever their class. Warriors,
rangers, ascetics, berserkers, bards, and thieves train Initiative (see
[Which skills you start with](skills.md#which-skills-you-start-with)).

{{#include generated/surprise-ladders.md}}

## No vitals

Some things have no vitals to find. Sneak attack dice never land on them, and a
[critical](combat.md#criticals) is never confirmed on them. [Vital strike](weapon-skills.md#weapon-ability-vital-strike)
and a master thief's sure critical are a plain blow against them ("It has no vital spot."). They can
still be surprised like anyone, and still be struck flat-footed. The fleshy undead (draugr, ghouls,
the drowned, vampires, mummies) do have vitals.

{{#include generated/surprise-vitals.md}}

## Creatures

Some creatures are good at surprise too. They may have sneak dice, a craft that makes your check
harder, an Initiative of their own, or a sharp eye against your slipping past.

A few creatures **lurk**. One that sees you waits where it stands instead of coming at you, and
attacks once you come beside it. If it can shoot or cast, it does that from where it stands. Once
it is hurt, it hunts like any other.

{{#include generated/creature-surprise.md}}
