//! Mikata's logo as real pixels, end to end through the model and the view: where a frame wants
//! it, never under an approval, never sent again for an unchanged frame, the same whoever works,
//! and no logo at all where the terminal draws no images.

use mikata_races::Race;
use mikata_server::protocol::Event;
use mikata_tools::Pending;
use mikata_tui::app::{Header, Model, Msg, update, view};
use mikata_tui::gate::Card;
use mikata_tui::kit::pixels::{self, Protocol};
use mikata_tui::pictures::{Art, Pictures, Placer, Slot, Want};
use mikata_tui::theme::Variant;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// VS Code's terminal: cells of 8 × 17 pixels.
const CELL: (u16, u16) = (8, 17);

fn model(protocol: Protocol) -> Model {
    let mut m = Model::new(
        Header {
            body: "summoner".into(),
            model: "deepseek-v4-flash".into(),
            world: "~/sqlglot".into(),
            mode: "light-coding".into(),
        },
        Variant::Dark,
        (120, 50),
    );
    m.pictures = Pictures::new(protocol, Some(CELL));
    // The logo heads the Session panel, which Advanced draws.
    m.view = mikata_tui::app::View::Advanced;
    m
}

fn draw(m: &mut Model, t: &mut Terminal<TestBackend>) -> Buffer {
    let size = t.backend().buffer().area;
    update(m, Msg::Resize(size.width, size.height));
    t.draw(|f| view(m, f)).unwrap();
    t.backend().buffer().clone()
}

fn card() -> Card {
    Card::of(
        &mikata_loop::GateAsk {
            body: "summoner".into(),
            race: Race::Summoner,
            class: mikata_loop::ToolClass::Outward,
            act: "git push".into(),
            why: vec!["reaches beyond the world".into()],
            outside: vec![],
            fingerprint: None,
        },
        Pending::new(),
    )
}

fn sync(placer: &mut Placer, m: &Model, buf: &Buffer) -> Vec<u8> {
    let mut out = Vec::new();
    let later = std::time::Instant::now() + mikata_tui::pictures::SETTLE;
    placer
        .sync_at(
            &m.pictures.wanted(),
            (buf.area.width, buf.area.height),
            buf,
            &mut out,
            later,
        )
        .unwrap();
    out
}

fn slot(wants: &[Want], s: Slot) -> Option<Want> {
    wants.iter().find(|w| w.slot == s).copied()
}

#[test]
fn the_sidebar_alone_wants_the_logo_over_plain_cells() {
    let mut m = model(Protocol::Iterm);
    let mut t = Terminal::new(TestBackend::new(160, 56)).unwrap();
    let buf = draw(&mut m, &mut t);
    let wants = m.pictures.wanted();
    let side = slot(&wants, Slot::Sidebar).expect("the sidebar's head");
    // 32 columns of 8 px, and a square of rows: 15 of 17 px.
    assert_eq!((side.area.width, side.area.height), (32, 15));
    assert_eq!(side.art, Art::Logo);
    assert_eq!(side.bg, m.theme.p.panel, "the Session panel's own fill");
    // The sidebar's alone: the welcome draws its wordmark in text.
    assert_eq!(wants.len(), 1);
    // Under the picture, plain cells on its own background: no glyph for it to hide.
    for y in side.area.top()..side.area.bottom() {
        for x in side.area.left()..side.area.right() {
            let c = &buf[(x, y)];
            assert_eq!(c.symbol(), " ", "at {x},{y}");
            assert_eq!(c.bg, side.bg.color(), "at {x},{y}");
        }
    }
    // A busy turn wants the same logo, and no tick of its own.
    m.outstanding = 1;
    draw(&mut m, &mut t);
    assert_eq!(
        slot(&m.pictures.wanted(), Slot::Sidebar).unwrap().art,
        Art::Logo
    );
    assert_eq!(
        m.next_tick(),
        Some(std::time::Duration::from_millis(
            mikata_tui::kit::spinner::FRAME_MS
        ))
    );
}

#[test]
fn no_picture_is_placed_while_an_approval_is_open() {
    let mut m = model(Protocol::Kitty);
    let mut t = Terminal::new(TestBackend::new(160, 56)).unwrap();
    let mut placer = Placer::new(&m.pictures);
    let buf = draw(&mut m, &mut t);
    sync(&mut placer, &m, &buf);
    assert_eq!(placer.sends, 1, "the sidebar alone");

    m.outstanding = 1;
    update(&mut m, Msg::Gate(card()));
    assert!(m.gate.is_some());
    // Before the frame that opens the card, Kitty's pictures are deleted: never one frame over it.
    let mut before = Vec::new();
    placer.before(m.gate.is_some(), &mut before).unwrap();
    let before = String::from_utf8(before).unwrap();
    assert!(before.contains(&pixels::kitty_delete(Slot::Sidebar.id())));
    let buf = draw(&mut m, &mut t);
    assert_eq!(
        m.pictures.wanted(),
        Vec::new(),
        "nothing wanted under the card"
    );
    assert_eq!(
        sync(&mut placer, &m, &buf),
        b"",
        "nothing sent while it is open"
    );

    // Denied, the card goes, and the logo comes back: sent again, built once.
    update(&mut m, Msg::Key(KeyEvent::from(KeyCode::Esc)));
    assert!(m.gate.is_none());
    let buf = draw(&mut m, &mut t);
    sync(&mut placer, &m, &buf);
    assert_eq!(placer.sends, 2);
    assert_eq!(sync(&mut placer, &m, &buf), b"", "nothing more once shown");
    assert_eq!(placer.builds, 1, "the logo, built once");
}

#[test]
fn the_logo_is_never_sent_again_whoever_works() {
    let mut m = model(Protocol::Iterm);
    let mut t = Terminal::new(TestBackend::new(160, 56)).unwrap();
    let mut placer = Placer::new(&m.pictures);
    let buf = draw(&mut m, &mut t);
    let first = String::from_utf8(sync(&mut placer, &m, &buf)).unwrap();
    let side = slot(&m.pictures.wanted(), Slot::Sidebar).unwrap();
    let at = format!(
        "\x1b7\x1b[{};{}H\x1b]1337;File=",
        side.area.y + 1,
        side.area.x + 1
    );
    assert!(first.contains(&at), "the logo at the sidebar's head");
    // Idle frames, then a turn, then a commissioned orc working: nothing is sent again.
    for _ in 0..5 {
        let buf = draw(&mut m, &mut t);
        assert_eq!(sync(&mut placer, &m, &buf), b"");
    }
    m.outstanding = 1;
    update(
        &mut m,
        Msg::Below {
            event: Event::TurnStarted {
                text: "judge".into(),
            },
            chain: vec!["orc-sql".into()],
        },
    );
    for _ in 0..5 {
        update(&mut m, Msg::Tick(std::time::Duration::from_millis(33)));
        let buf = draw(&mut m, &mut t);
        assert_eq!(
            sync(&mut placer, &m, &buf),
            b"",
            "the same logo for the orc"
        );
    }
    assert_eq!(placer.sends, 1);
}

#[test]
fn the_sidebar_folding_takes_its_picture_away() {
    let mut m = model(Protocol::Sixel);
    let mut t = Terminal::new(TestBackend::new(160, 56)).unwrap();
    draw(&mut m, &mut t);
    assert!(slot(&m.pictures.wanted(), Slot::Sidebar).is_some());
    // Under 100 columns the sidebar folds.
    let mut narrow = Terminal::new(TestBackend::new(90, 50)).unwrap();
    draw(&mut m, &mut narrow);
    assert!(slot(&m.pictures.wanted(), Slot::Sidebar).is_none());
    // Too short for the head: no picture either.
    let mut short = Terminal::new(TestBackend::new(120, 30)).unwrap();
    draw(&mut m, &mut short);
    assert!(slot(&m.pictures.wanted(), Slot::Sidebar).is_none());
}

#[test]
fn without_images_no_logo_is_drawn() {
    let mut m = model(Protocol::Blocks);
    let mut t = Terminal::new(TestBackend::new(160, 56)).unwrap();
    let buf = draw(&mut m, &mut t);
    assert_eq!(m.pictures.wanted(), Vec::new());
    // The sidebar's head: no picture, and none built from characters either. The sidebar starts
    // at the rightmost panel corner; the transcript's cards, left of it,
    // are made of half blocks of their own.
    let head = |buf: &Buffer| -> String {
        let left = (0..buf.area.width)
            .rev()
            .find(|&x| (0..6).any(|y| buf[(x, y)].symbol() == "┏"))
            .unwrap_or(85);
        (left..buf.area.width)
            .flat_map(|x| (2..14).map(move |y| (x, y)))
            .map(|(x, y)| buf[(x, y)].symbol().to_owned())
            .collect()
    };
    let blocks = |s: &str| s.chars().any(|c| ('\u{2580}'..='\u{259f}').contains(&c));
    assert!(!blocks(&head(&buf)), "idle");
    m.outstanding = 1;
    update(&mut m, Msg::Tick(std::time::Duration::from_millis(500)));
    assert!(!blocks(&head(&draw(&mut m, &mut t))), "working");
}

#[test]
fn a_late_probe_reply_never_answers_a_card_nor_types() {
    let mut m = model(Protocol::Blocks);
    let t0 = std::time::Instant::now();
    m.late = mikata_tui::probe::Late::after(t0);
    let soon = t0 + std::time::Duration::from_millis(300);
    // Kitty's reply, late, as the input reads it: Alt+_, `Gi=31;OK`, Alt+\.
    let mut reply = vec![KeyEvent::new(
        KeyCode::Char('_'),
        ratatui::crossterm::event::KeyModifiers::ALT,
    )];
    reply.extend("Gi=31;OK".chars().map(|c| KeyEvent::from(KeyCode::Char(c))));
    reply.push(KeyEvent::new(
        KeyCode::Char('\\'),
        ratatui::crossterm::event::KeyModifiers::ALT,
    ));
    for k in &reply {
        assert!(update(&mut m, Msg::KeyAt(*k, soon)).is_empty());
    }
    assert_eq!(m.input.text(), "", "nothing typed");
    m.outstanding = 1;
    update(&mut m, Msg::Gate(card()));
    for k in [KeyCode::Char('1'), KeyCode::Char('y'), KeyCode::Enter] {
        let cmds = update(&mut m, Msg::KeyAt(KeyEvent::from(k), soon));
        assert!(cmds.is_empty(), "{k:?} reached the card");
    }
    assert!(m.gate.is_some(), "the card still waits for the human");
}

#[test]
fn ctrl_c_and_esc_always_pass_a_late_window() {
    use mikata_tui::app::Cmd;
    use ratatui::crossterm::event::KeyModifiers;
    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    let t0 = std::time::Instant::now();
    let soon = t0 + std::time::Duration::from_millis(300);

    // A card open in the window: Esc denies it.
    let mut m = model(Protocol::Blocks);
    m.late = mikata_tui::probe::Late::after(t0);
    m.outstanding = 1;
    update(&mut m, Msg::Gate(card()));
    let cmds = update(&mut m, Msg::KeyAt(KeyEvent::from(KeyCode::Esc), soon));
    assert!(
        cmds.iter().any(|c| matches!(c, Cmd::Answer(..))),
        "Esc denies"
    );
    assert!(m.gate.is_none());

    // Ctrl+C denies a card too.
    update(&mut m, Msg::Gate(card()));
    let cmds = update(&mut m, Msg::KeyAt(ctrl_c, soon));
    assert!(
        cmds.iter().any(|c| matches!(c, Cmd::Answer(..))),
        "Ctrl+C denies"
    );
    assert!(m.gate.is_none());

    // Idle, no card: Ctrl+C quits, at once and in the window.
    let mut idle = model(Protocol::Blocks);
    idle.late = mikata_tui::probe::Late::after(t0);
    let mut quit = false;
    for _ in 0..2 {
        quit |= update(&mut idle, Msg::KeyAt(ctrl_c, soon))
            .iter()
            .any(|c| matches!(c, Cmd::Quit));
    }
    assert!(quit, "Ctrl+C quits in the window");

    // A plain `c` or `t`, the tail of a reply, is still dropped.
    let mut m = model(Protocol::Blocks);
    m.late = mikata_tui::probe::Late::after(t0);
    for c in ['c', 't'] {
        update(&mut m, Msg::KeyAt(KeyEvent::from(KeyCode::Char(c)), soon));
    }
    assert_eq!(m.input.text(), "");
}
