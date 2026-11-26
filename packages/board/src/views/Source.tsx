import type { Provenance } from '@mikata/protocol';
import { Safe } from '../safe/Safe.tsx';
import { Mark } from '../ui/Mark.tsx';

/** Where a part was read, and whether that source grants anything. */
export function Source({ what, p }: { what: string; p: Provenance }) {
  return (
    <p className="panel-source">
      {p.trusted ? (
        <Mark tone="holo" glyph="◆">
          {what}: trusted
        </Mark>
      ) : (
        <Mark tone="muted" glyph="◇">
          {what}: in-world, shown only
        </Mark>
      )}{' '}
      <span className="mono">
        <Safe text={p.source} />
      </span>
      {p.why && (
        <span className="panel-note">
          {' '}
          · <Safe text={p.why} />
        </span>
      )}
    </p>
  );
}
