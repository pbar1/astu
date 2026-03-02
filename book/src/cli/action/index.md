# Action

Actions resolve targets and perform an operation on that target set.

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
