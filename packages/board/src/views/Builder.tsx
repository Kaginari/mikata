/**
 * The Harness Builder, a page of its own: a header (the mark, the builder's views, Run agent);
 * the workspace, its status and the run's usage on the left; the units a session in this world
 * would open with as cards round the core in the middle, with the composition as a global file's
 * profile; and on the right the run itself — the ask, Start and Stop, its stages, its log, the end
 * gate's word and its numbers. An approval the run asks for is answered here, as in a chat.
 *
 * Every number on the page is the run's own or the configuration's; nothing is simulated.
 */

import type { BoardProfile, BoardUnit, BoardWorld } from '@mikata/protocol';
import {
  BookOpenText,
  Box,
  Brain,
  Check,
  CircleCheck,
  CircleX,
  Clock,
  Code,
  Coins,
  Copy,
  Download,
  Gauge,
  GitCompare,
  History,
  KeyRound,
  Layers,
  ListChecks,
  type LucideIcon,
  PencilRuler,
  Play,
  Puzzle,
  RotateCcw,
  Send,
  ShieldCheck,
  Shrink,
  SlidersHorizontal,
  Square,
  Swords,
  UserCheck,
} from 'lucide-react';
import { type ReactNode, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { type Chats, startChat } from '../data/chats.ts';
import { composed, opening, toml, useHarness } from '../data/harness.ts';
import { type Live, useLive } from '../data/live.ts';
import { format } from '../router.ts';
import { Safe } from '../safe/Safe.tsx';
import { Approval } from '../ui/Approval.tsx';
import { Icon } from '../ui/Icon.tsx';
import { Logo } from '../ui/Logo.tsx';
import './builder.css';
import { calls, logLines, Steps, stages, stamp, verdictOf } from './run.tsx';

/** What a native unit does, and how the card names it. */
const NATIVE: Record<string, { name: string; about: string; icon: LucideIcon }> = {
  memory: { name: 'Memory', about: 'Lessons, corrections and handoffs.', icon: Brain },
  hold: {
    name: 'Done when',
    about: 'Hold the ask as a checklist and enforce it.',
    icon: ListChecks,
  },
  plan: { name: 'Plan', about: 'Write down what done means before the code.', icon: PencilRuler },
  plan_lite: {
    name: 'Plan (lite)',
    about: 'Copy and check only the outputs the ask states.',
    icon: PencilRuler,
  },
  challenge: {
    name: 'Challenge',
    about: 'Additional scrutiny before a landing.',
    icon: Swords,
  },
  gate: { name: 'End gate', about: 'Verify changes before landing.', icon: ShieldCheck },
  approval: {
    name: 'Approval',
    about: 'Ask you before a push, deploy or delete.',
    icon: UserCheck,
  },
  secrets: {
    name: 'Secret files',
    about: 'Keep .env, keys and state files from the model.',
    icon: KeyRound,
  },
  reader: {
    name: 'Reader',
    about: 'Derive "Done when" lines from a prose ask.',
    icon: BookOpenText,
  },
  what_changed: { name: 'What changed', about: 'Compare before and after.', icon: GitCompare },
  fold: { name: 'Folding', about: 'Compact old tool output.', icon: Shrink },
  stress: { name: 'Stress signals', about: 'Detect repeated or stalled work.', icon: Gauge },
  returns: { name: 'Returns', about: 'Read the tree back after each act.', icon: RotateCcw },
};

/** The cards' order round the core: half on the left, the rest on the right. */
const ORDER = [
  'memory',
  'hold',
  'plan',
  'plan_lite',
  'challenge',
  'gate',
  'approval',
  'secrets',
  'reader',
  'what_changed',
  'fold',
  'stress',
  'returns',
];

const rank = (id: string) => {
  const i = ORDER.indexOf(id);
  return i < 0 ? ORDER.length : i;
};

function described(u: BoardUnit) {
  return (
    NATIVE[u.id] ?? {
      name: u.id.replace(/^plugin:/, ''),
      about: "A plugin from the human's global file.",
      icon: Puzzle,
    }
  );
}

function Switch({ name, on, onFlip }: { name: string; on: boolean; onFlip: () => void }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={`${name}: ${on ? 'on' : 'off'}`}
      className="bapp-switch"
      onClick={onFlip}
    />
  );
}

function UnitCard({
  u,
  on,
  onFlip,
  inspect = false,
}: {
  u: BoardUnit;
  on: boolean;
  onFlip: () => void;
  inspect?: boolean;
}) {
  const d = described(u);
  const plugin = u.kind === 'plugin';
  return (
    <article
      className="bapp-unit"
      data-on={on}
      data-kind={plugin ? 'plugin' : 'native'}
      data-unit={u.id}
    >
      <h3>
        <Icon icon={d.icon} className="bapp-unit-icon" />
        <span className="bapp-unitname">
          <Safe text={d.name} />
        </span>
        {!inspect && (
          <small className="bapp-pill">
            <Safe text={plugin ? 'plugin' : `v${u.version}`} />
          </small>
        )}
        <Switch name={d.name} on={on} onFlip={onFlip} />
      </h3>
      <p>
        <Safe text={d.about} />
      </p>
      {inspect ? (
        <footer>
          <span className="bapp-kind" data-kind={plugin ? 'plugin' : 'native'}>
            <Safe text={plugin ? 'plugin' : `native · v${u.version}`} />
          </span>
          <span className="bapp-kind" data-on={on}>
            {on ? (plugin ? 'connected' : 'loaded') : 'off'}
          </span>
          {u.frame && <span className="bapp-kind">set as a run opens</span>}
          {u.tightens && <span className="bapp-kind">a world may hold it on</span>}
          <span className="bapp-kind">
            <Safe text={u.id} />
          </span>
        </footer>
      ) : null}
    </article>
  );
}

type Wire = {
  id: string;
  kind: string;
  side: 'left' | 'right';
  on: boolean;
  d: string;
  a: [number, number];
  b: [number, number];
};

/**
 * The graph's wires: a curve from each unit's card to the core, measured from the cards as they
 * are laid out and drawn again whenever the graph changes size. Decoration only: the cards say
 * what is on.
 */
function Edges({ layout }: { layout: string }) {
  const ref = useRef<SVGSVGElement>(null);
  const [wires, setWires] = useState<Wire[]>([]);
  useLayoutEffect(() => {
    const graph = ref.current?.parentElement;
    if (!graph || layout === '') return;
    const draw = () => {
      const base = graph.getBoundingClientRect();
      const core = graph.querySelector('.bapp-core')?.getBoundingClientRect();
      if (!core || core.width === 0) return;
      const cards = [...graph.querySelectorAll<HTMLElement>('.bapp-stack .bapp-unit')];
      const sides: [boolean, HTMLElement[]][] = [
        [true, cards.filter((c) => c.getBoundingClientRect().right <= core.left)],
        [false, cards.filter((c) => c.getBoundingClientRect().left >= core.right)],
      ];
      const out: Wire[] = [];
      for (const [left, list] of sides) {
        list.forEach((c, i) => {
          const r = c.getBoundingClientRect();
          const x1 = (left ? r.right : r.left) - base.left;
          const y1 = r.top + r.height / 2 - base.top;
          const x2 = (left ? core.left : core.right) - base.left;
          const t = list.length > 1 ? i / (list.length - 1) : 0.5;
          const y2 = core.top - base.top + core.height * (0.18 + 0.64 * t);
          const mid = (x1 + x2) / 2;
          out.push({
            id: c.dataset.unit ?? `${left}-${i}`,
            kind: c.dataset.kind ?? 'native',
            side: left ? 'left' : 'right',
            on: c.dataset.on === 'true',
            d: `M ${x1} ${y1} C ${mid} ${y1}, ${mid} ${y2}, ${x2} ${y2}`,
            a: [x1, y1],
            b: [x2, y2],
          });
        });
      }
      setWires(out);
    };
    draw();
    const watch = new ResizeObserver(draw);
    watch.observe(graph);
    return () => watch.disconnect();
  }, [layout]);
  return (
    <svg ref={ref} className="bapp-edges" aria-hidden="true" focusable="false">
      {wires.map((w) => (
        <g key={w.id}>
          <path
            className="bapp-edge"
            data-kind={w.kind}
            data-side={w.side}
            data-on={w.on}
            d={w.d}
          />
          <circle
            className="bapp-port"
            data-kind={w.kind}
            data-side={w.side}
            data-on={w.on}
            cx={w.a[0]}
            cy={w.a[1]}
            r={3.5}
          />
          <circle
            className="bapp-port"
            data-kind={w.kind}
            data-side={w.side}
            data-on={w.on}
            cx={w.b[0]}
            cy={w.b[1]}
            r={3.5}
          />
        </g>
      ))}
    </svg>
  );
}

// ── the run, read from its entries ──────────────────────────────────────────────────────────────

/** A line of the profile's TOML, coloured as the picture's editor: tables, keys, strings, booleans. */
function TomlLine({ line }: { line: string }) {
  const table = /^\s*\[.*\]\s*$/.test(line);
  if (table)
    return (
      <span className="tl-table">
        <Safe text={line} />
      </span>
    );
  const eq = line.indexOf('=');
  if (eq < 0) return <Safe text={line} />;
  const value = line.slice(eq + 1);
  const v = value.trim();
  const tone =
    v === 'true' ? 'tl-true' : v === 'false' ? 'tl-false' : v.startsWith('"') ? 'tl-str' : 'tl-val';
  return (
    <>
      <span className="tl-key">
        <Safe text={line.slice(0, eq)} />
      </span>
      <span className="tl-eq">=</span>
      <span className={tone}>
        <Safe text={value} />
      </span>
    </>
  );
}

const seconds = (ms: number) => `${(Math.max(0, ms) / 1000).toFixed(1)}s`;

/** The chat `session`, live, with its ask sent once the replay is done. */
function WithLive({
  session,
  ask,
  children,
}: {
  session: string;
  ask: string;
  children: (live: Live) => ReactNode;
}) {
  const live = useLive(session);
  const sent = useRef(false);
  useEffect(() => {
    if (sent.current || live.link !== 'open' || !live.replayed) return;
    sent.current = true;
    void live.prompt(ask);
  }, [live, ask]);
  return <>{children(live)}</>;
}

/** The builder's pages, each at its own address in the workspace. */
export type View = 'builder' | 'plugins' | 'profiles';

const ICONS: Record<View, LucideIcon> = {
  builder: Layers,
  plugins: Puzzle,
  profiles: SlidersHorizontal,
};

const HEADS: Record<View, [string, string]> = {
  builder: [
    'Harness Builder',
    'Compose your coding agent by enabling units and plugins. Everything is swappable.',
  ],
  plugins: ['Plugin inspector', 'Native units and the plugins your global file declares.'],
  profiles: ['Profiles', 'A capability mix for a kind of work, to start from.'],
};

/** A follow-up to the run: a prompt when it waits, steering when it works. */
function SideChat({ live }: { live: Live | undefined }) {
  const [text, setText] = useState('');
  const ready = live !== undefined && live.role === 'controller' && live.link !== 'closed';
  const send = () => {
    const t = text.trim();
    if (!t || !live || !ready) return;
    setText('');
    if (live.view?.busy) void live.steer(t);
    else void live.prompt(t);
  };
  return (
    <div className="bapp-sidechat">
      <label htmlFor="bapp-follow">Message this session</label>
      <textarea
        id="bapp-follow"
        value={text}
        disabled={!ready}
        placeholder={ready ? 'Send a follow-up…' : 'Start a run first'}
        onChange={(e) => setText(e.target.value)}
      />
      <button
        type="button"
        className="bapp-primary"
        disabled={!ready || !text.trim()}
        onClick={send}
      >
        <Icon icon={Send} /> {live?.view?.busy ? 'Steer the run' : 'Send message'}
      </button>
    </div>
  );
}

export function BuilderView({ w, chats, view }: { w: BoardWorld; chats: Chats; view: View }) {
  const load = useHarness(w.id);
  const go = (to: View) => format({ name: 'section', world: w.id, section: to });
  const [profile, setProfile] = useState<string>('');
  const [want, setWant] = useState<Map<string, boolean> | undefined>(undefined);
  const [ask, setAsk] = useState('');
  const [run, setRun] = useState<{ session: string; ask: string; planOn: boolean; from: number }>();
  const [starting, setStarting] = useState(false);
  const [why, setWhy] = useState<string | undefined>();
  const [copied, setCopied] = useState(false);
  const [now, setNow] = useState(() => Date.now());
  const askRef = useRef<HTMLTextAreaElement>(null);

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
  useEffect(() => {
    const t = window.setInterval(() => setNow(Date.now()), 500);
    return () => window.clearInterval(t);
  }, []);

  const units = useMemo(
    () => [...(data?.units ?? [])].sort((a, b) => rank(a.id) - rank(b.id)),
    [data],
  );
  const on = (id: string) => want?.get(id) === true;
  const flip = (id: string) =>
    setWant((m) => {
      const next = new Map(m);
      next.set(id, !next.get(id));
      return next;
    });
  const list = want ? composed(units, base, want) : '';
  const text = want ? toml(profile || 'mine', units, want) : '';
  const enabled = units.filter((u) => on(u.id)).length;

  const start = async () => {
    if (!ask.trim()) {
      askRef.current?.focus();
      return;
    }
    if (starting) return;
    setStarting(true);
    setWhy(undefined);
    try {
      const started = await startChat(w.id, undefined, {
        ...(profile ? { profile } : {}),
        ...(list ? { units: list } : {}),
      });
      setRun({
        session: started.session.id,
        ask: ask.trim(),
        planOn: on('plan'),
        from: Date.now(),
      });
      chats.refresh();
    } catch (e) {
      setWhy(e instanceof Error ? e.message : String(e));
    } finally {
      setStarting(false);
    }
  };
  const exportToml = () => {
    const url = URL.createObjectURL(new Blob([text], { type: 'text/plain' }));
    const a = document.createElement('a');
    a.href = url;
    a.download = `mikata-${profile || 'mine'}.toml`;
    a.click();
    window.setTimeout(() => URL.revokeObjectURL(url), 500);
  };

  const card = (u: BoardUnit) => (
    <UnitCard key={u.id} u={u} on={on(u.id)} onFlip={() => flip(u.id)} />
  );
  const inspectCard = (u: BoardUnit) => (
    <UnitCard key={u.id} u={u} on={on(u.id)} onFlip={() => flip(u.id)} inspect />
  );

  const page = (live: Live | undefined) => {
    const v = live?.view;
    const verdict = verdictOf(v);
    const passed = verdict?.verdict === 'PASS';
    const steps = calls(v?.entries ?? []).length;
    const st = stages(v, run?.planOn ?? on('plan'));
    const busy = v?.busy ?? false;
    const state = !run
      ? 'Ready'
      : verdict
        ? passed
          ? 'Verified'
          : 'Not landed'
        : busy
          ? 'Running'
          : live?.closed
            ? 'Ended'
            : 'Starting';
    const usage = v?.usage;
    const usd = usage?.seen ? `$${usage.usd.toFixed(3)}${usage.unpriced ? '+' : ''}` : '$0.000';
    const lines = logLines(v);
    const [title, sub] = HEADS[view];
    const half = Math.ceil(units.length / 2);
    return (
      <div className="bapp">
        <div className="bapp-layout bapp-embedded">
          <section className="bapp-panel bapp-main" aria-label="The harness">
            <div className="bapp-top">
              <div className="bapp-titled">
                <Icon icon={ICONS[view]} className="bapp-titleicon" />
                <div>
                  <h1>{title}</h1>
                  <p className="bapp-muted">{sub}</p>
                </div>
              </div>
              <label className="bapp-profile">
                Profile{' '}
                <select value={profile} onChange={(e) => setProfile(e.target.value)}>
                  <option value="">default</option>
                  {data?.profiles.map((p) => (
                    <option key={p.name} value={p.name}>
                      {p.name}
                    </option>
                  ))}
                </select>
              </label>
            </div>
            {load.state === 'failed' && (
              <p className="bapp-warn">
                <Safe text={load.why} />
              </p>
            )}
            {load.state === 'loading' && <p className="bapp-muted">Reading the harness…</p>}
            {data && want && view === 'builder' && (
              <>
                <div className="bapp-graph" data-live={busy}>
                  <Edges layout={units.map((u) => `${u.id}:${on(u.id)}`).join(' ')} />
                  <div className="bapp-stack">{units.slice(0, half).map(card)}</div>
                  <section className="bapp-core" aria-label="The core">
                    <Logo className="bapp-core-logo" />
                    <h2>MIKATA</h2>
                    <p>Rust runtime</p>
                    <small>
                      Events · Hooks
                      <br />
                      Sandbox · Journal
                    </small>
                  </section>
                  <div className="bapp-stack">{units.slice(half).map(card)}</div>
                </div>
                <div className="bapp-config">
                  <div className="bapp-confpanel">
                    <div className="bapp-tabs">
                      <button type="button" className="bapp-active">
                        <Icon icon={Code} /> Configuration (TOML)
                      </button>
                      <a className="bapp-tablink" href={go('plugins')}>
                        <Icon icon={Puzzle} /> Plugins
                      </a>
                      <a className="bapp-tablink" href={go('profiles')}>
                        <Icon icon={SlidersHorizontal} /> Profiles
                      </a>
                      <span className="bapp-tabtools">
                        <button
                          type="button"
                          aria-label={copied ? 'Copied' : 'Copy TOML'}
                          title={copied ? 'Copied' : 'Copy TOML'}
                          onClick={() => {
                            void navigator.clipboard?.writeText(text).then(() => setCopied(true));
                          }}
                        >
                          <Icon icon={Copy} />
                        </button>
                        <button type="button" onClick={exportToml}>
                          <Icon icon={Download} /> Save
                        </button>
                      </span>
                    </div>
                    <ol
                      className="bapp-editor"
                      aria-label="The configuration, as TOML"
                      // biome-ignore lint/a11y/noNoninteractiveTabindex: it scrolls; the keys scroll it
                      tabIndex={0}
                    >
                      {text.split('\n').map((line, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: the lines of one text, in order
                        <li key={i}>
                          <TomlLine line={line} />
                        </li>
                      ))}
                    </ol>
                    <p className="bapp-muted bapp-units">
                      --units <Safe text={list || '(as the profile is)'} />
                    </p>
                  </div>
                  <div className="bapp-startbox">
                    <label className="visually-hidden" htmlFor="bapp-ask">
                      Your ask
                    </label>
                    <textarea
                      id="bapp-ask"
                      ref={askRef}
                      className="bapp-ask"
                      value={ask}
                      placeholder="Support 1h30m duration parsing. Keep numeric parsing unchanged."
                      onChange={(e) => setAsk(e.target.value)}
                    />
                    <button
                      type="button"
                      className="bapp-primary bapp-start"
                      disabled={!ask.trim() || starting || busy}
                      onClick={() => void start()}
                    >
                      <Icon icon={Play} /> {starting ? 'Starting…' : 'Start run'}
                    </button>
                    {why && (
                      <p className="bapp-warn" role="alert">
                        <Safe text={why} />
                      </p>
                    )}
                    <dl className="bapp-facts">
                      <div>
                        <dt>
                          <Icon icon={Layers} /> Profile
                        </dt>
                        <dd>
                          <Safe text={profile || 'default'} />
                        </dd>
                      </div>
                      <div>
                        <dt>
                          <Icon icon={Box} /> Units on
                        </dt>
                        <dd>
                          {enabled} / {units.length}
                        </dd>
                      </div>
                      <div>
                        <dt>
                          <Icon icon={History} /> Past runs
                        </dt>
                        <dd>{chats.runs.length}</dd>
                      </div>
                    </dl>
                  </div>
                </div>
              </>
            )}
            {data && want && view === 'plugins' && (
              <div className="bapp-list">
                <p className="bapp-section">Native units</p>
                <div className="bapp-inspect">
                  {units.filter((u) => u.kind === 'native').map(inspectCard)}
                </div>
                <p className="bapp-section">Plugins</p>
                {units.some((u) => u.kind === 'plugin') ? (
                  <div className="bapp-inspect">
                    {units.filter((u) => u.kind === 'plugin').map(inspectCard)}
                  </div>
                ) : (
                  <p className="bapp-muted">
                    No plugin declared: a <span className="mono">[plugins.&lt;id&gt;]</span> table
                    in your global config.toml adds one.
                  </p>
                )}
              </div>
            )}
            {data && view === 'profiles' && (
              <div className="bapp-list">
                <article className="bapp-unit" data-on={profile === ''}>
                  <h3>
                    default
                    <button type="button" onClick={() => setProfile('')}>
                      Apply
                    </button>
                  </h3>
                  <p>Every unit as your configuration has it.</p>
                </article>
                {data.profiles.map((p) => (
                  <article key={p.name} className="bapp-unit" data-on={profile === p.name}>
                    <h3>
                      <Safe text={p.name} />
                      <button type="button" onClick={() => setProfile(p.name)}>
                        Apply
                      </button>
                    </h3>
                    <p className="mono">
                      <Safe text={p.units} />
                    </p>
                    <small>{p.from === 'build' ? "The build's profile" : 'Your global file'}</small>
                  </article>
                ))}
              </div>
            )}
          </section>

          <section className="bapp-panel bapp-live" aria-label="Live run">
            <div className="bapp-top">
              <div className="bapp-titled">
                <Icon icon={Play} className="bapp-titleicon" />
                <div>
                  <h2>Live run</h2>
                  <p className="bapp-runline">
                    <Safe
                      text={
                        run
                          ? `#${(live?.hello?.run ?? run.session).slice(-6)} · ${w.name} · ${profile || 'default'}`
                          : `${w.name} · ${profile || 'default'}`
                      }
                    />
                  </p>
                </div>
              </div>
              {busy ? (
                <button type="button" className="bapp-stop" onClick={() => void live?.interrupt()}>
                  <Icon icon={Square} /> Stop
                </button>
              ) : (
                <span className="bapp-state" data-state={state}>
                  {state === 'Verified' && <Icon icon={Check} />}
                  {state.toUpperCase()}
                </span>
              )}
            </div>
            <Steps st={st} label="Stages" />
            <div className="bapp-log" role="log" aria-label="Run log">
              {run ? (
                <>
                  <p className="bapp-log-ask">
                    $ mikata run "<Safe text={run.ask} />"
                  </p>
                  {lines.map(([mark, tone, line, at], i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: the lines only grow, in order
                    <p key={i}>
                      <time>[{stamp((at ?? run.from) - run.from)}]</time>{' '}
                      <span data-tone={tone}>{mark}</span> <Safe text={line} />
                    </p>
                  ))}
                  {lines.length === 0 && <p className="bapp-muted">Waiting for the first step…</p>}
                </>
              ) : (
                <p className="bapp-muted">Waiting for an ask…</p>
              )}
            </div>
            {verdict && (
              <section className="bapp-result" data-ok={passed} aria-label="The end gate's word">
                <Icon icon={passed ? CircleCheck : CircleX} className="bapp-result-icon" />
                <div>
                  <h3>{passed ? 'Verified' : 'Not landed'}</h3>
                  <p>
                    <Safe
                      text={
                        passed
                          ? "The project's verify command passed and no test was deleted or skipped."
                          : (verdict.failures[0] ??
                            verdict.unverified[0] ??
                            'the end gate refused it')
                      }
                    />
                  </p>
                </div>
              </section>
            )}
            <dl className="bapp-numbers">
              <div>
                <dt>Wall time</dt>
                <dd>{run ? seconds(now - run.from) : '—'}</dd>
                <Icon icon={Clock} />
              </div>
              <div>
                <dt>Steps</dt>
                <dd>{steps}</dd>
                <Icon icon={Layers} />
              </div>
              <div>
                <dt>Cost</dt>
                <dd>{usd}</dd>
                <Icon icon={Coins} />
              </div>
            </dl>
            <p className="bapp-notice">
              A real chat in <Safe text={w.name} />, sandboxed; every act outside the world asks you
              first.
            </p>
            <SideChat live={live} />
            {live && <Approval live={live} />}
          </section>
        </div>
      </div>
    );
  };

  return run ? (
    <WithLive key={run.session} session={run.session} ask={run.ask}>
      {(live) => page(live)}
    </WithLive>
  ) : (
    page(undefined)
  );
}
