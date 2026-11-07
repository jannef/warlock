# Mimics

Not every [chest](chests.md) is a chest. Now and then a coffer, a hutch, or a strongbox standing
on the floor is a **mimic**: a thing that sits in the dark looking like a chest until something
touches it, and grows bigger the longer it sits. It looks exactly like a real chest of its kind,
both in sight and in memory, and [look mode](controls.md#look-mode) describes it in the same
words.

Each chest laid on a floor has a small chance, set by its band, of being a mimic. There is never
one in a [sanctuary](special-rooms.md#room-sanctuary), which creatures shun.

A mimic is always of the floor's level, as a [chest](chests.md) is, and is named by the same ten
bands. A **[mimicling](creatures.md#creature-mimic-1)** passes for a coffer, a **young mimic** for
a chest, and an **eldest mimic** for a gilded ark. It carries exactly what the chest would have
held, and drops it all, whole, when it dies. Nothing in it is ever smashed or burnt.

## Finding one out

A mimic does nothing until it is found out. It never moves, never wakes on its own, and no other
creature takes any notice of it. It is found out when:

- **You touch it.** Walking into it, opening it, [kicking](doors.md#kicking) it, or
  [picking its lock](doors.md#picking-locks) all count. It lurches ("The coffer lurches: it is a
  young mimic!") and strikes you at once with a free blow that may [hold](status-effects.md#status-held)
  you fast. Your action is not done and takes no time, so the next move is yours.
- **You cast [Knock](abilities.md#ability-knock) on it.** It lurches, but no blow comes. The
  [mana](mana.md#how-much) is spent.
- **Anything strikes it**: a blow, an arrow or a thrown dagger (which stop at it), a
  [fireball](magic.md#spell-fireball), or a [trap's](traps.md#finding-traps) burst ("The coffer
  shudders: it is a young mimic!").
- **You find it by [searching](searching.md#searching-1).** It is a hidden thing like a
  [secret door](doors.md#secret-doors), with a difficulty of 12 + level ÷ 5, plus the branch's
  addition per depth. You notice it [in passing](searching.md#noticing-in-passing) only if you
  notice everything (trained in [Spot](skills.md#skill-spot), or an [elf](races.md#race-elf)).
  "You notice the coffer breathing: a young mimic!"

Until it is found out it has seen no one, so a [thief's](classes.md#thief)
[sneak attack](talents.md#talent-sneak-attack) lands on it. Once found out it is an ordinary
[hostile](creatures.md#factions) creature of the vermin, and it never disguises itself again. It
is [slow](turns.md#speed) (speed 800), so you can walk away from it.

## Talents

- **[Chest-wise](talents.md#talent-chest-wise)** (delver; [Intelligence](stats.md#stat-intelligence)
  11 or [Detect traps](skills.md#skill-detect-traps) 15): within two cells you see a mimic for what
  it is. It shows as itself, not as a chest. It is still disguised, though: it waits, unaware,
  until something finds it out. A mimic's grip also never holds you.

## The bands

{{#include generated/mimic-bands.md}}
