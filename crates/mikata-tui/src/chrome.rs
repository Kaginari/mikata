//! The chrome around the conversation: a header bar of chips, panels framed in the accent's
//! gradient (Done when, Budgets, Checks, Guardrails, Units, Session), and a footer bar. Every
//! colour comes from the theme: the gradient runs from the accent toward the human's colour,
//! across the screen, so the panels' borders read as one sweep of light.

use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::app::Model;
use crate::kit::text::{Lines, cells, truncate};
use crate::theme::{Rgb, Theme};

/// The gradient's colour at `x` of a screen `w` cells wide: the accent on the left, through
/// their midpoint (Neon's coral) to the human's colour on the right.
#[must_use]
pub fn sweep(t: &Theme, x: u16, w: u16) -> Rgb {
    let f = f32::from(x) / f32::from(w.max(1));
    t.p.holo.mix(t.p.gold, f)
}

/// A panel: a heavy border in the gradient, its title bold on the top edge with a glyph before
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
        let c = sweep(t, x, screen_w);
        let cell = &mut buf[(x, y)];
        cell.set_symbol(s);
        cell.set_style(
            Style::new()
                .fg(c.color())
                .bg(t.p.panel.color())
                .add_modifier(Modifier::BOLD),
        );
    };
    // Heavy lines, in the gradient at full strength: the frame is the screen's light.
    for x in x0..=x1 {
        let (top, bottom) = match x {
            _ if x == x0 => ("┏", "┗"),
            _ if x == x1 => ("┓", "┛"),
            _ => ("━", "━"),
        };
        edge(buf, x, y0, top);
        edge(buf, x, y1, bottom);
    }
    for y in (y0 + 1)..y1 {
        edge(buf, x0, y, "┃");
        edge(buf, x1, y, "┃");
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
        let slashes = " //// ";
        let at = area.right() - 8;
        let c = sweep(t, at, screen_w);
        buf.set_string(
            at,
            area.y,
            slashes,
            Style::new()
                .fg(c.color())
                .bg(t.p.panel.color())
                .add_modifier(Modifier::BOLD),
        );
    }
    inside
}

/// A bar filled with the gradient, cell by cell across the screen, `line` on its middle row in
/// the page's colour (dark on the bright brand, light on the deep one): the header, as the
/// board's header strip is. A span that carries its own face (a chip's pill) is turned inside
/// out: the page's face, the pill's colour as its text.
pub fn gradient_bar(buf: &mut Buffer, area: Rect, line: &Line<'static>, t: &Theme, screen_w: u16) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            let cell = &mut buf[(x, y)];
            cell.reset();
            cell.set_style(Style::new().bg(sweep(t, x, screen_w).color()));
        }
    }
    let ink = t.p.bg.color();
    let spans: Vec<Span<'static>> = line
        .spans
        .iter()
        .map(|s| {
            let style = match s.style.bg {
                Some(pill) => Style::new().fg(pill).bg(ink),
                None => Style::new().fg(ink),
            };
            Span::styled(s.content.clone(), style.add_modifier(Modifier::BOLD))
        })
        .collect();
    let row = area.y + area.height / 2;
    let x = area.x + 2;
    let w = area.width.saturating_sub(4);
    // Each span over the gradient: its cells keep the bar's colour under them.
    let mut at = x;
    for s in spans {
        if at >= x + w {
            break;
        }
        for ch in s.content.chars() {
            let cw = u16::try_from(cells(&ch.to_string())).unwrap_or(1);
            if cw == 0 || at + cw > x + w {
                continue;
            }
            let cell = &mut buf[(at, row)];
            cell.set_symbol(&ch.to_string());
            let mut style = s.style;
            if style.bg.is_none() {
                style = style.bg(sweep(t, at, screen_w).color());
            }
            cell.set_style(style);
            at += cw;
        }
    }
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
            Some(if on { t.success } else { t.p.alarm }),
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

/// The time the turn has run, or the last one ran once it ended, as `m:ss`.
fn elapsed(m: &Model) -> String {
    let d = match m.activity.since {
        Some(s) => m.transcript.clock.saturating_sub(s),
        None => m.activity.last.unwrap_or(Duration::ZERO),
    };
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
    let lines = m.sidebar.done_now();
    if lines.is_empty() {
        return vec![truncate(
            Line::from(Span::styled(
                "your ask's Done when lines, or the plan's (--units +plan)",
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

/// The Units panel's lines: each unit the run lists as a tag — lit and ticked where it is on,
/// quiet where it is off — wrapped to `w`.
#[must_use]
pub fn units(m: &Model, w: usize) -> Lines {
    let t = &m.theme;
    let listed: Vec<(String, bool)> = m.units.as_ref().map_or_else(Vec::new, |d| {
        d.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .listed
            .iter()
            .map(|(id, _, state)| (id.clone(), state == "active"))
            .collect()
    });
    if listed.is_empty() {
        return vec![truncate(
            Line::from(Span::styled("listed once the session opens", t.muted())),
            w,
        )];
    }
    let mut out: Lines = Vec::new();
    let mut line: Vec<Span<'static>> = Vec::new();
    let mut used = 0;
    for (id, on) in listed {
        let name = crate::kit::text::clip(&crate::safe::one_line(&id), w.saturating_sub(4));
        let tag = if on {
            Span::styled(format!(" ✔ {name} "), t.ok().add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!(" ○ {name} "), t.muted())
        };
        let tag_w = cells(&tag.content);
        if used > 0 && used + 1 + tag_w > w {
            out.push(Line::from(std::mem::take(&mut line)));
            used = 0;
        }
        if used > 0 {
            line.push(Span::raw(" "));
            used += 1;
        }
        let face = if on { t.p.panel_raised } else { t.p.panel };
        line.push(tag.patch_style(Style::new().bg(face.color())));
        used += tag_w;
    }
    if !line.is_empty() {
        out.push(Line::from(line));
    }
    out
}

/// A check's state, as the Checks panel shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Held {
    /// It held; how long its last run took, when the body ran it.
    Ok(Option<Duration>),
    /// It failed.
    Failed,
    /// It ran and gave no result.
    NoResult,
    /// It runs now.
    Running,
    /// It has not run: the end gate runs it.
    Waits,
}

/// A duration as a check's chip: `340ms`, `1.27s`, `2m05s`.
fn took(d: Duration) -> String {
    let ms = d.as_millis();
    if ms < 1000 {
        format!("{ms}ms")
    } else if ms < 60_000 {
        // Display only.
        #[allow(clippy::cast_precision_loss)]
        let s = ms as f64 / 1000.0;
        format!("{s:.2}s")
    } else {
        format!("{}m{:02}s", ms / 60_000, (ms / 1000) % 60)
    }
}

/// The end gate's checks as the session last saw them: each verify command, from the gate's
/// latest verdict or the body's latest run of it, whichever came last; then the gate's own two,
/// tests intact and duties done, from the verdict alone.
fn held(m: &Model) -> Vec<(String, Held)> {
    use crate::transcript::Entry;
    let entries = m.transcript.entries();
    let verdict = entries
        .iter()
        .enumerate()
        .rev()
        .find_map(|(at, e)| match e {
            Entry::Verdict {
                failures,
                unverified,
                ..
            } => Some((at, failures, unverified)),
            _ => None,
        });
    let by_gate = |needle: &dyn Fn(&str) -> bool| {
        verdict.map(|(_, failures, unverified)| {
            if failures.iter().any(|f| needle(f)) {
                Held::Failed
            } else if unverified.iter().any(|f| needle(f)) {
                Held::NoResult
            } else {
                Held::Ok(None)
            }
        })
    };
    let mut out = Vec::new();
    let lines = m
        .sidebar
        .verify
        .as_ref()
        .map_or(&[][..], |v| v.line.as_slice());
    for cmd in lines {
        let quoted = format!("`{cmd}`");
        let ran = entries
            .iter()
            .enumerate()
            .rev()
            .find_map(|(at, e)| match e {
                Entry::Tool(call)
                    if call.name == "bash"
                        && call
                            .input
                            .get("command")
                            .and_then(|v| v.as_str())
                            .is_some_and(|s| s.trim() == cmd.trim()) =>
                {
                    Some((at, call))
                }
                _ => None,
            });
        let state = match (ran, verdict) {
            (Some((at, call)), seen) if seen.is_none_or(|(when, _, _)| at > when) => {
                match &call.outcome {
                    _ if call.running() => Held::Running,
                    Some(o) if o.is_error => Held::Failed,
                    Some(_) => Held::Ok(call.took),
                    None => Held::NoResult,
                }
            }
            _ => by_gate(&|f: &str| f.contains(&quoted)).unwrap_or(Held::Waits),
        };
        out.push((cmd.clone(), state));
    }
    for (name, prefix) in [
        ("tests intact", "tests intact"),
        ("duties done", "duties done"),
    ] {
        let state = by_gate(&|f: &str| f.starts_with(prefix)).unwrap_or(Held::Waits);
        out.push((name.to_owned(), state));
    }
    out
}

/// The Checks panel's lines: each check the end gate holds the answer to, its state and the time
/// its last run took, and a row of how many held.
#[must_use]
pub fn checks(m: &Model, w: usize) -> Lines {
    let t = &m.theme;
    let rows = held(m);
    let mut out: Lines = Vec::new();
    if m.sidebar.verify.as_ref().is_none_or(|v| v.line.is_empty()) {
        out.push(truncate(
            Line::from(vec![
                Span::styled("○ ", t.gold()),
                Span::styled("no verify line pinned", t.muted()),
            ]),
            w,
        ));
    }
    for (name, state) in &rows {
        let (mark, word, hue) = match state {
            Held::Ok(_) => ("✔", "ok", t.ok()),
            Held::Failed => ("✘", "failed", t.alarm()),
            Held::NoResult => ("○", "no result", t.gold()),
            Held::Running => ("◌", "running", t.gold()),
            Held::Waits => ("○", "at the gate", t.muted()),
        };
        let time = match state {
            Held::Ok(Some(d)) => format!(" {:>6}", took(*d)),
            _ => String::new(),
        };
        let right = format!("{word}{time}");
        let room = w.saturating_sub(cells(&right) + 3);
        let label = crate::kit::text::clip(&crate::safe::one_line(name), room);
        let pad = room.saturating_sub(cells(&label));
        out.push(Line::from(vec![
            Span::styled(format!("{mark} "), hue.add_modifier(Modifier::BOLD)),
            Span::styled(label, t.text()),
            Span::raw(" ".repeat(pad + 1)),
            Span::styled(word.to_owned(), hue),
            Span::styled(time, t.muted()),
        ]));
    }
    let ok = rows
        .iter()
        .filter(|(_, s)| matches!(s, Held::Ok(_)))
        .count();
    let label = format!(" {ok}/{} held", rows.len());
    // Display only.
    #[allow(clippy::cast_precision_loss)]
    let frac = ok as f64 / rows.len().max(1) as f64;
    let mut bar = gauge(t, frac, w.saturating_sub(cells(&label)).min(16));
    bar.push(Span::styled(label, t.muted()));
    out.push(Line::from(bar));
    out
}

/// The Guardrails panel's lines: what the sandbox holds every command to, and what the end gate
/// and the approval gate hold the session to — each ticked where it is in force.
#[must_use]
pub fn guardrails(m: &Model, w: usize) -> Lines {
    let t = &m.theme;
    let row = |on: Option<bool>, text: &str| {
        let (mark, hue) = match on {
            Some(true) => ("✔ ", t.ok()),
            Some(false) => ("✘ ", t.alarm()),
            None => ("○ ", t.gold()),
        };
        truncate(
            Line::from(vec![
                Span::styled(mark, hue.add_modifier(Modifier::BOLD)),
                Span::styled(text.to_owned(), t.text()),
            ]),
            w,
        )
    };
    let mut out = Vec::new();
    let s = m.sandbox.as_str();
    if s.is_empty() {
        out.push(row(None, "sandbox: not yet known"));
    } else if s.starts_with("none") {
        out.push(row(Some(false), "no sandbox: commands run as you"));
    } else {
        out.push(row(Some(true), "commands sandboxed to the project"));
        let net_off = s.contains("network off");
        out.push(row(
            Some(net_off),
            if net_off { "network off" } else { "network on" },
        ));
        if s.contains("private /tmp") {
            out.push(row(Some(true), "a /tmp of their own"));
        }
    }
    // A unit's state where the run lists its units; before it does, the default: on.
    let on = |unit: &str| {
        m.units.as_ref().is_none_or(|d| {
            let desk = d.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            desk.listed.is_empty()
                || desk
                    .listed
                    .iter()
                    .any(|(id, _, state)| id == unit && state == "active")
        })
    };
    // The gate's unit: off, the end gate holds nothing.
    let gate_on = on("gate");
    if gate_on {
        let pinned = m
            .sidebar
            .verify
            .as_ref()
            .is_some_and(|v| !v.line.is_empty());
        out.push(row(
            pinned.then_some(true),
            if pinned {
                "verify must pass"
            } else {
                "no verify line: a pass checks nothing"
            },
        ));
        out.push(row(Some(true), "no test deleted or skipped"));
    } else {
        out.push(row(Some(false), "end gate off"));
    }
    // The approval unit: off, outward and destructive acts run without asking.
    if on("approval") {
        out.push(row(Some(true), "push and outward acts ask you"));
    } else {
        out.push(row(Some(false), "outward acts run without asking"));
    }
    // The secrets unit: on, the model sees a secret file's keys, never its values.
    if on("secrets") {
        out.push(row(Some(true), "secret files held from the model"));
    } else {
        out.push(row(Some(false), "secret files readable by the model"));
    }
    out
}

/// Where a step of the turn stands, for the stepper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Not reached.
    Todo,
    /// Under way.
    Now,
    /// Done.
    Done,
    /// It stopped there: the gate failed, or the turn ended unanswered.
    Failed,
}

/// The latest turn's four steps — Plan, Work, Verify, Land — read from what it has done: a call
/// made means the plan is behind it, a run of the pinned verify line (or the gate's verdict)
/// means the work is, a passing verdict lands it.
#[must_use]
pub fn steps(m: &Model) -> [Step; 4] {
    use crate::transcript::Entry;
    let entries = m.transcript.entries();
    let Some(start) = entries.iter().rposition(|e| matches!(e, Entry::User(_))) else {
        return [Step::Todo; 4];
    };
    let turn = &entries[start + 1..];
    let verify: Vec<&str> = m
        .sidebar
        .verify
        .as_ref()
        .map_or_else(Vec::new, |v| v.line.iter().map(|s| s.trim()).collect());
    let is_verify = |call: &crate::transcript::Call| {
        call.name == "bash"
            && call
                .input
                .get("command")
                .and_then(|v| v.as_str())
                .is_some_and(|c| verify.contains(&c.trim()))
    };
    let worked = turn
        .iter()
        .any(|e| matches!(e, Entry::Tool(_) | Entry::Group(_)));
    let verifying = turn
        .iter()
        .any(|e| matches!(e, Entry::Tool(c) if is_verify(c)));
    let verdict = turn.iter().rev().find_map(|e| match e {
        Entry::Verdict { verdict, .. } => Some(verdict.as_str()),
        _ => None,
    });
    let ended = turn.iter().any(|e| matches!(e, Entry::Ending(_)));
    let busy = m.busy();
    match verdict {
        Some("PASS") => [Step::Done; 4],
        Some(_) if busy => [Step::Done, Step::Now, Step::Failed, Step::Todo],
        Some(_) => [Step::Done, Step::Done, Step::Failed, Step::Todo],
        None => {
            let stop = |s: Step| if ended && !busy { Step::Failed } else { s };
            if verifying {
                [Step::Done, Step::Done, stop(Step::Now), Step::Todo]
            } else if worked {
                [Step::Done, stop(Step::Now), Step::Todo, Step::Todo]
            } else if busy || ended {
                [stop(Step::Now), Step::Todo, Step::Todo, Step::Todo]
            } else {
                [Step::Done, Step::Done, Step::Todo, Step::Todo]
            }
        }
    }
}

/// The stepper's lines in `w` cells: the four steps' marks joined by rails, lit as far as the
/// turn has come, then their names under them.
#[must_use]
pub fn stepper(m: &Model, w: usize) -> Lines {
    let t = &m.theme;
    let names = ["Plan", "Work", "Verify", "Land"];
    let marks = steps(m);
    let look = |s: Step| match s {
        Step::Done => ("●", t.ok().add_modifier(Modifier::BOLD)),
        Step::Now => (m.spinner.view(), t.holo().add_modifier(Modifier::BOLD)),
        Step::Failed => ("✘", t.alarm().add_modifier(Modifier::BOLD)),
        Step::Todo => ("○", t.muted()),
    };
    // Each step's column is as wide as the widest name; the rails fill what is left, from one
    // mark to the next.
    let col = 6;
    let rail = (w.saturating_sub(col * 4) / 3).max(1);
    let mut top = Vec::new();
    let mut under = Vec::new();
    for (i, (name, s)) in names.iter().zip(marks).enumerate() {
        let (mark, style) = look(s);
        top.push(Span::styled(mark.to_owned(), style));
        let word = if s == Step::Todo { t.muted() } else { style };
        if i < 3 {
            let lit = s == Step::Done && marks[i + 1] != Step::Todo;
            let hue = if lit { t.ok() } else { t.line() };
            top.push(Span::styled("━".repeat(col - 1 + rail), hue));
            under.push(Span::styled(format!("{name:<0$}", col + rail), word));
        } else {
            under.push(Span::styled((*name).to_owned(), word));
        }
    }
    vec![truncate(Line::from(top), w), truncate(Line::from(under), w)]
}

/// The lines each changed path gained and lost this session, from the file tools' calls that
/// landed: an edit's diff counts both ways, a file written whole counts its lines as gained.
fn churn(m: &Model, path: &str) -> (usize, usize) {
    use crate::transcript::Entry;
    let mut sum = (0, 0);
    for e in m.transcript.entries() {
        let Entry::Tool(call) = e else { continue };
        let landed = call.outcome.as_ref().is_some_and(|o| !o.is_error);
        let named = call
            .input
            .get("path")
            .and_then(|v| v.as_str())
            .is_some_and(|p| p == path || p.ends_with(&format!("/{path}")));
        if !(landed && named) {
            continue;
        }
        match crate::diff::of(&call.name, &call.input) {
            Some(crate::diff::Change::Diff { added, removed, .. }) => {
                sum.0 += added;
                sum.1 += removed;
            }
            Some(crate::diff::Change::NewFile { lines, .. }) => sum.0 += lines,
            None => {}
        }
    }
    sum
}

/// The Files changed panel's lines: each path the session changed, newest first, its mark as a
/// tag (A added, M modified, D deleted) and the lines it gained and lost.
#[must_use]
pub fn files_changed(m: &Model, w: usize) -> Lines {
    let t = &m.theme;
    let list = m.sidebar.files.list();
    if list.is_empty() {
        return vec![truncate(
            Line::from(Span::styled("nothing changed yet", t.muted())),
            w,
        )];
    }
    list.iter()
        .rev()
        .map(|(path, mark)| {
            let hue = match mark {
                crate::sidebar::Mark::Added => t.success,
                crate::sidebar::Mark::Modified => t.p.gold,
                crate::sidebar::Mark::Deleted => t.p.alarm,
            };
            let tag = Span::styled(
                format!(" {} ", mark.letter()),
                Style::new()
                    .fg(t.p.bg.color())
                    .bg(hue.color())
                    .add_modifier(Modifier::BOLD),
            );
            let (plus, minus) = churn(m, path);
            let right = format!("+{plus} −{minus}");
            let room = w.saturating_sub(cells(&right) + 5);
            let name = crate::kit::text::clip(&crate::safe::one_line(path), room);
            let pad = room.saturating_sub(cells(&name));
            Line::from(vec![
                tag,
                Span::raw(" "),
                Span::styled(name, t.text()),
                Span::raw(" ".repeat(pad + 1)),
                Span::styled(format!("+{plus}"), t.ok().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(format!("−{minus}"), t.alarm().add_modifier(Modifier::BOLD)),
            ])
        })
        .collect()
}
