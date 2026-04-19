# Changelog

All notable changes to cosmux are documented here. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning: [SemVer](https://semver.org/).

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
