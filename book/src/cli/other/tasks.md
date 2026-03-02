# `astu tasks`

Alias: `t`, `task`

Displays a table of tasks and their metadata within a job. The last job will
automatically be used if job is not explicitly passed.

## Options

### Options

#### `-j`/`--job`

Default: Last run job ID

Job ID to display results for.

If not set, will use the last action job ID persisted in the DB.

## Examples

### Display all tasks in the last job

```sh
astu tasks
```

<details>
<summary>Output</summary>

```
| task_id                              | target            | status   |
|--------------------------------------|-------------------|----------|
| 019ca7da-534f-7fa0-874a-7b17b297d9e1 | local://localhost | complete |
| 019ca7da-534f-7fa0-874a-7b067d6636f7 | local://localhost | failed   |
```

</details>
