# Quickstart

### Resolve targets

DNS queries return all IPs that are found:

```sh
astu lookup -T something.com
104.21.59.206
172.67.183.168
```

Ports are also supported and preserved:

```sh
astu lookup -T localhost:22
127.0.0.1:22
```

### Ping targets

```sh
astu ping -T localhost:22 --confirm=1
```
