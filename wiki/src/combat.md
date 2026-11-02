# Combat

Fights are decided by dice, as in Dungeons & Dragons, but with two twelve-sided dice instead of
one twenty-sided die, and with ADOM's kind of [hit points](combat.md#hit-points). Every blow is rolled, can miss, can hit
hard, and hurts; what loses all its [hit points](combat.md#hit-points) dies. That goes for you too, and [death](combat.md#death) is final.

You attack by moving into a hostile creature (see [Controls](controls.md)), which takes a [turn](time.md#what-takes-time),
by [bashing](abilities.md#ability-shield-bash) it with your [shield](objects.md#class-shields), or from afar: you shoot or throw with `t` (or `T`), see
[Ranged combat](ranged.md). Each attack trains the [weapon skill](weapon-skills.md) it uses and
the [fighting style](fighting-styles.md) you fight in, and your [ranks](weapon-skills.md#marks-and-ranks) in them help you hit, hurt,
and defend. With a [weapon in each hand](fighting-styles.md#two-weapons), an attack up close is a blow with each.
Creatures fight each other and you by the same rules (see [Creatures](creatures.md)). [Spells](magic.md#the-spells) are
the exception: casters only announce them for now, and nothing is hurt by a spell.

## Hitting

An attack rolls two twelve-sided dice and adds them: 2 to 24, most often around 13. It hits when
that roll, plus the attacker's bonus to hit, reaches the target's **[Defence](combat.md#defence)**.

- Fighting up close, your **[Strength](stats.md#stat-strength)** helps you hit; shooting, your **[Dexterity](stats.md#stat-dexterity)** does. Some
  weapons are easier to hit with than others, and your [rank](weapon-skills.md#marks-and-ranks) in the weapon's
  [skill](weapon-skills.md) helps the more you have trained.
- **Two ones always miss, two twelves always hit**, whatever the odds.
- Because the sum of two dice gathers around the middle, each point of bonus or Defence matters
  most when the odds are close, and little when they are long either way:

{{#include generated/hit-chances.md}}

"The roll needs" is the target's Defence less the attacker's bonus. Against an unarmoured,
unremarkable target ([Defence](combat.md#defence) 10) an attacker with no bonus hits three times in four.

The game never shows you the rolls, but the [character screen](character.md) (`@`) shows your
own attacks beside your stats: the blow of your main hand (or bare hands), your off hand's when
you fight with two weapons, and the shot or throw of your [missile weapon](ranged.md#getting-ready), each with its bonus to
hit, its [damage](combat.md#damage) (`+4 to hit, 1d8+2 slashing`), and for a shot or throw its range in steps. They
count everything that goes into the attack as you stand: your [stats](stats.md), your [ranks](weapon-skills.md#marks-and-ranks) in its [weapon
skill](weapon-skills.md) and in the [fighting style](fighting-styles.md) you hold it in, [two-weapon](fighting-styles.md#two-weapons) penalties, your [stance](combat.md#stances), your [talents](talents.md),
and the magic you know of; a [magic item](magic-items.md)'s unknown powers count only once it is [identified](magic-items.md#unidentified-items), and
what depends on the moment (striking from hiding, being dazzled) is left out.
Exactly, this is how an attack is rolled, and what adds up to its bonus to hit and its damage:

{{#include generated/to-hit.md}}

A creature's attacks add up the same way; their numbers are on
[Creatures in numbers](creature-numbers.md#attacks).

## Defence

Your **Defence** (shown as `Dv` on the [character screen](character.md), `@`) is 10 plus what
protects you, in kinds:

- **Dexterity**: quick feet, your [Dexterity](stats.md#stat-dexterity) modifier (see [Stats](stats.md#modifiers)). A
  clumsy character's Defence is lower than 10. Heavy armour limits
  how much [Dexterity](stats.md#stat-dexterity) can add (see the table below), and so does a heavy load: at most +3 when
  you are [burdened](burden.md#burden-1), +1 when [strained](burden.md#burden-1), nothing when [overloaded](burden.md#burden-1) (see [Burden](burden.md)). The
  lower limit counts.
- **Armour**: what you wear on your body.
- **Shield**: a [shield](objects.md#class-shields) held in your off hand, and more with your [rank](weapon-skills.md#marks-and-ranks) in [shield blocking](weapon-skills.md#weapon-skill-shield-block) (and in
  [Sword and Board](fighting-styles.md#style-sword-and-board), with a weapon in the other hand).
- **Parry**: a weapon that turns blows aside, such as the quarterstaff, and your rank with the
  weapon you wield (blades, spears, and staves), and in [Duelist](fighting-styles.md#style-duelist) or [Staff Fighter](fighting-styles.md#style-staff-fighter) while you fight
  that way. It counts against blows only: an [arrow](ammunition.md), a [bolt](ammunition.md), or a thrown thing is not parried, but
  by a [duelist](fighting-styles.md#style-duelist) of rank 10 (Measure, see [Fighting styles](fighting-styles.md#what-a-rank-teaches)).
- **Natural**: a creature's hide, shell, or scales. You have none, unless your [race](races.md) gives you
  some (lizardfolk's scales soak blows but do not turn them: see
  [Races](races.md#the-evil-races)) or a [magic item](magic-items.md) does.
- **Dodge**: training and magic. The [tumble](skills.md#what-skills-give) skill adds to it, more
  the higher it is, and so do [magic items](magic-items.md) that make you nimble.
- **Deflection**: [magic items](magic-items.md) that turn blows aside.
- **Wall**: the stone around an incorporeal creature standing inside a wall
  ([Creatures](creatures.md#incorporeal-creatures)).

Within each kind only the best counts: two pieces of armour do not add up. Dodge is the one kind
that does.

Armour also has **Protection**, which takes [damage](combat.md#damage) off the blows that get through (see
[Protection](#protection) below): light armour is mostly Defence, heavy armour mostly
Protection.

{{#include generated/armour.md}}

Helms, cloaks, gloves, boots, rings, and amulets do not add to Defence by themselves, only by
their magic (see [Magic items](magic-items.md)), and magic armour or a magic shield may add more
than its base. The heavier helms, gloves, boots, and the fur cloak give [Protection](combat.md#protection).

### Why an attack missed

When an attack misses, the message says why, as far as you saw it:

- A roll so poor it would have missed a target standing still in no armour at all is a plain
  miss: "The kobold misses you."
- Otherwise your Defence turned it, and one kind is picked, more likely the more it adds: "You
  dodge the kobold's spear." ([Dexterity](stats.md#stat-dexterity)), "The kobold's spear glances off your armour.", "You
  block the kobold's spear with your shield.", "You parry the kobold's spear.", "Your sword
  glances off the cave beetle's shell."
- A clumsy character's low [Dexterity](stats.md#stat-dexterity) lowers their Defence, but is never why an attack missed.
- If you do not see one of the two, a miss is just a miss: "Something misses you."

## Criticals

A hit whose dice come up high enough **threatens a critical**: most weapons threaten on 22 to
24, swords and [daggers](weapon-skills.md#weapon-skill-daggers) on 20 to 24. A second roll, against the same [Defence](combat.md#defence), decides: if it would
hit too, the critical is **confirmed** and the damage is rolled again and added, twice in all for
most weapons, three times for [spears](weapon-skills.md#weapon-skill-spears) and [bows](weapon-skills.md#weapon-skill-bows). The message says so: "The kobold hits you
hard!" A confirmed critical also opens a [bleeding](status-effects.md#bleeding) wound (see [Status effects](status-effects.md)).

Some things have **no vitals** to find, and so are **immune to criticals** and to
[sneak attacks](skills.md#skill-sneak-attack): a threat on them is a plain hit, and
[Vital strike](weapon-skills.md#weapon-ability-vital-strike) or a
[master thief](talents.md#talent-master-thief)'s finisher only a plain blow ("It has no vital
spot."). They are the [incorporeal](creatures.md#incorporeal-creatures) (ghosts and wraiths),
[animated objects](animated.md), and every [creature](creatures.md) tagged elemental, skeleton,
spirit, or construct (the mud golem; [elementals](creatures.md#elementals)). The fleshy undead
(draugr, ghouls, the drowned, vampires, mummies) have vitals. The list is under [Surprise and sneak attacks](surprise.md#no-vitals).

{{#include generated/threats.md}}

## Damage

A hit does the weapon's damage dice, at least 1:

- **Up close**, your **[Strength](stats.md#stat-strength)** adds to the damage, as it does to hitting. With nothing in your
  hands you hit with your [fists](weapon-skills.md#weapon-skill-unarmed).
- **Shots** do the bow's own damage: [Dexterity](stats.md#stat-dexterity) aims them, but adds nothing to the damage. A bow
  is drawn by strength, though: each point of [Strength](stats.md#stat-strength) you lack of what the bow needs costs you
  a point to hit and a point of damage.
- Creatures without a weapon bite, claw, or kick, each in its own way.
- Your [rank](weapon-skills.md#marks-and-ranks) in the weapon's [skill](weapon-skills.md) adds to the damage, and at higher ranks
  widens the [critical range](combat.md#criticals).
- A **[shield bash](abilities.md#ability-shield-bash)** does the shield's own damage (below), plus your [Strength](stats.md#stat-strength), and takes the time
  of a plain blow: neither a slow great weapon's time nor a quick one's (nor its magic) changes
  it.

{{#include generated/weapons.md}}

Messages never give numbers, and say what kind of blow it was: "The kobold stabs you.", "You
slash the giant rat hard!", "The giant rat bites the kobold.", "You cleave the kobold." (see
[Damage types](damage-types.md)).

A **big hit**, one that takes at least a third of the target's [hit points](combat.md#hit-points) at once, makes it
[bleed](status-effects.md#bleeding), and some creatures' attacks can [poison](status-effects.md#status-poisoned): see [Status effects](status-effects.md). Some bites
carry a [disease](diseases.md): see [Diseases](diseases.md).

## Protection

[Defence](combat.md#defence) decides whether a blow lands; **Protection** (shown as `Pv` on the
[character screen](character.md), `@`, beside your Defence) takes damage off every blow that
does. Each hit loses between half your Protection (rounded up) and all of it, at random, every
amount in between as likely:

{{#include generated/protection.md}}

- **Every piece adds up**: body armour, helm, gloves, boots, and cloak each add their
  Protection (the armour table above), and so does the magic of armour that is
  *[Hardened](magic-items.md#prefixes)* (see [Magic items](magic-items.md)). A [shield](objects.md#class-shields) gives [Defence](combat.md#defence) only. Creatures have
  their own: a [cave beetle's](creatures.md#creature-beetle) shell turns aside much of what hits it.
- **[Criticals](combat.md#criticals) punch through**: a confirmed critical rolls its damage twice (or more) first, and
  the Protection comes off only once.
- **A hit can do nothing at all**: if Protection takes all of its damage, the blow glances off,
  and nothing else comes of it: no [blood](combat.md#blood), no [bleeding](status-effects.md#bleeding), no [poison](status-effects.md#status-poisoned). "Your sword glances off the
  cave beetle's shell.", "The kobold's spear glances off your chain mail shirt." It still landed,
  so it trains your [weapon skill](weapon-skills.md) as a hit does.
- **Some weapons pierce armour**, ignoring part of the Protection of whatever they hit. Their
  description says so ("Pierces armour"), but never how much.

{{#include generated/piercing.md}}

Protection soaks the blows of weapons, claws, and bites, and a [trap's](traps.md) arrows and spears; the
blast of a [fireball](magic.md#spell-fireball), a [lightning bolt](magic.md#spell-lightning-bolt), or a fall into a [pit](traps.md#trap-pit) goes straight through. It soaks
**[physical](damage-types.md)** damage only, and a piece may stand better or worse against one kind of it: chain
mail turns a blade well and a point badly; [fire](damage-types.md#damage-fire), [cold](damage-types.md#damage-cold), and the rest go through any armour, and
**[resistances](damage-types.md#armour-and-resistance)** take their share after it. See [Damage types](damage-types.md). In the game
you never see a creature's Protection, only how its armour turns your blows; see
[Creatures in numbers](creature-numbers.md#hit-points-defence-and-resistances).

## Stances

Like ADOM's, your fighting **stance** trades attack for defence. Press `+` to take a more aggressive stance
(coward, defensive, neutral, aggressive: it stops at the end) and `-` for a more defensive one; each takes no time, and you keep it
until you change it, even across a save. "You take an aggressive stance." You start neutral. The
[status bar](controls.md#the-screen) names your stance when it is not neutral, and the [character screen](character.md)
always does. The game never shows what a stance adds: you feel it in the fight.

- **Aggressive**: you hit more often and harder, but are easier to hit. ([Weapon skill marks](weapon-skills.md#marks-and-ranks) will
  count for more: you swing harder.)
- **Neutral**: no effect.
- **Defensive**: you hit less often and softer, but [dodge](combat.md#defence) better, and [parry](combat.md#defence) a little better if
  your weapon does not already turn blows aside as well.
- **Coward**: you hit much less often and softer, and your attacks take longer, but you [dodge](combat.md#defence)
  best, and stepping away from a hostile creature next to you is quicker and quieter.
- **Berserker**: a [barbarian's](classes.md#barbarian) alone, and only while they rage. Flying into a
  [rage](resources.md#rage) puts you in it at once, whatever your stance was, and `+` and `-` cannot
  take you out of it ("Your rage holds you in the berserker's stance."); no other class, talent,
  item, or creature ever has it. You hit more often and harder than an aggressive fighter, at a
  greater cost to your guard, and stepping away from a foe beside you takes longer. When the rage
  ends, however it ends, you fall back into the aggressive stance, not the one you had before.
  The barbarian's [Wild guard](talents.md#talent-wild-guard) talent halves its cost to your [Dodge](combat.md#defence).

A stance changes how long every attack takes: a blow, a [shield bash](abilities.md#ability-shield-bash), a [kick](doors.md#kicking), a [shot](ranged.md#shooting-and-throwing), or a [throw](ranged.md#shooting-and-throwing);
never reloading or kicking a [door](doors.md). Each attack takes the time it would as you begin it: a rank of
skill it earns you makes the next one quicker, not that one.

The numbers are first guesses, tuned by playtesting:

{{#include generated/stances.md}}

Dodge adds to your [Defence](combat.md#defence) on top of everything else; Parry only counts when it beats what your
weapon already gives. Creatures do not take stances yet.

## Free blows

Some fighters get a blow in for free, at no cost in time. [Talents](talents.md), [class abilities](talents.md#class-abilities), and the
ranks of weapon skills and fighting styles teach them (see
[What a rank teaches](weapon-skills.md#what-a-rank-teaches)).

- **[Cleave](talents.md#talent-cleave)**: when your attack kills, you strike another [hostile](creatures.md#factions) creature next to you: the next
  one round from where your victim stood, going clockwise. Once per attack, or, with Cleave's second [rank](talents.md#ranks-choices-prerequisites) (great
  cleave), again after every cleave that kills.
- **[Riposte](talents.md#talent-riposte)**: when your Parry turns a blow aside, you strike back once the attack is over (once
  per attack, however many of its blows you parried). Shots and throws are never riposted.
- **[Opportunist](talents.md#talent-opportunist)**: a [hostile](creatures.md#factions) creature that steps away from beside you is struck as it turns to
  go; it may not live to take the step. A creature that fights this way strikes you the same
  way when you step away from it.

A free blow is a blow like any other: it can miss, wound, and kill, and it trains your [weapon
skill](weapon-skills.md). It never brings another free blow, except a cleave after a cleave.

## Stamina

**Stamina** is the breath a fighter has for weapon feats (a [resource](resources.md), shown as
`Sta 7/12` on the status bar). Every [warrior](classes.md#warrior), [barbarian](classes.md#barbarian), [paladin](classes.md#paladin), [ranger](classes.md#ranger), [monk](classes.md#monk), [thief](classes.md#thief), and [bard](classes.md#bard)
holds it from [level](experience.md#levels) 1. Its most is

> 10 + your [Toughness](stats.md#stat-toughness) [modifier](stats.md#modifiers) + 1 per 10 [levels](experience.md#levels), at least 5

with [talents](talents.md) adding on top ([Deep lungs](talents.md#talent-deep-lungs): +4). [Toughness](stats.md#stat-toughness) 14 (modifier +2) at level 1 holds 12;
Toughness 18 (+4) at level 60 holds 20. [Damage to your Toughness](stats.md#drained-stats) lowers it, as it lowers your [hit
points](combat.md#hit-points). It comes back a point every 12 seconds (2 base [turns](time.md#what-takes-time)), [resting](resting.md#how-resting-works) too, half again as fast
with [Steady breathing](talents.md#talent-steady-breathing). Spent, it sits
at 0 and comes back. Creatures never hold it. The active weapon abilities spend it (see
[What a rank teaches](weapon-skills.md#what-a-rank-teaches)); anyone else is given it by the
first rank that teaches one. [Drillmaster](talents.md#talent-drillmaster) (the warrior's, at level 25) takes a point off every
active's cost, [Armour-breaker](talents.md#talent-armour-breaker) 2 off [Sunder's](weapon-skills.md#weapon-ability-sunder) and [Gap shot's](fighting-styles.md#style-ability-gap-shot) (never below 1), and [Practised
technique](talents.md#talent-practised-technique) a quarter off their time (never below half the action's).

## Hit points

Your **hit points** are your [Toughness](stats.md#stat-toughness) plus what your [race](races.md) and your
[class](classes.md) give:

{{#include generated/hit-points.md}}

A [dwarf](races.md#race-dwarf) [barbarian](classes.md#barbarian) with Toughness 16 has 6 + 8 + 16 = 30 hit points; an [elf](races.md#race-elf) [wizard](classes.md#wizard) with Toughness
8 has 10. You start every run unhurt. The [status bar](controls.md#the-screen) shows your hit points (`HP 18/24`), and so
does the character screen.

Creatures have hit points too, but you never see their numbers in the game (they are on
[Creatures in numbers](creature-numbers.md#hit-points-defence-and-resistances)).
[Look mode](controls.md#look-mode) says how hurt a creature looks: unhurt, lightly wounded (more than two
thirds left), wounded (more than a third), badly wounded (more than a sixth), or nearly dead:

{{#include generated/health.md}}

### Getting better

Hit points come back with time, faster the tougher you are, twice as fast while you
[rest](resting.md#how-resting-works), and slower or not at all while you [bleed](status-effects.md#bleeding), are [poisoned](status-effects.md#status-poisoned), or are [diseased](status-effects.md#status-diseased): see
[Healing](healing.md). A [disease](diseases.md) that lowers your [Toughness](stats.md#stat-toughness) lowers your most hit points too (see
[Diseases](diseases.md)). Creatures heal the same way, while you are on their level. In a
[sanctuary](special-rooms.md#room-sanctuary) (a [special room](special-rooms.md)) wounds heal three times as fast, yours and
anyone's.

## Death

A creature at 0 [hit points](combat.md#hit-points) dies: "The giant rat dies." It is gone, and everything it carried,
what it wore and wielded too, falls where it stood, for you to pick up, with its corpse on top
(see [Food and corpses](food.md)). A wounded creature may also die later of its wounds: "The
giant rat bleeds to death."

When **you** reach 0 [hit points](combat.md#hit-points), you die: "You die..." The run is over, and a screen tells you
what killed you (if you saw it, or "Bled to death" when your wounds did, or that you starved; see
[Hunger](hunger.md)), where, when, and after how many turns. There is no going back:
your save is deleted at once (see [Saving](saving.md)). Press `Enter` or `Esc` to return to the
main menu and start a new run.

## Blood

Big hits and kills leave blood where they land. A blow that takes at least a third of its
target's [hit points](combat.md#hit-points), or any [critical](combat.md#criticals) ("hits you hard!"), leaves a little spatter; a kill leaves a
pool where the victim fell and a spray behind it, away from whoever struck the blow. How much
depends on how big the victim was and how it died: a blow far past what killed it, or a critical,
spills more; an arrow or a thrown weapon less, sprayed along its flight. Blood that sprays into
a wall right beside the victim stains that wall on the side facing it.

Blood adds up: one kill is easy to miss, but where many died the floor is soaked. Fresh blood is
red, and it dries darker and browner over a few hours; the cave beetle bleeds a greenish-black
ichor. Water, lava, and fire take no stains. You see blood where you see the floor (on a wall,
only from the side that was splattered), remember it as you last saw it, and [look mode](controls.md#look-mode) mentions
it: "on blood-stained stone floor".

Blood is only for show: it never changes a fight, and the Blood switch in the
[settings](settings.md) hides it.
