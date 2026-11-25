<p align="center">
  <img src="./assets/brand/social-preview.png" alt="Mikata 味方: the coding agent on your side. Its mark, a living cell flowing into a digital one" width="100%">
</p>

# Mikata

**The coding agent on your side** (味方, *mikata*: ally). You give it an ask; one body does the
work and checks it before it answers. Every change then passes the end gate: your verify line
runs green, and no test is deleted or skipped. Say what done means in the ask and the harness
holds each line; ask for a plan (`--profile planned`) and the body writes what done means down
before any code. The plan is off by default: on our last screen it earned no pass and cost up to
a sixth of the calls.

On our bench, ten real fixes in a large Python library, with the same small open model in every
harness, run twice a week apart ([the bench](docs/bench.md)):

| Harness | Passed | Per pass |
|---|---|---|
| **Mikata** | **16 of 20** | **$0.50** |
| Maki | 15 of 20 | $0.91 |
| OpenCode | 14 of 20 | $0.87 |

On passes the three are level: by the bar we wrote first, no difference is shown. On a second
codebase (ten fixes in a JavaScript library) they are level again, 8, 7 and 6 of 10, and with a
stronger model every one of them passes all ten. Mikata is not always the cheapest: on the
JavaScript fixes, which are smaller, Maki's pass cost a third of Mikata's, and with the stronger
model half.

**Say what done means, and the harness holds it.** Put lines under `## Done when` in your ask
(`D1: ... must give ...`, `D2: ... must stay as it is today`), or name a reader, a model that
writes them from the ask alone. Mikata hands the harness one command a line. The harness reads
what must stay from your code before anything changes, and runs every line when the answer
lands. On the two asks our bench fails most, stated as prose: 1 pass in 6 without a list, 6 in 6
with one held. With a reader named and nothing added to the ask, four prose asks passed 10 times
in 12 on the release, where they passed 6 without one. The list is what does it: handed the same
lists as a plain note, Mikata passed 11, and OpenCode and Maki 8 each where they had passed 5 and
3. What the hold adds is a pass that names what verified it; it is not shown to add passes.

**An ask that spans several projects is split by itself.** Name two or more folders that each hold
a project of their own, and Mikata runs one body a project, at once, then checks each. Five whole
repositories in one ask: one body fixed 2.5 of 5 on average; one body a project, 4.5 of 5.

**It acts outside only with your word.** It reads and writes inside your repository, every command
runs in a sandbox, and an act that leaves your machine (a push, a pull request, a publish) is put to
you first, every time.

**It shows you what changed, by its own checks.** When an answer lands, Mikata's last checks are
run again on the code as you gave it and on the finished work, and you see where they print
differently. Its checks hold its reading of your ask; you are the one who knows whether a line
that changed should have.

| Read | For |
|---|---|
| [Overview](docs/01-overview.md) | what Mikata does with an ask, in one picture |
| [The loop](docs/02-loop.md) | one turn, budgets, the stress instruments, resume, the end gate |
| [Architecture](docs/03-architecture.md) | crates, the terminal UI, the board, storage, sandbox, configuration |
| [The client protocol](docs/04-protocol.md) | how the SDK and the board speak to `mikatad` |
| [Consumption](docs/05-consumption.md) | how every token is budgeted, metered and stopped |
| [MCP](docs/06-mcp.md) | MCP servers: declared, scoped, priced, ranked |
| [Memory](docs/07-memory.md) | the archive, notes, recall, the write rule |
| [The bench](docs/bench.md) | what we measured, against whom, and every screen since |
| [Roadmap](docs/roadmap.md) | what comes next |
| [Lessons](docs/lessons.md) · [Releasing](docs/releasing.md) | what we learned; how a version ships |
| [Experimental: the lab](docs/experimental/) | the races, the chain and the collective: Mikata's past as Mnemorph, kept and frozen |

## Install

**With the installer** (Linux x86_64 and arm64, macOS Apple Silicon and Intel):

```sh
curl -fsSL https://raw.githubusercontent.com/Kaginari/mikata/main/install.sh | sh
sh install.sh --version 1.0.0 --dir ~/bin    # a given release, somewhere else
```

It downloads the release's archive for this machine, refuses it unless it matches the release's
`SHA256SUMS`, and puts the binaries in `~/.local/bin` (`MIKATA_INSTALL_DIR` or `--dir` to
choose). It needs no sudo and edits no shell profile: when the directory is not on PATH, it prints
the line to add. Once a release carries build provenance ([releasing](docs/releasing.md#provenance-signing)),
an archive can also be checked against the workflow that built it:
`gh attestation verify mikata-<target>.tar.gz --repo Kaginari/mikata`.

**By hand**, from [the releases](https://github.com/Kaginari/mikata/releases): download
`mikata-<target>.tar.gz` and `SHA256SUMS`, check, unpack, and put the binaries side by side on
PATH:

```sh
sha256sum -c --ignore-missing SHA256SUMS     # macOS: shasum -a 256 -c --ignore-missing SHA256SUMS
tar -xzf mikata-x86_64-unknown-linux-gnu.tar.gz
cp mikata-*/mikata mikata-*/mikatad mikata-*/mikata-guard ~/.local/bin/   # no guard on macOS
```

**From source**, in a checkout:

```sh
cargo install --path crates/mikata-tui      # mikata, into ~/.cargo/bin
cargo install --path crates/mikata-sandbox  # mikata-guard, beside it (Linux)
pnpm install --frozen-lockfile && pnpm --filter @mikata/board build   # the board, built into…
cargo install --path crates/mikatad         # mikatad: the server, the board and the admin commands
```

Three binaries, one core:

| Binary | For | Use it to |
|---|---|---|
| `mikata` | you, at a terminal, and your scripts | a session in the terminal app (`mikata`), one ask without it (`mikata run "<ask>"`), pick up a run that stopped (`mikata resume last`), check the setup (`mikata status`) |
| `mikatad` | other programs, and the world's upkeep | serve the [client protocol](docs/04-protocol.md) to the TypeScript SDK, the web board on loopback (`mikatad board`), and the admin commands: `runs`, `memory`, `q`, `incident`, `housekeep` |
| `mikata-guard` | Linux | the kernel guard every command runs through, found beside `mikata`; without it, on Linux, `mikata` runs no command and `mikata status` says so |

`mikatad` embeds the board as it finds it built in `packages/board/dist` when it is compiled;
compiled without it, it serves a page saying so, and its API.

**Coming from Mnemorph?** The first time `mikata` or `mikatad` runs, a world's `.mnemorph/`
folder and the `mnemorph` folders under your config and state homes are moved to their Mikata
names, and the move is said. The `MNEMORPH_*` environment variables are no longer read: use
`MIKATA_*`.

## Set a model

Name a provider and a model once, for every world, in `~/.config/mikata/config.toml`
(`$XDG_CONFIG_HOME/mikata/config.toml`). A world's `.mikata/config.local.toml` overrides it,
except for providers, caps, MCP servers and anything that loosens the gate or the sandbox, which
only this global file sets ([configuration](docs/03-architecture.md#configuration)):

```toml
[providers.openrouter]
kind = "openai"
base_url = "https://openrouter.ai/api"
key_env = "OPENROUTER_API_KEY"     # the variable that holds the key (never write the key here)
key_file = "~/.config/mikata/openrouter.key"  # used when the variable is unset; chmod 600

[models]
provider = "openrouter"
default = "anthropic/claude-sonnet-5.5"
```

`key_file` is read only from this global file, never from a world's config. The file must be
yours, mode `600`, and outside every world.

**Give it your test command.** The end gate runs your project's verify line on every change.
Until a world has one, nothing a session claims is grounded, and `mikatad memory status` suggests
the line for your project (`cargo test --workspace`, `pytest`, `npm test`, …).

**Say when it was wrong.** `mikatad memory wrong "<what still fails>"` keeps your words as a
correction under that ask: the next session that meets it is told first. Say what must come out,
as ``mikatad memory wrong "`total 1 2` must give `6`; it gives `3`"``, and the harness holds it:
whenever that ask comes again the case is a line of its `Done when` list, and an answer that
leaves it wrong does not land. `mikatad memory withdraw <note>` takes a correction back.
Add `--check "<command>"` to name the command that shows your case: the harness then holds the
line to what your command prints, and no command the model writes stands in for it. In a session,
type `/wrong <what is wrong>` after an answer (with `--check <command>` the same way): it shows whether your words will be
held or kept as advice, and under which ask, and keeps them only on `/keep` (`/drop` keeps
nothing). `/withdraw` takes back the last one you kept.

## Run it

```sh
export OPENROUTER_API_KEY=…         # or keep it in the key_file
mikata                            # a session in this world, in the terminal app
mikata --world ~/code/project     # …in another
mikata status                     # the mode, the model, the key's origin, the sandbox, the config's holes
mikata run "<ask>"                # one ask without the app: the answer on stdout
echo "<ask>" | mikata run --json  # every event as a JSON line, then a result line
mikata resume last                # pick up the newest run that stopped
mikatad runs                      # the runs that may be resumed
mikatad board                     # the web board on 127.0.0.1:7878: open the URL it prints
mikata --web                      # the terminal app, with the board started and opened
```

`mikata run` goes through the same session and end gate as the app, and exits 0 only when the
answer landed; it denies every act that needs your approval unless you pass `--gate-tty`
([`mikata run`](docs/03-architecture.md#mikata-run)). From a checkout, `cargo run -p mikata-tui --`
runs the same binary.

### The terminal app

<img src="docs/img/tui/dashboard-dark.png" width="820" alt="The terminal app: a header bar, the Checks, Guardrails and Units panels, the agent loop with two-row tool cards, Done when, Budgets and Session, a footer bar">

A dashboard round the conversation: what the end gate checks and holds, the budgets, the units on
and off, the files and bodies at work. A tool card is two rows until you open it (a click or
Ctrl+T). **F2** (or `/view center|focus|zen`) switches to the conversation with its bars, or to
the conversation alone; Ctrl+O hides the panels
([the terminal app](docs/03-architecture.md#the-terminal-app)).

**Say what the result must be.** An ask with examples of the expected output passed twice as often
on our bench as the same ask in prose: Mikata checks its work against what you wrote, so one
example of what must change and one of what must not is worth more than a paragraph.

### The board

`mikatad board` prints `http://127.0.0.1:7878/#k=<key>`; the key works once, traded by the page for
a session cookie. It listens on loopback only: from another machine, forward the same port
(`ssh -L 7878:127.0.0.1:7878 <host>`), then open the printed URL. A second `mikatad board` prints
the running board's URL, with a new key waiting. The board answers at `127.0.0.1`, `[::1]` and
`localhost` and refuses every other host name, to block DNS rebinding. `mikatad board stop` stops
it ([the board](docs/03-architecture.md#the-board)).

It is every world on the machine in one page: chats in the terminal's grammar, the approval card
with the terminal's protections, memory and metrics, live. ⌘K (Ctrl+K) jumps anywhere; `?` lists
every shortcut.

| | Dark | Light |
|---|---|---|
| Chat | <img src="docs/img/board/chat-dark.png" width="400" alt="A chat, dark: tool cards, a numbered diff, the end gate's verdict, the session pane"> | <img src="docs/img/board/chat-light.png" width="400" alt="A chat, light"> |
| Approval | <img src="docs/img/board/approval-dark.png" width="400" alt="An approval card, dark: the act whole with every hidden character escaped, Yes disabled until read"> | <img src="docs/img/board/approval-light.png" width="400" alt="An approval card, light"> |
| Metrics | <img src="docs/img/board/metrics-dark.png" width="400" alt="The metrics page, dark: totals, spend per day, cache hit, burn"> | <img src="docs/img/board/metrics-light.png" width="400" alt="The metrics page, light"> |

**The Harness Builder** (`#/w/<world>/builder`, first under each world) composes a session's units
with switches, starts a run with exactly that harness and follows it live: its stages, its log,
the end gate's word, its spend, its approvals.

<img src="docs/img/board/builder-dark.png" width="820" alt="The Harness Builder: the units as cards round the core, the composition as TOML, the run live on the right with its stages, log and the end gate's verdict">

## Build it

| Tool | Version | Install |
|---|---|---|
| Rust | pinned in `rust-toolchain.toml` | [rustup](https://rustup.rs): the pin installs itself |
| cargo-nextest, cargo-deny, cargo-llvm-cov | latest | prebuilt: [nexte.st](https://nexte.st), [cargo-deny releases](https://github.com/EmbarkStudios/cargo-deny/releases) |
| just | latest | [just.systems](https://just.systems) |
| actionlint | latest | [releases](https://github.com/rhysd/actionlint/releases) |
| Node | 22 (`.nvmrc`) | your usual Node manager |
| pnpm | pinned in `package.json` | `corepack enable` |

```sh
just setup     # install the TypeScript workspace
just           # the full check: Rust, TypeScript, the layering, the book, the workflows
just docs-dev  # read the book as a website while writing it
```

| Path | Holds |
|---|---|
| `crates/` | the Rust core, one crate per concern, layered ([architecture](docs/03-architecture.md)) |
| `packages/` | the TypeScript side: protocol, sdk, theme, board |
| `docs/` | the book; `docs/experimental/` is the lab |
| `site/` | the book as a website (VitePress) |
| `assets/` | Mikata's logo and icons; the races' portraits and cards |
| `.mikata/` | Mikata's own world: its law, journal and the lab's creature cards |
| `tools/` | repository checks |

What changed in each version: [the changelog](CHANGELOG.md). Mikata is MIT licensed
([LICENSE](LICENSE)); what it bundles: [third-party notices](THIRD-PARTY-NOTICES.md).
