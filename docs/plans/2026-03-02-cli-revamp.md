# Astu CLI Revamp Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Converge Astu’s CLI/runtime to the book-defined desired state while preserving core library quality and behavior, removing obsolete CLI surface, and porting selected xec capabilities (`local:` executor, `dummy:` target/executor, durable stdin pipe fanout, and xec’s high-scale DuckDB schema/query model for deduplicated lines and fast `freq`).

**Architecture:** Keep Astu’s core layering (`cli` -> `lib::resolve` + `lib::action` + `lib::db`) and perform an API-compatible CLI shell rewrite around it. Use DuckDB persistence behind Astu’s existing DB trait/impl abstraction. Introduce durable stdin spool for `--stdin=pipe` fanout and add `local`/`dummy` client implementations in `lib::action`.

**Tech Stack:** Rust 2021, clap 4, tokio, tracing, Astu `lib`, DuckDB crate (single storage engine), existing resolver/client abstractions.

---

## Safety + Workflow Constraints

- [x] Create local branch `revamp` from current `main`.
- [x] Keep all work local only (`NO PUSH`).
- [x] Use per-command commit-signing override for commits only: `git -c commit.gpgsign=false commit ...`.
- [x] Do not execute any SSH auth flows, do not use SSH agent, do not touch remote targets.
- [x] Use only local/dummy targets in tests/examples.

## Phase 1: Gap Audit and Contract Lock

- [x] Produce explicit CLI contract matrix: `book` desired commands/options vs current `cli/src`.
- [x] Lock final command surface:
- [x] `action`: `run` (alias `r`, `exec`), `ping` (alias `p`), `lookup` (alias `l`, `resolve`), `resume`.
- [x] `result`: `freq` (alias `f`), `output` (alias `o`, `out`), `trace`.
- [x] `other`: `jobs` (alias `j`, `job`), `tasks` (alias `t`, `task`), `gc`.
- [x] Lock removals:
- [x] Remove `cp` subcommand.
- [x] Remove `file:` target type support from resolver graph and parser.
- [x] Keep `--target-file` (repeatable, `-` stdin support) as CLI file-input mechanism.
- [x] Confirm we retain K8s target parsing/resolution stubs only; no K8s action client implementation.

## Phase 2: CLI Argument Model Rewrite (Book-First)

- [x] Replace current top-level `Command` enum in `cli/src/cmd.rs` with grouped command structure matching book taxonomy.
- [x] Update global options to book contract:
- [x] `--data-dir` with `ASTU_DATA_DIR`.
- [x] `--log-level` with `ASTU_LOG` semantics.
- [x] `-o/--output` (`text|json`) global format.
- [x] Rework action options in `cli/src/args`:
- [x] `-T/--target` repeatable (default to `local:` when absent).
- [x] `-f/--target-file` repeatable; accept `-` and `/dev/stdin`.
- [x] `--stdin` (`auto|param|target|pipe`) with documented inference order.
- [x] `--timeout`, `--concurrency`, `--confirm`.
- [x] Remove legacy/holdover flags that are not in desired state.
- [x] Add clap tests for aliases, defaults, and incompatible combinations.

## Phase 3: Target + Resolver Changes

- [x] Remove `TargetKind::File` and parser constructors/short-form branches tied to file targets.
- [x] Remove `resolve::provider::file` from resolver chains and exports.
- [x] Keep target ingestion from `--target-file` in CLI layer by reading lines and parsing each line as a normal target URI/short form.
- [x] Add `TargetKind::Local` and `TargetKind::Dummy` in Astu target model.
- [x] Add parser/formatter tests for `local://...` and `dummy://...`.

## Phase 4: Client/Executor Additions

- [x] Add `LocalClientFactory`/`LocalClient` in `lib/src/action/client/local.rs`.
- [x] Implement `exec` for local client via `tokio::process::Command` with stdout/stderr capture and exit code.
- [x] Define expected behavior for `ping` on local targets (fast success/no-op output).
- [x] Add `DummyClientFactory`/`DummyClient` in `lib/src/action/client/dummy.rs`.
- [x] Implement deterministic dummy outputs from URI query params (`stdout`, `stderr`, `exitcode`) for testability.
- [x] Register factories in dynamic client mapping and CLI `client_factory` wiring.
- [x] Ensure no SSH agent usage unless explicitly requested; default action flows should not require it.

## Phase 5: Durable stdin Pipe Spooling

- [x] Introduce spool subsystem for `--stdin=pipe` (new module under `cli/src` or `lib/src/util`):
- [x] Read stdin once and persist to spool file under run-scoped temp path in data dir.
- [x] For each task, provide independent cursor/reader over spool file to guarantee full delivery even with slow readers.
- [x] Integrate spool lifecycle with job cancellation and cleanup policy.
- [x] Wire stdin mode behavior:
- [x] `param`: tokenize stdin into params and bind `{param}`.
- [x] `target`: feed target-file stdin loader.
- [x] `pipe`: multiplex persisted stdin to each executing task.
- [x] Add stress test with mixed fast/slow local readers proving identical byte delivery per task.

## Phase 6: Persistence Layer Convergence (DuckDB-Only, Abstracted)

- [x] Replace prior persistence implementation with DuckDB-backed implementation while preserving `Db` trait abstraction.
- [x] Keep `DbImpl` enum abstraction but single variant for DuckDB storage engine.
- [x] Port xec schema primitives as baseline data model (`jobs`, `tasks`, `task_vars`, `task_lines`, `line_dict`, `meta`) with append-oriented writes.
- [x] Preserve dictionary-backed line dedupe model (`task_lines.line_hash` -> `line_dict.line_text`) to support high-cardinality, high-volume fanout outputs.
- [x] Implement/port SQL query paths so `freq` and related aggregations run in DuckDB SQL (not row-by-row in Rust), matching xec performance intent.
- [x] Port schema and read paths needed by `jobs`, `tasks`, `freq`, `output`, `trace`, `resume`, and latest-job lookup.
- [x] Ensure migrations/schema init runs on startup.
- [x] Keep data model clean for action/task metadata and result payloads (stdout/stderr/exit/error/timings).
- [x] Add persistence tests for save/load + query behaviors used by new CLI commands.

## Phase 7: Command Implementations

- [x] `lookup`: resolve and print actionable targets (no plan confirmation).
- [x] `run`: plan/confirm, execute task sequence (`connect/auth/exec`), persist detailed task records.
- [x] `ping`: execute connect/ping sequence and persist outputs/errors.
- [x] `resume`: resume canceled/not-started tasks from last or explicit job.
- [x] `freq`: grouped aggregations (`stdout|stderr|exitcode|error`) with optional contains filter.
- [x] `output`: per-task values with optional field subset/contains/target filters.
- [x] `trace`: emit phase timing + error trace per task.
- [x] `jobs`: list jobs metadata.
- [x] `tasks`: list task metadata for selected/last job.
- [x] `gc`: delete old data by cutoff (`--before`).
- [x] Remove `cp` command module and references.

## Phase 8: Runtime Behavior + UX Details from Book

- [x] Implement per-run UUIDv7 run id and use as job id.
- [x] Write structured log file per run under `$ASTU_DATA_DIR/logs/<run-id>.log`.
- [x] Maintain `latest.log` symlink to most recent run log.
- [x] Persist and update “latest action-like job id” metadata for default `--job` resolution.
- [x] Keep TTY-aware progress rendering and stdout/stderr separation as documented.
- [x] Implement interactive confirmation prompt + non-interactive `--confirm=<target-count>` enforcement.
- [x] Implement Ctrl-C behavior: graceful first interrupt, forceful second interrupt, resumable canceled tasks.

## Phase 9: Testing + Verification Gates

- [x] Add CLI parsing tests for all command names/aliases and required flags.
- [x] Add integration tests for:
- [x] `run` with `local:` targets.
- [x] `run` with `dummy:` targets.
- [x] `--stdin=param|target|pipe` modes.
- [x] `--target-file` and stdin marker behavior.
- [x] `freq/output/jobs/tasks/trace` against persisted data.
- [x] `resume` semantics after cancellation.
- [x] Run verification commands and capture outputs:
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## Phase 10: Docs + Change Hygiene

- [x] Update book pages that still reference deprecated names (e.g., quickstart `resolve` examples) to desired command names if stale.
- [x] Add/update CLI help snapshots or doc snippets as needed.
- [x] Prepare logical local commits (small, reviewable) with commit signing disabled per commit command.
- [x] Provide final migration notes summarizing removed flags/commands and replacements.

## Execution Checkpoint Order

- [x] Checkpoint A: CLI contract and parser rewrite complete.
- [x] Checkpoint B: target/client changes + durable stdin spool complete.
- [x] Checkpoint C: DuckDB persistence + result/other commands complete.
- [x] Checkpoint D: tests/verification green; docs aligned; final cleanup.

## Audit Notes (2026-03-02, updated)

- All checklist items are marked complete for this revamp plan.
