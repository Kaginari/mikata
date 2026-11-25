<script setup lang="ts">
import { computed, ref } from 'vue';

// The site's front page. Every unit id, profile, command and number here is Mikata's own
// (crates/mikata-loop/src/units.rs, the README, docs/bench.md): keep them in step.

const GITHUB = 'https://github.com/Kaginari/mikata';
const BLOB = `${GITHUB}/blob/main`;

// The native units, in the order the loop meets them.
const units = [
  {
    id: 'plan',
    name: 'Plan',
    desc: 'The prompt asks for a Done when list and its paths before code',
  },
  { id: 'reader', name: 'Reader', desc: 'A second model writes the Done when list from the ask' },
  {
    id: 'hold',
    name: 'Done when, held',
    desc: 'No write while a line has no check; every line runs',
  },
  { id: 'gate', name: 'End gate', desc: 'Your verify line, and no test deleted or skipped' },
  { id: 'memory', name: 'Memory', desc: 'Recall, your corrections, the notes a run keeps' },
  { id: 'fold', name: 'Context fold', desc: 'Folds the oldest exchanges as the context fills' },
  { id: 'stress', name: 'Stress signals', desc: 'Warns when its acts circle; ends a doomed loop' },
  {
    id: 'what_changed',
    name: 'What changed',
    desc: 'Its checks, run before and after, side by side',
  },
  { id: 'challenge', name: 'Challenge', desc: 'Holds a landing in a routed domain for review' },
] as const;

type UnitId = (typeof units)[number]['id'];

const presets: Record<string, { label: string; off: UnitId[] }> = {
  default: { label: 'Default', off: [] },
  'small-fix': { label: 'small-fix', off: ['plan'] },
  bare: { label: 'Bare loop', off: units.map((u) => u.id) },
};

const on = ref<Record<string, boolean>>(Object.fromEntries(units.map((u) => [u.id, true])));
const preset = ref<string | null>('default');

function pick(name: string) {
  preset.value = name;
  on.value = Object.fromEntries(units.map((u) => [u.id, !presets[name].off.includes(u.id)]));
}

const off = computed(() => units.filter((u) => !on.value[u.id]).map((u) => u.id));
const count = computed(() => units.length - off.value.length);
const composition = computed(() => off.value.map((id) => `-${id}`).join(','));

const config = computed(() =>
  off.value.length === 0
    ? '# ~/.config/mikata/config.toml\n# Every unit is on unless a file says otherwise.\n[units]'
    : `# ~/.config/mikata/config.toml\n[units]\n${off.value.map((id) => `${id} = false`).join('\n')}\n\n# or name the mix, for --profile mine\n[profiles.mine]\nunits = "${composition.value}"`,
);

const command = computed(() => {
  if (preset.value === 'small-fix') return "mikata run --profile small-fix 'Fix the parser'";
  if (off.value.length === 0) return "mikata run 'Fix the parser'";
  return `mikata run --units ${composition.value} 'Fix the parser'`;
});

const risks = computed(() => {
  const r: string[] = [];
  if (!on.value.gate) r.push('End gate off: an answer lands unjudged.');
  if (!on.value.hold)
    r.push('Done when not held: the list is the body’s notes, and nothing holds it.');
  return r;
});

const modes = {
  interactive: {
    label: 'Interactive',
    code: 'mikata',
    desc: 'A session in the terminal app: say what you want in plain words.',
  },
  once: {
    label: 'One-shot',
    code: 'mikata run "Support 1h30m durations"',
    desc: 'One ask, without the terminal app.',
  },
  json: {
    label: 'JSON / CI',
    code: 'echo "Fix the duration parser" | mikata run --json',
    desc: 'Every event as a JSON line, then a result line.',
  },
  web: {
    label: 'Web board',
    code: 'mikata --web\n# or the board alone, on loopback:\nmikatad board',
    desc: 'Your worlds, approvals, memory and spend in the browser.',
  },
} as const;
type Mode = keyof typeof modes;
const mode = ref<Mode>('interactive');

const copied = ref<string | null>(null);
async function copy(key: string, text: string) {
  try {
    await navigator.clipboard.writeText(text);
    copied.value = key;
    setTimeout(() => {
      if (copied.value === key) copied.value = null;
    }, 1200);
  } catch {
    copied.value = null;
  }
}

const install = 'curl -fsSL https://raw.githubusercontent.com/Kaginari/mikata/main/install.sh | sh';
const model = `[providers.openrouter]
kind = "openai"
base_url = "https://openrouter.ai/api"
key_env = "OPENROUTER_API_KEY"

[models]
provider = "openrouter"
default = "anthropic/claude-sonnet-5.5"`;
const ask = `Support mixed hour and minute durations.

## Done when
D1: parse("1h30m") must give 5400
D2: parse("90") must stay as it is today`;
const plugin = `[plugins.no-force-push]
command = "python3"
args = ["/path/to/examples/plugins/no_force_push.py"]
grants = ["veto"]
races = ["mikata"]`;

const commands = [
  ['mikata', 'A session in the terminal app'],
  ['mikata run "<ask>"', 'One ask; --json for events, --units or --profile for the mix'],
  ['mikata resume last', 'Pick up a run that stopped'],
  ['mikata status', 'Check the provider, the sandbox and the setup'],
  ['mikata --web · mikatad board', 'The web board, on loopback'],
  [
    'mikatad memory wrong "<what still fails>"',
    'Say an answer was wrong; the next session is told first',
  ],
  ['mikatad usage · mikatad meters', 'What was spent, and the budgets'],
  ['mikata-guard', 'The Linux kernel guard every command runs through'],
];
</script>

<template>
  <div class="mk">
    <header>
      <nav>
        <a class="logo" href="/"><img src="/mikata-icon@64.png" alt="" width="28" height="28">
          <span class="pink">mikata</span> <span class="jp">味方</span></a>
        <a class="opt" href="#units">Units</a>
        <a class="opt" href="#builder">Compose</a>
        <a class="opt" href="#plugins">Plugins</a>
        <a href="#start">Quickstart</a>
        <a href="#bench">Bench</a>
        <a href="/book">Book</a>
        <a :href="GITHUB">GitHub ↗</a>
      </nav>
    </header>

    <main>
      <div class="hero">
        <div class="wrap hero-grid">
          <div>
            <div class="eyebrow">Open source · Rust · MIT</div>
            <h1>Your agent.<br>Your rules.<br><span class="gradient">Your harness.</span></h1>
            <p class="lede">
              The coding agent on your side. It writes down what done means before any code, does
              the work, and checks every line before it answers. Every change passes the end gate:
              your verify line runs green, and no test is deleted or skipped.
            </p>
            <div class="actions">
              <a class="btn primary" href="#start">Get started →</a>
              <a class="btn" href="#builder">Compose your loop</a>
            </div>
            <p class="small">Works inside your repository · every command sandboxed · units and plugins experimental on main</p>
          </div>
          <div class="terminal" aria-label="An example session">
            <div class="termhead"><span>● ● ● &nbsp; example / parser-rs</span><span class="green">● landed</span></div>
            <div class="termcontent">
              <div class="pink">❯ mikata run "Support 1h30m durations"</div>
              <div class="termrow">
                <span class="dim">00:01</span> Done when · 2 lines held<br>
                <span class="green">✓</span> D1 parse("1h30m") must give 5400<br>
                <span class="green">✓</span> D2 parse("90") must stay as it is today
              </div>
              <div class="termrow"><span class="dim">00:06</span> edit src/parse.rs <span class="green">+47</span> <span class="pink">−8</span></div>
              <div class="termrow">
                <span class="dim">00:18</span> End gate · cargo test<br>
                <span class="green">42 passed · 0 failed · 0 skipped</span>
              </div>
              <div class="termrow">
                WHAT CHANGED<br>
                <span class="pink">− before: Err(InvalidFormat)</span><br>
                <span class="green">+ after: Ok(5400)</span>
              </div>
              <div class="meter"><i style="width: 100%"></i></div>
              <div class="green">✓ Answer landed · every line held</div>
              <div class="small">GUARD ON &nbsp; 14 steps &nbsp; $0.038 &nbsp; 00:27</div>
            </div>
          </div>
        </div>
        <div class="wrap strip">
          <span>RUST CORE</span><span>MIT LICENSE</span><span>LINUX KERNEL SANDBOX</span>
          <span>MCP + PLUGIN HOOKS</span><span>LOCAL JOURNAL</span>
        </div>
      </div>

      <section id="units">
        <div class="wrap">
          <div class="eyebrow">01 / Architecture</div>
          <div class="two">
            <div>
              <h2>One small loop.<br><span class="gradient">Every part a unit.</span></h2>
              <p>
                Memory, the end gate, the held <code>Done when</code> list, the fold, what-changed and
                the rest are units: switched on and off by configuration, on the command line, or
                live in a session. Plugins are your own programs that speak MCP plus
                <code>mikata/hook</code>, loaded, unloaded or swapped while a session runs.
              </p>
              <p>The body codes. The harness holds the contract, the journal, the budgets and the sandbox.</p>
              <span class="tag experimental">Experimental · on main</span>
            </div>
            <div class="card architecture">
              <div class="nodes"><div class="node">Plan</div><div class="node">Reader</div><div class="node">Done when</div><div class="node">End gate</div></div>
              <div class="connector">↓ &nbsp; ↓ &nbsp; ↓ &nbsp; ↓</div>
              <div class="core">
                <strong class="gradient core-title">MIKATA 味方</strong><br>Rust runtime<br>
                <span class="small">Journal · sandbox · budgets · meters</span>
              </div>
              <div class="connector">↑ &nbsp; ↑ &nbsp; ↑ &nbsp; ↑</div>
              <div class="nodes"><div class="node">Memory</div><div class="node">Fold</div><div class="node">Stress</div><div class="node">What changed</div></div>
              <div class="node plug">plugin:&lt;id&gt; · your programs, over MCP + mikata/hook</div>
            </div>
          </div>
          <div class="grid">
            <div class="card"><span class="tag">01 / Contract</span><h3>Done when</h3><p>Lines you can run say what success is before any edit; what must stay is read from your code before anything changes.</p></div>
            <div class="card"><span class="tag">02 / Gate</span><h3>Proof, not promises</h3><p>The end gate runs your verify line on every change and refuses a deleted or skipped test.</p></div>
            <div class="card"><span class="tag">03 / Yours</span><h3>Local by design</h3><p>A journal, memory and config under <code>.mikata/</code> in your repository. A push, a pull request or a publish is put to you first.</p></div>
          </div>
        </div>
      </section>

      <section id="builder">
        <div class="wrap">
          <div class="eyebrow">02 / Compose · experimental</div>
          <h2>Build your own loop.</h2>
          <p>Switch units and see the configuration and the command for that mix. The unit ids and the <code>small-fix</code> profile are the build’s own.</p>
          <div class="tabs">
            <button v-for="(p, key) in presets" :key="key" type="button" :aria-pressed="preset === key" @click="pick(key)">{{ p.label }}</button>
          </div>
          <div class="builder">
            <div class="card">
              <h3>Native units <span class="pink">{{ count }}/{{ units.length }}</span></h3>
              <label v-for="u in units" :key="u.id" class="unit">
                <span>{{ u.name }} <code>{{ u.id }}</code><small>{{ u.desc }}</small></span>
                <input v-model="on[u.id]" type="checkbox" :aria-label="u.name" @change="preset = null">
              </label>
            </div>
            <div class="card">
              <h3>Configuration</h3>
              <div class="code">
                <div class="codehead">config.toml <button class="copy" type="button" @click="copy('config', config)">{{ copied === 'config' ? 'Copied' : 'Copy' }}</button></div>
                <pre>{{ config }}</pre>
              </div>
              <div v-for="r in risks" :key="r" class="warning">{{ r }}</div>
              <div class="code">
                <div class="codehead">One run <button class="copy" type="button" @click="copy('cmd', command)">{{ copied === 'cmd' ? 'Copied' : 'Copy' }}</button></div>
                <pre>{{ command }}</pre>
              </div>
              <p class="small">
                Live in the terminal app: <code>/units</code>, <code>/unit on|off &lt;id&gt;</code>.
                Your global file may switch any unit either way; a repository’s own file may only
                switch off what loosens nothing. <code>plan</code> is decided when a session opens.
              </p>
            </div>
          </div>
        </div>
      </section>

      <section id="plugins">
        <div class="wrap">
          <div class="eyebrow">03 / Extend the harness</div>
          <h2>Make the agent yours.</h2>
          <p>A plugin is a program in any language: an MCP server that also speaks <code>mikata/hook</code>. It runs in the body’s sandbox, with a deadline on every hook and three strikes before it is taken out.</p>
          <div class="grid">
            <div class="card"><span class="tag experimental">Native units</span><h3>Switch capabilities</h3><p>Turn loop units on and off in your global file, per run, or live in a session.</p><code>[units] · --units · /unit on|off</code></div>
            <div class="card"><span class="tag experimental">Plugins</span><h3>Extend or replace</h3><p>Watch, advise, veto a call, stop an ask, or bring tools. A memory plugin can stand in for the native one.</p><code>/plugin load|unload|swap</code></div>
            <div class="card"><span class="tag experimental">Profiles</span><h3>A mix per kind of work</h3><p>Name a mix of units once and pick it per run. <code>small-fix</code> leaves out the plan.</p><code>--profile small-fix</code></div>
          </div>
          <div class="two plugin-row">
            <div class="card">
              <h3>Grant only what it needs</h3>
              <p>A plugin gets what your global file grants it, weakest first. A reply above its grant is dropped and journaled.</p>
              <table class="grants">
                <tbody>
                  <tr><td><code>observe</code></td><td>see events</td></tr>
                  <tr><td><code>advise</code></td><td>add notes and preface text</td></tr>
                  <tr><td><code>veto</code></td><td>refuse a tool call or an answer</td></tr>
                  <tr><td><code>stop</code></td><td>end an ask</td></tr>
                  <tr><td><code>tools</code></td><td>offer its own tools</td></tr>
                </tbody>
              </table>
            </div>
            <div class="card">
              <h3>Declare it</h3>
              <div class="code">
                <div class="codehead">~/.config/mikata/config.toml <button class="copy" type="button" @click="copy('plugin', plugin)">{{ copied === 'plugin' ? 'Copied' : 'Copy' }}</button></div>
                <pre>{{ plugin }}</pre>
              </div>
              <div class="actions">
                <a class="btn" :href="`${GITHUB}/tree/main/examples/plugins`">Example plugins + Python SDK ↗</a>
                <a class="btn" href="/experimental/plugins">The plugin contract</a>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section id="start">
        <div class="wrap">
          <div class="eyebrow">04 / Quickstart</div>
          <h2>From ask to verified change.</h2>
          <div class="two top">
            <div>
              <div class="code">
                <div class="codehead">01 / Install (Linux, macOS) <button class="copy" type="button" @click="copy('install', install)">{{ copied === 'install' ? 'Copied' : 'Copy' }}</button></div>
                <pre>{{ install }}</pre>
              </div>
              <div class="code">
                <div class="codehead">02 / Set a model · ~/.config/mikata/config.toml <button class="copy" type="button" @click="copy('model', model)">{{ copied === 'model' ? 'Copied' : 'Copy' }}</button></div>
                <pre>{{ model }}</pre>
              </div>
              <div class="tabs">
                <button v-for="(m, key) in modes" :key="key" type="button" :aria-pressed="mode === key" @click="mode = key">{{ m.label }}</button>
              </div>
              <div class="code">
                <div class="codehead">03 / Run, in your repository <button class="copy" type="button" @click="copy('mode', modes[mode].code)">{{ copied === 'mode' ? 'Copied' : 'Copy' }}</button></div>
                <pre>{{ modes[mode].code }}</pre>
              </div>
              <p>{{ modes[mode].desc }}</p>
            </div>
            <div class="card">
              <h3>Give it a contract.</h3>
              <p>Lines under <code>## Done when</code> are held: one check a line, and an answer that leaves one wrong does not land. <code>must stay</code> lines protect what works today.</p>
              <div class="code">
                <div class="codehead">Example ask <button class="copy" type="button" @click="copy('ask', ask)">{{ copied === 'ask' ? 'Copied' : 'Copy' }}</button></div>
                <pre>{{ ask }}</pre>
              </div>
              <p class="small">Give it your test command: until a repository has a verify line, <code>mikatad memory status</code> suggests one. A run that stops at a budget picks up with <code>mikata resume last</code>.</p>
            </div>
          </div>
          <h3 class="ref">Command reference</h3>
          <table>
            <thead><tr><th>Command</th><th>For</th></tr></thead>
            <tbody>
              <tr v-for="[c, d] in commands" :key="c"><td><code>{{ c }}</code></td><td>{{ d }}</td></tr>
            </tbody>
          </table>
          <div class="actions">
            <a class="btn" :href="`${GITHUB}/releases`">Releases ↗</a>
            <a class="btn" :href="`${BLOB}/README.md#install`">Other ways to install ↗</a>
          </div>
        </div>
      </section>

      <section id="bench">
        <div class="wrap">
          <div class="eyebrow">05 / Evidence over hype</div>
          <h2>Measured, not marketed.</h2>
          <div class="grid">
            <div class="card"><div class="stat gradient">16 / 20</div><h3>Real Python fixes</h3><p>$0.50 a pass, with Maki 15 at $0.91 and OpenCode 14 at $0.87: level on passes, same small open model.</p></div>
            <div class="card"><div class="stat gradient">1/6 → 6/6</div><h3>Hard prose asks</h3><p>The two asks our bench fails most: 1 pass in 6 without a list, 6 in 6 with one held.</p></div>
            <div class="card"><div class="stat gradient">2×</div><h3>Say the result</h3><p>Asks with examples of the expected output passed twice as often as the same asks in prose.</p></div>
          </div>
          <p class="small note">Our own bench, two codebases, numbers as measured; with a stronger model every harness passed all ten JavaScript fixes, and Mikata is not always the cheapest. Writing down what done means and checking it takes calls. <a class="link" href="/bench">Every screen, and how →</a></p>
        </div>
      </section>

      <section>
        <div class="wrap">
          <div class="eyebrow">06 / FAQ</div>
          <h2>Know what you’re running.</h2>
          <details><summary>Is everything a plugin?</summary><p>No. Native units are compiled in and switch on and off live; their code changes by a build and a resume. Plugins are separate processes, started only from your global configuration.</p></details>
          <details><summary>Are the guarantees always on?</summary><p>Only while the end gate and the held <code>Done when</code> list are on. Switch the gate off and an answer lands unjudged; a repository’s own config can never do that, only your global file or your command line.</p></details>
          <details><summary>How does Mikata protect my repository?</summary><p>It reads and writes inside your repository, every command runs in a sandbox (<code>mikata-guard</code> on Linux), and an act that leaves your machine, a push, a pull request or a publish, is put to you first, every time.</p></details>
          <details><summary>Where is memory kept?</summary><p>In your repository, under <code>.mikata/</code>: the journal, the notes, your corrections. A correction that says what must come out becomes a held line the next time that ask comes.</p></details>
          <details><summary>Which models does it run?</summary><p>Anthropic’s API or any OpenAI-compatible endpoint, named once in <code>~/.config/mikata/config.toml</code>. A local OpenAI-compatible endpoint keeps inference on your machine.</p></details>
          <details><summary>Are units, profiles and plugins released?</summary><p>They are on main, experimental, under Unreleased in the changelog. Check the releases for what your version has.</p></details>
        </div>
      </section>
    </main>

    <footer>
      <div class="wrap two">
        <div>
          <div class="logo"><span class="pink">mikata</span> <span class="jp">味方</span></div>
          <h2 class="foot">Ship with evidence.</h2>
          <p>Open source · Rust · MIT</p>
        </div>
        <div class="actions">
          <a class="btn primary" href="#start">Get started →</a>
          <a class="btn" href="/book">Read the book</a>
          <a class="btn" :href="GITHUB">GitHub ↗</a>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
/* The owner's neon palette: hot to amber over plum. */
.mk {
  --hot: #ff168a;
  --amber: #ffb020;
  --accent: linear-gradient(135deg, var(--hot), color-mix(in oklab, var(--hot) 55%, var(--amber)));
  --bg: #0b0a0e;
  --panel: #151119;
  --line: #332735;
  --muted: #aaa0ae;
  --text: #f8f1f8;
  --green: #a3f77b;
  background: var(--bg);
  color: var(--text);
  font: 15px/1.65 system-ui, -apple-system, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  min-height: 100vh;
}
.mk * { box-sizing: border-box; }
.mk a { color: inherit; text-decoration: none; }
.mk button { font: inherit; cursor: pointer; color: inherit; }
.mk :is(a, button, input):focus-visible { outline: 2px solid var(--hot); outline-offset: 4px; }
header { position: sticky; top: 0; z-index: 5; background: #0b0a0ef2; border-bottom: 1px solid var(--line); backdrop-filter: blur(15px); }
nav { max-width: 1260px; margin: auto; display: flex; align-items: center; gap: 24px; padding: 15px 24px; flex-wrap: wrap; }
nav a:not(.logo) { font-size: 13px; color: var(--muted); }
nav a:hover { color: white; }
.logo { font: bold 23px monospace; margin-right: auto; display: inline-flex; align-items: center; gap: 8px; }
.logo img { border-radius: 6px; }
.jp { color: var(--muted); font-weight: 400; }
.pink { color: #ff659f; }
.wrap { max-width: 1200px; margin: auto; padding: 0 24px; }
section { padding: 84px 0; border-bottom: 1px solid var(--line); scroll-margin-top: 70px; }
.hero { padding: 95px 0 0; background: radial-gradient(ellipse at 75% 15%, #38152c 0, transparent 60%); border-bottom: 1px solid var(--line); }
.hero-grid, .two { display: grid; grid-template-columns: 1fr 1fr; gap: 36px; align-items: center; }
.two.top, .plugin-row { align-items: start; }
.plugin-row { margin-top: 16px; gap: 16px; }
.mk :is(h1, h2, h3, p) { margin: 0 0 16px; border: 0; padding: 0; }
h1 { font-size: clamp(48px, 6.5vw, 94px); line-height: 0.98; letter-spacing: -0.075em; font-weight: 800; }
h2 { font-size: clamp(30px, 4vw, 52px); line-height: 1.1; letter-spacing: -0.045em; font-weight: 750; }
h3 { font-size: 19px; line-height: 1.3; font-weight: 650; }
p { color: var(--muted); }
.lede { font-size: 18px; margin-top: 24px !important; }
.gradient { background: var(--accent); -webkit-background-clip: text; background-clip: text; color: transparent; }
.eyebrow { font: 12px/1.5 monospace; color: #ff83ad; letter-spacing: 0.15em; text-transform: uppercase; margin-bottom: 20px; }
.btn { display: inline-block; border: 1px solid var(--line); background: var(--panel); padding: 11px 17px; border-radius: 7px; font-weight: 650; font-size: 14px; }
.btn:hover { border-color: #ff659f; }
.btn.primary { background: var(--accent); border-color: transparent; color: #160a12; }
.actions { display: flex; flex-wrap: wrap; gap: 10px; margin: 24px 0; }
.small { font-size: 12px; color: var(--muted); }
.link { color: #ff83ad; }
.terminal, .card { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; overflow: hidden; }
.terminal { box-shadow: 0 18px 90px #ff168a18; border-color: #71334d; font: 13px/1.7 ui-monospace, monospace; }
.termhead { padding: 12px 16px; border-bottom: 1px solid var(--line); display: flex; justify-content: space-between; color: #f7acc7; }
.termcontent { padding: 18px; }
.green { color: var(--green); }
.dim { color: var(--muted); }
.termrow { padding: 7px 0; border-bottom: 1px solid #30242e; }
.meter { height: 5px; background: #372b38; border-radius: 4px; overflow: hidden; margin: 8px 0; }
.meter i { display: block; height: 100%; background: var(--accent); }
.strip { display: flex; justify-content: space-between; gap: 15px; flex-wrap: wrap; padding: 75px 24px 20px; font: 12px monospace; color: var(--muted); }
.grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; margin-top: 26px; }
.card { padding: 22px; }
.card p:last-child { margin-bottom: 0; }
.tag { display: inline-block; font: 11px monospace; color: #ff9dc0; border: 1px solid #63374b; padding: 5px 8px; border-radius: 4px; margin-bottom: 12px; }
.tag.experimental { color: #ffd18a; border-color: #69512e; }
.architecture { display: grid; gap: 12px; text-align: center; padding: 24px; }
.nodes { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
.node { border: 1px solid #67374e; border-radius: 6px; padding: 12px 5px; font: 12px monospace; background: #231622; }
.node.plug { border-style: dashed; color: #ffd18a; border-color: #69512e; }
.core { padding: 24px; border: 1px solid #ff548c; border-radius: 9px; background: linear-gradient(135deg, #321529, #271a1b); }
.core-title { font-size: 25px; }
.connector { color: #ff719e; font: 18px monospace; }
.builder { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-top: 26px; align-items: start; }
.unit { display: flex; gap: 12px; align-items: center; padding: 12px 0; border-bottom: 1px solid var(--line); cursor: pointer; }
.unit span { flex: 1; }
.unit code { font-size: 11px; margin-left: 4px; }
.unit small { display: block; color: var(--muted); font-size: 12px; }
.unit input { appearance: none; width: 36px; height: 20px; border-radius: 20px; background: #493943; position: relative; cursor: pointer; flex-shrink: 0; margin: 0; }
.unit input:checked { background: var(--accent); }
.unit input::before { content: ''; position: absolute; left: 3px; top: 3px; width: 14px; height: 14px; border-radius: 50%; background: white; transition: left 0.12s; }
.unit input:checked::before { left: 19px; }
.tabs { display: flex; flex-wrap: wrap; gap: 7px; margin: 18px 0; }
.tabs button { background: #1a151e; border: 1px solid var(--line); padding: 8px 12px; border-radius: 6px; font-size: 13px; }
.tabs button[aria-pressed='true'] { background: #482035; border-color: #ff659f; }
.code { background: #0c0b0f; border: 1px solid var(--line); border-radius: 8px; overflow: hidden; margin: 12px 0; }
.codehead { display: flex; justify-content: space-between; align-items: center; gap: 8px; padding: 8px 12px; border-bottom: 1px solid var(--line); font: 12px monospace; color: var(--muted); }
.copy { background: transparent; border: 1px solid var(--line); border-radius: 4px; padding: 4px 9px; font-size: 12px; }
pre { margin: 0; padding: 16px; overflow: auto; color: #f6d5e5; font: 13px/1.65 monospace; white-space: pre-wrap; overflow-wrap: anywhere; background: none; }
code { font-family: monospace; color: #ffb4d0; background: none; padding: 0; font-size: 0.92em; }
table { width: 100%; border-collapse: collapse; margin-top: 20px; display: table; }
td, th { text-align: left; padding: 12px; border: 0; border-bottom: 1px solid var(--line); vertical-align: top; background: none; }
tr { background: none !important; border: 0; }
th { color: var(--muted); font-size: 12px; }
.grants { margin-top: 0; }
.grants td { padding: 8px 12px 8px 0; color: var(--muted); }
.ref { margin-top: 28px !important; }
details { padding: 16px 0; border-bottom: 1px solid var(--line); }
summary { cursor: pointer; font-weight: 600; }
details p { margin: 10px 0 0 !important; }
.stat { font-size: 40px; font-weight: 750; letter-spacing: -0.05em; }
.mk .note { margin-top: 18px; }
footer { padding: 36px 0; }
.foot { margin-top: 12px !important; }
.warning { border-left: 3px solid var(--amber); padding: 12px; background: #302219; color: #ffe1ae; font-size: 13px; margin-top: 14px; }
@media (max-width: 800px) {
  .hero-grid, .two, .builder { grid-template-columns: 1fr; }
  .grid { grid-template-columns: 1fr 1fr; }
  section { padding: 55px 0; }
  nav { gap: 12px; }
  .hero { padding-top: 60px; }
  .nodes { grid-template-columns: repeat(2, 1fr); }
}
@media (max-width: 480px) {
  .grid { grid-template-columns: 1fr; }
  nav a:not(.logo) { font-size: 12px; }
  nav .opt { display: none; }
  .wrap { padding: 0 16px; }
  td, th { padding: 8px; font-size: 12px; }
}
</style>
