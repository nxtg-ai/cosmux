# cosmux — Claude Code Project Guide

## What This Is

`cosmux` is a Rust tmux pod manager that turns terminal workspaces into declarative YAML configurations with lifecycle hooks and pane recovery.

## Architecture

```text
src/
├── main.rs       # CLI
├── config.rs     # YAML parsing and validation
├── tmux.rs       # tmux IPC
├── hooks.rs      # lifecycle hook execution
├── templates.rs  # pod templates
├── state.rs      # local state file
├── recover.rs    # pane recovery
└── error.rs      # typed errors
```

## Spawn Lifecycle

1. resolve and load a pod YAML;
2. validate and merge templates;
3. run `before_start` hooks;
4. create tmux session/windows/panes;
5. install lifecycle hooks;
6. persist local pod state;
7. run `after_start` hooks.

Pane recovery should remain deterministic: detect the dead pane, restore its declared working directory and command, then run the configured recovery hook.

## YAML Rules

- Quote `~` in YAML paths. Bare `~` is YAML null.
- Keep the public schema backward-compatible within a major version.
- Use synthetic example workspaces in tests and docs.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## Public / Private Boundary

This is a public repository. Do not commit private fleet topology, organization-internal session names, private repository paths, internal governance hooks, machine-specific runtime paths, private cross-project memory references, credentials, or generated operating state.

Examples should use generic pods such as `demo-api`, `worker`, and `reviewer`, never production workspace names.

## Product Scope

Keep cosmux focused on tmux workspace lifecycle management. Avoid coupling the public product to private NXTG infrastructure or organization-specific control planes.
