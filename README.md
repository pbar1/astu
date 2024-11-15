<h1 align="center" style="border-bottom: none;"><code>astu</code></h1>

<p align="center"><b>Remote execution multitool</b></p>

<p align="center">
  <a href="https://github.com/pbar1/astu/actions/workflows/build.yml">
    <img alt="Build Status" src="https://github.com/pbar1/astu/actions/workflows/build.yml/badge.svg">
  </a>
  <a href="https://github.com/pbar1/astu/releases/latest">
    <img alt="GitHub release" src="https://img.shields.io/github/release/pbar1/astu.svg">
  </a>
</p>

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
