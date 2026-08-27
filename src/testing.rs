//! Test-only fixture helpers shared across unit and end-to-end tests.

use std::path::PathBuf;

use smell::code::{FileComplexity, FunctionComplexity};
use smell::{
    CheckFailure, CheckResult, DirectoryNode, FileNode, FileReport, Measure, Offender, PathError,
    Subject, TreeAnalysis, TreeNode,
};

/// Resolves a path under `fixtures/` at the crate root, independent of the
/// current working directory a test runs from.
pub fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(relative)
}

/// Builds a `FileReport` with one top-level function of the given complexity.
pub fn file_report(path: &str, lines: usize, complexity: usize) -> FileReport {
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

/// Builds a `TreeNode::File`, optionally with a report attached.
pub fn file_node(path: &str, report: Option<FileReport>) -> TreeNode {
    TreeNode::File(FileNode {
        path: PathBuf::from(path),
        report,
    })
}

/// Builds a `TreeNode::Directory` with the given children.
pub fn directory_node(path: &str, children: Vec<TreeNode>) -> TreeNode {
    TreeNode::Directory(DirectoryNode {
        path: PathBuf::from(path),
        children,
    })
}

/// Builds a `TreeAnalysis` from a set of roots, with no path errors.
pub fn tree(roots: Vec<TreeNode>) -> TreeAnalysis {
    TreeAnalysis {
        roots,
        errors: Vec::<PathError>::new(),
    }
}

/// Builds a `CheckResult` for the given measure, limit, and failures.
pub fn check_result(measure: Measure, limit: usize, failures: Vec<CheckFailure>) -> CheckResult {
    CheckResult {
        measure,
        limit,
        failures,
    }
}

/// A `CheckFailure` whose subject is named offenders (functions, types) —
/// each `(name, value)` pair becomes one `Offender`.
pub fn entries_failure(path: &str, offenders: &[(&str, usize)]) -> CheckFailure {
    CheckFailure {
        path: PathBuf::from(path),
        subject: Subject::Entries(
            offenders
                .iter()
                .map(|(name, value)| Offender {
                    name: (*name).to_string(),
                    value: *value,
                })
                .collect(),
        ),
    }
}

/// A `CheckFailure` whose subject is the whole file (lines, declarations).
pub fn file_failure(path: &str, value: usize) -> CheckFailure {
    CheckFailure {
        path: PathBuf::from(path),
        subject: Subject::File(value),
    }
}
