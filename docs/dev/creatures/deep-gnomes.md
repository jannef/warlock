# Deep gnomes

General conventions: `docs/dev/creatures.md`.

The deep gnomes (#1434) are the people of Gimmwīc, the town at the bottom of Dēopgang in the
Ascmearc's south-west corner (`docs/dev/campaign.md`). They are a race of the world but not a
playable one yet: no `data/races.toml` row (that file is New run's list and wants a kit, a
companion, a familiar, and names for each race), so their creatures carry no `race` field and are
known by the tag `deep-gnome` instead (the speech rule of `docs/dev/npcs.md` binds them by it).
They are never called by a borrowed name from another game.

The look: small, grey-skinned, bald folk, glyph `@` in pale greys and
violets (from `#9c9cb0` to `#e0b8e8`), apart from the surface gnomes. They see in the dark
(`vision = "darkvision"`), are `size = "small"`, `alignment = "good"`, and `grades = false` (each
is one of a people, never greater or lesser).

What they love: gemstones first, and making things. Their talk turns to the work (cutting,
setting, smithing), their town is built round its workshops (the cutting floor's three anvils,
braziers, and benches are free to use), and they would rather make a thing than buy it.

Their speech: the Cornish of the deep (`docs/dev/npcs.md` "Speech by race and station"), kept apart
from the Scouse of the surface gnomes.

## The types

All campaign-owned, in `assets/campaigns/ascmearc/data/creatures/` (`deep_gnome*`), tag
`deep-gnome`:

- `deep_gnome`: the townsfolk, peaceful; one-liners and mutters, no topics. One of them is
  Morwenna (`npc = "morwenna"`, #1428), the crystal-reader by the cutting floor's east brazier:
  her rotating lines hint, in words, what each magic crystal colour binds (the families of
  `data/crystals.toml`), what the heart-crystal and the soul-fetter do, and what consuming a
  crystal does (`docs/dev/enchanting.md`); no topics, no services. Her lines are written by hand:
  change them with the colours' families.
- `deep_gnome_headman`: Headman Peran (`npc = "peran"`), Gimmwīc's leader in its moot hall;
  topics on the town, the surface, and the deep (the letter quest of #1435 starts from him).
- `deep_gnome_priest`: Mother Senara (`senara`) of the Glimmering, the town's temple (a sanctuary
  room): `cure` (60) and `mend` (40), `temple = true`.
- `deep_gnome_jeweller`: Old Tregear (`tregear`) of the cutting floor: teaches Jewelcraft for 1500
  gold (`train_skill`, at 5).
- `deep_gnome_trader`: the keepers, faction `merchants`, level 18 (a keeper must hold its own at
  danger 14 to 24, as the duergar trader does at Fyrdelf's): Kerensa of Kerensa's Stones
  (`gem_dealer`, `npc = "kerensa"`, also `identify` for 100), Jory of Jory's Outfitters
  (`outfitter`; a level `[[shop]] creature`), and the generated stores on Dēopgang's gallery
  floors (the branch's `shops` table with `keeper = "deep_gnome_trader"`).

## Tests

`world/deep_gnome_tests.rs`: the town's people are all of the tag, none hostile, and the four
named ones are there; Dēopgang 11's `>` leads to the town and back.
