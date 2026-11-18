/**
 * A body's name as the human reads it: the session's own body is called `summoner` inside — its
 * memory, its MCP scope and its events keep that name — and Mikata on the page; the body of each
 * project of a split, `summoner/<project>`, is `mikata/<project>`. Every other name is as it is.
 * The terminal app's `safe::shown` holds the same rule.
 */
export function shownBody(name: string): string {
  if (name === 'summoner') return 'mikata';
  if (name.startsWith('summoner/')) return `mikata/${name.slice('summoner/'.length)}`;
  return name;
}
