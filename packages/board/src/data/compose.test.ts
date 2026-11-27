// @vitest-environment jsdom
import { afterEach, expect, test } from 'vitest';
import {
  composition,
  compositionOf,
  NONE,
  noteOpened,
  openedWith,
  sessionOptions,
  setComposition,
  setOnce,
} from './compose.ts';

afterEach(() => window.localStorage.clear());

test('a stored composition is read only when it holds together', () => {
  const good = { profile: 'small-fix', flips: { plan: false, 'plugin:x': true }, units: '-plan' };
  expect(compositionOf(good)).toEqual(good);
  for (const bad of [
    null,
    'small-fix',
    { profile: 3, flips: {}, units: '' },
    { profile: 'a b', flips: {}, units: '' },
    { profile: '', flips: { plan: 'off' }, units: '' },
    { profile: '', flips: { 'a;b': true }, units: '' },
    { profile: '', flips: {}, units: '-plan; rm -rf /' },
  ]) {
    expect(compositionOf(bad), JSON.stringify(bad)).toBeUndefined();
  }
});

test('a new session takes its workspace composition; another workspace keeps its own', () => {
  expect(composition('w1')).toEqual(NONE);
  expect(sessionOptions('w1')).toEqual({});
  setComposition('w1', { profile: 'small-fix', flips: { gate: false }, units: '-gate' });
  expect(sessionOptions('w1')).toEqual({ profile: 'small-fix', units: '-gate' });
  expect(sessionOptions('w2')).toEqual({});
  setComposition('w2', { profile: '', flips: { plan: false }, units: '-plan' });
  expect(sessionOptions('w2')).toEqual({ units: '-plan' });
  // Kept in this browser, per workspace.
  const kept = JSON.parse(window.localStorage.getItem('mikata.compose') ?? '{}');
  expect(Object.keys(kept).sort()).toEqual(['w1', 'w2']);
});

test('a guard switched off holds for the next session only, and is never kept', () => {
  setComposition('w3', { profile: '', flips: { plan: false }, units: '-plan' });
  setOnce('w3', { flips: { secrets: false }, units: '-plan,-secrets' });
  expect(sessionOptions('w3')).toEqual({ units: '-plan,-secrets' });
  // Spent: the session after it opens with the guard on again.
  expect(sessionOptions('w3')).toEqual({ units: '-plan' });
  // And it never reached the browser's storage.
  expect(window.localStorage.getItem('mikata.compose')).not.toContain('secrets');
});

test('what a session was opened with is said back only when it holds together', () => {
  noteOpened('s1', { profile: 'small-fix', units: '-secrets' });
  expect(openedWith('s1')).toEqual({ profile: 'small-fix', units: '-secrets' });
  expect(openedWith('nope')).toBeUndefined();
  window.sessionStorage.setItem('mikata.opened', JSON.stringify({ s2: { units: '-a; rm' } }));
  expect(openedWith('s2')).toBeUndefined();
});

test('the model chosen rides every new session here, and only a name the server takes is kept', () => {
  setComposition('w4', { profile: '', flips: {}, units: '', model: 'openrouter:qwen/qwen3-coder' });
  expect(sessionOptions('w4')).toEqual({ model: 'openrouter:qwen/qwen3-coder' });
  expect(sessionOptions('w4')).toEqual({ model: 'openrouter:qwen/qwen3-coder' });
  expect(compositionOf({ profile: '', flips: {}, units: '', model: 'a model' })).toBeUndefined();
  expect(compositionOf({ profile: '', flips: {}, units: '', model: 'x"; rm' })).toBeUndefined();
  // The guards switched off once are never stored with it.
  setOnce('w4', { flips: { gate: false }, units: '-gate' });
  setComposition('w4', { ...composition('w4'), units: '' });
  const kept = JSON.parse(window.localStorage.getItem('mikata.compose') ?? '{}');
  expect(kept.w4).toEqual({
    profile: '',
    flips: {},
    units: '',
    model: 'openrouter:qwen/qwen3-coder',
  });
  noteOpened('s4', { model: 'claude-haiku-5-5' });
  expect(openedWith('s4')).toEqual({ model: 'claude-haiku-5-5' });
});

test('the limits chosen ride every new session here, and only bounded whole numbers are kept', () => {
  setComposition('w5', {
    profile: '',
    flips: {},
    units: '',
    limits: { steps: 600, minutes: 180, context: 170_000 },
  });
  expect(sessionOptions('w5')).toEqual({ steps: 600, minutes: 180, context: 170_000 });
  for (const limits of [{ steps: 3 }, { minutes: 1.5 }, { context: '9000' }, { budget: 10 }]) {
    expect(
      compositionOf({ profile: '', flips: {}, units: '', limits }),
      JSON.stringify(limits),
    ).toBeUndefined();
  }
  noteOpened('s5', { steps: 600 });
  expect(openedWith('s5')).toEqual({ steps: 600 });
  window.sessionStorage.setItem('mikata.opened', JSON.stringify({ s6: { steps: 2 } }));
  expect(openedWith('s6')).toBeUndefined();
});
