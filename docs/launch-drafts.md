# cosmux launch drafts (internal)

Pre-baked copy for Show HN, Reddit r/tmux, LinkedIn, Twitter/X. **Do not publish without Asif sign-off.**

---

## Show HN (target: Mon 8 AM PDT, only if it doesn't compete with FP launch)

**Title**: Show HN: Cosmux – Declarative tmux pod manager in Rust with lifecycle hooks

**Body**:

```
We run ~20 Claude Code agents across a dozen projects in tmux. Every reboot, tmux-resurrect dutifully restored a 27-pane mess. We wanted: declarative configs (one YAML per "pod"), lifecycle hooks (run a script before the pod boots, when a pane dies, when we detach), and one binary on PATH.

Existing tools came close but each had a deal-breaker:
- tmuxinator: needs Ruby, no death-recovery hooks
- tmuxp: needs Python, no death-recovery hooks
- smug: Go, has hooks, but not Rust (we're rewriting our infra in Rust)
- Moxide: closest in spirit, but no shell-hook escape hatch

So we built cosmux. ~1000 lines of Rust, 2 MB binary, no runtime deps. The interesting bit is `on_pane_dead`: cosmux installs a tmux hook on `pane-died` that calls back into itself, reads the recorded cwd + command from `~/.cosmux/state.json`, and respawns the pane in place. Means a crashed Claude session comes back where you left it, no "did the worker die overnight?" anxiety.

Apache-2.0. Repo: https://github.com/nxtg-ai/cosmux

Honest acknowledgements: heavily inspired by Moxide and smug — read both before writing line one. Not a fork.
```

---

## Reddit r/tmux

**Title**: I built a declarative tmux pod manager in Rust with auto-respawn on pane death (cosmux v0.2)

**Body**: Same as Show HN body, with a friendlier tone. Add screenshots of `cosmux start pod-forge` → `tmux attach` showing 4 named panes appear.

---

## LinkedIn (Asif's account)

```
We open-sourced cosmux today: a tiny Rust tool that turns tmux sessions into declarative "pods" with lifecycle hooks.

Why we built it: we run dozens of AI coding agents in tmux all day. tmux-resurrect kept rehydrating last week's mess on every reboot. We wanted a clean, version-controlled definition: "this is what the Forge workspace looks like — boot it." So we shipped cosmux (Rust, single binary, Apache-2.0).

This is the third public Rust project from the NXTG.ai portfolio (after Forge orchestrator and Faultline-Pro). One more piece of the "OS in Rust, brain in Python" thesis we're betting the company on.

If you live in tmux and have ever bash-aliased your way out of session sprawl, give it a look: github.com/nxtg-ai/cosmux

#rust #devtools #tmux #opensource
```

---

## Twitter / X

```
🦀 shipped cosmux — declarative tmux pod manager in Rust

- single binary, no runtime deps
- yaml configs you can git
- before_start / after_start / on_pane_dead hooks
- auto-respawn dead panes from recorded state

apache-2.0 → https://github.com/nxtg-ai/cosmux
```

---

## awesome-tmux PR

Add to "Session Managers" section of https://github.com/rothgar/awesome-tmux:

```
- [cosmux](https://github.com/nxtg-ai/cosmux) - Declarative tmux pod manager with lifecycle hooks. Single Rust binary, YAML configs, auto-respawn on pane death.
```

---

## Press order (Asif decision required)

1. Wait until cosmux v1.0 ships (cargo install works, full hook lifecycle proven on the live portfolio).
2. Coordinate with FP Show HN — don't double-post on the same Monday morning.
3. Show HN first (the "submit your project" audience), then Reddit (the "Linux/tmux power user" audience), then LinkedIn (the "founder credibility" audience), then Twitter (the broad signal).
4. Award an `awesome-tmux` PR ~24h after Show HN, while traffic is high.
