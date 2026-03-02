# Astu CLI Revamp Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Converge Astu’s CLI/runtime to the book-defined desired state while preserving core library quality and behavior, removing obsolete CLI surface, and porting selected xec capabilities (`local:` executor, `dummy:` target/executor, durable stdin pipe fanout, and xec’s high-scale DuckDB schema/query model for deduplicated lines and fast `freq`).

**Architecture:** Keep Astu’s core layering (`cli` -> `lib::resolve` + `lib::action` + `lib::db`) and perform an API-compatible CLI shell rewrite around it. Replace SQLite persistence with DuckDB behind Astu’s existing DB trait/impl abstraction. Introduce durable stdin spool for `--stdin=pipe` fanout and add `local`/`dummy` client implementations in `lib::action`.

**Tech Stack:** Rust 2021, clap 4, tokio, tracing, Astu `lib`, DuckDB crate (single storage engine), existing resolver/client abstractions.

---

## Safety + Workflow Constraints

- [ ] Create local branch `revamp` from current `main`.
- [ ] Keep all work local only (`NO PUSH`).
- [ ] Use per-command commit-signing override for commits only: `git -c commit.gpgsign=false commit ...`.
- [ ] Do not execute any SSH auth flows, do not use SSH agent, do not touch remote targets.
- [ ] Use only local/dummy targets in tests/examples.

## Phase 1: Gap Audit and Contract Lock

- [ ] Produce explicit CLI contract matrix: `book` desired commands/options vs current `cli/src`.
- [ ] Lock final command surface:
- [ ] `action`: `run` (alias `r`, `exec`), `ping` (alias `p`), `lookup` (alias `l`, `resolve`), `resume`.
- [ ] `result`: `freq` (alias `f`), `output` (alias `o`, `out`), `trace`.
- [ ] `other`: `jobs` (alias `j`, `job`), `tasks` (alias `t`, `task`), `gc`.
- [ ] Lock removals:
- [ ] Remove `cp` subcommand.
- [ ] Remove `file:` target type support from resolver graph and parser.
- [ ] Keep `--target-file` (repeatable, `-` stdin support) as CLI file-input mechanism.
- [ ] Confirm we retain K8s target parsing/resolution stubs only; no K8s action client implementation.

## Phase 2: CLI Argument Model Rewrite (Book-First)

- [ ] Replace current top-level `Command` enum in `cli/src/cmd.rs` with grouped command structure matching book taxonomy.
- [ ] Update global options to book contract:
- [ ] `--data-dir` with `ASTU_DATA_DIR`.
- [ ] `--log-level` with `ASTU_LOG` semantics.
- [ ] `-o/--output` (`text|json`) global format.
- [ ] Rework action options in `cli/src/args`:
- [ ] `-T/--target` repeatable (default to `local:` when absent).
- [ ] `-f/--target-file` repeatable; accept `-` and `/dev/stdin`.
- [ ] `--stdin` (`auto|param|target|pipe`) with documented inference order.
- [ ] `--timeout`, `--concurrency`, `--confirm`.
- [ ] Remove legacy/holdover flags that are not in desired state.
- [ ] Add clap tests for aliases, defaults, and incompatible combinations.

## Phase 3: Target + Resolver Changes

- [ ] Remove `TargetKind::File` and parser constructors/short-form branches tied to file targets.
- [ ] Remove `resolve::provider::file` from resolver chains and exports.
- [ ] Keep target ingestion from `--target-file` in CLI layer by reading lines and parsing each line as a normal target URI/short form.
- [ ] Add `TargetKind::Local` and `TargetKind::Dummy` in Astu target model.
- [ ] Add parser/formatter tests for `local://...` and `dummy://...`.

## Phase 4: Client/Executor Additions

- [ ] Add `LocalClientFactory`/`LocalClient` in `lib/src/action/client/local.rs`.
- [ ] Implement `exec` for local client via `tokio::process::Command` with stdout/stderr capture and exit code.
- [ ] Define expected behavior for `ping` on local targets (fast success/no-op output).
- [ ] Add `DummyClientFactory`/`DummyClient` in `lib/src/action/client/dummy.rs`.
- [ ] Implement deterministic dummy outputs from URI query params (`stdout`, `stderr`, `exitcode`) for testability.
- [ ] Register factories in dynamic client mapping and CLI `client_factory` wiring.
- [ ] Ensure no SSH agent usage unless explicitly requested; default action flows should not require it.

## Phase 5: Durable stdin Pipe Spooling

- [ ] Introduce spool subsystem for `--stdin=pipe` (new module under `cli/src` or `lib/src/util`):
- [ ] Read stdin once and persist to spool file under run-scoped temp path in data dir.
- [ ] For each task, provide independent cursor/reader over spool file to guarantee full delivery even with slow readers.
- [ ] Integrate spool lifecycle with job cancellation and cleanup policy.
- [ ] Wire stdin mode behavior:
- [ ] `param`: tokenize stdin into params and bind `{param}`.
- [ ] `target`: feed target-file stdin loader.
- [ ] `pipe`: multiplex persisted stdin to each executing task.
- [ ] Add stress test with mixed fast/slow local readers proving identical byte delivery per task.

## Phase 6: Persistence Layer Convergence (DuckDB-Only, Abstracted)

- [ ] Replace SQLite implementation with DuckDB-backed implementation while preserving `Db` trait abstraction.
- [ ] Keep `DbImpl` enum abstraction but single variant for DuckDB storage engine.
- [ ] Port xec schema primitives as baseline data model (`jobs`, `tasks`, `task_vars`, `task_lines`, `line_dict`, `meta`) with append-oriented writes.
- [ ] Preserve dictionary-backed line dedupe model (`task_lines.line_hash` -> `line_dict.line_text`) to support high-cardinality, high-volume fanout outputs.
- [ ] Implement/port SQL query paths so `freq` and related aggregations run in DuckDB SQL (not row-by-row in Rust), matching xec performance intent.
- [ ] Port schema and read paths needed by `jobs`, `tasks`, `freq`, `output`, `trace`, `resume`, and latest-job lookup.
- [ ] Ensure migrations/schema init runs on startup.
- [ ] Keep data model clean for action/task metadata and result payloads (stdout/stderr/exit/error/timings).
- [ ] Add persistence tests for save/load + query behaviors used by new CLI commands.

## Phase 7: Command Implementations

- [ ] `lookup`: resolve and print actionable targets (no plan confirmation).
- [ ] `run`: plan/confirm, execute task sequence (`connect/auth/exec`), persist detailed task records.
- [ ] `ping`: execute connect/ping sequence and persist outputs/errors.
- [ ] `resume`: resume canceled/not-started tasks from last or explicit job.
- [ ] `freq`: grouped aggregations (`stdout|stderr|exitcode|error`) with optional contains filter.
- [ ] `output`: per-task values with optional field subset/contains/target filters.
- [ ] `trace`: emit phase timing + error trace per task.
- [ ] `jobs`: list jobs metadata.
- [ ] `tasks`: list task metadata for selected/last job.
- [ ] `gc`: delete old data by cutoff (`--before`).
- [ ] Remove `cp` command module and references.

## Phase 8: Runtime Behavior + UX Details from Book

- [ ] Implement per-run UUIDv7 run id and use as job id.
- [ ] Write structured log file per run under `$ASTU_DATA_DIR/logs/<run-id>.log`.
- [ ] Maintain `latest.log` symlink to most recent run log.
- [ ] Persist and update “latest action-like job id” metadata for default `--job` resolution.
- [ ] Keep TTY-aware progress rendering and stdout/stderr separation as documented.
- [ ] Implement interactive confirmation prompt + non-interactive `--confirm=<target-count>` enforcement.
- [ ] Implement Ctrl-C behavior: graceful first interrupt, forceful second interrupt, resumable canceled tasks.

## Phase 9: Testing + Verification Gates

- [ ] Add CLI parsing tests for all command names/aliases and required flags.
- [ ] Add integration tests for:
- [ ] `run` with `local:` targets.
- [ ] `run` with `dummy:` targets.
- [ ] `--stdin=param|target|pipe` modes.
- [ ] `--target-file` and stdin marker behavior.
- [ ] `freq/output/jobs/tasks/trace` against persisted data.
- [ ] `resume` semantics after cancellation.
- [ ] Run verification commands and capture outputs:
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`

## Phase 10: Docs + Change Hygiene

- [ ] Update book pages that still reference deprecated names (e.g., quickstart `resolve` examples) to desired command names if stale.
- [ ] Add/update CLI help snapshots or doc snippets as needed.
- [ ] Prepare logical local commits (small, reviewable) with commit signing disabled per commit command.
- [ ] Provide final migration notes summarizing removed flags/commands and replacements.

## Execution Checkpoint Order

- [ ] Checkpoint A: CLI contract and parser rewrite complete.
- [ ] Checkpoint B: target/client changes + durable stdin spool complete.
- [ ] Checkpoint C: DuckDB persistence + result/other commands complete.
- [ ] Checkpoint D: tests/verification green; docs aligned; final cleanup.
