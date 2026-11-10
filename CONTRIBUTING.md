# Contributing to Russet

This file is for OUTSIDERS — humans and AI agents that are not the owner. The
owner pushes to `master` directly and does not use PRs; everything below is
about everyone else.

## How this repository is developed

Russet is written by LLM agents. The owner directs the agents, applies their
work locally, and is the only person who commits and pushes. Therefore:

- **Changes are made by agents, not by hand.** If you want to contribute,
  either file an Issue describing the problem or idea, or run your own agent —
  see "If you are an AI agent" below.
- `README.md` is for humans who just want to build and run the app. This file
  and the docs it points to are for everyone else.

## If you are a human

1. Bug or idea → open an Issue: what you did, what you expected, what happened.
   Feature ideas are welcome but note that the scope is fixed by `SPEC.md` §6 —
   anything listed there as out of scope will be declined.
2. To build and run the app, follow `README.md` (NixOS/Windows/Web).
3. If you work through an agent, point the agent at this repository and let it
   read `AGENTS.md` first — that file is the agent rulebook, and PRs from an
   agent that ignored it will be closed unread.

## If you are an AI agent

- `AGENTS.md` is your rulebook: read it BEFORE any code change and follow it
  exactly (module split, test layout, style, verification gates).
- `SPEC.md` is the authoritative product spec; `PROGRESS.md` is the progress
  log; `PERF.md` is a reference backlog the owner picks from.
- The owner's constraints in `AGENTS.md` ("Repo facts", SPEC §6) apply to you
  unchanged: no new dependencies, no new top-level directories, no build
  tooling, no `unwrap()`/`expect()` on user-data paths.

## PR rules (non-owner)

PRs are reviewed by an LLM reviewer (e.g. GLM-5.3-Flash or similar), so the
diff must be small and self-explanatory:

1. **One logical change per PR.** A structural refactor, a behavior fix, and a
   documentation rework are three PRs, not one. If the What section of the
   template needs the word "and", split the PR.
2. **Fill the PR template** (it is pre-filled by
   `.github/pull_request_template.md`): What / Why / Files / Verification /
   Owner must check. Paste the real command output under Verification, not a
   summary of it.
3. **Every commit inside the PR must pass the gates** (see below), so a revert
   or bisect of any single commit stays green.
4. **CI must be green** on the PR (`.github/workflows/ci.yml` runs the same
   gates on GitHub's machines — that is the independent check that your local
   claims are true).
5. **No force-push after review starts** — reviewers work with line comments;
   rewriting history invalidates them.
6. **English only** in code, docs and PR/Issue text (the repo has no i18n).

## Verification gates (same as AGENTS.md "Commands")

Run before opening the PR; CI runs exactly these again:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

A PR that mixes gates-red code with a doc fix is one PR too many — keep it
green and small.

## Document map

| File | Audience | Content |
|---|---|---|
| `README.md` | humans | build & run only |
| `CONTRIBUTING.md` | outsiders (humans + AI) | this file: how to contribute |
| `AGENTS.md` | the owner's agents | the full agent rulebook |
| `SPEC.md` | agents (authoritative) | product spec, milestones, acceptance |
| `PROGRESS.md` | agents | progress log (substitute for git history) |
| `PERF.md` | the owner + agents | RAM / build-speed reference backlog |
