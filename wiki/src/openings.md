# Openings

Every [thief](class-thief.md) holds openings from level 1
([Sneak attack](class-thief.md#talent-sneak-attack)), starting at 0.

- Every [sneak attack](skills.md#skill-sneak-attack) that hits gives one, up to 5.
- Every [feint](skill-uses.md#feinting) the foe falls for gives one.
- Killing your [mark](#the-mark) yourself gives one more.
- One slips away every 20 base turns (2 minutes). When empty, the pool stays with you.
- From level 1, [Exploit](#exploit) (an ability, aimed in a direction) spends **every** opening
  you hold on one melee blow that counts as a sneak attack, whatever the foe knows. The thief's
  class talents make it bigger as they come (see [Exploit](#exploit)).

## The mark

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
- **Spending them on someone else is refused.** [Exploit](#exploit) at a creature other
  than your mark does nothing and takes no time ("Your openings are in the kobold's guard.").

## Exploit

Exploit is the [thief](class-thief.md)'s way to cash in openings, from level 1. Choose a
direction, and you strike the hostile there with what you wield, in the time of a melee blow.

- **It spends every opening you hold**, from 1 to 5 (more with an item *of Guile*, see
  [Suffixes](affixes.md#suffixes); there is no cap). You choose how many by when you strike.
- **It is a [sneak attack](surprise.md#sneak-attacks) whatever the foe knows.** The sneak attack
  dice land if it hits. The foe is not [Surprised](status-list.md#status-surprised) and keeps
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

{{#include generated/exploit-ladder.md}}

A [stunned](status-list.md#status-stunned) foe's next action costs half again as much, and
your next blow finds it flat-footed.

So at level 12 an exploit of 4 openings is +4 to hit and does the blow, the sneak dice, and 4d6
more, and the foe is stunned. At level 32 the same exploit does 8d6 more. At level 60 its
[critical](combat.md#criticals) threat is 16 lower, and 2 lower again as on any
[sneak attack](surprise.md#sneak-attacks) (that step never below 3), so a weapon that threatens on
20 threatens on 3. An exploit of 5 or more is a critical whenever it hits, with no confirmation roll. The
critical doubles only the weapon's part: the sneak dice and the exploit's dice are never doubled.

## Thrown exploit

From level 18 the [thief](class-thief.md) can also exploit by throwing, for a mark that
stepped back, blinked away, or a foe across the room after a kill. Choose the Thrown exploit
ability, then something to throw, then the target.

- **It is Exploit's blow, thrown.** It spends every opening, and it is a
  [sneak attack](surprise.md#sneak-attacks) whatever the foe knows. Every addition in the table
  above applies: the dice per opening, the stun and +1 to hit per opening, the lower threat and
  the sure critical at 5. It uses the thrown weapon's attack and your Thrown rank, not the
  weapon in your hand.
- **One of what you throw leaves your hand**, as any throw. It lands by the foe or breaks.
- **Its reach** is the weapon's throw range, but never more than 6 cells (more with
  [Long reach](talents.md#talent-long-reach)): past that the sneak dice would not land. A dagger
  throws 5.
- **It is refused**, taking no time and spending nothing, for everything Exploit refuses (no
  opening, no hostile you see there, a creature other than your mark, no vitals), for a flask
  ("You cannot exploit an opening with …"), for a target too far ("That is out of reach."), and
  for a wall or a creature you see between you and the target ("Something is in the way.").
- **A creature you did not see in the way** takes the throw instead, as a plain throw: no sneak
  dice and nothing per opening. The openings are spent all the same.

## The talent on it

**[Patient blade](talents.md#talent-patient-blade)** (Shadow; [thief](class-thief.md), level
32): openings slip away half as fast, a point every 40 base turns instead of every 20. Killing
your [mark](#the-mark) yourself gives two openings instead of one.
