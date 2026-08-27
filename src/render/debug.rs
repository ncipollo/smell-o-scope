//! Placeholder rendering of a [`TreeAnalysis`] and its [`CheckResult`]s.
//! Stands in for the real JSON/HTML documents, which land in a later issue;
//! `--format` currently only picks the destination (see
//! [`crate::feature::scope::output`]), not the document shape.

use smell::{CheckFailure, CheckResult, FileNode, FileReport, Subject, TreeAnalysis, TreeNode};

pub fn render(tree: &TreeAnalysis, results: &[CheckResult]) -> String {
    let mut out = String::from("smell-o-scope (placeholder render)\n\n");
    if tree.roots.is_empty() {
        out.push_str("no files analyzed\n\n");
    } else {
        render_nodes(&tree.roots, 0, &mut out);
        out.push('\n');
    }
    out.push_str(&render_checks(results));
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

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use smell::code::{FileComplexity, FunctionComplexity};
    use smell::{DirectoryNode, Measure, Offender, PathError};

    use super::*;

    fn file_report(path: &str, lines: usize, complexity: usize) -> FileReport {
        FileReport {
            path: PathBuf::from(path),
            lines,
            complexity: FileComplexity {
                functions: vec![FunctionComplexity {
                    name: "top".to_string(),
                    complexity,
                }],
                types: vec![],
            },
        }
    }

    fn file_node(path: &str, report: Option<FileReport>) -> TreeNode {
        TreeNode::File(FileNode {
            path: PathBuf::from(path),
            report,
        })
    }

    fn directory_node(path: &str, children: Vec<TreeNode>) -> TreeNode {
        TreeNode::Directory(DirectoryNode {
            path: PathBuf::from(path),
            children,
        })
    }

    fn tree(roots: Vec<TreeNode>) -> TreeAnalysis {
        TreeAnalysis {
            roots,
            errors: Vec::<PathError>::new(),
        }
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
        let text = render(&tree, &[]);
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
        let text = render(&tree, &[]);
        let directory_line = text.lines().find(|line| line.contains("src/")).unwrap();
        let file_line = text.lines().find(|line| line.contains("lib.rs")).unwrap();
        let directory_indent = directory_line.len() - directory_line.trim_start().len();
        let file_indent = file_line.len() - file_line.trim_start().len();
        assert_eq!(file_indent, directory_indent + 2);
    }

    #[test]
    fn render_marks_files_without_a_report() {
        let tree = tree(vec![file_node("src/lib.rs", None)]);
        let text = render(&tree, &[]);
        assert!(text.contains("(no report)"));
    }

    #[test]
    fn render_includes_file_metrics() {
        let tree = tree(vec![file_node(
            "src/lib.rs",
            Some(file_report("src/lib.rs", 42, 3)),
        )]);
        let text = render(&tree, &[]);
        assert!(text.contains("lines 42"));
        assert!(text.contains("total 3"));
        assert!(text.contains("max 3"));
        assert!(text.contains("avg 3.0"));
        assert!(text.contains("decls 1"));
    }

    fn complexity_result(failures: Vec<CheckFailure>) -> CheckResult {
        CheckResult {
            measure: Measure::Complexity,
            limit: 10,
            failures,
        }
    }

    #[test]
    fn render_summarizes_failing_checks() {
        let failure = CheckFailure {
            path: PathBuf::from("src/a.rs"),
            subject: Subject::Entries(vec![Offender {
                name: "Shape.area".to_string(),
                value: 12,
            }]),
        };
        let text = render(&tree(vec![]), &[complexity_result(vec![failure])]);
        assert!(text.contains("✗ complexity (limit 10): 1 file(s)"));
        assert!(text.contains("src/a.rs"));
        assert!(text.contains("Shape.area  12"));
    }

    #[test]
    fn render_omits_passing_checks() {
        let text = render(&tree(vec![]), &[complexity_result(vec![])]);
        assert!(text.contains("checks: all passed"));
        assert!(!text.contains("✗"));
    }

    #[test]
    fn render_notes_when_no_checks_are_configured() {
        let text = render(&tree(vec![]), &[]);
        assert!(text.contains("checks: none configured"));
    }

    #[test]
    fn render_notes_an_empty_tree() {
        let text = render(&tree(vec![]), &[]);
        assert!(text.contains("no files analyzed"));
    }

    #[test]
    fn render_renders_a_file_subject_as_a_flat_value() {
        let failure = CheckFailure {
            path: PathBuf::from("src/a.rs"),
            subject: Subject::File(150),
        };
        let result = CheckResult {
            measure: Measure::Lines,
            limit: 100,
            failures: vec![failure],
        };
        let text = render(&tree(vec![]), &[result]);
        assert!(text.contains("src/a.rs  150"));
    }
}
