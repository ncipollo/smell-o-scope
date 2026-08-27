use std::path::PathBuf;

use crate::feature::scope::output::Format;

/// Flat, testable input to [`crate::feature::scope::run`] — a direct move of
/// the parsed CLI flags, with no defaults resolved yet.
#[derive(Debug, Default)]
pub struct Request {
    pub paths: Vec<PathBuf>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub branches: Vec<String>,
    pub implements: Vec<String>,
    pub max_complexity: Option<usize>,
    pub max_methods: Option<usize>,
    pub max_lines: Option<usize>,
    pub max_declarations: Option<usize>,
    pub rule: Option<String>,
    pub format: Format,
    pub output: Option<PathBuf>,
}
