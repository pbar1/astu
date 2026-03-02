# CLI

Astu is CLI tool for running commands and debugging connectivity at scale.

## Subcommands

Astu subcommands are broken down into a few groups:

- Action
- Result
- Other

### Action

These commands take targets as input and perform an action.

#### `astu lookup` (alias: `l`, `resolve`)

Resolves and expands input targets into a set of actionable targets.

```sh
# Resolves an SSH target
astu lookup -T ssh://user@host 'whoami'
```

#### `astu ping` (alias: `p`)

Resolves targets and then connects to each one.

```sh
# Pings an SSH target
astu ping -T ssh://user@host
```

#### `astu run` (alias: `r`, `exec`)

Resolves targets and then connects, authenticates, and runs a command on on each
one.

```sh
# Executes a command on an SSH target
astu run -T ssh://user@host 'whoami'
```

### Result

These commands take a job as input and display result data collected during that
job's run. The last job will automatically be used if job is not explicitly
passed.

#### `astu output` (alias: `o`, `out`)

Displays tables of captured stdout/stderr/exitcode/error per task in a job.

```sh
# Displays all fields for all tasks in the last job
astu output
```

```sh
# Displays all fields for an explicit target in the last job
astu output -T ssh://user@host
```

```sh
# Displays all fields for all tasks where that field contains a string in the last job
astu output --contains=needle
```

```sh
# Displays only stdout for all tasks in an explicit job
astu output stdout --job=746677e7-b6f9-458b-857e-aa6a8638e101
```

#### `astu freq` (alias: `f`)

Displays tables of captured stdout/stderr/exitcode/error aggregated by count of
appearance in a job.

```sh
# Displays all fields aggregated in the last job
astu freq
```

```sh
# Displays all fields aggregated which contain an explicit target in the last job
astu freq -T ssh://user@host
```

```sh
# Displays all fields aggregated where that field contains a string in the last job
astu freq --contains=needle
```

```sh
# Displays only stdout aggregated for all tasks in an explicit job
astu freq stdout --job=746677e7-b6f9-458b-857e-aa6a8638e101
```

#### `astu trace`

Displays a diagnostic trace of timings for the sequence of actions and observed
errors for tasks in a job.

```sh
astu trace -T ssh://user@host
```

### Other

#### `astu jobs` (alias: `j`, `job`)

Displays a table of jobs and their metadata.

```sh
# Displays all jobs
astu jobs
```

#### `astu tasks` (alias: `t`, `task`)

Displays a table of tasks and their metadata within a job. The last job will
automatically be used if job is not explicitly passed.

```sh
# Displays all tasks in the last job
astu tasks
```

```sh
# Displays all tasks in an explicit job
astu tasks --job=746677e7-b6f9-458b-857e-aa6a8638e101
```

#### `astu gc`

Cleans the database of jobs and their associated data.

```sh
# Interactively select data to delete
astu gc
```

```sh
# Delete data that was collected 30 days ago and older
astu gc --before=30d
```
