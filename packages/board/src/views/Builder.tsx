/**
 * The Harness Builder and the plugin inspector, as the owner's picture draws them: the units a
 * session in this workspace opens with, as cards round the core with a switch each, a profile to
 * start from, and the composition as TOML. What they compose, the workspace's new sessions open
 * with; nothing here is simulated.
 */

import type { BoardProfile, BoardUnit, BoardWorld } from '@mikata/protocol';
import {
  BookOpenText,
  Box,
  Brain,
  Code,
  Copy,
  Download,
  Gauge,
  GitCompare,
  KeyRound,
  Layers,
  ListChecks,
  type LucideIcon,
  PencilRuler,
  Play,
  Puzzle,
  RotateCcw,
  ShieldAlert,
  ShieldCheck,
  Shrink,
  Swords,
  UserCheck,
} from 'lucide-react';
import { type ReactNode, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { type Chats, startChat } from '../data/chats.ts';
import {
  GUARDS,
  noteOpened,
  sessionOptions,
  setComposition,
  setOnce,
  useComposition,
} from '../data/compose.ts';
import { composed, opening, toml, useHarness } from '../data/harness.ts';
import { format } from '../router.ts';
import { Safe } from '../safe/Safe.tsx';
import { Icon } from '../ui/Icon.tsx';
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

/** No switch flipped. */
const NO_FLIPS: Readonly<Record<string, boolean>> = {};

/** The builder's pages, each at its own address in the workspace. */
export type View = 'builder' | 'plugins';

/**
 * The workspace's harness as the Builder composes it: the units a session opens with, the stored
 * composition over them ([`compose`](../data/compose.ts)), and the switch that changes it.
 */
function useComposed(w: BoardWorld) {
  const load = useHarness(w.id);
  const c = useComposition(w.id);
  const data = load.state === 'ready' ? load.data : undefined;
  const chosen: BoardProfile | undefined = data?.profiles.find((p) => p.name === c.profile);
  const units = useMemo(
    () => [...(data?.units ?? [])].sort((a, b) => rank(a.id) - rank(b.id)),
    [data],
  );
  const base = useMemo(
    () => (data ? opening(data.units, chosen) : new Map<string, boolean>()),
    [data, chosen],
  );
  const over = (...layers: Readonly<Record<string, boolean>>[]) => {
    const m = new Map(base);
    for (const flips of layers) {
      for (const [id, on] of Object.entries(flips)) if (m.has(id)) m.set(id, on);
    }
    return m;
  };
  const onceFlips = c.once?.flips ?? NO_FLIPS;
  // biome-ignore lint/correctness/useExhaustiveDependencies: `over` reads `base`, listed
  const want = useMemo(() => over(c.flips, onceFlips), [base, c.flips, onceFlips]);
  const kept = data ? composed(units, base, over(c.flips)) : '';
  const list = data ? composed(units, base, want) : '';
  // The kept `--units` follows the kept switches over the profile the units have now.
  useEffect(() => {
    if (data && kept !== c.units) {
      setComposition(w.id, { profile: c.profile, flips: c.flips, units: kept });
    }
  }, [data, kept, c, w.id]);
  /** A guard ([`GUARDS`]): switched off, it holds for one session and is never kept. */
  const guard = (id: string) => GUARDS.has(id);
  const on = (id: string) => want.get(id) === true;
  const flip = (id: string) => {
    const to = !on(id);
    const keep: Record<string, boolean> = { ...c.flips };
    const one: Record<string, boolean> = { ...onceFlips };
    delete keep[id];
    delete one[id];
    if (base.get(id) !== to) {
      if (!to && guard(id)) one[id] = false;
      else keep[id] = to;
    }
    setComposition(w.id, {
      profile: c.profile,
      flips: keep,
      units: composed(units, base, over(keep)),
    });
    setOnce(w.id, { flips: one, units: composed(units, base, over(keep, one)) });
  };
  const setProfile = (profile: string) => {
    setOnce(w.id, undefined);
    setComposition(w.id, { profile, flips: {}, units: '' });
  };
  const reset = () => {
    setOnce(w.id, undefined);
    setComposition(w.id, { profile: c.profile, flips: {}, units: '' });
  };
  const flipped = Object.keys(c.flips).length + Object.keys(onceFlips).length;
  return { load, data, c, units, want, list, on, flip, setProfile, reset, flipped, onceFlips };
}

function Head({
  icon,
  title,
  sub,
  children,
}: {
  icon: LucideIcon;
  title: string;
  sub: string;
  children?: ReactNode;
}) {
  return (
    <div className="bapp-top">
      <div className="bapp-titled">
        <Icon icon={icon} className="bapp-titleicon" />
        <div>
          <h1>{title}</h1>
          <p className="bapp-muted">{sub}</p>
        </div>
      </div>
      {children}
    </div>
  );
}

function Loading({ load }: { load: ReturnType<typeof useHarness> }) {
  if (load.state === 'failed') {
    return (
      <p className="bapp-warn">
        <Safe text={load.why} />
      </p>
    );
  }
  return load.state === 'loading' ? <p className="bapp-muted">Reading the harness…</p> : null;
}

/**
 * The Harness Builder (`#/w/<world>/builder`): every unit a session here opens with, as cards round
 * the core with a switch each, the profile to start from, and the composition as a global file's
 * `[profiles.<name>]` (copy, export). What it composes is what the workspace's next sessions open
 * with — New session, or a first ask —, kept in this browser.
 */
function Builder({ w, chats }: { w: BoardWorld; chats: Chats }) {
  const h = useComposed(w);
  const [copied, setCopied] = useState(false);
  const [starting, setStarting] = useState(false);
  const [why, setWhy] = useState<string | undefined>();
  // A new session is started here, where its harness is chosen: with this composition.
  const start = async () => {
    if (starting) return;
    setStarting(true);
    setWhy(undefined);
    try {
      const opts = sessionOptions(w.id);
      const started = await startChat(w.id, undefined, opts);
      noteOpened(started.session.id, opts);
      chats.refresh();
      window.location.hash = format({ name: 'session', world: w.id, session: started.session.id });
    } catch (e) {
      setWhy(e instanceof Error ? e.message : String(e));
    } finally {
      setStarting(false);
    }
  };
  const text = h.data ? toml(h.c.profile || 'mine', h.units, h.want) : '';
  const exportToml = () => {
    const url = URL.createObjectURL(new Blob([text], { type: 'text/plain' }));
    const a = document.createElement('a');
    a.href = url;
    a.download = `mikata-${h.c.profile || 'mine'}.toml`;
    a.click();
    window.setTimeout(() => URL.revokeObjectURL(url), 500);
  };
  const half = Math.ceil(h.units.length / 2);
  const card = (u: BoardUnit) => (
    <UnitCard key={u.id} u={u} on={h.on(u.id)} onFlip={() => h.flip(u.id)} />
  );
  const guardsOff = Object.keys(h.onceFlips);
  return (
    <section className="bapp-panel bapp-main" aria-label="The harness">
      <Head
        icon={Layers}
        title="Harness Builder"
        sub="Choose the profile and units, then start a session with them."
      >
        <label className="bapp-profile">
          Profile{' '}
          <select value={h.c.profile} onChange={(e) => h.setProfile(e.target.value)}>
            <option value="">default</option>
            {h.data?.profiles.map((p) => (
              <option key={p.name} value={p.name}>
                {p.name}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          className="bapp-go"
          disabled={!h.data || starting}
          onClick={() => void start()}
        >
          <Icon icon={Play} /> {starting ? 'Starting…' : 'Start session'}
        </button>
      </Head>
      {why && (
        <p className="bapp-warn" role="alert">
          <Safe text={why} />
        </p>
      )}
      <Loading load={h.load} />
      {h.data && (
        <>
          <div className="bapp-graph">
            <Edges layout={h.units.map((u) => `${u.id}:${h.on(u.id)}`).join(' ')} />
            <div className="bapp-stack">{h.units.slice(0, half).map(card)}</div>
            <section className="bapp-core" aria-label="The core">
              <Logo className="bapp-core-logo" />
              <h2>
                MIKATA <span lang="ja">味方</span>
              </h2>
              <p>Rust runtime</p>
              <small>
                Events · Hooks · Lifecycle
                <br />
                Sandbox · Journal
              </small>
            </section>
            <div className="bapp-stack">{h.units.slice(half).map(card)}</div>
          </div>
          <ul className="bapp-legend" aria-label="Legend">
            <li data-kind="native">Native unit</li>
            <li data-kind="plugin">External plugin</li>
            <li data-kind="off">Disabled</li>
          </ul>
          <div className="bapp-confpanel">
            <div className="bapp-tabs">
              <b>
                <Icon icon={Code} /> Configuration (TOML)
              </b>
              <span className="bapp-tabtools">
                {h.flipped > 0 && (
                  <button type="button" onClick={h.reset}>
                    <Icon icon={RotateCcw} /> Reset to the profile
                  </button>
                )}
                <button
                  type="button"
                  onClick={() => {
                    void navigator.clipboard?.writeText(text).then(() => setCopied(true));
                  }}
                >
                  <Icon icon={Copy} /> {copied ? 'Copied' : 'Copy'}
                </button>
                <button type="button" onClick={exportToml}>
                  <Icon icon={Download} /> Export
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
              New sessions in <Safe text={w.name} /> open with{' '}
              <span className="mono">
                {h.c.profile ? (
                  <>
                    --profile <Safe text={h.c.profile} />{' '}
                  </>
                ) : null}
                --units <Safe text={h.list || '(as the profile is)'} />
              </span>
              . Paste the TOML in your global config.toml to keep it everywhere.
            </p>
            {guardsOff.length > 0 && (
              <p className="bapp-warn" role="status">
                <Icon icon={ShieldAlert} /> Off for the next new session only, never kept:{' '}
                <Safe text={guardsOff.join(', ')} />. A guard switched off loosens what you rely on;
                the session after it opens with it on again.
              </p>
            )}
          </div>
        </>
      )}
    </section>
  );
}

/** The plugin inspector (`#/w/<world>/plugins`): each native unit and declared plugin, its kind,
 * whether it is on in the workspace's composition, and its id — with the same switch. */
function Plugins({ w }: { w: BoardWorld }) {
  const h = useComposed(w);
  const inspect = (u: BoardUnit) => (
    <UnitCard key={u.id} u={u} on={h.on(u.id)} onFlip={() => h.flip(u.id)} inspect />
  );
  return (
    <section className="bapp-panel bapp-main" aria-label="The plugins">
      <Head
        icon={Puzzle}
        title="Plugin inspector"
        sub="Native units and the plugins your global file declares."
      />
      <Loading load={h.load} />
      {h.data && (
        <div className="bapp-list">
          <p className="bapp-section">Native units</p>
          <div className="bapp-inspect">
            {h.units.filter((u) => u.kind === 'native').map(inspect)}
          </div>
          <p className="bapp-section">Plugins</p>
          {h.units.some((u) => u.kind === 'plugin') ? (
            <div className="bapp-inspect">
              {h.units.filter((u) => u.kind === 'plugin').map(inspect)}
            </div>
          ) : (
            <p className="bapp-muted">
              No plugin declared: a <span className="mono">[plugins.&lt;id&gt;]</span> table in your
              global config.toml adds one.
            </p>
          )}
        </div>
      )}
    </section>
  );
}

export function BuilderView({ w, view, chats }: { w: BoardWorld; view: View; chats: Chats }) {
  return (
    <div className="bapp">
      <div className="bapp-layout bapp-embedded">
        {view === 'plugins' ? <Plugins w={w} /> : <Builder w={w} chats={chats} />}
      </div>
    </div>
  );
}
