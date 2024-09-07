# `astu`[^1]

Remote execution multitool.

## Project Map

```
.
├── action   → Action performers
├── cli      → Main binary entrypoint
├── db       → Result storage
├── resolve  → Target resolution
└── util     → Convenience code
```

### Resolve

Resolvers expand composite targets into more targets.

For example:

- CIDR block resolves to all IP addresses in the block
- File resolves to targets parsed from lines in the file

| Target \ Resolver | CIDR expand | DNS lookup | File lines |
| ----------------- | ----------- | ---------- | ---------- |
| IP address\*      |             |            |            |
| Socket address\*  |             |            |            |
| SSH address\*     |             |            |            |
| CIDR block        | 🟢          |            |            |
| Domain name       |             | 🟢         |            |
| File              |             |            | 🟢         |

\* Target is already fully resolved

### Action

Actions are behavior that clients can perform on targets.

| Client \ Action | Connect | Auth | Exec | Shell | Cp  |
| --------------- | ------- | ---- | ---- | ----- | --- |
| TCP             | 🟢      |      |      |       |     |
| SSH             | 🟢      | 🟢   | 🟢   | 🟢    | 🟢  |
| Kubernetes      |         |      | 🟢   | 🟢    | 🟢  |

<!-- Footnotes -->

[^1]: [Hello friend.](https://github.com/pbar1/astu/blob/main/.github/assets/mr_robot.jpg)
