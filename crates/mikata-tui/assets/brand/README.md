# Mikata's logo, embedded

`mikata-icon.png` is the 256 px icon from the repository's `assets/brand/mikata-icon@256.png`
(rendered from `assets/source/mikata-icon.svg`): two cells mid-division, gold for you and cyan for
Mikata, glowing on their tile. `src/kit/pixels.rs` embeds it as `LOGO`; the sidebar's head shows it
as real pixels where the terminal draws images, and shows no logo where it does not.
