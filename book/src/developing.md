# Developing

## Project Structure

### CLI

The `astu` command line interface. Parses flags and loads configuration.

### Library

Core logic: types and drivers for target resolution and action execution.

## TODOs

- Eliminate all usages of `dyn` in favor of `enum_dispatch`
- Eliminate all usages of `async_trait` for more readable documentation
- Investigate if `internment` is really necessary for the target graph
