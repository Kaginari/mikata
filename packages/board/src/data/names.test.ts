import { expect, test } from 'vitest';
import { shownBody } from './names';

test("the session's body is shown as mikata, and no other name changes", () => {
  expect(shownBody('summoner')).toBe('mikata');
  expect(shownBody('summoner/01-duckdb')).toBe('mikata/01-duckdb');
  for (const other of ['summoners', 'elf-e', 'orc-sql', 'not-summoner', ''])
    expect(shownBody(other)).toBe(other);
});
