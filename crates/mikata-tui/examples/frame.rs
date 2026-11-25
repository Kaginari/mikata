//! A frame for the eye: a scripted session drawn into Ratatui's test terminal and written out as
//! cells — each one's text, colours and weight — for `tools/tui-shot.mjs` to photograph.
//!
//! `cargo run -p mikata-tui --example frame -- <width> <height> <dark|light> [scheme] > frame.json`

use mikata_server::protocol::Event;
use mikata_tui::app::{Header, Model, Msg, update, view};
use mikata_tui::theme::{Scheme, Theme, Variant};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::{Color, Modifier};
use serde_json::json;

fn ev(m: &mut Model, event: Event) {
    update(m, Msg::Event { event, below: None });
}

fn call(m: &mut Model, id: &str, name: &str, input: serde_json::Value, out: &str, error: bool) {
    ev(
        m,
        Event::ToolCall {
            id: id.into(),
            name: name.into(),
            input,
        },
    );
    ev(
        m,
        Event::ToolResult {
            id: id.into(),
            content: out.into(),
            is_error: error,
            cut: 0,
        },
    );
}

fn hex(c: Color) -> Option<String> {
    match c {
        Color::Rgb(r, g, b) => Some(format!("#{r:02x}{g:02x}{b:02x}")),
        Color::Reset => None,
        other => Some(format!("{other:?}")),
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "one scripted session, read top to bottom"
)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cols: u16 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(160);
    let rows: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(48);
    let variant = if args.get(3).map(String::as_str) == Some("light") {
        Variant::Light
    } else {
        Variant::Dark
    };
    let scheme = args
        .get(4)
        .and_then(|s| Scheme::parse(s))
        .unwrap_or_default();
    // A world for the panels to read: the body's plan and a branch.
    let world = std::env::temp_dir().join(format!("mikata-frame-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&world);
    std::fs::create_dir_all(world.join(".mikata/tmp")).unwrap();
    std::fs::create_dir_all(world.join(".git")).unwrap();
    std::fs::write(world.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
    std::fs::write(
        world.join(".mikata/tmp/plan.md"),
        "## Done when\n- D1 parse('1h30m') -> 5400\n- D2 parse('90') stays unchanged\n- D3 docs updated\n- D4 verify passes\n",
    )
    .unwrap();
    let mut m = Model::new(
        Header {
            body: "summoner".into(),
            model: "claude-haiku-5.5".into(),
            world: "~/parser-rs".into(),
            mode: "light-coding".into(),
        },
        variant,
        (cols, rows),
    );
    m.theme = Theme::of(scheme, variant);
    m.sandbox = "landlock ABI 6 + seccomp + limits, network off, private /tmp".into();
    m.sidebar.root = Some(world.clone());
    m.sidebar.surveyed(mikata_tui::sidebar::survey(&world));
    m.sidebar.verify = Some(mikata_tui::sidebar::Verify {
        line: vec!["cargo test".into()],
        suggested: None,
    });
    let desk = mikata_loop::units::UnitDesk::default();
    desk.lock().unwrap().listed = [
        "memory",
        "gate",
        "hold",
        "fold",
        "stress",
        "what_changed",
        "reader",
        "plan",
        "returns",
    ]
    .iter()
    .map(|id| {
        (
            (*id).to_owned(),
            "1.1.0".to_owned(),
            if *id == "reader" {
                "inactive".to_owned()
            } else {
                "active".to_owned()
            },
        )
    })
    .chain(std::iter::once((
        "plugin:no-force-push".to_owned(),
        "0.1.0".to_owned(),
        "active".to_owned(),
    )))
    .collect();
    m.units = Some(desk);
    m.outstanding = 1;
    ev(
        &mut m,
        Event::TurnStarted {
            text: "add support for 1h30m duration parsing".into(),
        },
    );
    ev(
        &mut m,
        Event::TextDelta {
            text: "I'll write down what done means first, then change the parser.\n\n".into(),
        },
    );
    call(
        &mut m,
        "1",
        "write",
        json!({"path": ".mikata/tmp/plan.md", "content": "## Done when\nD1: parse('1h30m') -> 5400\nD2: parse('90') stays unchanged\nD3: docs updated\nD4: verify passes\n"}),
        "wrote .mikata/tmp/plan.md",
        false,
    );
    call(
        &mut m,
        "2",
        "read",
        json!({"path": "src/parse.rs", "offset": 1, "end": 80}),
        &"fn parse(s: &str) -> u64 {\n".repeat(20),
        false,
    );
    call(
        &mut m,
        "3",
        "edit",
        json!({"path": "src/parse.rs", "old_string": "    s.parse().unwrap()\n", "new_string": "    if let Some((h, rest)) = s.split_once('h') {\n        return h.parse::<u64>().unwrap() * 3600 + minutes(rest);\n    }\n    s.parse().unwrap()\n"}),
        "edited src/parse.rs",
        false,
    );
    call(
        &mut m,
        "4",
        "bash",
        json!({"command": "cargo test -p parser --lib parse::tests::duration"}),
        "running 6 tests\n......\ntest result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s\n",
        false,
    );
    call(
        &mut m,
        "5",
        "bash",
        json!({"command": "cargo test"}),
        "running 42 tests\ntest result: ok. 42 passed; 0 failed; 0 ignored (0.27s)\n",
        false,
    );
    ev(&mut m, Event::TextDelta { text: "Added mixed hour-minute parsing; `parse(\"90\")` is unchanged. I ran D1 to D4 and the whole suite.".into() });
    ev(
        &mut m,
        Event::GateVerdict {
            passed: true,
            verdict: "PASS".into(),
            changed: vec!["src/parse.rs".into(), "README.md".into()],
            failures: vec![],
            unverified: vec![],
            unchecked: false,
            verified_by: vec!["cargo test".into()],
        },
    );
    ev(
        &mut m,
        Event::Usage {
            input: 12_000,
            output: 1_400,
            cache_read: 30_000,
            cache_write: 0,
            usd: Some(0.038),
            wt: 1,
            spent_wt: 1,
            occupancy: 18_400,
            stress_line: 150_000,
        },
    );
    // A few calls more, as a turn has: steps are counted per answered model call.
    for _ in 0..13 {
        ev(
            &mut m,
            Event::Usage {
                input: 0,
                output: 0,
                cache_read: 0,
                cache_write: 0,
                usd: Some(0.0),
                wt: 0,
                spent_wt: 0,
                occupancy: 18_400,
                stress_line: 150_000,
            },
        );
    }
    update(&mut m, Msg::Resize(cols, rows));
    // `gate` as the fifth argument: an approval waits over the view.
    if args.get(5).map(String::as_str) == Some("gate") {
        let card = mikata_tui::gate::Card::of(
            &mikata_loop::GateAsk {
                body: "summoner".into(),
                race: mikata_races::Race::Summoner,
                class: mikata_loop::ToolClass::Outward,
                act: "git push origin main".into(),
                why: vec![
                    "publishes the branch to a remote".into(),
                    "reaches beyond the world".into(),
                ],
                outside: vec![],
                fingerprint: None,
            },
            mikata_tools::Pending::new(),
        );
        update(&mut m, Msg::Gate(card));
    }
    let mut t = Terminal::new(TestBackend::new(cols, rows)).unwrap();
    t.draw(|f| view(&m, f)).unwrap();
    let b = t.backend().buffer();
    let mut grid = Vec::new();
    for y in 0..rows {
        let mut cells = Vec::new();
        for x in 0..cols {
            let c = &b[(x, y)];
            cells.push(json!({
                "s": c.symbol(),
                "fg": hex(c.fg),
                "bg": hex(c.bg),
                "b": c.modifier.contains(Modifier::BOLD),
                "i": c.modifier.contains(Modifier::ITALIC),
                "d": c.modifier.contains(Modifier::DIM),
                "u": c.modifier.contains(Modifier::UNDERLINED),
            }));
        }
        grid.push(cells);
    }
    println!("{}", json!({"w": cols, "h": rows, "rows": grid}));
    let _ = std::fs::remove_dir_all(&world);
}
