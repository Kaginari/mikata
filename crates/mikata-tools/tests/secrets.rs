//! The secrets unit as the tools hold it: a secret file's shape and never its values, no tool
//! writing or searching one, a command finding it empty, and every value struck from results.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mikata_races::Race;
use mikata_tools::{Context, Outcome, Secrets, call, registry};

const TOKEN: &str = "hvs.CAESIJ9realVaultRootToken";

fn world(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("mikata-secrets-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("world/src")).unwrap();
    fs::create_dir_all(root.join("home")).unwrap();
    let w = root.join("world");
    fs::write(
        w.join(".env"),
        format!("VAULT_TOKEN={TOKEN}\nexport DB_PASSWORD='p4ssw0rd-long-enough'\nDEBUG=1\n"),
    )
    .unwrap();
    fs::write(w.join(".env.example"), "VAULT_TOKEN=\n").unwrap();
    fs::write(
        w.join("src/app.py"),
        "import os\ntoken = os.environ['VAULT_TOKEN']\n",
    )
    .unwrap();
    // A copy of the token somewhere no pattern marks: struck all the same.
    fs::write(w.join("src/debug.log"), format!("using {TOKEN} now\n")).unwrap();
    root
}

fn cx(root: &Path) -> Context {
    let w = root.join("world");
    let secrets = Secrets::scan(&w.canonicalize().unwrap(), &[], &[]);
    Context::new(Race::Slime, &w, &["."])
        .with_home(root.join("home"))
        .with_secrets(Arc::new(secrets))
}

fn tool(cx: &mut Context, name: &str, args: &serde_json::Value) -> Outcome {
    call(&registry(), name, &args.to_string(), cx)
}

#[test]
fn a_secret_file_gives_its_keys_and_never_a_value() {
    let root = world("read");
    let mut cx = cx(&root);
    let o = tool(&mut cx, "read", &serde_json::json!({"path": ".env"}));
    assert!(!o.is_error, "{o:?}");
    assert!(
        o.content.contains("VAULT_TOKEN = ‹secret›"),
        "{}",
        o.content
    );
    assert!(
        o.content.contains("DB_PASSWORD = ‹secret›"),
        "{}",
        o.content
    );
    assert!(!o.content.contains("hvs.") && !o.content.contains("p4ssw0rd"));
    // An example a project commits is read as it is.
    let o = tool(
        &mut cx,
        "read",
        &serde_json::json!({"path": ".env.example"}),
    );
    assert!(o.content.contains("VAULT_TOKEN="), "{}", o.content);
    // index gives the same shape.
    let o = tool(&mut cx, "index", &serde_json::json!({"path": ".env"}));
    assert!(!o.content.contains(TOKEN), "{}", o.content);
}

#[test]
fn no_tool_writes_moves_or_deletes_a_secret_file() {
    let root = world("write");
    let mut cx = cx(&root);
    for (name, args) in [
        (
            "write",
            serde_json::json!({"path": ".env", "content": "X=1\n"}),
        ),
        (
            "edit",
            serde_json::json!({"path": ".env", "old_string": "DEBUG=1", "new_string": "DEBUG=0"}),
        ),
        ("delete", serde_json::json!({"path": ".env"})),
        (
            "move",
            serde_json::json!({"from": ".env", "to": "src/env.txt"}),
        ),
        // Nor is another file moved into a secret's name.
        (
            "move",
            serde_json::json!({"from": "src/app.py", "to": ".env.local"}),
        ),
    ] {
        let o = tool(&mut cx, name, &args);
        assert!(o.is_error, "{name}: {o:?}");
        assert!(o.content.contains("secret file"), "{name}: {}", o.content);
    }
    let kept = fs::read_to_string(root.join("world/.env")).unwrap();
    assert!(kept.contains(TOKEN), "the file stands as it was");
}

#[test]
fn grep_passes_over_a_secret_file_and_every_value_is_struck() {
    let root = world("grep");
    let mut cx = cx(&root);
    let o = tool(&mut cx, "grep", &serde_json::json!({"pattern": "VAULT"}));
    assert!(o.content.contains("src/app.py"), "{}", o.content);
    assert!(!o.content.contains(".env:"), "{}", o.content);
    assert!(
        o.content.contains("1 secret file not searched"),
        "{}",
        o.content
    );
    // The token copied into a file no pattern marks never reaches the model either.
    let o = tool(
        &mut cx,
        "read",
        &serde_json::json!({"path": "src/debug.log"}),
    );
    assert!(!o.content.contains(TOKEN), "{}", o.content);
    assert!(o.content.contains("using ‹secret› now"), "{}", o.content);
}

#[test]
fn with_the_unit_off_a_secret_file_is_a_file_like_any_other() {
    let root = world("off");
    let mut cx = cx(&root);
    cx.set_secrets_on(false);
    let o = tool(&mut cx, "read", &serde_json::json!({"path": ".env"}));
    assert!(o.content.contains(TOKEN), "{}", o.content);
}

/// A deploy the human approved reads the secret files as they are — it cannot do its work
/// without them — while a command nobody approved finds them empty; what either prints of a
/// value is struck.
#[cfg(target_os = "linux")]
#[test]
fn an_approved_act_reads_the_secret_files_and_nothing_else_does() {
    use mikata_sandbox::Sandbox;
    use mikata_tools::Answer;
    use mikata_tools::gate::Scripted;
    let exe = std::env::current_exe().unwrap();
    let guard = exe.parent().unwrap().parent().unwrap().join("mikata-guard");
    if !guard.is_file() {
        eprintln!("skipped: build the workspace so target/<profile>/mikata-guard exists");
        return;
    }
    let sandbox = Sandbox::Guard(guard);
    if !sandbox.private_tmp() {
        eprintln!("skipped: this kernel lets no unprivileged process mount in a user namespace");
        return;
    }
    let root = world("deploy");
    let outside = root.join("outside");
    fs::create_dir_all(&outside).unwrap();
    let gate = Arc::new(Scripted::new([Answer::Approve]));
    // A deploy post: its acts write under the deploy root, outside the world, and are asked.
    let w = root.join("world");
    let secrets = Secrets::scan(&w.canonicalize().unwrap(), &[], &[]);
    let mut cx = Context::new(Race::Kijin, &w, &["."])
        .with_body("kijin-deploy")
        .with_deploy_roots(std::slice::from_ref(&outside))
        .with_gate(gate.clone())
        .with_home(root.join("home"))
        .with_sandbox(sandbox)
        .with_secrets(Arc::new(secrets));
    // Nobody approved this one: the file is empty to it.
    let o = tool(
        &mut cx,
        "bash",
        &serde_json::json!({"command": "wc -c < .env"}),
    );
    assert_eq!(o.content.trim(), "0", "{o:?}");
    assert_eq!(gate.asked().len(), 0);
    // The deploy is approved: it reads the real file, and its output is struck.
    let sent = outside.join("sent");
    let o = tool(
        &mut cx,
        "bash",
        &serde_json::json!({"command": format!("cp .env {} && cat .env", sent.display())}),
    );
    assert!(!o.is_error, "{o:?}");
    assert_eq!(gate.asked().len(), 1, "the human was asked");
    // The card said first that the act names a secret file.
    let why = &gate.asked()[0].why;
    assert!(
        why.first()
            .is_some_and(|w| w.contains("names secret files (.env)")),
        "{why:?}"
    );
    assert!(
        fs::read_to_string(&sent).unwrap().contains(TOKEN),
        "the approved act had the real file"
    );
    assert!(!o.content.contains(TOKEN), "{}", o.content);
    assert!(o.content.contains("‹secret›"), "{}", o.content);
}

/// Under the guard, where the kernel allows a mount namespace, a command finds a secret file
/// empty, and what it prints of a copy is struck.
#[cfg(target_os = "linux")]
#[test]
fn a_command_finds_a_secret_file_empty_under_the_guard() {
    use mikata_sandbox::Sandbox;
    let exe = std::env::current_exe().unwrap();
    let guard = exe.parent().unwrap().parent().unwrap().join("mikata-guard");
    if !guard.is_file() {
        eprintln!("skipped: build the workspace so target/<profile>/mikata-guard exists");
        return;
    }
    let sandbox = Sandbox::Guard(guard);
    if !sandbox.private_tmp() {
        eprintln!("skipped: this kernel lets no unprivileged process mount in a user namespace");
        return;
    }
    let root = world("bash");
    let mut cx = cx(&root).with_sandbox(sandbox);
    let o = tool(
        &mut cx,
        "bash",
        &serde_json::json!({"command": "wc -c < .env; cat src/debug.log; cat src/app.py"}),
    );
    assert!(!o.is_error, "{o:?}");
    let lines: Vec<&str> = o.content.lines().collect();
    assert_eq!(lines.first().map(|l| l.trim()), Some("0"), "{}", o.content);
    assert!(!o.content.contains(TOKEN), "{}", o.content);
    assert!(
        o.content.contains("os.environ"),
        "the rest reads: {}",
        o.content
    );
    // And the file outside the command is as it was.
    assert!(
        fs::read_to_string(root.join("world/.env"))
            .unwrap()
            .contains(TOKEN)
    );
}
