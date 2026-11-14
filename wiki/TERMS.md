# Wiki term map

Where each mechanic and keyword of the player wiki is explained, for linking (#691). It lives
beside `wiki/src/`, not in it: it is a tool for page authors, not a page of the book.

**The rule** (CLAUDE.md, "Wiki"; the owner, #1609): new or edited wiki prose links each mechanic
or keyword to the section that explains it, with a relative link: `[Staggered](status-effects.md#status-staggered)`.
Link a term once per section, at its first mention there; later mentions in the same section stay
plain text, so the links that answer the reader's next question stand out. Never link in
headings, code, or quotes of in-game text, nor a page's own subject on its own page (there, link
the first mention in a section only when the link would go elsewhere than the heading it sits
under), nor a term in a table on the term's own page.
Inflected forms link too: `[bleeds](status-effects.md#bleeding)`, `[Dwarves](races.md#race-dwarf)`.
A term with no explaining section gets a short definition where its topic is, and a line here.

`cargo xtask wiki --check-links` builds the book and fails on a link to a missing page or
anchor (CI runs it). mdBook's heading anchors are the heading in lower case, spaces as hyphens,
punctuation dropped (`## Hit points, Defence, and resistances` is
`#hit-points-defence-and-resistances`). A heading that repeats the page's title or another
heading on the page gets a numbered anchor (`#burden-1`), which breaks when a heading is added
above it: give every heading its own words (`## Burden levels` on the Burden page).

Headings are what mdBook's search finds first, so phrase a topic's heading as what a player would
look for, the question or the action: `## Levelling up`, `## Enchanting an item`, `## Curing a
disease`, not `## What a level gives` or `## Cures`. Renaming a heading changes its anchor: fix
every link to it (pages, `xtask/src/wiki/`, this file); `cargo xtask wiki --check-links` finds
the ones missed.

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
| A talent | `talents.md#talent-ID`; a birth sign's gift `horoscope.md#talent-ID`; a class ability `class-CLASS.md#talent-ID` (its first class's table, on that class's page; a prestige class's own, `prestige-classes.md#talent-ID`); a profane gift `profane-gifts.md#talent-ID` | `talents.md#talent-hardy`, `horoscope.md#talent-fleet-footed`, `class-berserker.md#talent-rage` |
| An ability in the abilities menu (class abilities, sign actives, spells, skill uses) | `abilities.md#ability-ID` | `abilities.md#ability-lay-on-hands` |
| A spell | `magic.md#spell-ID` | `magic.md#spell-fireball` |
| A damage type (fire, slashing, …) | `damage-types.md#damage-ID` | `damage-types.md#damage-fire` |
| A resource (Rage, Stamina, Ki, Focus, Inspiration, Openings, Zeal) | `resources.md#resource-ID`; a class's own pool has a page beside its class: `rage.md`, `focus.md`, `openings.md`, `ki.md`, `inspiration.md`, `zeal.md` | `resources.md#resource-ki`, `ki.md` |
| The thief's mark; Exploit and what the class talents add to it | `openings.md#the-mark`, `openings.md#exploit` | |
| Thrown exploit (the thief's, from 18) | `openings.md#thrown-exploit` | |
| The unholy (what holy wrath falls hardest on) | `class-paladin.md` | |
| A stance | `stances.md#stance-NAME` | `stances.md#stance-frenzy` |
| A stat | `stats.md#stat-NAME` | `stats.md#stat-toughness` |
| A race | `races.md#race-ID` | `races.md#race-half-elf` |
| A weapon skill | `weapon-skills.md#weapon-skill-ID` | `weapon-skills.md#weapon-skill-long-blades` |
| A school of magic (Evocation, Restoration, …) | `schools.md#school-ID` | `schools.md#school-evocation` |
| A weapon skill's ability (Lunge, Sunder, …) | `weapon-skills.md#weapon-ability-ID` | `weapon-skills.md#weapon-ability-lunge` |
| A fighting style | `fighting-styles.md#style-ID` | `fighting-styles.md#style-sword-and-board` |
| A fighting style's ability (Riposte, False lead, …) | `fighting-styles.md#style-ability-ID` | `fighting-styles.md#style-ability-riposte` |
| A caster style (Sniper, Toucher, Battle caster) | `schools.md#style-ID` | `schools.md#style-battle-caster` |
| A caster style's ability (Spell guard) | `schools.md#style-ability-ID` | `schools.md#style-ability-spell-guard` |
| A special room | `special-rooms.md#room-ID` | `special-rooms.md#room-draining` |
| A trap | `traps.md#trap-ID` | `traps.md#trap-pit` |
| A creature | `creatures.md#creature-ID` | `creatures.md#creature-kobold` |
| What a creature's body leaves (its material drops) | `creature-drops.md#drops-ID` | `creature-drops.md#drops-wolf` |
| A creature tag's material drops | `creature-drops.md#drops-tag-TAG` | `creature-drops.md#drops-tag-beast` |
| An item class | `objects.md#class-ID` | `objects.md#class-melee-weapons` |
| A herb (glowcap, moonwort, mandrake, …) | `herbs.md#herb-ID` | `herbs.md#herb-moonwort` |
| An essence (of fire, ice, lightning, air, water, earth) | `materials.md#herbs-and-essences` | `materials.md#herbs-and-essences` |
| A potion | `potions.md#potion-ID` | `potions.md#potion-of-healing` |
| A use of the priest's prayer (disease, poison, wounds, release, mana, hunger, restoration) | `prayer.md#prayer-ID` | `prayer.md#prayer-wounds` |
| A scroll | `scrolls.md#scroll-ID` | `scrolls.md#scroll-of-identify` |
| A terrain | `terrain.md#terrain-ID` | `terrain.md#terrain-shallow-water` |

A term with a section of its own links to the section instead (Bleeding:
`status-effects.md#bleeding`; Rage: `rage.md`; a class: `class-berserker.md`).
The generators make these anchors with `text::anchored` and link rows with `text::link`
(`xtask/src/wiki/util.rs`, the one helper); a new kind of row is a new `Term` there and a line
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
| Running | `turns.md#running` |
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
| Encounter, meeting someone on the way, its chance | `encounters.md`, `encounters.md#the-chance-of-meeting-someone` |
| Lookout, stalk, having the drop, ambushed, passing each other by | `encounters.md#who-sees-whom` |
| Lying in wait, slipping past, giving them the slip, hailing, hunting | `encounters.md#deciding-what-to-do` |
| Talking your way past a party (Bluff's overworld use) | `encounters.md#talking-past` |
| Camping (resting on the overworld), the sleeper's lookout | `encounters.md#camping-and-foraging` |
| An encounter table | `encounters.md#the-ascmearcs-encounters` (a table: `encounters.md#encounter-table-ID`) |
| The Ascmearc and its places | `ascmearc.md#the-places` |
| Terrain, walls, water, stairs, creatures following you on stairs | `terrain.md#walls`, `terrain.md#water`, `terrain.md#stairs` |
| Deep water, swimming | `terrain.md#deep-water` |
| Digging, wall hardness | `digging.md#digging`, `digging.md#what-breaks-and-what-does-not` |
| Explosions, blasts | `digging.md#explosions` |
| Mining, ore, smelting, veins, each dungeon's ores | `mining.md` (a find: `mining.md#mined-ID`, `mining.md#mined-iron`; a vein: `mining.md#vein-ID`, `mining.md#veins`) |
| Doors; locked, kicked, broken doors | `doors.md`, `doors.md#locked-doors`, `doors.md#kicking-a-door-open` |
| Creatures opening doors | `doors.md#creatures-and-doors` |
| Keys, skeleton key | `doors.md#keys` |
| Picking locks | `doors.md#picking-locks` |
| Secret door | `doors.md#secret-doors` |
| Chest, locked and trapped chests | `chests.md`, `chests.md#locked-chests`, `chests.md#trapped-chests` |
| Dungeon, danger level, population | `dungeons.md`, `dungeons.md#who-lives-there-and-how-many` |
| Out of depth, strays | `dungeons.md#who-lives-there-and-how-many` |
| Dread, a dread draw | `dungeons.md#dread` |
| Lairs of the wanted | `dungeon-guide.md#the-lairs-of-the-wanted` |
| Grundlēas, floors forgotten | `dungeon-guide.md#grundlēas` |
| Hellegeat, a Pit | `pits.md`, `pits.md#how-a-pit-opens`, `pits.md#floors-that-forget` |
| A Pit's warden, a sealed Pit | `pits.md#the-warden` |
| Special room | `special-rooms.md` (a room: its row) |
| Forgotten altar, praying | `altars-forges-springs.md#forgotten-altars` |
| Forge, tempering, Fine, Marvellous, Masterwork, damaged items | `altars-forges-springs.md#forges` |
| Makeshift anvil, Field forge | `altars-forges-springs.md#the-makeshift-anvil`, `class-craftsperson.md#field-forge` |
| Item damage, corroded, rusted, scorched, withstanding | `materials.md#item-damage` |
| Protected items, of Proof | `materials.md#protected-items` |
| Healing spring | `altars-forges-springs.md#healing-springs` |
| Trap, hidden trap, known trap | `traps.md#finding-traps` |
| Disarming | `traps.md#disarming-traps` |
| Rig, rigging, Trapwright, a rigged trap | `traps.md#rigging-traps` (the talent: `class-craftsperson.md#talent-trapwright`) |
| Searching, noticing in passing | `searching.md#how-to-search`, `searching.md#noticing-in-passing` |
| **Objects** | |
| Object, item class | `objects.md#what-an-object-is`, `objects.md#item-classes` |
| Stack, pile | `objects.md#piles` |
| Pack, inventory | `objects.md#your-pack` |
| Material, pieces | `materials.md#materials`, `materials.md#pieces` |
| Material drops (stock a body leaves) | `creature-drops.md` |
| Gem, size and cut | `gems.md`, `gems.md#size-and-cut` |
| Scroll, reading, unknown scrolls | `scrolls.md`, `scrolls.md#unknown-scrolls` |
| Making out a scroll, the check to read one (Decipher script's use) | `scrolls.md#making-out-a-scroll` |
| Potion, elixir | `potions.md`, `potions.md#elixirs` |
| Flask, throwing a potion | `potions.md#flasks` |
| Equipment, slots, wielding and wearing | `equipment.md#what-fits-where` |
| The equipment screen | `equipment.md#the-equipment-screen` |
| Magic item, rarity | `magic-items.md`, `magic-items.md#rarity` |
| Loot, item level | `magic-items.md#finding-them` |
| Looking at an item, its description | `magic-items.md#looking-at-an-item` |
| Unidentified, identify | `magic-items.md#unidentified-items` |
| Capacity, enchantments an item can bear, the trim | `magic-items.md#capacity` |
| Deeper finds, better make and gems by item level | `magic-items.md#deeper-finds` |
| Binding (a material's) | `magic-items.md#capacity`, `materials.md` |
| Affix, prefix, suffix | `affixes.md#prefixes`, `affixes.md#suffixes` |
| Base item | `affixes.md#base-items` |
| Unique item | `magic-items.md#unique-items` |
| Ammunition, arrows, bolts | `ammunition.md` |
| Burden (Unburdened, Burdened, Strained, Overloaded), carrying limit | `burden.md#burden-levels`, `burden.md#how-much-you-can-carry` |
| Hunger, nutrition (Satiated, Hungry, Weak, Fainting, Starving, Bloated) | `hunger.md#how-hungry-you-are` |
| Eating, food, rot, corpses | `food.md#eating`, `food.md#food-spoils`, `food.md#corpses` |
| Everlasting (a keeper's corpse, which never rots) | `food.md#corpses` |
| Profane gift, profane corpse, eating a keeper's corpse | `profane-gifts.md`, `profane-gifts.md#eating-a-keepers-corpse` (a gift: its row) |
| Signature gift (a keeper's own, sure) | `profane-gifts.md#each-keepers-own-gift` |
| Shop, buying, selling, prices | `shops.md#buying`, `shops.md#selling`, `shops.md#prices` |
| Gold, money | `shops.md#money` |
| Theft from a shop | `shops.md#theft` |
| **Your character** | |
| Character, character creation | `character.md` |
| Name, gender | `character.md#name`, `character.md#gender` |
| Age, birthday, birth sign | `character.md#age-birthday-and-sign`, `horoscope.md#what-a-sign-does` |
| Race | `races.md` (a race: its row) |
| Evil races | `races.md#the-evil-races` |
| Class, class levels | `classes.md`, `classes.md#levels` (a class: its page, `class-ID.md`, as `class-berserker.md`) |
| A class's role and difficulty, choosing a class, the at-a-glance table | `classes.md#at-a-glance`, `first-run.md#choosing-a-class` |
| Which classes a race and gender may take, classes not offered | `classes.md#who-may-take-which` (a race's row: `classes.md#limits-drow`) |
| Starting kit | `starting-kit.md` |
| Background story, its chapters, how rare | `background.md`, `background.md#how-rare` |
| Story, a story's sentence (line), its kind (tier: common, uncommon, rare, fateful) | `background.md#how-rare` |
| Fateful (a story's rarest kind) | `background.md#how-rare` |
| Every story sentence, its conditions and effects; tags and word slots | `background.md#every-sentence` |
| The story screen, the Story page (page 8) | `background.md#when-it-is-told` |
| Horoscope, animal, element, lucky day | `horoscope.md`, `horoscope.md#animals`, `horoscope.md#elements`, `horoscope.md#lucky-days` |
| The great cycle | `horoscope.md#the-great-cycle` |
| Alignment, good, evil, band | `alignment.md`, `alignment.md#what-moves-you` |
| Talent, prerequisites | `talents.md`, `talents.md#choices-and-prerequisites` (a talent: its row) |
| Racial talents | `talents.md#racial-talents` |
| Trade, Crafts, the tools of the trade (Smith, Wright, Gemcutter, Brewster, Pitman, Wortcunner, Spellwright) | `talents.md#crafts-a-craftspersons-trades` (a trade: its row) |
| The craftsperson's two trades at creation, Time served | `class-craftsperson.md#trades` |
| Eye for the make, reading an item's make | `class-craftsperson.md#eye-for-the-make` |
| Make do and mend, mending in the field | `class-craftsperson.md#make-do-and-mend` |
| Maker's mark, a marked piece | `class-craftsperson.md#makers-mark` |
| Master's hand, grades and cuts sooner | `class-craftsperson.md#masters-hand` |
| Quick hands, every craft in half the time | `class-craftsperson.md#quick-hands` |
| Waste not, nothing lost in a craftsperson's hands | `class-craftsperson.md#waste-not` |
| A talent tree (Vigour, Shadow, Delver, …) | its heading, `talents.md#delver` |
| Class abilities | `classes.md#levels`, `abilities.md` |
| The bard, a bard's cap at 30, Travelled hands, Jack of all trades | `class-bard.md` |
| Songs (Inspire, the Lullaby), singing, a song's degrees, shaking a song off, Virtuoso | `abilities.md#songs` |
| Prestige class, the Hunter, total level, a class's cap (last level) | `prestige-classes.md#prestige-classes` (a prestige class's table: its heading, `prestige-classes.md#hunter`) |
| A calling, being called | `prestige-classes.md#callings` |
| Spirit weapon | `abilities.md#ability-spirit-weapon` |
| The Strange Spirit, the oath | `npcs.md#the-strange-spirit` |
| The priest's prayer (Orison, Prayer, Litany, Benediction, Miracle), what it answers | `prayer.md`, `prayer.md#what-it-answers` (a use: its row) |
| Strain, a fizzled prayer | `prayer.md#strain` |
| The holy and the dark powers, the dark face's twists (Whisper, Invocation, Rite, Pact) | `prayer.md#the-holy-and-the-dark` |
| Communion with the dark | `class-priest.md#talent-communion`, `prayer.md#communion-and-sanctuary` |
| Hallowed ground, dreaded ground (Sanctuary) | `cursed-and-hallowed.md#hallowed-ground` |
| Stats (Strength, Dexterity, …), score | `stats.md` (a stat: its row) |
| Modifier | `stats.md#modifiers` |
| Potential | `stats.md#potentials` |
| Stat training, training marks of a stat | `stats.md#raising-your-stats` |
| The daily check | `stats.md#the-checks` |
| Drained stats, stat damage | `stats.md#drained-stats`, `diseases.md#stat-damage` |
| Skills, trained and untrained | `skills.md`, `skills.md#what-skills-give` (a skill: its row) |
| Skill rolls, skill marks | `skills.md#improving-a-skill` |
| A skill's ladder (what it gives by value: Music's song degrees and difficulty, …) | `skills.md#what-skills-give` |
| Slipping past (a creature with darkvision, unlit) | `skill-uses.md#slipping-past` |
| Stealing | `skill-uses.md#stealing` |
| Feinting (a lie in combat, Bluff's use) | `skill-uses.md#feinting` |
| Intimidating (a threat at one foe) | `skill-uses.md#intimidating` |
| Treating yourself, binding a wound | `skill-uses.md#treating-yourself` |
| Fletching, brittle arrows and bolts, a knife | `skill-uses.md#fletching` |
| Arrowhead, a batch (counted recipe), sling stones, flaming arrows (made) | `handcraft.md#batches-arrowheads-arrows-and-stones` |
| Profession, recipe | `skill-uses.md#professions` |
| Crafting, forging, a recipe, stock to forge from | `crafting.md`, `forging.md#forging-a-weapon` (a recipe: its row, `recipe-ID`) |
| Hand-made weapons, knapping flint, a bone or flint knife, a club, a staff by hand | `handcraft.md#making-a-weapon-by-hand` |
| Masterwork (crafted), a crafted item's grade, margin | `crafting.md#how-well-it-comes-out` |
| Brewing, potion recipes, Alchemy's recipes, a failed brew (murky draught, brewer's blunder) | `recipes-alchemy.md`, `recipes-alchemy.md#how-a-brew-goes` (a recipe: its row, `brew-ID`) |
| Foraging | `skill-uses.md#foraging` |
| Crafting, a craft skill, craft points, who works which material, a material's work | `crafting.md`, `crafting.md#which-skill-makes-what` |
| Salvage, taking an item apart, a gem chipped or shattered | `salvage.md` |
| Magic crystal, a crystal's colour, consuming a crystal | `crystals.md`, a colour `crystals.md#crystal-ID` (`crystals.md#crystal-red`), `crystals.md#consuming-a-crystal` |
| Heart-crystal, soul-fetter (rare materials) | `crystals.md#rare-materials`; what they do: `enchanting.md#the-rare-materials` |
| Seal (in enchanting) | `enchanting.md#keeping-magic-out-with-seals` |
| Enchanting, Kindle, Quicken, Unbind, an enchanting recipe | `enchanting.md`, `enchanting.md#enchanting-an-item` |
| Reach (an enchanter's item level), warped (a natural 2 in enchanting) | `enchanting.md#how-strong-the-magic-is`, `enchanting.md#the-check` |
| Cut, gem cutting, a re-cut, a gem's cut grade | `gem-cutting.md`, `gem-cutting.md#how-well-it-comes-out` |
| Set, setting a gem in an item, a gem chipped in the setting | `gem-cutting.md#setting-a-gem` |
| Jewellery, making rings and amulets, a bone amulet | `jewellery.md` |
| Gather, gathering herbs, the glowing fungus | `herbs.md#gathering` |
| Herbs (in general) | `herbs.md` |
| Abilities, the abilities menu | `abilities.md#the-abilities-menu` |
| Experience, XP, levels | `experience.md`, `experience.md#levels` |
| The level curve to 100, the XP of every level | `experience.md#levels` |
| Experience for discoveries | `experience.md#experience-for-something-new` |
| A level-up, what a level gives | `experience.md#levelling-up` |
| Levelling up: every choice a level brings (the level-up screen) | `experience.md#when-you-level-up` |
| **Fighting** | |
| To hit, the 2d12 roll | `combat.md#hitting` |
| Finesse weapon | `combat.md#finesse-weapons` |
| Defence (Dodge, Parry, Armour, Shield, Natural, Deflection) | `combat.md#defence` |
| Critical, threat, confirmed critical | `combat.md#criticals` |
| Damage | `combat.md#damage` |
| Protection | `combat.md#protection` |
| Resistance, resists, immune, vulnerable | `damage-types.md#armour-and-resistance` |
| Big hit | `combat.md#damage` |
| Stance | `stances.md#stances` (a stance: its row) |
| Free blow, cleave | `stances.md#free-blows` |
| Stamina | `stances.md#stamina` |
| Hit points, regeneration | `hit-points.md#hit-points`, `healing.md#how-fast-you-heal` |
| Death, permadeath | `hit-points.md#death`, `saving.md` |
| Blood, stains | `hit-points.md#blood` |
| Char, scorch marks | `hit-points.md#char` |
| Ranged combat, shooting, throwing | `ranged.md#shooting-and-throwing` |
| Missile weapon, launcher, readied missiles | `ranged.md#getting-ready` |
| Shot distance, long range, a shot from beside the target | `ranged.md#distance` |
| Where a missile goes, missiles in flight | `ranged.md#where-a-missile-goes` |
| Flaming arrows | `ranged.md#flaming-arrows` |
| Weapon skill, rank, marks | `weapon-skills.md`, `weapon-skills.md#raising-a-weapon-skill` |
| Target dummy, training dummy, the Gilde's yard | `weapon-skills.md#target-dummies`, `guild.md#the-yard` |
| Fighting style | `fighting-styles.md` (a style: its row) |
| Two-weapon fighting, off hand | `fighting-styles.md#two-weapons` |
| Damage type | `damage-types.md` (a type: its row) |
| Status effect, severity, degrees | `status-effects.md#how-they-work` (a status: its row) |
| Buffs and cooldowns | `status-effects.md#buffs-and-cooldowns` |
| Disease, incubation, checks | `diseases.md#how-a-disease-goes`, `diseases.md#checks` |
| Healing, healing paces | `healing.md#how-fast-you-heal` |
| Getting your health back, every way; curing bleeding, poison, and disease | `healing.md#getting-your-health-back`, `healing.md#curing-bleeding-poison-and-disease` |
| Mana, mana regeneration | `mana.md#how-much-mana-you-have`, `mana.md#how-fast-it-comes-back` |
| Mana on a kill | `mana.md#mana-on-a-kill` |
| Spending mana | `mana.md#spending-it` |
| Magic, spells, casting | `magic.md`, `magic.md#the-spells` |
| Learning (what a grimoire teaches for certain, Decipher script's curve) | `learning-spells.md#your-learning` |
| Spell level, cantrip | `magic.md#spell-levels` |
| Ladder, rung, base | `magic.md#the-spells`, `spellcasting.md#how-a-spell-grows` |
| What talents and items do to a spell (a rung, bigger dice, a bonus, a reroll, a rider) | `spellcasting.md#how-a-spell-grows` |
| Spell shape (bolt, burst, cone, …) | `spellcasting.md#shapes` |
| Spell type, what a spell of a damage type does beyond its damage | `spell-types.md` |
| Arc, conducting through water, what an electricity spell does | `spell-types.md` |
| Burning | `status-effects.md#burning` |
| Corroded | `status-effects.md#corroded` |
| How a spell resolves (attack, auto, area, touch) | `spellcasting.md#how-a-spell-finds-its-target` |
| Healing spells on the undead (harm instead of healing) | `spell-rules.md#healing-harms-the-undead` |
| Mishap | `learning-spells.md#learning-a-spell` |
| Miscast, the Spellcraft check on casting, the miscast table | `miscasts.md#the-check`, `miscasts.md#the-miscast-table` |
| Protected results, the protected half (a miscast that cannot kill) | `miscasts.md#the-protected-half` |
| Gate (a miscast's: rift, hellmouth, ... lords' gate), the loosed | `miscasts.md#gates` (a gate: `creatures.md#creature-ID`) |
| A miscast's Pit stair | `miscasts.md#pits` |
| Learning a spell from a grimoire, overreach | `learning-spells.md#learning-a-spell` |
| Spell rank, deepening a spell, rereading a grimoire | `learning-spells.md#spell-ranks` |
| Forgetting a spell, a forgotten spell, ranks lost to a miscast | `miscasts.md#forgotten-spells` |
| Wand, rod, charges | `wands-and-rods.md#wands`, `wands-and-rods.md#rods` |
| Spell circle, buff, hex (as seen on a creature) | `spell-circles.md#spell-circles` |
| Resource (Rage, Ki, Focus, …) | `resources.md` (a resource: its row; Rage: `rage.md`) |
| **Creatures and people** | |
| Creature, monster | `creatures.md` (a creature: its row) |
| Faction, hostile, peaceful, the berserk, the loosed, the gates | `creatures.md#factions` |
| What creatures do, lurking | `creature-behaviour.md#what-they-do` |
| Peaceful folk, the fey courts | `creatures.md#factions` |
| Incorporeal | `creature-bodies.md#incorporeal-creatures` |
| Elementals | `creature-bodies.md#elementals` |
| Grades: a greater or lesser creature (big, scrawny, ancient, withered) | `creature-grades.md` |
| A creature's gear and its condition (rusted, Fine) | `creature-gear.md` |
| A creature's numbers (stats, hit points, attacks) | `creature-numbers.md` (the tables: its family's page, `creature-numbers-FAMILY.md`) |
| A creature's craft for surprise (its sneak attack, initiative, spot) | `creature-numbers.md#surprise` |
| Animated object | `animated.md` |
| Mimic | `mimics.md` |
| Ally, your wolf, orders (Come to heel, Hold here, Set on) | `allies.md`, `allies.md#orders` |
| Summon, summon spell, a call (the druid's) | `allies.md#summons`, `allies.md#the-druids-calls` |
| Companion, animal companion, the nose, a companion's level (grade) | `companions.md#companions`, `companions.md#starting-companions`, `companions.md#how-a-companion-grows` |
| The collar, Give | `companions.md#the-collar` |
| Tame, taming | `companions.md#taming` |
| Familiar, a familiar's touch, what it goes for, calling one | `familiars.md#familiars`, `familiars.md#the-touch`, `familiars.md#what-it-goes-for`, `familiars.md#call-familiar` |
| Familiar's sight | `familiars.md#familiars-sight` |
| Hunter's bond, shared sight, shared buffs | `companions.md#hunters-bond` |
| People, townsfolk, chatting, topics | `npcs.md#chatting` |
| Services (cure, mending, identification, training) | `npcs.md#services` |
| Alberich (teaches Mining) | `npcs.md#alberich` |
| Eadric the smith (teaches Metalcraft) | `npcs.md#eadric-the-smith` |
| Ceolred, the smith's apprentice (explains the materials) | `npcs.md#ceolred-the-smiths-apprentice` |
| Goodwill (a town's, cheaper services) | `npcs.md#goodwill` |
| The aristocrat's stipend, their servant in Cēaptūn | `class-aristocrat.md#stipend` |
| The merchant's investment, their business partner in Cēaptūn | `class-merchant.md#invest` |
| The craftsperson's patron, a commission | `patron.md`, `patron.md#what-does-the-patron-want` |
| The pedlar's offer, his wares | `npcs.md#the-pedlars-offer` |
| Quest, quest log | `quests.md`, `quests.md#the-quest-log` |
| Sorts of creatures (a quest's), drawn quests | `quests.md#sorts-of-creatures`, `quests.md#drawn-quests` |
| The Weyfarers' Gilde, bounties, ranks | `guild.md`, `guild.md#the-wanted`, `guild.md#ranks` |
| **The game** | |
| Getting started, a first run, the keys you need first | `first-run.md` |
| A task (how do I ...) | `how-do-i.md` |
| Controls, keys, rebinding | `controls.md`, `controls-rebinding.md` |
| Look mode | `controls-prompts.md#look-mode` |
| Aiming | `controls-prompts.md#aiming` |
| Something beside you, acting at once on the one target | `controls-prompts.md#something-beside-you` |
| Message line, status bar | `the-screen.md#the-screen`, `the-screen.md#messages` |
| Character screen, info pages | `the-screen.md#character`, `the-screen.md#info-pages` |
| The skills screen | `the-screen.md#skills` |
| Main menu, pause menu | `the-screen.md#menus` |
| Settings, reduced effects, palettes | `settings.md`, `settings.md#palettes` |
| Saving, autosave | `saving.md#when-the-game-saves` |
| The saved characters, the graveyard | `saving.md#your-characters-and-the-graveyard` |
