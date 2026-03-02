# `astu freq`

Alias: `f`

Displays tables of captured stdout/stderr/exitcode/error aggregated by count of
appearance in a job.

Empty strings in stdout/stderr will be displayed as such. Exitcodes that do not
exist (such as the task erroring before a real exitcode is received) will be
displayed as -1. Tasks that did not error at all will not be displayed in the
error table; thus it could potentially not sum to 100%. All other tables will
sum to 100%.

## Examples

### Display all fields aggregated in the last job

```sh
astu freq
```

<details>
<summary>Output</summary>

```
stdout
| value    | count | pct |
|----------|-------|-----|
| foo      | 6     | 60% |
| bar      | 3     | 30% |
| baz      | 1     | 10% |

stderr
(no rows)

exitcode
| value | count | pct  |
|-------|-------|------|
| 0     | 2     | 100% |

error-freq
| value     | count | pct |
|-----------|-------|-----|
| foo error | 3     | 30% |
| bar error | 2     | 20% |
| baz error | 1     | 10% |
```

</details>

### Display all fields aggregated where that field contains a string in the last job

```sh
astu freq --contains=foo
```

<details>
<summary>Output</summary>

```
stdout
| value    | count | pct |
|----------|-------|-----|
| foo      | 6     | 60% |

stderr
(no rows)

exitcode
(no rows)

error-freq
| value     | count | pct |
|-----------|-------|-----|
| foo error | 3     | 30% |
```

</details>

### Display only stdout aggregated for all tasks in an explicit job

```sh
astu freq stdout --job=746677e7-b6f9-458b-857e-aa6a8638e101
```

<details>
<summary>Output</summary>

```
stdout
| value    | count | pct |
|----------|-------|-----|
| foo      | 6     | 60% |
| bar      | 3     | 30% |
| baz      | 1     | 10% |
```

</details>
