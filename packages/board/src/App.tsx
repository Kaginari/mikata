/**
 * The board: one platform for every workspace — the top bar, the rail, and the page: a session,
 * the Harness Builder, the plugins, the runs, the journal, the memory, the board's own pages.
 */
import type { BoardWorld } from '@mikata/protocol';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { type Chats, useChats } from './data/chats.ts';
import { commands } from './data/commands.ts';
import { useAccess } from './data/session.ts';
import { type Load, useWorlds } from './data/worlds.ts';
import { format, type Route, useRoute, worldOf } from './router.ts';
import { shown } from './safe/shown.ts';
import { type Choice, type SchemeName, useScheme, useTheme } from './theme.ts';
import { Announcer } from './ui/Announcer.tsx';
import { CommandPalette } from './ui/CommandPalette.tsx';
import { cardUp, dialogUp, typing } from './ui/modal.ts';
import { Platform } from './ui/Platform.tsx';
import { ShortcutMap } from './ui/ShortcutMap.tsx';
import { BuilderView } from './views/Builder.tsx';
import { PastChat, type Side } from './views/Chat.tsx';
import { IntegrationsView } from './views/Integrations.tsx';
import { JournalView } from './views/Journal.tsx';
import { MemoryView } from './views/Memory.tsx';
import { MetricsView } from './views/Metrics.tsx';
import { HomeView, MissingWorld, NoWorld, SettingsView, UnknownView } from './views/Pages.tsx';
import { RunsView } from './views/Runs.tsx';
import { SessionView } from './views/Session.tsx';
import { SignedOut } from './views/SignedOut.tsx';
import './platform.css';

function useNow(): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const t = window.setInterval(() => setNow(Date.now()), 30_000);
    return () => window.clearInterval(t);
  }, []);
  return now;
}

const noSide = (_: Side | undefined) => {};

function Page({
  route,
  load,
  world,
  now,
  theme,
  onTheme,
  scheme,
  onScheme,
  chats,
}: {
  route: Route;
  load: Load;
  world: BoardWorld | undefined;
  now: number;
  theme: Choice;
  onTheme: (c: Choice) => void;
  scheme: SchemeName;
  onScheme: (s: SchemeName) => void;
  chats: Chats;
}) {
  switch (route.name) {
    case 'worlds':
      return <HomeView load={load} />;
    case 'metrics':
      return <MetricsView load={load} />;
    case 'integrations':
      return <IntegrationsView theme={theme} />;
    case 'settings':
      return <SettingsView theme={theme} onTheme={onTheme} scheme={scheme} onScheme={onScheme} />;
    case 'unknown':
      return <UnknownView path={route.path} />;
  }
  if (load.state !== 'ready') return <HomeView load={load} />;
  if (!world) {
    const gone = load.data.missing.find((m) => m.id === route.world);
    return gone ? <MissingWorld m={gone} now={now} /> : <NoWorld id={route.world} />;
  }
  switch (route.name) {
    case 'world':
      return <SessionView key={world.id} w={world} chats={chats} />;
    case 'session':
      return <SessionView key={route.session} w={world} chats={chats} session={route.session} />;
    case 'run':
      return (
        <div className="pg pg-chat">
          <PastChat key={route.run} w={world} run={route.run} chats={chats} onSide={noSide} />
        </div>
      );
  }
  switch (route.section) {
    case 'builder':
    case 'plugins':
    case 'profiles':
      return <BuilderView key={world.id} w={world} chats={chats} view={route.section} />;
    case 'runs':
      return <RunsView w={world} chats={chats} now={now} />;
    case 'journal':
      return (
        <div className="pg">
          <JournalView key={world.id} w={world} />
        </div>
      );
    case 'memory':
      return (
        <div className="pg">
          <MemoryView key={world.id} w={world} />
        </div>
      );
  }
}

const SECTION_TITLE: Record<string, string> = {
  builder: 'Harness Builder',
  plugins: 'Plugins',
  profiles: 'Profiles',
  runs: 'Runs',
  journal: 'Run journal',
  memory: 'Memory',
};

/** The page's title, for the tab. */
function title(route: Route, world: BoardWorld | undefined): string {
  switch (route.name) {
    case 'worlds':
      return 'Board';
    case 'metrics':
      return 'Metrics';
    case 'integrations':
      return 'Integrations';
    case 'settings':
      return 'Settings';
    case 'unknown':
      return 'Not found';
    case 'section':
      return `${SECTION_TITLE[route.section] ?? route.section} · ${world?.name ?? route.world}`;
    default:
      return world?.name ?? route.world;
  }
}

/** Which of the board's own dialogs is open: the palette, the shortcut map, or neither. */
type Open = 'palette' | 'shortcuts' | undefined;

/**
 * The board's shortcuts: ⌘K or Ctrl+K the palette, `?` the map. Neither opens while an approval
 * card (or any other dialog) holds the page, and no key here answers anything.
 */
function useShortcuts(setOpen: (o: Open | ((was: Open) => Open)) => void) {
  useEffect(() => {
    const on = (e: KeyboardEvent) => {
      if (e.defaultPrevented || e.repeat || cardUp()) return;
      const palette = (e.key === 'k' || e.key === 'K') && (e.metaKey || e.ctrlKey) && !e.altKey;
      if (palette) {
        e.preventDefault();
        setOpen((was) => (was === 'palette' ? undefined : dialogUp() ? was : 'palette'));
        return;
      }
      if (e.key === '?' && !e.metaKey && !e.ctrlKey && !e.altKey && !typing(e.target)) {
        if (dialogUp()) return;
        e.preventDefault();
        setOpen('shortcuts');
      }
    };
    window.addEventListener('keydown', on);
    return () => window.removeEventListener('keydown', on);
  }, [setOpen]);
}

/** The board, or — in a tab without a session — the page that says so, never a blank root. */
export function App() {
  const access = useAccess();
  return access === 'open' ? <Shell /> : <SignedOut why={access} />;
}

function Shell() {
  const route = useRoute();
  const load = useWorlds();
  const now = useNow();
  const [theme, setTheme] = useTheme();
  const [scheme, setScheme] = useScheme();

  // The front page opens the workspace worked in last.
  useEffect(() => {
    if (route.name !== 'worlds' || load.state !== 'ready') return;
    const newest = [...load.data.worlds].sort((a, b) =>
      (b.last ?? '').localeCompare(a.last ?? ''),
    )[0];
    if (newest) window.location.replace(format({ name: 'world', world: newest.id }));
  }, [route, load]);

  const here = worldOf(route);
  const worlds = load.state === 'ready' ? load.data.worlds : [];
  const world = here !== undefined ? worlds.find((w) => w.id === here) : undefined;
  const name = title(route, world);
  const chats = useChats(here);
  const [open, setOpen] = useState<Open>(undefined);
  const closeDialog = useCallback(() => setOpen(undefined), []);
  useShortcuts(setOpen);
  const all = useMemo(
    () => commands(worlds, here, chats.live, chats.runs),
    [worlds, here, chats.live, chats.runs],
  );
  const mainRef = useRef<HTMLElement>(null);

  useEffect(() => {
    // The name is a world's own text, or a decoded address: escaped, never a live bidi mark.
    document.title = `${shown(name)} · Mikata`;
  }, [name]);

  return (
    <div className="shell" data-load={load.state}>
      <button
        type="button"
        className="skip"
        onClick={() => {
          mainRef.current?.focus();
        }}
      >
        Skip to the content
      </button>
      <Platform
        route={route}
        worlds={worlds}
        world={world}
        chats={chats}
        theme={theme}
        onTheme={setTheme}
        onSearch={() => {
          if (!cardUp()) setOpen('palette');
        }}
      >
        <main
          ref={mainRef}
          id="main"
          // biome-ignore lint/a11y/noNoninteractiveTabindex: the page scrolls; the keys scroll it
          tabIndex={0}
          className="pf-main"
        >
          <Page
            route={route}
            load={load}
            world={world}
            now={now}
            theme={theme}
            onTheme={setTheme}
            scheme={scheme}
            onScheme={setScheme}
            chats={chats}
          />
        </main>
      </Platform>
      {open === 'palette' && <CommandPalette all={all} onClose={closeDialog} />}
      {open === 'shortcuts' && <ShortcutMap onClose={closeDialog} />}
      {createPortal(<Announcer />, document.body)}
    </div>
  );
}
