<div align="center">

# Agent Hub

**English** · [简体中文](./README.zh-CN.md)

A web UI for your local CLI agents — **Claude Code** and **Codex**. A single Rust binary with zero Node dependencies. All model capability comes from the CLIs already installed and logged in on your machine — this project never talks to any model API directly; it simply gives both CLIs a modern graphical interface.

</div>

## Features

- **Dual CLI detection & streaming chat**: auto-detects local `claude` / `codex` installations (resolving the real `.exe` behind npm shims), drives them in headless mode, and renders text / thinking / tool calls from the NDJSON stream in real time
- **Browse & resume history**: reads both CLIs' native session storage (`~/.claude/projects/`, `~/.codex/sessions/`) directly — view any past transcript and `resume` it; fully interoperable with terminal sessions
- **Multi-project management**: discovers projects from history, import the ones you want into the sidebar; filter sessions by agent (All / Claude / Codex)
- **Background tasks**: refreshing or closing the page never interrupts a running task — reopen and it reconnects to the live output automatically; Stop terminates the complete SAGE workflow process tree and confirms completion; sidebar shows running ● / done ✓ / stopped ■ / failed ✕
- **Deployment-aware UI**: an open tab detects a restarted local backend, preserves its current session and draft, then automatically reloads the newly embedded frontend instead of continuing with stale JavaScript
- **SAGE smart routing**: builds a candidate pool from locally discovered `CLI + model` executors, automatically chooses team size, models, and per-requirement reasoning effort, then executes official `SELF / COLLABORATE / HANDOFF` assignments through requirement-DAG waves — serial within one executor, parallel across independent executors — with executor/effort/requirement evidence feedback; automatic effort for GPT-5.6 Sol is capped at `xhigh`, while Luna/mini/Spark-class economical models may reach `max` when supported
- **Codex Fast by default and by enforcement**: every Codex model invocation uses request-level `service_tier="fast"`; the UI shows the switch as always on and does not allow it to be disabled
- **Skills / command palette**: type `/` to open — aggregates Claude skills (user / project / plugins), Codex skills & custom prompts, and built-in commands (`/review`, `/init`, `/diff`, `/status`, `/fork`…); keyboard navigation; per-skill color coding
- **Full image pipeline**: paste screenshots into the composer to send to the CLI; historical images render inline; click for a zoomable lightbox
- **Edited-files card**: each turn summarizes changed files (collapsed by default); click a row for a GitHub-style diff review (line numbers, full-row coloring, nested git repos and committed-change fallback supported); right-click to open in VS Code / File Explorer
- **Clickable file references**: paths and markdown links in transcripts open directly; code files jump to the exact line (`file.rs:100` → VS Code)
- **Full Markdown rendering + sandboxed HTML preview**: headings / lists / tables / quotes rendered live; `html` code blocks get a one-click preview in a sandboxed iframe
- **Light / dark themes**; every choice (model, reasoning effort, permissions, project…) is remembered locally
- **Auto-discovered models & reasoning levels**: read from the CLIs' own configs and history instead of being hardcoded

## Requirements

| Dependency | Notes |
|---|---|
| Windows 10/11 | Current implementation targets Windows (process resolution / path handling) |
| Rust stable | For building; install via `rustup` |
| [Claude Code CLI](https://claude.com/claude-code) | ≥ 2.x, logged in |
| [Codex CLI](https://github.com/openai/codex) | ≥ 0.148, logged in |
| Python 3.10+ (optional) | Required by SAGE routing; the feature disables itself if missing |
| git (optional) | Required for diff review / change stats |
| VS Code (optional) | For click-to-open with line jumping; falls back to the system default app |

Having either CLI installed is enough to start — the UI shows the install status of each.

## Quick start

```bash
git clone https://github.com/q520asdf0123/agent-hub.git
cd agent-hub
cargo run
```

Open http://127.0.0.1:8721 (override the port with the `AGENT_HUB_PORT` environment variable).

## How it works

```
Browser (embedded static SPA)
   │ REST + NDJSON stream
   ▼
axum backend (127.0.0.1:8721)
   ├─ Run registry: tasks decoupled from HTTP connections —
   │  disconnecting never kills the process; reconnect or stop explicitly
   ├─ claude.exe -p --output-format stream-json … (prompt via stdin, resume via --resume)
   ├─ codex.exe exec --json … (resume via exec resume / fork via exec fork)
   └─ History readers: parse both CLIs' JSONL session stores
      (mtime-based incremental index + TTL cache)
```

Key design decisions:

- **Spawn the real `.exe`, never the `.cmd` shim** (Windows `CreateProcessW` can't execute .cmd, and forwarding user input through cmd risks metacharacter injection); prompts are always passed via stdin
- Sessions persist in each CLI's native format, so **everything interoperates with the terminal**: a session started in the web UI can be continued with `claude --resume` in a terminal, and vice versa
- `~/.claude` and `~/.codex` are treated as **read-only** — nothing is written or migrated

## Project layout

```
src/
├─ main.rs        # routes & startup
├─ api.rs         # REST handlers (sessions/projects/skills/models/diff/open…)
├─ run.rs         # background run registry + CLI stream event mapping
├─ cli.rs         # CLI detection (shim → real exe resolution)
├─ history/       # claude / codex native session store parsers
├─ models.rs      # model & reasoning-level auto-discovery
├─ skills.rs      # skills / command scanning
└─ sage.rs        # SAGE routing bridge (Python subprocess)
static/           # frontend (vanilla JS, embedded into the binary at compile time)
├─ index.html
├─ app.js
├─ sage-scheduler.js      # testable SAGE requirement-DAG wave scheduler
└─ style.css
vendor/sprix-sage-router/   # SAGE algorithm library (MIT, LICENSE included)
docs/             # technical plan (PLAN.md) and module contract (CONTRACT.md)
```

## Configuration

- `~/.agenthub/config.json`: imported project list (maintained via the UI)
- Skill sources: `~/.claude/skills/`, project `.claude/skills/`, installed plugins, `~/.codex/skills/`, `~/.codex/prompts/`
- SAGE capability profiles: `DEFAULT_PROFILES` in `vendor/sprix-sage-router/sage_bridge.py`, tune to taste
- SAGE learning state: `~/.agenthub/sage_state.json`; execution-semantics or model-pool schema upgrades back up and migrate the prior state automatically

## Security notes

- The server binds to `127.0.0.1` only — never exposed to the network
- All transcript content renders as plain text (XSS-safe); HTML preview runs in a sandboxed iframe with no same-origin access
- No telemetry, no external requests; model calls, credentials and quotas all belong to your local CLIs

## Credits

- Routing algorithm from [Sprix AI's sprix-sage-router](https://github.com/wang2122/sprix-sage-router) (MIT, vendored with its original LICENSE)
- UI style inspired by Codex Desktop / Tutti
