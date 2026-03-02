# `astu gc`

Cleans the database of jobs and their associated data.

## Examples

Interactively select data to delete

```sh
astu gc
```

Delete data that was collected 30 days ago and older

```sh
astu gc --before=30d
```
