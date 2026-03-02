# Revamp vs Main Code Review (Read-Only)

Date: 2026-03-02
Branches compared: `main...revamp`
Scope: full repo review with emphasis on modularity, interface boundaries, non-branch-heavy idiomatic Rust, copy/allocation risks, DRY, separation of concerns, and style alignment between AI-heavy revamp code and established main-branch code.

## Highest-Risk Findings

### 1) Global interface boundary leak for data-dir (High)

`--data-dir` is not consistently respected.

- DB path is derived from global args (`GlobalArgs.data_dir`): `cli/src/cmd.rs:67`, `cli/src/args/global.rs:35`
- Stdin spool path for `run` is recomputed from env/defaults instead of global args: `cli/src/cmd/exec.rs:54`, `cli/src/args/action.rs:157`

Impact:
- Artifacts can be split across directories when users pass `--data-dir`, breaking operator expectations and system boundary consistency.

---

### 2) Auth errors are swallowed (High)

Task execution ignores authentication failures.

- Ignored result for user auth: `cli/src/args/action.rs:320`
- Ignored result for ssh-agent auth: `cli/src/args/action.rs:323`

Impact:
- Failure semantics are blurred (auth may fail but execution continues), making task outcomes and timing data less trustworthy.

---

### 3) DuckDB compatibility `load()` can drop tasks and alter bytes (High)

The load path for legacy `ResultEntry` reconstruction is anchored to stdout rows.

- Output bytes normalized through lossy UTF-8 text conversion: `lib/src/db/duckdb.rs:297`
- `load()` starts from stdout rows only: `lib/src/db/duckdb.rs:930`
- Iteration over stdout-only rows: `lib/src/db/duckdb.rs:951`
- Reconstructs bytes via `into_bytes()` from text: `lib/src/db/duckdb.rs:960`

Impact:
- Tasks without stdout may disappear from loaded results.
- Binary fidelity is not preserved for non-UTF8 streams despite `Vec<u8>` API surfaces.

## Architecture and Modularity

### 4) `ActionArgs` grew into a mixed-responsibility module (Medium)

One module now handles parsing, planning, orchestration, signal policy, normalization, and persistence interactions.

- Broad entry points and orchestration: `cli/src/args/action.rs:94`, `cli/src/args/action.rs:196`
- Stream normalization in same module: `cli/src/args/action.rs:537`

Impact:
- Separation of concerns regresses; maintenance and testability become harder.

---

### 5) DB abstraction boundary is weak at CLI layer (Medium)

Most commands downcast `DbImpl` directly to DuckDB.

- Examples: `cli/src/cmd/exec.rs:66`, `cli/src/cmd/ping.rs:42`, `cli/src/cmd/output.rs:65`, `cli/src/cmd/tasks.rs:27`
- Trait surface remains at `lib/src/db.rs:18`

Impact:
- Crisp backend boundary is reduced; backend replacement becomes costly.

---

### 6) DuckDB backend is over-concentrated (Medium)

Single file contains writer loop, schema init, read query API, GC, and compatibility adapter.

- Writer loop: `lib/src/db/duckdb.rs:636`
- Schema: `lib/src/db/duckdb.rs:840`
- `Db` compatibility impl: `lib/src/db/duckdb.rs:892`

Impact:
- Modularity and interface clarity suffer; higher chance of coupled regressions.

## Branching, Performance, and Copying

### 7) Branch-heavy hot path in task execution (Medium)

- Dense control flow in operation runner: `cli/src/args/action.rs:196`

Impact:
- Harder to reason about lifecycle correctness and failure states.

---

### 8) Copy/allocation churn in normalization and stream handling (Medium)

- Per-line string normalization with repeated replacement allocations: `cli/src/args/action.rs:537`, `lib/src/normalize.rs:29`, `lib/src/normalize.rs:31`
- Writer loop clone/allocation pressure in tight loops: `lib/src/db/duckdb.rs:788`, `lib/src/db/duckdb.rs:791`, `lib/src/db/duckdb.rs:832`

Impact:
- Elevated CPU/allocator overhead on high-output workloads.

---

### 9) Query-time reconstruction is expensive for repeated analysis (Medium)

- `string_agg`-based stdout/stderr assembly in `freq` and `output`: `lib/src/db/duckdb.rs:389`, `lib/src/db/duckdb.rs:467`, `lib/src/db/duckdb.rs:482`

Impact:
- Repeated analytics commands pay full reconstruction cost each time.

## DRY and Separation of Concerns

### 10) Duplicated field mapping/rendering logic (Low)

- Duplicate `FieldArg` conversions: `cli/src/cmd/freq.rs:24`, `cli/src/cmd/output.rs:28`
- Report rendering path duplicates table formatting style outside shared render helpers: `cli/src/report.rs:27`, `cli/src/cmd/render.rs:20`

Impact:
- Increased drift risk and small maintenance tax.

## AI-vs-Main Style Alignment

Overall style alignment with `main`: **moderate (about 6/10)**.

Observed mismatches:
- Main tends toward smaller focused command modules; revamp centralizes more behavior in larger files.
- Mixed use of user-facing `println!/eprintln!` and runtime internals where main previously favored tracing structure in more places.
- Naming drift: `Lookup` command wraps `ResolveArgs`: `cli/src/cmd.rs:48`, `cli/src/cmd/resolve.rs:10`.

Notable intentional/acceptable divergence:
- `run` now uses a single command template argument, documented in migration notes: `docs/cli/migration-revamp.md:30`, `cli/src/cmd/exec.rs:22`.

## Positives

- Cleaner command surface and aliases improve discoverability: `cli/src/cmd.rs:43`
- Reusable render utilities are a strong improvement: `cli/src/cmd/render.rs:11`
- Integration coverage for revamp behaviors is meaningfully stronger: `cli/tests/revamp_integration.rs:67`
- Async producer/writer split in DuckDB is a good throughput-oriented architecture choice: `lib/src/db/duckdb.rs:169`

## Suggested Priority Order (if/when implementing fixes)

1. Unify `--data-dir` usage for DB and spool paths.
2. Propagate auth failures instead of swallowing results.
3. Correct `Db::load` to avoid stdout-anchored row loss and clarify byte-fidelity guarantees.
4. Split `cli/src/args/action.rs` by concerns (args/planning/execution/normalization).
5. Reduce hot-path cloning and repeated allocation in normalization and writer loops.

## Expanded Remediation Plan (All Findings)

### R1) Unify runtime data-dir boundary (addresses Finding 1)

- Introduce a shared runtime context carrying `data_dir` from global args.
- Pass that context into command handlers instead of recomputing paths.
- Remove env/default fallback path recomputation in `run` spool creation.
- Ensure DB and spool both derive from one source of truth per invocation.

Primary files:
- `cli/src/cmd.rs`
- `cli/src/cmd/exec.rs`
- `cli/src/args/global.rs`
- `cli/src/args/action.rs`

Verification:
- Add integration test proving `--data-dir` forces DB and spool into same root.

---

### R2) Make authentication failure semantics explicit (addresses Finding 2)

- Replace ignored auth results (`let _ = client.auth(...)`) with explicit handling.
- Fail task early on auth error and persist clear task error metadata.
- Keep timing metrics coherent for connect/auth/exec stages when auth fails.

Primary files:
- `cli/src/args/action.rs`
- `cli/src/args/auth.rs`
- `lib/src/action.rs`

Verification:
- Add tests for invalid auth inputs (e.g., bad ssh-agent path) and assert `failed` status with meaningful error.

---

### R3) Resolve `Db::save/load` compatibility contract mismatch (addresses Finding 3)

- Decide whether legacy compatibility API (`ResultEntry`, `Db::save/load`) is retained or retired.
- If retained:
  - `load()` must not be stdout-anchored.
  - behavior for no-output and stderr-only tasks must be correct.
  - byte-fidelity contract must be explicit and tested.

Primary files:
- `lib/src/db.rs`
- `lib/src/db/duckdb.rs`

Verification:
- Add tests for stderr-only, no-output, and non-UTF8 stream cases.

---

### R4) Split `ActionArgs` into focused components (addresses Findings 4, 7, 8)

- Move orchestration logic out of `cli/src/args/action.rs` into cohesive modules:
  - confirm policy
  - stdin mode + input reading
  - task planning/spec expansion
  - task execution
  - templating/normalization helpers
- Keep `ActionArgs` as argument/config holder + lightweight delegation.

Primary files:
- `cli/src/args/action.rs`
- New modules under `cli/src/action/` (or similar)

Verification:
- Add focused unit tests for each extracted module.
- Keep existing integration behavior unchanged.

---

### R5) Make DB boundary honest and consistent in CLI (addresses Finding 5)

- Choose one design and apply consistently:
  1. Explicitly depend on `DuckDb` in CLI command layer (truthful single-backend design), or
  2. Expand trait boundary to include all query operations and stop downcasting.
- Remove inconsistent mixed use of trait + repeated `DbImpl::Duck` extraction.

Primary files:
- `lib/src/db.rs`
- `cli/src/cmd/*.rs`

Verification:
- Static check: command handlers follow one boundary model only.

---

### R6) Modularize DuckDB backend internals (addresses Finding 6)

- Split monolithic `duckdb.rs` into focused modules:
  - schema/init
  - writer/event loop
  - read queries
  - GC
  - legacy compatibility layer
- Preserve existing behavior and public API while reducing coupling.

Primary files:
- `lib/src/db/duckdb.rs` -> `lib/src/db/duckdb/*`

Verification:
- Existing DB tests pass; add module-level tests where coverage is thin.

---

### R7) Flatten branch-heavy task runner control flow (addresses Finding 7)

- Replace deep nested branching in `run_tasks_for_operation` with:
  - shared lifecycle stages (create/connect/auth/operate/finalize)
  - per-operation strategy/runners for exec vs ping
- Reduce duplicate task finalization logic.

Primary files:
- `cli/src/args/action.rs` or extracted executor module

Verification:
- Regression tests for cancellation/interrupt/resume flows.

---

### R8) Reduce copy/allocation hotspots (addresses Finding 8)

- Optimize stream normalization to avoid unnecessary whole-buffer rebuilds.
- Reduce tight-loop clones and transient allocations in writer path.
- Keep code idiomatic and avoid premature complexity.

Primary files:
- `cli/src/args/action.rs`
- `lib/src/normalize.rs`
- `lib/src/db/duckdb.rs`

Verification:
- Add large-output workload tests/bench harness and compare baseline allocations/time.

---

### R9) Improve query performance for repeated `freq`/`output` use (addresses Finding 9)

- Add/validate indexes on join/filter columns (`job_id`, `task_id`, `stream`, etc.).
- Evaluate caching/materialization strategy for assembled outputs if query cost remains high.
- Preserve exact result semantics and ordering.

Primary files:
- `lib/src/db/duckdb.rs` (or extracted query/schema modules)

Verification:
- Large synthetic dataset query-time comparisons before/after.

---

### R10) DRY and style-consistency sweep (addresses Finding 10)

- Consolidate duplicated field mapping used by `freq`/`output`.
- Use one shared rendering path for table output.
- Align naming/module sizing/style with the repo's non-AI baseline conventions.

Primary files:
- `cli/src/cmd/freq.rs`
- `cli/src/cmd/output.rs`
- `cli/src/report.rs`
- `cli/src/cmd/render.rs`

Verification:
- Snapshot/assertion tests for output format consistency across commands.

## Execution Checklist (All Findings)

- [ ] F1: add runtime context carrying `data_dir` and remove ad-hoc env/default data-dir recomputation.
- [ ] F1: add integration test proving DB and spool are co-located under `--data-dir`.

- [ ] F2: replace ignored auth results with explicit error handling.
- [ ] F2: ensure auth failure leads to deterministic failed task status and error text.
- [ ] F2: add tests for invalid auth path and failed auth behavior.

- [ ] F3: decide fate of legacy `Db::save/load` API (retain + fix or remove).
- [ ] F3: if retained, fix stdout-anchored loss behavior and byte-fidelity ambiguities.
- [ ] F3: add stderr-only/no-output/non-UTF8 coverage.

- [ ] F4: split `ActionArgs` responsibilities into focused modules.
- [ ] F4: keep CLI behavior stable while shrinking orchestration surface.
- [ ] F4: add unit tests for each extracted responsibility.

- [ ] F5: enforce one DB boundary model in CLI (explicit DuckDb or expanded trait).
- [ ] F5: remove inconsistent pattern of repeated downcasts in command modules.

- [ ] F6: split `lib/src/db/duckdb.rs` into schema/writer/queries/gc/compat modules.
- [ ] F6: ensure behavior and tests remain intact after module split.

- [ ] F7: flatten task runner control flow and deduplicate finalize paths.
- [ ] F7: add regression tests around interrupt/cancel/resume state transitions.

- [ ] F8: reduce normalization and writer-loop cloning/allocation overhead.
- [ ] F8: add large-output workload test/benchmark for before-vs-after comparison.

- [ ] F9: add/validate indexes and assess query plans for `freq`/`output`.
- [ ] F9: evaluate materialization only if indexes are insufficient.

- [ ] F10: deduplicate field mapping/rendering logic and align style with `main` conventions.
- [ ] F10: verify output consistency via tests.

- [ ] Run `cargo test -p astu`.
- [ ] Run `cargo test -p astu-cli`.
- [ ] Run `cargo test -p astu-cli --test revamp_integration`.
- [ ] Run `cargo clippy --workspace --all-targets`.
- [ ] Perform manual smoke run across `run`, `ping`, `resume`, `freq`, `output`, `trace`, `jobs`, `tasks`, and `gc`.
