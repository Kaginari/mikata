/**
 * The board's icons: Lucide's line set (ISC), one stroke width everywhere, drawn in the text's
 * colour and hidden from assistive technology — the words beside an icon carry its meaning.
 *
 * `Glyph` keeps the board's old glyph strings as the names of meanings (a status table says
 * `'⚠'`, a test counts distinct glyphs) and draws each as its line icon; a glyph with no icon,
 * such as `·`, stays a character.
 */
import {
  ArrowDown,
  ArrowUp,
  Boxes,
  Brain,
  ChartColumn,
  Check,
  Circle,
  CircleDashed,
  CircleDot,
  Contrast,
  Crown,
  Diamond,
  Equal,
  Hexagon,
  Layers,
  LoaderCircle,
  type LucideIcon,
  Menu,
  Moon,
  Network,
  Plug,
  RefreshCw,
  RotateCw,
  Scissors,
  ScrollText,
  Settings,
  Sparkle,
  Sun,
  Swords,
  TriangleAlert,
  X,
} from 'lucide-react';

/** The one stroke every icon on the board is drawn with. */
const STROKE = 2;

export function Icon({
  icon: Drawn,
  className,
  filled = false,
}: {
  icon: LucideIcon;
  className?: string | undefined;
  filled?: boolean;
}) {
  return (
    <Drawn
      aria-hidden="true"
      focusable="false"
      size="1em"
      strokeWidth={STROKE}
      fill={filled ? 'currentColor' : 'none'}
      className={className ? `mn-icon ${className}` : 'mn-icon'}
    />
  );
}

/** Each glyph's icon, and whether it is drawn filled (a live dot, a held diamond). */
const GLYPHS: Record<string, [LucideIcon, boolean]> = {
  '●': [Circle, true],
  '○': [Circle, false],
  '◉': [CircleDot, false],
  '◐': [Contrast, false],
  '◌': [CircleDashed, false],
  '◆': [Diamond, true],
  '◇': [Diamond, false],
  '◈': [Layers, false],
  '✕': [X, false],
  '✗': [X, false],
  '✓': [Check, false],
  '⚠': [TriangleAlert, false],
  '✂': [Scissors, false],
  '☾': [Moon, false],
  '☀': [Sun, false],
  '♛': [Crown, false],
  '⚔': [Swords, false],
  '▲': [ArrowUp, false],
  '▼': [ArrowDown, false],
  '⣾': [LoaderCircle, false],
  '↻': [RotateCw, false],
  '≡': [Equal, false],
  '⟳': [RefreshCw, false],
  '⬡': [Hexagon, false],
  '▤': [ChartColumn, false],
  '▦': [ScrollText, false],
  '⚇': [Plug, false],
  '⚙': [Settings, false],
  '⛬': [Boxes, false],
  '✧': [Brain, false],
  '☰': [Menu, false],
  '✦': [Sparkle, false],
  '⌬': [Network, false],
};

/** A glyph drawn as its line icon (named by `data-glyph`), or as itself when it has none. */
export function Glyph({ glyph, className }: { glyph: string; className?: string | undefined }) {
  const drawn = GLYPHS[glyph];
  if (!drawn) return <>{glyph}</>;
  const spin = glyph === '⣾' ? 'mn-spin' : undefined;
  const cls = [className, spin].filter(Boolean).join(' ') || undefined;
  return (
    <span className="mn-glyph" data-glyph={glyph}>
      <Icon icon={drawn[0]} filled={drawn[1]} className={cls} />
    </span>
  );
}
