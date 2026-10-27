# Orc roster

General conventions: `docs/dev/creatures.md`.

The orc tribe (#455): shared types, one file each in `assets/data/creatures/` (`orc_*`, `warg`,
`dire_warg`), faction `orcs` (inserted after `goblins`; listed in the four fixture factions files
too), tag `orc` on every orc (the wargs carry none, as the ogre carries no `goblin`), `archer` and
`caster` on the bowmen and shamans. Glyph `o`, the wargs `C`. The faction is hostile to the
player, the vermin, the kobolds, the undead, and the duergar, and neutral to the goblins.

## Roster

The orc runt 3 (small, a knife), the orc warrior 5 (racial scimitar, a jerkin, a cap, a shield now
and then), the orc archer 6 (a short bow and arrows, a scimitar in reserve), the warg 7 (quick, a
bite that may bleed), the orc raider 9 (quick, a battle axe), the orc shaman 10 (a caster with a
staff and a bone amulet, sometimes magic), the orc berserker 12 (quick for its bulk, unarmoured, a
great axe), the orc veteran 15 (scimitar, scale mail, helm, and shield), the orc bowman 17 (a
composite bow, studded leather), the orc war chief 20 (scimitar, mail and helm often magic, a
shield, coins), the dire warg 22 (very quick, thick hide, a deep-tearing bite), the orc high
shaman 24 (a caster, its amulet often magic), and the orc warlord 28 (very rare: banded mail, great
helm, large shield, a battle axe, gold).

## Tests

`content/orc_roster_tests.rs` holds the formulas (#455's issue): each `orcs` type has a role in
`ROLES`, is on the XP guideline, its St/Dx/To within 1 of `12 + 0.4L`, `10 + 0.25L`, `12 + 0.4L`
plus the role's adjustment, its max HP within ±35 % of the undead roster's shape, its racial
weapon at rank `1 + L/7`, and the wargs' bites on #383's dice; two types or more in each band of
3–7, 8–14, 15–21, and 22–30; the stances; and the gear rolls (armed nine times in ten, the bowmen
with arrows).

## Populations

Cealdhlāw: the runt, warrior, archer, and warg. Fyrdelf: the warg, raider, shaman, berserker,
veteran, and bowman (level 17: only out of depth, at the bottom). The war chief (20) left Fyrdelf
when its danger was squished to 5–16 (#534); it, the dire warg, high shaman, and warlord are in
none. Riders on `on_hit` fire only on natural attacks, so the armed orcs have
none; the wargs' bites carry `bleeding` (the mana tests' status fixture lists it for them).
