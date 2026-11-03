# Resources

Beside your [hit points](combat.md#hit-points) and your [mana](mana.md), which everyone has, a
character may come to hold **resources** of their own: pools of points that something grants
them, such as a [berserker](classes.md#berserker)'s [rage](resources.md#rage) or a [ranger](classes.md#ranger)'s [focus](resources.md#resource-focus). You may hold several at once, each with its
own rules:

- some **come back** by themselves with time, up to their most;
- some **wear off** with time, down to nothing;
- some **build up** from what you do: a kill, a blow that hits, a [critical](combat.md#criticals), being hit, a [sneak
  attack](talents.md#talent-sneak-attack);
- abilities may **spend** them: an ability that costs more than you have is greyed out in the
  [abilities menu](abilities.md) ("not enough focus");
- and some **end** when they run out: the resource is gone, and its bar with it, until something
  grants it again. Others stay with you, empty, and fill up again.

While you hold a resource, the status bar shows it after your mana, as `Rage 7` or `Focus 7/10`
(what you have, and for some your most), and the [character screen](character.md) (`@`) gives it
a row of its own. A resource that wears off while you [rest](resting.md) may run out before you
wake; one that does not come back while resting waits for you to get up. Creatures never hold
any.

Nothing converts between resources and mana: each is spent by what asks for it.

## The resources

The berserker's rage runs on Rage (see [Rage](#rage) below). The [ranger](classes.md#ranger)'s **focus** is in play:
[rapid shot](talents.md#talent-rapid-shot) (level 12) grants it, full, and you keep it for good. A shot you loose while you
hold 2 focus spends them and takes 10 % less time; a [volley](talents.md#talent-volley) (level 50) costs 6. Focus comes
back while you act, but not while you rest.

The [thief](classes.md#thief)'s **openings** are in too: every thief holds them from level 1 ([Sneak attack](talents.md#talent-sneak-attack)),
starting at 0. Every [sneak attack](skills.md#skill-sneak-attack) that hits gives one, up to 5, and one slips away every 20
base turns (2 minutes); empty, they stay with you. At level 12 [Low blow](abilities.md#ability-low-blow) (an ability,
aimed in a direction) spends 3 for a melee blow that is a [sneak attack](skills.md#skill-sneak-attack) whatever the foe
knows. At level 60 [Master thief](talents.md#talent-master-thief)'s **finisher** (an
ability, aimed in a direction) spends all 5 for a melee blow that is a [critical](combat.md#criticals) whenever it
hits: no confirmation roll is made. A miss still spends them.

The [ascetic](classes.md#ascetic)'s **ki** comes with [martial arts](talents.md#talent-martial-arts) at level 1, full. It holds 4, plus your
[Willpower](stats.md#stat-willpower) [modifier](stats.md#modifiers), plus a point per 6 [levels](experience.md#levels), never less than 3 (Willpower 12
holds 5 at level 1, Willpower 14 holds 10 at level 25, Willpower 16 holds 17 at level 60);
[inner calm](talents.md#talent-inner-calm) adds 2. Each second blow of a [flurry](talents.md#talent-flurry) (level 1) spends a point, and with none
the attack is one blow; [step of the wind](abilities.md#ability-step-of-the-wind) (level 6) spends 1, taking no time; [stunning strike](talents.md#talent-stunning-strike) (level 12) spends 1, [stillness](abilities.md#ability-stillness) (level 25) 2, taking no time; [diamond soul](talents.md#talent-diamond-soul) (level 40) spends 1 by itself on each missile it catches; and [quivering palm](talents.md#talent-quivering-palm) (level 50) 5. Ki comes back a point every 3 base
turns (18 seconds), resting too; [perfect self](talents.md#talent-perfect-self) (level 60) brings it back twice as fast, a point
every 9 seconds. [Patient defence](abilities.md#ability-patient-defence) (level 18) spends 1, with a wait's time.

The [bard](classes.md#bard)'s **inspiration** comes with [inspire](talents.md#talent-inspire) (level 9), starting at 0. Every kill of your own
gives one, up to 5, and one fades every 30 base turns (3 minutes); empty, it stays with you.
[Inspire](abilities.md#ability-inspire) alone spends it (the [lullaby](abilities.md#ability-lullaby) is paid in
[mana](mana.md#spending-it); see [Songs](abilities.md#songs)): 3 inspiration and nothing else, and with less you cannot sing it. A
[virtuoso](talents.md#talent-virtuoso)'s inspire spends none.

**Stamina** is the fighter's breath, the pool active weapon abilities are paid from (see
[Combat](combat.md#stamina)). Every [warrior](classes.md#warrior), [berserker](classes.md#berserker), [paladin](classes.md#paladin), [ranger](classes.md#ranger), [ascetic](classes.md#ascetic), [thief](classes.md#thief), and [bard](classes.md#bard)
holds it from level 1 ([Trained wind](talents.md#talent-trained-wind)), full. It holds 10, plus your [Toughness](stats.md#stat-toughness) [modifier](stats.md#modifiers), plus a
point per 10 [levels](experience.md#levels), never less than 5 (Toughness 14 holds 12 at level 1); it comes back a point
every 2 base turns (12 seconds), resting too, and sits at 0 when spent. A [berserker](classes.md#berserker) holds rage
and stamina at once; [raging](status-effects.md#status-raging) blocks spells, not weapon actives. Anyone else is given it, full,
by the first [weapon rank](weapon-skills.md#marks-and-ranks) that teaches an active.

A base turn is 6 seconds.

## Magic items on resources

A [magic item](magic-items.md) may name a pool and change it while worn or wielded, identified
or not: a larger **maximum** (so many points more), a faster **pace** (so many percent, added to
what talents add; [Steady breathing](talents.md#talent-steady-breathing)'s 50 % and an item's 30 % make 180 %), or, for stamina, every
weapon active **costing** so many points less (added to [Drillmaster](talents.md#talent-drillmaster)'s discount, never below 1),
or so many points more on a **kill** (*of the Hunt*; never above the maximum), under the same
rule as [mana](mana.md#how-much) *of Siphoning*: only your own blow, shot, or [spell](magic.md#the-spells), the one you killed within 3
cells of you (see [Mana on a kill](mana.md#mana-on-a-kill)). That is beside what a pool builds
on any kill by its own rule (inspiration's point).
Two items naming the same pool add up. Taking the item off lowers what you hold to the smaller
maximum at once.

Such an item never **grants** a pool: it does nothing for one who does not hold it. Who holds
which: stamina, every [warrior](classes.md#warrior), [berserker](classes.md#berserker), [paladin](classes.md#paladin), [ranger](classes.md#ranger), [ascetic](classes.md#ascetic), [thief](classes.md#thief), and [bard](classes.md#bard), and anyone
else once a [weapon rank](weapon-skills.md#marks-and-ranks) teaches an active; rage, a [raging](status-effects.md#status-raging) [berserker](classes.md#berserker) (only while the rage
lasts); focus, a [ranger](classes.md#ranger) from level 12; ki, an [ascetic](classes.md#ascetic) from level 1; inspiration, a [bard](classes.md#bard) from level
9; openings, a [thief](classes.md#thief) from level 1. A ring of rage on a [wizard](classes.md#wizard) is dead weight, and [creatures](creatures.md)
never hold a pool, so their gear does nothing for them, only for whoever takes it from them.

{{#include generated/resources.md}}

## Rage

A [berserker](classes.md#berserker)'s **Rage** [ability](abilities.md#ability-rage) (from the abilities menu, the time of a wait) gives them 12 rage,
and they are **[Raging](status-effects.md#status-raging)** (see [Status effects](status-effects.md)) for as long as they hold any:
[Strength](stats.md#stat-strength) +2 and [Toughness](stats.md#stat-toughness) +2, [Intelligence](stats.md#stat-intelligence) −2, and they can neither read nor spend [mana](mana.md#spending-it). Rage
wears off a point a base turn, the turn of the rage itself first, resting too, and fighting feeds it:

- every blow, shot, or throw that hits you builds 1, even one your [Protection](combat.md#protection) soaks or you
  [resist](damage-types.md#armour-and-resistance) to nothing (it need not hurt you);
- every [critical](combat.md#criticals) of yours that does damage builds 2;
- every kill of yours builds 3.

A blow of yours that merely [lands](combat.md#hitting) builds nothing. Rage holds 20 at most, so a long fight banks no
long calm after it. At none the rage ends, its bar with it, and the [berserker](classes.md#berserker) is
[Exhausted](status-effects.md#status-exhausted) for two and a half minutes (150 seconds), with no rage until that passes. Something that ends the [Raging](status-effects.md#status-raging) otherwise ends the rage too. The berserker's later levels
add to it (see [Talents](talents.md)):

- **[Battle-scarred](talents.md#talent-battle-scarred)** (level 6): [Protection](combat.md#protection) +1, and +2 more while raging (3 in all),
  whatever armour you wear.
- **[Headlong](talents.md#talent-headlong)** (level 12): while raging, every step costs a fifth less
  [time](time.md), in any armour and under any [burden](burden.md).
- **[Blood frenzy](talents.md#talent-blood-frenzy)** (level 40): while raging, every melee attack costs a fifth less time; and every kill builds 5 rage more (8 in all).
- **[Mighty rage](talents.md#talent-mighty-rage)** (level 50): [Raging](status-effects.md#status-raging) gives Strength +4 and Toughness +4, rage holds up to 30, and
  the exhaustion lasts 75 seconds.
- **[Unstoppable](talents.md#talent-unstoppable)** (level 60): while [raging](status-effects.md#status-raging), [bleeding](status-effects.md#status-bleeding) and [poison](status-effects.md#status-poisoned) cost no hit points, and a blow
  that would take your last [hit point](combat.md#hit-points) leaves it to you for 10 rage, as often as you have that
  much; spending the last of it ends the rage.

## Talents on resources

Four [talents](talents.md) change a resource, and only the class that has it may take them (a
pick from their trees, from the level given):

- **[Battle trance](talents.md#talent-battle-trance)** (Might; [berserker](classes.md#berserker), level 6): your rage starts at 18 instead of 12 (never
  above its most), and wears off at two thirds of its pace: a point about every 9 seconds
  instead of every 6.
- **[Inner calm](talents.md#talent-inner-calm)** (Finesse; [ascetic](classes.md#ascetic), level 18): ki holds up to 7 instead of 5, and comes back half
  again as fast: a point every 12 seconds instead of every 18.
- **[Steady hand](talents.md#talent-steady-hand)** (Weaponcraft; [ranger](classes.md#ranger), level 12): focus comes back half again as fast, a point
  every 4 seconds instead of every 6; still not while you rest.
- **[Patient blade](talents.md#talent-patient-blade)** (Shadow; [thief](classes.md#thief), level 32): openings slip away half as fast, a point every 40
  base turns instead of every 20, and a [sneak attack](talents.md#talent-sneak-attack) that kills gives two openings instead of
  one.

Inner calm changes nothing until something grants the [ascetic](classes.md#ascetic) [ki](resources.md#resource-ki).
