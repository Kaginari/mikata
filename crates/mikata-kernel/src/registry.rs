//! The registry: which units are loaded, which are active, in what order their hooks run, and
//! how each is taken back. A change is asked for with [`Registry::request`] and applied at the
//! next safe point the units it touches allow ([`Registry::reconcile`]). Every registration a
//! unit makes while it activates pushes an inverse the kernel wrote; a unit leaves by running
//! those inverses newest first, after every unit that depends on it has left.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};

use serde::{Deserialize, Serialize};

use crate::manifest::{Band, Manifest, SafePoint};
use crate::vocabulary::{Event, Grant, NOTE_CAP, Point, Reply};

/// One part of the loop that can be switched on, off or swapped while a session runs. `C` is the
/// context the host lends a unit while it activates and while its hooks run.
pub trait Unit<C>: Send {
    /// What the unit says about itself. Read once, when the unit is loaded.
    fn manifest(&self) -> &Manifest;

    /// The only place a unit registers. Every registration pushes a kernel-written inverse.
    ///
    /// # Errors
    ///
    /// A unit that cannot start says why; what it registered before is taken back and it is not
    /// loaded.
    fn activate(&mut self, act: &mut Activation<'_, C>) -> Result<(), UnitError>;

    /// One hook call, at a point the manifest names.
    fn on(&mut self, cx: &mut C, event: &Event) -> Reply;

    /// Closes open per-ask obligations before an unload (bounded by the caller).
    fn settle(&mut self, _cx: &mut C) {}

    /// Whether the unit has failed past repair, and why: asked after each hook call, and a unit
    /// that says so is quarantined. A plugin's bridge says so after its third strike.
    fn failed(&self) -> Option<String> {
        None
    }
}

/// Why a unit could not start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitError {
    /// Why, in a sentence.
    pub why: String,
}

impl UnitError {
    /// An error that says `why`.
    #[must_use]
    pub fn new(why: impl Into<String>) -> Self {
        UnitError { why: why.into() }
    }
}

impl fmt::Display for UnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.why)
    }
}

impl std::error::Error for UnitError {}

/// What a unit is lent while it activates: the context, the services it needs and the means to
/// register. Each registration pushes an inverse onto the unit's undo stack.
pub struct Activation<'a, C> {
    manifest: &'a Manifest,
    cx: &'a mut C,
    services: &'a mut BTreeMap<String, Service>,
    undo: &'a mut Vec<Inverse<C>>,
}

impl<C> Activation<'_, C> {
    /// The id of the unit activating.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    /// The context, to read. A change to it goes through [`Activation::register`].
    #[must_use]
    pub fn cx(&self) -> &C {
        self.cx
    }

    /// Applies `apply` to the context now and keeps `undo` to take it back when the unit leaves.
    pub fn register<R>(
        &mut self,
        apply: impl FnOnce(&mut C) -> R,
        undo: impl FnOnce(&mut C) + Send + 'static,
    ) -> R {
        let out = apply(self.cx);
        self.undo.push(Inverse::Cx(Box::new(undo)));
        out
    }

    /// Provides the service `key`, which the manifest must name in `provides`.
    ///
    /// # Errors
    ///
    /// A key the manifest does not name, or one already provided.
    pub fn provide<T: Any + Send>(&mut self, key: &str, value: T) -> Result<(), UnitError> {
        if !self.manifest.provides.iter().any(|k| k == key) {
            return Err(UnitError::new(format!(
                "{} provides `{key}`, which its manifest does not name",
                self.manifest.id
            )));
        }
        if let Some(held) = self.services.get(key) {
            return Err(UnitError::new(format!(
                "`{key}` is already provided by {}",
                held.by
            )));
        }
        self.services.insert(
            key.to_owned(),
            Service {
                by: self.manifest.id.clone(),
                value: Box::new(value),
            },
        );
        self.undo.push(Inverse::Service(key.to_owned()));
        Ok(())
    }

    /// A service the unit can read, when one is provided under `key` as a `T`.
    #[must_use]
    pub fn service<T: 'static>(&self, key: &str) -> Option<&T> {
        self.services.get(key)?.value.downcast_ref()
    }
}

/// A change asked of the registry, applied at the next safe point it allows.
pub enum Change<C> {
    /// Loads a unit with what the human grants it.
    Load(Box<dyn Unit<C>>, Vec<Grant>),
    /// Unloads the unit with this id, and every unit that depends on it first.
    Unload(String),
    /// Puts a unit in the place of the unit with this id, keeping its dependents.
    Swap(String, Box<dyn Unit<C>>, Vec<Grant>),
}

/// What a journal row says happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    /// A unit was loaded.
    Load,
    /// A unit was unloaded.
    Unload,
    /// A unit took another's place.
    Swap,
    /// A unit failed and was taken out.
    Quarantine,
    /// A change was refused and left nothing behind.
    Refused,
    /// A reply above the unit's grant, or at a point where it does not count, was read as a pass.
    Dropped,
}

/// One row for the journal: what changed in the composition, and the composition after it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitsRow {
    /// What happened.
    pub op: Op,
    /// The unit it happened to.
    pub id: String,
    /// That unit's version; empty when the change named an id that is not loaded.
    pub version: String,
    /// Why, when there is more to say than the op.
    pub why: Option<String>,
    /// [`Registry::digest`] after the row.
    pub digest: String,
}

/// Where a loaded unit stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fiber {
    /// Loaded, waiting for a service it needs.
    Inactive,
    /// Loaded and hooked.
    Active,
    /// Quarantined: its registrations were taken back and its code dropped.
    Failed,
}

struct Service {
    by: String,
    value: Box<dyn Any + Send>,
}

enum Inverse<C> {
    Service(String),
    Cx(Box<dyn FnOnce(&mut C) + Send>),
}

struct Entry<C> {
    manifest: Manifest,
    /// `None` once the unit is quarantined.
    unit: Option<Box<dyn Unit<C>>>,
    fiber: Fiber,
    /// What the manifest asks for, cut to what the human granted.
    grants: Vec<Grant>,
    undo: Vec<Inverse<C>>,
}

impl<C> Entry<C> {
    fn live(&self) -> bool {
        self.fiber != Fiber::Failed
    }
}

/// The units of one session: loaded, ordered, hooked, and taken back on request.
pub struct Registry<C> {
    entries: BTreeMap<String, Entry<C>>,
    services: BTreeMap<String, Service>,
    pending: Vec<Change<C>>,
    /// Every live unit (inactive or active), in resolved order.
    order: Vec<String>,
    /// The active units hooked at each point, in the order their hooks run.
    hooks: BTreeMap<Point, Vec<String>>,
    /// Rows written during a dispatch, for [`Registry::take_rows`].
    held: Vec<UnitsRow>,
}

impl<C> Default for Registry<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> Registry<C> {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Registry {
            entries: BTreeMap::new(),
            services: BTreeMap::new(),
            pending: Vec::new(),
            order: Vec::new(),
            hooks: BTreeMap::new(),
            held: Vec::new(),
        }
    }

    /// Asks for a change; applied at the next safe point the unit allows.
    pub fn request(&mut self, change: Change<C>) {
        self.pending.push(change);
    }

    /// How many requested changes wait for a safe point.
    #[must_use]
    pub fn pending(&self) -> usize {
        self.pending.len()
    }

    /// At a safe point of the given kind: applies what is due, in the order it was asked,
    /// dependents first on unload, newest registration first; returns rows for the journal.
    /// A change waits while any unit it would load, unload or rebind needs a coarser safe point.
    pub fn reconcile(&mut self, at: SafePoint, cx: &mut C) -> Vec<UnitsRow> {
        let mut rows = Vec::new();
        for change in std::mem::take(&mut self.pending) {
            if self.due(&change, at) {
                rows.extend(self.apply(change, cx));
            } else {
                self.pending.push(change);
            }
        }
        rows
    }

    /// Runs every active hook at `point`, in resolved order. Replies are cut to the unit's
    /// grant and to the points where they count (a cut reply is read as a pass and journaled),
    /// and notes to [`NOTE_CAP`] characters. The first veto or stop ends the point: the units
    /// after it are not called. A hook that panics, or a unit that says it has failed, is
    /// quarantined and answers [`Reply::Pass`]; the rows wait in [`Registry::take_rows`].
    pub fn dispatch(&mut self, point: Point, cx: &mut C, event: &Event) -> Vec<(String, Reply)> {
        let ids = self.hooks.get(&point).cloned().unwrap_or_default();
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            let Some(entry) = self.entries.get_mut(&id) else {
                continue;
            };
            if entry.fiber != Fiber::Active {
                continue;
            }
            let Some(unit) = entry.unit.as_mut() else {
                continue;
            };
            let called = catch_unwind(AssertUnwindSafe(|| unit.on(cx, event)));
            let failed = match &called {
                Ok(_) => unit.failed(),
                Err(payload) => Some(format!(
                    "its hook panicked at {}: {}",
                    point.name(),
                    panic_text(payload.as_ref())
                )),
            };
            if let Some(why) = failed {
                let rows = self.quarantine(&id, &why, cx);
                self.held.extend(rows);
                out.push((id, Reply::Pass));
                continue;
            }
            let reply = self.cut(&id, point, called.unwrap_or(Reply::Pass));
            let ends = reply.ends();
            out.push((id, reply));
            if ends {
                break;
            }
        }
        out
    }

    /// Takes the unit out at once, whatever safe point it asked for: its dependents are
    /// unloaded first, its registrations are taken back newest first without settling, its code
    /// is dropped, and it stays listed as [`Fiber::Failed`] until it is unloaded or replaced.
    pub fn quarantine(&mut self, id: &str, why: &str, cx: &mut C) -> Vec<UnitsRow> {
        if !self.entries.get(id).is_some_and(Entry::live) {
            return Vec::new();
        }
        let mut rows = self.unload_dependents(id, "quarantined", cx);
        self.deactivate(id, false, cx);
        let Some(entry) = self.entries.get_mut(id) else {
            return rows;
        };
        entry.unit = None;
        entry.fiber = Fiber::Failed;
        entry.undo.clear();
        let version = entry.manifest.version.clone();
        self.resolve();
        rows.push(self.row(Op::Quarantine, id, &version, Some(why.to_owned())));
        rows.extend(self.wake(cx));
        rows
    }

    /// The active units, `(id, version)`, in resolved order.
    #[must_use]
    pub fn active(&self) -> Vec<(&str, &str)> {
        self.order
            .iter()
            .filter_map(|id| self.entries.get(id))
            .filter(|e| e.fiber == Fiber::Active)
            .map(|e| (e.manifest.id.as_str(), e.manifest.version.as_str()))
            .collect()
    }

    /// Every loaded unit, `(id, version, fiber)`: live ones in resolved order, then the
    /// quarantined ones by id.
    #[must_use]
    pub fn fibers(&self) -> Vec<(&str, &str, Fiber)> {
        let live = self.order.iter().filter_map(|id| self.entries.get(id));
        let failed = self.entries.values().filter(|e| !e.live());
        live.chain(failed)
            .map(|e| (e.manifest.id.as_str(), e.manifest.version.as_str(), e.fiber))
            .collect()
    }

    /// The manifest a loaded unit was loaded with.
    #[must_use]
    pub fn manifest(&self, id: &str) -> Option<&Manifest> {
        self.entries.get(id).map(|e| &e.manifest)
    }

    /// What a loaded unit may do: what its manifest asked for, cut to what the human granted.
    #[must_use]
    pub fn grants(&self, id: &str) -> Option<&[Grant]> {
        self.entries.get(id).map(|e| e.grants.as_slice())
    }

    /// The composition's fingerprint: the active units' `id@version`, sorted, joined by commas.
    #[must_use]
    pub fn digest(&self) -> String {
        let mut stamps: Vec<String> = self
            .entries
            .values()
            .filter(|e| e.fiber == Fiber::Active)
            .map(|e| e.manifest.stamp())
            .collect();
        stamps.sort();
        stamps.join(",")
    }

    /// The service provided under `key`, when it is a `T`.
    #[must_use]
    pub fn service<T: 'static>(&self, key: &str) -> Option<&T> {
        self.services.get(key)?.value.downcast_ref()
    }

    /// The rows written since the last call outside [`Registry::reconcile`] and
    /// [`Registry::quarantine`]: quarantines and dropped replies during a dispatch.
    pub fn take_rows(&mut self) -> Vec<UnitsRow> {
        std::mem::take(&mut self.held)
    }

    // ---- applying changes ----

    fn due(&self, change: &Change<C>, at: SafePoint) -> bool {
        match change {
            Change::Load(unit, _) => unit.manifest().safe_point.allows(at),
            Change::Unload(id) => self.cascade_safe_point(id).allows(at),
            Change::Swap(id, unit, _) => self
                .cascade_safe_point(id)
                .max(unit.manifest().safe_point)
                .allows(at),
        }
    }

    /// The coarsest safe point among a unit and every unit that depends on it.
    fn cascade_safe_point(&self, id: &str) -> SafePoint {
        let Some(entry) = self.entries.get(id).filter(|e| e.live()) else {
            return SafePoint::Batch;
        };
        self.dependents(id)
            .iter()
            .filter_map(|d| self.entries.get(d))
            .map(|e| e.manifest.safe_point)
            .fold(entry.manifest.safe_point, SafePoint::max)
    }

    fn apply(&mut self, change: Change<C>, cx: &mut C) -> Vec<UnitsRow> {
        match change {
            Change::Load(unit, granted) => self.load(unit, &granted, cx),
            Change::Unload(id) => self.unload(&id, cx),
            Change::Swap(id, unit, granted) => self.swap(&id, unit, &granted, cx),
        }
    }

    fn load(&mut self, unit: Box<dyn Unit<C>>, granted: &[Grant], cx: &mut C) -> Vec<UnitsRow> {
        let manifest = unit.manifest().clone();
        if self.entries.get(&manifest.id).is_some_and(Entry::live) {
            return vec![self.refused(&manifest, "it is already loaded")];
        }
        if let Err(why) = self.check(&manifest, &[]) {
            return vec![self.refused(&manifest, &why)];
        }
        let tombstone = self.entries.remove(&manifest.id);
        let id = manifest.id.clone();
        self.insert(unit, manifest, granted);
        if self.ready(&id)
            && let Err(why) = self.activate(&id, cx)
        {
            let entry = self.entries.remove(&id).expect("just inserted");
            if let Some(tombstone) = tombstone {
                self.entries.insert(id, tombstone);
            }
            self.resolve();
            return vec![self.refused(&entry.manifest, &why)];
        }
        // Units that waited for what it provides start before its row is written, so the row's
        // digest is the composition after the load.
        let woken = self.wake(cx);
        let version = self.entries[&id].manifest.version.clone();
        let mut rows = vec![self.row(Op::Load, &id, &version, None)];
        rows.extend(woken);
        rows
    }

    fn unload(&mut self, id: &str, cx: &mut C) -> Vec<UnitsRow> {
        let Some(entry) = self.entries.get(id) else {
            return vec![self.row(Op::Refused, id, "", Some("it is not loaded".into()))];
        };
        let version = entry.manifest.version.clone();
        let mut rows = if entry.live() {
            self.unload_dependents(id, "unloaded", cx)
        } else {
            Vec::new()
        };
        self.remove(id, cx);
        rows.push(self.row(Op::Unload, id, &version, None));
        rows
    }

    fn swap(
        &mut self,
        old: &str,
        unit: Box<dyn Unit<C>>,
        granted: &[Grant],
        cx: &mut C,
    ) -> Vec<UnitsRow> {
        let manifest = unit.manifest().clone();
        let Some(old_entry) = self.entries.get(old) else {
            return vec![self.refused(&manifest, &format!("{old} is not loaded"))];
        };
        let old_stamp = old_entry.manifest.stamp();
        if !old_entry.live() {
            // A quarantined unit holds nothing: its place is simply taken.
            let tombstone = self.entries.remove(old).expect("checked above");
            let rows = self.load(unit, granted, cx);
            if rows.first().is_some_and(|r| r.op == Op::Load) {
                return swapped(rows, &old_stamp);
            }
            if !self.entries.contains_key(old) {
                self.entries.insert(old.to_owned(), tombstone);
            }
            return rows;
        }
        if manifest.id != old && self.entries.get(&manifest.id).is_some_and(Entry::live) {
            return vec![self.refused(&manifest, "it is already loaded")];
        }
        // A dependent that runs after the old id cannot stay when the newcomer has another id,
        // nor can what depends on it; the newcomer is checked against the units that stay.
        let orphans = if manifest.id == old {
            Vec::new()
        } else {
            self.orphans(old)
        };
        let mut leaving = orphans.clone();
        leaving.push(old.to_owned());
        if let Err(why) = self.check(&manifest, &leaving) {
            return vec![self.refused(&manifest, &why)];
        }

        // Dependents step back first, newest in order first, then the old unit leaves.
        let dependents = self.dependents(old);
        for d in dependents.iter().rev() {
            self.deactivate(d, true, cx);
        }
        self.deactivate(old, true, cx);
        // The orphans leave the graph before the newcomer joins it, so the order never holds
        // the newcomer and a unit it was checked without.
        let held: Vec<(String, Entry<C>)> = orphans
            .iter()
            .filter_map(|d| self.entries.remove(d).map(|e| (d.clone(), e)))
            .collect();
        let old_entry = self.entries.remove(old).expect("checked above");
        let new_id = manifest.id.clone();
        self.insert(unit, manifest, granted);
        let mut rows = Vec::new();
        if self.ready(&new_id)
            && let Err(why) = self.activate(&new_id, cx)
        {
            // The new unit took nothing; the old one comes back as it was.
            let failed = self.entries.remove(&new_id).expect("just inserted");
            self.entries.insert(old.to_owned(), old_entry);
            self.entries.extend(held);
            self.resolve();
            rows.push(self.refused(&failed.manifest, &why));
            if self.ready(old)
                && let Err(why) = self.activate(old, cx)
            {
                rows.extend(self.quarantine(old, &format!("it did not come back: {why}"), cx));
            }
            rows.extend(self.wake(cx));
            return rows;
        }
        drop(old_entry);
        for (d, entry) in held.into_iter().rev() {
            rows.push(self.row(
                Op::Unload,
                &d,
                &entry.manifest.version,
                Some(format!("it depends on {old}, which the swap took out")),
            ));
        }
        let woken = self.wake(cx);
        let version = self.entries[&new_id].manifest.version.clone();
        rows.push(self.row(
            Op::Swap,
            &new_id,
            &version,
            Some(format!("in place of {old_stamp}")),
        ));
        rows.extend(woken);
        rows
    }

    /// Whether `manifest` may join the live units but those `leaving` (the unit it replaces and
    /// what leaves with it): a well-formed manifest, every `after` target loaded, no second
    /// provider of a key, no cycle.
    fn check(&self, manifest: &Manifest, leaving: &[String]) -> Result<(), String> {
        if manifest.id.is_empty() {
            return Err("its id is empty".into());
        }
        let mut seen = BTreeSet::new();
        if let Some((point, _)) = manifest.points.iter().find(|(p, _)| !seen.insert(*p)) {
            return Err(format!("it hooks {} twice", point.name()));
        }
        let mut seen = BTreeSet::new();
        if let Some(key) = manifest.provides.iter().find(|k| !seen.insert(*k)) {
            return Err(format!("it provides `{key}` twice"));
        }
        let others: BTreeMap<&str, &Manifest> = self
            .entries
            .values()
            .filter(|e| e.live() && !leaving.contains(&e.manifest.id))
            .map(|e| (e.manifest.id.as_str(), &e.manifest))
            .collect();
        for target in &manifest.after {
            if target == &manifest.id {
                return Err("it runs after itself".into());
            }
            if !others.contains_key(target.as_str()) {
                return Err(format!("it runs after {target}, which is not loaded"));
            }
        }
        for key in &manifest.provides {
            if let Some(other) = others.values().find(|m| m.provides.contains(key)) {
                return Err(format!("`{key}` is already provided by {}", other.id));
            }
        }
        let mut all = others;
        all.insert(manifest.id.as_str(), manifest);
        resolve_order(&all).map(|_| ())
    }

    // ---- the parts ----

    fn insert(&mut self, unit: Box<dyn Unit<C>>, manifest: Manifest, granted: &[Grant]) {
        let mut grants: Vec<Grant> = manifest
            .grants
            .iter()
            .copied()
            .filter(|g| granted.contains(g))
            .collect();
        grants.sort();
        grants.dedup();
        let id = manifest.id.clone();
        self.entries.insert(
            id,
            Entry {
                manifest,
                unit: Some(unit),
                fiber: Fiber::Inactive,
                grants,
                undo: Vec::new(),
            },
        );
        self.resolve();
    }

    /// Whether an inactive unit has every service it needs.
    fn ready(&self, id: &str) -> bool {
        self.entries.get(id).is_some_and(|e| {
            e.fiber == Fiber::Inactive
                && e.manifest
                    .needs
                    .iter()
                    .all(|k| self.services.contains_key(k))
        })
    }

    /// Activates an inactive unit. On an error or a panic, what it registered is taken back and
    /// it stays inactive.
    fn activate(&mut self, id: &str, cx: &mut C) -> Result<(), String> {
        let entry = self
            .entries
            .get_mut(id)
            .expect("activate names a loaded unit");
        let mut unit = entry.unit.take().expect("a live unit holds its code");
        let mut undo = Vec::new();
        let result = {
            let mut act = Activation {
                manifest: &entry.manifest,
                cx: &mut *cx,
                services: &mut self.services,
                undo: &mut undo,
            };
            match catch_unwind(AssertUnwindSafe(|| unit.activate(&mut act))) {
                Ok(Ok(())) => entry
                    .manifest
                    .provides
                    .iter()
                    .find(|k| !act.services.contains_key(k.as_str()))
                    .map_or(Ok(()), |k| Err(format!("it did not provide `{k}`"))),
                Ok(Err(e)) => Err(e.why),
                Err(payload) => Err(format!("it panicked: {}", panic_text(payload.as_ref()))),
            }
        };
        entry.unit = Some(unit);
        match result {
            Ok(()) => {
                entry.undo = undo;
                entry.fiber = Fiber::Active;
                self.resolve();
                Ok(())
            }
            Err(why) => {
                run_inverses(undo, &mut self.services, cx);
                Err(why)
            }
        }
    }

    /// Takes an active unit's registrations back, newest first, after letting it settle.
    fn deactivate(&mut self, id: &str, settle: bool, cx: &mut C) {
        let Some(entry) = self.entries.get_mut(id) else {
            return;
        };
        if entry.fiber != Fiber::Active {
            return;
        }
        if settle && let Some(unit) = entry.unit.as_mut() {
            // A unit that panics while settling still leaves; its inverses run below.
            let _ = catch_unwind(AssertUnwindSafe(|| unit.settle(cx)));
        }
        let undo = std::mem::take(&mut entry.undo);
        entry.fiber = Fiber::Inactive;
        run_inverses(undo, &mut self.services, cx);
        self.resolve();
    }

    /// Unloads a unit (deactivated first, settled) and forgets it.
    fn remove(&mut self, id: &str, cx: &mut C) {
        self.deactivate(id, true, cx);
        self.entries.remove(id);
        self.resolve();
    }

    /// Unloads every unit that depends on `id`, the furthest dependents first.
    fn unload_dependents(&mut self, id: &str, what: &str, cx: &mut C) -> Vec<UnitsRow> {
        let mut rows = Vec::new();
        for d in self.dependents(id).into_iter().rev() {
            let Some(version) = self.entries.get(&d).map(|e| e.manifest.version.clone()) else {
                continue;
            };
            self.remove(&d, cx);
            rows.push(self.row(
                Op::Unload,
                &d,
                &version,
                Some(format!("it depends on {id}, which was {what}")),
            ));
        }
        rows
    }

    /// Activates every inactive unit whose needs are now met, in resolved order. One that fails
    /// to start is quarantined.
    fn wake(&mut self, cx: &mut C) -> Vec<UnitsRow> {
        let mut rows = Vec::new();
        while let Some(id) = self.order.iter().find(|id| self.ready(id)).cloned() {
            if let Err(why) = self.activate(&id, cx) {
                rows.extend(self.quarantine(&id, &format!("it did not start: {why}"), cx));
            }
        }
        rows
    }

    /// The live units that run after `id`, and every unit that depends on them, in resolved
    /// order: what a swap to another id takes out.
    fn orphans(&self, id: &str) -> Vec<String> {
        let mut found: BTreeSet<String> = BTreeSet::new();
        for e in self.entries.values().filter(|e| e.live()) {
            if e.manifest.after.iter().any(|a| a == id) {
                found.insert(e.manifest.id.clone());
                found.extend(self.dependents(&e.manifest.id));
            }
        }
        self.order
            .iter()
            .filter(|o| found.contains(*o))
            .cloned()
            .collect()
    }

    /// The live units that depend on `id`, directly or not, in resolved order: those that run
    /// after it and those that need a service it provides.
    fn dependents(&self, id: &str) -> Vec<String> {
        let mut found: BTreeSet<&str> = BTreeSet::new();
        let mut frontier = vec![id];
        while let Some(at) = frontier.pop() {
            let Some(provider) = self.entries.get(at) else {
                continue;
            };
            for e in self.entries.values().filter(|e| e.live()) {
                let d = e.manifest.id.as_str();
                let depends = e.manifest.after.iter().any(|a| a == at)
                    || e.manifest
                        .needs
                        .iter()
                        .any(|k| provider.manifest.provides.contains(k));
                if d != id && depends && found.insert(d) {
                    frontier.push(d);
                }
            }
        }
        self.order
            .iter()
            .filter(|o| found.contains(o.as_str()))
            .cloned()
            .collect()
    }

    /// Recomputes the resolved order and each point's hooks from the set of live units.
    fn resolve(&mut self) {
        let live: BTreeMap<&str, &Manifest> = self
            .entries
            .values()
            .filter(|e| e.live())
            .map(|e| (e.manifest.id.as_str(), &e.manifest))
            .collect();
        self.order = resolve_order(&live).expect("a load that makes a cycle is refused");
        self.hooks.clear();
        for point in Point::ALL {
            let mut hooked: Vec<(Band, &String)> = self
                .order
                .iter()
                .filter(|id| self.entries[*id].fiber == Fiber::Active)
                .filter_map(|id| self.entries[id].manifest.band_at(point).map(|b| (b, id)))
                .collect();
            // Stable: within a band the resolved order holds.
            hooked.sort_by_key(|(band, _)| *band);
            if !hooked.is_empty() {
                let ids = hooked.into_iter().map(|(_, id)| id.clone()).collect();
                self.hooks.insert(point, ids);
            }
        }
    }

    /// Cuts a reply to the unit's grant and to the points where it counts; a cut reply is a pass,
    /// journaled. A note longer than [`NOTE_CAP`] characters is cut to it.
    fn cut(&mut self, id: &str, point: Point, reply: Reply) -> Reply {
        let entry = &self.entries[id];
        let why = match reply.needs() {
            Some(_) if !reply.counts_at(point) => Some(format!(
                "a {} does not count at {}",
                reply.kind(),
                point.name()
            )),
            Some(grant) if !entry.grants.contains(&grant) => Some(format!(
                "a {} at {} needs the {} grant",
                reply.kind(),
                point.name(),
                grant.name()
            )),
            None | Some(_) => None,
        };
        if let Some(why) = why {
            let version = entry.manifest.version.clone();
            let row = self.row(Op::Dropped, id, &version, Some(why));
            self.held.push(row);
            return Reply::Pass;
        }
        match reply {
            Reply::Note { text } if text.chars().count() > NOTE_CAP => Reply::Note {
                text: text.chars().take(NOTE_CAP).collect(),
            },
            other => other,
        }
    }

    fn row(&self, op: Op, id: &str, version: &str, why: Option<String>) -> UnitsRow {
        UnitsRow {
            op,
            id: id.to_owned(),
            version: version.to_owned(),
            why,
            digest: self.digest(),
        }
    }

    fn refused(&self, manifest: &Manifest, why: &str) -> UnitsRow {
        self.row(
            Op::Refused,
            &manifest.id,
            &manifest.version,
            Some(why.to_owned()),
        )
    }
}

/// The swap's rows: the load row of the newcomer read as a swap.
fn swapped(mut rows: Vec<UnitsRow>, old_stamp: &str) -> Vec<UnitsRow> {
    if let Some(first) = rows.first_mut() {
        first.op = Op::Swap;
        first.why = Some(format!("in place of {old_stamp}"));
    }
    rows
}

/// Runs a unit's inverses newest first. One that panics is passed over; the rest still run.
fn run_inverses<C>(undo: Vec<Inverse<C>>, services: &mut BTreeMap<String, Service>, cx: &mut C) {
    for inverse in undo.into_iter().rev() {
        match inverse {
            Inverse::Service(key) => {
                services.remove(&key);
            }
            Inverse::Cx(undo) => {
                let _ = catch_unwind(AssertUnwindSafe(|| undo(cx)));
            }
        }
    }
}

/// The order of a set of units: a unit after every unit it names in `after` and after the
/// provider of every service it needs, ties by id. It depends on the set alone.
///
/// # Errors
///
/// A cycle, naming the units caught in it.
fn resolve_order(units: &BTreeMap<&str, &Manifest>) -> Result<Vec<String>, String> {
    let provider: BTreeMap<&str, &str> = units
        .values()
        .flat_map(|m| m.provides.iter().map(|k| (k.as_str(), m.id.as_str())))
        .collect();
    let mut before: BTreeMap<&str, BTreeSet<&str>> =
        units.keys().map(|id| (*id, BTreeSet::new())).collect();
    for m in units.values() {
        let edges = m.after.iter().map(String::as_str).chain(
            m.needs
                .iter()
                .filter_map(|k| provider.get(k.as_str()).copied()),
        );
        for b in edges.filter(|b| units.contains_key(b)) {
            before
                .get_mut(m.id.as_str())
                .expect("every unit has a row")
                .insert(b);
        }
    }
    let mut order = Vec::with_capacity(units.len());
    let mut placed = BTreeSet::new();
    while order.len() < units.len() {
        let next = before
            .iter()
            .find(|(id, needs)| !placed.contains(*id) && needs.iter().all(|b| placed.contains(b)))
            .map(|(id, _)| *id);
        let Some(next) = next else {
            let stuck: Vec<&str> = before
                .keys()
                .copied()
                .filter(|id| !placed.contains(id))
                .collect();
            return Err(format!("it closes a cycle among {}", stuck.join(", ")));
        };
        placed.insert(next);
        order.push(next.to_owned());
    }
    Ok(order)
}

fn panic_text(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic with no message".into())
}

#[cfg(test)]
mod tests;
