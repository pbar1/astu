# `astu`

Remote execution Swiss Army knife.

![Mr. Robot terminal](.github/assets/mr_robot.jpg "astu")

## Project Map

```
.
├── action   → Action performers
├── cli      → Main binary entrypoint
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
