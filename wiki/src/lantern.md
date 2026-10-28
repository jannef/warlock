# Your lantern

A lantern is a [tool](objects.md#class-tools): it gives light only while it is ready in the **[Tool](equipment.md#what-fits-where)** slot of your
[equipment](equipment.md), where a [pick-axe](digging.md#digging) would go, so you hold one or the other. Every [race](races.md)
without [darkvision](vision.md#the-four-types) ([elves](races.md#race-elf), [half-elves](races.md#race-half-elf), [humans](races.md#race-human), [halflings](races.md#race-halfling), [gnomes](races.md#race-gnome), [lizardfolk](races.md#race-lizardfolk), and [ratfolk](races.md#race-ratfolk)) starts
with one ready, unlit; the others see in the dark and start without one. It weighs 400 g.

Press `u` (use a tool) to change what it does: from unlit to lit, from lit to hooded, and from
hooded back to unlit. Lighting it or putting it out is a [normal action](turns.md#what-actions-cost) (6 seconds); lowering the
[hood](lantern.md#the-hood) takes half one (3 seconds). Its name says how it is: "a lantern (lit)", "a lantern
(hooded)". A lantern you take off or drop keeps how it was, but gives no light until it is ready
in the Tool slot again.

A lit lantern is a [light source](light.md) that moves with you: a warm light whose reach is in
the [table of lights](light.md#light-sources). It burns calmly: it [flickers](light.md#what-is-only-for-show) a little, but less
than a torch. What it lights is what your [vision](vision.md) can see, so it is the answer to dark
[dungeons](dungeons.md) and to night outdoors, where the [moon](sun.md#moonlight) gives little light or none, or is hidden by
[clouds](weather.md#clouds-and-light). In [daylight](sun.md#the-suns-path) you do not need it.

Light works both ways, though. [Creatures](creatures.md) see by the same light you do, so a lit lantern shows you
to every creature with a clear line to you that is near enough to see you, even one whose eyes
need light. With the lantern out, standing in the dark hides you from such eyes, and you can
watch creatures that carry their own light without being seen. See [Creatures](creatures.md).

## The hood

Your lantern has a hood: `u` on a lit lantern lowers it, and `u` again puts the lantern out
(lit again, it burns open).

Hooded, the lantern gives a third of its light and reaches 2 cells instead of 5 (see the [table of
lights](light.md#light-sources)), and the hood keeps the light off you: it lights the ground
around you but not the cell you stand on.

- **You see less.** With [normal eyes](vision.md#the-four-types) you see the 8 cells around you by it and no farther (they get
  about 24 % of full light straight beside you, 19 % on the diagonals, and normal eyes need 16 %).
  [Low-light eyes](vision.md#the-four-types) see out to 2 cells, but for the far corners. [Darkvision](vision.md#the-four-types) is as good as ever.
- **You are seen less.** Your own cell gets none of the lantern's light, only a glimmer of it
  reflected off the ground nearby (at most an eighth of the light around you, too little for any
  eyes), so creatures whose eyes need light do not see you by it, even next to you, while you see
  them in its ring. Creatures with [darkvision](vision.md#the-four-types) still see you within their darkvision, and any light
  that falls on you (a torch, daylight) shows you as usual.

There is no fuel yet: the lantern burns as long as you leave it lit. The **[Lamplighter](talents.md#talent-lamplighter)**
[talent](talents.md) makes it burn brighter, a cell farther (and so you are seen from a little
farther too); hooded, it gives the hooded light as before.

With no light ready, `u` asks which way to dig if you have a [digging](digging.md#digging) tool ready, and otherwise
tells you that you have none; that takes no time.
