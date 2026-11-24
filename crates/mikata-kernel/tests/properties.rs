//! The registry's rules over random sequences of loads, unloads and swaps: a clean-load oracle,
//! an order that is a function of the set, refusals that leave nothing, dependents out first, and
//! a panicking hook quarantined with a pass.

use std::collections::{BTreeMap, BTreeSet};

use mikata_kernel::{
    Activation, Band, Change, Event, Fiber, Manifest, Op, Point, Registry, Reply, SafePoint, Unit,
    UnitError, UnitsRow,
};
use proptest::prelude::*;

const IDS: usize = 6;
const KEYS: usize = 3;

/// What the probes write into: the registrations alive now, and the inverses as they ran.
#[derive(Default)]
struct Cx {
    live: BTreeSet<String>,
    undone: Vec<String>,
}

/// A unit drawn at random: its edges, its band, whether it fails to start or panics in a hook.
#[derive(Debug, Clone)]
struct Spec {
    id: usize,
    version: u8,
    provides: Option<usize>,
    needs: Option<usize>,
    after: Vec<usize>,
    band: u8,
    fail: bool,
}

impl Spec {
    fn manifest(&self, names: &[String]) -> Manifest {
        let band = [Band::Guard, Band::Native, Band::Outer][usize::from(self.band % 3)];
        let mut m = Manifest::new(names[self.id].clone(), self.version.to_string())
            .safe_point(SafePoint::Batch)
            .at(Point::AskBegin, band);
        if let Some(k) = self.provides {
            m = m.provides(format!("k{k}"));
        }
        if let Some(k) = self.needs {
            m = m.needs(format!("k{k}"));
        }
        for a in &self.after {
            m = m.after(names[*a].clone());
        }
        m
    }
}

struct Probe {
    manifest: Manifest,
    fail: bool,
    panics: bool,
}

impl Unit<Cx> for Probe {
    fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    fn activate(&mut self, act: &mut Activation<'_, Cx>) -> Result<(), UnitError> {
        let id = act.id().to_owned();
        for key in self.manifest.provides.clone() {
            act.provide(&key, id.clone())?;
        }
        let back = id.clone();
        act.register(
            |cx| assert!(cx.live.insert(id), "registered twice"),
            move |cx| {
                cx.live.remove(&back);
                cx.undone.push(back);
            },
        );
        if self.fail {
            return Err(UnitError::new("it would not start"));
        }
        Ok(())
    }

    fn on(&mut self, _cx: &mut Cx, _event: &Event) -> Reply {
        assert!(!self.panics, "a probe that panics");
        Reply::Pass
    }
}

fn boxed(spec: &Spec, names: &[String], panics: bool) -> Box<dyn Unit<Cx>> {
    Box::new(Probe {
        manifest: spec.manifest(names),
        fail: spec.fail,
        panics,
    })
}

#[derive(Debug, Clone)]
enum Step {
    Load(Spec),
    Unload(usize),
    Swap(usize, Spec),
}

fn spec() -> impl Strategy<Value = Spec> {
    (
        0..IDS,
        0..3_u8,
        proptest::option::of(0..KEYS),
        proptest::option::of(0..KEYS),
        proptest::collection::vec(0..IDS, 0..2),
        0..3_u8,
        proptest::bool::weighted(0.1),
    )
        .prop_map(|(id, version, provides, needs, after, band, fail)| Spec {
            id,
            version,
            provides,
            needs,
            after,
            band,
            fail,
        })
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        3 => spec().prop_map(Step::Load),
        2 => (0..IDS).prop_map(Step::Unload),
        1 => (0..IDS, spec()).prop_map(|(at, s)| Step::Swap(at, s)),
    ]
}

fn names() -> Vec<String> {
    (0..IDS).map(|i| format!("u{i}")).collect()
}

fn apply(reg: &mut Registry<Cx>, cx: &mut Cx, step: &Step, names: &[String]) -> Vec<UnitsRow> {
    let change = match step {
        Step::Load(s) => Change::Load(boxed(s, names, false), vec![]),
        Step::Unload(i) => Change::Unload(names[*i].clone()),
        Step::Swap(i, s) => Change::Swap(names[*i].clone(), boxed(s, names, false), vec![]),
    };
    reg.request(change);
    reg.reconcile(SafePoint::Session, cx)
}

/// Everything about a registry a caller can see, and the live registrations in the context.
#[derive(Debug, PartialEq)]
struct Shape {
    fibers: Vec<(String, String, Fiber)>,
    digest: String,
    services: Vec<Option<String>>,
    hooks: Vec<String>,
    live: BTreeSet<String>,
}

fn shape(reg: &mut Registry<Cx>, cx: &mut Cx) -> Shape {
    let hooks = reg
        .dispatch(Point::AskBegin, cx, &Event::AskBegin { ask: String::new() })
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    Shape {
        fibers: reg
            .fibers()
            .into_iter()
            .map(|(i, v, f)| (i.to_owned(), v.to_owned(), f))
            .collect(),
        digest: reg.digest(),
        services: (0..KEYS)
            .map(|k| reg.service::<String>(&format!("k{k}")).cloned())
            .collect(),
        hooks,
        live: cx.live.clone(),
    }
}

/// Whether the unit `a` depends directly on the unit `b`: it runs after it or needs what it
/// provides.
fn depends(a: &Manifest, b: &Manifest) -> bool {
    a.after.contains(&b.id) || a.needs.iter().any(|k| b.provides.contains(k))
}

/// The rules every reachable state holds.
fn holds(reg: &mut Registry<Cx>, cx: &mut Cx) -> Result<(), TestCaseError> {
    let fibers: Vec<(String, Fiber)> = reg
        .fibers()
        .into_iter()
        .map(|(i, _, f)| (i.to_owned(), f))
        .collect();
    let manifest = |id: &str| reg.manifest(id).cloned().expect("a listed unit");
    let live: BTreeMap<String, Manifest> = fibers
        .iter()
        .filter(|(_, f)| *f != Fiber::Failed)
        .map(|(i, _)| (i.clone(), manifest(i)))
        .collect();
    let active: BTreeSet<String> = fibers
        .iter()
        .filter(|(_, f)| *f == Fiber::Active)
        .map(|(i, _)| i.clone())
        .collect();
    // Each active unit registered once; nothing else did.
    prop_assert_eq!(&cx.live, &active);
    // Every `after` target is live, and each key has at most one live provider.
    let mut providers: BTreeMap<&String, &String> = BTreeMap::new();
    for (id, m) in &live {
        for a in &m.after {
            prop_assert!(live.contains_key(a), "{id} runs after {a}, which is gone");
        }
        for k in &m.provides {
            prop_assert!(providers.insert(k, id).is_none(), "two providers of {k}");
        }
    }
    // A service is held by an active provider, and an active unit has what it needs.
    for k in 0..KEYS {
        let key = format!("k{k}");
        if let Some(by) = reg.service::<String>(&key) {
            prop_assert!(active.contains(by) && providers.get(&key) == Some(&by));
        }
    }
    for id in &active {
        for k in &live[id].needs {
            prop_assert!(
                reg.service::<String>(k).is_some(),
                "{id} is active without {k}"
            );
        }
    }
    // Hooks: only active units, by band, and within a band after what they depend on.
    let hooks = shape(reg, cx).hooks;
    let band = |id: &String| live[id].band_at(Point::AskBegin).expect("hooked");
    prop_assert!(hooks.iter().all(|h| active.contains(h)));
    prop_assert_eq!(hooks.len(), active.len());
    for (i, a) in hooks.iter().enumerate() {
        for b in &hooks[i + 1..] {
            prop_assert!(band(a) <= band(b), "{a} runs before {b} from a later band");
            if band(a) == band(b) {
                prop_assert!(
                    !depends(&live[a], &live[b]),
                    "{a} runs before {b}, which it follows"
                );
            }
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Every state reached holds the rules; a refused change leaves the registry and the context
    /// as they were; an unload takes every dependent out before the unit, in the rows and in the
    /// inverses run.
    #[test]
    fn random_sequences_hold_the_rules(steps in proptest::collection::vec(step(), 1..30)) {
        let names = names();
        let mut reg = Registry::new();
        let mut cx = Cx::default();
        for step in &steps {
            let before = shape(&mut reg, &mut cx);
            let manifests: BTreeMap<String, Manifest> = reg
                .fibers()
                .into_iter()
                .map(|(i, _, _)| (i.to_owned(), reg.manifest(i).cloned().unwrap()))
                .collect();
            // A quarantined unit holds nothing: no unit depends on it and it takes none out.
            let live: BTreeSet<String> = reg
                .fibers()
                .into_iter()
                .filter(|(_, _, f)| *f != Fiber::Failed)
                .map(|(i, _, _)| i.to_owned())
                .collect();
            cx.undone.clear();
            let rows = apply(&mut reg, &mut cx, step, &names);
            prop_assert!(!rows.is_empty());
            if rows.len() == 1 && rows[0].op == Op::Refused {
                prop_assert_eq!(shape(&mut reg, &mut cx), before);
            }
            if let Step::Unload(i) = step {
                let unloads: Vec<&str> = rows
                    .iter()
                    .filter(|r| r.op == Op::Unload)
                    .map(|r| r.id.as_str())
                    .collect();
                for (k, a) in unloads.iter().enumerate() {
                    for b in &unloads[k + 1..] {
                        prop_assert!(
                            !depends(&manifests[*b], &manifests[*a]),
                            "{} left before {}, which depends on it", a, b
                        );
                    }
                }
                let undone: Vec<&str> = cx.undone.iter().map(String::as_str).collect();
                let was_active: Vec<&str> = unloads
                    .iter()
                    .copied()
                    .filter(|u| undone.contains(u))
                    .collect();
                prop_assert_eq!(undone, was_active);
                prop_assert!(reg.manifest(&names[*i]).is_none());
                if let Some(gone) = manifests.get(&names[*i]).filter(|_| live.contains(&names[*i])) {
                    for (id, m) in manifests.iter().filter(|(id, _)| live.contains(*id)) {
                        if depends(m, gone) {
                            prop_assert!(reg.manifest(id).is_none(), "{} outlived {}", id, gone.id);
                        }
                    }
                }
            }
            holds(&mut reg, &mut cx)?;
        }
    }

    /// Loading a unit and unloading it again leaves the registry as it was, whenever nothing
    /// loaded is waiting for what it provides.
    #[test]
    fn a_load_then_an_unload_leaves_the_registry_as_it_was(
        steps in proptest::collection::vec(step(), 0..20),
        extra in spec(),
    ) {
        let names = names();
        let mut reg = Registry::new();
        let mut cx = Cx::default();
        for step in &steps {
            apply(&mut reg, &mut cx, step, &names);
        }
        let m = extra.manifest(&names);
        let taken = reg.manifest(&m.id).is_some();
        let awaited = reg.fibers().iter().any(|(i, _, _)| {
            reg.manifest(i).unwrap().needs.iter().any(|k| m.provides.contains(k))
        });
        if !taken && !awaited {
            let before = shape(&mut reg, &mut cx);
            let rows = apply(&mut reg, &mut cx, &Step::Load(extra.clone()), &names);
            if rows[0].op == Op::Load {
                let rows = apply(&mut reg, &mut cx, &Step::Unload(extra.id), &names);
                prop_assert_eq!(rows.len(), 1);
            }
            prop_assert_eq!(shape(&mut reg, &mut cx), before);
        }
    }

    /// The same set of units, loaded in two orders, runs its hooks in one order.
    #[test]
    fn the_order_is_a_function_of_the_set(
        specs in proptest::collection::vec(spec(), 1..IDS),
        names in Just((0..IDS).map(|i| format!("n{i}")).collect::<Vec<_>>()).prop_shuffle(),
        seed in any::<u64>(),
    ) {
        // A set with no conflicts: distinct ids, one provider per key, edges only towards a
        // lower index, so it is acyclic whatever the names are.
        let mut set: BTreeMap<usize, Spec> = BTreeMap::new();
        let mut provided: BTreeMap<usize, usize> = BTreeMap::new();
        for mut s in specs {
            if set.contains_key(&s.id) {
                continue;
            }
            s.fail = false;
            if let Some(k) = s.provides {
                if let std::collections::btree_map::Entry::Vacant(slot) = provided.entry(k) {
                    slot.insert(s.id);
                } else {
                    s.provides = None;
                }
            }
            set.insert(s.id, s);
        }
        for s in set.values_mut() {
            let id = s.id;
            s.after.retain(|a| *a < id);
            if s.needs.is_some_and(|k| provided.get(&k).is_some_and(|p| *p >= id)) {
                s.needs = None;
            }
        }
        let after_targets: Vec<usize> = set.values().flat_map(|s| s.after.clone()).collect();
        for t in after_targets {
            if !set.contains_key(&t) {
                for s in set.values_mut() { s.after.retain(|a| *a != t); }
            }
        }
        let forward: Vec<Spec> = set.values().cloned().collect();
        let mut shuffled = forward.clone();
        // A Fisher-Yates shuffle from the drawn seed (a xorshift generator).
        let mut x = seed | 1;
        for i in (1..shuffled.len()).rev() {
            x ^= x << 13; x ^= x >> 7; x ^= x << 17;
            let j = usize::try_from(x % (u64::try_from(i).unwrap() + 1)).unwrap();
            shuffled.swap(i, j);
        }
        let mut shapes = Vec::new();
        for sequence in [forward, shuffled] {
            let mut reg = Registry::new();
            let mut cx = Cx::default();
            // A unit whose `after` target is not loaded yet is refused: try again until every
            // unit is in.
            let mut left = sequence;
            while !left.is_empty() {
                let tried = left.len();
                let mut next = Vec::new();
                for s in left.drain(..) {
                    let rows = apply(&mut reg, &mut cx, &Step::Load(s.clone()), &names);
                    if rows[0].op == Op::Refused { next.push(s); }
                }
                prop_assert!(next.len() < tried, "no progress");
                left = next;
            }
            holds(&mut reg, &mut cx)?;
            shapes.push(shape(&mut reg, &mut cx));
        }
        prop_assert_eq!(&shapes[0], &shapes[1]);
    }

    /// A hook that panics is quarantined: it answers a pass, its registrations and its
    /// dependents are gone, and the rest still hold the rules.
    #[test]
    fn a_panicking_hook_is_quarantined_and_answers_pass(
        steps in proptest::collection::vec(step(), 0..20),
        bad in spec(),
    ) {
        let names = names();
        let mut reg = Registry::new();
        let mut cx = Cx::default();
        for step in &steps {
            apply(&mut reg, &mut cx, step, &names);
        }
        let mut bad = bad;
        bad.fail = false;
        let id = names[bad.id].clone();
        reg.request(Change::Load(boxed(&bad, &names, true), vec![]));
        let rows = reg.reconcile(SafePoint::Batch, &mut cx);
        let active = reg.fibers().iter().any(|(i, _, f)| *i == id && *f == Fiber::Active);
        if rows[0].op == Op::Load && active {
            let dependents: Vec<String> = reg
                .fibers()
                .iter()
                .filter(|(i, _, f)| *f != Fiber::Failed && depends(reg.manifest(i).unwrap(), &bad.manifest(&names)))
                .map(|(i, _, _)| (*i).to_owned())
                .collect();
            let replies = reg.dispatch(
                Point::AskBegin,
                &mut cx,
                &Event::AskBegin { ask: String::new() },
            );
            prop_assert!(replies.contains(&(id.clone(), Reply::Pass)));
            prop_assert!(replies.iter().all(|(_, r)| *r == Reply::Pass));
            let rows = reg.take_rows();
            prop_assert_eq!(rows.last().map(|r| (r.op, r.id.as_str())), Some((Op::Quarantine, id.as_str())));
            prop_assert!(!cx.live.contains(&id));
            for d in &dependents {
                prop_assert!(reg.manifest(d).is_none(), "{} outlived its quarantined provider", d);
            }
            let failed: Vec<(&str, Fiber)> = reg.fibers().into_iter().map(|(i, _, f)| (i, f)).collect();
            prop_assert!(failed.contains(&(id.as_str(), Fiber::Failed)));
            holds(&mut reg, &mut cx)?;
        }
    }
}
