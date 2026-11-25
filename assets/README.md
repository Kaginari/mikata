# Assets

| Dir | Holds |
|---|---|
| `fonts/` | Cinzel and Rajdhani (SIL Open Font License, the texts beside them), for the cards |
| `portraits/` | one icon per race and ascended form: `<race>.png` (256 px), `<race>@64.png`, `<race>@32.png`, transparent |
| `cards/` | one card per race, ascended form and Player, plus the back — drawn by `tools/cards.py`; the board embeds the races' cards as WebP, cut by `tools/board-art.sh` into `packages/board/public/art/cards/` |
| `board/` | the board's screenshots, from its Playwright suite |
| `brand/` | Mikata's marks, the fusion cell (a living cell flowing into a digital one: a nucleus and its organelles, a hexagon with a circuit round its core): `mikata-logo-dark.png` and `mikata-logo-light.png` (the README, 512 px), `mikata-icon.png` and its sizes (the mark alone on its tile: 512, 256, 128, 64, 32), `favicon.ico` (32 and 64), `social-preview.png` (1280×640, GitHub's link card) |
| `source/` | `mikata-mark.png` (the mark alone: the owner's art, its cut-out fringe cleaned), `mikata-icon{,-small}.svg` (the small one, for 32 px and under, fills its tile), `mikata-logo-{dark,light}.svg` and `social-preview.svg`, which `node tools/brand.mjs` renders the brand files from; and the generated images the race crops were cut from |

Mnemorph's marks (the dragon and the crowned slime, the banner, the deck) are in
[`docs/experimental/assets/`](../docs/experimental/assets/), with the races they belong to.

Generated from image-model briefs; the cards' brief is [race-cards-prompt](../docs/experimental/race-cards-prompt.md). Missing: the Oni and Arch Dark Elf cards.
The source grids are low resolution (cards ≈ 245 × 395, icons ≈ 250 px): enough for the CLI and
the board's small sizes; the full-size card view waits for higher-resolution renders.

## The race icons

Cut to `portraits/` (256, 64 and 32 px, transparent) from the hexagons of
`source/race-icons.jpg`; Player wears the emblem's hexagon (`docs/experimental/assets/mnemorph-hex.png`).

| <img src="portraits/slime@64.png" width="64" alt="Slime"> | <img src="portraits/summoner@64.png" width="64" alt="Summoner"> | <img src="portraits/elf@64.png" width="64" alt="Elf"> | <img src="portraits/orc@64.png" width="64" alt="Orc"> | <img src="portraits/kijin@64.png" width="64" alt="Kijin"> | <img src="portraits/dark-elf@64.png" width="64" alt="Dark Elf"> |
|:---:|:---:|:---:|:---:|:---:|:---:|
| Slime | Summoner | Elf | Orc | Kijin | Dark Elf |

| <img src="portraits/player@64.png" width="64" alt="Player"> | <img src="portraits/high-elf@64.png" width="64" alt="High Elf"> | <img src="portraits/high-orc@64.png" width="64" alt="High Orc"> | <img src="portraits/named-slime@64.png" width="64" alt="Named Slime"> | <img src="portraits/oni@64.png" width="64" alt="Oni"> | <img src="portraits/arch-dark-elf@64.png" width="64" alt="Arch Dark Elf"> |
|:---:|:---:|:---:|:---:|:---:|:---:|
| Player | High Elf | High Orc | Named Slime | Oni | Arch Dark Elf |
