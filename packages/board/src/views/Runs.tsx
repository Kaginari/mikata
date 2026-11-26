/**
 * The workspace's runs (`#/w/<world>/runs`): the live sessions first, then the past runs, each a
 * card that opens it; the journal and the metrics a link away.
 */
import type { BoardWorld } from '@mikata/protocol';
import {
  Activity,
  ChartColumnIncreasing,
  CirclePlay,
  History,
  MessagesSquare,
  ScrollText,
} from 'lucide-react';
import type { CSSProperties } from 'react';
import type { Chats } from '../data/chats.ts';
import { format } from '../router.ts';
import { Safe } from '../safe/Safe.tsx';
import { ago } from '../time.ts';
import { Icon } from '../ui/Icon.tsx';

const STATE_WORD: Record<string, string> = {
  live: 'live',
  stopped: 'stopped',
  ended: 'ended',
  unknown: 'unknown',
};

export function RunsView({ w, chats, now }: { w: BoardWorld; chats: Chats; now: number }) {
  const ended = chats.runs.filter((r) => r.state === 'ended').length;
  const turns = chats.runs.reduce((n, r) => n + r.turns, 0);
  // A live session's run is listed once, as the live session.
  const liveRuns = new Set(chats.live.map((c) => c.run));
  const past = chats.runs.filter((r) => !liveRuns.has(r.id));
  const askOf = (run: string) => chats.runs.find((r) => r.id === run)?.ask;
  return (
    <div className="pg">
      <header className="pg-head">
        <span className="pg-icon" aria-hidden="true">
          <Icon icon={Activity} />
        </span>
        <div>
          <p className="ss-eyebrow">
            Runs{' '}
            <span>
              / <Safe text={w.name} />
            </span>
          </p>
          <h1>Agent runs</h1>
          <p className="pg-sub">The live sessions of this workspace, then its past runs.</p>
        </div>
        <nav className="pg-links" aria-label="More about the runs">
          <a href={format({ name: 'section', world: w.id, section: 'journal' })}>
            <Icon icon={ScrollText} /> Run journal
          </a>
          <a href={format({ name: 'metrics' })}>
            <Icon icon={ChartColumnIncreasing} /> Metrics
          </a>
        </nav>
      </header>

      <dl className="ss-tiles pg-tiles">
        <div>
          <dt>
            <Icon icon={CirclePlay} /> Live now
          </dt>
          <dd>{chats.live.length}</dd>
        </div>
        <div>
          <dt>
            <Icon icon={History} /> Past runs
          </dt>
          <dd>
            {chats.runs.length}
            {chats.more > 0 && <small>+{chats.more}</small>}
          </dd>
        </div>
        <div>
          <dt>
            <Icon icon={MessagesSquare} /> Turns
          </dt>
          <dd>{turns}</dd>
        </div>
        <div>
          <dt>
            <Icon icon={Activity} /> Ended cleanly
          </dt>
          <dd>
            {chats.runs.length > 0 ? `${Math.round((ended / chats.runs.length) * 100)}%` : '—'}
          </dd>
        </div>
      </dl>

      {chats.failed && (
        <p className="mapp-why" role="alert">
          <Safe text={chats.failed} />
        </p>
      )}

      <h2 className="pg-section">Live</h2>
      {chats.live.length === 0 ? (
        <p className="ss-muted">No session runs now: start one with New session.</p>
      ) : (
        <ul className="pg-grid">
          {chats.live.map((c, i) => (
            <li key={c.id} style={{ '--i': i } as CSSProperties}>
              <a
                className="pg-run"
                data-state="live"
                href={format({ name: 'session', world: w.id, session: c.id })}
              >
                <span className="pg-runstate">
                  <i className="pf-dot" aria-hidden="true" /> live
                </span>
                <b>
                  <Safe text={askOf(c.run) || `Session ${c.run.slice(-6)}`} />
                </b>
                <small>
                  {c.asks} ask{c.asks === 1 ? '' : 's'} · {c.tabs} tab{c.tabs === 1 ? '' : 's'}
                  {c.controlled ? ' · controlled' : ''}
                </small>
              </a>
            </li>
          ))}
        </ul>
      )}

      <h2 className="pg-section">Past</h2>
      {past.length === 0 ? (
        <p className="ss-muted">No past run yet.</p>
      ) : (
        <ul className="pg-grid">
          {past.map((r, i) => (
            <li key={r.id} style={{ '--i': chats.live.length + Math.min(i, 16) } as CSSProperties}>
              <a
                className="pg-run"
                data-state={r.state}
                href={format({ name: 'run', world: w.id, run: r.id })}
              >
                <span className="pg-runstate">
                  <i className="pf-dot" data-state={r.state} aria-hidden="true" />{' '}
                  {STATE_WORD[r.state] ?? r.state}
                </span>
                <b>
                  <Safe text={r.ask || r.id} />
                </b>
                <small>
                  {ago(r.when, now)} · {r.turns} turn{r.turns === 1 ? '' : 's'}
                  {r.why && (
                    <>
                      {' '}
                      · <Safe text={r.why} />
                    </>
                  )}
                </small>
              </a>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
