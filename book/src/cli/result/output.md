# `astu output`

Alias: `o`, `out`

Displays tables of captured stdout/stderr/exitcode/error per task in a job.

## Examples

### Display all fields for all tasks in the last job

```sh
astu output
```

<details>
<summary>Output</summary>

```
TODO
```

</details>

### Display all fields for an explicit target in the last job

```sh
astu output -T ssh://user@host
```

<details>
<summary>Output</summary>

```
TODO
```

</details>

### Display all fields for all tasks where that field contains a string in the last job

```sh
astu output --contains=needle
```

<details>
<summary>Output</summary>

```
TODO
```

</details>

### Display only stdout for all tasks in an explicit job

```sh
astu output stdout --job=746677e7-b6f9-458b-857e-aa6a8638e101
```

<details>
<summary>Output</summary>

```
TODO
```

</details>
