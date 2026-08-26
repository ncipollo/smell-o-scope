# smell-o-scope

## Architecture

Analysis is delegated to the [smell](https://github.com/ncipollo/smell) crate,
which handles traversal and code analysis and returns unaggregated per-file
results. This project owns aggregation, saving, and display. The code is broken
down into three layers:

- `cli` — The view layer, built with clap. `main` calls the top-level router in this module; each command gets its own file and is called from the router. No actual logic lives in this layer.
- `feature` — Where all domain logic lives (aggregation modes, orchestration of smell). The cli layer calls through to feature.
- `render` — Output generation (JSON and HTML documents).

## After Each Change
Run the following commands after every code change and fix any issues before considering the change complete:

1. `cargo fmt` - Format all code
2. `cargo test` - Run all tests
3. `cargo clippy` - Run linter; fix all warnings and errors before completing the change

### Fixing Clippy Complexity Warnings
When clippy reports `cognitive_complexity`, `too_many_lines`, or `too_many_arguments` warnings, fix them by refactoring — never suppress with `#[allow]`:
- Extract logical sub-steps into well-named helper functions.
- When a file accumulates many functions, reorganize into helper files and structs (following the module conventions below).

## Dependencies
Always use exact versions for dependencies in `Cargo.toml` (e.g., `"4.5.60"` not `"4"`). Check `Cargo.lock` for the resolved version when pinning.

## Module Conventions
Never use `mod.rs`. Always use the modern Rust style: create a top-level file (e.g., `foo.rs`) as the module root, and a matching folder (`foo/`) for any submodules.

## Imports
Always use `use` imports rather than full crate paths at call sites. For example, prefer `use crate::feature::aggregate;` + `aggregate::run(...)` over `crate::feature::aggregate::run(...)`.
