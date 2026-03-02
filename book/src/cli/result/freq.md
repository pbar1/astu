# `astu freq`

Alias: `f`

Displays tables of captured stdout/stderr/exitcode/error aggregated by count of
appearance in a job.

## Examples

### Display all fields aggregated in the last job

```sh
astu freq
```

<details>
<summary>Output</summary>

```
TODO
```

</details>

### Display all fields aggregated which contain an explicit target in the last job

```sh
astu freq -T ssh://user@host
```

<details>
<summary>Output</summary>

```
TODO
```

</details>

### Display all fields aggregated where that field contains a string in the last job

```sh
astu freq --contains=needle
```

<details>
<summary>Output</summary>

```
TODO
```

</details>

### Display only stdout aggregated for all tasks in an explicit job

```sh
astu freq stdout --job=746677e7-b6f9-458b-857e-aa6a8638e101
```

<details>
<summary>Output</summary>

```
TODO
```

</details>
