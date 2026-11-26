/**
 * A world's desk — what its session is held to: the session card's verify line (or the test
 * command to add when it has none), the body's `Done when` list in its plan, the branch
 * (`GET /api/w/<world>/desk`, `mikata_world::desk`). Read on opening and every few seconds while
 * the app is open: the body writes its plan during a run.
 */
import { useEffect, useState } from 'react';
import { getJson } from './chats.ts';

export interface Desk {
  verify: string[];
  suggested: string | null;
  done: string[];
  branch: string | null;
}

const EMPTY: Desk = { verify: [], suggested: null, done: [], branch: null };

/** How often the desk is read again. */
export const DESK_EVERY = 4000;

const strings = (v: unknown): string[] =>
  Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string') : [];
const text = (v: unknown): string | null => (typeof v === 'string' ? v : null);

/** A desk from the server's JSON; anything malformed reads as nothing. */
export function deskOf(v: unknown): Desk {
  const o = (v ?? {}) as Record<string, unknown>;
  return {
    verify: strings(o.verify),
    suggested: text(o.suggested),
    done: strings(o.done),
    branch: text(o.branch),
  };
}

/** `world`'s desk, kept fresh. */
export function useDesk(world: string): Desk {
  const [desk, setDesk] = useState<Desk>(EMPTY);
  useEffect(() => {
    const ctl = new AbortController();
    const read = () =>
      getJson(`/api/w/${encodeURIComponent(world)}/desk`, ctl.signal)
        .then((d) => setDesk(deskOf(d)))
        .catch(() => {});
    read();
    const t = window.setInterval(read, DESK_EVERY);
    return () => {
      ctl.abort();
      window.clearInterval(t);
    };
  }, [world]);
  return desk;
}
