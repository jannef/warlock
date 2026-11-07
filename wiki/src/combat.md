# Combat

Fights are decided by dice: two twelve-sided dice for every attack. Every blow is rolled. It can
miss, it can hit hard, and it hurts. Whatever loses all its [hit points](hit-points.md#hit-points) dies. That goes for you too, and
[death](hit-points.md#death) is final. How hit points work, how they come back, and what happens when you die is on
[Hit points and death](hit-points.md), and stances, free blows, and stamina are on
[Stances, free blows, and stamina](stances.md).

You attack by moving into a hostile creature (see [Controls](controls.md)), which takes a [turn](time.md#what-takes-time).
You can also [bash](abilities.md#ability-shield-bash) it with your [shield](objects.md#class-shields), or shoot or throw from afar with `t` (or `T`; see
[Ranged combat](ranged.md)).

Each attack trains the [weapon skill](weapon-skills.md) it uses and the [fighting style](fighting-styles.md) you fight in. Your
[ranks](weapon-skills.md#marks-and-ranks) in them help you hit, hurt, and defend. With a [weapon in each hand](fighting-styles.md#two-weapons), an attack up close is a
blow with each. Creatures fight each other and you by the same rules (see [Creatures](creatures.md)), and
casters cast the same [spells](magic.md#the-spells) you do (see [Spells](creature-tricks.md#spells)).

## Hitting

An attack rolls two twelve-sided dice and adds them, for 2 to 24, most often around 13. It hits
when that roll plus the attacker's bonus to hit reaches the target's **[Defence](combat.md#defence)**.

- Up close, your **[Strength](stats.md#stat-strength)** helps you hit. When shooting, your **[Dexterity](stats.md#stat-dexterity)** does. With a
  [finesse weapon](#finesse-weapons) up close, Dexterity helps you hit and Strength does not, even if your Strength is
  higher. Some weapons are easier to hit with than others, and your [rank](weapon-skills.md#marks-and-ranks) in the weapon's
  [skill](weapon-skills.md) helps more the more you have trained.
- **Two ones always miss, and a roll of 22 or better always hits.** 22, 23, and 24 are 6 of the
  144 ways the dice can fall, about one blow in 24. However high a [Defence](combat.md#defence) climbs, some blows
  still land.
- The sum of two dice gathers around the middle. So each point of bonus or Defence matters most
  when the odds are close, and little when they are long either way:

{{#include generated/hit-chances.md}}

"The roll needs" is the target's Defence minus the attacker's bonus. Against an unarmoured,
ordinary target ([Defence](combat.md#defence) 10), an attacker with no bonus hits three times in four.

The game never shows you the rolls. But the [character screen](character.md) (`@`) shows your own
attacks beside your stats:

- the blow of your main hand (or bare hands);
- your off hand's blow, when you fight with two weapons;
- the shot or throw of your [missile weapon](ranged.md#getting-ready), with its range in steps.

Each shows its bonus to hit and its [damage](combat.md#damage) (`+4 to hit, 1d8+2 slashing`). These count
everything that goes into the attack as you stand now: your [stats](stats.md), your [ranks](weapon-skills.md#marks-and-ranks) in its
[weapon skill](weapon-skills.md) and in the [fighting style](fighting-styles.md) you hold it in, [two-weapon](fighting-styles.md#two-weapons) penalties, your [stance](stances.md#stances), your
[talents](talents.md), and the magic you know about. A [magic item](magic-items.md)'s unknown powers count only once it is
[identified](magic-items.md#unidentified-items). Anything that depends on the moment, such as striking from hiding or being
dazzled, is left out.

This is exactly how an attack is rolled, and what adds up to its bonus to hit and its damage:

{{#include generated/to-hit.md}}

A creature's attacks add up the same way. Their numbers are on
[Creatures in numbers](creature-numbers.md#attacks).

## Finesse weapons

A finesse weapon's blows are guided by [Dexterity](stats.md#stat-dexterity) instead of [Strength](stats.md#stat-strength). Its bonus to [hit](#hitting) up close is
your Dexterity modifier, even when your Strength is higher. [Strength](stats.md#stat-strength) still adds to its
[damage](#damage).

{{#include generated/finesse-weapons.md}}

## Defence

Your **Defence** (shown as `Dv` on the [character screen](character.md), `@`) is 10 plus what
protects you. It comes in kinds:

- **Dexterity**: quick feet, your [Dexterity](stats.md#stat-dexterity) modifier (see [Stats](stats.md#modifiers)). A clumsy
  character's Defence is below 10. Heavy armour limits how much [Dexterity](stats.md#stat-dexterity) can add (see the table
  below), and so does a heavy load: at most +3 when you are [burdened](burden.md#burden-1), +1 when [strained](burden.md#burden-1),
  nothing when [overloaded](burden.md#burden-1) (see [Burden](burden.md)). The lower limit counts.
- **Armour**: what you wear on your body.
- **Shield**: a [shield](objects.md#class-shields) in your off hand. Your [rank](weapon-skills.md#marks-and-ranks) in [Sword and Board](fighting-styles.md#style-sword-and-board) adds more, with a weapon
  in the other hand. In [Two Shields](fighting-styles.md#style-two-shields), you hold a shield in each.
- **Parry**: a weapon that turns blows aside, such as the quarterstaff, plus your rank with the
  weapon you wield (blades, spears, and staves), plus your rank in [Duelist](fighting-styles.md#style-duelist) or [Staff Fighter](fighting-styles.md#style-staff-fighter) while
  you fight that way. Parry counts against blows only. An [arrow](ammunition.md), a [bolt](ammunition.md), or a thrown thing is not
  parried, except by a [duelist](fighting-styles.md#style-duelist) of rank 10 (Measure, see [Fighting styles](fighting-styles.md#what-a-rank-teaches)), and except
  against a shot from beside you ([Distance](ranged.md#distance)).
- **Natural**: a creature's hide, shell, or scales. You have none, unless your [race](races.md) or a
  [magic item](magic-items.md) gives you some. Lizardfolk's scales soak blows but do not turn them (see
  [Races](races.md#the-evil-races)).
- **Dodge**: training and magic. The [tumble](skills.md#what-skills-give) skill adds to it, more the higher it is, and so do
  [magic items](magic-items.md) that make you nimble.
- **Deflection**: [magic items](magic-items.md) that turn blows aside.
- **Wall**: the stone around an incorporeal creature standing inside a wall
  ([Creatures](creature-bodies.md#incorporeal-creatures)).

Within each kind only the best counts, so two pieces of armour do not add up. Dodge is the one kind
that does add up.

Armour also has **Protection**, which takes [damage](combat.md#damage) off the blows that get through (see
[Protection](#protection) below). Light armour is mostly Defence. Heavy armour is mostly
Protection.

{{#include generated/armour.md}}

Helms, cloaks, gloves, boots, rings, and amulets add to Defence only through their magic (see
[Magic items](magic-items.md)). Magic armour or a magic shield may add more than its base. The
heavier helms, gloves, and boots, and the fur cloak, give [Protection](combat.md#protection).

### Why an attack missed

When an attack misses, the message says why, as far as you saw it:

- A roll so poor it would have missed a target standing still in no armour at all is a plain
  miss: "The kobold misses you."
- Otherwise your Defence turned it. One kind is picked, more likely the more it adds: "You
  dodge the kobold's spear." ([Dexterity](stats.md#stat-dexterity)), "The kobold's spear glances off your armour.", "You
  block the kobold's spear with your shield.", "You parry the kobold's spear.", "Your sword
  glances off the cave beetle's shell."
- A clumsy character's low [Dexterity](stats.md#stat-dexterity) lowers their Defence, but is never given as the reason for a miss.
- If you do not see the attacker or the target, a miss is just a miss: "Something misses you."

## Criticals

A hit whose dice come up high enough **threatens a critical**. Most weapons threaten on 22 to 24.
Swords and [daggers](weapon-skills.md#weapon-skill-daggers) threaten on 20 to 24 or wider. Each weapon's range is in the [weapons table](#damage).

A second roll against the same [Defence](combat.md#defence) decides. If it would hit too, the critical is
**confirmed**: the damage is rolled again and added. That makes twice in all for most weapons, and
three times for the heavy ones, among them axes, hammers, [spears](weapon-skills.md#weapon-skill-spears), and [bows](weapon-skills.md#weapon-skill-bows) (the [weapons table](#damage) gives
each one's). The message says so: "The kobold hits you hard!" A confirmed critical also opens a
[bleeding](status-effects.md#bleeding) wound (see [Status effects](status-effects.md)).

Some things have **no vitals** to find, so they are **immune to criticals** and to
[sneak attacks](skills.md#skill-sneak-attack). A threat on them is a plain hit, and
[Vital strike](weapon-skills.md#weapon-ability-vital-strike) or a
[master thief](talents.md#talent-master-thief)'s finisher is only a plain blow ("It has no vital
spot."). These are the [incorporeal](creature-bodies.md#incorporeal-creatures) (ghosts and wraiths),
[animated objects](animated.md), and every [creature](creatures.md) tagged elemental, skeleton,
spirit, or construct (the mud golem; [elementals](creature-bodies.md#elementals)). The fleshy undead
(draugr, ghouls, the drowned, vampires, mummies) do have vitals. The full list is under
[Surprise and sneak attacks](surprise.md#no-vitals).

{{#include generated/threats.md}}

## Damage

A hit does the weapon's damage dice, at least 1:

- **Up close**, your **[Strength](stats.md#stat-strength)** adds to the damage, with a [finesse weapon](#finesse-weapons) too. With nothing in your
  hands you hit with your [fists](weapon-skills.md#weapon-skill-unarmed).
- **Shots** do the bow's own damage. [Dexterity](stats.md#stat-dexterity) aims them but adds nothing to the damage. A bow is
  drawn by strength, though: each point of [Strength](stats.md#stat-strength) you lack below what the bow needs costs you a
  point to hit and a point of damage.
- Creatures without a weapon bite, claw, or kick, each in its own way.
- Your [rank](weapon-skills.md#marks-and-ranks) in the weapon's [skill](weapon-skills.md) adds to the damage, and at higher ranks widens the
  [critical range](combat.md#criticals).
- A **[shield bash](abilities.md#ability-shield-bash)** does the shield's own damage (below) plus your [Strength](stats.md#stat-strength). It takes the time of a
  plain blow. Neither a slow great weapon's time nor a quick one's (nor its magic) changes that.

{{#include generated/weapons.md}}

Messages never give numbers. They say what kind of blow it was: "The kobold stabs you.", "You
slash the giant rat hard!", "The giant rat bites the kobold.", "You cleave the kobold." (see
[Damage types](damage-types.md)).

A **big hit**, one that takes at least a third of the target's [hit points](hit-points.md#hit-points) at once, makes it
[bleed](status-effects.md#bleeding). Some creatures' attacks can [poison](status-effects.md#status-poisoned) (see [Status effects](status-effects.md)). Some bites carry a
[disease](diseases.md) (see [Diseases](diseases.md)).

## Protection

[Defence](combat.md#defence) decides whether a blow lands. **Protection** (shown as `Pv` on the
[character screen](character.md), `@`, beside your Defence) takes damage off every blow that does.
Each hit loses between half your Protection (rounded up) and all of it, at random, with every
amount in between equally likely:

{{#include generated/protection.md}}

- **Every piece adds up.** Body armour, helm, gloves, boots, and cloak each add their Protection
  (see the armour table above). So does the magic of *[Hardened](affixes.md#prefixes)* armour (see [Magic items](magic-items.md)).
  Some [talents](talents.md) add to it too, in any armour: [Bulwark](talents.md#talent-bulwark) 2, and the [berserker's](classes.md#berserker)
  [Battle-scarred](talents.md#talent-battle-scarred) 1, or 3 while [raging](status-effects.md#status-raging). The `Pv` on the character
  screen shows your Protection as it is now. A [shield](objects.md#class-shields) gives [Defence](combat.md#defence) only. Creatures have their
  own: a [cave beetle's](creatures.md#creature-beetle) shell turns aside much of what hits it.
- **[Criticals](combat.md#criticals) punch through.** A confirmed critical rolls its damage twice (or more) first, and
  the Protection comes off only once.
- **A hit can do nothing at all.** If Protection takes all of its damage, the blow glances off and
  nothing else comes of it: no [blood](hit-points.md#blood), no [bleeding](status-effects.md#bleeding), no [poison](status-effects.md#status-poisoned). "Your sword glances off the
  cave beetle's shell.", "The kobold's spear glances off your chain mail shirt." It still landed,
  so it trains your [weapon skill](weapon-skills.md) as a hit does.
- **Some weapons pierce armour**, ignoring part of the Protection of whatever they hit. Their
  description says "Pierces armour", but never how much.

{{#include generated/piercing.md}}

Protection soaks the blows of weapons, claws, and bites, and a [trap's](traps.md) arrows and spears. The blast
of a [fireball](magic.md#spell-fireball), a [lightning bolt](magic.md#spell-lightning-bolt), or a fall into a [pit](traps.md#trap-pit) goes straight through. Protection soaks
**[physical](damage-types.md)** damage only, and a piece may do better or worse against one physical kind: chain
mail turns a blade well and a point badly. [Fire](damage-types.md#damage-fire), [cold](damage-types.md#damage-cold), and the rest go through any armour, and
**[resistances](damage-types.md#armour-and-resistance)** take their share after it (see [Damage types](damage-types.md)).

In the game you never see a creature's Protection, only how its armour turns your blows. The
numbers are on [Creatures in numbers](creature-numbers.md#hit-points-defence-and-resistances).
