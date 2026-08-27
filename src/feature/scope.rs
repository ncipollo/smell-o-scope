//! Orchestrates a single scan: resolve options, run `smell`, and build the
//! placeholder document ready for [`crate::cli::router`] to emit.

use std::env;
use std::io;
use std::path::Path;

use smell::{PathError, analyze_tree, check, resolve_options};

pub mod options;
pub mod output;
pub mod request;

use crate::feature::aggregate::{self, Mode};
use crate::render::debug;
pub use output::Destination;
pub use request::Request;

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
    let aggregated = aggregate::aggregate(Mode::default(), &tree, &results);
    Ok(Outcome {
        document: debug::render(&tree, &results, &aggregated),
        destination: output::destination(request.format, request.output.as_deref()),
        errors: error_messages(&tree.errors),
        analyzed: reports.len(),
    })
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
        let outcome = run_in(&fixture_path("tree"), &fixture_request()).expect("resolves");
        assert!(!outcome.document.contains("node_modules"));
        assert!(!outcome.document.contains(".hidden"));
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
    fn run_includes_a_violations_section() {
        let request = Request {
            max_complexity: Some(0),
            ..fixture_request()
        };
        let outcome = run_in(&fixture_path("tree"), &request).expect("resolves");
        assert!(outcome.document.contains("violations:"));
    }

    #[test]
    fn run_preserves_the_existing_debug_sections() {
        let request = Request {
            max_complexity: Some(0),
            ..fixture_request()
        };
        let outcome = run_in(&fixture_path("tree"), &request).expect("resolves");
        assert!(outcome.document.contains("simple.rs"));
        assert!(outcome.document.contains("checks:"));
        assert!(outcome.document.contains("violations:"));
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
}
