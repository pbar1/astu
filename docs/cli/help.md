# Astu CLI Help Snapshots (Revamp)

These snapshots document the current CLI surface after the revamp.

## `astu --help`

```text
Arbitrary Shell Targeting Utility

Usage: astu [OPTIONS] <COMMAND>

Commands:
  run     Execute commands on targets [aliases: r, exec]
  ping    Connect to targets [aliases: p]
  lookup  Resolve targets [aliases: l, resolve]
  resume  Resume a canceled job
  freq    Aggregate results from a prior run into frequency tables [aliases: f]
  output  Replay outputs from a prior run [aliases: o, out]
  trace   Display task trace timings and errors
  jobs    Display jobs metadata [aliases: j, job]
  tasks   Display tasks metadata for a job [aliases: t, task]
  gc      Garbage collect old persisted data
  help    Print this message or the help of the given subcommand(s)
```

## `astu run --help`

```text
Usage: astu run [OPTIONS] <COMMAND>

Arguments:
  <COMMAND>  Command template

Target Resolution Options:
  -T, --target <TARGETS>            Target query
  -f, --target-file <TARGET_FILES>  Path to file with target URIs

Action Options:
      --concurrency <CONCURRENCY>  Number of actions to process at once [default: 500]
      --confirm <CONFIRM>          Confirm target count
      --timeout <TIMEOUT>          Time to allow each action to complete [default: 30s]
      --stdin <STDIN>              How to interpret stdin [default: auto]
```

## `astu freq --help`

```text
Usage: astu freq [OPTIONS] [FIELDS]...

Arguments:
  [FIELDS]...  [possible values: stdout, stderr, exitcode, error]

Options:
  -j, --job <JOB>
      --contains <CONTAINS>
```
