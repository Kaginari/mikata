/**
 * A long list drawn in a window: only the rows near the viewport are in the DOM, between two
 * spacers that stand for the rest. Each row's height is measured once it is drawn (a
 * `ResizeObserver`) and estimated until then; a list scrolled to its end follows what is added,
 * one scrolled up stays where it is. The spacers' heights are set through the CSSOM — never a
 * style written as markup — so the board's CSP holds.
 *
 * Whether the list is at its end is known from scroll events, and a draw can come before the
 * event of a scroll just made: a message arriving, a row measured. So each draw looks at the
 * scroller itself first. One that is higher than the last draw left it, and off the end, was
 * moved by the reader: it is not pulled back, and the event that follows says the rest.
 */
import {
  type ReactNode,
  useCallback,
  useEffect,
  useLayoutEffect,
  useReducer,
  useRef,
  useState,
} from 'react';

/** The height a row is taken to have before it is measured. */
export const ESTIMATE = 72;
/** How far beyond the viewport rows are drawn, above and below. */
const OVERSCAN = 800;
/** How close to the end counts as at the end. */
const NEAR_END = 48;

/** The first index whose row ends below `y`, by binary search over the prefix sums. */
function find(offsets: Float64Array, y: number): number {
  let lo = 0;
  let hi = offsets.length - 1;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if ((offsets[mid + 1] ?? 0) <= y) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

export function Virtual<T>({
  items,
  keyOf,
  render,
  tail,
  label,
}: {
  items: readonly T[];
  keyOf: (item: T) => string;
  render: (item: T, index: number) => ReactNode;
  /** What follows the rows, always drawn: the text streaming, the live line. */
  tail?: ReactNode;
  label: string;
}) {
  const [away, setAway] = useState(false);
  const scroller = useRef<HTMLElement>(null);
  const top = useRef<HTMLDivElement>(null);
  const bottom = useRef<HTMLDivElement>(null);
  const heights = useRef(new Map<string, number>());
  const atEnd = useRef(true);
  /** The reader has scrolled away from the end and the scroll's event has not come yet. */
  const left = useRef(false);
  /** Where the last draw or scroll event left the scroller. */
  const lastTop = useRef(0);
  const view = useRef({ y: 0, h: 800 });
  const [, redraw] = useReducer((x: number) => x + 1, 0);
  const pending = useRef<number | undefined>(undefined);
  const later = useCallback(() => {
    if (pending.current !== undefined) return;
    pending.current = window.requestAnimationFrame(() => {
      pending.current = undefined;
      redraw();
    });
  }, []);
  useEffect(
    () => () => {
      if (pending.current !== undefined) window.cancelAnimationFrame(pending.current);
    },
    [],
  );

  const n = items.length;
  const offsets = new Float64Array(n + 1);
  for (let i = 0; i < n; i++) {
    const item = items[i] as T;
    offsets[i + 1] = (offsets[i] ?? 0) + (heights.current.get(keyOf(item)) ?? ESTIMATE);
  }
  const total = offsets[n] ?? 0;
  const live = scroller.current;
  if (
    live &&
    atEnd.current &&
    lastTop.current - live.scrollTop > NEAR_END &&
    live.scrollHeight - live.scrollTop - live.clientHeight > NEAR_END
  ) {
    left.current = true;
  }
  const pinned = atEnd.current && !left.current;
  const h = view.current.h;
  const y = left.current && live ? live.scrollTop : view.current.y;
  const from = pinned ? Math.max(0, total - h) : y;
  const start = n === 0 ? 0 : find(offsets, Math.max(0, from - OVERSCAN));
  const end = n === 0 ? 0 : Math.min(n, find(offsets, from + h + OVERSCAN) + 1);

  // Rows measured as they are drawn and whenever they change size.
  const observer = useRef<ResizeObserver | undefined>(undefined);
  if (!observer.current && typeof ResizeObserver !== 'undefined') {
    observer.current = new ResizeObserver((entries) => {
      let changed = false;
      for (const e of entries) {
        const el = e.target as HTMLElement;
        const key = el.dataset.vkey;
        if (key === undefined) continue;
        const height = el.getBoundingClientRect().height;
        if (Math.abs((heights.current.get(key) ?? -1) - height) > 0.5) {
          heights.current.set(key, height);
          changed = true;
        }
      }
      if (changed) later();
    });
  }
  useEffect(() => () => observer.current?.disconnect(), []);
  const measure = useCallback((el: HTMLDivElement | null) => {
    const o = observer.current;
    if (!el || !o) return;
    o.observe(el);
    return () => o.unobserve(el);
  }, []);

  useLayoutEffect(() => {
    if (top.current) top.current.style.height = `${offsets[start] ?? 0}px`;
    if (bottom.current) bottom.current.style.height = `${total - (offsets[end] ?? total)}px`;
    const s = scroller.current;
    if (s && pinned) s.scrollTop = s.scrollHeight;
    if (s) lastTop.current = s.scrollTop;
  });

  const onScroll = () => {
    const s = scroller.current;
    if (!s) return;
    const end = s.scrollHeight - s.scrollTop - s.clientHeight < NEAR_END;
    if (end !== atEnd.current) setAway(!end);
    atEnd.current = end;
    left.current = false;
    lastTop.current = s.scrollTop;
    view.current = { y: s.scrollTop, h: s.clientHeight };
    later();
  };

  useLayoutEffect(() => {
    const s = scroller.current;
    if (!s) return;
    view.current = { y: s.scrollTop, h: s.clientHeight };
    const ro = new ResizeObserver(() => {
      view.current = { y: s.scrollTop, h: s.clientHeight };
      later();
    });
    ro.observe(s);
    return () => ro.disconnect();
  }, [later]);

  const rows: ReactNode[] = [];
  for (let i = start; i < end; i++) {
    const item = items[i] as T;
    const key = keyOf(item);
    rows.push(
      <div key={key} data-vkey={key} ref={measure} className="v-row">
        {render(item, i)}
      </div>,
    );
  }

  return (
    <div className="v-frame">
      <section
        ref={scroller}
        className="v-scroll"
        // biome-ignore lint/a11y/noNoninteractiveTabindex: the transcript scrolls by keyboard too
        tabIndex={0}
        aria-label={label}
        onScroll={onScroll}
        data-rows={end - start}
        data-total={n}
      >
        <div ref={top} className="v-spacer" aria-hidden="true" />
        {rows}
        <div ref={bottom} className="v-spacer" aria-hidden="true" />
        {tail}
      </section>
      {away && (
        <button
          type="button"
          className="v-jump"
          onClick={() => {
            atEnd.current = true;
            left.current = false;
            setAway(false);
            toEnd(scroller.current);
            later();
          }}
        >
          ↓ Latest
        </button>
      )}
    </div>
  );
}

/** Scroll `el`, the list's scroller, to its end. */
export function toEnd(el: Element | null): void {
  if (el) el.scrollTop = el.scrollHeight;
}
