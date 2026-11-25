//! The frame, in Isekai's look: a rounded border naming the session on the left of its top edge
//! and its state on the right, the status bar on its bottom edge; inside, the panes labelled in
//! capitals, as the wordmark — CONVERSATION (the welcome, the transcript's visible window, the lines
//! waiting their turn, the activity strip while work runs), ASK (the input) and SESSION (the
//! sidebar, [`crate::sidebar`]). An approval floats over the view, dimmed, as a modal; a toast
//! floats at the top right while it fades.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Widget};

use crate::app::Model;
use crate::blocks::Ctx;
use crate::kit::keyhelp::{Binding, KeyHelp};
use crate::kit::text::{Lines, cells, clip, truncate};
use crate::pictures::{Art, Slot, Want};
use crate::theme::Rgb;

/// The smallest frame worth drawing.
const MIN: (u16, u16) = (24, 8);
/// The most rows the input grows to.
const INPUT_ROWS: usize = 6;

/// What a block of the transcript is drawn with, at `width`.
#[must_use]
pub fn ctx(m: &Model, width: usize) -> Ctx<'_> {
    Ctx {
        theme: m.theme,
        width,
        frame: m.spinner.frame(),
        spinner: m.spinner.view(),
        me: &m.header.body,
        now: m.transcript.clock,
    }
}

/// Draw `m` into the frame: a header bar, the agent loop on the left, the panels on the right
/// (Done when, Budgets, Units, Session), a footer bar; an approval floats over the loop as a modal,
/// a toast at its top right while it fades.
pub fn draw(m: &Model, f: &mut Frame<'_>) {
    let area = f.area();
    let buf = f.buffer_mut();
    let t = &m.theme;
    m.pictures.wants.borrow_mut().clear();
    Block::new().style(t.page()).render(area, buf);
    if area.width < MIN.0 || area.height < MIN.1 {
        let msg = Line::from(Span::styled(
            clip("enlarge the terminal", area.width.into()),
            t.muted(),
        ));
        buf.set_line(area.x, area.y, &msg, area.width);
        return;
    }
    // Bars of three rows (a framed line) where the rows are many, of one row where the
    // conversation needs them.
    let tall = area.height >= 36;
    let bar = if tall { 3 } else { 1 };
    let top = Rect {
        height: bar,
        ..area
    };
    let bottom = Rect {
        y: area.bottom() - bar,
        height: bar,
        ..area
    };
    let mid = Rect {
        y: area.y + bar,
        height: area.height.saturating_sub(2 * bar),
        ..area
    };
    let bar_line = |buf: &mut Buffer, r: Rect, line: Line<'static>| {
        if tall {
            let inner = crate::chrome::panel(buf, r, "", "", t, area.width);
            buf.set_line(inner.x + 1, inner.y, &line, inner.width.saturating_sub(2));
        } else {
            buf.set_line(r.x, r.y, &line, r.width);
        }
    };
    let inner_w = usize::from(area.width.saturating_sub(if tall { 4 } else { 0 }));
    bar_line(buf, top, crate::chrome::header(m, inner_w, state(m)));
    bar_line(
        buf,
        bottom,
        crate::chrome::footer(m, inner_w, |room| keys(m, room)),
    );
    // The panels' columns: the sidebar's width and its own frame each. Wide, the checks and the
    // rails stand in a column of their own on the left; else every panel shares the right.
    let side = if m.sidebar.width(mid.width.saturating_sub(4)) > 0 {
        (crate::sidebar::WIDTH + 4).min(mid.width / 3)
    } else {
        0
    };
    let wide = side > 0 && mid.width >= 140;
    let lead = if wide { side.min(mid.width / 4) } else { 0 };
    let left = Rect {
        x: mid.x + lead,
        width: mid.width - side - lead,
        ..mid
    };
    // A small terminal gives the conversation every cell: no frame round it.
    let below_label = if left.width >= 60 && left.height >= 16 {
        let lp = crate::chrome::panel(buf, left, "▶", "Agent loop", t, area.width);
        Rect {
            x: lp.x + 1,
            width: lp.width.saturating_sub(2),
            ..lp
        }
    } else {
        left
    };
    body(m, below_label, buf);
    m.hits.borrow_mut().side_w = 0;
    if lead > 0 {
        lead_column(m, buf, Rect { width: lead, ..mid }, area.width);
    }
    if side > 0 {
        panels(
            m,
            Rect {
                x: left.right(),
                width: side,
                ..mid
            },
            buf,
            area.width,
            wide,
        );
    }
    let inner = mid;
    if m.gate.is_some() {
        // The approval never sits under a picture: while it is open, none is wanted.
        m.pictures.wants.borrow_mut().clear();
        dim(buf, inner, t);
        modal(m, below_label, buf);
    }
    toast(m, below_label, buf);
}

/// The toast at the top right of `area` while it fades. It never covers the approval: it waits
/// under the card, unseen, until it is gone.
fn toast(m: &Model, area: Rect, buf: &mut Buffer) {
    let t = &m.theme;
    if let Some(toast) = m.toast.as_ref().filter(|_| m.gate.is_none()) {
        let lines = toast.view(t);
        let width = lines.first().map_or(0, crate::kit::text::line_cells);
        let left = area.right().saturating_sub(to_u16(width));
        for (row, line) in (area.y..area.bottom()).zip(&lines) {
            buf.set_line(left, row, line, to_u16(width));
        }
        // Nor does a picture sit over the toast.
        let over = Rect::new(left, area.y, to_u16(width), to_u16(lines.len()));
        m.pictures
            .wants
            .borrow_mut()
            .retain(|w| w.area.intersection(over).is_empty());
    }
}

/// The left column of a wide screen in `area`: Checks, Guardrails, and Units in the rows left.
fn lead_column(m: &Model, buf: &mut Buffer, area: Rect, screen_w: u16) {
    let w = usize::from(area.width.saturating_sub(4));
    let panes = [
        Pane::new("✓", "Checks", crate::chrome::checks(m, w)),
        Pane::new("◈", "Guardrails", crate::chrome::guardrails(m, w)),
        Pane::new("⚙", "Units", crate::chrome::units(m, w)),
    ];
    stack(m, buf, area, screen_w, &panes, 0);
}

/// A panel waiting its place in a column: its glyph, its title, its lines.
struct Pane {
    glyph: &'static str,
    title: String,
    lines: Lines,
}

impl Pane {
    fn new(glyph: &'static str, title: impl Into<String>, lines: Lines) -> Self {
        Self {
            glyph,
            title: title.into(),
            lines,
        }
    }
}

/// `panes` stacked down `area`, each as tall as its lines, `keep` rows left free under them (none
/// kept: the last pane takes the rows left); a pane with no room for four of its lines is left
/// out. Returns the first row under them.
fn stack(m: &Model, buf: &mut Buffer, area: Rect, screen_w: u16, panes: &[Pane], keep: u16) -> u16 {
    let mut y = area.y;
    for (i, p) in panes.iter().enumerate() {
        let room = area.bottom().saturating_sub(y).saturating_sub(keep);
        let fill = keep == 0 && i + 1 == panes.len();
        let tall = if fill {
            room
        } else {
            (to_u16(p.lines.len()) + 2).min(room)
        };
        // A pane shows four of its lines at least, or all of fewer: one row of a list reads as
        // the whole list.
        let least = to_u16(p.lines.len().clamp(1, 4)) + 2;
        if tall < least {
            continue;
        }
        let card = Rect::new(area.x, y, area.width, tall);
        let inner = crate::chrome::panel(buf, card, p.glyph, &p.title, &m.theme, screen_w);
        for (row, line) in (inner.y..inner.bottom()).zip(&p.lines) {
            buf.set_line(inner.x + 1, row, line, inner.width.saturating_sub(2));
        }
        y += tall;
    }
    y
}

/// The right-hand column in `area`: Done when and Budgets (and, when the screen is not `wide`,
/// Checks, Guardrails and Units, as room allows) each as tall as its lines, the Session panel (memory, files,
/// bodies) taking the rows left.
fn panels(m: &Model, area: Rect, buf: &mut Buffer, screen_w: u16, wide: bool) {
    let t = &m.theme;
    let cols = usize::from(area.width.saturating_sub(4));
    // A short column keeps its rows for what holds something: an empty list, or units not yet
    // listed, gives its panel up.
    let short = area.height < 30;
    let mut panes = Vec::new();
    if !(short && m.sidebar.done.is_empty()) {
        let title = format!("Done when ({})", m.sidebar.done.len());
        panes.push(Pane::new("☑", title, crate::chrome::done_when(m, cols)));
    }
    panes.push(Pane::new("▮", "Budgets", crate::chrome::budgets(m, cols)));
    if !wide {
        panes.push(Pane::new("✓", "Checks", crate::chrome::checks(m, cols)));
        panes.push(Pane::new(
            "◈",
            "Guardrails",
            crate::chrome::guardrails(m, cols),
        ));
        if !(short && m.units.is_none()) {
            panes.push(Pane::new("⚙", "Units", crate::chrome::units(m, cols)));
        }
    }
    // The Session panel keeps its rows — half the column where it shares it with every panel,
    // eight beside the left column: the files and the bodies at work are never squeezed out.
    let keep = if wide { 8 } else { (area.height / 2).max(12) };
    let y = stack(m, buf, area, screen_w, &panes, keep);
    let rest = area.bottom().saturating_sub(y);
    if rest >= 4 {
        let card = Rect::new(area.x, y, area.width, rest);
        let inner = crate::chrome::panel(buf, card, "◇", "Session", t, screen_w);
        // Mikata's logo heads the panel where the terminal draws images and the rows allow:
        // the picture goes over the panel's own plain cells, never a logo built from characters.
        let mut top = inner.y;
        if let Some((pad, cols, rows)) =
            crate::sidebar::logo_box(m, cols, usize::from(inner.height))
        {
            m.pictures.want(Want {
                slot: Slot::Sidebar,
                art: Art::Logo,
                area: Rect::new(inner.x + 1 + pad, top, cols, rows),
                bg: t.p.panel,
            });
            top += rows + 1;
        }
        let rows = usize::from(inner.bottom().saturating_sub(top));
        let lines = crate::sidebar::session(m, cols, rows);
        for (row, line) in (top..inner.bottom()).zip(&lines) {
            buf.set_line(inner.x + 1, row, line, inner.width.saturating_sub(2));
        }
        let mut hits = m.hits.borrow_mut();
        hits.side_x = inner.x + 1;
        hits.side_w = inner.width.saturating_sub(2);
        hits.side_y = top;
    }
}

/// The approval as a modal over `area`, centred across and a third of the way down. Its first
/// draw arms it, and the rows of the act it shows count as read.
fn modal(m: &Model, area: Rect, buf: &mut Buffer) {
    let Some(a) = &m.gate else {
        return;
    };
    let (cw, ch) = crate::approval::modal(usize::from(area.width), usize::from(area.height));
    let now = std::time::Instant::now();
    a.drawn(now, cw, ch);
    let card = a.view(&m.theme, cw, ch, now, m.waiting.len());
    let left = area.x + to_u16(usize::from(area.width).saturating_sub(cw) / 2);
    let top = area.y + to_u16(usize::from(area.height).saturating_sub(card.len()) / 3);
    for (row, line) in (top..area.bottom()).zip(&card) {
        buf.set_line(left, row, line, to_u16(cw));
    }
}

/// Everything in `area` dimmed toward the page: the view under a modal.
fn dim(buf: &mut Buffer, area: Rect, t: &crate::theme::Theme) {
    let page = t.p.bg;
    let fade = |c: Color, k: f32| match c {
        Color::Rgb(r, g, b) => Rgb(r, g, b).mix(page, k).color(),
        Color::Reset => c,
        _ => t.p.line.color(),
    };
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            let cell = &mut buf[(x, y)];
            let fg = fade(cell.fg, 0.6);
            let bg = fade(cell.bg, 0.5);
            cell.set_fg(fg);
            cell.set_bg(bg);
        }
    }
}

/// The section title under (`col`, `row`) in the sidebar the last frame drew.
#[must_use]
pub fn sidebar_title_at(m: &Model, col: u16, row: u16) -> Option<crate::sidebar::Section> {
    let h = m.hits.borrow();
    let inside = h.side_w > 0 && col >= h.side_x && col < h.side_x + h.side_w && row >= h.side_y;
    if !inside {
        return None;
    }
    m.sidebar.title_at(usize::from(row - h.side_y))
}

/// The transcript, the queue, the card and the input, stacked in `area`.
fn body(m: &Model, area: Rect, buf: &mut Buffer) {
    let theme = &m.theme;
    let w = usize::from(area.width);
    // While an approval waits it floats over the view; the input takes no keys then.
    let input = input_box(m, w);
    let strip = crate::activity::strip(m, w);
    let queued: Lines = m
        .transcript
        .queued
        .iter()
        .take(if m.gate.is_some() { 0 } else { 3 })
        .map(|q| {
            truncate(
                Line::from(vec![
                    Span::styled("⧗ waiting  ", theme.muted()),
                    Span::styled(crate::safe::one_line(q), theme.text()),
                ]),
                w,
            )
        })
        .collect();
    let below = input.len() + strip.len() + queued.len();
    let rows = usize::from(area.height).saturating_sub(below);
    let ctx = ctx(m, w);
    let head = crate::welcome::card(m, w);
    let crate::transcript::Window {
        lines: window,
        owners,
        top,
    } = m.transcript.window(&ctx, rows, m.viewport.offset(), &head);
    m.measured.set(top.unwrap_or(usize::MAX));
    // A short transcript starts at the top — the welcome first, like a page's head — and the
    // room it leaves stays below it, above the input; once it fills the rows it scrolls.
    let room = Line::from("");
    let gap = std::iter::repeat_n(&room, rows.saturating_sub(window.len()));
    let stack = window
        .iter()
        .chain(gap)
        .chain(&queued)
        .chain(&strip)
        .chain(&input);
    // A stack taller than the area keeps its bottom: the status and the input stay.
    let stack: Vec<&Line<'static>> = stack.collect();
    let skip = stack.len().saturating_sub(usize::from(area.height));
    let first = area.y;
    for (row, line) in (first..area.bottom()).zip(stack.into_iter().skip(skip)) {
        buf.set_line(area.x, row, line, area.width);
    }
    // Each row's entry, for a click: the transcript's rows only, as drawn.
    let mut hits = m.hits.borrow_mut();
    hits.x = area.x;
    hits.width = area.width;
    hits.y = first;
    hits.rows = rows;
    hits.owners.clear();
    hits.owners.extend(owners.into_iter().skip(skip));
    // The conversation's rows as drawn, for a selection to copy; then the selection over them.
    let drawn = hits.owners.len().min(usize::from(area.height));
    hits.cells = (first..)
        .take(drawn)
        .map(|y| {
            (area.x..area.right())
                .map(|x| buf[(x, y)].symbol().to_owned())
                .collect()
        })
        .collect();
    if let Some(sel) = m.selection {
        let ((c0, r0), (c1, r1)) = sel.span();
        let last = first + u16::try_from(drawn).unwrap_or(u16::MAX);
        for y in r0.max(first)..=r1.min(last.saturating_sub(1)) {
            let from = if y == r0 { c0.max(area.x) } else { area.x };
            let to = if y == r1 {
                c1.min(area.right().saturating_sub(1))
            } else {
                area.right().saturating_sub(1)
            };
            for x in from..=to {
                buf[(x, y)].set_style(theme.selection());
            }
        }
    }
    if !m.viewport.following() && rows > 0 {
        let note = Line::from(Span::styled(
            " ↓ pgdn · esc to the end ",
            theme.muted().add_modifier(Modifier::REVERSED),
        ));
        let x = area.x
            + area
                .width
                .saturating_sub(to_u16(cells(" ↓ pgdn · esc to the end ")));
        let y = area.y + to_u16(rows) - 1;
        buf.set_line(x, y, &note, area.width);
        let over = Rect::new(x, y, area.right().saturating_sub(x), 1);
        m.pictures
            .wants
            .borrow_mut()
            .retain(|w| w.area.intersection(over).is_empty());
    }
}

fn to_u16(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

/// The top edge's right: the session's id, and its state — `○ idle`, `⣾ running bash 3s`,
/// `◆ waiting for you`.
fn state(m: &Model) -> Line<'static> {
    let t = &m.theme;
    let mut spans = vec![Span::raw(" ")];
    if !m.session_id.is_empty() && m.size.0 >= 70 {
        spans.push(Span::styled("session ", t.muted()));
        spans.push(Span::styled(short_id(&m.session_id), t.text()));
        spans.push(Span::raw("  "));
    }
    let chip = crate::activity::chip(m);
    if m.size.0 < 60 {
        // Narrow, the state keeps its mark alone: the session's name comes first.
        if let Some(first) = chip.into_iter().next() {
            let mark: String = first.content.chars().take(1).collect();
            spans.push(Span::styled(mark, first.style));
        }
    } else {
        spans.extend(chip);
    }
    spans.push(Span::raw(" "));
    Line::from(spans)
}

/// A session id as the header shows it: its last part, the rest of it the date.
fn short_id(id: &str) -> String {
    let id = crate::safe::one_line(id);
    let tail = id.rsplit('-').next().unwrap_or(&id);
    clip(tail, 12)
}

/// The footer's middle: what waits (lines queued, lines held) and the keys that matter now, in
/// at most `room` cells.
fn keys(m: &Model, room: usize) -> Line<'static> {
    let t = &m.theme;
    let mut spans: Vec<Span<'static>> = Vec::new();
    let q = m.transcript.queued.len();
    if q > 0 {
        spans.push(Span::styled(format!("{q} queued  "), t.gold()));
    }
    if m.transcript.held > 0 {
        spans.push(Span::styled(
            format!("{} held  ", m.transcript.held),
            t.muted(),
        ));
    }
    let used = spans.iter().map(|s| cells(&s.content)).sum::<usize>();
    let help = KeyHelp {
        key: t.text(),
        action: t.muted(),
        sep: t.frame(),
    }
    .view(bindings(m), room.saturating_sub(used));
    spans.extend(help.spans);
    truncate(Line::from(spans), room)
}

fn bindings(m: &Model) -> &'static [Binding] {
    const GATE: &[Binding] = &[
        Binding {
            key: "y/n",
            action: "choose",
        },
        Binding {
            key: "enter",
            action: "confirm",
        },
        Binding {
            key: "esc",
            action: "deny",
        },
    ];
    const BUSY: &[Binding] = &[
        Binding {
            key: "esc",
            action: "stop",
        },
        Binding {
            key: "enter",
            action: "queue",
        },
        Binding {
            key: "pgup",
            action: "scroll",
        },
    ];
    const IDLE: &[Binding] = &[
        Binding {
            key: "enter",
            action: "send",
        },
        Binding {
            key: "alt+enter",
            action: "newline",
        },
        Binding {
            key: "↑↓",
            action: "history",
        },
        Binding {
            key: "pgup",
            action: "scroll",
        },
        Binding {
            key: "ctrl+c",
            action: "quit",
        },
    ];
    if m.gate.is_some() {
        GATE
    } else if m.busy() {
        BUSY
    } else {
        IDLE
    }
}

/// The rows the input keeps to write in, empty, on a window this tall: a few on a tall one, one
/// on a short one, where the transcript needs them more.
fn input_floor(height: u16) -> usize {
    if height >= 30 { 3 } else { 1 }
}

/// The input: a raised panel as a sent line is, the human's gold down its spine — room to write,
/// then under a quiet rule who the line goes to and what enter does: sends now, waits its turn
/// while a turn runs, or nothing while an approval waits.
fn input_box(m: &Model, w: usize) -> Lines {
    let t = &m.theme;
    let inner = crate::kit::slab::Slab::inner(w);
    let text_w = inner.saturating_sub(2);
    let focused = m.gate.is_none();
    let mut lines = m.input.view(text_w, INPUT_ROWS, t.text(), focused);
    if m.input.text().is_empty() && focused {
        let hint = if m.busy() {
            "a line now waits its turn"
        } else {
            "ask, or /help"
        };
        if let Some(first) = lines.first_mut() {
            first.spans.push(Span::styled(
                clip(hint, text_w.saturating_sub(1)),
                t.muted(),
            ));
        }
    }
    let floor = input_floor(m.size.1);
    while lines.len() < floor {
        lines.push(Line::from(""));
    }
    let prompt = Span::styled(
        "› ",
        if focused {
            t.gold().add_modifier(Modifier::BOLD)
        } else {
            t.muted()
        },
    );
    let mut rows = crate::kit::text::prefixed(lines, &[prompt], &[Span::raw("  ")]);
    // The footer: who the line goes to, in what mode, and what enter does now.
    let mut who = vec![Span::styled(
        format!("◆ {}", crate::safe::shown(&m.header.body)),
        t.holo().add_modifier(Modifier::BOLD),
    )];
    if !m.header.mode.is_empty() {
        who.push(Span::styled(
            format!(" · {}", crate::safe::one_line(&m.header.mode)),
            t.muted(),
        ));
    }
    let (keys, keys_style) = if m.gate.is_some() {
        ("answer the card first".to_owned(), t.gold())
    } else if m.busy() {
        (
            format!("enter queues for {}", crate::safe::shown(&m.header.body)),
            t.muted(),
        )
    } else {
        ("enter send · alt+enter line".to_owned(), t.muted())
    };
    let left = cells(&Line::from(who.clone()).to_string());
    let gap = inner.saturating_sub(left + cells(&keys));
    let mut foot = who;
    if gap >= 2 {
        foot.push(Span::raw(" ".repeat(gap)));
        foot.push(Span::styled(keys, keys_style));
    }
    rows.push(Line::from(Span::styled("─".repeat(inner), t.line())));
    rows.push(truncate(Line::from(foot), inner));
    let hue = if focused { t.races.player } else { t.p.line };
    let face = crate::kit::slab::Surface::card(t, Some(hue), crate::blocks::TINT * 1.5);
    crate::kit::slab::Slab::new(face).render(rows, w)
}
