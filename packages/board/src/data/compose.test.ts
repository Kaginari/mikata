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
