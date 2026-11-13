# Abilities

Some of what you can do comes from who you are: your [race](races.md), your [class](classes.md), the [skills](skills.md) you are trained
in, your [weapon skills](weapon-skills.md) and [fighting styles](fighting-styles.md), and later the things you carry. These are your
**abilities**. They do not each get a key of their own. Instead they are all in one menu.

The basic commands keep their keys (see [Controls](controls.md)): moving, opening [doors](doors.md), [kicking](doors.md#kicking), [searching](searching.md#searching-1),
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
- **Low blow**: the [thief](classes.md#thief)'s, from level 12, for 3 [openings](resources.md#resource-openings). It is a melee blow, in the time of
  one, at the hostile in the direction you choose. It is a [sneak attack](skills.md#skill-sneak-attack) whatever the foe knows: the
  dice land if it hits. The foe is not [Surprised](status-effects.md#status-surprised) by it and keeps its whole Defence. A thing
  with no vitals takes the blow without the dice (see [Criticals](combat.md#criticals)). With nobody hostile there, it
  is refused and costs nothing.
- **Shield bash**: anyone can bash with a shield (see [Weapon skills](weapon-skills.md)), so it is always in the menu,
  and you can use it while you hold a shield. It strikes whoever stands in the direction you
  choose, in the time of a blow.
- **Disarm a trap**: everyone trained in [Disable device](skills.md#skill-disable-device) (the [thief](classes.md#thief)) has it, in the skill's place. Choose
  a direction toward a [trap](traps.md#finding-traps) you know of next to you, on the floor or on a [door](doors.md), and you try to
  take it apart (see [Traps](traps.md#disarming-traps)).
- **Listen at a door**: everyone trained in [Listen](skills.md#skill-listen) has it, in the skill's place. Choose a direction
  toward a closed [door](doors.md) next to you. You stop for a turn and hear what moves beyond it (see
  [Hearing](hearing.md#listening-at-a-door)).
- **Pick lock**: everyone trained in [Open lock](skills.md#skill-open-lock) (the [thief](classes.md#thief)) has it, in the skill's place. Choose a
  direction toward a closed [door](doors.md) next to you, and you try its lock (see [Doors](doors.md#picking-locks)).
- **Steal**: anyone trained in the [Steal](skills.md#skill-steal) skill has it, in the skill's place among the Skills. It asks
  for a direction and tries to lift something from whoever stands there, if they are not hostile to
  you (see [Stealing](skill-uses.md#stealing)).
- **Feint**: anyone trained in the [Bluff](skills.md#skill-bluff) skill has it, in the skill's place among the Skills. It asks
  for a direction and lies to the hostile creature there. Unless it sees through you, it is
  [surprised](status-effects.md#status-surprised), and your next blow finds it off guard (see [Feinting](skill-uses.md#feinting)). For a
  [thief](classes.md#thief) it makes the creature the [mark](resources.md#the-mark), and a feint it
  falls for gives an [opening](resources.md#resource-openings). This is not the
  Duelist style's [False lead](fighting-styles.md#style-ability-false-lead), which is a blow.
- **Intimidate**: anyone trained in the [Intimidate](skills.md#skill-intimidate) skill has it, in the skill's place among the
  Skills. It asks for a place you see within 6 cells and threatens the hostile creature there.
  Unless it keeps its nerve, it is [frightened](status-effects.md#status-frightened) and flees, for longer the better your skill (see
  [Intimidating](skill-uses.md#intimidating)).
- **Heal**: anyone trained in the [Heal](skills.md#skill-heal) skill has it, in the skill's place among the Skills. It treats
  you, or an [ally](allies.md) beside you, for [bleeding](status-effects.md#bleeding), a [poison](status-effects.md#poisoned), or a [disease](diseases.md#how-a-disease-goes) that shows. It asks which when
  there is more than one. Otherwise it is greyed out: "nothing to treat" (see
  [Treating yourself](skill-uses.md#treating-yourself)).
- **Metalcraft…** and **Woodcraft…**: those trained in [Metalcraft](skills.md#skill-metalcraft)
  or [Woodcraft](skills.md#skill-woodcraft) have its row, in the skill's place among the Skills. Each opens that [profession](skill-uses.md#professions)'s list of recipes.
  Metalcraft's holds **Temper** (see [Forges](altars-forges-springs.md#forges)); Woodcraft's
  holds **Arrows** and **Crossbow bolts** (see [Fletching](skill-uses.md#fletching)), **Sling
  stones**, and **Whittle** (see [Whittling](skill-uses.md#whittling)). A recipe you cannot make
  now is greyed out with the reason: "no anvil beside you", "no knife", "no bow ready".
- **Jewelcraft…**: those trained in [Jewelcraft](skills.md#skill-jewelcraft) have it, in the
  skill's place. It holds **Cut** (see [Gem cutting](gem-cutting.md)), **Set** (see
  [Setting a gem](gem-cutting.md#setting-a-gem)), and the rings and amulets (see
  [Making jewellery](crafting.md#making-jewellery)).
- **Wedge a door**: anyone trained in [Woodcraft](skills.md#skill-woodcraft) has it, in the skill's place among the Skills. Choose
  a direction, and you spend a turn wedging the closed door that way shut, so that creatures must
  kick it in (see [Wedging a door](doors.md#wedging-a-door)).
- **Forage**: anyone trained in [Survival](skills.md#skill-survival) has it, in the skill's place among the Skills. It asks for
  nothing. You spend ten minutes searching the wild around you for food. It works only in the wild,
  on the [overworld](overworld.md#travel) and in its [local areas](overworld.md#local-areas). Elsewhere it is greyed out: "not usable here" (see
  [Foraging](skill-uses.md#foraging)).
- **Tame**: anyone trained in [Handle animal](skills.md#skill-handle-animal) has it, in the skill's place among the Skills. It asks for
  the direction of a beast next to you. A while beside it decides whether it becomes your
  [companion](companions.md#companions) for good (see [Taming](companions.md#taming)). It is greyed out with "you already have a companion"
  while you have one.
- **Other skills used as actions** ([Knowledge](skills.md#skill-knowledge), ...): each is in the menu once you are trained in it
  (see [Skills](skills.md)), greyed out until its use comes to the game.
- **[Class abilities](talents.md#class-abilities)** ([Rage](abilities.md#ability-rage), [War cry](abilities.md#ability-war-cry), [Charge](abilities.md#ability-charge), [Lullaby](abilities.md#ability-lullaby), [Cackle](abilities.md#ability-cackle), [Veteran's wind](abilities.md#ability-veterans-wind), [Lay on hands](abilities.md#ability-lay-on-hands), [Smite](abilities.md#ability-smite), [Holy radiance](abilities.md#ability-holy-radiance),
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
    the line refuses it before you move. So does too little stamina for the whole run. A refused
    charge takes no time and costs nothing.
  - Nobody you pass or leave behind gets a [free blow](stances.md#free-blows) at you on the way.
  - Someone you cannot see on the line stops you short, with no blow. A hidden trap goes off as
    you run over it and stops you there.
  - Charge also gives the warrior their stamina pool from level 1.
  - With [Whirling charge](talents.md#talent-whirling-charge) (the warrior's at level 50), every
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
- **[Virtuoso](talents.md#talent-virtuoso)** (the bard's at 30): every song costs nothing at all and brings one degree more.
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
