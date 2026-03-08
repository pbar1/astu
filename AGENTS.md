# Astu Development

## Project Structure

Main Cargo workspaces:

- `astu-cli`: CLI scaffolding. No logic.
- `astu-types`: Shared data models and types. No logic.
- `astu-core`: Core logic.
- `astu-db`: Storage layer.
- `astu-resolve`: Target resolvers.
- `astu-action`: Action clients.

Other Cargo workspaces:

- `.`: Very thin binary entrypoint. No logic here.
- `xtask`: Task runner using the [`cargo xtask`][xtask] pattern.

Other directories:

- `.cargo`: Cargo config. Off limits.
- `.github`: GitHub Actions CI/CD config.
- `.vscode`: VS Code project settings and debug config.
- `lib`: Existing code that is being broken into workspaces crates.
- `book`: mdbook for detailed docs.

## Guidelines

- Do not make commits to the repo unless explicitly told.
- Commits must be [Conventional Commits][convc].

<!-- References -->

[xtask]: https://github.com/matklad/cargo-xtask
[convc]: https://www.conventionalcommits.org/
