# smell-o-scope

Track down and visualize code smell. Recurses through a folder, analyzes it
with [smell](https://github.com/ncipollo/smell), and renders a heat map of
code smells per folder and file.

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

## HTML output

`--format html` (the default) renders a single self-contained `.html` file —
embedded data, CSS, and JS, opening from `file://` with zero network
requests. No server, no CDN, no sidecar assets to lose track of.

The directory view is a collapsible tree of folders and files: roots start
expanded, everything below starts collapsed. Each row carries a badge per
configured measure, heat-scaled by violation count; expanding a file reveals
its offender detail. With no `--max-*` limit configured (and none in
`smell.toml`), there's nothing to flag — the tree still renders, just without
violation badges. The color theme follows the OS light/dark setting
automatically.

## JSON output

`--format json` renders a single self-contained document: the traversal
structure, the aggregated violation counts, and per-file offender detail —
everything needed to render results without re-running analysis. It's also
the payload the HTML output embeds.

```jsonc
{
  "version": 1,                 // schema version; bumped on breaking changes
  "tool": { "name": "smell-o-scope", "version": "0.1.0" },
  "aggregation": "violations",
  "options": { "rule": "default", "include": [], "exclude": [ /* ... */ ],
               "branches": [], "implements": [],
               "maxComplexity": 10, "maxMethods": null, "maxLines": null, "maxDeclarations": null },
  "measures": ["complexity"],   // only measures with a configured limit
  "totals": { "total": 7, "complexity": 7 },
  "roots": [
    { "name": "src", "path": "/abs/src", "kind": "directory",
      "violations": { "total": 7, "complexity": 7 },
      "children": [
        { "name": "main.rs", "path": "/abs/src/main.rs", "kind": "file", "lines": 120,
          "violations": { "total": 1, "complexity": 1 },
          "detail": { "complexity": [ { "name": "run", "value": 22 } ] } }
      ] }
  ],
  "errors": [ { "path": "…", "message": "…" } ]
}
```

`version: 1` is a contract other tools can build against: `measures`,
`totals`, each node's `violations`, and each file's `detail` only ever list
measures that had a configured limit for that run; roots and their children
are sorted by path so the same scan always produces the same document.
