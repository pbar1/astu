# Astu Development

## Project Structure

Main Cargo workspaces:

- `crates/astu-cli`: CLI scaffolding. No logic.
  - Args are in `arg`.
  - Subcommands are in `cmd`. Files here must only be subcommands.
- `crates/astu-types`: Shared data models and types. No logic.
- `crates/astu-core`: Core logic.
- `crates/astu-db`: Storage layer.
- `crates/astu-resolve`: Target resolvers.
- `crates/astu-action`: Action clients.

Other Cargo workspaces:

- `.`: Very thin binary entrypoint. No logic here.
- `xtask`: Task runner using the [`cargo xtask`][xtask] pattern.

Other directories:

- `.cargo`: Cargo config. Off limits.
- `.github`: GitHub Actions CI/CD config.
- `book`: mdbook for detailed docs.

## Guidelines

- Do not make commits to the repo unless explicitly told.
- Commits must be [Conventional Commits][convc].

## Project Commands

- `cargo xtask`: General project automation. Run it to see all commands.
  - Do not run `cargo check` - use `cargo xtask lint` instead.

<!-- References -->

[xtask]: https://github.com/matklad/cargo-xtask
[convc]: https://www.conventionalcommits.org/
