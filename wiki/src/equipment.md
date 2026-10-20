# Equipment

You can wear and wield what you carry, as in ADOM: a helm on your head, an amulet around your
neck, armor on your body, a cloak, a weapon in your main hand and a shield (or a light weapon)
in your off hand, gloves, a ring on each hand, boots, a missile weapon with its missiles ready,
and a tool.

What you wield and wear decides how you fight (see [Combat](combat.md)): the weapon in your
hand does its damage, a bow shoots, body armour and a shield add to your Defence, and a
quarterstaff parries. Nothing you wear slows you down by itself, though a great sword or an axe
swings slower than a dagger. Magic items work while you wear or wield them (see
[Magic items](magic-items.md)). Putting things on and taking them off takes time, and what you
wear still weighs what it weighs.

## The equipment screen

Press `e`. The screen lists every slot with its letter and what is in it, or `—` when it is
empty, and below them what you carry and weigh, as the inventory does:

{{#include generated/equipment.md}}

"What goes there" lists every object in the game that fits the slot, and the last column is how
long putting one on there, or taking it off, takes at a normal pace.

- Press a slot's letter, or move the `>` with the arrow keys and press `Enter`, to choose what
  goes there. On this screen every letter names a slot, even `e`, `h`, `j`, `k`, `l`, and `m`.
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
do it: creatures act meanwhile, just as when you wait. Looking at the screens takes no time.

## What fits where

- **Rings** fit either ring slot. Choosing a ring for the right hand while it is on your left
  moves it over.
- **Light weapons** (listed below) fit either hand: one in your off hand beside a weapon in your
  main hand is fighting with two weapons (see [Fighting styles](fighting-styles.md#two-weapons)).
  Nothing else but a shield goes in the off hand.
- **Two-handed weapons**, such as the quarterstaff and the great sword, take both hands. The screen shows them in
  the main hand and the off hand, "(both hands)". Wielding one puts away whatever your hands
  held; a shield in your off hand puts a two-handed weapon away.
- **Missile weapons and missiles** have their own slots. A bow shoots arrows: each missile weapon
  shoots one kind of missile ("shoots arrows" in the table above). You can ready any missiles,
  matching or not, but only the matching ones can be shot ([Ranged combat](ranged.md)). The
  Missile weapon slot also takes any weapon and rocks: `t` throws what is readied there.
- **Tools** go in the Tool slot: a pick-axe or a mattock ready there is what you dig with (`U`;
  see [Digging and walls](digging.md)). Other tools, such as keys, rope, and candles, fit nowhere
  yet.
- Things that are not equipment (potions, food, coins) fit nowhere.

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

**Dropping** something you wear or wield takes it off first ("You take off a leather jerkin.")
and then drops it, which takes the time of both. What else you wear stays worn.

## Creatures

Creatures wear and wield some of what they carry, and you can see it. [Look mode](controls.md#look-mode)
says so: "A kobold (hostile, unhurt), wielding a spear, on stone floor.", "A kobold shaman
(hostile, unhurt), wearing a bone amulet, ...", "A kobold archer (hostile, unhurt), with a short
bow and 20 arrows ready, ...". You still cannot see anything else they carry. They fight with it
as you do, and keep what they start with until they die: then all of it falls where they stood.
