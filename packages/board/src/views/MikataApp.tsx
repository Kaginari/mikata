/**
 * Mikata as an app (`mikatad app`, `#/w/<world>/app`): one window on one world's session, drawn
 * as the owner's app pictures are — a glowing header bar, the conversation as a timeline of steps
 * on the left, what the run is held to and what it spends on the right, a status bar under it.
 *
 * Every line is the session's own: its events, the world's desk (verify line, `Done when`,
 * branch) and the session's hello. A number the core does not give — a step cap, a time cap — is
 * not drawn; a bar is drawn only against a line the core gives (the context's stress line, a
 * spending cap).
 */

import type { BoardWorld } from '@mikata/protocol';
import {
  Bot,
  ChartColumnIncreasing,
  Check,
  ChevronRight,
  CircleCheck,
  CircleX,
  Clock,
  Coins,
  Layers,
  LoaderCircle,
  type LucideIcon,
  Pencil,
  Play,
  Shield,
  SquareCheck,
  User,
} from 'lucide-react';
import { type ReactNode, useEffect, useMemo, useRef, useState } from 'react';
import type { Call, ChatView, Entry } from '../data/chat.ts';
import { humanWords, summary } from '../data/chat.ts';
import { type Chats, startChat } from '../data/chats.ts';
import { type Desk, useDesk } from '../data/desk.ts';
import { changeOf } from '../data/diff.ts';
import { type Live, useLive } from '../data/live.ts';
import { Markdown } from '../safe/Markdown.tsx';
import { Safe } from '../safe/Safe.tsx';
import { Approval } from '../ui/Approval.tsx';
import { Icon } from '../ui/Icon.tsx';
import './app.css';

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

/** A panel: a neon frame, its icon and title, the slashes at its right. */
function Panel({
  icon,
  title,
  children,
  className,
  label,
}: {
  icon: LucideIcon;
  title: ReactNode;
  children: ReactNode;
  className?: string;
  label?: string;
}) {
  return (
    <section className={`mapp-panel ${className ?? ''}`} aria-label={label}>
      <h2 className="mapp-panel-head">
        <Icon icon={icon} className="mapp-panel-icon" />
        <span>{title}</span>
        <i className="mapp-slashes" aria-hidden="true">
          {'////'}
        </i>
      </h2>
      <div className="mapp-panel-body">{children}</div>
    </section>
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
                <b key={j} data-tone={tone}>
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
              {added !== undefined && <b data-tone="ok">+{added}</b>}
              {removed > 0 && <b data-tone="alarm">−{removed}</b>}
              <span className="mapp-chip" data-tone="gold">
                {isNew ? 'A added' : 'M modified'}
              </span>
            </p>
            {shown.map((l) => (
              <p key={l.n} className="mapp-sub">
                └ <Safe text={l.text.trim()} />
              </p>
            ))}
            {s.call.outcome?.isError && (
              <p className="mapp-sub" data-tone="alarm">
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
                data-tone={l.startsWith('- ') ? 'alarm' : l.startsWith('+ ') ? 'holo' : undefined}
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
        <li className="mapp-note" data-tone={s.alarm ? 'gold' : undefined}>
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
        <li className="mapp-note" data-tone="alarm">
          ■ <Safe text={s.text} />
        </li>
      );
    default:
      return null;
  }
}

// ── the screen ──────────────────────────────────────────────────────────────────────────────────

function Screen({
  w,
  live,
  desk,
  onAsk,
  starting,
  why,
}: {
  w: BoardWorld;
  live: Live | undefined;
  desk: Desk;
  onAsk: (text: string) => void;
  starting: boolean;
  why: string | undefined;
}) {
  const v = live?.view;
  const hello = live?.hello;
  const [draft, setDraft] = useState('');
  const [now, setNow] = useState(() => Date.now());
  const bottom = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, []);
  const all = useMemo(() => turns(v, desk.verify), [v, desk.verify]);
  const last = all[all.length - 1];
  const verdict = [...(last?.steps ?? [])].reverse().find((s) => s.kind === 'verdict');
  const landed = verdict?.kind === 'verdict' && verdict.ok;
  // biome-ignore lint/correctness/useExhaustiveDependencies: follows the transcript as it grows
  useEffect(() => {
    bottom.current?.scrollIntoView({ block: 'end' });
  }, [v?.entries.length, v?.streaming]);

  const busy = v?.busy ?? false;
  const calls = (v?.entries ?? []).reduce(
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
  const cost = u?.seen ? `$${u.usd.toFixed(3)}${u.unpriced ? '+' : ''}` : u ? 'unpriced' : '—';
  const ctx = u && u.stressLine > 0 ? u.occupancy / u.stressLine : undefined;
  const k = (n: number) => (n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n));
  const sandboxOn = hello ? !hello.sandbox.startsWith('none') : undefined;
  const send = () => {
    const text = draft.trim();
    if (!text || starting) return;
    setDraft('');
    onAsk(text);
  };

  return (
    <div className="mapp">
      <header className="mapp-head">
        <span className="mapp-brand">
          <i aria-hidden="true">{'//'}</i> Mikata — coding agent harness <small>(MIT)</small>
        </span>
        <span className="mapp-sep" aria-hidden="true" />
        <dl className="mapp-chips">
          <div>
            <dt>repo:</dt>
            <dd>
              <Safe text={w.name} />
            </dd>
          </div>
          {desk.branch && (
            <div>
              <dt>branch:</dt>
              <dd>
                <Safe text={desk.branch} />
              </dd>
            </div>
          )}
          {hello && (
            <div>
              <dt>mode:</dt>
              <dd className="mapp-pill">
                <Safe text={hello.mode} />
              </dd>
            </div>
          )}
          {hello && (
            <div>
              <dt>model:</dt>
              <dd>
                <Safe text={hello.model} />
              </dd>
            </div>
          )}
          {sandboxOn !== undefined && (
            <div>
              <dt>sandbox:</dt>
              <dd className="mapp-pill" data-off={!sandboxOn}>
                {sandboxOn ? 'on' : 'off'}
              </dd>
            </div>
          )}
        </dl>
      </header>

      <div className="mapp-body">
        <main className="mapp-panel mapp-convo" aria-label="The conversation">
          {all.length === 0 && (
            <div className="mapp-empty">
              <Icon icon={Bot} />
              <p>
                Ready in <Safe text={w.name} />. Say what to change and how you will know it is
                done; every act outside the world asks you first.
              </p>
            </div>
          )}
          {all.map((t) => (
            <article key={t.key} className="mapp-turn">
              {t.ask && (
                <>
                  <p className="mapp-who">
                    <Icon icon={User} /> User <time>{clock(t.began)}</time>
                  </p>
                  <div className="mapp-ask">
                    <Safe text={t.ask} />
                  </div>
                </>
              )}
              {t.steps.length > 0 && (
                <>
                  <p className="mapp-who mapp-who-body">
                    <Icon icon={Bot} /> Mikata <time>{clock(t.began)}</time>
                  </p>
                  <ol className="mapp-steps">
                    {t.steps.map((s) => (
                      <StepView key={s.key} s={s} desk={desk} landed={t === last && landed} />
                    ))}
                  </ol>
                </>
              )}
            </article>
          ))}
          {v?.streaming && (
            <div className="mapp-box mapp-answer">
              <Markdown text={spoken(v.streaming)} />
            </div>
          )}
          {busy && (
            <p className="mapp-working">
              <Icon icon={LoaderCircle} className="mn-spin" /> working… {mmss(wall)} ·{' '}
              {k(v?.turnTokens ?? 0)} tokens
            </p>
          )}
          <div ref={bottom} />
        </main>

        <aside className="mapp-side" aria-label="What the run is held to">
          <Panel icon={SquareCheck} title="Done when" label="Done when">
            {desk.done.length === 0 ? (
              <p className="mapp-muted">
                No list yet: the plan writes one, or say what done means.
              </p>
            ) : (
              desk.done.map((d, i) => (
                <p key={d} className="mapp-check mapp-big" data-ok={landed}>
                  <Tick ok={landed} /> <span className="mapp-dn">D{i + 1}</span> <Safe text={d} />
                </p>
              ))
            )}
          </Panel>
          <Panel icon={ChartColumnIncreasing} title="Budgets" label="Budgets">
            <dl className="mapp-budgets">
              <div>
                <dt>steps</dt>
                <dd>{calls}</dd>
              </div>
              <div>
                <dt>wall clock</dt>
                <dd>{mmss(wall)}</dd>
              </div>
              <div>
                <dt>context</dt>
                <dd>
                  {u && u.stressLine > 0 ? `${k(u.occupancy)} / ${k(u.stressLine)}` : '—'}
                  {ctx !== undefined && (
                    <span className="mapp-meter">
                      <i style={{ width: `${Math.min(100, Math.max(2, ctx * 100))}%` }} />
                      <em>{Math.round(ctx * 100)}%</em>
                    </span>
                  )}
                </dd>
              </div>
              <div>
                <dt>cost</dt>
                <dd>
                  {cost}
                  {hello?.budget && <small> · cap {hello.budget.cap}</small>}
                </dd>
              </div>
            </dl>
          </Panel>
          <Panel icon={Shield} title="Guardrails" label="Guardrails" className="mapp-fill">
            <p className="mapp-check mapp-big" data-ok={desk.verify.length > 0}>
              <Tick ok={desk.verify.length > 0} /> Verify must pass
            </p>
            <p className="mapp-check mapp-big" data-ok="true">
              <Tick ok /> No test deleted or skipped
            </p>
            <p className="mapp-check mapp-big" data-ok="true">
              <Tick ok /> Approval needed for push/PR
            </p>
            <p className="mapp-check mapp-big" data-ok={sandboxOn === true}>
              <Tick ok={sandboxOn === true} />{' '}
              {sandboxOn === undefined
                ? 'Sandbox: checked as a run opens'
                : sandboxOn
                  ? 'Commands sandboxed'
                  : 'No sandbox'}
            </p>
          </Panel>
        </aside>
      </div>

      <form
        className="mapp-input"
        onSubmit={(e) => {
          e.preventDefault();
          send();
        }}
      >
        <Icon icon={ChevronRight} className="mapp-prompt" />
        <textarea
          aria-label="Ask Mikata"
          rows={1}
          value={draft}
          placeholder={
            busy ? 'a line now waits its turn' : 'ask Mikata to change, check or explain something'
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
        <span className="mapp-keys">
          <kbd>Enter</kbd> {busy ? 'queue' : 'run'} <kbd>Shift+Enter</kbd> newline
          {busy && (
            <>
              {' '}
              <kbd>Esc</kbd> stop
            </>
          )}
        </span>
        <button
          type="submit"
          className="mapp-go"
          disabled={!draft.trim() || starting}
          aria-label="Run"
        >
          <Icon icon={Play} />
        </button>
      </form>
      {why && (
        <p className="mapp-why" role="alert">
          <Safe text={why} />
        </p>
      )}

      <footer className="mapp-foot">
        <span className="mapp-verify">
          <Icon icon={ChevronRight} /> <b>verify:</b>{' '}
          <Safe text={desk.verify.join(' && ') || desk.suggested || 'none pinned'} />
          {landed && <Icon icon={Check} className="mapp-ok" />}
        </span>
        <span className="mapp-sep" aria-hidden="true" />
        <span>
          last answer:{' '}
          {verdict?.kind === 'verdict' ? (
            <b data-tone={verdict.ok ? 'ok' : 'alarm'}>{verdict.ok ? 'landed' : 'not landed'}</b>
          ) : (
            <b>—</b>
          )}
        </span>
        <span className="mapp-foot-chips">
          <span className="mapp-chip">
            <Icon icon={Layers} /> steps: {calls}
          </span>
          <span className="mapp-chip">
            <Icon icon={Clock} /> time: {mmss(wall)}
          </span>
          <span className="mapp-chip">
            <Icon icon={Coins} /> cost: {cost}
          </span>
        </span>
      </footer>
      {live && <Approval live={live} />}
    </div>
  );
}

/** The chat `session`, live. */
function WithLive({ session, children }: { session: string; children: (live: Live) => ReactNode }) {
  const live = useLive(session);
  return <>{children(live)}</>;
}

/** The app on `w`: the newest live chat of the world, or a new one at the first ask. */
export function MikataApp({ w, chats }: { w: BoardWorld; chats: Chats }) {
  const desk = useDesk(w.id);
  const [session, setSession] = useState<string | undefined>();
  const [pending, setPending] = useState<string | undefined>();
  const [starting, setStarting] = useState(false);
  const [why, setWhy] = useState<string | undefined>();
  const id = session ?? chats.live[0]?.id;
  useEffect(() => {
    document.title = `mikata — ${w.name}`;
  }, [w.name]);

  const open = async (text: string) => {
    setStarting(true);
    setWhy(undefined);
    try {
      const started = await startChat(w.id);
      setPending(text);
      setSession(started.session.id);
      chats.refresh();
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
      />
    );
  }
  return (
    <WithLive key={id} session={id}>
      {(live) => (
        <Sender live={live} pending={pending} onSent={() => setPending(undefined)}>
          <Screen
            w={w}
            live={live}
            desk={desk}
            onAsk={(t) => void live.prompt(t)}
            starting={starting}
            why={why ?? live.closed}
          />
        </Sender>
      )}
    </WithLive>
  );
}

/** Sends the ask that opened the chat, once its replay is done. */
function Sender({
  live,
  pending,
  onSent,
  children,
}: {
  live: Live;
  pending: string | undefined;
  onSent: () => void;
  children: ReactNode;
}) {
  const sent = useRef(false);
  useEffect(() => {
    if (sent.current || pending === undefined || live.link !== 'open' || !live.replayed) return;
    sent.current = true;
    void live.prompt(pending).then(onSent);
  }, [live, pending, onSent]);
  return <>{children}</>;
}
