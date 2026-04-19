# Changelog

All notable changes to cosmux are documented here. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning: [SemVer](https://semver.org/).

## [0.3.0] — 2026-04-19

Phase 3 — Dogfood polish. cosmux now governs the live NXTG-AI portfolio
(Forge, WORKSTREAMS, Dx3_Program — all migrated from legacy tmux + 27-pane mess).

### Added
- `cosmux ps` — lists only cosmux-managed pods with status (alive/stale) and source path.
  `cosmux list` still shows ALL tmux sessions; `ps` is the managed-only view.
- `cosmux gc` — prunes state.json entries whose tmux session no longer exists.
  Useful after manual `tmux kill-session` or sandbox cleanup.

### Migration notes
- Live portfolio cutover completed Sun 2026-04-19 PDT: 3 legacy sessions
  (Forge, WORKSTREAMS, Dx3_Program) replaced by cosmux-managed equivalents.
  Pod YAMLs at `~/ASIF/infra/tmux/*.yaml` mirror existing heartbeat target
  topology (zero heartbeat config changes required for cutover).
- Stale sessions (FPW, haiku-overclock) killed during migration.

## [0.2.0] — 2026-04-19

Phase 2 — Full hook lifecycle, HUD state, hidden recovery commands.

### Added
- **HUD state file** `~/.cosmux/state.json` — written on every `start`/`stop`. Records pod source path, window/pane layout, command, cwd, hooks. Inspect with `cosmux state`.
- **`after_detach` hook** — wired to tmux `client-detached` via `set-hook`.
- **`on_pane_dead` hook** — wired to tmux `pane-died` via `set-hook` + `remain-on-exit on`. Auto-respawns the pane in place via `respawn-pane -k` using the recorded cwd + command.
- Hidden subcommands `_pane-recover` and `_after-detach` (used by tmux hooks; not for direct invocation).
- `cosmux state` (alias `cosmux hud`) — prints the HUD state file path + JSON contents.
- `serde_json` dependency.

### Notes
- The recovery loop relies on `state.json` being present. If you delete it manually, hook recovery becomes a no-op (cosmux logs a warning).
- `tmux set-hook ... pane-died` is best-effort — older tmux versions ignore the call silently.

## [0.1.0] — 2026-04-19

Initial release. Phase 1 MVP.

### Added
- Single-binary CLI (`cosmux start | stop | list | validate | show`).
- Declarative YAML pod configs (`PodConfig`, `Window`, `Pane`).
- `before_start` and `after_start` lifecycle hooks (sh -c).
- Template scaffolding (`~/.config/cosmux/templates/`) with `default_command` merge.
- Pod resolution order: explicit path → `~/ASIF/infra/tmux/` → `~/.config/cosmux/pods/` → `./`.
- `--attach`, `--force`, `--verbose` flags.
- pod-forge sample config (3-repo Forge workspace).

### Known limitations
- `before_attach`, `after_detach`, `on_pane_dead` parsed but not yet wired (Phase 2).
- No HUD state emission yet (Phase 3).
- YAML bare `~` parses as null — quote tildes (documented in README).
