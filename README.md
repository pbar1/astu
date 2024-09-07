# `astu`[^1]

Remote execution multitool.

## Project Map

```
.
├── action   → Action performers
├── cli      → Main binary entrypoint
├── db       → Result storage
├── resolve  → Target resolution
└── util     → Miscellaneous utilities
```

### Resolve

Resolvers expand input queries into targets.

Types of targets:

- IP address
- Socket address
- SSH address
- CIDR block
- Domain name
- File path

Types of resolvers:

- CIDR expansion
- DNS lookup
- File lines

### Action

Actions are behavior that clients can perform on targets.

Types of actions:

- Connect
- Auth
- Ping
- Exec

Types of clients:

- TCP
- SSH
- Kubernetes

<!-- Footnotes -->

[^1]: [Hello friend.](https://github.com/pbar1/astu/blob/main/.github/assets/mr_robot.jpg)
