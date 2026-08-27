//! Placeholder rendering of a [`TreeAnalysis`], its [`CheckResult`]s, and
//! their [`AggregatedTree`]. Stands in for the real HTML document, which
//! lands in a later issue (#4-6); `--format json` now renders the real
//! document (see [`crate::render::json`]).

use smell::{CheckFailure, CheckResult, FileNode, FileReport, Subject, TreeAnalysis, TreeNode};

use crate::feature::aggregate::AggregatedTree;

pub mod violations;

pub fn render(tree: &TreeAnalysis, results: &[CheckResult], aggregated: &AggregatedTree) -> String {
    let mut out = String::from("smell-o-scope (placeholder render)\n\n");
    if tree.roots.is_empty() {
        out.push_str("no files analyzed\n\n");
    } else {
        render_nodes(&tree.roots, 0, &mut out);
        out.push('\n');
    }
    out.push_str(&render_checks(results));
    out.push('\n');
    out.push_str(&violations::render(aggregated));
    out
}

fn render_nodes(nodes: &[TreeNode], depth: usize, out: &mut String) {
    for node in nodes {
        render_node(node, depth, out);
    }
}

fn render_node(node: &TreeNode, depth: usize, out: &mut String) {
    match node {
        TreeNode::Directory(directory) => render_directory(directory, depth, out),
        TreeNode::File(file) => render_file(file, depth, out),
    }
}

fn render_directory(node: &smell::DirectoryNode, depth: usize, out: &mut String) {
    out.push_str(&format!("{}{}/\n", indent(depth), node.path.display()));
    render_nodes(&node.children, depth + 1, out);
}

fn render_file(node: &FileNode, depth: usize, out: &mut String) {
    let name = node.path.file_name().map_or_else(
        || node.path.display().to_string(),
        |name| name.to_string_lossy().to_string(),
    );
    let summary = node
        .report
        .as_ref()
        .map_or_else(|| "(no report)".to_string(), file_summary);
    out.push_str(&format!("{}{}  {}\n", indent(depth), name, summary));
}

fn file_summary(report: &FileReport) -> String {
    let rollup = report.complexity.rollup();
    format!(
        "lines {} · total {} · max {} · avg {:.1} · decls {}",
        report.lines,
        rollup.total,
        rollup.max,
        rollup.average,
        report.complexity.declarations()
    )
}

fn render_checks(results: &[CheckResult]) -> String {
    let failing: Vec<&CheckResult> = results.iter().filter(|result| result.failed()).collect();
    if results.is_empty() {
        return "checks: none configured\n".to_string();
    }
    if failing.is_empty() {
        return "checks: all passed\n".to_string();
    }
    let mut out = String::from("checks:\n");
    for result in failing {
        out.push_str(&render_check(result));
    }
    out
}

fn render_check(result: &CheckResult) -> String {
    let mut out = format!(
        "✗ {} (limit {}): {} file(s)\n",
        result.measure.name(),
        result.limit,
        result.failures.len()
    );
    for failure in &result.failures {
        out.push_str(&render_failure(failure));
    }
    out
}

fn render_failure(failure: &CheckFailure) -> String {
    match &failure.subject {
        Subject::File(value) => format!("  {}  {}\n", failure.path.display(), value),
        Subject::Entries(offenders) => {
            let mut out = format!("  {}\n", failure.path.display());
            for offender in offenders {
                out.push_str(&format!("    {}  {}\n", offender.name, offender.value));
            }
            out
        }
    }
}

pub(super) fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

#[cfg(test)]
mod tests {
    use smell::Measure;

    use super::*;
    use crate::testing::{
        check_result, directory_node, entries_failure, file_failure, file_node, file_report, tree,
    };

    /// These tests only exercise the tree/checks sections; the violations
    /// section has its own tests in `violations::tests`.
    fn no_violations() -> AggregatedTree {
        AggregatedTree::default()
    }

    #[test]
    fn render_lists_directories_and_files() {
        let tree = tree(vec![directory_node(
            "src",
            vec![file_node(
                "src/lib.rs",
                Some(file_report("src/lib.rs", 1, 1)),
            )],
        )]);
        let text = render(&tree, &[], &no_violations());
        assert!(text.contains("src/\n"));
        assert!(text.contains("lib.rs"));
    }

    #[test]
    fn render_indents_nested_nodes() {
        let tree = tree(vec![directory_node(
            "src",
            vec![file_node(
                "src/lib.rs",
                Some(file_report("src/lib.rs", 1, 1)),
            )],
        )]);
        let text = render(&tree, &[], &no_violations());
        let directory_line = text.lines().find(|line| line.contains("src/")).unwrap();
        let file_line = text.lines().find(|line| line.contains("lib.rs")).unwrap();
        let directory_indent = directory_line.len() - directory_line.trim_start().len();
        let file_indent = file_line.len() - file_line.trim_start().len();
        assert_eq!(file_indent, directory_indent + 2);
    }

    #[test]
    fn render_marks_files_without_a_report() {
        let tree = tree(vec![file_node("src/lib.rs", None)]);
        let text = render(&tree, &[], &no_violations());
        assert!(text.contains("(no report)"));
    }

    #[test]
    fn render_includes_file_metrics() {
        let tree = tree(vec![file_node(
            "src/lib.rs",
            Some(file_report("src/lib.rs", 42, 3)),
        )]);
        let text = render(&tree, &[], &no_violations());
        assert!(text.contains("lines 42"));
        assert!(text.contains("total 3"));
        assert!(text.contains("max 3"));
        assert!(text.contains("avg 3.0"));
        assert!(text.contains("decls 1"));
    }

    fn complexity_result(failures: Vec<CheckFailure>) -> CheckResult {
        check_result(Measure::Complexity, 10, failures)
    }

    #[test]
    fn render_summarizes_failing_checks() {
        let failure = entries_failure("src/a.rs", &[("Shape.area", 12)]);
        let text = render(
            &tree(vec![]),
            &[complexity_result(vec![failure])],
            &no_violations(),
        );
        assert!(text.contains("✗ complexity (limit 10): 1 file(s)"));
        assert!(text.contains("src/a.rs"));
        assert!(text.contains("Shape.area  12"));
    }

    #[test]
    fn render_omits_passing_checks() {
        let text = render(
            &tree(vec![]),
            &[complexity_result(vec![])],
            &no_violations(),
        );
        assert!(text.contains("checks: all passed"));
        assert!(!text.contains("✗"));
    }

    #[test]
    fn render_notes_when_no_checks_are_configured() {
        let text = render(&tree(vec![]), &[], &no_violations());
        assert!(text.contains("checks: none configured"));
    }

    #[test]
    fn render_notes_an_empty_tree() {
        let text = render(&tree(vec![]), &[], &no_violations());
        assert!(text.contains("no files analyzed"));
    }

    #[test]
    fn render_renders_a_file_subject_as_a_flat_value() {
        let failure = file_failure("src/a.rs", 150);
        let result = check_result(Measure::Lines, 100, vec![failure]);
        let text = render(&tree(vec![]), &[result], &no_violations());
        assert!(text.contains("src/a.rs  150"));
    }
}
