//! Test-only fixture helpers shared across unit and end-to-end tests.

use std::path::{Path, PathBuf};

use smell::code::{FileComplexity, FunctionComplexity};
use smell::{
    CheckFailure, CheckResult, DirectoryNode, FileNode, FileReport, Measure, Offender, PathError,
    Subject, TreeAnalysis, TreeNode,
};

use crate::feature::aggregate::tree::{
    AggregatedDirectory, AggregatedFile, AggregatedNode, Counts, Finding, MeasureLimit,
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

/// Builds a `Counts` from `(measure, count)` pairs.
pub fn counts(pairs: &[(Measure, usize)]) -> Counts {
    let mut counts = Counts::default();
    for (measure, value) in pairs {
        counts.increment(*measure, *value);
    }
    counts
}

/// Builds the `MeasureLimit` list an `AggregatedTree` carries, from
/// `(measure, limit)` pairs.
pub fn measure_limits(pairs: &[(Measure, usize)]) -> Vec<MeasureLimit> {
    pairs
        .iter()
        .map(|(measure, limit)| MeasureLimit {
            measure: *measure,
            limit: *limit,
        })
        .collect()
}

/// Builds an `AggregatedNode::File`.
pub fn aggregated_file(
    path: &str,
    lines: Option<usize>,
    counts: Counts,
    findings: Vec<Finding>,
) -> AggregatedNode {
    AggregatedNode::File(AggregatedFile {
        path: PathBuf::from(path),
        lines,
        counts,
        findings,
    })
}

/// Builds an `AggregatedNode::Directory`.
pub fn aggregated_directory(
    path: &str,
    counts: Counts,
    children: Vec<AggregatedNode>,
) -> AggregatedNode {
    AggregatedNode::Directory(AggregatedDirectory {
        path: PathBuf::from(path),
        counts,
        children,
    })
}

/// Normalizes a rendered document for snapshotting: replaces the fixture
/// root's absolute path (which varies by checkout location) and this
/// crate's own version (which varies by release) with stable placeholders.
pub fn normalize_document(document: &str, root: &Path) -> String {
    document
        .replace(&root.display().to_string(), "<fixture-root>")
        .replace(env!("CARGO_PKG_VERSION"), "<version>")
}
