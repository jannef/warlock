# Resources

Everyone has [hit points](hit-points.md#hit-points) and [mana](mana.md). Some characters also hold
**resources** of their own: pools of points that a class or a talent gives them, such as a
[berserker](classes.md#berserker)'s [rage](#rage) or a [ranger](classes.md#ranger)'s
[focus](#resource-focus). You may hold several at once, and each has its own rules:

- Some **come back** by themselves over time, up to their maximum.
- Some **wear off** over time, down to nothing.
- Some **build up** from what you do: a kill, a blow that hits, a [critical](combat.md#criticals),
  being hit, a [sneak attack](talents.md#talent-sneak-attack).
- Abilities may **spend** them. An ability that costs more than you have is greyed out in the
  [abilities menu](abilities.md) ("not enough focus").
- Some **end** when they run out: the resource is gone, and its bar with it, until something gives
  it to you again. Others stay with you, empty, and fill up again.

While you hold a resource, the status bar shows it after your mana, as `Rage 7` or `Focus 7/10`
(what you have, and for some your maximum). The [character screen](character.md) (`@`) gives it a
row of its own.

A resource that wears off while you [rest](resting.md) may run out before you wake. One that does
not come back while resting waits for you to get up. Creatures never hold any resources.

Nothing converts between resources and mana: each is spent only by what asks for it.

Times below are given in base turns. A base turn is 6 seconds.

## The resources

**Rage.** The berserker's rage runs on Rage (see [Rage](#rage) below).

**Focus.** The [ranger](classes.md#ranger) gets focus, full, with
[rapid shot](talents.md#talent-rapid-shot) at level 12, and keeps it for good. A shot you loose
while you hold 2 focus spends them and takes 10 % less time. A [volley](talents.md#talent-volley)
(level 50) costs 6. Focus comes back while you act, but not while you rest.

**Openings.** Every [thief](classes.md#thief) holds openings from level 1
([Sneak attack](talents.md#talent-sneak-attack)), starting at 0.

- Every [sneak attack](skills.md#skill-sneak-attack) that hits gives one, up to 5.
- Every [feint](skill-uses.md#feinting) the foe falls for gives one.
- Killing your [mark](#the-mark) yourself gives one more.
- One slips away every 20 base turns (2 minutes). When empty, the pool stays with you.
- From level 1, [Exploit](#exploit) (an ability, aimed in a direction) spends **every** opening
  you hold on one melee blow that counts as a sneak attack, whatever the foe knows.
- At level 60, [Master thief](talents.md#talent-master-thief) gives the **finisher** (an ability,
  aimed in a direction). It spends all 5 for a melee blow that is a [critical](combat.md#criticals)
  whenever it hits, with no confirmation roll. A miss still spends them.

### The mark

Your openings are gaps in one creature's guard: your **mark**. The character screen shows whose
guard they are in ("Openings 3/5, in the kobold's guard"), and looking at the creature says "your
mark".

- **Switching foes loses them.** The first creature you choose to attack becomes your mark. A
  blow, a kick, a shield bash, an ability's blow, a shot or a throw at its square, a spell that
  rolls to hit, or a [feint](skill-uses.md#feinting), made or not, all count. Choose to attack a different creature while your mark lives, and every
  opening is lost ("Your openings on the kobold are lost."); the new creature is your mark.
- **Blows you did not choose change nothing.** A [free blow](stances.md#free-blows) (a riposte, an
  opportunist's blow, a lurk's) makes its target your mark only when you have none. On anyone
  else, its [sneak attack](skills.md#skill-sneak-attack) gives no opening. Area effects, like a fireball, never touch your mark.
- **A kill carries them over.** When your mark dies, by anyone's hand, your openings stay, and
  the next creature you attack becomes your mark with them. If you killed it yourself (a bleeding
  wound you dealt counts), you get one more opening.
- **Your mark is forgotten** when it vanishes (a summoned creature's time runs out), when you
  take the stairs to another level, and when your openings run down to 0. Your openings stay.
  A mark that steps out of sight is still your mark.
- **Spending them on someone else is refused.** [Exploit](#exploit) and the
  [finisher](talents.md#talent-master-thief) at a creature other
  than your mark do nothing and take no time ("Your openings are in the kobold's guard.").

### Exploit

Exploit is the [thief](classes.md#thief)'s way to cash in openings, from level 1. Choose a
direction, and you strike the hostile there with what you wield, in the time of a melee blow.

- **It spends every opening you hold**, from 1 to 5. You choose how many by when you strike.
- **It is a [sneak attack](surprise.md#sneak-attacks) whatever the foe knows.** The sneak attack
  dice land if it hits. The foe is not [Surprised](status-effects.md#status-surprised) and keeps
  its whole Defence.
- **Each opening spent adds 1d6 damage** on a hit, after the sneak dice. Like them, it comes after
  [Protection](combat.md#protection), and a [critical](combat.md#criticals) never doubles it.
- **Its own sneak attack gives no opening.** After it you hold none, and your
  [mark](#the-mark) is forgotten. If the blow kills your mark, that kill still gives you one.
- **It is refused**, taking no time and spending nothing, with no opening ("you have no opening"
  in the menu), with no hostile there ("There is no one there."), at a creature other than your
  mark ("Your openings are in the kobold's guard."), and at a creature you see that has no vitals
  ("It has no vital spot.": skeletons, spirits, animated things; see
  [Criticals](combat.md#criticals)).
- The game tells you how deep it cuts: "You strike at the opening." (1), "You press your
  openings." (2 to 4), "You tear into every opening!" (5).

The thief's class talents add to every exploit, per opening spent:

| Level | Talent | Adds per opening spent |
|---|---|---|
| 1 | [Exploit](talents.md#talent-exploit) | +1d6 damage |
| 12 | [Low blow](talents.md#talent-low-blow) | +1 to hit; a hit leaves the foe [Stunned](status-effects.md#status-stunned) (its next action costs half again as much, and your next blow finds it flat-footed) |

So at level 12 an exploit of 4 openings is +4 to hit and does the blow, the sneak dice, and 4d6
more, and the foe is stunned.

**Ki.** The [ascetic](classes.md#ascetic) gets ki, full, with
[martial arts](talents.md#talent-martial-arts) at level 1. The pool holds 4, plus your
[Willpower](stats.md#stat-willpower) [modifier](stats.md#modifiers), plus a point per 6
[levels](experience.md#levels), and never less than 3. For example, Willpower 12 holds 5 at level
1, Willpower 14 holds 10 at level 25, and Willpower 16 holds 17 at level 60.
[Inner calm](talents.md#talent-inner-calm) adds 2.

Ki comes back a point every 3 base turns (18 seconds), resting too.
[Perfect self](talents.md#talent-perfect-self) (level 60) brings it back twice as fast, a point
every 9 seconds. What spends it:

- Each second blow of a [flurry](talents.md#talent-flurry) (level 1) spends a point. With none
  left, the attack is a single blow.
- [Step of the wind](abilities.md#ability-step-of-the-wind) (level 6) spends 1 and takes no time.
- [Stunning strike](talents.md#talent-stunning-strike) (level 12) spends 1.
- [Patient defence](abilities.md#ability-patient-defence) (level 18) spends 1 and takes a wait's
  time.
- [Stillness](abilities.md#ability-stillness) (level 25) spends 2 and takes no time.
- [Diamond soul](talents.md#talent-diamond-soul) (level 40) spends 1 by itself on each missile it
  catches.
- [Quivering palm](talents.md#talent-quivering-palm) (level 50) spends 5.

**Inspiration.** The [bard](classes.md#bard) gets inspiration with
[inspire](talents.md#talent-inspire) at level 9, starting at 0. Every kill of your own gives one,
up to 5, and one fades every 30 base turns (3 minutes). When empty, the pool stays with you.

Only [Inspire](abilities.md#ability-inspire) spends it: 3 inspiration and nothing else, and with
less you cannot sing it. (The [lullaby](abilities.md#ability-lullaby) is paid in
[mana](mana.md#spending-it); see [Songs](abilities.md#songs).) A
[virtuoso](talents.md#talent-virtuoso)'s inspire spends none.

**Zeal.** The [paladin](classes.md#paladin) gets zeal with [Smite](talents.md#talent-smite) at
level 1, holding 5. The pool holds 10, plus your [Presence](stats.md#stat-presence)
[modifier](stats.md#modifiers), plus a point per 10 [levels](experience.md#levels), and never less
than 6. For example, Presence 12 holds 11 at level 1, and Presence 16 holds 16 at level 30.

- Every blow, shot, or throw of yours that hits builds a point, and so does every blow that hits
  you.
- It also comes back a point every 10 base turns (a minute), resting too. Spent, it sits at 0, and
  an empty pool is full again in about ten minutes.
- [Smite](abilities.md#ability-smite) spends 3, [Branding smite](abilities.md#ability-branding-smite)
  3, [Wrathful smite](abilities.md#ability-wrathful-smite) 4,
  [Banishing smite](abilities.md#ability-banishing-smite) 6, and
  [Lay on hands](abilities.md#ability-lay-on-hands) 6 (3 with [Avatar](talents.md#talent-avatar)).
  [Retribution](talents.md#talent-retribution) (level 25) spends 4 for every blow it answers. So
  wrath and mercy are paid from the one pool.
- The blow that hits you builds its point first, so 3 zeal is enough for Retribution to answer it.
  With [Zealot](talents.md#talent-zealot) you need 2, and every blow that hits you builds 2.
- A holy gift that does not answer (because you are not Kind or better, see
  [Alignment](alignment.md)) spends none.
- **A deed that moves you toward evil** (see [Alignment](alignment.md)) empties the pool at once:
  "Your zeal deserts you."

**Stamina** is the fighter's breath: the pool that active weapon abilities are paid from (see
[Combat](stances.md#stamina)). Every [warrior](classes.md#warrior), [berserker](classes.md#berserker),
[paladin](classes.md#paladin), [ranger](classes.md#ranger), [ascetic](classes.md#ascetic),
and [bard](classes.md#bard) holds it, full, from level 1
([Trained wind](talents.md#talent-trained-wind)). Anyone else, the [thief](classes.md#thief) too,
gets it, full, from the first
[weapon rank](weapon-skills.md#marks-and-ranks) that teaches an active ability.

The pool holds 10, plus your [Toughness](stats.md#stat-toughness) [modifier](stats.md#modifiers),
plus a point per 10 [levels](experience.md#levels), and never less than 5 (Toughness 14 holds 12 at
level 1). It comes back a point every 2 base turns (12 seconds), resting too, and sits at 0 when
spent.

A [berserker](classes.md#berserker) holds rage and stamina at once.
[Raging](status-effects.md#status-raging) blocks spells, but not active weapon abilities.

## Magic items on resources

A [magic item](magic-items.md) may name a pool and change it while you wear or wield it, whether
identified or not. It can give:

- a larger **maximum**, by so many points;
- a faster **pace**, by so many percent, added to what talents add.
  [Steady breathing](talents.md#talent-steady-breathing)'s 50 % and an item's 30 % make 180 %;
- for stamina, every active weapon ability **costing** so many points less. This adds to
  [Drillmaster](talents.md#talent-drillmaster)'s discount, and a cost never goes below 1;
- so many more points on a **kill** (*of the Hunt*), never above the maximum. This follows the same
  rule as [mana](mana.md#how-much) *of Siphoning*: only your own blow, shot, or
  [spell](magic.md#the-spells) counts, with the one you killed within 3 cells of you (see
  [Mana on a kill](mana.md#mana-on-a-kill)). It comes on top of what a pool builds on any kill by
  its own rule (inspiration's point).

Two items naming the same pool add up. Taking the item off lowers what you hold to the smaller
maximum at once.

Such an item never **grants** a pool: it does nothing for someone who does not hold it. Who holds
which pool:

- **Stamina:** every [warrior](classes.md#warrior), [berserker](classes.md#berserker),
  [paladin](classes.md#paladin), [ranger](classes.md#ranger), [ascetic](classes.md#ascetic),
  [thief](classes.md#thief), and [bard](classes.md#bard), and anyone else once a
  [weapon rank](weapon-skills.md#marks-and-ranks) teaches an active ability.
- **Rage:** a [raging](status-effects.md#status-raging) [berserker](classes.md#berserker), only while
  the rage lasts.
- **Focus:** a [ranger](classes.md#ranger) from level 12.
- **Ki:** an [ascetic](classes.md#ascetic) from level 1.
- **Inspiration:** a [bard](classes.md#bard) from level 9.
- **Openings:** a [thief](classes.md#thief) from level 1.

A ring of rage on a [wizard](classes.md#wizard) is dead weight. [Creatures](creatures.md) never hold
a pool, so such gear does nothing for them, only for whoever takes it from them.

{{#include generated/resources.md}}

## Rage

A [berserker](classes.md#berserker)'s **Rage** [ability](abilities.md#ability-rage) (from the
abilities menu, taking a wait's time) gives them 12 rage. They are
**[Raging](status-effects.md#status-raging)** (see [Status effects](status-effects.md)) for as long
as they hold any: [Strength](stats.md#stat-strength) +2 and [Toughness](stats.md#stat-toughness)
+2, [Intelligence](stats.md#stat-intelligence) −2, and they can neither read nor spend
[mana](mana.md#spending-it).

Rage wears off a point every base turn, starting with the turn of the rage itself, and while
resting too. Fighting feeds it:

- Every blow, shot, or throw that hits you builds 1. It need not hurt you: one your
  [Protection](combat.md#protection) soaks or you [resist](damage-types.md#armour-and-resistance)
  to nothing still counts.
- Every [critical](combat.md#criticals) of yours that does damage builds 2.
- Every kill of yours builds 3.

A blow of yours that merely [lands](combat.md#hitting) builds nothing. Rage holds 20 at most, so a
long fight does not store up a long rage after it.

When rage reaches 0, the rage ends, its bar with it. The berserker is then
[Exhausted](status-effects.md#status-exhausted) for two and a half minutes (150 seconds), and can
have no rage until that passes. Anything else that ends being Raging ends the rage too.

The berserker's later levels add to it (see [Talents](talents.md)):

- **[Battle-scarred](talents.md#talent-battle-scarred)** (level 6):
  [Protection](combat.md#protection) +1, and +2 more while raging (3 in all), whatever armour you
  wear.
- **[Headlong](talents.md#talent-headlong)** (level 12): while raging, every step costs a fifth less
  [time](time.md), in any armour and under any [burden](burden.md).
- **[Blood frenzy](talents.md#talent-blood-frenzy)** (level 40): while raging, every melee attack
  costs a fifth less time, and every kill builds 5 more rage (8 in all).
- **[Mighty rage](talents.md#talent-mighty-rage)** (level 50): Raging gives Strength +4 and
  Toughness +4, rage holds up to 30, and the exhaustion lasts 75 seconds.
- **[Unstoppable](talents.md#talent-unstoppable)** (level 60): while raging,
  [bleeding](status-effects.md#status-bleeding) and [poison](status-effects.md#status-poisoned) cost
  no hit points. A blow that would take your last [hit point](hit-points.md#hit-points) leaves it to
  you for 10 rage, as often as you have that much. Spending the last of the rage this way ends it.

## Talents on resources

Five [talents](talents.md) change a resource. Only the class that has the resource may take them,
as a pick from its trees, from the level given:

- **[Battle trance](talents.md#talent-battle-trance)** (Might; [berserker](classes.md#berserker),
  level 6): your rage starts at 18 instead of 12 (never above its maximum), and wears off at two
  thirds of its pace: a point about every 9 seconds instead of every 6.
- **[Inner calm](talents.md#talent-inner-calm)** (Finesse; [ascetic](classes.md#ascetic), level 18):
  ki holds 2 more and comes back half again as fast, a point every 12 seconds instead of every 18.
- **[Steady hand](talents.md#talent-steady-hand)** (Weaponcraft; [ranger](classes.md#ranger), level
  12): focus comes back half again as fast, a point every 4 seconds instead of every 6. It still
  does not come back while you rest.
- **[Patient blade](talents.md#talent-patient-blade)** (Shadow; [thief](classes.md#thief), level
  32): openings slip away half as fast, a point every 40 base turns instead of every 20. Killing
  your [mark](#the-mark) yourself gives two openings instead of one.
- **[Zealot](talents.md#talent-zealot)** (Grit; whoever holds [Smite](talents.md#talent-smite), so
  the [paladin](classes.md#paladin), level 6): every blow that hits you builds 2
  [zeal](#resource-zeal) instead of 1. A blow you give still builds 1, and the pool holds no more.

Inner calm changes nothing until something gives the [ascetic](classes.md#ascetic) [ki](#resource-ki).
