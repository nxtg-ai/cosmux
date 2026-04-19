# NEXUS — cosmux Vision-to-Execution Dashboard

> **Owner**: Asif Waliuddin (NXTG AI)
> **Repo**: `nxtg-ai/cosmux` (https://github.com/nxtg-ai/cosmux)
> **Last Updated**: 2026-04-19
> **North Star**: Declarative tmux pod manager that owns multi-agent workspaces — single Rust binary, zero runtime deps, lifecycle hooks built in.

---

## Executive Dashboard

| ID | Initiative | Pillar | Status | Priority | Last Touched |
|----|-----------|--------|--------|----------|-------------|
| N-01 | Phase 1 MVP — clap CLI + YAML pod parser + tmux IPC | CORE | SHIPPED | P0 | 2026-04-19 |
| N-02 | Phase 1 — before_start / after_start hooks | HOOKS | SHIPPED | P0 | 2026-04-19 |
| N-03 | Phase 1 — pod-forge sample + end-to-end validation | DOGFOOD | SHIPPED | P0 | 2026-04-19 |
| N-04 | Phase 2 — full hook lifecycle (after_detach, on_pane_dead) | HOOKS | SHIPPED | P0 | 2026-04-19 |
| N-05 | Phase 2 — HUD state file (`~/.cosmux/state.json`) | HUD | SHIPPED | P1 | 2026-04-19 |
| N-06 | Phase 2 — pane auto-respawn via `respawn-pane -k` (verified live) | RECOVERY | SHIPPED | P0 | 2026-04-19 |
| N-07 | Phase 2 — 8 portfolio pod YAMLs + `claude-team-pane` template | DOGFOOD | SHIPPED | P1 | 2026-04-19 |
| N-08 | Phase 2 — migration script (snapshot, kill legacy, spawn pods) | OPS | SHIPPED | P1 | 2026-04-19 |
| N-09 | Phase 3 — `cosmux ps` + `cosmux gc` (managed-only listing + state cleanup) | DX | SHIPPED | P1 | 2026-04-19 |
| N-10 | Phase 3 — LIVE PORTFOLIO CUTOVER (Forge + WORKSTREAMS + Dx3_Program) | DOGFOOD | SHIPPED | P0 | 2026-04-19 |
| N-11 | Phase 3 — CI workflow (fmt + clippy + build + test + smoke) | QUALITY | SHIPPED | P1 | 2026-04-19 |
| N-12 | v1.0 prep — Show HN + Reddit + LinkedIn + Twitter drafts | LAUNCH | DRAFT | P1 | 2026-04-19 |
| N-13 | v1.0 — `cargo publish` to crates.io (name `cosmux` available) | DISTRIBUTION | PENDING-ASIF | P1 | 2026-04-19 |
| N-14 | v1.0 — `awesome-tmux` PR | LAUNCH | PENDING | P2 | -- |
| N-15 | v0.4 — `cosmux reload <pod>` (graceful re-spawn without context loss) | DX | PENDING | P2 | -- |
| N-16 | v0.4 — Strict-template mode + template inheritance | TEMPLATES | PENDING | P2 | -- |
| N-17 | v0.4 — CoS-aware `before_attach`/`after_detach` test coverage | QUALITY | PENDING | P2 | -- |

**Status legend**: SHIPPED · DRAFT · PENDING · PENDING-ASIF (awaiting decision) · BLOCKED

---

## Pillars

### CORE — the binary itself
- Rust 2021, ~1100 LOC across 7 modules (`main`, `config`, `tmux`, `hooks`, `templates`, `state`, `recover`, `error`).
- Single static binary. Stripped release ~2.1 MB. Zero runtime deps beyond `tmux` itself.
- clap 4 derive CLI, env_logger, serde_yaml, serde_json, anyhow + thiserror, shellexpand for `~` expansion.

### HOOKS — the differentiator
- `before_start` — fail aborts spawn (used for `git pull`, dependency checks, voice announcements).
- `after_start` — fail warns + continues (used for HUD writes, post-boot notifications).
- `before_attach` — runs before `cosmux start --attach` attaches (recall context, etc.).
- `after_detach` — wired via tmux `client-detached` set-hook.
- `on_pane_dead` — wired via tmux `pane-died` set-hook + `remain-on-exit on`. **Cosmux installs an internal `_pane-recover` subcommand that the hook calls; recover reads `state.json` and uses `respawn-pane -k` to bring the pane back with original cwd + command.**

### HUD — observable state
- `~/.cosmux/state.json` written on every spawn/stop. Schema: pods → status, started_at, source_path, windows[panes[index, cwd, command]], on_pane_dead[], after_detach[].
- `cosmux state` (alias `cosmux hud`) prints the file.
- `cosmux ps` shows alive/stale rollup. `cosmux gc` prunes stale entries.
- P-17 ASIF Dashboard will consume this for the "live pod view" (planned).

### RECOVERY — the killer feature
- Pane dies → tmux fires `pane-died` hook → cosmux `_pane-recover` reads state.json → finds dead pane via `#{pane_dead}` query → `respawn-pane -k -t <target> -c <cwd>` → `send-keys` original command + Enter.
- **Verified live** during dogfood: killed bash pane in pod-forge, came back as bash in correct cwd. Killed sandbox panes twice back-to-back, recovered both times.

### DOGFOOD — proof of value
- 2026-04-19 PDT: portfolio cutover. 6 sessions / 27 panes (with dupes + stale FPW + stray haiku-overclock) → 4 sessions / 16 clean panes (3 cosmux-managed + my COS_WOLF). Heartbeat verified scanning new sessions correctly (DORMANT detection + SAFETY_VALVE re-injection).

---

## CoS Directives

> No active directives. cosmux is Wolf-led; Asif sets strategic direction; no team has been spun up yet.

---

## Portfolio Intelligence

### PI-01 — Heartbeat coupling (2026-04-19)
The migration succeeded with zero heartbeat config changes by mirroring existing target topology in pod YAMLs (`WORKSTREAMS:Tier-1.1` etc.). This pattern — "cosmux owns session topology, heartbeat owns governance prompts" — generalizes to any tool that injects into specific tmux targets. Future v1.x migrations should follow the same playbook: read target keys, mirror in YAML, cut over with no governance pipe disruption.

### PI-02 — `cwd: ~` YAML gotcha (2026-04-19)
YAML treats bare `~` as null. Always quote: `cwd: "~"`, `root: "~/path"`. Documented in README + CHANGELOG. This bit us during early validation; preserved as a teaching moment for new users.

### PI-03 — Heartbeat respawn race (2026-04-19)
`tmux kill-session` followed by `cosmux start <same-name>` raced with heartbeat's auto-respawn logic — heartbeat detected the missing pane and respawned the OLD topology before cosmux could create the new one. Fix: pause heartbeat during migration, then resume. Documented in `~/ASIF/scripts/cosmux-migrate.sh` flow.

### PI-04 — `git pull --rebase || true` is fine (2026-04-19)
before_start hooks running `git pull --rebase` against dirty trees print stderr but exit 0 (because of `|| true`). This was misread as a failure during cutover; actually cosmux completed correctly. Lesson: distinguish stderr noise from exit codes.

---

## Team Questions

> No team yet. Asif is the only stakeholder. Wolf executes.

---

## Roadmap

### v0.4 (next, ~1 week)
- `cosmux reload <pod>` — restart without losing session/window structure (re-runs commands, preserves layout).
- Strict-template mode (fail if referenced template missing — currently warns).
- Window-level lifecycle hooks (per-window before_start/after_start in addition to pod-level).

### v0.5
- Multi-machine pod awareness (a pod YAML can declare which machine it belongs to; cosmux refuses to spawn on the wrong machine).
- `cosmux watch` — long-running daemon that emits events to a Unix socket (for HUD live updates without polling state.json).

### v1.0
- `cargo install cosmux` (publish to crates.io).
- GitHub Releases with prebuilt binaries (linux-x86_64 + macOS-arm64).
- Show HN / Reddit / awesome-tmux PR launch sequence.

### v2.0 (eventual)
- Plugin system if community asks (current stance: no — single binary, no plugin runtime).
- Web UI (separate project, consumes HUD state.json).

---

## Decision Log
- 2026-04-19 13:49 PDT — Asif: "A + make it our own.. rewrite for 100% IP ownership." → ADR-034.
- 2026-04-19 13:54 PDT — Asif: "compact before plan/implement" + "all phases at once".
- 2026-04-19 14:55 PDT — Asif: "let's dogfood this to 100% perfection and replacement what we have going on right now with these crazy tmux sessions" → live portfolio cutover (Forge + WORKSTREAMS + Dx3_Program).
- 2026-04-19 (pending) — `cargo publish` to crates.io: defaulted HOLD until post-FP Show HN.

---

## Related
- ADR-034: `~/ASIF/decisions/ADR-034-tmux-pod-manager-own-IP-rust.md`
- Plan: `~/ASIF/enrichment/2026-04-19-cosmux-build-plan-all-phases.md`
- Research: `~/ASIF/enrichment/2026-04-19-tmux-sota-pod-architecture-report.md`
- Product brief (this release): `~/ASIF/enrichment/2026-04-19-cosmux-product-brief.md`
- Pod YAMLs: `~/ASIF/infra/tmux/`
- Migration script: `~/ASIF/scripts/cosmux-migrate.sh`
- Templates: `~/.config/cosmux/templates/`
- HUD state: `~/.cosmux/state.json`
