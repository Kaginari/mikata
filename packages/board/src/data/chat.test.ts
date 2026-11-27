import type { Arrived } from '@mikata/protocol';
import { describe, expect, it } from 'vitest';
import { Chat, endsTurn, groupCounts, humanWords, summary, THE_ASK } from './chat.ts';
import { parseRuns } from './chats.ts';

let seq = 0;
const ev = (type: string, fields: Record<string, unknown> = {}, body = 'summoner') => {
  seq += 1;
  return { type, session: 's', run: 'r', body, seq, ...fields } as unknown as Arrived;
};

function turn(chat: Chat, events: Arrived[], now?: number) {
  for (const e of events) chat.fold(e, now);
  return chat.view();
}

describe('the transcript', () => {
  it('keeps what changed as its own entry, after the gate', () => {
    const chat = new Chat('summoner');
    const text =
      "what changed: of Mikata's last 2 checks, 1 prints differently than on the code you gave it:\n\n  in: a\n- out: 1\n+ out: 2";
    const v = turn(chat, [
      ev('turnStarted', { text: 'make f add two' }),
      ev('assistant', { text: 'Done.', stop: 'end_turn' }),
      ev('gateVerdict', {
        passed: true,
        verdict: 'PASS',
        changed: ['lib.py'],
        failures: [],
        unverified: [],
      }),
      ev('changes', { text }),
      ev('turnEnded', { ending: 'answered' }),
    ]);
    expect(v.entries.map((e) => e.kind)).toEqual(['user', 'assistant', 'verdict', 'changes']);
    expect(v.entries.at(-1)).toMatchObject({ kind: 'changes', text });
  });

  it('folds a turn in the TUI grammar: the line, the answer, the cards, the gate', () => {
    const chat = new Chat('summoner');
    const v = turn(chat, [
      ev('turnStarted', { text: 'fix the collate parse' }),
      ev('textDelta', { text: "I'll look first.\n\n" }),
      ev('toolStart', { id: 't1', name: 'read' }),
      ev('assistant', { text: "I'll look first.\n\n", stop: 'tool_use' }),
      ev('toolCall', { id: 't1', name: 'read', input: { path: 'parser.py' } }),
      ev('toolResult', { id: 't1', content: '1\ta', isError: false, cut: 0 }),
      ev('toolCall', { id: 't2', name: 'grep', input: { pattern: 'collate' } }),
      ev('toolResult', { id: 't2', content: 'x', isError: false, cut: 0 }),
      ev('toolCall', {
        id: 't3',
        name: 'edit',
        input: { path: 'parser.py', old_string: 'a\n', new_string: 'b\n' },
      }),
      ev('toolResult', { id: 't3', content: 'ok', isError: false, cut: 0 }),
      ev('assistant', { text: 'Fixed.', stop: 'end_turn' }),
      ev('gateVerdict', {
        passed: true,
        verdict: 'PASS',
        changed: ['parser.py'],
        failures: [],
        unverified: [],
      }),
      ev('turnEnded', { ending: 'answered' }),
    ]);
    expect(v.entries.map((e) => e.kind)).toEqual([
      'user',
      'assistant',
      'group',
      'tool',
      'assistant',
      'verdict',
    ]);
    const group = v.entries[2];
    expect(group?.kind === 'group' && groupCounts(group.calls)).toBe(
      'read 1 file · grep 1 pattern',
    );
    expect(v.entries[1]).toMatchObject({ kind: 'assistant', head: true, text: "I'll look first." });
    expect(v.entries[4]).toMatchObject({ kind: 'assistant', head: false, text: 'Fixed.' });
    expect(v.files).toEqual([['parser.py', 'M']]);
    expect(v.busy).toBe(false);
    // Keys are stable and unique: the virtual list keeps rows by them.
    expect(new Set(v.entries.map((e) => e.key)).size).toBe(v.entries.length);
  });

  it('keeps a paragraph streaming inside an open fence', () => {
    const chat = new Chat('summoner');
    const v = turn(chat, [
      ev('turnStarted', { text: 'go' }),
      ev('textDelta', { text: '```py\nx = 1\n\ny = 2\n' }),
    ]);
    expect(v.entries.map((e) => e.kind)).toEqual(['user']);
    expect(v.streaming).toContain('y = 2');
  });

  it('says what a turn that did not answer ended on, and stops its running calls', () => {
    const chat = new Chat('summoner');
    const v = turn(chat, [
      ev('turnStarted', { text: 'go' }),
      ev('toolCall', { id: 't1', name: 'bash', input: { command: 'sleep 9' } }),
      ev('turnEnded', { ending: 'interrupted' }),
    ]);
    const tool = v.entries[1];
    expect(tool?.kind === 'tool' && tool.call.stopped).toBe(true);
    expect(v.entries[2]).toMatchObject({ kind: 'ending', ending: 'interrupted' });
  });

  it('shows a commissioned body as lines below, and in the tree', () => {
    const chat = new Chat('summoner');
    const v = turn(chat, [
      ev('turnStarted', { text: 'go' }),
      ev('turnStarted', { text: 'judge the wire\nmore' }, 'orc-wire'),
      ev('toolCall', { id: 'b1', name: 'bash', input: { command: 'cargo test' } }, 'orc-wire'),
      ev('textDelta', { text: 'not shown' }, 'orc-wire'),
      ev('turnEnded', { ending: 'answered' }, 'orc-wire'),
    ]);
    expect(v.entries.slice(1).map((e) => (e.kind === 'below' ? e.text : e.kind))).toEqual([
      'is working on: judge the wire',
      'bash cargo test',
      'answered',
    ]);
    expect(v.bodies).toEqual([{ name: 'orc-wire', state: 'done', last: 'answered' }]);
  });

  it('counts the spend, and an unpriced turn is never $0', () => {
    const chat = new Chat('summoner');
    const usage = (usd: number | null) =>
      ev('usage', {
        input: 100,
        output: 20,
        cacheRead: 0,
        cacheWrite: 0,
        usd,
        wt: 120,
        spentWt: 120,
        occupancy: 41_000,
        stressLine: 100_000,
      });
    const v = turn(chat, [ev('turnStarted', { text: 'go' }), usage(0.01), usage(null)]);
    expect(v.usage).toMatchObject({ unpriced: true, tokens: 240, occupancy: 41_000 });
    expect(v.usage.usd).toBeCloseTo(0.01);
    expect(v.turnTokens).toBe(240);
  });

  it('takes a queued line off the queue when its turn starts', () => {
    const chat = new Chat('summoner');
    let v = turn(chat, [ev('turnStarted', { text: 'one' }), ev('queued', { text: 'two' })]);
    expect(v.queued).toEqual(['two']);
    v = turn(chat, [ev('turnEnded', { ending: 'answered' }), ev('turnStarted', { text: 'two' })]);
    expect(v.queued).toEqual([]);
    // The harness may say more before the line and after it: the line is still that turn's.
    v = turn(chat, [ev('queued', { text: 'three' })]);
    expect(v.queued).toEqual(['three']);
    v = turn(chat, [
      ev('turnEnded', { ending: 'answered' }),
      ev('turnStarted', {
        text: 'Recalled: a note\n\nthree\n\nThe ask holds a `## Done when` list (D1)',
      }),
    ]);
    expect(v.queued).toEqual([]);
  });

  it('times calls and a long wait only when seen live', () => {
    const chat = new Chat('summoner');
    chat.fold(ev('turnStarted', { text: 'go' }), 1000);
    chat.fold(ev('toolCall', { id: 't', name: 'bash', input: { command: 'ls' } }), 4000);
    chat.fold(ev('toolResult', { id: 't', content: '', isError: false, cut: 0 }), 4500);
    const v = chat.view();
    expect(v.entries[1]).toMatchObject({ kind: 'thought', took: 3000 });
    const tool = v.entries[2];
    expect(tool?.kind === 'tool' && tool.call.took).toBe(500);
    const replayed = new Chat('summoner');
    replayed.fold(ev('turnStarted', { text: 'go' }));
    replayed.fold(ev('toolCall', { id: 't', name: 'bash', input: { command: 'ls' } }));
    expect(replayed.view().entries.map((e) => e.kind)).toEqual(['user', 'tool']);
  });

  it('summarises a call as the TUI does', () => {
    expect(summary('bash', { command: 'pytest -q\nmore' })).toBe('pytest -q');
    expect(summary('read', { path: 'a.py', offset: 10, limit: 5 })).toBe('a.py:10-14');
    expect(summary('grep', { pattern: 'x' })).toBe('x');
    expect(summary('t', { n: 1 })).toBe('{"n":1}');
  });
});

it('a turn shows the human their own words, not what the harness quoted above them', () => {
  const quoted = `Recalled from memory (a proposed note is unproven):\n@M n1 personal proposed support: x\n\n${THE_ASK}\n\nAdd format(minutes).`;
  expect(humanWords(quoted)).toBe('Add format(minutes).');
  expect(humanWords('Add format(minutes).')).toBe('Add format(minutes).');
});

it("a run's title is the human's words, not the memory quoted before them", () => {
  const ask = `Recalled from memory (a proposed note is unproven):\n- n1 a note\n\n---\n\n${THE_ASK}\n\nmake total skip blanks`;
  const { runs } = parseRuns({
    runs: [
      {
        id: 'r1764237600-42',
        when: '2026-11-27T10:00:00Z',
        state: 'ended',
        why: '',
        ask,
        turns: 1,
      },
    ],
    more: 0,
  });
  expect(runs[0]?.ask).toBe('make total skip blanks');
});

describe('a turn the model ended with an answer', () => {
  it('reads the stop reason as the core writes it, and as a provider says it', () => {
    // The core journals and sends `mikata_provider::StopReason` by name.
    expect(endsTurn('EndTurn')).toBe(true);
    expect(endsTurn('StopSequence')).toBe(true);
    expect(endsTurn('ToolUse')).toBe(false);
    expect(endsTurn('MaxTokens')).toBe(false);
    expect(endsTurn('PauseTurn')).toBe(false);
    // A provider's own word, passed through.
    expect(endsTurn('end_turn')).toBe(true);
    expect(endsTurn('stop')).toBe(true);
    expect(endsTurn('tool_calls')).toBe(false);
  });

  it('marks the view answered until the next call or ask', () => {
    const chat = new Chat('summoner');
    chat.fold(ev('turnStarted', { text: 'go' }));
    chat.fold(ev('assistant', { text: 'Let me look.', stop: 'ToolUse' }));
    expect(chat.view().answered).toBe(false);
    chat.fold(ev('assistant', { text: 'Done.', stop: 'EndTurn' }));
    expect(chat.view().answered).toBe(true);
    chat.fold(ev('turnStarted', { text: 'next' }));
    expect(chat.view().answered).toBe(false);
  });
});
