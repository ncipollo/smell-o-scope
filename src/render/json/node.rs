//! The tree of directory/file nodes, sorted by path at every level so the
//! document is diffable regardless of the order `smell` traversed in.

use serde::Serialize;

use crate::feature::aggregate::tree::{
    AggregatedDirectory, AggregatedFile, AggregatedNode, MeasureLimit,
};
use crate::render::display_name;
use crate::render::json::breakdown::Breakdown;
use crate::render::json::detail::Detail;

/// `#[serde(tag = "kind")]` would emit `kind` first (`{"kind":…,"name":…}`);
/// `untagged` serializes the inner struct as-is, preserving each struct's
/// own field order (`name, path, kind, …`).
#[derive(Serialize)]
#[serde(untagged)]
pub enum Node<'a> {
    Directory(Directory<'a>),
    File(File<'a>),
}

#[derive(Serialize)]
pub struct Directory<'a> {
    pub name: String,
    pub path: String,
    pub kind: &'static str,
    pub violations: Breakdown<'a>,
    pub children: Vec<Node<'a>>,
}

/// `lines` is `None` when the file matched traversal filters but produced
/// no report — the only signal that its empty `detail` means "never
/// checked" rather than "checked and passed".
#[derive(Serialize)]
pub struct File<'a> {
    pub name: String,
    pub path: String,
    pub kind: &'static str,
    pub lines: Option<usize>,
    pub violations: Breakdown<'a>,
    pub detail: Detail<'a>,
}

/// Builds the JSON nodes for one level of the tree, sorted by path. Called
/// recursively, so every level (not just the roots) ends up sorted.
pub fn nodes<'a>(source: &'a [AggregatedNode], measures: &'a [MeasureLimit]) -> Vec<Node<'a>> {
    let mut sorted: Vec<&AggregatedNode> = source.iter().collect();
    sorted.sort_by(|a, b| a.path().cmp(b.path()));
    sorted
        .into_iter()
        .map(|node| build(node, measures))
        .collect()
}

fn build<'a>(node: &'a AggregatedNode, measures: &'a [MeasureLimit]) -> Node<'a> {
    match node {
        AggregatedNode::Directory(directory) => {
            Node::Directory(directory_node(directory, measures))
        }
        AggregatedNode::File(file) => Node::File(file_node(file, measures)),
    }
}

fn directory_node<'a>(
    directory: &'a AggregatedDirectory,
    measures: &'a [MeasureLimit],
) -> Directory<'a> {
    Directory {
        name: display_name(&directory.path),
        path: directory.path.display().to_string(),
        kind: "directory",
        violations: Breakdown {
            counts: directory.counts,
            measures,
        },
        children: nodes(&directory.children, measures),
    }
}

fn file_node<'a>(file: &'a AggregatedFile, measures: &'a [MeasureLimit]) -> File<'a> {
    File {
        name: display_name(&file.path),
        path: file.path.display().to_string(),
        kind: "file",
        lines: file.lines,
        violations: Breakdown {
            counts: file.counts,
            measures,
        },
        detail: Detail {
            findings: &file.findings,
            measures,
        },
    }
}

#[cfg(test)]
mod tests {
    use smell::Measure;

    use super::*;
    use crate::feature::aggregate::tree::Counts;
    use crate::testing::{aggregated_directory, aggregated_file, counts, measure_limits};

    fn to_value(nodes: &[Node]) -> serde_json::Value {
        serde_json::to_value(nodes).expect("nodes serialize")
    }

    #[test]
    fn directory_carries_kind_and_children() {
        let measures = measure_limits(&[]);
        let child = aggregated_file("src/a.rs", Some(1), Counts::default(), vec![]);
        let source = vec![aggregated_directory("src", Counts::default(), vec![child])];
        let value = to_value(&nodes(&source, &measures));
        assert_eq!(value[0]["kind"], "directory");
        assert_eq!(value[0]["children"][0]["kind"], "file");
    }

    #[test]
    fn file_carries_lines_and_detail() {
        let measures = measure_limits(&[(Measure::Lines, 100)]);
        let source = vec![aggregated_file("src/a.rs", Some(42), counts(&[]), vec![])];
        let value = to_value(&nodes(&source, &measures));
        assert_eq!(value[0]["lines"], 42);
        assert_eq!(value[0]["detail"]["lines"], serde_json::Value::Null);
    }

    #[test]
    fn name_is_the_file_name_and_path_is_the_full_path() {
        let measures = measure_limits(&[]);
        let source = vec![aggregated_file(
            "src/nested/a.rs",
            None,
            Counts::default(),
            vec![],
        )];
        let value = to_value(&nodes(&source, &measures));
        assert_eq!(value[0]["name"], "a.rs");
        assert_eq!(value[0]["path"], "src/nested/a.rs");
    }

    #[test]
    fn name_falls_back_to_the_path_without_a_file_name() {
        let measures = measure_limits(&[]);
        let source = vec![aggregated_directory(".", Counts::default(), vec![])];
        let value = to_value(&nodes(&source, &measures));
        assert_eq!(value[0]["name"], ".");
    }

    #[test]
    fn nodes_are_sorted_by_path_at_every_level() {
        let measures = measure_limits(&[]);
        let source = vec![
            aggregated_file("src/b.rs", None, Counts::default(), vec![]),
            aggregated_file("src/a.rs", None, Counts::default(), vec![]),
        ];
        let value = to_value(&nodes(&source, &measures));
        assert_eq!(value[0]["path"], "src/a.rs");
        assert_eq!(value[1]["path"], "src/b.rs");
    }
}
