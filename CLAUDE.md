# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`backito` is a Rust CLI that backs up a containerised Postgres database to
S3-compatible storage and proves the archive restores. `pg_dump`/`pg_restore`
run inside the database's own container via the `docker` CLI, so the host needs
`docker` but no Postgres client tools. See `README.md` for the user-facing
command reference and the wal-g point-in-time-recovery story.

## Commands

```bash
cargo build                       # debug build
cargo test                        # run all tests (213 test fns, all colocated)
cargo test <name>                 # filter by test-fn substring
cargo test -- verify              # e.g. everything touching verify
cargo clippy --all-targets        # lint; keep clean
cargo fmt                         # format
cargo run -- <subcommand>         # run the CLI locally
```

Tests are pure Rust units with no live Postgres or S3 dependency, so `cargo
test` runs offline. Docker- and network-touching code is behind traits/CLI
wrappers that tests drive with fakes.

## Layered architecture

Four layers, dependencies point inward (`cli` → `features` → `domain`, with
`infra` as the outward-facing adapters that `features` and `cli` call):

- **`domain/`** — values every command shares, no side effects: `ArchiveName`,
  `ArchiveDigest`, `Interval`, and the row-count comparison (`compare_counts`,
  `TableComparison`, `CountVerdict`).
- **`features/`** — one directory per behaviour (`backup`, `verify`, `restore`,
  `daemon`, `walg`, `init`, `progress`, `container`). Each splits into
  `domain.rs` (pure decision logic) and `services/` (orchestration that calls
  infra). Each has its own `errors.rs` (thiserror enum). `mod.rs` re-exports the
  public surface.
- **`infra/`** — adapters for everything outside the process: `config` (settings
  + secrets), `docker` (drives Postgres through the `docker` CLI),
  `object_store` (one S3 bucket via `object_store` crate), `logging`.
- **`cli/`** — parse args (`args.rs`), `dispatch` to one command handler per
  subcommand (`commands/`), return a `CommandReport`. Never writes stdout.

## Conventions that will trip you up if you miss them

**stdout has exactly one owner.** Commands return a `CommandReport { lines,
status }`; only `main.rs` prints it. This is what makes `KEY=$(backito backup)`
work. Never `println!` from a command — add lines to the report instead.
Progress and diagnostics go to stderr through the `ProgressObserver` trait
(`features/progress`), rendered by `TerminalReporter` in `cli/reporter.rs`. The
observer is display-agnostic, so a quiet run and a spinner are the same code
path with a different observer.

**Config is two independent halves.** `ConfigSource` supplies non-secret
settings from exactly one place (a TOML file *or* `BACKITO_*` env vars, chosen
by `--config`/`--env`, never both filling each other's gaps). `SecretSource`
supplies credentials, always from the environment. They never merge. Adding a
third backend (secret manager, remote config) is one more `ConfigSource` /
`SecretSource` impl, not a change to anything that reads settings. See
`infra/config/source.rs` and `mod.rs`.

**Exit codes carry meaning.** `ExitStatus` maps to: `0` success, `1` the command
could not run (`Failure`), `2` verification ran and found a mismatch. A
scheduled `verify` distinguishes an outage from a real mismatch by exit code, so
preserve these when touching verify/health.

**Container resolution is dynamic.** A database is pinned by `container` (exact
name) *or* `service` (resolved via docker labels on every pass, so a redeploy
under an orchestrator does not go stale) — exactly one, enforced at config load.
`daemon` re-resolves each pass.

**`pg_restore` errors are not verification failures**, drift (a restored copy
behind the source) is not loss, and a missing checksum *is* a failure. This
logic lives in `features/verify` — read `domain.rs` there before changing what
counts as a pass.

## Tests

Colocated next to the code they test with:

```rust
#[cfg(test)]
#[path = "domain_test.rs"]
mod domain_test;
```

So `foo.rs` has a sibling `foo_test.rs`. When adding a module, follow the same
pattern. Tests that read process-global credential env vars must take the shared
`ENV_TURN` mutex (`infra/config/mod.rs`) — two files with two different mutexes
race and clobber each other's variables.

## Edition and deps

Rust edition 2024. Notable dependency choice: `object_store` over `rust-s3`
because rust-s3 sizes multipart concurrency from host free memory (ignoring
cgroup limits) and OOM-kills on large archives under a container cap — see the
comment in `Cargo.toml`. `jiff` for time, `clap` derive for args, `thiserror`
for error enums, `indicatif` for progress.
