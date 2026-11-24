// Enforces the crate graph of docs/03-architecture.md: every dependency edge between workspace
// crates (normal, dev and build) must be one the architecture draws, and the sandbox's own lint
// table must mirror the workspace's but for unsafe_code. Fails with every offence.
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';

const ALLOWED = {
  'mikata-wire': [],
  'mikata-races': [],
  'mikata-sandbox': [],
  'mikata-kernel': [],
  'mikata-memory': ['mikata-ledger', 'mikata-sandbox'],
  'mikata-ledger': ['mikata-races'],
  'mikata-proving': ['mikata-wire', 'mikata-races'],
  'mikata-tools': ['mikata-races', 'mikata-sandbox', 'mikata-wire', 'mikata-kernel'],
  'mikata-config': ['mikata-races', 'mikata-tools', 'mikata-ledger'],
  'mikata-provider': ['mikata-ledger', 'mikata-wire'],
  'mikata-world': [
    'mikata-wire',
    'mikata-races',
    'mikata-ledger',
    'mikata-memory',
    'mikata-proving',
    'mikata-sandbox',
  ],
  'mikata-loop': [
    'mikata-config',
    'mikata-world',
    'mikata-provider',
    'mikata-tools',
    'mikata-ledger',
    'mikata-memory',
    'mikata-races',
    'mikata-wire',
    'mikata-proving',
    'mikata-kernel',
  ],
  'mikata-server': ['mikata-loop'],
  mikatad: ['mikata-server'],
  'mikata-tui': ['mikata-server', 'mikata-loop', 'mikata-tools', 'mikata-races'],
};

const meta = JSON.parse(
  execFileSync('cargo', ['metadata', '--format-version', '1', '--no-deps'], { encoding: 'utf8' }),
);
const members = new Set(meta.packages.map((p) => p.name));
const errors = [];
for (const name of Object.keys(ALLOWED)) {
  if (!members.has(name)) errors.push(`${name}: in the graph but not in the workspace`);
}
for (const pkg of meta.packages) {
  const allowed = ALLOWED[pkg.name];
  if (!allowed) {
    errors.push(`${pkg.name}: not in the graph — add it to tools/check-layers.mjs and the book`);
    continue;
  }
  for (const dep of pkg.dependencies) {
    // A crate's own tests may depend on it with a test feature on: no edge in the graph.
    if (dep.name === pkg.name && dep.kind === 'dev') continue;
    if (members.has(dep.name) && !allowed.includes(dep.name)) {
      errors.push(`${pkg.name} → ${dep.name}: an edge the architecture does not draw`);
    }
  }
}

// The sandbox's lint table: the workspace's, with unsafe_code relaxed to deny.
const lints = (toml) => {
  const out = {};
  let section = '';
  for (const line of toml.split('\n')) {
    const head = line.match(/^\[(.+)\]$/);
    if (head) section = head[1].replace(/^workspace\./, '');
    const kv = line.match(/^([a-z_]+)\s*=\s*(.+)$/);
    if (kv && section.startsWith('lints.')) out[`${section}.${kv[1]}`] = kv[2];
  }
  return out;
};
const ws = lints(readFileSync('Cargo.toml', 'utf8'));
const sb = lints(readFileSync('crates/mikata-sandbox/Cargo.toml', 'utf8'));
for (const [key, value] of Object.entries(ws)) {
  const want = key === 'lints.rust.unsafe_code' ? '"deny"' : value;
  if (sb[key] !== want)
    errors.push(`mikata-sandbox: ${key} is ${sb[key] ?? 'missing'}, want ${want}`);
}

if (errors.length) {
  console.error(`layer check failed:\n  ${errors.join('\n  ')}`);
  process.exit(1);
}
console.log(`layer check: ${members.size} crates, every edge drawn, sandbox lints mirrored`);
