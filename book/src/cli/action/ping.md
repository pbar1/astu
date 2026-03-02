# `astu ping`

Alias: `p`

Pings targets.

Performs this sequence of actions on each target in the set:

- Connect
- Ping

Persists the output of ping (if it exists) as stdout, as well as the timing of
each phase. Exitcode and stderr will never exist.

## Examples

### Ping a target with no errors

```sh
astu ping -T ssh://user@host
```

<details>
<summary>Output</summary>

```
error-freq
(no rows)
```

</details>

### Ping targets with some errors

```sh
astu ping -f targets-total-10.txt
```

<details>
<summary>Output</summary>

```
error-freq
| value     | count | pct |
|-----------|-------|-----|
| foo error | 3     | 30% |
| bar error | 2     | 20% |
| baz error | 1     | 10% |

Use `astu output` or `astu freq` for result analysis
```

</details>
