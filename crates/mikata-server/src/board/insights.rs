//! The board's insights: what each ask in a workspace's session runs did, what it cost and how it
//! ended — `GET /api/insights?world=<id|all>&window=7d|28d|all`. Read from the runs' journals (the
//! model and mode they rode, the units they changed, every turn, call, verdict and ending) and the
//! usage ledger, joined turn to row by the provider's message id. The page draws every chart from
//! these rows; nothing here is a model's word about itself.
//!
//! The journals live inside the world, which a body can write: they are read as the runs listing
//! reads them ([`super::runs`]): by session run id only, through a journals directory with no
//! symlink, each journal under a size cap and all of them under one budget.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use mikata_loop::beats::Beat;
use mikata_loop::resume::is_session_run;
use mikata_loop::world::ledger::{Timestamp, UsageRow};
use mikata_loop::world::spend::{self, Window};
use serde::Serialize;
use ts_rs::TS;

use super::BoardWorld;
use super::runs::{ENTRIES, SUMMARY_BYTES, beats, journals_dir, read_capped, rfc3339};

/// The most runs read, newest first.
pub const RUNS: usize = 300;

/// The bytes read from journals in all.
const BYTES: u64 = 96 << 20;

/// How an ask ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Outcome {
    /// The end gate passed the answer.
    Landed,
    /// The end gate refused the answer, and the ask ended so.
    Blocked,
    /// It answered, and nothing went through the gate: a question, or no change.
    Answered,
    /// It stopped before answering: a limit, an interrupt, a cut.
    Stopped,
    /// It failed: the model or the harness broke.
    Failed,
    /// It runs now, or its journal ends mid-turn.
    Open,
}

/// One ask of a session run: what it did, cost and ended as.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InsightAsk {
    /// The world's id.
    pub world: String,
    /// The session run.
    pub run: String,
    /// Its place among the run's asks, from 1.
    pub n: u32,
    /// When it began (RFC 3339): its first answer's time less that answer's duration, else the
    /// run's start.
    pub at: String,
    /// The model it rode, as the run sent it.
    pub model: Option<String>,
    /// The session's mode (`light-coding`, …).
    pub mode: Option<String>,
    /// The run's changes to the default units: `+plan`, `-reader`, `~gate` (swapped), `!x`
    /// (quarantined).
    pub units: Vec<String>,
    /// How it ended.
    pub outcome: Outcome,
    /// Why it stopped or failed, in one word: `steps`, `clock`, `context`, `stall`, `budget`,
    /// `interrupted`, `checkpoint`, `cut` or `error`.
    pub cause: Option<String>,
    /// The `Done when` lines it was held to.
    pub listed: u32,
    /// Its numbers.
    #[serde(flatten)]
    pub tally: Tally,
}

/// What one ask did and cost.
#[derive(Debug, Clone, Default, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Tally {
    /// Model turns.
    #[ts(type = "number")]
    pub turns: u64,
    /// Milliseconds in the model.
    #[ts(type = "number")]
    pub model_ms: u64,
    /// Tool calls.
    #[ts(type = "number")]
    pub calls: u64,
    /// Milliseconds in tools.
    #[ts(type = "number")]
    pub tool_ms: u64,
    /// Tool calls that failed.
    #[ts(type = "number")]
    pub failed: u64,
    /// The end gate's verdicts on its answers.
    pub verdicts: u32,
    /// Acts that wrote a file.
    #[ts(type = "number")]
    pub writes: u64,
    /// Lines they added.
    #[ts(type = "number")]
    pub added: u64,
    /// Lines they removed.
    #[ts(type = "number")]
    pub removed: u64,
    /// Whether a test ran after its last change to code; none when it changed none.
    pub tested: Option<bool>,
    /// Reads.
    #[ts(type = "number")]
    pub reads: u64,
    /// Reads of lines already read.
    #[ts(type = "number")]
    pub rereads: u64,
    /// Input tokens, from the usage ledger.
    #[ts(type = "number")]
    pub input: u64,
    /// Output tokens.
    #[ts(type = "number")]
    pub output: u64,
    /// Input tokens read from the cache.
    #[ts(type = "number")]
    pub cache_read: u64,
    /// Dollars: Mikata's price, else the provider's own; none when no call was priced.
    pub usd: Option<f64>,
    /// Calls with no price.
    pub unpriced: u32,
}

/// One tool across the window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InsightTool {
    /// The tool.
    pub name: String,
    /// Its calls.
    #[ts(type = "number")]
    pub calls: u64,
    /// Those that failed.
    #[ts(type = "number")]
    pub failed: u64,
    /// Milliseconds they took.
    #[ts(type = "number")]
    pub ms: u64,
}

/// `GET /api/insights`.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BoardInsights {
    /// The world asked for, or `all`.
    pub world: String,
    /// `7d`, `28d` or `all`.
    pub window: String,
    /// Every ask in the window, oldest first.
    pub asks: Vec<InsightAsk>,
    /// Every tool, most called first.
    pub tools: Vec<InsightTool>,
    /// Runs not read: past the newest [`RUNS`], too large, or past the budget.
    pub unread: u32,
    /// What the numbers could not cover, in words: a ledger that did not read, rows cut.
    pub notes: Vec<String>,
}

/// The words of an ending, as a cause; none for an answer.
fn cause_of(ending: &str) -> Option<&'static str> {
    let e = ending.to_ascii_lowercase();
    if e == "answered" {
        return None;
    }
    Some(if e == "interrupted" {
        "interrupted"
    } else if e.contains("steps") {
        "steps"
    } else if e.contains("clock") || e.contains("deadline") {
        "clock"
    } else if e.contains("context") {
        "context"
    } else if e.contains("stall") || e.contains("loop") {
        "stall"
    } else if e.contains("budget") || e.contains("spent") {
        "budget"
    } else if e.starts_with("checkpoint") {
        "checkpoint"
    } else {
        "error"
    })
}

/// The run's changes to the default units, each unit's last word kept.
fn composition(beats: &[Beat]) -> Vec<String> {
    let mut last: BTreeMap<String, String> = BTreeMap::new();
    for b in beats {
        if let Beat::Units { op, id, .. } = b {
            // The op as the journal writes it: `load`, `unload`, `swap`, `quarantine`, …
            let mark = match serde_json::to_value(op)
                .ok()
                .as_ref()
                .and_then(|v| v.as_str())
            {
                Some("load") => "+",
                Some("unload") => "-",
                Some("swap") => "~",
                Some("quarantine") => "!",
                _ => continue,
            };
            last.insert(id.clone(), format!("{mark}{id}"));
        }
    }
    last.into_values().collect()
}

/// What one ask's beats say: its numbers, its last verdict, its ending, its first answer's start
/// and its `Done when` lines.
struct Segment<'a> {
    tally: Tally,
    verdict: Option<&'a str>,
    ending: Option<&'a str>,
    began: Option<Timestamp>,
    listed: u32,
}

/// Read one ask's beats, its run's usage rows by message id beside them.
fn segment<'a>(seg: &'a [Beat], rows: &HashMap<&str, &UsageRow>) -> Segment<'a> {
    let m = mikata_loop::metrics::metrics(seg);
    let mut s = Segment {
        tally: Tally {
            turns: m.turns,
            model_ms: m.model_ms,
            calls: m.calls,
            tool_ms: m.tool_ms,
            failed: m.tools.values().map(|t| t.errors).sum(),
            writes: m.writes,
            added: m.lines_added,
            removed: m.lines_removed,
            tested: m.tested_after_last_write,
            reads: m.reads,
            rereads: m.rereads,
            ..Tally::default()
        },
        verdict: None,
        ending: None,
        began: None,
        listed: 0,
    };
    for b in seg {
        match b {
            Beat::Ask { text } => {
                s.listed = u32::try_from(mikata_loop::lines::of_ask(text).len()).unwrap_or(0);
            }
            Beat::Model { at, ms, id, .. } => {
                if s.began.is_none()
                    && let Some(at) = at
                {
                    let back = i64::try_from(ms.unwrap_or(0) / 1000).unwrap_or(0);
                    s.began = Some(Timestamp(at.0.saturating_sub(back)));
                }
                if let Some(row) = id.as_deref().and_then(|id| rows.get(id)) {
                    let t = &mut s.tally;
                    t.input += row.usage.input;
                    t.output += row.usage.output;
                    t.cache_read += row.usage.cache_read;
                    match row.usd.or(row.reported_cost) {
                        Some(d) => t.usd = Some(t.usd.unwrap_or(0.0) + d),
                        None => t.unpriced += 1,
                    }
                }
            }
            Beat::Gate { verdict, .. } => {
                s.tally.verdicts += 1;
                s.verdict = Some(verdict.as_str());
            }
            Beat::End { ending } => s.ending = Some(ending.as_str()),
            _ => {}
        }
    }
    s
}

/// How an ask ended and why, from its last verdict and its ending; `last` when it is the run's
/// last ask, which may run still.
fn outcome(
    verdict: Option<&str>,
    ending: Option<&str>,
    last: bool,
) -> (Outcome, Option<&'static str>) {
    match (verdict, ending) {
        (Some("PASS"), _) => (Outcome::Landed, None),
        (Some(_), Some(e)) if cause_of(e).is_none() => (Outcome::Blocked, None),
        (_, Some(e)) => match cause_of(e) {
            None => (Outcome::Answered, None),
            Some("error") => (Outcome::Failed, Some("error")),
            Some(c) => (Outcome::Stopped, Some(c)),
        },
        (_, None) if last => (Outcome::Open, None),
        (_, None) => (Outcome::Stopped, Some("cut")),
    }
}

/// The asks of one run's beats, its usage rows by message id beside it.
fn asks_of(
    world: &str,
    run: &str,
    started: Timestamp,
    beats: &[Beat],
    rows: &HashMap<&str, &UsageRow>,
) -> Vec<InsightAsk> {
    let units = composition(beats);
    let starts: Vec<usize> = beats
        .iter()
        .enumerate()
        .filter_map(|(i, b)| matches!(b, Beat::Ask { .. }).then_some(i))
        .collect();
    let (mut model, mut mode) = (None, None);
    let mut out = Vec::new();
    let mut from = 0;
    for (k, &to_ask) in starts.iter().enumerate() {
        // The model and the mode as they stood when this ask began.
        for b in &beats[from..to_ask] {
            match b {
                Beat::Rides { model: m, .. } => model = Some(m.clone()),
                Beat::Law { mode: Some(m), .. } => mode = Some(m.clone()),
                _ => {}
            }
        }
        let to = starts.get(k + 1).copied().unwrap_or(beats.len());
        let s = segment(&beats[to_ask..to], rows);
        for b in &beats[to_ask..to] {
            if let Beat::Rides { model: m, .. } = b {
                model = Some(m.clone());
            }
        }
        let (outcome, cause) = outcome(s.verdict, s.ending, k + 1 == starts.len());
        let at = s.began.unwrap_or(started);
        out.push(InsightAsk {
            world: world.to_owned(),
            run: run.to_owned(),
            n: u32::try_from(k + 1).unwrap_or(u32::MAX),
            at: rfc3339(
                SystemTime::UNIX_EPOCH + Duration::from_secs(u64::try_from(at.0).unwrap_or(0)),
            ),
            model: model.clone(),
            mode: mode.clone(),
            units: units.clone(),
            outcome,
            cause: cause.map(str::to_owned),
            listed: s.listed,
            tally: s.tally,
        });
        from = to;
    }
    out
}

/// When the session run `id` (`r<ms>-<pid>`) began.
fn started(id: &str) -> Option<Timestamp> {
    let ms: i64 = id.strip_prefix('r')?.split('-').next()?.parse().ok()?;
    Some(Timestamp(ms / 1000))
}

/// A run to read: when it began, its id, its world's id and its journal.
type Found = (Timestamp, String, String, PathBuf);

/// The session runs of `world` that began in the window.
fn found(world: &BoardWorld, path: &Path, since: Option<Timestamp>) -> Vec<Found> {
    let Some(dir) = journals_dir(path) else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    entries
        .take(ENTRIES)
        .filter_map(Result::ok)
        .filter_map(|e| {
            let run = e.file_name().to_str()?.strip_suffix(".jsonl")?.to_owned();
            if !is_session_run(&run) {
                return None;
            }
            let at = started(&run)?;
            if since.is_some_and(|s| at < s) {
                return None;
            }
            Some((at, run, world.id.clone(), dir.join(e.file_name())))
        })
        .collect()
}

/// Every world's usage rows by id, and what did not read.
fn ledgers(
    worlds: &[(BoardWorld, PathBuf)],
    notes: &mut Vec<String>,
) -> HashMap<String, Vec<UsageRow>> {
    let mut out = HashMap::new();
    for (w, path) in worlds {
        match spend::read(path) {
            Ok(l) => {
                if let Some(why) = &l.why {
                    notes.push(format!("{}: {why}", w.name));
                }
                if l.cut {
                    notes.push(format!(
                        "{}: the usage ledger was read from its last {} MiB",
                        w.name,
                        spend::READ_CAP >> 20
                    ));
                }
                out.insert(w.id.clone(), l.rows);
            }
            Err(e) => notes.push(format!("{}: the usage ledger did not read: {e}", w.name)),
        }
    }
    out
}

/// The insights of `worlds` (with their paths) over `window`, up to `now`; `asked` names the
/// world asked for, or `all`.
#[must_use]
pub fn insights(
    asked: &str,
    worlds: &[(BoardWorld, PathBuf)],
    window: Window,
    now: Timestamp,
) -> BoardInsights {
    let since = window.since(now);
    let mut notes = Vec::new();
    let ledgers = ledgers(worlds, &mut notes);
    let mut runs: Vec<Found> = worlds
        .iter()
        .flat_map(|(w, path)| found(w, path, since))
        .collect();
    // The newest runs, read newest first under the budget; shown oldest first.
    runs.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    let mut unread = u32::try_from(runs.len().saturating_sub(RUNS)).unwrap_or(u32::MAX);
    let mut budget = BYTES;
    let mut asks = Vec::new();
    let mut tools: BTreeMap<String, InsightTool> = BTreeMap::new();
    for (at, run, world, file) in runs.into_iter().take(RUNS) {
        let size = std::fs::symlink_metadata(&file).map_or(u64::MAX, |m| m.len());
        let bytes = if size > SUMMARY_BYTES || size > budget {
            None
        } else {
            read_capped(&file, SUMMARY_BYTES).ok()
        };
        let Some(bytes) = bytes else {
            unread += 1;
            continue;
        };
        budget = budget.saturating_sub(bytes.len() as u64);
        let (beats, _) = beats(&bytes);
        let rows: HashMap<&str, &UsageRow> = ledgers
            .get(&world)
            .into_iter()
            .flatten()
            .filter(|r| r.run == run)
            .filter_map(|r| r.message_id.as_deref().map(|id| (id, r)))
            .collect();
        for (name, t) in &mikata_loop::metrics::metrics(&beats).tools {
            let e = tools.entry(name.clone()).or_insert_with(|| InsightTool {
                name: name.clone(),
                calls: 0,
                failed: 0,
                ms: 0,
            });
            e.calls += t.calls;
            e.failed += t.errors;
            e.ms += t.ms;
        }
        asks.extend(asks_of(&world, &run, at, &beats, &rows));
    }
    asks.sort_by(|a, b| {
        a.at.cmp(&b.at)
            .then_with(|| a.run.cmp(&b.run))
            .then(a.n.cmp(&b.n))
    });
    let mut tools: Vec<InsightTool> = tools.into_values().collect();
    tools.sort_by(|a, b| b.calls.cmp(&a.calls).then_with(|| a.name.cmp(&b.name)));
    BoardInsights {
        world: asked.to_owned(),
        window: window.word().to_owned(),
        asks,
        tools,
        unread,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ending_reads_as_its_cause() {
        assert_eq!(cause_of("answered"), None);
        assert_eq!(cause_of("interrupted"), Some("interrupted"));
        assert_eq!(cause_of("checkpoint: steps: 30 taken"), Some("steps"));
        assert_eq!(
            cause_of("checkpoint: clock: the deadline passed between turns"),
            Some("clock")
        );
        assert_eq!(
            cause_of("checkpoint: context: 160000 of 150000 tokens"),
            Some("context")
        );
        assert_eq!(cause_of("checkpoint: stall: loop"), Some("stall"));
        assert_eq!(cause_of("model error: 502"), Some("error"));
    }

    #[test]
    fn a_verdict_and_an_ending_read_as_an_outcome() {
        assert_eq!(
            outcome(Some("PASS"), Some("answered"), true).0,
            Outcome::Landed
        );
        assert_eq!(
            outcome(Some("FAIL"), Some("answered"), true).0,
            Outcome::Blocked
        );
        assert_eq!(outcome(None, Some("answered"), false).0, Outcome::Answered);
        assert_eq!(
            outcome(Some("FAIL"), Some("checkpoint: steps: 30 taken"), true),
            (Outcome::Stopped, Some("steps"))
        );
        assert_eq!(outcome(None, None, true).0, Outcome::Open);
        assert_eq!(outcome(None, None, false), (Outcome::Stopped, Some("cut")));
    }

    #[test]
    fn a_run_id_says_when_it_began() {
        assert_eq!(
            started("r1795796299464-2935461"),
            Some(Timestamp(1_795_796_299))
        );
        assert_eq!(started("nope"), None);
    }
}
