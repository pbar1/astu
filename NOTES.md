# Notes

## Command structure

### `astu`

`Global Options` will only be shown here and will be omitted for the rest.

```
Arbitrary Shell Targeting Utility

Usage: astu [OPTIONS] <COMMAND>

Commands:
  cp       Copy files and directories to and from targets
  exec     Execute commands on targets
  freq     Aggregate run results
  ping     Connect to targets
  resolve  Resolve targets
  result   Print run results
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

Global Options:
      --log-level <LOG_LEVEL>  Log level [env: ASTU_LOG=] [default: error]
      --state-dir <PATH>       State directory [default: ~/.local/state/astu]
```

### `astu cp`

```
```

### `astu exec`

```
Execute commands on targets

Usage: astu exec [OPTIONS] [COMMAND]...

Arguments:
  [COMMAND]...  Command to run

Options:
      --stdin                  Pass stdin to the target sessions
      --tty                    Connect TTY to the target sessions
  -h, --help                   Print help

Target Resolution Options:
  -T, --target <QUERY>         Target query

Authentication Options:
  -u, --user <STRING>          Remote user to authenticate as [default: root]
      --password-file <PATH>   Path to password file
      --ssh-agent <PATH>       Path to SSH agent socket [env: SSH_AUTH_SOCK=]
      --ssh-key <PATH>         Path to SSH credential file
      --kubeconfig <PATH>      Path to kubeconfig file [env: KUBECONFIG=]

Action Options:
      --confirm <NUM>          Confirm target count
      --timeout <DURATION>     Time to allow each action to complete [default: 30s]
      --concurrency <NUM>      Number of actions to process at once [default: 500]

Output Options:
      --format <FORMAT>        Output format [default: text]
```

### `astu freq`

```
```

### `astu ping`

```
```

### `astu resolve`

```
```

### `astu result`

```
```