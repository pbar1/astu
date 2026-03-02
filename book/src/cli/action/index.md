# Action

Actions resolve targets and perform operations on that target set.

The general sequence goes like this:

1. Resolve input target queries into into a set of unique targets
2. Present an action plan and require either interactive approval or
   `--confirm=<targets>` with the exact number of targets that the action will
   affect
3. Perform the sequence of actions defined by the subcommand for each target in
   concurrently, bounded by `--concurrency`, displaying progress using
   `indicatif` as tasks complete
4. Display freq info for errors only (ie, automatically run `astu freq error`)
   and suggestions for the command to run next, ie `astu freq` or `astu output`.

If `astu` receives a `ctrl-c` interrupt during a run: currently running tasks
will wait for completion, while not-yet-started tasks will be persisted as
canceled. Canceled jobs may be resumed with `astu resume`. A second `ctrl-c`
will forcefully kill the run without waiting for running tasks to complete.

## Options

### Action Options

#### `-T`/`--target`

Target URI or short form.

If not passed, will default to the `local:` target. Can be passed multiple
times.

#### `-f`/`--target-file`

Path to a file to read target URIs from.

Can use `-` to read from stdin. If this is set, then `--stdin` is assumed to be
`target`. Can be passed multiple times.

#### `--stdin`

Default: `auto`

Possible values: `auto`, `param`, `target`, `pipe`

How to interpret stdin.

`auto` sets the mode based on this chain of priority:

1. If `{param}` is used in the command template -> `param`
2. If `--target-file` is `-` or `/dev/stdin` -> `target`
3. Else -> `pipe`

`param` splits incoming stdin into tokens based on whitespace.

`target` allows `--target-file` to read from stdin (must still be passed on its
own).

`pipe` multiplexes stdin to each of the tasks by writing to a spool file where
each task has a cursor, guaranteeing delivery.

#### `--timeout`

Default: `30s`

Per-task timeout value in humantime. 0 indicates no timeout.

#### `--confirm`

Auto-accept the plan if passed target count is correct.

Required if running non-interactively to proceed with action. Skips prompt for
confirmation if running interactively.
