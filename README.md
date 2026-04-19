# cosmux

> CoS-aware tmux pod manager — declarative YAML, lifecycle hooks. Built in Rust.

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.93%2B-orange.svg)](https://www.rust-lang.org/)

A single-binary tmux session manager with first-class lifecycle hooks. Spawn complex multi-pane workspaces from one YAML file. No Ruby, no Python, no Node — just `cosmux start <pod>`.

Built by [NXTG AI](https://nxtg.ai) to govern multi-project Claude Code workflows. Released as Apache-2.0 because the tmux ecosystem deserves better tooling.

---

## Why cosmux

- **Single binary, zero runtime deps** (~2 MB stripped)
- **Declarative YAML** — your workspace lives in version control
- **Lifecycle hooks** — `before_start`, `after_start`, `before_attach`, `after_detach`, `on_pane_dead`
- **Templates** — define `claude-team-pane` once, reuse across pods
- **No tmux-resurrect required** — your YAML is the source of truth

vs `tmuxinator` (Ruby), `tmuxp` (Python), `smug` (Go), and `Moxide` (Rust): cosmux is the only one with a portable hook model that runs arbitrary shell *and* knows about CoS-style workflows out of the box.

---

## Install

### From source

```bash
git clone https://github.com/nxtg-ai/cosmux.git
cd cosmux
cargo build --release
sudo install -m755 target/release/cosmux /usr/local/bin/cosmux
```

### From crates.io _(coming v1.0)_

```bash
cargo install cosmux
```

---

## Quickstart

1. Drop a pod YAML in `~/ASIF/infra/tmux/` or `~/.config/cosmux/pods/`:

```yaml
# pod-forge.yaml
name: pod-forge
root: "~/projects/NXTG-Forge"

before_start:
  - "git -C ~/projects/NXTG-Forge/forge-ui pull --rebase || true"

windows:
  - name: claude
    layout: tiled
    panes:
      - cwd: "~/projects/NXTG-Forge/forge-ui"
        command: ""
      - cwd: "~/projects/NXTG-Forge/forge-plugin"
        command: ""
      - cwd: "~/projects/NXTG-Forge/forge-orchestrator"
        command: ""
```

2. Spawn it:

```bash
cosmux start pod-forge
tmux attach -t pod-forge
```

3. Stop it:

```bash
cosmux stop pod-forge
```

---

## YAML schema

```yaml
name: <required, becomes the tmux session name>
root: "<optional, default cwd for panes that don't set their own>"
template: "<optional, name of a template under ~/.config/cosmux/templates/>"

before_start:  ["<shell command>", ...]   # fail = abort spawn
after_start:   ["<shell command>", ...]   # fail = warn, continue
before_attach: ["<shell command>", ...]   # before `cosmux start --attach` attaches
after_detach:  ["<shell command>", ...]   # (Phase 2: tmux hook integration)
on_pane_dead:  ["<shell command>", ...]   # (Phase 2: tmux hook integration)

windows:
  - name: <required>
    layout: tiled | even-horizontal | even-vertical | main-horizontal | main-vertical
    panes:
      - cwd: "<optional, falls back to root, then $PWD>"
        command: "<optional shell command sent via send-keys>"
        template: "<optional, per-pane template override>"
```

### YAML gotcha: bare `~` is NULL

In YAML, `~` is the null literal. **Always quote tildes:**

```yaml
root: "~"          # GOOD
cwd: "~/projects"  # GOOD
root: ~            # BAD — parses as null
```

---

## Commands

| Command | What it does |
|---|---|
| `cosmux start <pod>` | Spawn a pod (detached). Add `--attach` to attach. Add `--force` to replace existing. |
| `cosmux stop <pod>` | Kill the pod's tmux session. |
| `cosmux list` | List running tmux sessions. |
| `cosmux validate <pod>` | Parse and validate without side effects. |
| `cosmux show <pod>` | Print the resolved config (after template merge). |

Pod resolution order: explicit path → `~/ASIF/infra/tmux/<name>.yaml` → `~/.config/cosmux/pods/<name>.yaml` → `./<name>.yaml`.

---

## Templates _(Phase 1 partial — Phase 2 expands)_

Drop a template at `~/.config/cosmux/templates/<name>.yaml`:

```yaml
default_command: "ccyolo"
on_pane_dead:
  - "tmux send-keys -t {pane} 'cd {cwd} && ccyolo' Enter"
```

Reference it from a pod or pane:

```yaml
windows:
  - name: claude
    panes:
      - cwd: "~/project-a"
        template: "claude-team-pane"   # inherits default_command
```

Per-pane `cwd` and `command` always win over template defaults.

---

## Roadmap

- **v0.1** _(this release)_ — Core spawn, validate, stop, list, show. before_start / after_start hooks. Templates. **Status: shipped.**
- **v0.2** — Full hook lifecycle (before_attach, after_detach, on_pane_dead via tmux hook integration). 8 production pod YAMLs. Migration tooling.
- **v1.0** — HUD sidecar (`~/.cosmux/state.json`), `cargo install cosmux`, GitHub Releases, awesome-tmux entry.

---

## License

Apache-2.0 © NXTG AI

## Acknowledgements

Inspired by [Moxide](https://github.com/dlurak/moxide) (MIT) and [smug](https://github.com/ivaaaan/smug) (MIT). cosmux is an independent implementation, not a fork — designed from scratch around CoS-aware lifecycle hooks.
