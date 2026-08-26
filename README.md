# smell-o-scope

Track down and visualize code smell. Recurses through a folder, analyzes it
with [smell](https://github.com/ncipollo/smell), and renders a heat map of
code smells per folder and file.

> 🚧 Work in progress — nothing is implemented yet.

## Install

```sh
cargo install --git https://github.com/ncipollo/smell-o-scope
```

## Usage

```sh
smell-o-scope src/ --format html --output report.html # heat map of src/, saved as html
smell-o-scope src/ --format json                      # full traversal + aggregation as json
```

## Configuration

Flags will mirror `smell`'s analysis flags (`--include`, `--exclude`,
`--max-complexity`, `--rule`, ...) and propagate down to it, including its
`smell.toml` configuration.
