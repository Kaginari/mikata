//! Cards for Ratatui, drawn in the cell grid. A card is a solid block: a thick bar down its left
//! in the hue that says what it is (the human's gold, the answer's cyan, a tool's class), a face
//! tinted a few percent toward that hue filling it to its right edge, a lighter band for its head
//! rows, and a half-row lip on top that parts it from the card above — no border and no blank
//! line: cards stack butted together. A float (a toast, a dialog) keeps a rounded border in its
//! hue, so it reads as over the page.
//!
//! ```text
//! ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄       a float:  ╭──────────────────╮
//! ▌ ● bash  pytest                     │ ● bash  pytest   │
//! ▌   ⎿ ok · 2.0s                      ╰──────────────────╯
//! ```
//!
//! Every tone is a step of one scale taken from the theme's own palette, so a slab re-renders in
//! the other variant by rendering again.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use super::text::{Align, Lines, fit};
use crate::theme::{Rgb, Theme, Variant};

/// The cells a card's frame takes from its width: a border and a space each side.
pub const FRAME: usize = 4;

/// One surface's tones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Surface {
    /// The face the content sits on.
    pub face: Rgb,
    /// The header strip: the face a breath lit, and a breath more of the hue.
    pub band: Rgb,
    /// The lit top edge.
    pub gloss: Rgb,
    /// The shaded right and bottom bevel.
    pub shade: Rgb,
    /// The shadow on the plane, and its far, fading column.
    pub shadow: (Rgb, Rgb),
    /// The spine down the left edge, in the surface's hue.
    pub spine: Option<Rgb>,
    /// The page under it, for a solid card's lip.
    pub plane: Rgb,
    /// A solid card (bar, face, lip), else a framed float.
    pub solid: bool,
}

/// The tonal scale of a theme: the plane under everything, a card on it, a float over it, and
/// the tones a lit edge and a shaded one move toward.
#[derive(Debug, Clone, Copy)]
struct Scale {
    plane: Rgb,
    card: Rgb,
    float: Rgb,
    lift: Rgb,
    sink: Rgb,
    dark: bool,
}

impl Scale {
    fn of(t: &Theme) -> Self {
        let p = t.p;
        match t.variant {
            Variant::Dark => Self {
                plane: p.bg,
                card: p.panel.mix(p.panel_raised, 0.7),
                float: p.panel_raised.mix(p.line, 0.35),
                lift: p.text,
                sink: Rgb(0, 0, 0),
                dark: true,
            },
            Variant::Light => Self {
                plane: p.bg,
                card: p.panel,
                float: p.panel,
                lift: Rgb(255, 255, 255),
                sink: p.text_muted,
                dark: false,
            },
        }
    }
}

impl Surface {
    /// A card raised on the page, tinted `k` toward `hue`, its spine in `hue`.
    #[must_use]
    pub fn card(t: &Theme, hue: Option<Rgb>, k: f32) -> Self {
        let s = Scale::of(t);
        Self::raise(&s, s.card, hue, k, false)
    }

    /// A float over the frame: the deepest shadow.
    #[must_use]
    pub fn float(t: &Theme, hue: Option<Rgb>, k: f32) -> Self {
        let s = Scale::of(t);
        Self::raise(&s, s.float, hue, k, true)
    }

    fn raise(s: &Scale, base: Rgb, hue: Option<Rgb>, k: f32, deep: bool) -> Self {
        let face = hue.map_or(base, |h| base.mix(h, k));
        let (mut band, mut gloss, shade, shadow) = if s.dark {
            (
                face.mix(s.lift, 0.05),
                face.mix(s.lift, 0.3),
                face.mix(s.sink, 0.42),
                s.plane.mix(s.sink, if deep { 0.95 } else { 0.78 }),
            )
        } else {
            (
                face.mix(s.plane, 0.45),
                face.mix(s.sink, 0.12),
                face.mix(s.sink, 0.3),
                s.plane.mix(s.sink, if deep { 0.34 } else { 0.22 }),
            )
        };
        if let Some(h) = hue {
            band = band.mix(h, k * 0.6);
            gloss = gloss.mix(h, k * 1.2);
        }
        Self {
            face,
            band,
            gloss,
            shade,
            shadow: (shadow, shadow.mix(s.plane, 0.5)),
            spine: hue,
            plane: s.plane,
            solid: !deep,
        }
    }
}

/// A slab: a surface, which of its edges it draws, and how many rows of its top are the band.
/// A slab whose top or bottom is open continues another: an answer streamed in pieces reads as
/// one panel.
#[derive(Debug, Clone, Copy)]
pub struct Slab {
    /// The tones.
    pub surface: Surface,
    /// Draw the lit top edge.
    pub top: bool,
    /// Draw the bottom bevel.
    pub bottom: bool,
    /// The first rows on the band's tone.
    pub band: usize,
}

impl Slab {
    /// A whole slab of `surface`, no band.
    #[must_use]
    pub fn new(surface: Surface) -> Self {
        Self {
            surface,
            top: true,
            bottom: true,
            band: 0,
        }
    }

    /// The first `rows` on the band.
    #[must_use]
    pub fn band(mut self, rows: usize) -> Self {
        self.band = rows;
        self
    }

    /// Which edges it draws.
    #[must_use]
    pub fn edges(mut self, top: bool, bottom: bool) -> Self {
        self.top = top;
        self.bottom = bottom;
        self
    }

    /// The content's width inside a slab `width` cells wide.
    #[must_use]
    pub fn inner(width: usize) -> usize {
        width.saturating_sub(FRAME)
    }

    /// `rows` in the slab's card, `width` cells wide: a solid card — its lip, then each row behind
    /// its bar on the face — or, for a float, a rounded border in its hue (the rule's tone where
    /// it has none) and the face inside.
    #[must_use]
    pub fn render(&self, rows: Lines, width: usize) -> Lines {
        if self.surface.solid {
            return self.solid(rows, width);
        }
        let s = &self.surface;
        let bw = width.max(4);
        let inner = bw - 4;
        let edge = s.spine.unwrap_or_else(|| s.shade.mix(s.gloss, 0.5));
        let line = Style::new().fg(edge.color());
        let mut out: Lines = Vec::with_capacity(rows.len() + 2);
        if self.top {
            out.push(Line::from(Span::styled(
                format!("╭{}╮", "─".repeat(bw - 2)),
                line,
            )));
        }
        for (i, row) in rows.into_iter().enumerate() {
            let face = if i < self.band { s.band } else { s.face };
            let bg = Style::new().bg(face.color());
            let side = Style::new().fg(edge.color());
            let mut spans = vec![Span::styled("│", side), Span::styled(" ", bg)];
            let row = Line::from(
                row.spans
                    .into_iter()
                    .map(|sp| {
                        let style = if sp.style.bg.is_none() {
                            sp.style.bg(face.color())
                        } else {
                            sp.style
                        };
                        Span::styled(sp.content, style)
                    })
                    .collect::<Vec<_>>(),
            );
            spans.extend(fit(row, inner, Align::Left, bg).spans);
            spans.push(Span::styled(" ", bg));
            spans.push(Span::styled("│", side));
            out.push(Line::from(spans));
        }
        if self.bottom {
            out.push(Line::from(Span::styled(
                format!("╰{}╯", "─".repeat(bw - 2)),
                line,
            )));
        }
        out
    }

    /// A solid card: a lip of half blocks when it opens (a card it continues has none), then each
    /// row behind a thick bar in its hue, on its face to the right edge.
    fn solid(&self, rows: Lines, width: usize) -> Lines {
        let s = &self.surface;
        let bw = width.max(4);
        let inner = bw - 4;
        let edge = s.spine.unwrap_or_else(|| s.shade.mix(s.gloss, 0.5));
        let mut out: Lines = Vec::with_capacity(rows.len() + 1);
        if self.top {
            let first = if self.band > 0 { s.band } else { s.face };
            out.push(Line::from(vec![
                Span::styled("▄", Style::new().fg(edge.color()).bg(s.plane.color())),
                Span::styled(
                    "▄".repeat(bw - 1),
                    Style::new().fg(first.color()).bg(s.plane.color()),
                ),
            ]));
        }
        for (i, row) in rows.into_iter().enumerate() {
            let face = if i < self.band { s.band } else { s.face };
            let bg = Style::new().bg(face.color());
            let mut spans = vec![
                Span::styled("▌", Style::new().fg(edge.color()).bg(face.color())),
                Span::styled(" ", bg),
            ];
            let row = Line::from(
                row.spans
                    .into_iter()
                    .map(|sp| {
                        let style = if sp.style.bg.is_none() {
                            sp.style.bg(face.color())
                        } else {
                            sp.style
                        };
                        Span::styled(sp.content, style)
                    })
                    .collect::<Vec<_>>(),
            );
            spans.extend(fit(row, inner, Align::Left, bg).spans);
            spans.push(Span::styled("  ", bg));
            out.push(Line::from(spans));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::text::{line_cells, plain};
    use crate::theme::Variant;

    #[test]
    fn a_slab_keeps_its_width_and_casts_its_shadow() {
        let t = Theme::new(Variant::Dark);
        let s = Surface::card(&t, Some(t.races.summoner), 0.1);
        let lines = Slab::new(s)
            .band(1)
            .render(vec![Line::from("head"), Line::from("body")], 20);
        // A solid card: its lip, then its rows behind the bar — no border, no bottom row.
        assert_eq!(lines.len(), 3);
        for l in &lines {
            assert_eq!(line_cells(l), 20, "{:?}", plain(l));
        }
        assert_eq!(plain(&lines[0]), "▄".repeat(20));
        assert!(plain(&lines[1]).starts_with("▌ head"));
        assert!(plain(&lines[2]).starts_with("▌ body"));
        // The band is a breath lighter than the face; the bar wears the hue; the lip is the
        // band's tone over the page.
        assert_ne!(s.band, s.face);
        assert_eq!(lines[1].spans[0].style.fg, Some(t.races.summoner.color()));
        assert_eq!(lines[0].spans[1].style.fg, Some(s.band.color()));
        assert_eq!(lines[0].spans[1].style.bg, Some(s.plane.color()));
        assert_eq!(lines[2].spans[1].style.bg, Some(s.face.color()));
    }

    #[test]
    fn a_float_keeps_its_frame() {
        let t = Theme::new(Variant::Dark);
        let lines = Slab::new(Surface::float(&t, Some(t.p.holo), 0.1))
            .render(vec![Line::from("toast")], 16);
        assert_eq!(lines.len(), 3);
        assert!(plain(&lines[0]).starts_with("╭──"));
        assert!(plain(&lines[1]).starts_with("│ toast"));
        assert!(plain(&lines[2]).starts_with("╰─"));
    }

    #[test]
    fn a_solid_card_continued_has_no_lip() {
        let t = Theme::new(Variant::Light);
        let lines = Slab::new(Surface::card(&t, None, 0.0))
            .edges(false, false)
            .render(vec![Line::from("more")], 16);
        assert_eq!(lines.len(), 1);
        assert!(plain(&lines[0]).starts_with("▌ more"));
    }

    #[test]
    fn an_open_slab_continues_another() {
        let t = Theme::new(Variant::Light);
        let lines = Slab::new(Surface::float(&t, None, 0.0))
            .edges(false, false)
            .render(vec![Line::from("more")], 16);
        assert_eq!(lines.len(), 1);
        assert!(plain(&lines[0]).starts_with("│ more"));
    }
}
