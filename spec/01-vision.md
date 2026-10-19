# Warlock specification — 1. Vision

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

## 1. Vision

Warlock is a turn-based, grid-based roguelike in the spirit of classic ASCII ADOM, rendered natively
on the GPU. Everything on screen — creatures, terrain, items, effects, UI — is a glyph. No tiles, no
real terminal.

It should feel far beyond the ASCII roguelikes of old: colored dynamic light with real shadows,
perception that depends on the viewer's eyes, living animated glyphs, and glyph-based particles and
shader effects — while every glyph stays readable.

Targets: native desktop on Windows, macOS, and Linux. A web build (WASM + WebGPU) is nice to have.
