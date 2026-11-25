/**
 * The Harness Builder: the units a session in this world would open with, as cards round the
 * core, each switched on or off; a profile to start from; the composition as a global file's
 * profile would hold it; an ask and a button that opens a chat with exactly that harness — and,
 * beside it, that run live: its stages, its timeline, the end gate's word, its spend.
 *
 * Every number on the page is the run's own or the configuration's; nothing is estimated.
 */
import type { BoardProfile, BoardUnit, BoardWorld } from '@mikata/protocol';
import { useEffect, useMemo, useRef, useState } from 'react';
import type { Call, ChatView, Entry } from '../data/chat.ts';
import { summary } from '../data/chat.ts';
import { type Chats, startChat } from '../data/chats.ts';
import { composed, opening, toml, useHarness } from '../data/harness.ts';
import { useLive } from '../data/live.ts';
import { Safe } from '../safe/Safe.tsx';
import { Empty } from '../ui/Empty.tsx';
import './builder.css';

/** What a native unit does, and how the card names it. */
const NATIVE: Record<string, { name: string; glyph: string; about: string }> = {
  memory: { name: 'Memory', glyph: '◫', about: 'Keep lessons, handoffs and corrections.' },
  gate: {
    name: 'End gate',
    glyph: '⛉',
    about: "Run the project's verify command on every change.",
  },
  hold: {
    name: 'Done when',
    glyph: '☑',
    about: 'Turn the ask into a held checklist and enforce it.',
  },
  plan: { name: 'Plan', glyph: '✎', about: 'Write down what done means before the code.' },
  plan_lite: {
    name: 'Plan (lite)',
    glyph: '✎',
    about: 'Copy and check only the outputs the ask states.',
  },
  reader: { name: 'Reader', glyph: '❏', about: 'Write "Done when" lines from a prose ask.' },
  what_changed: {
    name: 'What changed',
    glyph: '⇄',
    about: 'Re-run checks and show where the output differs.',
  },
  challenge: { name: 'Challenge', glyph: '♜', about: 'Add extra verification before a landing.' },
  fold: { name: 'Fold', glyph: '≋', about: 'Fold long context as it grows.' },
  stress: {
    name: 'Stall warnings',
    glyph: '◔',
    about: 'Say so when the body stops making progress.',
  },
  returns: { name: 'Returns', glyph: '↺', about: 'Read the tree back to the body after each act.' },
};

/** The cards' order round the core: half on the left, the rest on the right. */
const ORDER = [
  'memory',
  'hold',
  'plan',
  'plan_lite',
  'returns',
  'gate',
  'reader',
  'what_changed',
  'challenge',
  'fold',
  'stress',
];

function described(u: BoardUnit) {
  const n = NATIVE[u.id];
  if (n) return n;
  return {
    name: u.id.replace(/^plugin:/, ''),
    glyph: '⬡',
    about: "A plugin from the human's global file.",
  };
}

function UnitCard({ u, on, onFlip }: { u: BoardUnit; on: boolean; onFlip: () => void }) {
  const d = described(u);
  return (
    <article className="unit-card" data-on={on}>
      <header className="unit-head">
        <span className="unit-glyph" aria-hidden="true">
          {d.glyph}
        </span>
        <h3 className="unit-name">
          <Safe text={d.name} />
        </h3>
        <button
          type="button"
          role="switch"
          aria-checked={on}
          aria-label={`${d.name}: ${on ? 'on' : 'off'}`}
          className="unit-switch"
          onClick={onFlip}
        >
          <span className="unit-knob" />
        </button>
      </header>
      <p className="unit-about">
        <Safe text={d.about} />
      </p>
      <p className="unit-tag">
        <Safe text={u.kind === 'plugin' ? 'plugin' : `${u.kind} · v${u.version}`} />
        {u.frame && <span className="unit-note"> · set as the session opens</span>}
      </p>
    </article>
  );
}

/** The stages a run goes through, read from its entries. */
type Stage = 'Plan' | 'Work' | 'Verify' | 'Land';
type StageState = 'done' | 'now' | 'next' | 'failed' | 'skipped';

const pathOf = (c: Call): string => {
  const i = c.input as Record<string, unknown> | null;
  return typeof i?.path === 'string' ? i.path : '';
};
const writes = (c: Call) => ['write', 'edit', 'patch'].includes(c.name);

function calls(entries: readonly Entry[]): Call[] {
  return entries.flatMap((e) => (e.kind === 'tool' ? [e.call] : e.kind === 'group' ? e.calls : []));
}

export function stages(view: ChatView | undefined, planOn: boolean): [Stage, StageState][] {
  const entries = view?.entries ?? [];
  const all = calls(entries);
  const done = (c: Call) => c.outcome !== undefined && !c.outcome.isError;
  const planned = all.some((c) => writes(c) && done(c) && pathOf(c).endsWith('plan.md'));
  const worked = all.some((c) => writes(c) && done(c) && !pathOf(c).endsWith('plan.md'));
  const verdict = [...entries].reverse().find((e) => e.kind === 'verdict');
  const verifying = all.some((c) => c.name === 'bash' && c.outcome === undefined && !c.stopped);
  const landed = verdict?.kind === 'verdict' && verdict.verdict === 'PASS';
  const failed = verdict?.kind === 'verdict' && verdict.verdict !== 'PASS';
  const busy = view?.busy ?? false;
  const list: [Stage, boolean][] = [
    ['Plan', planned || worked || verdict !== undefined],
    ['Work', worked || verdict !== undefined],
    ['Verify', verdict !== undefined],
    ['Land', landed],
  ];
  let now = false;
  return list.map(([s, ok]) => {
    if (s === 'Plan' && !planOn) return [s, 'skipped'];
    if (s === 'Land' && failed) return [s, 'failed'];
    if (ok) return [s, 'done'];
    if (!now && (busy || verifying)) {
      now = true;
      return [s, 'now'];
    }
    return [s, 'next'];
  });
}

const clock = (ms: number) => {
  const s = Math.max(0, Math.round(ms / 1000));
  return `${String(Math.floor(s / 60)).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`;
};

/** The timeline's lines: when, a mark, what. */
function timeline(view: ChatView | undefined, from: number): [string, string, string, string][] {
  const out: [string, string, string, string][] = [];
  for (const e of view?.entries ?? []) {
    const cs = e.kind === 'tool' ? [e.call] : e.kind === 'group' ? e.calls : [];
    for (const c of cs) {
      const at = c.began === undefined ? '' : clock(c.began - from);
      const mark = c.outcome === undefined ? '▸' : c.outcome.isError ? '✘' : '✔';
      const tone = c.outcome === undefined ? 'now' : c.outcome.isError ? 'alarm' : 'ok';
      out.push([at, mark, tone, `${c.name}: ${summary(c.name, c.input)}`]);
    }
    if (e.kind === 'verdict') {
      const ok = e.verdict === 'PASS';
      const at = e.at === undefined ? '' : clock(e.at - from);
      out.push([at, ok ? '✔' : '✘', ok ? 'ok' : 'alarm', `end gate: ${e.verdict.toLowerCase()}`]);
    }
    if (e.kind === 'assistant' && e.head) {
      out.push(['', '✦', 'holo', `answer: ${e.text.split('\n')[0] ?? ''}`]);
    }
  }
  return out;
}

function LiveRun({ session, ask, planOn }: { session: string; ask: string; planOn: boolean }) {
  const live = useLive(session);
  const sent = useRef(false);
  const [from] = useState(() => Date.now());
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (sent.current || live.link !== 'open' || !live.replayed) return;
    sent.current = true;
    void live.prompt(ask);
  }, [live, ask]);
  useEffect(() => {
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, []);
  const view = live.view;
  const steps = calls(view?.entries ?? []).length;
  const verdict = [...(view?.entries ?? [])].reverse().find((e) => e.kind === 'verdict');
  const passed = verdict?.kind === 'verdict' && verdict.verdict === 'PASS';
  const lines = timeline(view, from);
  const usd = view?.usage.seen
    ? `$${view.usage.usd.toFixed(3)}${view.usage.unpriced ? '+' : ''}`
    : '—';
  return (
    <>
      <header className="live-head">
        <h2 className="live-title">
          <span aria-hidden="true">▷</span> Live run
        </h2>
        {verdict?.kind === 'verdict' && (
          <span className="live-badge" data-ok={passed}>
            {passed ? '✔ Verified' : `✘ ${verdict.verdict.toLowerCase()}`}
          </span>
        )}
      </header>
      <p className="live-sub mono">
        <Safe text={`${session} · ${live.link}`} />
      </p>
      <ol className="stages" aria-label="Stages">
        {stages(view, planOn).map(([s, st]) => (
          <li key={s} className="stage" data-state={st}>
            <span className="stage-dot" aria-hidden="true">
              {st === 'done' ? '✔' : st === 'failed' ? '✘' : st === 'now' ? '◌' : ''}
            </span>
            <span className="stage-name">
              {s}
              <span className="visually-hidden">: {st}</span>
            </span>
          </li>
        ))}
      </ol>
      <div className="live-log mono" role="log" aria-label="Timeline">
        <p className="live-cmd">
          <span className="live-prompt">$</span> mikata run <Safe text={JSON.stringify(ask)} />
        </p>
        {lines.map(([at, mark, tone, text], i) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: the lines only grow, in order
          <p key={i} className="live-line">
            <span className="live-at">{at ? `[${at}]` : ''}</span>
            <span className="live-mark" data-tone={tone}>
              {mark}
            </span>
            <Safe text={text} />
          </p>
        ))}
        {lines.length === 0 && <p className="live-line live-wait">waiting for the first step…</p>}
      </div>
      {verdict?.kind === 'verdict' && (
        <section className="verified" data-ok={passed} aria-label="The end gate's word">
          <span className="verified-glyph" aria-hidden="true">
            {passed ? '✔' : '✘'}
          </span>
          <div>
            <h3 className="verified-title">{passed ? 'Verified' : 'Not landed'}</h3>
            <p className="verified-text">
              {passed
                ? "The project's verify command passed and no test was deleted or skipped."
                : (verdict.failures[0] ?? verdict.unverified[0] ?? 'the end gate refused it')}
            </p>
          </div>
        </section>
      )}
      <dl className="tiles">
        <div className="tile">
          <dt>Wall time</dt>
          <dd>{clock(now - from)}</dd>
        </div>
        <div className="tile">
          <dt>Steps</dt>
          <dd>{steps}</dd>
        </div>
        <div className="tile">
          <dt>Cost</dt>
          <dd>{usd}</dd>
        </div>
      </dl>
    </>
  );
}

export function BuilderView({ w, chats }: { w: BoardWorld; chats: Chats }) {
  const load = useHarness(w.id);
  const [profile, setProfile] = useState<string>('');
  const [want, setWant] = useState<Map<string, boolean> | undefined>(undefined);
  const [ask, setAsk] = useState('');
  const [run, setRun] = useState<{ session: string; ask: string; planOn: boolean } | undefined>();
  const [starting, setStarting] = useState(false);
  const [why, setWhy] = useState<string | undefined>();
  const [copied, setCopied] = useState(false);

  const data = load.state === 'ready' ? load.data : undefined;
  const chosen: BoardProfile | undefined = data?.profiles.find((p) => p.name === profile);
  const base = useMemo(
    () => (data ? opening(data.units, chosen) : new Map<string, boolean>()),
    [data, chosen],
  );
  // A new profile starts the switches again from it.
  useEffect(() => {
    if (data) setWant(new Map(base));
  }, [data, base]);

  if (load.state === 'loading')
    return (
      <Empty glyph="◈" title="Reading the harness…">
        One moment.
      </Empty>
    );
  if (load.state === 'failed' || !data || !want) {
    return (
      <Empty glyph="✘" title="The harness did not read">
        <Safe text={load.state === 'failed' ? load.why : ''} />
      </Empty>
    );
  }
  // The cards in the mockup's order: what shapes the ask on the left, what checks it on the right.
  const rank = (id: string) => {
    const i = ORDER.indexOf(id);
    return i < 0 ? ORDER.length : i;
  };
  const units = [...data.units].sort((a, b) => rank(a.id) - rank(b.id));
  const half = Math.ceil(units.length / 2);
  const left = units.slice(0, half);
  const right = units.slice(half);
  const flip = (id: string) =>
    setWant((m) => {
      const next = new Map(m);
      next.set(id, !next.get(id));
      return next;
    });
  const list = composed(units, base, want);
  const text = toml(profile || 'mine', units, want);
  const on = units.filter((u) => want.get(u.id)).length;
  const start = async () => {
    if (!ask.trim() || starting) return;
    setStarting(true);
    setWhy(undefined);
    try {
      const started = await startChat(w.id, undefined, {
        ...(profile ? { profile } : {}),
        ...(list ? { units: list } : {}),
      });
      setRun({ session: started.session.id, ask: ask.trim(), planOn: want.get('plan') === true });
      chats.refresh();
    } catch (e) {
      setWhy(e instanceof Error ? e.message : String(e));
    } finally {
      setStarting(false);
    }
  };
  const card = (u: BoardUnit) => (
    <UnitCard key={u.id} u={u} on={want.get(u.id) === true} onFlip={() => flip(u.id)} />
  );
  return (
    <div className="builder-room">
      <div className="builder">
        <section className="builder-main" aria-labelledby="builder-title">
          <header className="builder-head">
            <div>
              <h1 id="builder-title" className="builder-title">
                <span aria-hidden="true">◈</span> Harness Builder
              </h1>
              <p className="builder-sub">
                Compose your coding agent by switching units and plugins. Everything is swappable.
              </p>
            </div>
            <label className="builder-profile">
              <span>Profile</span>
              <select value={profile} onChange={(e) => setProfile(e.target.value)}>
                <option value="">default</option>
                {data.profiles.map((p) => (
                  <option key={p.name} value={p.name}>
                    {p.name}
                  </option>
                ))}
              </select>
            </label>
          </header>
          {data.holes.length > 0 && (
            <p className="hint">
              <Safe text={data.holes.join(' · ')} />
            </p>
          )}
          <div className="harness">
            <div className="harness-side">{left.map(card)}</div>
            <section className="harness-core" aria-label="The core">
              <p className="core-name">
                MIKATA <span lang="ja">味方</span>
              </p>
              <p className="core-what">Rust runtime</p>
              <p className="core-parts">Events · Hooks · Lifecycle</p>
              <p className="core-parts">Sandbox · Journal</p>
              <p className="core-count">
                {on} of {units.length} units on
              </p>
            </section>
            <div className="harness-side harness-right">{right.map(card)}</div>
          </div>
          <div className="builder-foot">
            <section className="config" aria-labelledby="config-title">
              <header className="config-head">
                <h2 id="config-title" className="config-title">
                  <span aria-hidden="true">‹›</span> Configuration (TOML)
                </h2>
                <button
                  type="button"
                  className="button"
                  onClick={() => {
                    void navigator.clipboard?.writeText(text).then(() => setCopied(true));
                  }}
                >
                  {copied ? 'Copied' : 'Copy'}
                </button>
              </header>
              <pre className="config-text mono">
                <Safe text={text} lines />
              </pre>
              <p className="config-note">
                <span className="mono">--units</span>{' '}
                <Safe text={list || '(the profile as it is)'} />
              </p>
            </section>
            <section className="launch" aria-labelledby="launch-title">
              <h2 id="launch-title" className="visually-hidden">
                Start a run
              </h2>
              <label className="launch-ask">
                <span>Ask</span>
                <textarea
                  value={ask}
                  rows={4}
                  placeholder="fix the time parser to support 1h30m"
                  onChange={(e) => setAsk(e.target.value)}
                />
              </label>
              <button
                type="button"
                className="launch-go"
                disabled={!ask.trim() || starting}
                onClick={() => void start()}
              >
                <span aria-hidden="true">▷</span> {starting ? 'Starting…' : 'Start run'}
              </button>
              {why && (
                <p className="launch-why" role="alert">
                  <Safe text={why} />
                </p>
              )}
              <p className="launch-note">
                Opens a chat in <Safe text={w.name} /> with this harness, sandboxed as every chat
                is.
              </p>
            </section>
          </div>
        </section>
        <aside className="live" aria-label="Live run">
          {run ? (
            <LiveRun key={run.session} session={run.session} ask={run.ask} planOn={run.planOn} />
          ) : (
            <>
              <header className="live-head">
                <h2 className="live-title">
                  <span aria-hidden="true">▷</span> Live run
                </h2>
              </header>
              <p className="live-sub">
                Start a run and follow it here: its stages, each step, the end gate.
              </p>
            </>
          )}
        </aside>
      </div>
    </div>
  );
}
