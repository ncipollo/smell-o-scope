//! Renders the `violations:` section: per-node counts from an
//! [`AggregatedTree`], appended below the existing tree and checks output.

use std::path::Path;

use crate::feature::aggregate::AggregatedTree;
use crate::feature::aggregate::tree::{
    AggregatedDirectory, AggregatedFile, AggregatedNode, Counts, Detail, Finding, MeasureLimit,
};
use crate::render::debug::indent;

pub fn render(tree: &AggregatedTree) -> String {
    if tree.measures.is_empty() {
        return "violations: none configured\n".to_string();
    }
    let mut out = String::from("violations:\n");
    render_nodes(&tree.roots, &tree.measures, 1, &mut out);
    out
}

fn render_nodes(
    nodes: &[AggregatedNode],
    measures: &[MeasureLimit],
    depth: usize,
    out: &mut String,
) {
    for node in nodes {
        render_node(node, measures, depth, out);
    }
}

fn render_node(node: &AggregatedNode, measures: &[MeasureLimit], depth: usize, out: &mut String) {
    match node {
        AggregatedNode::Directory(directory) => render_directory(directory, measures, depth, out),
        AggregatedNode::File(file) => render_file(file, measures, depth, out),
    }
}

fn render_directory(
    directory: &AggregatedDirectory,
    measures: &[MeasureLimit],
    depth: usize,
    out: &mut String,
) {
    out.push_str(&format!(
        "{}{}/  {}\n",
        indent(depth),
        directory.path.display(),
        breakdown(directory.counts, measures)
    ));
    render_nodes(&directory.children, measures, depth + 1, out);
}

fn render_file(file: &AggregatedFile, measures: &[MeasureLimit], depth: usize, out: &mut String) {
    out.push_str(&format!(
        "{}{}  {}\n",
        indent(depth),
        file_name(&file.path),
        breakdown(file.counts, measures)
    ));
    render_findings(&file.findings, depth + 1, out);
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().to_string(),
    )
}

fn breakdown(counts: Counts, measures: &[MeasureLimit]) -> String {
    let parts: Vec<String> = measures
        .iter()
        .map(|limit| format!("{} {}", limit.measure.name(), counts.get(limit.measure)))
        .collect();
    format!("total {} ({})", counts.total(), parts.join(" · "))
}

fn render_findings(findings: &[Finding], depth: usize, out: &mut String) {
    for finding in findings {
        render_finding(finding, depth, out);
    }
}

fn render_finding(finding: &Finding, depth: usize, out: &mut String) {
    match &finding.detail {
        Detail::Whole(value) => {
            out.push_str(&format!(
                "{}{}  {}\n",
                indent(depth),
                finding.measure.name(),
                value
            ));
        }
        Detail::Entries(offenders) => {
            for offender in offenders {
                out.push_str(&format!(
                    "{}{}  {}  {}\n",
                    indent(depth),
                    finding.measure.name(),
                    offender.name,
                    offender.value
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use smell::Measure;

    use super::*;
    use crate::feature::aggregate::tree::Offender;

    fn counts(pairs: &[(Measure, usize)]) -> Counts {
        let mut counts = Counts::default();
        for (measure, value) in pairs {
            counts.increment(*measure, *value);
        }
        counts
    }

    fn measures(pairs: &[(Measure, usize)]) -> Vec<MeasureLimit> {
        pairs
            .iter()
            .map(|(measure, limit)| MeasureLimit {
                measure: *measure,
                limit: *limit,
            })
            .collect()
    }

    #[test]
    fn render_lists_counts_per_node() {
        let file = AggregatedNode::File(AggregatedFile {
            path: PathBuf::from("src/a.rs"),
            lines: None,
            counts: counts(&[(Measure::Complexity, 2)]),
            findings: vec![],
        });
        let tree = AggregatedTree {
            measures: measures(&[(Measure::Complexity, 10)]),
            roots: vec![file],
        };
        let text = render(&tree);
        assert!(text.contains("a.rs  total 2 (complexity 2)"));
    }

    #[test]
    fn render_indents_children_below_directories() {
        let file = AggregatedNode::File(AggregatedFile {
            path: PathBuf::from("src/a.rs"),
            lines: None,
            counts: counts(&[(Measure::Complexity, 1)]),
            findings: vec![],
        });
        let directory = AggregatedNode::Directory(AggregatedDirectory {
            path: PathBuf::from("src"),
            counts: counts(&[(Measure::Complexity, 1)]),
            children: vec![file],
        });
        let tree = AggregatedTree {
            measures: measures(&[(Measure::Complexity, 10)]),
            roots: vec![directory],
        };
        let text = render(&tree);
        let directory_line = text.lines().find(|line| line.contains("src/")).unwrap();
        let file_line = text.lines().find(|line| line.contains("a.rs")).unwrap();
        let directory_indent = directory_line.len() - directory_line.trim_start().len();
        let file_indent = file_line.len() - file_line.trim_start().len();
        assert_eq!(file_indent, directory_indent + 2);
    }

    #[test]
    fn render_lists_offenders_under_a_file() {
        let file = AggregatedNode::File(AggregatedFile {
            path: PathBuf::from("src/a.rs"),
            lines: None,
            counts: counts(&[(Measure::Complexity, 1)]),
            findings: vec![Finding {
                measure: Measure::Complexity,
                detail: Detail::Entries(vec![Offender {
                    name: "f".to_string(),
                    value: 12,
                }]),
            }],
        });
        let tree = AggregatedTree {
            measures: measures(&[(Measure::Complexity, 10)]),
            roots: vec![file],
        };
        let text = render(&tree);
        assert!(text.contains("complexity  f  12"));
    }

    #[test]
    fn render_omits_unconfigured_measures() {
        let file = AggregatedNode::File(AggregatedFile {
            path: PathBuf::from("src/a.rs"),
            lines: None,
            counts: counts(&[(Measure::Complexity, 1)]),
            findings: vec![],
        });
        let tree = AggregatedTree {
            measures: measures(&[(Measure::Complexity, 10)]),
            roots: vec![file],
        };
        let text = render(&tree);
        assert!(!text.contains("lines"));
    }

    #[test]
    fn render_notes_when_no_measures_are_configured() {
        assert_eq!(
            render(&AggregatedTree::default()),
            "violations: none configured\n"
        );
    }
}
