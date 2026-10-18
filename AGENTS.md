# AGENTS.md - gs

Agent-facing brief for unsigned-gg/gs. Read this before editing. Lane owner:
the revenant lane (Linear project Revenant, team CER).

## What this is

`gs` holds the public Rust primitives for the Godseat control plane: gate
tiers (`crates/gs-domain`), estate sensing and the `gs` CLI (`crates/gs`), and
the task runner (`crates/xtask`). The repository is public. Never commit
secrets, internal hostnames or tokens.

## Rules that bind

- **LeanSpec first.** A new component or behaviour change starts as a spec in
  `specs/`, named `SPEC-NNNN-<slug>.md`. List it in the README
  "Specifications" section. Every `feat` or `fix` PR creates or updates a
  spec, or names one in its body (`SPEC-NNNN`). The CI `leanspec` job warns on
  an unlinked `feat` or `fix` PR (`scripts/ci/spec-link-check.sh`).
- **Host throttling.** Every cargo call on ceres uses `-j 2`
  (`.cargo/config.toml` sets `jobs = 2`).
- **Gates.** Before a PR, run: `cargo xtask hygiene`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --locked -j 2 -- -D warnings`,
  `cargo test --workspace --locked -j 2`, and `cargo deny check licenses bans sources`.
- **Commits.** Conventional Commits, signed. GitButler (`but`) is the
  mutation tool in agent clones.
- **CI runs on estate runners only** (SOP ci-runner-always-internal). Never
  add `ubuntu-latest` or any other GitHub-hosted label. A missing tool is a
  runner-image change, not a reason to use a vendor runner.
- **Tickets.** Every slice and PR has a Linear ticket, named in the branch or
  the PR body.
- **Gate output is not authority.** `gs gate` builds principal claims from
  command-line flags without verification (SPEC-0003, section 5). Do not use
  its output as an authorization decision.
