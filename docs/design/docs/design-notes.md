# Konco Nembang: design notes (v1, direction 1a “Jernih”)

## Files
- `tokens/kn-tokens.css`: all `--kn-*` tokens: palette, type, spacing, radius, motion, overlay geometry, 4 presets (`[data-kn-preset]`), light/dark window themes (`[data-kn-theme]`). Existing names used by `OverlayApp.styleVars` (`--kn-scale`, `--kn-align`, `--kn-text-align`, `--kn-text-opacity`, `--kn-bg-opacity`, `--kn-active-color`) are kept.
- `tokens/overlay.css`: drop-in replacement for `src/overlay/overlay.css`, same class names plus `.kn-chip`, `.kn-dots`, `.kn-progress`, `.kn-handle`.
- `Overlay.dc.html`: every overlay state × 4 backgrounds × 4 presets, plus the component spec.
- `Cari Lirik.dc.html`, `Onboarding.dc.html`: light + dark.
- `Brand.dc.html` and `assets/`: logo, icon masters (256 / 32 / 16), PNG 16–512, `icon.ico`, tray SVG + PNG (white/black × normal/hidden/edit × 16/24/32), NSIS header/sidebar PNG.

## Key decisions

**Readability: stroke, not a stack of shadows.** The current 4-layer `text-shadow` gives a soft, uneven edge and costs one blur pass per layer. `-webkit-text-stroke` with `paint-order: stroke fill` paints a crisp, even outline under the glyph in one pass. A 5 px stroke gives a 2.5 px visible edge, and one tight 3 px shadow adds depth. This holds on white pages, busy game scenes and mixed video with no background box. No blur and no `backdrop-filter`.

**Hierarchy by size and tone, not opacity.** Dimming the context lines to 55% also dims their outline, and that is what made them disappear on busy backgrounds. The context lines now stay fully opaque at 20 px in a cool grey (`#D3D8DF`) against 34 px white. They look quieter but still have their full edge.

**Font.** Atkinson Hyperlegible Next (OFL, variable 200–800, extended Latin) was built to be read at a glance: Il1, O0 and rn/m are all distinct. One family covers lyrics and UI; the Mono cut is used for hotkeys and timestamps. Noto Sans JP/KR/SC are subset fallbacks, loaded through `unicode-range` only when a lyric needs them.

**Default preset: Minimal.** It passed all four test backgrounds, covers the least of the game UI and is the cheapest to render. Pill is the recommended switch for people who mostly read over bright pages. Karaoke carries the brand accent, and Subtle is meant for working.

**Pause: dim, then hide.** When paused, fade to 45% within 180 ms and show a “Dijeda” chip. Hide after 10 s (setting: *Sembunyikan otomatis saat pause*, default on, 10 s). Resume restores full opacity instantly. Dimming confirms the pause was noticed without the lyrics vanishing during a short pause. Hiding afterwards gets the overlay out of the way during long pauses in games.

**Long lines.** 1) Shrink the active line to 85%, never below 28 px. 2) If it is still too wide, wrap to at most 2 lines with `text-wrap: balance`. The active line is never truncated. Context lines always stay on one line with an ellipsis.

**Unsynced lyrics.** A “≈ Waktu perkiraan” chip above the lines; everything else behaves the same. The Mode Edit badge also shows “(unsynced)”.

**Accent.** Kuningan (bonang brass) `oklch(.84 .14 82)` / `#F8C153` on dark grounds and fills, with ink text on top. On light grounds, accent text uses Brass 600 `#9C681A` (4.9:1 on white).

**Motion.** Only `opacity` and `transform`. Line change 220 ms (stack translateY 8 → 0 plus active-line fade), state change 180 ms, easing `cubic-bezier(.2,.7,.2,1)`. With `.no-anim` or `prefers-reduced-motion`, everything swaps instantly.

## Tray icon
Monochrome. The app picks the white or black set from `HKCU\...\Themes\Personalize\SystemUsesLightTheme` and switches again when the theme changes. States: normal; hidden (the same glyph with a slash); Mode Edit (a smaller glyph inside corner brackets, matching the overlay's handles). `tray.rs` currently reuses the app icon, so pass the matching PNG to `TrayIconBuilder::icon` / `tray.set_icon`.

## Open items
- Settings window: not in this round.
- Toast/OSD: not in this round. Tokens are ready (`.toast` can reuse `.kn-chip`).
- NSIS needs BMP: convert the two PNGs in `assets/installer/` (`magick in.png -type truecolor BMP3:out.bmp`).
- The logo lockup SVGs use live text; outline them before using them outside the app.
