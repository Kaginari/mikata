/**
 * A workspace's agent session (`#/w/<world>`, `#/w/<world>/s/<chat>`): the conversation as a
 * timeline of steps, the composer under it, and the Live run beside it — Plan · Work · Verify ·
 * Land, the log, what the run is held to, what it spends.
 *
 * Every line is the session's own: its events, the world's desk (verify line, `Done when`,
 * branch) and the session's hello. A number the core does not give is not drawn; a bar is drawn
 * only against a line the core gives (the context's stress line).
 */

import type { BoardWorld } from '@mikata/protocol';
import {
  Bot,
  Check,
  ChevronDown,
  CircleCheck,
  CircleX,
  Clock,
  Coins,
  Compass,
  Cpu,
  Eye,
  GitBranch,
  Hourglass,
  Layers,
  LoaderCircle,
  type LucideIcon,
  Pencil,
  Power,
  Send,
  Shield,
  ShieldAlert,
  SlidersHorizontal,
  Square,
  SquareCheck,
  User,
  Zap,
} from 'lucide-react';
import {
  type CSSProperties,
  memo,
  type ReactNode,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import type { Call, ChatView, Entry } from '../data/chat.ts';
import { type DoneLine, humanWords, running, summary } from '../data/chat.ts';
import { type Chats, startChat } from '../data/chats.ts';
import {
  GUARDS,
  noteOpened,
  type Opened,
  openedWith,
  sessionOptions,
  useComposition,
} from '../data/compose.ts';
import { type Desk, useDesk } from '../data/desk.ts';
import { changeOf } from '../data/diff.ts';
import { useHarness } from '../data/harness.ts';
import { type Live, useLive } from '../data/live.ts';
import { format } from '../router.ts';
import { Markdown } from '../safe/Markdown.tsx';
import { Safe } from '../safe/Safe.tsx';
import { Approval } from '../ui/Approval.tsx';
import { Icon } from '../ui/Icon.tsx';
import { useAnnouncements } from './Chat.tsx';
import { logLines, Steps, stages, stamp, verdictOf } from './run.tsx';
import { SessionUnits } from './SessionUnits.tsx';
import './session.css';

// ── the steps of a turn, read from its entries ──────────────────────────────────────────────────

type Mark = 'ok' | 'now' | 'failed';

/** One step of the body's turn, as the timeline draws it. */
export type Step =
  | { kind: 'edit'; key: string; mark: Mark; call: Call; path: string }
  | { kind: 'run'; key: string; mark: Mark; call: Call; verify: boolean }
  | { kind: 'look'; key: string; mark: Mark; calls: Call[] }
  | { kind: 'plan'; key: string; mark: Mark }
  | { kind: 'other'; key: string; mark: Mark; call: Call }
  | { kind: 'say'; key: string; text: string }
  | { kind: 'changes'; key: string; text: string }
  | { kind: 'note'; key: string; text: string; alarm: boolean }
  | {
      kind: 'verdict';
      key: string;
      ok: boolean;
      word: string;
      changed: number;
      why: string;
    }
  | { kind: 'ending'; key: string; text: string };

/** A turn: the human's words and the steps that answered them. */
export interface Turn {
  key: string;
  ask: string;
  /** The `Done when` list the ask holds: the human's, a reader's, or a correction kept. */
  list: readonly DoneLine[];
  steps: Step[];
  /** When its first step began (ms), if seen live. */
  began?: number | undefined;
}

const markOf = (c: Call): Mark =>
  c.outcome === undefined ? (c.stopped ? 'failed' : 'now') : c.outcome.isError ? 'failed' : 'ok';

const pathOf = (c: Call): string => {
  const i = c.input as Record<string, unknown> | null;
  return typeof i?.path === 'string' ? i.path : '';
};

const commandOf = (c: Call): string => {
  const i = c.input as Record<string, unknown> | null;
  return typeof i?.command === 'string' ? i.command : '';
};

/** A memory note's id: `n` then hex and digits in dashed parts — never a word a human reads. */
const NOTE_ID = /^n[0-9a-f]{6,}-[0-9a-f-]{8,}[,.;]?$/;

/** A warning's trailing list of note ids, counted: `… changed: 13 notes`. */
export function counted(text: string): string {
  const words = text.split(/\s+/);
  const first = words.findIndex((w) => NOTE_ID.test(w));
  if (first < 0) return text;
  const rest = words.slice(first).filter((w) => w !== '');
  if (rest.length < 2 || !rest.every((w) => NOTE_ID.test(w))) return text;
  return `${words.slice(0, first).join(' ').replace(/:$/, '')}: ${rest.length} notes`;
}

/** The memory's account of a turn, `[memory: 1 claim(s) of summoner's kept, …]`, said plainly. */
export function kept(text: string): string | undefined {
  const m =
    /^\[memory: (\d+) claim\(s\) of .+?'s kept, (\d+) grounded by checked, (\d+) by corroborated(.*)\]$/s.exec(
      text,
    );
  if (!m) return undefined;
  const [, n = '0', checked = '0', seconded = '0', why = ''] = m;
  const parts = [`memory kept ${n} note${n === '1' ? '' : 's'}`];
  if (checked !== '0') parts.push(`${checked} checked`);
  if (seconded !== '0') parts.push(`${seconded} seconded`);
  const unchecked = [...why.matchAll(/; (\d+) not checked/g)].reduce((s, x) => s + Number(x[1]), 0);
  if (unchecked > 0) parts.push(`${unchecked} not checked`);
  return parts.join(' · ');
}

/** The body's answer as the human reads it: its `@U` lines are the harness's. */
export function spoken(text: string): string {
  const lines = text.split('\n').filter((l) => !l.trimStart().startsWith('@U '));
  while (lines.length > 0 && /^\s*(---)?\s*$/.test(lines[lines.length - 1] ?? '')) lines.pop();
  return lines.join('\n');
}

/** The rows the timeline draws at first, and how many more each "Show earlier" adds. */
export const WINDOW = 200;

/**
 * The newest `limit` rows of `all` — a turn's ask is a row, each step a row — and how many rows
 * are left out above. A turn cut through keeps its newest steps, without its ask: a long session
 * draws what is near the end, not every step it ever took.
 */
export function windowed(all: readonly Turn[], limit: number): { shown: Turn[]; hidden: number } {
  const shown: Turn[] = [];
  let room = Math.max(0, limit);
  let hidden = 0;
  for (let i = all.length - 1; i >= 0; i--) {
    const t = all[i] as Turn;
    const rows = t.steps.length + (t.ask ? 1 : 0);
    if (room >= rows) {
      shown.unshift(t);
      room -= rows;
    } else if (room > 0) {
      const steps = t.steps.slice(t.steps.length - Math.min(room, t.steps.length));
      shown.unshift({ ...t, ask: '', steps });
      hidden += rows - steps.length;
      room = 0;
    } else {
      hidden += rows;
    }
  }
  return { shown, hidden };
}

/** Two steps alike for drawing: their own fields the same (lists of calls item by item). */
function sameStep(a: Step, b: Step): boolean {
  if (a === b) return true;
  const ka = Object.keys(a) as (keyof Step)[];
  if (ka.length !== Object.keys(b).length) return false;
  return ka.every((k) => {
    const x = a[k] as unknown;
    const y = b[k] as unknown;
    if (Array.isArray(x) && Array.isArray(y)) {
      return x.length === y.length && x.every((v, i) => v === y[i]);
    }
    return x === y;
  });
}

/** The view's entries as turns of steps. `verify` is the world's verify line. */
export function turns(view: ChatView | undefined, verify: readonly string[]): Turn[] {
  const out: Turn[] = [];
  let turn: Turn | undefined;
  const push = (s: Step) => {
    if (!turn) {
      turn = { key: `t-${s.key}`, ask: '', list: [], steps: [] };
      out.push(turn);
    }
    turn.steps.push(s);
  };
  const line = verify.map((v) => v.trim());
  const callStep = (c: Call, key: string): Step => {
    const mark = markOf(c);
    const path = pathOf(c);
    if (['edit', 'multiedit', 'write', 'patch'].includes(c.name)) {
      return path.endsWith('.mikata/tmp/plan.md') || path.endsWith('plan.md')
        ? { kind: 'plan', key, mark }
        : { kind: 'edit', key, mark, call: c, path };
    }
    if (c.name === 'bash') {
      return { kind: 'run', key, mark, call: c, verify: line.includes(commandOf(c).trim()) };
    }
    return { kind: 'other', key, mark, call: c };
  };
  for (const e of view?.entries ?? []) {
    switch (e.kind) {
      case 'user':
        turn = { key: e.key, ask: humanWords(e.text), list: e.list ?? [], steps: [] };
        out.push(turn);
        break;
      case 'tool':
        if (turn && turn.began === undefined) turn.began = e.call.began;
        push(callStep(e.call, e.key));
        break;
      case 'group': {
        if (turn && turn.began === undefined) turn.began = e.calls[0]?.began;
        const marks = e.calls.map(markOf);
        const mark: Mark = marks.includes('failed')
          ? 'failed'
          : marks.includes('now')
            ? 'now'
            : 'ok';
        push({ kind: 'look', key: e.key, mark, calls: e.calls });
        break;
      }
      case 'assistant': {
        const text = spoken(e.text);
        const last = turn?.steps[turn.steps.length - 1];
        if (last?.kind === 'say' && !e.head) last.text = `${last.text}\n\n${text}`;
        else if (text.trim() !== '') push({ kind: 'say', key: e.key, text });
        break;
      }
      case 'changes':
        push({ kind: 'changes', key: e.key, text: e.text });
        break;
      case 'note':
        push({ kind: 'note', key: e.key, text: e.text, alarm: false });
        break;
      case 'warning': {
        const plain = kept(e.text);
        push({
          kind: 'note',
          key: e.key,
          text: plain ?? counted(e.text),
          alarm: plain === undefined,
        });
        break;
      }
      case 'verdict':
        push({
          kind: 'verdict',
          key: e.key,
          ok: e.verdict === 'PASS',
          word: e.verdict,
          changed: e.changed.length,
          why: e.failures[0] ?? e.unverified[0] ?? '',
        });
        break;
      case 'ending':
        if (e.ending !== 'answered') push({ kind: 'ending', key: e.key, text: e.ending });
        break;
      default:
        break;
    }
  }
  return out;
}

// ── small parts ─────────────────────────────────────────────────────────────────────────────────

const clock = (ms: number | undefined) =>
  ms === undefined
    ? ''
    : new Date(ms).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: false });

const secs = (ms: number | undefined) =>
  ms === undefined ? '' : ms < 1000 ? `${Math.round(ms)}ms` : `${(ms / 1000).toFixed(2)}s`;

const mmss = (ms: number) => {
  const s = Math.max(0, Math.floor(ms / 1000));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
};

function StepMark({ mark }: { mark: Mark }) {
  const [icon, cls]: [LucideIcon, string] =
    mark === 'ok'
      ? [CircleCheck, 'ok']
      : mark === 'failed'
        ? [CircleX, 'alarm']
        : [LoaderCircle, 'now'];
  return (
    <span className="mapp-mark" data-mark={cls}>
      <Icon icon={icon} className={mark === 'now' ? 'mn-spin' : undefined} />
    </span>
  );
}

/** A tick box: lime and ticked when `ok`, an empty frame otherwise. */
function Tick({ ok }: { ok: boolean }) {
  return (
    <span className="mapp-tick" data-ok={ok}>
      {ok && <Icon icon={Check} />}
    </span>
  );
}

/** A command's last lines, `ok`, `passed` and `failed` lit. */
function Output({ text }: { text: string }) {
  const lines = text
    .split('\n')
    .filter((l) => l.trim() !== '')
    .slice(-3);
  return (
    <div className="mapp-out">
      {lines.map((l, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: a fixed slice of lines
        <p key={i}>
          {l
            .split(/(\bok\b\.?|\bOK\b|\bpassed\b|\bFAILED\b|\bfailed\b|\berror\b)/)
            .map((part, j) => {
              const tone = /^(ok\.?|OK|passed)$/.test(part)
                ? 'ok'
                : /^(FAILED|failed|error)$/.test(part)
                  ? 'alarm'
                  : undefined;
              return tone ? (
                // biome-ignore lint/suspicious/noArrayIndexKey: parts of one line, in order
                <b key={j} data-status={tone}>
                  <Safe text={part} />
                </b>
              ) : (
                // biome-ignore lint/suspicious/noArrayIndexKey: parts of one line, in order
                <Safe key={j} text={part} />
              );
            })}
        </p>
      ))}
    </div>
  );
}

function StepBody({ s, desk, landed }: { s: Step; desk: Desk; landed: boolean }) {
  switch (s.kind) {
    case 'plan':
      return (
        <li className="mapp-step">
          <StepMark mark={s.mark} />
          <h3>{s.mark === 'now' ? 'writing Done when list…' : 'wrote Done when list'}</h3>
          {desk.done.length > 0 && (
            <div className="mapp-box mapp-hatch">
              {desk.done.map((d) => (
                <p key={d} className="mapp-check" data-ok={landed}>
                  {landed ? <Icon icon={Check} /> : <span className="mapp-ring" />}{' '}
                  <Safe text={d} />
                </p>
              ))}
            </div>
          )}
        </li>
      );
    case 'edit': {
      const change = changeOf(s.call.name, s.call.input);
      const added = change?.kind === 'diff' ? change.added : change?.lines;
      const removed = change?.kind === 'diff' ? change.removed : 0;
      const shown =
        change?.kind === 'diff'
          ? change.lines.filter((l) => l.kind === 'add').slice(0, 2)
          : (change?.head.slice(0, 2).map((text, n) => ({ kind: 'add' as const, text, n })) ?? []);
      const isNew = s.call.name === 'write' && change?.kind === 'file';
      return (
        <li className="mapp-step">
          <StepMark mark={s.mark} />
          <h3>
            {s.mark === 'now' ? 'editing' : 'edited'} <Safe text={s.path} />
          </h3>
          <div className="mapp-box">
            <p className="mapp-file">
              <Icon icon={Pencil} /> <Safe text={s.path} />
              {added !== undefined && <b data-status="ok">+{added}</b>}
              {removed > 0 && <b data-status="alarm">−{removed}</b>}
              <span className="mapp-chip" data-status="gold">
                {isNew ? 'A added' : 'M modified'}
              </span>
            </p>
            {shown.map((l) => (
              <p key={l.n} className="mapp-sub">
                └ <Safe text={l.text.trim()} />
              </p>
            ))}
            {s.call.outcome?.isError && (
              <p className="mapp-sub" data-status="alarm">
                <Safe text={s.call.outcome.content.split('\n')[0] ?? ''} />
              </p>
            )}
          </div>
        </li>
      );
    }
    case 'run':
      return (
        <li className="mapp-step">
          <StepMark mark={s.mark} />
          <h3>{s.verify ? 'running verify command…' : 'running a check…'}</h3>
          <div className="mapp-box">
            <p className="mapp-cmd">
              $ <Safe text={commandOf(s.call)} />
              {s.call.took !== undefined && (
                <span className="mapp-chip">
                  <Icon icon={Clock} /> {secs(s.call.took)}
                </span>
              )}
            </p>
            {s.call.outcome && <Output text={s.call.outcome.content} />}
          </div>
        </li>
      );
    case 'look':
      return (
        <li className="mapp-step mapp-step-quiet">
          <StepMark mark={s.mark} />
          <h3>
            looking: {s.calls.length} {s.calls.length === 1 ? 'call' : 'calls'} —{' '}
            <Safe
              text={s.calls
                .slice(0, 3)
                .map((c) => `${c.name} ${summary(c.name, c.input)}`.trim())
                .join(' · ')}
            />
          </h3>
        </li>
      );
    case 'other':
      return (
        <li className="mapp-step mapp-step-quiet">
          <StepMark mark={s.mark} />
          <h3>
            <Safe text={`${s.call.name} ${summary(s.call.name, s.call.input)}`.trim()} />
          </h3>
        </li>
      );
    case 'say':
      return (
        <li className="mapp-step mapp-say">
          <div className="mapp-box mapp-answer">
            <Markdown text={s.text} />
          </div>
        </li>
      );
    case 'changes': {
      const [head = '', ...rest] = s.text.split('\n');
      const lines = rest.filter((l) => l.trim() !== '' && !l.includes('what-changed.md'));
      return (
        <li className="mapp-step mapp-step-quiet">
          <h3 className="mapp-holo">
            <Safe text={head} />
          </h3>
          <div className="mapp-box">
            {lines.slice(0, 8).map((l, i) => (
              <p
                // biome-ignore lint/suspicious/noArrayIndexKey: a fixed slice of lines
                key={i}
                data-status={l.startsWith('- ') ? 'alarm' : l.startsWith('+ ') ? 'holo' : undefined}
              >
                <Safe text={l} />
              </p>
            ))}
            {lines.length > 8 && <p className="mapp-sub">… +{lines.length - 8} lines</p>}
          </div>
        </li>
      );
    }
    case 'note':
      return (
        <li className="mapp-note" data-status={s.alarm ? 'gold' : undefined}>
          <Safe text={s.text} />
        </li>
      );
    case 'verdict':
      return (
        <li className="mapp-landed" data-ok={s.ok}>
          <Icon icon={s.ok ? Check : CircleX} className="mapp-landed-icon" />
          <p>
            <b>{s.ok ? 'Landed:' : s.word === 'NOT_VERIFIED' ? 'Not verified:' : 'Not landed:'}</b>{' '}
            <Safe
              text={
                s.ok
                  ? `the end gate passed — your verify line ran green and no test was deleted or skipped${s.changed > 0 ? ` · ${s.changed} changed` : ''}.`
                  : s.why || 'the end gate refused it.'
              }
            />
          </p>
        </li>
      );
    case 'ending':
      return (
        <li className="mapp-note" data-status="alarm">
          ■ <Safe text={s.text} />
        </li>
      );
    default:
      return null;
  }
}

/** A step, drawn again only when it changed: an event redraws the step it touched, not all. */
const StepView = memo(
  StepBody,
  (a, b) => a.desk === b.desk && a.landed === b.landed && sameStep(a.s, b.s),
);

// ── the screen ──────────────────────────────────────────────────────────────────────────────────

/** No ask: one array, so the announcements' effect does not rerun for a new empty one. */
const NO_ASKS: Live['asks'] = [];

type State = 'Ready' | 'Starting' | 'Running' | 'Verified' | 'Not landed' | 'Waiting' | 'Ended';

/** The Live run: Plan · Work · Verify · Land, the log, Done when, the guardrails, the numbers. */
function LiveRun({
  v,
  desk,
  hello,
  planOn,
  state,
  wall,
  steps,
  cost,
  ctx,
  landed,
  doneList,
  guardsOff,
  onStop,
}: {
  v: ChatView | undefined;
  desk: Desk;
  hello: Live['hello'];
  planOn: boolean;
  state: State;
  wall: number;
  steps: number;
  cost: string;
  ctx: number | undefined;
  landed: boolean;
  /** The `Done when` list the current ask is held to ([`currentList`]). */
  doneList: readonly string[];
  /** The guards the session runs without, as it says them. */
  guardsOff: readonly string[];
  onStop: (() => void) | undefined;
}) {
  // The Live run follows the ask now running: what an earlier ask did is in the timeline.
  const now = useMemo(() => latestTurn(v), [v]);
  const st = stages(now, planOn);
  const gateOn = !guardsOff.includes('gate');
  const askOn = !guardsOff.includes('approval');
  const secretsOn = !guardsOff.includes('secrets');
  const lines = logLines(now);
  const from = lines.find((l) => l[3] !== undefined)?.[3];
  const sandboxOn = hello ? !hello.sandbox.startsWith('none') : undefined;
  const log = useRef<HTMLDivElement>(null);
  // biome-ignore lint/correctness/useExhaustiveDependencies: follows the log as it grows
  useEffect(() => {
    const el = log.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [lines.length]);
  const done = landed ? doneList.length : 0;
  return (
    <aside className="ss-live" aria-label="Live run">
      <div className="ss-livehead">
        <h2>
          <Icon icon={Zap} /> Live run
        </h2>
        {onStop ? (
          <button type="button" className="ss-stop" onClick={onStop}>
            <Icon icon={Square} /> Stop
          </button>
        ) : (
          <span className="ss-badge" data-state={state}>
            {state.toUpperCase()}
          </span>
        )}
      </div>
      <Steps st={st} label="Stages" />
      {hello && (
        <p className="ss-model" data-busy={v?.busy ? 'true' : 'false'} aria-live="polite">
          <Icon icon={Cpu} />
          <b className="ss-modelname">
            <Safe text={hello.model} />
          </b>
          <span>{doing(now)}</span>
        </p>
      )}
      <div className="ss-log" role="log" aria-label="Run log" ref={log}>
        {lines.length === 0 ? (
          <p className="ss-muted">
            {state === 'Ready' ? 'Waiting for an ask…' : 'Waiting for the first step…'}
          </p>
        ) : (
          lines.map(([mark, tone, line, at], i) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: the lines only grow, in order
            <p key={i} style={{ '--i': Math.min(i, 12) } as CSSProperties}>
              <time>[{stamp((at ?? from ?? 0) - (from ?? 0))}]</time>{' '}
              <span data-status={tone}>{mark}</span> <Safe text={line} />
            </p>
          ))
        )}
        {v?.busy && <span className="ss-caret" aria-hidden="true" />}
      </div>

      <Fold
        id="done"
        label="Done when"
        head={
          <>
            <Icon icon={SquareCheck} /> Done when{' '}
            <small>
              {done}/{doneList.length}
            </small>
          </>
        }
      >
        {doneList.length === 0 ? (
          <p className="ss-muted">
            No list for this ask: write one under <code>## Done when</code>, or name a reader.
          </p>
        ) : (
          doneList.map((d) => (
            <p key={d} className="ss-check" data-ok={landed}>
              <Tick ok={landed} /> <Safe text={d} />
            </p>
          ))
        )}
      </Fold>

      <Fold
        id="guardrails"
        label="Guardrails"
        head={
          <>
            <Icon icon={Shield} /> Guardrails
          </>
        }
      >
        <p className="ss-check" data-ok={sandboxOn === true}>
          <Tick ok={sandboxOn === true} />
          <span>Sandbox</span>
          <b>{sandboxOn === undefined ? 'at start' : sandboxOn ? 'on' : 'off'}</b>
        </p>
        <p className="ss-check" data-ok={gateOn && desk.verify.length > 0}>
          <Tick ok={gateOn && desk.verify.length > 0} />
          <span>Verify must pass</span>
          <b>{!gateOn ? 'gate off' : desk.verify.length > 0 ? 'pinned' : 'none'}</b>
        </p>
        <p className="ss-check" data-ok={gateOn}>
          <Tick ok={gateOn} />
          <span>Test preservation</span>
          <b>{gateOn ? 'on' : 'off'}</b>
        </p>
        <p className="ss-check" data-ok={askOn}>
          <Tick ok={askOn} />
          <span>External actions</span>
          <b>{askOn ? 'ask first' : 'run unasked'}</b>
        </p>
        <p className="ss-check" data-ok={secretsOn}>
          <Tick ok={secretsOn} />
          <span>Secret files</span>
          <b>{secretsOn ? 'held' : 'readable'}</b>
        </p>
      </Fold>

      <dl className="ss-tiles">
        <div>
          <dt>
            <Icon icon={Clock} /> Wall time
          </dt>
          <dd>{mmss(wall)}</dd>
        </div>
        <div>
          <dt>
            <Icon icon={Layers} /> Steps
          </dt>
          <dd>{steps}</dd>
        </div>
        <div>
          <dt>
            <Icon icon={Coins} /> Cost
          </dt>
          <dd>{cost}</dd>
        </div>
      </dl>
      {ctx !== undefined && (
        <div className="ss-meter">
          <span>
            Context <b>{Math.round(ctx * 100)}%</b>
          </span>
          <div>
            <i style={{ width: `${Math.min(100, Math.max(2, ctx * 100))}%` }} />
          </div>
        </div>
      )}
    </aside>
  );
}

function Screen({
  w,
  live,
  desk,
  onAsk,
  starting,
  why,
  planOn,
  number,
  opened,
}: {
  w: BoardWorld;
  live: Live | undefined;
  desk: Desk;
  onAsk: (text: string) => void;
  starting: boolean;
  why: string | undefined;
  planOn: boolean;
  number: number;
  /** What this tab opened the session with, and the guards that were switched off for it. */
  opened?: { with: Opened; guardsOff: string[] } | undefined;
}) {
  const v = live?.view;
  const hello = live?.hello;
  const [draft, setDraft] = useState('');
  const [configuring, setConfiguring] = useState(false);
  const [now, setNow] = useState(() => Date.now());
  const convo = useRef<HTMLElement>(null);
  useEffect(() => {
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, []);
  useAnnouncements(v, live?.asks ?? NO_ASKS);
  const all = useMemo(() => turns(v, desk.verify), [v, desk.verify]);
  const last = all[all.length - 1];
  const verdict = [...(last?.steps ?? [])].reverse().find((s) => s.kind === 'verdict');
  const landed = verdict?.kind === 'verdict' && verdict.ok;
  const doneNow = useMemo(() => currentList(last, desk), [last, desk]);
  const [limit, setLimit] = useState(WINDOW);
  const { shown, hidden } = useMemo(() => windowed(all, limit), [all, limit]);
  // biome-ignore lint/correctness/useExhaustiveDependencies: follows the transcript as it grows
  useEffect(() => {
    const el = convo.current;
    if (el) el.scrollTo?.({ top: el.scrollHeight, behavior: 'smooth' });
  }, [v?.entries.length, v?.streaming]);

  const busy = v?.busy ?? false;
  const steps = (v?.entries ?? []).reduce(
    (n, e) => n + (e.kind === 'tool' ? 1 : e.kind === 'group' ? e.calls.length : 0),
    0,
  );
  const lastBegan = last?.began;
  const lastEnd = [...(v?.entries ?? [])]
    .reverse()
    .flatMap((e: Entry) => (e.kind === 'tool' ? [e.call] : e.kind === 'group' ? e.calls : []))
    .find((c) => c.began !== undefined);
  const wall =
    v?.since !== undefined
      ? now - v.since
      : lastBegan !== undefined && lastEnd?.began !== undefined
        ? lastEnd.began + (lastEnd.took ?? 0) - lastBegan
        : 0;
  const u = v?.usage;
  const cost = u?.seen ? `$${u.usd.toFixed(3)}${u.unpriced ? '+' : ''}` : '$0.000';
  const ctx = u && u.stressLine > 0 ? u.occupancy / u.stressLine : undefined;
  const k = (n: number) => (n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n));
  const sandboxOn = hello ? !hello.sandbox.startsWith('none') : undefined;
  // The guards switched off, as the session itself says them (every tab, wherever it was started);
  // what this tab noted is only a fallback for a server that does not say them.
  // The latest word wins: a `guards` event after a turn, else the hello as the session opened.
  const said = v?.guardsOff ?? (hello as { guardsOff?: unknown } | undefined)?.guardsOff;
  const guardsOff = Array.isArray(said)
    ? said.filter((g): g is string => typeof g === 'string')
    : (opened?.guardsOff ?? []);
  const state: State = starting
    ? 'Starting'
    : verdict?.kind === 'verdict'
      ? verdict.ok
        ? 'Verified'
        : 'Not landed'
      : busy
        ? 'Running'
        : live?.closed
          ? 'Ended'
          : live && all.length > 0
            ? 'Waiting'
            : 'Ready';
  const heading = all.find((t) => t.ask)?.ask.split('\n')[0] ?? 'Untitled session';
  const observer = live?.role === 'observer';
  const ready = !live || (live.role === 'controller' && live.link !== 'closed');
  // As the terminal app does: a line sent while a turn runs waits behind it; steering — Steer, or
  // `/send <text>` — is the explicit act that folds words into the running turn.
  const send = (steer = false) => {
    let text = draft.trim();
    if (text.startsWith('/send ')) {
      text = text.slice(6).trim();
      steer = true;
    }
    if (!text || starting || !ready) return;
    setDraft('');
    if (steer && busy && live) void live.steer(text);
    else onAsk(text);
  };

  return (
    <div className="mapp ss" data-state={state}>
      <section className="ss-main" aria-label="Agent session">
        <header className="ss-head">
          <div className="ss-titles">
            <p className="ss-eyebrow">
              Agent session <span>/ {String(number).padStart(2, '0')}</span>
            </p>
            <h1>
              <Safe text={heading.length > 90 ? `${heading.slice(0, 88)}…` : heading} />
              <span className="ss-badge" data-state={state}>
                {state === 'Running' && <i className="ss-pulse" aria-hidden="true" />}
                {state.toUpperCase()}
              </span>
            </h1>
            <ul className="ss-chips" aria-label="This session">
              {guardsOff.length > 0 && (
                <li data-status="alarm" title="Guards this session runs without">
                  <Icon icon={ShieldAlert} /> <Safe text={`guards off: ${guardsOff.join(', ')}`} />
                </li>
              )}
              {opened && (
                <li title="What this session was opened with, from the Harness Builder">
                  <Icon icon={SlidersHorizontal} />{' '}
                  <Safe
                    text={
                      [
                        opened.with.profile,
                        opened.with.units,
                        opened.with.steps && `${opened.with.steps} steps`,
                        opened.with.minutes && `${opened.with.minutes} min`,
                        opened.with.context && `${Math.round(opened.with.context / 1000)}k context`,
                      ]
                        .filter(Boolean)
                        .join(' · ') || 'default harness'
                    }
                  />
                </li>
              )}
              {desk.branch && (
                <li>
                  <Icon icon={GitBranch} /> <Safe text={desk.branch} />
                </li>
              )}
              <li data-status={sandboxOn === false ? 'alarm' : sandboxOn ? 'ok' : undefined}>
                <Icon icon={Shield} /> Sandbox{' '}
                {sandboxOn === undefined ? '' : sandboxOn ? 'on' : 'off'}
              </li>
              {hello && (
                <li>
                  <Icon icon={Cpu} /> <Safe text={hello.model} />
                </li>
              )}
              {hello && (
                <li>
                  <Icon icon={Layers} /> <Safe text={hello.mode} />
                </li>
              )}
            </ul>
          </div>
          {live && live.link !== 'closed' && (
            <button
              type="button"
              className="ss-configure"
              aria-haspopup="dialog"
              onClick={() => setConfiguring(true)}
            >
              <Icon icon={SlidersHorizontal} /> Configure
            </button>
          )}
          {live?.role === 'controller' && live.link !== 'closed' && (
            <button type="button" className="ss-configure" onClick={() => void live.end()}>
              <Icon icon={Power} /> End session
            </button>
          )}
        </header>

        <section className="ss-convo" aria-label="The conversation" ref={convo}>
          {all.length === 0 && (
            <div className="ss-empty">
              <span className="ss-orb" aria-hidden="true">
                <Icon icon={Bot} />
              </span>
              <h2>
                Ready in <Safe text={w.name} />
              </h2>
              <p>
                Say what to change and how you will know it is done.{' '}
                {guardsOff.includes('approval')
                  ? 'Approval is off for this session: acts outside the workspace run without asking.'
                  : 'Every act outside the workspace asks you first.'}
              </p>
            </div>
          )}
          {hidden > 0 && (
            <button
              type="button"
              className="ss-earlier"
              onClick={() => setLimit((l) => l + WINDOW)}
            >
              Show earlier steps <small>({hidden} more)</small>
            </button>
          )}
          {shown.map((t, ti) => (
            <article
              key={t.key}
              className="mapp-turn"
              style={{ '--i': Math.min(ti, 6) } as CSSProperties}
            >
              {t.ask && (
                <div className="ss-msg ss-msg-user">
                  <p className="mapp-who">
                    <Icon icon={User} /> You <time>{clock(t.began)}</time>
                  </p>
                  <div className="mapp-ask">
                    <Safe text={t.ask} lines />
                  </div>
                </div>
              )}
              {t.steps.length > 0 && (
                <div className="ss-msg">
                  <p className="mapp-who mapp-who-body">
                    <Icon icon={Bot} /> Mikata <time>{clock(t.began)}</time>
                  </p>
                  <ol className="mapp-steps">
                    {t.steps.map((s) => (
                      <StepView
                        key={s.key}
                        s={s}
                        desk={desk}
                        landed={t.key === last?.key && landed}
                      />
                    ))}
                  </ol>
                </div>
              )}
            </article>
          ))}
          {v?.streaming && (
            <div className="mapp-box mapp-answer ss-streaming">
              <Markdown text={spoken(v.streaming)} />
              <span className="ss-caret" aria-hidden="true" />
            </div>
          )}
          {busy && (
            <p className="mapp-working">
              <Icon icon={LoaderCircle} className="mn-spin" /> working… {mmss(wall)} ·{' '}
              {k(v?.turnTokens ?? 0)} tokens
            </p>
          )}
        </section>

        {v && v.queued.length > 0 && (
          <ul className="ss-queued" aria-label="Queued">
            {v.queued.map((q, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: the queue's lines, in order
              <li key={i}>
                <Icon icon={Hourglass} /> queued: <Safe text={q} />
              </li>
            ))}
          </ul>
        )}
        {live &&
          live.link !== 'closed' &&
          (observer || (live.role === undefined && live.hello)) && (
            <p className="ss-observer">
              <Icon icon={Eye} />
              <span>
                {live.controlled
                  ? 'This tab observes: another tab of this browser controls the session.'
                  : 'No tab controls this session.'}
              </span>
              <button type="button" className="ss-configure" onClick={() => void live.takeOver()}>
                Take over
              </button>
            </p>
          )}
        <form
          className="ss-composer"
          data-busy={busy}
          onSubmit={(e) => {
            e.preventDefault();
            send();
          }}
        >
          <textarea
            aria-label="Ask Mikata"
            rows={2}
            value={draft}
            disabled={!ready}
            placeholder={
              live?.link === 'closed'
                ? 'This session has ended'
                : busy
                  ? 'Queued for after this turn — or Steer to fold it into the run…'
                  : 'Ask Mikata to change, inspect or explain something…'
            }
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                send();
              }
              if (e.key === 'Escape' && busy) void live?.interrupt();
            }}
          />
          <div className="ss-composerfoot">
            <span className="ss-keys">
              <kbd>Enter</kbd> {busy ? 'queue' : 'send'} <kbd>Shift+Enter</kbd> newline
              {busy && (
                <>
                  {' '}
                  <kbd>Esc</kbd> stop
                </>
              )}
            </span>
            {busy && (
              <button type="button" className="ss-stop" onClick={() => void live?.interrupt()}>
                <Icon icon={Square} /> Stop
              </button>
            )}
            {busy && (
              <button
                type="button"
                className="ss-configure"
                title="Fold these words into the running turn: it reads them after its next step"
                disabled={!draft.trim() || !ready}
                onClick={() => send(true)}
              >
                <Icon icon={Compass} /> Steer
              </button>
            )}
            <button
              type="submit"
              className="pf-primary"
              disabled={!draft.trim() || starting || !ready}
            >
              <Icon icon={Send} /> {busy ? 'Queue' : 'Send'}
            </button>
          </div>
        </form>
        {why && (
          <p className="mapp-why" role="alert">
            <Safe text={why} />
          </p>
        )}
      </section>

      <LiveRun
        v={v}
        desk={desk}
        hello={hello}
        planOn={planOn}
        state={state}
        wall={wall}
        steps={steps}
        cost={cost}
        ctx={ctx}
        landed={landed}
        doneList={doneNow}
        guardsOff={guardsOff}
        onStop={busy && live ? () => void live.interrupt() : undefined}
      />
      {live && configuring && (
        <SessionUnits w={w} live={live} onClose={() => setConfiguring(false)} />
      )}
      {live && <Approval live={live} />}
    </div>
  );
}

/** The chat `session`, live. */
function WithLive({ session, children }: { session: string; children: (live: Live) => ReactNode }) {
  const live = useLive(session);
  return <>{children(live)}</>;
}

/**
 * A workspace's session: the one `session` names, else its newest live one, else a composer
 * whose first ask opens a new session at its own address.
 */
export function SessionView({
  w,
  chats,
  session,
}: {
  w: BoardWorld;
  chats: Chats;
  session?: string | undefined;
}) {
  const desk = useDesk(w.id);
  const harness = useHarness(w.id);
  const c = useComposition(w.id);
  // The plan as the workspace's composition has it: a flip over the configuration, else the unit.
  const planOn =
    c.flips.plan ??
    (harness.state === 'ready'
      ? (harness.data.units.find((u) => u.id === 'plan')?.on ?? false)
      : true);
  const id = session ?? chats.live[0]?.id;
  const at = chats.live.findIndex((c) => c.id === id);
  // What this tab opened the session with: a guard switched off for it is said in the alarm's tone.
  const opened = useMemo(() => {
    const with_ = id ? openedWith(id) : undefined;
    if (!with_) return undefined;
    const guardsOff = (with_.units ?? '')
      .split(',')
      .filter((t) => t.startsWith('-') && GUARDS.has(t.slice(1)))
      .map((t) => t.slice(1));
    return { with: with_, guardsOff };
  }, [id]);
  const number = at < 0 ? chats.live.length + 1 : chats.live.length - at;

  if (!id) return <NoSession w={w} />;
  return (
    <WithLive key={id} session={id}>
      {(live) => (
        <Screen
          w={w}
          live={live}
          desk={desk}
          onAsk={(t) => void live.prompt(t)}
          starting={false}
          why={live.closed}
          planOn={planOn}
          number={number}
          opened={opened}
        />
      )}
    </WithLive>
  );
}

/** A workspace with no session running: a session is started in the Harness Builder, where its
 * profile and units are chosen. */
function NoSession({ w }: { w: BoardWorld }) {
  return (
    <div className="mapp ss ss-none">
      <section className="ss-main" aria-label="Agent session">
        <div className="ss-empty">
          <span className="ss-orb" aria-hidden="true">
            <Icon icon={Bot} />
          </span>
          <h2>
            No session runs in <Safe text={w.name} />
          </h2>
          <p>
            A session starts from the Harness Builder, where its profile and units are chosen; with
            the approval unit on, every act outside the workspace asks you first.
          </p>
        </div>
      </section>
    </div>
  );
}

/** What the model does now, in a few words: what the Live run says beside its name. */
export function doing(v: ChatView | undefined): string {
  if (!v?.busy) return 'idle';
  if (verdictOf(v) === undefined && v.answered) return 'answered · the end gate checks';
  const calls = v.entries.flatMap((e) =>
    e.kind === 'tool' ? [e.call] : e.kind === 'group' ? e.calls : [],
  );
  const live = calls.findLast(running);
  if (live) return `running ${live.name}`;
  if (v.streaming !== '') return 'writing';
  return 'thinking';
}

/** A session's view from its latest ask on: the whole view when it holds no ask yet. */
export function latestTurn(v: ChatView | undefined): ChatView | undefined {
  const at = v?.entries.findLastIndex((e) => e.kind === 'user') ?? -1;
  return v && at > 0 ? { ...v, entries: v.entries.slice(at) } : v;
}

/**
 * The `Done when` list the current ask is held to: the list its own text carries (the human's, a
 * reader's, or a correction kept); else the plan's, only when this ask wrote the plan. A list left
 * in the plan by an earlier ask is that ask's, never this one's.
 */
export function currentList(turn: Turn | undefined, desk: Desk): string[] {
  if (!turn) return [];
  if (turn.list.length > 0) return turn.list.map(([name, text]) => `${name} ${text}`);
  return turn.steps.some((s) => s.kind === 'plan') ? desk.done : [];
}

/** The folds a reader closed, kept in this browser. */
const FOLDS = 'mikata.live.folds';

function foldsShut(): Set<string> {
  try {
    const v = JSON.parse(window.localStorage.getItem(FOLDS) ?? '[]') as unknown;
    return new Set(Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string') : []);
  } catch {
    return new Set();
  }
}

/**
 * A section of the Live run the reader may fold and unfold (its head a button), open unless they
 * closed it; kept closed in this browser until they open it again.
 */
function Fold({
  id,
  label,
  head,
  children,
}: {
  id: string;
  label: string;
  head: ReactNode;
  children: ReactNode;
}) {
  const [open, setOpen] = useState(() => !foldsShut().has(id));
  const toggle = () =>
    setOpen((was) => {
      const shut = foldsShut();
      if (was) shut.add(id);
      else shut.delete(id);
      try {
        window.localStorage.setItem(FOLDS, JSON.stringify([...shut]));
      } catch {
        // kept for this page only
      }
      return !was;
    });
  return (
    <section className="ss-card ss-fold" aria-label={label} data-open={open}>
      <h3>
        <button
          type="button"
          className="ss-foldhead"
          aria-expanded={open}
          aria-controls={`ss-fold-${id}`}
          onClick={toggle}
        >
          <Icon icon={ChevronDown} className="ss-chevron" />
          {head}
        </button>
      </h3>
      <div id={`ss-fold-${id}`} className="ss-foldbody" hidden={!open}>
        {children}
      </div>
    </section>
  );
}
