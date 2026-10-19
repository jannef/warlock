# Equipment

You can wear and wield what you carry, as in ADOM: a helm on your head, an amulet around your
neck, armor on your body, a cloak, a weapon in your main hand and a shield in your off hand,
gloves, a ring on each hand, boots, and a missile weapon with its missiles ready.

What you wield and wear decides how you fight (see [Combat](combat.md)): the weapon in your
hand does its damage, a bow shoots, body armour and a shield add to your Defence, and a
quarterstaff parries. Nothing you wear slows you down. Putting things on and taking them off
takes time, and what you wear still weighs what it weighs.

## The equipment screen

Press `e`. The screen lists every slot with its letter and what is in it, or `—` when it is
empty, and below them what you carry and weigh, as the inventory does:

{{#include generated/equipment.md}}

"What goes there" lists every object in the game that fits the slot, and the last column is how
long putting one on there, or taking it off, takes at a normal pace.

- Press a slot's letter, or move the `>` with the arrow keys and press `Enter`, to choose what
  goes there. On this screen every letter names a slot, even `e`, `h`, `j`, `k`, and `l`.
- The next screen lists what you carry that fits the slot, with each object's letter from your
  [pack](objects.md#your-pack). Press its letter, or `Enter` on the `>` row, to put it on.
- If the slot already holds something, it is taken off first: the two swap places. Choose what
  is already in the slot (the `>` starts on it) to take it off.
- `Esc` goes back to the equipment screen, and `Esc` there back to the game.

What you choose is done at once, and the equipment screen stays open, showing what changed, so
you can carry on dressing. Each change takes its time, and the game goes on around you while you
do it: creatures act meanwhile, just as when you wait. Looking at the screens takes no time.

## What fits where

- **Rings** fit either ring slot. Choosing a ring for the right hand while it is on your left
  moves it over.
- **Two-handed weapons**, such as the quarterstaff, take both hands. The screen shows them in
  the main hand and the off hand, "(both hands)". Wielding one puts away whatever your hands
  held; a shield in your off hand puts a two-handed weapon away.
- **Missile weapons and missiles** have their own slots. A bow shoots arrows: each missile weapon
  shoots one kind of missile ("shoots arrows" in the table above). You can ready any missiles,
  matching or not, but only the matching ones can be shot ([Ranged combat](ranged.md)); rocks
  readied there are thrown with `t`.
- Things that are not equipment (potions, food, coins, tools) fit nowhere.

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

What you wear and wield stays in your pack, with its letter. The [inventory](objects.md#your-pack)
marks it: "(being worn)", "(wielded)", "(in off hand)", "(on left hand)", "(on right hand)",
"(readied)", "(quivered)", and "(wielded in both hands)" for a two-handed weapon. It still counts
toward the weight you carry, and so toward your [burden](burden.md).

**Dropping** something you wear or wield takes it off first ("You take off a leather jerkin.")
and then drops it, which takes the time of both. The objects after it in your pack move up a
letter, and stay worn.

## Creatures

Creatures wear and wield some of what they carry, and you can see it. [Look mode](controls.md#look-mode)
says so: "A kobold (hostile, unhurt), wielding a spear, on stone floor.", "A kobold shaman
(hostile, unhurt), wearing a bone amulet, ...", "A kobold archer (hostile, unhurt), with a short
bow and 20 arrows ready, ...". You still cannot see anything else they carry. They fight with it
as you do, and keep what they start with until they die: then all of it falls where they stood.
