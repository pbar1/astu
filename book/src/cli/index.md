# CLI

At startup, `astu` will source information about its environment:

- The full cmdline of the current process
- Number of Tokio threads
- Ulimits
- Status of whether stdin/stdout/stderr are TTYs

```
$ASTU_DATA_DIR
├── astu.db
└── logs/
    ├── 019cacd7-f900-7e20-af4b-ff53ef7f8a9f.log
    └── latest.log -> 019cacd7-f900-7e20-af4b-ff53ef7f8a9f.log
```

The database will be created upon first use. Each run will ensure the database
schema is migrated. The latest action-like run will be persisted in the database
as the latest job ID (this may not always equal `latest.log` as such).

Each invocation will generate a unique run ID - this will be used as both the
job ID as well as the name of the debug log file. IDs are UUIDv7 strings so they
are time-sortable. Debug logs (via `tracing-appender`) will be written to this
file, and a symlink `latest.log` will always point to the latest run debug log.

Interactive progress will be displayed with `tracing-indicatif` if stderr is a
TTY. Outputs such as tables and JSON will be printed to stdout. Plans and
prompts for confirmation will be printed on stderr.

## Subcommand Groups

Astu subcommands are broken down into a few groups:

- Action
- Result
- Other

## Options

Flags are grouped based on scope and shared usage. Generally, each one will have
a hierarchy like this:

- Subcommand flags
- Subcommand group flags
- Global flags

### Global Options

Astu has a few global flags that are shared by all subcommands.

#### `--data-dir`

Env: `ASTU_DATA_DIR`

Default: `$XDG_DATA_HOME/astu` (if `XDG_DATA_HOME` set); `dirs` crate default
per platform (otherwise)

Astu database path.

#### `--log-level`

Env: `ASTU_LOG`

Default: `debug`

Filter directive for log file. Follows the `RUST_LOG` format.

#### `-o`/`--output`

Default: `text`

Possible values: `text`, `json`

Output format.

#### `-h`/`--help`

Prints help.
