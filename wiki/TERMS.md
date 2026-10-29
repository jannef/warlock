# Wiki term map

Where each mechanic and keyword of the player wiki is explained, for linking (#691). It lives
beside `wiki/src/`, not in it: it is a tool for page authors, not a page of the book.

**The rule** (CLAUDE.md, "Wiki"): new or edited wiki prose links every mechanic or keyword it
mentions to the section that explains it, with a relative link: `[Staggered](status-effects.md#status-staggered)`.
Link every mention in running prose, but not in headings, code, quotes of in-game text, or a
page's own subject on its own page (there, link the first mention in a section only when the
link would go elsewhere than the heading it sits under), and not twice in one short sentence.
Inflected forms link too: `[bleeds](status-effects.md#bleeding)`, `[Dwarves](races.md#race-dwarf)`.
A term with no explaining section gets a short definition where its topic is, and a line here.

`cargo xtask wiki --check-links` builds the book and fails on a link to a missing page or
anchor (CI runs it). mdBook's heading anchors are the heading in lower case, spaces as hyphens,
punctuation dropped (`## Hit points, Defence, and resistances` is
`#hit-points-defence-and-resistances`; a second `## Burden` on a page is `#burden-1`).

## Rows of the generated tables

Every row of these tables carries an anchor, made from the thing's id in the game's data
(`_` becomes `-`; an id that already starts with the prefix keeps it once:
`potion-of-healing`), so a link to it stays right when the data changes. To find one, look at the
table on its page, or grep the built page: `grep -o 'id="status-[a-z-]*"' wiki/book/status-effects.html`.

| Term | Anchor | Example |
|---|---|---|
| A status effect (Staggered, Winded, Raging, Hasted, …) | `status-effects.md#status-ID` | `status-effects.md#status-staggered` |
| A disease | `diseases.md#disease-ID` | `diseases.md#disease-filth-fever` |
| A skill (Stealth, Open lock, Spot, …) | `skills.md#skill-ID` | `skills.md#skill-open-lock` |
| A talent | `talents.md#talent-ID` | `talents.md#talent-hardy` |
| An ability in the abilities menu (class abilities, sign actives, spells, skill uses) | `abilities.md#ability-ID` | `abilities.md#ability-lay-on-hands` |
| A spell | `magic.md#spell-ID` | `magic.md#spell-fireball` |
| A damage type (fire, slashing, …) | `damage-types.md#damage-ID` | `damage-types.md#damage-fire` |
| A resource (Rage, Stamina, Ki, Focus, Inspiration, Openings) | `resources.md#resource-ID` | `resources.md#resource-ki` |
| A stance | `combat.md#stance-NAME` | `combat.md#stance-berserker` |
| A stat | `stats.md#stat-NAME` | `stats.md#stat-toughness` |
| A race | `races.md#race-ID` | `races.md#race-half-elf` |
| A weapon skill | `weapon-skills.md#weapon-skill-ID` | `weapon-skills.md#weapon-skill-long-blades` |
| A weapon skill's ability (Lunge, Sunder, …) | `weapon-skills.md#weapon-ability-ID` | `weapon-skills.md#weapon-ability-lunge` |
| A fighting style | `fighting-styles.md#style-ID` | `fighting-styles.md#style-sword-and-board` |
| A fighting style's ability (Riposte, Feint, …) | `fighting-styles.md#style-ability-ID` | `fighting-styles.md#style-ability-riposte` |
| A special room | `special-rooms.md#room-ID` | `special-rooms.md#room-draining` |
| A trap | `traps.md#trap-ID` | `traps.md#trap-pit` |
| A creature | `creatures.md#creature-ID` | `creatures.md#creature-kobold` |
| An item class | `objects.md#class-ID` | `objects.md#class-melee-weapons` |
| A potion | `potions.md#potion-ID` | `potions.md#potion-of-healing` |
| A scroll | `scrolls.md#scroll-ID` | `scrolls.md#scroll-of-identify` |
| A terrain | `terrain.md#terrain-ID` | `terrain.md#terrain-shallow-water` |

A term with a section of its own links to the section instead (Bleeding:
`status-effects.md#bleeding`; Rage: `resources.md#rage`; a class: `classes.md#barbarian`).
The generators make these anchors with `text::anchored` and link rows with `text::link`
(`xtask/src/wiki/text.rs`, the one helper); a new kind of row is a new `Term` there and a line
above.

## Mechanics and keywords

| Term | Link |
|---|---|
| **Time and movement** | |
| Turn, game time, seconds | `time.md#what-takes-time` |
| Action cost, a normal pace | `turns.md#what-actions-cost` |
| Speed (fast, slow) | `turns.md#speed` |
| Calendar, months, days | `time.md#the-calendar` |
| A month by name (Frostmere, Rainmoon, …) | `time.md#the-calendar` |
| Dawn, dusk, day, night | `time.md#days` |
| Resting, rest until dawn | `resting.md#how-resting-works` |
| **Light, sight, and sound** | |
| Light, lit, brightness | `light.md#what-light-is` |
| Light source, torch, lantern light | `light.md#light-sources` |
| Your lantern, its hood | `lantern.md`, `lantern.md#the-hood` |
| Sunlight, sky light | `sun.md#sky-light` |
| Moonlight, the moon's phases | `sun.md#moonlight`, `sun.md#phases` |
| Line of sight, field of view | `sight.md#line-of-sight` |
| Memory, remembered cells | `sight.md#memory` |
| Vision type (Normal, Low-light, Darkvision, UV sight), light-blind | `vision.md#the-four-types` |
| Hearing, noise, loudness | `hearing.md#what-makes-noise`, `hearing.md#how-loud` |
| Footsteps | `hearing.md#what-makes-noise` |
| Throwing your voice | `hearing.md#throwing-your-voice` |
| Weather (clear, overcast, rain, heavy rain) | `weather.md#how-the-weather-changes` |
| Temperature | `temperature.md` |
| Ice (thin ice, thick ice), falling in | `temperature.md#walking-on-ice`, `temperature.md#falling-in` |
| **The world** | |
| Overworld, travel | `overworld.md#travel` |
| Location, hidden location | `overworld.md#locations`, `overworld.md#hidden-locations` |
| Local area | `overworld.md#local-areas` |
| The Ascmearc and its places | `ascmearc.md#the-places` |
| Terrain, walls, water, stairs | `terrain.md#walls`, `terrain.md#water`, `terrain.md#stairs` |
| Digging, wall hardness | `digging.md#digging`, `digging.md#what-breaks-and-what-does-not` |
| Explosions, blasts | `digging.md#explosions` |
| Doors; locked, kicked, broken doors | `doors.md`, `doors.md#locked-doors`, `doors.md#kicking` |
| Creatures opening doors | `doors.md#creatures-and-doors` |
| Keys, skeleton key | `doors.md#keys` |
| Picking locks | `doors.md#picking-locks` |
| Secret door | `doors.md#secret-doors` |
| Chest, locked and trapped chests | `chests.md`, `chests.md#locked-chests`, `chests.md#trapped-chests` |
| Dungeon, danger level, population | `dungeons.md`, `dungeons.md#who-lives-there-and-how-many` |
| Out of depth, strays | `dungeons.md#who-lives-there-and-how-many` |
| Lairs of the wanted | `dungeons.md#the-lairs-of-the-wanted` |
| Special room | `special-rooms.md` (a room: its row) |
| Forgotten altar, praying | `special-rooms.md#forgotten-altars` |
| Forge, tempering | `special-rooms.md#forges` |
| Healing spring | `special-rooms.md#healing-springs` |
| Trap, hidden trap, known trap | `traps.md#finding-traps` |
| Disarming | `traps.md#disarming-traps` |
| Searching, noticing in passing | `searching.md#searching-1`, `searching.md#noticing-in-passing` |
| **Objects** | |
| Object, item class | `objects.md#what-an-object-is`, `objects.md#item-classes` |
| Stack, pile | `objects.md#piles` |
| Pack, inventory | `objects.md#your-pack` |
| Material, pieces | `objects.md#materials`, `objects.md#pieces` |
| Gem, size and cut | `gems.md`, `gems.md#size-and-cut` |
| Scroll, reading, unknown scrolls | `scrolls.md`, `scrolls.md#unknown-scrolls` |
| Potion, elixir | `potions.md`, `potions.md#elixirs` |
| Equipment, slots, wielding and wearing | `equipment.md#what-fits-where` |
| The equipment screen | `equipment.md#the-equipment-screen` |
| Magic item, rarity | `magic-items.md`, `magic-items.md#rarity` |
| Loot, item level | `magic-items.md#finding-them` |
| Looking at an item, its description | `magic-items.md#looking-at-an-item` |
| Unidentified, identify | `magic-items.md#unidentified-items` |
| Affix, prefix, suffix | `magic-items.md#prefixes`, `magic-items.md#suffixes` |
| Base item | `magic-items.md#base-items` |
| Unique item | `magic-items.md#unique-items` |
| Ammunition, arrows, bolts | `ammunition.md` |
| Burden (Unburdened, Burdened, Strained, Overloaded), carrying limit | `burden.md#burden-1`, `burden.md#how-much-you-can-carry` |
| Hunger, nutrition (Satiated, Hungry, Weak, Fainting, Starving, Bloated) | `hunger.md#how-hungry-you-are` |
| Eating, food, rot, corpses | `food.md#eating`, `food.md#food-spoils`, `food.md#corpses` |
| Shop, buying, selling, prices | `shops.md#buying`, `shops.md#selling`, `shops.md#prices` |
| Gold, money | `shops.md#money` |
| Theft from a shop | `shops.md#theft` |
| **Your character** | |
| Character, character creation | `character.md` |
| Name, gender | `character.md#name`, `character.md#gender` |
| Age, birthday, birth sign | `character.md#age-birthday-and-sign`, `horoscope.md#what-a-sign-does` |
| Race | `races.md` (a race: its row) |
| Evil races | `races.md#the-evil-races` |
| Class, class levels | `classes.md`, `classes.md#levels` (a class: its heading, `classes.md#barbarian`) |
| Starting kit | `starting-kit.md` |
| Background story, its chapters, how rare | `background.md`, `background.md#how-rare` |
| The story screen, the Story page (page 8) | `background.md#when-it-is-told` |
| Horoscope, animal, element, lucky day | `horoscope.md`, `horoscope.md#animals`, `horoscope.md#elements`, `horoscope.md#lucky-days` |
| The great cycle | `horoscope.md#the-great-cycle` |
| Alignment, good, evil, band | `alignment.md`, `alignment.md#what-moves-you` |
| Talent, talent ranks, prerequisites | `talents.md`, `talents.md#ranks-choices-prerequisites` (a talent: its row) |
| Racial talents | `talents.md#racial-talents` |
| A talent tree (Vigour, Shadow, Delver, …) | its heading, `talents.md#delver` |
| Class abilities | `talents.md#class-abilities`, `abilities.md` |
| Stats (Strength, Dexterity, …), score | `stats.md` (a stat: its row) |
| Modifier | `stats.md#modifiers` |
| Potential | `stats.md#potentials` |
| Stat training, training marks of a stat | `stats.md#training` |
| The daily check | `stats.md#the-daily-check` |
| Drained stats, stat damage | `stats.md#drained-stats`, `diseases.md#stat-damage` |
| Skills, trained and untrained | `skills.md`, `skills.md#what-skills-give` (a skill: its row) |
| Skill rolls, skill marks | `skills.md#getting-better` |
| Slipping past (a creature with darkvision, unlit) | `skills.md#slipping-past` |
| Stealing | `skills.md#stealing` |
| Treating yourself, binding a wound | `skills.md#treating-yourself` |
| Foraging | `skills.md#foraging` |
| Abilities, the abilities menu | `abilities.md#the-abilities-menu` |
| Experience, XP, levels | `experience.md`, `experience.md#levels` |
| Experience for discoveries | `experience.md#discovery` |
| A level-up, what a level gives | `experience.md#what-a-level-gives` |
| **Fighting** | |
| To hit, the 2d12 roll | `combat.md#hitting` |
| Defence (Dodge, Parry, Armour, Shield, Natural, Deflection) | `combat.md#defence` |
| Critical, threat, confirmed critical | `combat.md#criticals` |
| Damage | `combat.md#damage` |
| Protection | `combat.md#protection` |
| Resistance, resists, immune, vulnerable | `damage-types.md#armour-and-resistance` |
| Big hit | `combat.md#damage` |
| Stance | `combat.md#stances` (a stance: its row) |
| Free blow, cleave | `combat.md#free-blows` |
| Stamina | `combat.md#stamina` |
| Hit points, regeneration | `combat.md#hit-points`, `healing.md#how-fast` |
| Death, permadeath | `combat.md#death`, `saving.md` |
| Blood, stains | `combat.md#blood` |
| Ranged combat, shooting, throwing | `ranged.md#shooting-and-throwing` |
| Missile weapon, launcher, readied missiles | `ranged.md#getting-ready` |
| Where a missile goes, missiles in flight | `ranged.md#where-a-missile-goes` |
| Flaming arrows | `ranged.md#flaming-arrows` |
| Weapon skill, rank, marks | `weapon-skills.md`, `weapon-skills.md#marks-and-ranks` |
| Related weapon skills, defaults | `weapon-skills.md#related-skills` |
| Fighting style | `fighting-styles.md` (a style: its row) |
| Two-weapon fighting, off hand | `fighting-styles.md#two-weapons` |
| Damage type | `damage-types.md` (a type: its row) |
| Status effect, severity, degrees | `status-effects.md#how-they-work` (a status: its row) |
| Buffs and cooldowns | `status-effects.md#buffs-and-cooldowns` |
| Disease, incubation, checks | `diseases.md#how-a-disease-goes`, `diseases.md#checks` |
| Healing, healing paces | `healing.md#how-fast` |
| Mana, mana regeneration | `mana.md#how-much`, `mana.md#how-fast-it-comes-back` |
| Mana on a kill | `mana.md#mana-on-a-kill` |
| Spending mana | `mana.md#spending-it` |
| Magic, spells, casting | `magic.md`, `magic.md#the-spells` |
| Caster level | `magic.md#caster-level` |
| Spell tier | `magic.md#tiers` |
| Spell shape (bolt, burst, cone, …) | `magic.md#shapes` |
| How a spell resolves (attack, auto, area, touch) | `magic.md#how-a-spell-finds-its-target` |
| Mishap, damage ceiling | `magic.md#damage-and-its-cap` |
| Studying a spell from a scroll, overreach | `magic.md#learning-spells-from-scrolls` |
| Wand, rod, charges | `magic.md#wands`, `magic.md#rods` |
| Resource (Rage, Ki, Focus, …) | `resources.md` (a resource: its row; Rage: `resources.md#rage`) |
| **Creatures and people** | |
| Creature, monster | `creatures.md` (a creature: its row) |
| Faction, hostile, peaceful | `creatures.md#factions` |
| Peaceful folk, the fey courts | `creatures.md#factions` |
| Incorporeal | `creatures.md#incorporeal-creatures` |
| Elementals | `creatures.md#elementals` |
| A creature's numbers (stats, hit points, attacks) | `creature-numbers.md` |
| Animated object | `animated.md` |
| Mimic | `mimics.md` |
| People, townsfolk, chatting, topics | `npcs.md#chatting` |
| Services (cure, mending, identification) | `npcs.md#services` |
| Quest, quest log | `quests.md`, `quests.md#the-quest-log` |
| Sorts of creatures (a quest's), drawn quests | `quests.md#sorts-of-creatures`, `quests.md#drawn-quests` |
| The Weyfarers' Gilde, bounties, ranks | `guild.md`, `guild.md#the-wanted`, `guild.md#ranks` |
| **The game** | |
| Controls, keys, rebinding | `controls.md`, `controls.md#rebinding-keys` |
| Look mode | `controls.md#look-mode` |
| Aiming | `controls.md#aiming` |
| Message line, status bar | `controls.md#the-screen`, `controls.md#messages` |
| Character screen, info pages | `controls.md#character`, `controls.md#info-pages` |
| The skills screen | `controls.md#skills` |
| Main menu, pause menu | `controls.md#menus` |
| Settings, reduced effects, palettes | `settings.md`, `settings.md#palettes` |
| Saving, autosave | `saving.md#when-the-game-saves` |
