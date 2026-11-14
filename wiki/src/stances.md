# Stances, free blows, and stamina

Three things shape how you fight from blow to blow: your stance, the free blows some fighters get in, and the stamina that weapon feats spend. The rules of hitting and damage are on [Combat](combat.md).

## Stances

Your fighting **stance** trades attack for defence. Press `+` for a more aggressive stance and `-`
for a more defensive one. The order is coward, defensive, neutral, aggressive, and it stops at
either end. Changing stance takes no time, and you keep it until you change it, even across a save:
"You take an aggressive stance." You start neutral.

The [status bar](the-screen.md#the-screen) names your stance when it is not neutral, and the [character screen](character.md) always
does. The game never shows what a stance adds. You feel it in the fight.

- **Aggressive**: you hit more often and harder, but are easier to hit. [Weapon skill marks](weapon-skills.md#raising-a-weapon-skill) count
  for more, because you swing harder. Your [miscasts](miscasts.md#the-miscast-table) are worse:
  5 more on the total.
- **Neutral**: no effect.
- **Defensive**: you hit less often and less hard, but [dodge](combat.md#defence) better. You also parry a little
  better, unless your weapon already turns blows aside as well. Your
  miscasts are milder too: 5 off the total.
- **Coward**: you hit much less often and less hard, and your attacks take longer. But you dodge
  best, and stepping away from a hostile creature next to you is quicker and quieter.
- **Frenzy**: the [berserker's](class-berserker.md) alone, and only while they rage. Flying into a [rage](rage.md) puts you in it
  at once, whatever your stance was. `+` and `-` cannot take you out of it ("Your rage holds you
  in the frenzy."). No other class, talent, item, or creature ever has it. You hit more often and
  harder than an aggressive fighter, at a greater cost to your guard, and stepping away from a foe
  beside you takes longer. Your miscasts are worse still: 10 more
  on the total. When the rage ends, however it ends, you drop into the aggressive
  stance, not the one you had before. The berserker's [Wild guard](talents-body.md#talent-wild-guard) talent halves its cost to
  your Dodge.

A stance changes how long every attack takes: a blow, a [shield bash](abilities.md#ability-shield-bash), a [kick](doors.md#kicking-a-door-open), a [shot](ranged.md#shooting-and-throwing), or a
throw. It does not change reloading or kicking a [door](doors.md). Each attack takes the time it would
as you begin it, so a rank of skill it earns you makes the next attack quicker, not that one.

What each stance gives and costs:

{{#include generated/stances.md}}

Dodge adds to your Defence on top of everything else. Parry only counts when it beats what your
weapon already gives. Creatures do not take stances yet.

## Free blows

Some fighters get a blow in for free, at no cost in time. [Talents](talents.md), [class abilities](classes.md#levels), and the
ranks of weapon skills and fighting styles teach them (see
[What a rank teaches](weapon-skills.md#what-a-rank-teaches)).

- **[Cleave](talents-body.md#talent-cleave)**: when your attack kills, you strike another [hostile](creatures.md#factions) creature next to you. It is the
  next one round from where your victim stood, going clockwise. This happens once per attack, or,
  with [Great cleave](talents-body.md#talent-great-cleave), again after every cleave that kills.
- **[Retribution](class-paladin.md#talent-retribution)**: the [paladin](class-paladin.md)'s, from level 25. When a hostile creature beside you lands a
  blow of its melee attack on you, you answer with your [smite](abilities.md#ability-smite) once the attack is over. It uses
  the smite's dice, your [Presence](stats.md#stat-presence) to hit, and the extra die on the unholy, and costs 4 [zeal](resources.md#resource-zeal).
  With less zeal, or while you are not Kind or better (see [Alignment](alignment.md)), nothing happens. An
  attack it answers is not also riposted.
- **[Riposte](talents-body.md#talent-riposte)**: when your Parry turns a blow aside, you strike back once the attack is over. That
  is once per attack, however many of its blows you parried. Shots and throws are never riposted.
  While you are [poised](status-list.md#status-poised) (the [ascetic](class-ascetic.md)'s [patient defence](abilities.md#ability-patient-defence)), you riposte whatever your talents,
  and a blow that misses because of your Dexterity or Dodge is answered too.
- **[Opportunist](talents-body.md#talent-opportunist)**: a hostile creature that steps away from beside you is struck as it turns to go.
  It may not live to take the step. A creature that fights this way strikes you the same way when
  you step away from it.
- **[Gatekeeper](horoscope.md#talent-gatekeeper)**: the Barghest's gift (see [The horoscope](horoscope.md#what-a-sign-does)). A hostile creature that steps up
  beside you from out of your reach may take a blow as it comes, after its step. The chance
  depends on your [stance](#stances): one time in 4 when neutral, twice that when aggressive, half that when
  defensive, a quarter as a coward, and every time in a frenzy (the *Gatekeeper's blow* column of
  the stances table). [Brace](weapon-skills.md#weapon-ability-brace), a rank's blow of the same kind, strikes every time, and so does a
  [thief](class-thief.md) who [lurks](abilities.md#ability-lurk).

A free blow is a blow like any other. It can miss, wound, and kill, and it trains your
[weapon skill](weapon-skills.md). It never brings another free blow, except a cleave after a cleave.

## Stamina

**Stamina** is the breath a fighter has for weapon feats. It is a [resource](resources.md), shown as
`Sta 7/12` on the status bar. Every [warrior](class-warrior.md) has it from
[level](experience.md#levels) 1 ([Charge](class-warrior.md#talent-charge) spends it). Anyone else,
of any class, gets it, full, from the first [weapon rank](weapon-skills.md#what-a-rank-teaches)
that teaches an active ability. Its maximum is

> 10 + your [Toughness](stats.md#stat-toughness) [modifier](stats.md#modifiers) + 1 per 10 levels, at least 5

with [talents](talents.md) adding on top ([Deep lungs](talents-body.md#talent-deep-lungs): +4). Toughness 14 (modifier +2) at level 1 gives 12.
Toughness 18 (+4) at level 60 gives 20. [Damage to your Toughness](stats.md#drained-stats) lowers it, as it lowers your
[hit points](hit-points.md#hit-points).

Stamina comes back a point every 12 seconds (2 base [turns](time.md#what-takes-time)), while [resting](resting.md#how-resting-works) too, and half again as fast
with [Steady breathing](talents-fighting.md#talent-steady-breathing). Spent, it sits at 0 and comes back. Creatures never have it.

A warrior with [Battle rhythm](class-warrior.md#talent-battle-rhythm) (at level 40) also gets a point back
for every [melee](combat.md) blow of theirs that hits, and 1 more for every foe such a blow kills: a
killing blow gives 2 in all. It never goes past the maximum. Shots, throws, and spells give none.

The active weapon abilities spend it (see
What a rank teaches). Some talents make those abilities
cheaper:

- [Drillmaster](class-warrior.md#talent-drillmaster) (the warrior's, at level 25) takes a point off every active's cost.
- [Armour-breaker](talents-body.md#talent-armour-breaker) takes 2 off [Sunder's](weapon-skills.md#weapon-ability-sunder) and [Gap shot's](fighting-styles.md#style-ability-gap-shot) cost (never below 1).
- [Practised technique](talents-fighting.md#talent-practised-technique) takes a quarter off their time (never below half the action's).

See also: [Surprise and sneak attacks](surprise.md), [Abilities](abilities.md), and
[Ranged combat](ranged.md).
