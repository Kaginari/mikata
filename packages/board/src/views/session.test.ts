import { expect, test } from 'vitest';
import type { ChatView, Entry, Usage } from '../data/chat.ts';
import { THE_ASK } from '../data/chat.ts';
import { deskOf } from '../data/desk.ts';
import { counted, kept, spoken, type Turn, turns, WINDOW, windowed } from './Session.tsx';

const usage: Usage = {
  usd: 0,
  unpriced: false,
  seen: false,
  tokens: 0,
  spentWt: 0,
  occupancy: 0,
  stressLine: 0,
};

const view = (entries: Entry[]): ChatView => ({
  entries,
  streaming: '',
  queued: [],
  busy: false,
  turnTokens: 0,
  usage,
  files: [],
  bodies: [],
  held: 0,
});

test('a turn reads as the human words and the steps that answered them', () => {
  const ask = `Recalled from memory:\n- n1 a note\n\n${THE_ASK}\n\nadd days to format()`;
  const t = turns(
    view([
      { kind: 'user', key: 'u', text: ask },
      {
        kind: 'group',
        key: 'g',
        calls: [
          {
            id: '1',
            name: 'read',
            input: { path: 'd.py' },
            stopped: false,
            outcome: { content: 'x', isError: false, cut: 0 },
          },
        ],
      },
      {
        kind: 'tool',
        key: 'p',
        call: {
          id: '2',
          name: 'write',
          input: { path: '.mikata/tmp/plan.md', content: '# x' },
          stopped: false,
          outcome: { content: 'ok', isError: false, cut: 0 },
        },
      },
      {
        kind: 'tool',
        key: 'e',
        call: {
          id: '3',
          name: 'edit',
          input: { path: 'd.py', old_string: 'a', new_string: 'b' },
          stopped: false,
        },
      },
      {
        kind: 'tool',
        key: 'v',
        call: {
          id: '4',
          name: 'bash',
          input: { command: 'python3 -m unittest -q' },
          stopped: false,
          outcome: { content: 'OK', isError: true, cut: 0 },
        },
      },
      { kind: 'assistant', key: 'a', text: 'Done.\n\n---\n\n@U law d.py check=x', head: true },
      {
        kind: 'verdict',
        key: 'x',
        verdict: 'PASS',
        changed: ['d.py'],
        failures: [],
        unverified: [],
      },
    ]),
    ['python3 -m unittest -q'],
  );
  expect(t).toHaveLength(1);
  expect(t[0]?.ask).toBe('add days to format()');
  const kinds = t[0]?.steps.map((s) => s.kind);
  expect(kinds).toEqual(['look', 'plan', 'edit', 'run', 'say', 'verdict']);
  const [, , edit, run, say, verdict] = t[0]?.steps ?? [];
  expect(edit?.kind === 'edit' && edit.mark).toBe('now');
  expect(run?.kind === 'run' && run.verify && run.mark).toBe('failed');
  expect(say?.kind === 'say' && say.text).toBe('Done.');
  expect(verdict?.kind === 'verdict' && verdict.ok).toBe(true);
});

test("the harness's own lines are said plainly", () => {
  expect(
    counted(
      'memory: needs review, a file it rests on changed: n1a214cdac8e-06f936766279155f-0000000000000000, n1a214d03e53-79c33cabc826b875-0000000000000001',
    ),
  ).toBe('memory: needs review, a file it rests on changed: 2 notes');
  expect(counted('tests failed: see above')).toBe('tests failed: see above');
  expect(
    kept(
      "[memory: 2 claim(s) of summoner's kept, 1 grounded by checked, 0 by corroborated; 1 not checked: it names no check]",
    ),
  ).toBe('memory kept 2 notes · 1 checked · 1 not checked');
  expect(kept('[something else]')).toBeUndefined();
  expect(spoken('Fine.\n@U colony x\n')).toBe('Fine.');
  expect(spoken('a line with @U inside')).toBe('a line with @U inside');
});

test('a desk reads what is there and nothing malformed', () => {
  expect(
    deskOf({ verify: ['cargo test', 3], suggested: null, done: ['D1'], branch: 'main' }),
  ).toEqual({
    verify: ['cargo test'],
    suggested: null,
    done: ['D1'],
    branch: 'main',
  });
  expect(deskOf('nonsense')).toEqual({ verify: [], suggested: null, done: [], branch: null });
});

test('a long session draws its newest rows; a turn cut through keeps its newest steps', () => {
  const say = (key: string) => ({ kind: 'say' as const, key, text: key });
  const turn = (n: number, steps: number): Turn => ({
    key: `t${n}`,
    ask: `ask ${n}`,
    steps: Array.from({ length: steps }, (_, i) => say(`t${n}s${i}`)),
  });
  // Three turns: 1 + 3, 1 + 500, 1 + 2 rows.
  const all = [turn(0, 3), turn(1, 500), turn(2, 2)];
  const { shown, hidden } = windowed(all, WINDOW);
  expect(shown.map((t) => t.key)).toEqual(['t1', 't2']);
  // The cut turn keeps its newest steps and no ask; every row is drawn or counted.
  const cut = shown[0] as Turn;
  expect(cut.ask).toBe('');
  expect(cut.steps).toHaveLength(WINDOW - 3);
  expect(cut.steps.at(-1)?.key).toBe('t1s499');
  expect(hidden).toBe(4 + 501 - (WINDOW - 3));
  // A short session is drawn whole.
  expect(windowed([turn(0, 3)], WINDOW)).toEqual({ shown: [turn(0, 3)], hidden: 0 });
  // A larger limit shows more; enough shows all.
  expect(windowed(all, 10_000).hidden).toBe(0);
});
