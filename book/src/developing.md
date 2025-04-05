# Developing

## Project Structure

### CLI

The `astu` command line interface. Parses flags and loads configuration.

### Library

Core logic: types and drivers for target resolution and action execution.

## Coding Style

### Commit Messages

Uses [`pre-commit`][pc] for enforcing [Conventional Commits][cc]. This is used for automating the release process.

Initialize pre-commit's Git hooks in this repo after first clone:

```sh
pre-commit install
```

## Release

Inspired by [this post][autorel]. Uses [conventional commits][convcom] for automatically bumping versions.

## TODOs

- Eliminate all usages of `dyn` in favor of `enum_dispatch`
- Eliminate all usages of `async_trait` for more readable documentation
- Investigate if `internment` is really necessary for the target graph

[autorel]: https://blog.orhun.dev/automated-rust-releases/
[pc]: https://pre-commit.com/
[cc]: https://www.conventionalcommits.org/en/v1.0.0/#summary
