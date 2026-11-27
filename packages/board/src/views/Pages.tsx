/** The board-wide pages, and the page for an address the board does not have. */
import type { BoardMissing } from '@mikata/protocol';
import { Settings } from 'lucide-react';
import type { Load } from '../data/worlds.ts';
import { Safe } from '../safe/Safe.tsx';
import {
  CHOICES,
  type Choice,
  nativeLook,
  PALETTE_CHOICES,
  type PaletteChoice,
  palettes,
} from '../theme.ts';
import { shellWord, since } from '../time.ts';
import { Empty } from '../ui/Empty.tsx';
import { Icon } from '../ui/Icon.tsx';

export function SettingsView({
  theme,
  onTheme,
  palette,
  onPalette,
}: {
  theme: Choice;
  onTheme: (c: Choice) => void;
  palette: PaletteChoice;
  onPalette: (p: PaletteChoice) => void;
}) {
  const mode = document.documentElement.dataset.theme === 'light' ? 'light' : 'dark';
  return (
    <div className="pg">
      <header className="pg-head">
        <span className="pg-icon" aria-hidden="true">
          <Icon icon={Settings} />
        </span>
        <div>
          <p className="ss-eyebrow">
            Board <span>/ settings</span>
          </p>
          <h1>Settings</h1>
          <p className="pg-sub">The board's look, kept for this browser.</p>
        </div>
      </header>
      <section className="setting">
        <h2 className="setting-title" id="setting-theme">
          Theme
        </h2>
        <p className="setting-sub">Auto follows the system's light or dark preference.</p>
        <div className="radio-row" role="radiogroup" aria-labelledby="setting-theme">
          {CHOICES.map((c) => (
            <label key={c} className="radio">
              <input
                type="radio"
                name="theme"
                value={c}
                checked={theme === c}
                onChange={() => onTheme(c)}
              />
              {c}
            </label>
          ))}
        </div>
      </section>
      <section className="setting">
        <h2 className="setting-title" id="setting-palette">
          Palette
        </h2>
        <p className="setting-sub">
          The board's colours, the top bar's gradient with them, kept for this browser. The terminal
          app keeps its own (<code className="mono">/theme</code>).
        </p>
        <div className="palette-grid" role="radiogroup" aria-labelledby="setting-palette">
          {PALETTE_CHOICES.map((id) => {
            const p = id === 'native' ? nativeLook[mode] : palettes[id][mode];
            return (
              <label key={id} className="palette-card" data-checked={palette === id}>
                <input
                  type="radio"
                  name="palette"
                  value={id}
                  checked={palette === id}
                  onChange={() => onPalette(id)}
                />
                <span
                  className="palette-hero"
                  style={{
                    background: `linear-gradient(135deg, ${p.hot}, ${p.coral} 55%, ${p.amber}) ${p.hot}`,
                    color: p.onBrand,
                  }}
                >
                  <b>{id === 'native' ? 'Native' : palettes[id].name}</b>
                  <small>{id === 'native' ? 'Bio Digital · Bio Pearl' : palettes[id].about}</small>
                </span>
                <span className="palette-swatches" aria-hidden="true">
                  {[p.bg, p.panel, p.holo, p.gold].map((c, k) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: four fixed swatches
                    <i key={k} style={{ background: c }} />
                  ))}
                </span>
              </label>
            );
          })}
        </div>
      </section>
      <section className="setting">
        <h2 className="setting-title">Configuration</h2>
        <p className="setting-sub">
          The providers, keys' sources, MCP servers, the board's port and the effective
          configuration with each key's origin are on <a href="#/integrations">Integrations</a>,
          read-only.
        </p>
      </section>
    </div>
  );
}

export function UnknownView({ path }: { path: string }) {
  return (
    <div className="page">
      <Empty glyph="⌀" title="Nothing at this address">
        <p>
          The board has no page at{' '}
          <code className="mono">
            <Safe text={path || '/'} />
          </code>
          .
        </p>
        <p>
          <a href="#/">Back to the board</a>
        </p>
      </Empty>
    </div>
  );
}

/** The front page while there is no workspace to open: reading, unread, or none yet. */
export function HomeView({ load }: { load: Load }) {
  if (load.state === 'loading') {
    return (
      <div className="page">
        <Empty glyph="⣾" title="Reading the workspaces…">
          Asking the board server.
        </Empty>
      </div>
    );
  }
  if (load.state === 'failed') {
    return (
      <div className="page">
        <Empty glyph="✕" title="The workspaces could not be read">
          <p>
            <Safe text={load.reason} />
          </p>
        </Empty>
      </div>
    );
  }
  return (
    <div className="page">
      <Empty glyph="◇" title="No workspace yet">
        <p>
          Run <code className="mono">mikata</code> in a repository: it becomes a workspace here,
          with its sessions, its harness and its runs.
        </p>
      </Empty>
    </div>
  );
}

/** The workspace a route names, or why there is none. */
export function NoWorld({ id }: { id: string }) {
  return (
    <div className="page">
      <Empty glyph="✕" title="No such workspace">
        <p>
          No workspace with the id{' '}
          <code className="mono">
            <Safe text={id} />
          </code>{' '}
          is known to this board.
        </p>
      </Empty>
    </div>
  );
}

/** A workspace whose directory is gone: where it was, since when, how to forget it. */
export function MissingWorld({ m, now }: { m: BoardMissing; now: number }) {
  return (
    <div className="page">
      <Empty glyph="✕" title={<Safe text={`${m.name} is missing`} />}>
        <p>
          Its directory{' '}
          <code className="mono">
            <Safe text={m.path} />
          </code>{' '}
          is gone, or holds no <code className="mono">.mikata/</code>: missing since{' '}
          {since(m.since, now)}. It is live again once it is back.
        </p>
        <p>
          To forget it:{' '}
          <code className="mono">
            <Safe text={`mikatad board forget ${shellWord(m.path)}`} />
          </code>
        </p>
      </Empty>
    </div>
  );
}
