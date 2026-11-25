/**
 * Mikata's mark: three cells of one core, in the accent's gradient (hot through coral to amber),
 * a nucleus in each. `plain` draws it in the text's colour alone, for a coloured bar.
 */
export function Logo({
  id,
  plain = false,
  className,
}: {
  id: string;
  plain?: boolean;
  className?: string;
}) {
  const stroke = plain ? 'currentColor' : `url(#${id})`;
  return (
    <svg viewBox="-62 -48 124 128" aria-hidden="true" focusable="false" className={className}>
      {!plain && (
        <defs>
          <linearGradient id={id} x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" className="logo-hot" />
            <stop offset="55%" className="logo-coral" />
            <stop offset="100%" className="logo-amber" />
          </linearGradient>
        </defs>
      )}
      <path
        d="M0 -59 L39 -36 L39 9 L0 32 L-39 9 L-39 -36 Z M-39 9 L0 32 L0 77 L-39 100 L-78 77 L-78 32 Z M39 9 L78 32 L78 77 L39 100 L0 77 L0 32Z"
        transform="scale(.75)"
        fill="none"
        stroke={stroke}
        strokeWidth="8"
        strokeLinejoin="round"
        strokeLinecap="round"
      />
      <circle cx="0" cy="-10" r="7" className={plain ? 'logo-plain' : 'logo-coral-fill'} />
      <circle cx="-29" cy="41" r="7" className={plain ? 'logo-plain' : 'logo-hot-fill'} />
      <circle cx="29" cy="41" r="7" className={plain ? 'logo-plain' : 'logo-amber-fill'} />
    </svg>
  );
}
