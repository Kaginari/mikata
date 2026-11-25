# Mikata's logo, embedded

`mikata-icon.png` is the 256 px icon from the repository's `assets/brand/mikata-icon@256.png`
(rendered from `assets/source/mikata-icon.svg` by `tools/brand.mjs`): a living cell flowing into a digital one — membrane
and nucleus, a cell becoming, a hexagon and its circuit — on its tile. `src/kit/pixels.rs` embeds it as `LOGO`; the sidebar's head shows it
as real pixels where the terminal draws images, and shows no logo where it does not.
