# `astu lookup`

Alias: `l`, `resolve`

Resolves targets.

Expands a set of input targets into a set of actionable targets. Does not
display a plan, and no actual actions are performed on targets.

## Examples

### Resolve the default target

```sh
astu lookup
```

<details>
<summary>Output</summary>

```
local:
```

</details>

### Resolve a single target

```sh
astu lookup -T ssh://user@host
```

<details>
<summary>Output</summary>

```
ssh://user@host
```

</details>

### Resolve multiple targets

```sh
astu lookup -T cidr://user@[::1]:22/127
```

<details>
<summary>Output</summary>

```
ip://user@[::]:22
ip://user@[::1]:22
```

</details>

### Resolve targets from files, stdin, and flags

```sh
cat targets.txt \
| astu lookup \
    -f targets_a.txt \
    -f targets_b.txt \
    -f - \
    -T ssh://user@host
```

<details>
<summary>Output</summary>

```
dns://foo@host-from-a
dns://bar@host-from-b
dns://baz@host-from-stdin
dns://quux@host-from-stdin
ssh://user@host
```

</details>
