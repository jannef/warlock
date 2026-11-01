# Class ability gaps after the talent cleanup

A report for the owner, not a design: it lists which class tables have no class ability at which levels, and which PR emptied each. What fills a gap is the owner's call. Read from `assets/data/classes/*.toml` on `main` (after #780, #782, #798, #799, #800, #801).

## The cadence

From `docs/design/progression.md` section 4.1 ("ADOM gives every class seven fixed powers at levels 6, 12, 18, 25, 32, 40, and 50 ... we add a signature at level 1 and a capstone at 60"): **class abilities at 1, 6, 12, 18, 25, 32, 40, 50, 60** (nine levels), with bonus talent picks between them (levels 4, 10, 16, 22, 28, 34, 46, 58 for the warrior and thief; 10, 20, 30, 45, 55 for the rest). The table of nine entries per class is also what the design's count assumes ("15 classes x 9 entries").

The four warrior-like and spell-heavy tables drift from it on purpose: the priest, druid, witch and wizard hold early spell grants at 3, 4 and 9, and the wizard has 45 (Magister) beside 40. The ranger has an extra 2 (Stalker's eye, #788). A gap below means a level of the standard nine with no entry. Off-cadence entries are listed but not counted as covering a standard level.

"Spell" marks a talent whose effect is a spell grant paid in mana (`kind = "spell"`). Every `trained_wind` is a level-1 filler shared by the martial classes.

How a gap's cause is read: a talent removed in the git history of the class file (#780 `33dafc96`, #782, #798 `43a8aad8`, #799 `4c36cc13`, #800 `9628cf53` which only swapped ids, #801 `f772f320`). "Never" means none of those PRs removed an entry there, so the level was already empty before the wave.

Note: #780's description lists three class abilities it emptied (hearty, weather_sense, jeweller). Its diff of the class files also removed the duplicate cuts Wide learning (bard 6), Keen appraisal (craftsperson 1), Connoisseur (aristocrat 25) and Appraiser (merchant 32), and #782 removed Spell recall (wizard 12). Only the ones that left a gap are listed below: bard 6, druid 12 and wizard 12 are still filled by another entry.

## Per class

### Barbarian: 10 abilities, 0 gaps
1 Rage, Trained wind; 6 Thick hide; 12 Fast movement; 18 Brutal critical; 25 Hardened; 32 Mighty thews; 40 Blood frenzy; 50 Mighty rage; 60 Unstoppable. (#798 removed Hard living at 18; Brutal critical still holds it.)

### Bard: 9 abilities, 2 gaps
1 Lore, Mend (spell), Trained wind; 6 Blessing (spell); 12 Fast healer; 18 Inspire; 25 Jack of all trades; 32 Throw voice; 60 Virtuoso.
- 40: emptied by #798 (Honeyed words). Direction: voice.
- 50: emptied by #798 (Legend). Direction: legend.

### Priest: 10 abilities, 2 gaps (7 of 9 standard levels, plus 3 and 9)
1 Mend (spell); 3 Blessing (spell), Cure wounds (spell); 9 Consecrate; 12 Remove disease (spell); 25 Purify; 32 Purity of body; 40 Smite; 50 Miracle; 60 Saint.
- 6: never had one (the table spends its early picks at 3 and 9 instead).
- 18: emptied by #798 (Ward of faith). Direction: ward.

### Druid: 6 abilities, 5 gaps (4 of 9 standard levels, plus 3)
1 Beast-friend, Spark (spell); 3 Barkskin (spell); 12 Call lightning (spell); 18 Nature's ward; 50 Flame and frost (spell).
- 6: emptied by #801 (Woodland stride, outdoors only). Direction: beasts.
- 25: emptied by #801 (Winter's friend, outdoors only).
- 32: never had one.
- 40: emptied by #798 (Sun and moon).
- 60: emptied by #798 (Timeless body). Capstone.
- (12 lost Weather sense to #780 but Call lightning holds it.)

### Monk: 10 abilities, 0 gaps
1 Martial arts, Trained wind; 6 Fleet; 12 Deflect missiles; 18 Flurry; 25 Still mind; 32 Purity of body; 40 Diamond soul; 50 Quivering palm; 60 Perfect self. (#798 removed Discipline at 12.)

### Paladin: 10 abilities, 2 gaps
1 Lay on hands, Trained wind; 6 Divine grace, Blessing (spell); 12 Smite, Consecrate; 18 Purity of body; 40 Holy radiance; 50 Bulwark; 60 Avatar.
- 25: emptied by #798 (Shield of faith). Direction: shield.
- 32: emptied by #798 (Steadfast).

### Ranger: 9 abilities, 3 gaps (6 of 9 standard levels, plus 2)
1 Favoured prey, Trained wind; 2 Stalker's eye; 6 Tracker, Barkskin (spell); 12 Rapid shot; 32 Evasion; 50 Volley; 60 Master hunter.
- 18: emptied by #801 (Woodland stride, outdoors only).
- 25: emptied by #798 (Swift tracker).
- 40: emptied by #798 (Camouflage). Direction: stealth.

### Thief: 10 abilities, 0 gaps
1 Sneak attack, Trained wind; 6 Lurk; 12 Low blow; 18 Evasion; 25 Uncanny dodge; 32 Improved sneak attack; 40 Vanishing act; 50 Hide in plain sight; 60 Master thief.

### Wizard: 13 abilities, 1 gap (8 of 9 standard levels, plus 3, 4 and 45)
1 Scroll adept, Ray of frost (spell), Arcane training; 3 Magic missile (spell); 4 Knock; 6 Arcane lore; 12 Burning hands (spell); 18 Arcane bolt (spell); 25 Mage armour; 32 Empower; 45 Magister; 50 Arcane mastery; 60 Archmage.
- 40: never had one (Magister sits at 45 instead; may be intended).
- (12 lost Spell recall to #782; Burning hands holds it.)

### Witch: 10 abilities, 2 gaps (7 of 9 standard levels, plus 3)
1 Herb lore, Evil eye (spell); 3 Hex of weakness (spell); 12 Poison lore, Cause wounds (spell); 25 Second sight; 32 Curse of sloth (spell); 40 Blight (spell); 50 Cauldron; 60 Hag's bargain.
- 6: never had one (Hex of weakness sits at 3).
- 18: never had one.
- (12 lost Cauldron-wise to #798; two entries still hold it.)

### Warrior: 11 abilities, 0 gaps
1 Weapon drill, Trained wind; 6 Shield line; 12 Veteran's wind; 18 Cleaving strike; 25 Armour drill, Drillmaster; 32 Weapon mastery; 40 Iron stance; 50 Whirlwind; 60 Champion.

### Aristocrat: 4 abilities, 5 gaps (4 of 9 standard levels)
6 Fencing; 18 Riposte; 32 Iron will; 50 Peerless.
- 1: emptied by #798 (Well dressed). No signature at level 1.
- 12: emptied by #798 (Noblesse).
- 25: emptied by #780 (Connoisseur, a duplicate of Keen appraiser and Lore).
- 40: emptied by #798 (Commanding).
- 60: emptied by #798 (Legacy). Capstone.

### Commoner: 6 abilities, 3 gaps (6 of 9 standard levels)
1 Hard work; 12 Stubborn; 18 Big hands; 32 Second wind; 40 Salt of the earth; 60 Unlikely hero.
- 6: emptied by #780 (Hearty).
- 25: emptied by #798 (Thick skull).
- 50: emptied by #798 (Folk hero).

### Craftsperson: 3 abilities, 6 gaps (3 of 9 standard levels)
18 Sturdy work; 50 Masterwork; 60 Artisan.
- 1: emptied by #780 (Keen appraisal, a duplicate). No signature at level 1.
- 6: emptied by #798 (Whetstone). Direction: weapons.
- 12: emptied by #798 (Fletching). Direction: missiles.
- 25: emptied by #798 (Mining).
- 32: emptied by #798 (Tinker).
- 40: emptied by #780 (Jeweller).

### Merchant: 3 abilities, 6 gaps (3 of 9 standard levels)
1 Haggler; 6 Deep pockets; 12 Eye for quality.
- 18: emptied by #798 (Trade routes).
- 25: emptied by #798 (Credit).
- 32: emptied by #780 (Appraiser, a duplicate).
- 40: emptied by #799 (Insured).
- 50: emptied by #798 (Magnate).
- 60: emptied by #798 (Golden tongue). Capstone.

## Summary

37 gaps across 11 classes; barbarian, monk, thief and warrior have none.

| Class | Abilities now | Gaps | Gap levels | Most urgent |
|---|---|---|---|---|
| Barbarian | 10 | 0 | none | none |
| Bard | 9 | 2 | 40, 50 | 50 |
| Priest | 10 | 2 | 6, 18 | 6 (early) |
| Druid | 6 | 5 | 6, 25, 32, 40, 60 | 6, then the 60 capstone |
| Monk | 10 | 0 | none | none |
| Paladin | 10 | 2 | 25, 32 | 25 |
| Ranger | 9 | 3 | 18, 25, 40 | 18 |
| Thief | 10 | 0 | none | none |
| Wizard | 13 | 1 | 40 (45 holds Magister) | low |
| Witch | 10 | 2 | 6, 18 | 6 (early) |
| Warrior | 11 | 0 | none | none |
| Aristocrat | 4 | 5 | 1, 12, 25, 40, 60 | 1 (no signature), then 60 |
| Commoner | 6 | 3 | 6, 25, 50 | 6 |
| Craftsperson | 3 | 6 | 1, 6, 12, 25, 32, 40 | 1 (no signature), 6, 12 |
| Merchant | 3 | 6 | 18, 25, 32, 40, 50, 60 | 18, then 50 and 60 |

Ordered by urgency:
- **Level 1 (no signature):** aristocrat, craftsperson.
- **Early (6 and 12):** craftsperson 6 and 12, druid 6, priest 6, witch 6, commoner 6, aristocrat 12.
- **Capstones (50 and 60) missing:** aristocrat 60, bard 50, commoner 50, druid 60, merchant 50 and 60. Craftsperson keeps both.
- **Thinnest tables:** craftsperson (3 of 9), merchant (3 of 9), aristocrat and druid (4 of 9).
