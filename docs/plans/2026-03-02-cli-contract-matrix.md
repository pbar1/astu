# Astu CLI Contract Matrix (Book vs Current vs Target)

## Command Surface

| Group | Book (desired) | Current (`revamp` start) | Target after revamp |
|---|---|---|---|
| action | `run` (`r`,`exec`) | `exec` | `run` (`r`,`exec`) |
| action | `ping` (`p`) | `ping` | `ping` (`p`) |
| action | `lookup` (`l`,`resolve`) | `resolve` | `lookup` (`l`,`resolve`) |
| action | `resume` | missing | `resume` |
| result | `freq` (`f`) | `freq` (todo impl) | `freq` (`f`) |
| result | `output` (`o`,`out`) | `output` (todo impl) | `output` (`o`,`out`) |
| result | `trace` | missing | `trace` |
| other | `jobs` (`j`,`job`) | missing | `jobs` (`j`,`job`) |
| other | `tasks` (`t`,`task`) | missing | `tasks` (`t`,`task`) |
| other | `gc` | missing | `gc` |
| deprecated | `cp` removed | `cp` exists | removed |

## Action Option Surface

| Option | Book (desired) | Current | Target |
|---|---|---|---|
| `-T/--target` | yes, repeatable, default `local:` when absent | yes (repeatable), no default local applied in args | yes |
| `-f/--target-file` | yes, repeatable, `-` and `/dev/stdin` support | missing | yes |
| `--stdin` `auto|param|target|pipe` | yes | missing | yes |
| `--timeout` | yes | yes | yes |
| `--concurrency` | yes | yes | yes |
| `--confirm` | yes | yes | yes |

## Global Option Surface

| Option | Book (desired) | Current | Target |
|---|---|---|---|
| `--data-dir` env `ASTU_DATA_DIR` | yes | yes (`ASTU_DATA_DIR` not wired currently) | yes |
| `--log-level` env `ASTU_LOG` | yes | uses `RUST_LOG` + separate `--file-level` | `ASTU_LOG` + remove holdover `--file-level` |
| `-o/--output` `text|json` | yes | missing global output | yes |

## Target/Resolver Contract

| Item | Book / user direction | Current | Target |
|---|---|---|---|
| `file:` target type | remove | exists (`TargetKind::File`, file resolver) | removed |
| `--target-file` ingestion | keep | missing | added in CLI layer |
| `dummy:` target | add from xec | missing | added |
| `local:` target | add from xec | missing | added |
| `k8s:` action client | do not implement now | no k8s action client | keep unimplemented |

## Storage Contract

| Item | User direction | Current | Target |
|---|---|---|---|
| DB engine | DuckDB only (abstracted) | SQLite only | DuckDB only behind `Db` abstraction |
| High-scale schema | port xec schema for dedupe/freq speed | single `results` table | `jobs/tasks/task_vars/task_lines/line_dict/meta` baseline |
| `freq` performance | SQL-side aggregation over deduped lines | not implemented | DuckDB SQL aggregation |

## Runtime Contract

| Item | Book / user direction | Current | Target |
|---|---|---|---|
| stdin pipe mode | durable spool/cursor per task | no stdin mode support | file-backed spool with guaranteed delivery |
| ctrl-c behavior | graceful then forceful, resumable | not implemented | implemented |
| run logging | per-run uuidv7 log + `latest.log` symlink | `last.log` only | per-run file + symlink |

