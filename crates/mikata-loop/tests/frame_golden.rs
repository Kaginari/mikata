//! The frame — the system prompt and the tools — a session opens with, kept as golden files.
//!
//! Moving a feature out of the loop into a unit must not move one byte the body is sent: the
//! frame is the cached prefix of every request, and every measured run rode it. Each case opens a
//! session through the real open path, takes the first request's frame, puts placeholders where a
//! run's own place or day shows, and compares it with `tests/golden/frame-<case>.json`.
//! `MIKATA_BLESS=1` writes the files instead; a blessed change is a change to what the body is
//! sent, and its commit says why.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mikata_loop::open::{Bound, open_with};
use mikata_loop::{Config, Paths, ScriptedModel};
use serde_json::{Value, json};

/// A fresh world for one case, with the files the case writes into it.
fn world(case: &str, files: &[(&str, &str)]) -> PathBuf {
    let w = std::env::temp_dir().join(format!("mikata-frame-{case}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&w);
    fs::create_dir_all(w.join(".mikata")).unwrap();
    for (path, text) in files {
        let at = w.join(path);
        fs::create_dir_all(at.parent().unwrap()).unwrap();
        fs::write(at, text).unwrap();
    }
    w
}

/// The frame the first request of a session in `w` carries, as JSON with the world's path and
/// the attic's replaced by placeholders.
async fn frame(w: &Path, global: Option<&str>) -> Value {
    frame_with(w, global, &[]).await
}

/// [`frame`], the human's environment carrying `env` (`MIKATA_UNITS`, `MIKATA_PROFILE`).
async fn frame_with(w: &Path, global: Option<&str>, env: &[(&str, &str)]) -> Value {
    let model = Arc::new(ScriptedModel::new([ScriptedModel::text("done")]));
    let global_path = w.join("../").join(format!(
        "{}-global.toml",
        w.file_name().unwrap().to_string_lossy()
    ));
    match global {
        Some(text) => fs::write(&global_path, text).unwrap(),
        None => {
            let _ = fs::remove_file(&global_path);
        }
    }
    let config = Config::load(&Paths::with_global(global_path.clone(), w)).unwrap();
    let attic = w.with_extension("attic");
    let mut opened = open_with(
        w,
        &config,
        Bound {
            model: model.clone(),
            model_id: "scripted-model",
            attic: Some(&attic),
        },
        Arc::new(mikata_tools::NoHuman),
        Arc::new(|_: &mikata_loop::Seen| {}),
        |v| {
            env.iter()
                .find(|(k, _)| *k == v)
                .map(|(_, x)| (*x).to_owned())
        },
    )
    .unwrap();
    opened.session.run().ask("say done", &mut |_| {}).await;
    let request = &model.requests()[0];
    let tools: Vec<Value> = request
        .tools
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "input_schema": t.input_schema,
                "cache": t.cache,
                "defer": t.defer,
                "fill": t.fill,
            })
        })
        .collect();
    let system: Vec<Value> = request
        .system
        .iter()
        .map(|b| match b {
            mikata_provider::Block::Text { text, cache } => json!({"text": text, "cache": cache}),
            other => json!({"other": format!("{other:?}")}),
        })
        .collect();
    let text = serde_json::to_string_pretty(&json!({
        "system": system,
        "tools": tools,
    }))
    .unwrap();
    let text = text
        .replace(&attic.display().to_string(), "<attic>")
        .replace(&w.display().to_string(), "<world>")
        .replace(&global_path.display().to_string(), "<global>");
    let _ = fs::remove_dir_all(w);
    let _ = fs::remove_dir_all(&attic);
    let _ = fs::remove_file(&global_path);
    serde_json::from_str(&text).unwrap()
}

/// Compares `got` with the golden file of `case`, or writes it under `MIKATA_BLESS=1`.
fn golden(case: &str, got: &Value) {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/golden/frame-{case}.json"));
    let text = format!("{}\n", serde_json::to_string_pretty(got).unwrap());
    if std::env::var_os("MIKATA_BLESS").is_some() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        return;
    }
    let want = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}; run with MIKATA_BLESS=1 once", path.display()));
    assert!(
        want == text,
        "the frame of `{case}` moved: what the body is sent changed. Diff {} against the test's output, and bless it only with a reason in the commit.\n--- got ---\n{text}",
        path.display()
    );
}

#[tokio::test]
async fn a_bare_world_opens_with_its_golden_frame() {
    let w = world("bare", &[]);
    golden("bare", &frame(&w, None).await);
}

#[tokio::test]
async fn a_world_with_a_law_opens_with_its_golden_frame() {
    let law =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.mikata/law.md"))
            .unwrap();
    let w = world("law", &[(".mikata/law.md", &law)]);
    golden("law", &frame(&w, None).await);
}

#[tokio::test]
async fn a_world_with_an_mcp_server_opens_with_its_golden_frame() {
    let w = world("mcp", &[]);
    let global =
        "[mcp.servers.notes]\ncommand = \"/bin/false\"\nraces = [\"slime\", \"summoner\"]\n";
    golden("mcp", &frame(&w, Some(global)).await);
}

/// The `plan` unit off in the human's global file: the frame loses its two bullets, and nothing
/// else; on in the default, the frame is the bare world's.
#[tokio::test]
async fn the_plan_unit_off_drops_its_two_bullets_and_nothing_else() {
    let off = frame(&world("plan-off", &[]), Some("[units]\nplan = false\n")).await;
    golden("plan-off", &off);
    let on = frame(&world("plan-on", &[]), None).await;
    let text = |v: &Value| v["system"][0]["text"].as_str().unwrap().to_owned();
    let (on_text, off_text) = (text(&on), text(&off));
    for bullet in mikata_loop::solo::PLAN_BULLETS {
        assert!(on_text.contains(bullet), "{bullet}");
        assert!(!off_text.contains(bullet), "{bullet}");
    }
    let kept: Vec<&str> = on_text
        .lines()
        .filter(|l| {
            !mikata_loop::solo::PLAN_BULLETS
                .iter()
                .any(|b| l.starts_with(b))
        })
        .collect();
    assert_eq!(kept.join("\n"), off_text);
    assert_eq!(on["tools"], off["tools"], "the tools are the same");
}

/// The `plan` unit off in the human's global file is off in the run as well as in the prompt:
/// the session's journal says so before the first ask is answered.
#[tokio::test]
async fn the_plan_unit_off_is_journaled_as_the_session_opens() {
    let w = world("plan-journal", &[]);
    let global = w.with_extension("toml");
    fs::write(&global, "[units]\nplan = false\n").unwrap();
    let model = Arc::new(ScriptedModel::new([ScriptedModel::text("done")]));
    let config = Config::load(&Paths::with_global(global.clone(), &w)).unwrap();
    let attic = w.with_extension("attic");
    let mut opened = open_with(
        &w,
        &config,
        Bound {
            model,
            model_id: "scripted-model",
            attic: Some(&attic),
        },
        Arc::new(mikata_tools::NoHuman),
        Arc::new(|_: &mikata_loop::Seen| {}),
        |_| None,
    )
    .unwrap();
    assert!(
        opened
            .session
            .run()
            .units()
            .active()
            .iter()
            .all(|(id, _)| *id != "plan"),
        "off in the run before any ask"
    );
    opened.session.run().ask("say done", &mut |_| {}).await;
    let journal = fs::read_dir(w.join(".mikata/instruments/loop"))
        .unwrap()
        .map(|e| fs::read_to_string(e.unwrap().path()).unwrap())
        .collect::<String>();
    assert!(
        journal.contains(r#""beat":"units","op":"unload","id":"plan""#),
        "{journal}"
    );
    let _ = fs::remove_dir_all(&w);
    let _ = fs::remove_dir_all(&attic);
    let _ = fs::remove_file(&global);
}

/// The plan off on the command line (`--units -plan`) or by a profile in the global file is the
/// same frame as off in `[units]`: the prompt is made with the composition the human asked for.
#[tokio::test]
async fn the_plan_off_by_the_command_line_or_a_profile_is_the_same_frame() {
    let by_list = frame_with(&world("plan-cli", &[]), None, &[("MIKATA_UNITS", "-plan")]).await;
    golden("plan-off", &by_list);
    let profile = "[profiles.small-fix]\nunits = \"-plan\"\n";
    let by_profile = frame_with(
        &world("plan-profile", &[]),
        Some(profile),
        &[("MIKATA_PROFILE", "small-fix")],
    )
    .await;
    golden("plan-off", &by_profile);
    // The command line goes over the profile: `+plan` keeps the bullets.
    let back = frame_with(
        &world("plan-back", &[]),
        Some(profile),
        &[("MIKATA_PROFILE", "small-fix"), ("MIKATA_UNITS", "+plan")],
    )
    .await;
    golden("bare", &back);
}

/// The build's `small-fix` profile is the plan off, with no line in the human's file.
#[tokio::test]
async fn the_builds_small_fix_profile_is_the_plan_off() {
    let w = world("plan-builtin", &[]);
    golden(
        "plan-off",
        &frame_with(&w, None, &[("MIKATA_PROFILE", "small-fix")]).await,
    );
}
