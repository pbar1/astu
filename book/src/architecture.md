# Architecture

## Targets

URIs for each target type:

```
ip://127.0.0.1
cidr://10.0.0.0/31
tcp://127.0.0.1:22
ssh://user@127.0.0.1:2222
k8s://{user}@{cluster}/{namespace}/{pod}/{container}
file://{path}
dns:[//authority/]host[:port]
```

Shortcuts for avoiding having to specify full URIs:

```
127.0.0.1      -> ip://127.0.0.1
10.0.0.0/31    -> cidr://10.0.0.0/31
127.0.0.1:22   -> tcp://127.0.0.1:22
user@127.0.0.1 -> ssh://user@127.0.0.1
hosts.txt      -> file:///abs/path/hosts.txt
example.com    -> dns:///example.com
```

Sources of well-known URI schemes:

- [gRPC Name Resolution][uri_grpc]
- [SSH URI RFC][uri_ssh]

## Clients

[uri_grpc]: https://github.com/grpc/grpc/blob/ac90ebd310955024a188712b5231575e40dffcc5/doc/naming.md#detailed-design
[uri_ssh]: https://datatracker.ietf.org/doc/html/draft-salowey-secsh-uri-00
