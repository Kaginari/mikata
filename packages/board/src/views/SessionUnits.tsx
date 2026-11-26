/**
 * A running session's units, from its Configure: every unit the workspace's harness has, each with
 * a switch showing whether it is on in this session now (`units/list`), switched at its next safe
 * point (`units/change`) as the terminal app's `/unit` does — and kept for the project as that
 * is, but for a guard switched off: that holds for this session only, as the Builder's does. A dialog: the focus held inside it, Esc or its button closes it. Only the tab that controls
 * the session switches; another reads.
 */
import type { BoardWorld } from '@mikata/protocol';
import type { UnitsListed } from '@mikata/sdk/browser';
import { Save, ShieldAlert, SlidersHorizontal, X } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { GUARDS } from '../data/compose.ts';
import { useHarness } from '../data/harness.ts';
import type { Live } from '../data/live.ts';
import { Safe } from '../safe/Safe.tsx';
import { Icon } from '../ui/Icon.tsx';
import { cardUp, onCard, trapTab } from '../ui/modal.ts';
import { described, rank } from './Builder.tsx';

/** How often the list is read again while the dialog is open: a change lands at a safe point. */
const EVERY = 1500;

export function SessionUnits({
  w,
  live,
  onClose,
}: {
  w: BoardWorld;
  live: Live;
  onClose: () => void;
}) {
  const harness = useHarness(w.id);
  const [listed, setListed] = useState<UnitsListed | undefined>();
  const [why, setWhy] = useState<string | undefined>();
  const [asked, setAsked] = useState<Map<string, boolean>>(new Map());
  // The switches flipped and not yet saved: nothing reaches the session before Save.
  const [draft, setDraft] = useState<Map<string, boolean>>(new Map());
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState<string | undefined>();
  const box = useRef<HTMLDivElement>(null);
  const close = useRef<HTMLButtonElement>(null);
  const controls = live.role === 'controller' && live.link !== 'closed';

  const read = useCallback(async () => {
    try {
      const l = await live.units();
      setListed(l);
      // A change the run has taken is no longer waiting.
      setAsked((m) => {
        const next = new Map(m);
        for (const [id, on] of m) {
          const now = l.units.some((u) => u.id === id && u.state === 'active');
          if (now === on) next.delete(id);
        }
        return next;
      });
    } catch (e) {
      setWhy(e instanceof Error ? e.message : String(e));
    }
  }, [live]);

  useEffect(() => {
    void read();
    const t = window.setInterval(() => void read(), EVERY);
    return () => window.clearInterval(t);
  }, [read]);
  useEffect(() => {
    const was = document.activeElement as HTMLElement | null;
    close.current?.focus();
    return () => {
      if (!cardUp()) was?.focus?.();
    };
  }, []);
  useEffect(() => onCard(onClose), [onClose]);

  const onKey = (e: React.KeyboardEvent) => {
    if (trapTab(e, box.current)) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      onClose();
    }
  };

  const units = [...(harness.state === 'ready' ? harness.data.units : [])].sort(
    (a, b) => rank(a.id) - rank(b.id),
  );
  /** On in the session now, or as asked and not yet taken. */
  const sessionOn = (id: string) =>
    asked.get(id) ?? listed?.units.some((u) => u.id === id && u.state === 'active') ?? false;
  /** As the switch shows it: the draft over the session. */
  const isOn = (id: string) => draft.get(id) ?? sessionOn(id);
  const failed = (id: string) =>
    listed?.units.some((u) => u.id === id && u.state === 'failed') ?? false;
  const flip = (id: string) => {
    setSaved(undefined);
    setDraft((m) => {
      const next = new Map(m);
      const to = !isOn(id);
      if (to === sessionOn(id)) next.delete(id);
      else next.set(id, to);
      return next;
    });
  };
  const save = async () => {
    if (draft.size === 0 || saving) return;
    setSaving(true);
    setWhy(undefined);
    let done = 0;
    try {
      for (const [id, to] of draft) {
        // A guard switched off holds for this session only, as the Harness Builder's does: never
        // kept, so the next session opens with it. Any other switch is kept, as `/unit` keeps it.
        await live.switchUnit(id, to, !(GUARDS.has(id) && !to));
        setAsked((m) => new Map(m).set(id, to));
        done += 1;
      }
      setDraft(new Map());
      setSaved(
        `${done} change${done === 1 ? '' : 's'} saved: each lands at its unit's next safe point.`,
      );
      void read();
    } catch (e) {
      setWhy(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
      // The button that had the focus is spent: it goes back inside the dialog, for Esc and Tab.
      close.current?.focus();
    }
  };
  const guardsOff = units.filter((u) => GUARDS.has(u.id) && !isOn(u.id)).map((u) => u.id);
  const guardsDrafted = [...draft].filter(([id, to]) => GUARDS.has(id) && !to).map(([id]) => id);

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: a click beside the dialog closes it
    <div
      className="modal-scrim"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        ref={box}
        className="su"
        role="dialog"
        aria-modal="true"
        aria-labelledby="su-title"
        onKeyDown={onKey}
      >
        <header className="su-head">
          <h2 id="su-title">
            <Icon icon={SlidersHorizontal} /> This session's units
          </h2>
          <button
            ref={close}
            type="button"
            className="pf-icon su-close"
            aria-label="Close"
            onClick={onClose}
          >
            <Icon icon={X} />
          </button>
        </header>
        <p className="su-sub">
          Flip the switches, then Save: each change lands at its unit's next safe point and is kept
          for this project as <code className="mono">/unit</code> keeps it in the terminal — but a
          guard switched off holds for this session only.
          {!controls && ' Another tab controls this session: take over to switch.'}
        </p>
        {guardsOff.length > 0 && (
          <p className="su-warn" role="status">
            <Icon icon={ShieldAlert} />
            <span>
              This session runs without: <Safe text={guardsOff.join(', ')} />
              {guardsDrafted.length > 0 ? ' once saved, for this session only.' : '.'}
            </span>
          </p>
        )}
        {why && (
          <p className="mapp-why" role="alert">
            <Safe text={why} />
          </p>
        )}
        <ul className="su-list" aria-label="Units">
          {units.map((u) => {
            const d = described(u);
            const on = isOn(u.id);
            const waiting = asked.has(u.id);
            const changed = draft.has(u.id);
            return (
              <li key={u.id} data-on={on} data-kind={u.kind} data-changed={changed}>
                <Icon icon={d.icon} />
                <span className="su-name">
                  <b>
                    <Safe text={d.name} />
                  </b>
                  <small>
                    <Safe text={d.about} />
                  </small>
                </span>
                {changed && <small className="su-changed">unsaved</small>}
                {waiting && !changed && (
                  <small className="su-pending">at its next safe point</small>
                )}
                {failed(u.id) && <small className="su-failed">failed</small>}
                {GUARDS.has(u.id) && <small className="su-guard">guard</small>}
                <button
                  type="button"
                  role="switch"
                  aria-checked={on}
                  aria-label={`${d.name}: ${on ? 'on' : 'off'}`}
                  className="bapp-switch"
                  disabled={!controls || listed === undefined || saving}
                  onClick={() => flip(u.id)}
                />
                <span className="su-state" data-on={on} aria-hidden="true">
                  {on ? 'On' : 'Off'}
                </span>
              </li>
            );
          })}
        </ul>
        <footer className="su-foot">
          <span className="su-saved" role="status">
            {saved ??
              (draft.size > 0
                ? `${draft.size} unsaved change${draft.size === 1 ? '' : 's'}`
                : 'No change')}
          </span>
          <button
            type="button"
            className="ss-configure"
            disabled={draft.size === 0 || saving}
            onClick={() => {
              setDraft(new Map());
              close.current?.focus();
            }}
          >
            Discard
          </button>
          <button
            type="button"
            className="bapp-go"
            disabled={!controls || draft.size === 0 || saving}
            onClick={() => void save()}
          >
            <Icon icon={Save} /> {saving ? 'Saving…' : 'Save'}
          </button>
        </footer>
        {listed && listed.refused.length > 0 && (
          <ul className="su-refused" aria-label="Refused">
            {listed.refused.slice(-3).map((r) => (
              <li key={r}>
                <Safe text={r} />
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
