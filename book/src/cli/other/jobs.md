# `astu jobs`

Alias: `j`, `job`

Displays a table of jobs and their metadata.

## Examples

### Display all jobs

```sh
astu jobs
```

<details>
<summary>Output</summary>

```
| job_id                               | started_at                 | command                           | task_count |
|--------------------------------------|----------------------------|-----------------------------------|------------|
| 019ca7da-534f-7fa0-874a-7af651acbd65 | 2026-03-01 05:23:49.199082 | /usr/bin/printf 'x=%s\n' '{param} | 2          |
```

</details>
