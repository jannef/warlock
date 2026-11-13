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
  topics on the town, the surface, and the deep; Factor Wulfrun's letter (`gnome_letter`, #1435)
  is delivered to him, and he gives its answer back (`gnome_reply`; `docs/dev/campaign.md`).
- `deep_gnome_priest`: Mother Senara (`senara`) of the Glimmering, the town's temple (a sanctuary
  room): `cure` (60) and `mend` (40), `temple = true`.
- `deep_gnome_jeweller`: Old Tregear (`tregear`) of the cutting floor: teaches Jewelcraft for 1500
  gold (`train_skill`, at 5).
- `deep_gnome_trader`: the keepers, faction `merchants`, level 18 (a keeper must hold its own at
  danger 14 to 24, as the duergar trader does at Fyrdelf's): Kerensa of Kerensa's Stones
  (`gem_dealer`, `npc = "kerensa"`, also `identify` for 100), Jory of Jory's Outfitters
  (`outfitter`; a level `[[shop]] creature`), and the generated stores on Dēopgang's gallery
  floors (the branch's `shops` table with `keeper = "deep_gnome_trader"`).

- `deep_gnome_binder`: Ysella the Binder (`ysella`, #1446) in the garden at the town's east
  edge: topics on binding and Meyn Bew, and the lure quest `crystal_guardian`
  (`docs/dev/quests.md` "Lure quests"), which opens the crystal portal beside her.

## The crystal guardian

`crystal_guardian` (#1446, campaign-owned, `unique`): a crystalline earth elemental of level 26
(tags `elemental`, `earth`; faction `elemental`), slow (850) so it can be led, not incorporeal
(living crystal, not bedrock), with a staggering fist and thrown crystal shards like the earth
line's stones. In no population: only the lure quest makes it, on the crystal hollow at the bottom
of Meyn Bew, and anew there after a death while the quest is active. Its dialogue has no words:
one-liners and a `hums` mutter of notes. Bound, it is of the `peaceful` faction, carries `bound`
(its circle), and stands in the corner of Kerensa's Stones; Kerensa and Ysella then have lines
about it (`[[dialogue.after]]`).

## Tests

`world/deep_gnome_tests.rs`: the town's people are all of the tag, none hostile, and the four
named ones are there; Dēopgang 11's `>` leads to the town and back.
`world/crystal_guardian_tests.rs`: the lure quest, the portal, the guardian's following, the
binding, the boon, and its re-forming.
