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
  ArrowLeft,
  BookOpenText,
  Box,
  Brain,
  Copy,
  Download,
  FolderOpen,
  Gauge,
  GitCompare,
  History,
  Layers,
  ListChecks,
  Lock,
  type LucideIcon,
  MessagesSquare,
  PencilRuler,
  Play,
  Puzzle,
  RotateCcw,
  ScrollText,
  Send,
  ShieldCheck,
  Shrink,
  SlidersHorizontal,
  Square,
  Swords,
  Wrench,
} from 'lucide-react';
import { type ReactNode, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import type { Call, ChatView, Entry } from '../data/chat.ts';
import { summary } from '../data/chat.ts';
import { type Chats, startChat } from '../data/chats.ts';
import { composed, opening, toml, useHarness } from '../data/harness.ts';
import { type Live, useLive } from '../data/live.ts';
import { shownBody } from '../data/names.ts';
import { format } from '../router.ts';
import { Markdown } from '../safe/Markdown.tsx';
import { Safe } from '../safe/Safe.tsx';
import { Approval } from '../ui/Approval.tsx';
import { Glyph, Icon } from '../ui/Icon.tsx';
import { Logo } from '../ui/Logo.tsx';
import './builder.css';

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
        <Safe text={d.name} />
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
      ) : (
        <small>
          <Safe text={plugin ? 'Plugin' : `Native unit · v${u.version}`} />
          {u.frame && ' · set as a run opens'}
        </small>
      )}
    </article>
  );
}

type Wire = {
  id: string;
  kind: string;
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
          <path className="bapp-edge" data-kind={w.kind} data-on={w.on} d={w.d} />
          <circle
            className="bapp-port"
            data-kind={w.kind}
            data-on={w.on}
            cx={w.a[0]}
            cy={w.a[1]}
            r={3.5}
          />
          <circle
            className="bapp-port"
            data-kind={w.kind}
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

function verdictOf(view: ChatView | undefined) {
  const v = [...(view?.entries ?? [])].reverse().find((e) => e.kind === 'verdict');
  return v?.kind === 'verdict' ? v : undefined;
}

export function stages(view: ChatView | undefined, planOn: boolean): [Stage, StageState][] {
  const entries = view?.entries ?? [];
  const all = calls(entries);
  const done = (c: Call) => c.outcome !== undefined && !c.outcome.isError;
  const planned = all.some((c) => writes(c) && done(c) && pathOf(c).endsWith('plan.md'));
  const worked = all.some((c) => writes(c) && done(c) && !pathOf(c).endsWith('plan.md'));
  const verdict = verdictOf(view);
  const busy = view?.busy ?? false;
  const list: [Stage, boolean][] = [
    ['Plan', planned || worked || verdict !== undefined],
    ['Work', worked || verdict !== undefined],
    ['Verify', verdict !== undefined],
    ['Land', verdict?.verdict === 'PASS'],
  ];
  let now = false;
  return list.map(([s, ok]) => {
    if (s === 'Plan' && !planOn) return [s, 'skipped'];
    if (s === 'Land' && verdict !== undefined && verdict.verdict !== 'PASS') return [s, 'failed'];
    if (ok) return [s, 'done'];
    if (!now && busy) {
      now = true;
      return [s, 'now'];
    }
    return [s, 'next'];
  });
}

/** Plan · Work · Verify · Land, each marked by where the run is. */
function Steps({
  st,
  label,
  compact,
}: {
  st: [Stage, StageState][];
  label: string;
  compact?: boolean;
}) {
  return (
    <ol className={compact ? 'bapp-steps bapp-steps-compact' : 'bapp-steps'} aria-label={label}>
      {st.map(([s, at]) => (
        <li key={s} data-state={at}>
          <Glyph glyph={at === 'done' ? '✓' : at === 'failed' ? '✕' : at === 'now' ? '⣾' : '○'} />{' '}
          {s}
          <span className="visually-hidden">: {at}</span>
        </li>
      ))}
    </ol>
  );
}

/** The log's lines: a mark, its tone, and what happened. */
function logLines(view: ChatView | undefined): [string, string, string][] {
  const out: [string, string, string][] = [];
  for (const e of view?.entries ?? []) {
    for (const c of e.kind === 'tool' ? [e.call] : e.kind === 'group' ? e.calls : []) {
      const tone = c.outcome === undefined ? 'now' : c.outcome.isError ? 'alarm' : 'ok';
      const mark = c.outcome === undefined ? '›' : c.outcome.isError ? '✘' : '✔';
      out.push([mark, tone, `${c.name}: ${summary(c.name, c.input)}`]);
    }
    if (e.kind === 'verdict') {
      const ok = e.verdict === 'PASS';
      out.push([ok ? '✔' : '✘', ok ? 'ok' : 'alarm', `end gate: ${e.verdict.toLowerCase()}`]);
    }
    if (e.kind === 'assistant' && e.head) {
      out.push(['✦', 'holo', `answer: ${e.text.split('\n')[0] ?? ''}`]);
    }
    if (e.kind === 'ending' && e.ending !== 'answered') out.push(['■', 'alarm', e.ending]);
  }
  return out;
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

type View = 'session' | 'builder' | 'plugins' | 'runs' | 'profiles';

const VIEWS: [View, LucideIcon, string][] = [
  ['session', MessagesSquare, 'Sessions'],
  ['builder', Layers, 'Harness builder'],
  ['plugins', Puzzle, 'Plugins'],
  ['runs', History, 'Agent runs'],
  ['profiles', SlidersHorizontal, 'Profiles'],
];

const HEADS: Record<View, [string, string]> = {
  session: ['Agent session', 'Talk to Mikata and inspect the work as it happens.'],
  builder: ['Harness builder', 'Compose the capabilities of your agent.'],
  plugins: ['Plugin inspector', 'Native units and the plugins your global file declares.'],
  runs: ['Agent runs', "This world's chats: the live ones, then the past."],
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

/** The session's conversation as the v5 board draws it: the human's lines and Mikata's answers as
 * message cards; each run of tool calls one quiet line (the live run's log lists every step), and
 * the end gate's word as a pill. The full chat, every card, is a link away. */
function Conversation({ view, me, full }: { view: ChatView; me: string; full: string }) {
  const out: ReactNode[] = [];
  let calls: Call[] = [];
  const flush = (key: string) => {
    if (calls.length === 0) return;
    const failed = calls.filter((c) => c.outcome?.isError).length;
    const running = calls.filter((c) => c.outcome === undefined && !c.stopped).length;
    out.push(
      <p key={`calls-${key}`} className="bapp-calls">
        <Icon icon={Wrench} /> {calls.length} tool call{calls.length === 1 ? '' : 's'}
        {running > 0 ? ` · ${running} running` : failed > 0 ? ` · ${failed} failed` : ' · ok'} ·{' '}
        <a href={full}>see each</a>
      </p>,
    );
    calls = [];
  };
  for (const e of view.entries) {
    if (e.kind === 'tool') {
      calls.push(e.call);
      continue;
    }
    if (e.kind === 'group') {
      calls.push(...e.calls);
      continue;
    }
    flush(e.key);
    if (e.kind === 'user') {
      out.push(
        <article key={e.key} className="bapp-message bapp-message-user">
          <b>YOU</b>
          <Safe text={e.text} lines />
        </article>,
      );
    } else if (e.kind === 'assistant') {
      out.push(
        <article key={e.key} className="bapp-message">
          <b>{me.toUpperCase()}</b>
          <Markdown text={e.text} />
        </article>,
      );
    } else if (e.kind === 'verdict') {
      const ok = e.verdict === 'PASS';
      out.push(
        <p key={e.key} className="bapp-verdict" data-ok={ok}>
          {ok ? '✓ end gate · pass' : `✘ end gate · ${e.verdict.toLowerCase()}`}
          {e.changed.length > 0 && ` · ${e.changed.length} changed`}
        </p>,
      );
    } else if (e.kind === 'ending' && e.ending !== 'answered') {
      out.push(
        <p key={e.key} className="bapp-calls">
          <Safe text={e.ending} />
        </p>,
      );
    }
  }
  flush('end');
  if (view.streaming) {
    out.push(
      <article key="streaming" className="bapp-message">
        <b>{me.toUpperCase()}</b>
        <Markdown text={view.streaming} />
      </article>,
    );
  }
  return <>{out}</>;
}

/** The v5 composer, on the live session: a line runs, or steers the turn that runs. */
function SessionComposer({ live }: { live: Live }) {
  const [text, setText] = useState('');
  const busy = live.view?.busy ?? false;
  const ready = live.role === 'controller' && live.link !== 'closed';
  const send = () => {
    const t = text.trim();
    if (!t || !ready) return;
    setText('');
    if (busy) void live.steer(t);
    else void live.prompt(t);
  };
  return (
    <div className="bapp-composer">
      <textarea
        aria-label="Ask Mikata"
        value={text}
        disabled={!ready}
        placeholder={
          live.link === 'closed'
            ? 'This chat has ended'
            : busy
              ? 'Steer the run: it reads this after its next step…'
              : 'Ask Mikata to change, inspect or explain something…'
        }
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            send();
          }
        }}
      />
      <div className="bapp-composerfoot">
        <small>
          {busy ? 'The run works · Enter steers it' : 'Enter to send'} · Shift+Enter for a newline
        </small>
        <button
          type="button"
          className="bapp-primary"
          disabled={!ready || !text.trim()}
          onClick={send}
        >
          <Icon icon={Send} /> {busy ? 'Steer' : 'Send'}
        </button>
      </div>
    </div>
  );
}

export function BuilderView({
  w,
  worlds,
  chats,
}: {
  w: BoardWorld;
  worlds: readonly BoardWorld[];
  chats: Chats;
}) {
  const load = useHarness(w.id);
  const [view, setView] = useState<View>('session');
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
    const ctx = usage && usage.stressLine > 0 ? usage.occupancy / usage.stressLine : 0;
    const usd = usage?.seen ? `$${usage.usd.toFixed(3)}${usage.unpriced ? '+' : ''}` : '$0.000';
    const progress = st.filter(([, s]) => s === 'done').length / 4;
    const lines = logLines(v);
    const meter = (label: string, value: string, frac: number) => (
      <div className="bapp-metric" key={label} data-kind={label.toLowerCase()}>
        <span>
          {label}
          <b>{value}</b>
        </span>
        <div className="bapp-bar">
          <i style={{ width: `${Math.round(Math.min(1, Math.max(0, frac)) * 100)}%` }} />
        </div>
      </div>
    );
    const [title, sub] = HEADS[view];
    const half = Math.ceil(units.length / 2);
    return (
      <div className="bapp">
        <header className="bapp-head">
          <div className="bapp-brand">
            <span className="bapp-logo">
              <Logo />
            </span>
            <span>
              mikata <span lang="ja">味方</span>
            </span>
            <span className="bapp-branddesc">Composable Coding Agent Harness</span>
          </div>
          <nav className="bapp-nav" aria-label="The builder's views">
            {VIEWS.map(([k, icon, label]) => (
              <button
                key={k}
                type="button"
                aria-current={view === k ? 'page' : undefined}
                className={view === k ? 'bapp-active' : undefined}
                onClick={() => setView(k)}
              >
                <Icon icon={icon} /> {label}
              </button>
            ))}
            <a
              className="bapp-navlink"
              href={format({ name: 'section', world: w.id, section: 'journal' })}
            >
              <Icon icon={ScrollText} /> Journal
            </a>
          </nav>
          <span className="bapp-muted">
            <Glyph glyph="●" /> LOCAL · <Safe text={w.name} />
          </span>
          <button
            type="button"
            className="bapp-primary"
            disabled={starting || busy}
            onClick={() => void start()}
          >
            <Icon icon={Play} /> Run agent
          </button>
        </header>
        <div className="bapp-layout">
          <aside className="bapp-panel bapp-side" aria-label="Workspace">
            <p className="bapp-section">Workspace</p>
            <select
              aria-label="Workspace"
              value={w.id}
              onChange={(e) => {
                window.location.hash = format({
                  name: 'section',
                  world: e.target.value,
                  section: 'builder',
                });
              }}
            >
              {worlds.map((x) => (
                <option key={x.id} value={x.id}>
                  {x.name}
                </option>
              ))}
            </select>
            <p className="bapp-path">
              <Icon icon={FolderOpen} /> <Safe text={w.path} />
            </p>
            <p className="bapp-section">Navigation</p>
            {VIEWS.map(([k, icon, label]) => (
              <button
                key={k}
                type="button"
                className={view === k ? 'bapp-navbtn bapp-active' : 'bapp-navbtn'}
                onClick={() => setView(k)}
              >
                <Icon icon={icon} /> {label}
              </button>
            ))}
            <a className="bapp-navbtn" href={format({ name: 'world', world: w.id })}>
              <Icon icon={ArrowLeft} /> The full board
            </a>
            <p className="bapp-section">Status</p>
            <ul className="bapp-tags" aria-label="Status">
              {data && data.holes.length === 0 ? (
                <li data-tone="ok">
                  <i className="bapp-dot" aria-hidden="true" /> Harness ready
                </li>
              ) : (
                <li data-tone="alarm">
                  <i className="bapp-dot" aria-hidden="true" />{' '}
                  <Safe text={data?.holes.join(' · ') ?? 'reading the harness…'} />
                </li>
              )}
              <li data-tone={live?.hello?.sandbox ? 'holo' : undefined}>
                <Icon icon={Lock} />
                <span>
                  Sandbox: <Safe text={live?.hello?.sandbox ?? 'checked as a run opens'} />
                </span>
              </li>
              <li data-tone="holo">
                <Icon icon={Box} /> {enabled} of {units.length} units enabled
              </li>
            </ul>
            <p className="bapp-section">Usage · this run</p>
            {meter('Steps', String(steps), steps / 100)}
            {meter(
              'Context',
              usage && usage.stressLine > 0
                ? `${Math.round(usage.occupancy / 1000)}k / ${Math.round(usage.stressLine / 1000)}k`
                : '—',
              ctx,
            )}
            {meter('Cost', usd, 0)}
            {meter('Progress', `${Math.round(progress * 100)}%`, progress)}
          </aside>

          <main className="bapp-panel bapp-main">
            <div className="bapp-top">
              <div>
                <h1>{title}</h1>
                <p className="bapp-muted">{sub}</p>
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
            {view === 'session' && (
              <div className="bapp-session">
                <div className="bapp-sessionhead">
                  <p className="bapp-tag" data-tone={live?.link === 'open' ? 'ok' : 'holo'}>
                    <i className="bapp-dot" aria-hidden="true" />{' '}
                    <Safe
                      text={run ? `${run.session} · ${live?.link ?? 'connecting'}` : 'No run yet'}
                    />
                  </p>
                  {run && <Steps st={st} label="Session stages" compact />}
                  <button type="button" onClick={() => setView('builder')}>
                    <Icon icon={Layers} /> Harness builder
                  </button>
                </div>
                <div className="bapp-conversation">
                  {live?.view && live.view.entries.length > 0 ? (
                    <Conversation
                      view={live.view}
                      me={shownBody(live.hello?.body ?? 'mikata')}
                      full={format({ name: 'session', world: w.id, session: run?.session ?? '' })}
                    />
                  ) : (
                    <article className="bapp-message">
                      <b>MIKATA</b>
                      <p>
                        Ready to work in <Safe text={w.name} />. Describe the change you want; with
                        the plan on, I write down what done means before I touch the code.
                      </p>
                    </article>
                  )}
                </div>
                {live ? (
                  <SessionComposer live={live} />
                ) : (
                  <div className="bapp-composer">
                    <textarea
                      aria-label="Ask Mikata"
                      value={ask}
                      placeholder="Ask Mikata to change, inspect or explain something…"
                      onChange={(e) => setAsk(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' && !e.shiftKey) {
                          e.preventDefault();
                          void start();
                        }
                      }}
                    />
                    <div className="bapp-composerfoot">
                      <small>
                        A new chat with this harness · Enter to send · Shift+Enter for a newline
                      </small>
                      <button
                        type="button"
                        className="bapp-primary"
                        disabled={!ask.trim() || starting}
                        onClick={() => void start()}
                      >
                        <Icon icon={Send} /> Send
                      </button>
                    </div>
                  </div>
                )}
              </div>
            )}
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
                <div className="bapp-tabs">
                  <button type="button" className="bapp-active">
                    Configuration (TOML)
                  </button>
                  <button type="button" onClick={() => setView('plugins')}>
                    Plugins
                  </button>
                </div>
                <pre className="bapp-editor">
                  <Safe text={text} lines />
                </pre>
                <div className="bapp-toolbar">
                  <span className="bapp-muted bapp-units">
                    --units <Safe text={list || '(as the profile is)'} />
                  </span>
                  <button
                    type="button"
                    onClick={() => {
                      void navigator.clipboard?.writeText(text).then(() => setCopied(true));
                    }}
                  >
                    <Icon icon={Copy} /> {copied ? 'Copied' : 'Copy TOML'}
                  </button>
                  <button type="button" onClick={exportToml}>
                    <Icon icon={Download} /> Export TOML
                  </button>
                </div>
                <small className="bapp-muted">
                  Paste it into your global config.toml;{' '}
                  <span className="mono">mikata run --profile</span> opens the same harness.
                </small>
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
            {view === 'runs' && (
              <div className="bapp-list">
                <p className="bapp-section">Live</p>
                {chats.live.length === 0 && <p className="bapp-muted">No chat runs now.</p>}
                {chats.live.map((c) => (
                  <a
                    key={c.id}
                    className="bapp-unit bapp-run"
                    href={format({ name: 'session', world: w.id, session: c.id })}
                  >
                    <Safe text={`● ${c.id} · ${c.run}`} />
                  </a>
                ))}
                <p className="bapp-section">Past</p>
                {chats.runs.length === 0 && <p className="bapp-muted">No past run.</p>}
                {chats.runs.map((r) => (
                  <a
                    key={r.id}
                    className="bapp-unit bapp-run"
                    href={format({ name: 'run', world: w.id, run: r.id })}
                  >
                    <Safe text={`${r.state} · ${r.ask || r.id}`} />
                  </a>
                ))}
              </div>
            )}
          </main>

          <section className="bapp-panel bapp-live" aria-label="Live run">
            <div className="bapp-top">
              <h2>Live run</h2>
              <span className="bapp-state" data-state={state}>
                {state.toUpperCase()}
              </span>
            </div>
            <label className="bapp-section" htmlFor="bapp-ask">
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
            <div className="bapp-toolbar">
              <button
                type="button"
                disabled={!live || !busy}
                onClick={() => void live?.interrupt()}
              >
                <Icon icon={Square} /> Stop
              </button>
              <button
                type="button"
                className="bapp-primary"
                disabled={!ask.trim() || starting || busy}
                onClick={() => void start()}
              >
                <Icon icon={Play} /> {starting ? 'Starting…' : 'Start run'}
              </button>
            </div>
            {why && (
              <p className="bapp-warn" role="alert">
                <Safe text={why} />
              </p>
            )}
            <Steps st={st} label="Stages" />
            <div className="bapp-log" role="log" aria-label="Run log">
              {run ? (
                <>
                  <p className="bapp-log-ask">
                    &gt; <Safe text={run.ask} />
                  </p>
                  {lines.map(([mark, tone, line], i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: the lines only grow, in order
                    <p key={i}>
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
                <h3>{passed ? '✓ Verified' : '✘ Not landed'}</h3>
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
              </section>
            )}
            <dl className="bapp-numbers">
              <div>
                <dt>Wall time</dt>
                <dd>{run ? seconds(now - run.from) : '—'}</dd>
              </div>
              <div>
                <dt>Steps</dt>
                <dd>{steps}</dd>
              </div>
              <div>
                <dt>Cost</dt>
                <dd>{usd}</dd>
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
