# Fonts

- `JuliaMono-Medium.ttf`: the game's font (the map, the creatures, the UI), SIL Open Font
  License 1.1 (`JuliaMono-OFL.txt`).
- `NotoSansJP-Runes.ttf`: the spell circles' runes (spec §4.8 "Spell circles",
  `assets/data/runes.toml`), a subset of Noto Sans JP Medium holding the runes' characters and
  nothing else, SIL Open Font License 1.1 (`NotoSansJP-OFL.txt`). Rasterised by
  `crates/warlock/src/render/rune_atlas.rs`, whose test checks that every rune of `runes.toml`
  has a glyph in it.

## Making the rune subset

From the Google Fonts release of Noto Sans JP
(`https://github.com/google/fonts/raw/main/ofl/notosansjp/NotoSansJP%5Bwght%5D.ttf`, a variable
font; its `OFL.txt` beside it is `NotoSansJP-OFL.txt`), with fontTools
(`pip install fonttools`):

```sh
fonttools varLib.instancer 'NotoSansJP[wght].ttf' wght=500 -o NotoSansJP-Medium.ttf
pyftsubset NotoSansJP-Medium.ttf --text="ヒハ木シ目小止ネ井" \
  --output-file=assets/fonts/NotoSansJP-Runes.ttf
```

A new rune's PR adds its character to `--text` and runs both again.
