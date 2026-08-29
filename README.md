# smell-o-scope

If there are code smells anywhere in the universe, you can bet you won't be out of the loop!

Track down and visualize code smells. Recurses through a folder, analyzes it
with [smell](https://github.com/ncipollo/smell), and renders a heat map of code smells per folder and file.

## Install

```sh
cargo install smell-o-scope
```

## Usage

```sh
smell-o-scope src/ --format html --output report.html # heat map of src/, saved as html
smell-o-scope src/ --format json                      # full traversal + aggregation as json
smell-o-scope src/ --rule swift                        # use the "swift" rule from smell.toml instead of "default"
smell-o-scope src/ --max-complexity 15 --max-methods 20 --max-lines 500 # override complexity thresholds

smell-o-scope --help # For more info use 
```

## Configuration

```toml
[[rule]]
name = "default"
max_complexity = 10
max_methods = 15

[[rule]]
name = "swift"
include = ["*.swift"]
max_complexity = 15
```

See the [smell README](https://github.com/ncipollo/smell#configuration) for
the full set of rule fields and how `--rule` selects between them.

## HTML output

`--format html` (the default) renders a single self-contained `.html` file.

This html page contains the follwoing viewing options:
- **directory**: This mode shows a file tree, along with the aggregated violations for each file and folder.
- **heatmap**: This mode shows your folder as a heatmap. Clicking a folder box will let you drill into it and see it's internal heat map (and so on).

## JSON output

`--format json` renders a json which is suitable for custom use cases built on top of smell-o-scope.

```jsonc
{
  // schema version; bumped on breaking changes
  "version": 1,
  "tool": {
    "name": "smell-o-scope",
    "version": "0.1.0"
  },
  "aggregation": "violations",
  "options": {
    "rule": "default",
    "maxComplexity": 10
  },
  // only measures with a configured limit
  "measures": ["complexity"],
  "totals": {
    "total": 7,
    "complexity": 7
  },
  "roots": [
    {
      "name": "src",
      "path": "/abs/src",
      "kind": "directory",
      "violations": {
        "total": 7,
        "complexity": 7
      },
      "children": [
        {
          "name": "main.rs",
          "path": "/abs/src/main.rs",
          "kind": "file",
          "lines": 120,
          "violations": {
            "total": 1,
            "complexity": 1
          },
          "detail": {
            "complexity": [{ "name": "run", "value": 22 }]
          }
        }
      ]
    }
  ],
  "errors": []
}
```
