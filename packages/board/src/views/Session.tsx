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
  CircleCheck,
  CircleX,
  Clock,
  Coins,
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
  SlidersHorizontal,
  Square,
  SquareCheck,
  User,
  Zap,
} from 'lucide-react';
import { type CSSProperties, type ReactNode, useEffect, useMemo, useRef, useState } from 'react';
import type { Call, ChatView, Entry } from '../data/chat.ts';
import { humanWords, summary } from '../data/chat.ts';
import { type Chats, startChat } from '../data/chats.ts';
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
import { logLines, Steps, stages, stamp } from './run.tsx';
import './session.css';

// ── the steps of a turn, read from its entries ──────────────────────────────────────────────────

type Mark = 'ok' | 'now' | 'failed';

/** One step of the body's turn, as the timeline draws it. */
type Step =
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
interface Turn {
  key: string;
  ask: string;
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

/** The view's entries as turns of steps. `verify` is the world's verify line. */
export function turns(view: ChatView | undefined, verify: readonly string[]): Turn[] {
  const out: Turn[] = [];
  let turn: Turn | undefined;
  const push = (s: Step) => {
    if (!turn) {
      turn = { key: `t-${s.key}`, ask: '', steps: [] };
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
        turn = { key: e.key, ask: humanWords(e.text), steps: [] };
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

function StepView({ s, desk, landed }: { s: Step; desk: Desk; landed: boolean }) {
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
  onStop: (() => void) | undefined;
}) {
  const st = stages(v, planOn);
  const lines = logLines(v);
  const from = lines.find((l) => l[3] !== undefined)?.[3];
  const sandboxOn = hello ? !hello.sandbox.startsWith('none') : undefined;
  const log = useRef<HTMLDivElement>(null);
  // biome-ignore lint/correctness/useExhaustiveDependencies: follows the log as it grows
  useEffect(() => {
    const el = log.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [lines.length]);
  const done = landed ? desk.done.length : 0;
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

      <section className="ss-card" aria-label="Done when">
        <h3>
          <Icon icon={SquareCheck} /> Done when{' '}
          <small>
            {done}/{desk.done.length}
          </small>
        </h3>
        {desk.done.length === 0 ? (
          <p className="ss-muted">No list yet: the plan writes one, or say what done means.</p>
        ) : (
          desk.done.map((d) => (
            <p key={d} className="ss-check" data-ok={landed}>
              <Tick ok={landed} /> <Safe text={d} />
            </p>
          ))
        )}
      </section>

      <section className="ss-card" aria-label="Guardrails">
        <h3>
          <Icon icon={Shield} /> Guardrails
        </h3>
        <p className="ss-check" data-ok={sandboxOn === true}>
          <Tick ok={sandboxOn === true} />
          <span>Sandbox</span>
          <b>{sandboxOn === undefined ? 'at start' : sandboxOn ? 'on' : 'off'}</b>
        </p>
        <p className="ss-check" data-ok={desk.verify.length > 0}>
          <Tick ok={desk.verify.length > 0} />
          <span>Verify must pass</span>
          <b>{desk.verify.length > 0 ? 'pinned' : 'none'}</b>
        </p>
        <p className="ss-check" data-ok="true">
          <Tick ok />
          <span>Test preservation</span>
          <b>on</b>
        </p>
        <p className="ss-check" data-ok="true">
          <Tick ok />
          <span>External actions</span>
          <b>ask first</b>
        </p>
      </section>

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
}: {
  w: BoardWorld;
  live: Live | undefined;
  desk: Desk;
  onAsk: (text: string) => void;
  starting: boolean;
  why: string | undefined;
  planOn: boolean;
  number: number;
}) {
  const v = live?.view;
  const hello = live?.hello;
  const [draft, setDraft] = useState('');
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
  const heading = all.find((t) => t.ask)?.ask.split('\n')[0] ?? 'New session';
  const observer = live?.role === 'observer';
  const ready = !live || (live.role === 'controller' && live.link !== 'closed');
  const send = () => {
    const text = draft.trim();
    if (!text || starting || !ready) return;
    setDraft('');
    if (busy && live) void live.steer(text);
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
          <a
            className="ss-configure"
            href={format({ name: 'section', world: w.id, section: 'builder' })}
          >
            <Icon icon={SlidersHorizontal} /> Configure
          </a>
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
                Say what to change and how you will know it is done. Every act outside the workspace
                asks you first.
              </p>
            </div>
          )}
          {all.map((t, ti) => (
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
                    <Safe text={t.ask} />
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
                      <StepView key={s.key} s={s} desk={desk} landed={t === last && landed} />
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
                  ? 'Steer the run: it reads this after its next step…'
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
              <kbd>Enter</kbd> {busy ? 'steer' : 'send'} <kbd>Shift+Enter</kbd> newline
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
            <button
              type="submit"
              className="pf-primary"
              disabled={!draft.trim() || starting || !ready}
            >
              <Icon icon={Send} /> {busy ? 'Steer' : 'Send'}
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
        onStop={busy && live ? () => void live.interrupt() : undefined}
      />
      {live && <Approval live={live} />}
    </div>
  );
}

/** The chat `session`, live. */
function WithLive({ session, children }: { session: string; children: (live: Live) => ReactNode }) {
  const live = useLive(session);
  return <>{children(live)}</>;
}

/** The asks that opened a session, waiting for its replay; read once by the session's page. */
const opening = new Map<string, string>();

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
  const planOn =
    harness.state === 'ready'
      ? (harness.data.units.find((u) => u.id === 'plan')?.on ?? false)
      : true;
  const [starting, setStarting] = useState(false);
  const [why, setWhy] = useState<string | undefined>();
  const id = session ?? chats.live[0]?.id;
  const at = chats.live.findIndex((c) => c.id === id);
  const number = at < 0 ? chats.live.length + 1 : chats.live.length - at;

  const open = async (text: string) => {
    setStarting(true);
    setWhy(undefined);
    try {
      const started = await startChat(w.id);
      opening.set(started.session.id, text);
      chats.refresh();
      window.location.hash = format({ name: 'session', world: w.id, session: started.session.id });
    } catch (e) {
      setWhy(e instanceof Error ? e.message : String(e));
    } finally {
      setStarting(false);
    }
  };

  if (!id) {
    return (
      <Screen
        w={w}
        live={undefined}
        desk={desk}
        onAsk={(t) => void open(t)}
        starting={starting}
        why={why}
        planOn={planOn}
        number={number}
      />
    );
  }
  return (
    <WithLive key={id} session={id}>
      {(live) => (
        <Sender live={live} id={id}>
          <Screen
            w={w}
            live={live}
            desk={desk}
            onAsk={(t) => void live.prompt(t)}
            starting={starting}
            why={why ?? live.closed}
            planOn={planOn}
            number={number}
          />
        </Sender>
      )}
    </WithLive>
  );
}

/** Sends the ask that opened the chat, once its replay is done. */
function Sender({ live, id, children }: { live: Live; id: string; children: ReactNode }) {
  useEffect(() => {
    const ask = opening.get(id);
    if (ask === undefined || live.link !== 'open' || !live.replayed) return;
    opening.delete(id);
    void live.prompt(ask);
  }, [live, id]);
  return <>{children}</>;
}
