# Astu CLI Revamp Migration Notes

This revamp converges Astu CLI behavior to the book-defined surface.

## Command Renames and Grouping

- `resolve` is now surfaced as `lookup` (alias remains: `resolve`, plus `l`).
- `run` supports aliases: `r`, `exec`.
- `ping` supports alias: `p`.
- `freq` supports alias: `f`.
- `output` supports aliases: `o`, `out`.
- `jobs` supports aliases: `j`, `job`.
- `tasks` supports aliases: `t`, `task`.

## Removed Commands

- Removed: `cp`.

## Targeting Changes

- Removed target type: `file:`.
- Use `-f/--target-file` to load targets from files or stdin (`-` or `/dev/stdin`).
- Added/ported target kinds and executors:
- `local:` executes commands as local subprocesses.
- `dummy://...` returns deterministic mocked output for testing.
- Kubernetes target parsing/resolution remains available, but no Kubernetes action client is implemented.

## `run` Contract

- `run` now takes a single command template argument:
- `astu run 'echo hi'`
- `--stdin` modes: `auto|param|target|pipe`.

## Confirmation Semantics

Order of operations is now strict:

1. `--confirm=<count>`: proceeds only if the count exactly matches planned targets.
2. No `--confirm` in interactive mode: prompts for numeric target count input.
3. No `--confirm` in non-interactive mode: fails.

## Persistence and Results

- Storage engine is DuckDB-only behind Astu DB abstraction.
- Schema includes dictionary-backed line dedupe (`line_dict`, `task_lines`) to support fast `freq` queries at scale.
- `jobs`, `tasks`, `freq`, `output`, `trace`, and `resume` are backed by DuckDB query paths.

## Stdin Fanout (`--stdin=pipe`)

- stdin is read once and persisted to a spool file.
- each task receives an independent reader over the persisted spool data.
- spool file is cleaned up after execution lifecycle completes.
