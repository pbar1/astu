# `astu run`

Alias: `r`, `exec`

Runs a command on targets.

Performs this sequence of actions on each target in the set:

- Connect
- Auth (if required) (until authenticated)
- Exec

Persists stdout/stderr/exitcode/error, as well as the timing of each phase.

## Templates

Template strings that will be substituted with per-task context. Can also be
reverse substituted with `--dedupe` to reduce output cardinality. Not all
templates are guaranteed to exist; if a template cannot be substituted, the job
will fail fast.

- `{param}`: Split param from `--stdin=param` mode
- `{host}`: Target hostname
- `{user}`: Target login username
- `{ip}`: Target IP address

## Options

### Arguments

#### `<COMMAND>`

Command template.

### Options

#### `--live`

Stream task stdout and stderr the terminal.

Useful for things produce live tails of information such as bpftrace. Not to be
used with fullscreen programs like Vim. Output will still be captured to the
database.

#### `--dedupe`

Default: `param`, `host`, `user`, `ip`

Possible values: See TEMPLATES.

Deduplicators for line normalization.

These values will be substituted for their template tokens when seen, so that
aggregations like `astu freq` can more usefully display values that differ
predictably. Also helps with db size.

## Examples

### Execute a command on an SSH target

```sh
astu run -T ssh://user@host 'whoami'
```

<details>
<summary>Output</summary>

```
TODO
```

</details>
