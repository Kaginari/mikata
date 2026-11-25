//! The welcome: the card the conversation opens on, above its first entry — one title line,
//! **◆ Mikata** in the logo's gold into cyan, the version and the tagline; what Mikata does in
//! four lines; the world, the model, the session, the memory and the verify line in capitals;
//! and the keys that matter first. No block letters: the sidebar's head shows the logo, and a
//! line of text reads the same in every terminal and font.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::app::Model;
use crate::kit::slab::{Slab, Surface};
use crate::kit::text::{Lines, caps, prefixed, truncate, wrap_text};
use crate::safe::one_line;
use crate::theme::{Rgb, Theme};

/// The tagline under the wordmark.
pub const TAGLINE: &str = "the coding agent on your side";

/// What Mikata does, a line each.
pub const ABOUT: [&str; 4] = [
    "One body takes your ask: it writes down what done means, then works until every line holds.",
    "Every change passes the end gate: your verify line runs green and no test is deleted or skipped.",
    "An ask that names several projects runs one body a project, at once, each in its own folder.",
    "Memory with provenance: a note is grounded by a passing check, your word, or a second voice.",
];

/// The logo's gradient at `along` (0 at the left, 1 at the right): gold, for you, into Mikata's
/// cyan, made readable on the theme's page.
#[must_use]
pub fn hue(t: &Theme, along: f32) -> Rgb {
    t.readable(t.p.gold.mix(t.p.holo, along.clamp(0.0, 1.0)))
}

/// Where column `x` of `width` lies along the gradient.
fn along(x: usize, width: usize) -> f32 {
    // Display only: a column's place across a line of a few dozen cells.
    #[allow(clippy::cast_precision_loss)]
    let f = x as f32 / width.saturating_sub(1).max(1) as f32;
    f
}

/// The title's mark and name, each character a step of the logo's gradient, bold.
pub const TITLE: &str = "◆ Mikata";

/// The title line: [`TITLE`] in the gradient, then the version and the tagline, muted. On a card
/// too narrow for both, the version and the tagline wrap under the title.
#[must_use]
pub fn title(t: &Theme, inner: usize) -> Lines {
    let chars: Vec<char> = TITLE.chars().collect();
    let width = chars.len();
    let mut spans: Vec<Span<'static>> = chars
        .into_iter()
        .enumerate()
        .map(|(x, c)| {
            Span::styled(
                c.to_string(),
                Style::new()
                    .fg(hue(t, along(x, width)).color())
                    .add_modifier(Modifier::BOLD),
            )
        })
        .collect();
    let after = format!("{} · {TAGLINE}", env!("CARGO_PKG_VERSION"));
    if width + 2 + after.chars().count() <= inner {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(after, t.muted()));
        return vec![Line::from(spans)];
    }
    let mut rows = vec![Line::from(spans)];
    rows.extend(wrap_text(&after, inner, t.muted()));
    rows
}

fn surface(t: &Theme) -> Surface {
    Surface::card(t, Some(t.p.holo), 0.08)
}

/// The welcome card, `w` cells wide.
#[must_use]
pub fn card(m: &Model, w: usize) -> Lines {
    let t = &m.theme;
    let inner = Slab::inner(w);
    let mut rows: Lines = title(t, inner);
    rows.push(Line::from(""));
    for (n, text) in ABOUT.into_iter().enumerate() {
        let tone = if n % 2 == 0 { t.p.holo } else { t.p.gold };
        let bullet = Span::styled("▸ ", Style::new().fg(t.readable(tone).color()));
        let lines = wrap_text(text, inner.saturating_sub(2), t.muted());
        rows.extend(prefixed(lines, &[bullet], &[Span::raw("  ")]));
    }
    rows.push(Line::from(""));
    let field = |name: &str, value: &str| {
        truncate(
            Line::from(vec![
                label(t, name),
                Span::styled(one_line(value), t.text()),
            ]),
            inner,
        )
    };
    rows.push(field("world", &m.header.world));
    rows.push(field("model", &m.header.model));
    if !m.session_id.is_empty() {
        rows.push(field("session", &m.session_id));
    }
    rows.extend(world(m, inner));
    rows.push(Line::from(""));
    let mut keys = Vec::new();
    for (k, what) in [
        ("⏎", "ask"),
        ("/", "commands"),
        ("ctrl+o", "sidebar"),
        ("ctrl+t", "expand"),
        ("esc", "interrupt"),
    ] {
        keys.push(Span::styled(k, t.text().add_modifier(Modifier::BOLD)));
        keys.push(Span::styled(format!(" {what}   "), t.muted()));
    }
    rows.push(truncate(Line::from(keys), inner));
    Slab::new(surface(t)).render(rows, w)
}

/// A field's name in capitals, padded to the values' column.
fn label(t: &Theme, name: &str) -> Span<'static> {
    Span::styled(format!("{:<9}", caps(name)), t.muted())
}

/// The memory and verify rows, as the world's survey read them; "reading the world…" until it
/// has.
fn world(m: &Model, inner: usize) -> Lines {
    let t = &m.theme;
    let pending = || Span::styled("reading the world…", t.muted());
    let bold = |s: String| Span::styled(s, t.text().add_modifier(Modifier::BOLD));
    let mut rows = Vec::new();
    rows.push(Line::from(match &m.sidebar.memory {
        Some(st) => vec![
            label(t, "memory"),
            Span::styled("grounded ", t.muted()),
            bold(st.grounded.to_string()),
            Span::styled(" · shared ", t.muted()),
            bold(st.shared.to_string()),
        ],
        None => vec![label(t, "memory"), pending()],
    }));
    match &m.sidebar.verify {
        None => rows.push(Line::from(vec![label(t, "verify"), pending()])),
        Some(v) if !v.line.is_empty() => rows.push(Line::from(vec![
            label(t, "verify"),
            Span::styled(one_line(&v.line.join(" · ")), t.text()),
        ])),
        Some(v) => {
            rows.push(Line::from(vec![
                label(t, "verify"),
                Span::styled("none — memory can't be grounded", t.gold()),
            ]));
            let command = v
                .suggested
                .as_deref()
                .map_or_else(|| "<your test command>".to_owned(), one_line);
            rows.push(Line::from(vec![
                Span::raw(" ".repeat(9)),
                Span::styled("add to summoner's card  ", t.muted()),
                Span::styled(format!("- **Verify:** `{command}`"), t.text()),
            ]));
        }
    }
    rows.into_iter().map(|l| truncate(l, inner)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{Cmd, Header, Model, Msg, update, view};
    use crate::sidebar::{Survey, Verify};
    use crate::theme::Variant;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::style::Color;

    fn model(variant: Variant) -> Model {
        Model::new(
            Header {
                body: "summoner".into(),
                model: "deepseek-v4-flash".into(),
                world: "~/sqlglot".into(),
                mode: "light-coding".into(),
            },
            variant,
            (120, 50),
        )
    }

    fn surveyed(m: &mut Model, verify: Verify) {
        let Cmd::Survey(seq) = m.survey() else {
            unreachable!("a survey asks for a survey")
        };
        update(
            m,
            Msg::Surveyed {
                seq,
                survey: Survey {
                    verify,
                    ..Survey::default()
                },
            },
        );
    }

    fn screen(m: &mut Model, width: u16, height: u16) -> (String, Buffer) {
        update(m, Msg::Resize(width, height));
        let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
        term.draw(|f| view(m, f)).unwrap();
        let b = term.backend().buffer().clone();
        let text = (0..b.area.height)
            .map(|y| {
                (0..b.area.width)
                    .map(|x| b[(x, y)].symbol().to_owned())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        (text, b)
    }

    #[test]
    fn the_title_runs_the_logos_gradient_in_bold() {
        for variant in [Variant::Dark, Variant::Light] {
            let t = Theme::new(variant);
            let line = &title(&t, 200)[0];
            let fgs: Vec<Color> = line.spans[..TITLE.chars().count()]
                .iter()
                .filter(|s| s.content != " ")
                .filter_map(|s| s.style.fg)
                .collect();
            assert_eq!(
                fgs[0],
                hue(&t, 0.0).color(),
                "{variant:?}: gold first, for you"
            );
            assert_eq!(
                *fgs.last().unwrap(),
                hue(&t, 1.0).color(),
                "{variant:?}: Mikata's cyan last"
            );
            assert!(
                line.spans[..TITLE.chars().count()]
                    .iter()
                    .all(|s| s.style.add_modifier.contains(Modifier::BOLD))
            );
            let plain = crate::kit::text::plain(line);
            assert!(plain.starts_with(TITLE), "{plain}");
            assert!(plain.contains(env!("CARGO_PKG_VERSION")) && plain.contains(TAGLINE));
        }
        let t = Theme::new(Variant::Dark);
        assert_eq!(hue(&t, 0.0), t.readable(t.p.gold));
        assert_eq!(hue(&t, 1.0), t.readable(t.p.holo));
    }

    #[test]
    fn a_narrow_card_wraps_the_tagline_under_the_title() {
        let t = Theme::new(Variant::Dark);
        let one = title(&t, 120);
        assert_eq!(one.len(), 1);
        let narrow = title(&t, 24);
        assert!(narrow.len() > 1);
        assert_eq!(crate::kit::text::plain(&narrow[0]), TITLE);
        let rest: String = narrow[1..]
            .iter()
            .map(crate::kit::text::plain)
            .collect::<Vec<_>>()
            .join(" ");
        assert!(rest.contains("on your side"), "{rest}");
        for w in [40u16, 80, 120, 200] {
            let mut m = model(Variant::Dark);
            let (text, _) = screen(&mut m, w, 60);
            assert!(text.contains(TITLE), "{w} columns: {text}");
            assert!(!text.contains('█'), "{w}: no block letters");
        }
    }

    #[test]
    fn the_card_says_what_mikata_does() {
        for w in [80u16, 120, 200] {
            let mut m = model(Variant::Light);
            // The panels folded: on each row, the card's stretch is the second between borders.
            m.sidebar.open = false;
            let (text, _) = screen(&mut m, w, 60);
            // The card's rows joined, the frames and bullets cut away: wrapped lines read whole.
            let flat: String = text
                .lines()
                .map(|l| l.split('│').nth(2).unwrap_or(l))
                .map(|l| {
                    l.trim_matches(|c: char| " ▌▏▕▖▘▝▀▄│".contains(c))
                        .trim_start_matches("▸ ")
                        .trim()
                })
                .collect::<Vec<_>>()
                .join(" ");
            assert!(flat.contains(TAGLINE), "{w}: {text}");
            assert!(flat.contains(TITLE), "{w}: {text}");
            for about in ABOUT {
                assert!(flat.contains(about), "{w}: {about:?} in {text}");
            }
            for chip in [" elf ", " orc ", " slime "] {
                assert!(!text.contains(chip), "{w}: no race on the welcome: {chip}");
            }
            for label in ["world", "model", "memory", "verify"] {
                assert!(text.contains(&caps(label)), "{w}: {label}");
            }
            assert!(text.contains("ctrl+o sidebar"));
        }
    }

    #[test]
    fn verify_shows_the_line_or_the_suggestion() {
        let mut m = model(Variant::Dark);
        let (text, _) = screen(&mut m, 160, 60);
        assert!(text.contains("reading the world…"), "before the survey");
        surveyed(
            &mut m,
            Verify {
                line: Vec::new(),
                suggested: Some("cargo test --workspace".into()),
            },
        );
        let (text, _) = screen(&mut m, 160, 60);
        assert!(text.contains("none — memory can't be grounded"), "{text}");
        assert!(
            text.contains("add to summoner's card  - **Verify:** `cargo test --workspace`"),
            "{text}"
        );
        surveyed(
            &mut m,
            Verify {
                line: vec!["make check".into()],
                suggested: None,
            },
        );
        let (text, _) = screen(&mut m, 160, 60);
        assert!(text.contains(&format!("{:<9}make check", caps("verify"))));
        assert!(!text.contains("can't be grounded"));
    }

    /// The screen row the welcome's title is drawn on, if it is: below the header bar, which
    /// names Mikata too.
    fn wordmark_row(text: &str) -> Option<usize> {
        const BAR: usize = 3;
        text.lines()
            .skip(BAR)
            .position(|l| l.contains(TITLE))
            .map(|r| r + BAR)
    }

    /// The rows the welcome card draws above the title: its own top edge.
    fn above_wordmark(m: &Model) -> usize {
        card(m, 80)
            .iter()
            .position(|l| crate::kit::text::plain(l).contains(TITLE))
            .expect("the card holds the title")
    }

    fn tick(m: &mut Model) {
        update(m, Msg::Tick(std::time::Duration::from_millis(50)));
    }

    #[test]
    fn on_first_run_the_welcome_opens_the_conversation_at_the_top() {
        // The header bar's three rows, then the Agent loop panel's top edge: the transcript's
        // first row is 4.
        const TOP: usize = 4;
        for open in [true, false] {
            let mut m = model(Variant::Dark);
            m.sidebar.open = open;
            m.info("deepseek-v4-flash via deepseek · sandbox bwrap");
            let (text, _) = screen(&mut m, 120, 40);
            let row = wordmark_row(&text).expect("the title is drawn");
            assert_eq!(row, TOP + above_wordmark(&m), "open {open}:\n{text}");
            // The room the short transcript leaves is below it, above the input.
            let input = text
                .lines()
                .position(|l| l.contains("ask, or /help"))
                .expect("the input");
            assert!(
                input > row + 20,
                "open {open}: the input stays at the bottom:\n{text}"
            );
        }
    }

    #[test]
    fn the_welcome_scrolls_away_and_comes_back_at_the_top_of_the_history() {
        let mut m = model(Variant::Dark);
        let (text, _) = screen(&mut m, 120, 40);
        assert!(wordmark_row(&text).is_some());
        for i in 0..500 {
            m.info(format!("message {i}"));
        }
        let (text, _) = screen(&mut m, 120, 40);
        assert_eq!(wordmark_row(&text), None, "scrolled out:\n{text}");
        assert!(text.contains("message 499"), "the newest shows");
        // To the top of the history: the welcome first, where it opened.
        update(
            &mut m,
            Msg::Key(ratatui::crossterm::event::KeyEvent::new(
                ratatui::crossterm::event::KeyCode::Home,
                ratatui::crossterm::event::KeyModifiers::CONTROL,
            )),
        );
        for _ in 0..200 {
            let _ = screen(&mut m, 120, 40);
            tick(&mut m);
        }
        let (text, _) = screen(&mut m, 120, 40);
        assert_eq!(
            wordmark_row(&text),
            Some(4 + above_wordmark(&m)),
            "\n{text}"
        );
    }

    #[test]
    fn no_picture_is_wanted_for_the_welcome() {
        let mut m = model(Variant::Dark);
        m.pictures =
            crate::pictures::Pictures::new(crate::kit::pixels::Protocol::Kitty, Some((8, 17)));
        // The sidebar folded: whatever is wanted now could only be the welcome's.
        m.sidebar.open = false;
        let _ = screen(&mut m, 160, 56);
        assert_eq!(m.pictures.wanted(), Vec::new());
        m.sidebar.open = true;
        let _ = screen(&mut m, 160, 56);
        let wanted = m.pictures.wanted();
        assert_eq!(wanted.len(), 1, "the sidebar's alone");
        assert_eq!(wanted[0].slot, crate::pictures::Slot::Sidebar);
    }

    #[test]
    fn nothing_overflows_the_card() {
        for variant in [Variant::Dark, Variant::Light] {
            let mut m = model(variant);
            surveyed(
                &mut m,
                Verify {
                    line: Vec::new(),
                    suggested: Some("cargo test --workspace".into()),
                },
            );
            for w in [40usize, 58, 80, 120, 200] {
                for line in card(&m, w) {
                    assert!(
                        crate::kit::text::line_cells(&line) <= w,
                        "{variant:?} {w}: {:?}",
                        crate::kit::text::plain(&line)
                    );
                }
            }
        }
    }
}
