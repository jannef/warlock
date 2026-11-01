# Settings

Open **Settings** from the main menu or the pause menu (`Esc`). Up and Down choose a row, and
Left and Right change it; `Enter` also turns a switch on or off and picks **Reset to defaults**.
Changes apply at once and are saved at once, and nothing you change here affects the game
itself. Key bindings are changed in the Controls
screen instead; see [Controls](controls.md#rebinding-keys).

| Setting | What it does |
|---|---|
| Font size | The size of the glyphs, from 12 to 36 (18 to start with). Larger glyphs show less of the map. |
| Reduced effects | Turns every effect below but Spell circles and Blood off at once, whatever its own switch says. |
| Glyph animation | Water waves and flows, the altar pulses, and a [sound you hear](hearing.md) ripples in on the map. |
| Light flicker | Firelight flickers and drifts in color. Off, every [light](light.md#what-light-is) is steady. |
| Slides, camera | Creatures slide from cell to cell, attackers lunge, the view follows you smoothly, and a [fireball](magic.md#spell-fireball) or a [lightning bolt](magic.md#spell-lightning-bolt) plays out. Off, they jump, and a [spell](magic.md) is a brief still flash of the cells it hits. |
| Bloom and glow | A soft glow around glowing glyphs such as [lava](terrain.md#terrain-lava) and [crystals](terrain.md#terrain-crystal). |
| Heat haze | The air shimmers above [lava](terrain.md#terrain-lava) and fires. |
| Particles | Sparks, embers, splashes, dust, foam, [arrows in flight](ranged.md#where-a-missile-goes), and the sparkle of a [spell](magic.md). |
| Screen fades | The fade in from black after you change level, and the rest of the view darkening for a moment while a [fireball](magic.md#spell-fireball) or a [lightning bolt](magic.md#spell-lightning-bolt) flares. |
| Screen shake | The view shakes for a moment when a [fireball](magic.md#spell-fireball) or a [lightning bolt](magic.md#spell-lightning-bolt) goes off near you; the nearer, the harder. |
| Weather | [Rain](weather.md#how-the-weather-changes) falling, splashing, and ringing on water. |
| Spell circles | The turning circles of runes about everyone who has a [buff or a hex](magic.md#spell-circles) on them. Reduced effects only stills them, as Slides, camera off does: they say something. Off, they are not drawn; [look mode](controls.md#look-mode) still says what each creature suffers. |
| Blood | The stains big hits and kills leave on floors and walls (see [Combat](combat.md#blood)). Not one of the effects: Reduced effects leaves it as it is. |
| Color palette | The colors: Default, High contrast, or Colorblind (red-green). See below. |
| Clock | How the [time of day](time.md#days) is written: 24-hour (`14:32`, the default) or 12-hour (`2:32 PM`). It changes the [status bar](controls.md#the-screen) and the death screen; the game itself keeps one clock. |
| Temperature | The unit the [status bar](controls.md#the-screen) shows the [temperature](temperature.md) in: °C (Celsius, to start with) or °F (Fahrenheit). See [Temperature and ice](temperature.md). |
| Where are my files? | Opens a screen showing where your settings, save, and logs are, and why, with **Open folder** for each. See [Saving](saving.md#portable-mode). |
| Reset to defaults | Puts every setting above back to its default. Your [keys](controls.md#rebinding-keys) stay as they are. |

## Effects never hide anything

Every effect is only for show, and no information depends on one:

- A sound you hear is marked on the map whether or not it ripples in (see [Hearing](hearing.md)).
- With the Weather switch off, a [rainy](weather.md#how-the-weather-changes) day is still as dark as it is; only the falling drops are
  gone (see [Weather](weather.md)).
- With Light flicker off, [lights](light.md#what-light-is) are steady at the brightness the rules use for them.
- A [fireball](magic.md#spell-fireball) or a [lightning bolt](magic.md#spell-lightning-bolt) always shows the cells it hits, if only as a brief still flash;
  its light, the darkening, and the shake are only for show.
- Creature shadows are always drawn, and they never change what you or anyone else can see (see
  [Light](light.md#what-is-only-for-show)).
- With Spell circles off, nobody's [buffs and hexes](magic.md#spell-circles) are hidden: the
  [status bar](controls.md#the-screen) shows yours and [look mode](controls.md#look-mode) names a creature's.
- Blood is only for show: with the Blood switch off, nothing about a fight changes, and [look
  mode](controls.md#look-mode) stops mentioning it.

## Palettes

A palette changes how colors are drawn on the screen, not the colors of the world: a red [crystal](terrain.md#terrain-crystal)
is still red [light](light.md#what-light-is) by the rules, whatever the palette shows. The menus keep their text readable
in every palette, and never tell you something by color alone: the selected row is marked with
`>`, and a refused choice starts with `!`.

## Where settings are kept

On the desktop your settings, [key bindings](controls.md#rebinding-keys) included, are the file `settings.toml` in the game's
configuration folder; in the browser they are kept with the page (see
[Saving](saving.md#where-saves-are-kept)). A damaged settings file is not fatal: the game uses the
default for anything it cannot read.
