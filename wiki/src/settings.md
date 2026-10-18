# Settings

Open **Settings** from the main menu or the pause menu (`Esc`). Up and Down choose a row, and
Left and Right change it; `Enter` also turns a switch on or off and picks **Reset to defaults**.
Changes apply at once and are saved at once, and nothing you change here affects the game
itself. Key bindings are changed in the Controls
screen instead; see [Controls](controls.md#rebinding-keys).

| Setting | What it does |
|---|---|
| Font size | The size of the glyphs, from 12 to 36 (18 to start with). Larger glyphs show less of the map. |
| Reduced effects | Turns every effect below off at once, whatever its own switch says. |
| Glyph animation | Water waves and flows, the altar pulses, and a sound you hear ripples in on the map. |
| Light flicker | Firelight flickers and drifts in color. Off, every light is steady. |
| Slides, camera | Creatures slide from cell to cell, attackers lunge, and the view follows you smoothly. Off, they jump. |
| Bloom and glow | A soft glow around glowing glyphs such as lava and crystals. |
| Heat haze | The air shimmers above lava and fires. |
| Particles | Sparks, embers, splashes, dust, foam, arrows in flight, and the sparkle of a spell. |
| Screen fades | The fade in from black after you change level. |
| Weather | Rain falling, splashing, and ringing on water. |
| Color palette | The colors: Default, High contrast, or Colorblind (red-green). See below. |
| Reset to defaults | Puts every setting above back to its default. Your keys stay as they are. |

## Effects never hide anything

Every effect is only for show, and no information depends on one:

- A sound you hear is marked on the map whether or not it ripples in (see [Hearing](hearing.md)).
- With the Weather switch off, a rainy day is still as dark as it is; only the falling drops are
  gone (see [Weather](weather.md)).
- With Light flicker off, lights are steady at the brightness the rules use for them.
- Creature shadows are always drawn, and they never change what you or anyone else can see (see
  [Light](light.md#what-is-only-for-show)).

## Palettes

A palette changes how colors are drawn on the screen, not the colors of the world: a red crystal
is still red light by the rules, whatever the palette shows. The menus keep their text readable
in every palette, and never tell you something by color alone: the selected row is marked with
`>`, and a refused choice starts with `!`.

## Where settings are kept

On the desktop your settings, key bindings included, are the file `settings.toml` in the game's
configuration folder; in the browser they are kept with the page (see
[Saving](saving.md#where-saves-are-kept)). A damaged settings file is not fatal: the game uses the
default for anything it cannot read.
