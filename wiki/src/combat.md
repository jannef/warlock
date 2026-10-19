# Combat

Fights are decided by dice, as in Dungeons & Dragons, but with two twelve-sided dice instead of
one twenty-sided die, and with ADOM's kind of hit points. Every blow is rolled, can miss, can hit
hard, and hurts; what loses all its hit points dies. That goes for you too, and death is final.

You attack by moving into a hostile creature (see [Controls](controls.md)), which takes a turn,
by bashing it with your shield, or from afar: you shoot or throw with `t` (or `T`), see
[Ranged combat](ranged.md). Each attack trains the [weapon skill](weapon-skills.md) it uses, and
your rank in it helps you hit, hurt, and defend.
Creatures fight each other and you by the same rules (see [Creatures](creatures.md)). Spells are
the exception: casters only announce them for now, and nothing is hurt by a spell.

## Hitting

An attack rolls two twelve-sided dice and adds them: 2 to 24, most often around 13. It hits when
that roll, plus the attacker's bonus to hit, reaches the target's **Defence**.

- Fighting up close, your **Strength** helps you hit; shooting, your **Dexterity** does. Some
  weapons are easier to hit with than others, and your rank in the weapon's
  [skill](weapon-skills.md) helps the more you have trained.
- **Two ones always miss, two twelves always hit**, whatever the odds.
- Because the sum of two dice gathers around the middle, each point of bonus or Defence matters
  most when the odds are close, and little when they are long either way:

{{#include generated/hit-chances.md}}

"The roll needs" is the target's Defence less the attacker's bonus. Against an unarmoured,
unremarkable target (Defence 10) an attacker with no bonus hits three times in four.

The game never shows you your bonus to hit, the dice, or the rolls: you see what happens.

## Defence

Your **Defence** (shown as `Dv` on the [character screen](character.md), `@`) is 10 plus what
protects you, in kinds:

- **Dexterity**: quick feet. A clumsy character's Defence is lower than 10. Heavy armour limits
  how much Dexterity can add (see the table below), and so does a heavy load: at most +3 when
  you are burdened, +1 when strained, nothing when overloaded (see [Burden](burden.md)). The
  lower limit counts.
- **Armour**: what you wear on your body.
- **Shield**: a shield held in your off hand, and more with your rank in shield blocking.
- **Parry**: a weapon that turns blows aside, such as the quarterstaff, and your rank with the
  weapon you wield (blades, spears, and staves).
- **Natural**: a creature's hide, shell, or scales. You have none, unless a magic item gives you
  some.
- **Dodge**: training and magic. The [tumble](skills.md#what-skills-give) skill adds to it, more
  the higher it is, and so do magic items that make you nimble.
- **Deflection**: magic items that turn blows aside.

Within each kind only the best counts: two pieces of armour do not add up. Dodge is the one kind
that does.

{{#include generated/armour.md}}

Helms, cloaks, gloves, boots, rings, and amulets do not add to Defence by themselves, only by
their magic (see [Magic items](magic-items.md)), and magic armour or a magic shield may add more
than its base.

### Why an attack missed

When an attack misses, the message says why, as far as you saw it:

- A roll so poor it would have missed a target standing still in no armour at all is a plain
  miss: "The kobold misses you."
- Otherwise your Defence turned it, and one kind is picked, more likely the more it adds: "You
  dodge the kobold's spear." (Dexterity), "The kobold's spear glances off your armour.", "You
  block the kobold's spear with your shield.", "You parry the kobold's spear.", "Your sword
  glances off the cave beetle's shell."
- A clumsy character's low Dexterity lowers their Defence, but is never why an attack missed.
- If you do not see one of the two, a miss is just a miss: "Something misses you."

## Criticals

A hit whose dice come up high enough **threatens a critical**: most weapons threaten on 22 to
24, swords and daggers on 20 to 24. A second roll, against the same Defence, decides: if it would
hit too, the critical is **confirmed** and the damage is rolled again and added, twice in all for
most weapons, three times for spears and bows. The message says so: "The kobold hits you
hard!" A confirmed critical also opens a bleeding wound (see [Status effects](status-effects.md)).

{{#include generated/threats.md}}

## Damage

A hit does the weapon's damage dice, at least 1:

- **Up close**, your **Strength** adds to the damage, as it does to hitting. With nothing in your
  hands you hit with your fists.
- **Shots** do the bow's own damage: Dexterity aims them, but adds nothing to the damage. A bow
  is drawn by strength, though: each point of Strength you lack of what the bow needs costs you
  a point to hit and a point of damage.
- Creatures without a weapon bite, claw, or kick, each in its own way.
- Your rank in the weapon's [skill](weapon-skills.md) adds to the damage, and at higher ranks
  widens the critical range.
- A **shield bash** does the shield's own damage (below), plus your Strength.

{{#include generated/weapons.md}}

Messages never give numbers: "The kobold hits you.", "The giant rat bites the kobold.", "You hit
the giant rat hard!"

A **big hit**, one that takes at least a third of the target's hit points at once, makes it
bleed, and some creatures' attacks can poison: see [Status effects](status-effects.md). Some bites
carry a disease: see [Diseases](diseases.md).

## Stances

Like ADOM's, your fighting **stance** trades attack for defence. Press `+` to take a more aggressive stance
(coward, defensive, neutral, aggressive: it stops at the end) and `-` for a more defensive one; each takes no time, and you keep it
until you change it, even across a save. "You take an aggressive stance." You start neutral. The
status bar names your stance when it is not neutral, and the [character screen](character.md)
always does. The game never shows what a stance adds: you feel it in the fight.

- **Aggressive**: you hit more often and harder, but are easier to hit. (Weapon skill marks will
  count for more: you swing harder.)
- **Neutral**: no effect.
- **Defensive**: you hit less often and softer, but dodge better, and parry a little better if
  your weapon does not already turn blows aside as well.
- **Coward**: you hit much less often and softer, and your attacks take longer, but you dodge
  best, and stepping away from a hostile creature next to you is quicker and quieter.

The numbers are first guesses, tuned by playtesting:

{{#include generated/stances.md}}

Dodge adds to your Defence on top of everything else; Parry only counts when it beats what your
weapon already gives. Creatures do not take stances yet.

## Hit points

Your **hit points** are your Toughness plus what your [race](races.md) and your
[class](classes.md) give:

{{#include generated/hit-points.md}}

A dwarf barbarian with Toughness 16 has 6 + 8 + 16 = 30 hit points; an elf wizard with Toughness
8 has 10. You start every run unhurt. The status bar shows your hit points (`HP 18/24`), and so
does the character screen.

Creatures have hit points too, but you never see their numbers. [Look mode](controls.md) says how
hurt a creature looks: unhurt, lightly wounded, wounded, badly wounded, or nearly dead.

### Getting better

Hit points come back with time, faster the tougher you are, twice as fast while you
[rest](resting.md), and slower or not at all while you bleed, are poisoned, or are diseased: see
[Healing](healing.md). A disease that lowers your Toughness lowers your most hit points too (see
[Diseases](diseases.md)). Creatures heal the same way, while you are on their level. In a
sanctuary (a [special room](special-rooms.md)) wounds heal three times as fast, yours and
anyone's.

## Death

A creature at 0 hit points dies: "The giant rat dies." It is gone, and everything it carried,
what it wore and wielded too, falls where it stood, for you to pick up, with its corpse on top
(see [Food and corpses](food.md)). A wounded creature may also die later of its wounds: "The
giant rat bleeds to death."

When **you** reach 0 hit points, you die: "You die..." The run is over, and a screen tells you
what killed you (if you saw it, or "Bled to death" when your wounds did, or that you starved; see
[Hunger](hunger.md)), where, when, and after how many turns. There is no going back:
your save is deleted at once (see [Saving](saving.md)). Press `Enter` or `Esc` to return to the
main menu and start a new run.

## Blood

Big hits and kills leave blood where they land. A blow that takes at least a third of its
target's hit points, or any critical ("hits you hard!"), leaves a little spatter; a kill leaves a
pool where the victim fell and a spray behind it, away from whoever struck the blow. How much
depends on how big the victim was and how it died: a blow far past what killed it, or a critical,
spills more; an arrow or a thrown weapon less, sprayed along its flight. Blood that sprays into
a wall right beside the victim stains that wall on the side facing it.

Blood adds up: one kill is easy to miss, but where many died the floor is soaked. Fresh blood is
red, and it dries darker and browner over a few hours; the cave beetle bleeds a greenish-black
ichor. Water, lava, and fire take no stains. You see blood where you see the floor (on a wall,
only from the side that was splattered), remember it as you last saw it, and look mode mentions
it: "on blood-stained stone floor".

Blood is only for show: it never changes a fight, and the Blood switch in the
[settings](settings.md) hides it.
