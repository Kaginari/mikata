/**
 * Metrics (`#/metrics`): what every ask in the workspaces' session runs did, cost and ended as —
 * read from the runs' journals and the usage ledger ([`useInsights`](../data/insights.ts)). The
 * headline numbers, every ask as a bar of model and tool time with how it ended, how asks end and
 * why they stop, asks over time, the models and the harnesses side by side, the `Done when` list
 * against none, the tools, and how carefully the work went.
 */
import './metrics.css';
import { ChartColumnIncreasing } from 'lucide-react';
import { type ReactNode, useEffect, useMemo, useState } from 'react';
import {
  byDay,
  byList,
  byModel,
  byUnits,
  causes,
  duration,
  ends,
  type Group,
  type InsightAsk,
  type InsightTool,
  money,
  OUTCOMES,
  type Outcome,
  pct,
  secs,
  signals,
  summarize,
  useInsights,
} from '../data/insights.ts';
import type { Load } from '../data/worlds.ts';
import { Safe } from '../safe/Safe.tsx';
import { Icon } from '../ui/Icon.tsx';

const WINDOWS = ['7d', '28d', 'all'] as const;

/** How each ending is said: its word, its mark (never colour alone). */
const ENDS: Record<Outcome, [string, string]> = {
  landed: ['Landed', '✓'],
  blocked: ['Blocked by the end gate', '✕'],
  answered: ['Answered, nothing to land', '–'],
  stopped: ['Stopped', '■'],
  failed: ['Failed', '!'],
  open: ['Running or cut', '○'],
};

/** The most asks the bar chart draws, the newest. */
const BARS = 120;

/** A hover card: what is under the pointer or the focus, as text. */
interface Tip {
  x: number;
  y: number;
  body: ReactNode;
}

function useTip() {
  // The pointer and the focus each hold their own card: leaving one does not take the other's away
  // (a focus that scrolls the page moves the chart from under a resting pointer). The newer shows.
  const [hover, setHover] = useState<Tip | undefined>();
  const [focus, setFocus] = useState<Tip | undefined>();
  const [last, setLast] = useState<'hover' | 'focus'>('hover');
  // A pointer card goes once the pointer is over anything that holds no card. A bar's own
  // `mouseleave` alone is not enough: the browser does not always send it (seen in CI, a card
  // left standing after the pointer moved to the page's corner).
  useEffect(() => {
    if (!hover) return;
    const away = (e: MouseEvent) => {
      if (!(e.target instanceof Element && e.target.closest('[data-tip]'))) setHover(undefined);
    };
    document.addEventListener('mousemove', away);
    document.addEventListener('mouseover', away);
    return () => {
      document.removeEventListener('mousemove', away);
      document.removeEventListener('mouseover', away);
    };
  }, [hover]);
  const on = (body: ReactNode) => ({
    'data-tip': '',
    onMouseMove: (e: React.MouseEvent) => {
      setHover({ x: e.clientX, y: e.clientY, body });
      setLast('hover');
    },
    onMouseLeave: () => setHover(undefined),
    onFocus: (e: React.FocusEvent) => {
      const r = (e.currentTarget as Element).getBoundingClientRect();
      setFocus({ x: r.left + r.width / 2, y: r.top, body });
      setLast('focus');
    },
    onBlur: () => setFocus(undefined),
  });
  const tip = last === 'focus' ? (focus ?? hover) : (hover ?? focus);
  const card = tip ? (
    <div
      className="mx-tip"
      role="tooltip"
      style={{ left: Math.min(tip.x + 14, window.innerWidth - 280), top: tip.y + 14 }}
    >
      {tip.body}
    </div>
  ) : null;
  return { on, card };
}

/** A round step for a time axis: 30 s, 1, 2, 5, 10 or 15 minutes. */
function tickOf(max: number): number {
  return [30, 60, 120, 300, 600, 900, 1800].find((t) => max / t <= 4.5) ?? 3600;
}

const axis = (s: number) => (s >= 60 && s % 60 === 0 ? `${s / 60}m` : `${s}s`);

function AskBars({
  asks,
  on,
}: {
  asks: readonly InsightAsk[];
  on: ReturnType<typeof useTip>['on'];
}) {
  const shown = asks.slice(-BARS);
  const W = 1040;
  const H = 250;
  const L = 44;
  const B = 34;
  const T = 12;
  const tick = tickOf(Math.max(1, ...shown.map(secs)));
  const top = tick * (Math.floor(Math.max(1, ...shown.map(secs)) / tick) + 1);
  const step = (W - L - 8) / Math.max(1, shown.length);
  const bw = Math.max(4, Math.min(28, step - 3));
  const y = (s: number) => ((H - B - T) * s) / top;
  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="mx-chart"
      role="img"
      aria-label={`Every ask, oldest left: its time on the model and on tools, and how it ended; ${shown.length} asks`}
    >
      {Array.from({ length: Math.round(top / tick) + 1 }, (_, k) => {
        const v = k * tick;
        const yy = H - B - y(v);
        return (
          <g key={v}>
            <line className="mx-grid" x1={L} x2={W} y1={yy} y2={yy} />
            <text className="mx-axis" x={L - 8} y={yy + 4} textAnchor="end">
              {axis(v)}
            </text>
          </g>
        );
      })}
      {shown.map((a, i) => {
        const x = L + i * step + (step - bw) / 2;
        const hm = y(a.modelMs / 1000);
        const ht = y(a.toolMs / 1000);
        const base = H - B;
        const [word, mark] = ENDS[a.outcome];
        return (
          <g
            key={`${a.world}${a.run}${a.n}`}
            className="mx-hit"
            tabIndex={0}
            {...on(
              <>
                <p className="mx-tip-head">
                  <Safe
                    text={`${a.at.slice(5, 16).replace('T', ' ')} · ${a.model ?? 'unknown model'}`}
                  />
                </p>
                <p>
                  <b>
                    {mark} {word}
                  </b>
                  {a.cause ? <Safe text={` · ${a.cause}`} /> : null}
                </p>
                <p>
                  model {duration(a.modelMs / 1000)} · tools {duration(a.toolMs / 1000)}
                </p>
                <p>
                  {a.turns} steps · {a.calls} calls · {money(a.usd ?? undefined)}
                </p>
              </>,
            )}
          >
            <rect className="mx-hitbox" x={x - 1.5} y={T} width={bw + 3} height={H - T} />
            <rect
              className="mx-s1"
              x={x}
              y={base - hm}
              width={bw}
              height={Math.max(hm - 1, 0)}
              rx={2}
            />
            {ht > 1.5 && (
              <rect
                className="mx-s2"
                x={x}
                y={base - hm - ht}
                width={bw}
                height={Math.max(ht - 1, 0)}
                rx={2}
              />
            )}
            <text className="mx-mark" data-outcome={a.outcome} x={x + bw / 2} y={H - 12}>
              {mark}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

function Ends({ asks }: { asks: readonly InsightAsk[] }) {
  const e = ends(asks);
  return (
    <>
      <div className="mx-endbar" role="img" aria-label="How the asks ended, as shares">
        {e.map(([o, n]) => (
          <i key={o} data-outcome={o} style={{ flex: n }} />
        ))}
      </div>
      <ul className="mx-endlist">
        {e.map(([o, n]) => (
          <li key={o}>
            <b className="mx-dot" data-outcome={o} aria-hidden="true">
              {ENDS[o][1]}
            </b>
            {ENDS[o][0]}
            <span>{n}</span>
          </li>
        ))}
      </ul>
    </>
  );
}

function Causes({ asks }: { asks: readonly InsightAsk[] }) {
  const c = causes(asks);
  if (c.length === 0) return <p className="mx-muted">No ask stopped or failed.</p>;
  const max = Math.max(...c.map(([, n]) => n));
  return (
    <ul className="mx-bars mx-causes">
      {c.map(([why, n]) => (
        <li key={why}>
          <span className="mx-name">
            <Safe text={why} />
          </span>
          <span className="mx-track">
            <i style={{ width: `${(100 * n) / max}%` }} />
          </span>
          <span className="mx-val">{n}</span>
        </li>
      ))}
    </ul>
  );
}

function OverTime({
  asks,
  on,
}: {
  asks: readonly InsightAsk[];
  on: ReturnType<typeof useTip>['on'];
}) {
  const days = byDay(asks);
  if (days.length < 2)
    return <p className="mx-muted">One day of asks so far: the days fill in as you work.</p>;
  const max = Math.max(...days.map(([, r]) => r.reduce((s, n) => s + n, 0)));
  return (
    <div className="mx-days" role="img" aria-label="Asks per day, by how they ended">
      {days.map(([d, row]) => {
        const total = row.reduce((s, n) => s + n, 0);
        return (
          <div
            key={d}
            className="mx-day"
            // biome-ignore lint/a11y/noNoninteractiveTabindex: a day takes the focus to say what it holds
            tabIndex={0}
            {...on(
              <>
                <p className="mx-tip-head">{d}</p>
                {OUTCOMES.map((o, i) =>
                  row[i] ? (
                    <p key={o}>
                      {ENDS[o][1]} {ENDS[o][0]}: {row[i]}
                    </p>
                  ) : null,
                )}
              </>,
            )}
          >
            <div className="mx-daystack" style={{ height: `${(100 * total) / max}%` }}>
              {OUTCOMES.map((o, i) =>
                row[i] ? <i key={o} data-outcome={o} style={{ flex: row[i] }} /> : null,
              )}
            </div>
            <small>{d.slice(5)}</small>
          </div>
        );
      })}
    </div>
  );
}

function Compared({ rows, label }: { rows: Group[]; label: string }) {
  // A table wider than its card scrolls in the card: never the page.
  return (
    <div className="mx-tablewrap">
      <table className="mx-table">
        <thead>
          <tr>
            <th scope="col">{label}</th>
            <th scope="col">Asks</th>
            <th scope="col">Landed</th>
            <th scope="col">Steps</th>
            <th scope="col">Median</th>
            <th scope="col">Cache</th>
            <th scope="col">$ / landed</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((g) => (
            <tr key={g.key}>
              <td className="mx-mono">
                {/* a provider's model names break after their `/`, never inside a word */}
                {g.key.split('/').map((part, i, all) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: the parts of one name, in order
                  <span key={i}>
                    <Safe text={i < all.length - 1 ? `${part}/` : part} />
                    {i < all.length - 1 && <wbr />}
                  </span>
                ))}
              </td>
              <td>{g.asks}</td>
              <td>
                <span className="mx-pct">
                  <i
                    className="mx-pctbar"
                    style={{ width: `${Math.round((g.s.landedShare ?? 0) * 100)}%` }}
                  />
                </span>{' '}
                {pct(g.s.landedShare)}
              </td>
              <td>{Math.round(g.s.medianTurns)}</td>
              <td>{duration(g.s.medianSecs)}</td>
              <td>{pct(g.s.cacheHit)}</td>
              <td>{money(g.s.perLanded)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function Tools({ tools }: { tools: readonly InsightTool[] }) {
  if (tools.length === 0) return <p className="mx-muted">No tool was called.</p>;
  const max = Math.max(...tools.map((t) => t.calls));
  return (
    <ul className="mx-bars">
      {tools.slice(0, 10).map((t) => (
        <li key={t.name}>
          <span className="mx-name">
            <Safe text={t.name} />
          </span>
          <span className="mx-track">
            <i style={{ width: `${(100 * t.calls) / max}%` }} />
          </span>
          <span className="mx-val">
            {t.calls}
            {t.failed > 0 && <em> · {t.failed} failed</em>}
          </span>
        </li>
      ))}
    </ul>
  );
}

function Card({
  title,
  sub,
  className,
  children,
}: {
  title: string;
  sub?: string;
  className?: string;
  children: ReactNode;
}) {
  return (
    <section className={`pf-card mx-card ${className ?? ''}`} aria-label={title}>
      <h2>{title}</h2>
      {sub && <p className="mx-sub">{sub}</p>}
      {children}
    </section>
  );
}

export function MetricsView({ load }: { load: Load }) {
  const [world, setWorld] = useState('all');
  const [window, setWindow] = useState<(typeof WINDOWS)[number]>('7d');
  const [model, setModel] = useState('all');
  const data = useInsights(world, window);
  const worlds = load.state === 'ready' ? load.data.worlds : [];
  const all = data.state === 'ready' ? data.data.asks : [];
  const models = useMemo(() => [...new Set(all.map((a) => a.model ?? 'unknown'))].sort(), [all]);
  const asks = useMemo(
    () => (model === 'all' ? all : all.filter((a) => (a.model ?? 'unknown') === model)),
    [all, model],
  );
  const s = summarize(asks);
  const sig = signals(asks);
  const { on, card } = useTip();
  const modelMs = asks.reduce((n, a) => n + a.modelMs, 0);
  const toolMs = asks.reduce((n, a) => n + a.toolMs, 0);
  return (
    <div className="pg mx-pg">
      <header className="pg-head">
        <span className="pg-icon" aria-hidden="true">
          <Icon icon={ChartColumnIncreasing} />
        </span>
        <div>
          <p className="ss-eyebrow">
            Board <span>/ metrics</span>
          </p>
          <h1>Metrics</h1>
          <p className="pg-sub">
            What every ask did, cost and ended as — read from its run's journal, the usage ledger
            and the end gate.
          </p>
        </div>
        <div className="mx-filters">
          <select
            className="pf-select"
            aria-label="Workspace"
            value={world}
            onChange={(e) => setWorld(e.target.value)}
          >
            <option value="all">Every workspace</option>
            {worlds.map((w) => (
              <option key={w.id} value={w.id}>
                {w.name}
              </option>
            ))}
          </select>
          <fieldset className="mx-seg">
            <legend className="visually-hidden">Window</legend>
            {WINDOWS.map((w) => (
              <button
                key={w}
                type="button"
                aria-pressed={window === w}
                onClick={() => setWindow(w)}
              >
                {w}
              </button>
            ))}
          </fieldset>
          <select
            className="pf-select"
            aria-label="Model"
            value={model}
            onChange={(e) => setModel(e.target.value)}
          >
            <option value="all">Every model</option>
            {models.map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </select>
        </div>
      </header>
      {data.state === 'loading' && <p className="mx-muted">Reading the journals…</p>}
      {data.state === 'failed' && (
        <p className="mx-warn" role="alert">
          <Safe text={data.reason} />
        </p>
      )}
      {data.state === 'ready' && asks.length === 0 && (
        <p className="mx-muted">
          No ask in this window yet: start a session, and its asks show here.
        </p>
      )}
      {data.state === 'ready' && asks.length > 0 && (
        <div className="mx-grid12">
          <section className="mx-kpis" aria-label="Headline numbers">
            <div>
              <small>Asks</small>
              <b>{s.asks}</b>
              <span>
                {models.length} {models.length === 1 ? 'model' : 'models'}
              </span>
            </div>
            <div>
              <small>Landed</small>
              <b data-good="true">{pct(s.landedShare)}</b>
              <span>{s.landed} through the end gate</span>
            </div>
            <div>
              <small>Spent</small>
              <b>{money(s.usd)}</b>
              <span>
                {money(s.perLanded)} a landed ask
                {s.unpriced > 0 ? ` · ${s.unpriced} calls unpriced` : ''}
              </span>
            </div>
            <div>
              <small>Median ask</small>
              <b>{duration(s.medianSecs)}</b>
              <span>{Math.round(s.medianTurns)} steps</span>
            </div>
            <div>
              <small>Cache hit</small>
              <b>{pct(s.cacheHit)}</b>
              <span>of input read from cache</span>
            </div>
            <div>
              <small>Changed</small>
              <b>
                +{s.added} −{s.removed}
              </b>
              <span>lines</span>
            </div>
          </section>
          <Card
            className="mx-wide"
            title="Every ask"
            sub={`Time on the model and on tools, oldest left${asks.length > BARS ? `, the newest ${BARS}` : ''}; the mark under each says how it ended. Hover or focus one.`}
          >
            <ul className="mx-legend">
              <li>
                <i data-series="1" /> Model
              </li>
              <li>
                <i data-series="2" /> Tools
              </li>
              {OUTCOMES.filter((o) => asks.some((a) => a.outcome === o)).map((o) => (
                <li key={o} data-outcome={o}>
                  {ENDS[o][1]} {ENDS[o][0].toLowerCase()}
                </li>
              ))}
            </ul>
            <AskBars asks={asks} on={on} />
          </Card>
          <Card
            className="mx-side"
            title="How asks end"
            sub="Every ask of the window, by its ending."
          >
            <Ends asks={asks} />
            <h3>Why asks stop</h3>
            <Causes asks={asks} />
          </Card>
          <Card className="mx-half" title="Asks over time" sub="Per day, by how they ended.">
            <OverTime asks={asks} on={on} />
          </Card>
          <Card
            className="mx-half"
            title="Where the time goes"
            sub="Every ask's time, on the model and on tools."
          >
            <div className="mx-split" role="img" aria-label="Model and tool time, as shares">
              <i data-series="1" style={{ flex: modelMs || 1 }} />
              <i data-series="2" style={{ flex: toolMs || 0 }} />
            </div>
            <dl className="mx-sig">
              <dt>Model</dt>
              <dd>
                {pct(modelMs / Math.max(1, modelMs + toolMs))} · {duration(modelMs / 1000)}
              </dd>
              <dt>Tools</dt>
              <dd>
                {pct(toolMs / Math.max(1, modelMs + toolMs))} · {duration(toolMs / 1000)}
              </dd>
            </dl>
          </Card>
          <Card
            className="mx-half"
            title="Models compared"
            sub="Each model's asks: how often they land, how long they take, what a landing costs."
          >
            <Compared rows={byModel(asks)} label="Model" />
          </Card>
          <Card
            className="mx-half"
            title="Harnesses compared"
            sub="Asks by the units their session changed from the default: a unit or a plugin on or off, side by side."
          >
            <Compared rows={byUnits(asks)} label="Units" />
          </Card>
          <Card
            className="mx-half"
            title="Done when"
            sub="Asks that carried a Done when list against those that carried none."
          >
            <Compared rows={byList(asks)} label="Ask" />
          </Card>
          <Card className="mx-quarter" title="Tool mix" sub="Calls by tool; failed calls in red.">
            <Tools tools={data.data.tools} />
          </Card>
          <Card className="mx-quarter" title="Work signals" sub="How carefully it worked.">
            <dl className="mx-sig">
              <dt>Tested after the last write</dt>
              <dd>
                {sig.tested}/{sig.wrote}
              </dd>
              <dt>Failed calls</dt>
              <dd>{pct(sig.failedShare)}</dd>
              <dt>Re-reads</dt>
              <dd>{pct(sig.rereadShare)}</dd>
              <dt>Repaired after a refusal</dt>
              <dd>
                {sig.repaired}/{sig.gated}
              </dd>
            </dl>
          </Card>
          {(data.data.unread > 0 || data.data.notes.length > 0) && (
            <ul className="mx-notes" aria-label="What the numbers do not cover">
              {data.data.unread > 0 && (
                <li>
                  {data.data.unread} {data.data.unread === 1 ? 'run was' : 'runs were'} not read:
                  past the newest 300, too large, or a row that did not read.
                </li>
              )}
              {data.data.notes.map((n) => (
                <li key={n}>
                  <Safe text={n} />
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
      {card}
    </div>
  );
}
