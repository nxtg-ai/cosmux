# cosmux — Claude Code Project Guide

## What This Is

`cosmux` is a CoS-aware tmux pod manager — a single Rust binary that turns tmux sessions into declarative "pods" with first-class lifecycle hooks. Built by NXTG AI as the third public Rust artifact in the portfolio (after Forge orchestrator and Faultline-Pro). Apache-2.0.

**Repo**: `nxtg-ai/cosmux` (public on GitHub).
**Local path**: `~/projects/cosmux/`.
**ASIF NEXUS**: `~/projects/cosmux/.asif/NEXUS.md`.
**Portfolio ID**: P-18.

## Vision

Make tmux session sprawl extinct. Replace bash aliases + tmux-resurrect + manual layout commands with one YAML file per workspace. Lifecycle hooks (`before_start`, `after_start`, `before_attach`, `after_detach`, `on_pane_dead`) make cosmux strictly more powerful than tmuxinator/tmuxp/smug while staying just as simple.

## Architecture

```
src/
├── main.rs       — clap CLI: start, stop, list, ps, validate, show, state, gc
├── config.rs     — YAML PodConfig parsing + validation + path resolution
├── tmux.rs       — tmux IPC (new-session, split-window, send-keys, set-hook)
├── hooks.rs      — sh -c invocation for before_start / after_start / etc.
├── templates.rs  — `~/.config/cosmux/templates/` merge into pod panes
├── state.rs      — `~/.cosmux/state.json` HUD writer
├── recover.rs    — _pane-recover subcommand (called by tmux pane-died hook)
└── error.rs      — typed errors
```

## How a Spawn Works

1. `cosmux start <pod>` → resolve pod path (search `~/ASIF/infra/tmux/`, `~/.config/cosmux/pods/`, `./`).
2. Load YAML → validate → merge templates.
3. Run `before_start` hooks (fail aborts).
4. `tmux new-session -d -s <name>` for window 1, then `split-window` for additional panes.
5. For each window 2+: `tmux new-window` + splits.
6. Install tmux session hooks: `pane-died` → `cosmux _pane-recover <name>`, `client-detached` → `cosmux _after-detach <name>`. Set `remain-on-exit on` so dead panes can be detected.
7. Write `~/.cosmux/state.json` with full pod state (windows, panes, cwd, command, hooks).
8. Run `after_start` hooks (fail warns).

## How Pane Recovery Works

1. Pane process exits.
2. tmux fires the `pane-died` hook installed by cosmux.
3. Hook runs `cosmux _pane-recover <session>` (a hidden subcommand).
4. recover queries tmux for panes where `#{pane_dead} == 1`.
5. For each dead pane: look up its original cwd + command in `state.json`, then `respawn-pane -k -t <target> -c <cwd>` + `send-keys <command>` + Enter.
6. Run pod's `on_pane_dead` hooks (e.g., log to file, voice announce).

## YAML Schema (canonical)

```yaml
name: <required, becomes tmux session name>
root: "<optional, default cwd for panes that don't set their own>"
template: "<optional, name under ~/.config/cosmux/templates/>"

before_start:  ["<sh command>"]   # fail = abort
after_start:   ["<sh command>"]   # fail = warn
before_attach: ["<sh command>"]
after_detach:  ["<sh command>"]
on_pane_dead:  ["<sh command>"]

windows:
  - name: <required>
    layout: tiled | even-horizontal | even-vertical | main-horizontal | main-vertical
    panes:
      - cwd: "<optional, falls back to root, then $PWD>"
        command: "<optional shell command sent via send-keys>"
        template: "<optional per-pane template override>"
```

**YAML gotcha**: bare `~` is YAML null. ALWAYS quote: `cwd: "~"`, `root: "~/path"`. Don't write `cwd: ~`.

## Live Production Pods (NXTG-AI)

cosmux currently manages 3 production tmux sessions on this machine:

- `Forge` (1 window × 4 panes) — `~/ASIF/infra/tmux/Forge.yaml`. Forge Program Lead + 3 sub-repos (forge-ui, forge-plugin, forge-orchestrator).
- `WORKSTREAMS` (6 windows × 10 panes) — `~/ASIF/infra/tmux/WORKSTREAMS.yaml`. Tier-1 (CE/nxtg.ai), Tier-2 (PP/dx3), Tier-3 (synapps/VJJ), Tier-4 (FP/fpw4-builder), dx3, dx3-api.
- `Dx3_Program` (1 window × 1 pane) — `~/ASIF/infra/tmux/Dx3_Program.yaml`. dx3 active claude session.

Five additional pod YAMLs are staged for future use: `cos-wolf`, `cos-emma`, `pod-forge`, `pod-dx3`, `pod-faultline`, `pod-content`, `pod-voice-media`, `pod-infra`. These represent a cleaner project-aligned topology for a future redesign; current production pods mirror the existing heartbeat target structure.

## Coupling

- **Heartbeat** (`~/ASIF/scripts/cos-heartbeat-nxtg.sh`): injects governance prompts into specific tmux pane targets (e.g., `WORKSTREAMS:Tier-1.1`). cosmux is responsible for keeping those panes alive with the right cwd; heartbeat is responsible for what gets sent to them. Boundaries clean.
- **tmux-continuum**: `set -g @continuum-restore 'off'` in `~/.tmux.conf` (already off). cosmux is now the source of truth for "what should exist on boot"; users explicitly `cosmux start <pod>` rather than relying on auto-restore.
- **DX3 canon**: heartbeat reads desired topology from DX3 (`canon`, `desired-topology`, `NXTG-AI` tags) at startup with the hardcoded TEAMS array as fallback. cosmux pod YAMLs should stay consistent with DX3 canon.

## Build / Run

```bash
cargo build --release            # 2.1 MB binary at target/release/cosmux
cargo test                       # currently 0 tests (CI smoke covers validate)
cargo clippy --all-targets -- -D warnings   # zero warnings
cargo fmt --all                  # always before commit
```

## Versioning

SemVer. Breaking YAML changes bump major. New CLI subcommands or hook types bump minor. Bug fixes / non-breaking polish bump patch. Tag every release: `git tag -a vX.Y.Z -m "..."`. Update CHANGELOG.md before tag.

Current: **v0.3.0** (live in production). Path to **v1.0**: cargo publish + Show HN + cargo install instructions + (optional) prebuilt binaries.

## Standing Order

If you (Claude) are working on cosmux:
1. Always run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo build --release` before declaring done.
2. Don't add features beyond the YAML schema without ADR.
3. Don't break the `cosmux start <pod>` -> `tmux attach -t <pod>` quickstart — it's the readme promise.
4. Test pane-recovery before any release: spawn a sandbox pod with `on_pane_dead`, kill a pane, verify respawn.
5. Keep the binary single-purpose: tmux pod manager. No web UI, no plugin runtime, no daemon mode (yet).

## NXTG-Forge

This project does NOT use NXTG-Forge governance — it's intentionally minimal scope (one binary, no plugin runtime, narrow surface). Lighter touch via this CLAUDE.md + .asif/NEXUS.md is the right fit.
