# Equipment

You can wear and wield what you carry, as in ADOM: a [helm](objects.md#class-helmets) on your head, an [amulet](objects.md#class-amulets) around your
neck, [armor](objects.md#class-body-armour) on your body, a [cloak](objects.md#class-cloaks), a [weapon](objects.md#class-melee-weapons) in your main hand and a [shield](objects.md#class-shields) (or a light weapon)
in your [off hand](fighting-styles.md#two-weapons), [gloves](objects.md#class-gloves), a [ring](objects.md#class-rings) on each hand, [boots](objects.md#class-boots), a [missile weapon](objects.md#class-missile-weapons) with its [missiles](objects.md#class-missiles) ready,
and a [tool](objects.md#class-tools).

What you wield and wear decides how you fight (see [Combat](combat.md)): the weapon in your
hand does its damage, a bow [shoots](ranged.md#shooting-and-throwing), body armour and a shield add to your [Defence](combat.md#defence), and a
quarterstaff [parries](combat.md#defence). Nothing you wear slows you down by itself, though a great sword or an axe
swings slower than a dagger. Magic items work while you wear or wield them (see
[Magic items](magic-items.md)). Putting things on and taking them off takes time, and what you
wear still weighs what it weighs.

## The equipment screen

Press `i`. The screen lists every slot with its letter and what is in it, or `—` when it is
empty, and below them what you carry and weigh, as the [inventory](objects.md#your-pack) does:

{{#include generated/equipment.md}}

"What goes there" lists every object in the game that fits the slot, and the last column is how
long putting one on there, or taking it off, takes at a [normal pace](turns.md#what-actions-cost).

- Press a slot's letter, or move the `>` with the arrow keys and press `Enter`, to choose what
  goes there. On this screen every letter names a slot, even `e`, `i`, `k`, `l`, and `m`.
- Press a slot's letter in upper case (`A` to `M`) to look at what is in it (see
  [Magic items](magic-items.md#looking-at-an-item)).
- The next screen lists what you carry that fits the slot (see [Lists of
  objects](objects.md#lists-of-objects)). Press its letter, or `Enter` on the `>` row, to put it
  on.
- If the slot already holds something, it is taken off first: the two swap places. Choose what
  is already in the slot (the `>` starts on it) to take it off.
- `Esc` goes back to the equipment screen, and `Esc` there back to the game.

What you choose is done at once, and the equipment screen stays open, showing what changed, so
you can carry on dressing. Each change takes its time, and the game goes on around you while you
do it: [creatures](creatures.md) act meanwhile, just as when you wait. Looking at the screens takes no time.

## Numbers

Every list of things you carry or could pick up (the inventory, the equipment screen and its
choice, picking up, dropping, selling, throwing) writes after each item's name what it gives, and
looking at a pile says it too:

- **[Weapons](objects.md#class-melee-weapons)**: their [to-hit](combat.md#hitting) and [damage](combat.md#damage), "a long sword (+0 hit, 1d9)". The to-hit is added to
  your 2d12 attack roll and the damage is rolled on every hit, before what your [Strength](stats.md#stat-strength), [weapon
  skill](weapon-skills.md), [fighting style](fighting-styles.md), and [stance](combat.md#stances) add (the [character screen](character.md) shows your attacks
  with all of that counted). Extra dice and elemental [damage](damage-types.md) follow, "(+0 hit, 1d8+1d4 +1d6
  fire)". [Missile weapons](objects.md#class-missile-weapons) and throwing weapons add their [range](ranged.md#shooting-and-throwing) in cells, "a short bow (+0 hit,
  1d6, range 10)"; a shot does the bow's damage, not the arrow's.
- **[Missiles](objects.md#class-missiles)**: what they add to a shot, only when they add something: "12 flaming arrows (+1d4
  fire)", "12 Glinting arrows (+1 hit)", "12 Sharp arrows (+2 damage)".
- **Other things you wear** that add to every attack once [identified](magic-items.md#unidentified-items) (an *Accurate* helm, a
  *Vicious* ring): "an Accurate iron helm (+1 hit, Dv +0, Pv 1)".
- **Armour**: the [Defence](combat.md#defence) (Dv) and [Protection](combat.md#protection) (Pv) it gives, "a chain mail shirt (Dv +2, Pv 3)",
  even when both are 0. A **[shield](objects.md#class-shields)** gives Defence only, "a small shield (Dv +1)", and so does a
  weapon that parries, "a quarterstaff (+0 hit, 1d6, Dv +1)", or a [ring](objects.md#class-rings) or [amulet](objects.md#class-amulets) of [Dodge](combat.md#defence) or
  [Deflection](combat.md#defence) once [identified](magic-items.md#unidentified-items).

The numbers are the item's own: its [base item](magic-items.md#base-items), what it is made of, and its [affixes](magic-items.md#prefixes) once it is
[identified](magic-items.md#unidentified-items). An unidentified [magic item](magic-items.md) shows its base item's numbers only, though its magic
works all the same (see [Magic items](magic-items.md#unidentified-items)).

[Defence](combat.md#defence) bonuses of the same kind do not add up: of your armour pieces only the best Armour
counts, of shields the best, and so on (see [Combat](combat.md)); [Protection](combat.md#protection) does add up. The
equipment screen shows your totals under the slots, "Defence (Dv) 14 · Protection (Pv) 4", the
same numbers as the character screen; they count everything you wear, unidentified [magic](magic-items.md) too.

## What fits where

- **[Rings](objects.md#class-rings)** fit either ring slot. Choosing a ring for the right hand while it is on your left
  moves it over.
- **Light weapons** (listed below) fit either hand: one in your off hand beside a weapon in your
  main hand is fighting with two weapons (see [Fighting styles](fighting-styles.md#two-weapons)).
  Nothing else but a shield goes in the off hand.
- **[Shields](objects.md#class-shields)** fit either hand too: a shield goes to your off hand first, and a second one to
  your main hand is fighting with two shields (see
  [Fighting styles](fighting-styles.md#two-shields)).
- **Two-handed weapons**, such as the quarterstaff and the great sword, take both hands. The screen shows them in
  the main hand and the off hand, "(both hands)". Wielding one puts away whatever your hands
  held; a shield in your off hand puts a two-handed weapon away.
- **[Missile weapons](objects.md#class-missile-weapons) and [missiles](objects.md#class-missiles)** have their own slots. A bow shoots [arrows](ammunition.md): each missile weapon
  shoots one kind of missile ("shoots arrows" in the table above). You can ready any missiles,
  matching or not, but only the matching ones can be shot ([Ranged combat](ranged.md)). The
  Missile weapon slot also takes any weapon and rocks: `t` [throws](ranged.md#shooting-and-throwing) what is readied there.
- **Tools** go in the Tool slot: a pick-axe or a mattock ready there is what you dig with (`u`;
  see [Digging and walls](digging.md)). Other tools, such as [keys](doors.md#keys), rope, and candles, fit nowhere
  yet.
- Things that are not equipment ([potions](potions.md), [food](food.md), [coins](objects.md#class-coins)) fit nowhere.

{{#include generated/light-weapons.md}}

## How long it takes

{{#include generated/actions.md}}

Every thing you put on or take off takes its slot's time from the table:

- A swap takes both: taking off a leather jerkin and strapping on a chain mail shirt takes 36
  seconds.
- Wielding a quarterstaff while you hold a sword and a shield takes three times as long as
  wielding it with empty hands.
- Taking something off takes as long as putting it on.
- Something that does not fit, or an empty slot to take off from, takes no time at all.

## Your pack

What you wear and wield stays in your pack. The [inventory](objects.md#your-pack)
marks it: "(being worn)", "(wielded)", "(in off hand)", "(on left hand)", "(on right hand)",
"(readied)", "(quivered)", "(in use)" for a tool, and "(wielded in both hands)" for a two-handed
weapon. It still counts
toward the weight you carry, and so toward your [burden](burden.md).

**[Dropping](objects.md#dropping)** something you wear or wield takes it off first ("You take off a leather jerkin.")
and then drops it, which takes the time of both. What else you wear stays worn.

## Creatures

[Creatures](creatures.md) wear and wield some of what they carry, and you can see it. [Look mode](controls.md#look-mode)
says so: "A kobold (hostile, unhurt), wielding a spear, on stone floor.", "A kobold shaman
(hostile, unhurt), wearing a bone amulet, ...", "A kobold archer (hostile, unhurt), with a short
bow and 20 arrows ready, ...". You still cannot see anything else they carry. They fight with it
as you do, and keep what they start with until they die: then all of it falls where they stood.
