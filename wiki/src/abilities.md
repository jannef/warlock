# Abilities

Some of what you can do comes from who you are: your [race](races.md), your [class](classes.md), the [skills](skills.md) you are trained
in, your [weapon skills](weapon-skills.md) and [fighting styles](fighting-styles.md), and later the things you carry. These are your
**abilities**. They do not each get a key of their own. Instead they are all in one menu.

The basic commands keep their keys (see [Controls](controls.md)): moving, opening [doors](doors.md), [kicking](doors.md#kicking-a-door-open), [searching](searching.md#how-to-search),
[picking up](objects.md#picking-up), [eating](food.md#eating), [shooting](ranged.md#shooting-and-throwing), [resting](resting.md#how-resting-works), and so on.

## The abilities menu

`a` opens the abilities menu. It lists your abilities with a letter each, in groups by where they
come from: Racial, Class, Talents, Skills, Weapon skills, Items. A group you have nothing in is
left out.

- Beside each ability is what it asks for once you choose it, such as `in a direction`.
- An ability you cannot use right now is greyed out, with the reason: `you hold no shield`, `not
  usable yet` for a [skill](skills.md) whose use is not in the game yet, or `you are winded` while a
  [status effect](status-effects.md#buffs-and-cooldowns) keeps you from it. One that works on
  something next to you is greyed out while there is nothing beside you to use it on: `no lock
  beside you`, `no foe within reach` (see [Something beside you](controls.md#something-beside-you)).
- **Listen at a door** and **Wedge a door** are left out of the menu unless a closed
  [door](doors.md) is next to you (for Wedge, one not wedged already).
- The arrow keys move the `>` cursor, and the ability under it is described at the bottom.
- A letter, or `Enter` on the `>` row, uses the ability. The menu closes and the ability asks for
  what it needs: a direction on the message line (only when more than one thing beside you could
  be meant), or a place with the aiming cursor (see
  [Aiming](controls.md#aiming)). Then it takes its time like any other action. Choosing one you cannot use only
  tells you why.
- A long list shows a page at a time. `>` and `<` turn the pages, and the letters start again at
  `a` on every page.
- `Esc` closes the menu.

Looking at the menu takes no time. It remembers the ability you used last and opens on it, so `a`
`Enter` uses it again. `X` does the same in one key. With nothing used yet, `X` opens the menu.

## Every ability

Every ability the menu can show, grouped as the menu groups them. The last column links the
section that explains it in full: a [class](classes.md#levels)'s abilities under its class, the
priest's prayer on [Prayer](prayer.md), a skill's use on [Using your skills](skill-uses.md).

{{#include generated/abilities.md}}

[Spirit weapon](#ability-spirit-weapon) asks what shape the weapon should take. It shows a list of every melee weapon,
grouped by [weapon skill](weapon-skills.md), each with the numbers a plain one has. The weapon it calls into your hand
is that plain weapon in every way: its damage, its weight, and how it is [held](equipment.md#what-fits-where). On top of that it
has the talent's gifts: +3 to hit, a [threat](combat.md#criticals) 2 lower (a long sword's criticals on 18 or better
instead of 20), and every point of the target's [Protection](combat.md#protection) ignored. Calling it again reshapes
it. It fades the moment it leaves your hands, and it cannot be thrown.

What the ranks of [weapon skills](weapon-skills.md) and [fighting styles](fighting-styles.md) teach is in the menu too, under *Weapon
skills* after the skill's name ("Long blades: Deep cut"). These are greyed with `fires by itself`:
you never choose them, they work by themselves as you fight. They have their own tables on
[Weapon skills](weapon-skills.md#what-a-rank-teaches) and [Fighting styles](fighting-styles.md#what-a-rank-teaches).

- **Throw your voice**: a voice sounds at a place you choose, one you see and that is not solid,
  within 6 cells. You pick it with the [aiming](controls.md#aiming) cursor, as for a shot. It is as loud there as
  someone talking, and it is made there, not where you stand. So [creatures](creatures.md) that hear it and cannot
  see that spot go to have a look (see [Hearing](hearing.md#throwing-your-voice)). It takes a normal action. [Bards](classes.md#bard) learn it
  at level 16 (see [Classes](classes.md)).
- **Lurk**: the [thief](classes.md#thief)'s, from level 6. You wait in ambush, in the time of a wait, and are
  [Lurking](status-effects.md#status-lurking) until your next action. A hostile that steps up beside you meanwhile takes
  your blow at once, as a [free blow](stances.md#free-blows). Unless it was already hunting you, it is caught unawares:
  the blow is a [sneak attack](skills.md#skill-sneak-attack), it may be [Surprised](status-effects.md#status-surprised), and it is struck without its Dexterity
  and Dodge. You cannot lurk with a hostile you see already beside you ("Too late to lurk.", no
  time). It costs nothing and has no cooldown.
- **Exploit**: the [thief](classes.md#thief)'s, from level 1. It spends every [opening](resources.md#resource-openings) you hold on a melee blow, in the time of
  one, at the hostile in the direction you choose. It is a [sneak attack](skills.md#skill-sneak-attack) whatever the foe knows: the
  dice land if it hits, and 1d6 more for each opening spent. The foe is not [Surprised](status-effects.md#status-surprised) by it and keeps its whole Defence.
  [Low blow](classes.md#talent-low-blow) (level 12) adds +1 to hit per opening and stuns the foe it hits. It is refused, costing nothing, with no
  opening, nobody hostile there, a creature other than your [mark](resources.md#the-mark), or one you see without vitals (see [Exploit](resources.md#exploit)).
- **Thrown exploit**: the [thief](classes.md#thief)'s, from level 18. The same blow as Exploit,
  made by throwing a weapon at a hostile you see, in the time of a throw. Pick what to throw
  (a dagger, say; never a flask), then the target. It spends every opening you hold, and the
  weapon lands by the foe. It reaches as far as you can throw the weapon, but never more than 6
  cells, the farthest a [sneak attack](surprise.md#sneak-attacks) lands from a throw
  ([Long reach](talents.md#talent-long-reach) adds to that). It is refused, costing nothing, for
  Exploit's reasons, a target out of reach, or a wall or creature you see in the way (see
  [Thrown exploit](resources.md#thrown-exploit)).
- **Shield bash**: anyone can bash with a shield (see [Weapon skills](weapon-skills.md)), so it is always in the menu,
  and you can use it while you hold a shield. It strikes whoever stands in the direction you
  choose, in the time of a blow.
- **Skill uses**: a [skill](skills.md) that lets you do something puts its ability in the menu,
  in the skill's place among the Skills, once you are trained in it. These are **Disarm a trap**,
  **Listen at a door**, **Pick lock**, **Steal**, **Feint**, **Intimidate**, **Heal**, **Wedge a
  door**, **Forage**, **Tame**, and the professions' rows (**Metalcraft…**, **Woodcraft…**,
  **Jewelcraft…**). How each works is on [Using your skills](skill-uses.md), or on the page the
  table's last column links. A skill whose use is not in the game yet is listed too, greyed out
  (`not usable yet`).
- **[Class abilities](classes.md#levels)** ([Rage](abilities.md#ability-rage), [War cry](abilities.md#ability-war-cry), [Charge](abilities.md#ability-charge), [Lullaby](abilities.md#ability-lullaby), [Cackle](abilities.md#ability-cackle), [Veteran's wind](abilities.md#ability-veterans-wind), [Lay on hands](abilities.md#ability-lay-on-hands), [Smite](abilities.md#ability-smite), [Holy radiance](abilities.md#ability-holy-radiance),
  [Step of the wind](abilities.md#ability-step-of-the-wind), [Stunning strike](abilities.md#ability-stunning-strike), [Patient defence](abilities.md#ability-patient-defence), [Stillness](abilities.md#ability-stillness), [Quivering palm](abilities.md#ability-quivering-palm), [Consecrate](abilities.md#ability-consecrate), [Orison](abilities.md#ability-orison), [Sanctuary](abilities.md#ability-sanctuary), [Knock](abilities.md#ability-knock)) come
  from your [class](classes.md)'s level table (see [Classes](classes.md#levels)). Each is under Class from the level that brings
  it.
  - Some leave you [winded](status-effects.md#status-winded) for a while afterwards, and are greyed out until it passes.
  - Most are paid from a [resource](resources.md) instead, and are greyed out while you have too little.
  - A [paladin](classes.md#paladin)'s holy gifts are never greyed out by [alignment](alignment.md), but they do nothing unless you are
    Kind or better (see [Alignment](alignment.md)).
  - They have no key of their own yet: `X` uses the one you used last again.
- **[Charge](abilities.md#ability-charge)**, the [warrior](classes.md#warrior)'s at level 1, rushes a foe you see.
  Aim it like a shot, at the creature. You run along the straight line to it, stop in the cell
  before it, and strike it once, as an ordinary [melee attack](combat.md).
  - It costs 1 [stamina](resources.md#resource-stamina), plus 1 for every cell you run. It takes the time
    of those steps; the blow is free.
  - You must gain at least one cell, so it is refused when you already stand beside the foe.
  - A wall, a closed door, deep water, ice, a [trap](traps.md) you know of, or anyone you see on
    the line refuses it before you move. So does too little stamina for the whole run, or a run out of a
    [shop](shops.md)'s door while you owe it. A refused
    charge takes no time and costs nothing.
  - Nobody you pass or leave behind gets a [free blow](stances.md#free-blows) at you on the way.
  - Someone you cannot see on the line stops you short, with no blow. A hidden trap goes off as
    you run over it and stops you there.
  - Charge also gives the warrior their stamina pool from level 1.
  - With [Whirling charge](classes.md#talent-whirling-charge) (the warrior's at level 50), every
    hostile beside a cell you run into takes a [free blow](stances.md#free-blows) as you pass, in
    the order you pass them. Each is struck once a charge. The foe you charge at gets only the
    charge's own blow at the end. The blows cost no time and no stamina, and a foe that falls
    does not stop you.
- **[War cry](abilities.md#ability-war-cry)**, the [berserker](classes.md#berserker)'s at level 25, is paid in [rage](resources.md): 4 of it, plus the time of a wait.
  So it is greyed out with "you have not the rage for it" until you [rage](abilities.md#ability-rage). You roar as loud as a
  [howl](hearing.md), heard across the floor; it wakes sleepers and draws the curious.
  - Every hostile that hears you, sees you, and has [vitals](combat.md#criticals) (no skeletons, spirits, or animated
    things) rolls 2d12 plus its Willpower modifier against 10 + your Strength modifier + your
    [Intimidate](skills.md#skill-intimidate) bonus (+8 at 100, +12 at 200). If it falls short, it is [frightened](status-effects.md#status-frightened) three degrees and
    runs from you for about three turns.
  - There is no level cut: a troll checks as a kobold does, only with more nerve. For example, a
    creature of Willpower 10 against a berserker of Strength 15 (+2), untrained in Intimidate,
    holds on 12 or more, so it flees 38 % of the time. At Intimidate 100 it flees 90 %.
  - A roar with nobody near is still a roar: the turn, the rage, and the noise are spent.
  - Every cry trains Intimidate.
- **[Inspire](abilities.md#ability-inspire)** and the **[Lullaby](abilities.md#ability-lullaby)**, the [bard](classes.md#bard)'s at levels 9 and 20, are songs: see [Songs](#songs) below.
- **Allies** ([Come to heel](abilities.md#ability-heel), [Hold here](abilities.md#ability-hold), [Set on](abilities.md#ability-set-on)): once a creature fights on your side (a druid's call, a
  ranger's companion; see [Allies](allies.md)), the Allies group appears. Its three orders tell every ally of
  yours on the level what to do at once:
  - follow you and fight what threatens you (the default);
  - hold where they stand;
  - go to a place you see, fight what is near it, then come back.

  An order takes no time, but it is a shout, and anything near enough hears it (see [Hearing](hearing.md)).
  The orders are greyed out with "no allies here" while none of your allies is on the level with
  you. Each order, and each kill your allies make, trains [Handle animal](skills.md#skill-handle-animal), as does a try at [taming](companions.md#taming).

  [Give](abilities.md#ability-give) is not an order. It fits a companion with [something for its neck](companions.md#the-collar), and takes time. It is
  greyed out with "not beside you" while the companion is away. Set on asks for the place the way
  aiming a shot does, and Tab skips your own allies. On the map, an ally of yours has a short, faint
  green line under it, and looking at one says it is yours ("Your wolf (unhurt)").
- No [race](races.md) gives an ability yet.

## Songs

A [bard](classes.md#bard) sings two songs, both from the menu under Class: **[Inspire](abilities.md#ability-inspire)**, sung to yourself, and the
**[Lullaby](abilities.md#ability-lullaby)**, sung against your foes.

- **Price.** Each song costs one thing. Inspire costs 3 [inspiration](resources.md#resource-inspiration), and the Lullaby 6 [mana](mana.md#spending-it). With
  less, the song is greyed out ("not enough inspiration", "you have too little mana"), and choosing
  it costs nothing.
- **Sound.** A song takes as long as a chat (3 seconds at a [normal pace](turns.md#what-actions-cost)). It is [heard](hearing.md#what-makes-noise) as a voice of
  loudness 80 where you stand, so those who hear it and cannot see you come to look. You cannot
  sing on the [overworld](overworld.md#travel), or while [silenced](status-effects.md#status-silenced). Trying takes no time.
- **[Music](skills.md#skill-music) makes songs last longer.** Each song brings its status at one degree, and your Music
  adds degrees to every song, up to three, the most these statuses take. More degrees last longer;
  what a degree does stays the same. Sung again while it lasts, a song starts over at the degrees
  it brings now. Every song you sing trains Music, once you are trained in it, and your
  [Presence](stats.md#stat-presence).
- **Foes can shake a song off.** A song sung against foes is heard by every [hostile](creatures.md#factions) creature in
  earshot, seen or not. Each rolls 2d12 plus its modifier of the song's stat against the song's
  difficulty plus what your Music adds. If it falls short, it suffers the song. Friendly and
  peaceful creatures only hear it. There is no level cut: a troll checks as a kobold does, only
  with more nerve. A song with nobody to hear it is still sung: its price and its time are spent,
  and the noise is made.
- **[Virtuoso](classes.md#talent-virtuoso)** (the bard's at 30): every song costs nothing at all and brings one degree more.
  Inspire also lifts you higher: [greatly inspired](status-effects.md#status-inspired-greatly) (+2 to hit and +2 [Dodge](combat.md#defence), 10 minutes a degree)
  instead of [inspired](status-effects.md#status-inspired) (+1 and +1, 5 minutes a degree).

{{#include generated/songs.md}}

What Music adds to every song, by its value (see [Skills](skills.md#what-skills-give)):

{{#include generated/song-music.md}}

The chance a hostile creature shakes a song off, by its edge (its modifier of the stat the song
checks; see [Stats](stats.md)) and your Music, with the difficulty in brackets. A [drowsy](status-effects.md#status-drowsy) creature does
everything three times as slowly, 12 seconds a degree.

{{#include generated/song-odds.md}}

## Keys of their own

If you use an ability often, you can give it a key in the [Controls](controls.md#rebinding-keys) screen. None has a key by
default. The names there are:

- [shield bash](abilities.md#ability-shield-bash): "Bash with your shield"
- [disarming](traps.md#disarming-traps): "Disarm a trap"
- [picking a lock](doors.md#picking-locks): "Pick a lock"
- [stealing](skill-uses.md#stealing): "Steal"
- [throwing your voice](hearing.md#throwing-your-voice): "Throw your voice"

The key uses the ability just as the menu would, asking for a direction or a place if it needs
one. Without a shield, the bash key says so at once and takes no time. `a` ("Abilities") and `X`
("Use the last ability again") can be rebound like every key.

See also: [Using your skills](skill-uses.md), [Classes](classes.md#levels), and
[Resources](resources.md).
