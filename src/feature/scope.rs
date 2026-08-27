//! Orchestrates a single scan: resolve options, run `smell`, and render the
//! document ready for [`crate::cli::router`] to emit.

use std::env;
use std::io;
use std::path::Path;

use smell::{PathError, analyze_tree, check, resolve_options};

pub mod options;
pub mod output;
pub mod request;
pub mod scan;

use crate::feature::aggregate::{self, Mode};
use crate::feature::scope::output::Format;
use crate::render::{html, json};
pub use output::Destination;
pub use request::Request;
pub use scan::Scan;

/// The result of running a scan: the rendered document, where it should go,
/// non-fatal path errors to report, and how many files were analyzed.
pub struct Outcome {
    pub document: String,
    pub destination: Destination,
    pub errors: Vec<String>,
    pub analyzed: usize,
}

impl Outcome {
    pub fn analyzed_nothing(&self) -> bool {
        self.analyzed == 0
    }
}

/// Runs a scan, resolving `smell.toml` from the current working directory.
pub fn run(request: &Request) -> io::Result<Outcome> {
    run_in(&env::current_dir()?, request)
}

/// Runs a scan with `smell.toml` resolved from `config_dir` rather than the
/// process's working directory, so tests can point it at a fixture.
pub fn run_in(config_dir: &Path, request: &Request) -> io::Result<Outcome> {
    let overrides = options::overrides(request);
    let analysis_options = resolve_options(config_dir, &overrides)?;
    let tree = analyze_tree(&request.paths, &analysis_options);
    let reports = tree.reports();
    let results = check(&reports, &analysis_options);
    let mode = Mode::default();
    let aggregated = aggregate::aggregate(mode, &tree, &results);
    let scan = Scan {
        mode,
        settings: options::settings(&overrides, &analysis_options),
        tree: &aggregated,
        errors: &tree.errors,
    };
    Ok(Outcome {
        document: document(request.format, &scan),
        destination: output::destination(request.format, request.output.as_deref()),
        errors: error_messages(&tree.errors),
        analyzed: reports.len(),
    })
}

/// Renders the document `--format` selects: the raw JSON document, or that
/// same document wrapped in a self-contained HTML report.
fn document(format: Format, scan: &Scan) -> String {
    match format {
        Format::Json => json::render(scan),
        Format::Html => html::render(scan),
    }
}

fn error_messages(errors: &[PathError]) -> Vec<String> {
    errors
        .iter()
        .map(|error| format!("error: {}: {}", error.path.display(), error.error))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::testing::fixture_path;

    fn fixture_request() -> Request {
        Request {
            paths: vec![fixture_path("tree")],
            ..Request::default()
        }
    }

    #[test]
    fn run_renders_the_fixture_tree() {
        let outcome = run_in(&fixture_path("tree"), &fixture_request()).expect("resolves");
        assert_eq!(outcome.analyzed, 2);
        assert!(outcome.document.contains("simple.rs"));
        assert!(outcome.document.contains("nested"));
        assert!(outcome.document.contains("types.rs"));
    }

    #[test]
    fn run_excludes_default_ignored_directories() {
        // Not `!contains("node_modules")`: the echoed `options.exclude` glob
        // legitimately mentions it. Check the excluded files themselves are
        // absent from the analyzed tree instead.
        let outcome = run_in(&fixture_path("tree"), &fixture_request()).expect("resolves");
        assert!(!outcome.document.contains("ignored.js"));
        assert!(!outcome.document.contains("ignored.rs"));
    }

    #[test]
    fn run_reports_missing_paths_and_still_renders() {
        let mut request = fixture_request();
        request.paths.push(fixture_path("does-not-exist"));
        let outcome = run_in(&fixture_path("tree"), &request).expect("resolves");
        assert_eq!(outcome.errors.len(), 1);
        assert!(outcome.document.contains("simple.rs"));
    }

    #[test]
    fn run_surfaces_check_failures_in_the_document() {
        let request = Request {
            max_complexity: Some(0),
            ..fixture_request()
        };
        let outcome = run_in(&fixture_path("tree"), &request).expect("resolves");
        assert!(outcome.document.contains("complexity"));
    }

    #[test]
    fn run_embeds_violation_counts() {
        let request = Request {
            max_complexity: Some(0),
            ..fixture_request()
        };
        let outcome = run_in(&fixture_path("tree"), &request).expect("resolves");
        assert!(outcome.document.contains(r#""violations":{"total""#));
    }

    #[test]
    fn run_defaults_destination_from_the_format() {
        let outcome = run_in(&fixture_path("tree"), &fixture_request()).expect("resolves");
        assert_eq!(
            outcome.destination,
            Destination::File(PathBuf::from(output::DEFAULT_HTML_OUTPUT))
        );
    }

    #[test]
    fn run_fails_when_the_rule_is_unknown() {
        let request = Request {
            rule: Some("nope".to_string()),
            ..fixture_request()
        };
        assert!(run_in(&fixture_path("tree"), &request).is_err());
    }

    #[test]
    fn html_format_renders_a_self_contained_document() {
        let outcome = run_in(&fixture_path("tree"), &fixture_request()).expect("resolves");
        assert!(outcome.document.starts_with("<!doctype html>"));
        assert!(outcome.document.contains(r#"id="smell-data""#));
        assert!(!outcome.document.contains("http://"));
        assert!(!outcome.document.contains("https://"));
    }

    #[test]
    fn json_format_produces_a_parsable_document() {
        let request = Request {
            format: crate::feature::scope::output::Format::Json,
            ..fixture_request()
        };
        let outcome = run_in(&fixture_path("tree"), &request).expect("resolves");
        let document: serde_json::Value =
            serde_json::from_str(&outcome.document).expect("valid JSON");
        assert_eq!(document["version"], 1);
    }

    #[test]
    fn json_format_lists_only_configured_measures() {
        let request = Request {
            format: crate::feature::scope::output::Format::Json,
            max_complexity: Some(1),
            ..fixture_request()
        };
        let outcome = run_in(&fixture_path("tree"), &request).expect("resolves");
        let document: serde_json::Value =
            serde_json::from_str(&outcome.document).expect("valid JSON");
        assert_eq!(document["measures"], serde_json::json!(["complexity"]));
    }

    #[test]
    fn json_format_reports_unreadable_paths_in_errors() {
        let mut request = Request {
            format: crate::feature::scope::output::Format::Json,
            ..fixture_request()
        };
        request.paths.push(fixture_path("does-not-exist"));
        let outcome = run_in(&fixture_path("tree"), &request).expect("resolves");
        let document: serde_json::Value =
            serde_json::from_str(&outcome.document).expect("valid JSON");
        assert_eq!(document["errors"].as_array().expect("array").len(), 1);
    }

    #[test]
    fn json_document_matches_the_snapshot() {
        let request = Request {
            format: crate::feature::scope::output::Format::Json,
            max_complexity: Some(1),
            max_methods: Some(1),
            max_lines: Some(5),
            max_declarations: Some(1),
            ..fixture_request()
        };
        let root = fixture_path("tree");
        let outcome = run_in(&root, &request).expect("resolves");
        let normalized = crate::testing::normalize_document(&outcome.document, &root);
        insta::assert_snapshot!(normalized);
    }
}
