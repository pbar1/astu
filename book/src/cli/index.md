# CLI

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
