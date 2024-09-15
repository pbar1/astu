# Notes

## Diagrams

### Action Stream

```mermaid
stateDiagram-v2
    R : Resolve targets
    B : Build action plan
    P : Perform action
    S : Store result

    [*] --> R : initial target

    R --> B : each target...

    B --> P : ok
    B --> [*] : err

    P --> S : success
    P --> S : failure

    S --> [*]
```

## Target Parsing

Perform helpful speculation on what the given value may be. For example:

- Assume `arn:` prefixes are ARNs and parse them accordingly
- Assume `i-` prefixes are EC2
- Assume `-[a-z0-9]{10}-[a-z0-9]{5}` suffixes are K8s pods
- Assume IPv4 and IPv6 are SSH

## TODO

- Implement no-op-ok or no-op-err for action clients, to help with enum dispatch
