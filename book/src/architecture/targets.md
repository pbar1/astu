# Targets

A **target** is the basic unit of operation in Astu - it represents an object on which an action will be performed.

Targets are usually parsed from URIs, but they also support convenient short forms for common types. Not all target types support short forms; those that do will state so.

Here is an example long form for an IP target:

```
ip://127.0.0.1
```

While here is the equivalent short form for the same target:

```
127.0.0.1
```

Targets can be dynamically expanded and aggregated into other targets using [resolvers](./resolvers.md). The core Astu workflow revolves around dynamic target discovery using resolvers. Generally, one provides at least one seed target which is iteratively expanded using resolver chains.

[Clients](./clients.md) are drivers that perform actions on targets.

## Target Graph

A **target graph** is special target-centric data structure representing targets as a directed graph. Resolvers generally support resolving directly into a caller-provided target graph - this is useful for building a topological action plan.

Here's a simple example of the target graph for a DNS target that resolves to multiple different IP targets:

```dot process
digraph {
    rankdir=LR;
    0 [ label="dns://something.com"]
    1 [ label="ip://104.21.59.206"]
    2 [ label="ip://172.67.183.168"]
    0 -> 1 [ ]
    0 -> 2 [ ]
}
```

While here's a more complex example where targets have multiple parents. In this example, both the CIDR target `10.0.0.0/31` and the DNS target `myrouter.lan` point to the IP target `10.0.0.1`.

```dot process
digraph {
    rankdir=LR;
    0 [ label="cidr://10.0.0.0/31"]
    1 [ label="ip://10.0.0.0"]
    2 [ label="ip://10.0.0.1"]
    3 [ label="dns://myrouter.lan"]
    0 -> 1 [ ]
    0 -> 2 [ ]
    3 -> 2 [ ]
}
```

## Target Types

For each target type, the URI and short forms will be given along with some
examples.

### IP

Internet Protocol (IP) address.

- URI form: `ip://[user@]<ip>[:port]`
  - `ip://127.0.0.1`
  - `ip://root@127.0.0.1:22`
  - `ip://[::1]`
  - `ip://root@[::1]:22`
- Short form: `<ip>[:port]`
  - `127.0.0.1`
  - `127.0.0.1:22`
  - `::1`
  - `[::1]:22`

### TCP

Essentially an alias for IP.

### CIDR

Classless Inter-Domain Routing (CIDR) block.

- URI form: `cidr://[user@]<ip>[:port]/<prefix>`
  - `cidr://127.0.0.0/32`
  - `cidr://root@127.0.0.0:22/32`
  - `cidr://[::1]/128`
  - `cidr://root@[::1]:22/128`
- Short form: `<ip>/<prefix>`
  - `127.0.0.0/24`
  - `::1/128`

### DNS

Domain Name System (DNS) record.

- URI form: `dns://[user@]<name>[:port]`
  - `dns://localhost`
  - `dns://root@localhost:22`
- Short form: n/a

### SSH

Secure Shell (SSH) address.

- URI form: `ssh://[user@]<host>[:port]`
  - `ssh://127.0.0.1`
  - `ssh://localhost`
  - `ssh://root@localhost:2222`
- Short form: n/a

### File

Local file.

- URI form: `file:[//]<path>`
  - `file:///absolute/file.txt`
  - `file://relative/file.txt`
  - `file:relative/file.txt`
- Short form: `<path>` (if path exists locally)
  - `/absolute/file.txt`
  - `relative/file.txt`

### Kubernetes

Kubernetes pod.

- URI form: `k8s:[//][user@][cluster][/namespace]/<name>[#container][?kind]`
  - `k8s:coredns-ff8999cc5-x56jw`
  - `k8s:kube-system/coredns#coredns?deployment`
  - `k8s://user@cluster/kube-system/coredns#coredns?deployment`
- Short form: n/a

<!-- Links -->

[uri_grpc]: https://github.com/grpc/grpc/blob/ac90ebd310955024a188712b5231575e40dffcc5/doc/naming.md#detailed-design
[uri_ssh]: https://datatracker.ietf.org/doc/html/draft-salowey-secsh-uri-00
