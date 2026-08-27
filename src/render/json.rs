//! `--format json` rendering: the full document — traversal structure,
//! aggregation, and per-file detail — everything a consumer needs to
//! render results without re-running analysis. This is also the payload
//! the HTML views (#4-6) will embed. Mirrors `smell`'s own
//! `cli::complexity::json`: DTOs built from the domain types rather than
//! derived on them, since `feature::aggregate`'s types stay serde-free.

use serde::Serialize;
use smell::PathError;

pub mod breakdown;
pub mod detail;
pub mod node;
pub mod options;

use crate::feature::scope::Scan;
use crate::render::json::breakdown::Breakdown;
use crate::render::json::node::Node;
use crate::render::json::options::Options;

/// Schema version. Bump on any breaking change to the document shape.
const VERSION: u32 = 1;

/// Renders `scan` as a pretty-printed JSON document.
pub fn render(scan: &Scan) -> String {
    serde_json::to_string_pretty(&Document::new(scan))
        .expect("DTOs are always representable as JSON")
}

#[derive(Serialize)]
struct Document<'a> {
    version: u32,
    tool: Tool,
    aggregation: &'static str,
    options: Options<'a>,
    measures: Vec<&'static str>,
    totals: Breakdown<'a>,
    roots: Vec<Node<'a>>,
    errors: Vec<Error>,
}

impl<'a> Document<'a> {
    fn new(scan: &'a Scan) -> Document<'a> {
        Document {
            version: VERSION,
            tool: Tool::new(),
            aggregation: scan.mode.name(),
            options: Options::new(&scan.settings),
            measures: scan
                .tree
                .measures
                .iter()
                .map(|limit| limit.measure.name())
                .collect(),
            totals: Breakdown {
                counts: scan.tree.totals(),
                measures: &scan.tree.measures,
            },
            roots: node::nodes(&scan.tree.roots, &scan.tree.measures),
            errors: scan.errors.iter().map(Error::new).collect(),
        }
    }
}

#[derive(Serialize)]
struct Tool {
    name: &'static str,
    version: &'static str,
}

impl Tool {
    fn new() -> Tool {
        Tool {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

#[derive(Serialize)]
struct Error {
    path: String,
    message: String,
}

impl Error {
    fn new(error: &PathError) -> Error {
        Error {
            path: error.path.display().to_string(),
            message: error.error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::PathBuf;

    use serde_json::Value;
    use smell::Measure;

    use super::*;
    use crate::feature::aggregate::{Mode, tree::AggregatedTree};
    use crate::feature::scope::options::Settings;
    use crate::testing::{aggregated_file, counts, measure_limits};

    fn settings() -> Settings {
        Settings {
            rule: "default".to_string(),
            include: vec![],
            exclude: vec![],
            branches: vec![],
            implements: vec![],
            max_complexity: Some(10),
            max_methods: None,
            max_lines: None,
            max_declarations: None,
        }
    }

    fn scan<'a>(tree: &'a AggregatedTree, errors: &'a [PathError]) -> Scan<'a> {
        Scan {
            mode: Mode::default(),
            settings: settings(),
            tree,
            errors,
        }
    }

    fn value(tree: &AggregatedTree, errors: &[PathError]) -> Value {
        serde_json::from_str(&render(&scan(tree, errors))).expect("renders valid JSON")
    }

    #[test]
    fn render_emits_the_schema_version() {
        let tree = AggregatedTree::default();
        assert_eq!(value(&tree, &[])["version"], 1);
    }

    #[test]
    fn render_lists_only_configured_measures() {
        let tree = AggregatedTree {
            measures: measure_limits(&[(Measure::Complexity, 10)]),
            roots: vec![],
        };
        assert_eq!(
            value(&tree, &[])["measures"],
            serde_json::json!(["complexity"])
        );
    }

    #[test]
    fn render_echoes_the_options() {
        let tree = AggregatedTree::default();
        assert_eq!(value(&tree, &[])["options"]["maxComplexity"], 10);
    }

    #[test]
    fn render_maps_path_errors() {
        let tree = AggregatedTree::default();
        let errors = vec![PathError {
            path: PathBuf::from("missing"),
            error: io::Error::new(io::ErrorKind::NotFound, "not found"),
        }];
        let rendered = value(&tree, &errors);
        assert_eq!(rendered["errors"][0]["path"], "missing");
        assert!(
            rendered["errors"][0]["message"]
                .as_str()
                .unwrap()
                .contains("not found")
        );
    }

    #[test]
    fn render_emits_an_empty_errors_array() {
        let tree = AggregatedTree::default();
        assert_eq!(value(&tree, &[])["errors"], serde_json::json!([]));
    }

    #[test]
    fn render_of_an_empty_tree_has_zero_totals() {
        let tree = AggregatedTree::default();
        assert_eq!(value(&tree, &[])["totals"]["total"], 0);
    }

    #[test]
    fn render_is_pretty_printed() {
        let tree = AggregatedTree {
            measures: measure_limits(&[(Measure::Complexity, 10)]),
            roots: vec![aggregated_file("a.rs", None, counts(&[]), vec![])],
        };
        let rendered = render(&scan(&tree, &[]));
        assert!(rendered.contains('\n'));
    }
}
