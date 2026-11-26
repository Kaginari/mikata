//! `mikata` — a Mikata session in the terminal, the core hosted in this process: the terminal
//! app with no subcommand, one ask without it with `run`.

use std::io::{IsTerminal, Read};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use mikata_loop::{Law, Mode};
use mikata_tui::app::Model;
use mikata_tui::cli::{Command, HELP, VERSION, parse};
use mikata_tui::headless;
use mikata_tui::kit::highlight;
use mikata_tui::pictures::Pictures;
use mikata_tui::probe::{Late, probe};
use mikata_tui::run::{Host, channel, gate, header, observer, run};
use mikata_tui::theme::{Scheme, Theme, Variant};
use mikata_tui::transcript::Tag;

fn main() -> ExitCode {
    // First, before any key is read: the harness's environment and memory close to every
    // non-root process, sandboxed or not.
    let hardened = mikata_server::harden_self()
        .map_err(|e| format!("this process could not be made non-dumpable ({e}): other processes of the user may read its environment, the provider key included"));
    if let Err(e) = &hardened {
        eprintln!("mikata: warning: {e}");
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match parse(&args) {
        Ok(c) => c,
        Err(e) => return fail(&e, 2),
    };
    match command {
        Command::Help => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Command::Version => {
            println!("{VERSION}");
            ExitCode::SUCCESS
        }
        Command::Probe => {
            print!("{}", probe(&|k| std::env::var(k).ok()).report());
            ExitCode::SUCCESS
        }
        Command::Chat {
            world,
            resume,
            theme,
            mode,
            web,
        } => chat(
            world,
            resume.as_deref(),
            theme.as_deref(),
            mode,
            web,
            hardened.err(),
        ),
        Command::Run {
            ask,
            world,
            json,
            gate_tty,
            no_law,
            mode,
        } => {
            let units = match mikata_tui::cli::units(&args) {
                Ok(u) => u.unwrap_or_default(),
                Err(e) => return fail(&e, 2),
            };
            let profile = match mikata_tui::cli::profile(&args) {
                Ok(p) => p.unwrap_or_default(),
                Err(e) => return fail(&e, 2),
            };
            ask_once(
                ask,
                world,
                json,
                gate_tty,
                (
                    if no_law { Law::Off } else { Law::On },
                    mode,
                    units,
                    profile,
                ),
            )
        }
        Command::Status { world, mode } => status(world, mode),
    }
}

/// The world: the directory given, or the current one, canonical.
fn world_of(world: Option<PathBuf>) -> Result<PathBuf, String> {
    let world = world
        .map_or_else(std::env::current_dir, Ok)
        .map_err(|e| format!("the current directory: {e}"))?;
    match world.canonicalize() {
        Ok(w) if w.is_dir() => {
            // Mnemorph's folders take Mikata's names the first time Mikata runs.
            for said in mikata_tools::dirs::migrate_homes()
                .into_iter()
                .chain(mikata_tools::dirs::migrate_world(&w))
            {
                eprintln!("mikata: {said}");
            }
            Ok(w)
        }
        Ok(w) => Err(format!("{} is not a directory", w.display())),
        Err(e) => Err(format!("{}: {e}", world.display())),
    }
}

/// `run`: one ask, without the app.
fn ask_once(
    ask: Option<String>,
    world: Option<PathBuf>,
    json: bool,
    gate_tty: bool,
    (law, mode, units, profile): (Law, Option<Mode>, mikata_loop::config::Composition, String),
) -> ExitCode {
    let ask = match ask {
        Some(a) => a,
        None if std::io::stdin().is_terminal() => {
            return fail("run needs an ask: mikata run \"<ask>\", or one on stdin", 2);
        }
        None => {
            let mut text = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut text) {
                return fail(&format!("stdin: {e}"), 2);
            }
            text
        }
    };
    let ask = ask.trim();
    if ask.is_empty() {
        return fail("run: nothing to ask", 2);
    }
    let world = match world_of(world) {
        Ok(w) => w,
        Err(e) => return fail(&e, 2),
    };
    let (tx, rx) = headless::channel();
    let gate = headless::gate(tx.clone(), gate_tty);
    // The law switch and the mode asked for are the human's command line alone: no configuration
    // a world holds sets either.
    // The composition asked for is read as the session opens: a unit of the prompt cannot wait.
    let asked = units.to_string();
    let opened = mikata_loop::open_mode(
        &world,
        mode,
        law,
        gate.clone(),
        headless::observer(tx.clone()),
        |v| match v {
            "MIKATA_UNITS" if !asked.is_empty() => Some(asked.clone()),
            "MIKATA_PROFILE" if !profile.is_empty() => Some(profile.clone()),
            _ => std::env::var(v).ok(),
        },
    );
    let opened = match opened {
        Ok(o) => o,
        Err(e) => return fail(&e, 2),
    };

    #[cfg(unix)]
    if let Err(e) = headless::on_ctrl_c(opened.session.interrupt(), gate) {
        return fail(&format!("ctrl+c could not be caught: {e}"), 2);
    }
    let options = headless::Options {
        json,
        escape: std::io::stdout().is_terminal(),
    };
    let (mut out, mut err) = (std::io::stdout().lock(), std::io::stderr());
    match headless::run(opened, ask, options, (tx, rx), &mut out, &mut err) {
        Ok(code) => ExitCode::from(code),
        Err(e) => fail(&e, 1),
    }
}

/// `status`: what a session here would open with.
fn status(world: Option<PathBuf>, mode: Option<Mode>) -> ExitCode {
    let world = match world_of(world) {
        Ok(w) => w,
        Err(e) => return fail(&e, 2),
    };
    let opened = mikata_loop::open_mode(
        &world,
        mode,
        Law::On,
        Arc::new(mikata_tools::NoHuman),
        Arc::new(|_: &mikata_loop::Seen| {}),
        |v| std::env::var(v).ok(),
    );
    match opened {
        Ok(opened) => {
            for line in headless::status(&opened) {
                println!("{line}");
            }
            for line in headless::verify(&world) {
                println!("{line}");
            }
            for line in headless::roster(&world) {
                println!("{line}");
            }
            for line in headless::minds(&world) {
                println!("{line}");
            }
            for line in headless::meters(&world) {
                println!("{line}");
            }
            for line in headless::guts(&world) {
                println!("{line}");
            }
            for line in headless::envelopes(&world) {
                println!("{line}");
            }
            for line in headless::evolve(&world) {
                println!("{line}");
            }
            for line in headless::ci(&world) {
                println!("{line}");
            }
            for line in headless::scout(&world) {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("{:<9}{}", "mikata", env!("CARGO_PKG_VERSION"));
            println!(
                "{:<9}did not open — {}",
                "session",
                mikata_tui::safe::one_line(&e)
            );
            // The mode needs no key: what a session here would open in, or why it cannot.
            println!("{}", headless::mode(&world, mode));
            for line in headless::verify(&world) {
                println!("{line}");
            }
            for line in headless::roster(&world) {
                println!("{line}");
            }
            for line in headless::minds(&world) {
                println!("{line}");
            }
            for line in headless::meters(&world) {
                println!("{line}");
            }
            for line in headless::guts(&world) {
                println!("{line}");
            }
            for line in headless::envelopes(&world) {
                println!("{line}");
            }
            for line in headless::evolve(&world) {
                println!("{line}");
            }
            for line in headless::ci(&world) {
                println!("{line}");
            }
            for line in headless::scout(&world) {
                println!("{line}");
            }
            ExitCode::FAILURE
        }
    }
}

/// The terminal app.
fn chat(
    world: Option<PathBuf>,
    resume: Option<&str>,
    theme: Option<&str>,
    mode: Option<Mode>,
    web: bool,
    unhardened: Option<String>,
) -> ExitCode {
    let world = match world_of(world) {
        Ok(w) => w,
        Err(e) => return fail(&e, 1),
    };
    // The first run in a fresh or cloned world: its cards' verify lines are pinned, or not, by
    // the human's one answer, before the gate is built from them.
    // The creatures are adopted, or not, first: a stranger runs its own seat alone.
    if std::io::stdin().is_terminal() {
        let _ = mikata_tui::adopt::offer(
            &world,
            &mut std::io::stdout(),
            &mut mikata_tui::adopt::at_terminal,
        );
        let _ = mikata_tui::pin::offer(
            &world,
            &mut std::io::stdout(),
            &mut mikata_tui::pin::at_terminal,
        );
    }
    // `--web` is the board instead of the terminal app: one or the other, never both.
    if web {
        return web_only(&world, resume.is_some());
    }
    let variant = Variant::choose(theme, |k| std::env::var(k).ok());
    let scheme = Scheme::choose(theme, |k| std::env::var(k).ok());
    highlight::warm();
    let (tx, rx) = channel();
    // The session opens before the terminal is taken, so a missing key reads as plain text.
    let env = |v: &str| std::env::var(v).ok();
    let opened = match resume {
        Some(run) => mikata_loop::resume_mode(
            &world,
            run,
            mode,
            gate(tx.clone()),
            observer(tx.clone()),
            env,
        ),
        None => mikata_loop::open_mode(
            &world,
            mode,
            Law::On,
            gate(tx.clone()),
            observer(tx.clone()),
            env,
        ),
    };
    let opened = match opened {
        Ok(o) => o,
        Err(e) => return fail(&e, 1),
    };
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let mut model = Model::new(header(&opened, home.as_deref()), variant, (80, 24));
    model.theme = Theme::of(scheme, variant);
    model.sidebar.root = Some(opened.world.clone());
    model.session_id.clone_from(&opened.session_id);
    // The session's facts, one tagged row each: what matters sharp, the rest quiet. The full
    // account stays where it was: `mikata status`, the journal's `law` beat, `--help`.
    model.fact(Tag::Model, &opened.model, &opened.provider);
    model.fact(Tag::Sandbox, &opened.sandbox, "");
    // The human's approval switched off is said first and in gold: acts leave unseen.
    if opened.approval.starts_with("OFF") {
        model.fact(Tag::Warn, &opened.approval, "");
    }
    model.sandbox.clone_from(&opened.sandbox);
    model.fact(Tag::Mode, opened.mode.word(), &opened.mode_why);
    if let Some(e) = unhardened {
        model.fact(Tag::Warn, e, "");
    }
    if resume.is_some() {
        model.fact(
            Tag::Resume,
            format!("run {}", opened.run_id),
            "taken up from its journal: ask to go on",
        );
    }
    for hole in &opened.holes {
        model.fact(Tag::Config, hole, "");
    }
    // What the terminal draws, asked before the screen is taken: within 200 ms, never blocking.
    let caps = probe(&|k| std::env::var(k).ok());
    for note in &caps.notes {
        model.fact(Tag::Images, note, "");
    }
    model.pictures = Pictures::new(caps.protocol, caps.cell);
    if caps.timed_out {
        model.late = Late::after(std::time::Instant::now());
    }
    let host = match Host::start(opened, tx.clone()) {
        Ok(h) => h,
        Err(e) => return fail(&e, 1),
    };
    model.units = Some(host.unit_desk());
    // The sidebar's memory and races come from the world, read off the UI thread.
    host.run(model.survey());
    let result = run(model, &host, tx, &rx);
    host.stop();
    // The session's runs fill evolution's windows: one tick after it, in the background.
    #[cfg(unix)]
    mikata_server::board::autostart::refresh_after(&world);
    #[cfg(unix)]
    if let Some(line) = mikata_server::board::autostart::evolve_after(&world) {
        eprintln!("mikata: {}", mikata_tui::safe::one_line(&line));
    }
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => fail(&format!("the terminal: {e}"), 1),
    }
}

/// `mikata --web`: make sure a board runs with this world on it and open it in the browser, then
/// leave — the board is the app, and its sessions are its own; no terminal app opens beside it.
/// Its one status line never holds a key. `board.port` is read; a run to resume is taken up on the
/// board, not here.
fn web_only(world: &std::path::Path, resume: bool) -> ExitCode {
    if resume {
        return fail(
            "--web opens the board, not a run: open the board and pick the run under Runs",
            2,
        );
    }
    let paths = mikata_loop::Paths::resolve(world);
    let port = mikata_loop::Config::load_global(&paths)
        .ok()
        .and_then(|c| c.board().port);
    #[cfg(unix)]
    {
        use mikata_server::board::autostart;
        let ask = autostart::Ask { open: true, port };
        let status = match autostart::sibling() {
            Ok(d) => autostart::ensure(&d, world, ask, std::time::Duration::from_secs(5)),
            Err(e) => return fail(&format!("board: not started: {e}"), 1),
        };
        println!("mikata: {}", mikata_tui::safe::one_line(&status));
        if status.contains("not started") {
            return ExitCode::FAILURE;
        }
        ExitCode::SUCCESS
    }
    #[cfg(not(unix))]
    {
        let _ = (world, port);
        fail(
            "--web needs a Unix socket for the board; run `mikatad board --open`",
            1,
        )
    }
}

/// Say why on stderr, as `mikata: <why>` — a script reads the last such line — and exit `code`.
fn fail(why: &str, code: u8) -> ExitCode {
    eprintln!("mikata: {}", mikata_tui::safe::one_line(why));
    ExitCode::from(code)
}
