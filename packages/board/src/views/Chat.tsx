/**
 * A world's chat: a live one over the bridge — the transcript, the activity strip, the asks
 * waiting, the composer — or a past one, read-only from its run journal, with a resume when it
 * stopped. The session's vitals, files and bodies go to the side pane.
 */
import type { BoardWorld, Hello } from '@mikata/protocol';
import { useCallback, useEffect, useRef, useState } from 'react';
import { Chat, type ChatView, type Entry } from '../data/chat.ts';
import { type Chats, startChat, useRun } from '../data/chats.ts';
import type { Live } from '../data/live.ts';
import { shownBody } from '../data/names';
import { format } from '../router.ts';
import { Safe } from '../safe/Safe.tsx';
import { announce } from '../ui/Announcer.tsx';
import { Empty } from '../ui/Empty.tsx';
import { Mark } from '../ui/Mark.tsx';
import { EntryView, opensAnswer, Streaming, useOpen } from '../ui/Transcript.tsx';
import { Virtual } from '../ui/Virtual.tsx';

/** What a chat tells the side pane. */
export interface Side {
  world: string;
  view: ChatView | undefined;
  hello: Hello | undefined;
  /** A live chat, rather than a past one. */
  live: boolean;
}

const keyOf = (e: Entry) => e.key;

/** The transcript, virtualized, the text streaming and the live line after it. */
export function TranscriptView({
  view,
  me,
  tail,
}: {
  view: ChatView;
  me: string;
  tail?: React.ReactNode;
}) {
  const [open, toggle] = useOpen();
  const render = useCallback(
    (e: Entry) => <EntryView e={e} me={me} open={open.has(e.key)} onToggle={toggle} />,
    [me, open, toggle],
  );
  return (
    <Virtual
      items={view.entries}
      keyOf={keyOf}
      render={render}
      label="Transcript"
      tail={
        <>
          <Streaming text={view.streaming} me={me} head={opensAnswer(view.entries)} />
          {tail}
        </>
      }
    />
  );
}

/**
 * What a screen reader is told of a live chat, once each: an answer when a turn ends, an ask when
 * one arrives at the gate. What was already there when the chat opened (a replay) is not said.
 */
export function useAnnouncements(view: ChatView | undefined, asks: Live['asks']) {
  const heard = useRef<{ answers: number; asks: Set<string> } | undefined>(undefined);
  const busy = view?.busy ?? false;
  useEffect(() => {
    if (!view) return;
    const answers = view.entries.filter((e) => e.kind === 'assistant');
    if (heard.current === undefined) {
      heard.current = { answers: answers.length, asks: new Set(asks.map((a) => a.id)) };
      return;
    }
    if (!busy && answers.length > heard.current.answers) {
      const fresh = answers
        .slice(heard.current.answers)
        .map((e) => (e.kind === 'assistant' ? e.text : ''))
        .join(' ');
      announce(`Answered: ${fresh}`);
    }
    if (!busy) heard.current.answers = answers.length;
  }, [view, busy, asks]);
  useEffect(() => {
    const h = heard.current;
    if (!h) return;
    for (const a of asks) {
      if (h.asks.has(a.id)) continue;
      h.asks.add(a.id);
      announce(
        `An ask waits at the gate: ${a.ask.class} by ${shownBody(a.ask.body)}. The approval card is open.`,
        true,
      );
    }
  }, [asks]);
}

/** A past chat, from its run journal: read-only, with a resume when it stopped. */
export function PastChat({
  w,
  run,
  chats,
  onSide,
}: {
  w: BoardWorld;
  run: string;
  chats: Chats;
  onSide: (s: Side | undefined) => void;
}) {
  const load = useRun(w.id, run);
  const [view, setView] = useState<ChatView | undefined>(undefined);
  const [why, setWhy] = useState<string | undefined>(undefined);
  const [starting, setStarting] = useState(false);
  useEffect(() => {
    if (load.state !== 'ready') {
      setView(undefined);
      return;
    }
    const chat = new Chat('');
    for (const e of load.data.arrived) chat.fold(e);
    setView(chat.view());
  }, [load]);
  useEffect(() => {
    onSide({ world: w.id, view, hello: undefined, live: false });
  }, [onSide, w.id, view]);
  useEffect(() => () => onSide(undefined), [onSide]);
  const listed = chats.runs.find((r) => r.id === run);
  const state = load.state === 'ready' ? load.data.state : listed?.state;
  const liveChat = chats.live.find((s) => s.run === run);
  const resume = async () => {
    setStarting(true);
    setWhy(undefined);
    try {
      const started = await startChat(w.id, run);
      chats.refresh();
      window.location.hash = format({ name: 'session', world: w.id, session: started.session.id });
    } catch (e) {
      setWhy(e instanceof Error ? e.message : String(e));
    } finally {
      setStarting(false);
    }
  };
  return (
    <div className="page page-chat">
      <header className="chat-head">
        <h1 className="chat-title">
          <span className="mono">
            <Safe text={run} />
          </span>
        </h1>
        <Mark
          tone={state === 'stopped' ? 'gold' : state === 'live' ? 'holo' : 'muted'}
          glyph={state === 'live' ? '●' : '○'}
        >
          {state === 'live'
            ? 'live elsewhere'
            : state === 'stopped'
              ? `stopped · ${listed?.why ?? ''}`.replace(/ · $/, '')
              : state === 'ended'
                ? `ended · ${listed?.why ?? ''}`.replace(/ · $/, '')
                : 'past'}
        </Mark>
        {liveChat ? (
          <a
            className="button"
            href={format({ name: 'session', world: w.id, session: liveChat.id })}
          >
            Open the live chat
          </a>
        ) : (
          state === 'stopped' && (
            <button
              type="button"
              className="button button-primary"
              disabled={starting}
              onClick={() => void resume()}
            >
              {starting ? 'Resuming…' : 'Resume'}
            </button>
          )
        )}
      </header>
      <div className="chat-body">
        {load.state === 'loading' && (
          <div className="chat-empty">
            <Empty glyph="◌" title="Loading the run's journal…" />
          </div>
        )}
        {load.state === 'failed' && (
          <div className="chat-empty">
            <Empty glyph="✕" title="This run's journal could not be read">
              <Safe text={load.reason} />
            </Empty>
          </div>
        )}
        {view && <TranscriptView view={view} me="" />}
      </div>
      <div className="chat-foot">
        {load.state === 'ready' && (load.data.dropped > 0 || load.data.skipped.length > 0) && (
          <p className="hint">
            {load.data.dropped > 0 && `${load.data.dropped} older events not shown. `}
            {load.data.skipped.length > 0 &&
              `${load.data.skipped.length} journal line(s) did not read and are left out.`}
          </p>
        )}
        {why && (
          <p className="refused" role="alert">
            <Mark tone="alarm" glyph="✕">
              <Safe text={why} />
            </Mark>
          </p>
        )}
        <div className="composer composer-closed" aria-disabled="true">
          <span className="composer-label">read-only</span>
          <span className="composer-hint">
            A past chat, from its run journal{state === 'stopped' ? ': resume it to go on.' : '.'}
          </span>
        </div>
      </div>
    </div>
  );
}
