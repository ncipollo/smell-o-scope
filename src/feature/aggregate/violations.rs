//! The `violations` aggregation mode: each check failure counts as one
//! violation under its measure, summed up the directory tree.

use smell::{CheckResult, DirectoryNode, FileNode, TreeAnalysis, TreeNode};

use crate::feature::aggregate::Aggregator;
use crate::feature::aggregate::failures::{self, FailureIndex};
use crate::feature::aggregate::tree::{
    AggregatedDirectory, AggregatedFile, AggregatedNode, AggregatedTree, Counts, Finding,
};

/// One violation per offender: each function over `max_complexity`, each
/// type over `max_methods`, or the file itself over `max_lines` /
/// `max_declarations`.
pub struct Violations;

impl Aggregator for Violations {
    fn aggregate(&self, tree: &TreeAnalysis, checks: &[CheckResult]) -> AggregatedTree {
        let index = FailureIndex::build(checks);
        AggregatedTree {
            measures: failures::limits(checks),
            roots: aggregate_nodes(&tree.roots, &index),
        }
    }
}

fn aggregate_nodes(nodes: &[TreeNode], index: &FailureIndex) -> Vec<AggregatedNode> {
    nodes
        .iter()
        .map(|node| aggregate_node(node, index))
        .collect()
}

fn aggregate_node(node: &TreeNode, index: &FailureIndex) -> AggregatedNode {
    match node {
        TreeNode::Directory(directory) => aggregate_directory(directory, index),
        TreeNode::File(file) => aggregate_file(file, index),
    }
}

fn aggregate_directory(directory: &DirectoryNode, index: &FailureIndex) -> AggregatedNode {
    let children = aggregate_nodes(&directory.children, index);
    let mut counts = Counts::default();
    for child in &children {
        counts.merge(child.counts());
    }
    AggregatedNode::Directory(AggregatedDirectory {
        path: directory.path.clone(),
        counts,
        children,
    })
}

fn aggregate_file(file: &FileNode, index: &FailureIndex) -> AggregatedNode {
    let findings = index.findings(&file.path).to_vec();
    AggregatedNode::File(AggregatedFile {
        path: file.path.clone(),
        lines: file.report.as_ref().map(|report| report.lines),
        counts: count(&findings),
        findings,
    })
}

fn count(findings: &[Finding]) -> Counts {
    let mut counts = Counts::default();
    for finding in findings {
        counts.increment(finding.measure, finding.detail.count());
    }
    counts
}

#[cfg(test)]
mod tests {
    use smell::Measure;

    use super::*;
    use crate::testing::{
        check_result, directory_node, entries_failure, file_failure, file_node, file_report, tree,
    };

    fn aggregate(checks: &[CheckResult], nodes: Vec<TreeNode>) -> AggregatedTree {
        Violations.aggregate(&tree(nodes), checks)
    }

    fn expect_file(node: &AggregatedNode) -> &AggregatedFile {
        match node {
            AggregatedNode::File(file) => file,
            AggregatedNode::Directory(_) => panic!("expected a file node"),
        }
    }

    fn expect_directory(node: &AggregatedNode) -> &AggregatedDirectory {
        match node {
            AggregatedNode::Directory(directory) => directory,
            AggregatedNode::File(_) => panic!("expected a directory node"),
        }
    }

    #[test]
    fn violations_counts_a_complexity_offender_as_one() {
        let checks = vec![check_result(
            Measure::Complexity,
            10,
            vec![entries_failure("a.rs", &[("f", 12)])],
        )];
        let aggregated = aggregate(&checks, vec![file_node("a.rs", None)]);
        assert_eq!(
            expect_file(&aggregated.roots[0])
                .counts
                .get(Measure::Complexity),
            1
        );
    }

    #[test]
    fn violations_counts_a_method_offender_as_one() {
        let checks = vec![check_result(
            Measure::Methods,
            5,
            vec![entries_failure("a.rs", &[("Big", 6)])],
        )];
        let aggregated = aggregate(&checks, vec![file_node("a.rs", None)]);
        assert_eq!(
            expect_file(&aggregated.roots[0])
                .counts
                .get(Measure::Methods),
            1
        );
    }

    #[test]
    fn violations_counts_a_lines_failure_as_one() {
        let checks = vec![check_result(
            Measure::Lines,
            100,
            vec![file_failure("a.rs", 150)],
        )];
        let aggregated = aggregate(&checks, vec![file_node("a.rs", None)]);
        assert_eq!(
            expect_file(&aggregated.roots[0]).counts.get(Measure::Lines),
            1
        );
    }

    #[test]
    fn violations_counts_a_declarations_failure_as_one() {
        let checks = vec![check_result(
            Measure::Declarations,
            5,
            vec![file_failure("a.rs", 8)],
        )];
        let aggregated = aggregate(&checks, vec![file_node("a.rs", None)]);
        assert_eq!(
            expect_file(&aggregated.roots[0])
                .counts
                .get(Measure::Declarations),
            1
        );
    }

    #[test]
    fn violations_counts_every_offender_in_one_entries_subject() {
        let checks = vec![check_result(
            Measure::Complexity,
            10,
            vec![entries_failure("a.rs", &[("f", 12), ("g", 15)])],
        )];
        let aggregated = aggregate(&checks, vec![file_node("a.rs", None)]);
        assert_eq!(
            expect_file(&aggregated.roots[0])
                .counts
                .get(Measure::Complexity),
            2
        );
    }

    #[test]
    fn violations_sums_measures_for_a_multi_measure_file() {
        let checks = vec![
            check_result(
                Measure::Complexity,
                10,
                vec![entries_failure("a.rs", &[("f", 12), ("g", 15)])],
            ),
            check_result(Measure::Lines, 100, vec![file_failure("a.rs", 150)]),
        ];
        let aggregated = aggregate(&checks, vec![file_node("a.rs", None)]);
        let counts = expect_file(&aggregated.roots[0]).counts;
        assert_eq!(counts.get(Measure::Complexity), 2);
        assert_eq!(counts.get(Measure::Lines), 1);
        assert_eq!(counts.total(), 3);
    }

    #[test]
    fn violations_sums_child_counts_into_directories() {
        let checks = vec![check_result(
            Measure::Complexity,
            10,
            vec![
                entries_failure("src/a.rs", &[("f", 12)]),
                entries_failure("src/nested/b.rs", &[("g", 12), ("h", 13)]),
            ],
        )];
        let nodes = vec![directory_node(
            "src",
            vec![
                file_node("src/a.rs", None),
                directory_node("src/nested", vec![file_node("src/nested/b.rs", None)]),
            ],
        )];
        let aggregated = aggregate(&checks, nodes);
        let src = expect_directory(&aggregated.roots[0]);
        assert_eq!(src.counts.total(), 3);
        let nested = expect_directory(&src.children[1]);
        assert_eq!(nested.counts.total(), 2);
    }

    #[test]
    fn violations_keeps_nodes_without_failures() {
        let checks = vec![check_result(Measure::Complexity, 10, vec![])];
        let aggregated = aggregate(
            &checks,
            vec![file_node("a.rs", Some(file_report("a.rs", 1, 1)))],
        );
        assert_eq!(expect_file(&aggregated.roots[0]).counts.total(), 0);
    }

    #[test]
    fn violations_keeps_files_without_a_report() {
        let aggregated = aggregate(&[], vec![file_node("a.rs", None)]);
        assert_eq!(expect_file(&aggregated.roots[0]).counts.total(), 0);
    }

    #[test]
    fn violations_keeps_directories_without_failures() {
        let nodes = vec![directory_node("src", vec![file_node("src/a.rs", None)])];
        let aggregated = aggregate(&[], nodes);
        let src = expect_directory(&aggregated.roots[0]);
        assert_eq!(src.counts.total(), 0);
        assert_eq!(src.children.len(), 1);
    }

    #[test]
    fn violations_ignores_measures_without_a_configured_limit() {
        let checks = vec![check_result(
            Measure::Complexity,
            10,
            vec![entries_failure("a.rs", &[("f", 12)])],
        )];
        let aggregated = aggregate(&checks, vec![file_node("a.rs", None)]);
        assert_eq!(aggregated.measures.len(), 1);
        assert_eq!(
            expect_file(&aggregated.roots[0]).counts.get(Measure::Lines),
            0
        );
    }

    #[test]
    fn violations_returns_zero_counts_when_no_measures_are_configured() {
        let aggregated = aggregate(&[], vec![file_node("a.rs", None)]);
        assert!(aggregated.measures.is_empty());
        assert_eq!(expect_file(&aggregated.roots[0]).counts.total(), 0);
    }

    #[test]
    fn violations_records_the_line_count_from_the_report() {
        let aggregated = aggregate(
            &[],
            vec![file_node("a.rs", Some(file_report("a.rs", 42, 1)))],
        );
        assert_eq!(expect_file(&aggregated.roots[0]).lines, Some(42));
    }

    #[test]
    fn violations_leaves_lines_unknown_without_a_report() {
        let aggregated = aggregate(&[], vec![file_node("a.rs", None)]);
        assert_eq!(expect_file(&aggregated.roots[0]).lines, None);
    }
}
