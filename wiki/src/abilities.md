# Abilities

Some of what you can do comes from who you are: your [race](races.md), your [class](classes.md), the [skills](skills.md) you are
trained in, your [weapon skills](weapon-skills.md) and [fighting styles](fighting-styles.md), and, later, things you carry. These are your
**abilities**. Unlike in ADOM, they do not each get a key of their own: they are all in one
menu. The basic commands (moving, opening [doors](doors.md), [kicking](doors.md#kicking), [searching](searching.md#searching-1), [picking up](objects.md#picking-up), [eating](food.md#eating), [shooting](ranged.md#shooting-and-throwing),
[resting](resting.md#how-resting-works), ...) keep their keys (see [Controls](controls.md)).

## The abilities menu

`a` opens the abilities menu. It lists your abilities in groups, by where they come from
(Racial, Class, Talents, Skills, Weapon skills, Items; a group you have nothing in is left out), each with
a letter:

- beside each, what it asks for once you choose it, such as `in a direction`;
- an ability you cannot use right now is greyed out, with why: `you hold no shield`, `not
  usable yet` for a [skill](skills.md) whose use is not in the game yet, or `you are winded` while a
  [status effect](status-effects.md#buffs-and-cooldowns) keeps you from it;
- the arrow keys move the `>` cursor, and the ability under it is described at the bottom;
- a letter, or `Enter` on the `>` row, uses it: the menu closes, the ability asks for what it
  needs (a direction, on the message line, or a place, with the aiming cursor, see
  [Aiming](controls.md#aiming)), and then it takes its time like any other action.
  Choosing one you cannot use only tells you why;
- a long list shows a page at a time: `>` and `<` turn the pages, and the letters start again at
  `a` on every page;
- `Esc` closes the menu.

Looking at the menu takes no time. It remembers the ability you used last and opens on it, so
`a` `Enter` uses it again, and `X` does the same in one key (with nothing used yet, `X` opens the
menu).

## Every ability

{{#include generated/abilities.md}}

[Spirit weapon](#ability-spirit-weapon) asks for a weapon's shape: a list of every melee weapon,
grouped by [weapon skill](weapon-skills.md), each with the numbers a plain one has. The weapon it
calls into your hand is that plain weapon in every way (its damage, weight, and how it is
[held](equipment.md#what-fits-where)) with the talent's gifts besides: +3 to hit, a
[threat](combat.md#criticals) 2 lower (a long sword's criticals on 18 or better instead of 20),
and every point of the target's [Protection](combat.md#protection) ignored. Calling it again
reshapes it; it fades the moment it leaves your hands, and it cannot be thrown.

What the ranks of [weapon skills](weapon-skills.md) and [fighting styles](fighting-styles.md) teach is listed in the menu too, under
*Weapon skills* after the skill's name ("Long blades: Deep cut"), greyed with `fires by itself`:
those abilities are never chosen, they work by themselves as you fight. They have tables of their
own: [Weapon skills](weapon-skills.md#what-a-rank-teaches) and
[Fighting styles](fighting-styles.md#what-a-rank-teaches).

- **Throw your voice**: a voice sounds at a place you choose that you see, not solid, within 6
  cells (the [aiming](controls.md#aiming) cursor, as for a shot). It is as loud there as someone talking, and it is
  made there, not where you stand, so [creatures](creatures.md) that hear it and cannot see that spot go to have
  a look (see [Hearing](hearing.md#throwing-your-voice)). It takes a normal action. [Bards](classes.md#bard) learn it at level 16
  (see [Classes](classes.md)).
- **Lurk**: the [thief](classes.md#thief)'s, from level 6. You wait in ambush, in the time of a
  wait, and are [Lurking](status-effects.md#status-lurking) until your next action. A hostile
  that steps up beside you meanwhile takes your blow at once, a [free blow](combat.md#free-blows),
  and unless it was already hunting you it is caught unawares by it: the blow is a
  [sneak attack](skills.md#skill-sneak-attack), it may be [Surprised](status-effects.md#status-surprised),
  and it is struck without its Dexterity and Dodge. You cannot lurk with a hostile you see beside
  you already ("Too late to lurk.", no time). It costs nothing and has no cooldown.
- **Low blow**: the [thief](classes.md#thief)'s, from level 12, for 3
  [openings](resources.md#resource-openings). A melee blow, in the time of one, at the hostile in
  the direction you choose, that is a [sneak attack](skills.md#skill-sneak-attack) whatever it
  knows: the dice land if it hits. The foe is not [Surprised](status-effects.md#status-surprised)
  by it and keeps its whole Defence. A thing with no vitals takes the blow without the dice (see
  [Criticals](combat.md#criticals)). With nobody hostile there it is refused, spending nothing.
- **Shield bash**: anyone can bash with a shield (see [Weapon skills](weapon-skills.md)), so it is
  always in the menu, and you can use it while you hold a shield. It strikes whoever stands in
  the direction you choose, in the time of a blow.
- **Disarm a trap**: everyone trained in [Disable device](skills.md#skill-disable-device) (the [thief](classes.md#thief)) has it, in place of the
  skill. Choose a direction, toward a [trap](traps.md#finding-traps) you know of next to you, on the floor or on a [door](doors.md):
  you try to take it apart (see [Traps](traps.md#disarming-traps)).
- **Pick lock**: everyone trained in [Open lock](skills.md#skill-open-lock) (the [thief](classes.md#thief)) has it, in place of the skill. Choose a
  direction, toward a closed [door](doors.md) next to you: you try its lock (see
  [Doors](doors.md#picking-locks)).
- **Steal**: anyone trained in the [Steal](skills.md#skill-steal) skill has it, in the skill's place among the Skills.
  It asks for a direction and tries to lift something from whoever stands there, if they are
  not hostile to you (see [Stealing](skills.md#stealing)).
- **Intimidate**: anyone trained in the [Intimidate](skills.md#skill-intimidate) skill has it, in the skill's place among
  the Skills. It asks for a place you see, within 6 cells, and threatens the hostile creature
  there: unless it keeps its nerve it is [frightened](status-effects.md#status-frightened) and
  flees, longer the better your skill (see [Intimidating](skills.md#intimidating)).
- **Heal**: anyone trained in the [Heal](skills.md#skill-heal) skill has it, in the skill's place among the Skills. It
  treats you: your [bleeding](status-effects.md#bleeding), a [poison](status-effects.md#poisoned), or a [disease](diseases.md#how-a-disease-goes) that shows, asking which when there is more
  than one; greyed out "nothing to treat" otherwise (see
  [Treating yourself](skills.md#treating-yourself)).
- **Forage**: anyone trained in [Survival](skills.md#skill-survival) has it, in the skill's place among the Skills. It asks
  for nothing: you spend ten minutes searching the wild around you for food. Only in the wild,
  on the [overworld](overworld.md#travel) and in its [local areas](overworld.md#local-areas); elsewhere it is greyed out, "not usable here" (see
  [Foraging](skills.md#foraging)).
- **Other skills used as actions** ([Handle animal](skills.md#skill-handle-animal), ...): each is in the menu once you are trained
  in it (see [Skills](skills.md)), greyed out until its use comes to the game.
- **[Class abilities](talents.md#class-abilities)** ([Rage](abilities.md#ability-rage), [War cry](abilities.md#ability-war-cry), [Lullaby](abilities.md#ability-lullaby), [Cackle](abilities.md#ability-cackle), [Veteran's wind](abilities.md#ability-veterans-wind), [Whirlwind](abilities.md#ability-whirlwind), [Lay on hands](abilities.md#ability-lay-on-hands), [Smite](abilities.md#ability-smite), [Holy radiance](abilities.md#ability-holy-radiance),
  [Step of the wind](abilities.md#ability-step-of-the-wind), [Stunning strike](abilities.md#ability-stunning-strike), [Patient defence](abilities.md#ability-patient-defence), [Stillness](abilities.md#ability-stillness), [Quivering palm](abilities.md#ability-quivering-palm), [Consecrate](abilities.md#ability-consecrate), [Orison](abilities.md#ability-orison), [Sanctuary](abilities.md#ability-sanctuary), [Knock](abilities.md#ability-knock)): your [class](classes.md)'s level table gives them (see [Classes](classes.md#levels)),
  each under Class from the level that brings it. Some leave you [winded](status-effects.md#status-winded) for a while afterwards, greyed out until it passes, and most are paid
  from a [resource](resources.md) instead (greyed out while you hold too little); a [paladin](classes.md#paladin)'s holy gifts are never greyed out by
  [alignment](alignment.md), but do nothing unless you are Kind or better (see [Alignment](alignment.md)). They have no key of their own
  yet: `X` uses the one you used last again.
- **[War cry](abilities.md#ability-war-cry)**, the [berserker](classes.md#berserker)'s at level 25, is paid in
  [rage](resources.md): 4 of it and the time of a wait, so it is greyed out "you have not the
  rage for it" until you [rage](abilities.md#ability-rage). You roar as loud as a
  [howl](hearing.md), heard across the floor (it wakes sleepers and draws the curious). Every
  hostile that hears you, sees you, and has [vitals](combat.md#criticals) (no skeletons, spirits,
  or animated things) rolls its Willpower modifier on 2d12 against 10 + your Strength
  modifier + your [Intimidate](skills.md#skill-intimidate) (+8 at 100, +12 at 200); short of it,
  it is [frightened](status-effects.md#status-frightened) three degrees and runs from you, about
  three turns. There is no level cut: a troll checks as a kobold does, only with more nerve.
  For example, a creature of Willpower 10 against a berserker of Strength 15 (+2), untrained in
  Intimidate, stands on 14 or more: it flees 54 % of the time; at Intimidate 100 it flees
  96 %. A roar with nobody near is still a roar: the turn, the rage, and the noise are spent.
  Every cry trains Intimidate.
- **[Inspire](abilities.md#ability-inspire)** and the **[Lullaby](abilities.md#ability-lullaby)**,
  the [bard](classes.md#bard)'s at levels 9 and 20, are songs: see [Songs](#songs) below.
- **Allies** ([Come to heel](abilities.md#ability-heel), [Hold here](abilities.md#ability-hold), [Set on](abilities.md#ability-set-on)): once a creature fights on your side (the
  druid's calls and the ranger's companion are on their way), the Allies group appears, and the
  three orders tell every ally of yours on the level at once what to do: follow you and fight
  what threatens you (the default), hold where they stand, or go to a place you see and fight
  what is near it, then come back. An order takes no time, but it is a shout: anything near
  enough hears it (see [Hearing](hearing.md)). Greyed out "no allies here" while none of them is
  on the level with you. Each order, and each kill of theirs, trains [Handle animal](skills.md#skill-handle-animal).
  Set on asks for the place as aiming a shot does; Tab skips your own allies. On the map an ally
  of yours has a short, faint green line under it, and looking at one says it is yours ("Your
  wolf (unhurt)").
- No [race](races.md) gives an ability yet.

## Songs

A [bard](classes.md#bard) sings two songs, both from the menu, under Class: **[Inspire](abilities.md#ability-inspire)**, sung
to yourself, and the **[Lullaby](abilities.md#ability-lullaby)**, sung against your foes.

- **Paid in one thing each.** Inspire costs 3 [inspiration](resources.md#resource-inspiration) and nothing
  else, the Lullaby 6 [mana](mana.md#spending-it) and nothing else. With less, the song is greyed
  out ("not enough inspiration", "you have too little mana") and choosing it costs
  nothing.
- **Heard.** A song takes as long as a chat (3 seconds at a [normal pace](turns.md#what-actions-cost)) and is
  [heard](hearing.md#what-makes-noise) as a voice of loudness 80 where you stand: those who
  hear it and cannot see you come to look. You cannot sing on the
  [overworld](overworld.md#travel), or while [silenced](status-effects.md#status-silenced); trying
  takes no time.
- **Lengthened by [Music](skills.md#skill-music).** Each song brings its status at one degree, and
  your Music adds degrees to every song (more degrees last longer; what a degree does stays the
  same), up to three, the most these statuses take. Sung again while it lasts, a song starts over at the
  degrees it brings now. Every song you sing trains Music, once you are trained in it, and your
  [Presence](stats.md#stat-presence).
- **Shaken off.** A song sung against foes is heard by every [hostile](creatures.md#factions)
  creature in earshot, seen or not, and each rolls its stat's modifier on 2d12 against the
  song's difficulty plus what your Music adds; short of it, it suffers the song. Friendly and
  peaceful creatures only hear it. There is no level cut: a troll checks as a kobold does, only
  with more nerve. A song with nobody to hear it is still sung: its price and its time are spent,
  and the noise made.
- **[Virtuoso](talents.md#talent-virtuoso)** (the bard's at 30): every song costs nothing at all and
  brings a degree more, and Inspire lifts you higher, [greatly inspired](status-effects.md#status-inspired-greatly)
  (+2 to hit and +2 [Dodge](combat.md#defence), 10 minutes a degree) instead of
  [inspired](status-effects.md#status-inspired) (+1 and +1, 5 minutes a degree).

{{#include generated/songs.md}}

What Music adds to every song, by its value (see [Skills](skills.md#what-skills-give)):

{{#include generated/song-music.md}}

The chance a hostile creature shakes a song off, by its edge (its modifier of the stat the song
checks; see [Stats](stats.md)) and your Music, the difficulty in brackets. A
[drowsy](status-effects.md#status-drowsy) creature does everything three times as slowly, 12
seconds a degree.

{{#include generated/song-odds.md}}

## Keys of their own

If you use an ability often, you can give it a key in the [Controls](controls.md#rebinding-keys) screen (the [shield bash](abilities.md#ability-shield-bash) is
"Bash with your shield", [disarming](traps.md#disarming-traps) "Disarm a trap", [picking a lock](doors.md#picking-locks) "Pick a lock", [stealing](skills.md#stealing)
"Steal", and [throwing your voice](hearing.md#throwing-your-voice) "Throw your voice"; none has a
key by default): the key uses the ability just as the menu would, asking for a direction or a
place if it needs one. Without a shield, it says so at once and takes no time. `a` ("Abilities")
and `X` ("Use the last ability again") can be rebound like every key.
