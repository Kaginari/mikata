import { expect, test } from 'vitest';
import type { ChatView, DoneLine, Entry, Usage } from '../data/chat.ts';
import { askOf, THE_ASK } from '../data/chat.ts';
import { deskOf } from '../data/desk.ts';
import { stages } from './run.tsx';
import {
  counted,
  currentList,
  kept,
  latestTurn,
  phaseOf,
  spoken,
  type Turn,
  turns,
  WINDOW,
  windowed,
} from './Session.tsx';

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
  answered: false,
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
      { kind: 'user', key: 'u', text: ask, list: [] },
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
  expect(
    kept(
      "[memory: 1 claim(s) of summoner's kept, 0 grounded by checked, 0 by corroborated; 1 held for a later check: each cites a test this session wrote; 2 earlier grounded: their tests stood unchanged through this gate]",
    ),
  ).toBe('memory kept 1 note · 1 held · 2 grounded later');
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
    list: [],
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

test('a new ask shows its own Done when list, never the plan an earlier ask left', () => {
  const desk = { verify: [], suggested: null, done: ['D1 a line of an earlier ask'], branch: null };
  const turn = (list: DoneLine[], planned: boolean): Turn => ({
    key: 't',
    ask: 'ask',
    list,
    steps: planned ? [{ kind: 'plan', key: 'p', mark: 'ok' }] : [],
  });
  expect(currentList(turn([], false), desk)).toEqual([]);
  expect(currentList(turn([], true), desk)).toEqual(['D1 a line of an earlier ask']);
  expect(currentList(turn([['D1', 'parse("1d") must give 1440']], false), desk)).toEqual([
    'D1 parse("1d") must give 1440',
  ]);
  expect(currentList(undefined, desk)).toEqual([]);
});

test("an ask shows the human's words and the list the harness holds, not its notes to the body", () => {
  const text = `${THE_ASK}\n\nmake total skip blanks\n\n## Done when\nD1: total(['1', '', '2']) must give 3\nD2: total(['4']) must stay as it is today\n\nD2 is the human's own word: they said an earlier answer was wrong.\n\nThe ask holds a \`## Done when\` list (D1 to D2), and the harness holds your work to it.`;
  const { words, list } = askOf(text);
  expect(list).toEqual([
    ['D1', "total(['1', '', '2']) must give 3"],
    ['D2', "total(['4']) must stay as it is today"],
  ]);
  expect(words).toContain('make total skip blanks');
  expect(words).toContain('D1: total');
  expect(words).not.toContain('harness holds your work');
  expect(words).not.toContain("the human's own word");
  // A command the human pinned to a line is said right under the list, in the same paragraph.
  const pinned = `${THE_ASK}\n\nfix it\n\n## Done when\nD1: total(['1']) must give 1\nD1 is checked by the human's own command, handed in already: \`t 1\`.\n\nThe ask holds a \`## Done when\` list (D1), and the harness holds your work to it.`;
  expect(askOf(pinned).list).toEqual([['D1', "total(['1']) must give 1"]]);
});

test('the Live run follows the ask now running, not what an earlier ask did', () => {
  const verdict: Entry = {
    kind: 'verdict',
    key: 'x',
    verdict: 'PASS',
    changed: ['d.py'],
    failures: [],
    unverified: [],
  };
  const first: Entry = { kind: 'user', key: 'u1', text: 'one', list: [] };
  const second: Entry = { kind: 'user', key: 'u2', text: 'two', list: [] };
  expect(latestTurn(view([first, verdict, second]))?.entries).toEqual([second]);
  expect(latestTurn(view([first, verdict]))?.entries).toEqual([first, verdict]);
  expect(latestTurn(undefined)).toBeUndefined();
});

test('the stepper says what the body does now: writing a file is Work until it answers', () => {
  const edit = (id: string, path: string, done: boolean): Entry => ({
    kind: 'tool',
    key: id,
    call: {
      id,
      name: 'edit',
      input: { path },
      stopped: false,
      ...(done ? { outcome: { content: 'ok', isError: false, cut: 0 } } : {}),
    },
  });
  const ask: Entry = { kind: 'user', key: 'u', text: 'fix it', list: [] };
  const states = (v: ChatView, planOn = false) =>
    stages(v, planOn)
      .map(([, s]) => s)
      .join(' ');
  const running = (entries: Entry[], answered = false) => ({
    ...view(entries),
    busy: true,
    answered,
  });
  expect(states(running([ask]))).toBe('skipped now next next');
  expect(states(running([ask]), true)).toBe('now next next next');
  // One file written, more to come: still Work, never Verify.
  expect(states(running([ask, edit('1', 'd.py', true), edit('2', 'e.py', false)]))).toBe(
    'skipped now next next',
  );
  // The model answered: the end gate runs.
  expect(states(running([ask, edit('1', 'd.py', true)], true))).toBe('skipped done now next');
  // The gate's word.
  const passed: Entry = {
    kind: 'verdict',
    key: 'x',
    verdict: 'PASS',
    changed: ['d.py'],
    failures: [],
    unverified: [],
  };
  expect(states(view([ask, edit('1', 'd.py', true), passed]))).toBe('skipped done done done');
  // Refused, and the body goes on: back at work, Land not failed yet.
  const refused = { ...passed, verdict: 'FAIL' } as Entry;
  expect(states(running([ask, edit('1', 'd.py', true), refused]))).toBe('skipped now next next');
  expect(
    states(view([ask, edit('1', 'd.py', true), { ...passed, verdict: 'FAIL' } as Entry])),
  ).toBe('skipped done done failed');
});

test('the phase says what the running ask does: a failed call it can repair is recovering, a refusal is blocked', () => {
  const ask: Entry = { kind: 'user', key: 'u', text: 'fix it', list: [] };
  const call = (id: string, name: string, out?: boolean): Entry => ({
    kind: 'tool',
    key: id,
    call: {
      id,
      name,
      input: {},
      stopped: false,
      ...(out === undefined ? {} : { outcome: { content: 'x', isError: !out, cut: 0 } }),
    },
  });
  const verdict = (word: string): Entry => ({
    kind: 'verdict',
    key: `v${word}`,
    verdict: word,
    changed: ['a.py'],
    failures: word === 'PASS' ? [] : ['pytest -q'],
    unverified: [],
  });
  const busy = (entries: Entry[], extra: Partial<ChatView> = {}) => ({
    ...view(entries),
    busy: true,
    ...extra,
  });
  expect(phaseOf(busy([ask])).phase).toBe('thinking');
  expect(phaseOf(busy([ask, call('1', 'bash')]))).toEqual({
    phase: 'running',
    word: 'running bash',
  });
  expect(phaseOf(busy([ask, call('1', 'bash', false)]))).toEqual({
    phase: 'recovering',
    word: 'repairing a failed bash',
  });
  // A later call that worked: back to work, the failure stays a mark on its own line.
  expect(phaseOf(busy([ask, call('1', 'bash', false), call('2', 'edit', true)])).phase).toBe(
    'thinking',
  );
  expect(phaseOf(busy([ask], { streaming: 'Now I' })).word).toBe('writing');
  expect(phaseOf(busy([ask], { answered: true })).phase).toBe('checking');
  expect(phaseOf(busy([ask]), 1).phase).toBe('waiting');
  // The gate refused and the body goes on: it repairs.
  expect(phaseOf(busy([ask, verdict('FAIL')])).phase).toBe('recovering');
  // Done: the gate's word.
  expect(phaseOf(view([ask, verdict('PASS')])).phase).toBe('verified');
  expect(phaseOf(view([ask, verdict('FAIL')])).phase).toBe('blocked');
  expect(phaseOf(view([ask, { kind: 'ending', key: 'e', ending: 'interrupted' }])).phase).toBe(
    'paused',
  );
  expect(phaseOf(view([ask])).phase).toBe('idle');
});
