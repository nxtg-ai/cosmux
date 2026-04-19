# Contributing to cosmux

Thanks for taking a look. cosmux is built and maintained by NXTG AI but PRs and issues from the community are very welcome.

## Quick reality check

If you're proposing a feature, please open an issue first so we can talk about scope. cosmux is intentionally small — single binary, no plugin runtime, no web UI. Things that broaden the scope of "declarative tmux pod manager with lifecycle hooks" are a hard sell. Things that make the existing surface tighter (better error messages, broader OS support, clearer docs) are an easy sell.

## Local development

```bash
git clone https://github.com/nxtg-ai/cosmux.git
cd cosmux
cargo build
cargo test
./target/debug/cosmux validate path/to/your-pod.yaml
```

You'll need `tmux` installed (>= 3.0 recommended).

## Pull request checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo test --all-features` passes
- [ ] CHANGELOG.md updated under `[Unreleased]` (or a SemVer-appropriate section)
- [ ] If user-facing CLI changes: README.md updated

## Commit style

Conventional Commits, short subject, why-not-what in the body:

```
feat(hooks): support window-level after_start hooks

Until now, hooks were pod-scoped. Window-scoped hooks let
templates (e.g. claude-team-pane) bring their own setup
without polluting every pod that uses them.
```

## License

By contributing you agree your work is released under Apache-2.0 (the project license).

## Code of conduct

Be kind. Disagree about ideas, never about people. Maintainer reserves the right to lock threads that turn into bikesheds.
