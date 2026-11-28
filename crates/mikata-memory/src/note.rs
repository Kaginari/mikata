//! A trace and the events that make it: a note is never edited, only told about. Its current
//! state is a fold over its events, which never trusts line order — two machines that both
//! appended merge without a conflict.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// What a note says of what it points at.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Polarity {
    /// It holds.
    #[default]
    Support,
    /// It does not hold.
    Counter,
    /// It held, and must be read again: a correction.
    Revise,
}

impl Polarity {
    /// A counter or a revise: the dissent, which recall keeps a seat for.
    #[must_use]
    pub fn dissents(self) -> bool {
        matches!(self, Polarity::Counter | Polarity::Revise)
    }
}

/// How far a note is trusted, or where it went. Ordered by strength, the order only events the
/// time cannot separate fall back on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Said, not proved: personal only.
    #[default]
    Proposed,
    /// Proved by a route: it may be shared.
    Grounded,
    /// Kept, not recalled.
    Dormant,
    /// Gone from recall for good; its events stay.
    Tombstoned,
}

/// Who wrote an event: the harness's own record, the end gate, a route's promotion, the human,
/// or an import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum By {
    /// The harness, recording what a body said.
    Record,
    /// The end gate.
    Gate,
    /// A route promoting a note, with its proof.
    Route,
    /// The human.
    Human,
    /// Brought in from elsewhere.
    Import,
}

/// A note's link to another, with its weight: a revise note's to a note it revises (0.9 towards one
/// the failed act cited, 0.4 towards one merely recalled), a schema note's to a note it distilled
/// or abstracted (1.0).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// The note revised.
    pub note: String,
    /// How strongly, in `[0, 1]`.
    pub weight: f64,
}

/// What a claim rests on: when it moves, the claim is marked for review.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "on", rename_all = "lowercase")]
pub enum Dependency {
    /// Another claim, by its fixed id: superseded by a new version, struck by an attributable
    /// failure or retired, it moves.
    Claim {
        /// The claim's id.
        id: String,
    },
    /// A fact of the ontology, by its id: contested, it moves.
    Fact {
        /// The fact's id.
        id: String,
    },
    /// A creature, by name — the one whose word or doc the claim rests on: rebirthed, rewritten or
    /// retired, it moves.
    Creature {
        /// The creature's name.
        name: String,
    },
    /// A file of the world, with the SHA-256 of its bytes when the claim was proposed or grounded:
    /// a landing that changes them moves it.
    File {
        /// The path, relative to the world.
        path: String,
        /// The content hash it had.
        hash: String,
    },
}

/// One version of a claim: its text as it was said, never rewritten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    /// Its number, from 1.
    pub n: u32,
    /// What it said.
    pub text: String,
    /// When, in milliseconds since the epoch.
    pub time: u64,
    /// The event that said it.
    pub event: String,
}

/// What one event does.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Change {
    /// A note is born.
    Create {
        /// What it says.
        text: String,
        /// Its tags, for recall.
        #[serde(default)]
        tags: Vec<String>,
        /// The creature whose note it is; `None` for the world's.
        #[serde(default)]
        owner: Option<String>,
        /// What it says of what it points at.
        polarity: Polarity,
        /// How much it matters, in `[0, 1]`.
        salience: f64,
        /// How sure, in `[0, 1]`.
        confidence: f64,
        /// The evidence it points at: a journal entry, a verdict, a commit, a `file:line`.
        #[serde(default)]
        source: Option<String>,
        /// The notes a revise note revises, each with its weight.
        #[serde(default)]
        links: Vec<Link>,
        /// What a claim rests on: the files its run read or changed — the anchors a second voice
        /// must not share to corroborate it.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence: Vec<String>,
        /// What it rests on: claims, facts, files with their hashes.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        depends_on: Vec<Dependency>,
        /// The first line of the ask it was learned under, when a session learned it answering
        /// one: recall matches the note with these words beside its own.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        asked: Option<String>,
    },
    /// A correction of its text: the next version, under the same id. The earlier versions stay;
    /// the new one is proposed until a route grounds it.
    Version {
        /// What it says now.
        text: String,
        /// What this version rests on: it replaces the earlier version's.
        #[serde(default)]
        depends_on: Vec<Dependency>,
    },
    /// More it rests on, recorded when it is grounded: a file named again takes its new hash.
    Depends {
        /// The dependencies.
        on: Vec<Dependency>,
    },
    /// Something it rests on moved: it needs review until a route grounds it again. Never a
    /// deletion, never a retirement.
    Review {
        /// What moved.
        cause: String,
    },
    /// Its standing changes: a promotion, a demotion, a tombstone.
    Status {
        /// The new standing.
        status: Status,
    },
    /// Its confidence moves: a pass gained, a failure that reconsolidates it.
    Confidence {
        /// The new confidence, in `[0, 1]`.
        confidence: f64,
    },
    /// Held for a later check: the claim cites a test written in the session that made it, and a
    /// test its author just wrote proves little. A later gate that passes its verify line while
    /// that file stands as it was here grounds it: by then the test stood before the work.
    Pending {
        /// The test file it cites, relative to the world.
        test: String,
        /// The SHA-256 of that file's bytes when the claim was held.
        hash: String,
        /// The check the claim named, as it named it.
        check: String,
    },
}

/// One event: one full line of a note file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Unique among events: with `time`, the fold's order.
    pub id: String,
    /// When, in milliseconds since the epoch.
    pub time: u64,
    /// The note it is about: its fixed id, whatever its text becomes. A line written without one
    /// — an import, a hand-written note — is given one on reading ([`crate::store::read`]).
    #[serde(default)]
    pub note: String,
    /// Who wrote it.
    pub by: By,
    /// The proof an event that claims a standing cites: a `log.md` entry, a commit, a verdict.
    #[serde(default)]
    pub proof: Option<String>,
    /// The unit that wrote it, as `unit@version` ([`UnitStamp`]), when one did: so a bad unit's
    /// notes can be found and withdrawn. Rows written before units have none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// What it does.
    #[serde(flatten)]
    pub change: Change,
}

/// Which unit wrote an event, and which version of it: `memory@1.2.0`, `plugin:cognee@0.1.0`.
/// The unit's id and its version are each printable ASCII without spaces or `@`, at most 64
/// characters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitStamp(String);

impl UnitStamp {
    /// The stamp of version `version` of unit `id`.
    ///
    /// # Errors
    ///
    /// Either part is empty, longer than 64 characters, or holds a space, a control character, a
    /// character outside ASCII or `@`.
    pub fn new(id: &str, version: &str) -> Result<Self, String> {
        let lawful = |part: &str| {
            !part.is_empty()
                && part.len() <= 64
                && part.chars().all(|c| c.is_ascii_graphic() && c != '@')
        };
        if lawful(id) && lawful(version) {
            Ok(Self(format!("{id}@{version}")))
        } else {
            Err(format!(
                "`{id}@{version}` is no unit's stamp: unit@version, each part printable ASCII \
                 without spaces or `@`, at most 64 characters"
            ))
        }
    }

    /// The stamp as it is written: `unit@version`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::str::FromStr for UnitStamp {
    type Err = String;

    /// Read `unit@version`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('@') {
            Some((id, version)) => Self::new(id, version),
            None => Err(format!(
                "`{s}` is no unit's stamp: unit@version, as `memory@1.2.0`"
            )),
        }
    }
}

impl std::fmt::Display for UnitStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A note as its events leave it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Note {
    /// Its id.
    pub id: String,
    /// What it says.
    pub text: String,
    /// Its tags.
    pub tags: Vec<String>,
    /// Whose it is; `None` for the world's.
    pub owner: Option<String>,
    /// What it says of what it points at.
    pub polarity: Polarity,
    /// How much it matters.
    pub salience: f64,
    /// How sure.
    pub confidence: f64,
    /// Its standing now.
    pub status: Status,
    /// What it points at.
    pub source: Option<String>,
    /// The proof its standing rests on, when one was cited.
    pub proof: Option<String>,
    /// When it was born, in milliseconds since the epoch: its creation's time.
    pub born: u64,
    /// The notes it revises, each with its weight: a revise note's.
    pub links: Vec<Link>,
    /// Words recall borrows for matching: a correction's from the notes it cites, to match where
    /// they match, and any note's from the ask it was learned under ([`Note::asked`]). Never
    /// counted in how common a word is.
    pub lent: Vec<String>,
    /// The first line of the ask it was learned under, when a session learned it answering one.
    pub asked: Option<String>,
    /// What the claim rests on: the files its run read or changed.
    pub evidence: Vec<String>,
    /// Its version now, from 1: its id stays, each correction a new version.
    pub version: u32,
    /// Every version, oldest first: the chain, never rewritten.
    pub versions: Vec<Version>,
    /// What its current version rests on.
    pub depends_on: Vec<Dependency>,
    /// Why it needs review, while it does: something it rests on moved, and no route has grounded
    /// it since.
    pub review: Option<String>,
    /// The distinct proofs that grounded its current version, oldest first: more than one is a
    /// claim stood behind more than once.
    pub groundings: Vec<String>,
    /// When it was last grounded, in milliseconds since the epoch.
    pub grounded_at: Option<u64>,
    /// The unit that wrote its creation, as `unit@version`, when one did.
    pub unit: Option<String>,
    /// The later check it waits on, while it does ([`Change::Pending`]): a proposed claim whose
    /// test was written in its own session.
    pub pending: Option<Held>,
}

/// A claim's later check: the test file it cites and that file's hash when it was held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    /// The test file, relative to the world.
    pub test: String,
    /// Its SHA-256 when the claim was held.
    pub hash: String,
    /// The check the claim named.
    pub check: String,
    /// When it was held, in milliseconds since the epoch.
    pub since: u64,
}

impl Note {
    /// Add `deps`: a file already named takes its new hash; anything else is added once.
    fn rest_on(&mut self, deps: &[Dependency]) {
        for d in deps {
            if let Dependency::File { path, .. } = d {
                self.depends_on
                    .retain(|o| !matches!(o, Dependency::File { path: p, .. } if p == path));
            }
            if !self.depends_on.contains(d) {
                self.depends_on.push(d.clone());
            }
        }
    }
}

/// Fold events into notes. Events are taken in `(time, id)` order and the latest sets a note's
/// status and confidence, so a demotion lowers what came before it; only events the order cannot
/// separate — the same time and id, a merge's duplicate — take the stronger status. An event about
/// a note never created is set aside, never guessed at; a second creation of one is ignored.
#[must_use]
#[allow(clippy::too_many_lines, reason = "one arm per kind of event")]
pub fn fold(events: &[Event]) -> BTreeMap<String, Note> {
    // A total order, whatever the line order: two machines whose union merges listed the same
    // events differently fold them the same.
    let mut ordered: Vec<(&Event, String)> = events.iter().map(|e| (e, format!("{e:?}"))).collect();
    ordered.sort_by(|(a, x), (b, y)| (a.time, &a.id, x).cmp(&(b.time, &b.id, y)));
    let ordered = ordered.into_iter().map(|(e, _)| e);
    let mut notes: BTreeMap<String, Note> = BTreeMap::new();
    let mut last: BTreeMap<String, (u64, String)> = BTreeMap::new();
    for e in ordered {
        match &e.change {
            Change::Create {
                text,
                tags,
                owner,
                polarity,
                salience,
                confidence,
                source,
                links,
                evidence,
                depends_on,
                asked,
            } => {
                notes.entry(e.note.clone()).or_insert_with(|| Note {
                    id: e.note.clone(),
                    text: text.clone(),
                    tags: tags.clone(),
                    owner: owner.clone(),
                    polarity: *polarity,
                    salience: salience.clamp(0.0, 1.0),
                    confidence: confidence.clamp(0.0, 1.0),
                    status: Status::Proposed,
                    source: source.clone(),
                    proof: None,
                    born: e.time,
                    links: links.clone(),
                    lent: Vec::new(),
                    asked: asked.clone(),
                    evidence: evidence.clone(),
                    version: 1,
                    versions: vec![Version {
                        n: 1,
                        text: text.clone(),
                        time: e.time,
                        event: e.id.clone(),
                    }],
                    depends_on: depends_on.clone(),
                    review: None,
                    groundings: Vec::new(),
                    grounded_at: None,
                    unit: e.unit.clone(),
                    pending: None,
                });
            }
            Change::Version { text, depends_on } => {
                let Some(n) = notes.get_mut(&e.note) else {
                    continue;
                };
                // A new version is a new claim under the old id: what grounded the last one
                // grounded other words, so it waits for a route of its own.
                n.version += 1;
                n.versions.push(Version {
                    n: n.version,
                    text: text.clone(),
                    time: e.time,
                    event: e.id.clone(),
                });
                n.text.clone_from(text);
                n.depends_on.clone_from(depends_on);
                if n.status == Status::Grounded {
                    n.status = Status::Proposed;
                    n.proof = None;
                }
                n.groundings.clear();
                n.grounded_at = None;
                n.review = None;
                // A held check was for other words.
                n.pending = None;
            }
            Change::Depends { on } => {
                if let Some(n) = notes.get_mut(&e.note) {
                    n.rest_on(on);
                }
            }
            Change::Review { cause } => {
                if let Some(n) = notes.get_mut(&e.note)
                    && matches!(n.status, Status::Proposed | Status::Grounded)
                {
                    n.review = Some(cause.clone());
                }
            }
            Change::Status { status } => {
                let Some(n) = notes.get_mut(&e.note) else {
                    continue;
                };
                let key = (e.time, e.id.clone());
                // The same time and id twice is a merge's duplicate: the stronger one stands.
                let tie = last.get(&e.note) == Some(&key);
                if !tie || *status > n.status {
                    n.status = *status;
                    n.proof.clone_from(&e.proof);
                    // Any standing but proposed ends a wait on a later check.
                    if *status != Status::Proposed {
                        n.pending = None;
                    }
                    // A route that grounds it again answers the review: the claim stands anew.
                    if *status == Status::Grounded {
                        n.review = None;
                        n.grounded_at = Some(e.time);
                        if let Some(p) = &e.proof
                            && !n.groundings.contains(p)
                        {
                            n.groundings.push(p.clone());
                        }
                    }
                }
                last.insert(e.note.clone(), key);
            }
            Change::Confidence { confidence } => {
                if let Some(n) = notes.get_mut(&e.note) {
                    n.confidence = confidence.clamp(0.0, 1.0);
                }
            }
            Change::Pending { test, hash, check } => {
                if let Some(n) = notes.get_mut(&e.note)
                    && n.status == Status::Proposed
                {
                    n.pending = Some(Held {
                        test: test.clone(),
                        hash: hash.clone(),
                        check: check.clone(),
                        since: e.time,
                    });
                }
            }
        }
    }
    notes
}
