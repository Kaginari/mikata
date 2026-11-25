/**
 * An icon from Font Awesome Free's solid set (icons CC BY 4.0, code MIT): its path drawn inline
 * in the text's colour, hidden from assistive technology — the words beside it carry the meaning.
 */
import type { IconDefinition } from '@fortawesome/free-solid-svg-icons';

export function Icon({ icon, className }: { icon: IconDefinition; className?: string }) {
  const [width, height, , , path] = icon.icon;
  const paths = Array.isArray(path) ? path : [path];
  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      aria-hidden="true"
      focusable="false"
      className={className ? `fa-icon ${className}` : 'fa-icon'}
    >
      {paths.map((d) => (
        <path key={d.slice(0, 24)} d={d} fill="currentColor" />
      ))}
    </svg>
  );
}
