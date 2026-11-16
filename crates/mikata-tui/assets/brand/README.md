# Mikata's logo, embedded

- `mikata-icon.png` — the 256 px icon from the repository's `assets/brand/mikata-icon@256.png`
  (rendered from `assets/source/mikata-icon.svg`): two cells mid-division, gold for you and cyan for
  Mikata, glowing on their tile. `src/kit/pixels.rs` embeds it as `LOGO`; the sidebar's head shows
  it as real pixels where the terminal draws images.
- `mikata-mark-{half,quadrants,sextants,octants}.png` — the mark flat, without its glow, from
  `assets/source/mikata-mark-cells.svg`, rendered at the exact sub-pixels of each glyph set for 32
  columns by 10 rows (32 × 20, 64 × 20, 64 × 30, 64 × 40). `pixels::mark` picks one; the sidebar
  draws it in glyph cells, every sub-pixel its own, with no resize: a glowing picture shrunk to 32
  sub-pixels across is a blur.
