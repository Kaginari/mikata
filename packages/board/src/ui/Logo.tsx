/**
 * Mikata's mark (assets/source/mikata-mark.png, the board's copy 600 px wide): a living cell
 * flowing into a digital one — membrane and nucleus, a cell becoming, a hexagon and its circuit.
 * `plain` draws it in white alone, for a coloured bar. Decorative: the words beside it name Mikata.
 */
export function Logo({ plain = false, className }: { plain?: boolean; className?: string }) {
  return (
    <img
      src="/mikata-mark.png"
      alt=""
      width={600}
      height={217}
      className={`${className ?? ''} mikata-mark${plain ? ' mikata-mark-plain' : ''}`.trim()}
    />
  );
}
