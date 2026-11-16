/**
 * The client for mikatad: one JSON-RPC 2.0 connection, typed requests and events, the human
 * gate answered by a handler, and replay by `seq` so a client that reconnects loses nothing.
 * Its transports: `stdio()` (Node) starts the core; `ws()` (the browser) speaks to a board chat.
 * A browser imports `@mikata/sdk/browser`, which leaves the Node transport out.
 */
export type * from '@mikata/protocol';
export * from './client.js';
export * from './stdio.js';
export * from './ws.js';
