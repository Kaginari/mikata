/**
 * The board's frame, one for every page: the glowing top bar (Mikata's mark, the workspace's
 * pages as tabs, search, the theme) over the rail (the workspace, its pages, its
 * sessions, the runtime's state) and the page itself. At phone width the rail is a drawer.
 */
import type { BoardWorld } from '@mikata/protocol';
import {
  Activity,
  ArrowRight,
  ChartColumnIncreasing,
  Cpu,
  Database,
  History,
  Layers,
  type LucideIcon,
  Menu,
  MessageSquare,
  MessagesSquare,
  Moon,
  Plug,
  Puzzle,
  ScrollText,
  Search,
  Settings,
  Sun,
  SunMoon,
  X,
} from 'lucide-react';
import { type ReactNode, useEffect, useRef, useState } from 'react';
import type { Chats } from '../data/chats.ts';
import { useComposition } from '../data/compose.ts';
import { opening, useHarness } from '../data/harness.ts';
import { format, type Route } from '../router.ts';
import { Safe } from '../safe/Safe.tsx';
import { shown } from '../safe/shown.ts';
import type { Choice } from '../theme.ts';
import { Icon } from './Icon.tsx';
import { Logo } from './Logo.tsx';

/** Which tab and rail entry a route lights. */
export type Place =
  | 'sessions'
  | 'builder'
  | 'plugins'
  | 'runs'
  | 'journal'
  | 'memory'
  | 'metrics'
  | 'integrations'
  | 'settings'
  | 'none';

export function placeOf(route: Route): Place {
  switch (route.name) {
    case 'world':
    case 'session':
      return 'sessions';
    case 'run':
      return 'runs';
    case 'section':
      return route.section;
    case 'metrics':
    case 'integrations':
    case 'settings':
      return route.name;
    default:
      return 'none';
  }
}

const TABS: [Place, LucideIcon, string][] = [
  ['sessions', MessagesSquare, 'Sessions'],
  ['builder', Layers, 'Harness Builder'],
  ['plugins', Puzzle, 'Plugins'],
  ['runs', Activity, 'Runs'],
];

const RAIL: [Place, LucideIcon, string][] = [
  ['sessions', MessagesSquare, 'Agent sessions'],
  ['builder', Layers, 'Harness Builder'],
  ['plugins', Puzzle, 'Plugins'],
  ['runs', Activity, 'Runs'],
  ['journal', ScrollText, 'Run journal'],
  ['memory', Database, 'Memory'],
];

const BOARD: [Place, LucideIcon, string][] = [
  ['metrics', ChartColumnIncreasing, 'Metrics'],
  ['integrations', Plug, 'Integrations'],
  ['settings', Settings, 'Settings'],
];

/** The address of `place` in `world`. */
function href(place: Place, world: string | undefined): string {
  switch (place) {
    case 'metrics':
    case 'integrations':
    case 'settings':
      return format({ name: place });
    case 'sessions':
      return world ? format({ name: 'world', world }) : '#/';
    case 'none':
      return '#/';
    default:
      return world ? format({ name: 'section', world, section: place }) : '#/';
  }
}

/** The width at which the rail is a drawer (platform.css's phone block). */
const PHONE = '(max-width: 860px)';

/** Whether the page is at phone width, following it. */
function usePhone(): boolean {
  const [phone, setPhone] = useState(() => window.matchMedia(PHONE).matches);
  useEffect(() => {
    const mq = window.matchMedia(PHONE);
    const on = () => setPhone(mq.matches);
    mq.addEventListener('change', on);
    return () => mq.removeEventListener('change', on);
  }, []);
  return phone;
}

const tilde = (path: string) => path.replace(/^\/(?:home|Users)\/[^/]+/, '~');

const NEXT: Record<Choice, Choice> = { auto: 'light', light: 'dark', dark: 'auto' };
const THEME_ICON: Record<Choice, LucideIcon> = { auto: SunMoon, light: Sun, dark: Moon };

/** The runtime's state for the workspace: the harness, and what a new session opens with — the
 * Builder's composition (its profile, the units it leaves on) — and how many sessions run. */
function Status({ world, chats }: { world: BoardWorld; chats: Chats }) {
  const load = useHarness(world.id);
  const c = useComposition(world.id);
  const data = load.state === 'ready' ? load.data : undefined;
  const base = data
    ? opening(
        data.units,
        data.profiles.find((p) => p.name === c.profile),
      )
    : null;
  const on =
    data && base
      ? data.units.filter((u) => c.once?.flips[u.id] ?? c.flips[u.id] ?? base.get(u.id)).length
      : 0;
  const ok = data !== undefined && data.holes.length === 0;
  return (
    <ul className="pf-status" aria-label="Runtime status">
      <li data-status={ok ? 'ok' : load.state === 'loading' ? 'wait' : 'alarm'}>
        <i className="pf-dot" aria-hidden="true" />
        <span>Harness</span>
        <b>{ok ? 'ready' : load.state === 'loading' ? 'reading' : 'holes'}</b>
      </li>
      <li>
        <Icon icon={Layers} />
        <span>Profile</span>
        <b>
          <Safe text={c.profile || 'default'} />
        </b>
      </li>
      <li>
        <Icon icon={Cpu} />
        <span>Units on</span>
        <b>{data ? `${on} / ${data.units.length}` : '—'}</b>
      </li>
      <li data-status={chats.live.length > 0 ? 'ok' : undefined}>
        <Icon icon={Activity} />
        <span>Live sessions</span>
        <b>{chats.live.length}</b>
      </li>
    </ul>
  );
}

/** The workspace's sessions: the live ones, each by its ask, marked when it waits for you; then
 * the latest past runs, and the way to every run. */
function Sessions({ world, chats, route }: { world: BoardWorld; chats: Chats; route: Route }) {
  // A live session's run is listed once, as the live session.
  const liveRuns = new Set(chats.live.map((c) => c.run));
  const past = chats.runs.filter((r) => !liveRuns.has(r.id));
  const askOf = (run: string) => chats.runs.find((r) => r.id === run)?.ask;
  const here = route.name === 'session' ? route.session : undefined;
  const run = route.name === 'run' ? route.run : undefined;
  return (
    <section className="pf-card pf-sessions" aria-label="Sessions">
      <p className="pf-label">Sessions</p>
      {chats.live.length === 0 ? (
        <p className="pf-muted">No session runs.</p>
      ) : (
        <ul className="pf-list" aria-label="Live sessions">
          {chats.live.map((c, i) => (
            <li key={c.id} style={{ '--i': i } as React.CSSProperties}>
              <a
                className="pf-session"
                data-live="true"
                href={format({ name: 'session', world: world.id, session: c.id })}
                aria-current={here === c.id ? 'page' : undefined}
              >
                <Icon icon={MessageSquare} />
                <span>
                  <Safe text={askOf(c.run) || 'Untitled session'} />
                </span>
                {c.asks > 0 ? (
                  <small className="pf-needs" title="An approval waits for you">
                    needs you
                  </small>
                ) : (
                  <i className="pf-dot" aria-hidden="true" />
                )}
              </a>
            </li>
          ))}
        </ul>
      )}
      {past.length > 0 && (
        <>
          <p className="pf-sublabel">Recent runs</p>
          <ul className="pf-list" aria-label="Recent runs">
            {past.slice(0, 4).map((r, i) => (
              <li key={r.id} style={{ '--i': chats.live.length + i } as React.CSSProperties}>
                <a
                  className="pf-session pf-past"
                  href={format({ name: 'run', world: world.id, run: r.id })}
                  aria-current={run === r.id ? 'page' : undefined}
                >
                  <Icon icon={History} />
                  <span>
                    <Safe text={r.ask || r.id} />
                  </span>
                </a>
              </li>
            ))}
          </ul>
          <a
            className="pf-all"
            href={format({ name: 'section', world: world.id, section: 'runs' })}
          >
            All runs ({past.length + chats.more}) <Icon icon={ArrowRight} />
          </a>
        </>
      )}
    </section>
  );
}

export function Platform({
  route,
  worlds,
  world,
  chats,
  theme,
  onTheme,
  onSearch,
  children,
}: {
  route: Route;
  worlds: readonly BoardWorld[];
  world: BoardWorld | undefined;
  chats: Chats;
  theme: Choice;
  onTheme: (c: Choice) => void;
  onSearch: () => void;
  children: ReactNode;
}) {
  const place = placeOf(route);
  const [drawer, setDrawer] = useState(false);
  // biome-ignore lint/correctness/useExhaustiveDependencies: the drawer closes when the page moves
  useEffect(() => setDrawer(false), [route]);
  const phone = usePhone();
  const menuRef = useRef<HTMLButtonElement>(null);
  const railRef = useRef<HTMLElement>(null);
  // The drawer's focus: into it as it opens, back to the menu as it closes; Esc closes it.
  const was = useRef(drawer);
  useEffect(() => {
    if (phone && drawer && !was.current) {
      railRef.current
        ?.querySelector<HTMLElement>('a[href], button:not([disabled]), select')
        ?.focus();
    } else if (phone && !drawer && was.current) {
      menuRef.current?.focus();
    }
    was.current = drawer;
  }, [drawer, phone]);
  useEffect(() => {
    if (!drawer) return;
    const on = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !e.defaultPrevented) setDrawer(false);
    };
    window.addEventListener('keydown', on);
    return () => window.removeEventListener('keydown', on);
  }, [drawer]);
  const session = route.name === 'session' ? route.session : undefined;

  return (
    <div className="pf" data-drawer={drawer ? 'open' : 'closed'}>
      <header className="pf-top">
        <button
          type="button"
          ref={menuRef}
          className="pf-icon pf-menu"
          aria-label={drawer ? 'Close the menu' : 'Menu'}
          aria-expanded={drawer}
          aria-controls="pf-rail"
          onClick={() => setDrawer((d) => !d)}
        >
          <Icon icon={drawer ? X : Menu} />
        </button>
        <a className="pf-brand" href={href('sessions', world?.id)}>
          <Logo className="pf-logo" />
          <span className="pf-name">
            mikata <span lang="ja">味方</span>
          </span>
          <span className="pf-tagline">Composable Coding Agent Harness</span>
        </a>
        {world && (
          <nav className="pf-tabs" aria-label="The workspace's pages">
            {TABS.map(([p, icon, label]) => (
              <a
                key={p}
                href={href(p, world.id)}
                className="pf-tab"
                aria-current={place === p ? 'page' : undefined}
              >
                <Icon icon={icon} />
                <span>{label}</span>
              </a>
            ))}
          </nav>
        )}
        <div className="pf-tools">
          <button
            type="button"
            className="pf-icon"
            aria-label="Search: jump to a workspace, a page, a session or a run"
            aria-keyshortcuts="Control+K Meta+K"
            onClick={onSearch}
          >
            <Icon icon={Search} />
          </button>
          <button
            type="button"
            className="pf-icon"
            aria-label={`Theme: ${theme}; switch to ${NEXT[theme]}`}
            onClick={() => onTheme(NEXT[theme])}
          >
            <Icon icon={THEME_ICON[theme]} />
          </button>
        </div>
      </header>

      <div className="pf-body">
        <aside
          id="pf-rail"
          ref={railRef}
          className="pf-rail"
          aria-label="Workspace"
          inert={phone && !drawer}
        >
          <section className="pf-card">
            <p className="pf-label">Workspace</p>
            {worlds.length > 0 ? (
              <>
                <select
                  className="pf-select"
                  aria-label="Workspace"
                  value={world?.id ?? ''}
                  onChange={(e) => {
                    window.location.hash = format({ name: 'world', world: e.target.value });
                  }}
                >
                  {!world && <option value="">Choose a workspace</option>}
                  {worlds.map((w) => (
                    <option key={w.id} value={w.id}>
                      {shown(w.name)}
                    </option>
                  ))}
                </select>
                {world && (
                  <p className="pf-path">
                    <Safe text={tilde(world.path)} />
                  </p>
                )}
              </>
            ) : (
              <p className="pf-muted">No workspace yet</p>
            )}
            <nav aria-label="Pages" className="pf-nav">
              {world &&
                RAIL.map(([p, icon, label]) => (
                  <a
                    key={p}
                    href={href(p, world.id)}
                    className="pf-navlink"
                    aria-current={place === p ? 'page' : undefined}
                  >
                    <Icon icon={icon} />
                    <span>{label}</span>
                  </a>
                ))}
            </nav>
          </section>

          {world && <Sessions world={world} chats={chats} route={route} />}

          {world && (
            <section className="pf-card" aria-label="Runtime">
              <p className="pf-label">Runtime status</p>
              <Status world={world} chats={chats} />
            </section>
          )}

          <section className="pf-card">
            <p className="pf-label">Board</p>
            <nav aria-label="The board's pages" className="pf-nav">
              {BOARD.map(([p, icon, label]) => (
                <a
                  key={p}
                  href={href(p, world?.id)}
                  className="pf-navlink"
                  aria-current={place === p ? 'page' : undefined}
                >
                  <Icon icon={icon} />
                  <span>{label}</span>
                </a>
              ))}
            </nav>
          </section>
          <p className="pf-foot">
            MIKATA <span>/ web board</span>
          </p>
        </aside>
        <button
          type="button"
          className="pf-scrim"
          aria-label="Close the menu"
          tabIndex={-1}
          onClick={() => setDrawer(false)}
        />
        <div className="pf-page" key={place + (session ?? '')}>
          {children}
        </div>
      </div>
    </div>
  );
}
