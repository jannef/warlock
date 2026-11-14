# Rage

A [berserker](class-berserker.md)'s **Rage** [ability](abilities.md#ability-rage) (from the
abilities menu, taking a wait's time) gives them 12 rage. They are
**[Raging](status-list.md#status-raging)** (see [Status effects](status-effects.md)) for as long
as they hold any: [Strength](stats.md#stat-strength) +2 and [Toughness](stats.md#stat-toughness)
+2, [Intelligence](stats.md#stat-intelligence) −2, and they can neither read nor spend
[mana](mana.md#spending-it).

Rage wears off a point every base turn (6 seconds), starting with the turn of the rage itself,
and while resting too. Fighting feeds it:

- Every blow, shot, or throw that hits you builds 1. It need not hurt you: one your
  [Protection](combat.md#protection) soaks or you [resist](damage-types.md#armour-and-resistance)
  to nothing still counts.
- Every [critical](combat.md#criticals) of yours that does damage builds 2.
- Every kill of yours builds 3.

A blow of yours that merely [lands](combat.md#hitting) builds nothing. Rage holds 20 at most, so a
long fight does not store up a long rage after it.

When rage reaches 0, the rage ends, its bar with it. The berserker is then
[Exhausted](status-list.md#status-exhausted) for two and a half minutes (150 seconds), and can
have no rage until that passes. Anything else that ends being Raging ends the rage too.

The berserker's later levels add to it (see [Talents](talents.md)):

- **[Battle-scarred](class-berserker.md#talent-battle-scarred)** (level 6):
  [Protection](combat.md#protection) +1, and +2 more while raging (3 in all), whatever armour you
  wear.
- **[Headlong](class-berserker.md#talent-headlong)** (level 12): while raging, every step costs a fifth less
  [time](time.md), in any armour and under any [burden](burden.md).
- **[Blood frenzy](class-berserker.md#talent-blood-frenzy)** (level 40): while raging, every melee attack
  costs a fifth less time, and every kill builds 5 more rage (8 in all).
- **[Mighty rage](class-berserker.md#talent-mighty-rage)** (level 50): Raging gives Strength +4 and
  Toughness +4, rage holds up to 30, and the exhaustion lasts 75 seconds.
- **[Unstoppable](class-berserker.md#talent-unstoppable)** (level 60): while raging,
  [bleeding](status-list.md#status-bleeding) and [poison](status-list.md#status-poisoned) cost
  no hit points. A blow that would take your last [hit point](hit-points.md#hit-points) leaves it to
  you for 10 rage, as often as you have that much. Spending the last of the rage this way ends it.

## The talent on it

**[Battle trance](talents-body.md#talent-battle-trance)** (Might; [berserker](class-berserker.md),
level 6): your rage starts at 18 instead of 12 (never above its maximum), and wears off at two
thirds of its pace: a point about every 9 seconds instead of every 6.

See also: [Resources](resources.md), [Buffs and hexes](buffs.md), and
[Stances and stamina](stances.md).
