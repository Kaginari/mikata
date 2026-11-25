//! The chrome around the conversation: a header bar of chips, panels framed in the accent's
//! gradient (Done when, Budgets, Units, Session), and a footer bar. Every colour comes from the
//! theme: the gradient runs from the accent toward the human's colour, across the screen, so the
//! panels' borders read as one sweep of light.

use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::app::Model;
use crate::kit::text::{Lines, cells, truncate};
use crate::theme::{Rgb, Theme};

/// The gradient's colour at `x` of a screen `w` cells wide: the accent on the left, warming toward
/// the human's colour on the right, never all the way.
#[must_use]
pub fn sweep(t: &Theme, x: u16, w: u16) -> Rgb {
    let f = f32::from(x) / f32::from(w.max(1));
    t.p.holo.mix(t.p.gold, f * 0.55)
}

/// A panel: a rounded border in the gradient, its title bold on the top edge with a glyph before
/// it, a few slashes at its right end, and its inside on the panel's surface. Returns the inside.
pub fn panel(
    buf: &mut Buffer,
    area: Rect,
    glyph: &str,
    title: &str,
    t: &Theme,
    screen_w: u16,
) -> Rect {
    if area.width < 4 || area.height < 2 {
        return Rect::default();
    }
    let inside = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    let fill = Style::new().bg(t.p.panel.color()).fg(t.p.text.color());
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            buf[(x, y)].reset();
            buf[(x, y)].set_style(fill);
        }
    }
    let (x0, x1, y0, y1) = (area.x, area.right() - 1, area.y, area.bottom() - 1);
    let edge = |buf: &mut Buffer, x: u16, y: u16, s: &str| {
        let c = sweep(t, x, screen_w).mix(t.p.panel, 0.15);
        let cell = &mut buf[(x, y)];
        cell.set_symbol(s);
        cell.set_style(Style::new().fg(c.color()).bg(t.p.panel.color()));
    };
    for x in x0..=x1 {
        let (top, bottom) = match x {
            _ if x == x0 => ("╭", "╰"),
            _ if x == x1 => ("╮", "╯"),
            _ => ("─", "─"),
        };
        edge(buf, x, y0, top);
        edge(buf, x, y1, bottom);
    }
    for y in (y0 + 1)..y1 {
        edge(buf, x0, y, "│");
        edge(buf, x1, y, "│");
    }
    let hue = sweep(t, area.x + 2, screen_w);
    let head = Line::from(vec![
        Span::styled(" ", Style::new().bg(t.p.panel.color())),
        Span::styled(
            glyph.to_owned(),
            Style::new().fg(hue.color()).bg(t.p.panel.color()),
        ),
        Span::styled(" ", Style::new().bg(t.p.panel.color())),
        Span::styled(
            title.to_owned(),
            Style::new()
                .fg(t.p.text.color())
                .bg(t.p.panel.color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::new().bg(t.p.panel.color())),
    ]);
    let room = area.width.saturating_sub(8);
    if !title.is_empty() {
        buf.set_line(area.x + 2, area.y, &truncate(head, room.into()), room);
    }
    if area.width > 24 {
        let slashes = "╱╱╱╱";
        let at = area.right() - 7;
        let c = sweep(t, at, screen_w);
        buf.set_string(
            at,
            area.y,
            slashes,
            Style::new().fg(c.color()).bg(t.p.panel.color()),
        );
    }
    inside
}

/// A chip: a quiet label and its value, bold.
fn chip(t: &Theme, label: &str, value: &str, hot: Option<Rgb>) -> Vec<Span<'static>> {
    let mut v = vec![Span::styled(format!("{label} "), t.muted())];
    match hot {
        Some(c) => v.push(Span::styled(
            format!(" {value} "),
            Style::new()
                .fg(t.p.bg.color())
                .bg(c.color())
                .add_modifier(Modifier::BOLD),
        )),
        None => v.push(Span::styled(
            value.to_owned(),
            t.text().add_modifier(Modifier::BOLD),
        )),
    }
    v
}

/// The header bar's one line: Mikata's name, then the session's chips, the state at the right.
#[must_use]
pub fn header(m: &Model, w: usize, state: Line<'static>) -> Line<'static> {
    let t = &m.theme;
    let sep = || Span::styled("   ", t.muted());
    let mut spans = vec![
        Span::styled("◆ ", Style::new().fg(t.p.holo.color())),
        Span::styled(
            "Mikata",
            Style::new()
                .fg(t.p.holo.color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  ", t.muted()),
    ];
    let world = m
        .header
        .world
        .rsplit('/')
        .next()
        .unwrap_or(&m.header.world)
        .to_owned();
    spans.extend(chip(t, "repo", &world, None));
    if let Some(b) = &m.sidebar.branch {
        spans.push(sep());
        spans.extend(chip(t, "branch", b, None));
    }
    if !m.header.mode.is_empty() {
        spans.push(sep());
        spans.extend(chip(t, "mode", &m.header.mode, Some(t.p.gold)));
    }
    spans.push(sep());
    spans.extend(chip(t, "model", &m.header.model, None));
    if !m.sandbox.is_empty() {
        let on = !m.sandbox.starts_with("none");
        spans.push(sep());
        spans.extend(chip(
            t,
            "sandbox",
            if on { "on" } else { "off" },
            Some(if on {
                t.readable(t.races.elf)
            } else {
                t.p.alarm
            }),
        ));
    }
    let left = truncate(
        Line::from(spans),
        w.saturating_sub(crate::kit::text::line_cells(&state) + 2),
    );
    let pad = w
        .saturating_sub(crate::kit::text::line_cells(&left) + crate::kit::text::line_cells(&state));
    let mut all = left.spans;
    all.push(Span::raw(" ".repeat(pad)));
    all.extend(state.spans);
    Line::from(all)
}

/// The footer bar's line: the verify line and the last answer's word on the left, the steps, the
/// time and the spend as chips on the right, and between them what `keys` draws in the room left
/// (the keys that matter now).
#[must_use]
pub fn footer(m: &Model, w: usize, keys: impl Fn(usize) -> Line<'static>) -> Line<'static> {
    let t = &m.theme;
    let usage = &m.transcript.usage;
    let mut left = vec![Span::styled("› ", Style::new().fg(t.p.holo.color()))];
    match m.sidebar.verify.as_ref().filter(|v| !v.line.is_empty()) {
        Some(v) => {
            left.push(Span::styled("verify ", t.muted()));
            left.push(Span::styled(
                v.line.join(" && "),
                t.text().add_modifier(Modifier::BOLD),
            ));
        }
        None => left.push(Span::styled("no verify line pinned", t.muted())),
    }
    if let Some((word, passed)) = &m.last {
        left.push(Span::styled("   │   last answer ", t.muted()));
        let hue = if *passed { t.ok() } else { t.gold() };
        left.push(Span::styled(word.clone(), hue.add_modifier(Modifier::BOLD)));
    }
    let clock = elapsed(m);
    let mut right = vec![
        Span::styled("◈ steps ", t.muted()),
        Span::styled(
            usage.calls.to_string(),
            t.text().add_modifier(Modifier::BOLD),
        ),
        Span::styled("   ◷ time ", t.muted()),
        Span::styled(clock, t.text().add_modifier(Modifier::BOLD)),
        Span::styled("   $ cost ", t.muted()),
        Span::styled(cost(m), t.text().add_modifier(Modifier::BOLD)),
    ];
    // Narrow, the chips go first: the verify line is what the footer is for.
    if w < 60 {
        right.clear();
    }
    let left = truncate(Line::from(left), w.saturating_sub(cells_of(&right) + 2));
    let used = crate::kit::text::line_cells(&left) + cells_of(&right);
    let room = w.saturating_sub(used + 6);
    let middle = if room >= 12 {
        keys(room)
    } else {
        Line::default()
    };
    let mid_w = crate::kit::text::line_cells(&middle);
    let pad = w.saturating_sub(used + mid_w);
    let mut all = left.spans;
    if mid_w > 0 {
        let lead = pad / 2;
        all.push(Span::raw(" ".repeat(lead)));
        all.extend(middle.spans);
        all.push(Span::raw(" ".repeat(pad - lead)));
    } else {
        all.push(Span::raw(" ".repeat(pad)));
    }
    all.append(&mut right);
    Line::from(all)
}

fn cells_of(spans: &[Span<'_>]) -> usize {
    spans.iter().map(|s| cells(&s.content)).sum()
}

/// The time the turn has run (or the last one ran), as `m:ss`.
fn elapsed(m: &Model) -> String {
    let d = m
        .activity
        .since
        .map_or(Duration::ZERO, |s| m.transcript.clock.saturating_sub(s));
    let s = d.as_secs();
    format!("{}:{:02}", s / 60, s % 60)
}

fn cost(m: &Model) -> String {
    let u = &m.transcript.usage;
    if u.tokens == 0 && u.usd == 0.0 {
        "—".to_owned()
    } else if u.unpriced && u.usd == 0.0 {
        "unpriced".to_owned()
    } else {
        format!("${:.3}{}", u.usd, if u.unpriced { "+" } else { "" })
    }
}

/// The Done when panel's lines: each line of the body's list, ticked once the end gate passed the
/// answer, open before.
#[must_use]
pub fn done_when(m: &Model, w: usize) -> Lines {
    let t = &m.theme;
    let lines = &m.sidebar.done;
    if lines.is_empty() {
        return vec![truncate(
            Line::from(Span::styled(
                "written when the body writes its plan",
                t.muted(),
            )),
            w,
        )];
    }
    let passed = m.last.as_ref().is_some_and(|(_, p)| *p);
    lines
        .iter()
        .map(|l| {
            let mark = if passed {
                Span::styled("✔ ", t.ok().add_modifier(Modifier::BOLD))
            } else {
                Span::styled("○ ", t.muted())
            };
            truncate(
                Line::from(vec![mark, Span::styled(crate::safe::one_line(l), t.text())]),
                w,
            )
        })
        .collect()
}

/// A gauge `w` cells wide filled to `frac`, in the gradient, on a quiet track.
fn gauge(t: &Theme, frac: f64, w: usize) -> Vec<Span<'static>> {
    // Display only.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let full = ((frac.clamp(0.0, 1.0) * w as f64).round() as usize).max(usize::from(frac > 0.0));
    let mut v = Vec::new();
    for i in 0..w {
        if i < full {
            // Display only.
            #[allow(clippy::cast_precision_loss)]
            let c = t.p.holo.mix(t.p.gold, i as f32 / w.max(1) as f32 * 0.7);
            v.push(Span::styled("━", Style::new().fg(c.color())));
        } else {
            v.push(Span::styled("━", t.line()));
        }
    }
    v
}

/// The Budgets panel's lines: the steps, the clock, the context against its line (a gauge), the
/// spend.
#[must_use]
pub fn budgets(m: &Model, w: usize) -> Lines {
    let t = &m.theme;
    let u = &m.transcript.usage;
    let row = |label: &str, value: String| {
        truncate(
            Line::from(vec![
                Span::styled(format!("{label:<11}"), t.muted()),
                Span::styled(value, t.text().add_modifier(Modifier::BOLD)),
            ]),
            w,
        )
    };
    let mut out = vec![
        row("steps", u.calls.to_string()),
        row("wall clock", elapsed(m)),
    ];
    match u.occupancy.saturating_mul(100).checked_div(u.stress_line) {
        Some(pct) => {
            out.push(row(
                "context",
                format!(
                    "{} / {}",
                    crate::sidebar::short(u.occupancy),
                    crate::sidebar::short(u.stress_line)
                ),
            ));
            let label = format!(" {pct:>3}%");
            let bar = w.saturating_sub(11 + cells(&label));
            let mut spans = vec![Span::raw(" ".repeat(11))];
            // Display only.
            #[allow(clippy::cast_precision_loss)]
            spans.extend(gauge(t, u.occupancy as f64 / u.stress_line as f64, bar));
            spans.push(Span::styled(label, crate::sidebar::band(t, pct)));
            out.push(Line::from(spans));
        }
        None => out.push(row("context", "—".to_owned())),
    }
    out.push(row("cost", cost(m)));
    out
}

/// What each native unit does, in a few words, for the Units panel.
fn unit_words(id: &str) -> &'static str {
    match id {
        "memory" => "recall and lessons",
        "gate" => "verify at the end",
        "hold" => "held Done when list",
        "fold" => "fold long context",
        "stress" => "stall warnings",
        "what_changed" => "what changed",
        "reader" => "a second reader",
        "plan" => "plan before code",
        "plan_lite" => "plan what is stated",
        "returns" => "tree readings",
        "challenge" => "review challenge",
        _ => "plugin",
    }
}

/// The Units panel's lines: each unit the run lists, what it does, and an ON or OFF pill.
#[must_use]
pub fn units(m: &Model, w: usize) -> Lines {
    let t = &m.theme;
    let listed: Vec<(String, String)> = m.units.as_ref().map_or_else(Vec::new, |d| {
        d.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .listed
            .iter()
            .map(|(id, _, state)| (id.clone(), state.clone()))
            .collect()
    });
    if listed.is_empty() {
        return vec![truncate(
            Line::from(Span::styled("listed once the session opens", t.muted())),
            w,
        )];
    }
    let on_pill = Style::new()
        .fg(t.p.bg.color())
        .bg(t.readable(t.races.elf).color())
        .add_modifier(Modifier::BOLD);
    let off_pill = Style::new()
        .fg(t.p.text_muted.color())
        .bg(t.p.panel_raised.color());
    listed
        .into_iter()
        .map(|(id, state)| {
            let on = state == "active";
            let name = format!("{:<14}", crate::kit::text::clip(&id, 13));
            let words = unit_words(&id);
            let pill = if on { " ON " } else { " OFF" };
            let room = w.saturating_sub(cells(&name) + 5);
            let words = if id.starts_with("plugin:") { "" } else { words };
            let words = crate::kit::text::clip(words, room);
            let pad = room.saturating_sub(cells(&words));
            Line::from(vec![
                Span::styled(name, t.text()),
                Span::styled(words, t.muted()),
                Span::raw(" ".repeat(pad + 1)),
                Span::styled(pill, if on { on_pill } else { off_pill }),
            ])
        })
        .collect()
}
