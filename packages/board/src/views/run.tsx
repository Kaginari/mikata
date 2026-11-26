/**
 * A run read from its chat: its calls, the end gate's word, where it stands in Plan · Work ·
 * Verify · Land, and its log — shared by the Harness Builder and the sessions' Live run panel.
 */
import type { Call, ChatView, Entry } from '../data/chat.ts';
import { summary } from '../data/chat.ts';
import { Glyph } from '../ui/Icon.tsx';
import './builder.css';

export type Stage = 'Plan' | 'Work' | 'Verify' | 'Land';
export type StageState = 'done' | 'now' | 'next' | 'failed' | 'skipped';

const pathOf = (c: Call): string => {
  const i = c.input as Record<string, unknown> | null;
  return typeof i?.path === 'string' ? i.path : '';
};
const writes = (c: Call) => ['write', 'edit', 'patch'].includes(c.name);

export function calls(entries: readonly Entry[]): Call[] {
  return entries.flatMap((e) => (e.kind === 'tool' ? [e.call] : e.kind === 'group' ? e.calls : []));
}

export function verdictOf(view: ChatView | undefined) {
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
export function Steps({
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
          <span className="bapp-stepdot">
            <Glyph glyph={at === 'done' ? '✓' : at === 'failed' ? '✕' : at === 'now' ? '⣾' : '○'} />
          </span>
          <span className="bapp-stepname">{s}</span>
          <span className="visually-hidden">: {at}</span>
        </li>
      ))}
    </ol>
  );
}

/** The log's lines: a mark, its tone, what happened, and when (ms), if seen live. */
export function logLines(
  view: ChatView | undefined,
): [string, string, string, number | undefined][] {
  const out: [string, string, string, number | undefined][] = [];
  let last: number | undefined;
  for (const e of view?.entries ?? []) {
    for (const c of e.kind === 'tool' ? [e.call] : e.kind === 'group' ? e.calls : []) {
      const tone = c.outcome === undefined ? 'now' : c.outcome.isError ? 'alarm' : 'ok';
      const mark = c.outcome === undefined ? '▸' : c.outcome.isError ? '✘' : '✔';
      last = c.began ?? last;
      out.push([mark, tone, `${c.name}: ${summary(c.name, c.input)}`, c.began]);
    }
    if (e.kind === 'verdict') {
      const ok = e.verdict === 'PASS';
      last = e.at ?? last;
      out.push([ok ? '✔' : '✘', ok ? 'ok' : 'alarm', `end gate: ${e.verdict.toLowerCase()}`, last]);
    }
    if (e.kind === 'assistant' && e.head) {
      out.push(['▸', 'holo', `says: ${e.text.split('\n')[0] ?? ''}`, last]);
    }
    if (e.kind === 'ending' && e.ending !== 'answered') out.push(['■', 'alarm', e.ending, last]);
  }
  return out;
}

/** `ms` since the run began, as the log's `[mm:ss]`. */
export const stamp = (ms: number) => {
  const t = Math.max(0, Math.floor(ms / 1000));
  return `${String(Math.floor(t / 60)).padStart(2, '0')}:${String(t % 60).padStart(2, '0')}`;
};
