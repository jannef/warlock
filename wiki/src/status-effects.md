# Status effects

Some harm lasts longer than a blow. A deep wound keeps bleeding, and a bite can leave poison
behind. These **status effects** cost hit points as time passes, slow your
[healing](healing.md), sometimes weaken you, and wear off on their own. They can kill.

You always know what ails you: the [status bar](controls.md#the-screen) names it right after your
hit points (`HP 12/17 · Bleeding · Poisoned`), the [character screen](character.md) lists it, and
the message line tells you when it starts and stops. You never see how bad it is, or how long it
will last, as a number: you see your hit points go.

{{#include generated/status-effects.md}}

## How they work

- **Severity.** Every status effect has a severity, in degrees. The worse it is, the faster it
  costs hit points: a wound twice as bad bleeds twice as fast.
- **They add up.** A second wound while you are still bleeding makes the bleeding worse; a
  second dose of poison makes you sicker, and stronger poison lasts longer. Each status has a
  worst it can get.
- **They wear off.** Bit by bit, a degree at a time, until they are gone.
- **Time is time.** Status effects run on game time, whatever you do meanwhile: waiting, walking,
  [resting](resting.md), or a long journey on the [overworld](overworld.md) all count the same.
  The game only skips them for creatures on other levels, which wait for you to come back.
- **They can kill.** If a status effect takes your last hit point, you die of it, and the death
  screen says so: "Bled to death".

## Bleeding

A **big hit**, one that takes at least a third of your hit points at once, or a **confirmed
critical** ("The kobold hits you hard!") opens a wound: "You are bleeding." A critical opens a
worse one than a big hit. Every blow can do it for now, a club's as well as a spear's.

While you bleed, you lose a hit point every so often, faster the worse you bleed, and your hit
points do not come back at all. The bleeding eases by itself and stops after a while: "The
bleeding stops." Later, a **bandage** (with the Heal skill) will stop it at once.

Creatures bleed the same way, and you can see it: [look mode](controls.md#look-mode) says "A giant
rat (hostile, wounded, bleeding)". A creature can bleed to death after a fight: "The giant rat
bleeds to death."

## Poisoned

Some creatures' attacks carry poison: now and then a giant rat's filthy bite leaves you sick.
"You feel sick." So does bad food: a rotten corpse that stays down, and the flesh of some
creatures (see [Food and corpses](food.md)).

Poison works slowly: a hit point every couple of minutes, faster the stronger it is, and your hit
points come back only half as fast. **Strong poison** (a few doses at once) also makes you
weaker and clumsier: your Strength and Dexterity count one less, in everything they do, from
hitting and Defence to how much you can carry. The character screen shows them lowered. Poison
wears off by itself: "The poison wears off."

You cannot tell whether a creature is poisoned by looking at it.

## In the message line

"You are bleeding." and "You feel sick." stop a [rest](resting.md), like any news. "The bleeding
stops." and "The poison wears off." are written to the message line but let you rest on.
