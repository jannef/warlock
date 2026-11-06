# Warlock — Monsters that differ, and the ones to dread

Design document for the owner's review (#1098; first draft 2026-11-06, revised the same day
after the owner's first review: the decisions are §17 and are applied throughout). Written
against `origin/main` at 187bcac3. Nothing of this document is implemented; §12 cuts it into
issues.
It builds on what is there: the creature data and its riders (`assets/data/creatures/README.toml`,
`docs/dev/creatures.md` and the tribe files under `docs/dev/creatures/`), the statuses and
their fields (`data/status.toml`, `docs/dev/status-effects.md`: holds, `escape`, `flees`,
`retaliates`, `hides`, `sleeps`, cooldown statuses), drains (`docs/dev/stats.md` "Drain"),
summoning (`docs/dev/summoning.md`), surprise and lurking (`docs/dev/surprise.md`), the danger
draw and rarities (`content/danger.rs`, `data/creature_rarities.toml`), hearing
(`docs/dev/hearing.md`, `noise.rs::SoundKind`), the spells' shared area code, the mimics (the one
dread creature the game already has, `docs/dev/creatures/mimics.md`), and the balance probe
(`docs/balance.md`). Every number is data and hidden in the game; the wiki states it (the owner's
rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-11-06, #1098): monsters should differ from each other, not only by
numbers along one curve, and a class of rare monsters should inspire **dread**, as ADOM's
vortices and claw bugs do. Sort every creature type into *bulk* and *dread*, propose each dread
creature's gimmick as a reusable mechanic with counterplay, say how much more spread a level's
monsters should have (with #1097's to-hit baseline of L/4 as the start), how rare a dread
creature should be and how the player learns of it before it is too late, and cut it into
issues, rules first. The tribes are the agent's call, approved by the integrator; the animated
objects are the owner's (§13).

Three issues of the same day are referenced, not redesigned: **#1097** (monster to-hit L/4 for
natural attacks, armed monsters unchanged; merged), **#1099** (the dragon AI: breath with a
cooldown, claws and bite, several natural attacks; merged), and **#1102** (enemy casters cast
real spells; low-level casters rare and weak; being built). Where a dread creature's gimmick
*is* breath or a spell list, this document points at them. Also merged since the first draft
and read where they touch this: **#1096** (a natural 22 to 24 always hits: the strikers' threat
range of §7 sits under it), **#1093** (shield blocking removed: no counterplay here reads a
shield), **#1095** (weapon marks never above 100 %, slower at high ranks: §5.9's "free marks"),
**#1101** (every damage spell's dice doubled: the bursts of §5.3 are scaled to the new ladder),
**#1100** (area spells ruin items on the floor: a burst does too), and **#1103** (caster corpses
teach Decipher script: nothing here).

## Contents

1. Summary
2. Research, tied to the decisions
3. What exists, and what this builds on
4. What dread is, and what it is not
5. The gimmicks: reusable mechanics, each with its cost and counterplay
6. The sort: every creature type, bulk or dread
7. Variance: how far a level's monsters should spread
8. Rarity: how often dread comes, and where
9. Warning signs: how the player learns of it in time
10. What the player sees
11. Talents and class abilities
12. Implementation plan (slices)
13. The animated objects: the owner's part
14. Determinism, saves, replays, dumps
15. Rejected ideas
16. Open questions for the owner, each with a recommended default
17. Decisions (owner, 2026-11-06)

## 1. Summary

| Piece | Proposal |
|---|---|
| **The sort** | of the 311 written types (14 people and the player, 19 beasts, **278 fighting types**) and the five generated families (16 household things, 16 blades, 24 hollow armours, 51 panoplies, 60 mimics): **51 fighting types are dread**, and one new type, the **invisible stalker** (§5.11), makes 52; some thirty more gain a *trait* (a rider that gives a floor its texture without changing how the fight is fought: a howl, a web, a troll's regeneration), and the rest, about two hundred, **stay bulk as they are** (§6). Dread goes by name and trope, mostly things the rosters already named for a trope they do not yet deliver: the basilisk, the shadow, the banshee, the nymph, the vampires, the motes, the charnel heap, the rug of smothering. |
| **Eleven gimmicks** (§5), and a table of riders that exist | seven are **new mechanisms**, each one field on the creature and one hook: the **gaze** (a rider at sight, not touch: G1), the **burst** (G3), the **aura** (G4), **splitting** and **what it dies into** (G5), **theft** (G6, on a cooldown, the thief staying to fight) with the **blink to reach you** (G6b), the **shadowy** (G10, unseen in the dark), and **invisibility** (G11: innate on the stalker, cast on a cooldown by the duergar, the imp, the quasit, and the wisp); the other four are nearly data: **stiffening** to stone (G2, a status with two small fields), **engulfing** (G7, a status with a hold and a tick: no code), **corroding gear** (G8, the smith's `damaged` state, with the materials that resist it: mithril, adamantium, gold, silver, the stones), **regeneration and what stops it** (G9, one number, only on what is dangerous by itself). Drains, holds, fear, transfixion, disease, sleep, webs, and retaliation are data on what exists (§5.12). Breath is #1099, spells #1102. Every gimmick has counterplay that exists; the one thing added for everyone is **Averted eyes** (§5.1), which the basilisk forces on whoever sees it rather than a toggle the player sets; never a talent gate. |
| **Fear gets teeth** | Frightened does nothing to the player today (only a creature `flees`). Proposed: the player's blows at **−3 to hit** while Frightened (`combat` on the status, data). Fear auras and howls then mean something; Iron will, Divine grace, Unbowed, the Ward power, and Still mind's Stillness already answer it. §5.4 |
| **Variance** (§7) | on #1097's L/4: to-hit spread **±L/8 by role** (the quick +, the brute −), hit points **0.5× to 2.0×** the standard's (today 0.6 to 1.6), damage spread by **threat and crit** (the strikers threaten from 19, the brutes ×3) rather than by mean, speed **bimodal** (bulk between 800 and 1250, no tribe all at 1000; dread at 600 or at 1400 and up). A measurable target: on a floor the fastest tenth of its draws kill the probe's warrior in half the median rounds and the toughest tenth take twice; the probe gains the row. |
| **Grades** (§7.4; decided: ship) | one in six spawns is **greater** (+25 % hit points, +1 to hit, +2 damage, a slower step) and one in six **lesser** (the reverse), a word in the look line ("a big kobold", "a scrawny kobold"), one `grade` field on the actor. Cheap, and the one change that makes two kobolds two fights. Its own slice. |
| **Rarity** (§8) | **no cap** (the owner): a `dread = true` flag on the type, and one roll at **every random monster draw**, level population and spawner alike: **one in `dread_one_in`** (default **200**, `dungeons.toml`'s `[defaults]`, a branch may override it as the draw odds are). On a hit the draw is among the dread types, **each at its own rarity**; otherwise among the rest; with no dread type eligible, the ordinary draw. No floor-level state; vaults and legends place what they place. Dread is not one rarity: the barbed devil is common, the basilisk very rare (the ghouls' hold stays a common trait of every floor, not dread). |
| **Warning signs** (§9) | four channels, all cheap: a type's **cry** (an idle sound now and then: a hiss, a wail, a giggle, a rattle, heard through doors by the hearing rules), its **light** (the motes, the hand of glory), its **leavings** laid by the generator in its vault or room (statues by a basilisk, webs by a spider, acid-pitted floor, torn cloth), and a **first-sight line** said once per run per type ("A basilisk. Do not meet its eyes.") that also goes into the message history and the wiki. And a fifth, the owner's (§9.5): the **graveyard file**, a list of every character who has died that survives every version, so a basilisk's statues can be **the player's own petrified dead by name**. |
| **Talents** | **none new** (decided): a creature gimmick extends the creatures system (CLAUDE.md's talent rule); the counters are existing talents, resistances, items, and play. §11 |
| **Slices** (§12) | D0 fear's teeth, the `dread` flag, the draw's roll, and the sort's rarity pass (data, first); D1 the gaze and the forced Averted eyes; D2 stiffening, the basilisk, and the statue; D3 the burst for every elemental line and the earth line's stones; D4 auras and cries; D5 splitting and dying into; D6 engulf, theft on a cooldown, and the blink to reach; D7 corroding gear with the materials' `corrodes` and the two new metals, and regeneration; D8 the shadowy and invisibility; D9 leavings and first-sight lines; D10 the variance pass and the probe's row; D11 grades; D12 the animated objects (decided: the rug, the bellows, the lantern, the blades); D13 the graveyard file and the named statues; D14 the second-wave types issue. |

**Key choices, and why**

1. *Dread is a gimmick that changes how you fight, not a bigger number.* ADOM's claw bug is dreaded for being fast and hitting hard, its vortex for exploding, its ghost lord for ageing you. The first is variance (§7); the second two are gimmicks (§5). Both are needed, and they are different work.
2. *Reuse before building.* Half of what the dread creatures do is riders and statuses the data already has, often on the very creature the trope belongs to (ghouls hold, wraiths drain, hags transfix). The seven new mechanisms are each one creature field and one hook, in the style of `lurks`, `incorporeal`, and `summon`.
3. *Counterplay by play, never by build.* Every gimmick is answered by something anyone can do (break line of sight, bring a light, use another weapon, burn it, step back, run) and by things the character may have (a resistance, a talent, a scroll). Nothing is gated behind a talent, and a failed check is never a lock: stiffening wears off, drains mend, stolen things drop where the thief dies (the no-anti-farming rule).
4. *Rare enough to be feared, and not all alike.* A gimmick met every floor is a chore (ADOM's grues). Dread comes by a roll at every random draw, one in a couple of hundred, and then by each type's own rarity (the owner: no hard cap; a ghoul-common dread and a basilisk-rare dread are both wanted); the player's dread is the name in the message line, not the fight.
5. *Warned, not ambushed.* The owner asked how the player learns of it before it is too late. Sounds through doors, light, leavings, and one line at first sight: the game stays vague, but it does not stay silent.

## 2. Research, tied to the decisions

| Source | What it does | Taken | Left, and why |
|---|---|---|---|
| ADOM (vortices that explode beside you; claw bugs and greater claw bugs, fast with many attacks; ghost lords that age; gorgons and basilisks that petrify; floating eyes that paralyse the one who hits them; dorn beasts; writhing masses of primal chaos that corrupt; mimics; grues in the dark) | the owner's model | the burst (G3, the motes become vortices); variance as a dread of its own (§7); the gaze and stiffening (G1, G2) with a wear-off; retaliation as the floating eye (the `retaliates` status, innate: G4b); the shadowy in the dark (G10) | ageing (age has no rule here); corruption (no system; a whole design); instant petrification (a permadeath with no warning is not dread, it is a dice roll: four degrees with lines instead) |
| NetHack (cockatrice and the touch of stone; nymphs that steal and teleport; leprechauns; rust monsters and disenchanters; puddings that split under iron; soldier ants; mind flayers; floating eyes; invisible stalkers and "It hits!"; "You hear the footsteps of a guard on patrol") | theft (G6, the nymph: on a cooldown, the thief staying), splitting (G5), corroding gear (G8), invisibility (G11, the stalker), the hearing warnings (§9) | the teleport after the theft (the owner: an annoying flee mechanic; the thief stays, and what it took falls where it dies); stoning by touching a corpse (too cute for this world); the disenchanter (an item losing its magic is a lock on a build: the `damaged` state the smith mends instead) |
| DCSS (hydras whose heads multiply; ugly things; orbs of fire; the OOD "out of depth" dread; sixfirhies; shadow dragons' draining breath) | dying into (G5b: the hydra's and wormhost's way); out-of-depth draws kept as they are (#375) | malmutation (no mutations) |
| Brogue (bog monsters that submerge and hold; pink jellies that split; acid mounds that corrode; vampire bats; goblin conjurers; dar blademasters that blink; the kraken) | engulf (G7), the blink on a creature (G6b, the blink dog), acid hides (G8) | the kraken (water-dwelling creatures are a rule of their own, `campaign-waves.md`) |
| Angband (worm masses that breed; Zephyr hounds in packs that breathe; carrion crawlers that paralyse; umber hulks that confuse; drainers of experience) | breeding kept to one creature by the existing summon cap (G5c); packs of breathers are #1099's | experience drain (rejected, §15); confusion (no status, and it is a design of its own) |
| Warlock's own mimics (#616) | a disguise in perception, a reveal by touch, a hold | the pattern: one status, one perception read, one reveal; dread needs no more machinery than the mimic has | — |

## 3. What exists, and what this builds on

| Part | Today (merged) | Used for |
|---|---|---|
| Creature fields | `on_hit` riders (statuses with chance and severity, diseases, drains with a check), `natural`, `resist`, `immune`, `speed`, `attack_cost`, `throw`, `summon`, `incorporeal`, `lurks`, `light`, `vitals`, `opens_doors`, `rarity`, `unique`, `stray`, the sneak-attack skills | every data-only gimmick (§5, "data on what exists"); the shape of the new fields (one table each, checked at load) |
| Statuses | `hold` (a struggle by Strength or Willpower; `turns`), `escape` (Webbed, Pinned), `flees`, `retaliates` (Fire-cloaked: a blow at the bearer costs the striker `1d6` fire), `hides` (the mimic), `sleeps`, `tick`/`damage`, `cost`, `combat`, `penalty`, `blocks`, `regen`, cooldowns (a status with no tick that eases), `treat` (the Heal skill), `start`/`end`/`dies`/`death` lines | engulf (G7: a status with `hold` and a tick, no code), the aura's statuses, stiffening (G2: a new status with `death`), a gaze's cooldown, fear's teeth (`combat` on Frightened) |
| Drains | `Affliction::Drain` on a hit: a stat, points, a check against a difficulty; mended at the temple; `Actor::drained` | the gaze may carry a drain as a hit does (the sea hag's death gaze) |
| Summoning | `[creature.summon]`: a pool, a count, a cooldown status, `most` alive; `summon_cells`; the AI summons before anything else | breeding (G5c: a summon of its own type), what a creature dies into (G5b reuses `summon_cells` and `make_creature`) |
| Perception | the snapshot: `sight`, `seen` actors, the mimic's `Disguise` read, Blood scent's pass (creatures within a radius however dark) | the shadowy (G10: one more read, the mirror of Blood scent's), the gaze's "you see it and it sees you" |
| Sounds | `SoundKind` (footsteps, doors, fighting, bowstring, chanting, banging, digging, voice, howl, heartbeat, bell, explosion, thunder), `Act::Howl`, `Act::Mutter` (an npc's mutter), the dump's `sound` layer | cries (§9: a creature's idle `Act::Cry(kind)`), the banshee's wail heard through walls |
| The danger draw | `Content::candidates`, `danger::weight(level, factor, danger)`, rarities as tiers with a factor, `most` on a summon, the spawner's cap by stance | the `dread` flag and the draw's one-in-x roll (§8) |
| The spells' area code | the cone and the burst (`cast.rs`, shared with traps and scrolls) | the burst (G3), as #1099's breath |
| Temper | the smith's loop: an item `damaged`, repaired at an anvil by Metalcraft | corroding gear (G8): acid makes a piece `damaged`; the anvil mends it |
| The balance probe | the player against the monsters of their level; `docs/balance.md` | the variance target (§7.3) and the probe's new row |
| Teleport | the teleport trap moves the victim to a free cell of the level | the blink to reach you (G6b, the blink dog) |
| Throws | `[creature.throw]`: a natural ranged attack; not beside its target the creature throws, beside it it strikes (the giants' boulders) | the earth elementals' stones (§5.3): data only |
| Materials | `data/materials/`, one file each, with group, weight, to-hit, damage, protection, value, rarity, gear; "rusting" listed as not yet modelled | `corrodes` on a material (G8), and two new metals, mithril and adamantium |
| Saves and the Characters screen | one save per character, deleted at death; the list of saved characters; saves pinned to the version | the graveyard file (§9.5): the one file that outlives every version, written where the save is deleted |

## 4. What dread is, and what it is not

A **bulk** creature is a fight of numbers: it comes, it hits, it dies; the player's choices are
whether to fight, where, and with what. Rats, kobolds, soldiers, skeletons, and the ranks of the
fiends are bulk, and should stay so: they are the bread of a floor, and a floor needs bread.

A **dread** creature changes *what the player must do*: look away, bring a light, put the sword
down and take up the mace, keep a cell between you, burn it, run. Its name promises it (nobody
walks up to a basilisk), its gimmick delivers it, and it is rare enough that the name in the
message line is the event. The test for the sort (§6) is: *does the trope the name comes from
ask for a different fight?* If yes, dread; if the trope is "a bigger one of those", bulk.

Between them is **bulk with a trait**: a rider that gives the floor texture (a warg's bleeding
bite, a ghoul's hold, a troll's regeneration) without changing the plan. Most tribes' traits
are already in the data; §6 names the few to add. A trait never gets the dread tier's rarity.

What dread is **not**: a boss (the uniques have their own place: the fire drake's hoard, the
outlaws' lairs, the bog hag), a caster (#1102 makes them real, and a lich is dread because it
is a lich, not because it casts), or a bigger number (that is §7).

## 5. The gimmicks: reusable mechanics, each with its cost and counterplay

Each gimmick is one creature field (checked at load, stated in the wiki's creature tables), one
hook in the rules, and words. The **cost** is what it takes from the player; the **counterplay** is
what already answers it (and, marked *new*, what this document adds). Who carries it is §6.

### 5.1 G1 The gaze: a rider at sight, not touch

`[creature.gaze]` with `status` (or `drain`, as an `on_hit` entry: `status`, `severity`, or
`drain`, `points`, `check`, `difficulty`), `range` (1 to 8), `chance` (percent, default 100),
and `cooldown` (seconds, a status as the summon's, default 12). On the creature's turn, if a
hostile within `range` is in its line of sight **and sees it** (for the player: its cell is in
the snapshot's `sight` and it is not disguised or shadowy; for a creature: `sight::sees`), the
gaze takes: the rider is applied as a hit's would be (a status by its chance, a drain by its
check), the cooldown status is given to the gazer, and the turn is spent (a gaze costs
`cast`'s price). It is a look, so it never lands on what has no eyes to meet (no vitals, and the
`blind`-less: everything today) and never through a wall.

- **Cost**: a transfixion, a fright, a drain, or stiffening, before the creature is in reach.
- **Counterplay**: *break line of sight* (fight it in a doorway it must come through; it is beside
  you before it can look, and then it may look every cooldown, so the fight is short or it is
  lost); *darkness*: a gazer in an unlit cell you cannot see cannot catch your eye (the `dark`
  zone spell, dousing your lantern against one with darkvision is no help, since **you** must see
  **it**); *kill it at range* (missiles, spells); *the check* (Willpower or Toughness, with Iron
  will, Divine grace, Still mind); *the Ward power* on the turncoat; Still mind's Stillness ends
  the status; and, for the gaze that kills, **Averted eyes, forced** (the owner, §17 D4: no
  toggle; "the basilisk forces the averting eyes condition automatically, unless there is
  another sight blocking mechanism deployed"). A gaze marked `averted = true` (the basilisk's
  only, §6) makes the player avert their eyes by reflex: while such a gazer is in the player's
  sight they are **Averted**, a condition derived each turn from the perception snapshot as
  Dazzled is (no action, no toggle, no actor flag, nothing saved; the status bar shows
  "Averted"), and they fight half-blind: **−4 to hit**, no free blows, Dodge halved
  (`world/avert.rs`: one read in the attack and free-blow paths). The gaze still looks for the
  glance the player cannot help: against averted eyes it takes at **a quarter of its chance**
  (`glance = 25` on the gaze, percent, data), then the Toughness check as usual; so a basilisk
  fought face to face is a fight at −4 that may still end in stone, and one walked away from is
  a fright. When *another* sight-blocking mechanism is deployed (the gazer's cell unlit so the
  player does not see it, a door or a wall between, the `dark` zone, a disguise), the player is
  not Averted and no gaze takes: the condition reads "do I see it", nothing more. The other
  gazes (the vampires', the hags', the spirits') have no averting: their stare is a check, met
  or failed, and the counterplay is the rest of this list. It is the ADOM tradition without the
  blindfold item or the choice.
- **Where it moves existing riders**: the vampires' transfixing stare (today on the bite, where it
  is a dead rider: a bite that transfixes lands after the bite), the sea hag's death gaze, the
  white lady's, the night mare's, the ultroloth's, the rakshasa's, the hand of glory's candle. The
  bite keeps its drain.
- **Words**: "The vampire's eyes find yours." then the status's `start`; "You feel a cold stare
  on you." when the gazer is unseen (so the player knows to look for it).

### 5.2 G2 Stiffening: the basilisk's gaze, to stone

A new status, **Stiffening** (`stiffening`: `max = 4`, `stacking = "add"`, `eases = 120`,
`seen = "stiffening"`, `cost = 150`, `penalty = { from = 2, dexterity = -4 }`, `treat =
{ difficulty = 10, per_degree = 3 }`), given by a gaze with a Toughness check against
12 + L ÷ 5 (15 for the basilisk at 16). At the **fourth degree** the bearer is stone: a new
`death` line ("Turned to stone"), `dies = "turns to stone"` for a creature, and the player's
death screen says so. A status that kills at its max is one new field, `kills_at_max = true`,
checked in `World::afflict`'s degree step (one hook; today only ticks kill).

- **Per-degree lines**, a small status feature used here first: `stages = ["Your limbs
  stiffen.", "Your skin greys and hardens.", "You can barely move."]`, said as each degree is
  reached (the `start` line is the first). The look line shows "stiffening".
- **Cost**: three warnings, then death. Two minutes a degree to ease, so a basilisk looked at
  twice and fled from is a fright, not a death; one fought face to face for eight turns is.
- **Counterplay**: all of G1's; the scroll of restoration ends it (one line in `read.rs`); the
  temple's mend; the Heal skill treats it a degree a try; Still mind's Stillness ends it; a
  basilisk is slow (speed 900), so walking away works; **it has eyes**: a basilisk in the dark
  behind a door is the floor's quietest killer, and §9's leavings (statues) are its sign.
- **The statue** (decided, §17 D9): a stoned creature leaves no corpse but a **statue**, a cell
  fixture laid as a chest is (`terrain = statue`, impassable, searchable), holding what it
  carried. It can be **dug** through with a digging tool (the wall rules, hardness between
  stone and granite) and it can be **kicked** (the Kick command at a statue: a door's kick,
  `kick_breaks(Strength, sturdiness)` against a sturdiness well above a door's, so it is hard to
  destroy and most kicks only bruise; every kick earns athletics marks as a door's does and a
  Strength mark, so a statue is a thing to train Strength on, the owner's intent; a statue broken
  falls to rubble and what it held). The player's statue is their death; the save ends as any
  death's, and the character goes into the graveyard file with `cause = "stone"` (§9.5), so that
  a later run's basilisk may have a statue of them. The generator lays statues near a basilisk's
  vault as leavings (§9), named from the graveyard where it can.

### 5.3 G3 The burst: vortices, and death throes

`[creature.burst]` with `damage`, `damage_type`, `radius` (0 to 3), and `when`: `"dies"` (it
bursts where it dies: the death throes) or `"touches"` (it bursts **instead of attacking** when
beside its target, and dies of it: the vortex). The burst resolves through the spells' shared
area code (a fireball with no flight), everyone in the radius struck but things of its own kind
(the burst's type immune), resistances applied, the dead creature's own cell the centre. A
`touches` burst is the creature's whole offence: it has no `attack`, and the AI's melee arm
bursts where it would strike. A vortex that is killed at range does **not** burst (its dying is
quiet): the counterplay is the whole point.

- **Cost**: area damage you cannot parry, at the worst moment (it dies beside you, or it reached
  you). An ember mote at level 2 bursting for `2d4` fire is a third of a level-2 warrior; a frost
  mote beside a wizard is a death.
- **Counterplay**: kill it before it arrives (anything thrown; a sling); resist its element;
  step back as it comes (they are quick, so one step buys one turn); doors (a mote does not open
  them); allies take the burst too, so a companion is a reason to shoot. Death throes: step back
  from a dying one (the message line says it "flickers" at a third of its hit points: the `hurt`
  words already grade wounds).
- **Who**: the six motes become vortices (§6: ember, frost, spark, dust, stone, water; `touches`;
  their `rarity` goes from common to **uncommon** and their speed up), the burning skull and the
  balor burst when they die (`dies`: the balor's `8d12` fire at radius 2 is the ADOM balor's
  death throes), the magma crawler too. And, the owner's (§17 D1): **every elemental line where
  its element bursts** gets death throes, so an elemental is a thing to kill from a cell away:

  | Line | Burst (`dies`) | Radius by rung (lesser / plain / greater / elder / eldest) |
  |---|---|---|
  | fire (lesser 9 to the elder 50, the balefire 64, the cinder elemental) | fire, the `burning` rider at the plain rung and up | 1 / 2 / 2 / 3 / 3 |
  | lightning (lesser 15 to the elder 52, the thunderhead 70) | electricity, `staggered` | 1 / 2 / 2 / 3 / 3 |
  | ice (lesser 11 to the elder 48, the rimethurse 66) | cold, `sluggish` | 1 / 2 / 2 / 3 / 3 |
  | air (lesser 10 to the elder 45, the welkin wind 58) | a gust: bludgeoning, `staggered`, and it blows the burst's cells' loose items a cell outward | 1 / 1 / 2 / 2 / 3 |
  | water (lesser 7 to the elder 42, the brimwolf 55) | a wave: bludgeoning, douses `burning` in the radius (the one kind burst) | 1 / 1 / 2 / 2 / 3 |
  | earth (lesser 6 to the elder 40, the hoarstone 60) | **none**: stone does not burst. The earth line instead **throws stones** (`[creature.throw]`, the giants' rule exactly: not beside its target it hurls, beside it it pounds; range 5 + L/10, the type's dice at a step less, `cost = 1500`, bludgeoning, the `staggered` rider), so the slow thing the player used to walk away from now hits them as they go. Data only: the throw and its switch to melee exist | — |

  The dice: a `dies` burst rolls **half the damage-spell dice of the type's level on #1101's
  doubled ladder** (a fire elemental at 27 bursts for half a level-27 fireball's dice: death
  throes are a parting shot, not a spell), the motes' `touches` burst at `2d4` to `2d6` as the
  whole of their offence. A burst through the area code **ruins what lies on the floor** as an
  area spell does since #1100 (the loot under a dying fire elemental may burn: a reason to lure
  it off the pile). The roster test for the elementals pins each line's burst and the earth
  line's throw.
- **Words**: "The ember mote bursts!" and the area's damage lines as a fireball's; "The fire
  elemental comes apart in flame!" for death throes; unseen, "an explosion" (the existing
  sound).

### 5.4 G4 Auras: fear, cold, stench, silence; fear's teeth

`[creature.aura]` with `status`, `severity`, `radius` (1 to 6), `check` and `difficulty`
(optional: without a check it always takes), `every` (seconds between touches, default 12), and
`heard = true` (it reaches by hearing, not sight: the banshee's wail, through walls, to anyone
who hears a voice at her loudness). On the creature's turn, each hostile within the radius in its
line of sight (or earshot) makes the check or takes the status, caused by the creature, once
every `every`. A `retaliates` status given innately (G4b: `innate = ["fire_cloaked"]`, a list of
statuses an actor of the type is made with, refreshed each turn, never shown as "cloaked" in the
look line when the type carries it always) is the aura's cousin for the thing that hurts to hit.

- **Fear's teeth, first**: Frightened does nothing to the player today (only a creature with a
  status that `flees` runs; the player's `flees` in `travel.rs` is the stance's flight price).
  Proposed, data only: `combat = { to_hit = -3 }` on `frightened`. Shaking hands, nothing more;
  Frightened already refreshes rather than stacks, and eases in six seconds a degree, so a howl
  is a bad turn, not a bad fight. (Rejected: ADOM's "too scared to attack", which with permadeath
  is a lock; a Dodge bonus for the fleeing, which rewards the status.)
- **Cost**: fear (−3 to hit, the ally runs), cold (`sluggish`: everything slower near a wraith),
  stench (`poisoned` a degree now and then: the hezrou), silence (`silenced` within 3 of the
  gibbering shade: a caster's dread; a priest's prayer too), drowsiness (the chasme's drone, the
  satyr's pipes: `drowsy`, cost 300 %: do not fall asleep beside it).
- **Counterplay**: the check (Willpower for fear and the drone, Toughness for stench and cold);
  the resistances (cold resists the cold aura's status, since the status's `damage_type` is
  read); range (the aura's radius is short: shoot it); the Ward power, Iron will, Divine grace,
  Unbowed, Still mind, Poison resistance; earplugs do not exist, so the banshee's wail is answered
  by Willpower and by not standing in her hall: her wail is **also the warning** (§9).
- **Words**: the status's `start`; the first touch of an aura adds the cause: "A chill comes off
  the wraith." / "The stench of the hezrou turns your stomach." / "A wail rises somewhere below."

### 5.5 G5 Splitting, dying into, and breeding

- **G5a Splitting**: `splits = { on = ["slashing", "piercing"], least = 8 }`. A blow of a listed
  type that leaves the creature alive with at least `2 × least` hit points halves them and makes
  a second creature of the type beside it with the other half (`make_creature`, no gear, the
  spawner's cap ignored, `Actor::split_from` for the kill credit: a split's kills pay a quarter,
  so a pudding farm is a poor one, and no quest counts the halves twice). A thing cut in two is
  two things.
  - **Cost**: the sword makes more of it; the room fills.
  - **Counterplay**: the listed types are the sword's; **blunt weapons, fire, and acid do not
    split it** (the mace, the staff, a torch, Burning hands), and the message says so on the
    first split ("The charnel heap comes apart, and both halves crawl."); below `least` it dies
    like anything.
- **G5b Dying into**: `dies_into = { creatures = ["grave_worm"], count = "2d4", spread = 1 }`:
  what crawls out of the corpse (the wormhost's worms, the hydra's way), placed by
  `summon_cells` on its death, its faction, paying a tenth each (they are what it was). A warning
  in the look line: "wormhost (hostile, wounded), *writhing*". Counterplay: fire (the worms are
  vermin of level 1; a burst spell clears them), or walk on: they are slow.
- **G5c Breeding**: no new field: a `[creature.summon]` of **its own type** (allowed when the
  pool is `creatures` naming itself; the tag pool keeps leaving the summoner out), `mana` made
  optional when the cost is 0 (a cooldown is a limit enough), `most` the cap. A floor of
  lichens is Angband's bore and NetHack's; one breeder (the wormlord's host of worms, the rat
  king if one is ever written) with `most = 6` is a fight that gets worse the longer it lasts.
  Counterplay: kill the breeder first (it is the one standing still); the cap.

### 5.6 G6 Theft on a cooldown, and the blink to reach you

The owner (§17 D1): theft is on a cooldown, and there is no fleeing or blinking away after it.
The first draft's `then = "blink" | "flee"` is gone; the thief stays and fights.

- **G6a Theft**: `steals = { what = "small" | "any", chance = 30, cooldown = 60 }` on a creature
  with hands. On a hit that lands while the thief carries no cooldown status, the chance; made:
  the creature takes one stack from the player's pack (never what is wielded or worn; `small`:
  coins, gems, scrolls, potions, a ring; the draw from the gameplay stream after the blow), says
  "The nymph snatches your scroll!", and is given the cooldown status (a status with no tick that
  eases, the summon's pattern), so it takes one thing a minute, not one a blow. It keeps
  fighting, as it was. What it took is in its pack and **falls where it dies**, as any creature's
  things do: there is no lock, no chase, only the fight you were already in, now with a reason to
  win it. It never steals from a creature.
  - **Cost**: your scroll of restoration in a nymph's hands until she is dead; the one who runs
    from a thief leaves what it took on the level.
  - **Counterplay**: kill it (it is a `caster` role, soft, and it is beside you by definition);
    the Steal ability works both ways (steal it back); allies set on it (Sic 'em); Stealth so it
    never notices you. The stolen stack is listed in the message history, so the player knows
    what to look for in the drop.
- **G6b The blink to reach you**: `blinks = { range = 6, cooldown = 30 }`. A creature with it
  moves to a free cell within `range` in its line of sight, **beside its target**, when its path
  is blocked (a door it cannot open, a chokepoint held, a corridor full of its fellows): the
  teleport trap's placement, the gameplay stream's draw, a cooldown status. It is the blink
  dog's, and the blink dog's only: fighting it in a doorway does not work, and that is its whole
  gimmick. No creature blinks **away** (the first draft's `hurt` and `stolen` triggers are gone
  with the owner's word; the quasit, the will-o'-wisp, and the archlich, which had them, are
  given other things in §6).
  - **Cost**: a corridor is no fortress.
  - **Counterplay**: it blinks to a cell in **its** sight, so a closed door is still a door; the
    cooldown (30 s); missiles; it is quick and soft, as a dog is.
  - **Words**: "The blink dog is suddenly beside you." Unseen, nothing.

### 5.7 G7 Engulfing: a status with a hold and a tick

No code: a new status, **Engulfed** (`engulfed`: `hold = { check = "strength", difficulty = 14,
turns = "2d4" }`, `tick = 6`, `damage = 2`, `damage_type` the engulfer's, `blocks = ["step",
"fire", "throw", "read"]`, `seen = "engulfed"`, `start = "It closes over you."`), given by an
`on_hit` rider as a hold is. The hold's rules do the rest (adjacent holder, the struggle, ended
when the holder dies or moves). A sword still works from inside (`attack` is not blocked: hack
your way out), a spell too.

- **Cost**: every turn inside costs hit points and the struggle costs the turn; nothing ranged.
- **Counterplay**: the struggle (Strength or Willpower, `World::slipping`: the better of the
  two); kill it from inside (it is beside you by definition; its Defence is the brute's, low);
  an ally's blows; Chest-wise's `slick` reads the mimic's tag only and stays so; the small (a
  halfling) is **not** easier to engulf: the hold's check is the same for every size, on purpose
  (a size rule is a burden rule, `docs/dev/burden.md`, and the owner did not ask for one).
- **Who**: the charnel heap, the greater water elemental and above (drowning: `cold`), the
  nightworm (swallowed), the giant leech (`latched`: a weaker cousin, `damage = 1`, `tick = 12`,
  with its bleeding), the rug of smothering (the owner's, §13).

### 5.8 G8 Corroding gear: acid hides and acid blows

`corrodes = { chance = 25, what = "armour" | "weapon" }`. `armour`: a hit that lands on the
player rolls the chance; made, one worn piece of metal or leather (the slot drawn from those
worn, the gameplay stream) becomes `damaged` (`Worked::damaged`, the smith's state: its
Protection or Defence a point lower until mended) — "The acid eats at your mail." `weapon`: a
blow that strikes the creature with a metal weapon rolls it on the striker's weapon ("Your
sword comes away pitted."). A piece already damaged is not damaged twice: there is no
destruction, only the smith's bill (Temper at an anvil mends it, or the forge's smith for coin,
`docs/dev/skills.md` "Temper").

- **Cost**: a point of armour, a trip to the anvil; a hoarder's dread, not a killer.
- **Counterplay**: fight it with wood (a staff, a club, a bow from range) or bare hands or spells;
  wear or wield what does not corrode (below); acid resistance halves the chance (one read);
  Chest-wise does not apply.
- **What corrodes, and what does not** (the owner, §17 D1: "add corrosion resistant materials:
  mithril, adamantium. gold, silver etc. also should resist corrosion (review each material)"):
  a material field **`corrodes`** (default true for the metal and hide groups, false for the
  rest; stated per file where it differs; the wiki's materials table gains the column). The
  roll picks only among the worn pieces (or the wielded weapon's working part) whose material
  corrodes; a character in nothing that corrodes is never touched, and the message says why
  once ("The acid runs off your mithril mail."). The review of the 52 materials:

  | Corrodes | Does not |
  |---|---|
  | **Metals**: iron, steel, copper, bronze, brass, tin, zinc, lead, pewter, cadmium, aluminium, duralumin, bendalloy | **Metals**: gold, silver, electrum (the noble metals), chromium (what makes steel stainless), nicrosil (a nickel alloy), and the two new ones, **mithril** and **adamantium** |
  | **Hides and cloth**: leather, hardened leather, fur, cloth, wool, silk (acid eats them) | dragonscale (the hide that resists everything) |
  | **Bone**: bone, horn, ivory, chitin (acid etches them: slowly, so at half the chance, `corrodes = "half"`) | dragonbone |
  | paper, wax (never worn: moot) | **Wood**: wood, hardwood, ironwood (the owner's counterplay: fight it with wood); **stone**: stone, flint, obsidian, jade; glass; amber and every gem; flesh and plant |

  **Mithril** and **adamantium** are two new material files (`data/materials/mithril.toml`,
  `adamantium.toml`; `group = "metal"`, `corrodes = false`): mithril light and quick (weight 50,
  to-hit +1, damage 0, protection 100, value 2000, loot `rarity` very rare from item level 20,
  `gear` from creature level 30), adamantium heavy and hard (weight 120, to-hit 0, damage +1,
  protection 130, value 3000, very rare from item level 30, gear from 40). Neither is `precious`
  (a mithril long sword is named as a bronze one is, by its material). Their exact numbers are
  the slice's to tune against the material tables; the wiki states them. They are also the
  smith's best work: Temper's improvements apply to them as to steel.
- **Who**: the ember beetle's acid blood (`weapon`), the hezrou (`armour`), the dretch, the
  black pudding if one is written (§15 keeps new types to a short list), the corrosive mimic?
  (the owner's, §13: the animated are made of materials already).

### 5.9 G9 Regeneration, and what stops it

`regen = 400` on the creature type (percent of the normal healing pace, as a status's `regen`
is; default 100), read where passive healing reads the pace (`status.rs`'s one place, with the
statuses' slowest-wins rule), so a troll heals four times as fast as its Toughness alone would:
a troll left for ten turns is whole. Then, **data on two statuses**: `burning` and `corroded` get
`regen = 0`, so fire and acid stop every regeneration, the player's own included, while they
last (true to the trope and to the house taste: a burning man does not heal).

- **Cost**: a troll fight that is not finished is a troll fight that starts over.
- **Counterplay**: fire (a torch, Burning hands, a Flaming blade, oil: everything the tribe file
  already lists as follow-ups), acid, burst damage to kill it in one go, and the message: "The
  troll's wounds close." when the player sees it heal a tenth in a turn.
- **Only on what is dangerous by itself** (the owner, §17 D1: "regen is kinda meh, if the
  creature is not dangerous by itself. regenerating sack of potatoes is just free weapon
  marks"): the rule, pinned by the roster test, is that **a regenerating type is a brute of its
  level** (its dice a step above the standard's at its level and its to-hit the brute's) **or
  carries a dread gimmick**; never a shambler, a small thing, or a standard. A thing that heals
  and cannot hurt you is a training post, and #1095's slower marks only make it a slower one.
  So: the trolls proper keep it (they are brutes, `3d8` at 15 to `4d8` at 37); the **marsh
  troll** (level 9, `1d10`, 24 hit points: the sack) **gets the troll's dice** (`2d8`, a brute's
  hit points: it is a troll, and Fennmynster's first) rather than losing the trait; the rotting
  troll at half, as a shambler, is the one troll that **does not** regenerate (its rot is its
  trait). No lock on marks, no anti-farming rule: the danger is the rule.
- **Who**: the troll tribe but the rotting troll; the hydra-kin if any. The `troll` roster test
  gains the row and the rule.

### 5.10 G10 The shadowy: unseen in the dark

`shadowy = true`: the mirror of Blood scent's perception pass. A shadowy creature standing on a
cell whose perceived light is below the `dim` threshold (the vision profile's, as the player's
own reading of brightness) is **not in the snapshot** unless adjacent to the player, who then
sees "a shadow" beside them, or revealed by Blood scent, Quick-sensed (a heartbeat: not the
undead, so a shadow has none; the darkling has), or the dev reveal. Its blows land from the dark
as "something"; its footsteps are silent already (the spirits' rule). Darkvision does not help:
a shadow is the absence of a thing, not a thing in the dark (the trope; and darkvision helping
would make it a human-only dread). Creature sight is unchanged: it sees you.

- **Cost**: Strength drained by something you cannot see, in a dark corridor.
- **Counterplay**: **light** (a lantern lit, a torch, the Light spell, a room's braziers: a lit
  cell shows it as anything; the `lit` status on the player lights their cell and the next),
  Blood scent, Quick-sensed, fighting in doorways, a Flaming blade's glow; and the sign: "You
  feel a cold draught." when one is within 2 (the warning, §9).
- **Who**: the shadow (8), the greater shadow (31), the deathshade (60), the darkling (9, a fey
  of the Unseelie dark), the sluagh (43: a host out of the night).

### 5.11 G11 Invisibility: unseen in any light

The owner (§17 D1): "why not also implement some invisible creatures while at it? like
invisible stalker etc. Give duergars ability to use invisibility on CD (and fix the AI so they
just cast it on cd and then continue their regular activities)." The first draft deferred it;
it is the shadowy's twin and shares its perception pass, so it comes in the same slice (D8).

Two ways to be invisible, one status. **`invisible = true`** on a type is innate: the creature
is never seen (the invisible stalker). The **Invisible** status (`invisible`: `max = 1`,
`eases = 120`, no tick, `end = "You can see it again."` when the player sees the cell, `seen`
none: it is the one status nobody sees) is what a caster gives itself: **`[creature.invisible]`
with `cooldown = 300`** (seconds; a cooldown status as the summon's) on the duergar, the imp,
the quasit, and the will-o'-wisp, cast as a self-buff. The perception pass: an actor that is
invisible is **not in the snapshot** whatever the light and however near, unless Blood scent,
Quick-sensed (it has a heartbeat: the stalker has, as a thing of air, none; the duergar has), the
**companion's nose** (`CreatureDef::nose`, docs/dev/companion.md: a hound beside you growls at
the cell, and the look line says "your hound bristles at something"), the new **Seeing** status
(`seeing`, granted by the spell *See invisible* in #1102's Divination list and a potion, five
minutes; it also shows the shadowy), or the dev reveal. **Its blows are "something hits you"**,
and the cell it struck from is **marked for a turn** as a sound is (the `sound` layer's `x`
fighting at the cell): the player may strike back at the mark. **Hearing is untouched**:
invisibility is sight only, so an invisible duergar's hobnails are heard as any duergar's, and
Listen, the `sound` layer, and keen ears are the counterplay the trope is about. **Striking at
the unseen**: a move into a cell that holds an unseen creature becomes an attack at **−4** (the
Averted penalty reused: `world/avert.rs`'s one number), "You strike at something.", and a hit
reveals it for that turn ("Your sword bites something unseen."); a creature that strikes at an
unseen player (the player under the same status, from a scroll or ring later) does the same.

- **The AI** (the owner's fix, with #1102's caster step): a self-buff is a **free-standing step
  before the fight step**: when the cooldown is off and the status is not held, the creature
  casts it on its turn whether or not it has a target, at the spell's price, and then goes on
  with what it was doing (patrolling, approaching, fighting). It never stands still to recast,
  never recasts while the status holds, and never prefers the buff to a blow it could land this
  turn when the status holds. The same step serves #1102's other self-buffs (haste, barkskin):
  one rule, data on which spell. A duergar met in a corridor is a duergar who **was** invisible
  and will be again in five minutes; the fight in between is a plain one.
- **Cost**: fighting what you cannot see: its first blow lands unopposed (the surprise rules:
  a creature the player has not noticed catches them flat-footed, docs/dev/surprise.md), every
  swing at it is at −4 and at a cell, and in a room it is anywhere.
- **Counterplay**: **hearing** (Listen at the door; the sound marker of its steps every turn it
  moves: an invisible thing that walks is a thing you can follow), a **corridor** (one cell it
  must come through: strike at the cell), the **companion's nose**, Blood scent (once it bleeds:
  the stalker does not), Quick-sensed, Seeing, allies (they fight by the same rules), the status's
  two minutes against the cooldown's five (three minutes of every five it is a plain duergar),
  and the dust: an invisible creature walking through blood leaves prints the blood layer shows
  (`docs/dev/blood.md`'s trail, unchanged: it is the trail of a thing that is there).
- **Who**: the **invisible stalker** (new, the one new type of this design: an air thing of
  level 22, uncommon, dread, `invisible` innate, quick (1500), a slashing wind of `3d8`, no
  corpse, no blood, no heartbeat; it hunts by scent, so it comes to the player wherever they
  are on the level: `lurks = false`, and it is the one creature that never idles; NetHack's and
  D&D's, placed in the air elementals' populations and as a stray); **every duergar fighter**
  (the warrior, veteran, thane, miner, crossbowman, slinger, spider rider, hulk, smith, the
  king: the tribe's follow-up, docs/dev/creatures/duergar.md, delivered; the priest and
  mindbreaker cast it among #1102's lists); the **imp** and the **quasit** (the devils' and
  demons' follow-ups: they sting and are gone from sight, not from the room); the
  **will-o'-wisp** (it *winks out*: the status also puts its light out, `light` suppressed while
  it holds, so the fen's lure vanishes and returns; its mana-drain touch kept).
- **Words**: "The duergar fades from sight." (seen cast) / "Something hits you!" / "You hear
  hobnails on stone." (its steps, the hearing rules' line) / "Your hound bristles."

### 5.12 Data on what exists (no code)

| Gimmick | The field | Already carried by | Added to (§6) |
|---|---|---|---|
| **Stat drain** (for good until mended) | `on_hit` `drain` | 48 undead, the succubus, the sea hag | the gaze carries it (G1) for the vampires and the sea hag |
| **Hold** | `on_hit` `held` (a struggle) | ghasts, ghouls, the gallows dead, the lich, the annis hag, the korred, the meenlock, the chain devil, the glabrezu, the balor's whip, the canoloth, the elder elementals | engulf (G7) replaces it on the water elementals and the nightworm |
| **Web** | `webbed` (an escape status) | the web trap | the barrow spider and the large spider spit it: a `throw` of 0 damage with the rider (range 3, cost 1000), and the korred's hair |
| **Transfixion** | `transfixed` | hags, fey casters, the vampires' bites, the chasme | the gaze (G1) where it is a stare, kept on the touch where it is a touch (the pixie, the dryad) |
| **Fear** | `frightened` (`flees`; teeth in G4) | 20 types | the auras (G4) where it is a presence, the howl (`Act::Howl` as a cry, §9) where it is a sound |
| **Disease** | `on_hit` `disease` | rats, the ghouls, mummies, the drowned | — |
| **Sleep** | `drowsy` (cost 300 %) | the drow's bolts, the Lullaby | the pixie's dust (replacing its transfix), the chasme's drone and the satyr's pipes as auras |
| **Mana drain** | `ebbing` | the Siphoning affix | the will-o'-wisp's touch, the arcanaloth's, the gibbering shade's: a caster's dread that costs no hit points |
| **Retaliation** | `retaliates` on a status | the player's Fire cloak | innate on the fire salamander, the balor, the barbed devil (`1d6` piercing: its barbs), the ember beetle's acid blood (acid) |
| **Hunger** | — (no rider exists) | — | a `satiation = -500` rider, one line beside the status riders in `after_blow`: the bheur hag's cold hunger, the nightwing | 

### 5.13 Not gimmicks here

Breath and several natural attacks are **#1099**; creature spells, darkness, and curses cast are
**#1102** (invisibility cast on a cooldown is G11's, through #1102's self-buff step); flight,
swimming, and a large size are the rosters' standing follow-ups and stay so.

## 6. The sort: every creature type, bulk or dread

The rules of the sort: **people** (the showcase's and the Ascmearc's folk, the merchants, the
strange spirit), the **beasts** (wildlife, the companions), and the **player** are out. Every
other written type is listed under its tribe; a type not named in a tribe's row is **bulk, as it
is**. *Trait* marks bulk with a rider to add; **dread** gives the gimmick and the tier it moves
to (§8; `—` keeps its tier). The counts: 51 dread, some thirty traits added or moved, the
rest bulk.

**Undead** (94 types; the roster's drains, holds, and diseases are traits already in place)

| Type (L) | Sort | Gimmick | Tier |
|---|---|---|---|
| crawling hand 1, skeletal rat 1, kobold zombie 2, skeleton 3, zombie 3, skeleton archer 4, drowned 5, skeletal wolf 5, skeleton warrior 7, bog body 9, ogre zombie 9, grave hound 10, draugr 12, charred dead 14, duergar skeleton 15, revenant 19, grave knight 22, headless knight 29, draugr lord 34, bone colossus 40, death knight 45, wild huntsman 57, dread lord 59, charnel titan 67 | bulk | — | — |
| barrow ghoul 4, ghast 7, fen ghoul 11, gallows dead 17, ghoul lord 32, ghoul king 44 | trait | the hold they have (the ghoul's touch), kept | — |
| corpse candle 2, stone haunt 3, poltergeist 6, ghost 8, knocker 13, fetch 19 | trait | incorporeal, kept; the knocker and poltergeist get a **cry** (knocking: `banging`; §9); the fetch **looks like you** (`looks_like = "player"`: your glyph and colour in the snapshot until it strikes; one perception read; it is your death's double) | fetch → rare |
| pale drinker 7 | trait | its Strength drain, kept (the first drainer, Fennmynster's) | — |
| **shadow 8**, **greater shadow 31**, **deathshade 60** | **dread** | **G10 shadowy** + the Strength drain they have; the deathshade's fear as an aura (G4) | rare, dread |
| **wraith 11**, **dread wraith 28**, **wraith lord 41** | **dread** | **G4 cold aura** (`sluggish`, radius 2) + the Toughness drain they have; incorporeal | rare, dread (the wraith is Fennmynster's and stays as placed) |
| spectre 14, gibbering shade 18, white lady 39, washer at the ford 48, night mare 49, ghost lord 53 | **dread** | the spectre: its Willpower drain as a **gaze** (G1); the gibbering shade: a **silence aura** (G4, radius 3) with its wits drain; the white lady and night mare: their transfix as a **gaze**; the washer: her sight is an omen: **fear aura** + Presence drain; the ghost lord: #1102 and the gaze | rare, dread |
| hand of glory 16 | trait | its candle transfixes as a **gaze** (G1, range 3, by its own light) | — |
| mummy 10, mummy priest 20, mummy lord 36, mummy king 56 | trait | mummy rot, kept; the priest, lord, and king cast (#1102) | — |
| **banshee 23** | **dread** | **G4 heard aura**: her wail frightens everyone in earshot through the walls, and is the warning; her Presence drain on touch | rare, dread |
| bodach 21, barghest 13, black shuck 33, church grim 38, hound of the hunt 55 | trait | fear, kept; each a **howl** cry (§9: the hound heard before it is seen) | — |
| **vampire spawn 16**, **vampire 25**, **nosferatu 35**, **vampire lord 45**, **elder vampire 58**, **eldest vampire 69** | **dread** | **G1 gaze** (transfixed, moved off the bite) then the bite's Toughness drain; quick; open doors; fire hurts them (kept) | the spawn uncommon → rare; the rest rare and very rare as they are, dread |
| **lich 30**, barrow king 42, **archlich 54**, deathless king 66, demilich 70 | **dread** | #1102's spell lists (the lich's is the deepest); the lich's touch holds and drains wits (kept); the archlich and demilich a **fear aura** (G4, radius 3) in place of the first draft's blink away (gone: §17 D1) | lich, archlich dread; every tier kept |
| **wormhost 26**, **wormlord 47** | **dread** | **G5b dies into** grave worms (`2d4`; the lord `3d6`), burrowing worms kept; the lord **breeds** them (G5c, `most = 6`) | uncommon / rare, dread |
| **charnel heap 22** | **dread** | **G7 engulf** + **G5a splits** under slashing and piercing; filth fever kept; slow | common → uncommon, dread |
| burning skull 12, screaming skull 37 | trait | the burning skull **bursts** when it dies (G3, `2d6` fire, radius 1); the screaming skull's fear as a **scream** cry heard (G4 heard, radius 4) | — |
| bone wyvern 18, bone drake 27, rotting drake 30, bone dragon 50, rotting dragon 52, nightwing 61, dread wyrm 62, lich-drake 64, ghost dragon 68 | trait → #1099 | the dragon AI: a breath (cold for the bone, necrotic for the rotting, the lich-drake's cold with its drain) | — |
| sluagh 43, ankou 65, nightganger 46 | trait | the sluagh **shadowy** (G10) with its sneak attack; the rest kept | — |
| **nightworm 63** | **dread** | **G7 engulf** (swallowed: `damage = 4`) in place of its hold | very rare, dread |

**Demons** (10)

| Type (L) | Sort | Gimmick | Tier |
|---|---|---|---|
| dretch 8, balgura 24, marilith 40 | bulk | the dretch's filth fever and the marilith's six blades (`attack_cost`), kept | — |
| quasit 12 | trait | **G11 invisibility on a cooldown**: it stings and is gone from sight, not from the room; poison kept | — |
| **chasme 28** | **dread** | **G4 aura** `drowsy` (its drone, radius 3, `heard = true`: it is heard first, a **buzzing** cry) | uncommon, dread |
| **hezrou 30** | **dread** | **G4 stench aura** (`poisoned`, radius 2, Toughness) + **G8 corrodes armour** (its filth) | uncommon, dread |
| vrock 34, nalfeshnee 50 | trait | their fear as **auras** (G4; the vrock's a **screech** cry) | — |
| glabrezu 44 | trait | the hold, kept; #1102's spells | — |
| **balor 62** | **dread** | innate **Fire-cloaked** (`retaliates`), **G3 bursts** when it dies (`8d12` fire, radius 2), the whip's hold | very rare, dread |

**Devils** (10)

| Type (L) | Sort | Gimmick | Tier |
|---|---|---|---|
| lemure 5, bearded devil 20, erinyes 38, horned devil 48 | bulk | — | — |
| imp 9 | trait | **G6a steals** `small` on its cooldown and **G11 invisibility on a cooldown**: an imp takes a coin purse and fades, and is still in the room; its poison kept | — |
| **barbed devil 26** | **dread** | innate **retaliates** (`1d6` piercing: its barbs), its bleeding kept | common, dread |
| bone devil 32 | trait | **fear aura** (G4, radius 2) | — |
| chain devil 42 | trait | its hold becomes **pinned** (the chains: the escape status, 3 degrees) | — |
| ice devil 55 | trait | **cold aura** (G4, `sluggish`, radius 2) | — |
| **pit fiend 66** | **dread** | innate **Fire-cloaked**, **fear aura** (radius 3), #1102's spells | very rare, dread |

**Yugoloths** (5), **fiends of no family** (7), **hags** (5)

| Type (L) | Sort | Gimmick | Tier |
|---|---|---|---|
| mezzoloth 22, nycaloth 36, canoloth 29 | bulk / trait | the canoloth's tongue: **engulf-lite** (`latched`: held with a `1` tick) in place of its hold | — |
| arcanaloth 46 | trait | **mana drain** (`ebbing` rider) and #1102 | — |
| **ultroloth 58** | **dread** | **G1 gaze** (transfixed and frightened, moved off its blows) | very rare, dread |
| hell hound 13, nightmare 21, cambion 39 | trait → #1099 / #1102 | the hound's and the nightmare's fire as a short **breath** (the hound's `3d6` cone of 3) | — |
| **succubus 27 / incubus 27** | **dread** | **G1 gaze** (transfixed) and the kiss's Toughness drain; **G6a steals** `small` on its cooldown (a seduction is a theft; it stays, as a seducer does) | rare, dread |
| night hag 33 | trait | her hold on the sleeper: the rider `drowsy` then `held` (kept) | — |
| **rakshasa 52** | **dread** | **G1 gaze** + #1102; its `physical 50` kept (blessed steel is a follow-up) | very rare, dread |
| green hag 18, annis hag 34 | trait | kept (transfix and hex; the embrace) | — |
| **sea hag 26** | **dread** | **G1 death gaze** (frightened and the Toughness drain, range 4) | rare, dread |
| bheur hag 42 | trait | **cold aura** + the **hunger rider** (§5.12) | — |
| bog hag 11 (the Ascmearc's, unique) | as it is | #1102 | — |

**Fey** (13)

| Type (L) | Sort | Gimmick | Tier |
|---|---|---|---|
| sprite 2, boggle 7, satyr 10, redcap 12, korred 25, yeth hound 31 | bulk / trait | the satyr's pipes a **`drowsy` aura** (heard); the korred's hair **webs** (the rider); the yeth hound's fear and a **howl** cry; the boggle kept | — |
| **pixie 4** | **dread** (the shallowest) | **G6a steals** `small` on its cooldown; its touch gives `drowsy` (its dust) instead of transfixing | uncommon, dread |
| quickling 6 | trait | kept: speed 2000 and its sneak attack are the claw bug of this game (§7) | — |
| **darkling 9** | **dread** | **G10 shadowy** with its sneak attack | uncommon, dread |
| **blink dog 14** | **dread** | **G6b blinks** to reach you (the one blinker) | uncommon, dread |
| meenlock 16 | trait | kept: lurks, frightens, holds | — |
| dryad 19 | trait | kept; #1102 | — |
| **nymph 22** | **dread** | **G6a steals** `any` on her cooldown; her touch transfixes (kept) | rare, dread |

**Elementals** (37)

| Type (L) | Sort | Gimmick | Tier |
|---|---|---|---|
| **ember mote 2, frost mote 5, spark mote 8, dust devil 3, water mote 4, stone mote 1** | **dread** (vortices) | **G3 bursts on touch** (`2d4` of its element at radius 1; the stone mote `bludgeoning`, the dust devil `slashing`; no attack; speed +200) | common → **uncommon**, **not flagged** (a vortex is a mote: it comes on any floor, several at a time, as ADOM's do; the burst is dread enough at level 2) |
| the lesser rungs and the plain names of fire, lightning, ice, air, water (7 to 27) | trait | **G3 death throes** by line (§5.3's table: radius 1 or 2); their riders kept (`staggered`, `sluggish`); the fire line's `burning` rider added | — |
| greater water 26, elder water 42, brimwolf 55 | trait | **G7 engulf** (drowning, `cold`) in place of their hold; the wave's death throes (§5.3) | — |
| lesser earth 6, earth 13, greater earth 24, elder earth 40, hoarstone 60 | trait | **throw stones** (`[creature.throw]`, the giants' rule: hurled at range, pounded beside; §5.3) in place of a burst; incorporeal in stone and the holds kept | — |
| the greater, elder, and eldest of fire, air, ice, lightning (31 to 70) | trait | **G3 death throes** (radius 2 or 3); the air line's `staggered` kept; the eldest rungs are #1099 candidates for a breath (welkin wind, thunderhead) | — |
| **invisible stalker 22** (new, §5.11) | **dread** | **G11 invisible** innate; quick; a hunter by scent that never idles; no corpse, no blood, no heartbeat | uncommon, dread (the air elementals' populations and a stray) |

**Vermin, marsh things, fire things** (17)

| Type (L) | Sort | Gimmick | Tier |
|---|---|---|---|
| grave worm 1, rat 1, giant centipede 3, giant frog 4, crocodile 6, mud golem 8, marsh troll 9, cultist, cult priest | bulk / trait | the crocodile's **hold** added (its jaws; the death roll is bleeding on the hold's turns); the marsh troll regenerates (G9) **with a troll's dice** (`2d8`, a brute's hit points: §5.9's rule, never a regenerating sack) | — |
| barrow spider 2 | trait | **web spit** (the `throw` with `webbed`, range 3) with its lurk and sneak attack | — |
| giant leech 4 | trait | **latched** (engulf-lite, G7) with its bleeding | — |
| **will-o'-wisp 7** | **dread** | **G11 invisibility on a cooldown** (it winks out, its light with it) and a **mana drain** touch (`ebbing`); its light is the lure and the warning, and its going out the second | rare, dread |
| **basilisk 16** | **dread** | **G2 stiffening** by **G1 gaze** (range 6, Toughness against 15); its bite's `sluggish` kept; slow | rare → **very rare**, dread (the owner: a basilisk is rarer by far than a ghoul) |
| ember beetle 8 | trait | **G8 corrodes weapons** (acid blood, `retaliates` acid `1d3`) | — |
| fire salamander 12 | trait | innate **Fire-cloaked** | — |
| magma crawler 14 | trait | `burning` rider; **bursts on death** (G3, `2d8` fire, radius 1) | — |
| cinder elemental 17, fire drake 19 | as it is / #1099 | — | — |

**Giants, trolls, orcs, goblins and hobgoblins, kobolds, duergar, drow, outlaws** (the peoples: 78)

| Type (L) | Sort | Gimmick | Tier |
|---|---|---|---|
| every giant and ogre | bulk | the throw, kept; the frost giant's `sluggish` and the fire giant's `burning` riders on the boulders added; the shaman #1102 | — |
| every troll but the rotting troll | trait | **G9 regeneration** (400 %; fire and acid stop it; they are brutes, §5.9's rule) | — |
| every orc, goblin, hobgoblin, kobold, outlaw | bulk | the casters #1102; the goblin wizard's summons kept | — |
| every duergar | trait | **G11 invisibility on a cooldown** (the owner's: the fighters cast it when it is off and carry on; the priest and mindbreaker among #1102's lists); the mindbreaker's mind-lash as a **gaze** (frightened and transfixed, range 4) | — |
| the wargs | trait | kept (bleeding) and a **howl** cry (a pack heard) | — |
| drow stalker 19, **drow assassin 32** | trait / **dread** | the assassin's sneak attack is its gimmick already (`6d6`, initiative 100): dread by variance (§7), the flag only | assassin: rare, dread |
| the drow priestesses | as they are | summoning kept; #1102 | — |

**Mimics** (60): dread already, the model; nothing changes (a mimic is placed by the chest rule, not drawn, so the dread roll of §8 does not touch it). **The animated objects** (107): §13.

The sort's tier moves are collected in §8.3.

## 7. Variance: how far a level's monsters should spread

### 7.1 Today

`docs/balance.md`'s Monsters table (within two levels, median and range): at level 15, hit points
57, to-hit 4 (1 to 7), damage a blow 13 (4 to 20), Defence 12 (8 to 19), Protection 0 (0 to 7),
speed by role. The rosters hold every type within **±35 %** of the role formulas for hit points
and dice average (shambler, brute, standard, quick, spirit, caster, small; `base_hp = 2.5 × L ×
role` with role factors 0.6 to 1.6). To-hit was `L/10`; since #1097 a **natural attack hits at
`L/4`** (in each type's data) and an armed monster hits with its weapon skill and gear as
before, and since #1096 a natural 22 to 24 always hits, whoever rolls it. The spread is real but
**narrow where it is felt**: two level-15 monsters with 50 and 65 hit points and +3 and +5 to
hit are the same fight to the player; a floor of them is a row.

### 7.2 The proposal: spread by role, on #1097's baseline

| Number | Today | Proposed | The spread at L 20 (standard: HP 73, to-hit 5, damage 16) |
|---|---|---|---|
| **To-hit** | `L/4` for natural attacks since #1097, every role alike; armed monsters by their gear | `L/4` the standard; **quick and assassin +L/8**; **brute and shambler −L/8**; casters −L/8; the roster tests pin the role's offset; armed monsters unchanged (their spread is the gear's and the skill's) | 2 (brute) / 5 (standard) / 7 (quick), before gear |
| **Hit points** | role factors 0.6 to 1.6, ±35 % | **0.5 (quick, small, caster) to 2.0 (brute)**, ±35 % kept; the dread brutes at the top | 37 / 73 / 146 |
| **Damage** | the dice ladder's nearest to `2 + 0.55 L`, one step by role | the mean as today; the **shape** by role: **strikers** (quick, assassin) **threaten from 19** (`threat = 19`, crit ×2; a natural 22 to 24 already always hits since #1096, so the striker's threat range adds the crit, not the hit); **brutes** `crit = 3` on the natural 24 only; standards as today. The roster tests check threat and crit by role | a quick's blow of 16 crits one turn in four; a brute's of 22 lands a 66 once in a long fight |
| **Speed** | `1000` for most, quick `1200 + 5L`, brutes 800 to 950, a few at 600 | **bimodal**: no tribe where every type is at 1000; bulk between **800 and 1250** picked by temperament (the rosters' README names each), dread at **600 or 700** (the stone, the heap, the stiffening thing: slow and awful) or **1400 and up** (the quick dread). A people's rank-and-file at 950 to 1050 | — |
| **Defence** | by level and role | as today (the player's to-hit is already near sure against bulk: 92 to 99 % from level 10 in the probe) | — |

Why these: the to-hit offset turns #1097's one line into three (a quick thing that hits and a
brute that misses but kills), the hit-point range makes "it will not die" and "it dies in one"
both happen on one floor, threat and crit make the damage *spiky* without moving its mean (the
probe's rounds-to-die stay where #1097 puts them; the variance of the fight rises), and the
speed rule is the cheapest texture there is: a floor of creatures that arrive at different
turns is not a row.

### 7.3 The target, measured

Add a row to the balance probe (`balance_probe`): per level band, **the rounds the probe's
warrior takes to kill each monster in the band, as a distribution**: the median, the fastest
tenth, the slowest tenth. The target: **the fastest tenth at half the median, the slowest at
twice**. Today (reading the Monsters table's ranges) the fastest tenth is about 0.7× and the
slowest 1.4×. The same for rounds-to-die, where the thief, wizard, and priest columns tell
whether a dread brute is a wall or a death (it should be the second, rarely: §8). The
pass (slice D10) tunes the role offsets until the row holds and reports the probe's tables in
`docs/balance.md` as #1097 will.

### 7.4 Grades: two kobolds, two fights (decided: ship, §17 D7)

`Actor::grade: i8` (−1, 0, +1; saved, hashed, left out at 0), drawn as a creature is made
(`make_creature`, after the court, from `grade/<actor id>`): **one in six is greater** (+25 %
max hit points, +1 to hit, +2 damage, speed −10 %), **one in six lesser** (the reverse), by the
type's `grades = false` to switch off (uniques, people, the generated families, mimics, the
companions: the roster tests name them). Read in five places (`max_hp`, the melee profile's
to-hit and damage, the energy price, the wiki's table says the odds). The look line says it:
"a big kobold", "a scrawny kobold" (the type's `grade_words = ["scrawny", "big"]`, defaults by
tag: the undead "withered"/"ancient", the fiends "lesser"/"greater"? — no: the fiends have
lessers already; defaults "small"/"big", the tribes may name theirs; the blades "notched"/"keen",
§13). The name in messages stays the type's ("The kobold hits you."); only look mode and the
`actors` dump say the grade. No experience change (a big kobold pays a kobold's). It is the one
proposal here that makes two of the same thing two fights, and it costs a byte and five reads.
The owner: ship it (§17 D7), with "small"/"big" wherever a tribe names nothing.

## 8. Rarity: how often dread comes, and where

The owner (§17 D6, D-next): "There should be no hard cap on dread creatures. just have configurable
1-in-x chance for dread creatures per dungeon level. But not all dangerous creatures should be
similarly dreadful. Ghouls can be pretty common compared to basilisks etc." Then, leaving level
generation out of it: "Just roll the 1-in-x every time generating a random monster. 1-in-200 or
whatever". The first draft's `dread` tier and its `most_alive` cap are gone; this is the rule.

### 8.1 The flag, and the roll

**`dread = true`** on a creature type (`CreatureDef::dread`, default false; the wiki's creature
tables show it) says what the type is; its **rarity tier stays its own**. Every time a random
monster is drawn (a level's population draw and the spawner's later draws alike), one roll
against **`dread_one_in`** (`data/dungeons.toml`'s `[defaults]`, beside `stray` and `out_of_depth`;
**default 200**; a branch may override it as it overrides the draw odds). On a hit the draw is
made among the `dread = true` types, each at its own level weight and rarity factor (ghouls
common, the basilisk very rare); otherwise among the non-dread types. If no dread type is
eligible, the draw falls back to the ordinary one. There is no floor-level state. **The vault
monster pools and the legends place what they place**, a scenario's hand-placed creatures are
untouched, and mimics are placed by the chest rule.

### 8.2 The odds, roughly

A floor draws about 40 creatures at the Fyrdelf scale (`population_count`: 25 to 80). At 1 in 200
a draw, the chance of at least one dread draw on the floor is 1 - (199/200)^40, about 18 %: roughly
**one floor in five or six** has a dread creature, before the spawner's later draws, and a hit
may come up empty if only a rare type is eligible. Which dread type comes is its own rarity's
weight among the dread types near the danger.

### 8.3 The tier moves (from §6)

Few now, since dread is a flag and not a tier. **To `very rare`**: the basilisk (from rare: the
owner's "compared to basilisks"). **To `rare`**: the vampire spawn (from uncommon), the fetch
(from uncommon). **To `uncommon`**: the six motes (from common: a floor of vortices is a floor
of explosions; they are not flagged, so they come in the ordinary draw), the charnel heap (from common). Every other dread type keeps its tier, the
barbed devil its common. The `very_rare` tier keeps the chiefs and kings (the warlords, the
jarl, the matron, the duergar king): rare because strong, not because dread, and they are
not flagged dread.

## 9. Warning signs: how the player learns of it in time

Four channels, all reading what exists, none of them a number:

1. **Cries** (`cry = { kind = "hiss", loudness = 60, every = 60 }` on the type): an idle sound
   the creature makes now and then where it stands (`Act::Cry(kind)`, one line in the creature's
   turn when it is not fighting, the draw from the gameplay stream only on its turn), heard by the
   hearing rules (through doors muffled, at the Listen ability's threshold at a door), shown in
   the `sound` layer and said as "You hear a hiss." New `SoundKind`s: **hiss** (the basilisk, the
   crocodile), **wail** (the banshee: it is also her aura), **giggle** (the pixie, the nymph: a
   voice), **buzzing** (the chasme), **rattle** (the charnel heap; the bone dragons), **knocking**
   (the knocker: the existing `banging`), **scream** (the screaming skull), **howl** (exists: the
   barghest and the hounds, the wargs' packs). A cry is a cheap warning that rewards the Listen
   skill and keen ears, and it is the floor's texture even when nothing is dread.
2. **Light**: the motes, the hand of glory, the will-o'-wisp, the balor, and the fire things carry
   light already; a vortex's glow coming down a dark corridor is the ADOM feel exactly.
3. **Leavings** (`leavings = ["statue", "web", "acid_pit", "torn_cloth"]`, a cell dressing the
   generator lays within a few cells of where a dread type is placed by a population draw or a
   vault: `mapgen::population::populate` after the placement, from the level's stream; a terrain
   or a fixture per kind: the **statue** (G2's, a stoned nobody, "a statue of a dwarf, mouth
   open", holding nothing), **webs** (the web trap's terrain, unarmed), **acid-pitted floor** (a
   floor variant, the `view` shows `.` in a sick green), **torn cloth** (an object, worth
   nothing)). A room with three statues is a room with a basilisk, and the player who reads it
   lives. Look mode names them.
4. **The first-sight line** (`first_sight = "A basilisk. Do not meet its eyes."` on the type),
   said **once per run per type** when one is first in the player's sight (`Progress::sighted`:
   a set of type ids, saved), after the usual perception lines. The game stays vague (no numbers,
   no mechanism named beyond the trope), the history keeps it, and the wiki's creature rows show
   it. Every dread type gets one; the traits do not.

And the gimmicks' own tells: the aura's first touch names its source, a gaze from the unseen
says "You feel a cold stare on you.", the shadowy within 2 "a cold draught", the invisible's
steps heard, the wormhost "writhing" in its look line, the splitting line on the first cut, the
troll's wounds closing.

### 9.5 The dead remembered: the graveyard file, and statues with names

The owner (§17 D9): "Can we somehow persist information about past characters? like list of
dead ones — how many characters have died? We could then track previously petrified characters
and potentially reuse their names as dungeon features. This should probably be pretty consistent
file, that does not get invalidated every time new version comes."

- **The file**: `graveyard.toml` beside `settings.toml` in the config dir (portable mode keeps it
  in `warlock-data` with the rest; `--paths` names it; the web build keeps it in `localStorage`
  under its own key, as the settings are), **append-only**, one `[[fallen]]` a character:
  `name`, `race`, `gender`, `class` (and `prestige` if called), `level`, `cause` (the death
  line's id: `stone` for the basilisk's, `starved`, a trap's, a killer's creature id with its
  name in words), `place` (the campaign, branch, and depth, or the overworld), `turn`, `age`
  (the character's), `when` (wall-clock seconds, the app's, as a save's), and `version` (the
  game that wrote it, for curiosity only). Written by the app in `screens::on_death` where the
  character's save is deleted today (docs/dev/saves.md "Permadeath"), never by core, never
  rewritten, never read into the rules.
- **Its format outlives every version** (the owner's word, and the one deliberate exception to
  the saves' pinning, recorded here): a reader ignores fields it does not know and defaults the
  ones it misses; a field once written is never renamed or given another meaning; an entry
  another version wrote is an entry. No format number, no migration: TOML with those rules is
  enough, and a test pins that a fixture written by 0.11 reads under the current version.
- **What reads it**: the **Characters screen** gains a line, "Twelve characters have died."
  (and, later, a page of them: a line each, the most recent first; an issue of its own), and the
  **death screen** says the entry was written ("Remembered in the graveyard."). The **generator**
  reads the **petrified** entries (`cause = "stone"`) to name a basilisk's statues (§9's
  leavings): "a statue of Brakka the dwarf, mouth open" instead of "a statue of a dwarf". A
  statue with a name is the player's own dead come back as a warning, which is the best warning
  this design has. One name a statue, the most recent petrified first, never twice on a floor.
- **Determinism**: the app reads the file once when a run begins and gives the run its list
  (`RunInput::fallen`, saved with the run so a resumed run sees the list it began with); the
  generator draws which statue gets which name from the level's stream, and the **name is
  decoration on the fixture, left out of the state hash**, so a replay made on a machine with
  another graveyard verifies (its statues unnamed, or named otherwise). Nothing of the rules
  reads a name.
- **Later** (deferred, a line on #196): gravestones for the rest of the fallen as leavings in
  the barrows, a bone with a name in a ghoul's den; the graveyard page.

## 10. What the player sees

- **Messages**: §5's and §9's words; nothing new in the message machinery (`perceive_events`,
  the existing `Event`s plus `Gazed`, `Burst`, `Split`, `Stole`, `Blinked`, `Cried`, each kept
  only when the player sees or hears its end as today).
- **The status bar**: Stiffening, Engulfed, Averted (derived, while a basilisk is in sight, as
  Dazzled is), Seeing, the auras' statuses as any.
- **Look mode**: the `seen` words ("stiffening", "engulfed", "writhing"), the grade's word, the
  leavings named, a statue "of a dwarf" or "of Brakka the dwarf" (§9.5), "your hound bristles
  at something" (G11).
- **The character screen**: nothing new.
- **The abilities menu**: nothing new (the first draft's Averted eyes toggle is gone: the
  basilisk forces it).
- **The Characters screen**: "Twelve characters have died." (§9.5).
- **The death screen**: "Turned to stone", "Eaten by the charnel heap" (the status's `death`),
  and "Remembered in the graveyard."
- **The dumps**: `actors` adds the grade, the innate statuses, and the gaze's, theft's,
  invisibility's, and blink's cooldowns among the effects, and `invisible` on an actor; `truth`
  shows statues (with their names) and leavings; the
  header's `averted`; JSON `grade` on a seen actor (the word), `leavings`. The dev `afflict:stiffening` and `afflict:invisible` work at once (statuses);
  dev `spawn` brings a dread type whatever the roll (the roll is the draw's, not the spawn's).
- **The wiki**: the creatures page gains "Dread" (what dread is, in words, and the counterplay
  by play: light, line of sight, blunt weapons, fire, range), and the generated creature tables
  gain the columns (gaze, aura, burst, splits, dies into, steals, blinks, corrodes, regen,
  shadowy, cry, first-sight line), the rarity tier, and the dread flag; the status-effects page the
  new statuses; the materials page the `corrodes` column and the two new metals; the balance
  page the probe's variance row; the dungeons page `dread_one_in` by branch.
  `xtask/src/wiki/creatures.rs` picks the fields up.

## 11. Talents and class abilities

**None new.** A creature gimmick extends the creatures system (CLAUDE.md: "a sub-feature (a
room effect, a creature gimmick, …) extends its parent and adds no tag and no talent"). What
answers each gimmick exists:

| Gimmick | Existing counters | By play (everyone) |
|---|---|---|
| Gaze, stiffening | Iron will, Divine grace, Still mind (Stillness ends it), Dwarven resilience and Stubborn (Toughness checks), the Ward power, the scroll of restoration, the Heal skill, the temple | Averted eyes (forced by the basilisk: the fight at −4 is the choice), doorways, darkness, range |
| Burst | the resistances, Poison resistance, Warden of the gate | shoot it, step back, doors |
| Fear aura | Iron will, Divine grace, Unbowed, the turncoat's Ward, Cool head | range, leave the hall |
| Cold, stench, silence, drowsy auras | the resistances, Still mind, Poison resistance | range |
| Splitting | — | blunt weapons, fire, acid |
| Dying into, breeding | burst spells | kill the breeder first |
| Theft | Sic 'em, Stealth, Steal | kill it; it drops where it dies |
| Blink to reach you | — | a closed door; missiles; it is soft |
| Engulf | Unstoppable, Stubborn (the struggle), Still mind | hack from inside |
| Corroding gear | Temper (the Metalcraft skill), acid resistance | wood, bare hands, spells; mithril, adamantium, gold, silver, stone |
| Regeneration | Flaming blade, Burning hands, Branding smite, the fire affixes | fire, burst it down |
| Shadowy | Nightsight does **not** (by design); Blood scent, Quick-sensed, the Light spell, Alert | a lantern, doorways |
| Invisibility | Blood scent, Quick-sensed, Alert and the Listen skill, the companion's nose, See invisible (#1102's list) and its potion | hearing, a corridor, strike at the cell, wait out the two minutes |

The owner (§17 D8): no talent, not Unblinking either; the first draft's offer is withdrawn.

## 12. Implementation plan (slices)

Rules first, each an issue; the content that uses a mechanism ships in the mechanism's PR where
the type is one or two, else as the tribe's content PR after it. Every slice: `warlock-core`
tests, the dumps, a replay created and verified, the wiki tables, the tribe file edited (never
`docs/dev/creatures.md` but for a new family), the spec's creatures section.

| Slice | What | Size |
|---|---|---|
| **D0** | Fear's teeth (`combat` on `frightened`, data; the probe's thief column checked); the `dread` flag and `dread_one_in` (in `[defaults]` and per branch, the roll in the monster draw, `candidates`, for the population and the spawner alike); the sort's tier moves (§8.3), data; the roster tests' rarity and dread rows | small; first |
| **D1** | The gaze (`[creature.gaze]`, the hook in the AI's fight step before throws, the cooldown status, the "you see it" read from the snapshot for the player and `sight::sees` for creatures, the unseen-stare line); the forced Averted condition (`averted` and `glance` on the gaze, `world/avert.rs`'s derived read, the status bar, the dumps); the vampires', hags', spirits', and ultroloth's riders moved onto the gaze | medium |
| **D2** | Stiffening (the status with `kills_at_max`, `stages`, `treat`; the scroll of restoration's line; the death line); the statue fixture (a cell fixture as a chest is; digging breaks it; kicking it, hard, with the marks); the basilisk's gaze with `averted` | medium |
| **D3** | The burst (`[creature.burst]`, `touches` and `dies`, through the area code with #1100's floor ruin; the vortex AI arm); the six motes as vortices (data, speed, tier); every elemental line's death throes by §5.3's table and the earth line's stones (data: `[creature.throw]`); the burning skull, magma crawler, and balor's death throes | medium |
| **D4** | Auras (`[creature.aura]`, sight and `heard`, the check, `every`); innate statuses (`innate`); cries (`cry`, `Act::Cry`, the new `SoundKind`s, the `sound` layer); the wraiths, banshee, hezrou, chasme, gibbering shade, bone and ice devils, satyr, the howling hounds, the salamander, barbed devil, balor, pit fiend, archlich and demilich | medium |
| **D5** | Splitting (`splits`, the kill credit at a quarter, the first-split line) and dying into (`dies_into` through `summon_cells`); breeding (a self-pool, `mana` optional at cost 0); the charnel heap, wormhost, wormlord | medium |
| **D6** | Theft on a cooldown (`steals`, the draw, the stack moved, the cooldown status) and the blink to reach you (`blinks`, the cooldown, the blocked-path trigger in the AI's approach); the pixie, nymph, imp, succubus and incubus, blink dog | medium |
| **D7** | Corroding gear (`corrodes` on the creature and `corrodes` on the materials with the review's values, `Worked::damaged` by acid, the "runs off" line; the two new material files, mithril and adamantium, with their loot and gear odds and the wiki's materials table; the ember beetle, hezrou); regeneration (`regen` on the type with §5.9's rule in the roster test, `regen = 0` on Burning and Corroded; the trolls, the marsh troll's dice) | medium |
| **D8** | The shadowy (`shadowy`, the perception pass, the adjacent "a shadow", the draught line) and invisibility (`invisible` innate and `[creature.invisible]` on a cooldown, the Invisible and Seeing statuses, the self-buff step of the AI shared with #1102, the sound mark of an unseen blow, the strike at a cell at −4, the companion's nose, the wisp's light out); the shadows, darkling, sluagh; the invisible stalker (new type), the duergar, imp, quasit, will-o'-wisp | large; after #1102's caster step lands, or carrying it |
| **D9** | Leavings (`leavings`, the four dressings, the generator's placement, look mode) and first-sight lines (`first_sight`, `Progress::sighted`); every dread type's line (content) | small |
| **D10** | The variance pass on #1097: the role offsets for to-hit, the 0.5 to 2.0 hit-point range, threat and crit by role, the speed rule; the roster tests; the probe's distribution row; `docs/balance.md` | medium |
| **D11** | Grades (`Actor::grade`, the draw, the five reads, the words, `grades = false` on what has none; the blades' words) | small |
| **D12** | The animated objects as decided (§13): the rug engulfs, the bellows staggers, the lantern bursts, the blades take grades with their words | small; after D3, D7, D11 |
| **D13** | The graveyard file (§9.5: the app's writer at death, the stable reader with its fixture test, `RunInput::fallen`, the Characters screen's line, the death screen's line) and the named statues (the generator's draw, the name on the fixture out of the hash, look mode) | medium; after D2 and D9 |
| **D14** | The second-wave types issue (§15: a black pudding, a cockatrice, a hydra, a rust monster, a gorgon), filed when D1 to D9 have landed, content only on the mechanisms built (the owner: "create that issue too") | an issue, not a slice |

D1 to D9 are independent of each other but for D2 on D1 and D4's innate statuses, which D3's
balor also wants (D3 may ship the balor's burst without its cloak), and D8's self-buff step,
which is #1102's caster AI's: D8 waits for it or carries it. D0 goes first because it is data
and the draw's roll shapes every later slice's placement. #1099 and #1102 run beside: the
dragons and the casters in §6 point at them and need nothing here.

## 13. The animated objects: the owner's part

Animated objects are the owner's design (`docs/design/animated-objects.md`, §16 the owner's
decisions). The sort above leaves the five families alone; what follows is the list of where the
gimmicks fit, asked as questions in the first draft and **decided by the owner as recommended**
(§17 D10: "go with recommendations"): the rug engulfs, the book gets nothing, the bellows
staggers, the lantern bursts when it dies, the blades take grades with their own words, the
armours and panoplies stay as they are, and no new family. Slice D12.

1. **The rug of smothering** (a household body, level 1 to 8): its name is G7's: should the rug
   **engulf** (`engulfed` with `bludgeoning`, `damage = 1`), the first engulfer the player meets,
   in Fennmynster's minster? (Recommended: yes; it is the one household thing with a trope.)
2. **The animated book**: a **silence aura** (its pages whisper over your words, radius 1)? Or a
   mana-drain touch? (Recommended: no; a book that fights is a joke the player should get, and a
   joke with a gimmick is a trap.)
3. **The bellows**: a gust that **staggers** (the rider)? (Recommended: yes, a trait.)
4. **The lantern**: it carries light already (the warning); a burst of oil when it dies (G3,
   `1d6` fire, radius 1)? (Recommended: yes: the lantern of Fennmynster's cellar as the first
   death throes.)
5. **Blades**: fast and hard already; a **grade** (§7.4) would give "a notched blade" and "a keen
   blade": the words would be the family's own. (Recommended: let grades apply, with the family's
   words.)
6. **Hollow armour and the panoply**: their pieces' magic already works for them; a panoply that
   wears a ring of Fire cloak retaliates today. Is that gimmick enough, with the dread roll of §8
   not touching them (they are not flagged dread)? (Recommended: yes, nothing added.)
7. **A new family** for dread: the **mirror** (shows your double: the fetch's
   `looks_like`), or the **noose** (an engulfer that hangs). (Recommended: not now; the sixteen
   bodies are the household's and a new family is the owner's design to extend.)

## 14. Determinism, saves, replays, dumps

- **Draws**: every new draw is from the gameplay stream on the creature's own turn after its AI
  decided without drawing (the house rule: `summon.rs`'s pattern): a gaze's chance, a burst's
  damage, a split's placement, a theft's stack and the blink's cell, an aura's check, a cry's
  moment. Grades and leavings draw from their own streams (`grade/<actor id>`,
  `leavings/<level id>`), so a run without them hashes as before.
- **Saves**: new actor fields, all `serde(default, skip_serializing_if)` so an old-shaped actor
  hashes the same: `grade`, `innate` statuses are ordinary effects refreshed each turn,
  cooldowns (the gaze's, the theft's, the blink's, invisibility's) and Invisible and Seeing are
  ordinary statuses (the house rule), `split_from` for the kill credit; `Progress::sighted` for
  the first-sight lines; `RunInput::fallen` with the run; statues
  (with their names, out of the hash) and leavings are cell fixtures as chests are. Averted is
  derived, never saved. Saves stay pinned to the version (no shims); the graveyard file is the
  one file that is not (§9.5, the owner's).
- **Replays**: no new player action (the first draft's `avert` is gone with the toggle);
  everything else is a creature's. A replay that meets a dread thing verifies as any, and one
  made on a machine with another graveyard verifies too (the names are out of the hash).
- **Dumps**: §10.
- **Performance**: an aura walks the hostiles within its radius on its turn (a handful); the
  shadowy adds one pass over the level's shadowy actors to `Perception::compute` (none on most
  floors); the dread roll is one draw and one flag read per candidate; the invisible's pass is the shadowy's. `bench-turns`'s rows are unchanged; D4 adds a
  floor of wraiths to `delf`'s fixture if the aura's cost shows.

## 15. Rejected ideas

| Idea | Why not |
|---|---|
| **Experience or level drain** (Angband, NetHack) | a level lost is a lock on what the player chose at that level (talents, rolls); the house already has stat drain mended at the temple, and Toughness drain *is* life drain here (hit points follow it). |
| **Instant petrification** (ADOM's gorgon, NetHack's cockatrice) | a permadeath on one failed roll with no warning is a dice roll, not dread; four degrees with lines and a wear-off keep the terror and give the player the three turns the trope is about. |
| **Ageing** (ADOM's ghost lords) | age has no rule to read; adding one for a gimmick is a system for a sub-feature. |
| **Corruption, mutation** (ADOM, DCSS) | a design of its own, with its own worth; not a creature gimmick. |
| **Confusion** (Angband's umber hulk) | no status; random steps are a design of their own (the stance of the drunk, if ever). |
| **Disenchanting** (NetHack) | an item losing its magic is a lock on a build; the smith's `damaged` state is the house's "it can be mended". |
| **Fear that forbids attacking** (ADOM's panic) | with permadeath a forbidden attack is a lock; −3 to hit is felt and fair. |
| **A thief that blinks or flees after the theft** (NetHack's nymph; the first draft's G6a) | the owner: an annoying flee mechanic. The thief stays; the theft is on a cooldown (§5.6). |
| **Blinking away when hurt** (the first draft's `when = ["hurt"]`: the quasit, the wisp, the archlich) | the same word; a thing that cannot be finished is a chore, not a dread. The blink stays for reaching you (the blink dog). |
| **A hard cap of dread creatures per floor, and a `dread` rarity tier** (the first draft's §8) | the owner: no cap, a configurable one-in-x, finally rolled at every random draw (§17 D-next), and dread of every rarity (§8). |
| **Regeneration on what cannot hurt you** | the owner: a regenerating sack of potatoes is free weapon marks (§5.9's rule). |
| **An Averted eyes toggle the player sets** (the first draft's §5.1) | the owner: the basilisk forces it (§5.1); no action, no flag, no replay step. |
| **Invisibility deferred** (the first draft's §15) | the owner wants it now: G11 (§5.11), with the invisible stalker and the duergar's cooldown cast. |
| **A size rule for engulfing** | a burden rule for a gimmick; the hold's check is the same for all. |
| **New creature types in the first slices** (puddings, cockatrices, hydras, rust monsters, gorgons) | the sort finds a bearer for every gimmick in the rosters as written; a short list for later: a black pudding (G5a, G8), a cockatrice (G2 by touch, a trait), a hydra (G5b heads), a rust monster (G8 `armour` and `weapon`), a gorgon (G2 by breath, with #1099). |
| **The dread tier kept out of out-of-depth draws** | the owner's rule (2026-10-27): anything may come at astronomical odds; that is where the stories come from. |
| **Talents per gimmick** | the talent rule, and every gimmick has counterplay by play. |

## 16. Open questions for the owner, each with a recommended default

The first draft's ten questions are answered in §17. What this round raises, each with a
default the slices take unless the owner says otherwise:

1. **The glance under forced averting** (§5.1): with the basilisk in sight the player is Averted
   and its gaze takes at a quarter of its chance, then the Toughness check. A quarter, or should
   averted eyes stop the gaze outright (then the basilisk's only cost is the −4 fight, and its
   stone comes only to the one who does not see it coming, e.g. in the dark)? *Default: a
   quarter; the owner answered "death" for stiffening's end, and a gaze that never lands while
   you look away is no death.*
2. **The roll's default** (§8.1): `dread_one_in = 200`, about one floor in five or six; per
   branch values (Grundlēas lower, the Ascmearc's first floors higher?). *Default: 200 for all;
   the numbers are data and the first dread PRs will say how it plays.*
3. **The invisible stalker's place** (§5.11): level 22, uncommon, among the air elementals'
   populations and a stray; one new type only. *Default: as written.*
4. **The graveyard's reach** (§9.5): write every death, and name statues from the petrified
   only; or write only the petrified for now? *Default: every death (the count the owner asked
   for needs them all; the page of the fallen is an issue of its own).*
5. **Mithril and adamantium** (§5.8): the numbers given are a first guess against the material
   tables; and `adamantium` as the id (the owner's word), not `adamantine`. *Default: as
   written, tuned in D7.*
6. **The marsh troll** (§5.9): a brute's dice (`2d8`) so it may keep its regeneration, or no
   regeneration and the sack it is? *Default: the dice; it is Fennmynster's troll and should
   fight like one.*

## 17. Decisions (owner, 2026-11-06)

The first draft's review on the PR (#1105), in two comments: one on the gimmicks, one answering
the ten questions of §16. Quoted where the owner wrote more than a word; "recommendation" where
the owner took the default. The sections changed are those this revision rewrote.

| # | Question or point | Decision | Sections changed |
|---|---|---|---|
| D1 | The gimmicks (the first comment) | **The burst**: "Burst is good. Give it to all elementals where appropriate (fire, electricity etc). Earth elemental should have stone throwing attack (but it switches to melee when player comes close)": death throes on every elemental line but earth, by §5.3's table; the earth line throws stones by the giants' rule, which already switches to melee beside its target. **Theft**: "theft should be on a cooldown, and not have annoying flee/blink away mechanic": §5.6, a cooldown status, the thief stays; the blink kept only for reaching you (the blink dog), every blink away gone. **Corroding gear**: "fine. Add corrosion resistant materials: mithril, adamantium. gold, silver etc. also should resist corrosion (review each material)": `corrodes` on every material with the review in §5.8, two new metals. **Regeneration**: "kinda meh, if the creature is not dangerous by itself. regenerating sack of potatoes is just free weapon marks!": §5.9's rule, only on brutes and dread; the marsh troll gets a troll's dice, the rotting troll loses the trait. **The shadowy**: "sounds good. why not also implement some invisible creatures while at it? like invisible stalker etc. Give duargars ability to use invisibility on CD (and fix the AI so they just cast it on cd and then continue their regular activities)": G11 (§5.11), the invisible stalker as a new type, the duergar's, imp's, quasit's, and wisp's cooldown cast, and the AI's self-buff step that casts and carries on. **Rarity**: "There should be no hard cap on dread creatures. just have configurable 1-in-x chance for dread creatures per dungeon level. But not all dangerous creatures should be similarly dreadful. Ghouls can be pretty common compared to basilisks etc.": §8 rewritten (D5 below) | §1, §2, §3, §5.3, §5.6, §5.8, §5.9, §5.11 (new), §5.13, §6, §8, §11, §12, §14, §15 |
| D2 | Q1 the sort as listed? | **"as listed"**; this revision's changes to it follow D1 only (the elementals, the blinkers, the duergar, the stalker) | §6 |
| D3 | Q2 fear's teeth, −3 to hit while Frightened? | **"sure"** | — |
| D4 | Q3 Averted eyes: a toggle, or none? | **"perhaps just basilisk forces the 'averting eyes' condition automatically, unless there is another sight blocking mechanism deployed"**: no toggle, no action; the basilisk's gaze carries `averted = true`, the player is Averted while they see it (derived, as Dazzled), its gaze takes at a quarter of its chance (the agent's reading, §16 Q1: the owner also chose death for stiffening, so the gaze must still land); another sight-blocking mechanism (darkness, a wall, a disguise) means neither the penalty nor the gaze; the other gazes have no averting | §1, §5.1, §10, §11, §12, §14, §15 |
| D5 | Q4 stiffening's end: death? | **"yeah, death"** | — |
| D6 | Q5 the cap: one or two? | **"no caps, see previous comment"**: no `dread` tier, no `most_alive`; `dread = true` on the type, a roll at `dread_one_in` per draw (D-next amends the first form of it), dread types drawn at their own rarity; the basilisk to very rare, the barbed devil common | §1, §3, §8, §10, §12, §15 |
| D7 | Q6 the grades? | **"ship it"**; "small"/"big" where a tribe names nothing (recommendation) | §1, §7.4, §12 |
| D8 | Q7 a talent? | **"none"** | §11 |
| D9 | Q8 the statue, and the player's own | **"fixture that can be digged. also can be kicked (hard to destroy) to train St."**: §5.2, dug through and kicked with the marks. And: **"Can we somehow persist information about past characters? like list of dead ones — how many characters have died? We could then track previously petrified characters and potentially reuse their names as dungeon features. This should probably be pretty consistent file, that does not get invalidated every time new version comes."**: the graveyard file (§9.5): every death appended, a format that outlives every version (the one exception to the saves' pinning, by the owner's word), the Characters screen's count, a basilisk's statues named from the petrified; slice D13 | §1, §3, §5.2, §9.5 (new), §10, §12, §14 |
| D10 | Q9 the animated objects | **"go with recommendations"**: the rug engulfs, the bellows staggers, the lantern bursts, the blades take grades with their words, nothing on the book, the armours, or a new family; slice D12 | §7.4, §12, §13 |
| D11 | Q10 the second-wave types | **"sure, create that issue too"**: an issue filed when D1 to D9 have landed (D14) | §12 |
| D-next | Q5 again: the dread floor (owner, 2026-11-06, round 3) | **"lets leave level generation out of this one. Just roll the 1-in-x every time generating a random monster. 1-in-200 or whatever"**: the dread floor is gone (no `Level::dread`, no roll in `populate`, no `truth` header); `dread_one_in` (default 200) is rolled at every random monster draw, the population's and the spawner's; a hit draws among the dread types at their own rarity | §1, §3, §6, §8, §10, §12, §14, §16 |
