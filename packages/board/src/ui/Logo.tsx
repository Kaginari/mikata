/**
 * Mikata's mark (assets/source/mikata-mark.png, the board's copy 400 px wide): a living cell — its
 * nucleus, its organelles — flowing into a digital one, a hexagon with a circuit round its core.
 * Decorative: the words beside it name Mikata.
 */
export function Logo({ className }: { className?: string }) {
  return (
    <img
      src="/mikata-mark.png"
      alt=""
      width={400}
      height={254}
      className={className ? `mikata-mark ${className}` : 'mikata-mark'}
    />
  );
}
